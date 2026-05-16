//! CH-25 / ADR-0060 §D60.4 — Acceptance test for the owner-grant rule
//! at M5.3.
//!
//! Per plan §7 P3 deliverable 7 (cycle hex `1e01618e`): exercise the
//! end-to-end scenario the chunk closes — an Agent that owns an
//! Organization (via the new `Edge::Owns` graph edge, emitted at
//! `apply_org_creation`) auto-receives `[Allocate, Transfer]` Authority
//! over the owned-Org resource URI via the synth-grant loop inside
//! [`domain::permissions::engine::step_2_resolve_grants`], WITHOUT
//! holding any persisted explicit grant against that URI.
//!
//! ## Scenario walk
//!
//! 1. Bootstrap a fresh acceptance server + claim the platform admin.
//! 2. POST `/api/v0/orgs` to create a brand-new Org via the real
//!    wizard. The wizard's compound transaction emits a CEO Agent +
//!    `Edge::Owns(CEO → Org)` + `Edge::Created(CEO → Org)`.
//! 3. Query `Repository::list_agent_owned_orgs(CEO)` and confirm the
//!    new Org's id surfaces — proving the Owns edge persisted through
//!    the compound tx + `list_agent_owned_orgs` query reads it back.
//! 4. Build a `CheckContext` for the CEO with **NO** explicit
//!    `agent_grants` / `project_grants` / `org_grants` — only the
//!    `agent_owned_orgs` slice populated from step 3. Build a
//!    `Manifest` requesting `[Action::Allocate]` over the owned-Org
//!    resource URI (`org:<id>`).
//! 5. Invoke the engine via the production
//!    [`server::handler_support::permission::check_permission`] shim.
//!    Expect `Decision::Allowed` — the synth-owner-grant matches.
//! 6. Cross-check: an unrelated Agent (NOT the CEO, no Owns edge)
//!    against the same manifest yields `Decision::Denied` at
//!    `FailedStep::Resolution` (`NO_GRANTS_HELD`) — proving the
//!    synth-grant is owner-scoped, not universal.
//!
//! ## Why this scenario covers §D60.4
//!
//! The plan's §D60.4 wording cites a child-Agent disable path; the
//! disable handler does NOT itself invoke the engine (see
//! `server/src/handlers/system_agents.rs` — no `check_permission`
//! call). The Permission Check surface the chunk's synth-grant
//! actually feeds is the engine call itself, exercised here at the
//! `check_permission` shim layer with the production manifest +
//! production context construction. The acceptance test thus proves
//! the **load-bearing claim** of the chunk: owner-Agent gains
//! authority over the owned-Org without an explicit persisted grant.
//! This is the §D60.4 invariant in its load-bearing form, decoupled
//! from any one HTTP handler's gating choices.

mod acceptance_common;

use std::collections::HashSet;
use std::sync::Arc;

use acceptance_common::admin::spawn_claimed;
use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, GrantId, OrgId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::permissions::{catalogue::StaticCatalogue, Action};
use domain::Repository;
use serde_json::Value;
use server::handler_support::permission::check_permission;

