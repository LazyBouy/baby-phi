//! `list_secrets` — metadata-only vault catalogue read.
//!
//! Returns every non-sealed field: id, slug, custodian, sensitive flag,
//! last_rotated_at, created_at. Ciphertext bytes are NOT returned — the
//! only path to plaintext is [`super::reveal::reveal_secret`].
//!
//! Emits a `Logged`-tier audit event (`vault.secret.list_read`) on
//! success — platform operators listing the vault is routine, but the
//! chain must stay live for replay.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::secrets as secret_events;
use domain::audit::AuditEmitter;
use domain::model::ids::AgentId;
use domain::model::SecretCredential;
use domain::repository::Repository;

use super::SecretError;

pub struct ListOutcome {
    pub credentials: Vec<SecretCredential>,
}

pub async fn list_secrets(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    actor: AgentId,
    now: DateTime<Utc>,
) -> Result<ListOutcome, SecretError> {
    let credentials = repo.list_secrets().await?;
    let event = secret_events::secret_list_read(actor, credentials.len(), now);
    audit
        .emit(event)
        .await
        .map_err(|e| SecretError::AuditEmit(e.to_string()))?;
    Ok(ListOutcome { credentials })
}
