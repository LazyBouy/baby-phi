//! SurrealDB-backed implementation of `domain::Repository`.
//!
//! v0.1 defaults to the embedded RocksDB backend. See
//! `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §"Scaling path" for the
//! migration route to standalone SurrealDB server or TiKV cluster — the
//! connection string is the only thing that changes at those tiers.

use std::path::Path;

use async_trait::async_trait;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

use domain::repository::{Repository, RepositoryError, RepositoryResult};

/// SurrealDB adapter. Holds a connected client and exposes domain operations
/// through the `Repository` trait.
#[derive(Clone)]
pub struct SurrealStore {
    db: Surreal<Db>,
}

impl SurrealStore {
    /// Open (or create) an embedded SurrealDB instance backed by RocksDB at
    /// `path`, selecting the given namespace + database names.
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
        Ok(Self { db })
    }

    /// Expose the underlying SurrealDB client — escape hatch for migrations
    /// and ad-hoc queries in M1+. Prefer going through the `Repository` trait.
    pub fn client(&self) -> &Surreal<Db> {
        &self.db
    }
}

#[async_trait]
impl Repository for SurrealStore {
    async fn ping(&self) -> RepositoryResult<()> {
        self.db
            .health()
            .await
            .map_err(|e| RepositoryError::Backend(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("failed to connect to SurrealDB: {0}")]
    Connect(String),
    #[error("query failed: {0}")]
    Query(String),
}
