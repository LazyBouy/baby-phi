//! Forward-only SurrealDB migration runner with a startup fail-safe gate.
//!
//! The runner walks a sorted list of [`Migration`] records, compares their
//! `version` numbers to the rows already present in the `_migrations` meta
//! table, and applies any migration that has not yet been recorded. Every
//! applied migration becomes a permanent row in `_migrations` — migrations
//! never run twice.
//!
//! The baked migration set is [`EMBEDDED_MIGRATIONS`]. The binary does not
//! need a `migrations/` directory at runtime — each migration file is
//! `include_str!`-embedded at compile time. To add a migration, create a new
//! `NNNN_slug.surql` file under `modules/crates/store/migrations/`, then add
//! a line to [`EMBEDDED_MIGRATIONS`].
//!
//! Fail-safe: any applied migration that errors is surfaced as
//! [`MigrationError::Apply`]; callers must abort startup rather than serve
//! traffic against a half-migrated DB.

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

/// One forward-only migration.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Monotonically increasing version number. Must be unique across the
    /// entire set and must match the prefix of the source file
    /// (e.g. `0001_initial.surql` → `version = 1`).
    pub version: i64,
    /// Human-readable slug (matches the filename's non-prefix portion).
    pub slug: &'static str,
    /// The raw SurrealQL DDL. Typically populated via `include_str!`.
    pub sql: &'static str,
}

