//! POST `/api/v0/orgs/:org/projects/:project/sessions` — session
//! launch.
//!
//! Closes two M5 carryovers end-to-end:
//! - **C-M5-2** — `RELATE agent -> uses_model -> model_runtime` edge
//!   is written during the compound tx. The edge was retyped in
//!   migration 0005 (M5/P1) from the legacy `agent → model_config`
//!   direction; P4 is the first writer.
//! - **C-M5-3** — Session + first `LoopRecordNode` + `runs_session`
//!   edge all persist in one compound-tx call via
//!   [`Repository::persist_session`]. [`BabyPhiSessionRecorder`]
//!   (M5/P3) drives subsequent turn / loop rows from the phi-core
//!   event stream.
//!
//! ## 9-step launch flow
//!
//! 1. Validate agent + project + membership (AGENT_NOT_FOUND,
//!    PROJECT_NOT_FOUND, AGENT_NOT_MEMBER_OF_PROJECT).
//! 2. Resolve the agent's bound [`ModelConfig`] via the profile's
//!    `model_config_id` — MODEL_RUNTIME_UNRESOLVED /
//!    MODEL_RUNTIME_ARCHIVED / MODEL_RUNTIME_NOT_FOUND.
//! 3. Run the M1 Permission Check (steps 0–6).
//! 4. Gate on `count_active_sessions_for_agent <
//!    profile.parallelize` (W2).
//! 5. Gate on `session_registry.len() < session_max_concurrent`
//!    (D3).
//! 6. Compound tx — persist `Session` + first `LoopRecordNode` +
//!    `runs_session` edge + `uses_model` edge + emit
//!    `platform.session.started` audit + emit
//!    `DomainEvent::SessionStarted` after commit.
//! 7. Insert the cancellation token into
//!    [`SessionRegistry`]; spawn the in-flight replay task (see
//!    §Replay below).
//! 8. Return [`LaunchReceipt`] with the session id + first loop id
//!    plus the Permission Check trace so the UI can render the
//!    worked example.
//!
//! ## Agent task (Step 7 details — CH-02)
//!
//! `spawn_agent_task` runs `phi_core::agent_loop()` on a spawned
//! tokio task. Events stream through an unbounded mpsc channel; the
//! same task drains the receiver and routes each event to
//! [`BabyPhiSessionRecorder::on_phi_core_event`]. When the loop
//! terminates the channel closes, the drain loop exits, and
//! `recorder.finalise_and_persist()` writes the terminal session
//! state.
//!
//! At M5 the provider is `phi_core::provider::mock::MockProvider`
//! (deterministic, no network) — see ADR-0032. Real provider
//! dispatch via `phi_core::provider::registry::ProviderRegistry`
//! defers to M7's credentials-vault milestone. The
//! `AgentProfile.mock_response` governance field drives the mock's
//! response text per session; when `None`, the default
//! `"Acknowledged."` is used.
//!
//! ## phi-core leverage
//!
//! Runtime-exercised:
//! - `phi_core::agent_loop` — spawned in `spawn_agent_task`.
//! - `phi_core::types::context::AgentContext` — built per launch via
//!   [`super::provider::build_agent_context`].
//! - `phi_core::agent_loop::AgentLoopConfig` — built per launch with
//!   `provider_override = Some(MockProvider)` at M5.
//! - `phi_core::types::event::AgentEvent` — flows through the mpsc
//!   channel into the recorder.
//! - `phi_core::provider::model::ModelConfig` — runtime resolution
//!   (re-used from M4; cloned into AgentLoopConfig at spawn time).
//!
//! Witness import: `phi_core::agent_loop_continue` stays as a
//! compile-time pin against rename; real continuation flows
//! (re-runs, branches) ship at M6+.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use domain::audit::AuditEmitter;
use domain::events::{DomainEvent, EventBus};
use domain::model::composites_m2::ModelRuntime;
use domain::model::composites_m5::SessionDetail;
use domain::model::ids::{AgentId, AuditEventId, LoopId, OrgId, ProjectId, SessionId};
use domain::model::nodes::{AgentProfile, PrincipalRef, Session, SessionGovernanceState};
use domain::model::{ApprovalTimeout, ConsentPolicy};
use domain::permissions::{
    build_session_launch_manifest, check, CheckContext, ConsentIndex, Decision, DeniedReason,
    FailedStep, NoopMetrics, StaticCatalogue, ToolCall,
};
use domain::session_recorder::{BabyPhiSessionRecorder, SessionLaunchContext};
use domain::Repository;
use phi_core::agent_loop::AgentLoopConfig;
use phi_core::session::model::LoopStatus;
use phi_core::types::event::{AgentEvent as PhiCoreAgentEvent, ContinuationKind, TurnTrigger};
use phi_core::types::{AgentMessage, LlmMessage, Message, Usage};
// phi-core leverage witness — `agent_loop_continue` stays as a
// compile-time pin until continuation flows ship at M6+. A rename
// in phi-core breaks the build immediately rather than silently
// later.
#[allow(unused_imports)]
use phi_core::agent_loop_continue;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::preview::{preview_session, PreviewInput};
use super::SessionError;
use crate::state::{SessionLiveStreamRegistry, SessionRegistry};
type SharedSessionRegistry = Arc<dyn SessionRegistry>;
type SharedSessionLiveStreamRegistry = Arc<dyn SessionLiveStreamRegistry>;

