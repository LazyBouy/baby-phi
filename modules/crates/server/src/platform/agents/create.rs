//! Agent-creation orchestrator (page 09 create mode, M4/P5).
//!
//! ## phi-core leverage
//!
//! - **Q1 direct imports**:
//!   - [`phi_core::agents::profile::AgentProfile`] — the CLI + web
//!     wizard supply a phi-core blueprint; we clone + tweak it into the
//!     baby-phi governance `AgentProfile.blueprint` field.
//!   - [`phi_core::context::execution::ExecutionLimits`] — optional
//!     initial override (ADR-0027 opt-in path).
//!   - [`phi_core::provider::model::ModelConfig`] — the `model_config`
//!     handle on the blueprint; validated against the org catalogue.
//!   - [`phi_core::types::ThinkingLevel`] — passed through in the
//!     supplied blueprint (no special handling here).
//! - **Q2 transitive**: the compound tx payload `AgentCreationPayload`
//!   ships the full `AgentProfile` (with phi-core blueprint) and
//!   optional `AgentExecutionLimitsOverride` (wrapping phi-core's
//!   `ExecutionLimits`) to SurrealDB via serde.
//! - **Q3 rejections**: `ContextConfig` + `RetryConfig` stay
//!   inherit-from-snapshot (ADR-0023). No per-agent row for those.
//!
//! ## Flow
//!
//! 1. Pre-flight validation (kind/role pairing, parallelize range,
//!    org existence, initial override bounds).
//! 2. Build the `Agent`, `InboxObject`, `OutboxObject`, optional
//!    `AgentProfile`, optional `AgentExecutionLimitsOverride`.
//! 3. Call [`domain::Repository::apply_agent_creation`] — single
//!    compound tx (M4/P3 commitment C10).
//! 4. Emit `platform.agent.created` (Alerted) via the audit emitter.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;
use phi_core::context::execution::ExecutionLimits;

use domain::audit::events::m4::agents::agent_created;
use domain::audit::AuditEmitter;
use domain::model::composites_m4::AgentExecutionLimitsOverride;
use domain::model::ids::{AgentId, AuditEventId, GrantId, NodeId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentProfile, AgentRole, InboxObject, OutboxObject};
use domain::repository::{AgentCreationPayload, Repository};

use super::AgentError;

/// Upper bound on `parallelize` at M4. Per-org caps are M5 work.
pub const PARALLELIZE_MAX_CAP: u32 = 64;

/// Compile-time coercion checks — pin the phi-core types the
/// orchestrator binds to. If a shadowing local redeclaration ever
/// drifts in, these stop compiling.
#[allow(dead_code)]
fn _is_phi_core_agent_profile(_: &PhiCoreAgentProfile) {}
#[allow(dead_code)]
fn _is_phi_core_execution_limits(_: &ExecutionLimits) {}

/// Orchestrator input. Shape mirrors the wire payload on
/// `POST /api/v0/orgs/:org_id/agents`.
#[derive(Debug, Clone)]
pub struct CreateAgentInput {
    pub org_id: OrgId,
    pub display_name: String,
    pub kind: AgentKind,
    pub role: Option<AgentRole>,
    /// phi-core blueprint the operator-supplied wizard filled in
    /// (system prompt, thinking level, model config, etc.). For
    /// humans, typically `AgentProfile::default()`.
    pub blueprint: PhiCoreAgentProfile,
    /// Concurrent-session cap. `1..=PARALLELIZE_MAX_CAP`.
    pub parallelize: u32,
    /// Opt-in per-agent `ExecutionLimits` override (ADR-0027). `None`
    /// = inherit from org snapshot (ADR-0023, default path).
    pub initial_execution_limits_override: Option<ExecutionLimits>,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

/// Orchestrator output. Every id pairs with a row the compound tx
/// persisted.
#[derive(Debug, Clone)]
pub struct CreatedAgent {
    pub agent_id: AgentId,
    pub owning_org_id: OrgId,
    pub inbox_id: NodeId,
    pub outbox_id: NodeId,
    pub profile_id: Option<NodeId>,
    pub default_grant_ids: Vec<GrantId>,
    pub execution_limits_override_id: Option<NodeId>,
    pub audit_event_id: AuditEventId,
}

pub async fn create_agent(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: CreateAgentInput,
) -> Result<CreatedAgent, AgentError> {
    // ---- Shape validation -------------------------------------------------
    if input.display_name.trim().is_empty() {
        return Err(AgentError::Validation(
            "display_name must be non-empty".into(),
        ));
    }
    if input.parallelize == 0 || input.parallelize > PARALLELIZE_MAX_CAP {
        return Err(AgentError::ParallelizeCeilingExceeded {
            requested: input.parallelize,
            ceiling: PARALLELIZE_MAX_CAP,
        });
    }
    if let Some(role) = input.role {
        if !role.is_valid_for(input.kind) {
            return Err(AgentError::RoleInvalidForKind {
                role,
                kind: input.kind,
            });
        }
    }

    // ---- Org existence ----------------------------------------------------
    let org = repo
        .get_organization(input.org_id)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?
        .ok_or(AgentError::OrgNotFound(input.org_id))?;

    // ---- Assemble rows ----------------------------------------------------
    let agent_id = AgentId::new();
    let agent = Agent {
        id: agent_id,
        kind: input.kind,
        display_name: input.display_name.trim().to_string(),
        owning_org: Some(org.id),
        role: input.role,
        created_at: input.now,
    };
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: input.now,
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: input.now,
    };

