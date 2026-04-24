use std::sync::Arc;

use dashmap::DashMap;
use domain::audit::AuditEmitter;
use domain::events::{
    AgentCatalogListener, EventBus, InProcessEventBus, MemoryExtractionListener,
    TemplateAFireListener, TemplateCFireListener, TemplateDFireListener,
};
use domain::model::ids::SessionId;
use domain::Repository;
use store::crypto::MasterKey;
use tokio_util::sync::CancellationToken;

use crate::platform::projects::{
    RepoActorResolver, RepoAdoptionArResolver, RepoTemplateCAdoptionArResolver,
    RepoTemplateDAdoptionArResolver,
};
use crate::session::SessionKey;

/// Per-worker session registry — the platform-wide cancellation and
/// concurrency-cap surface that [ADR-0031](../../../docs/specs/v0/implementation/m5/decisions/0031-session-cancellation-and-concurrency.md) pins.
///
/// Every successful `sessions::launch` inserts an entry. `terminate`
/// and natural session-end remove it. A launch that would grow the
/// map beyond `config.session.max_concurrent` fails with 503 code
/// `SESSION_WORKER_SATURATED`.
///
/// `DashMap` gives lock-free per-key access. The governance-plane
/// event bus is the only cross-session coordination surface. It
/// already uses snapshot-and-release semantics (ADR-0028).
pub type SessionRegistry = Arc<DashMap<SessionId, CancellationToken>>;

/// Construct an empty [`SessionRegistry`]. Callers (boot site,
/// acceptance tests) use this rather than importing `dashmap`
/// directly so the dependency is confined to the server crate's
/// `[dependencies]` block.
pub fn new_session_registry() -> SessionRegistry {
    Arc::new(DashMap::new())
}

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
/// - `session_registry` (M5/P4) tracks every live session's
///   cancellation token keyed on `SessionId`. `sessions::launch`
///   inserts; `sessions::terminate` calls `cancel()` + removes. The
///   map's size is the platform-wide concurrency count for
///   ADR-0031's `SESSION_WORKER_SATURATED` gate.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub session: SessionKey,
    pub audit: Arc<dyn AuditEmitter>,
    pub master_key: Arc<MasterKey>,
    pub event_bus: Arc<dyn EventBus>,
    pub session_registry: SessionRegistry,
    /// Platform-wide concurrency ceiling. When
    /// `session_registry.len() >= max_concurrent`, a new launch is
    /// refused with 503 `SESSION_WORKER_SATURATED`. Default 16
    /// (config/default.toml `[session] max_concurrent = 16`).
    pub session_max_concurrent: u32,
}

/// Build an [`InProcessEventBus`] with every M5-era listener
/// subscribed.
///
/// Called from `main.rs` at boot time and from the
/// `handler_count_is_five_at_m5` test so both paths exercise the
/// same wiring. After return, `InProcessEventBus::handler_count()`
/// equals **5**:
/// - [`TemplateAFireListener`] — M4 (HAS_LEAD → lead grant).
/// - [`TemplateCFireListener`] — M5/P3 (MANAGES → manager grant).
/// - [`TemplateDFireListener`] — M5/P3 (HAS_AGENT_SUPERVISOR →
///   supervisor grant).
/// - [`MemoryExtractionListener`] — M5/P3 stub (body at M5/P8).
/// - [`AgentCatalogListener`] — M5/P3 stub (body at M5/P8).
pub fn build_event_bus_with_m5_listeners(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
) -> Arc<InProcessEventBus> {
    let bus = Arc::new(InProcessEventBus::new());

    // Template A (M4 — re-wired here so all listeners register via
    // the same helper).
    bus.subscribe(Arc::new(TemplateAFireListener::new(
        repo.clone(),
        audit.clone(),
        Arc::new(RepoAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo.clone())),
    )));

    // Template C (M5/P3).
    bus.subscribe(Arc::new(TemplateCFireListener::new(
        repo.clone(),
        audit.clone(),
        Arc::new(RepoTemplateCAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo.clone())),
    )));

    // Template D (M5/P3).
    bus.subscribe(Arc::new(TemplateDFireListener::new(
        repo.clone(),
        audit.clone(),
        Arc::new(RepoTemplateDAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo.clone())),
    )));

    // Memory extraction (M5/P3 stub).
    bus.subscribe(Arc::new(MemoryExtractionListener::new(
        repo.clone(),
        audit.clone(),
    )));

    // Agent catalog (M5/P3 stub).
    bus.subscribe(Arc::new(AgentCatalogListener::new(repo, audit)));

    bus
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::audit::NoopAuditEmitter;
    use domain::in_memory::InMemoryRepository;

    #[tokio::test]
    async fn handler_count_is_five_at_m5() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
        let bus = build_event_bus_with_m5_listeners(repo, audit);
        assert_eq!(
            bus.handler_count(),
            5,
            "M5/P3 wires Template A + C + D + MemoryExtraction (stub) \
             + AgentCatalog (stub) — exactly 5 subscribers",
        );
    }
}
