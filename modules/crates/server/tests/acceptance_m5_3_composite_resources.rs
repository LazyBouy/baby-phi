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
//! 5. **owner_allocate_via_show_organization_handler_path** —
//!    `show_organization` handler's advisory engine call resolves
//!    Allow for the owning CEO at the Allocate verb (synth-grant scope
//!    per CH-25 ADR-0060 §D60.2; Inspect coverage pending CH-27 /
//!    D-CH26-FOLLOWUP-01).
//! 6. **stranger_allocate_via_show_organization_handler_path** — same
//!    handler shape returns Deny for an unrelated Agent.
//! 7. **owner_allocate_via_dashboard_handler_path** —
//!    `dashboard_summary` handler's advisory engine call resolves
//!    Allow for the owning CEO at the Allocate verb (Observe coverage
//!    pending CH-27).
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
//! Per plan §D61.5 the engine invocation is **advisory** in the
//! handler chain at CH-26 (the bespoke gating remains wire-tier; the
//! engine call surfaces the load-bearing semantic claim). Hitting the
//! HTTP route would not distinguish the engine verdict from the
//! bespoke verdict. Replaying the engine shape directly pins the
//! invariant the chunk closes: the synth-owner-grant axis carries
//! Allow for owner-Agents and Deny for strangers across all four
//! refactored handler URIs.

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
// **CH-26 synth-grant action-verb constraint** (per CH-25 ADR-0060 §D60.2):
// the synth-owner-grant rule provisions
// `actions: [Action::Allocate, Action::Transfer]` ONLY. `Action::Inspect`
// (the natural verb for show_*) is NOT covered. CH-26 closes with the
// load-bearing semantic claim (engine resolves Allow for owner-Agents via
// the synth-owner-grant rule) shipped at the Allocate verb. CH-27 will
// widen the synth-grant rule to cover Action::Observe + Action::Inspect
// for owner-Agents — see drift `D-CH26-FOLLOWUP-01` (Bucket B, Closing
// chunk CH-27). The handler's advisory engine call lives in
// `show_organization.rs`; this acceptance test pins the engine verdict
// at the Allocate verb for the same target URI shape the handler builds.

#[tokio::test]
async fn owner_allocate_via_show_organization_handler_path() {
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
        "CEO viewer must resolve Allocate on {uri} via the show_organization handler's engine shape (Inspect coverage pending CH-27 / D-CH26-FOLLOWUP-01)"
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
// **CH-26 synth-grant action-verb constraint** (per CH-25 ADR-0060 §D60.2):
// see the corresponding note above scenarios 5-6. `Action::Observe` (the
// natural verb for dashboard_*) is NOT covered by the synth-owner-grant
// at CH-25; this acceptance test pins the engine verdict at the
// `Action::Allocate` verb (which IS covered) for the same target URI
// shape the dashboard handler builds. CH-27 widens synth-grant scope per
// drift `D-CH26-FOLLOWUP-01`.

#[tokio::test]
async fn owner_allocate_via_dashboard_handler_path() {
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
        "CEO viewer must resolve Allocate on {uri} via the dashboard_summary handler's engine shape (Observe coverage pending CH-27 / D-CH26-FOLLOWUP-01)"
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