    // Profile — always write one at creation for LLMs; optional for
    // humans (governance may want to attach a profile stub for
    // human-operated agents later).
    let profile = if matches!(input.kind, AgentKind::Llm) {
        Some(AgentProfile {
            id: NodeId::new(),
            agent_id,
            parallelize: input.parallelize,
            blueprint: input.blueprint.clone(),
            created_at: input.now,
        })
    } else {
        None
    };

    // Optional initial override — bounds-check against the org
    // ceiling (the snapshot's `execution_limits`; phi-core default
    // when the snapshot is absent).
    let initial_override = match input.initial_execution_limits_override.as_ref() {
        None => None,
        Some(lim) => {
            let ceiling = org
                .defaults_snapshot
                .as_ref()
                .map(|s| s.execution_limits.clone())
                .unwrap_or_default();
            let row = AgentExecutionLimitsOverride {
                id: NodeId::new(),
                owning_agent: agent_id,
                limits: lim.clone(),
                created_at: input.now,
            };
            if !row.is_bounded_by(&ceiling) {
                return Err(AgentError::ExecutionLimitsExceedOrgCeiling(format!(
                    "initial override (turns={}, tokens={}, duration_secs={}, cost={:?}) exceeds org ceiling \
                     (turns={}, tokens={}, duration_secs={}, cost={:?})",
                    row.limits.max_turns,
                    row.limits.max_total_tokens,
                    row.limits.max_duration.as_secs(),
                    row.limits.max_cost,
                    ceiling.max_turns,
                    ceiling.max_total_tokens,
                    ceiling.max_duration.as_secs(),
                    ceiling.max_cost,
                )));
            }
            Some(row)
        }
    };

    let initial_override_id = initial_override.as_ref().map(|o| o.id);

    let payload = AgentCreationPayload {
        agent: agent.clone(),
        inbox: inbox.clone(),
        outbox: outbox.clone(),
        profile: profile.clone(),
        default_grants: vec![],
        initial_execution_limits_override: initial_override,
        catalogue_entries: vec![
            (format!("agent:{}", agent_id), "control_plane".into()),
            (format!("agent:{}/inbox", agent_id), "control_plane".into()),
            (format!("agent:{}/outbox", agent_id), "control_plane".into()),
        ],
    };

    let receipt = repo
        .apply_agent_creation(&payload)
        .await
        .map_err(AgentError::from)?;

    // ---- Audit (after successful commit) ----------------------------------
    let event = agent_created(
        input.actor,
        &agent,
        org.id,
        profile.as_ref(),
        input.initial_execution_limits_override.as_ref(),
        None, // no provenance AR at M4 — self-serve create
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| AgentError::AuditEmit(e.to_string()))?;

    Ok(CreatedAgent {
        agent_id: receipt.agent_id,
        owning_org_id: receipt.owning_org_id,
        inbox_id: receipt.inbox_id,
        outbox_id: receipt.outbox_id,
        profile_id: receipt.profile_id,
        default_grant_ids: receipt.default_grant_ids,
        execution_limits_override_id: initial_override_id.or(receipt.execution_limits_override_id),
        audit_event_id: event_id,
    })
}
