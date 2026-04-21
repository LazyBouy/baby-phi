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

use super::errors::ApiError;

/// Emit an event; map any repository error to `ApiError::audit_emit_failed`.
pub async fn emit_audit(emitter: &dyn AuditEmitter, event: AuditEvent) -> Result<(), ApiError> {
    emitter
        .emit(event)
        .await
        .map_err(|e| ApiError::audit_emit_failed(e.to_string()))
}