/// The canonical migration list baked into the binary. Ordered by `version`
/// ascending. Extending it is how subsequent milestones evolve the schema.
pub const EMBEDDED_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        slug: "initial",
        sql: include_str!("../migrations/0001_initial.surql"),
    },
    Migration {
        version: 2,
        slug: "platform_setup",
        sql: include_str!("../migrations/0002_platform_setup.surql"),
    },
    Migration {
        version: 3,
        slug: "org_creation",
        sql: include_str!("../migrations/0003_org_creation.surql"),
    },
    Migration {
        version: 4,
        slug: "agents_projects",
        sql: include_str!("../migrations/0004_agents_projects.surql"),
    },
    Migration {
        version: 5,
        slug: "sessions_templates_system_agents",
        sql: include_str!("../migrations/0005_sessions_templates_system_agents.surql"),
    },
    Migration {
        version: 6,
        slug: "agent_profile_mock_response",
        sql: include_str!("../migrations/0006_agent_profile_mock_response.surql"),
    },
    Migration {
        version: 7,
        slug: "agent_active_archived",
        sql: include_str!("../migrations/0007_agent_active_archived.surql"),
    },
    // CH-06 — instance identity tags rollout (D-new-11 closure).
    // Adds `tags ARRAY<string> DEFAULT []` to 10 SurrealDB tables.
    Migration {
        version: 8,
        slug: "instance_identity_tags",
        sql: include_str!("../migrations/0008_instance_identity_tags.surql"),
    },
    // CH-16 — Identity node materialization (D-new-01 + D-new-23 closure).
    // Adds the `identity` table with the four-field v0 commitment shape
    // + UNIQUE-on-`agent_id` index.
    Migration {
        version: 9,
        slug: "identity_node",
        sql: include_str!("../migrations/0009_identity_node.surql"),
    },
    // CH-09 — Consent node full shape (D-new-04 closure).
    // Lifts the consent table from the 5-field M2 stub to the 11-field
    // shape mandated by `concepts/permissions/06-multi-scope-consent.md`
    // §"Consent Node". CH-10 will add the state-transition function
    // (drift D-new-05).
    Migration {
        version: 10,
        slug: "consent_full_shape",
        sql: include_str!("../migrations/0010_consent_full_shape.surql"),
    },
    // CH-23 — Template C/D production triggers (ADR-0046).
    // Adds the `manages` + `has_agent_supervisor` tables that back
    // the inaugural production writers of the two new Edge variants.
    // UNIQUE indexes on the (scope, from, to) triples enforce
    // idempotency at the storage layer.
    Migration {
        version: 11,
        slug: "manages_supervisor_edges",
        sql: include_str!("../migrations/0011_manages_supervisor_edges.surql"),
    },
    // CH-10 — Consent state-machine deadline column (ADR-0047 §D47.6).
    // Adds `deadline_at: option<string>` to the `consent` table so the
    // sweeper can scan for past-deadline `Requested` consents and flip
    // them to `TimedOut`.
    Migration {
        version: 12,
        slug: "consent_deadline",
        sql: include_str!("../migrations/0012_consent_deadline.surql"),
    },
    // CH-11 — Per-session consent gating (ADR-0048 §D48.1 + §D48.4).
    // Adds three columns:
    //   * `grant.approval_mode` (FLEXIBLE TYPE object — tag-payload
    //      typed enum carrying `{kind, policy}` for the
    //      subordinate_required variant)
    //   * `organization.approval_timeout` (FLEXIBLE TYPE object —
    //      tag-payload typed enum carrying `{kind, duration}` for the
    //      fixed variant)
    //   * `organization.approval_timeout_default_response` (TYPE string
    //      DEFAULT "deny" — snake-case enum)
    // The `consent.scope.session_id` field rides under the existing
    // FLEXIBLE `scope` shield (no schema change required).
    Migration {
        version: 13,
        slug: "per_session_consent_gating",
        sql: include_str!("../migrations/0013_per_session_consent_gating.surql"),
    },
    // CH-14 — Authority chain walker + recursive revocation cascade
    // (ADR-0053 §D53.5). Adds the `descends_from_grant` column to the
    // `auth_request` table so the walker can climb AR → Grant → AR →
    // Grant ... up to the bootstrap node. Pre-CH-14 rows decode as
    // `None` via the `#[serde(default)]` shield on
    // `AuthRequest.descends_from_grant`.
    Migration {
        version: 14,
        slug: "authority_chain",
        sql: include_str!("../migrations/0014_authority_chain.surql"),
    },
    // CH-15 — Template A session-object grant backfill (ADR-0054
    // §D54.4). Walks every active legacy Template A grant
    // (resource_uri STARTS_WITH "project:" + descends_from points at
    // an auth_request with kinds CONTAINS "#template:a") and inserts
    // a paired `session_object/project:<uuid>` grant. The paired
    // grant carries the same holder + action + provenance + audit
    // class as the legacy grant; differs only in `resource_uri` +
    // `fundamentals = ["data_object", "tag"]`. Idempotent on re-run
    // via the migration-runner ledger + the inline NOT-EXISTS guard.
    Migration {
        version: 15,
        slug: "template_a_session_object_grant",
        sql: include_str!("../migrations/0015_template_a_session_object_grant.surql"),
    },
    // CH-17 — Append `Action::Observe` to every legacy Template A
    // grant (ADR-0055 §D55.9). Pre-CH-17 grants carried
    // [read, inspect, list]; CH-17 extends `fire_grant_on_lead_assignment`
    // to mint [read, inspect, list, observe] so the SSE live-event
    // tail handler's `[Observe]`-only manifest covers at Step 3.
    // Idempotent on re-run via the migration-runner ledger + the
    // inline `array::find_index(action, "observe") = NONE` guard.
    Migration {
        version: 16,
        slug: "template_a_session_object_grant_add_observe",
        sql: include_str!("../migrations/0016_template_a_session_object_grant_add_observe.surql"),
    },
    // CH-25 — Agent-as-creator-and-owner edge model (ADR-0060 §D60.1).
    // Adds the `owns` relation table backing the new `Edge::Owns`
    // variant for Agent → Org/Project ownership semantics. F1.b user-
    // lock divergence from planner-recommended F1.a: NEW dedicated
    // relation table rather than reusing the generic `owned_by`
    // relation; Org/Project STAY Principal-only in the v0 ontology.
    Migration {
        version: 17,
        slug: "add_owns_relation",
        sql: include_str!("../migrations/0017_add_owns_relation.surql"),
    },
];

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("failed to read migration ledger: {0}")]
    Ledger(String),
    #[error("migration {version} ({slug}) failed: {error}")]
    Apply {
        version: i64,
        slug: String,
        error: String,
    },
    #[error("embedded migration list is out of order at version {0}")]
    OutOfOrder(i64),
}

