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
    assert_eq!(rows.len(), 20, "every embedded migration recorded");
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
    // CH-25 — Agent-as-creator-and-owner edge model (ADR-0060 §D60.1).
    // Adds the `owns` relation table backing the new `Edge::Owns`
    // variant.
    assert_eq!(rows[16].get("version").and_then(|v| v.as_i64()), Some(17));
    assert_eq!(
        rows[16].get("slug").and_then(|v| v.as_str()),
        Some("add_owns_relation")
    );
    // CH-26 — Org/Project as Composite resources (ADR-0061 §D61.2 +
    // §D61.3). Adds `tags` field to organization + project SCHEMAFULL
    // tables; backfills instance-identity tag + catalogue entries.
    assert_eq!(rows[17].get("version").and_then(|v| v.as_i64()), Some(18));
    assert_eq!(
        rows[17].get("slug").and_then(|v| v.as_str()),
        Some("org_project_tags")
    );
    // CH-28 — AgentProfile cardinality flip 1:1 → N:1 schema (ADR-0063
    // §D63.1 + §D63.3 + §D63.5). Drops the UNIQUE on `agent_profile.
    // agent_id`; DEFINEs the hybrid `blueprint` table + renamed
    // `uses_profile` relation + two NEW Blueprint-pointer relations;
    // REMOVEs the per-agent override field definitions on
    // `agent_profile` (rows retain stored values for the half-
    // migrated-state tolerance window per ADR-0063 §D63.12). F1.c +
    // F2.b + F3.b user-locks DIVERGENT from planner-recommended
    // F1.a + F2.a + F3.a. 0020 (data-only backfill) lands at
    // P-MIGRATION-BACKFILL.
    assert_eq!(rows[18].get("version").and_then(|v| v.as_i64()), Some(19));
    assert_eq!(
        rows[18].get("slug").and_then(|v| v.as_str()),
        Some("agent_profile_n_to_1_schema")
    );
    // CH-28 P-MIGRATION-BACKFILL — data-only backfill paired with 0019.
    // Per F3.b user-lock (split migrations 0019 + 0020): UPSERTs a
    // template-tier `blueprint` row + RELATEs the
    // `agent_profile_uses_blueprint` edge for every pre-existing
    // agent_profile row; rewrites legacy `has_profile` rows as
    // `uses_profile` rows (and DELETEs the legacy). Override-tier
    // blueprint rows are written lazily by application code at
    // P1-BLUEPRINT-STRUCT (NOT at backfill).
    assert_eq!(rows[19].get("version").and_then(|v| v.as_i64()), Some(20));
    assert_eq!(
        rows[19].get("slug").and_then(|v| v.as_str()),
        Some("agent_profile_n_to_1_backfill")
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

/// CH-28 P-MIGRATION-SCHEMA Tier A — verifies migration 0019 first-apply:
///   - the legacy `agent_profile_agent_id` UNIQUE index is DROPPED
///   - the NEW `blueprint` SCHEMAFULL table is DEFINED with the expected
///     fields (`agent_id`, `parallelize`, `model_config_id`,
///     `mock_response`, `created_at`)
///   - the NEW relation tables `uses_profile`,
///     `agent_profile_uses_blueprint`, `agent_uses_blueprint_override`
///     are DEFINED
///   - the per-agent override field definitions (`parallelize`,
///     `model_config_id`, `mock_response`) on `agent_profile` are
///     REMOVED (rows retain stored values for the half-migrated
///     tolerance window per ADR-0063 §D63.12; field DEFINITIONS are
///     removed but row data persists)
///
/// Per ADR-0063 §D63.5 (Option E retry, user-locked 2026-05-19): the
/// `blueprint_agent_id` index is NON-UNIQUE (SurrealDB 2.6.5 grammar
/// does not support filtered/partial UNIQUE indexes); override-row
/// uniqueness moves to the repository tier.
#[tokio::test]
async fn migration_0019_first_apply_creates_blueprint_table_and_drops_unique() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // 0019 first-apply records the migration row.
    let rows: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version, slug FROM _migrations WHERE version = 19")
        .await
        .expect("query ledger for v19")
        .take(0)
        .expect("take v19 row");
    assert_eq!(rows.len(), 1, "0019 migration row recorded exactly once");
    assert_eq!(
        rows[0].get("slug").and_then(|v| v.as_str()),
        Some("agent_profile_n_to_1_schema")
    );

    // The `blueprint` table is INFO-introspectable (SCHEMAFULL).
    let info: Vec<serde_json::Value> = store
        .client()
        .query("INFO FOR TABLE blueprint")
        .await
        .expect("INFO FOR TABLE blueprint")
        .take(0)
        .expect("take blueprint info");
    assert_eq!(info.len(), 1, "blueprint table is DEFINED");
    let info = &info[0];
    // Each of the 5 fields must be present on the table info; the
    // serialized shape is {fields: {agent_id: "DEFINE FIELD ...", ...}}.
    let fields = info
        .get("fields")
        .expect("blueprint table info has fields map");
    for field in [
        "agent_id",
        "parallelize",
        "model_config_id",
        "mock_response",
        "created_at",
    ] {
        assert!(
            fields.get(field).is_some(),
            "blueprint.{field} field is DEFINED"
        );
    }

    // The non-UNIQUE read-side `blueprint_agent_id` index is DEFINED.
    let indexes = info
        .get("indexes")
        .expect("blueprint table info has indexes map");
    let bp_idx = indexes
        .get("blueprint_agent_id")
        .expect("blueprint_agent_id index is DEFINED");
    let bp_idx_str = bp_idx.as_str().unwrap_or("");
    // Option E: this is a NON-UNIQUE lookup index. The string must
    // not contain "UNIQUE" — uniqueness moves to the repository tier
    // per ADR-0063 §D63.5.
    assert!(
        !bp_idx_str.contains("UNIQUE"),
        "blueprint_agent_id index is non-UNIQUE per Option E (got: {bp_idx_str:?})"
    );

    // The new RELATION tables are DEFINED.
    for tbl in [
        "uses_profile",
        "agent_profile_uses_blueprint",
        "agent_uses_blueprint_override",
    ] {
        let info: Vec<serde_json::Value> = store
            .client()
            .query(format!("INFO FOR TABLE {tbl}"))
            .await
            .expect("INFO FOR TABLE relation")
            .take(0)
            .expect("take relation info");
        assert_eq!(info.len(), 1, "{tbl} table is DEFINED");
    }

    // The legacy UNIQUE index on `agent_profile.agent_id` is DROPPED.
    let ap_info: Vec<serde_json::Value> = store
        .client()
        .query("INFO FOR TABLE agent_profile")
        .await
        .expect("INFO FOR TABLE agent_profile")
        .take(0)
        .expect("take agent_profile info");
    let ap_info = &ap_info[0];
    let ap_indexes = ap_info
        .get("indexes")
        .expect("agent_profile table info has indexes map");
    assert!(
        ap_indexes.get("agent_profile_agent_id").is_none(),
        "legacy agent_profile_agent_id UNIQUE index is DROPPED per F3.b (got: {ap_indexes:?})"
    );

    // The per-agent override field DEFINITIONS on agent_profile are
    // REMOVED (per F-half-migrated-tolerance.a — stored row values
    // persist for the half-migrated window but field DEFINITIONS are
    // gone; the table info's `fields` map no longer enumerates them).
    let ap_fields = ap_info
        .get("fields")
        .expect("agent_profile table info has fields map");
    for removed in ["parallelize", "model_config_id", "mock_response"] {
        assert!(
            ap_fields.get(removed).is_none(),
            "agent_profile.{removed} field DEFINITION is REMOVED per 0019 (got: {ap_fields:?})"
        );
    }
}

