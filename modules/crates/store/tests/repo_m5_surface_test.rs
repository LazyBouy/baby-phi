//! M5/P2 integration tests for the `SurrealStore: Repository`
//! surface. Mirrors `domain/tests/in_memory_m5_test.rs` against the
//! real SurrealDB backend so the two impls stay in lock-step (M5
//! plan commitment C6).
//!
//! Drift-addendum invariant D1.1 (from M5/P1 close): the `session` +
//! `turn` tables carry a mandatory `created_at: string` column from
//! 0001. The P2 persist writers populate it alongside `started_at`.
//!
//! Tests exercise that path end-to-end.

use chrono::{TimeZone, Utc};
use tempfile::TempDir;

use domain::model::ids::{
    AgentCatalogEntryId, AgentId, AuthRequestId, LoopId, OrgId, ProjectId, SessionId,
    SystemAgentRuntimeStatusId,
};
use domain::model::nodes::{AgentKind, LoopRecordNode, Session, SessionGovernanceState};
use domain::model::{AgentCatalogEntry, ShapeBPendingProject, SystemAgentRuntimeStatus};
use domain::repository::Repository;
use store::SurrealStore;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

fn sample_phi_core_session() -> phi_core::session::model::Session {
    serde_json::from_value(serde_json::json!({
        "session_id": "s-0001",
        "agent_id": "a-0001",
        "created_at": "2026-04-23T00:00:00Z",
        "last_active_at": "2026-04-23T00:00:00Z",
        "formation": {"Explicit": {"timestamp": "2026-04-23T00:00:00Z"}},
        "parent_spawn_ref": null,
        "scope": "ephemeral",
        "loops": []
    }))
    .expect("phi-core Session JSON deserialises")
}

fn sample_phi_core_loop_record() -> phi_core::session::model::LoopRecord {
    serde_json::from_value(serde_json::json!({
        "loop_id": "l-0001",
        "session_id": "s-0001",
        "agent_id": "a-0001",
        "parent_loop_id": null,
        "started_at": "2026-04-23T00:00:00Z",
        "ended_at": null,
        "status": "Running",
        "rejection": null,
        "config": null,
        "messages": [],
        "usage": {"input": 0, "output": 0, "reasoning": 0, "cache_read": 0, "cache_write": 0, "total_tokens": 0},
        "metadata": null,
        "events": [],
        "children_loop_ids": [],
        "child_loop_refs": [],
        "parallel_group": null
    }))
    .expect("phi-core LoopRecord JSON deserialises")
}

fn make_session(started_by: AgentId, project: ProjectId, org: OrgId, started_hour: u32) -> Session {
    Session {
        id: SessionId::new(),
        inner: sample_phi_core_session(),
        owning_org: org,
        owning_project: project,
        started_by,
        governance_state: SessionGovernanceState::Running,
        started_at: Utc
            .with_ymd_and_hms(2026, 4, 23, started_hour, 0, 0)
            .unwrap(),
        ended_at: None,
        tokens_spent: 0,
    }
}

fn make_loop(session_id: SessionId, loop_index: u32) -> LoopRecordNode {
    LoopRecordNode {
        id: LoopId::new(),
        inner: sample_phi_core_loop_record(),
        session_id,
        loop_index,
    }
}

// -------- Session tier ------------------------------------------------------

#[tokio::test]
async fn persist_session_round_trips_via_fetch_session() {
    let (store, _dir) = fresh_store().await;
    let s = make_session(AgentId::new(), ProjectId::new(), OrgId::new(), 10);
    let lr = make_loop(s.id, 0);
    store.persist_session(&s, &lr).await.unwrap();
    let got = store.fetch_session(s.id).await.unwrap();
    assert!(got.is_some());
    let detail = got.unwrap();
    assert_eq!(detail.session.id, s.id);
    assert_eq!(
        detail.session.governance_state,
        SessionGovernanceState::Running
    );
    assert_eq!(detail.loops.len(), 1);
    assert_eq!(detail.loops[0].id, lr.id);
    assert!(detail
        .turns_by_loop
        .get(&lr.id)
        .is_some_and(|v| v.is_empty()));
}

