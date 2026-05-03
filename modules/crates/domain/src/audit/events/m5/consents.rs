//! Audit-event builders for Consent state transitions (CH-10 / ADR-0047 §D47.4).
//!
//! Six `Logged`-class builders:
//!
//! - `consent.requested` — initial mint (CH-11 / ADR-0048 §D48.10) —
//!   emitted by `Repository::request_consent` when the engine surfaces
//!   `Pending` for the first time on a `(subordinate, org, session?)`
//!   triple.
//! - `consent.acknowledged` — `Requested → Acknowledged`
//! - `consent.declined` — `Requested → Declined`
//! - `consent.revoked` — `Acknowledged → Revoked`
//! - `consent.timed_out` — `Requested → TimedOut` (sweeper-driven)
//! - `consent.expired` — any non-terminal → `Expired` (natural-scope-end)
//!
//! All emit `AuditClass::Logged` — routine relationship-mutation traffic;
//! `Alerted` would flood the audit chain on every consent flip. The diff
//! captures `(consent_id, from_state, to_state, at)` plus the field
//! mutated by the transition (`responded_at` / `revoked_at` / nothing).
//! For `consent.requested` there is no source state, so the diff carries
//! `(consent_id, subordinate, org, session_id, deadline_at, requested_at)`
//! under `after` only. `org_scope` populated from `consent.scope.org`.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, ConsentId, NodeId, OrgId, SessionId};
use crate::model::nodes::ConsentState;

#[allow(clippy::too_many_arguments)]
fn build_consent_audit(
    event_type: &'static str,
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    from_state: ConsentState,
    to_state: ConsentState,
    at: DateTime<Utc>,
    extra_after: Option<(&'static str, serde_json::Value)>,
) -> AuditEvent {
    let mut after = serde_json::json!({
        "consent_id": consent_id.to_string(),
        "from_state": serde_json::to_value(from_state).expect("ConsentState serialises"),
        "to_state":   serde_json::to_value(to_state).expect("ConsentState serialises"),
        "at":         at.to_rfc3339(),
    });
    if let Some((k, v)) = extra_after {
        after[k] = v;
    }
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*consent_id.as_uuid())),
        timestamp: at,
        diff: serde_json::json!({
            "before": { "state": serde_json::to_value(from_state).expect("ConsentState serialises") },
            "after":  after,
        }),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `consent.acknowledged` — `Requested → Acknowledged`.
pub fn consent_acknowledged(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    at: DateTime<Utc>,
) -> AuditEvent {
    build_consent_audit(
        "consent.acknowledged",
        actor,
        org,
        consent_id,
        ConsentState::Requested,
        ConsentState::Acknowledged,
        at,
        Some(("responded_at", serde_json::Value::String(at.to_rfc3339()))),
    )
}

/// `consent.declined` — `Requested → Declined`.
pub fn consent_declined(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    at: DateTime<Utc>,
) -> AuditEvent {
    build_consent_audit(
        "consent.declined",
        actor,
        org,
        consent_id,
        ConsentState::Requested,
        ConsentState::Declined,
        at,
        Some(("responded_at", serde_json::Value::String(at.to_rfc3339()))),
    )
}

/// `consent.revoked` — `Acknowledged → Revoked` (forward-only).
pub fn consent_revoked(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    at: DateTime<Utc>,
) -> AuditEvent {
    build_consent_audit(
        "consent.revoked",
        actor,
        org,
        consent_id,
        ConsentState::Acknowledged,
        ConsentState::Revoked,
        at,
        Some(("revoked_at", serde_json::Value::String(at.to_rfc3339()))),
    )
}

/// `consent.timed_out` — `Requested → TimedOut`. Emitted by the sweeper.
pub fn consent_timed_out(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    at: DateTime<Utc>,
) -> AuditEvent {
    build_consent_audit(
        "consent.timed_out",
        actor,
        org,
        consent_id,
        ConsentState::Requested,
        ConsentState::TimedOut,
        at,
        None,
    )
}

/// `consent.expired` — any non-terminal → `Expired`. Caller passes the
/// `from_state` that the transition started from, since `Expired` is
/// reachable from both `Requested` and `Acknowledged`.
pub fn consent_expired(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    from_state: ConsentState,
    at: DateTime<Utc>,
) -> AuditEvent {
    build_consent_audit(
        "consent.expired",
        actor,
        org,
        consent_id,
        from_state,
        ConsentState::Expired,
        at,
        None,
    )
}

