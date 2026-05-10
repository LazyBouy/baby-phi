//! Audit-event builder for the AuthRequest per-state ACL hard-deny path
//! (CH-18 / [ADR-0056](../../../../../../docs/specs/v0/implementation/m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.6). One event:
//!
//! - `auth_request.access_denied` (Alerted) — emitted by every AR-mutation
//!   handler whenever
//!   [`crate::auth_requests::access::check_auth_request_access`] returns
//!   `Err(AuthRequestAccessError::*)` against a principal who is not
//!   authorised for the (state × intended_op) cell of concept doc 02's
//!   per-state matrix. The handler emits this BEFORE returning the typed
//!   `*Error::AccessDenied(...)` to the caller.
//!
//! ## Precedent
//!
//! Mirrors the shape of [`super::session_launch`]'s
//! `platform.session.launch_denied` builder (CH-15 / ADR-0054 §D54.5)
//! and `super::tool_authority`'s `frozen_tag_write_rejected` builder
//! (CH-12 / ADR-0049 §D49.4). Both ship an Alerted-class deny event that
//! lives alongside the typed handler-side error.
//!
//! ## Emission frequency
//!
//! Per F3.B.list-filter.a (planner-auto-resolved at gate-1 — silent
//! post-filter at the 5 list-side reads): emission fires only at the **4
//! mutation callsites** (`templates/{approve,deny,revoke}.rs` +
//! `projects/create.rs` slot-fill mutation). The dashboard's silent
//! list-side post-filter does NOT emit per-filtered-entry — would
//! multiply audit-event volume by ~10× per non-admin viewer per render.
//! See plan §3.B A7 — emission frequency is low-volume and the event-bus
//! single-writer guarantee is preserved.
//!
//! ## Hash-chain symmetry
//!
//! The builder produces an [`AuditEvent`] whose `canonical_bytes`
//! excludes `prev_event_hash` (additive event-type — does not perturb
//! prior events' canonical bytes). Plan §3.B A7 + CH-12 plan §3.B A7
//! + CH-15 plan §3.B A7 precedent.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::auth_requests::access::{AuthRequestAccessError, IntendedOp};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};

/// Stable wire-form snake_case label for an [`IntendedOp`] variant.
///
/// The audit-event diff payload carries the op as a stable string so
/// downstream operators can join across audit dashboards without depending
/// on Rust's `Debug` representation. See plan §7 P2a deliverable 1.
fn intended_op_label(op: IntendedOp) -> &'static str {
    match op {
        IntendedOp::Read => "read",
        IntendedOp::Modify => "modify",
        IntendedOp::Submit => "submit",
        IntendedOp::Cancel => "cancel",
        IntendedOp::Approve => "approve",
        IntendedOp::Deny => "deny",
        IntendedOp::Reconsider => "reconsider",
        IntendedOp::Revoke => "revoke",
        IntendedOp::OverrideApprove => "override_approve",
        IntendedOp::CloseAsDenied => "close_as_denied",
        IntendedOp::Expire => "expire",
    }
}

/// Stable snake_case label for an [`AuthRequestAccessError`] variant.
///
/// The audit-event diff payload carries the deny class as a stable string
/// so dashboards can group denies by class without depending on Rust's
/// `Debug` representation.
fn error_kind_label(error: &AuthRequestAccessError) -> &'static str {
    match error {
        AuthRequestAccessError::NotAuthorisedForRead { .. } => "not_authorised_for_read",
        AuthRequestAccessError::NotAuthorisedForModify { .. } => "not_authorised_for_modify",
        AuthRequestAccessError::OperationForbiddenInState { .. } => "operation_forbidden_in_state",
        AuthRequestAccessError::RequestorOnlyOperation { .. } => "requestor_only_operation",
        AuthRequestAccessError::UnfilledApproverSlotOnly { .. } => "unfilled_approver_slot_only",
    }
}

/// Extract the principal's wire-form classification from the error variant.
///
/// `NotAuthorisedForRead` and `NotAuthorisedForModify` carry the principal-
/// kind string directly (`"agent"`, `"user"`, …). The other three deny
/// classes are state-shape rejections that don't depend on the principal's
/// classification — for those we surface `"unspecified"` in the diff.
fn principal_kind_label(error: &AuthRequestAccessError) -> String {
    match error {
        AuthRequestAccessError::NotAuthorisedForRead { principal_kind, .. } => {
            principal_kind.clone()
        }
        AuthRequestAccessError::NotAuthorisedForModify { principal_kind, .. } => {
            principal_kind.clone()
        }
        AuthRequestAccessError::OperationForbiddenInState { .. }
        | AuthRequestAccessError::RequestorOnlyOperation { .. }
        | AuthRequestAccessError::UnfilledApproverSlotOnly { .. } => "unspecified".to_string(),
    }
}

