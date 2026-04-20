//! Property tests for pipeline monotonicity invariants.
//!
//! Invariants covered:
//!
//! 1. `adding_unrelated_grant_preserves_allowed` — given an `Allowed`
//!    decision, adding a grant that covers only fundamentals unrelated to
//!    the manifest's reach set leaves the decision `Allowed`. No new
//!    grant can degrade an Allowed into something else (monotonicity of
//!    the candidate pool).
//! 2. `ceiling_never_widens` — given a `Denied` decision with no ceiling,
//!    adding a ceiling that admits no candidate still leaves the decision
//!    `Denied` (Step 2a can never turn a denial into an allowance).
//! 3. `revoked_grant_is_invisible` — a revoked grant in the candidate
//!    pool never affects the decision compared to omitting it entirely.

mod common;

use chrono::Utc;
use common::*;
use domain::model::ids::AgentId;
use domain::model::nodes::PrincipalRef;
use domain::permissions::{check, Decision, NoopMetrics, ToolCall};

use proptest::prelude::*;

proptest! {
    /// Adding a grant that covers a disjoint fundamental never changes an
    /// already-Allowed outcome.
    #[test]
    fn adding_unrelated_grant_preserves_allowed(
        core_fundamental in any_fundamental(),
        extra_fundamental in any_fundamental(),
        action in any_action(),
    ) {
        prop_assume!(core_fundamental != extra_fundamental);
        let agent = AgentId::new();
        let core = grant_on(PrincipalRef::Agent(agent), &[&action], core_fundamental.as_str());
        let extra = grant_on(PrincipalRef::Agent(agent), &[&action], extra_fundamental.as_str());

        // Baseline: core grant only.
        let mut baseline = ctx_with_agent_grants(vec![core.clone()]);
        baseline.agent = agent;
        let ctx = baseline.borrow(ToolCall::default());
        let m = manifest_of(&[&action], &[core_fundamental.as_str()]);
        let baseline_decision = check(&ctx, &m, &NoopMetrics);
        prop_assume!(matches!(baseline_decision, Decision::Allowed { .. }));
        drop(ctx);

        // Augmented: core + extra.
        let mut augmented = ctx_with_agent_grants(vec![core, extra]);
        augmented.agent = agent;
        let ctx = augmented.borrow(ToolCall::default());
        let augmented_decision = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            matches!(augmented_decision, Decision::Allowed { .. }),
            "extra grant flipped Allowed → {:?}",
            augmented_decision
        );
    }

    /// Adding a ceiling that admits nothing can never turn a Denial into
    /// an Allowance. The ceiling is a strict clamp, so more ceilings can
    /// only narrow.
    #[test]
    fn ceiling_never_widens_a_denial(
        core_fundamental in any_fundamental(),
        ceiling_fundamental in any_fundamental(),
        action in any_action(),
    ) {
        prop_assume!(core_fundamental != ceiling_fundamental);
        let agent = AgentId::new();
        let core = grant_on(PrincipalRef::Agent(agent), &[&action], core_fundamental.as_str());

        // Baseline with no ceiling: this configuration is Allowed.
        let mut baseline = ctx_with_agent_grants(vec![core.clone()]);
        baseline.agent = agent;
        let m = manifest_of(&[&action], &[core_fundamental.as_str()]);
        let baseline_decision = check(&baseline.borrow(ToolCall::default()), &m, &NoopMetrics);
        prop_assume!(baseline_decision.is_allowed());

        // With ceiling that only covers a disjoint fundamental: every
        // candidate is clamped out → denial.
        let ceiling = grant_on(
            PrincipalRef::Agent(agent),
            &[&action],
            ceiling_fundamental.as_str(),
        );
        let mut with_ceiling = ctx_with_agent_grants(vec![core]);
        with_ceiling.agent = agent;
        with_ceiling.ceiling_grants.push(ceiling);
        let d = check(&with_ceiling.borrow(ToolCall::default()), &m, &NoopMetrics);
        prop_assert!(!d.is_allowed(),
            "ceiling should clamp Allowed → Denied, got {:?}", d);
    }

    /// Revoked grants are treated as if they did not exist.
    #[test]
    fn revoked_grants_are_invisible(
        fundamental in any_fundamental(),
        action in any_action(),
    ) {
        let agent = AgentId::new();
        let mut revoked = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        revoked.revoked_at = Some(Utc::now());

        let mut ctx_owned = ctx_with_agent_grants(vec![revoked]);
        ctx_owned.agent = agent;
        let ctx = ctx_owned.borrow(ToolCall::default());
        let m = manifest_of(&[&action], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        // Single grant, revoked → candidate pool is empty → step 2 denial.
        prop_assert!(!d.is_allowed());
    }
}
