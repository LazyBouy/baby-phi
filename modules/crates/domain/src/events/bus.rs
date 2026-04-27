//! The [`EventBus`] trait + its [`InProcessEventBus`] default impl.
//!
//! See the parent module's doc-comment for rationale. Short version:
//! in-process pub/sub driven by `Arc<dyn EventHandler>` subscribers
//! registered at `AppState` boot. `emit` is called *after* the owning
//! compound-tx commits — if any handler errors, the tx is already
//! durable (fail-safe).

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use super::DomainEvent;

/// Returned by [`EventBus::drain`] when in-flight handler tasks fail
/// to complete within the supplied timeout. The `remaining` count
/// reports how many emit-or-handler tasks were still in progress when
/// the deadline expired — operators can correlate against logs to
/// decide whether to escalate.
#[derive(Debug, thiserror::Error)]
#[error("event-bus drain timed out with {remaining} in-flight emit(s) still running")]
pub struct DrainError {
    pub remaining: usize,
}

/// A subscriber callback registered against an [`EventBus`].
///
/// The trait is `Send + Sync` so an `Arc<dyn EventHandler>` can be
/// shared across tasks. Implementations handle errors internally
/// (log + drop) — the bus does not aggregate results because a single
/// failing listener must not block other listeners.
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_event(&self, event: &DomainEvent);
}

/// Publisher side of the domain-event pub/sub.
///
/// Object-safe so `Arc<dyn EventBus>` can be injected into handlers.
/// Implementations are responsible for snapshot-and-release semantics
/// on `emit` so listener code is not called while holding the
/// subscriber-list lock (to keep handler code re-entrant).
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Register a new subscriber. Order of registration is preserved;
    /// emit calls handlers in registration order.
    fn subscribe(&self, handler: Arc<dyn EventHandler>);

    /// Publish `event` to every registered subscriber.
    ///
    /// Implementations MUST:
    /// - snapshot the subscriber list before awaiting any handler;
    /// - call every subscriber even if one fails;
    /// - never hold a mutable lock across a `.await` point.
    /// - return early (no-op) when [`shutdown`](Self::shutdown) has
    ///   been called — emits arriving post-shutdown are dropped
    ///   silently. M7b's broker-backed impl will optionally buffer
    ///   to a durable replay queue; the in-process default does not.
    async fn emit(&self, event: DomainEvent);

    /// Signal that no further emits should be delivered.
    ///
    /// Idempotent (safe to call multiple times). After `shutdown`
    /// returns, [`emit`](Self::emit) calls become no-ops. Pair with
    /// [`drain`](Self::drain) to wait for in-flight emits to complete
    /// before exiting.
    ///
    /// CH-K8S-PREP P-4 / ADR-0033. Required by the SIGTERM handler
    /// in [`server::main`](../../../../../server/src/main.rs) as part
    /// of the graceful-shutdown sequence (ADR-0031 §D31.5).
    async fn shutdown(&self);

    /// Wait for in-flight emit calls to complete (up to `timeout`).
    ///
    /// Returns `Ok(())` when no emits are in flight, or
    /// `Err(DrainError { remaining })` when the deadline expires with
    /// in-flights still pending.
    ///
    /// For [`InProcessEventBus`] handlers run inline within `emit`,
    /// so this is a poll on an in-flight counter. M7b's broker-backed
    /// impl has stronger semantics (e.g., await ack from every
    /// subscriber group).
    async fn drain(&self, timeout: Duration) -> Result<(), DrainError>;
}

/// In-process default bus — a `Vec<Arc<dyn EventHandler>>` behind a
/// `std::sync::RwLock`. Suitable for both production (single process)
/// and tests (no extra infrastructure). Cross-process fan-out is a
/// future addition (ADR-0028 §Future evolution; the M7b broker-backed
/// impl per ADR-0033 / [readiness doc B2](../../../../../docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md)).
pub struct InProcessEventBus {
    handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
    /// CH-K8S-PREP P-4 — flipped by [`shutdown`]; checked at the top
    /// of every [`emit`] to drop late events.
    is_shutdown: AtomicBool,
    /// CH-K8S-PREP P-4 — incremented at emit-start, decremented at
    /// emit-end via [`EmitGuard`]. [`drain`] polls this until 0 or
    /// timeout. For the in-process impl handlers run inline so
    /// in-flight is bounded by the number of concurrent caller tasks.
    in_flight: AtomicUsize,
}

