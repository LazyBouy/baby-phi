//! SurrealDB-backed implementation of `domain::Repository`.
//!
//! v0.1 ships with the embedded RocksDB backend by default; CH-K8S-PREP P-2
//! adds [`SurrealStore::open_remote`] for connecting to a standalone
//! SurrealDB server (e.g., a K8s cluster). Both constructors yield the same
//! [`SurrealStore`] type — they differ only in the URI passed to
//! `surrealdb::engine::any::connect`.
//!
//! See `docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md`
//! and ADR-0033 for the M7b carve-out context.
//!
//! Submodules:
//! - [`crypto`] — AES-GCM envelope sealing for `secrets_vault.value`.
//! - [`migrations`] — forward-only SurrealDB migration runner with fail-safe
//!   startup gate.

pub mod audit_emitter;
pub mod crypto;
pub mod migrations;
pub mod repo_impl;
pub mod repo_impl_m2;

pub use audit_emitter::SurrealAuditEmitter;

use std::path::Path;

use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;

pub use crypto::{
    open as open_sealed, seal as seal_plaintext, CryptoError, MasterKey, SealedSecret,
    MASTER_KEY_ENV,
};
pub use migrations::{run_migrations, Migration, MigrationError, EMBEDDED_MIGRATIONS};

/// SurrealDB adapter. Holds a connected client and exposes domain operations
/// through the `Repository` trait.
///
/// Uses `surrealdb::engine::any::Any` for runtime backend selection — the
/// same struct backs both the embedded RocksDB and remote (ws/wss/http/https)
/// modes. Pre-CH-K8S-PREP this was hard-typed against `Surreal<Db>`
/// (local-only); the switch to `Surreal<Any>` is the K8s-prep that lets the
/// M7b carve-out flip to a remote URI without refactoring repository impls.
#[derive(Clone)]
pub struct SurrealStore {
    db: Surreal<Any>,
}

impl SurrealStore {
    /// Open (or create) an embedded SurrealDB instance backed by RocksDB at
    /// `path`, selecting the given namespace + database names. Runs every
    /// embedded migration that has not yet been applied — startup fails
    /// with [`StoreError::Migration`] if any migration errors, matching the
    /// fail-safe gate described in the M1 plan's commitment ledger row C7.
    pub async fn open_embedded(
        path: impl AsRef<Path>,
        namespace: &str,
        database: &str,
    ) -> Result<Self, StoreError> {
        let path = path.as_ref().to_string_lossy().to_string();
        let uri = format!("rocksdb://{path}");
        Self::open_with_uri(&uri, namespace, database).await
    }

    /// Connect to a remote (or in-memory) SurrealDB instance via a URI.
    ///
    /// CH-K8S-PREP P-2 / ADR-0033. The `uri` schema selects the backend:
    /// - `ws://host:port` / `wss://host:port` — remote SurrealDB server
    ///   (the typical M7b path for K8s deployments — pair with an
    ///   externalised SurrealDB cluster + a `[storage] mode = "remote"`
    ///   server config)
    /// - `memory://` — in-memory backend, primarily for tests
    /// - `rocksdb://path` — embedded (use [`open_embedded`] directly for
    ///   ergonomics; this constructor accepts it for completeness)
    ///
    /// Migrations run identically on remote backends — they're embedded
    /// SurrealQL files, not local-engine-specific. Multi-replica startup
    /// migration races are an M7b concern (per the readiness doc §B8) and
    /// are NOT addressed by this constructor.
    pub async fn open_remote(
        uri: &str,
        namespace: &str,
        database: &str,
    ) -> Result<Self, StoreError> {
        Self::open_with_uri(uri, namespace, database).await
    }

    async fn open_with_uri(uri: &str, namespace: &str, database: &str) -> Result<Self, StoreError> {
        let db = connect(uri)
            .await
            .map_err(|e| StoreError::Connect(e.to_string()))?;
        db.use_ns(namespace)
            .use_db(database)
            .await
            .map_err(|e| StoreError::Connect(e.to_string()))?;

        let applied = migrations::run_migrations(&db, migrations::EMBEDDED_MIGRATIONS)
            .await
            .map_err(StoreError::Migration)?;
        if !applied.is_empty() {
            tracing::info!(?applied, "startup-gate: migrations applied");
        }

        Ok(Self { db })
    }

    /// Expose the underlying SurrealDB client — escape hatch for migrations
    /// and ad-hoc queries in M1+. Prefer going through the `Repository` trait.
    pub fn client(&self) -> &Surreal<Any> {
        &self.db
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("failed to connect to SurrealDB: {0}")]
    Connect(String),
    #[error("query failed: {0}")]
    Query(String),
    #[error("schema migration failed: {0}")]
    Migration(#[from] MigrationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_remote_dispatches_via_any_engine_and_runs_migrations() {
        // CH-K8S-PREP P-2 / ADR-0033 — proves `open_remote` works
        // end-to-end through the `surrealdb::engine::any::connect`
        // dispatch path. The `rocksdb://` URI exercises the same Any
        // dispatch + migration-runner code path that a `ws://...`
        // remote URI would, but without requiring a network-reachable
        // SurrealDB or sticky test infra. (At M7b, prod swaps to
        // `ws://surrealdb.svc:8000` — the constructor body is
        // identical.)
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("db").to_string_lossy().to_string();
        let uri = format!("rocksdb://{path}");
        let store = SurrealStore::open_remote(&uri, "test-ns", "test-db")
            .await
            .expect("open_remote dispatches via Any engine");

        // Sanity-check the client is reachable + the migration ledger
        // is populated (proves migrations ran across the same code
        // path open_embedded uses).
        let row_count: Vec<i64> = store
            .client()
            .query("SELECT count() FROM _migrations GROUP ALL")
            .await
            .expect("query ledger")
            .take((0, "count"))
            .expect("take");
        let count = row_count.into_iter().next().unwrap_or(0);
        assert!(
            count >= 6,
            "every embedded migration must apply on the Any-engine \
             dispatch path (expect ≥6 rows in _migrations; got {count})"
        );
    }

    #[tokio::test]
    async fn open_remote_returns_connect_error_for_unreachable_uri() {
        // Unreachable `ws://` URI — proves the error type is
        // `StoreError::Connect`, the path the M7b health-check / pod
        // readiness gate will rely on.
        let result = SurrealStore::open_remote("ws://127.0.0.1:9", "test-ns", "test-db").await;
        match result {
            Err(StoreError::Connect(_)) => {} // expected
            Err(other) => panic!("expected Connect error, got {other:?}"),
            Ok(_) => panic!("unexpectedly connected to ws://127.0.0.1:9"),
        }
    }
}