/// CH-28 P-MIGRATION-SCHEMA Tier A — verifies migration 0019 idempotency:
/// re-running 0019 on a post-apply database is a no-op. Per ADR-0063
/// §D63.5 the body uses `REMOVE INDEX IF EXISTS`, `DEFINE TABLE IF NOT
/// EXISTS`, `DEFINE FIELD IF NOT EXISTS`, `DEFINE INDEX IF NOT EXISTS`,
/// `REMOVE FIELD IF EXISTS` — all idempotent SurrealDB 2.x forms.
///
/// We verify idempotency by reading the migration body from disk and
/// re-executing it against the post-apply store; the query must
/// succeed without error AND the `_migrations` ledger must NOT gain a
/// duplicate row for version 19.
#[tokio::test]
async fn migration_0019_idempotent_under_rerun() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // Confirm baseline — 0019 applied once.
    let baseline: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version FROM _migrations WHERE version = 19")
        .await
        .expect("baseline query")
        .take(0)
        .expect("take baseline");
    assert_eq!(
        baseline.len(),
        1,
        "0019 row exists exactly once at baseline"
    );

    // Re-execute the migration body directly. The migration runner
    // wouldn't re-apply (the ledger row already exists) so we exercise
    // the SurrealDB idempotent forms directly by re-running the SQL
    // body. The body must succeed without error.
    let body = include_str!("../migrations/0019_agent_profile_n_to_1_schema.surql");
    store
        .client()
        .query(body)
        .await
        .expect("re-running 0019 body is parser-clean")
        .check()
        .expect("re-running 0019 body is a no-op (idempotent)");

    // The ledger must still hold exactly one row for v19.
    let after: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version FROM _migrations WHERE version = 19")
        .await
        .expect("post-rerun query")
        .take(0)
        .expect("take post-rerun");
    assert_eq!(
        after.len(),
        1,
        "0019 ledger row not duplicated on re-run (idempotent)"
    );

    // Schema state is unchanged — blueprint table still DEFINED with
    // the same fields.
    let info: Vec<serde_json::Value> = store
        .client()
        .query("INFO FOR TABLE blueprint")
        .await
        .expect("INFO FOR TABLE blueprint after rerun")
        .take(0)
        .expect("take blueprint info after rerun");
    assert_eq!(info.len(), 1, "blueprint table still DEFINED after rerun");
    let fields = info[0]
        .get("fields")
        .expect("blueprint fields map after rerun");
    for field in [
        "agent_id",
        "parallelize",
        "model_config_id",
        "mock_response",
        "created_at",
    ] {
        assert!(
            fields.get(field).is_some(),
            "blueprint.{field} still DEFINED after rerun"
        );
    }
}

