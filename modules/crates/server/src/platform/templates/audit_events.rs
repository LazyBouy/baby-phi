//! Audit-event builders for page 12 actions (R-ADMIN-12-N1/N2/N3).
//!
//! Four event types land at M5/P5:
//! - `platform.template.adopted` (Alerted) — emitted by approve +
//!   adopt (inline). Approve distinguishes via the `mode` field
//!   on the diff.
//! - `platform.template.adoption_denied` (Alerted) — emitted by
//!   deny.
//! - `platform.template.revoked` (Alerted) — emitted by revoke;
//!   carries `grant_count_revoked` in the diff per R-ADMIN-12-N3.
//! - Per-grant template-fire events are emitted by the M5/P3
//!   fire listeners ([`template_a_grant_fired`] + sibs) — not
//!   re-introduced here.

use chrono::{DateTime, Utc};
use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};
use domain::model::nodes::TemplateKind;

pub fn template_adopted_approved(
    actor: AgentId,
    org: OrgId,
    kind: TemplateKind,
    adoption_ar: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.template.adopted".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*adoption_ar.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "template_kind": kind.as_str(),
                "mode": "approve_existing",
                "adoption_auth_request_id": adoption_ar.to_string(),
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(adoption_ar),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

pub fn template_adopted_inline(
    actor: AgentId,
    org: OrgId,
    kind: TemplateKind,
    adoption_ar: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.template.adopted".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*adoption_ar.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "template_kind": kind.as_str(),
                "mode": "adopt_inline",
                "adoption_auth_request_id": adoption_ar.to_string(),
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(adoption_ar),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

pub fn template_adoption_denied(
    actor: AgentId,
    org: OrgId,
    kind: TemplateKind,
    adoption_ar: AuthRequestId,
    reason: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.template.adoption_denied".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*adoption_ar.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "template_kind": kind.as_str(),
                "adoption_auth_request_id": adoption_ar.to_string(),
                "reason": reason,
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(adoption_ar),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn template_revoked(
    actor: AgentId,
    org: OrgId,
    kind: TemplateKind,
    adoption_ar: AuthRequestId,
    grant_count_revoked: u32,
    reason: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.template.revoked".into(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*adoption_ar.as_uuid())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "template_kind": kind.as_str(),
                "adoption_auth_request_id": adoption_ar.to_string(),
                "grant_count_revoked": grant_count_revoked,
                "reason": reason,
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(adoption_ar),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adopted_approved_has_stable_event_type() {
        let e = template_adopted_approved(
            AgentId::new(),
            OrgId::new(),
            TemplateKind::A,
            AuthRequestId::new(),
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.template.adopted");
        assert_eq!(e.audit_class, AuditClass::Alerted);
        assert_eq!(e.diff["after"]["mode"], "approve_existing");
    }

    #[test]
    fn adopted_inline_has_adopt_mode() {
        let e = template_adopted_inline(
            AgentId::new(),
            OrgId::new(),
            TemplateKind::C,
            AuthRequestId::new(),
            Utc::now(),
        );
        assert_eq!(e.diff["after"]["mode"], "adopt_inline");
    }

    #[test]
    fn revoked_carries_grant_count() {
        let e = template_revoked(
            AgentId::new(),
            OrgId::new(),
            TemplateKind::A,
            AuthRequestId::new(),
            7,
            "reorg",
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.template.revoked");
        assert_eq!(e.diff["after"]["grant_count_revoked"], 7);
        assert_eq!(e.diff["after"]["reason"], "reorg");
    }

    #[test]
    fn denied_carries_reason() {
        let e = template_adoption_denied(
            AgentId::new(),
            OrgId::new(),
            TemplateKind::B,
            AuthRequestId::new(),
            "policy mismatch",
            Utc::now(),
        );
        assert_eq!(e.event_type, "platform.template.adoption_denied");
        assert_eq!(e.diff["after"]["reason"], "policy mismatch");
    }
}
