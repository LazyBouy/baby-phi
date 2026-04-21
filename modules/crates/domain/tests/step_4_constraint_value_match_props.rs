//! Property tests for the M2 widening of Step 4 of the Permission
//! Check — the `constraint_requirements` value-match addition (G3 in
//! the archived M2 plan).
//!
//! Two invariants under random inputs:
//!   1. Identity: when the invocation's context value equals the
//!      manifest's required value, Step 4 passes.
//!   2. Mismatch: when they differ, Step 4 denies at
//!      `FailedStep::Constraint`.

use chrono::Utc;
use proptest::prelude::*;
use std::collections::HashMap;

use domain::model::ids::{AgentId, GrantId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::permissions::{
    catalogue::StaticCatalogue,
    check,
    decision::FailedStep,
    manifest::{CheckContext, ConsentIndex, Manifest, ToolCall},
    metrics::NoopMetrics,
};

fn fixture_grant(agent: AgentId) -> Grant {
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(agent),
        action: vec!["read".into()],
        resource: ResourceRef {
            uri: "filesystem_object".into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    }
}

fn build_ctx<'a>(
    agent: AgentId,
    grants: &'a [Grant],
    call: ToolCall,
    catalogue: &'a StaticCatalogue,
    consents: &'a ConsentIndex,
    template_gated: &'a std::collections::HashSet<domain::model::ids::AuthRequestId>,
) -> CheckContext<'a> {
    CheckContext {
        agent,
        current_org: None,
        current_project: None,
        agent_grants: grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue,
        consents,
        template_gated_auth_requests: template_gated,
        call,
    }
}

/// Arbitrary JSON scalar — the comparators Step 4 uses are
/// straightforward `serde_json::Value` equality, so strings, numbers,
/// and booleans cover the invariant adequately.
fn arb_scalar_json() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        "[a-z0-9_-]{1,12}".prop_map(serde_json::Value::from),
        any::<i64>().prop_map(serde_json::Value::from),
        any::<bool>().prop_map(serde_json::Value::from),
    ]
}

proptest! {
    #[test]
    fn identity_value_always_passes_step_4(
        key in "[a-z][a-z0-9_]{0,16}",
        required in arb_scalar_json(),
    ) {
        let agent = AgentId::new();
        let grants = [fixture_grant(agent)];
        let catalogue = StaticCatalogue::empty();
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();

        let mut call = ToolCall::default();
        call.constraint_context.insert(key.clone(), required.clone());

        let mut m = Manifest {
            actions: vec!["read".into()],
            resource: vec!["filesystem_object".into()],
            constraints: vec![key.clone()],
            ..Default::default()
        };
        m.constraint_requirements.insert(key, required);

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            d.is_allowed(),
            "identity value match should pass, got {:?}",
            d
        );
    }

    #[test]
    fn mismatched_value_always_denies_at_step_4(
        key in "[a-z][a-z0-9_]{0,16}",
        required in arb_scalar_json(),
        provided in arb_scalar_json(),
    ) {
        // Shrink-friendly: if the proptest happens to generate identical
        // values, just skip this input (the identity case is covered
        // above).
        prop_assume!(required != provided);

        let agent = AgentId::new();
        let grants = [fixture_grant(agent)];
        let catalogue = StaticCatalogue::empty();
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();

        let mut call = ToolCall::default();
        call.constraint_context.insert(key.clone(), provided);

        let mut m = Manifest {
            actions: vec!["read".into()],
            resource: vec!["filesystem_object".into()],
            constraints: vec![key.clone()],
            ..Default::default()
        };
        m.constraint_requirements.insert(key, required);

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert_eq!(
            d.failed_step(),
            Some(FailedStep::Constraint),
            "mismatch must fail at Step 4"
        );
    }

    #[test]
    fn presence_only_is_unchanged_when_no_requirement_set(
        key in "[a-z][a-z0-9_]{0,16}",
        provided in arb_scalar_json(),
    ) {
        // No `constraint_requirements` entry — only presence is checked.
        let agent = AgentId::new();
        let grants = [fixture_grant(agent)];
        let catalogue = StaticCatalogue::empty();
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();

        let mut call = ToolCall::default();
        call.constraint_context.insert(key.clone(), provided);

        let m = Manifest {
            actions: vec!["read".into()],
            resource: vec!["filesystem_object".into()],
            constraints: vec![key],
            constraint_requirements: HashMap::new(),
            ..Default::default()
        };

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            d.is_allowed(),
            "presence-only should still pass when value is arbitrary"
        );
    }
}
