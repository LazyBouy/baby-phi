//! `reveal_secret` — unseal a vault entry and return the plaintext.
//!
//! Of every M2 operation this one is the strictest: the **engine itself**
//! decides whether the caller may see plaintext (decision D11). The
//! handler is never allowed to short-circuit the check.
//!
//! Flow:
//! 1. Load the catalogue row + sealed blob.
//! 2. Snapshot the caller's agent-scope grants + the single catalogue URI
//!    (`secret:<slug>`) into the engine-facing [`CheckContext`].
//! 3. Build a [`Manifest`] with `constraints = ["purpose"]` and
//!    `constraint_requirements = {"purpose": "reveal"}`. Callers that
//!    omit the constraint context fail at [`FailedStep::Constraint`].
//! 4. Invoke [`domain::permissions::check`] directly (bypasses
//!    [`handler_support::check_permission`] so we can branch on the
//!    decision to emit the denied-attempt audit event).
//! 5. On `Allowed`: unseal → emit `vault.secret.revealed` (**before**
//!    returning bytes) → return plaintext.
//! 6. On `Denied`/`Pending`: emit `vault.secret.reveal_attempt_denied`
//!    then convert the engine's decision to an [`ApiError`] via the
//!    shared [`denial_to_api_error`] mapping.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::secrets as secret_events;
use domain::audit::AuditEmitter;
use domain::model::ids::AgentId;
use domain::model::nodes::PrincipalRef;
use domain::model::SecretRef;
use domain::permissions::catalogue::StaticCatalogue;
use domain::permissions::check;
use domain::permissions::decision::{Decision, DeniedReason, FailedStep};
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::repository::Repository;
use store::crypto::{open, MasterKey, SealedSecret};

use crate::handler_support::permission::denial_to_api_error;

use super::{secret_uri, validate_slug, RevealOutcome, SecretError, KIND_TAG};

pub struct RevealInput<'a> {
    pub slug: &'a str,
    /// Free-form justification; surfaced into the audit diff under
    /// `after.purpose`. Must be non-empty — the constraint-context
    /// value is separately asserted against `"reveal"`.
    pub justification: &'a str,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn reveal_secret(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    master_key: &MasterKey,
    input: RevealInput<'_>,
) -> Result<RevealOutcome, SecretError> {
    validate_slug(input.slug)?;
    if input.justification.trim().is_empty() {
        return Err(SecretError::Validation(
            "reveal requires a non-empty justification".into(),
        ));
    }
    let slug = SecretRef::new(input.slug);
    let uri = secret_uri(input.slug);

    let (credential, sealed_blob) = repo
        .get_secret_by_slug(&slug)
        .await?
        .ok_or_else(|| SecretError::NotFound(input.slug.to_string()))?;

    // Snapshot catalogue for the one URI the engine needs.
    let mut catalogue = StaticCatalogue::empty();
    if repo.catalogue_contains(None, &uri).await? {
        catalogue.seed(None, &uri);
    }

    let agent_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(input.actor))
        .await?;
    let consents = ConsentIndex::empty();
    let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

    let (manifest, call) = build_reveal_manifest_and_call(&uri);

    let ctx = CheckContext {
        agent: input.actor,
        current_org: None,
        current_project: None,
        agent_grants: &agent_grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &template_gated,
        call,
    };

    let decision = check(&ctx, &manifest, &NoopMetrics);

    match decision {
        Decision::Allowed { .. } => {
            let sealed =
                SealedSecret::from_base64(&sealed_blob.ciphertext_b64, &sealed_blob.nonce_b64)?;
            let plaintext = open(master_key, &sealed)?;

            // Emit BEFORE returning so a crash between emit + return
            // still leaves an audit trail (plan §P4 step 3).
            let event = secret_events::secret_revealed(
                input.actor,
                &credential,
                input.justification,
                None,
                input.now,
            );
            let audit_event_id = event.event_id;
            audit
                .emit(event)
                .await
                .map_err(|e| SecretError::AuditEmit(e.to_string()))?;

            Ok(RevealOutcome {
                secret_id: credential.id,
                slug: input.slug.to_string(),
                plaintext,
                audit_event_id,
            })
        }
        Decision::Denied {
            failed_step,
            reason,
        } => {
            let reason_text = denial_reason_text(&reason);
            let step_label = failed_step_label(failed_step);

            let denied_event = secret_events::secret_reveal_attempt_denied(
                input.actor,
                credential.id,
                &credential.slug,
                step_label,
                &reason_text,
                input.now,
            );
            audit
                .emit(denied_event)
                .await
                .map_err(|e| SecretError::AuditEmit(e.to_string()))?;

            let api_error = denial_to_api_error(failed_step, &reason);
            Err(SecretError::RevealDenied {
                secret_id: credential.id,
                slug: input.slug.to_string(),
                failed_step: step_label.to_string(),
                reason: reason_text,
                api_error,
            })
        }
        Decision::Pending { .. } => Err(SecretError::RevealPending),
    }
}

