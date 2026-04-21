//! Property tests for M2/P4.5: `Grant.fundamentals` drives the
//! engine's `resolve_grant` Case D (G19 / D17 / C21).
//!
//! Three invariants:
//!
//! 1. **Instance-URI grants with explicit fundamentals resolve.**
//!    When a grant carries `resource.uri = "<class>:<instance>"` +
//!    `fundamentals = [F]`, and the manifest asks for reach
//!    `(F, action)` against the same target URI (catalogued), the
//!    engine returns `Decision::Allowed`.
//!
//! 2. **Legacy class-URI grants unchanged (regression guard).**
//!    When a grant carries `resource.uri = "<fundamental_class>"` +
//!    `fundamentals = []`, resolution behaves exactly as M1 did
//!    (fundamentals derived from the URI name).
//!
//! 3. **Opaque-URI grants with empty fundamentals still fail.**
//!    When a grant carries an instance URI + empty fundamentals
//!    (the exact pre-P4.5 trap case), it cannot cover any reach —
//!    `Decision::Denied { failed_step: Match, .. }` or Resolution
//!    (no candidates after Step 3 filtering).

use chrono::Utc;
use proptest::prelude::*;

use domain::model::ids::{AgentId, GrantId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::Fundamental;
use domain::permissions::{
    catalogue::StaticCatalogue,
    check,
    decision::FailedStep,
    manifest::{CheckContext, ConsentIndex, Manifest, ToolCall},
    metrics::NoopMetrics,
};

fn arb_fundamental() -> impl Strategy<Value = Fundamental> {
    prop_oneof![
        Just(Fundamental::SecretCredential),
        Just(Fundamental::NetworkEndpoint),
        Just(Fundamental::DataObject),
        Just(Fundamental::FilesystemObject),
    ]
}

fn arb_kind_and_instance() -> impl Strategy<Value = (String, String)> {
    ("[a-z][a-z0-9-]{1,10}", "[a-z0-9][a-z0-9-]{2,20}")
}

fn arb_action() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("read".to_string()),
        Just("invoke".to_string()),
        Just("modify".to_string()),
    ]
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

proptest! {
    /// Invariant 1: instance-URI grant with explicit fundamentals resolves.
    #[test]
    fn instance_uri_grant_with_fundamentals_is_allowed(
        (kind, instance) in arb_kind_and_instance(),
        fundamental in arb_fundamental(),
        action in arb_action(),
    ) {
        let uri = format!("{kind}:{instance}");
        let agent = AgentId::new();
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec![action.clone()],
            resource: ResourceRef { uri: uri.clone() },
            fundamentals: vec![fundamental],
            descends_from: None,
            delegable: false,
            issued_at: Utc::now(),
            revoked_at: None,
        };
        let grants = [grant];
        let mut catalogue = StaticCatalogue::empty();
        catalogue.seed(None, &uri);
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();

        let call = ToolCall {
            target_uri: uri.clone(),
            target_tags: vec![uri.clone()],
            target_agent: None,
            constraint_context: std::collections::HashMap::new(),
        };
        let manifest = Manifest {
            actions: vec![action],
            resource: vec![fundamental.as_str().to_string()],
            ..Default::default()
        };

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &manifest, &NoopMetrics);
        prop_assert!(
            d.is_allowed(),
            "instance-URI grant with fundamentals must resolve; got {:?}",
            d
        );
    }

    /// Invariant 2: legacy class-URI grants with empty fundamentals
    /// preserve M1 semantics (Case A — URI names a fundamental).
    #[test]
    fn legacy_class_uri_grant_still_resolves(
        fundamental in arb_fundamental(),
        action in arb_action(),
    ) {
        let agent = AgentId::new();
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec![action.clone()],
            resource: ResourceRef {
                uri: fundamental.as_str().to_string(),
            },
            fundamentals: vec![], // legacy — empty triggers URI-derivation
            descends_from: None,
            delegable: false,
            issued_at: Utc::now(),
            revoked_at: None,
        };
        let grants = [grant];
        // Empty `target_uri` skips Step 0 catalogue — same as the M1
        // step_4 fixtures.
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();
        let catalogue = StaticCatalogue::empty();

        let call = ToolCall {
            target_uri: String::new(),
            target_tags: vec![],
            target_agent: None,
            constraint_context: std::collections::HashMap::new(),
        };
        let manifest = Manifest {
            actions: vec![action],
            resource: vec![fundamental.as_str().to_string()],
            ..Default::default()
        };

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &manifest, &NoopMetrics);
        prop_assert!(
            d.is_allowed(),
            "legacy class-URI grant must still resolve (M1 regression guard); got {:?}",
            d
        );
    }

    /// Invariant 3: opaque instance-URI grants with **empty**
    /// fundamentals still fail — i.e. the P4.5 fix didn't accidentally
    /// admit the pre-P4.5 trap case.
    #[test]
    fn instance_uri_grant_without_fundamentals_still_fails(
        (kind, instance) in arb_kind_and_instance(),
        fundamental in arb_fundamental(),
        action in arb_action(),
    ) {
        let uri = format!("{kind}:{instance}");
        // Skip the degenerate case where `{kind}` happens to be a real
        // fundamental name (e.g. `"filesystem_object:..."` — Case A on the
        // Selector parse would then succeed on the prefix). The generator
        // caps `{kind}` at 11 chars, so in practice this rarely fires; the
        // assume keeps the invariant honest.
        prop_assume!(Fundamental::ALL.iter().all(|f| f.as_str() != kind));

        let agent = AgentId::new();
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec![action.clone()],
            resource: ResourceRef { uri: uri.clone() },
            fundamentals: vec![], // the trap case: empty
            descends_from: None,
            delegable: false,
            issued_at: Utc::now(),
            revoked_at: None,
        };
        let grants = [grant];
        let mut catalogue = StaticCatalogue::empty();
        catalogue.seed(None, &uri);
        let consents = ConsentIndex::empty();
        let template_gated = std::collections::HashSet::new();

        let call = ToolCall {
            target_uri: uri.clone(),
            target_tags: vec![uri.clone()],
            target_agent: None,
            constraint_context: std::collections::HashMap::new(),
        };
        let manifest = Manifest {
            actions: vec![action],
            resource: vec![fundamental.as_str().to_string()],
            ..Default::default()
        };

        let ctx = build_ctx(agent, &grants, call, &catalogue, &consents, &template_gated);
        let d = check(&ctx, &manifest, &NoopMetrics);
        prop_assert!(
            !d.is_allowed(),
            "opaque instance-URI with empty fundamentals must still be denied; got {:?}",
            d
        );
        // Specifically, the denial should land at Step 3 Match (no grant
        // covers the required reach) — the grant's fundamentals are
        // empty so `covers(fundamental, _)` returns false for every
        // fundamental.
        prop_assert_eq!(
            d.failed_step(),
            Some(FailedStep::Match),
            "expected denial at Step 3 Match"
        );
    }
}
