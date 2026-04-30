//! CH-23 / ADR-0046 — orchestrator for
//! `POST /api/v0/projects/:project_id/agents/:agent_id/supervisor`.
//!
//! Lifts the `HAS_AGENT_SUPERVISOR` edge into a real production HTTP
//! path. The orchestrator:
//!
//! 1. Calls [`Repository::create_has_agent_supervisor_edge`], whose
//!    compound tx persists the edge row + writes the
//!    `platform.has_agent_supervisor.edge.created` audit event
//!    atomically.
//! 2. AFTER receipt returns `Ok`, emits
//!    [`DomainEvent::HasAgentSupervisorEdgeCreated`] on the
//!    in-process bus so Template D's listener fires and mints the
//!    `[read, inspect]` grant on `project:<p>/agent:<supervisee>`.
//!
//! Per ADR-0028 (event-bus fail-safe): durable write before bus emit.
//! Idempotency: a re-POST returns `created = false`; the bus emit is
//! suppressed on that branch so Template D doesn't double-fire.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, EdgeId, ProjectId};
use domain::repository::{Repository, RepositoryError};

/// Orchestrator inputs for `set_agent_supervisor`.
#[derive(Debug, Clone)]
pub struct SetSupervisorInput {
    pub project_id: ProjectId,
    pub supervisee_agent_id: AgentId,
    pub supervisor_agent_id: AgentId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSupervisorOutcome {
    pub edge_id: EdgeId,
    pub audit_event_id: Option<AuditEventId>,
    pub created: bool,
}

#[derive(Debug)]
pub enum SetSupervisorError {
    Validation(String),
    AgentInactive(String),
    Repository(String),
}

impl std::fmt::Display for SetSupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetSupervisorError::Validation(m) => write!(f, "validation: {m}"),
            SetSupervisorError::AgentInactive(m) => write!(f, "agent inactive: {m}"),
            SetSupervisorError::Repository(m) => write!(f, "repository: {m}"),
        }
    }
}

impl std::error::Error for SetSupervisorError {}

impl From<RepositoryError> for SetSupervisorError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::InvalidArgument(m) => SetSupervisorError::Validation(m),
            RepositoryError::Conflict(m) => SetSupervisorError::AgentInactive(m),
            RepositoryError::NotFound => {
                SetSupervisorError::Validation("project or agent not found".into())
            }
            other => SetSupervisorError::Repository(other.to_string()),
        }
    }
}

pub async fn set_agent_supervisor(
    repo: Arc<dyn Repository>,
    event_bus: Arc<dyn EventBus>,
    input: SetSupervisorInput,
) -> Result<SetSupervisorOutcome, SetSupervisorError> {
    let receipt = repo
        .create_has_agent_supervisor_edge(
            input.project_id,
            input.supervisor_agent_id,
            input.supervisee_agent_id,
            input.actor,
            input.now,
        )
        .await?;

    if receipt.created {
        let event_id = receipt
            .audit_event_id
            .expect("created=true must carry audit_event_id");
        event_bus
            .emit(DomainEvent::HasAgentSupervisorEdgeCreated {
                project_id: input.project_id,
                supervisor: input.supervisor_agent_id,
                supervisee: input.supervisee_agent_id,
                at: input.now,
                event_id,
            })
            .await;
    }

    Ok(SetSupervisorOutcome {
        edge_id: receipt.edge_id,
        audit_event_id: receipt.audit_event_id,
        created: receipt.created,
    })
}
