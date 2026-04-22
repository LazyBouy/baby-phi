//! HTTP handlers for the Agents surface (M4/P4 — admin page 08).
//!
//! Routes (all gated by `AuthenticatedSession`):
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `GET` | `/api/v0/orgs/:org_id/agents` | list agents in an org |
//!
//! Query params:
//!
//! - `role=executive|admin|member|intern|contract|system` — filter to
//!   agents whose `role` matches (optional).
//! - `search=<text>` — case-insensitive substring over `display_name`
//!   (optional; empty-after-trim returns 400).
//!
//! ## phi-core leverage
//!
//! Q1 **none** at the handler layer — the response wire shape carries
//! phi governance fields only. The broader page-08 pre-audit is
//! pinned in [`crate::platform::agents`] module docs.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::error;

use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{AgentKind, AgentRole};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::agents::{
    list::{list_agents, ListAgentsInput},
    AgentError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes
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

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

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
        AgentError::Repository(m) => {
            error!(error = %m, "agents: repository error");
            ApiError::internal()
        }
    }
}
