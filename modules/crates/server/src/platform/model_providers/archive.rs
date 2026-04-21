//! `archive_provider` — soft-delete a model runtime row.
//!
//! Sets `archived_at` + emits `platform.model_provider.archived` (Alerted).
//! Grants descending from the original registration AR are **not
//! revoked** in M2 — M2 has a single admin holding the grant, so
//! there's no multi-principal exposure; M3 wires cascade revocation
//! when delegated grants become common (see archived plan Part 11 Q8).
//!
//! Double-archive is a no-op on the row (`archived_at` stays at the
//! first archive time) but still emits a new Alerted event — useful
//! if an operator wants to re-check status or leave an audit trail
//! note.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::providers as provider_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::{AgentId, ModelProviderId};
use domain::model::nodes::{PrincipalRef, ResourceRef};
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{provider_uri, ArchiveOutcome, ProviderError, KIND_TAG};

pub struct ArchiveInput {
    pub provider_id: ModelProviderId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn archive_provider(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: ArchiveInput,
) -> Result<ArchiveOutcome, ProviderError> {
    // Look up the row (for the audit diff + a clean 404 on unknown id).
    let existing = repo
        .list_model_providers(true)
        .await?
        .into_iter()
        .find(|r| r.id == input.provider_id)
        .ok_or(ProviderError::NotFound(input.provider_id))?;

    // Template E AR for the archive write — self-approved.
    let uri = provider_uri(input.provider_id);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec!["archive".to_string()],
        justification: Some(format!(
            "self-approved platform-admin write: archive model provider `{}`",
            input.provider_id
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;
    repo.create_auth_request(&ar).await?;

    // Flip the row.
    repo.archive_model_provider(input.provider_id, input.now)
        .await?;

    // Build the post-archive snapshot for the audit "before" diff —
    // we emit with the PREVIOUS state so a reviewer sees what the
    // provider looked like before archival.
    let event = provider_events::model_provider_archived(
        input.actor,
        &existing,
        Some(auth_request_id),
        input.now,
    );
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| ProviderError::AuditEmit(e.to_string()))?;

    Ok(ArchiveOutcome {
        provider_id: input.provider_id,
        audit_event_id,
    })
}