/// Build the engine-facing `Manifest` + `ToolCall` pair that the
/// reveal path passes to [`domain::permissions::check`].
///
/// Extracted as a helper so the D11 contract (decision D11 in the
/// archived M2 plan: "reveal is a Permission-Check invocation with
/// `purpose=reveal`") can be pinned by unit tests rather than only
/// verified indirectly through the HTTP acceptance layer.
///
/// The manifest + call together make three assertions the engine
/// relies on:
///
/// 1. Action `read` on the `secret_credential` fundamental class.
/// 2. `constraints = ["purpose"]` + `constraint_requirements.purpose
///    = "reveal"` — Step 4 denies on any value-mismatch.
/// 3. Target URI `secret:<slug>` carrying the `#kind:secret_credential`
///    tag — matches the per-instance grant issued at add time.
fn build_reveal_manifest_and_call(uri: &str) -> (Manifest, ToolCall) {
    let manifest = Manifest {
        actions: vec!["read".to_string()],
        resource: vec!["secret_credential".to_string()],
        transitive: vec![],
        constraints: vec!["purpose".to_string()],
        constraint_requirements: std::iter::once((
            "purpose".to_string(),
            serde_json::Value::String("reveal".to_string()),
        ))
        .collect(),
        kinds: vec![KIND_TAG.to_string()],
    };
    let call = ToolCall {
        target_uri: uri.to_string(),
        target_tags: vec![KIND_TAG.to_string(), uri.to_string()],
        target_agent: None,
        constraint_context: std::iter::once((
            "purpose".to_string(),
            serde_json::Value::String("reveal".to_string()),
        ))
        .collect(),
    };
    (manifest, call)
}

/// Stable string labels for [`FailedStep`] variants — mirrors the
/// `failed_step` value recorded in the audit event.
fn failed_step_label(step: FailedStep) -> &'static str {
    match step {
        FailedStep::Catalogue => "Catalogue",
        FailedStep::Expansion => "Expansion",
        FailedStep::Resolution => "Resolution",
        FailedStep::Ceiling => "Ceiling",
        FailedStep::Match => "Match",
        FailedStep::Constraint => "Constraint",
        FailedStep::Scope => "Scope",
        FailedStep::Consent => "Consent",
    }
}

fn denial_reason_text(reason: &DeniedReason) -> String {
    match reason {
        DeniedReason::CatalogueMiss { resource_uri } => {
            format!("resource `{resource_uri}` not in catalogue")
        }
        DeniedReason::ManifestEmpty => "manifest declared no resources or actions".to_string(),
        DeniedReason::NoGrantsHeld => "no grants held".to_string(),
        DeniedReason::CeilingEmptied => "every candidate clamped by ceiling".to_string(),
        DeniedReason::NoMatchingGrant {
            fundamental,
            action,
        } => format!("no grant covers `{fundamental:?}` for `{action}`"),
        DeniedReason::ConstraintViolation { constraint, .. } => {
            format!("constraint `{constraint}` not satisfied")
        }
        DeniedReason::ScopeUnresolvable {
            fundamental,
            action,
        } => format!("scope cascade failed for `{fundamental:?}`/`{action}`"),
    }
}

#[cfg(test)]
mod tests {
    //! D11 contract pin (plan decision D11 + M2/P4 verification): the
    //! reveal path is a Permission-Check invocation, not a handler
    //! bypass, and the manifest it constructs enforces
    //! `purpose=reveal` at the engine's Step 4.
    //!
    //! Three things worth nailing down at the unit level:
    //!   1. `build_reveal_manifest_and_call` produces the exact shape
    //!      the engine needs (constraint keys + required value + the
    //!      catalogued `secret:<slug>` target URI).
    //!   2. When the grant + catalogue are in place and the call's
    //!      constraint context matches the requirement, the engine
    //!      returns `Allowed`.
    //!   3. If a hypothetical caller bypassed the handler and
    //!      constructed a call whose `purpose` value didn't match
    //!      `"reveal"`, the engine would deny at `FailedStep::Constraint`
    //!      — i.e. the engine, not the handler, is the enforcement
    //!      authority.
    //!
    //! This complements the domain-level
    //! `step_4_constraint_value_match_props` proptest (generic over
    //! arbitrary constraints) by pinning the *specific* shape the
    //! credentials-vault reveal handler emits.

    use super::*;
    use chrono::Utc;
    use domain::model::ids::{AgentId, GrantId};
    use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
    use domain::model::Fundamental;
    use domain::permissions::{
        catalogue::StaticCatalogue,
        check,
        manifest::{CheckContext, ConsentIndex},
        metrics::NoopMetrics,
    };
    use std::collections::HashSet;

