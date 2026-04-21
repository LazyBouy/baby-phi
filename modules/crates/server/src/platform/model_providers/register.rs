//! `register_provider` — bind a new LLM runtime.
//!
//! Flow (plan §P5 + §P4.5 grant shape):
//! 1. Validate input — secret_ref slug shape, `ModelConfig` non-empty.
//! 2. Verify `secret_ref` exists in the vault (referential integrity).
//! 3. Check duplicate `(provider, config.id)` pair.
//! 4. Scrub `config.api_key` → empty sentinel (vault holds the real key).
//! 5. Mint a Template E Auth Request.
//! 6. Persist in order: AR → model-runtime row → catalogue seed
//!    (`provider:<id>`) → per-instance `[invoke]` grant with
//!    `fundamentals = [NetworkEndpoint, SecretCredential,
//!    EconomicResource, Tag]` (matches `Composite::ModelRuntimeObject`).
//! 7. Emit `platform.model_provider.registered` (Alerted).
//!
//! phi-core leverage: the persisted `config` field is a verbatim
//! [`phi_core::provider::model::ModelConfig`]. The wire body has the
//! same shape — serde round-trips cleanly. Baby-phi never transcribes
//! individual fields.
//!
//! Sequential-write caveat: same as `platform/secrets/add.rs` — M2
//! accepts the TOCTOU window between steps; M3 introduces an atomic
//! batch API (see archived plan §Part 11 Q8).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::providers as provider_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::{AgentId, GrantId, ModelProviderId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::{Composite, Fundamental, ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{provider_uri, ProviderError, RegisterOutcome, KIND_TAG};

/// Inputs the HTTP handler hands over after decoding the JSON body.
///
/// `config` is the phi-core struct, accepted verbatim. `api_key` is
/// overwritten server-side before persistence — the plaintext never
/// hits the DB.
pub struct RegisterInput {
    pub config: phi_core::provider::model::ModelConfig,
    pub secret_ref: SecretRef,
    pub tenants_allowed: TenantSet,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn register_provider(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: RegisterInput,
) -> Result<RegisterOutcome, ProviderError> {
    // 1. Validate — reuse the vault's slug validator; same grammar.
    crate::platform::secrets::validate_slug(input.secret_ref.as_str())
        .map_err(|e| ProviderError::Validation(format!("secret_ref: {e}")))?;
    if input.config.id.trim().is_empty() {
        return Err(ProviderError::Validation(
            "config.id must be non-empty".into(),
        ));
    }
    if input.config.name.trim().is_empty() {
        return Err(ProviderError::Validation(
            "config.name must be non-empty".into(),
        ));
    }
    if input.config.provider.trim().is_empty() {
        return Err(ProviderError::Validation(
            "config.provider must be non-empty".into(),
        ));
    }
    if input.config.base_url.trim().is_empty() {
        return Err(ProviderError::Validation(
            "config.base_url must be non-empty".into(),
        ));
    }

    // 2. Referential integrity on secret_ref — the vault entry must
    //    exist BEFORE the runtime points at it.
    if repo.get_secret_by_slug(&input.secret_ref).await?.is_none() {
        return Err(ProviderError::SecretRefNotFound(
            input.secret_ref.as_str().to_string(),
        ));
    }

    // 3. Duplicate check — list existing (cheap at M2 scale: ~tens).
    let existing = repo.list_model_providers(true).await?;
    if let Some(dup) = existing
        .iter()
        .find(|r| r.config.provider == input.config.provider && r.config.id == input.config.id)
    {
        return Err(ProviderError::DuplicateProvider {
            provider: dup.config.provider.clone(),
            model_id: dup.config.id.clone(),
        });
    }

    // 4. Scrub the plaintext api_key — the vault is the only place
    //    plaintext material lives. `api_key` at persist time is a
    //    sentinel; M5+ session launch splices the real key in from
    //    the vault.
    let mut config = input.config;
    config.api_key.clear();

    // 5. Template E AR — self-approved platform-admin write.
    let runtime_id = ModelProviderId::new();
    let uri = provider_uri(runtime_id);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri: uri.clone() },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec![uri.clone()],
        justification: Some(format!(
            "self-approved platform-admin write: register model provider `{}` (model `{}`)",
            config.provider, config.id
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;

    // 6. Persist sequentially.
    repo.create_auth_request(&ar).await?;

    let runtime = ModelRuntime {
        id: runtime_id,
        config,
        secret_ref: input.secret_ref.clone(),
        tenants_allowed: input.tenants_allowed,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: input.now,
    };
    repo.put_model_provider(&runtime).await?;

    // Catalogue seed — `Composite::ModelRuntimeObject` is a real
    // composite, so we use the typed wrapper (unlike the vault's
    // tagged-fundamental-bundle pattern).
    repo.seed_catalogue_entry_for_composite(None, &uri, Composite::ModelRuntimeObject)
        .await?;

    // Per-instance `[invoke]` grant on `provider:<id>` with the
    // ModelRuntimeObject's constituent fundamentals. The engine's
    // Case D (P4.5) picks this up with the URI-scoped selector.
    let grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(input.actor),
        action: vec!["invoke".to_string()],
        resource: ResourceRef { uri: uri.clone() },
        fundamentals: vec![
            Fundamental::NetworkEndpoint,
            Fundamental::SecretCredential,
            Fundamental::EconomicResource,
            Fundamental::Tag,
        ],
        descends_from: Some(auth_request_id),
        delegable: true,
        issued_at: input.now,
        revoked_at: None,
    };
    repo.create_grant(&grant).await?;

    // 7. Audit.
    let event = provider_events::model_provider_registered(
        input.actor,
        &runtime,
        Some(auth_request_id),
        input.now,
    );
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| ProviderError::AuditEmit(e.to_string()))?;

    Ok(RegisterOutcome {
        runtime,
        auth_request_id,
        audit_event_id,
    })
}