/// Run every migration whose `version` is not yet recorded in `_migrations`,
/// in ascending order. Returns the list of newly-applied versions so the
/// caller can log or surface them.
pub async fn run_migrations(
    db: &Surreal<Any>,
    migrations: &[Migration],
) -> Result<Vec<i64>, MigrationError> {
    validate_ordering(migrations)?;

    bootstrap_ledger_table(db).await?;
    let applied = read_applied_versions(db).await?;

    let mut newly_applied = Vec::new();
    for migration in migrations {
        if applied.contains(&migration.version) {
            continue;
        }
        apply_one(db, migration).await?;
        newly_applied.push(migration.version);
    }

    Ok(newly_applied)
}

fn validate_ordering(migrations: &[Migration]) -> Result<(), MigrationError> {
    let mut last = i64::MIN;
    for m in migrations {
        if m.version <= last {
            return Err(MigrationError::OutOfOrder(m.version));
        }
        last = m.version;
    }
    Ok(())
}

/// Create the `_migrations` ledger if it does not already exist. The initial
/// migration (`0001_initial.surql`) also `DEFINE TABLE`s it; running this
/// here lets the runner read from the ledger before the first migration has
/// applied.
async fn bootstrap_ledger_table(db: &Surreal<Any>) -> Result<(), MigrationError> {
    db.query(
        "DEFINE TABLE IF NOT EXISTS _migrations SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS version    ON _migrations TYPE int;
         DEFINE FIELD IF NOT EXISTS slug       ON _migrations TYPE string;
         DEFINE FIELD IF NOT EXISTS applied_at ON _migrations TYPE string;
         DEFINE INDEX IF NOT EXISTS _migrations_version ON _migrations FIELDS version UNIQUE;",
    )
    .await
    .map_err(|e| MigrationError::Ledger(e.to_string()))?
    .check()
    .map_err(|e| MigrationError::Ledger(e.to_string()))?;
    Ok(())
}

async fn read_applied_versions(db: &Surreal<Any>) -> Result<Vec<i64>, MigrationError> {
    let mut resp = db
        .query("SELECT version FROM _migrations")
        .await
        .map_err(|e| MigrationError::Ledger(e.to_string()))?;
    let rows: Vec<i64> = resp
        .take((0, "version"))
        .map_err(|e| MigrationError::Ledger(e.to_string()))?;
    Ok(rows)
}