    /// Shape-pin: the helper wires the D11 contract fields correctly.
    #[test]
    fn manifest_declares_purpose_reveal_requirement() {
        let (manifest, call) = build_reveal_manifest_and_call("secret:anthropic-api-key");
        assert_eq!(manifest.actions, vec!["read".to_string()]);
        assert_eq!(manifest.resource, vec!["secret_credential".to_string()]);
        assert_eq!(manifest.constraints, vec!["purpose".to_string()]);
        assert_eq!(
            manifest.constraint_requirements.get("purpose"),
            Some(&serde_json::Value::String("reveal".to_string())),
            "manifest must REQUIRE purpose=reveal"
        );
        assert!(manifest.kinds.iter().any(|k| k == KIND_TAG));

        assert_eq!(call.target_uri, "secret:anthropic-api-key");
        assert_eq!(
            call.constraint_context.get("purpose"),
            Some(&serde_json::Value::String("reveal".to_string())),
            "call must ASSERT purpose=reveal"
        );
        assert!(call.target_tags.iter().any(|t| t == KIND_TAG));
    }

    /// Happy-path engine run: manifest + call + matching instance-URI
    /// grant + catalogue → Allowed.
    #[test]
    fn engine_allows_reveal_when_purpose_matches() {
        let agent = AgentId::new();
        let uri = "secret:anthropic-api-key".to_string();

        // The per-instance grant that `add_secret` issues (P4.5 shape).
        let grants = [Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec!["read".to_string()],
            resource: ResourceRef { uri: uri.clone() },
            fundamentals: vec![Fundamental::SecretCredential],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
        }];
        let mut catalogue = StaticCatalogue::empty();
        catalogue.seed(None, &uri);
        let consents = ConsentIndex::empty();
        let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

        let (manifest, call) = build_reveal_manifest_and_call(&uri);
        let ctx = CheckContext {
            agent,
            current_org: None,
            current_project: None,
            agent_grants: &grants,
            project_grants: &[],
            org_grants: &[],
            ceiling_grants: &[],
            catalogue: &catalogue,
            consents: &consents,
            template_gated_auth_requests: &template_gated,
            call,
        };
        let d = check(&ctx, &manifest, &NoopMetrics);
        assert!(d.is_allowed(), "happy path must resolve; got {d:?}");
    }

    /// Contract pin: if a caller's constraint context carried a
    /// different purpose value, the engine would deny at Step 4 —
    /// proving the enforcement lives in the engine, not the handler.
    #[test]
    fn engine_denies_reveal_when_purpose_value_mismatches() {
        let agent = AgentId::new();
        let uri = "secret:anthropic-api-key".to_string();
        let grants = [Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec!["read".to_string()],
            resource: ResourceRef { uri: uri.clone() },
            fundamentals: vec![Fundamental::SecretCredential],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
        }];
        let mut catalogue = StaticCatalogue::empty();
        catalogue.seed(None, &uri);
        let consents = ConsentIndex::empty();
        let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

        // Start from the real helper output, then mutate the call's
        // constraint_context value — this is the smallest diff that
        // exercises the engine's Step 4 value-match guard.
        let (manifest, mut call) = build_reveal_manifest_and_call(&uri);
        call.constraint_context.insert(
            "purpose".to_string(),
            serde_json::Value::String("not-reveal".to_string()),
        );
        let ctx = CheckContext {
            agent,
            current_org: None,
            current_project: None,
            agent_grants: &grants,
            project_grants: &[],
            org_grants: &[],
            ceiling_grants: &[],
            catalogue: &catalogue,
            consents: &consents,
            template_gated_auth_requests: &template_gated,
            call,
        };
        let d = check(&ctx, &manifest, &NoopMetrics);
        assert_eq!(
            d.failed_step(),
            Some(FailedStep::Constraint),
            "value-mismatch must deny at Step 4 Constraint; got {d:?}"
        );
    }

    /// Corollary: missing constraint context value also denies at
    /// Step 4 (manifest requires `purpose`; call omitted it).
    #[test]
    fn engine_denies_reveal_when_purpose_missing() {
        let agent = AgentId::new();
        let uri = "secret:anthropic-api-key".to_string();
        let grants = [Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec!["read".to_string()],
            resource: ResourceRef { uri: uri.clone() },
            fundamentals: vec![Fundamental::SecretCredential],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
        }];
        let mut catalogue = StaticCatalogue::empty();
        catalogue.seed(None, &uri);
        let consents = ConsentIndex::empty();
        let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

        let (manifest, mut call) = build_reveal_manifest_and_call(&uri);
        call.constraint_context.remove("purpose");
        let ctx = CheckContext {
            agent,
            current_org: None,
            current_project: None,
            agent_grants: &grants,
            project_grants: &[],
            org_grants: &[],
            ceiling_grants: &[],
            catalogue: &catalogue,
            consents: &consents,
            template_gated_auth_requests: &template_gated,
            call,
        };
        let d = check(&ctx, &manifest, &NoopMetrics);
        assert_eq!(
            d.failed_step(),
            Some(FailedStep::Constraint),
            "missing constraint must deny at Step 4 Constraint; got {d:?}"
        );
    }
}