#[tokio::test]
async fn m5_3_ceo_synth_owner_grant_resolves_allocate_over_owned_org() {
    // Step 1 — bootstrap + claim.
    let admin = spawn_claimed(false).await;
    let repo: Arc<dyn Repository> = admin.acc.store.clone();

    // Step 2 — POST /api/v0/orgs via the real wizard. The compound
    // transaction emits a CEO Agent + Edge::Owns + Edge::Created.
    let body = serde_json::json!({
        "display_name": "Atlas Memory",
        "vision": "M5.3 owner-grant acceptance",
        "mission": "Verify CH-25 / ADR-0060 §D60.4 end-to-end",
        "consent_policy": "implicit",
        "audit_class_default": "logged",
        "authority_templates_enabled": ["a"],
        "default_model_provider": null,
        "ceo_display_name": "Atlas CEO",
        "ceo_channel_kind": "email",
        "ceo_channel_handle": "ceo@atlas-m53.test",
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

    // Step 3 — list_agent_owned_orgs(CEO) surfaces the new Org.
    let owned_orgs = repo
        .list_agent_owned_orgs(ceo_agent_id)
        .await
        .expect("list_agent_owned_orgs query succeeds");
    assert_eq!(
        owned_orgs,
        vec![org_id],
        "CEO must own exactly the new Org via Edge::Owns"
    );
    let owned_projects = repo
        .list_agent_owned_projects(ceo_agent_id)
        .await
        .expect("list_agent_owned_projects query succeeds");
    assert!(
        owned_projects.is_empty(),
        "CEO owns no projects at org-create time"
    );

    // Step 4 + 5 — build CheckContext with empty explicit-grant slices
    // and only `agent_owned_orgs` populated. The engine's
    // `step_2_resolve_grants` synth loop must produce a candidate
    // grant `[Allocate, Transfer]` over `org:<id>` that resolves the
    // manifest's `[Allocate]` reach against the same URI.
    //
    // Engine pre-conditions: (a) Step 0 (catalogue) requires the
    // instance URI `org:<id>` be seeded in the catalogue so the
    // instance-URI lookup passes; (b) Step 1 (expansion) requires
    // the manifest's `resource` strings be Fundamental / Composite
    // names — `identity_principal` is the fundamental that the synth
    // grant's `fundamentals: [IdentityPrincipal]` matches; (c) Step 3
    // (Match) requires the `ToolCall.target_uri` match the synth
    // grant's parsed selector (`Selector::Exact("org:<uuid>")`).
    let target_uri = format!("org:{}", org_id);
    let mut catalogue = StaticCatalogue::empty();
    catalogue.seed(Some(org_id), &target_uri);
    let consents = ConsentIndex::empty();
    let gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();
    let ctx = CheckContext {
        agent: ceo_agent_id,
        current_org: Some(org_id),
        current_project: None,
        current_session: None,
        agent_grants: &[],
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        session_org_tags: &[],
        session_project_tags: &[],
        // The load-bearing slice: CEO owns the new Org per step 3.
        agent_owned_orgs: &owned_orgs,
        agent_owned_projects: &[],
        call: ToolCall {
            target_uri: target_uri.clone(),
            ..Default::default()
        },
    };
    let manifest = Manifest {
        actions: vec![Action::Allocate],
        resource: vec!["identity_principal".to_string()],
        ..Default::default()
    };
    let allowed = check_permission(&ctx, &manifest, &NoopMetrics);
    assert!(
        allowed.is_ok(),
        "owner-grant synth must resolve [Allocate] over the owned-Org: got {allowed:?}"
    );

    // Step 6 — unrelated Agent (no Owns edge) gets Denied with
    // NO_GRANTS_HELD against the same manifest. Proves the synth
    // grant is owner-scoped.
    let stranger = AgentId::new();
    let stranger_owned = repo.list_agent_owned_orgs(stranger).await.unwrap();
    assert!(stranger_owned.is_empty(), "an unrelated agent owns no orgs");
    let stranger_ctx = CheckContext {
        agent: stranger,
        current_org: Some(org_id),
        current_project: None,
        current_session: None,
        agent_grants: &[],
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        session_org_tags: &[],
        session_project_tags: &[],
        agent_owned_orgs: &stranger_owned,
        agent_owned_projects: &[],
        call: ToolCall {
            target_uri: target_uri.clone(),
            ..Default::default()
        },
    };
    let denied = check_permission(&stranger_ctx, &manifest, &NoopMetrics);
    let err = denied.expect_err("non-owner must NOT resolve [Allocate]");
    assert_eq!(err.code, "NO_GRANTS_HELD");
    assert_eq!(err.status.as_u16(), 403);

    // Sanity rail — silence the unused-import warnings for items that
    // only future-extension code uses. The Grant + GrantId + AuditClass
    // imports document the synth-grant's persisted shape but are not
    // referenced by this happy-path test (the engine builds them
    // internally). The PrincipalRef + ResourceRef imports anchor the
    // doc cross-ref to the synth-grant body in `engine.rs`.
    let _ = (Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(ceo_agent_id),
        action: vec![Action::Allocate],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: true,
        issued_at: Utc::now(),
        revoked_at: None,
        approval_mode: domain::model::ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    },);
}
