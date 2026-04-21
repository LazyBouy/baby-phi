//! HTTP handlers for the MCP-servers surface (page 03).
//!
//! Four routes, all gated by `AuthenticatedSession`:
//!
//! | Method  | Path | Op |
//! |---|---|---|
//! | `GET`   | `/api/v0/platform/mcp-servers` | list registered MCP servers |
//! | `POST`  | `/api/v0/platform/mcp-servers` | register a new MCP server |
//! | `PATCH` | `/api/v0/platform/mcp-servers/{id}/tenants` | update `tenants_allowed` (cascades on narrow) |
//! | `POST`  | `/api/v0/platform/mcp-servers/{id}/archive` | soft-delete |
//!
//! Wire body for POST carries `display_name`, `kind`, `endpoint`,
//! `secret_ref` (optional), `tenants_allowed`. The `endpoint` string is
//! phi-core's transport argument verbatim (`stdio:///cmd args…` or
//! `http[s]://…`); handlers never reinterpret it.
//!
//! The PATCH route is a narrow slice — only `tenants_allowed` is
//! mutable in M2. Other fields (endpoint, secret_ref, display_name) are
//! immutable; an operator wanting to change them archives the row and
//! registers a new one.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use domain::model::ids::{AuditEventId, AuthRequestId, McpServerId};
use domain::model::{ExternalServiceKind, RuntimeStatus, SecretRef, TenantSet};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::mcp_servers::{
    archive::{archive_mcp_server, ArchiveInput},
    list::{list_mcp_servers, ListInput},
    patch_tenants::{patch_mcp_tenants, PatchTenantsInput},
    register::{register_mcp_server, RegisterInput},
    McpError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes — requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RegisterServerRequest {
    pub display_name: String,
    /// `mcp` / `open_api` / `webhook` / `other` — M2 only wires `mcp`.
    pub kind: ExternalServiceKind,
    /// Transport argument passed verbatim to phi-core's MCP client.
    /// `stdio:///command args…` or `http[s]://…`.
    pub endpoint: String,
    /// Optional vault slug — MCP services that require no auth may
    /// omit this.
    #[serde(default)]
    pub secret_ref: Option<String>,
    /// Which orgs may invoke this server. Defaults to `All`.
    #[serde(default = "default_tenants")]
    pub tenants_allowed: TenantSet,
}

#[derive(Debug, Deserialize)]
pub struct PatchTenantsRequest {
    pub tenants_allowed: TenantSet,
}

fn default_tenants() -> TenantSet {
    TenantSet::All
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub include_archived: bool,
}

// ---------------------------------------------------------------------------
// Wire shapes — responses
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ServerSummary {
    pub id: McpServerId,
    pub display_name: String,
    pub kind: ExternalServiceKind,
    pub endpoint: String,
    pub secret_ref: Option<String>,
    pub tenants_allowed: TenantSet,
    pub status: RuntimeStatus,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListServersResponse {
    pub servers: Vec<ServerSummary>,
}

#[derive(Debug, Serialize)]
pub struct RegisterServerResponse {
    pub mcp_server_id: McpServerId,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// One cascade entry surfaced back to the operator — enough to render a
/// "we revoked X grants across Y ARs for Z orgs" summary.
#[derive(Debug, Serialize)]
pub struct TenantRevocationSummary {
    pub org: String,
    pub auth_request: String,
    pub revoked_grants: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PatchTenantsResponse {
    pub mcp_server_id: McpServerId,
    /// List of cascade entries. Empty when the PATCH did not shrink
    /// the set (no-op or widen).
    pub cascade: Vec<TenantRevocationSummary>,
    /// The summary event id when a cascade ran; `None` otherwise.
    pub audit_event_id: Option<AuditEventId>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveServerResponse {
    pub mcp_server_id: McpServerId,
    pub audit_event_id: AuditEventId,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Query(params): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let outcome = list_mcp_servers(
        state.repo.clone(),
        ListInput {
            include_archived: params.include_archived,
        },
    )
    .await
    .map_err(error_to_api_error)?;
    let servers = outcome
        .servers
        .into_iter()
        .map(|s| ServerSummary {
            id: s.id,
            display_name: s.display_name,
            kind: s.kind,
            endpoint: s.endpoint,
            secret_ref: s.secret_ref.map(|r| r.as_str().to_string()),
            tenants_allowed: s.tenants_allowed,
            status: s.status,
            archived_at: s.archived_at,
            created_at: s.created_at,
        })
        .collect();
    Ok((StatusCode::OK, Json(ListServersResponse { servers })).into_response())
}

pub async fn register(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Json(body): Json<RegisterServerRequest>,
) -> Result<Response, ApiError> {
    let outcome = register_mcp_server(
        state.repo.clone(),
        state.audit.clone(),
        RegisterInput {
            display_name: body.display_name,
            kind: body.kind,
            endpoint: body.endpoint,
            secret_ref: body.secret_ref.map(SecretRef::new),
            tenants_allowed: body.tenants_allowed,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        mcp_server_id = %outcome.service.id,
        display_name = %outcome.service.display_name,
        endpoint = %outcome.service.endpoint,
        auth_request_id = %outcome.auth_request_id,
        audit_event_id = %outcome.audit_event_id,
        "mcp_server: registered",
    );

    Ok((
        StatusCode::CREATED,
        Json(RegisterServerResponse {
            mcp_server_id: outcome.service.id,
            auth_request_id: outcome.auth_request_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn patch_tenants(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(id): Path<String>,
    Json(body): Json<PatchTenantsRequest>,
) -> Result<Response, ApiError> {
    let mcp_server_id = parse_id(&id)?;

    let outcome = patch_mcp_tenants(
        state.repo.clone(),
        state.audit.clone(),
        PatchTenantsInput {
            mcp_server_id,
            new_allowed: body.tenants_allowed,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    let cascade: Vec<TenantRevocationSummary> = outcome
        .cascade
        .iter()
        .map(|r| TenantRevocationSummary {
            org: r.org.to_string(),
            auth_request: r.auth_request.to_string(),
            revoked_grants: r.revoked_grants.iter().map(|g| g.to_string()).collect(),
        })
        .collect();

    let cascade_grant_count: usize = outcome.cascade.iter().map(|r| r.revoked_grants.len()).sum();
    info!(
        mcp_server_id = %outcome.mcp_server_id,
        cascade_ar_count = outcome.cascade.len(),
        cascade_grant_count,
        audit_event_id = ?outcome.audit_event_id,
        "mcp_server: tenants patched",
    );

    Ok((
        StatusCode::OK,
        Json(PatchTenantsResponse {
            mcp_server_id: outcome.mcp_server_id,
            cascade,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn archive(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let mcp_server_id = parse_id(&id)?;

    let outcome = archive_mcp_server(
        state.repo.clone(),
        state.audit.clone(),
        ArchiveInput {
            mcp_server_id,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        mcp_server_id = %outcome.mcp_server_id,
        audit_event_id = %outcome.audit_event_id,
        "mcp_server: archived",
    );

    Ok((
        StatusCode::OK,
        Json(ArchiveServerResponse {
            mcp_server_id: outcome.mcp_server_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_id(id: &str) -> Result<McpServerId, ApiError> {
    Uuid::parse_str(id)
        .map(McpServerId::from_uuid)
        .map_err(|_| ApiError::validation_failed("mcp server id must be a UUID"))
}

fn error_to_api_error(err: McpError) -> ApiError {
    match err {
        McpError::Validation(m) => ApiError::validation_failed(m),
        McpError::SecretRefNotFound(slug) => ApiError::new(
            StatusCode::BAD_REQUEST,
            "SECRET_REF_NOT_FOUND",
            format!("secret_ref `{slug}` not found in vault"),
        ),
        McpError::NotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "MCP_SERVER_NOT_FOUND",
            format!("no MCP server with id `{id}`"),
        ),
        McpError::Repository(m) => {
            error!(error = %m, "mcp_server: repository error");
            ApiError::internal()
        }
        McpError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
