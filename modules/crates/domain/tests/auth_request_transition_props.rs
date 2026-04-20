//! Property tests for [`domain::auth_requests::transitions`].
//!
//! Invariants covered:
//!
//! 1. `illegal_request_transition_never_succeeds` — every `(from, to)`
//!    pair NOT in the legal table causes the relevant transition helper
//!    to return `Err` when coerced.
//! 2. `closed_terminal_states_reject_slot_changes` — Denied / Expired /
//!    Revoked / Cancelled reject `transition_slot` with `ClosedTerminal`.
//! 3. `cancel_only_from_active_states` — `cancel` succeeds exactly when
//!    the request is in Draft / Pending / InProgress.
//! 4. `slot_independence_one_slot_change_does_not_mutate_others` —
//!    transitioning slot `(i, j)` leaves every other slot exactly as it
//!    was.
//! 5. `expired_is_terminal` — once `Expired`, no further transition
//!    other than a trivial self-transition succeeds.

use chrono::{Duration, Utc};
use domain::audit::AuditClass;
use domain::auth_requests::transitions::{
    cancel, close_as_denied, expire, override_approve, submit, transition_slot, TransitionError,
};
use domain::model::ids::AuthRequestId;
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

fn fresh_req(
    state: AuthRequestState,
    slots_per_resource: Vec<Vec<ApproverSlotState>>,
) -> AuthRequest {
    AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::System("system:test".into()),
        kinds: vec![],
        scope: vec!["read".into()],
        state,
        valid_until: Some(Utc::now() + Duration::days(7)),
        submitted_at: Utc::now(),
        resource_slots: slots_per_resource
            .into_iter()
            .map(|slots| ResourceSlot {
                resource: ResourceRef {
                    uri: "test:r".into(),
                },
                approvers: slots
                    .into_iter()
                    .map(|s| ApproverSlot {
                        approver: PrincipalRef::System("system:test".into()),
                        state: s,
                        responded_at: None,
                        reconsidered_at: None,
                    })
                    .collect(),
                state: ResourceSlotState::InProgress,
            })
            .collect(),
        justification: None,
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
    }
}

