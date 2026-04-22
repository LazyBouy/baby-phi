//! Integration tests for `Repository::apply_agent_creation`
//! (M4/P3 plan commitment **C10**) against the real SurrealDB backend.
//!
//! Covers:
//! 1. Happy path — Agent + inbox + outbox + edges.
//! 2. Happy path with initial ExecutionLimits override (ADR-0027).
//! 3. Rollback on role-kind mismatch (role invalid for kind).
//! 4. Rollback on agent without owning_org.

use chrono::Utc;

use domain::audit::AuditClass;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, NodeId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentRole, InboxObject, Organization, OutboxObject};
use domain::model::AgentExecutionLimitsOverride;
use domain::repository::{AgentCreationPayload, Repository, RepositoryError};
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

fn limits(max_turns: usize) -> phi_core::context::execution::ExecutionLimits {
    phi_core::context::execution::ExecutionLimits {
        max_turns,
        max_total_tokens: 500_000,
        max_duration: std::time::Duration::from_secs(300),
        max_cost: Some(5.0),
    }
}

#[tokio::test]
async fn happy_path_agent_plus_inbox_outbox_plus_edges() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();

    let agent = make_agent(org, AgentKind::Llm, Some(AgentRole::Intern));
    let agent_id = agent.id;
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let receipt = store
        .apply_agent_creation(&AgentCreationPayload {
            agent,
            inbox,
            outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await
        .expect("apply_agent_creation ok");
    assert_eq!(receipt.agent_id, agent_id);
    assert_eq!(receipt.owning_org_id, org);

    // Agent reachable via list_agents_in_org.
    let in_org = store.list_agents_in_org(org).await.unwrap();
    assert!(in_org.iter().any(|a| a.id == agent_id));
}

#[tokio::test]
async fn happy_path_with_initial_execution_limits_override() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();

    let agent = make_agent(org, AgentKind::Llm, Some(AgentRole::Intern));
    let agent_id = agent.id;
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let ovr = AgentExecutionLimitsOverride {
        id: NodeId::new(),
        owning_agent: agent_id,
        limits: limits(25),
        created_at: Utc::now(),
    };
    store
        .apply_agent_creation(&AgentCreationPayload {
            agent,
            inbox,
            outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: Some(ovr),
            catalogue_entries: vec![],
        })
        .await
        .expect("apply_agent_creation with override");

    let hit = store
        .get_agent_execution_limits_override(agent_id)
        .await
        .unwrap()
        .expect("override persisted");
    assert_eq!(hit.limits.max_turns, 25);
}

#[tokio::test]
async fn role_kind_mismatch_is_rejected_before_open_tx() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store.create_organization(&minimal_org(org)).await.unwrap();

    // Executive is a Human-only role; pairing with Llm fails
    // `is_valid_for(kind)`.
    let agent = make_agent(org, AgentKind::Llm, Some(AgentRole::Executive));
    let agent_id = agent.id;
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let bad = store
        .apply_agent_creation(&AgentCreationPayload {
            agent,
            inbox,
            outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await;
    assert!(matches!(bad, Err(RepositoryError::InvalidArgument(_))));

    // Nothing persisted.
    let out = store.get_agent(agent_id).await.unwrap();
    assert!(out.is_none());
}

#[tokio::test]
async fn agent_without_owning_org_is_rejected() {
    let (store, _dir) = fresh_store().await;

    let mut agent = make_agent(OrgId::new(), AgentKind::Llm, None);
    agent.owning_org = None; // force the pre-check failure
    let agent_id = agent.id;
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id,
        created_at: Utc::now(),
    };
    let bad = store
        .apply_agent_creation(&AgentCreationPayload {
            agent,
            inbox,
            outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await;
    assert!(matches!(bad, Err(RepositoryError::InvalidArgument(_))));
    let out = store.get_agent(agent_id).await.unwrap();
    assert!(out.is_none());
}