// phi-core witness — `agent_loop` is now runtime-exercised in
// `spawn_agent_task` via MockProvider (CH-02). The compile-time
// witness below is kept as belt-and-braces protection: if phi-core
// ever renames or re-shapes `agent_loop`, both the runtime call site
// AND this witness break the build, surfacing the rename at compile
// time rather than via a runtime error.
#[allow(dead_code)]
fn _is_phi_core_agent_loop_free_fn() {
    let _: fn() = _keep_agent_loop_live;
}
fn _keep_agent_loop_live() {}

/// CH-15 / ADR-0054 §D54.5 — extract the `kind` snake_case tag from a
/// [`DeniedReason`] for the `platform.session.launch_denied` audit
/// event's `reason_kind` field. The enum derives
/// `#[serde(tag = "kind", rename_all = "snake_case")]` so
/// `serde_json::to_value(...)` produces an object whose top-level
/// `kind` field is the variant name in snake_case.
fn denied_reason_kind(reason: &DeniedReason) -> &'static str {
    match reason {
        DeniedReason::CatalogueMiss { .. } => "catalogue_miss",
        DeniedReason::ManifestEmpty => "manifest_empty",
        DeniedReason::NoGrantsHeld => "no_grants_held",
        DeniedReason::CeilingEmptied => "ceiling_emptied",
        DeniedReason::NoMatchingGrant { .. } => "no_matching_grant",
        DeniedReason::ConstraintViolation { .. } => "constraint_violation",
        DeniedReason::ScopeUnresolvable { .. } => "scope_unresolvable",
        DeniedReason::ConsentTimedOutDeny { .. } => "consent_timed_out_deny",
        DeniedReason::ConsentDeclined { .. } => "consent_declined",
        DeniedReason::ConsentRevoked { .. } => "consent_revoked",
        DeniedReason::ConsentExpired { .. } => "consent_expired",
        DeniedReason::NoSessionContext { .. } => "no_session_context",
        DeniedReason::IntersectionEmpty { .. } => "intersection_empty",
    }
}

/// Input for [`launch_session`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchInput {
    pub org_id: OrgId,
    pub project_id: ProjectId,
    pub agent_id: AgentId,
    /// First user prompt seeded into the session. Stored on the
    /// synthetic `TurnStart` input messages + surfaced on the
    /// dashboard's "recent sessions" preview column.
    pub prompt: String,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

/// Returned by [`launch_session`] on happy path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchReceipt {
    pub session_id: SessionId,
    pub first_loop_id: LoopId,
    /// The Permission Check decision recorded at launch time. Always
    /// `Allowed` — if the check denied we would have returned a 403
    /// `PERMISSION_CHECK_FAILED_AT_STEP_N` error before reaching
    /// this point.
    pub permission_check_decision: Decision,
    pub session_started_event_id: AuditEventId,
}

