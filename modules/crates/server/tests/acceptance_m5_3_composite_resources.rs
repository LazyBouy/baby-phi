//! CH-26 / ADR-0061 §D61.4 — Acceptance test for the unified resource
//! model: Organization + Project as Composite resources, with the
//! Permission Check engine resolving Allow for owner-Agents (via the
//! CH-25 synth-owner-grant rule) and Deny for cross-org strangers.
//!
//! Per plan §7 P3 + §11 Audit A claim 8: ≥ 8 scenarios across
//! engine-level invariants (4) + per-handler PASS/FAIL (≥ 4).
//!
//! ## Scenario topology
//!
//! 1. **catalogue_seed_succeeds** — `apply_org_creation` seeds the
//!    `org:<id>` catalogue entry at-creation-time (CH-26 §D61.3).
//! 2. **applies_to_composite_for_org_and_project** — pure-engine smoke
//!    that `Action::Allocate.applies_to_composite(Composite::Organization
//!    Object) == true` and same for `ProjectObject` (constituent-union
//!    via `IdentityPrincipal`).
//! 3. **engine_resolves_allocate_over_owned_org** — engine round-trip
//!    against `org:<O>` with owner-Agent context returns `Allowed`.
//! 4. **stranger_denied_on_org** — engine returns Denied + handler
//!    shim maps to `NO_GRANTS_HELD` for an unrelated Agent.
//! 5. **owner_inspect_via_show_organization_handler_path** —
//!    `show_organization` handler's **blocking** engine call resolves
//!    Allow for the owning CEO at the Inspect verb (widened synth-grant
//!    scope per CH-27 / ADR-0062 §D62.2; the handler's engine call is
//!    blocking per ADR-0062 §D62.1).
//! 6. **stranger_allocate_via_show_organization_handler_path** — same
//!    handler shape returns Deny for an unrelated Agent.
//! 7. **owner_observe_via_dashboard_handler_path** —
//!    `dashboard_summary` handler's **blocking** engine call resolves
//!    Allow for the owning CEO at the Observe verb (widened synth-grant
//!    scope per CH-27 / ADR-0062 §D62.2).
//! 8. **stranger_allocate_via_dashboard_handler_path** — same handler
//!    shape returns Deny for an unrelated Agent.
//! 9. **owner_allocate_via_create_project_handler_path** —
//!    `create_project` handler's `is_allocate_allowed_on_org` advisory
//!    engine call returns Allow for the owning CEO on the parent org.
//! 10. **stranger_allocate_via_create_project_handler_path** — same
//!     handler's engine call returns Deny for an unrelated Agent.
//!
//! The "handler_path" scenarios reproduce the exact CheckContext +
//! Manifest shape each refactored handler builds. The handlers' advisory
//! invocation lives inside `server::platform::{orgs,projects}::*` and
//! is not directly observable from the acceptance crate; replaying the
//! same shape here proves the load-bearing semantic claim per
//! §D61.4 (5-8+).
//!
//! ## Why "handler_path" replay vs hitting HTTP
//!
//! Per CH-26 ADR-0061 §D61.5 the engine invocation was advisory in the
//! handler chain at CH-26 (the bespoke gating carried the wire-tier
//! rejection; the engine call surfaced the load-bearing semantic claim).
//! CH-27 / ADR-0062 §D62.1 tightens these invocations to **blocking**
//! (engine deny → HTTP 403 NO_GRANTS_HELD). These per-handler-path
//! scenarios replay the engine shape directly to pin the load-bearing
//! semantic claim. End-to-end HTTP 403-block scenarios live alongside
//! these (added at CH-27 / P3) — they hit the route and assert the
//! tightened wire-tier behaviour.

mod acceptance_common;

use std::collections::HashSet;
use std::sync::Arc;

use acceptance_common::admin::spawn_claimed;
use domain::model::ids::{AgentId, AuthRequestId, OrgId};
use domain::model::nodes::PrincipalRef;
use domain::permissions::catalogue::StaticCatalogue;
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::permissions::{Action, NOOP_SET_REF_REGISTRY};
use domain::Repository;
use serde_json::Value;
use server::handler_support::permission::check_permission;

// ---------------------------------------------------------------------------
// Fixture — wizard-create an Org + claim its OrgId + CEO AgentId.
// ---------------------------------------------------------------------------

struct OrgFixture {
    repo: Arc<dyn Repository>,
    org_id: OrgId,
    ceo_agent_id: AgentId,
}

