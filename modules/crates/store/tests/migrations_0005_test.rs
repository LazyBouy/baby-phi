//! Targeted integration test for migration
//! `0005_sessions_templates_system_agents.surql`.
//!
//! Verifies the M5/P1 schema additions are queryable against a live
//! SurrealDB instance — not just that the migration "applied
//! successfully" at the ledger level.
//!
//! Covers M5 plan commitment C2 (forward-only migration 0005 apply)
//! and the eight schema changes enumerated in the migration header.

use store::SurrealStore;
use tempfile::tempdir;

async fn fresh_store() -> SurrealStore {
    let dir = tempdir().expect("tempdir");
    SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store")
}

// ---- 1. template kind UNIQUE index — multi-org adoption unblocked ---------

#[tokio::test]
async fn template_kind_unique_index_permits_multi_org_adoption() {
    let store = fresh_store().await;

    // Insert Template A once — should succeed after migration 0005 drops
    // the old `template.name` UNIQUE index.
    store
        .client()
        .query(
            "CREATE type::thing('template', 't-a') SET \
             name = 'Template A', kind = 'a', \
             created_at = time::now()",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first check");

    // Two orgs adopting Template A no longer require two Template rows
    // (adoption lives on the AR's provenance_template per ADR-0030).
    // A second row with `kind = 'a'` must be rejected by the new
    // UNIQUE(kind) index.
    let dup = store
        .client()
        .query(
            "CREATE type::thing('template', 't-a-dup') SET \
             name = 'Template A again', kind = 'a', \
             created_at = time::now()",
        )
        .await
        .expect("issue dup insert")
        .check();
    assert!(
        dup.is_err(),
        "UNIQUE(kind) must reject a second Template row of the same kind"
    );
}

// ---- 2. uses_model retype — FROM agent TO model_runtime -------------------

#[tokio::test]
async fn uses_model_relation_retyped_to_model_runtime() {
    let store = fresh_store().await;
    let mut resp = store
        .client()
        .query("INFO FOR TABLE uses_model")
        .await
        .expect("info for uses_model");
    let info: Option<serde_json::Value> = resp.take(0).expect("take info");
    let info = info.expect("INFO FOR TABLE returned null");
    let text = serde_json::to_string(&info).unwrap();
    assert!(
        text.contains("model_runtime"),
        "uses_model must target model_runtime post-migration 0005 (got: {text})"
    );
    assert!(
        !text.contains("TO model_config"),
        "uses_model must no longer target model_config (got: {text})"
    );
}

// ---- 3. session / loop_record / turn — 3-way wrap tier --------------------

#[tokio::test]
async fn session_table_accepts_governance_row_with_inner_object() {
    let store = fresh_store().await;
    // The `session` table was scaffolded in 0001 with `created_at:
    // string` NOT NULL; M5/P1 layers inner / owning_org / etc. on
    // top. Production writes (the M5/P2 `persist_session` repo
    // method) must populate BOTH `created_at` and `started_at`
    // (typically the same wall-clock value).
    store
        .client()
        .query(
            "CREATE type::thing('session', 's-demo') SET \
             created_at = '2026-04-23T00:00:00Z', \
             inner = { session_id: 's-demo', agent_id: 'a-demo', \
                       created_at: '2026-04-23T00:00:00Z', \
                       last_active_at: '2026-04-23T00:00:00Z', \
                       formation: 'SpontaneousFollowup', \
                       parent_spawn_ref: null, loops: [] }, \
             owning_org = 'org-a', owning_project = 'p-a', \
             started_by = 'a-demo', governance_state = 'running', \
             started_at = '2026-04-23T00:00:00Z', tokens_spent = 0",
        )
        .await
        .expect("insert session")
        .check()
        .expect("check session");
}

#[tokio::test]
async fn session_governance_state_assert_rejects_unknown_value() {
    let store = fresh_store().await;
    let bad = store
        .client()
        .query(
            "CREATE type::thing('session', 's-bogus') SET \
             created_at = '2026-04-23T00:00:00Z', \
             inner = {}, owning_org = 'o', owning_project = 'p', \
             started_by = 'a', governance_state = 'running_amok', \
             started_at = '2026-04-23T00:00:00Z', tokens_spent = 0",
        )
        .await
        .expect("issue bad state")
        .check();
    assert!(
        bad.is_err(),
        "session.governance_state ASSERT must reject values outside the 4-variant set"
    );
}

// ---- 4. runs_session relation ---------------------------------------------

#[tokio::test]
async fn runs_session_relation_table_defined() {
    let store = fresh_store().await;
    let mut resp = store.client().query("INFO FOR DB").await.expect("info db");
    let info: Option<serde_json::Value> = resp.take(0).expect("take info");
    let info = info.expect("INFO FOR DB returned null");
    let tables = info
        .get("tables")
        .or_else(|| info.get("tb"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
    let text = serde_json::to_string(&tables).unwrap();
    assert!(
        text.contains("runs_session"),
        "migration 0005 must define RELATION table `runs_session` (INFO: {text})"
    );
}

// ---- 5. shape_b_pending_projects UNIQUE(auth_request_id) ------------------

#[tokio::test]
async fn shape_b_pending_projects_unique_index_rejects_duplicate_ar() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('shape_b_pending_projects', 'sb-1') SET \
             auth_request_id = 'ar-x', \
             payload = { name: 'demo' }, \
             created_at = time::now()",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first check");

    let dup = store
        .client()
        .query(
            "CREATE type::thing('shape_b_pending_projects', 'sb-2') SET \
             auth_request_id = 'ar-x', \
             payload = {}, \
             created_at = time::now()",
        )
        .await
        .expect("issue dup")
        .check();
    assert!(
        dup.is_err(),
        "UNIQUE(auth_request_id) must reject a second sidecar row for the same AR"
    );
}

// ---- 6. agent_profile.model_config_id column ------------------------------

#[tokio::test]
async fn agent_profile_accepts_optional_model_config_id() {
    let store = fresh_store().await;
    // Pre-M5 rows (model_config_id absent) must continue to work.
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', 'ap-legacy') SET \
             agent_id = 'a-legacy', parallelize = 1, blueprint = {}, \
             created_at = time::now()",
        )
        .await
        .expect("legacy insert")
        .check()
        .expect("legacy check");

    // M5 rows with the new column populated must also work.
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', 'ap-m5') SET \
             agent_id = 'a-m5', parallelize = 1, blueprint = {}, \
             model_config_id = 'model-gpt5', \
             created_at = time::now()",
        )
        .await
        .expect("m5 insert")
        .check()
        .expect("m5 check");
}

