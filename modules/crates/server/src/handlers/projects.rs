//! HTTP handlers for the Projects surface (M4/P6 — admin page 10).
//!
//! Routes (all gated by `AuthenticatedSession`):
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `POST` | `/api/v0/orgs/:org_id/projects` | Create project (Shape A → materialised 201; Shape B → pending 202) |
//! | `POST` | `/api/v0/projects/_pending/:ar_id/approve` | Transition one slot on a Shape B AR (approve or deny) |
//!
//! ## phi-core leverage
//!
//! **Q1 direct imports: 0** at this layer. The handler is a thin JSON
//! shim over [`crate::platform::projects::create`] — no phi-core
//! types transit the wire shape at P6.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use domain::model::composites_m4::{KeyResult, Objective, ResourceBoundaries};
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, EdgeId, OrgId, ProjectId};
use domain::model::nodes::{AuthRequestState, ProjectShape};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::projects::{
    create::{
        approve_pending_shape_b, create_project, ApprovalOutcome, ApproveShapeBInput,
        CreateProjectInput, CreateProjectOutcome,
    },
    detail::{apply_okr_patch, project_detail, DetailOutcome, OkrPatchEntry, ProjectDetail},
    ProjectError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Create wire shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub project_id: ProjectId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub goal: Option<String>,
    pub shape: ProjectShape,
    #[serde(default)]
    pub co_owner_org_id: Option<OrgId>,
    pub lead_agent_id: AgentId,
    #[serde(default)]
    pub member_agent_ids: Vec<AgentId>,
    #[serde(default)]
    pub sponsor_agent_ids: Vec<AgentId>,
    #[serde(default)]
    pub token_budget: Option<u64>,
    #[serde(default)]
    pub objectives: Vec<Objective>,
    #[serde(default)]
    pub key_results: Vec<KeyResult>,
    #[serde(default)]
    pub resource_boundaries: Option<ResourceBoundaries>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum CreateProjectResponse {
    Materialised {
        project_id: ProjectId,
        lead_agent_id: AgentId,
        has_lead_edge_id: EdgeId,
        owning_org_ids: Vec<OrgId>,
        audit_event_id: AuditEventId,
    },
    Pending {
        pending_ar_id: AuthRequestId,
        approver_ids: [AgentId; 2],
        audit_event_id: AuditEventId,
    },
}

pub async fn create(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Response, ApiError> {
    let outcome = create_project(
        state.repo.clone(),
        state.audit.clone(),
        state.event_bus.clone(),
        CreateProjectInput {
            org_id,
            project_id: body.project_id,
            name: body.name,
            description: body.description,
            goal: body.goal,
            shape: body.shape,
            co_owner_org_id: body.co_owner_org_id,
            lead_agent_id: body.lead_agent_id,
            member_agent_ids: body.member_agent_ids,
            sponsor_agent_ids: body.sponsor_agent_ids,
            token_budget: body.token_budget,
            objectives: body.objectives,
            key_results: body.key_results,
            resource_boundaries: body.resource_boundaries,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    match outcome {
        CreateProjectOutcome::Materialised(m) => {
            info!(project_id = %m.project_id, "projects: shape A materialised");
            Ok((
                StatusCode::CREATED,
                Json(CreateProjectResponse::Materialised {
                    project_id: m.project_id,
                    lead_agent_id: m.lead_agent_id,
                    has_lead_edge_id: m.has_lead_edge_id,
                    owning_org_ids: m.owning_org_ids,
                    audit_event_id: m.audit_event_id,
                }),
            )
                .into_response())
        }
        CreateProjectOutcome::Pending(p) => {
            info!(ar_id = %p.pending_ar_id, "projects: shape B pending");
            Ok((
                StatusCode::ACCEPTED,
                Json(CreateProjectResponse::Pending {
                    pending_ar_id: p.pending_ar_id,
                    approver_ids: p.approver_ids,
                    audit_event_id: p.audit_event_id,
                }),
            )
                .into_response())
        }
    }
}

// ---------------------------------------------------------------------------
// Approve pending Shape B wire shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ApprovePendingRequest {
    /// The agent doing the approving/denying (one of the AR's slots).
    pub approver_id: AgentId,
    /// `true` to approve; `false` to deny.
    pub approve: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ApprovePendingResponse {
    StillPending {
        ar_id: AuthRequestId,
    },
    Terminal {
        ar_id: AuthRequestId,
        state: AuthRequestState,
        /// Present only when the terminal state is Approved AND the
        /// materialisation-after-approve path is wired (M5 — see
        /// C-M5-6 in the base build plan). At M4 this is `null` for
        /// every terminal outcome.
        project_id: Option<ProjectId>,
    },
}

pub async fn approve_pending(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(ar_id): Path<AuthRequestId>,
    Json(body): Json<ApprovePendingRequest>,
) -> Result<Response, ApiError> {
    let outcome = approve_pending_shape_b(
        state.repo.clone(),
        state.audit.clone(),
        state.event_bus.clone(),
        ApproveShapeBInput {
            ar_id,
            approver_id: body.approver_id,
            approve: body.approve,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    match outcome {
        ApprovalOutcome::StillPending { ar_id } => {
            info!(ar_id = %ar_id, "projects: shape B slot transitioned, still pending");
            Ok((
                StatusCode::OK,
                Json(ApprovePendingResponse::StillPending { ar_id }),
            )
                .into_response())
        }
        ApprovalOutcome::Terminal {
            ar_id,
            state,
            project,
        } => {
            info!(ar_id = %ar_id, terminal = ?state, "projects: shape B terminal");
            Ok((
                StatusCode::OK,
                Json(ApprovePendingResponse::Terminal {
                    ar_id,
                    state,
                    project_id: project.map(|p| p.project_id),
                }),
            )
                .into_response())
        }
    }
}

// ---------------------------------------------------------------------------
// Detail (M4/P7) — `GET /api/v0/projects/:id`
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ProjectDetailResponse {
    #[serde(flatten)]
    pub detail: Box<ProjectDetail>,
}

pub async fn show(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(project_id): Path<ProjectId>,
) -> Result<Response, ApiError> {
    let outcome = project_detail(state.repo.clone(), project_id, session.agent_id)
        .await
        .map_err(error_to_api_error)?;
    match outcome {
        DetailOutcome::Found(detail) => {
            Ok((StatusCode::OK, Json(ProjectDetailResponse { detail })).into_response())
        }
        DetailOutcome::NotFound => Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "PROJECT_NOT_FOUND",
            format!("no project with id {project_id}"),
        )),
        DetailOutcome::AccessDenied => Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "PROJECT_ACCESS_DENIED",
            "caller has no relation to any owning org or project roster",
        )),
    }
}

// ---------------------------------------------------------------------------
// OKR patch (M4/P7) — `PATCH /api/v0/projects/:id/okrs`
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OkrPatchRequest {
    pub patches: Vec<OkrPatchEntry>,
}

#[derive(Debug, Serialize)]
pub struct OkrPatchResponse {
    pub project_id: ProjectId,
    pub audit_event_ids: Vec<AuditEventId>,
    pub objectives: Vec<domain::model::composites_m4::Objective>,
    pub key_results: Vec<domain::model::composites_m4::KeyResult>,
}

pub async fn update_okrs(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(project_id): Path<ProjectId>,
    Json(body): Json<OkrPatchRequest>,
) -> Result<Response, ApiError> {
    let receipt = apply_okr_patch(
        state.repo.clone(),
        state.audit.clone(),
        project_id,
        session.agent_id,
        body.patches,
        Utc::now(),
    )
    .await
    .map_err(error_to_api_error)?;
    info!(project_id = %receipt.project_id, edits = receipt.audit_event_ids.len(), "projects: OKR patch applied");
    Ok((
        StatusCode::OK,
        Json(OkrPatchResponse {
            project_id: receipt.project_id,
            audit_event_ids: receipt.audit_event_ids,
            objectives: receipt.objectives_after,
            key_results: receipt.key_results_after,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: ProjectError) -> ApiError {
    match err {
        ProjectError::Validation(m) => ApiError::validation_failed(m),
        ProjectError::OkrValidation(m) => {
            ApiError::new(StatusCode::BAD_REQUEST, "OKR_VALIDATION_FAILED", m)
        }
        ProjectError::OrgNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "ORG_NOT_FOUND",
            format!("no organization with id {id}"),
        ),
        ProjectError::CoOwnerInvalid(m) => {
            ApiError::new(StatusCode::BAD_REQUEST, "CO_OWNER_INVALID", m)
        }
        ProjectError::LeadNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "LEAD_NOT_FOUND",
            format!("no agent with id {id}"),
        ),
        ProjectError::LeadNotInOwningOrg => ApiError::new(
            StatusCode::BAD_REQUEST,
            "LEAD_NOT_IN_OWNING_ORG",
            "lead agent does not belong to an owning org",
        ),
        ProjectError::MemberInvalid(m) => {
            ApiError::new(StatusCode::BAD_REQUEST, "MEMBER_INVALID", m)
        }
        ProjectError::ShapeBMissingCoOwner => ApiError::new(
            StatusCode::BAD_REQUEST,
            "SHAPE_B_MISSING_CO_OWNER",
            "Shape B requires co_owner_org_id",
        ),
        ProjectError::ShapeAHasCoOwner => ApiError::new(
            StatusCode::BAD_REQUEST,
            "SHAPE_A_HAS_CO_OWNER",
            "Shape A must not supply co_owner_org_id",
        ),
        ProjectError::ProjectIdInUse(id) => ApiError::new(
            StatusCode::CONFLICT,
            "PROJECT_ID_IN_USE",
            format!("project id {id} already in use"),
        ),
        ProjectError::PendingArNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "PENDING_AR_NOT_FOUND",
            format!("no pending auth request with id {id}"),
        ),
        ProjectError::PendingArNotShapeB => ApiError::new(
            StatusCode::BAD_REQUEST,
            "PENDING_AR_NOT_SHAPE_B",
            "auth request is not a Shape B project-creation AR",
        ),
        ProjectError::PendingArAlreadyTerminal => ApiError::new(
            StatusCode::CONFLICT,
            "PENDING_AR_ALREADY_TERMINAL",
            "auth request is already terminal",
        ),
        ProjectError::ApproverNotAuthorized => ApiError::new(
            StatusCode::FORBIDDEN,
            "APPROVER_NOT_AUTHORIZED",
            "caller is not a designated approver for this auth request",
        ),
        ProjectError::Repository(m) => {
            error!(error = %m, "projects: repository error");
            ApiError::internal()
        }
        ProjectError::AuditEmit(m) => ApiError::audit_emit_failed(m),
        ProjectError::Transition(m) => {
            ApiError::new(StatusCode::BAD_REQUEST, "TRANSITION_ILLEGAL", m)
        }
    }
}
