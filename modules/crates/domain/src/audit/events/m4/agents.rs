//! Audit-event builders for pages 08 (roster list — read-only, no
//! events) + 09 (agent profile editor — creates + edits).
//!
//! - `platform.agent.created` (Alerted) — emitted after M4/P5's
//!   `apply_agent_creation` compound tx commits. Captures the new
//!   Agent row + role + owning_org + optional initial ExecutionLimits
//!   override + optional profile.
//! - `platform.agent_profile.updated` (Logged by default; Alerted
//!   when `model_config_id` or `role` changes since those are
//!   security-sensitive). The diff carries the before/after snapshot
//!   so the dashboard + ops reviewers can see exactly what changed
//!   without re-querying storage.
//!
//! ## phi-core leverage
//!
//! None directly. The events transit phi-core-typed fields
//! (`AgentProfile.blueprint`, `AgentExecutionLimitsOverride.limits`)
//! via the `diff` JSON — the wrap transit is identical to the
//! pattern M3 uses for `organization.defaults_snapshot`.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};
use crate::model::nodes::{Agent, AgentProfile, AgentRole};

/// Scaffold for every M4 agent-scoped audit event — `org_scope = Some`
/// continues the per-org chain opened at M3.
#[allow(clippy::too_many_arguments)]
fn scaffold(
    event_type: &str,
    actor: AgentId,
    target_agent: AgentId,
    org: OrgId,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    audit_class: AuditClass,
    provenance_auth_request_id: Option<AuthRequestId>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*target_agent.as_uuid())),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `platform.agent.created` — Alerted.
///
/// Emitted by the M4/P5 agent-creation handler immediately after
/// [`crate::Repository::apply_agent_creation`] commits. Carries a
/// snapshot of the new Agent + optional profile + optional initial
/// execution-limits override so a reviewer can reconstruct the
/// compound write from a single event.
pub fn agent_created(
    actor: AgentId,
    agent: &Agent,
    org: OrgId,
    profile: Option<&AgentProfile>,
    initial_limits: Option<&phi_core::context::execution::ExecutionLimits>,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "agent_id":     agent.id.to_string(),
            "kind":         agent.kind,
            "role":         agent.role.map(|r| r.as_str()),
            "display_name": agent.display_name,
            "owning_org":   agent.owning_org.map(|o| o.to_string()),
            "has_profile":  profile.is_some(),
            // The blueprint transits phi-core's `AgentProfile` wrap
            // via serde. The audit diff omits the full blueprint for
            // size reasons; reviewers can dereference via
            // `Repository::get_agent_profile` when needed.
            "profile_parallelize": profile.map(|p| p.parallelize),
            "initial_execution_limits_override": initial_limits.map(|l| serde_json::json!({
                "max_turns":        l.max_turns,
                "max_total_tokens": l.max_total_tokens,
                "max_duration_secs": l.max_duration.as_secs(),
                "max_cost":          l.max_cost,
            })),
            "created_at": agent.created_at,
        },
    });
    scaffold(
        "platform.agent.created",
        actor,
        agent.id,
        org,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// Structured patch for `agent_profile_updated`. Captures the
/// operator-visible fields M4/P5's editor exposes; other callers that
/// need to record lower-frequency edits (e.g. M5 role promotions)
/// introduce their own builders.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AgentProfilePatchDiff {
    pub display_name: Option<(String, String)>,
    pub system_prompt: Option<(Option<String>, Option<String>)>,
    pub temperature: Option<(Option<f32>, Option<f32>)>,
    pub thinking_level: Option<(
        Option<phi_core::types::ThinkingLevel>,
        Option<phi_core::types::ThinkingLevel>,
    )>,
    pub parallelize: Option<(u32, u32)>,
    pub role: Option<(Option<AgentRole>, Option<AgentRole>)>,
    pub model_config_id: Option<(Option<String>, Option<String>)>,
    /// Source of truth for effective execution limits: `Inherit` = no
    /// override row (ADR-0023 default); `Override` = row present.
    pub execution_limits_source: Option<(ExecutionLimitsSource, ExecutionLimitsSource)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionLimitsSource {
    Inherit,
    Override,
}

impl AgentProfilePatchDiff {
    /// True when at least one field is populated — lets the handler
    /// avoid emitting a no-op audit event if the patch is effectively
    /// empty.
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.system_prompt.is_none()
            && self.temperature.is_none()
            && self.thinking_level.is_none()
            && self.parallelize.is_none()
            && self.role.is_none()
            && self.model_config_id.is_none()
            && self.execution_limits_source.is_none()
    }

    /// Audit-class promotion rule: `model_config_id` and `role`
    /// changes are security-sensitive; everything else is Logged.
    pub fn audit_class(&self) -> AuditClass {
        if self.model_config_id.is_some() || self.role.is_some() {
            AuditClass::Alerted
        } else {
            AuditClass::Logged
        }
    }
}