impl InProcessEventBus {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
            is_shutdown: AtomicBool::new(false),
            in_flight: AtomicUsize::new(0),
        }
    }

    /// Count of registered subscribers — used by boot-time wiring asserts
    /// (e.g. "exactly one TemplateAFireListener registered after AppState::new").
    pub fn handler_count(&self) -> usize {
        self.handlers.read().expect("event-bus lock poisoned").len()
    }
}

/// RAII guard for [`InProcessEventBus::in_flight`] — increments on
/// construction, decrements on drop. Safe under panic (Drop runs).
struct EmitGuard<'a> {
    counter: &'a AtomicUsize,
}

impl<'a> EmitGuard<'a> {
    fn new(counter: &'a AtomicUsize) -> Self {
        counter.fetch_add(1, Ordering::AcqRel);
        Self { counter }
    }
}

impl<'a> Drop for EmitGuard<'a> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Default for InProcessEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InProcessEventBus {
    fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        self.handlers
            .write()
            .expect("event-bus lock poisoned")
            .push(handler);
    }

    async fn emit(&self, event: DomainEvent) {
        // CH-K8S-PREP P-4 — drop late events post-shutdown. Audit
        // durability of dropped events is M7b scope (see
        // `deferred-from-ch-k8s-prep.md`).
        if self.is_shutdown.load(Ordering::Acquire) {
            return;
        }
        let _guard = EmitGuard::new(&self.in_flight);

        // Snapshot-and-release: clone the Arc list, drop the read guard
        // before awaiting any handler. Prevents deadlocks if a handler
        // re-subscribes (which would otherwise acquire the write lock
        // while we still hold the read guard).
        let snapshot: Vec<Arc<dyn EventHandler>> = {
            let guard = self.handlers.read().expect("event-bus lock poisoned");
            guard.clone()
        };
        for handler in snapshot {
            handler.on_event(&event).await;
        }
    }

    async fn shutdown(&self) {
        // Idempotent — Release ordering pairs with the Acquire load
        // in `emit`. Subsequent emits become no-ops.
        self.is_shutdown.store(true, Ordering::Release);
    }

    async fn drain(&self, timeout: Duration) -> Result<(), DrainError> {
        let deadline = Instant::now() + timeout;
        let poll = Duration::from_millis(50);
        loop {
            let in_flight = self.in_flight.load(Ordering::Acquire);
            if in_flight == 0 {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(DrainError {
                    remaining: in_flight,
                });
            }
            tokio::time::sleep(poll).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, AuditEventId, ProjectId};
    use chrono::Utc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Counter {
        hits: AtomicUsize,
    }

    #[async_trait]
    impl EventHandler for Counter {
        async fn on_event(&self, _event: &DomainEvent) {
            self.hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn sample_event() -> DomainEvent {
        DomainEvent::HasLeadEdgeCreated {
            project: ProjectId::new(),
            lead: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[tokio::test]
    async fn emit_delivers_to_every_subscriber_in_registration_order() {
        let bus = InProcessEventBus::new();
        let a = Arc::new(Counter {
            hits: AtomicUsize::new(0),
        });
        let b = Arc::new(Counter {
            hits: AtomicUsize::new(0),
        });
        bus.subscribe(a.clone());
        bus.subscribe(b.clone());
        bus.emit(sample_event()).await;
        assert_eq!(a.hits.load(Ordering::Relaxed), 1);
        assert_eq!(b.hits.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn emit_without_subscribers_is_noop() {
        let bus = InProcessEventBus::new();
        bus.emit(sample_event()).await;
        assert_eq!(bus.handler_count(), 0);
    }

    #[tokio::test]
    async fn bus_via_trait_object_dispatches() {
        let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
        let counter = Arc::new(Counter {
            hits: AtomicUsize::new(0),
        });
        bus.subscribe(counter.clone());
        bus.emit(sample_event()).await;
        assert_eq!(counter.hits.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn shutdown_drops_subsequent_emits_without_invoking_handlers() {
        // CH-K8S-PREP P-4 / ADR-0033 — proves the SIGTERM-handler's
        // `event_bus.shutdown()` call prevents late emits (e.g. from
        // a finalising recorder racing the shutdown) from invoking
        // listener code post-shutdown.
        let bus = InProcessEventBus::new();
        let counter = Arc::new(Counter {
            hits: AtomicUsize::new(0),
        });
        bus.subscribe(counter.clone());

        bus.emit(sample_event()).await;
        assert_eq!(
            counter.hits.load(Ordering::Relaxed),
            1,
            "pre-shutdown emit delivered"
        );

        bus.shutdown().await;
        bus.emit(sample_event()).await;
        bus.emit(sample_event()).await;
        assert_eq!(
            counter.hits.load(Ordering::Relaxed),
            1,
            "post-shutdown emits are dropped silently"
        );

        // Idempotent — second shutdown is a no-op.
        bus.shutdown().await;
    }

    #[tokio::test]
    async fn drain_returns_immediately_when_no_emits_are_in_flight() {
        let bus = InProcessEventBus::new();
        bus.drain(Duration::from_secs(1))
            .await
            .expect("idle bus drains immediately");
    }

    #[tokio::test]
    async fn drain_waits_for_in_flight_emits_to_complete() {
        // CH-K8S-PREP P-4 — proves the in_flight counter tracks
        // concurrent emits and `drain` waits for them. Uses a slow
        // handler to create an observable in-flight window.
        use tokio::sync::Notify;

        struct SlowHandler {
            entered: Arc<Notify>,
            release: Arc<Notify>,
        }

        #[async_trait]
        impl EventHandler for SlowHandler {
            async fn on_event(&self, _event: &DomainEvent) {
                self.entered.notify_one();
                self.release.notified().await;
            }
        }

        let bus = Arc::new(InProcessEventBus::new());
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        bus.subscribe(Arc::new(SlowHandler {
            entered: entered.clone(),
            release: release.clone(),
        }));

        // Spawn a slow emit; wait until the handler is mid-flight.
        let bus_for_emit = bus.clone();
        let emit_task = tokio::spawn(async move {
            bus_for_emit.emit(sample_event()).await;
        });
        entered.notified().await;

        // Mid-flight: drain with a short timeout must report
        // remaining = 1 (the slow handler is still awaiting release).
        let timeout_err = bus
            .drain(Duration::from_millis(100))
            .await
            .expect_err("drain times out while emit is mid-flight");
        assert_eq!(timeout_err.remaining, 1);

        // Release the handler, await emit completion, then drain
        // succeeds immediately.
        release.notify_one();
        emit_task.await.unwrap();
        bus.drain(Duration::from_secs(1))
            .await
            .expect("drain succeeds after in-flight emit completes");
    }

    #[tokio::test]
    async fn subscribe_during_emit_is_safe() {
        // Regression guard for the snapshot-and-release rule: if we
        // held the read lock across `.await`, a handler re-subscribing
        // would deadlock (writer waits forever). With snapshot-and-
        // release this works.
        let bus = Arc::new(InProcessEventBus::new());

        struct SelfSubscriber {
            bus: Arc<InProcessEventBus>,
            done: std::sync::atomic::AtomicBool,
        }
        #[async_trait]
        impl EventHandler for SelfSubscriber {
            async fn on_event(&self, _event: &DomainEvent) {
                if !self.done.swap(true, Ordering::Relaxed) {
                    let noop: Arc<dyn EventHandler> = Arc::new(());
                    self.bus.subscribe(noop);
                }
            }
        }

        bus.subscribe(Arc::new(SelfSubscriber {
            bus: bus.clone(),
            done: std::sync::atomic::AtomicBool::new(false),
        }));
        bus.emit(sample_event()).await;
        assert_eq!(bus.handler_count(), 2);
    }
}
