//! HTTP handlers for the Agents surface (M4/P4+P5 — admin pages 08+09).
//!
//! Routes (all gated by `AuthenticatedSession`):
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `GET`    | `/api/v0/orgs/:org_id/agents` | list agents in an org (P4) |
//! | `POST`   | `/api/v0/orgs/:org_id/agents` | create an agent (P5) |
//! | `PATCH`  | `/api/v0/agents/:id/profile` | update agent profile (P5) |
//! | `DELETE` | `/api/v0/agents/:id/execution-limits-override` | revert override (P5) |
//!
//! ## phi-core leverage
//!
//! Q1 at the handler layer: handlers themselves are thin JSON shims,
//! so the direct imports happen under [`crate::platform::agents`].
//! The wire shapes for create + update embed phi-core types
//! (`AgentProfile`, `ExecutionLimits`) via serde transit — identical
//! pattern to M3/P4's `CreateOrgRequest` embedding
//! `OrganizationDefaultsSnapshot`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;
use phi_core::context::execution::ExecutionLimits;

use domain::audit::events::m4::agents::ExecutionLimitsSource;
use domain::model::ids::{AgentId, AuditEventId, GrantId, NodeId, OrgId};
use domain::model::nodes::{AgentKind, AgentRole};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::agents::{
    create::{create_agent, CreateAgentInput},
    execution_limits::clear_override,
    list::{list_agents, ListAgentsInput},
    update::{update_agent_profile, ExecutionLimitsPatch, UpdateAgentPatch},
    AgentError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// List (M4/P4)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ListAgentsQuery {
    #[serde(default)]
    pub role: Option<AgentRole>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentRosterItemWire {
    pub id: AgentId,
    pub kind: AgentKind,
    pub display_name: String,
    pub owning_org: Option<OrgId>,
    pub role: Option<AgentRole>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListAgentsResponse {
    pub org_id: OrgId,
    pub agents: Vec<AgentRosterItemWire>,
}

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
    Query(q): Query<ListAgentsQuery>,
) -> Result<Response, ApiError> {
    let rows = list_agents(
        state.repo.clone(),
        org_id,
        ListAgentsInput {
            role: q.role,
            search: q.search,
        },
    )
    .await
    .map_err(error_to_api_error)?;

    let agents = rows
        .into_iter()
        .map(|r| AgentRosterItemWire {
            id: r.agent.id,
            kind: r.agent.kind,
            display_name: r.agent.display_name,
            owning_org: r.agent.owning_org,
            role: r.agent.role,
            created_at: r.agent.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(ListAgentsResponse { org_id, agents })).into_response())
}

// ---------------------------------------------------------------------------
// Create (M4/P5)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub display_name: String,
    pub kind: AgentKind,
    #[serde(default)]
    pub role: Option<AgentRole>,
    /// phi-core blueprint filled in by the wizard. Transits verbatim
    /// via serde — no field-level translation at this layer.
    #[serde(default)]
    pub blueprint: PhiCoreAgentProfile,
    pub parallelize: u32,
    /// Opt-in initial override (ADR-0027). Absent = inherit from org
    /// snapshot (ADR-0023).
    #[serde(default)]
    pub initial_execution_limits_override: Option<ExecutionLimits>,
}

#[derive(Debug, Serialize)]
pub struct CreateAgentResponse {
    pub agent_id: AgentId,
    pub owning_org_id: OrgId,
    pub inbox_id: NodeId,
    pub outbox_id: NodeId,
    pub profile_id: Option<NodeId>,
    pub default_grant_ids: Vec<GrantId>,
    pub execution_limits_override_id: Option<NodeId>,
    pub audit_event_id: AuditEventId,
}

