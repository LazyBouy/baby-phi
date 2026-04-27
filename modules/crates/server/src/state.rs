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
/// registry beyond `config.session.max_concurrent` fails with 503
/// code `SESSION_WORKER_SATURATED`.
///
/// Trait-shaped per CH-K8S-PREP P-1 / ADR-0033 so the M7b Redis-backed
/// shared registry (per ADR-0031 §D31.1) can be a new impl rather
/// than a multi-file refactor. The default impl
/// [`InProcessSessionRegistry`] wraps `DashMap` for lock-free per-key
/// access — same single-pod semantics as before.
pub trait SessionRegistry: Send + Sync {
    /// Register a live session's cancellation token.
    fn insert(&self, session_id: SessionId, token: CancellationToken);

    /// Atomically remove and return the cancellation token for a
    /// session. Returns `None` if the session is not registered (e.g.
    /// already terminated, or never launched).
    fn remove(&self, session_id: &SessionId) -> Option<CancellationToken>;

    /// Current count of live sessions tracked by this registry.
    /// Used at launch time to enforce the platform-wide concurrency
    /// ceiling (`session_max_concurrent` / ADR-0031 §D31.2).
    fn len(&self) -> usize;

    /// `true` when no sessions are tracked. Default impl uses `len`;
    /// concrete impls may override for cheaper checks.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Fire `cancel()` on every live cancellation token without
    /// removing the entries — each `spawn_agent_task` removes its own
    /// entry when `agent_loop` returns post-cancellation.
    ///
    /// Used by the graceful-shutdown handler (CH-K8S-PREP P-3 / ADR-0031
    /// §D31.5). At M7b's broker-backed impl this fans out a "cancel"
    /// message over the shared store so cancellation is global.
    fn cancel_all(&self);
}

/// In-process [`SessionRegistry`] impl backed by `DashMap` for
/// lock-free per-key access. The single-pod default since M5;
/// remains the dev/CI default after CH-K8S-PREP P-1.
///
/// At M7b, a sibling `RedisSessionRegistry` impl will satisfy the
/// trait against a shared store so cancellation tokens flow across
/// pods.
pub struct InProcessSessionRegistry {
    inner: DashMap<SessionId, CancellationToken>,
}

impl InProcessSessionRegistry {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }
}

impl Default for InProcessSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRegistry for InProcessSessionRegistry {
    fn insert(&self, session_id: SessionId, token: CancellationToken) {
        self.inner.insert(session_id, token);
    }

    fn remove(&self, session_id: &SessionId) -> Option<CancellationToken> {
        self.inner.remove(session_id).map(|(_k, v)| v)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn cancel_all(&self) {
        for entry in self.inner.iter() {
            entry.value().cancel();
        }
    }
}

/// Construct an empty in-process [`SessionRegistry`]. Callers (boot
/// site, acceptance tests) use this rather than importing `dashmap`
/// directly so the dependency is confined to the server crate's
/// `[dependencies]` block.
pub fn new_session_registry() -> Arc<dyn SessionRegistry> {
    Arc::new(InProcessSessionRegistry::new())
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
    pub session_registry: Arc<dyn SessionRegistry>,
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

    #[test]
    fn in_process_session_registry_round_trips_through_trait_object() {
        // CH-K8S-PREP P-1 / ADR-0033 — confirms trait-object dispatch
        // preserves DashMap insert/remove/len semantics so the M7b
        // Redis-backed swap is a new impl, not a refactor.
        let registry: Arc<dyn SessionRegistry> = new_session_registry();
        assert_eq!(registry.len(), 0, "fresh registry is empty");

        let session_id = SessionId::new();
        let token = CancellationToken::new();
        registry.insert(session_id, token.clone());
        assert_eq!(registry.len(), 1, "len reflects the inserted entry");

        let removed = registry
            .remove(&session_id)
            .expect("remove returns the inserted token");
        assert!(
            !removed.is_cancelled(),
            "remove yields the original token, not a cancelled clone"
        );
        assert_eq!(registry.len(), 0, "len drops back to zero after remove");

        // Cancelling the returned token must affect the original
        // (Arc-shared internals) — proves remove returns a live
        // handle, not a snapshot.
        removed.cancel();
        assert!(
            token.is_cancelled(),
            "the original token sees the cancellation through the Arc"
        );

        // Removing an unknown id is a no-op.
        assert!(
            registry.remove(&SessionId::new()).is_none(),
            "remove on an unknown session_id yields None"
        );
    }

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