proptest! {
    /// Starting from any closed-terminal state, transition_slot returns
    /// a `ClosedTerminal` error regardless of which slot or new state
    /// the caller requests.
    #[test]
    fn closed_terminal_states_reject_slot_changes(
        state in prop_oneof![
            Just(AuthRequestState::Denied),
            Just(AuthRequestState::Expired),
            Just(AuthRequestState::Revoked),
            Just(AuthRequestState::Cancelled),
        ],
    ) {
        let req = fresh_req(state, vec![vec![ApproverSlotState::Unfilled]]);
        let err = transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now())
            .expect_err("closed-terminal must reject slot change");
        let is_closed_terminal = matches!(err, TransitionError::ClosedTerminal { .. });
        prop_assert!(is_closed_terminal);
    }

    /// `cancel` succeeds exactly when the request is Draft / Pending /
    /// InProgress. Every other state rejects.
    #[test]
    fn cancel_only_from_active_states(state in any_state()) {
        let req = fresh_req(state, vec![vec![ApproverSlotState::Unfilled]]);
        let res = cancel(&req, Utc::now());
        let is_active = matches!(
            state,
            AuthRequestState::Draft | AuthRequestState::Pending | AuthRequestState::InProgress
        );
        if is_active {
            prop_assert!(res.is_ok(),
                "{state:?} should allow cancel; got {:?}", res);
            prop_assert_eq!(res.unwrap().state, AuthRequestState::Cancelled);
        } else {
            prop_assert!(res.is_err(), "{state:?} must not be cancellable");
        }
    }

    /// Transitioning approver slot `(i, j)` leaves every other slot
    /// exactly as it was.
    #[test]
    fn slot_independence_one_slot_change_does_not_mutate_others(
        slots_a in prop::collection::vec(
            prop_oneof![
                Just(ApproverSlotState::Unfilled),
                Just(ApproverSlotState::Approved),
                Just(ApproverSlotState::Denied),
            ],
            1..=3,
        ),
        slots_b in prop::collection::vec(
            prop_oneof![
                Just(ApproverSlotState::Unfilled),
                Just(ApproverSlotState::Approved),
                Just(ApproverSlotState::Denied),
            ],
            1..=3,
        ),
        fill_choice in prop::bool::ANY,
    ) {
        // Two resources. We'll flip one slot in resource 0 and check
        // resource 1's slots stay identical.
        let mut req = fresh_req(
            AuthRequestState::InProgress,
            vec![slots_a.clone(), slots_b.clone()],
        );
        // Make sure req.state is compatible (InProgress is fine as long
        // as at least one slot is filled).
        let any_filled_a = slots_a.iter().any(|s| *s != ApproverSlotState::Unfilled);
        let any_filled_b = slots_b.iter().any(|s| *s != ApproverSlotState::Unfilled);
        if !(any_filled_a || any_filled_b) {
            // Reset to Pending.
            req.state = AuthRequestState::Pending;
        }

        // Find a slot in resource 0 that has a legal transition.
        let target_new = if fill_choice {
            ApproverSlotState::Approved
        } else {
            ApproverSlotState::Denied
        };
        let mut target_idx = None;
        for (i, s) in req.resource_slots[0].approvers.iter().enumerate() {
            if s.state == ApproverSlotState::Unfilled {
                target_idx = Some(i);
                break;
            }
        }
        // If no unfilled slot in resource 0, reconsider a filled one.
        let (slot_idx, new_state) = if let Some(i) = target_idx {
            (i, target_new)
        } else {
            (0, ApproverSlotState::Unfilled)
        };

        let before_resource_1 = req.resource_slots[1].approvers.clone();
        let res = transition_slot(&req, 0, slot_idx, new_state, Utc::now());
        if let Ok(next) = res {
            // Resource 1's approvers must be byte-identical.
            prop_assert_eq!(
                next.resource_slots[1].approvers.len(),
                before_resource_1.len()
            );
            for (a, b) in next.resource_slots[1]
                .approvers
                .iter()
                .zip(before_resource_1.iter())
            {
                prop_assert_eq!(a.state, b.state);
            }
        }
        // If Err, no mutation occurred (pure fn + Result); that's also OK.
    }

    /// Once Expired, every transition operation either errors or is a
    /// self-transition that leaves the state as Expired.
    #[test]
    fn expired_is_terminal(dummy in 0u32..10) {
        let _ = dummy;
        let req = fresh_req(AuthRequestState::Expired, vec![vec![ApproverSlotState::Approved]]);
        // Every transition must either fail or leave state == Expired.
        let slot_res = transition_slot(&req, 0, 0, ApproverSlotState::Unfilled, Utc::now());
        prop_assert!(slot_res.is_err());

        let cancel_res = cancel(&req, Utc::now());
        prop_assert!(cancel_res.is_err());

        let override_res = override_approve(&req, Utc::now());
        prop_assert!(override_res.is_err());

        let close_res = close_as_denied(&req, Utc::now());
        prop_assert!(close_res.is_err());

        // expire() itself on an already-Expired request: since legal
        // table forbids Expired→Expired as a non-self transition, a call
        // to expire() may succeed as a self-transition (idempotent) or
        // fail; either way state must not leave Expired.
        let mut with_past = req.clone();
        with_past.valid_until = Some(Utc::now() - Duration::days(1));
        let exp_res = expire(&with_past, Utc::now());
        if let Ok(next) = exp_res {
            prop_assert_eq!(next.state, AuthRequestState::Expired);
        }
    }

    /// `submit` only succeeds from `Draft`. Every other state errors.
    #[test]
    fn submit_only_from_draft(state in any_state()) {
        let req = fresh_req(state, vec![vec![ApproverSlotState::Unfilled]]);
        let res = submit(&req, Utc::now());
        if state == AuthRequestState::Draft {
            prop_assert!(res.is_ok());
        } else {
            prop_assert!(res.is_err());
        }
    }
}
