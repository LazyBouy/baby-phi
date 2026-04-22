//! `add_secret` business logic — registers a new vault entry.
//!
//! Flow (plan §P4 + §P4.5):
//! 1. Validate slug.
//! 2. Check slug uniqueness via [`Repository::get_secret_by_slug`].
//! 3. Seal plaintext with the master key.
//! 4. Mint a Template E Auth Request (self-approved).
//! 5. Persist in order: AR → SecretCredential/SealedBlob →
//!    catalogue seed → per-instance `[read]` grant on
//!    `secret:<slug>` with `fundamentals = [SecretCredential]`.
//! 6. Emit `vault.secret.added` (Alerted).
//!
//! Non-atomic sequential persistence is a **known limitation** for M2:
//! there is no "AR + entity + catalogue + grant + audit" batch API on
//! the Repository trait. A failure between `put_secret` and the
//! audit-emit leaves a half-written vault row with no corresponding
//! AR / grant / audit event. Callable impact is negligible in M2
//! because:
//!
//! - the platform has exactly one admin (no concurrent writers), so the
//!   only way a half-write lands is a process crash between steps —
//!   the operator can re-run the command after verifying storage;
//! - the slug uniqueness index catches the retry's duplicate slug + any
//!   orphan row from a prior attempt will be visible via
//!   `phi secret list` as a row with no matching audit event;
//! - **no automated sweep ships in M2** — closing the TOCTOU window
//!   is tracked as M3 work (archived plan §Part 11 Q8 / D6 note); the
//!   fix is an atomic `apply_secret_add` repository method mirroring
//!   M1's `apply_bootstrap_claim` pattern.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::secrets as secret_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::{AgentId, GrantId, SecretId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::{Fundamental, SecretCredential, SecretRef};
use domain::repository::{Repository, SealedBlob};
use domain::templates::e::{build_auto_approved_request, BuildArgs};
use store::crypto::{seal, MasterKey};

use super::{secret_uri, validate_slug, AddOutcome, SecretError, KIND_TAG};

/// Inputs the HTTP handler hands over after validating the JSON body.
pub struct AddInput<'a> {
    /// Human-readable slug (e.g. `anthropic-api-key`). Stable across
    /// rotations.
    pub slug: &'a str,
    /// Raw plaintext material. The function seals it and drops the
    /// reference before returning — never logged, never stored in
    /// plaintext.
    pub plaintext: &'a [u8],
    /// Mask the value in list views + audit diffs. Defaults to
    /// `true` on the wire; the handler enforces a true default.
    pub sensitive: bool,
    /// The admin's agent id — captured off the `AuthenticatedSession`
    /// extractor.
    pub actor: AgentId,
    /// Wall clock. Injected so tests can pin timestamps.
    pub now: DateTime<Utc>,
}

/// Add a new vault entry. Returns the persisted catalogue row + the
/// AR id + the audit event id; plaintext bytes are discarded.
pub async fn add_secret(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    master_key: &MasterKey,
    input: AddInput<'_>,
) -> Result<AddOutcome, SecretError> {
    validate_slug(input.slug)?;
    let slug = SecretRef::new(input.slug);

    // Uniqueness check. Race-free enough for M2 (one admin); M3's atomic
    // API closes the TOCTOU window.
    if repo.get_secret_by_slug(&slug).await?.is_some() {
        return Err(SecretError::SlugInUse(input.slug.to_string()));
    }

    // Seal plaintext — phi-core-level crypto, unchanged since M1.
    let sealed = seal(master_key, input.plaintext)?;
    let (ct_b64, nonce_b64) = sealed.to_base64();
    let sealed_blob = SealedBlob {
        ciphertext_b64: ct_b64,
        nonce_b64,
    };

    // Template E AR — self-approved platform-admin write on the
    // secret's URI.
    let uri = secret_uri(input.slug);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri: uri.clone() },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec![uri.clone()],
        justification: Some(format!(
            "self-approved platform-admin write: add secret `{}`",
            input.slug
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;

    // Persist AR, then the vault row, then the catalogue seed.
    repo.create_auth_request(&ar).await?;

    let credential = SecretCredential {
        id: SecretId::new(),
        slug: slug.clone(),
        custodian: input.actor,
        last_rotated_at: None,
        sensitive: input.sensitive,
        created_at: input.now,
    };
    repo.put_secret(&credential, &sealed_blob).await?;

    // `secret_credential` is an ontology-tagged fundamental bundle
    // rather than a `Composite` enum variant (per M2 plan §1.5 G1).
    // Seed via the raw catalogue method with an explicit kind string —
    // the same string Permission Check Step 0 will look up.
    repo.seed_catalogue_entry(None, &uri, "secret_credential")
        .await?;

    // Issue the per-instance [read] grant on `secret:<slug>` (M2/P4.5 —
    // G19 / D17 / C21). The grant carries explicit
    // `fundamentals = [SecretCredential]` so the engine's
    // `resolve_grant` Case D picks it up with a selector scoped to
    // this exact URI — not a class-wide wildcard. This unlocks
    // per-secret revocation + M3's delegated-custody handoff
    // without a data migration.
    let grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(input.actor),
        action: vec!["read".to_string()],
        resource: ResourceRef { uri: uri.clone() },
        fundamentals: vec![Fundamental::SecretCredential],
        descends_from: Some(auth_request_id),
        delegable: true,
        issued_at: input.now,
        revoked_at: None,
    };
    repo.create_grant(&grant).await?;

    // Audit. The emitter fills `prev_event_hash` against the
    // platform-scope chain.
    let event =
        secret_events::secret_added(input.actor, &credential, Some(auth_request_id), input.now);
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SecretError::AuditEmit(e.to_string()))?;

    Ok(AddOutcome {
        credential,
        auth_request_id,
        audit_event_id,
    })
}
