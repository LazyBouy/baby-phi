//! Per-agent ExecutionLimits resolver + mutation helpers (ADR-0027).
//!
//! ## phi-core leverage
//!
//! - **Q1 direct imports**: [`phi_core::context::execution::ExecutionLimits`]
//!   — the entire module traffics in this type. Every call-site gets
//!   the resolved limits by **value** of phi-core's type directly (no
//!   wrapping, no re-declaration). This is the "zero coupling loss"
//!   pattern ADR-0027 mandates.
//! - **Q2 transitive**: `AgentExecutionLimitsOverride` wraps
//!   `ExecutionLimits` via serde through the `agent_execution_limits`
//!   SurrealDB row (per migration 0004).
//! - **Q3 rejections**: `phi_core::ContextConfig` + `RetryConfig` stay
//!   inherit-from-snapshot per ADR-0023 (M3 decision). No per-agent
//!   override for those at M4.
//!
//! ## Three resolution paths
//!
//! Per [`domain::Repository::resolve_effective_execution_limits`]:
//!
//! 1. Override row present → return it (the operator explicitly opted
//!    in; bounds have been checked at write time).
//! 2. No override, agent has `owning_org` with a
//!    `defaults_snapshot.execution_limits` → return the snapshot value
//!    (ADR-0023 inherit path — the default).
//! 3. Neither — return [`phi_core::ExecutionLimits::default()`] and
//!    emit a `tracing::warn!` so operators see the fallback. Only
//!    happens for pre-M3 rows deserialised from a non-snapshot
//!    org; should be empty at M4+.

use std::sync::Arc;

use phi_core::context::execution::ExecutionLimits;
use tracing::warn;

use domain::model::composites_m4::AgentExecutionLimitsOverride;
use domain::model::ids::AgentId;
use domain::repository::Repository;

use super::AgentError;

/// Compile-time coercion check — the resolver must return
/// phi-core's type by value (no local re-declaration). If someone
/// introduces a `struct ExecutionLimits` under
/// `modules/crates/domain/` this function stops compiling against
/// our wrap, catching the drift at build time.
///
/// Called nowhere at runtime; the compiler uses it as a witness.
#[allow(dead_code)]
fn _is_phi_core_execution_limits(_: &ExecutionLimits) {}

/// Resolve an agent's effective `ExecutionLimits`. Single entry point
/// — both read-only handlers (dashboards, agent-profile show) and
/// mutation-path bounds checks should call this helper rather than
/// walking the graph themselves.
pub async fn resolve_effective_limits(
    repo: Arc<dyn Repository>,
    agent: AgentId,
) -> Result<ExecutionLimits, AgentError> {
    match repo
        .resolve_effective_execution_limits(agent)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?
    {
        Some(limits) => Ok(limits),
        None => {
            warn!(
                agent_id = %agent,
                "agent has neither override nor org snapshot — falling back to phi_core::ExecutionLimits::default()",
            );
            Ok(ExecutionLimits::default())
        }
    }
}

/// Read the org ceiling — the value per-agent overrides must stay
/// `≤`. Returns the org's `defaults_snapshot.execution_limits` if
/// present, else phi-core's default (which is then the effective
/// ceiling).
pub async fn org_ceiling_for_agent(
    repo: Arc<dyn Repository>,
    agent: AgentId,
) -> Result<ExecutionLimits, AgentError> {
    let agent_row = repo
        .get_agent(agent)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?
        .ok_or(AgentError::AgentNotFound(agent))?;
    let org = agent_row.owning_org.ok_or_else(|| {
        AgentError::Validation("agent has no owning_org — cannot compute org ceiling".into())
    })?;
    let org_row = repo
        .get_organization(org)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?
        .ok_or(AgentError::OrgNotFound(org))?;
    Ok(org_row
        .defaults_snapshot
        .map(|s| s.execution_limits)
        .unwrap_or_default())
}

/// Persist a per-agent override. Validates
/// [`AgentExecutionLimitsOverride::is_bounded_by`] against the
/// resolved org ceiling; returns
/// [`AgentError::ExecutionLimitsExceedOrgCeiling`] on breach.
pub async fn apply_override(
    repo: Arc<dyn Repository>,
    override_row: &AgentExecutionLimitsOverride,
) -> Result<(), AgentError> {
    let ceiling = org_ceiling_for_agent(repo.clone(), override_row.owning_agent).await?;
    if !override_row.is_bounded_by(&ceiling) {
        return Err(AgentError::ExecutionLimitsExceedOrgCeiling(format!(
            "override (turns={}, tokens={}, duration_secs={}, cost={:?}) exceeds org ceiling \
             (turns={}, tokens={}, duration_secs={}, cost={:?})",
            override_row.limits.max_turns,
            override_row.limits.max_total_tokens,
            override_row.limits.max_duration.as_secs(),
            override_row.limits.max_cost,
            ceiling.max_turns,
            ceiling.max_total_tokens,
            ceiling.max_duration.as_secs(),
            ceiling.max_cost,
        )));
    }
    repo.set_agent_execution_limits_override(override_row)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;
    Ok(())
}