/// `auth_request.access_denied` — Alerted class (high-tier retention,
/// 60s delivery to org alert channel per `nfr-observability.md`).
///
/// Emitted by every AR-mutation handler whenever
/// [`crate::auth_requests::access::check_auth_request_access`] returns
/// `Err(AuthRequestAccessError::*)`. The handler emits this BEFORE
/// returning the typed `*Error::AccessDenied(...)` to the caller per
/// ADR-0056 §D56.6.
///
/// ## Diff shape
///
/// ```json
/// {
///   "before": null,
///   "after": {
///     "auth_request_id": "<uuid>",
///     "intended_op":     "approve" | "deny" | "revoke" | …,  // snake_case label
///     "error_kind":      "not_authorised_for_modify" | …,    // snake_case label
///     "principal_kind":  "agent" | "user" | "organization" | "project" | "system" | "unspecified",
///     "attempted_at":    "<rfc3339-timestamp>"
///   }
/// }
/// ```
///
/// Mirrors [`super::tool_authority::frozen_tag_write_rejected`]'s
/// "before:null / after:full-payload" shape for create-style rejection
/// events.
pub fn auth_request_access_denied(
    actor: AgentId,
    target_ar: AuthRequestId,
    org: OrgId,
    error: &AuthRequestAccessError,
    attempted_op: IntendedOp,
    attempted_at: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "auth_request.access_denied".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*target_ar.as_uuid())),
        timestamp: attempted_at,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": {
                "auth_request_id": target_ar.to_string(),
                "intended_op":     intended_op_label(attempted_op),
                "error_kind":      error_kind_label(error),
                "principal_kind":  principal_kind_label(error),
                "attempted_at":    attempted_at.to_rfc3339(),
            },
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(target_ar),
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, AuthRequestId, OrgId};
    use crate::model::nodes::AuthRequestState;

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp")
    }

    #[test]
    fn auth_request_access_denied_carries_alerted_class_and_org_scope() {
        let actor = AgentId::new();
        let ar = AuthRequestId::new();
        let org = OrgId::new();
        let err = AuthRequestAccessError::NotAuthorisedForModify {
            state: AuthRequestState::Pending,
            principal_kind: "agent".to_string(),
            intended_op: IntendedOp::Approve,
        };
        let evt = auth_request_access_denied(
            actor,
            ar,
            org,
            &err,
            IntendedOp::Approve,
            fixed_timestamp(),
        );

        assert_eq!(evt.event_type, "auth_request.access_denied");
        assert_eq!(evt.audit_class, AuditClass::Alerted);
        assert_eq!(evt.org_scope, Some(org));
        assert_eq!(evt.actor_agent_id, Some(actor));
        assert_eq!(evt.provenance_auth_request_id, Some(ar));
        assert_eq!(evt.target_entity_id, Some(NodeId::from_uuid(*ar.as_uuid())));
        assert_eq!(
            evt.diff["after"]["intended_op"].as_str().unwrap(),
            "approve"
        );
        assert_eq!(
            evt.diff["after"]["error_kind"].as_str().unwrap(),
            "not_authorised_for_modify"
        );
        assert_eq!(
            evt.diff["after"]["principal_kind"].as_str().unwrap(),
            "agent"
        );
        assert_eq!(
            evt.diff["after"]["auth_request_id"].as_str().unwrap(),
            ar.to_string()
        );
        assert!(evt.diff["before"].is_null());
        assert!(evt.prev_event_hash.is_none());
    }

    #[test]
    fn auth_request_access_denied_diff_carries_attempted_at_rfc3339() {
        let actor = AgentId::new();
        let ar = AuthRequestId::new();
        let org = OrgId::new();
        let err = AuthRequestAccessError::OperationForbiddenInState {
            state: AuthRequestState::Approved,
            intended_op: IntendedOp::Modify,
        };
        let ts = fixed_timestamp();
        let evt = auth_request_access_denied(actor, ar, org, &err, IntendedOp::Modify, ts);
        let attempted = evt.diff["after"]["attempted_at"]
            .as_str()
            .expect("attempted_at present as string");
        let parsed = chrono::DateTime::parse_from_rfc3339(attempted)
            .expect("attempted_at parses as RFC3339");
        assert_eq!(parsed.with_timezone(&Utc), ts);
        // OperationForbiddenInState carries no principal_kind data; the
        // builder surfaces "unspecified".
        assert_eq!(
            evt.diff["after"]["principal_kind"].as_str().unwrap(),
            "unspecified"
        );
        assert_eq!(
            evt.diff["after"]["error_kind"].as_str().unwrap(),
            "operation_forbidden_in_state"
        );
        assert_eq!(evt.diff["after"]["intended_op"].as_str().unwrap(), "modify");
    }

    #[test]
    fn auth_request_access_denied_canonical_bytes_byte_stable_for_identical_inputs() {
        // Builder must produce events whose `canonical_bytes()` is byte-equal
        // when given identical inputs (modulo the random `event_id` field,
        // which is part of canonical_bytes by design). This pins the F5.B
        // byte-stability claim from plan §3.B A7: additive event types do
        // not introduce non-determinism beyond what's already inherent in
        // `event_id`. Also pins that `prev_event_hash` is excluded from
        // canonical_bytes (CH-21 invariant + plan §3.B A7).
        let actor = AgentId::new();
        let ar = AuthRequestId::new();
        let org = OrgId::new();
        let ts = fixed_timestamp();
        let err = AuthRequestAccessError::NotAuthorisedForRead {
            state: AuthRequestState::Pending,
            principal_kind: "agent".to_string(),
        };
        let mut evt_a = auth_request_access_denied(actor, ar, org, &err, IntendedOp::Read, ts);
        let mut evt_b = auth_request_access_denied(actor, ar, org, &err, IntendedOp::Read, ts);
        // Force the same event_id so canonical_bytes only compare the
        // deterministic surface.
        evt_a.event_id = evt_b.event_id;
        // Re-run the existing CH-21 invariant: the prev_event_hash field
        // must NOT affect canonical_bytes.
        evt_a.prev_event_hash = None;
        evt_b.prev_event_hash = Some([7u8; 32]);
        assert_eq!(
            evt_a.canonical_bytes(),
            evt_b.canonical_bytes(),
            "F5.B: prev_event_hash must not affect canonical bytes for the new event_type"
        );
    }
}