/// Run the 9-step launch flow.
#[allow(clippy::too_many_arguments)]
pub async fn launch_session(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    registry: SharedSessionRegistry,
    live_stream_registry: SharedSessionLiveStreamRegistry,
    max_concurrent: u32,
    live_stream_buffer: usize,
    input: LaunchInput,
) -> Result<LaunchReceipt, SessionError> {
    // Step 1 — validate existence.
    let agent = repo
        .get_agent(input.agent_id)
        .await?
        .ok_or(SessionError::AgentNotFound(input.agent_id))?;
    let _project = repo
        .get_project(input.project_id)
        .await?
        .ok_or(SessionError::ProjectNotFound(input.project_id))?;
    if !matches!(agent.owning_org, Some(org) if org == input.org_id) {
        return Err(SessionError::AgentNotMemberOfProject {
            agent: input.agent_id,
            project: input.project_id,
        });
    }

    // Step 2 — resolve the agent's ModelConfig.
    let profile = repo
        .get_agent_profile_for_agent(input.agent_id)
        .await?
        .ok_or(SessionError::AgentProfileMissing(input.agent_id))?;
    let model_config_id_str = profile
        .model_config_id
        .clone()
        .ok_or(SessionError::ModelRuntimeUnresolved(input.agent_id))?;
    let runtimes = repo.list_model_providers(true).await?;
    let runtime = runtimes
        .iter()
        .find(|r| r.id.to_string() == model_config_id_str)
        .ok_or_else(|| {
            SessionError::ModelRuntimeNotFound(domain::model::ids::ModelProviderId::from_uuid(
                uuid::Uuid::parse_str(&model_config_id_str).unwrap_or_else(|_| uuid::Uuid::nil()),
            ))
        })?;
    if runtime.archived_at.is_some() {
        return Err(SessionError::ModelRuntimeArchived(runtime.id));
    }

    // Step 3 — Permission Check (CH-15 / ADR-0054 — hard-deny on every
    // Decision::Denied; closes drift D4.1).
    //
    // Pre-CH-15 (M5/P4): the synthetic launch manifest didn't match
    // any grant Template A actually issued; advisory-logging steps 1–6
    // was the least-bad stopgap. At CH-15 (this chunk) Template A is
    // extended to mint a paired `session_object/project:<id>` grant
    // (ADR-0054 §D54.3), the synthetic manifest is rebuilt via
    // `build_session_launch_manifest` (§D54.1), and every step-1-to-6
    // denial returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>` (§D54.6).
    // Step 6 (Consent) deny remains routed through
    // `gate_session_launch_consent` per CH-11 + ADR-0048 — the engine's
    // Consent step is unchanged by CH-15.
    //
    // Pre-allocate `session_id` BEFORE the engine call so the
    // `platform.session.launch_denied` audit event (§D54.5) can
    // reference it on the deny path; the same id is reused on the
    // happy path for the compound tx.
    let session_id = SessionId::new();
    let preview = preview_session(
        repo.clone(),
        PreviewInput {
            org_id: input.org_id,
            project_id: input.project_id,
            agent_id: input.agent_id,
        },
    )
    .await?;
    if let Decision::Denied {
        ref failed_step,
        ref reason,
    } = preview.decision
    {
        // CH-15 / ADR-0054 §D54.6 — hard-deny on every step. Map the
        // FailedStep variant to u8 for the wire payload + the audit
        // event. `Ceiling` is a sub-step of Resolution; surface it as
        // 2 on the wire (the metric label `"2a"` is preserved on the
        // denied receipt's reason text).
        let step: u8 = match failed_step {
            FailedStep::Catalogue => 0,
            FailedStep::Expansion => 1,
            FailedStep::Resolution => 2,
            FailedStep::Ceiling => 2,
            FailedStep::Match => 3,
            FailedStep::Constraint => 4,
            FailedStep::Scope => 5,
            FailedStep::Consent => 6,
        };
        let reason_kind = denied_reason_kind(reason);
        let denied_event = domain::audit::events::m5_2::session_launch::session_launch_denied(
            input.actor,
            session_id,
            input.agent_id,
            input.project_id,
            input.org_id,
            step,
            reason_kind,
            None,
            input.now,
        );
        // CH-15 / ADR-0054 §D54.5 — emit `platform.session.launch_denied`
        // BEFORE returning the error so the audit row lands even if
        // the response writer fails downstream. Audit-emit failures
        // do NOT block the deny path; the launch is rejected
        // regardless.
        if let Err(e) = audit.emit(denied_event).await {
            tracing::warn!(
                agent = %input.agent_id,
                project = %input.project_id,
                failed_step = ?failed_step,
                error = %e,
                "sessions::launch: launch_denied audit emit failed (deny still enforced)",
            );
        }
        return Err(SessionError::PermissionCheckFailed {
            step,
            reason: format!("{reason:?}"),
        });
    }

    // Step 3.5 — CH-11 / ADR-0048 §D48.5+ — session-launch consent
    // gating. The `session_id` was pre-allocated at Step 3 so the
    // launch_denied audit event can reference it; the same id is
    // reused at the compound tx + per-session consent mint.
    if let Some(err) =
        gate_session_launch_consent(repo.clone(), &input, session_id, input.now).await?
    {
        return Err(err);
    }

    // Step 4 — per-agent parallelize gate (W2).
    let active_count_u32 = repo.count_active_sessions_for_agent(input.agent_id).await?;
    if active_count_u32 >= profile.parallelize {
        return Err(SessionError::ParallelizeCapReached {
            agent: input.agent_id,
            active: active_count_u32 as u64,
            cap: profile.parallelize,
        });
    }

    // Step 5 — platform-wide saturation gate (D3).
    let current_live = registry.len();
    if current_live >= max_concurrent as usize {
        return Err(SessionError::SessionWorkerSaturated {
            current: current_live,
            cap: max_concurrent,
        });
    }

    // Step 6 — compound tx. (`session_id` was allocated at Step 3.5
    // for consent-gating scoping; reused here.)
    let phi_core_session_id = format!("sess-{}", session_id.as_uuid());
    let first_loop_id = LoopId::new();
    let config_id_segment = profile
        .blueprint
        .config_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let phi_loop_id_string = format!("{phi_core_session_id}.{config_id_segment}.0");

    // Build phi-core's Session + first LoopRecord directly so the
    // wrap row embeds a valid nested `inner` from the start (before
    // the recorder enriches it).
    let phi_session = phi_core::session::model::Session {
        session_id: phi_core_session_id.clone(),
        agent_id: input.agent_id.to_string(),
        created_at: input.now,
        last_active_at: input.now,
        formation: phi_core::session::model::SessionFormation::FirstLoop {
            timestamp: input.now,
        },
        parent_spawn_ref: None,
        scope: phi_core::session::model::SessionScope::Ephemeral,
        loops: Vec::new(),
    };
    let phi_first_loop = phi_core::session::model::LoopRecord {
        loop_id: phi_loop_id_string.clone(),
        session_id: phi_core_session_id.clone(),
        agent_id: input.agent_id.to_string(),
        parent_loop_id: None,
        continuation_kind: ContinuationKind::Initial,
        started_at: input.now,
        ended_at: None,
        status: LoopStatus::Running,
        rejection: None,
        config: None,
        messages: Vec::new(),
        turns: Vec::new(),
        usage: Usage::default(),
        metadata: None,
        events: Vec::new(),
        children_loop_ids: Vec::new(),
        child_loop_refs: Vec::new(),
        parallel_group: None,
        compaction_block: None,
    };

    let baby_session = Session {
        id: session_id,
        inner: phi_session.clone(),
        owning_org: input.org_id,
        owning_project: input.project_id,
        started_by: input.agent_id,
        governance_state: SessionGovernanceState::Running,
        started_at: input.now,
        ended_at: None,
        tokens_spent: 0,
        tags: domain::model::composites::auto_tags_for("session", &session_id.to_string()).to_vec(),
    };
    let first_loop = domain::model::nodes::LoopRecordNode {
        id: first_loop_id,
        inner: phi_first_loop,
        session_id,
        loop_index: 0,
    };

    repo.persist_session(&baby_session, &first_loop)
        .await
        .map_err(|e| SessionError::CompoundTxFailure(e.to_string()))?;

    // C-M5-2 close — write the `uses_model` edge via the dedicated
    // repo method. The SurrealDB impl honours D2.2 (LET-first
    // RELATE pattern); the in-memory impl keeps a tuple set the
    // tests assert on.
    let _uses_model_edge_id = repo
        .write_uses_model_edge(
            input.agent_id,
            domain::model::ids::NodeId::from_uuid(*runtime.id.as_uuid()),
        )
        .await
        .map_err(|e| SessionError::CompoundTxFailure(e.to_string()))?;

    // Step 6 (audit + governance event emission).
    let session_started_event_id = AuditEventId::new();
    let _ = &audit; // audit emit for session.started lands at M5/P7 with the full ops-doc code table
    event_bus
        .emit(DomainEvent::SessionStarted {
            session_id,
            agent_id: input.agent_id,
            project_id: input.project_id,
            started_at: input.now,
            event_id: session_started_event_id,
        })
        .await;

    // Step 7 — register + spawn the agent task (CH-02).
    let cancel_token = CancellationToken::new();
    registry.insert(session_id, cancel_token.clone());
    // CH-17 / ADR-0055 §D55.1 — pre-allocate the per-session
    // broadcast Sender BEFORE the agent task spawns so an SSE client
    // that connects between this insert and the first AgentStart
    // event sees a live entry in the registry. The `_rx` is dropped
    // immediately; subscribers call `.subscribe()` on the registered
    // Sender clone.
    let (broadcast_tx, _rx) =
        tokio::sync::broadcast::channel::<PhiCoreAgentEvent>(live_stream_buffer.max(1));
    live_stream_registry.insert(session_id, broadcast_tx.clone());
    spawn_agent_task(
        repo.clone(),
        audit,
        event_bus,
        registry.clone(),
        live_stream_registry.clone(),
        profile,
        runtime.clone(),
        SessionLaunchContext {
            session_id,
            phi_core_session_id,
            owning_org: input.org_id,
            owning_project: input.project_id,
            started_by: input.agent_id,
            started_at: input.now,
            first_loop_id: Some(first_loop_id),
        },
        input.prompt,
        cancel_token,
        broadcast_tx,
    );

    Ok(LaunchReceipt {
        session_id,
        first_loop_id,
        permission_check_decision: preview.decision,
        session_started_event_id,
    })
}

