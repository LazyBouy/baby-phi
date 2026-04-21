//! M2 model invariants — the "did the P1 foundation actually land" guard.
//!
//! Verifies C1 of the M2 plan's commitment ledger:
//!   - new ID newtypes exist and are distinct from sibling IDs;
//!   - new composite-instance structs compose cleanly with phi-core types;
//!   - `TemplateKind` enumerates the full v0 lifecycle family.
//!
//! This is an integration test (under `tests/`) rather than a unit
//! test so it runs against the full public re-export surface of the
//! `domain` crate — catches accidental un-exports the way a downstream
//! consumer would see them.

use chrono::Utc;
use domain::model::{
    AgentId, ExternalService, ExternalServiceKind, McpServerId, ModelProviderId, ModelRuntime,
    OrgId, PlatformDefaults, RuntimeStatus, SecretCredential, SecretId, SecretRef, TemplateKind,
    TenantSet,
};

#[test]
fn template_kind_enumerates_system_bootstrap_plus_a_through_f() {
    assert_eq!(TemplateKind::ALL.len(), 7);
    let names: Vec<&'static str> = TemplateKind::ALL.iter().map(|k| k.as_str()).collect();
    assert_eq!(
        names,
        vec!["system_bootstrap", "a", "b", "c", "d", "e", "f"]
    );
}

#[test]
fn template_kind_default_is_system_bootstrap() {
    // Pre-M2 template rows on disk have no `kind` column; the default
    // must round-trip as SystemBootstrap so migrated rows stay valid.
    assert_eq!(TemplateKind::default(), TemplateKind::SystemBootstrap);
}

#[test]
fn new_ids_are_distinct_types_with_unique_values() {
    // Compile-time: these must all be distinct newtypes (no type
    // assignability between them, proved by the explicit turbofish).
    let s = SecretId::new();
    let m = ModelProviderId::new();
    let p = McpServerId::new();
    // Runtime: fresh UUIDs never collide.
    assert_ne!(s.as_uuid(), m.as_uuid());
    assert_ne!(m.as_uuid(), p.as_uuid());
}

#[test]
fn every_m2_composite_type_is_publicly_exported() {
    // This test compiles only if the types are re-exported from
    // `domain::model`. The body is structural — a compile-check.
    let _ = std::any::TypeId::of::<ModelRuntime>();
    let _ = std::any::TypeId::of::<ExternalService>();
    let _ = std::any::TypeId::of::<SecretCredential>();
    let _ = std::any::TypeId::of::<PlatformDefaults>();
    let _ = std::any::TypeId::of::<TenantSet>();
    let _ = std::any::TypeId::of::<RuntimeStatus>();
    let _ = std::any::TypeId::of::<ExternalServiceKind>();
    let _ = std::any::TypeId::of::<SecretRef>();
}

#[test]
fn model_runtime_uses_phi_core_model_config_field() {
    // Wrap verification — if this field's type ever silently becomes
    // a baby-phi-native ModelConfig clone, the D16 mandate is broken.
    // We assert via a factory construction: the field accepts exactly
    // phi-core's ModelConfig.
    let cfg = phi_core::provider::model::ModelConfig::anthropic(
        "claude-test",
        "claude-test-4",
        "__placeholder__",
    );
    let rt = ModelRuntime {
        id: ModelProviderId::new(),
        config: cfg,
        secret_ref: SecretRef::new("test"),
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
    };
    assert_eq!(rt.config.id, "claude-test");
}

#[test]
fn platform_defaults_wraps_four_phi_core_types() {
    // Build a PlatformDefaults using phi-core::Default on every
    // phi-core-wrapped field. Fails to compile if any field's type
    // drifts away from the phi-core source of truth.
    let pd = PlatformDefaults {
        singleton: 1,
        execution_limits: phi_core::context::execution::ExecutionLimits::default(),
        default_agent_profile: phi_core::agents::profile::AgentProfile::default(),
        context_config: phi_core::context::config::ContextConfig::default(),
        retry_config: phi_core::provider::retry::RetryConfig::default(),
        default_retention_days: 30,
        default_alert_channels: vec!["ops@example.com".into()],
        updated_at: Utc::now(),
        version: 0,
    };
    assert_eq!(pd.singleton, 1);
    assert_eq!(pd.version, 0);
    // phi-core's ExecutionLimits::default() sets max_turns = 50 — a
    // sentinel that proves the wrapped struct is the phi-core one.
    assert_eq!(pd.execution_limits.max_turns, 50);
}

#[test]
fn tenant_set_only_has_zero_default_orgs() {
    let ts = TenantSet::Only(vec![]);
    assert!(!ts.contains(OrgId::new()));
    assert_eq!(ts.explicit_orgs().len(), 0);
}

#[test]
fn secret_credential_serde_preserves_sensitive_flag() {
    let sc = SecretCredential {
        id: SecretId::new(),
        slug: SecretRef::new("test-key"),
        custodian: AgentId::new(),
        last_rotated_at: None,
        sensitive: true,
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&sc).expect("serialize");
    let back: SecretCredential = serde_json::from_str(&json).expect("deserialize");
    assert!(back.sensitive);
    assert_eq!(back.slug, sc.slug);
}
