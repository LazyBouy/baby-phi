//! Audit-event builder for `AgentCatalogListener` debug-mode fires
//! (M5 / CH-22 / ADR-0035).
//!
//! Emitted only when the listener is constructed with
//! [`crate::events::listeners::CatalogAuditMode::Debug`]. The default
//! mode is `Silent` so production audit logs are not flooded by the
//! listener's high-volume fires (see ADR-0035 §Context).
//!
//! Audit class: `Silent` — debug-only retention. The listener-mode
//! flag is the primary gate; the per-event class keeps the entry in
//! the lowest retention tier so even when an operator flips the flag
//! on a busy org the trail expires after the standard Silent window.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, NodeId, OrgId};

/// `agent_catalog_refreshed` — Silent.
///
/// Emitted by the agent-catalog listener after upserting an
/// `agent_catalog_entry` row in response to a triggering `DomainEvent`.
/// The diff names the refreshed agent + the event kind that triggered
/// the refresh + the triggering event's id (so the listener fire is
/// joinable against the originating audit row).
pub fn agent_catalog_refreshed(
    actor: AgentId,
    org: OrgId,
    refreshed_agent: AgentId,
    triggering_event_id: AuditEventId,
    triggering_event_kind: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "refreshed_agent_id":     refreshed_agent.to_string(),
            "triggering_event_id":    triggering_event_id.to_string(),
            "triggering_event_kind":  triggering_event_kind,
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "agent_catalog_refreshed".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*refreshed_agent.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Silent,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_catalog_refreshed_has_silent_class_and_expected_shape() {
        let actor = AgentId::new();
        let org = OrgId::new();
        let refreshed = AgentId::new();
        let trigger = AuditEventId::new();
        let now = Utc::now();

        let evt = agent_catalog_refreshed(actor, org, refreshed, trigger, "agent_created", now);

        assert_eq!(evt.event_type, "agent_catalog_refreshed");
        assert_eq!(evt.audit_class, AuditClass::Silent);
        assert_eq!(evt.actor_agent_id, Some(actor));
        assert_eq!(evt.org_scope, Some(org));
        assert_eq!(evt.provenance_auth_request_id, None);
        assert_eq!(evt.diff["after"]["triggering_event_kind"], "agent_created");
        assert_eq!(
            evt.diff["after"]["refreshed_agent_id"],
            refreshed.to_string()
        );
        assert_eq!(
            evt.diff["after"]["triggering_event_id"],
            trigger.to_string()
        );
    }
}
