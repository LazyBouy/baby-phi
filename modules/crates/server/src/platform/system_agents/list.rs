//! GET `/api/v0/orgs/:org/system-agents` — page-13 listing
//! (R-ADMIN-13-R1/R2/R3/R4).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::model::composites_m5::SystemAgentRuntimeStatus;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentProfile, AgentRole};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{is_standard_system_agent, SystemAgentError};

/// One row in the standard / org_specific bucket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemAgentRow {
    pub agent_id: AgentId,
    pub display_name: String,
    pub profile_ref: Option<String>,
    pub parallelize: Option<u32>,
    pub queue_depth: u32,
    pub last_fired_at: Option<DateTime<Utc>>,
    /// `active` mirrors the agent row + the most-recent disable
    /// state; page 13 shows `status: running|degraded|disabled`.
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAgentsListing {
    pub standard: Vec<SystemAgentRow>,
    pub org_specific: Vec<SystemAgentRow>,
    /// Recent cross-agent fire events for the timeline panel
    /// (R-ADMIN-13-R3). At M5/P6 this surfaces the `last_fired_at`
    /// value per row — sorted newest-first, capped at 20. Per-fire
    /// event rows land when M5/P8 memory-extraction + agent-catalog
    /// listeners write per-event log lines (drift D6.X).
    pub recent_events: Vec<SystemAgentRecentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemAgentRecentEvent {
    pub agent_id: AgentId,
    pub at: DateTime<Utc>,
}

pub async fn list_system_agents(
    repo: Arc<dyn Repository>,
    org: OrgId,
) -> Result<SystemAgentsListing, SystemAgentError> {
    let organization = repo
        .get_organization(org)
        .await?
        .ok_or(SystemAgentError::OrgNotFound(org))?;

    // System agents in the org: either explicitly tagged on the
    // `Organization.system_agents` composite (fixture + M3 wizard
    // path) OR carry `AgentRole::System` (M5/P6 add-agent path).
    // Union both to be robust to fixture + production shapes.
    let system_agent_ids: std::collections::HashSet<AgentId> =
        organization.system_agents.iter().copied().collect();
    let agents = repo.list_agents_in_org(org).await?;
    let system_agents: Vec<Agent> = agents
        .into_iter()
        .filter(|a| system_agent_ids.contains(&a.id) || matches!(a.role, Some(AgentRole::System)))
        .collect();

    // Runtime-status rows indexed by agent_id.
    let statuses: Vec<SystemAgentRuntimeStatus> =
        repo.fetch_system_agent_runtime_status_for_org(org).await?;
    let status_by_agent: std::collections::HashMap<AgentId, &SystemAgentRuntimeStatus> =
        statuses.iter().map(|s| (s.agent_id, s)).collect();

    let mut standard: Vec<SystemAgentRow> = Vec::new();
    let mut org_specific: Vec<SystemAgentRow> = Vec::new();
    let mut recent_events: Vec<SystemAgentRecentEvent> = Vec::new();

    for agent in &system_agents {
        let profile: Option<AgentProfile> = repo.get_agent_profile_for_agent(agent.id).await?;
        let status = status_by_agent.get(&agent.id);
        let profile_ref = profile.as_ref().and_then(|p| p.blueprint.config_id.clone());
        let parallelize = profile.as_ref().map(|p| p.parallelize);
        let row = SystemAgentRow {
            agent_id: agent.id,
            display_name: agent.display_name.clone(),
            profile_ref: profile_ref.clone(),
            parallelize,
            queue_depth: status.map(|s| s.queue_depth).unwrap_or(0),
            last_fired_at: status.and_then(|s| s.last_fired_at),
            active: true, // soft-disable flag lands in D6.1 drift; default true at M5
        };
        if let Some(fired_at) = row.last_fired_at {
            recent_events.push(SystemAgentRecentEvent {
                agent_id: agent.id,
                at: fired_at,
            });
        }
        if is_standard_system_agent(profile_ref.as_deref()) {
            standard.push(row);
        } else {
            org_specific.push(row);
        }
    }

    // Stable ordering per requirements: display_name ASC within
    // bucket; recent_events newest-first capped at 20.
    standard.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    org_specific.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    recent_events.sort_by_key(|e| std::cmp::Reverse(e.at));
    recent_events.truncate(20);

    Ok(SystemAgentsListing {
        standard,
        org_specific,
        recent_events,
    })
}
