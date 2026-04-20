//! Property tests for Step 4 — constraint satisfaction.
//!
//! Invariants covered:
//!
//! 1. `missing_constraint_denies_at_step_4` — for any constraint name
//!    declared on the manifest that the call's `constraint_context` does
//!    not carry, the engine returns `Denied { failed_step: Constraint }`
//!    (assuming earlier steps pass).
//! 2. `full_constraint_context_passes_step_4` — when the call carries a
//!    value for every required constraint, Step 4 never denies (outcome
//!    is `Allowed` for a minimal grant/manifest pair).

mod common;

use std::collections::HashMap;

use common::*;
use domain::model::ids::AgentId;
use domain::model::nodes::PrincipalRef;
use domain::permissions::{check, FailedStep, NoopMetrics, ToolCall};

use proptest::prelude::*;

fn constraint_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("path_prefix".to_string()),
        Just("command_pattern".to_string()),
        Just("sandbox".to_string()),
        Just("timeout_secs".to_string()),
        Just("max_size_bytes".to_string()),
    ]
}

proptest! {
    #[test]
    fn missing_constraint_denies_at_step_4(
        fundamental in any_fundamental(),
        action in any_action(),
        required_constraints in proptest::collection::vec(constraint_name(), 1..=3),
    ) {
        let agent = AgentId::new();
        let g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        // Empty constraint_context → at least one constraint is missing.
        let ctx = ctx_owned.borrow(ToolCall::default());
        let mut m = manifest_of(&[&action], &[fundamental.as_str()]);
        m.constraints = required_constraints;
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert_eq!(d.failed_step(), Some(FailedStep::Constraint));
    }

    #[test]
    fn full_constraint_context_never_denies_at_step_4(
        fundamental in any_fundamental(),
        action in any_action(),
        required_constraints in proptest::collection::vec(constraint_name(), 0..=3),
    ) {
        let agent = AgentId::new();
        let g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        // Populate constraint_context with every required constraint.
        let mut constraint_context: HashMap<String, serde_json::Value> = HashMap::new();
        for c in &required_constraints {
            constraint_context.insert(c.clone(), serde_json::json!("value"));
        }
        let call = ToolCall {
            constraint_context,
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let mut m = manifest_of(&[&action], &[fundamental.as_str()]);
        m.constraints = required_constraints;
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }
}
