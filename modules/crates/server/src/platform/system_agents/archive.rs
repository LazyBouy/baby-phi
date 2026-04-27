//! POST `/api/v0/orgs/:org/system-agents/:agent_id/archive`
//! (R-ADMIN-13-W4).
//!
//! Archive is permitted on **org-specific** system agents only.
//! The two standard system agents (memory-extraction +
//! agent-catalog) can be disabled but not archived — archiving
//! them would break the platform invariants that rely on the
//! profile_ref being bound to a live Agent node.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, OrgId};
use domain::model::nodes::AgentRole;
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{is_standard_system_agent, SystemAgentError};

#[derive(Debug, Clone)]
pub struct ArchiveInput {
    pub org_id: OrgId,
    pub agent_id: AgentId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOutcome {
    pub agent_id: AgentId,
    pub archived_at: DateTime<Utc>,
    pub audit_event_id: AuditEventId,
}

pub async fn archive_system_agent(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    input: ArchiveInput,
) -> Result<ArchiveOutcome, SystemAgentError> {
    let agent = repo
        .get_agent(input.agent_id)
        .await?
        .ok_or(SystemAgentError::NotFound {
            org: input.org_id,
            agent: input.agent_id,
        })?;
    if !matches!(agent.role, Some(AgentRole::System)) {
        return Err(SystemAgentError::WrongKind(input.agent_id));
    }
    if !matches!(agent.owning_org, Some(org) if org == input.org_id) {
        return Err(SystemAgentError::NotFound {
            org: input.org_id,
            agent: input.agent_id,
        });
    }

    let profile = repo.get_agent_profile_for_agent(input.agent_id).await?;
    let profile_ref = profile.and_then(|p| p.blueprint.config_id);
    if is_standard_system_agent(profile_ref.as_deref()) {
        return Err(SystemAgentError::StandardNotArchivable(input.agent_id));
    }

    // CH-01 / ADR-0034 D34.4 — durable state-flip BEFORE audit emit.
    // `archived_at = Some(input.now)` marks the row as terminally
    // soft-deleted; AgentCatalogListener (CH-22) reads this when
    // computing tile state. Re-archive is idempotent because we
    // overwrite with the latest timestamp; the flow rejects standard
    // system agents earlier so there's no state conflict.
    repo.set_agent_archived_at(input.agent_id, Some(input.now))
        .await?;

    let event = super::audit_events::system_agent_archived(
        input.actor,
        input.org_id,
        input.agent_id,
        profile_ref.as_deref(),
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SystemAgentError::AuditEmit(e.to_string()))?;

    // CH-22 — emit `AgentArchived` so the catalog listener flips the
    // catalog row's `active` to false (archived_at = Some triggers
    // ADR-0034 §D34.5 archive-wins-ties). ADR-0028 fail-safe: emit
    // after the durable flip + audit so listener faults never
    // invalidate the persisted state.
    event_bus
        .emit(DomainEvent::AgentArchived {
            agent_id: input.agent_id,
            at: input.now,
            event_id: AuditEventId::new(),
        })
        .await;

    Ok(ArchiveOutcome {
        agent_id: input.agent_id,
        archived_at: input.now,
        audit_event_id: event_id,
    })
}
