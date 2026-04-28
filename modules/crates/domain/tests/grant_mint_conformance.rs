//! CH-06 P3 — grant-mint URI conformance.
//!
//! ADR-0036 §D36.5 commits to "semantic continuity" for the 9 M1 grant-mint
//! call sites: each issues a `ResourceRef.uri: String` whose shape must
//! continue to parse via the legacy fast-path (`Selector::Any` /
//! `Selector::Exact` / `Selector::Prefix` / `Selector::KindTag`) under the
//! new grammar parser. This file pins the conformance — one test per URI
//! shape asserting that:
//!
//! 1. `parse_selector_or_uri(uri)` returns the expected legacy variant.
//! 2. The resulting selector matches a target with the same URI.
//! 3. The selector does NOT match a target with a different URI (when
//!    applicable — `Any` matches everything by design).
//!
//! Plus one cross-engine test that exercises a NEW-grammar selector
//! (`tags contains <namespace_tag> AND tags contains #kind:<name>`)
//! through the full evaluator.

use domain::permissions::{
    parse_selector_or_uri, BoolExpr, NoopSetRefRegistry, Predicate, Selector, Tag,
};

/// Helper — assert a URI parses as `Selector::Exact(uri)` and the round-trip
/// match works.
fn assert_exact_round_trip(uri: &str) {
    let s = parse_selector_or_uri(uri);
    assert_eq!(
        s,
        Selector::Exact(uri.to_string()),
        "uri {} parsed wrong",
        uri
    );
    assert!(
        s.matches(uri, &[]),
        "Exact({}) must match its own target",
        uri
    );
    assert!(
        !s.matches("different:uri", &[]),
        "Exact({}) must not match a different target",
        uri
    );
}

// ---- The 9 grant-mint call-site URI shapes ---------------------------------

#[test]
fn secrets_add_secret_uri_parses_as_exact() {
    // secrets/add.rs writes `secret:<slug>` (e.g. `secret:anthropic-api-key`)
    assert_exact_round_trip("secret:anthropic-api-key");
}

#[test]
fn mcp_servers_register_mcp_server_uri_parses_as_exact() {
    // mcp_servers/register.rs writes `mcp_server:<id>`
    assert_exact_round_trip("mcp_server:abcd1234-5678-90ef");
}

#[test]
fn model_providers_register_provider_uri_parses_as_exact() {
    // model_providers/register.rs writes `provider:<id>`
    assert_exact_round_trip("provider:openai-default");
}

#[test]
fn orgs_create_org_scope_uri_parses_as_exact() {
    // orgs/create.rs writes `org:<uuid>` for the CEO grant
    assert_exact_round_trip("org:00000000-0000-0000-0000-000000000abc");
}

#[test]
fn orgs_create_system_root_parses_as_exact() {
    // orgs/create.rs CEO bootstrap also references `system:root`. The
    // engine's resolve_grant special-cases this URI to admit every
    // fundamental — the selector still parses as Exact for the matcher.
    assert_exact_round_trip("system:root");
}

#[test]
fn secrets_reveal_test_fixture_uri_parses_as_exact() {
    // secrets/reveal.rs test fixtures use the same secret:<slug> shape
    // as the production secrets/add path.
    assert_exact_round_trip("secret:reveal-test-key");
}

#[test]
fn projects_create_project_uri_parses_as_exact() {
    // projects/create.rs writes `project:<id>` for the AR resource slot
    assert_exact_round_trip("project:proj-website-redesign");
}

#[test]
fn mcp_servers_archive_uri_parses_as_exact() {
    // mcp_servers/archive.rs (and patch_tenants.rs) reference the same
    // mcp_server:<id> shape as register.rs.
    assert_exact_round_trip("mcp_server:archive-target-id");
}

#[test]
fn defaults_put_provider_uri_parses_as_exact() {
    // defaults/put.rs (and model_providers/archive.rs) reference the
    // same provider:<id> shape as register.rs.
    assert_exact_round_trip("provider:default-fallback-provider");
}

// ---- Legacy non-Exact shapes (also part of the M1 baseline) ----------------

#[test]
fn star_uri_parses_as_any() {
    let s = parse_selector_or_uri("*");
    assert_eq!(s, Selector::Any);
    assert!(s.matches("anything:goes", &["#kind:any".into()]));
}

#[test]
fn double_star_suffix_uri_parses_as_prefix() {
    let s = parse_selector_or_uri("filesystem:/workspace/**");
    assert_eq!(s, Selector::Prefix("filesystem:/workspace/".into()));
    assert!(s.matches("filesystem:/workspace/main.rs", &[]));
    assert!(!s.matches("filesystem:/other/main.rs", &[]));
}

#[test]
fn kind_tag_uri_parses_as_kind_tag() {
    let s = parse_selector_or_uri("#kind:memory");
    assert_eq!(s, Selector::KindTag("memory".into()));
    assert!(s.matches("memory:m-1", &["#kind:memory".into()]));
    assert!(!s.matches("memory:m-1", &["#kind:session".into()]));
}

// ---- Cross-engine: NEW grammar selector through full evaluate() -----------

#[test]
fn new_grammar_selector_evaluates_through_full_pipeline() {
    use domain::permissions::parse_selector;

    // A worked-example-style selector: AND of two contains predicates.
    let s = parse_selector("tags contains org:acme AND tags contains #kind:session")
        .expect("parse new-grammar selector");

    // Smoke-check the AST shape (BoolExpr::And of two Pred(Contains(...))).
    let Selector::Bool(b) = &s else {
        panic!("expected Bool top-level, got {:?}", s);
    };
    let BoolExpr::And(parts) = b.as_ref() else {
        panic!("expected AND, got {:?}", b);
    };
    assert_eq!(parts.len(), 2);
    assert!(matches!(
        &parts[0],
        BoolExpr::Pred(Predicate::Contains(Tag::Namespace(ns, _))) if ns == "org"
    ));
    assert!(matches!(
        &parts[1],
        BoolExpr::Pred(Predicate::Contains(Tag::Reserved(ns, Some(v))))
            if ns == "kind" && v == "session"
    ));

    // Match against a target with both required tags — Allowed.
    let registry = &NoopSetRefRegistry;
    assert!(
        s.evaluate(
            "session:s-9831",
            &["org:acme".into(), "#kind:session".into()],
            registry,
        ),
        "AND of two contains must match when both tags are present"
    );

    // Match against a target missing one — Denied.
    assert!(
        !s.evaluate("session:s-9831", &["org:acme".into()], registry,),
        "AND of two contains must NOT match when #kind:session is missing"
    );
    assert!(
        !s.evaluate("session:s-9831", &["#kind:session".into()], registry,),
        "AND of two contains must NOT match when org:acme is missing"
    );
}