async fn bootstrap_org_via_wizard() -> (acceptance_common::admin::ClaimedAdmin, OrgFixture) {
    let admin = spawn_claimed(false).await;
    let repo: Arc<dyn Repository> = admin.acc.store.clone();

    let body = serde_json::json!({
        "display_name": "Composite Resources Atlas",
        "vision": "M5.3 composite-resources acceptance",
        "mission": "Verify CH-26 / ADR-0061 §D61.4 end-to-end",
        "consent_policy": "implicit",
        "audit_class_default": "logged",
        "authority_templates_enabled": ["a"],
        "default_model_provider": null,
        "ceo_display_name": "Composite CEO",
        "ceo_channel_kind": "email",
        "ceo_channel_handle": "ceo@composite-m53.test",
        "token_budget": 1_000_000_u64,
    });
    let res = admin
        .authed_client
        .post(admin.url("/api/v0/orgs"))
        .json(&body)
        .send()
        .await
        .expect("POST /api/v0/orgs");
    assert_eq!(res.status().as_u16(), 201, "wizard must return 201 Created");
    let receipt: Value = res.json().await.expect("decode wizard receipt");
    let org_id = OrgId::from_uuid(
        uuid::Uuid::parse_str(receipt["org_id"].as_str().expect("org_id present"))
            .expect("org_id parses as uuid"),
    );
    let ceo_agent_id = AgentId::from_uuid(
        uuid::Uuid::parse_str(
            receipt["ceo_agent_id"]
                .as_str()
                .expect("ceo_agent_id present"),
        )
        .expect("ceo_agent_id parses as uuid"),
    );

    let fx = OrgFixture {
        repo,
        org_id,
        ceo_agent_id,
    };
    (admin, fx)
}

// ---------------------------------------------------------------------------
// CheckContext shape replicas — one per refactored handler's manifest.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn run_engine_check(
    agent: AgentId,
    target_uri: String,
    kind_tag_short: &str,
    action: Action,
    current_org: Option<OrgId>,
    agent_grants: &[domain::model::nodes::Grant],
    agent_owned_orgs: &[OrgId],
    agent_owned_projects: &[domain::model::ids::ProjectId],
) -> bool {
    let mut catalogue = StaticCatalogue::empty();
    catalogue.seed(current_org, &target_uri);
    let consents = ConsentIndex::empty();
    let gated: HashSet<AuthRequestId> = HashSet::new();
    let ctx = CheckContext {
        agent,
        current_org,
        current_project: None,
        current_session: None,
        agent_grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &gated,
        set_ref_registry: &NOOP_SET_REF_REGISTRY,
        session_org_tags: &[],
        session_project_tags: &[],
        agent_owned_orgs,
        agent_owned_projects,
        call: ToolCall {
            target_uri: target_uri.clone(),
            target_tags: vec![format!("{}:{}", kind_tag_short, target_uri)],
            ..Default::default()
        },
    };
    let manifest = Manifest {
        actions: vec![action],
        resource: vec!["identity_principal".to_string()],
        ..Default::default()
    };
    check_permission(&ctx, &manifest, &NoopMetrics).is_ok()
}

// ===========================================================================
// 1. catalogue_seed_succeeds
// ===========================================================================

#[tokio::test]
async fn catalogue_seed_succeeds_on_wizard_create() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let uri = format!("org:{}", fx.org_id);
    let present = fx
        .repo
        .catalogue_contains(Some(fx.org_id), &uri)
        .await
        .expect("catalogue_contains query succeeds");
    assert!(
        present,
        "apply_org_creation must seed the org's catalogue entry at-creation-time per ADR-0061 §D61.3"
    );
}

// ===========================================================================
// 2. applies_to_composite for new variants
// ===========================================================================

