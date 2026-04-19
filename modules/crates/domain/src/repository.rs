//! Persistence boundary.
//!
//! The `store` crate implements this trait against SurrealDB. The domain
//! layer only ever talks to `Repository`, keeping SurrealDB as a swappable
//! adapter.

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("backend error: {0}")]
    Backend(String),
    #[error("not found")]
    NotFound,
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Readiness check used by `/healthz/ready`. Returns `Ok(())` if the
    /// backend is reachable and usable.
    async fn ping(&self) -> RepositoryResult<()>;
}
