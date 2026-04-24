//! Audit-event builders for Template C + D firing (M5/P3).
//!
//! Mirrors the M4 `template.a.grant_fired` shape — Logged class
//! (routine auto-issue; Alerted level would flood the audit trail on
//! every manager / supervisor relationship change).
//!
//! Emitted by M5/P3's
//! [`crate::events::listeners::TemplateCFireListener`] /
//! [`crate::events::listeners::TemplateDFireListener`] after
//! persisting the Grant returned by the template's `fire_*` pure-fn.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, NodeId, OrgId, ProjectId};

/// `template.c.grant_fired` — Logged.
///
/// Diff captures the firing triple (manager, subordinate, grant_id)
/// so a reviewer can trace "which manager got which grant under
/// which adoption AR" in one log line.
pub fn template_c_grant_fired(
    actor: AgentId,
    org: OrgId,
    manager: AgentId,
    subordinate: AgentId,
    grant: GrantId,
    adoption_auth_request_id: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "manager_agent_id":         manager.to_string(),
            "subordinate_agent_id":     subordinate.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            "actions":                  ["read", "inspect"],
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.c.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: Some(adoption_auth_request_id),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `template.d.grant_fired` — Logged.
///
/// Project-scoped Template D counterpart of
/// [`template_c_grant_fired`]. Diff includes `project_id` so the
/// reader knows the grant is not cross-project.
#[allow(clippy::too_many_arguments)]
pub fn template_d_grant_fired(
    actor: AgentId,
    org: OrgId,
    project: ProjectId,
    supervisor: AgentId,
    supervisee: AgentId,
    grant: GrantId,
    adoption_auth_request_id: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "project_id":               project.to_string(),
            "supervisor_agent_id":      supervisor.to_string(),
            "supervisee_agent_id":      supervisee.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            "actions":                  ["read", "inspect"],
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.d.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: Some(adoption_auth_request_id),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_c_grant_fired_is_logged_and_org_scoped() {
        let org = OrgId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();
        let grant = GrantId::new();
        let ar = AuthRequestId::new();
        let ev = template_c_grant_fired(
            AgentId::new(),
            org,
            manager,
            subordinate,
            grant,
            ar,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "template.c.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.provenance_auth_request_id, Some(ar));
        assert_eq!(ev.diff["after"]["manager_agent_id"], manager.to_string());
        assert_eq!(
            ev.diff["after"]["subordinate_agent_id"],
            subordinate.to_string()
        );
        assert_eq!(
            ev.diff["after"]["actions"]
                .as_array()
                .expect("actions is array")
                .len(),
            2
        );
    }

    #[test]
    fn template_d_grant_fired_carries_project_scope() {
        let project = ProjectId::new();
        let ev = template_d_grant_fired(
            AgentId::new(),
            OrgId::new(),
            project,
            AgentId::new(),
            AgentId::new(),
            GrantId::new(),
            AuthRequestId::new(),
            Utc::now(),
        );
        assert_eq!(ev.event_type, "template.d.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.diff["after"]["project_id"], project.to_string());
    }

    #[test]
    fn target_entity_id_is_the_grant_node() {
        let grant = GrantId::new();
        let ev = template_c_grant_fired(
            AgentId::new(),
            OrgId::new(),
            AgentId::new(),
            AgentId::new(),
            grant,
            AuthRequestId::new(),
            Utc::now(),
        );
        assert_eq!(
            ev.target_entity_id,
            Some(NodeId::from_uuid(*grant.as_uuid()))
        );
    }
}
