//! Property tests for M2/P7: the `PlatformDefaults` non-retroactive
//! invariant + factory-baseline + version-monotonicity contract.
//!
//! Three invariants:
//!
//! 1. **Non-retroactive.** Writing `PlatformDefaults` does NOT mutate
//!    any pre-existing graph state (orgs, agents, grants, ARs,
//!    catalogue). The `platform_defaults` singleton is the ONLY row
//!    that changes. This is the ADR-0019 invariant pinned
//!    structurally — M3's org-creation wizard consumes the defaults
//!    at creation time only; existing orgs keep their snapshot.
//!
//! 2. **Factory baseline is phi-core's canonical default.** Calling
//!    `PlatformDefaults::factory(now)` produces a struct whose four
//!    phi-core-wrapped fields are identical to each phi-core type's
//!    `Default::default()`. This is the "phi-core is the single
//!    source of truth" contract — bumping phi-core automatically
//!    bumps phi's factory without a migration.
//!
//! 3. **Version is monotonic under put.** Repeatedly calling
//!    `put_platform_defaults` with bumped versions produces a row
//!    whose final version equals the last put's version.

#![cfg(feature = "in-memory-repo")]

use chrono::Utc;
use proptest::prelude::*;

use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, Organization};
use domain::model::PlatformDefaults;
use domain::repository::Repository;

// ----------------------------------------------------------------------------
// Invariant 1 — non-retroactive (writing PlatformDefaults touches NOTHING else)
// ----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn put_platform_defaults_does_not_mutate_existing_graph(
        org_count in 0usize..5,
        agent_count in 0usize..5,
        retention_days in 0u32..365,
        max_turns in 1usize..200,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        rt.block_on(async move {
            let repo = InMemoryRepository::new();

            // Seed an initial graph: N orgs + M agents. Snapshot
            // everything we care about BEFORE the PUT.
            let mut org_ids = Vec::<OrgId>::new();
            for i in 0..org_count {
                let o = Organization {
                    id: OrgId::new(),
                    display_name: format!("org-{i}"),
                    vision: None,
                    mission: None,
                    consent_policy: domain::model::ConsentPolicy::Implicit,
                    audit_class_default: domain::audit::AuditClass::Logged,
                    authority_templates_enabled: vec![],
                    defaults_snapshot: None,
                    default_model_provider: None,
                    system_agents: vec![],
                    created_at: Utc::now(),
                };
                repo.create_organization(&o).await.unwrap();
                org_ids.push(o.id);
            }
            let mut agent_ids = Vec::<AgentId>::new();
            for i in 0..agent_count {
                let a = Agent {
                    id: AgentId::new(),
                    kind: AgentKind::Human,
                    display_name: format!("agent-{i}"),
                    owning_org: None,
                    role: None,
                    created_at: Utc::now(),
                };
                repo.create_agent(&a).await.unwrap();
                agent_ids.push(a.id);
            }

            // Snapshot before.
            let orgs_before: Vec<_> = {
                let mut v = vec![];
                for id in &org_ids {
                    v.push(repo.get_organization(*id).await.unwrap());
                }
                v
            };
            let agents_before: Vec<_> = {
                let mut v = vec![];
                for id in &agent_ids {
                    v.push(repo.get_agent(*id).await.unwrap());
                }
                v
            };

            // PUT PlatformDefaults. Bump a couple of fields so we'd
            // definitely notice if they leaked onto an org.
            let mut pd = PlatformDefaults::factory(Utc::now());
            pd.default_retention_days = retention_days;
            pd.execution_limits.max_turns = max_turns;
            pd.version = 1;
            repo.put_platform_defaults(&pd).await.unwrap();

            // Snapshot after — every existing org/agent must be
            // byte-identical (serde round-trip guards against silent
            // mutation of nested fields).
            for (before_opt, id) in orgs_before.iter().zip(org_ids.iter()) {
                let after_opt = repo.get_organization(*id).await.unwrap();
                prop_assert_eq!(
                    serde_json::to_value(before_opt).unwrap(),
                    serde_json::to_value(&after_opt).unwrap(),
                    "org {} mutated by PlatformDefaults write", id
                );
            }
            for (before_opt, id) in agents_before.iter().zip(agent_ids.iter()) {
                let after_opt = repo.get_agent(*id).await.unwrap();
                prop_assert_eq!(
                    serde_json::to_value(before_opt).unwrap(),
                    serde_json::to_value(&after_opt).unwrap(),
                    "agent {} mutated by PlatformDefaults write", id
                );
            }

            // And the defaults row reflects the write.
            let after = repo.get_platform_defaults().await.unwrap().unwrap();
            prop_assert_eq!(after.version, 1);
            prop_assert_eq!(after.default_retention_days, retention_days);
            prop_assert_eq!(after.execution_limits.max_turns, max_turns);
            Ok(())
        }).unwrap();
    }
}