/// `consent.requested` — initial mint (CH-11 / ADR-0048 §D48.10).
///
/// Emitted by [`crate::repository::Repository::request_consent`] when
/// the engine surfaces `Pending` for the first time on a
/// `(subordinate, org, session?)` triple.
///
/// The diff has no source state — the row is being created. The `after`
/// payload carries the full identifying tuple
/// `(consent_id, subordinate, org, session_id, deadline_at)` plus the
/// `requested_at` timestamp so reviewers can correlate the mint with
/// the sweeper's eventual `consent.timed_out` (if no ack lands in
/// time).
pub fn consent_requested(
    actor: AgentId,
    org: OrgId,
    consent_id: ConsentId,
    subordinate: AgentId,
    session_id: Option<SessionId>,
    deadline_at: Option<DateTime<Utc>>,
    requested_at: DateTime<Utc>,
) -> AuditEvent {
    let after = serde_json::json!({
        "consent_id": consent_id.to_string(),
        "subordinate": subordinate.to_string(),
        "org": org.to_string(),
        "session_id": session_id.map(|s| s.to_string()),
        "deadline_at": deadline_at.map(|d| d.to_rfc3339()),
        "to_state": serde_json::to_value(ConsentState::Requested).expect("ConsentState serialises"),
        "requested_at": requested_at.to_rfc3339(),
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "consent.requested".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*consent_id.as_uuid())),
        timestamp: requested_at,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": after,
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
    fn consent_acknowledged_carries_org_scope_and_responded_at() {
        let org = OrgId::new();
        let cid = ConsentId::new();
        let actor = AgentId::new();
        let at = Utc::now();
        let ev = consent_acknowledged(actor, org, cid, at);
        assert_eq!(ev.event_type, "consent.acknowledged");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.actor_agent_id, Some(actor));
        assert_eq!(ev.diff["after"]["consent_id"], cid.to_string());
        assert_eq!(ev.diff["after"]["from_state"], "requested");
        assert_eq!(ev.diff["after"]["to_state"], "acknowledged");
        assert!(ev.diff["after"]["responded_at"].is_string());
    }

    #[test]
    fn consent_revoked_diff_carries_revoked_at_field() {
        let ev = consent_revoked(AgentId::new(), OrgId::new(), ConsentId::new(), Utc::now());
        assert_eq!(ev.event_type, "consent.revoked");
        assert_eq!(ev.diff["after"]["from_state"], "acknowledged");
        assert_eq!(ev.diff["after"]["to_state"], "revoked");
        assert!(ev.diff["after"]["revoked_at"].is_string());
    }

    #[test]
    fn consent_timed_out_has_no_timestamp_extra_field() {
        let ev = consent_timed_out(AgentId::new(), OrgId::new(), ConsentId::new(), Utc::now());
        assert_eq!(ev.event_type, "consent.timed_out");
        assert_eq!(ev.diff["after"]["from_state"], "requested");
        assert_eq!(ev.diff["after"]["to_state"], "timed_out");
        assert!(ev.diff["after"]["responded_at"].is_null());
        assert!(ev.diff["after"]["revoked_at"].is_null());
    }

    #[test]
    fn consent_expired_carries_caller_supplied_from_state() {
        // From Requested.
        let ev = consent_expired(
            AgentId::new(),
            OrgId::new(),
            ConsentId::new(),
            ConsentState::Requested,
            Utc::now(),
        );
        assert_eq!(ev.diff["after"]["from_state"], "requested");
        assert_eq!(ev.diff["after"]["to_state"], "expired");
        // From Acknowledged.
        let ev = consent_expired(
            AgentId::new(),
            OrgId::new(),
            ConsentId::new(),
            ConsentState::Acknowledged,
            Utc::now(),
        );
        assert_eq!(ev.diff["after"]["from_state"], "acknowledged");
    }

    #[test]
    fn target_entity_id_is_the_consent_node() {
        let cid = ConsentId::new();
        let ev = consent_declined(AgentId::new(), OrgId::new(), cid, Utc::now());
        assert_eq!(ev.target_entity_id, Some(NodeId::from_uuid(*cid.as_uuid())));
    }

    #[test]
    fn consent_requested_carries_full_after_payload_and_no_before_state() {
        let cid = ConsentId::new();
        let actor = AgentId::new();
        let sub = AgentId::new();
        let org = OrgId::new();
        let session = SessionId::new();
        let deadline = Utc::now() + chrono::Duration::hours(24);
        let at = Utc::now();
        let ev = consent_requested(actor, org, cid, sub, Some(session), Some(deadline), at);

        assert_eq!(ev.event_type, "consent.requested");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.actor_agent_id, Some(actor));
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.target_entity_id, Some(NodeId::from_uuid(*cid.as_uuid())));
        // before is explicit null — no source state for a create.
        assert!(ev.diff["before"].is_null());
        let after = &ev.diff["after"];
        assert_eq!(after["consent_id"], cid.to_string());
        assert_eq!(after["subordinate"], sub.to_string());
        assert_eq!(after["org"], org.to_string());
        assert_eq!(after["session_id"], session.to_string());
        assert!(after["deadline_at"].is_string());
        assert_eq!(after["to_state"], "requested");
        assert!(after["requested_at"].is_string());
    }

    #[test]
    fn consent_requested_with_no_session_or_deadline_emits_explicit_nulls() {
        let ev = consent_requested(
            AgentId::new(),
            OrgId::new(),
            ConsentId::new(),
            AgentId::new(),
            None,
            None,
            Utc::now(),
        );
        assert!(ev.diff["after"]["session_id"].is_null());
        assert!(ev.diff["after"]["deadline_at"].is_null());
    }
}
