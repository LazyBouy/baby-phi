//! HTTP handlers for the M5/P5 authority-template surface.
//!
//! Five routes + one shared error-to-ApiError mapper:
//! - `GET  /api/v0/orgs/:org_id/authority-templates` — list.
//! - `POST /api/v0/orgs/:org_id/authority-templates/:kind/approve`
//! - `POST /api/v0/orgs/:org_id/authority-templates/:kind/deny`
//! - `POST /api/v0/orgs/:org_id/authority-templates/:kind/adopt`
//! - `POST /api/v0/orgs/:org_id/authority-templates/:kind/revoke`

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use domain::model::ids::OrgId;
use domain::model::nodes::TemplateKind;
use serde::Deserialize;
use tracing::error;

use crate::handler_support::errors::ApiError;
use crate::handler_support::session::AuthenticatedSession;
use crate::platform::templates::{
    adopt_template_inline, approve_adoption_ar, deny_adoption_ar, http_status_for,
    list_templates_for_org, revoke_template, wire_code_for, AdoptInput, ApproveInput, DenyInput,
    RevokeInput, TemplateError,
};
use crate::state::AppState;

fn parse_kind(s: &str) -> Result<TemplateKind, ApiError> {
    match s.to_ascii_lowercase().as_str() {
        "a" => Ok(TemplateKind::A),
        "b" => Ok(TemplateKind::B),
        "c" => Ok(TemplateKind::C),
        "d" => Ok(TemplateKind::D),
        // `e` surfaces as always-available via the `list` endpoint
        // but returns a 400 on approve / deny / adopt / revoke so
        // the operator sees a clear reason.
        "e" | "e-always" => Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "TEMPLATE_E_ALWAYS_AVAILABLE",
            "Template E is always available on demand; no adoption lifecycle",
        )),
        "system_bootstrap" | "f" => Err(ApiError::new(
            StatusCode::CONFLICT,
            "TEMPLATE_KIND_NOT_ADOPTABLE",
            format!("template kind `{s}` is not adoptable via this surface"),
        )),
        other => Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "TEMPLATE_INPUT_INVALID",
            format!("unknown template kind `{other}`"),
        )),
    }
}

// ---------------------------------------------------------------------------
// GET /api/v0/orgs/:org_id/authority-templates
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
) -> Result<Response, ApiError> {
    let listing = list_templates_for_org(state.repo.clone(), org_id)
        .await
        .map_err(template_error_to_api)?;
    Ok((StatusCode::OK, Json(listing)).into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/authority-templates/:kind/approve
// ---------------------------------------------------------------------------

pub async fn approve(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, kind_slug)): Path<(OrgId, String)>,
) -> Result<Response, ApiError> {
    let kind = parse_kind(&kind_slug)?;
    let outcome = approve_adoption_ar(
        state.repo.clone(),
        state.audit.clone(),
        ApproveInput {
            org_id,
            kind,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(template_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/authority-templates/:kind/deny
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DenyRequest {
    pub reason: String,
}

pub async fn deny(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, kind_slug)): Path<(OrgId, String)>,
    Json(body): Json<DenyRequest>,
) -> Result<Response, ApiError> {
    let kind = parse_kind(&kind_slug)?;
    let outcome = deny_adoption_ar(
        state.repo.clone(),
        state.audit.clone(),
        DenyInput {
            org_id,
            kind,
            reason: body.reason,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(template_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/authority-templates/:kind/adopt
// ---------------------------------------------------------------------------

pub async fn adopt(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, kind_slug)): Path<(OrgId, String)>,
) -> Result<Response, ApiError> {
    let kind = parse_kind(&kind_slug)?;
    let outcome = adopt_template_inline(
        state.repo.clone(),
        state.audit.clone(),
        AdoptInput {
            org_id,
            kind,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(template_error_to_api)?;
    Ok((StatusCode::CREATED, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// POST /api/v0/orgs/:org_id/authority-templates/:kind/revoke
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    pub reason: String,
}

pub async fn revoke(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path((org_id, kind_slug)): Path<(OrgId, String)>,
    Json(body): Json<RevokeRequest>,
) -> Result<Response, ApiError> {
    let kind = parse_kind(&kind_slug)?;
    let outcome = revoke_template(
        state.repo.clone(),
        state.audit.clone(),
        RevokeInput {
            org_id,
            kind,
            reason: body.reason,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(template_error_to_api)?;
    Ok((StatusCode::OK, Json(outcome)).into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn template_error_to_api(err: TemplateError) -> ApiError {
    let status =
        StatusCode::from_u16(http_status_for(&err)).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let code = wire_code_for(&err);
    match &err {
        TemplateError::Repository(m) | TemplateError::AuditEmit(m) => {
            error!(error = %m, kind = code, "templates: internal error");
        }
        _ => {}
    }
    ApiError::new(status, code, err.to_string())
}
