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

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, EdgeId, ProjectId};
use domain::model::nodes::PrincipalRef;
use domain::permissions::catalogue::StaticCatalogue;
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::permissions::Action;
use domain::repository::{Repository, RepositoryError};

use crate::handler_support::permission::check_permission;

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
    /// CH-26 / ADR-0061 §D61.5 — Permission Check engine returned
    /// Denied for the requested action on `project:<id>`. Maps to 403
    /// `NO_GRANTS_HELD` per CH-25 wire convention at the HTTP boundary.
    PermissionDenied(String),
    Repository(String),
}

impl std::fmt::Display for SetSupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetSupervisorError::Validation(m) => write!(f, "validation: {m}"),
            SetSupervisorError::AgentInactive(m) => write!(f, "agent inactive: {m}"),
            SetSupervisorError::PermissionDenied(m) => write!(f, "permission denied: {m}"),
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
    // CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — engine-routed
    // Observe check on `project:<id>` before mutation. **BLOCKING** as
    // of CH-27: engine deny propagates as
    // `SetSupervisorError::PermissionDenied` → HTTP 403 NO_GRANTS_HELD.
    // The CH-25 synth-owner-grant rule (widened to 4-verb scope at CH-27
    // / ADR-0062 §D62.2) resolves Allow for the owning Agent of the
    // project. The bespoke compound-tx discipline (project + agent
    // existence + active-status) stays as defence-in-depth.
    is_observe_allowed_on_project(&*repo, input.actor, input.project_id).await?;

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

/// CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — engine-routed
/// Observe gate on `project:<id>`. **BLOCKING** as of CH-27. Symmetric
/// peer to the orgs-tier helpers; resolves Allow via the CH-25
/// synth-owner-grant rule when the actor owns the project (widened to
/// 4-verb scope at CH-27 / ADR-0062 §D62.2). Engine denials propagate
/// as `SetSupervisorError::PermissionDenied` mapped to HTTP 403
/// NO_GRANTS_HELD per CH-25 wire convention.
async fn is_observe_allowed_on_project(
    repo: &dyn Repository,
    actor: AgentId,
    project_id: ProjectId,
) -> Result<(), SetSupervisorError> {
    let uri = format!("project:{}", project_id);
    let agent_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(actor))
        .await
        .map_err(|e| SetSupervisorError::Repository(e.to_string()))?;
    let agent_owned_orgs = repo
        .list_agent_owned_orgs(actor)
        .await
        .map_err(|e| SetSupervisorError::Repository(e.to_string()))?;
    let agent_owned_projects = repo
        .list_agent_owned_projects(actor)
        .await
        .map_err(|e| SetSupervisorError::Repository(e.to_string()))?;
    let mut catalogue = StaticCatalogue::empty();
    catalogue.seed(None, &uri);
    let consents = ConsentIndex::empty();
    let gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();
    let ctx = CheckContext {
        agent: actor,
        current_org: None,
        current_project: Some(project_id),
        current_session: None,
        agent_grants: &agent_grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        session_org_tags: &[],
        session_project_tags: &[],
        agent_owned_orgs: &agent_owned_orgs,
        agent_owned_projects: &agent_owned_projects,
        call: ToolCall {
            target_uri: uri.clone(),
            target_tags: vec![format!("project:{}", project_id)],
            ..Default::default()
        },
    };
    let manifest = Manifest {
        actions: vec![Action::Observe],
        resource: vec!["identity_principal".to_string()],
        ..Default::default()
    };
    check_permission(&ctx, &manifest, &NoopMetrics)
        .map(|_resolved| ())
        .map_err(|api_err| SetSupervisorError::PermissionDenied(api_err.message))
}
