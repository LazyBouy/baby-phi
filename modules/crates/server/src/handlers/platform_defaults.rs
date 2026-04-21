//! HTTP handlers for the Platform Defaults surface (page 05).
//!
//! Two routes, both gated by `AuthenticatedSession`:
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `GET` | `/api/v0/platform/defaults` | read current + factory + `persisted` flag |
//! | `PUT` | `/api/v0/platform/defaults` | update (requires `if_version` for optimistic concurrency) |
//!
//! The wire body for PUT is:
//!
//! ```jsonc
//! {
//!   "if_version": 3,
//!   "defaults": { /* full PlatformDefaults struct — phi-core fields */ }
//! }
//! ```
//!
//! The server is **JSON-only**. YAML / TOML import/export is a
//! CLI-side concern — the CLI converts into JSON before PUTting. This
//! matches the phi-core leverage boundary (`parse_config` is phi-core's
//! multi-format pipeline for `AgentConfig`; baby-phi's `PlatformDefaults`
//! is a different struct, so we don't reach through phi-core's parser
//! on the server side).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use domain::model::ids::{AuditEventId, AuthRequestId};
use domain::model::PlatformDefaults;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::defaults::{
    get::{get_platform_defaults, GetInput},
    put::{put_platform_defaults, PutInput},
    DefaultsError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes — request / response
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PutDefaultsRequest {
    pub if_version: u64,
    pub defaults: PlatformDefaults,
}

#[derive(Debug, Serialize)]
pub struct GetDefaultsResponse {
    pub defaults: PlatformDefaults,
    /// `true` when a row exists; `false` when `defaults` is
    /// synthesised from `PlatformDefaults::factory(now)`.
    pub persisted: bool,
    /// Factory baseline — the "revert" target, always present.
    pub factory: PlatformDefaults,
}

#[derive(Debug, Serialize)]
pub struct PutDefaultsResponse {
    pub new_version: u64,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
) -> Result<Response, ApiError> {
    let outcome = get_platform_defaults(state.repo.clone(), GetInput { now: Utc::now() })
        .await
        .map_err(error_to_api_error)?;
    Ok((
        StatusCode::OK,
        Json(GetDefaultsResponse {
            defaults: outcome.defaults,
            persisted: outcome.persisted,
            factory: outcome.factory,
        }),
    )
        .into_response())
}

pub async fn put(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Json(body): Json<PutDefaultsRequest>,
) -> Result<Response, ApiError> {
    let outcome = put_platform_defaults(
        state.repo.clone(),
        state.audit.clone(),
        PutInput {
            if_version: body.if_version,
            defaults: body.defaults,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        new_version = outcome.new_version,
        auth_request_id = %outcome.auth_request_id,
        audit_event_id = %outcome.audit_event_id,
        "platform_defaults: updated",
    );

    Ok((
        StatusCode::OK,
        Json(PutDefaultsResponse {
            new_version: outcome.new_version,
            auth_request_id: outcome.auth_request_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: DefaultsError) -> ApiError {
    match err {
        DefaultsError::Validation(m) => ApiError::validation_failed(m),
        DefaultsError::StaleWrite { current_version } => ApiError::new(
            StatusCode::CONFLICT,
            "PLATFORM_DEFAULTS_STALE_WRITE",
            format!(
                "stale if_version; current server-side version is {current_version} — re-read and retry"
            ),
        ),
        DefaultsError::Repository(m) => {
            error!(error = %m, "platform_defaults: repository error");
            ApiError::internal()
        }
        DefaultsError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
