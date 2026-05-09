//! Integration tests for migration `0016_template_a_session_object_grant_add_observe.surql`
//! (CH-17 / ADR-0055 §D55.9).
//!
//! Mirrors the deterministic-seed shape of `migration_0015_test.rs` /
//! `acceptance_template_a_session_grant_backfill.rs`: seed a legacy
//! Template A grant, run the migration, assert the action array now
//! contains `"observe"` at the tail. Idempotency is exercised by
//! deleting the ledger row for version 16 and re-running — the
//! migration body's `array::find_index(action, "observe") = NONE` guard
//! is the belt-and-braces against ledger drift.
//!
//! Coverage:
//!   1. `migration_0016_appends_observe_to_legacy_template_a_grant_action`
//!      — happy path: seed `[read, inspect, list]`; run 0016; assert
//!      `[read, inspect, list, observe]`.
//!   2. `migration_0016_is_idempotent_across_repeat_application`
//!      — delete ledger row 16; re-run; assert action array UNCHANGED
//!      (no duplicate `observe`).

use chrono::Utc;
use store::{run_migrations, SurrealStore, EMBEDDED_MIGRATIONS};
use tempfile::tempdir;

/// Seed a Template A adoption AR + a single-grant legacy lead with the
/// pre-CH-17 action array `["read", "inspect", "list"]`. Returns
/// `(ar_id_str, grant_id_str)`.
async fn seed_legacy_template_a_grant(store: &SurrealStore) -> (String, String) {
    let now = Utc::now().to_rfc3339();
    let ar_id = uuid::Uuid::new_v4().to_string();
    let grant_id = uuid::Uuid::new_v4().to_string();
    let project_id = uuid::Uuid::new_v4().to_string();
    let lead_id = uuid::Uuid::new_v4().to_string();

    // Seed the AR.
    store
        .client()
        .query(format!(
            "CREATE type::thing(\"auth_request\", \"{ar_id}\") SET \
             requestor_kind = \"agent\", \
             requestor_id = \"{lead_id}\", \
             kinds = [\"#template:a\"], \
             scope = [\"read\"], \
             state = \"approved\", \
             submitted_at = \"{now}\", \
             resource_slots = [], \
             audit_class = \"silent\", \
             archived = false, \
             active_window_days = 90, \
             tags = [], \
             terminal_state_entered_at = \"{now}\""
        ))
        .await
        .expect("seed AR")
        .check()
        .expect("AR check");

    // Seed the legacy single-grant with [read, inspect, list].
    store
        .client()
        .query(format!(
            "CREATE type::thing(\"grant\", \"{grant_id}\") SET \
             holder_kind = \"agent\", \
             holder_id = \"{lead_id}\", \
             action = [\"read\", \"inspect\", \"list\"], \
             resource_uri = \"project:{project_id}\", \
             fundamentals = [\"tag\"], \
             descends_from = \"{ar_id}\", \
             delegable = false, \
             issued_at = \"{now}\", \
             revoked_at = NONE, \
             approval_mode = {{ kind: \"implicit\" }}, \
             audit_class = \"silent\""
        ))
        .await
        .expect("seed grant")
        .check()
        .expect("grant check");
    (ar_id, grant_id)
}

/// Read the action array off the seeded grant.
async fn read_action_array(store: &SurrealStore, grant_id: &str) -> Vec<String> {
    let mut resp = store
        .client()
        .query(format!(
            "SELECT action FROM type::thing(\"grant\", \"{grant_id}\")"
        ))
        .await
        .expect("query action");
    let rows: Vec<serde_json::Value> = resp.take(0).expect("take");
    rows.first()
        .and_then(|v| v.get("action"))
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn migration_0016_appends_observe_to_legacy_template_a_grant_action() {
    let dir = tempdir().expect("tempdir");
    // The harness opens the store + runs every embedded migration
    // including 0016. We then seed AFTER spawn finishes and re-run
    // 0016 by deleting its ledger row + re-invoking run_migrations.
    let store = SurrealStore::open_embedded(dir.path().join("db"), "ch17", "test")
        .await
        .expect("open store");

    let (_ar_id, grant_id) = seed_legacy_template_a_grant(&store).await;

    // Pre-condition: the seeded grant has the legacy 3-action array.
    let pre = read_action_array(&store, &grant_id).await;
    assert_eq!(
        pre,
        vec![
            "read".to_string(),
            "inspect".to_string(),
            "list".to_string()
        ],
        "seed shape pre-migration"
    );

    // Force migration 0016 to re-run by clearing its ledger row.
    store
        .client()
        .query("DELETE FROM _migrations WHERE version = 16")
        .await
        .expect("delete ledger 16")
        .check()
        .expect("delete check");
    let applied = run_migrations(store.client(), EMBEDDED_MIGRATIONS)
        .await
        .expect("re-run migrations");
    assert!(applied.contains(&16), "version 16 must re-apply");

    // Post-condition: action array now carries `observe` at the tail.
    let post = read_action_array(&store, &grant_id).await;
    assert_eq!(
        post,
        vec![
            "read".to_string(),
            "inspect".to_string(),
            "list".to_string(),
            "observe".to_string(),
        ],
        "migration 0016 appends observe to the legacy action array"
    );
}

#[tokio::test]
async fn migration_0016_is_idempotent_across_repeat_application() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "ch17-idem", "test")
        .await
        .expect("open store");

    let (_ar_id, grant_id) = seed_legacy_template_a_grant(&store).await;

    // First re-run: appends observe.
    store
        .client()
        .query("DELETE FROM _migrations WHERE version = 16")
        .await
        .expect("delete ledger 16 (1st)")
        .check()
        .expect("delete check");
    let _ = run_migrations(store.client(), EMBEDDED_MIGRATIONS)
        .await
        .expect("first re-run");
    let after_first = read_action_array(&store, &grant_id).await;
    assert_eq!(after_first.len(), 4, "first re-run lands observe");
    assert_eq!(
        after_first.last().map(String::as_str),
        Some("observe"),
        "observe at the tail"
    );

    // Second re-run: action array UNCHANGED (NOT-EXISTS guard fires).
    store
        .client()
        .query("DELETE FROM _migrations WHERE version = 16")
        .await
        .expect("delete ledger 16 (2nd)")
        .check()
        .expect("delete check");
    let _ = run_migrations(store.client(), EMBEDDED_MIGRATIONS)
        .await
        .expect("second re-run");
    let after_second = read_action_array(&store, &grant_id).await;
    assert_eq!(
        after_second, after_first,
        "second re-run is a no-op — guard prevents duplicate observe"
    );
}
