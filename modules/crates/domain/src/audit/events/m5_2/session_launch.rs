//! Audit-event builder for session-launch hard-deny path
//! (CH-15 / ADR-0054 §D54.5). One event:
//!
//! - `platform.session.launch_denied` (Alerted) — emitted by
//!   `server::platform::sessions::launch::launch_session` whenever
//!   the launch-time Permission Check returns
//!   `Decision::Denied { failed_step, reason }` for any step 0..6.
//!   The launch handler emits this BEFORE returning
//!   `Err(SessionError::PermissionCheckFailed { step, reason })`.
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

/// `platform.session.launch_denied` — Alerted class (high-tier
/// retention, 60s delivery to org alert channel per
/// `nfr-observability.md`).
///
/// Emitted by `launch_session` whenever the launch-time engine call
/// returns `Decision::Denied { failed_step, reason }`. The
/// `failed_step` numeric is the same `as_metric_label()` projection
/// used by `SessionError::PermissionCheckFailed { step }` (0..6) so
/// dashboards can join on it.
///
/// ## Diff shape (canonical_bytes contributors marked `*`)
///
/// ```json
/// {
///   "before": null,
///   "after": {
///     "session_id":   "<uuid>",          // *
///     "agent_id":     "<uuid>",          // *
///     "project_id":   "<uuid>",          // *
///     "org_id":       "<uuid>",          // *
///     "failed_step":  0,                  // * (u8, 0..6)
///     "reason_kind":  "no_grants_held",  // * (DeniedReason variant tag, snake_case)
///     "reason_detail": { ... },          // optional, non-canonical (operator data)
///     "emitted_at":   "<rfc3339>"        // *
///   }
/// }
/// ```
///
/// `session_id` is allocated by the launch handler at Step 3.5 even
/// on the deny path so the audit row carries a stable cross-ref to
/// the (would-be) session row + matches the `session_started` event's
/// shape on the happy path.
#[allow(clippy::too_many_arguments)]
pub fn session_launch_denied(
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
        event_type: "platform.session.launch_denied".to_string(),
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

    /// CH-15 / ADR-0054 §D54.5 — basic shape: event_type +
    /// audit_class + org_scope + step + reason_kind all flow through.
    #[test]
    fn launch_denied_carries_alerted_class_and_org_scope() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let agent = AgentId::new();
        let project = ProjectId::new();
        let org = OrgId::new();
        let evt = session_launch_denied(
            actor,
            session,
            agent,
            project,
            org,
            3,
            "no_grants_held",
            None,
            fixed_timestamp(),
        );

        assert_eq!(evt.event_type, "platform.session.launch_denied");
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
            Some("no_grants_held")
        );
        // No reason_detail key when the parameter is None.
        assert!(evt.diff["after"].get("reason_detail").is_none());
    }

    /// CH-15: optional `reason_detail` object lands under
    /// `diff.after.reason_detail` when supplied. The detail field is
    /// non-canonical (excluded from canonical_bytes per the diff
    /// shape's intent).
    #[test]
    fn launch_denied_carries_optional_reason_detail() {
        let detail = serde_json::json!({"missing_grant_uri": "session_object"});
        let evt = session_launch_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            5,
            "scope_resolution_failed",
            Some(detail.clone()),
            fixed_timestamp(),
        );
        assert_eq!(evt.diff["after"]["reason_detail"], detail);
    }

    /// CH-15: `prev_event_hash` does not affect `canonical_bytes` —
    /// pins the F5.B byte-stability claim.
    #[test]
    fn launch_denied_canonical_bytes_excludes_prev_event_hash() {
        let actor = AgentId::new();
        let session = SessionId::new();
        let agent = AgentId::new();
        let project = ProjectId::new();
        let org = OrgId::new();
        let mut a = session_launch_denied(
            actor,
            session,
            agent,
            project,
            org,
            2,
            "manifest_empty",
            None,
            fixed_timestamp(),
        );
        let mut b = session_launch_denied(
            actor,
            session,
            agent,
            project,
            org,
            2,
            "manifest_empty",
            None,
            fixed_timestamp(),
        );
        b.event_id = a.event_id;
        a.prev_event_hash = None;
        b.prev_event_hash = Some([7u8; 32]);
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    }

    /// CH-15: target_entity_id derives from session_id (matches
    /// `platform.session.started`'s convention so dashboards can join
    /// the deny-event with the would-be session row).
    #[test]
    fn launch_denied_target_entity_id_matches_session_id() {
        let session = SessionId::new();
        let evt = session_launch_denied(
            AgentId::new(),
            session,
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            0,
            "catalogue_not_declared",
            None,
            fixed_timestamp(),
        );
        assert_eq!(
            evt.target_entity_id,
            Some(NodeId::from_uuid(*session.as_uuid()))
        );
    }

    /// CH-15: emitted_at is rendered in the diff payload as RFC3339
    /// so dashboards can group by hour without re-parsing.
    #[test]
    fn launch_denied_diff_emitted_at_is_rfc3339() {
        let ts = fixed_timestamp();
        let evt = session_launch_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            6,
            "consent_declined",
            None,
            ts,
        );
        let emitted = evt.diff["after"]["emitted_at"]
            .as_str()
            .expect("emitted_at present");
        let parsed =
            chrono::DateTime::parse_from_rfc3339(emitted).expect("emitted_at parses as RFC3339");
        assert_eq!(parsed.with_timezone(&Utc), ts);
    }

    /// CH-15: diff carries `before: null` matching the create-style
    /// rejection-event convention (mirrors
    /// `tool.frozen_tag_write_rejected` shape).
    #[test]
    fn launch_denied_diff_before_is_null() {
        let evt = session_launch_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            4,
            "constraint_violation",
            None,
            fixed_timestamp(),
        );
        assert!(evt.diff["before"].is_null());
    }

    /// CH-15: provenance_auth_request_id is None — the launch deny
    /// event isn't tied to an AR (the deny is a runtime-side decision
    /// from the engine, not a Template fire).
    #[test]
    fn launch_denied_provenance_auth_request_id_is_none() {
        let evt = session_launch_denied(
            AgentId::new(),
            SessionId::new(),
            AgentId::new(),
            ProjectId::new(),
            OrgId::new(),
            5,
            "scope_unresolvable",
            None,
            fixed_timestamp(),
        );
        assert!(evt.provenance_auth_request_id.is_none());
    }
}