/// Idempotent delete. Safe whether or not a row exists.
pub async fn clear_override(repo: Arc<dyn Repository>, agent: AgentId) -> Result<(), AgentError> {
    repo.clear_agent_execution_limits_override(agent)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::audit::AuditClass;
    use domain::in_memory::InMemoryRepository;
    use domain::model::composites_m3::{ConsentPolicy, OrganizationDefaultsSnapshot};
    use domain::model::ids::{NodeId, OrgId};
    use domain::model::nodes::{Agent, AgentKind, Organization};
    use std::time::Duration;

    fn org_with_ceiling(id: OrgId, ceiling: ExecutionLimits) -> Organization {
        Organization {
            id,
            display_name: format!("org-{id}"),
            vision: None,
            mission: None,
            consent_policy: ConsentPolicy::Implicit,
            audit_class_default: AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: Some(OrganizationDefaultsSnapshot {
                execution_limits: ceiling,
                default_agent_profile: Default::default(),
                context_config: Default::default(),
                retry_config: Default::default(),
                default_retention_days: 30,
                default_alert_channels: vec![],
            }),
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        }
    }

    fn agent_in_org(org: OrgId) -> Agent {
        Agent {
            id: domain::model::ids::AgentId::new(),
            kind: AgentKind::Llm,
            display_name: "probe".into(),
            owning_org: Some(org),
            role: None,
            created_at: Utc::now(),
        }
    }

    fn limits(max_turns: usize, max_duration_secs: u64) -> ExecutionLimits {
        ExecutionLimits {
            max_turns,
            max_total_tokens: 500_000,
            max_duration: Duration::from_secs(max_duration_secs),
            max_cost: Some(5.0),
        }
    }

    #[tokio::test]
    async fn resolve_returns_override_when_present() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let org_id = OrgId::new();
        repo.create_organization(&org_with_ceiling(org_id, limits(50, 600)))
            .await
            .unwrap();
        let agent = agent_in_org(org_id);
        let aid = agent.id;
        repo.create_agent(&agent).await.unwrap();
        let ovr = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: aid,
            limits: limits(10, 60),
            created_at: Utc::now(),
        };
        apply_override(repo.clone(), &ovr).await.unwrap();
        let effective = resolve_effective_limits(repo, aid).await.unwrap();
        assert_eq!(effective.max_turns, 10);
    }

    #[tokio::test]
    async fn resolve_returns_org_snapshot_when_no_override() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let org_id = OrgId::new();
        repo.create_organization(&org_with_ceiling(org_id, limits(99, 900)))
            .await
            .unwrap();
        let agent = agent_in_org(org_id);
        let aid = agent.id;
        repo.create_agent(&agent).await.unwrap();
        let effective = resolve_effective_limits(repo, aid).await.unwrap();
        assert_eq!(effective.max_turns, 99);
    }

    #[tokio::test]
    async fn override_rejected_when_exceeds_ceiling() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let org_id = OrgId::new();
        repo.create_organization(&org_with_ceiling(org_id, limits(50, 600)))
            .await
            .unwrap();
        let agent = agent_in_org(org_id);
        let aid = agent.id;
        repo.create_agent(&agent).await.unwrap();
        let too_high = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: aid,
            limits: limits(200, 60), // turns > ceiling 50
            created_at: Utc::now(),
        };
        let err = apply_override(repo, &too_high).await.unwrap_err();
        assert!(matches!(
            err,
            AgentError::ExecutionLimitsExceedOrgCeiling(_)
        ));
    }

    #[tokio::test]
    async fn clear_override_is_idempotent() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let org_id = OrgId::new();
        repo.create_organization(&org_with_ceiling(org_id, limits(50, 600)))
            .await
            .unwrap();
        let agent = agent_in_org(org_id);
        let aid = agent.id;
        repo.create_agent(&agent).await.unwrap();
        // No override yet — clear should still succeed.
        clear_override(repo.clone(), aid).await.unwrap();
        // Persist + clear.
        let ovr = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: aid,
            limits: limits(10, 60),
            created_at: Utc::now(),
        };
        apply_override(repo.clone(), &ovr).await.unwrap();
        clear_override(repo.clone(), aid).await.unwrap();
        let effective = resolve_effective_limits(repo, aid).await.unwrap();
        assert_eq!(effective.max_turns, 50, "should fall back to org ceiling");
    }
}
