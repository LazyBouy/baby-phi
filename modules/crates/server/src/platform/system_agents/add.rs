//! POST `/api/v0/orgs/:org/system-agents` — create a new
//! org-specific System agent (R-ADMIN-13-W2).
//!
//! Flow:
//! 1. Validate org exists + no id clash + trigger shape + parallelize.
//! 2. Build the Agent row with `kind: System`, the InboxObject,
//!    OutboxObject, AgentProfile.
//! 3. `apply_agent_creation` compound tx persists the agent +
//!    profile + default edges.
//! 4. Emit `platform.system_agent.added` audit.
//!
//! ## phi-core leverage
//!
//! One new direct import at P6:
//! - `phi_core::agents::profile::AgentProfile` — the system
//!   agent's profile carries a phi-core blueprint verbatim
//!   (name / description / system_prompt / config_id /
//!   thinking_level). The `profile_ref` field on the input maps
//!   to `blueprint.config_id`.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentProfile, AgentRole, InboxObject, OutboxObject};
use domain::repository::AgentCreationPayload;
use domain::Repository;
use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;
use serde::{Deserialize, Serialize};

use super::SystemAgentError;

/// The five trigger kinds page-13 surfaces (§Part 1.5 Q3 rejects
/// `phi_core::AgentEvent` — triggers are a governance concept,
/// not agent-loop telemetry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemAgentTrigger {
    SessionEnd,
    EdgeChange,
    Periodic,
    Explicit,
    CustomEvent,
}

impl SystemAgentTrigger {
    pub fn parse(slug: &str) -> Result<Self, SystemAgentError> {
        match slug {
            "session_end" => Ok(Self::SessionEnd),
            "edge_change" => Ok(Self::EdgeChange),
            "periodic" => Ok(Self::Periodic),
            "explicit" => Ok(Self::Explicit),
            "custom_event" => Ok(Self::CustomEvent),
            other => Err(SystemAgentError::TriggerTypeInvalid(other.to_string())),
        }
    }
}

const SYSTEM_AGENT_PARALLELIZE_CAP: u32 = 32;

#[derive(Debug, Clone)]
pub struct AddInput {
    pub org_id: OrgId,
    pub display_name: String,
    pub profile_ref: String,
    pub parallelize: u32,
    pub trigger: SystemAgentTrigger,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOutcome {
    pub agent_id: AgentId,
    pub audit_event_id: AuditEventId,
}

pub async fn add_system_agent(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    input: AddInput,
) -> Result<AddOutcome, SystemAgentError> {
    if input.display_name.trim().is_empty() {
        return Err(SystemAgentError::InputInvalid(
            "display_name must not be empty".into(),
        ));
    }
    if input.parallelize == 0 || input.parallelize > SYSTEM_AGENT_PARALLELIZE_CAP {
        return Err(SystemAgentError::ParallelizeCeilingExceeded {
            requested: input.parallelize,
            ceiling: SYSTEM_AGENT_PARALLELIZE_CAP,
        });
    }
    if input.profile_ref.trim().is_empty() {
        return Err(SystemAgentError::ProfileRefUnknown(
            input.profile_ref.clone(),
        ));
    }

    // Org must exist.
    let _ = repo
        .get_organization(input.org_id)
        .await?
        .ok_or(SystemAgentError::OrgNotFound(input.org_id))?;

    // Agent row.
    let agent_id = AgentId::new();
    // System agents are LLM-kind (no Humans provisioned via this
    // flow) with `AgentRole::System` — that role is the sole
    // discriminator for "is this a system agent" throughout the
    // rest of the module.
    let agent = Agent {
        id: agent_id,
        kind: AgentKind::Llm,
        display_name: input.display_name.trim().to_string(),
        owning_org: Some(input.org_id),
        role: Some(AgentRole::System),
        created_at: input.now,
        active: true,
        archived_at: None,
    };
    let inbox_id = NodeId::new();
    let outbox_id = NodeId::new();
    let inbox = InboxObject {
        id: inbox_id,
        agent_id,
        created_at: input.now,
        tags: domain::model::composites::auto_tags_for("inbox", &inbox_id.to_string()).to_vec(),
    };
    let outbox = OutboxObject {
        id: outbox_id,
        agent_id,
        created_at: input.now,
        tags: domain::model::composites::auto_tags_for("outbox", &outbox_id.to_string()).to_vec(),
    };

    // Profile — phi-core's AgentProfile is the single source of
    // truth for `config_id` (used as the `profile_ref`).
    let blueprint = PhiCoreAgentProfile {
        config_id: Some(input.profile_ref.clone()),
        name: Some(agent.display_name.clone()),
        ..PhiCoreAgentProfile::default()
    };
    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize: input.parallelize,
        blueprint,
        model_config_id: None,
        mock_response: None,
        created_at: input.now,
    };

    // CH-16 / ADR-0038 §D38.1 — system agents are LLM-kind by definition;
    // every system agent gets an Identity row (eager creation in the same
    // compound tx as the Agent row).
    let identity = Some(domain::model::nodes::Identity::default_for_llm(
        agent_id, input.now,
    ));
    let payload = AgentCreationPayload {
        agent: agent.clone(),
        inbox,
        outbox,
        profile: Some(profile),
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity,
    };
    let _receipt = repo.apply_agent_creation(&payload).await?;

    let event = super::audit_events::system_agent_added(
        input.actor,
        input.org_id,
        agent_id,
        &input.profile_ref,
        input.parallelize,
        input.trigger,
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SystemAgentError::AuditEmit(e.to_string()))?;

    // CH-16 — system agents are LLM-kind by definition, so an Identity
    // row was always written; emit the matching `platform.identity.created`
    // audit event after the system-agent-added event.
    if let Some(ref iden) = payload.identity {
        let identity_event = domain::audit::events::m5_2::identity::identity_created(
            input.actor,
            iden,
            input.org_id,
            None,
            input.now,
        );
        audit
            .emit(identity_event)
            .await
            .map_err(|e| SystemAgentError::AuditEmit(e.to_string()))?;
    }

    // CH-22 — emit `AgentCreated` so the catalog listener seeds a
    // catalog row for the new system agent. ADR-0028 fail-safe:
    // emit AFTER commit + audit.
    event_bus
        .emit(DomainEvent::AgentCreated {
            agent_id,
            owning_org: input.org_id,
            agent_kind: AgentKind::Llm,
            role: Some(AgentRole::System),
            at: input.now,
            event_id: AuditEventId::new(),
        })
        .await;

    Ok(AddOutcome {
        agent_id,
        audit_event_id: event_id,
    })
}
