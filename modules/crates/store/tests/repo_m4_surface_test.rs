//! Integration tests for M4/P2 additions to the `SurrealStore:
//! Repository` surface. Mirrors `domain/tests/in_memory_m4_test.rs`
//! against the real SurrealDB backend so the two impls stay in
//! lock-step (M4 plan commitment C6).
//!
//! Seed strategy for Project + edge rows: the production write path
//! (`apply_project_creation`) lands at M4/P3. Until then, tests seed
//! via direct SurrealQL `CREATE` + `RELATE` so the reader methods
//! have something to return. Each test uses a fresh tempdir.

use chrono::Utc;

use domain::audit::AuditClass;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, Organization, Project, ProjectShape, ProjectStatus,
};
use domain::model::{AgentExecutionLimitsOverride, OrganizationDefaultsSnapshot};
use domain::repository::Repository;
use store::SurrealStore;
use tempfile::TempDir;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

fn minimal_org(id: OrgId) -> Organization {
    Organization {
        id,
        display_name: format!("org-{}", id),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        created_at: Utc::now(),
    }
}

fn make_agent(org: OrgId, kind: AgentKind, role: Option<AgentRole>) -> Agent {
    Agent {
        id: AgentId::new(),
        kind,
        display_name: "probe".into(),
        owning_org: Some(org),
        role,
        created_at: Utc::now(),
    }
}

fn make_project(shape: ProjectShape) -> Project {
    Project {
        id: ProjectId::new(),
        name: "Atlas".into(),
        description: "seed".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        created_at: Utc::now(),
    }
}

async fn seed_project(store: &SurrealStore, project: &Project, owning_orgs: &[OrgId]) {
    // Strip `id` to match the repo_impl write-path convention.
    let mut body = serde_json::to_value(project).unwrap();
    if let serde_json::Value::Object(map) = &mut body {
        map.remove("id");
    }
    store
        .client()
        .query("CREATE type::thing('project', $id) CONTENT $body RETURN NONE")
        .bind(("id", project.id.to_string()))
        .bind(("body", body))
        .await
        .expect("create project")
        .check()
        .expect("check project create");
    for org in owning_orgs {
        store
            .client()
            .query(
                "LET $p = type::thing('project', $pid); \
                 LET $o = type::thing('organization', $oid); \
                 RELATE $p -> belongs_to -> $o RETURN NONE",
            )
            .bind(("pid", project.id.to_string()))
            .bind(("oid", org.to_string()))
            .await
            .expect("relate belongs_to")
            .check()
            .expect("check belongs_to");
    }
}

async fn seed_has_lead(store: &SurrealStore, project: ProjectId, lead: AgentId) {
    store
        .client()
        .query(
            "LET $p = type::thing('project', $pid); \
             LET $a = type::thing('agent', $aid); \
             RELATE $p -> has_lead -> $a RETURN NONE",
        )
        .bind(("pid", project.to_string()))
        .bind(("aid", lead.to_string()))
        .await
        .expect("relate has_lead")
        .check()
        .expect("check has_lead");
}

// ---- list_agents_in_org_by_role -------------------------------------------

#[tokio::test]
async fn list_agents_in_org_by_role_role_filter_is_honoured() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    // Two interns + one executive + one agent without role.
    store
        .create_agent(&make_agent(org, AgentKind::Llm, Some(AgentRole::Intern)))
        .await
        .unwrap();
    store
        .create_agent(&make_agent(org, AgentKind::Llm, Some(AgentRole::Intern)))
        .await
        .unwrap();
    store
        .create_agent(&make_agent(
            org,
            AgentKind::Human,
            Some(AgentRole::Executive),
        ))
        .await
        .unwrap();
    store
        .create_agent(&make_agent(org, AgentKind::Human, None))
        .await
        .unwrap();

    let all = store.list_agents_in_org_by_role(org, None).await.unwrap();
    assert_eq!(all.len(), 4, "no-filter path returns everyone in org");

    let interns = store
        .list_agents_in_org_by_role(org, Some(AgentRole::Intern))
        .await
        .unwrap();
    assert_eq!(interns.len(), 2);

    let executives = store
        .list_agents_in_org_by_role(org, Some(AgentRole::Executive))
        .await
        .unwrap();
    assert_eq!(executives.len(), 1);
}

// ---- get_project + list_projects_in_org + list_by_shape + count -----------

#[tokio::test]
async fn project_read_methods_honour_belongs_to_edges() {
    let (store, _dir) = fresh_store().await;
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    store
        .create_organization(&minimal_org(org_a))
        .await
        .unwrap();
    store
        .create_organization(&minimal_org(org_b))
        .await
        .unwrap();

    let p1 = make_project(ProjectShape::A);
    let p1_id = p1.id;
    let p2 = make_project(ProjectShape::A);
    let p3 = make_project(ProjectShape::B);
    seed_project(&store, &p1, &[org_a]).await;
    seed_project(&store, &p2, &[org_a]).await;
    seed_project(&store, &p3, &[org_a, org_b]).await;

    let in_org_a = store.list_projects_in_org(org_a).await.unwrap();
    assert_eq!(in_org_a.len(), 3, "org_a sees its 2 Shape A + 1 Shape B");

    let in_org_b = store.list_projects_in_org(org_b).await.unwrap();
    assert_eq!(
        in_org_b.len(),
        1,
        "Shape B co-owner org_b sees only the shared project"
    );

    let shape_a = store
        .list_projects_by_shape_in_org(org_a, ProjectShape::A)
        .await
        .unwrap();
    assert_eq!(shape_a.len(), 2);

    let shape_b = store
        .list_projects_by_shape_in_org(org_a, ProjectShape::B)
        .await
        .unwrap();
    assert_eq!(shape_b.len(), 1);

    let counts = store.count_projects_by_shape_in_org(org_a).await.unwrap();
    assert_eq!(counts.shape_a, 2);
    assert_eq!(counts.shape_b, 1);
    assert_eq!(counts.total(), 3);

    let found = store.get_project(p1_id).await.unwrap();
    assert!(found.is_some());
    let missed = store.get_project(ProjectId::new()).await.unwrap();
    assert!(missed.is_none());
}

