//! Property tests for [`domain::auth_requests::revocation`].
//!
//! Invariants covered:
//!
//! 1. `revocation_is_forward_only` — once a request is `Revoked`, no
//!    further transition (cancel / revoke / override / slot change)
//!    moves it out of `Revoked`.
//! 2. `revoke_only_from_approved_or_partial` — every other state
//!    rejects revoke with `NotRevocable` or `AlreadyClosed`.
//! 3. `revoked_request_emits_alerted_audit_event` — every successful
//!    revoke produces an `AuditClass::Alerted` event targeting the
//!    original request id.

use chrono::{Duration, Utc};
use domain::audit::{AuditClass, AuditEvent};
use domain::auth_requests::revocation::{revoke, RevocationError};
use domain::auth_requests::transitions::{
    cancel, override_approve, transition_slot, TransitionError,
};
use domain::model::ids::{AgentId, AuthRequestId, NodeId};
use domain::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef, ResourceRef,
    ResourceSlot, ResourceSlotState,
};

use proptest::prelude::*;

fn any_state() -> impl Strategy<Value = AuthRequestState> {
    prop_oneof![
        Just(AuthRequestState::Draft),
        Just(AuthRequestState::Pending),
        Just(AuthRequestState::InProgress),
        Just(AuthRequestState::Approved),
        Just(AuthRequestState::Denied),
        Just(AuthRequestState::Partial),
        Just(AuthRequestState::Expired),
        Just(AuthRequestState::Revoked),
        Just(AuthRequestState::Cancelled),
    ]
}

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
        terminal_state_entered_at: Some(Utc::now()),
        archived: false,
        active_window_days: 90,
        provenance_template: None,
    }
}

proptest! {
    /// Once Revoked, every subsequent transition either errors or leaves
    /// the state as Revoked. Cannot "un-revoke".
    #[test]
    fn revocation_is_forward_only(dummy in 0u32..10) {
        let _ = dummy;
        let req = req_in(AuthRequestState::Approved);
        let (revoked, _event) = revoke(&req, None, "done", Utc::now()).expect("revoke");
        prop_assert_eq!(revoked.state, AuthRequestState::Revoked);

        // Attempting every public transition must leave state == Revoked
        // (either via error or a no-op).
        let slot_res = transition_slot(&revoked, 0, 0, ApproverSlotState::Unfilled, Utc::now());
        let is_closed_terminal_err =
            matches!(slot_res, Err(TransitionError::ClosedTerminal { .. }));
        prop_assert!(is_closed_terminal_err);

        let cancel_res = cancel(&revoked, Utc::now());
        prop_assert!(cancel_res.is_err());

        let override_res = override_approve(&revoked, Utc::now());
        prop_assert!(override_res.is_err());

        let revoke_again = revoke(&revoked, None, "second", Utc::now());
        let is_already_closed =
            matches!(revoke_again, Err(RevocationError::AlreadyClosed { .. }));
        prop_assert!(is_already_closed);
    }

    /// `revoke` succeeds iff the state is Approved or Partial. Every
    /// other starting state rejects.
    #[test]
    fn revoke_only_from_approved_or_partial(state in any_state()) {
        let req = req_in(state);
        let res = revoke(&req, None, "test", Utc::now());
        match state {
            AuthRequestState::Approved | AuthRequestState::Partial => {
                prop_assert!(res.is_ok(),
                    "{state:?} should allow revoke; got {:?}", res);
            }
            _ => {
                prop_assert!(res.is_err(), "{state:?} must reject revoke");
            }
        }
    }

    /// Every successful revoke emits an Alerted audit event targeting
    /// the revoked request.
    #[test]
    fn revoked_request_emits_alerted_audit_event(
        actor in prop::option::of(prop::bool::ANY.prop_map(|_| AgentId::new())),
    ) {
        let req = req_in(AuthRequestState::Approved);
        let at = Utc::now();
        let (next, event): (AuthRequest, AuditEvent) =
            revoke(&req, actor, "compliance purge", at).expect("revoke");
        prop_assert_eq!(next.state, AuthRequestState::Revoked);
        prop_assert_eq!(event.event_type.as_str(), "auth_request.revoked");
        prop_assert_eq!(event.audit_class, AuditClass::Alerted);
        prop_assert_eq!(event.actor_agent_id, actor);
        prop_assert_eq!(
            event.target_entity_id,
            Some(NodeId::from_uuid(*req.id.as_uuid()))
        );
        prop_assert_eq!(event.provenance_auth_request_id, Some(req.id));
        prop_assert_eq!(event.timestamp, at);
    }
}