#[test]
fn applies_to_composite_for_organization_and_project_object() {
    use domain::model::composites::Composite;

    // Allocate is in the Authority category — universal across all
    // fundamentals; both new variants include IdentityPrincipal so
    // the verdict is true.
    assert!(
        Action::Allocate.applies_to_composite(Composite::OrganizationObject),
        "Action::Allocate must apply to Composite::OrganizationObject (IdentityPrincipal constituent)"
    );
    assert!(
        Action::Allocate.applies_to_composite(Composite::ProjectObject),
        "Action::Allocate must apply to Composite::ProjectObject (IdentityPrincipal constituent)"
    );
    // Inspect (Discovery category) — universal across all fundamentals.
    assert!(
        Action::Inspect.applies_to_composite(Composite::OrganizationObject),
        "Action::Inspect must apply to Composite::OrganizationObject (universal)"
    );
    assert!(
        Action::Inspect.applies_to_composite(Composite::ProjectObject),
        "Action::Inspect must apply to Composite::ProjectObject (universal)"
    );
    // Observe (Observability category) — universal.
    assert!(
        Action::Observe.applies_to_composite(Composite::OrganizationObject),
        "Action::Observe must apply to Composite::OrganizationObject (universal)"
    );
    assert!(
        Action::Observe.applies_to_composite(Composite::ProjectObject),
        "Action::Observe must apply to Composite::ProjectObject (universal)"
    );
}

// ===========================================================================
// 3-4. Engine-level Allow / Deny on org:<O> via synth-owner-grant
// ===========================================================================

#[tokio::test]
async fn engine_resolves_allocate_over_owned_org_for_ceo() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let uri = format!("org:{}", fx.org_id);
    let agent_grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(fx.ceo_agent_id))
        .await
        .expect("grants query");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(fx.ceo_agent_id)
        .await
        .expect("list_agent_owned_orgs query");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(fx.ceo_agent_id)
        .await
        .expect("list_agent_owned_projects query");
    let allowed = run_engine_check(
        fx.ceo_agent_id,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &agent_grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        allowed,
        "CEO (owner via Edge::Owns) must resolve Allocate on {uri} via the synth-owner-grant rule"
    );
}

#[tokio::test]
async fn stranger_denied_allocate_on_org() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let uri = format!("org:{}", fx.org_id);
    let stranger_grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(stranger))
        .await
        .expect("grants query");
    let stranger_owned_orgs = fx
        .repo
        .list_agent_owned_orgs(stranger)
        .await
        .expect("list_agent_owned_orgs query");
    let stranger_owned_projects = fx
        .repo
        .list_agent_owned_projects(stranger)
        .await
        .expect("list_agent_owned_projects query");
    let allowed = run_engine_check(
        stranger,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &stranger_grants,
        &stranger_owned_orgs,
        &stranger_owned_projects,
    );
    assert!(
        !allowed,
        "stranger Agent (no Owns edge) must NOT resolve Allocate on {uri}"
    );
}

// ===========================================================================
// 5-6. Per-handler shape — show_organization handler engine shape
// ===========================================================================
//
// **CH-27 synth-grant action-verb scope** (per ADR-0062 §D62.2): the
// synth-owner-grant rule provisions
// `actions: [Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]`
// — covering all 4 universal-applicability verbs (Authority + Discovery +
// Observability) per concept-doc 03 line 44. `Action::Inspect` (the natural
// verb for show_*) is now covered, so this acceptance test pins the engine
// verdict at the Inspect verb for the same target URI shape the
// `show_organization` handler builds. The handler's engine call is
// **blocking** as of CH-27 / ADR-0062 §D62.1 (engine deny → HTTP 403
// NO_GRANTS_HELD via `denial_to_api_error`).

#[tokio::test]
async fn owner_inspect_via_show_organization_handler_path() {
    let (admin, fx) = bootstrap_org_via_wizard().await;
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(fx.ceo_agent_id))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(fx.ceo_agent_id)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(fx.ceo_agent_id)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        fx.ceo_agent_id,
        uri.clone(),
        "organization",
        Action::Inspect,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        allowed,
        "CEO viewer must resolve Inspect on {uri} via the show_organization handler's engine shape (CH-27 / ADR-0062 §D62.2 widened scope)"
    );

    // CH-27 / ADR-0062 §D62.1 — HTTP-tier extension: the CEO is the
    // synth-grant owner of `org:<O>` (via Edge::Owns at `apply_org_creation`),
    // so the blocking gate at `show_organization` admits the CEO viewer
    // → HTTP 200. F4.b: the owner-grant wire-up is via the canonical
    // CH-25 production path (no explicit `seed_owner_grants` call needed
    // for the CEO because the synth-grant rule covers them; the helper
    // is consumed elsewhere for non-owner viewers).
    let ceo_client = acceptance_common::admin::authed_client_for(&admin, fx.ceo_agent_id)
        .expect("mint CEO client");
    let res = ceo_client
        .get(admin.url(&format!("/api/v0/orgs/{}", fx.org_id)))
        .send()
        .await
        .expect("GET /api/v0/orgs/:id as CEO");
    assert_eq!(
        res.status().as_u16(),
        200,
        "CEO (owner via synth-grant) must pass the CH-27 blocking gate on Inspect"
    );
}

