//! Domain event bus — phi governance-plane pub/sub.
//!
//! M4/P1 introduces an in-process event bus so the Template A firing
//! listener ([`listeners::TemplateAFireListener`], wired at M4/P3) can
//! react to [`DomainEvent::HasLeadEdgeCreated`] emissions from M4/P3's
//! `apply_project_creation` compound tx.
//!
//! ## Not the same as `phi_core::AgentEvent`
//!
//! phi-core's `AgentEvent` is **agent-loop telemetry** (MessageUpdate,
//! ToolExecutionEnd, etc.) — streamed to callers during a session. This
//! module's [`DomainEvent`] is **governance-plane edge-change
//! notifications** — emitted after a compound-tx commit to drive
//! reactive governance logic (template firing, alert dispatch, etc.).
//! The two surfaces are intentionally orthogonal per
//! [`phi/CLAUDE.md`](../../../../../CLAUDE.md) §Orthogonal surfaces.
//!
//! ## Fail-safe semantics
//!
//! Events emit **after** the owning compound tx is committed (not as
//! part of it). If a listener fails after emit, the tx is already
//! durable — listener errors are logged with the event id so operators
//! can replay; M4 does not auto-retry (retry machinery lands at M7b).
//!
//! ## phi-core leverage
//!
//! None. The bus, the event enum, and the listener trait are all pure
//! phi governance concepts.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::model::ids::{AgentId, AuditEventId, ProjectId};

pub mod bus;
pub mod listeners;

pub use bus::{EventBus, EventHandler, InProcessEventBus};
pub use listeners::{ActorResolver, AdoptionArResolver, TemplateAFireListener};

/// The set of reactive governance events phi emits post-commit.
///
/// M4 introduces a single variant ([`DomainEvent::HasLeadEdgeCreated`])
/// so Template A firing (s05) can react. Future variants land alongside
/// their owning milestones — the enum is intentionally extensible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEvent {
    /// Emitted after `apply_project_creation` commits a `HAS_LEAD` edge.
    /// The Template A fire-listener subscribes and calls
    /// `fire_grant_on_lead_assignment` (M4/P2) → persists the Grant →
    /// emits `TemplateAAdoptionFired` audit.
    HasLeadEdgeCreated {
        project: ProjectId,
        lead: AgentId,
        at: DateTime<Utc>,
        /// Stable event id so listener errors can be cross-referenced
        /// with the audit chain at incident time.
        event_id: AuditEventId,
    },
}

impl DomainEvent {
    /// Stable string form for log lines + listener dispatch.
    pub fn kind(&self) -> &'static str {
        match self {
            DomainEvent::HasLeadEdgeCreated { .. } => "has_lead_edge_created",
        }
    }

    /// Stable event id regardless of variant — lets `EventHandler`
    /// implementations key dedupe tables / retry logs.
    pub fn event_id(&self) -> AuditEventId {
        match self {
            DomainEvent::HasLeadEdgeCreated { event_id, .. } => *event_id,
        }
    }
}

/// Convenience alias — hot paths clone `Arc<dyn EventBus>` cheaply.
pub type SharedEventBus = Arc<dyn EventBus>;

/// Bounded no-op listener callback used when tests don't care about
/// reactive behaviour. Keeps the bus semantics identical between
/// production and test without requiring a stub impl per test file.
#[async_trait]
impl EventHandler for () {
    async fn on_event(&self, _event: &DomainEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::AuditEventId;

    #[test]
    fn domain_event_has_lead_edge_created_roundtrips() {
        let evt = DomainEvent::HasLeadEdgeCreated {
            project: ProjectId::new(),
            lead: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        let j = serde_json::to_string(&evt).expect("serialize");
        let back: DomainEvent = serde_json::from_str(&j).expect("deserialize");
        match back {
            DomainEvent::HasLeadEdgeCreated { .. } => {}
        }
    }

    #[test]
    fn domain_event_kind_is_stable() {
        let evt = DomainEvent::HasLeadEdgeCreated {
            project: ProjectId::new(),
            lead: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        assert_eq!(evt.kind(), "has_lead_edge_created");
    }
}
