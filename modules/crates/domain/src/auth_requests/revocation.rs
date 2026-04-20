//! Forward-only revocation of an Auth Request.
//!
//! Per `concepts/permissions/02` §State Machine:
//!
//! > Owner revokes approved → Revoked; Grant auto-revoked; past reads
//! > remain in audit log; future actions denied.
//!
//! "Forward-only" means:
//!
//! 1. Once a request is `Revoked`, no transition can move it out
//!    (terminal guard in [`super::state::is_closed_terminal`]).
//! 2. Any grants descending from this request must be revoked as a
//!    consequence. P4 models the **request-side** of that rule; the
//!    store-side cascade is a repository concern (P5 bootstrap flow
//!    calls `Repository::revoke_grant` for every grant whose
//!    `descends_from == request.id`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, NodeId};
use crate::model::nodes::{AuthRequest, AuthRequestState};

use super::state::is_closed_terminal;
use super::transitions::TransitionError;

/// Structured error for a rejected revocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RevocationError {
    /// Request is not in a state that can be revoked. Per the concept
    /// doc, only `Approved` and `Partial` are revocable — the owner has
    /// a Grant (or partial Grant) to pull back.
    #[error("revoke only valid from Approved or Partial (got {state:?})")]
    NotRevocable { state: AuthRequestState },
    /// Request is already in a closed terminal — re-revocation is a
    /// semantic no-op, surfaced as an explicit error so callers notice.
    #[error("request is already in closed terminal state {state:?}")]
    AlreadyClosed { state: AuthRequestState },
    /// A deeper transition-layer error (should be rare — the revocation
    /// path validates state first).
    #[error("inner transition error: {0}")]
    Transition(#[from] TransitionError),
}

/// Revoke an Auth Request. Returns the revoked request **and** the audit
/// event the caller must persist. The audit event is pre-built with
/// `prev_event_hash = None` — the `AuditEmitter` fills in the hash when
/// it writes the event.
///
/// Callers are responsible for the downstream grant-revocation cascade
/// (repository query + `revoke_grant` per matching grant id). Keeping the
/// cascade outside this pure function avoids dragging `Repository` into
/// the domain-layer state machine.
pub fn revoke(
    req: &AuthRequest,
    revoked_by: Option<AgentId>,
    reason: &str,
    at: DateTime<Utc>,
) -> Result<(AuthRequest, AuditEvent), RevocationError> {
    if is_closed_terminal(req.state) {
        return Err(RevocationError::AlreadyClosed { state: req.state });
    }
    if !matches!(
        req.state,
        AuthRequestState::Approved | AuthRequestState::Partial
    ) {
        return Err(RevocationError::NotRevocable { state: req.state });
    }

    // Use the transition helper so the legal-transition table stays
    // authoritative for terminal stamping.
    let next = super::transitions::transition_to_revoked(req, at)?;
    let event = AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "auth_request.revoked".into(),
        actor_agent_id: revoked_by,
        target_entity_id: Some(NodeId::from_uuid(*req.id.as_uuid())),
        timestamp: at,
        diff: serde_json::json!({
            "before": {"state": req.state},
            "after":  {"state": next.state, "revoked_at": at, "reason": reason},
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(req.id),
        org_scope: None,
        prev_event_hash: None,
    };
    Ok((next, event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::AuthRequestId;
    use crate::model::nodes::{
        ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef, ResourceRef,
        ResourceSlot, ResourceSlotState,
    };
    use chrono::Duration;

    fn req_in(state: AuthRequestState) -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor: PrincipalRef::System("system:test".into()),
            kinds: vec![],
            scope: vec!["read".into()],
            state,
            valid_until: Some(Utc::now() + Duration::days(7)),
            submitted_at: Utc::now(),
            resource_slots: vec![ResourceSlot {
                resource: ResourceRef {
                    uri: "test:r".into(),
                },
                approvers: vec![ApproverSlot {
                    approver: PrincipalRef::System("system:test".into()),
                    state: ApproverSlotState::Approved,
                    responded_at: Some(Utc::now()),
                    reconsidered_at: None,
                }],
                state: ResourceSlotState::Approved,
            }],
            justification: None,
            audit_class: AuditClass::Logged,
            terminal_state_entered_at: None,
            archived: false,
            active_window_days: 90,
            provenance_template: None,
        }
    }

    #[test]
    fn revoke_from_approved_succeeds_and_emits_audit_event() {
        let req = req_in(AuthRequestState::Approved);
        let actor = AgentId::new();
        let at = Utc::now();
        let (next, event) = revoke(&req, Some(actor), "not needed anymore", at).unwrap();

        assert_eq!(next.state, AuthRequestState::Revoked);
        assert_eq!(next.terminal_state_entered_at, Some(at));
        assert_eq!(event.event_type, "auth_request.revoked");
        assert_eq!(event.actor_agent_id, Some(actor));
        assert_eq!(event.audit_class, AuditClass::Alerted);
        assert_eq!(event.provenance_auth_request_id, Some(req.id));
    }

    #[test]
    fn revoke_from_partial_also_succeeds() {
        let req = req_in(AuthRequestState::Partial);
        let (next, _) = revoke(&req, None, "partial revoke", Utc::now()).unwrap();
        assert_eq!(next.state, AuthRequestState::Revoked);
    }

    #[test]
    fn revoke_from_denied_fails_as_already_closed() {
        let req = req_in(AuthRequestState::Denied);
        let err = revoke(&req, None, "never approved", Utc::now()).unwrap_err();
        assert!(matches!(err, RevocationError::AlreadyClosed { .. }));
    }

    #[test]
    fn revoke_from_pending_fails_as_not_revocable() {
        let req = req_in(AuthRequestState::Pending);
        let err = revoke(&req, None, "too early", Utc::now()).unwrap_err();
        assert!(matches!(err, RevocationError::NotRevocable { .. }));
    }
}
