use std::sync::Arc;

use domain::audit::AuditEmitter;
use domain::events::EventBus;
use domain::Repository;
use store::crypto::MasterKey;

use crate::session::SessionKey;

/// Shared application state injected into every axum handler via
/// `State<AppState>`.
///
/// - `repo` is held behind a trait object so acceptance tests can swap in
///   in-memory fakes without touching handler code.
/// - `session` carries the HS256 signing key + cookie-shape settings for
///   [`crate::session::sign_and_build_cookie`] / [`crate::session::verify_from_cookies`].
/// - `audit` is the M2 audit emitter — every M2+ write handler emits
///   through this. Trait-object so acceptance tests can inject fakes.
/// - `master_key` is the 32-byte AES-GCM key used by the credentials
///   vault (page 04). Held behind `Arc` so handlers can pass it by
///   reference without cloning the inner bytes.
/// - `event_bus` is the M4/P3 in-process domain-event bus.
///   `apply_project_creation` callers emit
///   [`domain::events::DomainEvent::HasLeadEdgeCreated`] on it so
///   the [`domain::events::TemplateAFireListener`] subscriber issues
///   the lead grant. Held behind a trait object so tests can swap in
///   a bus-less / no-op implementation when reactive behaviour is
///   out-of-scope.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub session: SessionKey,
    pub audit: Arc<dyn AuditEmitter>,
    pub master_key: Arc<MasterKey>,
    pub event_bus: Arc<dyn EventBus>,
}