// ---- list_projects_led_by_agent -------------------------------------------

#[tokio::test]
async fn list_projects_led_by_agent_tracks_has_lead_edge() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let lead_agent = make_agent(org, AgentKind::Human, Some(AgentRole::Executive));
    let lead_id = lead_agent.id;
    store.create_agent(&lead_agent).await.unwrap();
    let p1 = make_project(ProjectShape::A);
    let p2 = make_project(ProjectShape::A);
    let p3 = make_project(ProjectShape::A);
    let p1_id = p1.id;
    let p2_id = p2.id;
    seed_project(&store, &p1, &[org]).await;
    seed_project(&store, &p2, &[org]).await;
    seed_project(&store, &p3, &[org]).await;
    seed_has_lead(&store, p1_id, lead_id).await;
    seed_has_lead(&store, p2_id, lead_id).await;

    let led = store.list_projects_led_by_agent(lead_id).await.unwrap();
    assert_eq!(led.len(), 2);
    let ids: std::collections::HashSet<_> = led.iter().map(|p| p.id).collect();
    assert!(ids.contains(&p1_id));
    assert!(ids.contains(&p2_id));

    // Unrelated agent — no led projects.
    let unrelated = store
        .list_projects_led_by_agent(AgentId::new())
        .await
        .unwrap();
    assert!(unrelated.is_empty());
}

// ---- agent_execution_limits override CRUD + resolver ----------------------

fn sample_limits(max_turns: usize) -> phi_core::context::execution::ExecutionLimits {
    phi_core::context::execution::ExecutionLimits {
        max_turns,
        max_total_tokens: 500_000,
        max_duration: std::time::Duration::from_secs(300),
        max_cost: Some(5.0),
    }
}

fn make_override(
    agent: AgentId,
    limits: phi_core::context::execution::ExecutionLimits,
) -> AgentExecutionLimitsOverride {
    AgentExecutionLimitsOverride {
        id: NodeId::new(),
        owning_agent: agent,
        limits,
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn agent_execution_limits_override_crud_round_trips() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let agent = make_agent(org, AgentKind::Llm, Some(AgentRole::Intern));
    let agent_id = agent.id;
    store.create_agent(&agent).await.unwrap();

    assert!(store
        .get_agent_execution_limits_override(agent_id)
        .await
        .unwrap()
        .is_none());

    store
        .set_agent_execution_limits_override(&make_override(agent_id, sample_limits(25)))
        .await
        .unwrap();
    let hit = store
        .get_agent_execution_limits_override(agent_id)
        .await
        .unwrap()
        .expect("override present");
    assert_eq!(hit.limits.max_turns, 25);

    // Create-or-replace semantics: second set lands and overwrites.
    store
        .set_agent_execution_limits_override(&make_override(agent_id, sample_limits(40)))
        .await
        .unwrap();
    let replaced = store
        .get_agent_execution_limits_override(agent_id)
        .await
        .unwrap()
        .expect("replaced row");
    assert_eq!(replaced.limits.max_turns, 40);

    // Clear — idempotent even when a row is absent.
    store
        .clear_agent_execution_limits_override(agent_id)
        .await
        .unwrap();
    assert!(store
        .get_agent_execution_limits_override(agent_id)
        .await
        .unwrap()
        .is_none());
    store
        .clear_agent_execution_limits_override(agent_id)
        .await
        .unwrap();
}

#[tokio::test]
async fn resolve_effective_execution_limits_walks_override_then_snapshot() {
    let (store, _dir) = fresh_store().await;
    let org_id = OrgId::new();
    let mut org = minimal_org(org_id);
    org.defaults_snapshot = Some(OrganizationDefaultsSnapshot {
        execution_limits: sample_limits(80),
        default_agent_profile: phi_core::agents::profile::AgentProfile::default(),
        context_config: phi_core::context::config::ContextConfig::default(),
        retry_config: phi_core::provider::retry::RetryConfig::default(),
        default_retention_days: 30,
        default_alert_channels: vec![],
    });
    store.create_organization(&org).await.unwrap();
    let agent = make_agent(org_id, AgentKind::Llm, Some(AgentRole::Intern));
    let agent_id = agent.id;
    store.create_agent(&agent).await.unwrap();

    // Snapshot-only path.
    let via_snapshot = store
        .resolve_effective_execution_limits(agent_id)
        .await
        .unwrap()
        .expect("snapshot value");
    assert_eq!(via_snapshot.max_turns, 80);

    // Override-wins path.
    store
        .set_agent_execution_limits_override(&make_override(agent_id, sample_limits(15)))
        .await
        .unwrap();
    let via_override = store
        .resolve_effective_execution_limits(agent_id)
        .await
        .unwrap()
        .expect("override value");
    assert_eq!(via_override.max_turns, 15);
}