#[tokio::test]
async fn fetch_session_returns_none_for_unknown_id() {
    let (store, _dir) = fresh_store().await;
    assert!(store
        .fetch_session(SessionId::new())
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn list_sessions_in_project_orders_newest_first() {
    let (store, _dir) = fresh_store().await;
    let project = ProjectId::new();
    let agent = AgentId::new();
    let org = OrgId::new();
    let s_early = make_session(agent, project, org, 8);
    let s_late = make_session(agent, project, org, 18);
    store
        .persist_session(&s_early, &make_loop(s_early.id, 0))
        .await
        .unwrap();
    store
        .persist_session(&s_late, &make_loop(s_late.id, 0))
        .await
        .unwrap();
    let out = store.list_sessions_in_project(project).await.unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].id, s_late.id);
    assert_eq!(out[1].id, s_early.id);
}

#[tokio::test]
async fn count_active_sessions_for_agent_flips_from_zero_stub() {
    // C-M5-5 SurrealDB flip: the M4 trait default was Ok(0). After
    // M5/P2 the SurrealDB impl overrides with a real SurrealQL
    // count() query. Acceptance path for the M4/P5 409
    // `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` gate.
    let (store, _dir) = fresh_store().await;
    let agent = AgentId::new();
    let project = ProjectId::new();
    let org = OrgId::new();
    assert_eq!(
        store.count_active_sessions_for_agent(agent).await.unwrap(),
        0
    );
    let s1 = make_session(agent, project, org, 10);
    store
        .persist_session(&s1, &make_loop(s1.id, 0))
        .await
        .unwrap();
    assert_eq!(
        store.count_active_sessions_for_agent(agent).await.unwrap(),
        1
    );
    let s2 = make_session(agent, project, org, 11);
    store
        .persist_session(&s2, &make_loop(s2.id, 0))
        .await
        .unwrap();
    assert_eq!(
        store.count_active_sessions_for_agent(agent).await.unwrap(),
        2
    );
    // Terminal sessions do NOT count.
    store
        .mark_session_ended(s1.id, Utc::now(), SessionGovernanceState::Completed)
        .await
        .unwrap();
    assert_eq!(
        store.count_active_sessions_for_agent(agent).await.unwrap(),
        1
    );
}

#[tokio::test]
async fn mark_session_ended_rejects_already_terminal() {
    let (store, _dir) = fresh_store().await;
    let s = make_session(AgentId::new(), ProjectId::new(), OrgId::new(), 10);
    store
        .persist_session(&s, &make_loop(s.id, 0))
        .await
        .unwrap();
    store
        .mark_session_ended(s.id, Utc::now(), SessionGovernanceState::Completed)
        .await
        .unwrap();
    let again = store
        .mark_session_ended(s.id, Utc::now(), SessionGovernanceState::Aborted)
        .await;
    assert!(matches!(
        again,
        Err(domain::repository::RepositoryError::Conflict(_))
    ));
}

#[tokio::test]
async fn terminate_session_transitions_to_aborted() {
    let (store, _dir) = fresh_store().await;
    let s = make_session(AgentId::new(), ProjectId::new(), OrgId::new(), 10);
    store
        .persist_session(&s, &make_loop(s.id, 0))
        .await
        .unwrap();
    store.terminate_session(s.id, Utc::now()).await.unwrap();
    let detail = store.fetch_session(s.id).await.unwrap().unwrap();
    assert_eq!(
        detail.session.governance_state,
        SessionGovernanceState::Aborted
    );
}

// -------- Shape B sidecar ---------------------------------------------------

#[tokio::test]
async fn shape_b_sidecar_round_trip_via_surrealdb() {
    let (store, _dir) = fresh_store().await;
    let ar_id = AuthRequestId::new();
    let row = ShapeBPendingProject {
        auth_request_id: ar_id,
        payload: serde_json::json!({"name": "demo", "shape": "shape_b"}),
        created_at: Utc::now(),
    };
    store.persist_shape_b_pending(&row).await.unwrap();
    let got = store.fetch_shape_b_pending(ar_id).await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().auth_request_id, ar_id);
    store.delete_shape_b_pending(ar_id).await.unwrap();
    assert!(store.fetch_shape_b_pending(ar_id).await.unwrap().is_none());
}

#[tokio::test]
async fn shape_b_sidecar_unique_index_enforced_at_surrealdb_tier() {
    let (store, _dir) = fresh_store().await;
    let ar_id = AuthRequestId::new();
    let row = ShapeBPendingProject {
        auth_request_id: ar_id,
        payload: serde_json::json!({}),
        created_at: Utc::now(),
    };
    store.persist_shape_b_pending(&row).await.unwrap();
    let dup = store
        .persist_shape_b_pending(&ShapeBPendingProject {
            auth_request_id: ar_id,
            payload: serde_json::json!({"note": "second attempt"}),
            created_at: Utc::now(),
        })
        .await;
    assert!(matches!(
        dup,
        Err(domain::repository::RepositoryError::Conflict(_))
    ));
}

