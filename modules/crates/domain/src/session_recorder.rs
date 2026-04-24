//! baby-phi's session-recorder wrap (M5/P3).
//!
//! Composes `phi_core::SessionRecorder` as an `inner` field and adds
//! SurrealDB persistence hooks + governance-event emission on session
//! lifecycle boundaries. See
//! [ADR-0029](../../../../docs/specs/v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md).
//!
//! ## Design
//!
//! - phi-core's recorder is the **source of truth** for Session /
//!   LoopRecord / Turn materialisation. Every `phi_core::AgentEvent`
//!   passes through it first.
//! - baby-phi is the **sink**: after a session reaches a terminal
//!   state (phi-core's `AgentEnd` on the last loop), the wrap reads
//!   phi-core's materialised view, writes it into SurrealDB via the
//!   M5/P2 repo surface (`persist_session` / `append_loop_record` /
//!   `append_turn`), and emits `DomainEvent::SessionStarted` /
//!   `SessionEnded` on the governance bus.
//! - Double-materialisation is avoided: phi-core does the
//!   state-machine, baby-phi only records the result.
//!
//! ## phi-core leverage
//!
//! Two direct imports — the two new imports that take M5's phi-core
//! import count from 17 (post-P2) to 19 (post-P3):
//! - `phi_core::session::recorder::SessionRecorder` — composed via
//!   the nested `inner` field.
//! - `phi_core::types::event::AgentEvent` — consumed by value on
//!   each `on_phi_core_event` call.
//!
//! M5/P4 extends this wrap with in-flight per-turn persistence (so
//! the "session running" panel on page 14 shows turns as they
//! materialise). P3 ships the terminal-state-only sink, which is the
//! piece the governance event bus + memory-extraction listener
//! need to function end-to-end.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

use phi_core::session::recorder::SessionRecorder as PhiCoreSessionRecorder;
use phi_core::types::event::AgentEvent as PhiCoreAgentEvent;

use crate::audit::AuditEmitter;
use crate::events::{DomainEvent, EventBus};
use crate::model::composites_m5::SessionDetail;
use crate::model::ids::{AgentId, AuditEventId, LoopId, OrgId, ProjectId, SessionId, TurnNodeId};
use crate::model::nodes::{LoopRecordNode, SessionGovernanceState, TurnNode};
use crate::repository::RepositoryResult;
use crate::Repository;

/// Launch context the recorder needs to map phi-core's string-keyed
/// session onto baby-phi's UUID-keyed governance rows.
///
/// The launch chain (M5/P4) builds this at session-launch time and
/// passes it to the recorder. At P3 the struct is used directly by
/// the recorder-wrap test (the real launch chain lands at P4).
#[derive(Debug, Clone)]
pub struct SessionLaunchContext {
    /// baby-phi governance session id (UUID).
    pub session_id: SessionId,
    /// phi-core's string session id — the recorder keys on this.
    pub phi_core_session_id: String,
    pub owning_org: OrgId,
    pub owning_project: ProjectId,
    pub started_by: AgentId,
    pub started_at: DateTime<Utc>,
    /// ID of the launch-chain-written first `LoopRecordNode`.
    /// At M5/P4 the launch chain pre-persists the Session row +
    /// the first loop inside a compound tx; the recorder's
    /// `finalise_and_persist` must attach turn rows to THIS
    /// loop id (not a fresh one) to honour the existing
    /// `session → loop` foreign-key relationship. When `None`,
    /// the recorder allocates its own LoopId (P3 / standalone
    /// path).
    pub first_loop_id: Option<LoopId>,
}

/// Reading from a completed session the wrap returns to the caller
/// on `finalise_and_persist`.
#[derive(Debug, Clone)]
pub struct RecorderFinalisation {
    pub session_detail: SessionDetail,
    pub session_started_event_id: AuditEventId,
    pub session_ended_event_id: AuditEventId,
}

/// Composes phi-core's `SessionRecorder` with baby-phi persistence +
/// governance-event emission. See the module doc for the full
/// design note.
pub struct BabyPhiSessionRecorder {
    inner: Arc<Mutex<PhiCoreSessionRecorder>>,
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    ctx: SessionLaunchContext,
    /// `true` once `SessionStarted` has been emitted for `ctx` —
    /// guards against double-emission on restarts / mid-stream
    /// re-entry.
    started_emitted: Arc<Mutex<bool>>,
}

