//! `list_providers` — catalogue read over model-runtime rows.
//!
//! Returns every field including the embedded phi-core `ModelConfig`
//! (minus the always-empty `api_key` sentinel). No audit event —
//! list ops are routine; the hash-chain stays live via the writes
//! those reads surface.

use std::sync::Arc;

use domain::model::ModelRuntime;
use domain::repository::Repository;

use super::ProviderError;

pub struct ListInput {
    pub include_archived: bool,
}

pub struct ListOutcome {
    pub runtimes: Vec<ModelRuntime>,
}

pub async fn list_providers(
    repo: Arc<dyn Repository>,
    input: ListInput,
) -> Result<ListOutcome, ProviderError> {
    let runtimes = repo.list_model_providers(input.include_archived).await?;
    Ok(ListOutcome { runtimes })
}
