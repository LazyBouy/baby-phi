//! POST `/api/v0/orgs/:org/system-agents/:agent_id/disable`
//! (R-ADMIN-13-W3).
//!
//! Disable is distinct from archive: the Agent row stays + the
//! profile_ref stays — only the durable `active: bool` flag flips
//! to `false`. Re-enable is a future phase.
//!
//! CH-01 / ADR-0034 D34.4 — the durable `active: false` flip lands
//! BEFORE the audit emit so the persisted state is correct even if
//! audit emission fails (audit is replayable; durable state is
//! authoritative). Closes drift D6.5.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::model::ids::{AgentId, AuditEventId, OrgId};
use domain::model::nodes::AgentRole;
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{is_standard_system_agent, SystemAgentError};

#[derive(Debug, Clone)]
pub struct DisableInput {
    pub org_id: OrgId,
    pub agent_id: AgentId,
    pub confirm: bool,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableOutcome {
    pub agent_id: AgentId,
    pub disabled_at: DateTime<Utc>,
    pub audit_event_id: AuditEventId,
    /// True when the target was a standard system agent (memory /
    /// catalog). R-ADMIN-13-W3 flagged this as an alerted
    /// warning path; the audit class already handles it.
    pub was_standard: bool,
}

pub async fn disable_system_agent(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: DisableInput,
) -> Result<DisableOutcome, SystemAgentError> {
    if !input.confirm {
        return Err(SystemAgentError::DisableConfirmationRequired);
    }

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
    let was_standard = is_standard_system_agent(profile_ref.as_deref());

    // CH-01 / ADR-0034 D34.4 — durable state-flip BEFORE audit emit.
    // If audit emission later fails, the persisted state is still
    // correct; audit is replayable, durable state is authoritative.
    repo.set_agent_active(input.agent_id, false).await?;

    let event = super::audit_events::system_agent_disabled(
        input.actor,
        input.org_id,
        input.agent_id,
        was_standard,
        profile_ref.as_deref(),
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SystemAgentError::AuditEmit(e.to_string()))?;

    Ok(DisableOutcome {
        agent_id: input.agent_id,
        disabled_at: input.now,
        audit_event_id: event_id,
        was_standard,
    })
}
