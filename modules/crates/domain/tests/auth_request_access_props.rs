//! CH-18 / ADR-0056 §D56.5 — property test for the per-state access
//! matrix (F4.A MAY-COVER, MUST-SHIP per plan §8 v2).
//!
//! Invariants covered:
//!
//! 1. **Read on closed-terminal states by requestor or slot-approver
//!    always returns Ok.** The matrix's row 5 (Denied / Cancelled /
//!    Revoked / Expired) admits Read by either class regardless of
//!    slot fill state.
//! 2. **Modify on closed-terminal states is forbidden to every
//!    principal.** OperationForbiddenInState is the only error class
//!    on closed-terminals.
//! 3. **Submit by `ar.requestor` on a synthetic-Draft AR always
//!    returns Ok.** The defence-in-depth invariant the F3.B.create-
//!    side.a callsites rely on.

use domain::audit::AuditClass;
use domain::auth_requests::access::{
    check_auth_request_access, AuthRequestAccessError, IntendedOp,
};
use domain::model::ids::{AgentId, AuthRequestId};
use domain::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef, ResourceRef,
    ResourceSlot, ResourceSlotState,
};

use chrono::Utc;
use proptest::prelude::*;

fn arbitrary_ar(
    state: AuthRequestState,
    requestor: PrincipalRef,
    slots: Vec<ApproverSlot>,
) -> AuthRequest {
    AuthRequest {
        id: AuthRequestId::new(),
        requestor,
        kinds: vec![],
        scope: vec![],
        state,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "test:r".into(),
            },
            approvers: slots,
            state: ResourceSlotState::InProgress,
        }],
        justification: None,
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
        tags: Vec::new(),
        descends_from_grant: None,
    }
}

fn closed_terminal_state() -> impl Strategy<Value = AuthRequestState> {
    prop_oneof![
        Just(AuthRequestState::Denied),
        Just(AuthRequestState::Cancelled),
        Just(AuthRequestState::Revoked),
        Just(AuthRequestState::Expired),
    ]
}

fn any_approver_state() -> impl Strategy<Value = ApproverSlotState> {
    prop_oneof![
        Just(ApproverSlotState::Unfilled),
        Just(ApproverSlotState::Approved),
        Just(ApproverSlotState::Denied),
    ]
}

proptest! {
    /// On any closed-terminal state, Read by the requestor returns Ok,
    /// and Read by a slot approver returns Ok regardless of slot state.
    #[test]
    fn closed_terminal_read_by_requestor_or_slot_approver_always_ok(
        state in closed_terminal_state(),
        slot_state in any_approver_state(),
    ) {
        let requestor_id = AgentId::new();
        let approver_id = AgentId::new();
        let slot = ApproverSlot {
            approver: PrincipalRef::Agent(approver_id),
            state: slot_state,
            responded_at: None,
            reconsidered_at: None,
        };
        let ar = arbitrary_ar(state, PrincipalRef::Agent(requestor_id), vec![slot]);

        prop_assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor_id),
            IntendedOp::Read,
        )
        .is_ok());

        prop_assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(approver_id),
            IntendedOp::Read,
        )
        .is_ok());
    }

    /// On any closed-terminal state, Modify by ANYONE returns
    /// OperationForbiddenInState.
    #[test]
    fn closed_terminal_modify_by_anyone_forbidden(
        state in closed_terminal_state(),
    ) {
        let requestor_id = AgentId::new();
        let stranger_id = AgentId::new();
        let ar = arbitrary_ar(state, PrincipalRef::Agent(requestor_id), vec![]);

        let err_requestor = check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor_id),
            IntendedOp::Modify,
        )
        .unwrap_err();
        let is_forbidden = matches!(
            err_requestor,
            AuthRequestAccessError::OperationForbiddenInState { .. }
        );
        prop_assert!(is_forbidden);

        let err_stranger = check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(stranger_id),
            IntendedOp::Modify,
        )
        .unwrap_err();
        let is_forbidden = matches!(
            err_stranger,
            AuthRequestAccessError::OperationForbiddenInState { .. }
        );
        prop_assert!(is_forbidden);
    }

    /// Synthetic-Draft Submit by ar.requestor always returns Ok. This
    /// is the defence-in-depth invariant the F3.B.create-side.a callsites
    /// rely on.
    #[test]
    fn synthetic_draft_submit_by_requestor_always_ok(
        slot_state in any_approver_state(),
    ) {
        let requestor_id = AgentId::new();
        let approver_id = AgentId::new();
        let slot = ApproverSlot {
            approver: PrincipalRef::Agent(approver_id),
            state: slot_state,
            responded_at: None,
            reconsidered_at: None,
        };
        // Synthetic-Draft probe: AR built with state=Draft so the
        // matrix's `Draft × Submit by requestor` cell is queried.
        let ar = arbitrary_ar(
            AuthRequestState::Draft,
            PrincipalRef::Agent(requestor_id),
            vec![slot],
        );
        prop_assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor_id),
            IntendedOp::Submit,
        )
        .is_ok());
    }
}
