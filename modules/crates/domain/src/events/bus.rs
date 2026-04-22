//! The [`EventBus`] trait + its [`InProcessEventBus`] default impl.
//!
//! See the parent module's doc-comment for rationale. Short version:
//! in-process pub/sub driven by `Arc<dyn EventHandler>` subscribers
//! registered at `AppState` boot. `emit` is called *after* the owning
//! compound-tx commits — if any handler errors, the tx is already
//! durable (fail-safe).

use async_trait::async_trait;
use std::sync::{Arc, RwLock};

use super::DomainEvent;

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
    async fn emit(&self, event: DomainEvent);
}

/// In-process default bus — a `Vec<Arc<dyn EventHandler>>` behind a
/// `std::sync::RwLock`. Suitable for both production (single process)
/// and tests (no extra infrastructure). Cross-process fan-out is a
/// future addition (ADR-0028 §Future evolution).
pub struct InProcessEventBus {
    handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
}

impl InProcessEventBus {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    /// Count of registered subscribers — used by boot-time wiring asserts
    /// (e.g. "exactly one TemplateAFireListener registered after AppState::new").
    pub fn handler_count(&self) -> usize {
        self.handlers.read().expect("event-bus lock poisoned").len()
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
