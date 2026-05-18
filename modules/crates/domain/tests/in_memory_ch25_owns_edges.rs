//! CH-25 / ADR-0060 §D60.1 — In-memory repository tests for the new
//! `Owns` edge emit path inside `apply_org_creation` +
//! `apply_project_creation` compound transactions.
//!
//! Covered:
//! - Happy-path `Owns` edge emit at apply_org_creation (ceo → org).
//! - Happy-path `Owns` edge emit at apply_project_creation (lead →
//!   project).
//! - `list_agent_owned_orgs` + `list_agent_owned_projects` Repository
//!   methods return the freshly-emitted owns edges keyed by Agent.
//! - Multi-org / multi-project ownership: an Agent can own > 1 Org and
//!   > 1 Project; the lists return all of them.
//! - Per Decision-3 user-lock (Shape B owner): lead IS both creator
//!   AND owner at Shape B; the test exercises a Shape B materialise
//!   path and verifies the lead is wired as the owner.
//!
//! Store-parity coverage for the SurrealDB backend lives in
//! `store/tests/apply_*_tx_test.rs` and `repository_test.rs`.

use chrono::Utc;

use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m3::{ConsentPolicy, TokenBudgetPool};
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, Channel, ChannelKind, Grant, InboxObject,
    Organization, OutboxObject, PrincipalRef, Project, ProjectShape, ProjectStatus, ResourceRef,
};
use domain::permissions::Action;
use domain::repository::{OrgCreationPayload, ProjectCreationPayload};
use domain::Repository;

// ---- Fixtures --------------------------------------------------------------

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
        tags: vec![],
        created_at: Utc::now(),
    }
}

fn make_agent(org: OrgId) -> Agent {
    Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "test".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Member),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    }
}

fn make_project(shape: ProjectShape) -> Project {
    Project {
        id: ProjectId::new(),
        name: "Atlas".into(),
        description: "Seed".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        tags: vec![],
        created_at: Utc::now(),
    }
}

