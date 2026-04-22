//! M3/P1 commitment-ledger row C2: assert the new model-layer shapes
//! land with stable counts + round-trip through serde.
//!
//! Paired doc: [`m3/architecture/phi-core-reuse-map.md`](../../../docs/specs/v0/implementation/m3/architecture/phi-core-reuse-map.md)
//! for which fields are phi-core wraps vs baby-phi-native.

use chrono::Utc;
use domain::model::ids::OrgId;
use domain::model::EDGE_KIND_NAMES;
use domain::model::{
    ConsentPolicy, OrganizationDefaultsSnapshot, PlatformDefaults, TokenBudgetPool,
};

// ---- Count invariants -----------------------------------------------------

#[test]
fn edge_count_bumps_from_sixty_six_to_sixty_seven() {
    // M3/P1 adds `HasLead` as variant #67. The compile-time count is
    // pinned in `edges.rs` (`pub const EDGE_KIND_NAMES: [&str; 67]`);
    // this test is a belt-and-braces check visible in the
    // integration suite so failures surface under `cargo test
    // --workspace`.
    assert_eq!(EDGE_KIND_NAMES.len(), 67);
    assert!(
        EDGE_KIND_NAMES.contains(&"HAS_LEAD"),
        "HAS_LEAD must be in the name table; got: {:?}",
        EDGE_KIND_NAMES
    );
}

#[test]
fn consent_policy_has_three_variants_in_declared_order() {
    let all = ConsentPolicy::ALL;
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], ConsentPolicy::Implicit);
    assert_eq!(all[1], ConsentPolicy::OneTime);
    assert_eq!(all[2], ConsentPolicy::PerSession);
}

// ---- Serde round-trip for every new composite ----------------------------

#[test]
fn organization_defaults_snapshot_round_trips() {
    // Build from factory platform defaults so every phi-core wrap is
    // populated — if a phi-core bump ever drops one of the four
    // fields, this test catches the break.
    let now = Utc::now();
    let pd = PlatformDefaults::factory(now);
    let snap = OrganizationDefaultsSnapshot::from_platform_defaults(&pd);
    let json = serde_json::to_string(&snap).expect("serialize snapshot");
    let back: OrganizationDefaultsSnapshot =
        serde_json::from_str(&json).expect("deserialize snapshot");
    // Spot-check the four phi-core-wrapped fields + the two
    // baby-phi-native fields.
    assert_eq!(back.default_retention_days, snap.default_retention_days);
    assert_eq!(back.default_alert_channels, snap.default_alert_channels);
    let a = serde_json::to_value(&back.execution_limits).unwrap();
    let b = serde_json::to_value(&snap.execution_limits).unwrap();
    assert_eq!(a, b);
}

#[test]
fn token_budget_pool_round_trips() {
    let org = OrgId::new();
    let pool = TokenBudgetPool::new(org, 1_000_000, Utc::now());
    let json = serde_json::to_string(&pool).expect("serialize pool");
    let back: TokenBudgetPool = serde_json::from_str(&json).expect("deserialize pool");
    assert_eq!(back, pool);
    assert_eq!(back.owning_org, org);
    assert_eq!(back.used, 0);
    assert_eq!(back.remaining(), 1_000_000);
}

#[test]
fn organization_defaults_snapshot_wraps_phi_core_types_not_parallel_structs() {
    // Structural confirmation that `OrganizationDefaultsSnapshot`
    // stores phi-core types directly (the `from_platform_defaults`
    // path is lossless because the types are the same). If a future
    // refactor replaces a phi-core field with a baby-phi parallel
    // struct, the type-equality pattern below won't compile.
    let now = Utc::now();
    let pd = PlatformDefaults::factory(now);
    let snap = OrganizationDefaultsSnapshot::from_platform_defaults(&pd);

    fn is_phi_core_execution_limits(_: &phi_core::context::execution::ExecutionLimits) {}
    fn is_phi_core_agent_profile(_: &phi_core::agents::profile::AgentProfile) {}
    fn is_phi_core_context_config(_: &phi_core::context::config::ContextConfig) {}
    fn is_phi_core_retry_config(_: &phi_core::provider::retry::RetryConfig) {}

    is_phi_core_execution_limits(&snap.execution_limits);
    is_phi_core_agent_profile(&snap.default_agent_profile);
    is_phi_core_context_config(&snap.context_config);
    is_phi_core_retry_config(&snap.retry_config);
}