pub async fn create(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(org_id): Path<OrgId>,
    Json(body): Json<CreateAgentRequest>,
) -> Result<Response, ApiError> {
    let outcome = create_agent(
        state.repo.clone(),
        state.audit.clone(),
        CreateAgentInput {
            org_id,
            display_name: body.display_name,
            kind: body.kind,
            role: body.role,
            blueprint: body.blueprint,
            parallelize: body.parallelize,
            initial_execution_limits_override: body.initial_execution_limits_override,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        agent_id = %outcome.agent_id,
        owning_org_id = %outcome.owning_org_id,
        "agents: created",
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateAgentResponse {
            agent_id: outcome.agent_id,
            owning_org_id: outcome.owning_org_id,
            inbox_id: outcome.inbox_id,
            outbox_id: outcome.outbox_id,
            profile_id: outcome.profile_id,
            default_grant_ids: outcome.default_grant_ids,
            execution_limits_override_id: outcome.execution_limits_override_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Update profile (M4/P5)
// ---------------------------------------------------------------------------

/// Wire shape for the ExecutionLimits patch arm.
///
/// - `{"unchanged": null}` (or omit the field entirely)
/// - `{"revert": null}`
/// - `{"set": { ... ExecutionLimits shape ... }}`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionLimitsPatchWire {
    #[default]
    Unchanged,
    Revert,
    Set(ExecutionLimits),
}

impl From<ExecutionLimitsPatchWire> for ExecutionLimitsPatch {
    fn from(w: ExecutionLimitsPatchWire) -> Self {
        match w {
            ExecutionLimitsPatchWire::Unchanged => ExecutionLimitsPatch::Unchanged,
            ExecutionLimitsPatchWire::Revert => ExecutionLimitsPatch::Revert,
            ExecutionLimitsPatchWire::Set(l) => ExecutionLimitsPatch::Set(l),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentProfileRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub parallelize: Option<u32>,
    #[serde(default)]
    pub blueprint: Option<PhiCoreAgentProfile>,
    /// Per-agent `ModelConfig` binding (C-M5-5).
    #[serde(default)]
    pub model_config_id: Option<String>,
    #[serde(default)]
    pub execution_limits: ExecutionLimitsPatchWire,
}

#[derive(Debug, Serialize)]
pub struct UpdateAgentProfileResponse {
    pub agent_id: AgentId,
    pub audit_event_id: Option<AuditEventId>,
    pub execution_limits_source: ExecutionLimitsSource,
}

pub async fn update(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(agent_id): Path<AgentId>,
    Json(body): Json<UpdateAgentProfileRequest>,
) -> Result<Response, ApiError> {
    let outcome = update_agent_profile(
        state.repo.clone(),
        state.audit.clone(),
        agent_id,
        UpdateAgentPatch {
            new_kind: None,
            new_role: None,
            new_owning_org: None,
            display_name: body.display_name,
            parallelize: body.parallelize,
            blueprint: body.blueprint,
            model_config_id: body.model_config_id,
            execution_limits: body.execution_limits.into(),
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    Ok((
        StatusCode::OK,
        Json(UpdateAgentProfileResponse {
            agent_id: outcome.agent_id,
            audit_event_id: outcome.audit_event_id,
            execution_limits_source: outcome.execution_limits_source,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Revert ExecutionLimits override (M4/P5)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RevertLimitsResponse {
    pub agent_id: AgentId,
    pub execution_limits_source: ExecutionLimitsSource,
}

pub async fn revert_execution_limits_override(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Path(agent_id): Path<AgentId>,
) -> Result<Response, ApiError> {
    // Guard: the agent must exist, otherwise we 404 rather than
    // silently succeeding on a no-op delete.
    let exists = state.repo.get_agent(agent_id).await.map_err(|e| {
        error!(error = %e, "agents: repository error on get_agent");
        ApiError::internal()
    })?;
    if exists.is_none() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "AGENT_NOT_FOUND",
            format!("no agent with id {agent_id}"),
        ));
    }

    clear_override(state.repo.clone(), agent_id)
        .await
        .map_err(error_to_api_error)?;

    Ok((
        StatusCode::OK,
        Json(RevertLimitsResponse {
            agent_id,
            execution_limits_source: ExecutionLimitsSource::Inherit,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: AgentError) -> ApiError {
    match err {
        AgentError::Validation(m) => ApiError::validation_failed(m),
        AgentError::OrgNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "ORG_NOT_FOUND",
            format!("no organization with id {id}"),
        ),
        AgentError::AgentNotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "AGENT_NOT_FOUND",
            format!("no agent with id {id}"),
        ),
        AgentError::AgentIdInUse => ApiError::new(
            StatusCode::CONFLICT,
            "AGENT_ID_IN_USE",
            "agent with the same id already exists",
        ),
        AgentError::RoleInvalidForKind { role, kind } => ApiError::new(
            StatusCode::BAD_REQUEST,
            "AGENT_ROLE_INVALID_FOR_KIND",
            format!("role {} invalid for kind {kind:?}", role.as_str()),
        ),
        AgentError::ImmutableFieldChanged(field) => ApiError::new(
            StatusCode::BAD_REQUEST,
            "AGENT_IMMUTABLE_FIELD_CHANGED",
            format!("cannot change immutable field `{field}` post-creation"),
        ),
        AgentError::ParallelizeCeilingExceeded { requested, ceiling } => ApiError::new(
            StatusCode::BAD_REQUEST,
            "PARALLELIZE_CEILING_EXCEEDED",
            format!("parallelize {requested} exceeds ceiling {ceiling}"),
        ),
        AgentError::ExecutionLimitsExceedOrgCeiling(m) => ApiError::new(
            StatusCode::BAD_REQUEST,
            "EXECUTION_LIMITS_EXCEED_ORG_CEILING",
            m,
        ),
        AgentError::ActiveSessionsBlockModelChange => ApiError::new(
            StatusCode::CONFLICT,
            "ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE",
            "cannot change model_config while active sessions exist",
        ),
        AgentError::SystemAgentReadOnly => ApiError::new(
            StatusCode::FORBIDDEN,
            "SYSTEM_AGENT_READ_ONLY",
            "system agents are read-only on page 09",
        ),
        AgentError::Repository(m) => {
            error!(error = %m, "agents: repository error");
            ApiError::internal()
        }
        AgentError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
