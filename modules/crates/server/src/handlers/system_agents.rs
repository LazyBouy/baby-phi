//! HTTP handlers for the M5/P6 system-agent surface.
//!
//! Five routes + one shared error-to-ApiError mapper:
//! - `GET   /api/v0/orgs/:org_id/system-agents`
//! - `PATCH /api/v0/orgs/:org_id/system-agents/:agent_id`
//! - `POST  /api/v0/orgs/:org_id/system-agents`
//! - `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/disable`
//! - `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/archive`

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use domain::model::ids::{AgentId, OrgId};
use serde::Deserialize;
use tracing::error;

use crate::handler_support::errors::ApiError;
use crate::handler_support::session::AuthenticatedSession;
use crate::platform::system_agents::{
    add::SystemAgentTrigger, add_system_agent, archive_system_agent, disable_system_agent,
    http_status_for, list_system_agents, tune_system_agent, wire_code_for, AddInput, ArchiveInput,
    DisableInput, SystemAgentError, TuneInput,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// GET /system-agents
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
) -> Result<Response, ApiError> {
    let listing = list_system_agents(state.repo.clone(), org_id)
        .await
        .map_err(system_agent_error_to_api)?;
    Ok((StatusCode::OK, Json(listing)).into_response())
}

// ---------------------------------------------------------------------------
// PATCH /system-agents/:agent_id — tune
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TuneRequest {
    #[serde(default)]
    pub parallelize: Option<u32>,
}

pub async fn tune(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, agent_id)): Path<(OrgId, AgentId)>,
    Json(body): Json<TuneRequest>,
) -> Result<Response, ApiError> {
    let outcome = tune_system_agent(
        state.repo.clone(),
        state.audit.clone(),
        TuneInput {
            org_id,
            agent_id,
            parallelize: body.parallelize,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(system_agent_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /system-agents — add
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub display_name: String,
    pub profile_ref: String,
    pub parallelize: u32,
    pub trigger: String,
}

pub async fn add(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
    Json(body): Json<AddRequest>,
) -> Result<Response, ApiError> {
    let trigger = SystemAgentTrigger::parse(&body.trigger).map_err(system_agent_error_to_api)?;
    let outcome = add_system_agent(
        state.repo.clone(),
        state.audit.clone(),
        AddInput {
            org_id,
            display_name: body.display_name,
            profile_ref: body.profile_ref,
            parallelize: body.parallelize,
            trigger,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(system_agent_error_to_api)?;
    Ok((StatusCode::CREATED, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /system-agents/:agent_id/disable
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DisableRequest {
    #[serde(default)]
    pub confirm: bool,
}

pub async fn disable(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, agent_id)): Path<(OrgId, AgentId)>,
    Json(body): Json<DisableRequest>,
) -> Result<Response, ApiError> {
    let outcome = disable_system_agent(
        state.repo.clone(),
        state.audit.clone(),
        DisableInput {
            org_id,
            agent_id,
            confirm: body.confirm,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(system_agent_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /system-agents/:agent_id/archive
// ---------------------------------------------------------------------------

pub async fn archive(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, agent_id)): Path<(OrgId, AgentId)>,
) -> Result<Response, ApiError> {
    let outcome = archive_system_agent(
        state.repo.clone(),
        state.audit.clone(),
        ArchiveInput {
            org_id,
            agent_id,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(system_agent_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn system_agent_error_to_api(err: SystemAgentError) -> ApiError {
    let status =
        StatusCode::from_u16(http_status_for(&err)).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let code = wire_code_for(&err);
    if let SystemAgentError::Repository(m) | SystemAgentError::AuditEmit(m) = &err {
        error!(error = %m, kind = code, "system_agents: internal error");
    }
    ApiError::new(status, code, err.to_string())
}
