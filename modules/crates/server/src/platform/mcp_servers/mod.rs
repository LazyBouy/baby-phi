// `McpError`-style error enum carries both strings and an ApiError —
// the `api_error` path is allocation-heavy but only lives on the cold
// denial path. Matches the vault + model-providers modules' allow.
#![allow(clippy::result_large_err)]

//! Page 03 — MCP (external services) business logic.
//!
//! phi-core leverage (mandatory per `CLAUDE.md`):
//! - The live [`phi_core::mcp::client::McpClient`] is **constructed on
//!   demand** from an [`ExternalService`] record at probe/invocation
//!   time; never stored. Health probing (shape only in M2; real wiring
//!   in M7b) calls `McpClient::connect_stdio` / `connect_http` followed
//!   by `list_tools()`.
//! - phi-core has no health-probe abstraction (§1.5 🚫); the tiny
//!   timeout+retry wrapper in [`health_probe`] is the only
//!   baby-phi-native MCP code.
//! - The `ExternalService` composite itself is baby-phi-only (phi-core
//!   has no "persisted MCP binding" container — it only ships the
//!   client).
//!
//! Cascade flow (plan §P6 + D7):
//! - PATCHing `tenants_allowed` to a smaller set calls
//!   [`Repository::narrow_mcp_tenants`] which runs the grant-revocation
//!   sweep inside the same SurrealQL transaction. The handler then
//!   emits one [`platform.mcp_server.tenant_access_revoked`] summary
//!   event + one [`auth_request.revoked`] per affected AR.

pub mod archive;
pub mod health_probe;
pub mod list;
pub mod patch_tenants;
pub mod register;

use domain::model::ids::{AuditEventId, AuthRequestId, McpServerId};
use domain::model::ExternalService;
use domain::repository::TenantRevocation;

/// Stable resource-uri scheme for MCP-server rows — matches the
/// catalogue seed done at [`register::register_mcp_server`] time and
/// the `target_uri` used at invocation time (M5+).
pub fn mcp_server_uri(id: McpServerId) -> String {
    format!("external_service:{id}")
}

/// Tag placed on every MCP-server record so Permission Check Step 3
/// can match `#kind:` filters on manifests targeting the MCP surface.
pub const KIND_TAG: &str = "#kind:external_service";

/// Outcome returned by [`register::register_mcp_server`].
#[derive(Debug, Clone)]
pub struct RegisterOutcome {
    pub service: ExternalService,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`patch_tenants::patch_mcp_tenants`].
///
/// When the PATCH did NOT shrink the set, `cascade` is empty and only
/// the server row was updated — `audit_event_id` is `None` (no cascade
/// event to emit; the operator-visible effect is just the new set).
///
/// When the PATCH shrank the set, `cascade` holds the
/// [`TenantRevocation`] entries the repository returned and
/// `audit_event_id` is the summary event's id (the per-AR events are
/// emitted as side effects — their ids are not surfaced back to the
/// HTTP response).
#[derive(Debug, Clone)]
pub struct PatchTenantsOutcome {
    pub mcp_server_id: McpServerId,
    pub cascade: Vec<TenantRevocation>,
    pub audit_event_id: Option<AuditEventId>,
}

/// Outcome returned by [`archive::archive_mcp_server`].
#[derive(Debug, Clone)]
pub struct ArchiveOutcome {
    pub mcp_server_id: McpServerId,
    pub audit_event_id: AuditEventId,
}

/// Business-logic error vocabulary. Matches model-providers + vault so
/// `error_to_api_error` translations stay symmetric across M2 page
/// handlers.
#[derive(Debug)]
pub enum McpError {
    /// Input failed shape validation (empty name, malformed endpoint,
    /// etc.).
    Validation(String),
    /// The supplied `secret_ref` does not exist in the vault.
    SecretRefNotFound(String),
    /// No MCP server matches the supplied id.
    NotFound(McpServerId),
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::Validation(m) => write!(f, "validation failed: {m}"),
            McpError::SecretRefNotFound(s) => write!(f, "secret_ref `{s}` not found in vault"),
            McpError::NotFound(id) => write!(f, "no MCP server with id `{id}`"),
            McpError::Repository(m) => write!(f, "repository: {m}"),
            McpError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for McpError {}

impl From<domain::repository::RepositoryError> for McpError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        McpError::Repository(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::ids::McpServerId;

    #[test]
    fn mcp_server_uri_has_stable_prefix() {
        let id = McpServerId::new();
        assert_eq!(mcp_server_uri(id), format!("external_service:{id}"));
    }

    #[test]
    fn kind_tag_matches_composite() {
        // The tag MUST match Composite::ExternalServiceObject.kind_tag()
        // because Permission-Check Step 3 uses the tag to resolve
        // `#kind:` manifest filters.
        assert_eq!(
            KIND_TAG,
            domain::model::Composite::ExternalServiceObject.kind_tag()
        );
    }
}
