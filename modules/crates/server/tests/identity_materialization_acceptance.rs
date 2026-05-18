//! CH-16 / D-new-01 + D-new-23 — end-to-end Identity materialization
//! acceptance test. Exercises the full `apply_agent_creation` compound
//! tx through the in-memory Repository + audit emitter, asserting the
//! six P3-locked scenarios from the chunk plan §7 P3.

use chrono::Utc;
use std::sync::Arc;

use domain::audit::events::m5_2::identity::{identity_created, identity_updated};
use domain::audit::AuditClass;
use domain::events::IdentityUpdateTrigger;
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::NodeId;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, ExtractionScopeDistribution, Identity, InboxObject,
    LivedExperience, Organization, OutboxObject, RatingPoint, SkillRef, WitnessedExperience,
};
use domain::repository::{AgentCreationPayload, Repository, RepositoryError};

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
        approval_timeout: domain::model::ApprovalTimeout::ProjectDuration,
        approval_timeout_default_response: domain::model::TimeoutResponse::Deny,
        tags: vec![],
        created_at: Utc::now(),
    }
}

fn make_payload(agent: Agent, identity: Option<Identity>) -> AgentCreationPayload {
    let now = Utc::now();
    let aid = agent.id;
    AgentCreationPayload {
        agent,
        inbox: InboxObject {
            id: NodeId::new(),
            agent_id: aid,
            created_at: now,
            tags: vec![],
        },
        outbox: OutboxObject {
            id: NodeId::new(),
            agent_id: aid,
            created_at: now,
            tags: vec![],
        },
        profile: None,
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity,
    }
}

async fn fresh_repo_with_org() -> (Arc<InMemoryRepository>, OrgId) {
    let repo = Arc::new(InMemoryRepository::new());
    let org = OrgId::new();
    repo.create_organization(&minimal_org(org)).await.unwrap();
    (repo, org)
}

// ---- Six P3-locked scenarios from plan §7 P3 -------------------------------

#[tokio::test]
async fn scenario_1_creating_an_llm_agent_materialises_identity_row() {
    let (repo, org) = fresh_repo_with_org().await;
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "test-llm".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Intern),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let aid = agent.id;
    let now = Utc::now();
    let payload = make_payload(agent, Some(Identity::default_for_llm(aid, now)));
    let receipt = repo.apply_agent_creation(&payload).await.unwrap();
    assert!(receipt.identity_id.is_some());

    let iden = repo
        .get_identity(aid)
        .await
        .unwrap()
        .expect("identity row materialised");
    assert_eq!(iden.agent_id, aid);
    assert_eq!(iden.self_description, "");
    assert_eq!(iden.lived, LivedExperience::default());
    assert_eq!(iden.witnessed, WitnessedExperience::default());
    assert!(iden.embedding.is_empty());
}

#[tokio::test]
async fn scenario_2_creating_a_human_agent_does_not_materialise_identity() {
    let (repo, org) = fresh_repo_with_org().await;
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "test-human".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let aid = agent.id;
    let payload = make_payload(agent, None);
    let receipt = repo.apply_agent_creation(&payload).await.unwrap();
    assert!(receipt.identity_id.is_none());
    assert!(repo.get_identity(aid).await.unwrap().is_none());
}

#[tokio::test]
async fn scenario_3_direct_upsert_for_human_kind_returns_typed_error() {
    let (repo, org) = fresh_repo_with_org().await;
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "test-human".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let aid = agent.id;
    let payload = make_payload(agent, None);
    repo.apply_agent_creation(&payload).await.unwrap();
    let iden = Identity::default_for_llm(aid, Utc::now());
    let err = repo
        .upsert_identity(&iden)
        .await
        .expect_err("Human kind must reject");
    match err {
        RepositoryError::HumanAgentHasNoIdentity { agent_id } => assert_eq!(agent_id, aid),
        other => panic!("expected HumanAgentHasNoIdentity, got {:?}", other),
    }
}

#[tokio::test]
async fn scenario_4_list_identities_for_org_returns_only_llm_kind_entries() {
    let (repo, org) = fresh_repo_with_org().await;

    // Two LLM agents + one Human agent in the same org.
    let llm_a = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "llm-a".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Intern),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let llm_a_id = llm_a.id;
    let llm_b = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "llm-b".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Intern),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let llm_b_id = llm_b.id;
    let human = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "human-a".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let now = Utc::now();
    repo.apply_agent_creation(&make_payload(
        llm_a,
        Some(Identity::default_for_llm(llm_a_id, now)),
    ))
    .await
    .unwrap();
    repo.apply_agent_creation(&make_payload(
        llm_b,
        Some(Identity::default_for_llm(llm_b_id, now)),
    ))
    .await
    .unwrap();
    repo.apply_agent_creation(&make_payload(human, None))
        .await
        .unwrap();

    let listed = repo.list_identities_for_org(org).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|i| i.agent_id == llm_a_id));
    assert!(listed.iter().any(|i| i.agent_id == llm_b_id));
}

#[tokio::test]
async fn scenario_5_round_trip_serde_with_populated_lived_and_witnessed() {
    let (repo, org) = fresh_repo_with_org().await;
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "populated-llm".into(),
        owning_org: Some(org),
        role: Some(AgentRole::Intern),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let aid = agent.id;
    let now = Utc::now();
    let mut iden = Identity::default_for_llm(aid, now);
    iden.self_description = "agent-authored bio".into();
    iden.lived = LivedExperience {
        sessions_completed: 5,
        sessions_successful: 4,
        ratings_window: vec![RatingPoint {
            rater: AgentId::new(),
            score: 0.92,
            at: now,
        }],
        skills: vec![SkillRef {
            name: "rust".into(),
        }],
        specializations: vec!["security".into()],
    };
    iden.witnessed = WitnessedExperience {
        memories_extracted: 3,
        subordinates_observed: vec![AgentId::new()],
        extraction_scope_distribution: ExtractionScopeDistribution {
            private: 2,
            public: 1,
        },
    };
    repo.apply_agent_creation(&make_payload(agent, Some(iden.clone())))
        .await
        .unwrap();
    let got = repo.get_identity(aid).await.unwrap().unwrap();
    assert_eq!(got.self_description, iden.self_description);
    assert_eq!(got.lived, iden.lived);
    assert_eq!(got.witnessed, iden.witnessed);
}

#[tokio::test]
async fn scenario_6_audit_emitters_produce_correct_event_types_and_classes() {
    let (_repo, org) = fresh_repo_with_org().await;
    let aid = AgentId::new();
    let now = Utc::now();
    let iden = Identity::default_for_llm(aid, now);

    // platform.identity.created — Alerted
    let created = identity_created(AgentId::new(), &iden, org, None, now);
    assert_eq!(created.event_type, "platform.identity.created");
    assert_eq!(created.audit_class, AuditClass::Alerted);
    assert_eq!(created.target_entity_id, Some(iden.id));
    assert_eq!(created.org_scope, Some(org));

    // platform.identity.updated — Logged + trigger label
    let mut after = iden.clone();
    after.self_description = "synthesised".into();
    let updated = identity_updated(
        AgentId::new(),
        &iden,
        &after,
        IdentityUpdateTrigger::MemoryExtracted,
        org,
        now,
    );
    assert_eq!(updated.event_type, "platform.identity.updated");
    assert_eq!(updated.audit_class, AuditClass::Logged);
    assert_eq!(
        updated.diff["trigger"].as_str().unwrap(),
        "memory_extracted"
    );
}
