//! HTTP handlers for the M5/P4 session surface.
//!
//! Six routes + one shared error-to-ApiError mapper:
//! - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions` — launch.
//! - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview` — D5.
//! - `GET  /api/v0/sessions/:id` — full SessionDetail.
//! - `POST /api/v0/sessions/:id/terminate` — operator abort.
//! - `GET  /api/v0/projects/:project_id/sessions` — session header list.
//! - `GET  /api/v0/sessions/:id/tools` — C-M5-4 tools resolver.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use domain::model::composites_m5::SessionDetail;
use domain::model::ids::{AgentId, OrgId, ProjectId, SessionId};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::handler_support::errors::ApiError;
use crate::handler_support::session::AuthenticatedSession;
use crate::platform::sessions::{
    http_status_for, launch_session, list::list_sessions_in_project, preview_session, show_session,
    terminate_session, tools::resolve_tools_for_session, wire_code_for, LaunchInput, LaunchReceipt,
    PreviewInput, PreviewOutcome, SessionError, TerminateInput, TerminateOutcome, ToolSummary,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/projects/:project_id/sessions
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LaunchRequest {
    pub agent_id: AgentId,
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct LaunchResponse {
    pub session_id: SessionId,
    pub first_loop_id: domain::model::ids::LoopId,
    pub session_started_event_id: domain::model::ids::AuditEventId,
    pub permission_check: domain::permissions::Decision,
}

pub async fn launch(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, project_id)): Path<(OrgId, ProjectId)>,
    Json(body): Json<LaunchRequest>,
) -> Result<Response, ApiError> {
    let receipt: LaunchReceipt = launch_session(
        state.repo.clone(),
        state.audit.clone(),
        state.event_bus.clone(),
        state.session_registry.clone(),
        state.session_max_concurrent,
        LaunchInput {
            org_id,
            project_id,
            agent_id: body.agent_id,
            prompt: body.prompt,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(session_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(LaunchResponse {
            session_id: receipt.session_id,
            first_loop_id: receipt.first_loop_id,
            session_started_event_id: receipt.session_started_event_id,
            permission_check: receipt.permission_check_decision,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub agent_id: AgentId,
}

pub async fn preview(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path((org_id, project_id)): Path<(OrgId, ProjectId)>,
    Json(body): Json<PreviewRequest>,
) -> Result<Response, ApiError> {
    let outcome: PreviewOutcome = preview_session(
        state.repo.clone(),
        PreviewInput {
            org_id,
            project_id,
            agent_id: body.agent_id,
        },
    )
    .await
    .map_err(session_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// GET /api/v0/sessions/:id — SessionDetail
// ---------------------------------------------------------------------------

pub async fn show(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(session_id): Path<SessionId>,
) -> Result<Response, ApiError> {
    let detail: SessionDetail = show_session(state.repo.clone(), session_id, session.agent_id)
        .await
        .map_err(session_error_to_api)?;
    Ok((StatusCode::OK, Json(detail)).into_response())
}

// ---------------------------------------------------------------------------
// GET /api/v0/projects/:project_id/sessions — header strip
// ---------------------------------------------------------------------------

pub async fn list_in_project(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(project_id): Path<ProjectId>,
) -> Result<Response, ApiError> {
    let list = list_sessions_in_project(state.repo.clone(), project_id)
        .await
        .map_err(session_error_to_api)?;
    Ok((StatusCode::OK, Json(list)).into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/sessions/:id/terminate
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TerminateRequest {
    pub reason: String,
}

pub async fn terminate(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(session_id): Path<SessionId>,
    Json(body): Json<TerminateRequest>,
) -> Result<Response, ApiError> {
    let outcome: TerminateOutcome = terminate_session(
        state.repo.clone(),
        state.audit.clone(),
        state.event_bus.clone(),
        state.session_registry.clone(),
        TerminateInput {
            session_id,
            reason: body.reason,
            terminated_by: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(session_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// GET /api/v0/sessions/:id/tools
// ---------------------------------------------------------------------------

pub async fn tools(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(session_id): Path<SessionId>,
) -> Result<Response, ApiError> {
    let list: Vec<ToolSummary> =
        resolve_tools_for_session(state.repo.clone(), session_id, session.agent_id)
            .await
            .map_err(session_error_to_api)?;
    Ok((StatusCode::OK, Json(list)).into_response())
}

// ---------------------------------------------------------------------------
// Error mapping — SessionError → ApiError
// ---------------------------------------------------------------------------

fn session_error_to_api(err: SessionError) -> ApiError {
    let status =
        StatusCode::from_u16(http_status_for(&err)).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let code = wire_code_for(&err);
    match &err {
        SessionError::RecorderFailure(m)
        | SessionError::CompoundTxFailure(m)
        | SessionError::Repository(m)
        | SessionError::AuditEmit(m)
        | SessionError::SessionReplayPanic(m) => {
            error!(error = %m, kind = code, "sessions: internal error");
        }
        _ => {}
    }
    ApiError::new(status, code, err.to_string())
}
