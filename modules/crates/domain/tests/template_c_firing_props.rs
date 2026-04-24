//! Property tests for [`domain::templates::c::fire_grant_on_manages_edge`].
//!
//! M5/P3 proptest — mirrors Template A's shape invariants
//! (`template_a_firing_props.rs`) with Template C's own narrower
//! action set + agent-scoped (non-project) resource URI.
//!
//! Invariants pinned (50 proptest cases):
//! 1. `holder == Agent(args.manager)`.
//! 2. `action == ["read", "inspect"]` (stable order — note **no** `list`).
//! 3. `resource.uri == "agent:<subordinate-uuid>"`.
//! 4. `descends_from == Some(args.adoption_auth_request_id)`.
//! 5. `delegable == false`.
//! 6. `fundamentals == [Tag]`.
//! 7. `revoked_at.is_none()`.
//! 8. Distinct `GrantId` across independent calls.

use chrono::{DateTime, TimeZone, Utc};
use domain::model::ids::{AgentId, AuthRequestId};
use domain::model::{Fundamental, PrincipalRef};
use domain::templates::c::{fire_grant_on_manages_edge, FireArgs};
use proptest::prelude::*;

fn arb_now() -> impl Strategy<Value = DateTime<Utc>> {
    (1_577_836_800i64..2_208_988_800i64).prop_map(|s| Utc.timestamp_opt(s, 0).unwrap())
}

fn arb_args() -> impl Strategy<Value = FireArgs> {
    arb_now().prop_map(|now| FireArgs {
        manager: AgentId::new(),
        subordinate: AgentId::new(),
        adoption_auth_request_id: AuthRequestId::new(),
        now,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn holder_is_the_supplied_manager(args in arb_args()) {
        let expected = args.manager;
        let g = fire_grant_on_manages_edge(args);
        match g.holder {
            PrincipalRef::Agent(a) => prop_assert_eq!(a, expected),
            other => prop_assert!(false, "expected Agent holder, got {:?}", other),
        }
    }

    #[test]
    fn action_is_read_inspect_in_stable_order(args in arb_args()) {
        let g = fire_grant_on_manages_edge(args);
        prop_assert_eq!(g.action, vec!["read", "inspect"]);
    }

    #[test]
    fn resource_uri_names_the_subordinate_uuid(args in arb_args()) {
        let expected = format!("agent:{}", args.subordinate);
        let g = fire_grant_on_manages_edge(args);
        prop_assert_eq!(g.resource.uri, expected);
    }

    #[test]
    fn descends_from_supplied_adoption_ar(args in arb_args()) {
        let expected = args.adoption_auth_request_id;
        let g = fire_grant_on_manages_edge(args);
        prop_assert_eq!(g.descends_from, Some(expected));
    }

    #[test]
    fn grant_is_non_delegable_and_unrevoked(args in arb_args()) {
        let g = fire_grant_on_manages_edge(args);
        prop_assert!(!g.delegable);
        prop_assert!(g.revoked_at.is_none());
    }

    #[test]
    fn fundamentals_list_is_exactly_tag(args in arb_args()) {
        let g = fire_grant_on_manages_edge(args);
        prop_assert_eq!(g.fundamentals, vec![Fundamental::Tag]);
    }

    #[test]
    fn issued_at_matches_supplied_now(args in arb_args()) {
        let expected = args.now;
        let g = fire_grant_on_manages_edge(args);
        prop_assert_eq!(g.issued_at, expected);
    }

    #[test]
    fn independent_calls_produce_distinct_grant_ids(a in arb_args(), b in arb_args()) {
        let ga = fire_grant_on_manages_edge(a);
        let gb = fire_grant_on_manages_edge(b);
        prop_assert_ne!(ga.id, gb.id);
    }
}
