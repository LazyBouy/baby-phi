//! Shared shim every M2+ HTTP handler builds on.
//!
//! Four submodules, each wrapping one chunk of boilerplate that would
//! otherwise be rewritten by every page handler:
//!
//! - [`errors`] — stable `{code, message}` envelope that every 4xx/5xx
//!   response serialises through.
//! - [`session`] — `AuthenticatedSession` axum extractor that replaces
//!   the per-handler `verify_from_cookies` dance with a failed-401
//!   extraction.
//! - [`permission`] — `check_permission` that runs the engine + maps
//!   each `Decision` to either `Ok(_)` or a status-tagged `ApiError`
//!   per ADR-0018 / plan decision D10.
//! - [`audit`] — `emit_audit` helper that turns an `AuditEmitter`
//!   failure into `ApiError::audit_emit_failed` (500).
//!
//! See [ADR-0018 (handler_support module)](../../../../docs/specs/v0/implementation/m2/decisions/0018-handler-support-module.md)
//! for the design rationale.

pub mod audit;
pub mod errors;
pub mod permission;
pub mod session;

pub use audit::emit_audit;
pub use errors::ApiError;
pub use permission::{check_permission, denial_to_api_error};
pub use session::AuthenticatedSession;
