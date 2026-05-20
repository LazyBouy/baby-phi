//! Domain event bus — phi governance-plane pub/sub.
//!
//! M4/P1 introduces an in-process event bus so the Template A firing
//! listener ([`listeners::TemplateAFireListener`], wired at M4/P3) can
//! react to [`DomainEvent::HasLeadEdgeCreated`] emissions from M4/P3's
//! `apply_project_creation` compound tx.
//!
//! M5/P3 extends the enum with eight additional variants:
//! - **Session lifecycle** — [`DomainEvent::SessionStarted`],
//!   [`DomainEvent::SessionEnded`], [`DomainEvent::SessionAborted`].
//!   The P8 memory-extraction listener reacts to `SessionEnded`.
//! - **Manager/supervisor edge signals** —
//!   [`DomainEvent::ManagesEdgeCreated`] (drives Template C firing),
//!   [`DomainEvent::HasAgentSupervisorEdgeCreated`] (drives Template D
//!   firing).
//! - **Agent catalog triggers** —
//!   [`DomainEvent::AgentCreated`], [`DomainEvent::AgentArchived`],
//!   [`DomainEvent::UsesProfileEdgeChanged`] (consumed by the P8
//!   agent-catalog listener).
//!
//! See `docs/specs/v0/implementation/m5/architecture/event-bus-m5-extensions.md`
//! for emit callsite + eventual-consistency semantics.
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
//! phi governance concepts. (The M5/P3
//! [`crate::session_recorder::BabyPhiSessionRecorder`] is a separate
//! module that wraps `phi_core::SessionRecorder` and emits these
//! governance events — the wrap lives there, not here.)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::model::ids::{AgentId, AuditEventId, MemoryId, NodeId, OrgId, ProjectId, SessionId};
use crate::model::nodes::{AgentKind, AgentRole, BlueprintId};

pub mod bus;
pub mod listeners;

pub use bus::{EventBus, EventHandler, InProcessEventBus};
pub use listeners::{
    record_system_agent_fire, ActorResolver, AdoptionArResolver, AgentCatalogListener,
    CatalogAuditMode, MemoryExtractionListener, TemplateAFireListener, TemplateCAdoptionArResolver,
    TemplateCFireListener, TemplateDAdoptionArResolver, TemplateDFireListener,
    MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME,
};

