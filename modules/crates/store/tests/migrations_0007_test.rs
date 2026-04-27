//! Targeted integration test for migration `0007_agent_active_archived.surql`.
//!
//! Verifies the CH-01 schema additions are queryable against a live
//! SurrealDB instance — not just that the migration applied at the
//! ledger level. Covers ADR-0034 D34.1 (`active: bool DEFAULT true`)
//! and D34.2 (`archived_at: option<string>`).

use chrono::{TimeZone, Utc};
use domain::model::nodes::{Agent, AgentKind};
use domain::Repository;
use store::SurrealStore;
use tempfile::tempdir;

async fn fresh_store() -> SurrealStore {
    let dir = tempdir().expect("tempdir");
    SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store")
}

// ---- agent.active column --------------------------------------------------

/// ADR-0034 D34.1 — `active` column has `DEFAULT true`. A row that omits the
/// field on INSERT must materialise as active. Pre-CH-01 rows (which
/// pre-date the migration) hit this path on first read after upgrade.
#[tokio::test]
async fn agent_active_defaults_true_for_inserted_rows() {
    let store = fresh_store().await;

    // Issue a CREATE that does NOT set `active` — schema DEFAULT must fill in.
    store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-default') SET \
             kind = 'human', display_name = 'no-active-set', \
             created_at = time::now()",
        )
        .await
        .expect("insert without active")
        .check()
        .expect("DEFAULT true accepts the row");

    // Read back via raw SELECT to confirm the value materialised as `true`.
    let mut resp = store
        .client()
        .query("SELECT active FROM type::thing('agent', 'a-default')")
        .await
        .expect("select active");
    let actives: Vec<bool> = resp.take((0, "active")).expect("take active");
    assert_eq!(
        actives,
        vec![true],
        "DEFAULT true must materialise on insert"
    );
}

/// `active` column accepts an explicit `false` (the disable path's write).
#[tokio::test]
async fn agent_active_accepts_explicit_false() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-disabled') SET \
             kind = 'llm', display_name = 'sys-disabled', \
             role = 'system', active = false, \
             created_at = time::now()",
        )
        .await
        .expect("insert with active=false")
        .check()
        .expect("active accepts explicit false");

    let mut resp = store
        .client()
        .query("SELECT active FROM type::thing('agent', 'a-disabled')")
        .await
        .expect("select active");
    let actives: Vec<bool> = resp.take((0, "active")).expect("take active");
    assert_eq!(actives, vec![false], "explicit false must persist");
}

// ---- agent.archived_at column ---------------------------------------------

/// ADR-0034 D34.2 — `archived_at: option<string>`. Accepts both `NONE` (the
/// not-archived state) and a serialised RFC3339 datetime string (the
/// archive handler's write). Mirrors the project convention used by
/// `grant.revoked_at` and `consent.revoked_at` from migration 0001.
#[tokio::test]
async fn agent_archived_at_accepts_none_and_rfc3339_string() {
    let store = fresh_store().await;

    // (a) Default / not-archived path — omit the field; option<string>
    //     accepts NONE.
    store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-not-archived') SET \
             kind = 'human', display_name = 'still-here', \
             created_at = time::now()",
        )
        .await
        .expect("insert without archived_at")
        .check()
        .expect("option<string> accepts absence");

    // (b) Archived path — supply an RFC3339 string. chrono's serde feature
    //     produces this format for DateTime<Utc> by default; the archive
    //     handler will pass `Utc::now().to_rfc3339()` into the SET binding.
    let archived_at = "2026-04-27T10:15:30Z";
    store
        .client()
        .query(
            "CREATE type::thing('agent', 'a-archived') SET \
             kind = 'llm', display_name = 'sys-archived', \
             role = 'system', archived_at = $ts, \
             created_at = time::now()",
        )
        .bind(("ts", archived_at))
        .await
        .expect("insert with archived_at")
        .check()
        .expect("option<string> accepts RFC3339 string");

    let mut resp = store
        .client()
        .query("SELECT archived_at FROM type::thing('agent', 'a-archived')")
        .await
        .expect("select archived_at");
    let stamps: Vec<Option<String>> = resp.take((0, "archived_at")).expect("take archived_at");
    assert_eq!(stamps, vec![Some(archived_at.to_string())]);
}

