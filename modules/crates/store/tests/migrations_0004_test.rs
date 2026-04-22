//! Targeted integration test for migration `0004_agents_projects.surql`.
//!
//! Verifies the M4/P1 schema additions are queryable against a live
//! SurrealDB instance — not just that the migration "applied
//! successfully" at the ledger level.
//!
//! Covers M4 plan commitment C4 (forward-only migration 0004 apply).

use store::SurrealStore;
use tempfile::tempdir;

async fn fresh_store() -> SurrealStore {
    let dir = tempdir().expect("tempdir");
    SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store")
}

// ---- agent.role column -----------------------------------------------------

#[tokio::test]
async fn agent_role_column_accepts_every_six_variants_and_none() {
    let store = fresh_store().await;

    // The serde `rename_all = "snake_case"` renders each variant directly —
    // identical to `AgentRole::as_str()`.
    for role in [
        "executive",
        "admin",
        "member",
        "intern",
        "contract",
        "system",
    ] {
        store
            .client()
            .query(
                "CREATE type::thing('agent', $id) SET \
                 kind = 'human', display_name = $name, role = $role, \
                 created_at = time::now()",
            )
            .bind(("id", format!("a-{role}")))
            .bind(("name", format!("probe-{role}")))
            .bind(("role", role))
            .await
            .unwrap_or_else(|e| panic!("role={role} insert: {e}"))
            .check()
            .unwrap_or_else(|e| panic!("role={role} check: {e}"));
    }

    // Pre-M4 rows land with role = NONE — must be accepted.
    store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-legacy') SET \
             kind = 'human', display_name = 'legacy', \
             created_at = time::now()",
        )
        .await
        .expect("insert legacy row")
        .check()
        .expect("ASSERT on option<string> accepts NONE");
}

#[tokio::test]
async fn agent_role_assert_rejects_unknown_value() {
    let store = fresh_store().await;
    let bad = store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-bogus') SET \
             kind = 'human', display_name = 'bogus', role = 'chairman', \
             created_at = time::now()",
        )
        .await
        .expect("issue bad create")
        .check();
    assert!(
        bad.is_err(),
        "agent.role ASSERT must reject values outside the 6-variant set"
    );
}

// ---- project full-shape schema --------------------------------------------

#[tokio::test]
async fn project_table_accepts_shape_a_row_with_embedded_okrs() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('project', 'p-alpha') SET \
             name = 'Alpha', description = 'inaugural project', \
             status = 'planned', shape = 'shape_a', \
             token_budget = 10000, tokens_spent = 0, \
             objectives = [ { objective_id: 'o1', name: 'O1' } ], \
             key_results = [], \
             created_at = time::now()",
        )
        .await
        .expect("shape_a insert")
        .check()
        .expect("shape_a check");
}

#[tokio::test]
async fn project_shape_assert_rejects_unknown_shape() {
    let store = fresh_store().await;
    let bad = store
        .client()
        .query(
            "CREATE type::thing('project', 'p-bogus') SET \
             name = 'bogus', shape = 'shape_c', status = 'planned', \
             created_at = time::now()",
        )
        .await
        .expect("issue bad shape")
        .check();
    assert!(
        bad.is_err(),
        "project.shape ASSERT must reject values outside {{shape_a, shape_b}}"
    );
}

#[tokio::test]
async fn project_status_assert_rejects_unknown_status() {
    let store = fresh_store().await;
    let bad = store
        .client()
        .query(
            "CREATE type::thing('project', 'p-weird') SET \
             name = 'weird', status = 'zombie', shape = 'shape_a', \
             created_at = time::now()",
        )
        .await
        .expect("issue bad status")
        .check();
    assert!(
        bad.is_err(),
        "project.status ASSERT must reject values outside the 4-variant set"
    );
}

// ---- agent_execution_limits table -----------------------------------------

#[tokio::test]
async fn agent_execution_limits_accepts_a_wrapped_phi_core_limits_object() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('agent_execution_limits', 'a1-override') SET \
             owning_agent = 'a-one', \
             limits = { max_turns: 20, max_total_tokens: 500000, \
                        max_duration: { secs: 300, nanos: 0 }, \
                        max_cost: 5.0 }, \
             created_at = time::now()",
        )
        .await
        .expect("insert override")
        .check()
        .expect("check override");
}

#[tokio::test]
async fn agent_execution_limits_unique_index_rejects_duplicate_owner() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('agent_execution_limits', 'a1-first') SET \
             owning_agent = 'a-dup', \
             limits = {}, \
             created_at = time::now()",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first check");

    let dup = store
        .client()
        .query(
            "CREATE type::thing('agent_execution_limits', 'a1-second') SET \
             owning_agent = 'a-dup', \
             limits = {}, \
             created_at = time::now()",
        )
        .await
        .expect("second insert issued")
        .check();
    assert!(
        dup.is_err(),
        "UNIQUE index on owning_agent must reject a second row for the same agent"
    );
}

// ---- new RELATION tables --------------------------------------------------

#[tokio::test]
async fn has_lead_has_subproject_has_config_tables_exist() {
    // These TYPE RELATION tables were the missing pieces called out by
    // the P0 ontology audit. Smoke-test by listing table metadata.
    let store = fresh_store().await;
    let mut resp = store
        .client()
        .query("INFO FOR DB")
        .await
        .expect("info query");
    let info: Option<serde_json::Value> = resp.take(0).expect("take info");
    let info = info.expect("INFO FOR DB returned null");
    let tables = info
        .get("tables")
        .or_else(|| info.get("tb"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
    let text = serde_json::to_string(&tables).unwrap();
    for expected in ["has_lead", "has_subproject", "has_config"] {
        assert!(
            text.contains(expected),
            "migration 0004 must define RELATION table `{}` (INFO output: {})",
            expected,
            text,
        );
    }
}
