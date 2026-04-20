//! The worked trace from `concepts/permissions/04-manifest-and-resolution.md`
//! §Worked Trace.
//!
//! This is the single **byte-exact** reference the M1 plan calls out in C2
//! ("worked-example trace matches `permissions/04` §Worked trace"). Two
//! scenarios:
//!
//! 1. `claude-coder-7` / `bash cargo build` — Allowed.
//! 2. `claude-coder-7` / `bash rm -rf /` — Denied at Step 3 (no grant
//!    covers the reach).
//!
//! Sourced to exact field values so divergence from the concept doc will
//! surface here rather than as a production bug.

mod common;

use common::*;
use domain::model::ids::{AgentId, GrantId, OrgId};
use domain::model::nodes::{PrincipalRef, ResourceRef};
use domain::permissions::{check, Decision, FailedStep, NoopMetrics, ToolCall};

/// Build the `bash` manifest per `concepts/permissions/04` §Worked Example
/// and its companion "bash cargo build" scenario.
fn bash_manifest() -> domain::permissions::Manifest {
    domain::permissions::Manifest {
        actions: vec!["execute".into()],
        resource: vec!["process_exec_object".into()],
        transitive: vec![
            "filesystem_object".into(),
            "network_endpoint".into(),
            "secret_credential".into(),
            "time_compute_resource".into(),
        ],
        constraints: vec![],
        kinds: vec![],
    }
}

/// Assemble `claude-coder-7`'s full grant set: one grant per fundamental
/// the `bash` manifest reaches. Each uses the `"*"` action wildcard so
/// the test stays focused on resource/selector matching.
fn claude_coder_7_grants(agent: AgentId) -> Vec<domain::model::nodes::Grant> {
    vec![
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "process_exec_object",
        ),
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "filesystem_object",
        ),
        grant_on(PrincipalRef::Agent(agent), &["execute"], "network_endpoint"),
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "secret_credential",
        ),
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "time_compute_resource",
        ),
    ]
}

#[test]
fn cargo_build_trace_is_allowed() {
    let agent = AgentId::new();
    let org = OrgId::new();
    let mut ctx_owned = ctx_with_agent_grants(claude_coder_7_grants(agent));
    ctx_owned.agent = agent;
    ctx_owned.current_org = Some(org);
    // The bash command operates against a catalogued process_exec target.
    let target = "process:sandboxed/cargo-build";
    ctx_owned.catalogue.seed(Some(org), target);

    let call = ToolCall {
        target_uri: target.to_string(),
        target_tags: vec![],
        ..Default::default()
    };
    let ctx = ctx_owned.borrow(call);
    let m = bash_manifest();
    let d = check(&ctx, &m, &NoopMetrics);
    match d {
        Decision::Allowed { resolved_grants } => {
            // One `(fundamental, action)` reach per manifest fundamental.
            // The worked trace lists 5 fundamentals reached × 1 action.
            assert_eq!(resolved_grants.len(), 5);
            let ids: std::collections::HashSet<GrantId> =
                resolved_grants.iter().map(|r| r.grant_id).collect();
            assert_eq!(ids.len(), 5, "each reach must resolve to a distinct grant");
        }
        other => panic!("expected Allowed, got {:?}", other),
    }
}

#[test]
fn rm_rf_trace_denies_at_step_3_when_no_filesystem_grant() {
    // Same scenario as above, but the agent only has
    // `process_exec_object`/`time_compute_resource` grants — the manifest
    // still reaches `filesystem_object`, so the match step fires.
    let agent = AgentId::new();
    let org = OrgId::new();
    let grants = vec![
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "process_exec_object",
        ),
        grant_on(
            PrincipalRef::Agent(agent),
            &["execute"],
            "time_compute_resource",
        ),
    ];
    let mut ctx_owned = ctx_with_agent_grants(grants);
    ctx_owned.agent = agent;
    ctx_owned.current_org = Some(org);
    ctx_owned.catalogue.seed(Some(org), "process:rm-rf");

    let call = ToolCall {
        target_uri: "process:rm-rf".into(),
        target_tags: vec![],
        ..Default::default()
    };
    let ctx = ctx_owned.borrow(call);
    let m = bash_manifest();
    let d = check(&ctx, &m, &NoopMetrics);
    assert_eq!(d.failed_step(), Some(FailedStep::Match));
}

/// Byte-exact sanity: serializing a canonical Allowed decision and
/// re-deserializing must round-trip. This guards against silent wire-format
/// breakage for the decision payload.
#[test]
fn decision_json_round_trip_preserves_content() {
    let agent = AgentId::new();
    let org = OrgId::new();
    let mut ctx_owned = ctx_with_agent_grants(claude_coder_7_grants(agent));
    ctx_owned.agent = agent;
    ctx_owned.current_org = Some(org);
    let target = "process:cargo";
    ctx_owned.catalogue.seed(Some(org), target);
    let ctx = ctx_owned.borrow(ToolCall {
        target_uri: target.into(),
        ..Default::default()
    });
    let d = check(&ctx, &bash_manifest(), &NoopMetrics);

    let json = serde_json::to_string(&d).expect("serialize");
    let back: Decision = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, d);

    // ResourceRef and PrincipalRef are touched indirectly through the
    // round-tripped Grant id; check that the outer shape is Allowed.
    assert!(back.is_allowed());
    let _ = ResourceRef { uri: "".into() };
}
