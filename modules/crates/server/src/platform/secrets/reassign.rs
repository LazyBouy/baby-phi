//! `reassign_custody` — hand custodianship to a different Agent.
//!
//! The sealed material is untouched; only the `custodian` field on the
//! catalogue row changes. Audit diff carries both the old and new
//! custodian so the reviewer can trace delegation without a second
//! lookup.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::secrets as secret_events;
use domain::audit::AuditEmitter;
use domain::model::ids::AgentId;
use domain::model::SecretRef;
use domain::repository::Repository;

use super::{validate_slug, ReassignOutcome, SecretError};

pub struct ReassignInput<'a> {
    pub slug: &'a str,
    pub new_custodian: AgentId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn reassign_custody(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: ReassignInput<'_>,
) -> Result<ReassignOutcome, SecretError> {
    validate_slug(input.slug)?;
    let slug = SecretRef::new(input.slug);

    let (existing, _sealed) = repo
        .get_secret_by_slug(&slug)
        .await?
        .ok_or_else(|| SecretError::NotFound(input.slug.to_string()))?;
    let previous_custodian = existing.custodian;

    repo.reassign_secret_custodian(existing.id, input.new_custodian)
        .await?;

    let mut reassigned = existing.clone();
    reassigned.custodian = input.new_custodian;

    let event = secret_events::secret_custody_reassigned(
        input.actor,
        &reassigned,
        previous_custodian,
        None,
        input.now,
    );
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| SecretError::AuditEmit(e.to_string()))?;

    Ok(ReassignOutcome {
        secret_id: existing.id,
        slug: input.slug.to_string(),
        audit_event_id,
    })
}
