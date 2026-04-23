//! Cross-page acceptance for M4 — pages 08/09/10/11 + dashboard
//! retroactive update (commitment C18).
//!
//! End-to-end flow:
//!
//! 1. Boot a claimed org via the M3 fixture (`spawn_claimed_with_org`).
//! 2. List agents on page 08 → baseline (CEO + 2 system agents).
//! 3. Create an LLM Intern on page 09 → agent + inbox + outbox +
//!    profile, audit event lands on the per-org chain.
//! 4. Edit the Intern's profile (bump `temperature`) on page 09.
//! 5. Create a Shape A project on page 10 with the Intern as lead.
//! 6. Read the project detail on page 11 as the CEO — roster carries
//!    the lead, `recent_sessions: []` (M4 placeholder; M5/C-M5-3).
//! 7. Apply an OKR patch on page 11 (create one objective + one KR).
//! 8. Re-fetch the dashboard → `agents_summary` + `projects_summary`
//!    reflect the new agent, new role, and the Shape A project.
//!
//! The test drives the full HTTP stack (axum + SurrealDB embedded RocksDB).
//! No direct repo pokes — every mutation goes through the wire.
//!
//! ## phi-core leverage
//!
//! Q1 direct imports: **0** in this file. The fixture types we touch
//! (`Agent`, `AgentRole`, `AgentKind`) are all baby-phi governance.
//! Q2 transitive: the create payload embeds `blueprint` (phi-core
//! `AgentProfile`) via serde — documented at M4/P5 ADR-0027 and the
//! page 09 architecture doc. Q3: `phi_core::Session` deliberately
//! deferred to M5 (C-M5-3).

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, InboxObject, Organization, OutboxObject, Project, ProjectShape, ProjectStatus,
};
use domain::repository::{AgentCreationPayload, ProjectCreationPayload, Repository};
use serde_json::{json, Value};
use std::sync::Arc;

/// Mint a client authenticating as the org's CEO (the "first Human"
/// agent per the M3 fixture). The CEO is a member of the org and so
/// passes every access gate (dashboard + project show), unlike the
/// bootstrap admin (platform-level, not a member).
fn ceo_client(org: &ClaimedOrg) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(&org.admin, org.ceo_agent_id)
        .expect("mint CEO session")
}

async fn list_agents(org: &ClaimedOrg) -> Value {
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = org
        .admin
        .authed_client
        .get(url)
        .send()
        .await
        .expect("GET agents");
    assert_eq!(res.status().as_u16(), 200);
    res.json().await.unwrap()
}

async fn create_intern(org: &ClaimedOrg, name: &str) -> AgentId {
    let body = json!({
        "display_name": name,
        "kind": "llm",
        "role": "intern",
        "blueprint": {
            "system_prompt": "you are a helper",
            "thinking_level": "medium",
            "temperature": 0.2
        },
        "parallelize": 1,
        "initial_execution_limits_override": null
    });
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = org
        .admin
        .authed_client
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201, "create: {:?}", res.text().await);
    let body: Value = res.json().await.unwrap();
    body["agent_id"]
        .as_str()
        .and_then(|s| s.parse::<uuid::Uuid>().ok())
        .map(AgentId::from_uuid)
        .expect("agent_id uuid string")
}

async fn update_profile_temperature(org: &ClaimedOrg, agent_id: AgentId, temperature: f64) {
    let body = json!({
        "blueprint": {
            "temperature": temperature
        }
    });
    let url = org.url(&format!("/api/v0/agents/{agent_id}/profile"));
    let res = org
        .admin
        .authed_client
        .patch(url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200, "patch: {:?}", res.text().await);
}

async fn create_shape_a_project(org: &ClaimedOrg, lead: AgentId, name: &str) -> ProjectId {
    let project_id = ProjectId::new();
    let body = json!({
        "project_id": project_id.to_string(),
        "name": name,
        "description": "cross-page acceptance fixture",
        "shape": "shape_a",
        "co_owner_org_id": null,
        "lead_agent_id": lead.to_string(),
        "member_agent_ids": [],
        "sponsor_agent_ids": [],
        "token_budget": null,
        "objectives": [],
        "key_results": [],
    });
    let url = org.url(&format!("/api/v0/orgs/{}/projects", org.org_id));
    let res = org
        .admin
        .authed_client
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status().as_u16(),
        201,
        "create project: {:?}",
        res.text().await
    );
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["outcome"].as_str(), Some("materialised"));
    project_id
}

