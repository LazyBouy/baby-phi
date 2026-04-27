//! Unit-ish integration tests for the CH-01 / ADR-0034 D34.3 repo methods
//! `set_agent_active` and `set_agent_archived_at`.
//!
//! These exercise the Surreal-backed `Repository` impl against a fresh
//! embedded RocksDB store. The InMemory backend has the same trait
//! contract; tests here use Surreal to also validate the SurrealQL
//! UPDATE shape.

use chrono::{TimeZone, Utc};
use domain::model::ids::AgentId;
use domain::model::nodes::{Agent, AgentKind, AgentRole};
use domain::repository::RepositoryError;
use domain::Repository;
use store::SurrealStore;
use tempfile::tempdir;

async fn fresh_store() -> SurrealStore {
    let dir = tempdir().expect("tempdir");
    SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store")
}

fn sample_llm_agent() -> Agent {
    Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "test-llm".into(),
        owning_org: None,
        role: Some(AgentRole::System),
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    }
}

#[tokio::test]
async fn set_agent_active_flips_column_and_round_trips_through_get_agent() {
    let store = fresh_store().await;
    let agent = sample_llm_agent();
    store.create_agent(&agent).await.expect("create_agent");

    // Sanity — newly created agent is active.
    let before = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(before.active);

    // Disable.
    store
        .set_agent_active(agent.id, false)
        .await
        .expect("set_agent_active(false)");

    let after_disable = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(!after_disable.active, "active must be false after flip");

    // Re-enable (un-disable). Symmetric path; not exercised by the
    // current handler but shipped on the trait.
    store
        .set_agent_active(agent.id, true)
        .await
        .expect("set_agent_active(true)");
    let after_reenable = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(
        after_reenable.active,
        "active must be true again after re-enable"
    );
}

#[tokio::test]
async fn set_agent_archived_at_writes_rfc3339_and_round_trips() {
    let store = fresh_store().await;
    let agent = sample_llm_agent();
    store.create_agent(&agent).await.expect("create_agent");

    // Sanity — newly created agent is not archived.
    let before = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(before.archived_at.is_none());

    // Archive at a known timestamp. We use a fixed ymd_hms so the
    // RFC3339 round-trip equality is exact (no precision drift from
    // Utc::now() nanos).
    let archived_at = Utc.with_ymd_and_hms(2026, 4, 27, 10, 15, 30).unwrap();
    store
        .set_agent_archived_at(agent.id, Some(archived_at))
        .await
        .expect("set_agent_archived_at(Some)");

    let after = store.get_agent(agent.id).await.unwrap().unwrap();
    assert_eq!(
        after.archived_at,
        Some(archived_at),
        "archived_at must round-trip exactly through the option<string> column"
    );
}

#[tokio::test]
async fn set_agent_active_returns_not_found_for_missing_agent() {
    let store = fresh_store().await;
    let phantom_id = AgentId::new();
    let result = store.set_agent_active(phantom_id, false).await;
    assert!(
        matches!(result, Err(RepositoryError::NotFound)),
        "set_agent_active on missing agent must return NotFound, got {:?}",
        result
    );
}

#[tokio::test]
async fn set_agent_archived_at_with_none_clears_column() {
    let store = fresh_store().await;
    let agent = sample_llm_agent();
    store.create_agent(&agent).await.expect("create_agent");

    // First archive it.
    let archived_at = Utc.with_ymd_and_hms(2026, 4, 27, 10, 15, 30).unwrap();
    store
        .set_agent_archived_at(agent.id, Some(archived_at))
        .await
        .expect("set Some");
    let archived = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(archived.archived_at.is_some());

    // Now clear it (un-archive path; not exercised at M5 but shipped
    // for symmetry per ADR-0034 D34.3).
    store
        .set_agent_archived_at(agent.id, None)
        .await
        .expect("set None");

    let cleared = store.get_agent(agent.id).await.unwrap().unwrap();
    assert!(
        cleared.archived_at.is_none(),
        "archived_at must be None after clear-write"
    );
}

#[tokio::test]
async fn set_agent_archived_at_returns_not_found_for_missing_agent() {
    let store = fresh_store().await;
    let phantom_id = AgentId::new();
    let result = store
        .set_agent_archived_at(phantom_id, Some(Utc::now()))
        .await;
    assert!(
        matches!(result, Err(RepositoryError::NotFound)),
        "set_agent_archived_at on missing agent must return NotFound, got {:?}",
        result
    );
}
