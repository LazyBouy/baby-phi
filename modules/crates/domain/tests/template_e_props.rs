//! Property tests for `domain::templates::e::build_auto_approved_request`.
//!
//! Verifies the invariants the M2 plan pins (§P2 + C4 commitment):
//!   1. Every built Auth Request is in `AuthRequestState::Approved`.
//!   2. The requestor and the single approver slot always match
//!      bit-for-bit.
//!   3. `submitted_at` and `terminal_state_entered_at` are identical
//!      (Template E enters its terminal state at construction time).
//!
//! These are proptests rather than example-based unit tests so the
//! "arbitrary requestor, resource, scope" axis is actually explored.

use chrono::{TimeZone, Utc};
use proptest::prelude::*;

use domain::audit::AuditClass;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{
    ApproverSlotState, AuthRequestState, PrincipalRef, ResourceRef, ResourceSlotState,
};
use domain::templates::e::{build_auto_approved_request, BuildArgs};

fn arb_principal() -> impl Strategy<Value = PrincipalRef> {
    prop_oneof![
        Just(PrincipalRef::Agent(AgentId::new())),
        Just(PrincipalRef::Organization(OrgId::new())),
        Just(PrincipalRef::System("system:genesis".to_string())),
    ]
}

fn arb_resource() -> impl Strategy<Value = ResourceRef> {
    "[a-z][a-z0-9:_-]{0,30}".prop_map(|uri| ResourceRef { uri })
}

fn arb_scope() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::vec("[a-z][a-z0-9:_-]{0,20}", 0..5)
}

fn arb_kinds() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::vec("#kind:[a-z_]{3,20}", 0..4)
}

fn arb_audit_class() -> impl Strategy<Value = AuditClass> {
    prop_oneof![
        Just(AuditClass::Silent),
        Just(AuditClass::Logged),
        Just(AuditClass::Alerted),
    ]
}

proptest! {
    #[test]
    fn every_built_request_is_approved(
        requestor in arb_principal(),
        resource in arb_resource(),
        kinds in arb_kinds(),
        scope in arb_scope(),
        audit_class in arb_audit_class(),
    ) {
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 0, 0).unwrap();
        let ar = build_auto_approved_request(BuildArgs {
            requestor_and_approver: requestor,
            resource,
            kinds,
            scope,
            justification: None,
            audit_class,
            now,
        });
        prop_assert_eq!(ar.state, AuthRequestState::Approved);
        prop_assert_eq!(ar.resource_slots.len(), 1);
        prop_assert_eq!(ar.resource_slots[0].state, ResourceSlotState::Approved);
        prop_assert_eq!(ar.resource_slots[0].approvers.len(), 1);
        prop_assert_eq!(
            ar.resource_slots[0].approvers[0].state,
            ApproverSlotState::Approved
        );
    }

    #[test]
    fn requestor_always_matches_single_approver(
        requestor in arb_principal(),
        resource in arb_resource(),
        kinds in arb_kinds(),
        scope in arb_scope(),
    ) {
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 0, 0).unwrap();
        let ar = build_auto_approved_request(BuildArgs {
            requestor_and_approver: requestor.clone(),
            resource,
            kinds,
            scope,
            justification: None,
            audit_class: AuditClass::Alerted,
            now,
        });
        let approver = &ar.resource_slots[0].approvers[0].approver;
        let ok = match (&requestor, approver) {
            (PrincipalRef::Agent(a), PrincipalRef::Agent(b)) => a == b,
            (PrincipalRef::User(a), PrincipalRef::User(b)) => a == b,
            (PrincipalRef::Organization(a), PrincipalRef::Organization(b)) => a == b,
            (PrincipalRef::Project(a), PrincipalRef::Project(b)) => a == b,
            (PrincipalRef::System(a), PrincipalRef::System(b)) => a == b,
            _ => false,
        };
        prop_assert!(ok, "requestor must equal the single filled approver");
    }

    #[test]
    fn submitted_at_equals_terminal_state_entered_at(
        requestor in arb_principal(),
        resource in arb_resource(),
        secs in 0i64..(60i64 * 60 * 24 * 365 * 10),
    ) {
        let now = Utc.timestamp_opt(secs, 0).single().unwrap();
        let ar = build_auto_approved_request(BuildArgs {
            requestor_and_approver: requestor,
            resource,
            kinds: vec![],
            scope: vec![],
            justification: None,
            audit_class: AuditClass::Logged,
            now,
        });
        prop_assert_eq!(ar.submitted_at, now);
        prop_assert_eq!(ar.terminal_state_entered_at, Some(now));
    }
}