async fn show_project_as_ceo(org: &ClaimedOrg, project_id: ProjectId) -> Value {
    let url = org.url(&format!("/api/v0/projects/{project_id}"));
    let res = ceo_client(org).get(url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    res.json().await.unwrap()
}

async fn patch_okrs(org: &ClaimedOrg, project_id: ProjectId, owner: AgentId) -> Value {
    let body = json!({
        "patches": [
            {
                "kind": "objective",
                "op":   "create",
                "payload": {
                    "objective_id":  "q2-recall",
                    "name":          "hit 0.85 recall",
                    "description":   "",
                    "status":        "active",
                    "owner":         owner.to_string(),
                    "key_result_ids": []
                }
            },
            {
                "kind": "key_result",
                "op":   "create",
                "payload": {
                    "kr_id":            "q2-recall-1",
                    "objective_id":     "q2-recall",
                    "name":             "recall",
                    "description":      "",
                    "measurement_type": "percentage",
                    "target_value":     { "kind": "percentage", "value": 0.85 },
                    "owner":            owner.to_string(),
                    "status":           "not_started"
                }
            }
        ]
    });
    let url = org.url(&format!("/api/v0/projects/{project_id}/okrs"));
    let res = ceo_client(org).patch(url).json(&body).send().await.unwrap();
    assert_eq!(
        res.status().as_u16(),
        200,
        "patch okrs: {:?}",
        res.text().await
    );
    res.json().await.unwrap()
}

async fn dashboard_as_ceo(org: &ClaimedOrg) -> Value {
    let url = org.url(&format!("/api/v0/orgs/{}/dashboard", org.org_id));
    let res = ceo_client(org).get(url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    res.json().await.unwrap()
}

// ---------------------------------------------------------------------------
// Cross-page acceptance — the full M4 happy path in one test.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_m4_happy_path_bootstrap_to_dashboard() {
    let org = spawn_claimed_with_org(false).await;

    // --- Baseline on page 08 -------------------------------------------------
    let baseline = list_agents(&org).await;
    let baseline_len = baseline["agents"].as_array().unwrap().len();
    assert_eq!(baseline_len, 3, "fresh fixture: CEO + 2 system agents = 3");

    // --- Create a new Intern on page 09 -------------------------------------
    let intern_id = create_intern(&org, "iris").await;

    // Page 08 should now see 4.
    let after_create = list_agents(&org).await;
    assert_eq!(after_create["agents"].as_array().unwrap().len(), 4);
    // The new row carries role=intern.
    let iris_row = after_create["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == intern_id.to_string())
        .expect("intern row in roster");
    assert_eq!(iris_row["role"], "intern");

    // --- Edit the intern's blueprint temperature on page 09 ------------------
    update_profile_temperature(&org, intern_id, 0.4).await;

    // --- Create a Shape A project on page 10 --------------------------------
    let project_id = create_shape_a_project(&org, intern_id, "Atlas").await;

    // --- Read the project detail on page 11 (as CEO) -------------------------
    let detail = show_project_as_ceo(&org, project_id).await;
    assert_eq!(
        detail["project"]["id"].as_str(),
        Some(project_id.to_string().as_str())
    );
    assert_eq!(detail["project"]["shape"], "shape_a");
    assert_eq!(
        detail["lead_agent_id"].as_str(),
        Some(intern_id.to_string().as_str())
    );
    assert!(
        detail["recent_sessions"].as_array().unwrap().is_empty(),
        "recent_sessions is the M4 C-M5-3 placeholder — must stay empty"
    );

    // --- Apply an OKR patch on page 11 --------------------------------------
    let patched = patch_okrs(&org, project_id, intern_id).await;
    assert_eq!(
        patched["audit_event_ids"].as_array().unwrap().len(),
        2,
        "one audit event per applied mutation"
    );
    assert_eq!(patched["objectives"].as_array().unwrap().len(), 1);
    assert_eq!(patched["key_results"].as_array().unwrap().len(), 1);

    // --- Dashboard retrofit: counters reflect M4 state ----------------------
    let dash = dashboard_as_ceo(&org).await;
    // 4 agents now: CEO (human, role=None=unclassified) + 2 system
    // (llm, role=None=unclassified) + the Intern we just created.
    let agents_summary = &dash["agents_summary"];
    assert_eq!(agents_summary["total"].as_u64(), Some(4));
    assert_eq!(
        agents_summary["intern"].as_u64(),
        Some(1),
        "the new Intern must surface in the intern bucket"
    );
    assert_eq!(
        agents_summary["unclassified"].as_u64(),
        Some(3),
        "CEO + 2 system agents (fixture-seeded with role=None) stay in unclassified"
    );
    // Projects: one Shape A project exists.
    let projects_summary = &dash["projects_summary"];
    assert_eq!(projects_summary["active"].as_u64(), Some(1));
    assert_eq!(
        projects_summary["shape_a"].as_u64(),
        Some(1),
        "shape_a counter must retrofit from count_projects_by_shape_in_org"
    );
    assert_eq!(projects_summary["shape_b"].as_u64(), Some(0));
}

/// A project-lead viewer (not the CEO) sees `viewer.role = project_lead`
/// on the dashboard — asserts that the M4/P8 dashboard rewrite plumbs
/// `list_projects_led_by_agent` through `resolve_viewer_role`.
#[tokio::test]
async fn project_lead_viewer_role_surfaces_on_dashboard() {
    let org = spawn_claimed_with_org(false).await;
    // Create a Human agent (Member role) so we can make them a project
    // lead. At M4 `is_valid_for(kind)` requires Human + (Executive|Admin|Member)
    // or Llm + (Intern|Contract|System); Member fits.
    let body = json!({
        "display_name": "Eve",
        "kind": "human",
        "role": "member",
        "blueprint": {},
        "parallelize": 1
    });
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = org
        .admin
        .authed_client
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let agent_body: Value = res.json().await.unwrap();
    let eve_id = agent_body["agent_id"]
        .as_str()
        .and_then(|s| s.parse::<uuid::Uuid>().ok())
        .map(AgentId::from_uuid)
        .expect("agent_id");

    // Create a Shape A project with Eve as lead.
    let _project = create_shape_a_project(&org, eve_id, "Eve's project").await;

    // Dashboard as Eve (she's a member + project lead → expects
    // `project_lead` role, NOT `admin`, NOT `member`).
    let eve_client =
        acceptance_common::admin::authed_client_for(&org.admin, eve_id).expect("mint eve session");
    let url = org.url(&format!("/api/v0/orgs/{}/dashboard", org.org_id));
    let res = eve_client.get(url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["viewer"]["role"].as_str(),
        Some("project_lead"),
        "Eve leads a project; dashboard must surface `project_lead` viewer role"
    );
    // AlertedEventsCount tile hidden for project leads — the tile's
    // data is still in the payload (we leave gating to the web tier),
    // but `can_admin_manage` is false so the tile-render condition
    // (`viewer.can_admin_manage`) keeps it off-screen.
    assert_eq!(body["viewer"]["can_admin_manage"].as_bool(), Some(false));
}

// ---------------------------------------------------------------------------
// Cross-org isolation — P8 follow-up, addresses the audit-identified gap
// around test coverage for cross-org scenarios.
// ---------------------------------------------------------------------------

/// Seed an entirely separate org + its CEO + one materialised Shape A
/// project led by that CEO, via the repo layer (bypassing HTTP). The
/// returned project belongs ONLY to the seeded org — it has no
/// `BELONGS_TO` edge to any other org. Used by cross-org tests to
/// exercise access gates and counter isolation without driving two
/// full HTTP wizards in one test.
///
/// Note: the second org is created via `create_organization` (bare
/// insert) rather than `apply_org_creation` (compound tx that also
/// creates the token-budget pool). Since every test consumer of this
/// helper queries the FIRST org's dashboard (never the second's), the
/// missing pool never trips the "token_budget_pool missing" error in
/// the dashboard orchestrator.
async fn seed_unrelated_org_with_project(
    store: Arc<dyn Repository>,
    name_prefix: &str,
    shape: ProjectShape,
) -> (OrgId, AgentId, ProjectId) {
    let other_org_id = OrgId::new();
    store
        .create_organization(&Organization {
            id: other_org_id,
            display_name: format!("{name_prefix} Org"),
            vision: None,
            mission: None,
            consent_policy: domain::model::composites_m3::ConsentPolicy::Implicit,
            audit_class_default: domain::audit::AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        })
        .await
        .expect("create second org");

    // Seed a Human CEO for the other org — required for the Shape A
    // `lead_agent_id` to pass `apply_project_creation`'s compound-tx
    // arity check (which matches the HTTP validator's membership rule:
    // lead.owning_org must be in the project's owning_orgs).
    let other_ceo = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: format!("{name_prefix} CEO"),
        owning_org: Some(other_org_id),
        role: None,
        created_at: Utc::now(),
    };
    let ceo_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: other_ceo.id,
        created_at: Utc::now(),
    };
    let ceo_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: other_ceo.id,
        created_at: Utc::now(),
    };
    store
        .apply_agent_creation(&AgentCreationPayload {
            agent: other_ceo.clone(),
            inbox: ceo_inbox,
            outbox: ceo_outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await
        .expect("seed other-org CEO");

    let project_id = ProjectId::new();
    let project = Project {
        id: project_id,
        name: format!("{name_prefix} Project"),
        description: "cross-org fixture".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        created_at: Utc::now(),
    };
    store
        .apply_project_creation(&ProjectCreationPayload {
            project,
            owning_orgs: vec![other_org_id],
            lead_agent_id: other_ceo.id,
            member_agent_ids: vec![],
            sponsor_agent_ids: vec![],
            catalogue_entries: vec![(format!("project:{project_id}"), "project".into())],
        })
        .await
        .expect("seed other-org project");

    (other_org_id, other_ceo.id, project_id)
}

/// Cross-org access gate — a viewer with no relation to the target
/// project's owning org must get 403 on `GET /api/v0/projects/:id`,
/// even if they are authenticated and a member of some other org.
///
/// Pins the invariant that [`project_detail`]'s access gate is
/// `ProjectId`-keyed (not name-keyed), so identically-named projects
/// in different orgs stay isolated.
#[tokio::test]
async fn cross_org_project_show_denies_foreign_viewer() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // Seed a fully-isolated second org with its own Shape A project.
    // The project's owning_orgs = [other_org_id] (NOT including `org`).
    let (_other_org_id, _other_ceo, foreign_project_id) =
        seed_unrelated_org_with_project(store.clone(), "Foreign", ProjectShape::A).await;

    // Also seed an identically-named Shape A project in OUR org via
    // HTTP so the test body explicitly exercises the "same name, two
    // orgs" scenario. This project belongs to `org`, not the other org.
    let own_lead = org.ceo_agent_id;
    let _own_project = create_shape_a_project(&org, own_lead, "Foreign Project").await;

    // Mint a session for the CEO of OUR org (she's authenticated, is a
    // member of `org`, but has no relation to the foreign project).
    let ceo = ceo_client(&org);
    let url = org.url(&format!("/api/v0/projects/{foreign_project_id}"));
    let res = ceo.get(url).send().await.unwrap();

    assert_eq!(
        res.status().as_u16(),
        403,
        "CEO of one org must not see another org's project — \
         name-collision must not leak across orgs"
    );
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"], "PROJECT_ACCESS_DENIED");
}

