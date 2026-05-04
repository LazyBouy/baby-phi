//! Audit-event builders for tool-authority security events
//! (CH-12 / ADR-0049 §D49.7). One event today:
//!
//! - `tool.frozen_tag_write_rejected` — Alerted class. Emitted when a
//!   future tag-write Repository method's call to
//!   [`crate::permissions::manifest::validator::validate_tag_write_on_session`]
//!   returns `Err(FrozenTagViolation)`. Today no production callsite
//!   emits this event; the builder is forward-defensive symmetric to
//!   `validate_tag_write_on_session`. The Repository trait docstring
//!   documents the pairing contract: validator + audit-event are paired
//!   at any future tag-write callsite.
//!
//! Concept doc cross-ref: `permissions/05-memory-sessions.md`
//! Example 7 lines 516–525 ("worker tries to retag... Denied. The
//! request never reaches the storage layer"). The audit event makes
//! the rejection operator-visible at Alerted retention tier (60s
//! delivery to org alert channel per `nfr-observability.md`).
//!
//! Hash-chain symmetry: the builder produces an [`AuditEvent`] whose
//! `canonical_bytes` excludes `prev_event_hash` (per
//! [`AuditEvent::canonical_bytes`]). Additive event types do not
//! perturb prior events' canonical bytes — see CH-12 plan §3.B A7.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, NodeId, OrgId, SessionId};
use crate::permissions::manifest::validator::FrozenTagViolation;

/// `tool.frozen_tag_write_rejected` — Alerted class (high-tier retention,
/// 60s delivery to org alert channel per `nfr-observability.md`).
///
/// Emitted by any future Repository method that proposes to update a
/// `Session`'s tag set when its precondition call to
/// [`crate::permissions::manifest::validator::validate_tag_write_on_session`]
/// returns `Err(FrozenTagViolation)`. The actual `audit.emit(...)`
/// callsite lands in a future chunk that wires `update_session_tags`;
/// this builder ships forward-defensive at CH-12.
///
/// ## Diff shape
///
/// ```json
/// {
///   "before": null,
///   "after": {
///     "session_id":     "<uuid>",
///     "violation_kind": "frozen_tag_added" | "frozen_tag_removed",
///     "tag":            "<offending-tag>",
///     "attempted_at":   "<rfc3339-timestamp>"
///   }
/// }
/// ```
///
/// Mirrors `consent.requested`'s "before:null / after:full-payload"
/// shape for create-style rejection events.
pub fn frozen_tag_write_rejected(
    actor: AgentId,
    target_session: SessionId,
    org: OrgId,
    violation: &FrozenTagViolation,
    attempted_at: DateTime<Utc>,
) -> AuditEvent {
    let (kind, tag) = match violation {
        FrozenTagViolation::FrozenTagAdded { tag, .. } => ("frozen_tag_added", tag.clone()),
        FrozenTagViolation::FrozenTagRemoved { tag, .. } => ("frozen_tag_removed", tag.clone()),
    };
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "tool.frozen_tag_write_rejected".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*target_session.as_uuid())),
        timestamp: attempted_at,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "session_id":     target_session.to_string(),
                "violation_kind": kind,
                "tag":            tag,
                "attempted_at":   attempted_at.to_rfc3339(),
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, OrgId, SessionId};

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp")
    }

    #[test]
    fn frozen_tag_write_rejected_added_carries_alerted_class_and_org_scope() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let org = OrgId::new();
        let v = FrozenTagViolation::FrozenTagAdded {
            session_id: session,
            tag: "session:other".to_string(),
        };
        let evt = frozen_tag_write_rejected(actor, session, org, &v, fixed_timestamp());

        assert_eq!(evt.event_type, "tool.frozen_tag_write_rejected");
        assert_eq!(evt.audit_class, AuditClass::Alerted);
        assert_eq!(evt.org_scope, Some(org));
        assert_eq!(evt.actor_agent_id, Some(actor));
        assert_eq!(
            evt.target_entity_id,
            Some(NodeId::from_uuid(*session.as_uuid()))
        );
        assert_eq!(
            evt.diff["after"]["violation_kind"].as_str().unwrap(),
            "frozen_tag_added"
        );
        assert_eq!(evt.diff["after"]["tag"].as_str().unwrap(), "session:other");
        assert_eq!(
            evt.diff["after"]["session_id"].as_str().unwrap(),
            session.to_string()
        );
        assert!(evt.diff["before"].is_null());
        assert!(evt.provenance_auth_request_id.is_none());
        assert!(evt.prev_event_hash.is_none());
    }

    #[test]
    fn frozen_tag_write_rejected_removed_branch() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let org = OrgId::new();
        let v = FrozenTagViolation::FrozenTagRemoved {
            session_id: session,
            tag: "#kind:session".to_string(),
        };
        let evt = frozen_tag_write_rejected(actor, session, org, &v, fixed_timestamp());
        assert_eq!(evt.event_type, "tool.frozen_tag_write_rejected");
        assert_eq!(evt.audit_class, AuditClass::Alerted);
        assert_eq!(
            evt.diff["after"]["violation_kind"].as_str().unwrap(),
            "frozen_tag_removed"
        );
        assert_eq!(evt.diff["after"]["tag"].as_str().unwrap(), "#kind:session");
    }

    #[test]
    fn frozen_tag_write_rejected_diff_carries_attempted_at_rfc3339() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let org = OrgId::new();
        let v = FrozenTagViolation::FrozenTagAdded {
            session_id: session,
            tag: "org:Y".to_string(),
        };
        let ts = fixed_timestamp();
        let evt = frozen_tag_write_rejected(actor, session, org, &v, ts);
        let attempted = evt.diff["after"]["attempted_at"]
            .as_str()
            .expect("attempted_at present as string");
        let parsed = chrono::DateTime::parse_from_rfc3339(attempted)
            .expect("attempted_at parses as RFC3339");
        assert_eq!(parsed.with_timezone(&Utc), ts);
    }

    #[test]
    fn frozen_tag_write_rejected_canonical_bytes_byte_stable_for_identical_inputs() {
        // Builder must produce events whose `canonical_bytes()` is
        // byte-equal when given identical inputs (modulo the random
        // `event_id` field, which is part of canonical_bytes by design).
        // This pins the F5.B byte-stability claim from CH-12 plan §3.B
        // A7: additive event types do not introduce non-determinism
        // beyond what's already inherent in `event_id`.
        let actor = AgentId::new();
        let session = SessionId::new();
        let org = OrgId::new();
        let ts = fixed_timestamp();
        let v = FrozenTagViolation::FrozenTagAdded {
            session_id: session,
            tag: "agent:rogue".to_string(),
        };
        let mut evt_a = frozen_tag_write_rejected(actor, session, org, &v, ts);
        let mut evt_b = frozen_tag_write_rejected(actor, session, org, &v, ts);
        // Force the same event_id so canonical_bytes only compare the
        // deterministic surface (event_id is the only random field by
        // design; its inclusion in canonical_bytes is intentional per
        // `audit/mod.rs::canonical_bytes_excludes_prev_event_hash`).
        evt_a.event_id = evt_b.event_id;
        // Re-run the existing CH-21 invariant: the prev_event_hash
        // field must NOT affect canonical_bytes.
        evt_a.prev_event_hash = None;
        evt_b.prev_event_hash = Some([7u8; 32]);
        assert_eq!(
            evt_a.canonical_bytes(),
            evt_b.canonical_bytes(),
            "F5.B: prev_event_hash must not affect canonical bytes for the new event_type"
        );
    }
}
