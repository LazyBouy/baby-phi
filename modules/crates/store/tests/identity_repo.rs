//! CH-16 / D-new-01 + D-new-23 — SurrealDB-backed Identity repo tests.
//!
//! Mirrors `domain/tests/in_memory_identity.rs` against the real
//! SurrealDB backend so the two impls stay in lock-step (per the M5
//! plan commitment that the in-memory + Surreal Repository surfaces
//! present identical semantics).

use chrono::Utc;
use tempfile::TempDir;

use domain::audit::AuditClass;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentRole, Identity, Organization};
use domain::repository::{Repository, RepositoryError};
use store::SurrealStore;

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

async fn seed_llm_agent(store: &SurrealStore, org: OrgId) -> AgentId {
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
    store.create_agent(&agent).await.expect("create_agent llm");
    aid
}

async fn seed_human_agent(store: &SurrealStore, org: OrgId) -> AgentId {
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
    store
        .create_agent(&agent)
        .await
        .expect("create_agent human");
    aid
}

#[tokio::test]
async fn upsert_identity_round_trips_via_surrealdb() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let aid = seed_llm_agent(&store, org).await;

    let iden = Identity::default_for_llm(aid, Utc::now());
    store.upsert_identity(&iden).await.expect("upsert_identity");

    let got = store
        .get_identity(aid)
        .await
        .expect("get_identity")
        .expect("identity present after upsert");
    assert_eq!(got.agent_id, aid);
    assert_eq!(got.self_description, "");
    assert!(got.embedding.is_empty());
}

#[tokio::test]
async fn upsert_identity_rejects_human_agent_at_surreal_layer() {
    // Defensive guard at the SurrealQL layer (ADR-0039 §D39.1) — the
    // Rust-side check reads `Agent.kind` and rejects before the
    // SurrealDB write is attempted.
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let aid = seed_human_agent(&store, org).await;

    let iden = Identity::default_for_llm(aid, Utc::now());
    let err = store
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
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let aid = seed_llm_agent(&store, org).await;

    store
        .upsert_identity(&Identity::default_for_llm(aid, Utc::now()))
        .await
        .unwrap();
    assert!(store.get_identity(aid).await.unwrap().is_some());

    store.delete_identity(aid).await.unwrap();
    assert!(store.get_identity(aid).await.unwrap().is_none());
}

#[tokio::test]
async fn list_identities_for_org_filters_correctly() {
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

    let a1 = seed_llm_agent(&store, org_a).await;
    let a2 = seed_llm_agent(&store, org_a).await;
    let b1 = seed_llm_agent(&store, org_b).await;
    let now = Utc::now();
    for aid in [a1, a2, b1] {
        store
            .upsert_identity(&Identity::default_for_llm(aid, now))
            .await
            .unwrap();
    }

    let in_a = store.list_identities_for_org(org_a).await.unwrap();
    let in_b = store.list_identities_for_org(org_b).await.unwrap();
    assert_eq!(in_a.len(), 2);
    assert_eq!(in_b.len(), 1);
    assert!(in_a.iter().any(|i| i.agent_id == a1));
    assert!(in_a.iter().any(|i| i.agent_id == a2));
    assert!(in_b.iter().any(|i| i.agent_id == b1));
}
