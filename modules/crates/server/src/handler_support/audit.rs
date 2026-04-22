//! Thin `emit_audit` helper — turns an `AuditEmitter` + `AuditEvent`
//! into a handler-friendly `Result<(), ApiError>`.
//!
//! Why a shim rather than `.await?` at the call site?
//!   1. Handlers handle `ApiError`, not `RepositoryError`. The helper
//!      does the mapping.
//!   2. Audit-emit failure is a **500**, not a pass-through of the
//!      underlying `RepositoryError` — a compromised audit trail means
//!      the handler's write should NOT appear to have succeeded from
//!      the operator's perspective.
//!   3. Handlers that forget to await the emitter trip the lint here.

use domain::audit::{AuditEmitter, AuditEvent};
use domain::model::ids::AuditEventId;

use super::errors::ApiError;

/// Emit an event; map any repository error to `ApiError::audit_emit_failed`.
pub async fn emit_audit(emitter: &dyn AuditEmitter, event: AuditEvent) -> Result<(), ApiError> {
    emitter
        .emit(event)
        .await
        .map_err(|e| ApiError::audit_emit_failed(e.to_string()))
}

/// Emit a batch of events in input order, returning their ids on
/// success.
///
/// Introduced in M3/P3 for `apply_org_creation` callers that need to
/// emit `platform.organization.created` + N × `authority_template.adopted`
/// after a successful compound-tx commit. Emitting each event via
/// `emit_audit` in sequence preserves **per-org hash-chain
/// continuity**: each event's `prev_event_hash` references the one
/// emitted immediately before it in this batch (via the emitter's
/// `last_event_hash_for_org(event.org_scope)` lookup).
///
/// **Fail-fast semantics.** On the first emit failure, the helper
/// returns `Err(ApiError::audit_emit_failed(_))` immediately — it does
/// NOT attempt to emit the remaining events. Downstream callers must
/// treat audit-emit failure as a 500 (see [`emit_audit`] doc) and the
/// handler's state-changing write is NOT considered "user-visible
/// success" — typical recovery is manual chain repair or a fresh
/// retry against an idempotent repo write. Partial batches leave the
/// hash chain consistent up to the last successfully-emitted event;
/// events after that point are simply never recorded.
///
/// **Ordering guarantee.** The returned `Vec<AuditEventId>` has the
/// same length as the input and the same element order. Callers can
/// pair each id with its source event by index.
pub async fn emit_audit_batch(
    emitter: &dyn AuditEmitter,
    events: Vec<AuditEvent>,
) -> Result<Vec<AuditEventId>, ApiError> {
    let mut ids = Vec::with_capacity(events.len());
    for event in events {
        let id = event.event_id;
        emit_audit(emitter, event).await?;
        ids.push(id);
    }
    Ok(ids)
}
