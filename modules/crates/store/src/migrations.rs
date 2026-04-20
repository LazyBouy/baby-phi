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

use surrealdb::engine::local::Db;
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
pub const EMBEDDED_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    slug: "initial",
    sql: include_str!("../migrations/0001_initial.surql"),
}];

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
    db: &Surreal<Db>,
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
async fn bootstrap_ledger_table(db: &Surreal<Db>) -> Result<(), MigrationError> {
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

async fn read_applied_versions(db: &Surreal<Db>) -> Result<Vec<i64>, MigrationError> {
    let mut resp = db
        .query("SELECT version FROM _migrations")
        .await
        .map_err(|e| MigrationError::Ledger(e.to_string()))?;
    let rows: Vec<i64> = resp
        .take((0, "version"))
        .map_err(|e| MigrationError::Ledger(e.to_string()))?;
    Ok(rows)
}

async fn apply_one(db: &Surreal<Db>, migration: &Migration) -> Result<(), MigrationError> {
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
    use surrealdb::engine::local::RocksDb;
    use tempfile::TempDir;

    /// Fixture: a fresh RocksDB-backed store in a tempdir. The `TempDir`
    /// handle is returned so it drops (and deletes the dir) when the test
    /// ends.
    async fn fresh_db() -> (Surreal<Db>, TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("db").to_string_lossy().to_string();
        let db = Surreal::new::<RocksDb>(path.as_str())
            .await
            .expect("open rocksdb");
        db.use_ns("test").use_db("test").await.expect("ns/db");
        (db, dir)
    }

    #[tokio::test]
    async fn runs_embedded_migrations_from_empty_db() {
        let (db, _dir) = fresh_db().await;
        let applied = run_migrations(&db, EMBEDDED_MIGRATIONS)
            .await
            .expect("fresh db migrates green");
        assert_eq!(applied, vec![1]);

        let seen = read_applied_versions(&db).await.expect("read ledger");
        assert_eq!(seen, vec![1]);
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
