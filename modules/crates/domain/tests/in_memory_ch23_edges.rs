//! CH-23 / ADR-0046 — Repository tests for the new
//! `create_manages_edge` and `create_has_agent_supervisor_edge`
//! compound-tx methods on the in-memory backend.
//!
//! Covered:
//! - Happy-path edge creation + audit emission.
//! - Idempotent re-POST returns the existing edge id with `created = false`.
//! - Self-loop rejection.
//! - Archived-agent rejection (CH-01 / ADR-0034 invariant).
//! - Cross-org / cross-project membership rejection.
//!
//! Store-parity coverage for the SurrealDB backend lives in
//! `store/tests/repository_test.rs` (CH-23 section).

use chrono::Utc;

use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, Organization, Project, ProjectShape, ProjectStatus,
};
use domain::repository::ProjectCreationPayload;
use domain::Repository;

// ---- Fixtures --------------------------------------------------------------

fn agent_in_org(org: OrgId) -> Agent {
    Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "probe".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Member),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
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
        approval_timeout: domain::model::ApprovalTimeout::ProjectDuration,
        approval_timeout_default_response: domain::model::TimeoutResponse::Deny,
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

async fn seed_org_with_two_agents(repo: &InMemoryRepository) -> (OrgId, AgentId, AgentId) {
    let org = OrgId::new();
    repo.create_organization(&minimal_org(org)).await.unwrap();
    let a = agent_in_org(org);
    let b = agent_in_org(org);
    let aid = a.id;
    let bid = b.id;
    repo.create_agent(&a).await.unwrap();
    repo.create_agent(&b).await.unwrap();
    (org, aid, bid)
}

async fn seed_project_with_two_agents(
    repo: &InMemoryRepository,
) -> (ProjectId, OrgId, AgentId, AgentId) {
    let org = OrgId::new();
    repo.create_organization(&minimal_org(org)).await.unwrap();
    let a = agent_in_org(org);
    let b = agent_in_org(org);
    let lead = agent_in_org(org);
    let (aid, bid, lead_id) = (a.id, b.id, lead.id);
    repo.create_agent(&a).await.unwrap();
    repo.create_agent(&b).await.unwrap();
    repo.create_agent(&lead).await.unwrap();

    let proj = project(ProjectShape::A);
    let pid = proj.id;
    let payload = ProjectCreationPayload {
        project: proj,
        owning_orgs: vec![org],
        lead_agent_id: lead_id,
        // CH-25 / ADR-0060 §D60.1 — Decision-3: lead = creator.
        creator_agent: lead_id,
        member_agent_ids: vec![aid, bid],
        sponsor_agent_ids: vec![],
        catalogue_entries: vec![(format!("project:{pid}"), "project".into())],
    };
    repo.apply_project_creation(&payload).await.unwrap();
    (pid, org, aid, bid)
}

// ---- create_manages_edge --------------------------------------------------

#[tokio::test]
async fn create_manages_edge_happy_path_returns_created_true_and_audit_id() {
    let repo = InMemoryRepository::new();
    let (org, manager, subordinate) = seed_org_with_two_agents(&repo).await;

    let receipt = repo
        .create_manages_edge(org, manager, subordinate, manager, Utc::now())
        .await
        .expect("happy path persists the edge");

    assert!(receipt.created);
    assert!(receipt.audit_event_id.is_some());
}

#[tokio::test]
async fn create_manages_edge_re_post_is_idempotent() {
    let repo = InMemoryRepository::new();
    let (org, manager, subordinate) = seed_org_with_two_agents(&repo).await;

    let first = repo
        .create_manages_edge(org, manager, subordinate, manager, Utc::now())
        .await
        .unwrap();
    let second = repo
        .create_manages_edge(org, manager, subordinate, manager, Utc::now())
        .await
        .unwrap();

    assert!(first.created);
    assert!(!second.created, "re-POST must not write a new edge");
    assert_eq!(first.edge_id, second.edge_id, "edge id stays stable");
    assert!(
        second.audit_event_id.is_none(),
        "idempotent path must not emit a second audit event"
    );
}

#[tokio::test]
async fn create_manages_edge_rejects_self_loop() {
    let repo = InMemoryRepository::new();
    let (org, manager, _) = seed_org_with_two_agents(&repo).await;

    let err = repo
        .create_manages_edge(org, manager, manager, manager, Utc::now())
        .await
        .expect_err("self-loop must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::InvalidArgument(_)
    ));
}

#[tokio::test]
async fn create_manages_edge_rejects_archived_subordinate() {
    let repo = InMemoryRepository::new();
    let (org, manager, subordinate) = seed_org_with_two_agents(&repo).await;
    // Mark subordinate archived (mirrors CH-01 / ADR-0034 archive-flip path).
    repo.set_agent_archived_at(subordinate, Some(Utc::now()))
        .await
        .unwrap();

    let err = repo
        .create_manages_edge(org, manager, subordinate, manager, Utc::now())
        .await
        .expect_err("archived agent must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::Conflict(_)
    ));
}