impl BabyPhiSessionRecorder {
    pub fn new(
        repo: Arc<dyn Repository>,
        audit: Arc<dyn AuditEmitter>,
        event_bus: Arc<dyn EventBus>,
        ctx: SessionLaunchContext,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PhiCoreSessionRecorder::new(Default::default()))),
            repo,
            audit,
            event_bus,
            ctx,
            started_emitted: Arc::new(Mutex::new(false)),
        }
    }

    /// Accept a single phi-core event.
    ///
    /// - Routes the event to the composed recorder first (phi-core
    ///   owns state-machine semantics).
    /// - On the first `AgentStart` for this context's session, emits
    ///   `DomainEvent::SessionStarted`.
    pub async fn on_phi_core_event(&self, event: PhiCoreAgentEvent) {
        let is_agent_start_for_ctx = matches!(
            &event,
            PhiCoreAgentEvent::AgentStart { session_id, .. }
                if session_id == &self.ctx.phi_core_session_id
        );

        {
            let mut rec = self.inner.lock().expect("recorder lock poisoned");
            rec.on_event(event);
        }

        if is_agent_start_for_ctx {
            // Claim the `started_emitted` flag inside a narrow scope
            // so the `MutexGuard` drops before `.await`. Clippy's
            // `await_holding_lock` would otherwise fire — and hold-
            // across-await would deadlock any re-entrant subscriber.
            let should_emit = {
                let mut flag = self
                    .started_emitted
                    .lock()
                    .expect("started-emitted lock poisoned");
                if *flag {
                    false
                } else {
                    *flag = true;
                    true
                }
            };
            if should_emit {
                self.event_bus
                    .emit(DomainEvent::SessionStarted {
                        session_id: self.ctx.session_id,
                        agent_id: self.ctx.started_by,
                        project_id: self.ctx.owning_project,
                        started_at: self.ctx.started_at,
                        event_id: AuditEventId::new(),
                    })
                    .await;
            }
        }
    }

    /// Flush + drain the composed recorder.
    ///
    /// Wraps phi-core's Session / LoopRecord / Turn view into
    /// baby-phi rows; persists via the M5/P2 repo surface; emits
    /// `SessionEnded` on the governance bus.
    ///
    /// Returns the persisted `SessionDetail` so the caller can
    /// surface it to the user-facing HTTP response without a second
    /// repo read.
    pub async fn finalise_and_persist(&self) -> RepositoryResult<RecorderFinalisation> {
        let (session_snapshot, ended_at) = {
            let mut rec = self.inner.lock().expect("recorder lock poisoned");
            rec.flush();
            // `flush` moves every open session to the completed
            // collection; drain to claim ownership.
            let completed = rec.drain_completed();
            let session = completed
                .into_iter()
                .find(|s| s.session_id == self.ctx.phi_core_session_id)
                .ok_or_else(|| {
                    crate::repository::RepositoryError::Backend(format!(
                        "phi-core session {} not found in recorder",
                        self.ctx.phi_core_session_id
                    ))
                })?;
            let ended_at = session.last_active_at;
            (session, ended_at)
        };

        if session_snapshot.loops.is_empty() {
            return Err(crate::repository::RepositoryError::Backend(format!(
                "phi-core session {} completed with zero loops — nothing to persist",
                self.ctx.phi_core_session_id
            )));
        }

        // tokens_spent = sum of every loop's Usage.total_tokens.
        let tokens_spent: u64 = session_snapshot
            .loops
            .iter()
            .map(|l| l.usage.total_tokens)
            .sum();
        let turn_count: u32 = session_snapshot
            .loops
            .iter()
            .map(|l| l.turns.len() as u32)
            .sum();

        // Two persistence modes share this method:
        //
        // - **launch-chain-driven** (M5/P4): `ctx.first_loop_id` is
        //   `Some`. The launch chain already wrote the Session row
        //   + the first LoopRecordNode. This path only appends
        //   turns to the existing first loop + appends any extra
        //   loops + flips the session state via
        //   `mark_session_ended`.
        // - **standalone** (M5/P3 recorder-wrap test): launch chain
        //   absent, recorder writes everything via `persist_session`.

        let mut loop_nodes: Vec<LoopRecordNode> = Vec::with_capacity(session_snapshot.loops.len());
        let mut turns_for_each_loop: Vec<Vec<TurnNode>> = Vec::new();
        for (idx, phi_loop) in session_snapshot.loops.iter().enumerate() {
            let id = if idx == 0 {
                self.ctx.first_loop_id.unwrap_or_default()
            } else {
                LoopId::new()
            };
            let loop_node = LoopRecordNode {
                id,
                inner: phi_loop.clone(),
                session_id: self.ctx.session_id,
                loop_index: idx as u32,
            };
            let mut turn_nodes: Vec<TurnNode> = Vec::with_capacity(phi_loop.turns.len());
            for (tidx, phi_turn) in phi_loop.turns.iter().enumerate() {
                turn_nodes.push(TurnNode {
                    id: TurnNodeId::new(),
                    inner: phi_turn.clone(),
                    loop_id: loop_node.id,
                    turn_index: tidx as u32,
                });
            }
            turns_for_each_loop.push(turn_nodes);
            loop_nodes.push(loop_node);
        }

        if self.ctx.first_loop_id.is_some() {
            // launch-chain path: skip persist_session (row exists);
            // only append extra loops + all turns; flip state.
            for lnode in loop_nodes.iter().skip(1) {
                self.repo.append_loop_record(lnode).await?;
            }
            for turns in &turns_for_each_loop {
                for tnode in turns {
                    self.repo.append_turn(tnode).await?;
                }
            }
            self.repo
                .mark_session_ended(
                    self.ctx.session_id,
                    ended_at,
                    SessionGovernanceState::Completed,
                )
                .await?;
        } else {
            // standalone path (recorder-wrap test).
            let baby_session = crate::model::nodes::Session {
                id: self.ctx.session_id,
                inner: session_snapshot.clone(),
                owning_org: self.ctx.owning_org,
                owning_project: self.ctx.owning_project,
                started_by: self.ctx.started_by,
                governance_state: SessionGovernanceState::Completed,
                started_at: self.ctx.started_at,
                ended_at: Some(ended_at),
                tokens_spent,
            };
            self.repo
                .persist_session(&baby_session, &loop_nodes[0])
                .await?;
            for lnode in loop_nodes.iter().skip(1) {
                self.repo.append_loop_record(lnode).await?;
            }
            for turns in &turns_for_each_loop {
                for tnode in turns {
                    self.repo.append_turn(tnode).await?;
                }
            }
        }

        // Re-fetch the canonical SessionDetail from the repo.
        // For the launch-chain path this picks up the Session row
        // the launch wrote (with the now-flipped ended_at); for
        // the standalone path the recorder's own persist sees the
        // same row. Single source of truth.
        let session_detail = self
            .repo
            .fetch_session(self.ctx.session_id)
            .await?
            .ok_or_else(|| {
                crate::repository::RepositoryError::Backend(format!(
                    "session {} not readable after finalise",
                    self.ctx.session_id
                ))
            })?;
        // Unused local variable suppression — kept so a compile
        // error surfaces if the aggregation logic needs them again.
        let _ = (loop_nodes, turns_for_each_loop);

        let session_started_event_id = AuditEventId::new();
        let session_ended_event_id = AuditEventId::new();
        let duration_ms = (ended_at - self.ctx.started_at).num_milliseconds().max(0) as u64;

        self.event_bus
            .emit(DomainEvent::SessionEnded {
                session_id: self.ctx.session_id,
                agent_id: self.ctx.started_by,
                project_id: self.ctx.owning_project,
                ended_at,
                duration_ms,
                turn_count,
                tokens_spent,
                event_id: session_ended_event_id,
            })
            .await;

        // The `audit` sink is held but not written here at P3 — the
        // M5/P4 launch chain owns the SessionStarted / SessionEnded
        // audit-event emission (different audit event types from
        // the governance bus events above). Keeping the field here
        // avoids a wiring churn at P4.
        let _ = &self.audit;

        Ok(RecorderFinalisation {
            session_detail,
            session_started_event_id,
            session_ended_event_id,
        })
    }
}

