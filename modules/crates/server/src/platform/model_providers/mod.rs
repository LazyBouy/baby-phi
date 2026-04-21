// `SecretError`-style error enum carries both strings and an ApiError —
// the `api_error` field is allocation-heavy but only lives on the cold
// denial path. Matches the vault module's allow.
#![allow(clippy::result_large_err)]

//! Page 02 — Model Providers business logic.
//!
//! phi-core leverage (mandatory per `CLAUDE.md`):
//! - The persisted `ModelRuntime.config` field IS
//!   [`phi_core::provider::model::ModelConfig`] directly — no parallel
//!   baby-phi shape. The wire POST body deserialises into `ModelConfig`;
//!   the server only adds platform-governance fields (`secret_ref`,
//!   `tenants_allowed`, `status`).
//! - Provider-kind enumeration (`/api/v0/platform/provider-kinds`)
//!   returns [`phi_core::provider::registry::ProviderRegistry::default().protocols()`] —
//!   phi-core owns the single source of truth for which API protocols
//!   exist. Baby-phi never hard-codes its own list.
//! - Health probing — deferred to M7b per plan §G5. M2 ships the
//!   `platform.model_provider.health_degraded` event shape only.

pub mod archive;
pub mod list;
pub mod provider_kinds;
pub mod register;

use domain::model::ids::{AuditEventId, AuthRequestId, ModelProviderId};
use domain::model::ModelRuntime;

/// Stable resource-uri scheme for provider-registration rows — matches
/// the catalogue seed done at [`register::register_provider`] time and
/// the `target_uri` used at invocation time (M5+).
pub fn provider_uri(id: ModelProviderId) -> String {
    format!("provider:{id}")
}

/// Tag placed on every model-runtime record so Permission Check
/// Step 3 can match `#kind:` filters on manifests targeting the
/// provider registry.
pub const KIND_TAG: &str = "#kind:model_runtime";

/// Outcome returned by [`register::register_provider`].
#[derive(Debug, Clone)]
pub struct RegisterOutcome {
    pub runtime: ModelRuntime,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`archive::archive_provider`].
#[derive(Debug, Clone)]
pub struct ArchiveOutcome {
    pub provider_id: ModelProviderId,
    pub audit_event_id: AuditEventId,
}

/// Business-logic error vocabulary. Matches the vault module's
/// conventions so `error_to_api_error` translations stay symmetric
/// across M2 page handlers.
#[derive(Debug)]
pub enum ProviderError {
    /// Input failed shape validation (missing required field,
    /// malformed secret_ref, etc.).
    Validation(String),
    /// A provider with the same `(provider, config.id)` pair is
    /// already registered.
    DuplicateProvider { provider: String, model_id: String },
    /// The supplied `secret_ref` does not exist in the vault.
    SecretRefNotFound(String),
    /// No provider matches the supplied id.
    NotFound(ModelProviderId),
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Validation(m) => write!(f, "validation failed: {m}"),
            ProviderError::DuplicateProvider { provider, model_id } => {
                write!(
                    f,
                    "provider `{provider}` + model `{model_id}` already registered"
                )
            }
            ProviderError::SecretRefNotFound(s) => {
                write!(f, "secret_ref `{s}` not found in vault")
            }
            ProviderError::NotFound(id) => write!(f, "no model provider with id `{id}`"),
            ProviderError::Repository(m) => write!(f, "repository: {m}"),
            ProviderError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<domain::repository::RepositoryError> for ProviderError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        ProviderError::Repository(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::ids::ModelProviderId;

    #[test]
    fn provider_uri_has_stable_prefix() {
        let id = ModelProviderId::new();
        assert_eq!(provider_uri(id), format!("provider:{id}"));
    }
}
