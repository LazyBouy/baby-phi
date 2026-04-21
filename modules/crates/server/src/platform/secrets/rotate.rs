//! `rotate_secret` — replace the sealed material on an existing row.
//!
//! Slug + custodian are unchanged; `last_rotated_at` is bumped. Plaintext
//! bytes are dropped after sealing and never appear in the audit diff.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::secrets as secret_events;
use domain::audit::AuditEmitter;
use domain::model::ids::AgentId;
use domain::model::SecretRef;
use domain::repository::{Repository, SealedBlob};
use store::crypto::{seal, MasterKey};

use super::{validate_slug, RotateOutcome, SecretError};

pub struct RotateInput<'a> {
    pub slug: &'a str,
    /// New plaintext material. Dropped after sealing.
    pub plaintext: &'a [u8],
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn rotate_secret(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    master_key: &MasterKey,
    input: RotateInput<'_>,
) -> Result<RotateOutcome, SecretError> {
    validate_slug(input.slug)?;
    let slug = SecretRef::new(input.slug);

    let (existing, _sealed) = repo
        .get_secret_by_slug(&slug)
        .await?
        .ok_or_else(|| SecretError::NotFound(input.slug.to_string()))?;
    let prev_rotated_at = existing.last_rotated_at;

    let sealed = seal(master_key, input.plaintext)?;
    let (ct_b64, nonce_b64) = sealed.to_base64();
    let new_sealed = SealedBlob {
        ciphertext_b64: ct_b64,
        nonce_b64,
    };

    repo.rotate_secret(existing.id, &new_sealed, input.now)
        .await?;

    // Refresh the credential with the new timestamp for the audit diff.
    let mut rotated = existing.clone();
    rotated.last_rotated_at = Some(input.now);

    let event = secret_events::secret_rotated(
        input.actor,
        &rotated,
        prev_rotated_at,
        None, // rotate is a Template E write but we don't persist a fresh AR per rotation in M2.
        input.now,
    );
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SecretError::AuditEmit(e.to_string()))?;

    Ok(RotateOutcome {
        secret_id: existing.id,
        slug: input.slug.to_string(),
        audit_event_id,
    })
}
