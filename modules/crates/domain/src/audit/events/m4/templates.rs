//! Audit-event builder for Template A firing on lead assignment (s05).
//!
//! One event type at M4/P2:
//!
//! - `template.a.grant_fired` (Logged) — companion to the Grant
//!   minted by [`crate::templates::a::fire_grant_on_lead_assignment`].
//!   Emitted by the M4/P3 `TemplateAFireListener` after persisting
//!   the Grant. Diff captures the firing triple (project, lead,
//!   grant_id) so a reviewer can trace "which lead got which grant
//!   under which adoption AR" in one log line.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, NodeId, OrgId, ProjectId};

/// `template.a.grant_fired` — Logged (routine auto-issue; Alerted
/// level would flood the audit trail on every lead assignment).
///
/// Emitted by M4/P3's [`crate::events::listeners`] after persisting
/// the Grant returned by
/// [`crate::templates::a::fire_grant_on_lead_assignment`].
pub fn template_a_grant_fired(
    actor: AgentId,
    org: OrgId,
    project: ProjectId,
    lead: AgentId,
    grant: GrantId,
    adoption_auth_request_id: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "project_id":               project.to_string(),
            "lead_agent_id":            lead.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            // Stable `[read, inspect, list]` per
            // `fire_grant_on_lead_assignment`; pinned here so the
            // audit diff matches the grant shape without a second
            // query.
            "actions":                  ["read", "inspect", "list"],
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.a.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Logged,
        // Provenance = the adoption AR, which links every fire back
        // to the CEO's initial self-approval.
        provenance_auth_request_id: Some(adoption_auth_request_id),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_a_grant_fired_is_logged_and_org_scoped() {
        let org = OrgId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();
        let grant = GrantId::new();
        let ar = AuthRequestId::new();
        let ev = template_a_grant_fired(AgentId::new(), org, project, lead, grant, ar, Utc::now());
        assert_eq!(ev.event_type, "template.a.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.provenance_auth_request_id, Some(ar));
        assert_eq!(ev.diff["after"]["project_id"], project.to_string());
        assert_eq!(ev.diff["after"]["lead_agent_id"], lead.to_string());
        assert_eq!(ev.diff["after"]["grant_id"], grant.to_string());
        assert_eq!(
            ev.diff["after"]["actions"]
                .as_array()
                .expect("actions is array")
                .len(),
            3
        );
    }

    #[test]
    fn prev_event_hash_starts_unset() {
        let ev = template_a_grant_fired(
            AgentId::new(),
            OrgId::new(),
            ProjectId::new(),
            AgentId::new(),
            GrantId::new(),
            AuthRequestId::new(),
            Utc::now(),
        );
        assert!(ev.prev_event_hash.is_none());
    }

    #[test]
    fn target_entity_id_is_the_grant_node() {
        let grant = GrantId::new();
        let ev = template_a_grant_fired(
            AgentId::new(),
            OrgId::new(),
            ProjectId::new(),
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
