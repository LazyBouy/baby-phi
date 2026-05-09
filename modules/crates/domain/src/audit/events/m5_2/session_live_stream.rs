//! Audit-event builder for the SSE live-event tail hard-deny path
//! (CH-17 / ADR-0055 §D55.7). One event:
//!
//! - `platform.session.live_stream_denied` (Alerted) — emitted by
//!   `server::platform::sessions::events::open_live_stream` whenever
//!   the SSE-time Permission Check returns
//!   `Decision::Denied { failed_step, reason }` for any step 0..6.
//!   The handler emits this BEFORE returning
//!   `Err(SessionError::PermissionCheckFailed { step, reason })`.
//!
//! Mirrors `session_launch_denied` (CH-15 / ADR-0054 §D54.5) verbatim
//! in shape — operators reading the audit log can join across both
//! events on `session_id` to reconstruct a full launch + observe
//! deny trace.
//!
//! ## Concept-doc cross-reference
//!
//! `permissions/04-manifest-and-resolution.md` §"Permission Check
//! (Runtime Reconciliation)" — every Decision::Denied at the
//! security boundary requires an audit trail (invariant 5: "audit
//! trail on every outcome"). `permissions/07-templates-and-tools.md`
//! §"audit_class composition" — failed permission checks default to
//! `Alerted`.
//!
//! ## Hash-chain symmetry
//!
//! The builder produces an [`AuditEvent`] whose `canonical_bytes`
//! excludes `prev_event_hash` (per
//! [`AuditEvent::canonical_bytes`]). Additive event types do not
//! perturb prior events' canonical bytes — see CH-12 plan §3.B A7.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, NodeId, OrgId, ProjectId, SessionId};

/// `platform.session.live_stream_denied` — Alerted class (high-tier
/// retention, 60s delivery to org alert channel per
/// `nfr-observability.md`).
///
/// Emitted by `open_live_stream` whenever the SSE-time engine call
/// returns `Decision::Denied { failed_step, reason }`. The
/// `failed_step` numeric is the same `as_metric_label()` projection
/// used by `SessionError::PermissionCheckFailed { step }` (0..6) so
/// dashboards can join on it.
#[allow(clippy::too_many_arguments)]
pub fn session_live_stream_denied(
    actor: AgentId,
    session_id: SessionId,
    agent_id: AgentId,
    project_id: ProjectId,
    org_id: OrgId,
    failed_step: u8,
    reason_kind: &str,
    reason_detail: Option<serde_json::Value>,
    emitted_at: DateTime<Utc>,
) -> AuditEvent {
    let mut after = serde_json::json!({
        "session_id":   session_id.to_string(),
        "agent_id":     agent_id.to_string(),
        "project_id":   project_id.to_string(),
        "org_id":       org_id.to_string(),
        "failed_step":  failed_step,
        "reason_kind":  reason_kind,
        "emitted_at":   emitted_at.to_rfc3339(),
    });
    if let Some(detail) = reason_detail {
        if let Some(obj) = after.as_object_mut() {
            obj.insert("reason_detail".to_string(), detail);
        }
    }
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.session.live_stream_denied".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*session_id.as_uuid())),
        timestamp: emitted_at,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after":  after,
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: None,
        org_scope: Some(org_id),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp")
    }

    /// CH-17 / ADR-0055 §D55.7 — basic shape: event_type +
    /// audit_class + org_scope + step + reason_kind all flow through.
    #[test]
    fn live_stream_denied_carries_alerted_class_and_org_scope() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let agent = AgentId::new();
        let project = ProjectId::new();
        let org = OrgId::new();
        let evt = session_live_stream_denied(
            actor,
            session,
            agent,
            project,
            org,
            3,
            "no_matching_grant",
            None,
            fixed_timestamp(),
        );

        assert_eq!(evt.event_type, "platform.session.live_stream_denied");
        assert_eq!(evt.audit_class, AuditClass::Alerted);
        assert_eq!(evt.org_scope, Some(org));
        assert_eq!(evt.actor_agent_id, Some(actor));
        assert_eq!(
            evt.target_entity_id,
            Some(NodeId::from_uuid(*session.as_uuid()))
        );
        assert_eq!(evt.diff["after"]["failed_step"].as_u64(), Some(3));
        assert_eq!(
            evt.diff["after"]["reason_kind"].as_str(),
            Some("no_matching_grant")
        );
        // No reason_detail key when the parameter is None.
        assert!(evt.diff["after"].get("reason_detail").is_none());
    }

    /// CH-17 / ADR-0055 §D55.7 — `reason_detail: Some(...)` populates
    /// the `reason_detail` key in the diff so dashboards can join on
    /// the human-readable summary.
    #[test]
    fn live_stream_denied_carries_reason_detail_when_supplied() {
        let evt = session_live_stream_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            5,
            "scope_unresolvable",
            Some(serde_json::json!("project tag missing on grant")),
            fixed_timestamp(),
        );
        assert_eq!(evt.diff["after"]["failed_step"].as_u64(), Some(5));
        assert_eq!(
            evt.diff["after"]["reason_kind"].as_str(),
            Some("scope_unresolvable")
        );
        assert_eq!(
            evt.diff["after"]["reason_detail"].as_str(),
            Some("project tag missing on grant")
        );
    }

    /// CH-17 / ADR-0055 §D55.7 — `prev_event_hash` is None on the
    /// builder return; the AuditEmitter chain-link wrap fills the
    /// hash at emit time. Pinpoints that the builder does NOT
    /// pre-populate the chain link (single-writer guarantee).
    #[test]
    fn live_stream_denied_leaves_prev_event_hash_unset() {
        let evt = session_live_stream_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            0,
            "catalogue_miss",
            None,
            fixed_timestamp(),
        );
        assert_eq!(
            evt.prev_event_hash, None,
            "builder leaves chain-link wrap to the emitter"
        );
    }

    /// CH-17 — actor + agent are decoupled (operator-on-behalf-of-self
    /// is the most common case but not the only). The builder accepts
    /// distinct ids and records both.
    #[test]
    fn live_stream_denied_records_distinct_actor_and_agent_ids() {
        let actor = AgentId::new();
        let agent = AgentId::new();
        assert_ne!(actor, agent, "AgentId::new() yields distinct uuids");
        let evt = session_live_stream_denied(
            actor,
            SessionId::new(),
            agent,
            ProjectId::new(),
            OrgId::new(),
            2,
            "no_grants_held",
            None,
            fixed_timestamp(),
        );
        assert_eq!(evt.actor_agent_id, Some(actor));
        // The agent_id is recorded inside the diff payload so the
        // audit row keeps both axes of identity.
        let after = &evt.diff["after"];
        assert!(
            after.get("agent_id").is_some(),
            "agent_id is recorded in the diff alongside actor_agent_id"
        );
    }
}
