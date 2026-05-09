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
    assert_eq!(rows.len(), 16, "every embedded migration recorded");
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
    assert_eq!(rows[4].get("version").and_then(|v| v.as_i64()), Some(5));
    assert_eq!(
        rows[4].get("slug").and_then(|v| v.as_str()),
        Some("sessions_templates_system_agents")
    );
    assert_eq!(rows[5].get("version").and_then(|v| v.as_i64()), Some(6));
    assert_eq!(
        rows[5].get("slug").and_then(|v| v.as_str()),
        Some("agent_profile_mock_response")
    );
    assert_eq!(rows[6].get("version").and_then(|v| v.as_i64()), Some(7));
    assert_eq!(
        rows[6].get("slug").and_then(|v| v.as_str()),
        Some("agent_active_archived")
    );
    // CH-06 — instance-identity tags rollout (D-new-11 closure).
    assert_eq!(rows[7].get("version").and_then(|v| v.as_i64()), Some(8));
    assert_eq!(
        rows[7].get("slug").and_then(|v| v.as_str()),
        Some("instance_identity_tags")
    );
    // CH-16 — Identity node materialization (D-new-01 + D-new-23 closure).
    assert_eq!(rows[8].get("version").and_then(|v| v.as_i64()), Some(9));
    assert_eq!(
        rows[8].get("slug").and_then(|v| v.as_str()),
        Some("identity_node")
    );
    // CH-09 — Consent node full shape (D-new-04 closure).
    assert_eq!(rows[9].get("version").and_then(|v| v.as_i64()), Some(10));
    assert_eq!(
        rows[9].get("slug").and_then(|v| v.as_str()),
        Some("consent_full_shape")
    );
    // CH-23 — Template C/D production triggers (ADR-0046).
    assert_eq!(rows[10].get("version").and_then(|v| v.as_i64()), Some(11));
    assert_eq!(
        rows[10].get("slug").and_then(|v| v.as_str()),
        Some("manages_supervisor_edges")
    );
    // CH-10 — Consent state-machine deadline column (ADR-0047).
    assert_eq!(rows[11].get("version").and_then(|v| v.as_i64()), Some(12));
    assert_eq!(
        rows[11].get("slug").and_then(|v| v.as_str()),
        Some("consent_deadline")
    );
    // CH-11 — Per-session consent gating (ADR-0048 §D48.1 + §D48.4).
    assert_eq!(rows[12].get("version").and_then(|v| v.as_i64()), Some(13));
    assert_eq!(
        rows[12].get("slug").and_then(|v| v.as_str()),
        Some("per_session_consent_gating")
    );
    // CH-14 — Authority chain walker + recursive revocation cascade
    // (ADR-0053 §D53.5).
    assert_eq!(rows[13].get("version").and_then(|v| v.as_i64()), Some(14));
    assert_eq!(
        rows[13].get("slug").and_then(|v| v.as_str()),
        Some("authority_chain")
    );
    // CH-15 — Template A session-object grant backfill (ADR-0054
    // §D54.4).
    assert_eq!(rows[14].get("version").and_then(|v| v.as_i64()), Some(15));
    assert_eq!(
        rows[14].get("slug").and_then(|v| v.as_str()),
        Some("template_a_session_object_grant")
    );
    // CH-17 — Append `Action::Observe` to every legacy Template A
    // grant (ADR-0055 §D55.9).
    assert_eq!(rows[15].get("version").and_then(|v| v.as_i64()), Some(16));
    assert_eq!(
        rows[15].get("slug").and_then(|v| v.as_str()),
        Some("template_a_session_object_grant_add_observe")
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
