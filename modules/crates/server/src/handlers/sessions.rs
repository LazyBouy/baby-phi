//! HTTP handlers for the M5/P4 session surface.
//!
//! Six routes + one shared error-to-ApiError mapper:
//! - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions` — launch.
//! - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview` — D5.
//! - `GET  /api/v0/sessions/:id` — full SessionDetail.
//! - `POST /api/v0/sessions/:id/terminate` — operator abort.
//! - `GET  /api/v0/projects/:project_id/sessions` — session header list.
//! - `GET  /api/v0/sessions/:id/tools` — C-M5-4 tools resolver.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use domain::model::composites_m5::SessionDetail;
use domain::model::ids::{AgentId, OrgId, ProjectId, SessionId};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tracing::error;

use crate::handler_support::errors::ApiError;
use crate::handler_support::session::AuthenticatedSession;
use crate::platform::sessions::{
    events::{open_live_stream, LiveStreamInput},
    http_status_for, launch_session,
    list::list_sessions_in_project,
    preview_session, show_session, terminate_session,
    tools::resolve_tools_for_session,
    wire_code_for, LaunchInput, LaunchReceipt, PreviewInput, PreviewOutcome, SessionError,
    TerminateInput, TerminateOutcome, ToolSummary,
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
        state.session_live_stream_registry.clone(),
        state.session_max_concurrent,
        state.session_live_stream_buffer,
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
// GET /api/v0/sessions/:id/events — SSE live-event tail (CH-17)
// ---------------------------------------------------------------------------

/// CH-17 / ADR-0055 — operator-facing live transcript surface.
///
/// Subscribes to the per-session
/// `tokio::sync::broadcast::Sender<phi_core::AgentEvent>` populated by
/// the recorder's broadcast tap. Every event becomes an SSE `data:`
/// JSON line on the wire.
///
/// 30-second keep-alive per ADR-0055 §D55.4. Lagging consumers
/// receive a typed `lagged` SSE error event then the stream closes
/// (ADR-0055 §D55.3).
pub async fn events(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(session_id): Path<SessionId>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let subscription = open_live_stream(
        state.repo.clone(),
        state.audit.clone(),
        state.session_live_stream_registry.clone(),
        LiveStreamInput {
            session_id,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(session_error_to_api)?;

    let stream = BroadcastStream::new(subscription.receiver).map(
        |result: Result<phi_core::types::event::AgentEvent, BroadcastStreamRecvError>| match result
        {
            Ok(event) => {
                // Serialise the AgentEvent JSON and surface it as the
                // SSE `data:` line. Failure to serialise is converted
                // into a typed SSE error event so observers see the
                // error rather than a silent close.
                match serde_json::to_string(&event) {
                    Ok(payload) => Ok(Event::default().event("agent_event").data(payload)),
                    Err(e) => Ok(Event::default()
                        .event("serialize_error")
                        .data(format!("{{\"error\":\"{e}\"}}"))),
                }
            }
            Err(BroadcastStreamRecvError::Lagged(missed)) => {
                // CH-17 / ADR-0055 §D55.3 — slow consumer fell behind
                // the broadcast buffer. Emit a typed `lagged` SSE
                // error event then let the stream close (the
                // BroadcastStream returns `None` on the next poll
                // after a Lagged error).
                Ok(Event::default()
                    .event("lagged")
                    .data(format!("{{\"missed\":{missed}}}")))
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text(": keep-alive"),
    ))
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