// ===========================================================================
// Compile-time coercion witnesses (M5 discipline)
// ===========================================================================
//
// These fns never run; they exist so a rename in phi-core
// (`SessionRecorder` / `AgentEvent`) breaks the baby-phi build
// immediately. Mirrors the M3 `OrganizationDefaultsSnapshot`
// + M4 `AgentProfile` pattern.

#[allow(dead_code)]
fn _is_phi_core_session_recorder(_: &PhiCoreSessionRecorder) {}

#[allow(dead_code)]
fn _is_phi_core_agent_event(_: &PhiCoreAgentEvent) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::InProcessEventBus;
    use crate::in_memory::InMemoryRepository;
    use crate::model::nodes::SessionGovernanceState;

    use phi_core::session::model::{SessionFormation, SessionScope};
    use phi_core::types::event::{ContinuationKind, TurnTrigger};
    use phi_core::types::{AgentMessage, LlmMessage, Message, Usage};

    use async_trait::async_trait;

    struct NoopAudit;
    #[async_trait]
    impl AuditEmitter for NoopAudit {
        async fn emit(
            &self,
            _event: crate::audit::AuditEvent,
        ) -> crate::repository::RepositoryResult<()> {
            Ok(())
        }
    }

    struct CapturingBus {
        events: Mutex<Vec<DomainEvent>>,
    }
    #[async_trait::async_trait]
    impl crate::events::EventHandler for CapturingBus {
        async fn on_event(&self, event: &DomainEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    fn fixture() -> (
        Arc<dyn Repository>,
        Arc<CapturingBus>,
        Arc<dyn EventBus>,
        SessionLaunchContext,
    ) {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let capture = Arc::new(CapturingBus {
            events: Mutex::new(Vec::new()),
        });
        let bus = Arc::new(InProcessEventBus::new());
        bus.subscribe(capture.clone());
        let bus: Arc<dyn EventBus> = bus;

        let ctx = SessionLaunchContext {
            session_id: SessionId::new(),
            phi_core_session_id: "sess-p3-recorder-test".to_string(),
            owning_org: OrgId::new(),
            owning_project: ProjectId::new(),
            started_by: AgentId::new(),
            started_at: Utc::now() - chrono::Duration::seconds(5),
            first_loop_id: None,
        };
        (repo, capture, bus, ctx)
    }

    fn user_message(text: &str) -> AgentMessage {
        AgentMessage::Llm(LlmMessage::new(Message::user(text)))
    }

    #[tokio::test]
    async fn wrap_persists_session_loop_turn_and_emits_lifecycle_events() {
        let (repo, capture, bus, ctx) = fixture();
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAudit);
        let recorder = BabyPhiSessionRecorder::new(repo.clone(), audit, bus.clone(), ctx.clone());

        let agent_id = "agent-p3".to_string();
        let session_id = ctx.phi_core_session_id.clone();
        let loop_id = format!("{session_id}.cfg.0");
        let start_at = ctx.started_at;
        let mid_at = start_at + chrono::Duration::seconds(1);
        let end_at = start_at + chrono::Duration::seconds(2);

        recorder
            .on_phi_core_event(PhiCoreAgentEvent::AgentStart {
                agent_id: agent_id.clone(),
                session_id: session_id.clone(),
                loop_id: loop_id.clone(),
                parent_loop_id: None,
                continuation_kind: ContinuationKind::Initial,
                timestamp: start_at,
                metadata: None,
                config_snapshot: None,
            })
            .await;

        recorder
            .on_phi_core_event(PhiCoreAgentEvent::TurnStart {
                loop_id: loop_id.clone(),
                turn_index: 0,
                timestamp: start_at,
                triggered_by: TurnTrigger::User,
            })
            .await;

        recorder
            .on_phi_core_event(PhiCoreAgentEvent::TurnEnd {
                loop_id: loop_id.clone(),
                message: user_message("echo"),
                usage: Usage {
                    input: 10,
                    output: 20,
                    reasoning: 0,
                    cache_read: 0,
                    cache_write: 0,
                    total_tokens: 30,
                },
                timestamp: mid_at,
                tool_results: vec![],
            })
            .await;

        recorder
            .on_phi_core_event(PhiCoreAgentEvent::AgentEnd {
                loop_id: loop_id.clone(),
                messages: vec![],
                usage: Usage {
                    input: 10,
                    output: 20,
                    reasoning: 0,
                    cache_read: 0,
                    cache_write: 0,
                    total_tokens: 30,
                },
                timestamp: end_at,
                rejection: None,
            })
            .await;

        let outcome = recorder.finalise_and_persist().await.expect("finalise");

        // Session persisted + readable.
        let fetched = repo
            .fetch_session(ctx.session_id)
            .await
            .expect("fetch ok")
            .expect("session present");
        assert_eq!(fetched.session.id, ctx.session_id);
        assert_eq!(
            fetched.session.governance_state,
            SessionGovernanceState::Completed
        );
        assert_eq!(fetched.loops.len(), 1, "one loop persisted");
        assert_eq!(fetched.loops[0].loop_index, 0, "loop index zero-based");
        let turns = fetched
            .turns_by_loop
            .get(&fetched.loops[0].id)
            .expect("turns for loop");
        assert_eq!(turns.len(), 1, "one turn persisted");
        assert_eq!(turns[0].turn_index, 0);
        assert_eq!(outcome.session_detail.loops.len(), 1);

        // Governance events: exactly SessionStarted + SessionEnded.
        let captured = capture.events.lock().unwrap().clone();
        assert_eq!(
            captured.len(),
            2,
            "bus saw SessionStarted + SessionEnded only"
        );
        match &captured[0] {
            DomainEvent::SessionStarted {
                session_id: sid, ..
            } => assert_eq!(*sid, ctx.session_id),
            other => panic!("expected SessionStarted, got {other:?}"),
        }
        match &captured[1] {
            DomainEvent::SessionEnded {
                session_id: sid,
                turn_count,
                tokens_spent,
                ..
            } => {
                assert_eq!(*sid, ctx.session_id);
                assert_eq!(*turn_count, 1);
                assert_eq!(*tokens_spent, 30);
            }
            other => panic!("expected SessionEnded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn wrap_emits_session_started_exactly_once_even_on_re_entry() {
        let (repo, capture, bus, ctx) = fixture();
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAudit);
        let recorder = BabyPhiSessionRecorder::new(repo.clone(), audit, bus.clone(), ctx.clone());

        let loop_id_a = format!("{}.cfg.0", ctx.phi_core_session_id);
        let loop_id_b = format!("{}.cfg.1", ctx.phi_core_session_id);
        let now = Utc::now();

        for lid in [loop_id_a, loop_id_b] {
            recorder
                .on_phi_core_event(PhiCoreAgentEvent::AgentStart {
                    agent_id: "agent".to_string(),
                    session_id: ctx.phi_core_session_id.clone(),
                    loop_id: lid,
                    parent_loop_id: None,
                    continuation_kind: ContinuationKind::Initial,
                    timestamp: now,
                    metadata: None,
                    config_snapshot: None,
                })
                .await;
        }

        let captured = capture.events.lock().unwrap().clone();
        let started_count = captured
            .iter()
            .filter(|e| matches!(e, DomainEvent::SessionStarted { .. }))
            .count();
        assert_eq!(started_count, 1, "session_started de-duplicated");
    }

    #[tokio::test]
    async fn scope_default_is_ephemeral() {
        // Sanity: phi-core's Session defaults to `SessionScope::Ephemeral`.
        // Documenting via test so a phi-core default flip surfaces here
        // instead of at a page-14 acceptance test.
        let (repo, _capture, bus, ctx) = fixture();
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAudit);
        let recorder = BabyPhiSessionRecorder::new(repo, audit, bus, ctx.clone());
        recorder
            .on_phi_core_event(PhiCoreAgentEvent::AgentStart {
                agent_id: "a".to_string(),
                session_id: ctx.phi_core_session_id.clone(),
                loop_id: format!("{}.cfg.0", ctx.phi_core_session_id),
                parent_loop_id: None,
                continuation_kind: ContinuationKind::Initial,
                timestamp: ctx.started_at,
                metadata: None,
                config_snapshot: None,
            })
            .await;

        let inner = recorder.inner.lock().unwrap();
        let session = inner
            .get_session(&ctx.phi_core_session_id)
            .expect("session present");
        assert_eq!(session.scope, SessionScope::Ephemeral);
        match &session.formation {
            SessionFormation::FirstLoop { .. } => {}
            other => panic!("expected FirstLoop formation, got {other:?}"),
        }
    }
}
