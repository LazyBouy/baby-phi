//! Shape B approval-matrix proptest — M4 plan commitment **C11**.
//!
//! Shape B is phi's first two-approver governance flow. An AR is
//! minted with two approver slots (one per co-owning org). Operators
//! approve/deny each slot; the 4-outcome decision matrix is:
//!
//! | Slot 1  | Slot 2  | AR terminal | Project materialises? |
//! |---------|---------|-------------|----------------------|
//! | Approve | Approve | `Approved`  | ✅ yes                |
//! | Approve | Deny    | `Partial`   | ❌ no                 |
//! | Deny    | Approve | `Partial`   | ❌ no                 |
//! | Deny    | Deny    | `Denied`    | ❌ no                 |
//!
//! The proptest generates every (slot1, slot2) pair across 50 cases,
//! drives the slots through the existing
//! [`crate::auth_requests::transitions`] state machine, and asserts:
//!
//! 1. The AR's aggregated state matches the matrix row.
//! 2. The `should_materialize_project` predicate (pure) returns true
//!    iff the outcome is both-approve.
//!
//! Pinning the 4-outcome table at the domain layer means P6's
//! project-creation handler can rely on `should_materialize_project`
//! without re-deriving the logic on every code path.

use chrono::Utc;

use domain::auth_requests::transitions::transition_slot;
use domain::model::ids::{AgentId, AuthRequestId};
use domain::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef, ResourceRef,
    ResourceSlot, ResourceSlotState,
};

use proptest::prelude::*;

/// Pure helper: given a Shape B AR's terminal state, should the
/// caller materialise the project?
fn should_materialize_project(state: AuthRequestState) -> bool {
    matches!(state, AuthRequestState::Approved)
}

/// Build a 2-approver Shape B AR in `Pending` state — the shape the
/// page 10 submit handler produces at M4/P6.
fn build_shape_b_ar(approver_a: AgentId, approver_b: AgentId) -> AuthRequest {
    let now = Utc::now();
    let resource = ResourceRef {
        uri: "project:shape-b-probe".to_string(),
    };
    let slots = vec![
        ApproverSlot {
            approver: PrincipalRef::Agent(approver_a),
            state: ApproverSlotState::Unfilled,
            responded_at: None,
            reconsidered_at: None,
        },
        ApproverSlot {
            approver: PrincipalRef::Agent(approver_b),
            state: ApproverSlotState::Unfilled,
            responded_at: None,
            reconsidered_at: None,
        },
    ];
    AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::Agent(approver_a),
        kinds: vec!["#shape:b".to_string()],
        scope: vec!["project:shape-b-probe".to_string()],
        state: AuthRequestState::Pending,
        valid_until: None,
        submitted_at: now,
        resource_slots: vec![ResourceSlot {
            resource,
            approvers: slots,
            state: ResourceSlotState::InProgress,
        }],
        justification: Some("proptest".into()),
        audit_class: domain::audit::AuditClass::Alerted,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 7,
        provenance_template: None,
    }
}

#[derive(Debug, Clone, Copy)]
enum Decision {
    Approve,
    Deny,
}

fn arb_decision() -> impl Strategy<Value = Decision> {
    prop_oneof![Just(Decision::Approve), Just(Decision::Deny)]
}

fn apply(ar: AuthRequest, slot_idx: usize, _approver: AgentId, d: Decision) -> AuthRequest {
    let now = Utc::now();
    let new_state = match d {
        Decision::Approve => ApproverSlotState::Approved,
        Decision::Deny => ApproverSlotState::Denied,
    };
    transition_slot(&ar, 0, slot_idx, new_state, now).expect("slot transition")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn shape_b_approval_matrix_yields_expected_terminal_and_materialise_predicate(
        d1 in arb_decision(),
        d2 in arb_decision(),
    ) {
        let a1 = AgentId::new();
        let a2 = AgentId::new();
        let ar0 = build_shape_b_ar(a1, a2);
        let ar1 = apply(ar0, 0, a1, d1);
        let ar2 = apply(ar1, 1, a2, d2);

        // Expected terminal per the 4-outcome table.
        let expected = match (d1, d2) {
            (Decision::Approve, Decision::Approve) => AuthRequestState::Approved,
            (Decision::Deny, Decision::Deny) => AuthRequestState::Denied,
            _ => AuthRequestState::Partial,
        };
        prop_assert_eq!(ar2.state, expected);

        // Materialisation rule: only both-approve triggers project
        // creation.
        let expect_materialise = matches!((d1, d2), (Decision::Approve, Decision::Approve));
        prop_assert_eq!(
            should_materialize_project(ar2.state),
            expect_materialise
        );
    }
}

// ---- Unit smoke tests (explicit 4 outcomes) ------------------------------

#[test]
fn shape_b_both_approve_materialises_project() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let ar0 = build_shape_b_ar(a1, a2);
    let ar1 = apply(ar0, 0, a1, Decision::Approve);
    let ar2 = apply(ar1, 1, a2, Decision::Approve);
    assert_eq!(ar2.state, AuthRequestState::Approved);
    assert!(should_materialize_project(ar2.state));
}

#[test]
fn shape_b_both_deny_closes_as_denied_no_project() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let ar0 = build_shape_b_ar(a1, a2);
    let ar1 = apply(ar0, 0, a1, Decision::Deny);
    let ar2 = apply(ar1, 1, a2, Decision::Deny);
    assert_eq!(ar2.state, AuthRequestState::Denied);
    assert!(!should_materialize_project(ar2.state));
}

#[test]
fn shape_b_mixed_approve_deny_is_partial_no_project() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let ar0 = build_shape_b_ar(a1, a2);
    let ar1 = apply(ar0, 0, a1, Decision::Approve);
    let ar2 = apply(ar1, 1, a2, Decision::Deny);
    assert_eq!(ar2.state, AuthRequestState::Partial);
    assert!(!should_materialize_project(ar2.state));
}

#[test]
fn shape_b_mixed_deny_approve_is_partial_no_project() {
    let a1 = AgentId::new();
    let a2 = AgentId::new();
    let ar0 = build_shape_b_ar(a1, a2);
    let ar1 = apply(ar0, 0, a1, Decision::Deny);
    let ar2 = apply(ar1, 1, a2, Decision::Approve);
    assert_eq!(ar2.state, AuthRequestState::Partial);
    assert!(!should_materialize_project(ar2.state));
}
