use std::sync::Arc;

use dashmap::DashMap;
use domain::audit::AuditEmitter;
use domain::events::{
    AgentCatalogListener, CatalogAuditMode, EventBus, InProcessEventBus, MemoryExtractionListener,
    TemplateAFireListener, TemplateCFireListener, TemplateDFireListener,
};
use domain::model::ids::SessionId;
use domain::Repository;
use phi_core::types::event::AgentEvent;
use store::crypto::MasterKey;
use tokio::sync::broadcast;
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

/// Per-session live-event broadcast registry — the substrate the
/// CH-17 SSE tail endpoint reads from to fan out
/// `phi_core::AgentEvent`s to one-to-many connected clients.
///
/// Trait-shaped per CH-K8S-PREP §D33.1's precedent so the M7b
/// Redis-pub/sub-backed cross-pod fan-out (`CHK8S-D-10`) can swap a
/// new impl in rather than refactor the SSE handler. The default
/// impl [`InProcessSessionLiveStreamRegistry`] wraps a `DashMap`
/// keyed on `SessionId`; each value is a
/// `tokio::sync::broadcast::Sender<AgentEvent>` (cloning the Sender
/// is cheap; subscribers call `.subscribe()` on the clone).
///
/// **Single-pod-only at v0.** Live events produced on pod A are not
/// visible to SSE clients connected to pod B without Redis pub/sub
/// fan-out — see CHK8S-D-10 in
/// `m7b/architecture/deferred-from-ch-k8s-prep.md`.
pub trait SessionLiveStreamRegistry: Send + Sync {
    /// Register the broadcast Sender for a live session. Called by
    /// the launch handler immediately before spawning the agent task
    /// so the SSE handler can `.get()` and `.subscribe()` once the
    /// session is in flight.
    fn insert(&self, session_id: SessionId, tx: broadcast::Sender<AgentEvent>);

