//! HTTP handlers for the Orgs surface (M3/P4 — admin page 06).
//!
//! Routes, all gated by `AuthenticatedSession`:
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `POST` | `/api/v0/orgs` | create org via wizard payload |
//! | `GET`  | `/api/v0/orgs` | list all orgs |
//! | `GET`  | `/api/v0/orgs/:id` | show one org (detail; defaults_snapshot included) |
//! | `GET`  | `/api/v0/orgs/:id/dashboard` | consolidated aggregate read (M3/P5) |
//!
//! ## phi-core leverage
//!
//! Q1 **none** at the handler layer — handlers are JSON shims over
//! the business logic in [`crate::platform::orgs`]. Q2: the
//! `CreateOrgRequest` wire shape *may* carry
//! `defaults_snapshot_override` (which wraps 4 phi-core types);
//! handlers pass it through serde untouched. The show-response wire
//! shape surfaces `defaults_snapshot` by design (operator drill-down
//! use case per D11/Q6).
//!
//! The server is **JSON-only**. YAML import/export is a CLI-side
//! concern — matches the `platform/defaults` handler convention.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;

use domain::audit::AuditClass;
use domain::model::composites_m3::{ConsentPolicy, OrganizationDefaultsSnapshot};
use domain::model::ids::{
    AgentId, AuditEventId, AuthRequestId, GrantId, ModelProviderId, NodeId, OrgId,
};
use domain::model::nodes::{ChannelKind, Organization, TemplateKind};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::orgs::{
    create::{create_organization, CreateInput},
    dashboard::{dashboard_summary, DashboardOutcome},
    list::list_organizations,
    show::show_organization,
    OrgError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub display_name: String,
    #[serde(default)]
    pub vision: Option<String>,
    #[serde(default)]
    pub mission: Option<String>,
    pub consent_policy: ConsentPolicy,
    pub audit_class_default: AuditClass,
    pub authority_templates_enabled: Vec<TemplateKind>,
    /// Optional explicit override of the frozen defaults snapshot.
    /// When absent, the server snapshot-copies current platform
    /// defaults (ADR-0019 non-retroactive path).
    #[serde(default)]
    pub defaults_snapshot_override: Option<OrganizationDefaultsSnapshot>,
    #[serde(default)]
    pub default_model_provider: Option<ModelProviderId>,
    pub ceo_display_name: String,
    pub ceo_channel_kind: ChannelKind,
    pub ceo_channel_handle: String,
    pub token_budget: u64,
}

#[derive(Debug, Serialize)]
pub struct CreateOrgResponse {
    pub org_id: OrgId,
    pub ceo_agent_id: AgentId,
    pub ceo_channel_id: NodeId,
    pub ceo_inbox_id: NodeId,
    pub ceo_outbox_id: NodeId,
    pub ceo_grant_id: GrantId,
    pub system_agent_ids: [AgentId; 2],
    pub token_budget_pool_id: NodeId,
    pub adoption_auth_request_ids: Vec<AuthRequestId>,
    pub audit_event_ids: Vec<AuditEventId>,
}

#[derive(Debug, Serialize)]
pub struct OrgListItem {
    pub id: OrgId,
    pub display_name: String,
    pub consent_policy: ConsentPolicy,
    pub authority_templates_enabled: Vec<TemplateKind>,
    pub member_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ListOrgsResponse {
    pub orgs: Vec<OrgListItem>,
}

#[derive(Debug, Serialize)]
pub struct ShowOrgResponse {
    pub organization: Organization,
    pub member_count: usize,
    pub project_count: usize,
    pub adopted_template_count: usize,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn create(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Json(body): Json<CreateOrgRequest>,
) -> Result<Response, ApiError> {
    let outcome = create_organization(
        state.repo.clone(),
        state.audit.clone(),
        CreateInput {
            display_name: body.display_name,
            vision: body.vision,
            mission: body.mission,
            consent_policy: body.consent_policy,
            audit_class_default: body.audit_class_default,
            authority_templates_enabled: body.authority_templates_enabled,
            defaults_snapshot_override: body.defaults_snapshot_override,
            default_model_provider: body.default_model_provider,
            ceo_display_name: body.ceo_display_name,
            ceo_channel_kind: body.ceo_channel_kind,
            ceo_channel_handle: body.ceo_channel_handle,
            token_budget: body.token_budget,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        org_id = %outcome.org_id,
        ceo_agent_id = %outcome.ceo_agent_id,
        audit_event_count = outcome.audit_event_ids.len(),
        "orgs: created",
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateOrgResponse {
            org_id: outcome.org_id,
            ceo_agent_id: outcome.ceo_agent_id,
            ceo_channel_id: outcome.ceo_channel_id,
            ceo_inbox_id: outcome.ceo_inbox_id,
            ceo_outbox_id: outcome.ceo_outbox_id,
            ceo_grant_id: outcome.ceo_grant_id,
            system_agent_ids: outcome.system_agent_ids,
            token_budget_pool_id: outcome.token_budget_pool_id,
            adoption_auth_request_ids: outcome.adoption_auth_request_ids,
            audit_event_ids: outcome.audit_event_ids,
        }),
    )
        .into_response())
}

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
) -> Result<Response, ApiError> {
    let summaries = list_organizations(state.repo.clone())
        .await
        .map_err(error_to_api_error)?;
    let orgs = summaries
        .into_iter()
        .map(|s| OrgListItem {
            id: s.id,
            display_name: s.display_name,
            consent_policy: s.consent_policy,
            authority_templates_enabled: s.authority_templates_enabled,
            member_count: s.member_count,
        })
        .collect();
    Ok((StatusCode::OK, Json(ListOrgsResponse { orgs })).into_response())
}

pub async fn dashboard(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(id): Path<OrgId>,
) -> Result<Response, ApiError> {
    let outcome = dashboard_summary(state.repo.clone(), id, session.agent_id, Utc::now())
        .await
        .map_err(error_to_api_error)?;
    match outcome {
        DashboardOutcome::Found(summary) => Ok((StatusCode::OK, Json(*summary)).into_response()),
        DashboardOutcome::NotFound => Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "ORG_NOT_FOUND",
            format!("no organization with id {id}"),
        )),
        DashboardOutcome::AccessDenied => Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "ORG_ACCESS_DENIED",
            format!("viewer has no relation to org {id}"),
        )),
    }
}

pub async fn show(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(id): Path<OrgId>,
) -> Result<Response, ApiError> {
    let maybe = show_organization(state.repo.clone(), id)
        .await
        .map_err(error_to_api_error)?;
    match maybe {
        Some(d) => Ok((
            StatusCode::OK,
            Json(ShowOrgResponse {
                organization: d.organization,
                member_count: d.member_count,
                project_count: d.project_count,
                adopted_template_count: d.adopted_template_count,
            }),
        )
            .into_response()),
        None => Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "ORG_NOT_FOUND",
            format!("no organization with id {id}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: OrgError) -> ApiError {
    match err {
        OrgError::Validation(m) => ApiError::validation_failed(m),
        OrgError::OrgIdInUse => ApiError::new(
            StatusCode::CONFLICT,
            "ORG_ID_IN_USE",
            "organization with the same id already exists",
        ),
        OrgError::TemplateNotAdoptable(m) => {
            ApiError::new(StatusCode::BAD_REQUEST, "TEMPLATE_NOT_ADOPTABLE", m)
        }
        OrgError::Repository(m) => {
            error!(error = %m, "orgs: repository error");
            ApiError::internal()
        }
        OrgError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