/// `platform.agent_profile.updated` — Logged, promoted to Alerted
/// when `model_config_id` or `role` changes (see
/// [`AgentProfilePatchDiff::audit_class`]).
pub fn agent_profile_updated(
    actor: AgentId,
    target_agent: AgentId,
    org: OrgId,
    patch: AgentProfilePatchDiff,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let audit_class = patch.audit_class();
    let diff = serde_json::json!({ "patch": patch });
    scaffold(
        "platform.agent_profile.updated",
        actor,
        target_agent,
        org,
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::NodeId;
    use crate::model::nodes::AgentKind;

    fn sample_agent(org: OrgId, role: Option<AgentRole>) -> Agent {
        Agent {
            id: AgentId::new(),
            kind: AgentKind::Llm,
            display_name: "Worker".into(),
            owning_org: Some(org),
            role,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn agent_created_is_alerted_and_org_scoped() {
        let org = OrgId::new();
        let a = sample_agent(org, Some(AgentRole::Intern));
        let ev = agent_created(AgentId::new(), &a, org, None, None, None, Utc::now());
        assert_eq!(ev.event_type, "platform.agent.created");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(
            ev.target_entity_id,
            Some(NodeId::from_uuid(*a.id.as_uuid()))
        );
        assert_eq!(ev.diff["after"]["role"], "intern");
    }

    #[test]
    fn agent_created_without_role_renders_role_as_null() {
        let org = OrgId::new();
        let a = sample_agent(org, None);
        let ev = agent_created(AgentId::new(), &a, org, None, None, None, Utc::now());
        assert!(ev.diff["after"]["role"].is_null());
    }

    #[test]
    fn agent_profile_updated_audit_class_is_logged_by_default() {
        let patch = AgentProfilePatchDiff {
            display_name: Some(("old".into(), "new".into())),
            ..Default::default()
        };
        assert_eq!(patch.audit_class(), AuditClass::Logged);

        let ev = agent_profile_updated(
            AgentId::new(),
            AgentId::new(),
            OrgId::new(),
            patch,
            None,
            Utc::now(),
        );
        assert_eq!(ev.audit_class, AuditClass::Logged);
    }

    #[test]
    fn agent_profile_updated_audit_class_is_alerted_on_role_change() {
        let patch = AgentProfilePatchDiff {
            role: Some((Some(AgentRole::Intern), Some(AgentRole::Contract))),
            ..Default::default()
        };
        assert_eq!(patch.audit_class(), AuditClass::Alerted);
    }

    #[test]
    fn agent_profile_updated_audit_class_is_alerted_on_model_change() {
        let patch = AgentProfilePatchDiff {
            model_config_id: Some((Some("gpt-4".into()), Some("claude-3".into()))),
            ..Default::default()
        };
        assert_eq!(patch.audit_class(), AuditClass::Alerted);
    }

    #[test]
    fn patch_is_empty_when_every_field_is_none() {
        let patch = AgentProfilePatchDiff::default();
        assert!(patch.is_empty());
    }
}