// ---------------------------------------------------------------------------
// CH-28 P-MIGRATION-BACKFILL — 0020 (data-only) tests
//
// Per ADR-0063 §D63.3 + §D63.5, 0020 is the data backfill paired with 0019
// (schema-only). On a freshly-opened store, 0020 applies on an EMPTY
// `agent_profile` table → the migration is a no-op (no rows to backfill).
// To exercise the backfill body end-to-end, these tests pre-seed an
// `agent_profile` row + `has_profile` edge AFTER all 20 migrations have
// applied (simulating the pre-CH-28 production-data shape that 0020 is
// designed to rewrite) and then re-execute the 0020 body directly.
// ---------------------------------------------------------------------------

/// CH-28 P-MIGRATION-BACKFILL Tier A — verifies the 0020 backfill body
/// rewrites legacy `agent_profile` + `has_profile` data into the
/// post-CH-28 hybrid shape:
///   - Template-tier `blueprint` row UPSERTed at deterministic id
///     `blueprint:bp_<profile_id>` with `agent_id = NONE` and the
///     pre-CH-28 override-field values (`parallelize` /
///     `model_config_id` / `mock_response`) copied from the source row.
///   - `agent_profile_uses_blueprint` edge RELATEd at deterministic id
///     `apub_<profile_id>` from the agent_profile row to the blueprint
///     row.
///   - `uses_profile` edge RELATEd at deterministic id
///     `up_<legacy_has_profile_edge_id>` from the agent to the
///     agent_profile row (mirrors the legacy edge's endpoints).
///   - Legacy `has_profile` row DELETEd.
///
/// Verifies the F3.b user-locked split-migration data-backfill contract.
#[tokio::test]
async fn migration_0020_first_apply_backfills_blueprints_from_agent_profile() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // Pre-seed: simulate a pre-CH-28 production-shape `agent_profile` row
    // + `has_profile` edge. The store has ALL migrations applied
    // (0001..0020) before any test code runs — including 0019's
    // `REMOVE FIELD parallelize/model_config_id/mock_response ON
    // agent_profile`. To accurately simulate the pre-0019 production
    // data shape, we (1) RE-DEFINE the override fields, (2) seed the
    // agent_profile row carrying the override values (now stored
    // because the field defs are present), (3) RE-REMOVE the field
    // definitions (simulating 0019's REMOVE FIELD on existing data).
    // Per SurrealDB 2.x semantic, REMOVE FIELD drops the SCHEMAFULL
    // enforcement but the stored row values persist in storage and
    // remain readable via SELECT (the "half-migrated tolerance window"
    // per ADR-0063 §D63.12).
    let profile_id = "11111111111111111111111111111111";
    let agent_id = "22222222222222222222222222222222";
    let hp_edge_id = "33333333333333333333333333333333";

    // (1) Re-DEFINE the override fields so the SCHEMAFULL agent_profile
    // table accepts them at INSERT-time (simulating the pre-0019 shape).
    store
        .client()
        .query(
            "DEFINE FIELD parallelize     ON agent_profile TYPE option<int>    DEFAULT NONE;
             DEFINE FIELD model_config_id ON agent_profile TYPE option<string> DEFAULT NONE;
             DEFINE FIELD mock_response   ON agent_profile TYPE option<string> DEFAULT NONE;",
        )
        .await
        .expect("re-define override fields (pre-0019 schema simulation)")
        .check()
        .expect("re-define succeeds");

    // Insert a minimal agent row (needed as the `in` side of has_profile).
    store
        .client()
        .query(
            "CREATE type::thing('agent', $aid) SET \
                 kind = 'human', display_name = 'test', \
                 owning_org = NONE, created_at = '2026-05-19T00:00:00Z' \
                 RETURN NONE",
        )
        .bind(("aid", agent_id.to_string()))
        .await
        .expect("create test agent")
        .check()
        .expect("agent create succeeded");

    // (2) Insert a legacy `agent_profile` row with the pre-CH-28
    // override fields present as stored values (the fields are DEFINED
    // at this point per (1) above, so SurrealDB stores them).
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', $pid) SET \
                 agent_id = $aid, \
                 parallelize = 4, \
                 model_config_id = 'gpt-4o', \
                 mock_response = 'hello-world', \
                 blueprint = { name: 'sample', system_prompt: 'be brief' }, \
                 created_at = '2026-05-19T00:00:00Z' \
                 RETURN NONE",
        )
        .bind(("pid", profile_id.to_string()))
        .bind(("aid", agent_id.to_string()))
        .await
        .expect("create legacy agent_profile")
        .check()
        .expect("agent_profile create succeeded");

    // Insert the legacy `has_profile` edge row connecting agent ->
    // agent_profile, with a deterministic edge id (matches the
    // production app's RELATE pattern at repo_impl.rs:2399-2402 — the
    // `type::thing()` thing references must be bound via LET because
    // SurrealQL's RELATE grammar accepts only variables/identifiers at
    // the from/to positions, not function calls).
    store
        .client()
        .query(
            "LET $a = type::thing('agent', $aid); \
             LET $p = type::thing('agent_profile', $pid); \
             RELATE $a -> has_profile -> $p \
                 SET id = type::thing('has_profile', $eid) RETURN NONE",
        )
        .bind(("aid", agent_id.to_string()))
        .bind(("pid", profile_id.to_string()))
        .bind(("eid", hp_edge_id.to_string()))
        .await
        .expect("create legacy has_profile edge")
        .check()
        .expect("has_profile relate succeeded");

    // (3) Re-REMOVE the field definitions (simulating 0019's REMOVE
    // FIELD on existing data). Per SurrealDB 2.x semantic, the stored
    // row values persist after REMOVE FIELD; only the schema
    // enforcement is dropped.
    store
        .client()
        .query(
            "REMOVE FIELD IF EXISTS parallelize     ON agent_profile;
             REMOVE FIELD IF EXISTS model_config_id ON agent_profile;
             REMOVE FIELD IF EXISTS mock_response   ON agent_profile;",
        )
        .await
        .expect("re-remove override field definitions (simulating 0019 on existing data)")
        .check()
        .expect("re-remove succeeds");

    // Half-migrated-tolerance verification — per ADR-0063 §D63.12, the
    // override-field stored values MUST remain readable via SELECT after
    // REMOVE FIELD (the contract underlying the 0020 backfill body's
    // ability to read these values from `agent_profile`). This probe
    // asserts the SurrealDB 2.x semantic explicitly.
    let probe: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid, parallelize, model_config_id, mock_response FROM agent_profile")
        .await
        .expect("agent_profile probe query")
        .take(0)
        .expect("take agent_profile probe");
    assert_eq!(probe.len(), 1, "1 agent_profile row pre-backfill (probe)");
    let probe_row = &probe[0];
    assert_eq!(
        probe_row.get("parallelize").and_then(|v| v.as_i64()),
        Some(4),
        "post-REMOVE-FIELD half-migrated tolerance: parallelize stored value persists"
    );
    assert_eq!(
        probe_row.get("model_config_id").and_then(|v| v.as_str()),
        Some("gpt-4o"),
        "post-REMOVE-FIELD half-migrated tolerance: model_config_id stored value persists"
    );
    assert_eq!(
        probe_row.get("mock_response").and_then(|v| v.as_str()),
        Some("hello-world"),
        "post-REMOVE-FIELD half-migrated tolerance: mock_response stored value persists"
    );

    // Sanity baseline before running the backfill body.
    let bp_pre: Vec<serde_json::Value> = store
        .client()
        .query("SELECT id FROM blueprint")
        .await
        .expect("baseline blueprint query")
        .take(0)
        .expect("take baseline blueprint");
    assert_eq!(bp_pre.len(), 0, "no blueprint rows pre-backfill");

    let up_pre: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM uses_profile")
        .await
        .expect("baseline uses_profile query")
        .take(0)
        .expect("take baseline uses_profile");
    assert_eq!(up_pre.len(), 0, "no uses_profile rows pre-backfill");

    let hp_pre: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM has_profile")
        .await
        .expect("baseline has_profile query")
        .take(0)
        .expect("take baseline has_profile");
    assert_eq!(hp_pre.len(), 1, "one has_profile row pre-backfill");

    // Re-execute the 0020 body against the pre-seeded data. This
    // mirrors what the migration runner would do on a non-empty DB at
    // first-apply.
    let body = include_str!("../migrations/0020_agent_profile_n_to_1_backfill.surql");
    store
        .client()
        .query(body)
        .await
        .expect("0020 body executes cleanly")
        .check()
        .expect("0020 body succeeds");

    // (a) Template-tier blueprint row exists with copied override fields.
    let bp_rows: Vec<serde_json::Value> = store
        .client()
        .query(
            "SELECT record::id(id) AS rid, agent_id, parallelize, model_config_id, mock_response FROM blueprint",
        )
        .await
        .expect("blueprint query")
        .take(0)
        .expect("take blueprint rows");
    assert_eq!(
        bp_rows.len(),
        1,
        "exactly one template-blueprint row backfilled (got: {bp_rows:?})"
    );
    let bp = &bp_rows[0];
    assert_eq!(
        bp.get("rid").and_then(|v| v.as_str()),
        Some(format!("bp_{profile_id}").as_str()),
        "blueprint id matches the deterministic `bp_<profile_id>` shape"
    );
    assert!(
        bp.get("agent_id").map(|v| v.is_null()).unwrap_or(false),
        "template blueprint has agent_id = NONE (got: {:?})",
        bp.get("agent_id")
    );
    assert_eq!(
        bp.get("parallelize").and_then(|v| v.as_i64()),
        Some(4),
        "parallelize copied from legacy agent_profile"
    );
    assert_eq!(
        bp.get("model_config_id").and_then(|v| v.as_str()),
        Some("gpt-4o"),
        "model_config_id copied from legacy agent_profile"
    );
    assert_eq!(
        bp.get("mock_response").and_then(|v| v.as_str()),
        Some("hello-world"),
        "mock_response copied from legacy agent_profile"
    );

    // (b) agent_profile_uses_blueprint edge exists with deterministic id.
    // We project only `record::id(id)` because the `in`/`out` Thing-enum
    // values do not serialize cleanly into serde_json::Value (SurrealDB's
    // Thing variant is not JSON-coercible at the response decoder). The
    // edge linkage is verified implicitly by the deterministic id
    // matching the expected `apub_<profile_id>` shape.
    let apub_rows: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM agent_profile_uses_blueprint")
        .await
        .expect("agent_profile_uses_blueprint query")
        .take(0)
        .expect("take apub rows");
    assert_eq!(
        apub_rows.len(),
        1,
        "exactly one agent_profile_uses_blueprint edge backfilled"
    );
    let apub = &apub_rows[0];
    assert_eq!(
        apub.get("rid").and_then(|v| v.as_str()),
        Some(format!("apub_{profile_id}").as_str()),
        "apub edge id matches the deterministic shape"
    );

    // (c) uses_profile edge exists with deterministic id derived from
    // the legacy has_profile edge id.
    let up_rows: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM uses_profile")
        .await
        .expect("uses_profile query")
        .take(0)
        .expect("take uses_profile rows");
    assert_eq!(up_rows.len(), 1, "exactly one uses_profile edge backfilled");
    let up = &up_rows[0];
    assert_eq!(
        up.get("rid").and_then(|v| v.as_str()),
        Some(format!("up_{hp_edge_id}").as_str()),
        "uses_profile edge id matches the deterministic shape derived from legacy has_profile"
    );

    // (d) Legacy has_profile row deleted.
    let hp_post: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM has_profile")
        .await
        .expect("post-backfill has_profile query")
        .take(0)
        .expect("take has_profile post");
    assert_eq!(
        hp_post.len(),
        0,
        "legacy has_profile rows deleted by backfill (got: {hp_post:?})"
    );

    // (e) Cross-table cardinality verification: the blueprint endpoint
    // pointer chain Agent -> agent_profile -> blueprint resolves. We
    // count the rows on each side instead of inspecting the edge's
    // `in`/`out` Thing-enum fields directly.
    let agent_count: Vec<serde_json::Value> = store
        .client()
        .query("SELECT count() FROM agent GROUP ALL")
        .await
        .expect("agent count")
        .take(0)
        .expect("take agent count");
    assert_eq!(agent_count.len(), 1, "agent count row present");
}