    /// Return a clone of the registered Sender, if any. Callers
    /// invoke `.subscribe()` on the returned Sender to obtain a fresh
    /// `Receiver`. Returns `None` when no session is live for the id
    /// (already finalised, or never launched).
    fn get(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;

    /// Atomically remove the registered Sender. The launch handler
    /// invokes this after `recorder.finalise_and_persist()` so
    /// subsequent SSE connections see the session as already
    /// terminated.
    fn remove(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;

    /// Current count of live sessions tracked by this registry.
    fn len(&self) -> usize;

    /// `true` when no sessions are tracked.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-process [`SessionLiveStreamRegistry`] impl backed by `DashMap`
/// for lock-free per-key access. The single-pod default.
pub struct InProcessSessionLiveStreamRegistry {
    inner: DashMap<SessionId, broadcast::Sender<AgentEvent>>,
}

impl InProcessSessionLiveStreamRegistry {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }
}

impl Default for InProcessSessionLiveStreamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLiveStreamRegistry for InProcessSessionLiveStreamRegistry {
    fn insert(&self, session_id: SessionId, tx: broadcast::Sender<AgentEvent>) {
        self.inner.insert(session_id, tx);
    }

    fn get(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>> {
        self.inner
            .get(session_id)
            .map(|entry| entry.value().clone())
    }

    fn remove(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>> {
        self.inner.remove(session_id).map(|(_k, v)| v)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Construct an empty in-process [`SessionLiveStreamRegistry`].
/// Callers (boot site, acceptance tests) use this rather than
/// importing `dashmap` directly so the dependency is confined to the
/// server crate's `[dependencies]` block.
pub fn new_session_live_stream_registry() -> Arc<dyn SessionLiveStreamRegistry> {
    Arc::new(InProcessSessionLiveStreamRegistry::new())
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
/// - `session_live_stream_registry` (CH-17 / ADR-0055 §D55.1) tracks
///   the per-session `tokio::sync::broadcast::Sender<AgentEvent>` the
///   recorder publishes into and the SSE handler subscribes from.
///   Trait-shaped per CH-K8S-PREP §D33.1's precedent so the M7b
///   Redis-pub/sub-backed cross-pod fan-out (CHK8S-D-10) can swap an
///   impl in rather than refactor.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub session: SessionKey,
    pub audit: Arc<dyn AuditEmitter>,
    pub master_key: Arc<MasterKey>,
    pub event_bus: Arc<dyn EventBus>,
    pub session_registry: Arc<dyn SessionRegistry>,
    pub session_live_stream_registry: Arc<dyn SessionLiveStreamRegistry>,
    /// Platform-wide concurrency ceiling. When
    /// `session_registry.len() >= max_concurrent`, a new launch is
    /// refused with 503 `SESSION_WORKER_SATURATED`. Default 16
    /// (config/default.toml `[session] max_concurrent = 16`).
    pub session_max_concurrent: u32,
    /// Per-session broadcast channel buffer size (CH-17 / ADR-0055
    /// §D55.2). Default 64 (config/default.toml
    /// `[session_live_stream] buffer = 64`). Lagging consumers receive
    /// a typed `Lagged` SSE error event then close per ADR-0055
    /// §D55.3.
    pub session_live_stream_buffer: usize,
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
/// - [`MemoryExtractionListener`] — CH-21 body shipped (heuristic v0;
///   one Memory + Identity bump + two audits + D6.1 first call site
///   fire on every non-aborted SessionEnded).
/// - [`AgentCatalogListener`] — CH-22 body shipped (8-variant catalog
///   refresh + D6.1 second call site).
pub fn build_event_bus_with_m5_listeners(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    catalog_audit_mode: CatalogAuditMode,
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

    // Memory extraction (CH-21 — body shipped; heuristic v0).
    bus.subscribe(Arc::new(MemoryExtractionListener::new(
        repo.clone(),
        audit.clone(),
    )));

    // Agent catalog (CH-22 — body shipped; audit_mode wired via
    // `[listeners.catalog]` config block).
    bus.subscribe(Arc::new(AgentCatalogListener::new(
        repo,
        audit,
        catalog_audit_mode,
    )));

    bus
}

/// CH-10 / ADR-0047 §D47.7 — Spawn the consent state-machine sweeper
/// task. The task loops `tokio::time::sleep(interval)` then calls
/// [`Repository::sweep_consent_timeouts`], which scans for past-deadline
/// `Requested` consents and flips them to `TimedOut` (with one
/// `consent.timed_out` audit per flip).
///
/// Returns a [`tokio::task::JoinHandle<()>`] so the caller can store +
/// abort it on graceful shutdown. When `interval` is `Duration::ZERO`
/// the task is not spawned and the handle returned points at an
/// immediately-completed future — used by acceptance tests that drive
/// the sweep manually.
///
/// **Single-pod-only at v0.** Multi-pod leader-election is deferred to
/// M7b per the entry in `m7b/architecture/deferred-from-ch-k8s-prep.md`.
/// Running this task on more than one pod simultaneously causes
/// duplicate `consent.timed_out` audit emissions for the same flip
/// (the storage UPDATE is idempotent, but the audit is not).
pub fn spawn_consent_sweeper(
    repo: Arc<dyn Repository>,
    interval: std::time::Duration,
) -> tokio::task::JoinHandle<()> {
    if interval.is_zero() {
        // Disabled — return a handle that immediately resolves so the
        // caller's bookkeeping doesn't need a special-case.
        return tokio::spawn(async {});
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // Skip the first immediate tick so the sweeper waits one
        // interval before its first sweep — matches operator
        // expectations ("set interval=60s" → first sweep ~60s after boot).
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ticker.tick().await; // consume the immediate first tick.
        loop {
            ticker.tick().await;
            match repo.sweep_consent_timeouts(chrono::Utc::now()).await {
                Ok(flipped) if !flipped.is_empty() => {
                    tracing::info!(
                        flipped_count = flipped.len(),
                        "consent sweeper: flipped {} consent(s) to TimedOut",
                        flipped.len(),
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "consent sweeper tick failed; retrying next interval",
                    );
                }
            }
        }
    })
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

    #[test]
    fn in_process_session_live_stream_registry_round_trips_through_trait_object() {
        // CH-17 / ADR-0055 §D55.1 — confirms trait-object dispatch
        // preserves DashMap insert/get/remove/len semantics so the M7b
        // Redis-pub/sub-backed swap (CHK8S-D-10) is a new impl, not a
        // refactor. Mirrors the SessionRegistry trait-object test
        // above (CH-K8S-PREP §D33.1 precedent).
        let registry: Arc<dyn SessionLiveStreamRegistry> = new_session_live_stream_registry();
        assert_eq!(registry.len(), 0, "fresh registry is empty");
        assert!(registry.is_empty(), "fresh registry is_empty");

        let session_id = SessionId::new();
        let (tx, rx) = broadcast::channel::<AgentEvent>(64);
        // Drop the launch-side initial receiver so the only receiver
        // count comes from a downstream `.subscribe()` call below.
        drop(rx);
        assert_eq!(
            tx.receiver_count(),
            0,
            "Sender starts with no live receivers after dropping the launch-side rx"
        );
        registry.insert(session_id, tx.clone());
        assert_eq!(registry.len(), 1, "len reflects the inserted entry");

        // get returns a clone of the registered Sender; subscribers
        // call .subscribe() on the returned clone.
        let cloned = registry.get(&session_id).expect("registered Sender");
        let _rx2 = cloned.subscribe();
        assert_eq!(
            tx.receiver_count(),
            1,
            "subscribe via get() clone bumps the original's receiver count"
        );

        // get on an unknown id returns None.
        assert!(
            registry.get(&SessionId::new()).is_none(),
            "get on unknown session_id yields None"
        );

        // remove yields the original Sender + drops the registry entry.
        let removed = registry
            .remove(&session_id)
            .expect("remove returns the inserted Sender");
        let _ = removed; // discard
        assert_eq!(registry.len(), 0, "len drops back to zero after remove");

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
        let bus = build_event_bus_with_m5_listeners(repo, audit, CatalogAuditMode::default());
        assert_eq!(
            bus.handler_count(),
            5,
            "M5/P3 + CH-21 + CH-22 wire Template A + C + D + \
             MemoryExtraction (body) + AgentCatalog (body) — exactly 5 subscribers",
        );
    }
}
