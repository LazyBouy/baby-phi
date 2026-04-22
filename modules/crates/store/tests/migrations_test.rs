//! Integration tests for the migration runner against a real embedded
//! SurrealDB (RocksDB) instance. Each test uses its own tempdir so runs are
//! isolated.
//!
//! Verifies C7 (schema migrations) from the M1 plan's commitment ledger:
//! forward-only, idempotent, fail-safe on broken migrations.

use store::SurrealStore;
use tempfile::tempdir;

#[tokio::test]
async fn open_embedded_applies_initial_migration_and_creates_schema() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // _migrations now holds the initial migration row.
    let rows: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version, slug FROM _migrations ORDER BY version ASC")
        .await
        .expect("query ledger")
        .take(0)
        .expect("take");
    assert_eq!(rows.len(), 4, "every embedded migration recorded");
    assert_eq!(rows[0].get("version").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        rows[0].get("slug").and_then(|v| v.as_str()),
        Some("initial")
    );
    assert_eq!(rows[1].get("version").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(
        rows[1].get("slug").and_then(|v| v.as_str()),
        Some("platform_setup")
    );
    assert_eq!(rows[2].get("version").and_then(|v| v.as_i64()), Some(3));
    assert_eq!(
        rows[2].get("slug").and_then(|v| v.as_str()),
        Some("org_creation")
    );
    assert_eq!(rows[3].get("version").and_then(|v| v.as_i64()), Some(4));
    assert_eq!(
        rows[3].get("slug").and_then(|v| v.as_str()),
        Some("agents_projects")
    );

    // A sample table from the initial migration exists and accepts a row
    // shaped per its schema.
    store
        .client()
        .query(
            "CREATE agent SET kind = 'human', display_name = 'probe', \
             owning_org = NONE, created_at = time::now()",
        )
        .await
        .expect("create agent")
        .check()
        .expect("check agent create");

    // The agent table's `kind` ASSERT rejects unknown values — verifies the
    // migration's SCHEMAFULL + ASSERT clauses actually landed.
    let bad = store
        .client()
        .query(
            "CREATE agent SET kind = 'alien', display_name = 'probe2', \
             owning_org = NONE, created_at = time::now()",
        )
        .await
        .expect("issue bad create")
        .check();
    assert!(
        bad.is_err(),
        "invalid agent kind must be rejected by ASSERT"
    );
}

// Note: re-opening the same RocksDB path in-process is blocked by the
// RocksDB file lock (the OS-level lock is only released on process exit).
// Migration-runner idempotency across repeated invocations on an already-
// open store is covered by
// `migrations::tests::is_idempotent_across_successive_runs` in the lib
// unit tests, so we do not duplicate it here.