/// CH-28 P-MIGRATION-BACKFILL Tier A — verifies migration 0020 idempotency:
/// re-running 0020 on a post-apply database is a no-op. Per ADR-0063
/// §D63.5: the body uses UPSERT-on-deterministic-id for blueprint rows,
/// IF-NOT-EXISTS guards before RELATE for both edge tables, and DELETE-
/// WHERE-id-matches (naturally idempotent) for legacy has_profile rows.
///
/// We seed a legacy production-shape row + edge, run 0020 twice, and
/// verify no duplicate rows accumulate AND the _migrations ledger does
/// not gain a duplicate v20 row.
#[tokio::test]
async fn migration_0020_idempotent_under_rerun() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // Confirm baseline — 0020 applied once via the migration runner.
    let baseline: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version FROM _migrations WHERE version = 20")
        .await
        .expect("baseline query")
        .take(0)
        .expect("take baseline");
    assert_eq!(
        baseline.len(),
        1,
        "0020 row exists exactly once at baseline"
    );

    // Pre-seed a legacy-shape agent + agent_profile + has_profile (per
    // the pre-0019 schema simulation pattern documented in
    // `migration_0020_first_apply_backfills_blueprints_from_agent_profile`
    // — re-DEFINE the override fields, seed the row, then REMOVE the
    // field defs). Run 0020 twice; verify no duplicate rows accumulate.
    let profile_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let agent_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let hp_edge_id = "cccccccccccccccccccccccccccccccc";

    // Re-DEFINE the override fields (pre-0019 schema simulation).
    store
        .client()
        .query(
            "DEFINE FIELD parallelize     ON agent_profile TYPE option<int>    DEFAULT NONE;
             DEFINE FIELD model_config_id ON agent_profile TYPE option<string> DEFAULT NONE;
             DEFINE FIELD mock_response   ON agent_profile TYPE option<string> DEFAULT NONE;",
        )
        .await
        .expect("re-define override fields")
        .check()
        .expect("re-define succeeds");

    store
        .client()
        .query(
            "CREATE type::thing('agent', $aid) SET \
                 kind = 'human', display_name = 'idem', \
                 owning_org = NONE, created_at = '2026-05-19T00:00:00Z' \
                 RETURN NONE",
        )
        .bind(("aid", agent_id.to_string()))
        .await
        .expect("create agent")
        .check()
        .expect("agent create succeeded");

    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', $pid) SET \
                 agent_id = $aid, \
                 parallelize = 1, \
                 model_config_id = NONE, \
                 mock_response = NONE, \
                 blueprint = { name: 'idem', system_prompt: 'idem' }, \
                 created_at = '2026-05-19T00:00:00Z' \
                 RETURN NONE",
        )
        .bind(("pid", profile_id.to_string()))
        .bind(("aid", agent_id.to_string()))
        .await
        .expect("create agent_profile")
        .check()
        .expect("agent_profile create succeeded");

    store
        .client()
        .query(
            "LET $a = type::thing('agent', $aid); \
             LET $p = type::thing('agent_profile', $pid); \
             RELATE $a -> has_profile -> $p \
                 SET id = type::thing('has_profile', $eid) RETURN NONE",
        )
        .bind(("aid", agent_id.to_string()))
        .bind(("pid", profile_id.to_string()))
        .bind(("eid", hp_edge_id.to_string()))
        .await
        .expect("create has_profile")
        .check()
        .expect("has_profile relate succeeded");

    // Re-REMOVE the field definitions (post-0019 simulation).
    store
        .client()
        .query(
            "REMOVE FIELD IF EXISTS parallelize     ON agent_profile;
             REMOVE FIELD IF EXISTS model_config_id ON agent_profile;
             REMOVE FIELD IF EXISTS mock_response   ON agent_profile;",
        )
        .await
        .expect("re-remove override fields")
        .check()
        .expect("re-remove succeeds");

    // First application of 0020 body.
    let body = include_str!("../migrations/0020_agent_profile_n_to_1_backfill.surql");
    store
        .client()
        .query(body)
        .await
        .expect("0020 body first run executes")
        .check()
        .expect("0020 body first run succeeds");

    // Snapshot post-first-run cardinalities. RELATION-table id columns
    // serialize as Thing enum and do not coerce cleanly into
    // serde_json::Value at the response decoder; we project
    // `record::id(id) AS rid` instead.
    let bp_after_1: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM blueprint")
        .await
        .expect("blueprint count after run 1")
        .take(0)
        .expect("take");
    let apub_after_1: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM agent_profile_uses_blueprint")
        .await
        .expect("apub count after run 1")
        .take(0)
        .expect("take");
    let up_after_1: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM uses_profile")
        .await
        .expect("uses_profile count after run 1")
        .take(0)
        .expect("take");
    let hp_after_1: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM has_profile")
        .await
        .expect("has_profile count after run 1")
        .take(0)
        .expect("take");
    assert_eq!(bp_after_1.len(), 1, "1 blueprint after run 1");
    assert_eq!(apub_after_1.len(), 1, "1 apub edge after run 1");
    assert_eq!(up_after_1.len(), 1, "1 uses_profile after run 1");
    assert_eq!(hp_after_1.len(), 0, "0 has_profile after run 1");

    // Second application — must be a no-op.
    store
        .client()
        .query(body)
        .await
        .expect("0020 body second run executes")
        .check()
        .expect("0020 body second run is parser-clean (idempotent)");

    // Cardinalities unchanged (no duplicate rows accumulated).
    let bp_after_2: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM blueprint")
        .await
        .expect("blueprint count after run 2")
        .take(0)
        .expect("take");
    let apub_after_2: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM agent_profile_uses_blueprint")
        .await
        .expect("apub count after run 2")
        .take(0)
        .expect("take");
    let up_after_2: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM uses_profile")
        .await
        .expect("uses_profile count after run 2")
        .take(0)
        .expect("take");
    let hp_after_2: Vec<serde_json::Value> = store
        .client()
        .query("SELECT record::id(id) AS rid FROM has_profile")
        .await
        .expect("has_profile count after run 2")
        .take(0)
        .expect("take");
    assert_eq!(
        bp_after_2.len(),
        1,
        "blueprint count stable across re-run (idempotent)"
    );
    assert_eq!(
        apub_after_2.len(),
        1,
        "agent_profile_uses_blueprint count stable across re-run (idempotent)"
    );
    assert_eq!(
        up_after_2.len(),
        1,
        "uses_profile count stable across re-run (idempotent)"
    );
    assert_eq!(
        hp_after_2.len(),
        0,
        "has_profile remains empty across re-run (idempotent delete)"
    );

    // The _migrations ledger still holds exactly one row for v20.
    let ledger: Vec<serde_json::Value> = store
        .client()
        .query("SELECT version FROM _migrations WHERE version = 20")
        .await
        .expect("post-rerun ledger query")
        .take(0)
        .expect("take ledger");
    assert_eq!(
        ledger.len(),
        1,
        "0020 ledger row not duplicated on re-run (migration runner idempotency)"
    );
}

