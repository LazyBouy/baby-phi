//! CH-16 / D-new-01 + D-new-23 — in-memory Repository Identity tests.
//!
//! Concept docs:
//! - `concepts/agent.md` § "Identity (Emergent, Event-Driven)" — 4-field
//!   v0 commitment + reactive update model.
//! - `concepts/human-agent.md` § "No Identity" — Human Agents have no
//!   system-computed Identity; defensive guard at `upsert_identity` per
//!   ADR-0039 §D39.1.

use chrono::Utc;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentRole, Identity};
use domain::repository::{Repository, RepositoryError};

/// Helper — an LLM agent registered in the in-memory repo.
async fn seed_llm_agent(repo: &InMemoryRepository, org: OrgId) -> AgentId {
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
    repo.create_agent(&agent).await.expect("create_agent llm");
    aid
}

/// Helper — a Human agent registered in the in-memory repo.
async fn seed_human_agent(repo: &InMemoryRepository, org: OrgId) -> AgentId {
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
    repo.create_agent(&agent).await.expect("create_agent human");
    aid
}

#[tokio::test]
async fn upsert_identity_round_trips_for_llm_agent() {
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;

    let iden = Identity::default_for_llm(aid, Utc::now());
    repo.upsert_identity(&iden).await.expect("upsert_identity");

    let got = repo
        .get_identity(aid)
        .await
        .expect("get_identity")
        .expect("identity present after upsert");
    assert_eq!(got, iden);
}

#[tokio::test]
async fn upsert_identity_replaces_on_repeat_call() {
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;

    let mut iden = Identity::default_for_llm(aid, Utc::now());
    repo.upsert_identity(&iden).await.unwrap();

    iden.self_description = "agent-authored bio".into();
    iden.lived.sessions_completed = 5;
    iden.updated_at = Utc::now();
    repo.upsert_identity(&iden).await.unwrap();

    let got = repo.get_identity(aid).await.unwrap().unwrap();
    assert_eq!(got.self_description, "agent-authored bio");
    assert_eq!(got.lived.sessions_completed, 5);
}

#[tokio::test]
async fn upsert_identity_rejects_human_agent_with_typed_error() {
    let repo = InMemoryRepository::new();
    let aid = seed_human_agent(&repo, OrgId::new()).await;

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
async fn delete_identity_removes_the_row() {
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;

    let iden = Identity::default_for_llm(aid, Utc::now());
    repo.upsert_identity(&iden).await.unwrap();
    assert!(repo.get_identity(aid).await.unwrap().is_some());

    repo.delete_identity(aid).await.unwrap();
    assert!(repo.get_identity(aid).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_identity_is_idempotent_on_missing_row() {
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;

    repo.delete_identity(aid).await.expect("missing-row delete");
    assert!(repo.get_identity(aid).await.unwrap().is_none());
}

#[tokio::test]
async fn list_identities_for_org_filters_by_owning_org() {
    let repo = InMemoryRepository::new();
    let org_a = OrgId::new();
    let org_b = OrgId::new();

    let a1 = seed_llm_agent(&repo, org_a).await;
    let a2 = seed_llm_agent(&repo, org_a).await;
    let b1 = seed_llm_agent(&repo, org_b).await;
    let now = Utc::now();
    repo.upsert_identity(&Identity::default_for_llm(a1, now))
        .await
        .unwrap();
    repo.upsert_identity(&Identity::default_for_llm(a2, now))
        .await
        .unwrap();
    repo.upsert_identity(&Identity::default_for_llm(b1, now))
        .await
        .unwrap();

    let in_a = repo.list_identities_for_org(org_a).await.unwrap();
    let in_b = repo.list_identities_for_org(org_b).await.unwrap();
    assert_eq!(in_a.len(), 2);
    assert_eq!(in_b.len(), 1);
    assert!(in_a.iter().any(|i| i.agent_id == a1));
    assert!(in_a.iter().any(|i| i.agent_id == a2));
    assert!(in_b.iter().any(|i| i.agent_id == b1));
}

#[tokio::test]
async fn list_identities_for_org_excludes_human_agents() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    let llm = seed_llm_agent(&repo, org).await;
    let _human = seed_human_agent(&repo, org).await;
    repo.upsert_identity(&Identity::default_for_llm(llm, Utc::now()))
        .await
        .unwrap();
    // Human agent never gets a row (defensive guard) — so the listing
    // for `org` should contain only the LLM agent's identity.

    let listed = repo.list_identities_for_org(org).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].agent_id, llm);
}

// ---- ADR-0038 §D38.6 — orphan-on-archive policy ratification --------------

#[tokio::test]
async fn archive_does_not_delete_identity_row() {
    // ADR-0038 §D38.6 (Fork 4 LEAVE QUERYABLE): archiving an LLM agent
    // flips `Agent.active = false` but the Identity row stays
    // queryable. Concept-`agent.md` § "Materialization" treats Identity
    // as a continuously-updated record; preserving it after archive
    // supports forensic / hiring / evaluation queries.
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;
    repo.upsert_identity(&Identity::default_for_llm(aid, Utc::now()))
        .await
        .unwrap();

    // Archive the agent (ADR-0034 / CH-01 lifecycle).
    repo.set_agent_archived_at(aid, Some(Utc::now()))
        .await
        .expect("archive llm agent");

    // Identity is still queryable — no auto-cascade delete.
    let got = repo.get_identity(aid).await.unwrap();
    assert!(
        got.is_some(),
        "Identity must survive archive; ADR-0038 §D38.6 LEAVE QUERYABLE policy"
    );
}

#[tokio::test]
async fn operator_driven_delete_identity_after_archive_succeeds() {
    // ADR-0038 §D38.6 second half: `delete_identity` exists for
    // operator-driven cleanup. Archive does NOT call it; an operator
    // who needs the Identity gone (GDPR erasure, etc.) calls it
    // explicitly.
    let repo = InMemoryRepository::new();
    let aid = seed_llm_agent(&repo, OrgId::new()).await;
    repo.upsert_identity(&Identity::default_for_llm(aid, Utc::now()))
        .await
        .unwrap();
    repo.set_agent_archived_at(aid, Some(Utc::now()))
        .await
        .unwrap();
    // Confirm row still exists post-archive (orphan policy).
    assert!(repo.get_identity(aid).await.unwrap().is_some());
    // Operator-driven cleanup.
    repo.delete_identity(aid).await.unwrap();
    assert!(repo.get_identity(aid).await.unwrap().is_none());
}