/// Dashboard counter isolation — `count_projects_by_shape_in_org`
/// must filter by the `BELONGS_TO` edge so one org's dashboard never
/// surfaces another org's projects.
///
/// Constructs: one Shape A project in `org`, plus a Shape B project
/// in an unrelated second org. Expected: `org`'s dashboard shows
/// `shape_a = 1, shape_b = 0`.
#[tokio::test]
async fn dashboard_shape_counters_are_org_scoped() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // One Shape A project in OUR org, via HTTP.
    let lead = org.ceo_agent_id;
    let _own = create_shape_a_project(&org, lead, "Atlas").await;

    // Seed a Shape B project in a completely isolated second org.
    // (Shape B with a single-org owning_orgs vec bypasses the Shape B
    // 2-owner HTTP validator — we're testing the repo-layer counter,
    // not the creation flow — so we go through `apply_project_creation`
    // directly. The compound tx accepts owning_orgs.len() = 1 for
    // Shape A; we instead seed with a true 2-org setup to stay
    // faithful to the Shape B definition.)
    let third_org_id = OrgId::new();
    store
        .create_organization(&Organization {
            id: third_org_id,
            display_name: "Third Org".into(),
            vision: None,
            mission: None,
            consent_policy: domain::model::composites_m3::ConsentPolicy::Implicit,
            audit_class_default: domain::audit::AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    // Now seed a Shape B project across the two unrelated orgs (NOT
    // involving `org`). We need a human lead in one of them.
    let (second_org_id, second_ceo, _second_project) =
        seed_unrelated_org_with_project(store.clone(), "Second", ProjectShape::A).await;

    let shape_b_project = Project {
        id: ProjectId::new(),
        name: "Shared".into(),
        description: "co-owned fixture".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape: ProjectShape::B,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        created_at: Utc::now(),
    };
    let shape_b_id = shape_b_project.id;
    store
        .apply_project_creation(&ProjectCreationPayload {
            project: shape_b_project,
            owning_orgs: vec![second_org_id, third_org_id],
            lead_agent_id: second_ceo,
            member_agent_ids: vec![],
            sponsor_agent_ids: vec![],
            catalogue_entries: vec![(format!("project:{shape_b_id}"), "project".into())],
        })
        .await
        .expect("seed cross-org Shape B project");

    // Dashboard for OUR org: must see exactly ONE Shape A project and
    // zero Shape B projects, regardless of what exists in other orgs.
    let dash = dashboard_as_ceo(&org).await;
    let projects_summary = &dash["projects_summary"];
    assert_eq!(
        projects_summary["active"].as_u64(),
        Some(1),
        "dashboard `active` must count only OUR org's projects"
    );
    assert_eq!(
        projects_summary["shape_a"].as_u64(),
        Some(1),
        "shape_a counter must filter by BELONGS_TO — the second org's \
         Shape A project and the Shape B project must not leak here"
    );
    assert_eq!(
        projects_summary["shape_b"].as_u64(),
        Some(0),
        "shape_b counter must be 0 — the cross-org Shape B project \
         belongs to orgs {{second, third}}, not to OUR org"
    );
}