/// The set of reactive governance events phi emits post-commit.
///
/// M4 introduces a single variant ([`DomainEvent::HasLeadEdgeCreated`])
/// so Template A firing (s05) can react. M5/P3 adds eight more
/// variants — session lifecycle (3), manager/supervisor edge signals
/// (2), agent catalog triggers (3). Future variants land alongside
/// their owning milestones — the enum is intentionally extensible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEvent {
    /// Emitted after `apply_project_creation` commits a `HAS_LEAD` edge.
    /// The Template A fire-listener subscribes and calls
    /// `fire_grant_on_lead_assignment` (M4/P2; CH-15 / ADR-0054 §D54.3
    /// extends the return type to `Vec<Grant>` of length 2) →
    /// persists each Grant → emits one `template.a.grant_fired`
    /// audit per Grant.
    HasLeadEdgeCreated {
        project: ProjectId,
        lead: AgentId,
        at: DateTime<Utc>,
        /// Stable event id so listener errors can be cross-referenced
        /// with the audit chain at incident time.
        event_id: AuditEventId,
    },

    /// Emitted when a session's first `AgentStart` is observed and the
    /// baby-phi `Session` row + `RUNS_IN` edge + `UsesModel` edge have
    /// committed (M5/P4's session-launch compound tx).
    SessionStarted {
        session_id: SessionId,
        agent_id: AgentId,
        project_id: ProjectId,
        started_at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted when a session reaches a natural terminal state
    /// (phi-core's `AgentEnd` with no rejection, final turn). The P8
    /// memory-extraction listener reacts to this — runs the
    /// supervisor extractor loop + emits `MemoryExtracted` audit per
    /// candidate.
    SessionEnded {
        session_id: SessionId,
        agent_id: AgentId,
        project_id: ProjectId,
        ended_at: DateTime<Utc>,
        duration_ms: u64,
        turn_count: u32,
        tokens_spent: u64,
        event_id: AuditEventId,
    },

    /// Emitted when a session is terminated (operator action via
    /// `POST /sessions/:id/terminate`, or phi-core's `AgentEnd` with
    /// a non-null `rejection`). The catalog listener counts these
    /// separately from natural ends.
    SessionAborted {
        session_id: SessionId,
        reason: String,
        terminated_by: AgentId,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted after a compound tx commits a `MANAGES` edge
    /// (`agent:<manager> -> MANAGES -> agent:<subordinate>` within an
    /// org's reporting tree). The P3 `TemplateCFireListener`
    /// subscribes and issues the Template C `[read, inspect]` grant
    /// on `agent:<subordinate>` for the manager.
    ManagesEdgeCreated {
        org_id: OrgId,
        manager: AgentId,
        subordinate: AgentId,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted after a compound tx commits a `HAS_AGENT_SUPERVISOR`
    /// edge within a project scope. The P3 `TemplateDFireListener`
    /// subscribes and issues the Template D `[read, inspect]` grant
    /// on `project:<p>/agent:<supervisee>` for the supervisor.
    HasAgentSupervisorEdgeCreated {
        project_id: ProjectId,
        supervisor: AgentId,
        supervisee: AgentId,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted after M4/M5's `apply_agent_creation` compound tx
    /// commits a new `Agent` row. The P8 `AgentCatalogListener`
    /// subscribes + upserts the catalog entry.
    ///
    /// Note: the variant's [`AgentKind`] field is named
    /// `agent_kind` (not `kind`) because serde's `tag = "kind"`
    /// discriminator already claims the `kind` JSON key at the
    /// enum level.
    AgentCreated {
        agent_id: AgentId,
        owning_org: OrgId,
        agent_kind: AgentKind,
        role: Option<AgentRole>,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted when an agent is archived (soft-deleted / disabled).
    /// The catalog listener flips the entry's `active` flag.
    AgentArchived {
        agent_id: AgentId,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// Emitted when an agent's `USES_PROFILE` edge is rewritten
    /// (profile swap on the update-agent path). The catalog listener
    /// refreshes the cached profile snapshot on the catalog entry.
    ///
    /// Renamed from `HasProfileEdgeChanged` per CH-28 / ADR-0063 §D63.7
    /// (F2.b USER-LOCKED DIVERGENT). The enclosing enum carries
    /// `#[serde(tag = "kind", rename_all = "snake_case")]`, so this
    /// variant serializes with `"kind": "uses_profile_edge_changed"` by
    /// default. The `serde(rename = "uses_profile_edge_changed", alias
    /// = "has_profile_edge_changed")` attribute pins the wire-format
    /// `kind` value for new emissions while accepting the legacy
    /// `has_profile_edge_changed` form on deserialization (back-compat
    /// per ADR-0063 §D63.7).
    #[serde(
        rename = "uses_profile_edge_changed",
        alias = "has_profile_edge_changed"
    )]
    UsesProfileEdgeChanged {
        agent_id: AgentId,
        old_profile_id: Option<NodeId>,
        new_profile_id: NodeId,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// CH-16 / ADR-0038 §D38.5 — emitted when an LLM agent's
    /// [`crate::model::nodes::Identity`] row is reactively updated.
    /// CH-16 ships the variant + bus-dispatch arms; the first
    /// production emitter lands at CH-21 (memory-extraction listener
    /// body) when `MemoryExtracted` triggers a `witnessed` update.
    /// Future writers (skill-change, rating-received at M6+) plug in
    /// without touching CH-21's call site.
    IdentityUpdated {
        agent_id: AgentId,
        trigger: IdentityUpdateTrigger,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// CH-21 / ADR-0041 §D41.1 — emitted when the
    /// `MemoryExtractionListener` body extracts a Memory from a
    /// non-aborted ended session. The variant ships at CH-21 for
    /// forward-compat; v0 has no consumers (the audit log captures the
    /// reactive state via `platform.memory.extracted`). Future
    /// listeners (e.g., a memory-rebalancer in M6+) can subscribe via
    /// a `Weak<dyn EventBus>` design that breaks the bus-listener Arc
    /// cycle — out of scope for CH-21 (see ADR-0040 §D40.6).
    MemoryExtracted {
        memory_id: MemoryId,
        session_id: SessionId,
        owning_agent: AgentId,
        org_scope: Option<OrgId>,
        tags: Vec<String>,
        extracted_at: DateTime<Utc>,
        event_id: AuditEventId,
    },

    /// CH-28 / ADR-0063 §D63.1 + §D63.3 (F1.c USER-LOCKED DIVERGENT) —
    /// emitted after a Blueprint row is upserted (either template-tier
    /// or per-agent override-tier). Consumers reactively refresh the
    /// cached `profile_snapshot` on every `AgentCatalogEntry` row whose
    /// Agent points at the upserted Blueprint:
    ///   - For an override-tier row (`agent_id = Some(a)`): refresh the
    ///     catalog entry for Agent `a`.
    ///   - For a template-tier row (`agent_id = None`): refresh the
    ///     catalog entries of all Agents whose AgentProfile points at
    ///     the upserted Blueprint via `AGENT_PROFILE_USES_BLUEPRINT`.
    ///
    /// The listener arm lives at
    /// [`crate::events::listeners::AgentCatalogListener::on_event`].
    BlueprintUpserted {
        blueprint_id: BlueprintId,
        /// `None` → template-tier Blueprint (refresh catalogs of all
        /// agents pointing at the blueprint via
        /// `AGENT_PROFILE_USES_BLUEPRINT`); `Some(_)` → override-tier
        /// Blueprint (refresh catalog of that specific agent).
        agent_id: Option<AgentId>,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },
}

/// CH-16 / ADR-0038 §D38.5 — what triggered the identity update. Keyed
/// to the four reactive update sources concept-`agent.md` § "Materialization"
/// enumerates: session end, memory extraction, skill change, rating
/// received. Listeners can branch on this when a future use case (e.g.
/// embedding re-derivation, dashboard refresh) wants to react to a
/// subset of triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityUpdateTrigger {
    SessionEnded,
    MemoryExtracted,
    SkillChanged,
    RatingReceived,
}

impl DomainEvent {
    /// Stable string form for log lines + listener dispatch.
    pub fn kind(&self) -> &'static str {
        match self {
            DomainEvent::HasLeadEdgeCreated { .. } => "has_lead_edge_created",
            DomainEvent::SessionStarted { .. } => "session_started",
            DomainEvent::SessionEnded { .. } => "session_ended",
            DomainEvent::SessionAborted { .. } => "session_aborted",
            DomainEvent::ManagesEdgeCreated { .. } => "manages_edge_created",
            DomainEvent::HasAgentSupervisorEdgeCreated { .. } => {
                "has_agent_supervisor_edge_created"
            }
            DomainEvent::AgentCreated { .. } => "agent_created",
            DomainEvent::AgentArchived { .. } => "agent_archived",
            DomainEvent::UsesProfileEdgeChanged { .. } => "uses_profile_edge_changed",
            DomainEvent::IdentityUpdated { .. } => "identity_updated",
            DomainEvent::MemoryExtracted { .. } => "memory_extracted",
            DomainEvent::BlueprintUpserted { .. } => "blueprint_upserted",
        }
    }

    /// Stable event id regardless of variant — lets `EventHandler`
    /// implementations key dedupe tables / retry logs.
    pub fn event_id(&self) -> AuditEventId {
        match self {
            DomainEvent::HasLeadEdgeCreated { event_id, .. }
            | DomainEvent::SessionStarted { event_id, .. }
            | DomainEvent::SessionEnded { event_id, .. }
            | DomainEvent::SessionAborted { event_id, .. }
            | DomainEvent::ManagesEdgeCreated { event_id, .. }
            | DomainEvent::HasAgentSupervisorEdgeCreated { event_id, .. }
            | DomainEvent::AgentCreated { event_id, .. }
            | DomainEvent::AgentArchived { event_id, .. }
            | DomainEvent::UsesProfileEdgeChanged { event_id, .. }
            | DomainEvent::IdentityUpdated { event_id, .. }
            | DomainEvent::MemoryExtracted { event_id, .. }
            | DomainEvent::BlueprintUpserted { event_id, .. } => *event_id,
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
    use crate::model::ids::{AuditEventId, MemoryId, NodeId};

    fn roundtrip(evt: &DomainEvent) -> DomainEvent {
        let j = serde_json::to_string(evt).expect("serialize");
        serde_json::from_str(&j).expect("deserialize")
    }

    // ---- HasLeadEdgeCreated (M4 baseline) ---------------------------------

    #[test]
    fn domain_event_has_lead_edge_created_roundtrips() {
        let evt = DomainEvent::HasLeadEdgeCreated {
            project: ProjectId::new(),
            lead: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        match roundtrip(&evt) {
            DomainEvent::HasLeadEdgeCreated { .. } => {}
            other => panic!("unexpected variant: {other:?}"),
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

    // ---- SessionStarted ---------------------------------------------------

    fn sample_session_started() -> DomainEvent {
        DomainEvent::SessionStarted {
            session_id: SessionId::new(),
            agent_id: AgentId::new(),
            project_id: ProjectId::new(),
            started_at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn session_started_roundtrips() {
        match roundtrip(&sample_session_started()) {
            DomainEvent::SessionStarted { .. } => {}
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn session_started_kind_is_stable() {
        assert_eq!(sample_session_started().kind(), "session_started");
    }

    // ---- SessionEnded -----------------------------------------------------

    fn sample_session_ended() -> DomainEvent {
        DomainEvent::SessionEnded {
            session_id: SessionId::new(),
            agent_id: AgentId::new(),
            project_id: ProjectId::new(),
            ended_at: Utc::now(),
            duration_ms: 1_250,
            turn_count: 3,
            tokens_spent: 4_096,
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn session_ended_roundtrips() {
        match roundtrip(&sample_session_ended()) {
            DomainEvent::SessionEnded {
                duration_ms,
                turn_count,
                tokens_spent,
                ..
            } => {
                assert_eq!(duration_ms, 1_250);
                assert_eq!(turn_count, 3);
                assert_eq!(tokens_spent, 4_096);
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn session_ended_kind_is_stable() {
        assert_eq!(sample_session_ended().kind(), "session_ended");
    }

    // ---- SessionAborted ---------------------------------------------------

    fn sample_session_aborted() -> DomainEvent {
        DomainEvent::SessionAborted {
            session_id: SessionId::new(),
            reason: "operator terminate".to_string(),
            terminated_by: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn session_aborted_roundtrips() {
        match roundtrip(&sample_session_aborted()) {
            DomainEvent::SessionAborted { reason, .. } => {
                assert_eq!(reason, "operator terminate");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn session_aborted_kind_is_stable() {
        assert_eq!(sample_session_aborted().kind(), "session_aborted");
    }

    // ---- ManagesEdgeCreated ----------------------------------------------

    fn sample_manages_edge() -> DomainEvent {
        DomainEvent::ManagesEdgeCreated {
            org_id: OrgId::new(),
            manager: AgentId::new(),
            subordinate: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn manages_edge_created_roundtrips() {
        match roundtrip(&sample_manages_edge()) {
            DomainEvent::ManagesEdgeCreated { .. } => {}
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn manages_edge_created_kind_is_stable() {
        assert_eq!(sample_manages_edge().kind(), "manages_edge_created");
    }

    // ---- HasAgentSupervisorEdgeCreated -----------------------------------

    fn sample_has_agent_supervisor_edge() -> DomainEvent {
        DomainEvent::HasAgentSupervisorEdgeCreated {
            project_id: ProjectId::new(),
            supervisor: AgentId::new(),
            supervisee: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn has_agent_supervisor_edge_created_roundtrips() {
        match roundtrip(&sample_has_agent_supervisor_edge()) {
            DomainEvent::HasAgentSupervisorEdgeCreated { .. } => {}
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn has_agent_supervisor_edge_created_kind_is_stable() {
        assert_eq!(
            sample_has_agent_supervisor_edge().kind(),
            "has_agent_supervisor_edge_created"
        );
    }

    // ---- AgentCreated ----------------------------------------------------

    fn sample_agent_created() -> DomainEvent {
        DomainEvent::AgentCreated {
            agent_id: AgentId::new(),
            owning_org: OrgId::new(),
            agent_kind: AgentKind::Llm,
            role: Some(AgentRole::Intern),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn agent_created_roundtrips() {
        match roundtrip(&sample_agent_created()) {
            DomainEvent::AgentCreated {
                agent_kind, role, ..
            } => {
                assert_eq!(agent_kind, AgentKind::Llm);
                assert_eq!(role, Some(AgentRole::Intern));
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn agent_created_kind_is_stable() {
        assert_eq!(sample_agent_created().kind(), "agent_created");
    }

    // ---- AgentArchived ---------------------------------------------------

    fn sample_agent_archived() -> DomainEvent {
        DomainEvent::AgentArchived {
            agent_id: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn agent_archived_roundtrips() {
        match roundtrip(&sample_agent_archived()) {
            DomainEvent::AgentArchived { .. } => {}
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn agent_archived_kind_is_stable() {
        assert_eq!(sample_agent_archived().kind(), "agent_archived");
    }

    // ---- UsesProfileEdgeChanged ------------------------------------------

    fn sample_uses_profile_edge_changed() -> DomainEvent {
        DomainEvent::UsesProfileEdgeChanged {
            agent_id: AgentId::new(),
            old_profile_id: Some(NodeId::new()),
            new_profile_id: NodeId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn uses_profile_edge_changed_roundtrips() {
        match roundtrip(&sample_uses_profile_edge_changed()) {
            DomainEvent::UsesProfileEdgeChanged { old_profile_id, .. } => {
                assert!(old_profile_id.is_some());
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn uses_profile_edge_changed_kind_is_stable() {
        assert_eq!(
            sample_uses_profile_edge_changed().kind(),
            "uses_profile_edge_changed"
        );
    }

    // ---- Serde back-compat alias: legacy `has_profile_edge_changed` ------
    // deserializes into the renamed variant per CH-28 / ADR-0063 §D63.7
    // (F2.b USER-LOCKED DIVERGENT). Validates the `#[serde(alias =
    // "has_profile_edge_changed")]` decorator absorbs already-serialized
    // rows produced before this chunk (e.g., audit-event payloads, in-flight
    // wire-format snapshots) without panicking. Pairs with the rename arm's
    // forward-tag pinning at `"uses_profile_edge_changed"`.
    #[test]
    fn uses_profile_edge_changed_deserializes_from_legacy_has_profile_via_alias() {
        let agent_id = AgentId::new();
        let old_profile_id = NodeId::new();
        let new_profile_id = NodeId::new();
        let event_id = AuditEventId::new();
        let at = Utc::now();

        // Hand-build the legacy JSON wire-form using the pre-rename
        // `"kind": "has_profile_edge_changed"` tag.
        let legacy_json = serde_json::json!({
            "kind": "has_profile_edge_changed",
            "agent_id": agent_id,
            "old_profile_id": old_profile_id,
            "new_profile_id": new_profile_id,
            "at": at,
            "event_id": event_id,
        });

        let parsed: DomainEvent = serde_json::from_value(legacy_json)
            .expect("legacy `has_profile_edge_changed` wire-form deserializes via serde alias");

        match parsed {
            DomainEvent::UsesProfileEdgeChanged {
                agent_id: a,
                old_profile_id: opi,
                new_profile_id: npi,
                event_id: eid,
                ..
            } => {
                assert_eq!(a, agent_id);
                assert_eq!(opi, Some(old_profile_id));
                assert_eq!(npi, new_profile_id);
                assert_eq!(eid, event_id);
            }
            other => panic!(
                "legacy `has_profile_edge_changed` payload deserialized into the \
                 wrong variant: {other:?}"
            ),
        }
    }

    // ---- event_id() parity across all variants ---------------------------

    #[test]
    fn event_id_accessor_matches_emitted_value_for_every_variant() {
        let fresh = AuditEventId::new();
        let all: Vec<DomainEvent> = vec![
            DomainEvent::HasLeadEdgeCreated {
                project: ProjectId::new(),
                lead: AgentId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::SessionStarted {
                session_id: SessionId::new(),
                agent_id: AgentId::new(),
                project_id: ProjectId::new(),
                started_at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::SessionEnded {
                session_id: SessionId::new(),
                agent_id: AgentId::new(),
                project_id: ProjectId::new(),
                ended_at: Utc::now(),
                duration_ms: 0,
                turn_count: 0,
                tokens_spent: 0,
                event_id: fresh,
            },
            DomainEvent::SessionAborted {
                session_id: SessionId::new(),
                reason: String::new(),
                terminated_by: AgentId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::ManagesEdgeCreated {
                org_id: OrgId::new(),
                manager: AgentId::new(),
                subordinate: AgentId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::HasAgentSupervisorEdgeCreated {
                project_id: ProjectId::new(),
                supervisor: AgentId::new(),
                supervisee: AgentId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::AgentCreated {
                agent_id: AgentId::new(),
                owning_org: OrgId::new(),
                agent_kind: AgentKind::Human,
                role: None,
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::AgentArchived {
                agent_id: AgentId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::UsesProfileEdgeChanged {
                agent_id: AgentId::new(),
                old_profile_id: None,
                new_profile_id: NodeId::new(),
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::IdentityUpdated {
                agent_id: AgentId::new(),
                trigger: IdentityUpdateTrigger::SessionEnded,
                at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::MemoryExtracted {
                memory_id: MemoryId::new(),
                session_id: SessionId::new(),
                owning_agent: AgentId::new(),
                org_scope: Some(OrgId::new()),
                tags: vec![],
                extracted_at: Utc::now(),
                event_id: fresh,
            },
            DomainEvent::BlueprintUpserted {
                blueprint_id: BlueprintId::new(),
                agent_id: None,
                at: Utc::now(),
                event_id: fresh,
            },
        ];
        for evt in &all {
            assert_eq!(
                evt.event_id(),
                fresh,
                "event_id accessor mismatch for variant {}",
                evt.kind()
            );
        }
    }

    // ---- CH-16 / ADR-0038 §D38.5 — IdentityUpdated event ------------------

    #[test]
    fn identity_updated_roundtrips() {
        let evt = DomainEvent::IdentityUpdated {
            agent_id: AgentId::new(),
            trigger: IdentityUpdateTrigger::MemoryExtracted,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        match roundtrip(&evt) {
            DomainEvent::IdentityUpdated { trigger, .. } => {
                assert_eq!(trigger, IdentityUpdateTrigger::MemoryExtracted);
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn identity_updated_kind_is_stable() {
        let evt = DomainEvent::IdentityUpdated {
            agent_id: AgentId::new(),
            trigger: IdentityUpdateTrigger::SkillChanged,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        assert_eq!(evt.kind(), "identity_updated");
    }

    // ---- CH-21 / ADR-0041 §D41.1 — MemoryExtracted event ------------------

    fn sample_memory_extracted() -> DomainEvent {
        DomainEvent::MemoryExtracted {
            memory_id: MemoryId::new(),
            session_id: SessionId::new(),
            owning_agent: AgentId::new(),
            org_scope: Some(OrgId::new()),
            tags: vec!["agent:abc".into(), "session:def".into()],
            extracted_at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[test]
    fn memory_extracted_roundtrips() {
        let evt = sample_memory_extracted();
        match roundtrip(&evt) {
            DomainEvent::MemoryExtracted {
                tags, org_scope, ..
            } => {
                assert_eq!(tags.len(), 2);
                assert!(tags.iter().any(|t| t.starts_with("agent:")));
                assert!(org_scope.is_some());
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn memory_extracted_kind_is_stable() {
        assert_eq!(sample_memory_extracted().kind(), "memory_extracted");
    }

    #[test]
    fn identity_update_trigger_serde_round_trip_for_all_variants() {
        for variant in [
            IdentityUpdateTrigger::SessionEnded,
            IdentityUpdateTrigger::MemoryExtracted,
            IdentityUpdateTrigger::SkillChanged,
            IdentityUpdateTrigger::RatingReceived,
        ] {
            let j = serde_json::to_string(&variant).expect("serialize");
            let back: IdentityUpdateTrigger = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, variant);
        }
    }

    // ---- CH-28 / ADR-0063 §D63.1 + §D63.3 — BlueprintUpserted event -----

    #[test]
    fn blueprint_upserted_event_roundtrips() {
        // F1.c USER-LOCKED — verify serde roundtrip preserves all fields
        // (blueprint_id, agent_id, at, event_id) for BOTH template-tier
        // (agent_id None) AND override-tier (agent_id Some) shapes.
        let template_evt = DomainEvent::BlueprintUpserted {
            blueprint_id: BlueprintId::new(),
            agent_id: None,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        match roundtrip(&template_evt) {
            DomainEvent::BlueprintUpserted {
                agent_id,
                blueprint_id,
                ..
            } => {
                assert!(
                    agent_id.is_none(),
                    "template-tier wire form preserves agent_id None"
                );
                if let DomainEvent::BlueprintUpserted {
                    blueprint_id: orig, ..
                } = template_evt
                {
                    assert_eq!(blueprint_id, orig, "blueprint_id preserved on roundtrip");
                }
            }
            other => panic!("unexpected variant: {other:?}"),
        }

        let agent = AgentId::new();
        let override_evt = DomainEvent::BlueprintUpserted {
            blueprint_id: BlueprintId::new(),
            agent_id: Some(agent),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        match roundtrip(&override_evt) {
            DomainEvent::BlueprintUpserted { agent_id, .. } => {
                assert_eq!(
                    agent_id,
                    Some(agent),
                    "override-tier wire form preserves agent_id Some(_)"
                );
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn blueprint_upserted_event_kind_is_stable() {
        // Wire-format stability — `kind()` is the listener-dispatch key.
        let evt = DomainEvent::BlueprintUpserted {
            blueprint_id: BlueprintId::new(),
            agent_id: None,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        assert_eq!(evt.kind(), "blueprint_upserted");
    }
}