// ---- 7. agent_catalog_entry identity UNIQUE -------------------------------

#[tokio::test]
async fn agent_catalog_entry_unique_index_rejects_duplicate_agent() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('agent_catalog_entry', 'ace-1') SET \
             agent_id = 'a-cat', owning_org = 'o', display_name = 'cat', \
             kind = 'llm', active = true, \
             last_seen_at = time::now(), updated_at = time::now()",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first check");

    let dup = store
        .client()
        .query(
            "CREATE type::thing('agent_catalog_entry', 'ace-2') SET \
             agent_id = 'a-cat', owning_org = 'o', display_name = 'cat2', \
             kind = 'llm', active = true, \
             last_seen_at = time::now(), updated_at = time::now()",
        )
        .await
        .expect("issue dup")
        .check();
    assert!(
        dup.is_err(),
        "UNIQUE(agent_id) must reject a second catalog entry for the same agent"
    );
}

// ---- 8. system_agent_runtime_status identity UNIQUE -----------------------

#[tokio::test]
async fn system_agent_runtime_status_unique_index_rejects_duplicate_agent() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('system_agent_runtime_status', 'sars-1') SET \
             agent_id = 'a-sys', owning_org = 'o', queue_depth = 0, \
             effective_parallelize = 1, updated_at = time::now()",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first check");

    let dup = store
        .client()
        .query(
            "CREATE type::thing('system_agent_runtime_status', 'sars-2') SET \
             agent_id = 'a-sys', owning_org = 'o', queue_depth = 7, \
             effective_parallelize = 1, updated_at = time::now()",
        )
        .await
        .expect("issue dup")
        .check();
    assert!(
        dup.is_err(),
        "UNIQUE(agent_id) must reject a second runtime-status row for the same agent"
    );
}
