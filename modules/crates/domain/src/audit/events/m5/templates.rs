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
use crate::permissions::AuditClassSource;

/// `template.c.grant_fired` — strictest-wins composed audit class
/// (CH-13 / ADR-0050). Pre-CH-13 the class was hardcoded `Logged`;
/// the parameter eliminates the silent-downgrade path concept-doc
/// 07 line 71 forbids.
///
/// Diff captures the firing triple (manager, subordinate, grant_id)
/// so a reviewer can trace "which manager got which grant under
/// which adoption AR" in one log line, plus the `audit_class_source`
/// attribution so operators see which input (org_default /
/// template_ar / override) supplied the winning class.
#[allow(clippy::too_many_arguments)]
pub fn template_c_grant_fired(
    actor: AgentId,
    org: OrgId,
    manager: AgentId,
    subordinate: AgentId,
    grant: GrantId,
    adoption_auth_request_id: AuthRequestId,
    audit_class: AuditClass,
    audit_class_source: AuditClassSource,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let source_str = serde_json::to_value(audit_class_source)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string());
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "manager_agent_id":         manager.to_string(),
            "subordinate_agent_id":     subordinate.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            "actions":                  ["read", "inspect"],
            "audit_class_source":       source_str,
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.c.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id: Some(adoption_auth_request_id),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `template.d.grant_fired` — strictest-wins composed audit class
/// (CH-13 / ADR-0050).
///
/// Project-scoped Template D counterpart of
/// [`template_c_grant_fired`]. Diff includes `project_id` so the
/// reader knows the grant is not cross-project, plus the
/// `audit_class_source` attribution.
#[allow(clippy::too_many_arguments)]
pub fn template_d_grant_fired(
    actor: AgentId,
    org: OrgId,
    project: ProjectId,
    supervisor: AgentId,
    supervisee: AgentId,
    grant: GrantId,
    adoption_auth_request_id: AuthRequestId,
    audit_class: AuditClass,
    audit_class_source: AuditClassSource,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let source_str = serde_json::to_value(audit_class_source)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string());
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "project_id":               project.to_string(),
            "supervisor_agent_id":      supervisor.to_string(),
            "supervisee_agent_id":      supervisee.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            "actions":                  ["read", "inspect"],
            "audit_class_source":       source_str,
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.d.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id: Some(adoption_auth_request_id),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_c_grant_fired_passes_through_composed_class_and_source() {
        let org = OrgId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();
        let grant = GrantId::new();
        let ar = AuthRequestId::new();
        // CH-13: builder no longer hardcodes `Logged`. Confirm the
        // composed `(class, source)` round-trips into the event +
        // diff.
        let ev = template_c_grant_fired(
            AgentId::new(),
            org,
            manager,
            subordinate,
            grant,
            ar,
            AuditClass::Alerted,
            AuditClassSource::TemplateAr,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "template.c.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.provenance_auth_request_id, Some(ar));
        assert_eq!(ev.diff["after"]["manager_agent_id"], manager.to_string());
        assert_eq!(
            ev.diff["after"]["subordinate_agent_id"],
            subordinate.to_string()
        );
        assert_eq!(ev.diff["after"]["audit_class_source"], "template_ar");
        assert_eq!(
            ev.diff["after"]["actions"]
                .as_array()
                .expect("actions is array")
                .len(),
            2
        );
    }

    #[test]
    fn template_d_grant_fired_carries_project_scope_and_composed_class() {
        let project = ProjectId::new();
        // CH-13: builder no longer hardcodes `Logged`. Confirm the
        // composed `(class, source)` round-trips for the
        // project-scoped variant too.
        let ev = template_d_grant_fired(
            AgentId::new(),
            OrgId::new(),
            project,
            AgentId::new(),
            AgentId::new(),
            GrantId::new(),
            AuthRequestId::new(),
            AuditClass::Logged,
            AuditClassSource::OrgDefault,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "template.d.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.diff["after"]["project_id"], project.to_string());
        assert_eq!(ev.diff["after"]["audit_class_source"], "org_default");
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
            AuditClass::Logged,
            AuditClassSource::TemplateAr,
            Utc::now(),
        );
        assert_eq!(
            ev.target_entity_id,
            Some(NodeId::from_uuid(*grant.as_uuid()))
        );
    }
}
