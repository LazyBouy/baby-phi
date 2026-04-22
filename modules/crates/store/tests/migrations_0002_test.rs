//! Targeted integration test for migration `0002_platform_setup.surql`.
//!
//! Verifies the new tables / columns that M2/P1 ships are actually
//! queryable against a live SurrealDB instance — not just that the
//! migration "applied successfully" at the ledger level.
//!
//! Covers M2 plan commitment C2 (forward-only migration apply).

use store::SurrealStore;
use tempfile::tempdir;

#[tokio::test]
async fn template_kind_column_accepts_every_template_kind_variant() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    // Every accepted kind value from the ASSERT INSIDE [...] clause in
    // 0002_platform_setup.surql. Serde's snake_case matches the stored
    // wire format.
    for kind in ["system_bootstrap", "a", "b", "c", "d", "e", "f"] {
        store
            .client()
            .query(
                "CREATE type::thing('template', $id) SET name = $name, kind = $kind, created_at = time::now()",
            )
            .bind(("id", format!("t-{kind}")))
            .bind(("name", format!("template:probe_{kind}")))
            .bind(("kind", kind))
            .await
            .unwrap_or_else(|e| panic!("kind={kind} insert: {e}"))
            .check()
            .unwrap_or_else(|e| panic!("kind={kind} check: {e}"));
    }
}

#[tokio::test]
async fn template_kind_assert_rejects_unknown_values() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    let bad = store
        .client()
        .query(
            "CREATE type::thing('template', 't-bogus') SET name = 'bogus', kind = 'alien', created_at = time::now()",
        )
        .await
        .expect("issue bad create")
        .check();
    assert!(
        bad.is_err(),
        "template.kind ASSERT must reject values outside the 7-kind set"
    );
}

#[tokio::test]
async fn model_runtime_table_accepts_a_well_shaped_row() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    // Minimally-shaped row — the `config` + `tenants_allowed` fields
    // are FLEXIBLE so any object shape is accepted.
    store
        .client()
        .query(
            "CREATE type::thing('model_runtime', 'mr-1') SET \
                config = { id: 'x', api: 'AnthropicMessages' }, \
                secret_ref = 'anthropic-api-key', \
                tenants_allowed = { mode: 'all' }, \
                status = 'ok', \
                archived_at = NONE, \
                created_at = time::now()",
        )
        .await
        .expect("create model_runtime")
        .check()
        .expect("check model_runtime");
}

#[tokio::test]
async fn model_runtime_status_assert_rejects_unknown() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    let bad = store
        .client()
        .query(
            "CREATE type::thing('model_runtime', 'mr-bad') SET \
                config = {}, secret_ref = '', tenants_allowed = {}, \
                status = 'catastrophic', created_at = time::now()",
        )
        .await
        .expect("issue bad create")
        .check();
    assert!(
        bad.is_err(),
        "status ASSERT must reject values outside the RuntimeStatus set"
    );
}

#[tokio::test]
async fn platform_defaults_singleton_accepts_first_insert() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    store
        .client()
        .query(
            "CREATE type::thing('platform_defaults', 'pd-1') SET \
                singleton = 1, \
                execution_limits = { max_turns: 50 }, \
                default_agent_profile = {}, \
                context_config = {}, \
                retry_config = {}, \
                default_retention_days = 30, \
                default_alert_channels = ['ops@example.com'], \
                updated_at = time::now(), \
                version = 0",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first insert check");
}

#[tokio::test]
async fn platform_defaults_singleton_unique_index_rejects_second_row() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    // First row — success.
    store
        .client()
        .query(
            "CREATE type::thing('platform_defaults', 'pd-1') SET \
                singleton = 1, \
                execution_limits = {}, default_agent_profile = {}, \
                context_config = {}, retry_config = {}, \
                default_retention_days = 30, \
                default_alert_channels = [], \
                updated_at = time::now(), version = 0",
        )
        .await
        .expect("first insert")
        .check()
        .expect("first insert check");

    // Second row with the same `singleton = 1` — must fail the UNIQUE
    // INDEX.
    let bad = store
        .client()
        .query(
            "CREATE type::thing('platform_defaults', 'pd-2') SET \
                singleton = 1, \
                execution_limits = {}, default_agent_profile = {}, \
                context_config = {}, retry_config = {}, \
                default_retention_days = 30, \
                default_alert_channels = [], \
                updated_at = time::now(), version = 0",
        )
        .await
        .expect("second insert returns")
        .check();
    assert!(
        bad.is_err(),
        "UNIQUE INDEX on platform_defaults.singleton must reject a second row"
    );
}

#[tokio::test]
async fn mcp_server_new_columns_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");

    store
        .client()
        .query(
            "CREATE type::thing('mcp_server', 'mcp-1') SET \
                display_name = 'memory-mcp', \
                kind = 'mcp', \
                endpoint = 'stdio:///usr/local/bin/memory-mcp', \
                secret_ref = NONE, \
                tenants_allowed = { mode: 'only', orgs: [] }, \
                status = 'ok', \
                archived_at = NONE, \
                created_at = time::now()",
        )
        .await
        .expect("create mcp_server")
        .check()
        .expect("check mcp_server");
}
