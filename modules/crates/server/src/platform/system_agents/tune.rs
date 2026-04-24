//! PATCH `/api/v0/orgs/:org/system-agents/:agent_id` —
//! adjust system agent `parallelize` (R-ADMIN-13-W1).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::model::ids::{AgentId, AuditEventId, OrgId};
use domain::model::nodes::AgentRole;
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::SystemAgentError;

/// Absolute max `parallelize` for system agents at M5. Matches the
/// M4 project-agent ceiling from
/// `server::platform::agents::create::PARALLELIZE_MAX_CAP`.
const SYSTEM_AGENT_PARALLELIZE_CAP: u32 = 32;

#[derive(Debug, Clone)]
pub struct TuneInput {
    pub org_id: OrgId,
    pub agent_id: AgentId,
    pub parallelize: Option<u32>,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuneOutcome {
    pub agent_id: AgentId,
    pub updated_at: DateTime<Utc>,
    pub audit_event_id: Option<AuditEventId>,
}

pub async fn tune_system_agent(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: TuneInput,
) -> Result<TuneOutcome, SystemAgentError> {
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

    // Validate parallelize if supplied.
    if let Some(p) = input.parallelize {
        if p == 0 || p > SYSTEM_AGENT_PARALLELIZE_CAP {
            return Err(SystemAgentError::ParallelizeCeilingExceeded {
                requested: p,
                ceiling: SYSTEM_AGENT_PARALLELIZE_CAP,
            });
        }
    }

    let current_profile = repo.get_agent_profile_for_agent(input.agent_id).await?;
    let mut changed = false;
    let mut diff_before: Option<u32> = None;
    let mut diff_after: Option<u32> = None;

    if let Some(new_parallelize) = input.parallelize {
        match current_profile.as_ref() {
            Some(p) if p.parallelize != new_parallelize => {
                diff_before = Some(p.parallelize);
                diff_after = Some(new_parallelize);
                let mut next = p.clone();
                next.parallelize = new_parallelize;
                repo.upsert_agent_profile(&next).await?;
                changed = true;
            }
            Some(_) => {
                // no-op — requested value equals current.
            }
            None => {
                // No profile yet — refuse; system agents always
                // have a profile at creation time.
                return Err(SystemAgentError::InputInvalid(format!(
                    "agent {} has no profile row to tune",
                    input.agent_id
                )));
            }
        }
    }

    let audit_event_id = if changed {
        let event = super::audit_events::system_agent_reconfigured(
            input.actor,
            input.org_id,
            input.agent_id,
            diff_before,
            diff_after,
            input.now,
        );
        let id = event.event_id;
        audit
            .emit(event)
            .await
            .map_err(|e| SystemAgentError::AuditEmit(e.to_string()))?;
        Some(id)
    } else {
        None
    };

    Ok(TuneOutcome {
        agent_id: input.agent_id,
        updated_at: input.now,
        audit_event_id,
    })
}