// -------- Agent catalog -----------------------------------------------------

fn catalog_entry(org: OrgId, agent: AgentId, name: &str) -> AgentCatalogEntry {
    AgentCatalogEntry {
        id: AgentCatalogEntryId::new(),
        agent_id: agent,
        owning_org: org,
        display_name: name.into(),
        kind: AgentKind::Llm,
        role: Some("system".into()),
        active: true,
        profile_snapshot: None,
        last_seen_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn agent_catalog_upsert_round_trips() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let agent = AgentId::new();
    let entry = catalog_entry(org, agent, "memory-extraction");
    store.upsert_agent_catalog_entry(&entry).await.unwrap();
    let got = store.get_agent_catalog_entry(agent).await.unwrap().unwrap();
    assert_eq!(got.agent_id, agent);
    assert_eq!(got.display_name, "memory-extraction");
}

#[tokio::test]
async fn agent_catalog_list_scopes_by_org() {
    let (store, _dir) = fresh_store().await;
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    store
        .upsert_agent_catalog_entry(&catalog_entry(org_a, AgentId::new(), "a1"))
        .await
        .unwrap();
    store
        .upsert_agent_catalog_entry(&catalog_entry(org_a, AgentId::new(), "a2"))
        .await
        .unwrap();
    store
        .upsert_agent_catalog_entry(&catalog_entry(org_b, AgentId::new(), "b1"))
        .await
        .unwrap();
    let a_rows = store
        .list_agent_catalog_entries_in_org(org_a)
        .await
        .unwrap();
    assert_eq!(a_rows.len(), 2);
    let b_rows = store
        .list_agent_catalog_entries_in_org(org_b)
        .await
        .unwrap();
    assert_eq!(b_rows.len(), 1);
}

// -------- System-agent runtime status ---------------------------------------

#[tokio::test]
async fn system_agent_runtime_status_round_trips() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let row = SystemAgentRuntimeStatus {
        id: SystemAgentRuntimeStatusId::new(),
        agent_id: AgentId::new(),
        owning_org: org,
        queue_depth: 3,
        last_fired_at: None,
        effective_parallelize: 2,
        last_error: None,
        updated_at: Utc::now(),
    };
    store
        .upsert_system_agent_runtime_status(&row)
        .await
        .unwrap();
    let rows = store
        .fetch_system_agent_runtime_status_for_org(org)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].queue_depth, 3);
}

// -------- Template reads ----------------------------------------------------

#[tokio::test]
async fn list_authority_templates_for_org_returns_empty_before_seed() {
    // The platform-level Template rows are seeded by bootstrap-init
    // / migration seed at M5/P5+. Pre-seed the table is empty; the
    // handler tier at M5/P5 can then augment per-org adoption state
    // on top of whatever rows the read returns.
    let (store, _dir) = fresh_store().await;
    let out = store
        .list_authority_templates_for_org(OrgId::new())
        .await
        .unwrap();
    assert!(out.is_empty());
}

#[tokio::test]
async fn count_grants_fired_by_adoption_returns_zero_for_unknown_ar() {
    let (store, _dir) = fresh_store().await;
    assert_eq!(
        store
            .count_grants_fired_by_adoption(AuthRequestId::new())
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn list_revoked_adoptions_for_org_returns_empty_baseline() {
    let (store, _dir) = fresh_store().await;
    let out = store
        .list_revoked_adoptions_for_org(OrgId::new())
        .await
        .unwrap();
    assert!(out.is_empty());
}

// -------- Phi-core leverage verification ------------------------------------

#[test]
fn repo_impl_carries_zero_new_phi_core_imports_at_p2() {
    // Q1 close-audit: the SurrealDB impl file must NOT add any new
    // `use phi_core::*` imports at M5/P2. All phi-core types flow
    // through the already-wrapped domain::model::{Session,
    // LoopRecordNode, TurnNode} nodes.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/repo_impl.rs"))
        .expect("read repo_impl.rs");
    let phi_core_imports = src
        .lines()
        .filter(|l| l.trim_start().starts_with("use phi_core::"))
        .count();
    assert_eq!(
        phi_core_imports, 0,
        "repo_impl.rs must ship zero `use phi_core::*` lines at M5/P2 close"
    );
}