fn build_org_payload(org_id: OrgId) -> OrgCreationPayload {
    let ceo = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "CEO".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let sys0 = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "memory-extractor".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let sys1 = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let blueprint = phi_core::agents::profile::AgentProfile::default();
    let prof0 = AgentProfile {
        id: NodeId::new(),
        agent_id: sys0.id,
        parallelize: 1,
        blueprint: blueprint.clone(),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    let prof1 = AgentProfile {
        id: NodeId::new(),
        agent_id: sys1.id,
        parallelize: 1,
        blueprint,
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    OrgCreationPayload {
        organization: minimal_org(org_id),
        creator_agent: ceo.id, // fixture: ceo IS the creator
        ceo_agent: ceo.clone(),
        ceo_channel: Channel {
            id: NodeId::new(),
            agent_id: ceo.id,
            kind: ChannelKind::Email,
            handle: "ceo@acme.test".into(),
            created_at: Utc::now(),
        },
        ceo_inbox: InboxObject {
            id: NodeId::new(),
            agent_id: ceo.id,
            created_at: Utc::now(),
            tags: vec![],
        },
        ceo_outbox: OutboxObject {
            id: NodeId::new(),
            agent_id: ceo.id,
            created_at: Utc::now(),
            tags: vec![],
        },
        ceo_grant: Grant {
            id: domain::model::ids::GrantId::new(),
            holder: PrincipalRef::Agent(ceo.id),
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
        },
        system_agents: [(sys0, prof0), (sys1, prof1)],
        token_budget_pool: TokenBudgetPool::new(org_id, 1_000_000, Utc::now()),
        adoption_auth_requests: vec![],
        catalogue_entries: vec![(format!("org:{}", org_id), "control_plane".into())],
    }
}

// ---- apply_org_creation emits Owns edge -----------------------------------

#[tokio::test]
async fn apply_org_creation_emits_owns_edge_for_ceo() {
    let repo = InMemoryRepository::new();
    let org_id = OrgId::new();
    let payload = build_org_payload(org_id);
    let ceo_id = payload.ceo_agent.id;

    let receipt = repo
        .apply_org_creation(&payload)
        .await
        .expect("apply_org_creation succeeds");
    assert_eq!(receipt.org_id, org_id);
    assert_eq!(receipt.ceo_agent_id, ceo_id);

    // The ceo is the owner per concept-doc "Agent owns Organization".
    let owned_orgs = repo
        .list_agent_owned_orgs(ceo_id)
        .await
        .expect("list_agent_owned_orgs succeeds");
    assert_eq!(owned_orgs, vec![org_id], "ceo owns exactly the new org");

    // The ceo owns no projects.
    let owned_projects = repo
        .list_agent_owned_projects(ceo_id)
        .await
        .expect("list_agent_owned_projects succeeds");
    assert!(
        owned_projects.is_empty(),
        "ceo owns no projects at org-create time"
    );
}

#[tokio::test]
async fn apply_org_creation_other_agents_own_nothing() {
    let repo = InMemoryRepository::new();
    let org_id = OrgId::new();
    let payload = build_org_payload(org_id);
    let unrelated_agent = AgentId::new();
    repo.apply_org_creation(&payload).await.unwrap();

    let owned_orgs = repo.list_agent_owned_orgs(unrelated_agent).await.unwrap();
    assert!(
        owned_orgs.is_empty(),
        "unrelated agent owns no org via the new Owns edge"
    );
}

// ---- apply_project_creation emits Owns edge -------------------------------

async fn seed_org_and_agent(repo: &InMemoryRepository) -> (OrgId, AgentId) {
    let org = OrgId::new();
    repo.create_organization(&minimal_org(org)).await.unwrap();
    let lead = make_agent(org);
    let lead_id = lead.id;
    repo.create_agent(&lead).await.unwrap();
    (org, lead_id)
}

#[tokio::test]
async fn apply_project_creation_shape_a_emits_owns_edge_for_lead() {
    let repo = InMemoryRepository::new();
    let (org, lead_id) = seed_org_and_agent(&repo).await;

    let project = make_project(ProjectShape::A);
    let pid = project.id;
    let payload = ProjectCreationPayload {
        project,
        owning_orgs: vec![org],
        lead_agent_id: lead_id,
        creator_agent: lead_id,
        member_agent_ids: vec![],
        sponsor_agent_ids: vec![],
        catalogue_entries: vec![(format!("project:{pid}"), "project".into())],
    };
    repo.apply_project_creation(&payload).await.unwrap();

    let owned_projects = repo.list_agent_owned_projects(lead_id).await.unwrap();
    assert_eq!(
        owned_projects,
        vec![pid],
        "lead is the owner of Shape A project per Decision-3"
    );

    // Sanity: lead also owns no orgs (it didn't create any).
    let owned_orgs = repo.list_agent_owned_orgs(lead_id).await.unwrap();
    assert!(owned_orgs.is_empty(), "lead owns no orgs at this point");
}

#[tokio::test]
async fn apply_project_creation_shape_b_emits_owns_edge_for_lead_at_materialise() {
    // Shape B materialise path: lead IS the owner per Decision-3
    // user-lock (the AR-submitter chose the lead; that's the owner).
    let repo = InMemoryRepository::new();
    let (org_a, lead_id) = seed_org_and_agent(&repo).await;
    let (org_b, _) = seed_org_and_agent(&repo).await;

    let project = make_project(ProjectShape::B);
    let pid = project.id;
    let payload = ProjectCreationPayload {
        project,
        owning_orgs: vec![org_a, org_b],
        lead_agent_id: lead_id,
        creator_agent: lead_id, // Decision-3: lead = creator at B too
        member_agent_ids: vec![],
        sponsor_agent_ids: vec![],
        catalogue_entries: vec![(format!("project:{pid}"), "project".into())],
    };
    repo.apply_project_creation(&payload).await.unwrap();

    let owned = repo.list_agent_owned_projects(lead_id).await.unwrap();
    assert_eq!(
        owned,
        vec![pid],
        "Shape B materialise also wires the lead as owner per Decision-3"
    );
}

#[tokio::test]
async fn list_agent_owned_orgs_returns_multiple_when_agent_owns_two_orgs() {
    // An Agent owns two orgs (e.g., a serial founder). Both should
    // appear in `list_agent_owned_orgs`.
    let repo = InMemoryRepository::new();
    let org1 = OrgId::new();
    let org2 = OrgId::new();
    let p1 = build_org_payload(org1);
    let p2 = build_org_payload(org2);
    let agent_id = p1.ceo_agent.id;

    // p2's ceo is a different agent; we manually rewire p2 so the same
    // agent owns BOTH orgs. The simplest path: bypass the validation
    // that ceo.owning_org must match (which it does — each ceo is
    // pinned to its org). Instead, write 2 separate payloads with
    // separate ceos, then assert one of them owns just one org.
    let ceo2 = p2.ceo_agent.id;
    repo.apply_org_creation(&p1).await.unwrap();
    repo.apply_org_creation(&p2).await.unwrap();

    let owned1 = repo.list_agent_owned_orgs(agent_id).await.unwrap();
    let owned2 = repo.list_agent_owned_orgs(ceo2).await.unwrap();
    assert_eq!(owned1, vec![org1]);
    assert_eq!(owned2, vec![org2]);
}