// ----------------------------------------------------------------------------
// Invariant 2 — factory baseline equals phi-core defaults
// ----------------------------------------------------------------------------

// This test is authoritative for the "factory baseline == phi-core
// defaults" contract. If phi-core's `::default()` for any of the four
// wrapped types (ExecutionLimits / AgentProfile / ContextConfig /
// RetryConfig) changes, this test will catch the drift. If phi-core
// adds a new NON-DETERMINISTIC default field (like the existing
// `AgentProfile::profile_id` UUID), update the skip logic inside the
// test body — don't weaken the invariant by dropping the check
// wholesale.
proptest! {
    #![proptest_config(ProptestConfig { cases: 8, ..ProptestConfig::default() })]

    #[test]
    fn factory_baseline_matches_phi_core_defaults(offset_secs in 0i64..86_400) {
        // `now` varies across runs; every other field must be phi-core
        // ::default() verbatim.
        let now = Utc::now() + chrono::Duration::seconds(offset_secs);
        let pd = PlatformDefaults::factory(now);
        let json = serde_json::to_value(&pd).unwrap();

        let el_default = serde_json::to_value(
            phi_core::context::execution::ExecutionLimits::default()
        ).unwrap();
        let ap_default = serde_json::to_value(
            phi_core::agents::profile::AgentProfile::default()
        ).unwrap();
        let cc_default = serde_json::to_value(
            phi_core::context::config::ContextConfig::default()
        ).unwrap();
        let rc_default = serde_json::to_value(
            phi_core::provider::retry::RetryConfig::default()
        ).unwrap();

        prop_assert_eq!(&json["execution_limits"], &el_default);
        prop_assert_eq!(&json["retry_config"], &rc_default);
        // AgentProfile carries a UUID profile_id that phi-core regenerates
        // per Default::default() — so equality is field-wise modulo that
        // id. Verify the shape but skip the non-deterministic field.
        let mut ap_actual = json["default_agent_profile"].clone();
        let mut ap_expected = ap_default;
        if let Some(o) = ap_actual.as_object_mut() {
            o.remove("profile_id");
        }
        if let Some(o) = ap_expected.as_object_mut() {
            o.remove("profile_id");
        }
        prop_assert_eq!(ap_actual, ap_expected);
        // Same treatment for ContextConfig if it carries any
        // non-deterministic default.
        prop_assert_eq!(&json["context_config"], &cc_default);

        prop_assert_eq!(pd.singleton, 1);
        prop_assert_eq!(pd.version, 0);
        prop_assert!(pd.default_alert_channels.is_empty());
    }
}

// ----------------------------------------------------------------------------
// Invariant 3 — put is monotonic (puts with ascending versions compose)
// ----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, ..ProptestConfig::default() })]

    #[test]
    fn successive_puts_record_highest_version(
        n_puts in 1u64..10,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        rt.block_on(async move {
            let repo = InMemoryRepository::new();
            for v in 1..=n_puts {
                let mut pd = PlatformDefaults::factory(Utc::now());
                pd.version = v;
                repo.put_platform_defaults(&pd).await.unwrap();
            }
            let after = repo.get_platform_defaults().await.unwrap().unwrap();
            prop_assert_eq!(after.version, n_puts);
            Ok(())
        }).unwrap();
    }
}
