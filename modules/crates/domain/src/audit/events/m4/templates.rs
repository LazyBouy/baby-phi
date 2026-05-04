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
use crate::permissions::AuditClassSource;

/// `template.a.grant_fired` — strictest-wins composed audit class
/// per CH-13 / ADR-0050 (P2 wires
/// [`crate::permissions::compose_audit_class_with_source`] over
/// (org_default, template_ar, optional_override) and feeds the
/// resolved `(class, source)` tuple here). Pre-CH-13 the class was
/// hardcoded `Logged`; the parameter eliminates the silent-downgrade
/// path concept-doc 07 line 71 forbids.
///
/// Emitted by M4/P3's [`crate::events::listeners`] after persisting
/// the Grant returned by
/// [`crate::templates::a::fire_grant_on_lead_assignment`].
#[allow(clippy::too_many_arguments)]
pub fn template_a_grant_fired(
    actor: AgentId,
    org: OrgId,
    project: ProjectId,
    lead: AgentId,
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
            "lead_agent_id":            lead.to_string(),
            "grant_id":                 grant.to_string(),
            "adoption_auth_request_id": adoption_auth_request_id.to_string(),
            // Stable `[read, inspect, list]` per
            // `fire_grant_on_lead_assignment`; pinned here so the
            // audit diff matches the grant shape without a second
            // query.
            "actions":                  ["read", "inspect", "list"],
            // CH-13 / concept-doc-07 line 69 — operators must see
            // which of (a) org_default / (b) template_ar / (c)
            // override supplied the winning audit class.
            "audit_class_source":       source_str,
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "template.a.grant_fired".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*grant.as_uuid())),
        timestamp,
        diff,
        audit_class,
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
    fn template_a_grant_fired_passes_through_composed_class_and_source() {
        let org = OrgId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();
        let grant = GrantId::new();
        let ar = AuthRequestId::new();
        // CH-13: builder no longer hardcodes `Logged` — caller passes
        // the composed `(class, source)` tuple. Confirm a non-default
        // (Alerted / OrgDefault) round-trips into the event.
        let ev = template_a_grant_fired(
            AgentId::new(),
            org,
            project,
            lead,
            grant,
            ar,
            AuditClass::Alerted,
            AuditClassSource::OrgDefault,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "template.a.grant_fired");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.provenance_auth_request_id, Some(ar));
        assert_eq!(ev.diff["after"]["project_id"], project.to_string());
        assert_eq!(ev.diff["after"]["lead_agent_id"], lead.to_string());
        assert_eq!(ev.diff["after"]["grant_id"], grant.to_string());
        assert_eq!(ev.diff["after"]["audit_class_source"], "org_default");
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
            AuditClass::Logged,
            AuditClassSource::TemplateAr,
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