// ---- back-compat: pre-CH-01 rows round-trip via Agent struct ---------------

/// ADR-0034 D34.1+D34.2 — pre-CH-01 rows that pre-date this migration land
/// in the database without the `active` / `archived_at` columns. After
/// migration 0007 applies, the schema's DEFAULT (for `active`) + the
/// optional column (for `archived_at`) make those columns queryable, but
/// older clients that wrote rows BEFORE the column existed are
/// represented in storage with the columns absent. Reading such rows
/// through the `Agent` struct must succeed via `#[serde(default = ...)]`
/// (active → true) and `#[serde(default)]` (archived_at → None).
///
/// We simulate the pre-CH-01 scenario by inserting via raw SurrealQL
/// without setting either field, then fetching via the typed
/// `Repository::get_agent` path (which deserialises through serde).
#[tokio::test]
async fn pre_ch01_rows_deserialise_with_active_true_and_archived_at_none() {
    let store = fresh_store().await;

    // Insert via raw SurrealQL — emulates a row written by an older code
    // version that didn't know about the new columns. We allocate a fresh
    // AgentId, render it as the SurrealDB record-id string, and read it
    // back through the typed `Repository` path.
    use domain::model::ids::AgentId;
    let agent_id = AgentId::new();
    store
        .client()
        .query(
            "CREATE type::thing('agent', $id) SET \
             kind = 'human', display_name = 'legacy-row', \
             created_at = time::now()",
        )
        .bind(("id", agent_id.to_string()))
        .await
        .expect("insert legacy row")
        .check()
        .expect("schema accepts pre-CH-01 row shape");

    // Round-trip through the typed Repository. The Agent struct's
    // `#[serde(default = "default_agent_active")]` and
    // `#[serde(default)]` annotations must fill in the missing columns.
    let agent: Agent = store
        .get_agent(agent_id)
        .await
        .expect("get_agent")
        .expect("agent row exists");

    assert_eq!(agent.kind, AgentKind::Human);
    assert_eq!(agent.display_name, "legacy-row");
    // The two key assertions for back-compat:
    assert!(
        agent.active,
        "pre-CH-01 rows must deserialise as active = true via serde default"
    );
    assert!(
        agent.archived_at.is_none(),
        "pre-CH-01 rows must deserialise as archived_at = None via serde default"
    );
}

// ---- domain::Agent serde round-trip with explicit values ------------------

/// Round-trip a domain::Agent (with active=false + archived_at=Some) through
/// the repository — exercises the path the CH-01 disable/archive handlers
/// will use at P3 (write durable state then read back via get_agent).
#[tokio::test]
async fn agent_round_trips_explicit_active_false_and_archived_at_some() {
    let store = fresh_store().await;

    use domain::model::ids::AgentId;
    use domain::model::nodes::AgentRole;

    let id = AgentId::new();
    let archived_at = Utc.with_ymd_and_hms(2026, 4, 27, 10, 15, 30).unwrap();
    let agent = Agent {
        id,
        kind: AgentKind::Llm,
        display_name: "explicit-archived".into(),
        owning_org: None,
        role: Some(AgentRole::System),
        created_at: Utc::now(),
        active: false,
        archived_at: Some(archived_at),
    };

    store.create_agent(&agent).await.expect("create_agent");
    let fetched: Agent = store
        .get_agent(id)
        .await
        .expect("get_agent")
        .expect("agent row exists");

    assert!(!fetched.active, "explicit false must round-trip");
    assert_eq!(fetched.archived_at, Some(archived_at));
}
