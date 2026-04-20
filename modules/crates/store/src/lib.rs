//! SurrealDB-backed implementation of `domain::Repository`.
//!
//! v0.1 defaults to the embedded RocksDB backend. See
//! `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §"Scaling path" for the
//! migration route to standalone SurrealDB server or TiKV cluster — the
//! connection string is the only thing that changes at those tiers.
//!
//! Submodules:
//! - [`crypto`] — AES-GCM envelope sealing for `secrets_vault.value`.
//! - [`migrations`] — forward-only SurrealDB migration runner with fail-safe
//!   startup gate.

pub mod crypto;
pub mod migrations;
pub mod repo_impl;

use std::path::Path;

use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

pub use crypto::{
    open as open_sealed, seal as seal_plaintext, CryptoError, MasterKey, SealedSecret,
    MASTER_KEY_ENV,
};
pub use migrations::{run_migrations, Migration, MigrationError, EMBEDDED_MIGRATIONS};

/// SurrealDB adapter. Holds a connected client and exposes domain operations
/// through the `Repository` trait.
#[derive(Clone)]
pub struct SurrealStore {
    db: Surreal<Db>,
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
        let db = Surreal::new::<RocksDb>(path.as_str())
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
    pub fn client(&self) -> &Surreal<Db> {
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
