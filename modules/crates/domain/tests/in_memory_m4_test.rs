//! M4/P2 commitment C6 — populated + empty cases for every new M4
//! read method on the Repository trait, exercised through the in-
//! memory fake. Store parity lands in `store/tests/repo_m4_surface_test.rs`.
//!
//! Covered here:
//!
//! - `list_agents_in_org_by_role(org, None)`  — no filter path
//! - `list_agents_in_org_by_role(org, Some(role))` — role filter path
//! - `get_project(id)` — hit + miss
//! - `list_projects_in_org(org)` — populated + empty
//! - `list_projects_by_shape_in_org(org, shape)` — populated + empty
//! - `count_projects_by_shape_in_org(org)` — mixed + zeroed
//! - `list_projects_led_by_agent(agent)` — populated + empty
//! - `get_agent_execution_limits_override(agent)` — hit + miss
//! - `set_agent_execution_limits_override` / `clear_*` round-trip
//! - `resolve_effective_execution_limits(agent)` — override / snapshot /
//!   empty fallback

use chrono::Utc;

use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, Organization, Project, ProjectShape, ProjectStatus,
};
use domain::model::{AgentExecutionLimitsOverride, OrganizationDefaultsSnapshot};
use domain::Repository;

fn agent_with_role(org: OrgId, kind: AgentKind, role: Option<AgentRole>) -> Agent {
    Agent {
        id: AgentId::new(),
        kind,
        display_name: "probe".into(),
        owning_org: Some(org),
        role,
        created_at: Utc::now(),
    }
}

fn project(shape: ProjectShape) -> Project {
    Project {
        id: ProjectId::new(),
        name: "Atlas".into(),
        description: "Seed project".into(),
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

fn minimal_org(id: OrgId) -> Organization {
    Organization {
        id,
        display_name: "Acme".into(),
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

// ---- list_agents_in_org_by_role -------------------------------------------

#[tokio::test]
async fn list_agents_in_org_by_role_no_filter_returns_everyone_in_the_org() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    repo.create_agent(&agent_with_role(
        org,
        AgentKind::Human,
        Some(AgentRole::Executive),
    ))
    .await
    .unwrap();
    repo.create_agent(&agent_with_role(
        org,
        AgentKind::Llm,
        Some(AgentRole::Intern),
    ))
    .await
    .unwrap();
    let out = repo.list_agents_in_org_by_role(org, None).await.unwrap();
    assert_eq!(out.len(), 2);
}

#[tokio::test]
async fn list_agents_in_org_by_role_filter_returns_matching_role_only() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    repo.create_agent(&agent_with_role(
        org,
        AgentKind::Human,
        Some(AgentRole::Executive),
    ))
    .await
    .unwrap();
    repo.create_agent(&agent_with_role(
        org,
        AgentKind::Llm,
        Some(AgentRole::Intern),
    ))
    .await
    .unwrap();
    repo.create_agent(&agent_with_role(
        org,
        AgentKind::Llm,
        Some(AgentRole::Intern),
    ))
    .await
    .unwrap();
    let interns = repo
        .list_agents_in_org_by_role(org, Some(AgentRole::Intern))
        .await
        .unwrap();
    assert_eq!(interns.len(), 2);
    let executives = repo
        .list_agents_in_org_by_role(org, Some(AgentRole::Executive))
        .await
        .unwrap();
    assert_eq!(executives.len(), 1);
}

#[tokio::test]
async fn list_agents_in_org_by_role_ignores_agents_outside_the_org() {
    let repo = InMemoryRepository::new();
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    repo.create_agent(&agent_with_role(
        org_a,
        AgentKind::Llm,
        Some(AgentRole::Intern),
    ))
    .await
    .unwrap();
    repo.create_agent(&agent_with_role(
        org_b,
        AgentKind::Llm,
        Some(AgentRole::Intern),
    ))
    .await
    .unwrap();
    let out_a = repo
        .list_agents_in_org_by_role(org_a, Some(AgentRole::Intern))
        .await
        .unwrap();
    assert_eq!(out_a.len(), 1);
}

// ---- get_project -----------------------------------------------------------

#[tokio::test]
async fn get_project_hit_returns_the_seeded_row() {
    let repo = InMemoryRepository::new();
    let p = project(ProjectShape::A);
    let pid = p.id;
    repo.test_seed_project(p, &[OrgId::new()]);
    let found = repo.get_project(pid).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, pid);
}

#[tokio::test]
async fn get_project_miss_returns_none() {
    let repo = InMemoryRepository::new();
    let out = repo.get_project(ProjectId::new()).await.unwrap();
    assert!(out.is_none());
}

// ---- list_projects_in_org -------------------------------------------------

#[tokio::test]
async fn list_projects_in_org_returns_every_belonging_project() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::B), &[org, OrgId::new()]);
    // Project that belongs to a DIFFERENT org — must be excluded.
    repo.test_seed_project(project(ProjectShape::A), &[OrgId::new()]);

    let out = repo.list_projects_in_org(org).await.unwrap();
    assert_eq!(out.len(), 2);
}