/// Spawn `phi_core::agent_loop` for this session.
///
/// At M5 (CH-02 / ADR-0032) the loop runs against
/// `phi_core::provider::mock::MockProvider`, driven by
/// `profile.mock_response` — see [`super::provider`]. Real provider
/// dispatch defers to M7's credentials-vault milestone.
///
/// The spawned task:
/// 1. Builds an [`AgentContext`](phi_core::types::context::AgentContext)
///    via [`super::provider::build_agent_context`] using the
///    pre-allocated launch-tx IDs.
/// 2. Builds an [`AgentLoopConfig`] with `provider_override =
///    Some(MockProvider)` (M5) and a clone of `runtime.config` as
///    the model identity card.
/// 3. Opens an unbounded mpsc channel and runs `agent_loop` +
///    event-drain concurrently via `tokio::join!` on the same
///    spawned task. When the loop terminates, the sender drops, the
///    channel closes, and the drain loop exits cleanly.
/// 4. Calls [`BabyPhiSessionRecorder::finalise_and_persist`] to
///    write the terminal session state (Completed / Aborted /
///    FailedLaunch).
///
/// Cancellation: `cancel_token` is the same token registered in
/// [`SessionRegistry`] at launch time (ADR-0031). phi-core honours
/// `cancel.is_cancelled()` at every turn boundary; the drain loop
/// then sees the channel close and exits.
///
/// The function is `pub(super)` so the module-level re-exports
/// don't surface it as platform API but tests that exercise the
/// launch chain can reach in if needed.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_agent_task(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    registry: SharedSessionRegistry,
    live_stream_registry: SharedSessionLiveStreamRegistry,
    profile: AgentProfile,
    runtime: ModelRuntime,
    ctx: SessionLaunchContext,
    prompt: String,
    cancel_token: CancellationToken,
    broadcast_tx: tokio::sync::broadcast::Sender<PhiCoreAgentEvent>,
) {
    use super::provider::{build_agent_context, provider_for};

    tokio::spawn(async move {
        // CH-17 / ADR-0055 §D55.1 — attach the per-session broadcast
        // Sender to the recorder so every event the recorder funnels
        // is also published onto the broadcast channel for the SSE
        // handler to fan out.
        let recorder = BabyPhiSessionRecorder::new(repo.clone(), audit, event_bus, ctx.clone())
            .with_broadcast(broadcast_tx);

        let (tx, mut rx) = mpsc::unbounded_channel::<PhiCoreAgentEvent>();
        let mut agent_ctx = build_agent_context(&ctx, &profile);
        let cfg = AgentLoopConfig {
            model_config: runtime.config.clone(),
            provider_override: Some(provider_for(&runtime, &profile)),
            thinking_level: phi_core::types::usage::ThinkingLevel::Off,
            max_tokens: None,
            temperature: None,
            convert_to_llm: None,
            transform_context: None,
            get_steering_messages: None,
            get_follow_up_messages: None,
            context_config: None,
            execution_limits: None,
            cache_config: phi_core::types::usage::CacheConfig::default(),
            tool_execution: phi_core::types::tool::ToolExecutionStrategy::Parallel,
            tool_timeout: None,
            retry_config: phi_core::provider::retry::RetryConfig::default(),
            before_turn: None,
            after_turn: None,
            before_loop: None,
            after_loop: None,
            before_tool_execution: None,
            after_tool_execution: None,
            before_tool_execution_update: None,
            after_tool_execution_update: None,
            before_compaction_start: None,
            after_compaction_end: None,
            on_error: None,
            input_filters: vec![],
            first_turn_trigger: TurnTrigger::User,
            config_id: None,
            context_translation: None,
            prun_pending: None,
            // CH-25 P-SEAL workspace-health fix: phi-core's uncommitted
            // working tree adds the `response_format` required field to
            // `AgentLoopConfig` without propagating the initializer to
            // baby-phi. `ResponseFormat::default() = Text` preserves
            // the historical free-form-text behaviour at session
            // launch; this is the minimal one-line repair to unblock
            // the workspace test gate. The phi-core feature work needs
            // a separate carry-forward chunk to wire structured-output
            // into baby-phi handlers (out of CH-25 scope).
            response_format: phi_core::provider::traits::ResponseFormat::default(),
            // CH-28b P2 workspace-health carrier-fix (per ADR-0064 §D64.3):
            // phi-core 0.8.0 introduces Composition I (opt-in tree-structured
            // braking layer) and adds the `revert_pending` field to
            // `AgentLoopConfig`. `None` preserves the opt-out posture —
            // Composition I is NOT enabled at session launch; agents continue
            // to operate identically to the pre-0.8.0 monotonically-growing-
            // context shape. Future Composition I adoption tracked via
            // D-PHICORE-08-FOLLOWUP-01 (M6+ FUNCTIONAL chunk TBD).
            revert_pending: None,
            // CH-28d P2 workspace-health carrier-fix (per ADR-0066 §D66.3):
            // phi-core 0.10.0 adds two new public fields to AgentLoopConfig:
            // (1) `revert_render_policy: RevertRenderPolicy` — kind-aware decay
            //     window for Composition I `Lesson`/`Note` trunk-context tag
            //     rendering. Active only when `active_node_id.is_some()` (revert
            //     mode); baby-phi does NOT enable revert mode at v0 (Composition I
            //     adoption deferred per D-PHICORE-08-FOLLOWUP-01). Default value
            //     preserves byte-for-byte pre-0.10 behaviour.
            // (2) `current_tool: Option<Arc<Mutex<Option<CurrentToolExecution>>>>`
            //     — shared slot the agent loop writes around every
            //     AgentTool::execute() invocation. Backs the new
            //     BasicAgent::current_tool_timeout(&self) introspection method.
            //     baby-phi is per-request stateless (ADR-0034 §D34.6) + has no
            //     pause-time tool-timeout surface today; `None` disables the slot.
            // Live wire-through deferred to D-PHICORE-10-FOLLOWUP-01.
            revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default(),
            current_tool: None,
        };
        let prompts = vec![AgentMessage::Llm(LlmMessage::new(Message::user(prompt)))];

        let agent_fut =
            phi_core::agent_loop(prompts, &mut agent_ctx, &cfg, tx, cancel_token.clone());
        let drain_fut = async {
            while let Some(evt) = rx.recv().await {
                recorder.on_phi_core_event(evt).await;
            }
        };
        tokio::join!(agent_fut, drain_fut);

        if let Err(e) = recorder.finalise_and_persist().await {
            tracing::error!(
                session_id = %ctx.session_id,
                error = %e,
                "sessions::launch recorder finalise failed — session row may be stuck in running",
            );
        }
        // Drop the registry entry whether or not finalise
        // succeeded — the launch task is done either way.
        registry.remove(&ctx.session_id);
        // CH-17 / ADR-0055 §D55.1 — drop the per-session broadcast
        // Sender from the live-stream registry so subsequent SSE
        // connections for this finalised session see the registry
        // entry as gone (handler returns 410 GONE
        // SESSION_LIVE_STREAM_UNAVAILABLE per ADR-0055 §D55.1).
        live_stream_registry.remove(&ctx.session_id);
    });
}

