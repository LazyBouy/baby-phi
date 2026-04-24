//! Audit-event builders for page 13 actions (R-ADMIN-13-N1/N2).
//!
//! Four event types land at M5/P6:
//! - `platform.system_agent.reconfigured` (Alerted for standard,
//!   Logged for org-specific — resolved inline by the tune handler
//!   based on the agent's profile_ref).
//! - `platform.system_agent.added` (Logged) — new org-specific
//!   system agent provisioned.
//! - `platform.system_agent.disabled` (Alerted) — `was_standard`
//!   flag carried on the diff per R-ADMIN-13-N1 alerted convention.
//! - `platform.system_agent.archived` (Logged).

use chrono::{DateTime, Utc};
use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId};

use super::add::SystemAgentTrigger;

pub fn system_agent_reconfigured(
    actor: AgentId,
    org: OrgId,
    agent: AgentId,
    parallelize_before: Option<u32>,
    parallelize_after: Option<u32>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.system_agent.reconfigured".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*agent.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": {
                "parallelize": parallelize_before,
            },
            "after": {
                "parallelize": parallelize_after,
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn system_agent_added(
    actor: AgentId,
    org: OrgId,
    agent: AgentId,
    profile_ref: &str,
    parallelize: u32,
    trigger: SystemAgentTrigger,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let trigger_slug = match trigger {
        SystemAgentTrigger::SessionEnd => "session_end",
        SystemAgentTrigger::EdgeChange => "edge_change",
        SystemAgentTrigger::Periodic => "periodic",
        SystemAgentTrigger::Explicit => "explicit",
        SystemAgentTrigger::CustomEvent => "custom_event",
    };
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.system_agent.added".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*agent.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "profile_ref": profile_ref,
                "parallelize": parallelize,
                "trigger": trigger_slug,
            },
        }),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

pub fn system_agent_disabled(
    actor: AgentId,
    org: OrgId,
    agent: AgentId,
    was_standard: bool,
    profile_ref: Option<&str>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.system_agent.disabled".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*agent.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "was_standard": was_standard,
                "profile_ref": profile_ref,
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

pub fn system_agent_archived(
    actor: AgentId,
    org: OrgId,
    agent: AgentId,
    profile_ref: Option<&str>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.system_agent.archived".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*agent.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "profile_ref": profile_ref,
            },
        }),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconfigured_carries_before_after() {
        let e = system_agent_reconfigured(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            Some(1),
            Some(4),
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.system_agent.reconfigured");
        assert_eq!(e.audit_class, AuditClass::Alerted);
        assert_eq!(e.diff["before"]["parallelize"], 1);
        assert_eq!(e.diff["after"]["parallelize"], 4);
    }

    #[test]
    fn added_carries_trigger_slug() {
        let e = system_agent_added(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            "compliance-audit",
            2,
            SystemAgentTrigger::SessionEnd,
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.system_agent.added");
        assert_eq!(e.audit_class, AuditClass::Logged);
        assert_eq!(e.diff["after"]["trigger"], "session_end");
        assert_eq!(e.diff["after"]["profile_ref"], "compliance-audit");
        assert_eq!(e.diff["after"]["parallelize"], 2);
    }

    #[test]
    fn disabled_carries_standard_flag() {
        let e = system_agent_disabled(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            true,
            Some("system-memory-extraction"),
            Utc::now(),
        );
        assert_eq!(e.diff["after"]["was_standard"], true);
        assert_eq!(e.audit_class, AuditClass::Alerted);
    }

    #[test]
    fn archived_is_logged() {
        let e = system_agent_archived(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            Some("grading-agent"),
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.system_agent.archived");
        assert_eq!(e.audit_class, AuditClass::Logged);
    }
}
