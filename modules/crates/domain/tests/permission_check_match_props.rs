//! Property tests for Step 2 (resolution) and Step 3 (reach matching).
//!
//! Invariants covered:
//!
//! 1. `empty_grants_deny_at_resolution_step` — the candidate grant set is
//!    empty ⇒ `Denied { failed_step: Resolution }`.
//! 2. `disjoint_grant_and_manifest_deny_at_match_step` — candidate grants
//!    cover only fundamentals the manifest does not need ⇒
//!    `Denied { failed_step: Match }`.
//! 3. `manifest_covered_by_single_grant_is_allowed` — for every
//!    fundamental in the universe, a grant covering exactly that
//!    fundamental + the manifest's action ⇒ `Allowed`.

mod common;

use common::*;
use domain::model::ids::AgentId;
use domain::model::nodes::PrincipalRef;
use domain::model::Fundamental;
use domain::permissions::{check, Decision, FailedStep, NoopMetrics, ToolCall};

use proptest::prelude::*;

proptest! {
    #[test]
    fn empty_grants_deny_at_resolution_step(
        action in any_action(),
        resource_fs in any_fundamental(),
    ) {
        let ctx_owned = ctx_with_agent_grants(vec![]);
        let ctx = ctx_owned.borrow(ToolCall::default());
        let m = manifest_of(&[&action], &[resource_fs.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert_eq!(d.failed_step(), Some(FailedStep::Resolution));
    }

    #[test]
    fn disjoint_grant_and_manifest_deny_at_match_step(
        grant_fundamental in any_fundamental(),
        manifest_fundamental in any_fundamental(),
        action in any_action(),
    ) {
        // Only run when the two fundamentals are distinct — otherwise the
        // grant would cover the manifest.
        prop_assume!(grant_fundamental != manifest_fundamental);
        let agent = AgentId::new();
        let g = grant_on(PrincipalRef::Agent(agent), &[&action], grant_fundamental.as_str());
        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        let ctx = ctx_owned.borrow(ToolCall::default());
        let m = manifest_of(&[&action], &[manifest_fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert_eq!(d.failed_step(), Some(FailedStep::Match));
    }

    #[test]
    fn single_covering_grant_yields_allowed(
        fundamental in any_fundamental(),
        action in any_action(),
    ) {
        let agent = AgentId::new();
        let g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        let ctx = ctx_owned.borrow(ToolCall::default());
        let m = manifest_of(&[&action], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            matches!(d, Decision::Allowed { .. }),
            "expected Allowed, got {:?}",
            d
        );
    }

    #[test]
    fn every_fundamental_has_its_own_covering_grant(
        fundamental in any_fundamental(),
    ) {
        // Sanity: for every one of the 9 fundamentals, a minimal grant /
        // manifest pair is Allowed. This guards against enum-expansion
        // regressions in Fundamental::ALL.
        prop_assert!(Fundamental::ALL.contains(&fundamental));
        let agent = AgentId::new();
        let g = grant_on(PrincipalRef::Agent(agent), &["read"], fundamental.as_str());
        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        let ctx = ctx_owned.borrow(ToolCall::default());
        let m = manifest_of(&["read"], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(d.is_allowed());
    }
}