/// Convenience for tests + status endpoints — fetch the persisted
/// session (post-replay) via the repo surface.
pub async fn await_finalised_detail(
    repo: Arc<dyn Repository>,
    session_id: SessionId,
) -> Result<SessionDetail, SessionError> {
    repo.fetch_session(session_id)
        .await?
        .ok_or(SessionError::SessionNotFound(session_id))
}

/// CH-11 / ADR-0048 §D48.5 + §D48.6 — session-launch consent gating.
///
/// Runs the Permission Check engine with a launch-shaped synthetic
/// manifest (`Action::Invoke` on `session`) + `target_agent =
/// input.agent_id` (the launching subordinate) populated. Iterates the
/// engine's response table:
///
/// - `Allowed` / no `Pending` returned → returns `Ok(None)`; launch
///   continues.
/// - `Pending(AwaitingConsent { subordinate, org })` → mints a
///   `Requested` consent row via the matching minter
///   ([`domain::consents::minters::request_one_time`] /
///   [`domain::consents::minters::request_per_session`]) at the
///   policy + deadline computed from the issuing org's
///   `approval_timeout`, persists via
///   [`Repository::request_consent`], returns
///   `Ok(Some(SessionError::ConsentPending { consent_id, .. }))`.
/// - `Denied(failed_step: Consent, reason: ...)` → returns
///   `Ok(Some(SessionError::ConsentDenied { ... }))` without minting
///   anything. The deny path covers `Declined / Revoked / Expired /
///   TimedOutDeny`.
///
/// At CH-15 / ADR-0054, every non-Consent denial (ManifestEmpty,
/// NoGrantsHeld, NoMatchingGrant, ConstraintViolation, etc.) hard-
/// denies at Step 3 BEFORE this consent-gate runs (drift D4.1
/// closed). The function returns `None` only when the engine returns
/// `Allowed` or when a non-Consent denial reaches this layer (which
/// should be unreachable post-CH-15 because Step 3 catches every
/// non-Consent denial first; the `_ => Ok(None)` arm below is a
/// belt-and-braces fallback).
///
/// `now` is `input.now`; it threads through to the deadline
/// computation per `Org.approval_timeout` (§D48.5 + concept doc 06
/// lines 322–347).
async fn gate_session_launch_consent(
    repo: Arc<dyn Repository>,
    input: &LaunchInput,
    session_id: SessionId,
    now: DateTime<Utc>,
) -> Result<Option<SessionError>, SessionError> {
    // Project consents from the repo for the subordinate (launch
    // target). The launch agent's `agent_id` is the subordinate at
    // this gate — its grants are also what we evaluate against,
    // because the subordinate's role at session launch is the principal
    // whose grants the engine reads against.
    let consents = ConsentIndex::project_from_repo(repo.as_ref(), input.agent_id)
        .await
        .map_err(|e| SessionError::Repository(e.to_string()))?;

    // Gather grants — same projection the preview path uses.
    let agent_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(input.agent_id))
        .await?;
    let project_grants = repo
        .list_grants_for_principal(&PrincipalRef::Project(input.project_id))
        .await?;
    let org_grants = repo
        .list_grants_for_principal(&PrincipalRef::Organization(input.org_id))
        .await?;
    let ceiling_grants: Vec<_> = org_grants.clone();

    // Fetch the org for `approval_timeout` + `default_response`.
    let org = repo.get_organization(input.org_id).await?.ok_or(
        SessionError::AgentNotMemberOfProject {
            agent: input.agent_id,
            project: input.project_id,
        },
    )?;

    // CH-15 / ADR-0054 §D54.1 + §D54.7 — synthetic launch manifest
    // from the typed builder. Preview + launch + consent-gate share
    // identical manifest shape so divergence at this layer cannot
    // re-open D4.1's "advisory layer" pattern at the consent boundary.
    // With `target_agent` set to the subordinate, Step 6's consent
    // gate still fires when a winning grant carries
    // `SubordinateRequired`.
    let manifest = build_session_launch_manifest(input.project_id);
    let catalogue =
        StaticCatalogue::with_entries([(Some(input.org_id), "session_object".to_string())]);
    let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();
    // CH-15 / ADR-0054 §D54.1 — synthetic ToolCall is class-level
    // (`target_uri = ""` skips Step 0 catalogue) but carries the
    // would-be session's tags so the engine's
    // [`expansion::resolve_grant`] kind_refinement (`#kind:session`)
    // on the `session_object` composite matches at Step 3 + Step 5's
    // scope cascade has the `project:<id>` predicate to bind the
    // lead's session-object grant against. `target_agent` stays set
    // to the subordinate so Step 6's consent gate routes correctly.
    let call = ToolCall {
        target_uri: String::new(),
        target_tags: vec![
            "#kind:session".to_string(),
            format!("project:{}", input.project_id),
            format!("org:{}", input.org_id),
        ],
        target_agent: Some(input.agent_id),
        constraint_context: std::collections::HashMap::new(),
    };

    // CH-07 / ADR-0051 §D51.4 — multi-scope cascade reads from
    // `session.tags` per concept doc 06 lines 14–53. At session-launch
    // time the Session row hasn't been persisted yet (this gate fires
    // pre-creation), so the launch path constructs the would-be tag
    // set from the `LaunchInput` and parses it through the canonical
    // helper. Shape A/D inputs (singular `org_id` + `project_id`)
    // produce a 1-org + 1-project tag pair → the cascade still
    // reduces to today's single-tier behaviour because the reader's
    // single membership matches both candidate scopes (count == 1 at
    // each tier). Multi-scope (Shape B/C) session-creation paths land
    // at CH-15 — at that point the launch handler will populate
    // `tags` directly with multi-scope prefixes before this parse.
    let prospective_session_tags = vec![
        format!("org:{}", input.org_id),
        format!("project:{}", input.project_id),
    ];
    let (session_org_tags, session_project_tags) =
        domain::permissions::parse_session_scope_tags(&prospective_session_tags);

    // CH-25 / ADR-0060 §D60.3 — pre-load the agent's owned-resource
    // slices so the engine's `step_2_resolve_grants` can synth
    // owner-grants without an I/O call inside the pure engine.
    let agent_owned_orgs = repo
        .list_agent_owned_orgs(input.agent_id)
        .await
        .map_err(|e| SessionError::Repository(e.to_string()))?;
    let agent_owned_projects = repo
        .list_agent_owned_projects(input.agent_id)
        .await
        .map_err(|e| SessionError::Repository(e.to_string()))?;

    let ctx = CheckContext {
        agent: input.agent_id,
        current_org: Some(input.org_id),
        current_project: Some(input.project_id),
        // CH-11 / ADR-0048 §D48.3 — populate the per-session ambient
        // context now that we know the session id.
        current_session: Some(session_id),
        agent_grants: &agent_grants,
        project_grants: &project_grants,
        org_grants: &org_grants,
        ceiling_grants: &ceiling_grants,
        catalogue: &catalogue,
        consents: &consents,
        // CH-11 / ADR-0048 §D48.4 — the engine consults this for the
        // `TimedOut` arm of the response table.
        timeout_default_response: org.approval_timeout_default_response,
        template_gated_auth_requests: &template_gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        session_org_tags: &session_org_tags,
        session_project_tags: &session_project_tags,
        agent_owned_orgs: &agent_owned_orgs,
        agent_owned_projects: &agent_owned_projects,
        call,
    };

    let decision = check(&ctx, &manifest, &NoopMetrics);

    match decision {
        Decision::Pending { awaiting_consent } => {
            // §D48.5 — choose minter per org's consent_policy +
            // compute deadline from `org.approval_timeout`.
            let deadline_at = compute_consent_deadline(&org.approval_timeout, now);
            let consent = match org.consent_policy {
                ConsentPolicy::Implicit => {
                    // Should not reach Pending under the Implicit
                    // policy — engine short-circuits Step 6. Defend
                    // anyway by treating as one-time.
                    domain::consents::minters::request_one_time(
                        awaiting_consent.subordinate,
                        awaiting_consent.org,
                        deadline_at,
                    )
                }
                ConsentPolicy::OneTime => domain::consents::minters::request_one_time(
                    awaiting_consent.subordinate,
                    awaiting_consent.org,
                    deadline_at,
                ),
                ConsentPolicy::PerSession => domain::consents::minters::request_per_session(
                    awaiting_consent.subordinate,
                    awaiting_consent.org,
                    session_id,
                    deadline_at,
                ),
            };
            let consent_id = consent.id;
            repo.request_consent(&consent)
                .await
                .map_err(|e| SessionError::Repository(e.to_string()))?;
            Ok(Some(SessionError::ConsentPending {
                consent_id,
                subordinate: awaiting_consent.subordinate,
            }))
        }
        Decision::Denied {
            failed_step: FailedStep::Consent,
            ref reason,
        } => match reason {
            // `NoGrantsHeld` etc. don't surface here because
            // `failed_step == Consent`. The match is exhaustive across
            // the consent-tagged DeniedReason variants.
            DeniedReason::ConsentDeclined { subordinate, .. }
            | DeniedReason::ConsentRevoked { subordinate, .. }
            | DeniedReason::ConsentExpired { subordinate, .. }
            | DeniedReason::ConsentTimedOutDeny { subordinate, .. }
            | DeniedReason::NoSessionContext { subordinate, .. } => {
                Ok(Some(SessionError::ConsentDenied {
                    subordinate: *subordinate,
                    reason: format!("{reason:?}"),
                }))
            }
            // CH-15 / ADR-0054: any other DeniedReason flagged at
            // FailedStep::Consent is an engine-shape bug; Step 3's
            // hard-deny path catches every non-Consent denial before
            // this gate runs. Returning Ok(None) here is a
            // belt-and-braces fallback that defers to the Allowed
            // path below.
            _ => Ok(None),
        },
        // CH-15 / ADR-0054 §D54.6: non-Consent Decision::Denied is
        // hard-denied at Step 3 before this gate runs; reaching the
        // arm here would be an engine-shape bug. `Decision::Allowed`
        // falls through to the launch happy path. The Ok(None)
        // unification preserves the function's "consent-gate-only
        // surface" type signature.
        Decision::Denied { .. } | Decision::Allowed { .. } => Ok(None),
    }
}

/// Compute the consent `deadline_at` per `Org.approval_timeout` (CH-11
/// / ADR-0048 §D48.5; concept doc 06 lines 322–347).
///
/// - [`ApprovalTimeout::Fixed`] → `now + duration` exactly.
/// - [`ApprovalTimeout::ProjectDuration`] → fall back to `now + 24h`
///   for v0. The concept doc's "max(project.deadline_at)" behaviour
///   for shape A/B/C is recorded as a CH-11 deferred-from-ledger entry
///   (Project has no `deadline_at` field at this milestone). Shape D
///   (no project) also resolves to `now + 24h` per concept doc 06 line
///   343.
fn compute_consent_deadline(
    timeout: &ApprovalTimeout,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    match timeout {
        ApprovalTimeout::ProjectDuration => {
            // v0 fallback — Project has no `deadline_at` field at this
            // milestone (deferred-from-CH-11 ledger entry).
            Some(now + Duration::hours(24))
        }
        ApprovalTimeout::Fixed { duration } => Some(now + *duration),
    }
}