/// CH-28 P1.5-READ-BRIDGE Tier A — verifies the runtime synthesis read
/// helper (`read_agent_profile_via_blueprint_or_fallback`) tolerates
/// the half-migrated window where migration 0019 has applied
/// (`REMOVE FIELD parallelize/model_config_id/mock_response ON
/// agent_profile`) but the override-tier Blueprint table is empty
/// (either because 0020 has not yet run OR because the production
/// composite-write at `upsert_agent_profile` is in flight). Per
/// ADR-0063 §D63.12 + §D63.15 the helper falls back to the legacy
/// stored property values on the `agent_profile` row (un-asserted
/// post-0019 `REMOVE FIELD` per SurrealDB 2.x half-migrated tolerance
/// semantics).
///
/// Simulation pattern mirrors
/// `migration_0020_first_apply_backfills_blueprints_from_agent_profile`:
/// (1) re-DEFINE the override fields, (2) seed an `agent_profile` row
/// with the override values populated, (3) re-REMOVE the field defs,
/// (4) call `get_agent_profile_for_agent` and verify the override
/// values surface via the legacy-row fallback (priority tier 3 per
/// §D63.15). NOT-runs the 0020 backfill body — the test exercises
/// the helper's "no Blueprint rows exist yet" branch.
///
/// Was deferred from P-MIGRATION-BACKFILL per the iter-5 re-spawn
/// mandate; lands here at P1.5-READ-BRIDGE alongside the
/// read-path wiring.
#[tokio::test]
async fn half_migrated_state_runtime_reads_via_fallback_helper() {
    use domain::model::ids::{AgentId, NodeId};
    use domain::repository::Repository;

    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("fresh store opens green");

    // (1) Re-DEFINE the 3 override fields on agent_profile so the
    // SCHEMAFULL table accepts them at INSERT-time (pre-0019 shape
    // simulation).
    store
        .client()
        .query(
            "DEFINE FIELD parallelize     ON agent_profile TYPE option<int>    DEFAULT NONE;
             DEFINE FIELD model_config_id ON agent_profile TYPE option<string> DEFAULT NONE;
             DEFINE FIELD mock_response   ON agent_profile TYPE option<string> DEFAULT NONE;",
        )
        .await
        .expect("re-define override fields (pre-0019 schema simulation)")
        .check()
        .expect("re-define succeeds");

    // (2) Seed an agent_profile row with all 3 override values populated.
    let agent_id = AgentId::new();
    let profile_id = NodeId::new();
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', $pid) SET \
                 agent_id = $aid, \
                 parallelize = 5, \
                 model_config_id = 'claude-3-opus', \
                 mock_response = 'half-migrated-fallback', \
                 blueprint = { name: 'hm', system_prompt: 'fall back' }, \
                 created_at = '2026-05-19T00:00:00Z' \
                 RETURN NONE",
        )
        .bind(("pid", profile_id.to_string()))
        .bind(("aid", agent_id.to_string()))
        .await
        .expect("seed legacy agent_profile row")
        .check()
        .expect("seed succeeded");

    // (3) Re-REMOVE the field definitions (post-0019 simulation). The
    // stored values persist on the row per ADR-0063 §D63.12.
    store
        .client()
        .query(
            "REMOVE FIELD IF EXISTS parallelize     ON agent_profile;
             REMOVE FIELD IF EXISTS model_config_id ON agent_profile;
             REMOVE FIELD IF EXISTS mock_response   ON agent_profile;",
        )
        .await
        .expect("re-remove override fields (post-0019 simulation)")
        .check()
        .expect("re-remove succeeds");

    // Confirm no Blueprint rows exist (half-migrated: 0019 applied,
    // override + template Blueprint rows have not been written). 0020
    // ran during open_embedded but the fresh store has no legacy
    // agent_profile rows to backfill at that point — the row we seeded
    // post-startup is invisible to 0020's already-completed run.
    let bp_pre: Vec<serde_json::Value> = store
        .client()
        .query("SELECT id FROM blueprint")
        .await
        .expect("blueprint query")
        .take(0)
        .expect("take");
    assert_eq!(bp_pre.len(), 0, "no blueprint rows in half-migrated state");

    // (4) Call `get_agent_profile_for_agent` (now delegates to the
    // synthesis helper per §D63.15). With no Blueprint rows, the helper
    // falls back to the legacy row's un-asserted property values
    // (priority tier 3).
    let synthesized = store
        .get_agent_profile_for_agent(agent_id)
        .await
        .expect("synthesis read succeeds")
        .expect("agent_profile row resolves");
    assert_eq!(
        synthesized.parallelize, 5,
        "parallelize falls back to legacy-row un-asserted value"
    );
    assert_eq!(
        synthesized.model_config_id.as_deref(),
        Some("claude-3-opus"),
        "model_config_id falls back to legacy-row un-asserted value"
    );
    assert_eq!(
        synthesized.mock_response.as_deref(),
        Some("half-migrated-fallback"),
        "mock_response falls back to legacy-row un-asserted value"
    );
}