#[tokio::test]
async fn stranger_allocate_via_show_organization_handler_path() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(stranger))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(stranger)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(stranger)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        stranger,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        !allowed,
        "stranger Agent must NOT resolve Allocate on {uri} via the show_organization handler's engine shape"
    );
}

// ===========================================================================
// 7-8. Per-handler shape — dashboard_summary handler engine shape
// ===========================================================================
//
// **CH-27 synth-grant action-verb scope** (per ADR-0062 §D62.2): see the
// corresponding note above scenarios 5-6. `Action::Observe` (the natural
// verb for dashboard_*) IS now covered by the widened synth-owner-grant
// rule. This acceptance test pins the engine verdict at the
// `Action::Observe` verb for the same target URI shape the dashboard
// handler builds. The handler's engine call is **blocking** as of CH-27
// / ADR-0062 §D62.1.

#[tokio::test]
async fn owner_observe_via_dashboard_handler_path() {
    let (admin, fx) = bootstrap_org_via_wizard().await;
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(fx.ceo_agent_id))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(fx.ceo_agent_id)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(fx.ceo_agent_id)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        fx.ceo_agent_id,
        uri.clone(),
        "organization",
        Action::Observe,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        allowed,
        "CEO viewer must resolve Observe on {uri} via the dashboard_summary handler's engine shape (CH-27 / ADR-0062 §D62.2 widened scope)"
    );

    // CH-27 / ADR-0062 §D62.1 — HTTP-tier extension: CEO via the
    // canonical CH-25 synth-grant path passes the blocking gate at
    // `org_dashboard` (Observe verb now covered by the widened scope).
    let ceo_client = acceptance_common::admin::authed_client_for(&admin, fx.ceo_agent_id)
        .expect("mint CEO client");
    let res = ceo_client
        .get(admin.url(&format!("/api/v0/orgs/{}/dashboard", fx.org_id)))
        .send()
        .await
        .expect("GET /api/v0/orgs/:id/dashboard as CEO");
    assert_eq!(
        res.status().as_u16(),
        200,
        "CEO (owner via synth-grant) must pass the CH-27 blocking gate on Observe"
    );
}

#[tokio::test]
async fn stranger_allocate_via_dashboard_handler_path() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(stranger))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(stranger)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(stranger)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        stranger,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        !allowed,
        "stranger Agent must NOT resolve Allocate on {uri} via the dashboard_summary handler's engine shape"
    );
}

// ===========================================================================
// 9-10. Per-handler shape — create_project Allocate gate on parent org
// ===========================================================================

#[tokio::test]
async fn owner_allocate_via_create_project_handler_path() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(fx.ceo_agent_id))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(fx.ceo_agent_id)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(fx.ceo_agent_id)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        fx.ceo_agent_id,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        allowed,
        "CEO (owner) must resolve Allocate on parent {uri} via the create_project handler's engine shape"
    );
}

