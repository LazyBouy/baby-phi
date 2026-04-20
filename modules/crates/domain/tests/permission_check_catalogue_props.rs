//! Property tests for Step 0 — Resource Catalogue precondition.
//!
//! Invariants covered (2, each expanded to `PROPTEST_CASES=100` random
//! inputs in CI):
//!
//! 1. `catalogue_miss_always_denies_at_step_0` — if the call names a
//!    specific target URI that is NOT in the catalogue, the engine returns
//!    `Denied { failed_step: Catalogue }` regardless of what grants / what
//!    manifest the caller has.
//! 2. `catalogue_hit_never_denies_at_step_0` — if the target URI is in the
//!    catalogue, the decision's failed step (if any) is not Catalogue.

mod common;

use common::*;
use domain::permissions::{check, FailedStep, NoopMetrics, ToolCall};

use proptest::prelude::*;

proptest! {
    /// Step 0 is a strict pre-filter. When the target URI is non-empty and
    /// not catalogued, the outcome is always a Step-0 denial.
    #[test]
    fn catalogue_miss_always_denies_at_step_0(
        target in "[a-z]{1,6}:[a-z0-9/]{1,10}",
    ) {
        let ctx_owned = ctx_with_agent_grants(vec![
            // Give the agent some grants — irrelevant, because Step 0 fires
            // first.
            grant_on(
                domain::model::nodes::PrincipalRef::Agent(domain::model::ids::AgentId::new()),
                &["read"],
                "filesystem_object",
            ),
        ]);
        let call = ToolCall {
            target_uri: target.clone(),
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let m = manifest_of(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert_eq!(d.failed_step(), Some(FailedStep::Catalogue));
    }

    /// If the target IS in the catalogue, the denial (if any) is never
    /// attributed to Step 0 — the engine moves on to later steps.
    #[test]
    fn catalogue_hit_never_denies_at_step_0(
        target in "[a-z]{1,6}:[a-z0-9/]{1,10}",
    ) {
        let mut ctx_owned = ctx_with_agent_grants(vec![]);
        ctx_owned.catalogue.seed(None, target.clone());
        let call = ToolCall {
            target_uri: target,
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let m = manifest_of(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        // May be denied at a later step (e.g. no grants held), but not at
        // Step 0.
        prop_assert_ne!(d.failed_step(), Some(FailedStep::Catalogue));
    }
}