#[tokio::test]
async fn list_projects_in_org_empty_when_no_rows_belong_to_it() {
    let repo = InMemoryRepository::new();
    let out = repo.list_projects_in_org(OrgId::new()).await.unwrap();
    assert!(out.is_empty());
}

// ---- list_projects_by_shape_in_org ----------------------------------------

#[tokio::test]
async fn list_projects_by_shape_in_org_narrows_by_shape() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::B), &[org, OrgId::new()]);

    let shape_a = repo
        .list_projects_by_shape_in_org(org, ProjectShape::A)
        .await
        .unwrap();
    assert_eq!(shape_a.len(), 2);
    let shape_b = repo
        .list_projects_by_shape_in_org(org, ProjectShape::B)
        .await
        .unwrap();
    assert_eq!(shape_b.len(), 1);
}

#[tokio::test]
async fn list_projects_by_shape_in_org_empty_org_is_empty() {
    let repo = InMemoryRepository::new();
    let out = repo
        .list_projects_by_shape_in_org(OrgId::new(), ProjectShape::A)
        .await
        .unwrap();
    assert!(out.is_empty());
}

// ---- count_projects_by_shape_in_org ---------------------------------------

#[tokio::test]
async fn count_projects_by_shape_in_org_reports_split() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::A), &[org]);
    repo.test_seed_project(project(ProjectShape::B), &[org, OrgId::new()]);
    let counts = repo.count_projects_by_shape_in_org(org).await.unwrap();
    assert_eq!(counts.shape_a, 3);
    assert_eq!(counts.shape_b, 1);
    assert_eq!(counts.total(), 4);
}

#[tokio::test]
async fn count_projects_by_shape_in_org_empty_org_reports_zero() {
    let repo = InMemoryRepository::new();
    let counts = repo
        .count_projects_by_shape_in_org(OrgId::new())
        .await
        .unwrap();
    assert_eq!(counts.shape_a, 0);
    assert_eq!(counts.shape_b, 0);
}

// ---- list_projects_led_by_agent ------------------------------------------

#[tokio::test]
async fn list_projects_led_by_agent_returns_only_led_projects() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    let lead = AgentId::new();
    let p1 = project(ProjectShape::A);
    let p2 = project(ProjectShape::A);
    let p3 = project(ProjectShape::A);
    let p1_id = p1.id;
    let p2_id = p2.id;
    repo.test_seed_project(p1, &[org]);
    repo.test_seed_project(p2, &[org]);
    repo.test_seed_project(p3, &[org]);
    repo.test_seed_project_lead(p1_id, lead);
    repo.test_seed_project_lead(p2_id, lead);

    let out = repo.list_projects_led_by_agent(lead).await.unwrap();
    assert_eq!(out.len(), 2);
    let ids: std::collections::HashSet<_> = out.iter().map(|p| p.id).collect();
    assert!(ids.contains(&p1_id));
    assert!(ids.contains(&p2_id));
}