#[tokio::test]
async fn stranger_allocate_via_create_project_handler_path() {
    let (_admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let uri = format!("org:{}", fx.org_id);
    let grants = fx
        .repo
        .list_grants_for_principal(&PrincipalRef::Agent(stranger))
        .await
        .expect("grants");
    let owned_orgs = fx
        .repo
        .list_agent_owned_orgs(stranger)
        .await
        .expect("owned_orgs");
    let owned_projects = fx
        .repo
        .list_agent_owned_projects(stranger)
        .await
        .expect("owned_projects");
    let allowed = run_engine_check(
        stranger,
        uri.clone(),
        "organization",
        Action::Allocate,
        Some(fx.org_id),
        &grants,
        &owned_orgs,
        &owned_projects,
    );
    assert!(
        !allowed,
        "stranger Agent must NOT resolve Allocate on parent {uri} via the create_project handler's engine shape"
    );
}

// ===========================================================================
// CH-27 / ADR-0062 §D62.1 — HTTP-tier blocking-gate scenarios
// ===========================================================================
//
// The 7 admin handlers tightened from advisory `.is_ok()` consumption to
// blocking `?`-propagation via `denial_to_api_error` (CH-25 wire
// convention). These scenarios hit the actual HTTP routes with a
// stranger-Agent session and assert the canonical 403 + `NO_GRANTS_HELD`
// envelope. Complementary to the engine-shape scenarios above which pin
// the load-bearing semantic claim; these pin the wire-tier closure.

/// HTTP 403-block — stranger calls `GET /api/v0/orgs/:id`.
/// Engine deny on Inspect resolves to 403 NO_GRANTS_HELD per
/// ADR-0062 §D62.1 + CH-25 wire convention.
#[tokio::test]
async fn unauthorized_actor_blocked_at_show_organization_returns_403() {
    let (admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let stranger_client = acceptance_common::admin::authed_client_for(&admin, stranger)
        .expect("mint stranger client");
    let res = stranger_client
        .get(admin.url(&format!("/api/v0/orgs/{}", fx.org_id)))
        .send()
        .await
        .expect("GET /api/v0/orgs/:id as stranger");
    assert_eq!(
        res.status().as_u16(),
        403,
        "stranger must be blocked at show_organization (CH-27 blocking gate)"
    );
    let err: Value = res.json().await.expect("decode 403 envelope");
    assert_eq!(
        err["code"].as_str(),
        Some("NO_GRANTS_HELD"),
        "blocking gate must canonicalise to NO_GRANTS_HELD per CH-25 wire convention"
    );
}

/// HTTP 403-block — stranger calls `GET /api/v0/orgs/:id/dashboard`.
#[tokio::test]
async fn unauthorized_actor_blocked_at_org_dashboard_returns_403() {
    let (admin, fx) = bootstrap_org_via_wizard().await;
    let stranger = AgentId::new();
    let stranger_client = acceptance_common::admin::authed_client_for(&admin, stranger)
        .expect("mint stranger client");
    let res = stranger_client
        .get(admin.url(&format!("/api/v0/orgs/{}/dashboard", fx.org_id)))
        .send()
        .await
        .expect("GET /api/v0/orgs/:id/dashboard as stranger");
    assert_eq!(
        res.status().as_u16(),
        403,
        "stranger must be blocked at org_dashboard (CH-27 blocking gate)"
    );
    let err: Value = res.json().await.expect("decode 403 envelope");
    assert_eq!(err["code"].as_str(), Some("NO_GRANTS_HELD"));
}

/// HTTP 403-block — stranger calls `GET /api/v0/projects/:id`. The
/// project is created via the canonical `spawn_claimed_with_org_and_project`
/// fixture so the BELONGS_TO + Owns + roster edges are all in place.
#[tokio::test]
async fn unauthorized_actor_blocked_at_project_detail_returns_403() {
    let project = acceptance_common::admin::spawn_claimed_with_org_and_project(false).await;
    let stranger = AgentId::new();
    let stranger_client =
        acceptance_common::admin::authed_client_for(&project.claimed_org.admin, stranger)
            .expect("mint stranger client");
    let res = stranger_client
        .get(project.url(&format!("/api/v0/projects/{}", project.project_id)))
        .send()
        .await
        .expect("GET /api/v0/projects/:id as stranger");
    assert_eq!(
        res.status().as_u16(),
        403,
        "stranger must be blocked at project_detail (CH-27 blocking gate)"
    );
    let err: Value = res.json().await.expect("decode 403 envelope");
    assert_eq!(err["code"].as_str(), Some("NO_GRANTS_HELD"));
}

/// HTTP 403-block — stranger calls `POST /api/v0/projects/:id/agents/:supervisee/supervisor`.
#[tokio::test]
async fn unauthorized_actor_blocked_at_set_agent_supervisor_returns_403() {
    let project = acceptance_common::admin::spawn_claimed_with_org_and_project(false).await;
    let stranger = AgentId::new();
    let supervisor = AgentId::new();
    let supervisee = AgentId::new();
    let stranger_client =
        acceptance_common::admin::authed_client_for(&project.claimed_org.admin, stranger)
            .expect("mint stranger client");
    let body = serde_json::json!({"supervisor_agent_id": supervisor.to_string()});
    let res = stranger_client
        .post(project.url(&format!(
            "/api/v0/projects/{}/agents/{}/supervisor",
            project.project_id, supervisee
        )))
        .json(&body)
        .send()
        .await
        .expect("POST set supervisor as stranger");
    assert_eq!(
        res.status().as_u16(),
        403,
        "stranger must be blocked at set_agent_supervisor (CH-27 blocking gate)"
    );
    let err: Value = res.json().await.expect("decode 403 envelope");
    assert_eq!(err["code"].as_str(), Some("NO_GRANTS_HELD"));
}
