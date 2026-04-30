//! CH-23 / ADR-0046 — orchestrator for `POST /api/v0/orgs/:org_id/agents/:agent_id/manager`.
//!
//! Lifts the `MANAGES` edge into a real production HTTP path. The
//! orchestrator:
//!
//! 1. Calls [`Repository::create_manages_edge`], whose compound tx
//!    persists the edge row + writes the `platform.manages.edge.created`
//!    audit event atomically.
//! 2. AFTER receipt returns `Ok`, emits
//!    [`DomainEvent::ManagesEdgeCreated`] on the in-process bus so
//!    Template C's listener fires and mints the `[read, inspect]`
//!    grant on `agent:<subordinate>`.
//!
//! Per ADR-0028 (event-bus fail-safe): the durable write happens
//! **before** the bus emit, mirroring the M4 `HasLeadEdgeCreated`
//! pattern at [`crate::platform::projects::create`].
//!
//! Idempotency: a re-POST of the same `(org, manager, subordinate)`
//! triple returns `created = false` from the receipt; the
//! orchestrator suppresses the bus emit on that branch so Template C
//! doesn't double-fire.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, EdgeId, OrgId};
use domain::repository::{Repository, RepositoryError};

/// Orchestrator inputs for `set_agent_manager`.
#[derive(Debug, Clone)]
pub struct SetManagerInput {
    pub org_id: OrgId,
    pub subordinate_agent_id: AgentId,
    pub manager_agent_id: AgentId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

/// Orchestrator output. `created = true` maps to HTTP 201 + bus emit;
/// `created = false` maps to HTTP 200 + suppressed emit (idempotent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetManagerOutcome {
    pub edge_id: EdgeId,
    pub audit_event_id: Option<AuditEventId>,
    pub created: bool,
}

/// Surface error variants the handler maps to HTTP status codes.
#[derive(Debug)]
pub enum SetManagerError {
    /// 400 — input violates a business rule (self-loop, cross-org,
    /// agent not found, etc.).
    Validation(String),
    /// 409 — counterpart agent is archived or disabled (CH-01 /
    /// ADR-0034 invariant).
    AgentInactive(String),
    /// Storage / event-bus / audit failure.
    Repository(String),
}

impl std::fmt::Display for SetManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetManagerError::Validation(m) => write!(f, "validation: {m}"),
            SetManagerError::AgentInactive(m) => write!(f, "agent inactive: {m}"),
            SetManagerError::Repository(m) => write!(f, "repository: {m}"),
        }
    }
}

impl std::error::Error for SetManagerError {}

impl From<RepositoryError> for SetManagerError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::InvalidArgument(m) => SetManagerError::Validation(m),
            RepositoryError::Conflict(m) => SetManagerError::AgentInactive(m),
            RepositoryError::NotFound => SetManagerError::Validation("agent not found".into()),
            other => SetManagerError::Repository(other.to_string()),
        }
    }
}

pub async fn set_agent_manager(
    repo: Arc<dyn Repository>,
    event_bus: Arc<dyn EventBus>,
    input: SetManagerInput,
) -> Result<SetManagerOutcome, SetManagerError> {
    let receipt = repo
        .create_manages_edge(
            input.org_id,
            input.manager_agent_id,
            input.subordinate_agent_id,
            input.actor,
            input.now,
        )
        .await?;

    // Emit the domain event AFTER the durable receipt — fail-safe
    // semantics per ADR-0028. Suppress the emit on the idempotent
    // path so Template C doesn't double-fire.
    if receipt.created {
        let event_id = receipt
            .audit_event_id
            .expect("created=true must carry audit_event_id");
        event_bus
            .emit(DomainEvent::ManagesEdgeCreated {
                org_id: input.org_id,
                manager: input.manager_agent_id,
                subordinate: input.subordinate_agent_id,
                at: input.now,
                event_id,
            })
            .await;
    }

    Ok(SetManagerOutcome {
        edge_id: receipt.edge_id,
        audit_event_id: receipt.audit_event_id,
        created: receipt.created,
    })
}