#[tokio::test]
async fn create_manages_edge_rejects_cross_org() {
    let repo = InMemoryRepository::new();
    // Two orgs, manager in org_a, subordinate in org_b.
    let (org_a, manager, _) = seed_org_with_two_agents(&repo).await;
    let (org_b, _, foreign) = seed_org_with_two_agents(&repo).await;
    assert_ne!(org_a, org_b);

    let err = repo
        .create_manages_edge(org_a, manager, foreign, manager, Utc::now())
        .await
        .expect_err("cross-org assignment must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::InvalidArgument(_)
    ));
}

#[tokio::test]
async fn create_manages_edge_writes_audit_event_with_expected_diff() {
    let repo = InMemoryRepository::new();
    let (org, manager, subordinate) = seed_org_with_two_agents(&repo).await;

    let before = repo
        .list_recent_audit_events_for_org(org, 100)
        .await
        .unwrap()
        .len();
    repo.create_manages_edge(org, manager, subordinate, manager, Utc::now())
        .await
        .unwrap();
    let after = repo
        .list_recent_audit_events_for_org(org, 100)
        .await
        .unwrap();
    assert_eq!(after.len(), before + 1, "exactly one audit event emitted");
    let ev = after.last().expect("at least one audit event");
    assert_eq!(ev.event_type, "platform.manages.edge.created");
    assert_eq!(ev.org_scope, Some(org));
    assert_eq!(ev.diff["after"]["edge_kind"], "MANAGES");
    assert_eq!(ev.diff["after"]["manager_agent_id"], manager.to_string());
    assert_eq!(
        ev.diff["after"]["subordinate_agent_id"],
        subordinate.to_string()
    );
}

// ---- create_has_agent_supervisor_edge -------------------------------------

#[tokio::test]
async fn create_has_agent_supervisor_edge_happy_path() {
    let repo = InMemoryRepository::new();
    let (project, _org, supervisor, supervisee) = seed_project_with_two_agents(&repo).await;

    let receipt = repo
        .create_has_agent_supervisor_edge(project, supervisor, supervisee, supervisor, Utc::now())
        .await
        .expect("happy path persists the edge");

    assert!(receipt.created);
    assert!(receipt.audit_event_id.is_some());
}

#[tokio::test]
async fn create_has_agent_supervisor_edge_re_post_is_idempotent() {
    let repo = InMemoryRepository::new();
    let (project, _org, supervisor, supervisee) = seed_project_with_two_agents(&repo).await;

    let first = repo
        .create_has_agent_supervisor_edge(project, supervisor, supervisee, supervisor, Utc::now())
        .await
        .unwrap();
    let second = repo
        .create_has_agent_supervisor_edge(project, supervisor, supervisee, supervisor, Utc::now())
        .await
        .unwrap();

    assert!(first.created);
    assert!(!second.created);
    assert_eq!(first.edge_id, second.edge_id);
    assert!(second.audit_event_id.is_none());
}

#[tokio::test]
async fn create_has_agent_supervisor_edge_rejects_self_loop() {
    let repo = InMemoryRepository::new();
    let (project, _org, supervisor, _supervisee) = seed_project_with_two_agents(&repo).await;

    let err = repo
        .create_has_agent_supervisor_edge(project, supervisor, supervisor, supervisor, Utc::now())
        .await
        .expect_err("self-loop must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::InvalidArgument(_)
    ));
}

#[tokio::test]
async fn create_has_agent_supervisor_edge_rejects_archived_supervisee() {
    let repo = InMemoryRepository::new();
    let (project, _org, supervisor, supervisee) = seed_project_with_two_agents(&repo).await;
    repo.set_agent_archived_at(supervisee, Some(Utc::now()))
        .await
        .unwrap();

    let err = repo
        .create_has_agent_supervisor_edge(project, supervisor, supervisee, supervisor, Utc::now())
        .await
        .expect_err("archived agent must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::Conflict(_)
    ));
}

#[tokio::test]
async fn create_has_agent_supervisor_edge_rejects_cross_project() {
    let repo = InMemoryRepository::new();
    let (project_a, _, supervisor, _) = seed_project_with_two_agents(&repo).await;
    let (_project_b, _, _, supervisee_in_b) = seed_project_with_two_agents(&repo).await;

    let err = repo
        .create_has_agent_supervisor_edge(
            project_a,
            supervisor,
            supervisee_in_b,
            supervisor,
            Utc::now(),
        )
        .await
        .expect_err("cross-project assignment must reject");
    assert!(matches!(
        err,
        domain::repository::RepositoryError::InvalidArgument(_)
    ));
}

#[tokio::test]
async fn create_has_agent_supervisor_edge_writes_audit_with_project_scope() {
    let repo = InMemoryRepository::new();
    let (project, org, supervisor, supervisee) = seed_project_with_two_agents(&repo).await;

    let before = repo
        .list_recent_audit_events_for_org(org, 100)
        .await
        .unwrap()
        .len();
    repo.create_has_agent_supervisor_edge(project, supervisor, supervisee, supervisor, Utc::now())
        .await
        .unwrap();
    let after = repo
        .list_recent_audit_events_for_org(org, 100)
        .await
        .unwrap();
    assert_eq!(after.len(), before + 1);
    let ev = after.last().expect("at least one audit event");
    assert_eq!(ev.event_type, "platform.has_agent_supervisor.edge.created");
    assert_eq!(ev.diff["after"]["project_id"], project.to_string());
    assert_eq!(ev.diff["after"]["edge_kind"], "HAS_AGENT_SUPERVISOR");
}
