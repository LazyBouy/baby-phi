//! `get_platform_defaults` — synthesise the fullest GET response.
//!
//! Always returns both the live defaults (factory baseline on a
//! fresh install) and a `factory` snapshot — the web UI renders
//! them side-by-side so operators can see what they'd revert to.
//!
//! No audit event — list ops are routine; the hash-chain stays live
//! via the PUT writes these reads surface.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::model::PlatformDefaults;
use domain::repository::Repository;

use super::{DefaultsError, GetOutcome};

pub struct GetInput {
    pub now: DateTime<Utc>,
}

pub async fn get_platform_defaults(
    repo: Arc<dyn Repository>,
    input: GetInput,
) -> Result<GetOutcome, DefaultsError> {
    let persisted_row = repo.get_platform_defaults().await?;
    let factory = PlatformDefaults::factory(input.now);
    let (defaults, persisted) = match persisted_row {
        Some(row) => (row, true),
        None => (factory.clone(), false),
    };
    Ok(GetOutcome {
        defaults,
        persisted,
        factory,
    })
}