#[tokio::test]
async fn list_projects_led_by_agent_empty_when_no_edges_exist() {
    let repo = InMemoryRepository::new();
    // M4/P2 note: no production writer of HAS_LEAD yet (arrives at
    // M4/P3). This test pins the "returns empty" contract the docs
    // promise.
    let out = repo
        .list_projects_led_by_agent(AgentId::new())
        .await
        .unwrap();
    assert!(out.is_empty());
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
async fn override_get_miss_is_none_before_set() {
    let repo = InMemoryRepository::new();
    let out = repo
        .get_agent_execution_limits_override(AgentId::new())
        .await
        .unwrap();
    assert!(out.is_none());
}

#[tokio::test]
async fn override_set_then_get_round_trips() {
    let repo = InMemoryRepository::new();
    let agent = AgentId::new();
    let row = make_override(agent, sample_limits(25));
    repo.set_agent_execution_limits_override(&row)
        .await
        .unwrap();
    let back = repo
        .get_agent_execution_limits_override(agent)
        .await
        .unwrap()
        .expect("override row present");
    assert_eq!(back.owning_agent, agent);
    assert_eq!(back.limits.max_turns, 25);
}

#[tokio::test]
async fn override_set_is_create_or_replace_on_duplicate_agent() {
    let repo = InMemoryRepository::new();
    let agent = AgentId::new();
    repo.set_agent_execution_limits_override(&make_override(agent, sample_limits(10)))
        .await
        .unwrap();
    repo.set_agent_execution_limits_override(&make_override(agent, sample_limits(40)))
        .await
        .unwrap();
    let back = repo
        .get_agent_execution_limits_override(agent)
        .await
        .unwrap()
        .expect("second override landed");
    assert_eq!(back.limits.max_turns, 40);
}

#[tokio::test]
async fn override_clear_is_idempotent_whether_or_not_a_row_exists() {
    let repo = InMemoryRepository::new();
    let agent = AgentId::new();
    // Clear with no row present — must not error.
    repo.clear_agent_execution_limits_override(agent)
        .await
        .unwrap();
    // Set then clear — get returns None.
    repo.set_agent_execution_limits_override(&make_override(agent, sample_limits(10)))
        .await
        .unwrap();
    repo.clear_agent_execution_limits_override(agent)
        .await
        .unwrap();
    let out = repo
        .get_agent_execution_limits_override(agent)
        .await
        .unwrap();
    assert!(out.is_none());
}

#[tokio::test]
async fn resolve_effective_limits_prefers_override_over_snapshot() {
    let repo = InMemoryRepository::new();
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
    repo.create_organization(&org).await.unwrap();
    let agent = agent_with_role(org_id, AgentKind::Llm, Some(AgentRole::Intern));
    let agent_id = agent.id;
    repo.create_agent(&agent).await.unwrap();
    // Snapshot-only path: override absent → snapshot value.
    let via_snapshot = repo
        .resolve_effective_execution_limits(agent_id)
        .await
        .unwrap()
        .expect("snapshot value returned");
    assert_eq!(via_snapshot.max_turns, 80);
    // Override-wins path: set override → returns override.
    repo.set_agent_execution_limits_override(&make_override(agent_id, sample_limits(15)))
        .await
        .unwrap();
    let via_override = repo
        .resolve_effective_execution_limits(agent_id)
        .await
        .unwrap()
        .expect("override value returned");
    assert_eq!(via_override.max_turns, 15);
}

#[tokio::test]
async fn resolve_effective_limits_returns_none_when_agent_and_org_absent() {
    let repo = InMemoryRepository::new();
    let out = repo
        .resolve_effective_execution_limits(AgentId::new())
        .await
        .unwrap();
    assert!(out.is_none());
}