async fn apply_one(db: &Surreal<Any>, migration: &Migration) -> Result<(), MigrationError> {
    // Apply the migration's DDL. We deliberately do NOT wrap it in a
    // BEGIN/COMMIT since SurrealDB does not support DDL inside transactions
    // in the embedded backend; instead we rely on the `_migrations` ledger
    // row being written only after the DDL succeeds — partial application
    // leaves the ledger without a row and the next startup retries.
    db.query(migration.sql)
        .await
        .map_err(|e| MigrationError::Apply {
            version: migration.version,
            slug: migration.slug.to_string(),
            error: e.to_string(),
        })?
        .check()
        .map_err(|e| MigrationError::Apply {
            version: migration.version,
            slug: migration.slug.to_string(),
            error: e.to_string(),
        })?;

    // Record the applied version. If this fails, the next startup re-applies
    // the DDL — callers that ship non-idempotent DDL should add a follow-on
    // migration rather than rely on retries.
    //
    // `applied_at` is declared as `string` (RFC3339) in the ledger schema,
    // so binding a chrono RFC3339 value is the direct path.
    db.query("CREATE _migrations SET version = $version, slug = $slug, applied_at = $applied_at")
        .bind(("version", migration.version))
        .bind(("slug", migration.slug.to_string()))
        .bind(("applied_at", chrono::Utc::now().to_rfc3339()))
        .await
        .map_err(|e| MigrationError::Apply {
            version: migration.version,
            slug: migration.slug.to_string(),
            error: e.to_string(),
        })?
        .check()
        .map_err(|e| MigrationError::Apply {
            version: migration.version,
            slug: migration.slug.to_string(),
            error: e.to_string(),
        })?;

    tracing::info!(
        version = migration.version,
        slug = migration.slug,
        "migration applied"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Unit tests run against embedded RocksDB in a tempdir — same backend
    //! the release binary uses. Each test owns its own tempdir so runs are
    //! independent.

    use super::*;
    use surrealdb::engine::any::connect;
    use tempfile::TempDir;

    /// Fixture: a fresh RocksDB-backed store in a tempdir. The `TempDir`
    /// handle is returned so it drops (and deletes the dir) when the test
    /// ends.
    async fn fresh_db() -> (Surreal<Any>, TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("db").to_string_lossy().to_string();
        let uri = format!("rocksdb://{path}");
        let db = connect(uri.as_str()).await.expect("open rocksdb");
        db.use_ns("test").use_db("test").await.expect("ns/db");
        (db, dir)
    }

    #[tokio::test]
    async fn runs_embedded_migrations_from_empty_db() {
        let (db, _dir) = fresh_db().await;
        let applied = run_migrations(&db, EMBEDDED_MIGRATIONS)
            .await
            .expect("fresh db migrates green");
        // Every version in `EMBEDDED_MIGRATIONS` applies in order. Asserting
        // the full list (rather than just `vec![1]`) keeps the test honest
        // as new migrations are appended; each one must apply successfully
        // against an empty DB.
        let expected: Vec<i64> = EMBEDDED_MIGRATIONS.iter().map(|m| m.version).collect();
        assert_eq!(applied, expected);

        // `read_applied_versions` does not impose an ORDER BY — SurrealDB is
        // free to return ledger rows in any order — so sort both sides
        // before comparing.
        let mut seen = read_applied_versions(&db).await.expect("read ledger");
        seen.sort();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();
        assert_eq!(seen, expected_sorted);
    }

    #[tokio::test]
    async fn is_idempotent_across_successive_runs() {
        let (db, _dir) = fresh_db().await;
        let _ = run_migrations(&db, EMBEDDED_MIGRATIONS)
            .await
            .expect("first run");
        let second = run_migrations(&db, EMBEDDED_MIGRATIONS)
            .await
            .expect("second run");
        assert!(second.is_empty(), "no new migrations on second run");
    }

    #[tokio::test]
    async fn rejects_out_of_order_migrations() {
        let (db, _dir) = fresh_db().await;
        let bad = &[
            Migration {
                version: 2,
                slug: "second",
                sql: "",
            },
            Migration {
                version: 1,
                slug: "first",
                sql: "",
            },
        ];
        match run_migrations(&db, bad).await {
            Err(MigrationError::OutOfOrder(v)) => assert_eq!(v, 1),
            other => panic!("expected OutOfOrder, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn broken_migration_surfaces_apply_error_without_ledger_row() {
        let (db, _dir) = fresh_db().await;
        let bad = &[Migration {
            version: 1,
            slug: "broken",
            sql: "THIS IS NOT VALID SURQL AT ALL;",
        }];
        match run_migrations(&db, bad).await {
            Err(MigrationError::Apply { version, slug, .. }) => {
                assert_eq!(version, 1);
                assert_eq!(slug, "broken");
            }
            other => panic!("expected Apply error, got {:?}", other),
        }

        // Ledger must NOT have recorded the failed migration — the next
        // startup will retry.
        let seen = read_applied_versions(&db).await.expect("read ledger");
        assert!(seen.is_empty(), "failed migration must not be recorded");
    }
}
