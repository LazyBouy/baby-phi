//! Audit-event builders for `MANAGES` + `HAS_AGENT_SUPERVISOR` edge
//! creation (CH-23 / ADR-0046 §D46.5).
//!
//! These are the **handler-side** audits: emitted by the HTTP
//! compound-tx that persists the edge row, BEFORE the
//! `DomainEvent::ManagesEdgeCreated` / `HasAgentSupervisorEdgeCreated`
//! is fanned out to listeners. The downstream
//! `template.c.grant_fired` / `template.d.grant_fired` audits at
//! [`super::templates`] are the listener-side audits emitted after
//! grant minting.
//!
//! Both events are `Logged` class — routine relationship-mutation
//! traffic; `Alerted` would flood the audit chain on every
//! manager / supervisor assignment.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, EdgeId, NodeId, OrgId, ProjectId};

/// `platform.manages.edge.created` — Logged.
///
/// Diff captures `(manager, subordinate)` plus the edge id so a
/// reviewer can pivot from the audit row to the underlying graph
/// edge without a join.
pub fn manages_edge_created(
    actor: AgentId,
    org: OrgId,
    manager: AgentId,
    subordinate: AgentId,
    edge_id: EdgeId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "manager_agent_id":     manager.to_string(),
            "subordinate_agent_id": subordinate.to_string(),
            "edge_id":              edge_id.to_string(),
            "edge_kind":            "MANAGES",
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.manages.edge.created".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*edge_id.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `platform.has_agent_supervisor.edge.created` — Logged.
///
/// Project-scoped supervision edge counterpart of
/// [`manages_edge_created`]. Diff carries `project_id` so the reader
/// can scope the relationship to its project without a follow-up
/// query.
pub fn has_agent_supervisor_edge_created(
    actor: AgentId,
    org: OrgId,
    project: ProjectId,
    supervisor: AgentId,
    supervisee: AgentId,
    edge_id: EdgeId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "project_id":           project.to_string(),
            "supervisor_agent_id":  supervisor.to_string(),
            "supervisee_agent_id":  supervisee.to_string(),
            "edge_id":              edge_id.to_string(),
            "edge_kind":            "HAS_AGENT_SUPERVISOR",
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.has_agent_supervisor.edge.created".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*edge_id.as_uuid())),
        timestamp,
        diff,
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
    fn manages_edge_created_is_logged_and_org_scoped() {
        let org = OrgId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();
        let edge = EdgeId::new();
        let ev = manages_edge_created(AgentId::new(), org, manager, subordinate, edge, Utc::now());
        assert_eq!(ev.event_type, "platform.manages.edge.created");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.diff["after"]["manager_agent_id"], manager.to_string());
        assert_eq!(
            ev.diff["after"]["subordinate_agent_id"],
            subordinate.to_string()
        );
        assert_eq!(ev.diff["after"]["edge_kind"], "MANAGES");
    }

    #[test]
    fn has_agent_supervisor_edge_created_carries_project_scope() {
        let project = ProjectId::new();
        let supervisor = AgentId::new();
        let supervisee = AgentId::new();
        let edge = EdgeId::new();
        let ev = has_agent_supervisor_edge_created(
            AgentId::new(),
            OrgId::new(),
            project,
            supervisor,
            supervisee,
            edge,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.has_agent_supervisor.edge.created");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.diff["after"]["project_id"], project.to_string());
        assert_eq!(ev.diff["after"]["edge_kind"], "HAS_AGENT_SUPERVISOR");
    }

    #[test]
    fn target_entity_id_is_the_edge_node() {
        let edge = EdgeId::new();
        let ev = manages_edge_created(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            AgentId::new(),
            edge,
            Utc::now(),
        );
        assert_eq!(
            ev.target_entity_id,
            Some(NodeId::from_uuid(*edge.as_uuid()))
        );
    }
}
