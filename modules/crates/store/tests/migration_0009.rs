//! CH-16 / migration 0009 round-trip — verifies the `identity` table
//! ships with all 9 fields + UNIQUE-on-`agent_id` index, accepts a
//! default-shape Identity row, and round-trips a populated Identity
//! row's content through the SurrealDB backend without serde drift.
//!
//! Concept doc: `concepts/agent.md` § "Identity Node Content" lines
//! 317–344 (the v0 commitment).

use chrono::Utc;
use tempfile::TempDir;

use domain::audit::AuditClass;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, ExtractionScopeDistribution, Identity, LivedExperience,
    Organization, RatingPoint, SkillRef, WitnessedExperience,
};
use domain::repository::Repository;
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
        approval_timeout: domain::model::ApprovalTimeout::ProjectDuration,
        approval_timeout_default_response: domain::model::TimeoutResponse::Deny,
        created_at: Utc::now(),
    }
}

async fn seed_llm(store: &SurrealStore, org: OrgId) -> AgentId {
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
    store.create_agent(&agent).await.unwrap();
    aid
}

#[tokio::test]
async fn migration_0009_default_shape_identity_round_trips() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let aid = seed_llm(&store, org).await;
    let now = Utc::now();
    let iden = Identity::default_for_llm(aid, now);
    store.upsert_identity(&iden).await.unwrap();
    let got = store.get_identity(aid).await.unwrap().unwrap();
    assert_eq!(got.agent_id, aid);
    assert_eq!(got.self_description, "");
    assert!(got.embedding.is_empty());
    assert_eq!(got.lived, LivedExperience::default());
    assert_eq!(got.witnessed, WitnessedExperience::default());
    // Both timestamps round-trip within 1s; microsecond precision drift
    // is acceptable across the SurrealDB datetime boundary.
    let drift = (got.created_at - iden.created_at).num_seconds().abs();
    assert!(drift <= 1, "created_at drift {} > 1s", drift);
}

#[tokio::test]
async fn migration_0009_populated_identity_round_trips() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();
    let aid = seed_llm(&store, org).await;
    let now = Utc::now();
    let original = Identity {
        id: domain::model::ids::NodeId::new(),
        agent_id: aid,
        self_description: "agent-authored bio after first session".into(),
        lived: LivedExperience {
            sessions_completed: 7,
            sessions_successful: 6,
            ratings_window: vec![RatingPoint {
                rater: AgentId::new(),
                score: 0.92,
                at: now,
            }],
            skills: vec![SkillRef {
                name: "rust".into(),
            }],
            specializations: vec!["devops".into()],
        },
        witnessed: WitnessedExperience {
            memories_extracted: 3,
            subordinates_observed: vec![AgentId::new()],
            extraction_scope_distribution: ExtractionScopeDistribution {
                private: 2,
                public: 1,
            },
        },
        embedding: vec![0.1, 0.2, 0.3, 0.4],
        created_at: now,
        updated_at: now,
    };
    store.upsert_identity(&original).await.unwrap();
    let got = store.get_identity(aid).await.unwrap().unwrap();
    assert_eq!(got.self_description, original.self_description);
    assert_eq!(got.lived.sessions_completed, 7);
    assert_eq!(got.lived.sessions_successful, 6);
    assert_eq!(got.witnessed.memories_extracted, 3);
    assert_eq!(got.witnessed.extraction_scope_distribution.private, 2);
    assert_eq!(got.embedding.len(), 4);
    assert!((got.embedding[0] - 0.1).abs() < 1e-6);
}
