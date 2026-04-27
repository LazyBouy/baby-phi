//! Session launch + preview + terminate + show + list + tools
//! (M5/P4 — admin page 14).
//!
//! Each submodule owns one handler path:
//! - [`launch`] — POST `/api/v0/orgs/:org/projects/:project/sessions`
//!   (C-M5-2 + C-M5-3 close; writes Session + `runs_session` edge +
//!   `uses_model` edge + first `LoopRecordNode` in a compound tx,
//!   spawns the replay task, returns a `LaunchReceipt`).
//! - [`preview`] — POST `/api/v0/orgs/:org/projects/:project/sessions/preview`
//!   (D5 — server-side Permission Check trace).
//! - [`show`] — GET `/api/v0/sessions/:id` (full `SessionDetail`).
//! - [`list`] — GET `/api/v0/projects/:project/sessions` (header strip).
//! - [`terminate`] — POST `/api/v0/sessions/:id/terminate` (cancel
//!   token fire + governance state flip to Aborted + `SessionAborted`
//!   emit).
//! - [`tools`] — GET `/api/v0/sessions/:id/tools` (C-M5-4 —
//!   resolves tool summaries for the session's agent).
//!
//! ## phi-core leverage
//!
//! Four new imports land at P4 — see each submodule:
//! - `launch.rs` — `phi_core::{agent_loop, agent_loop_continue}` +
//!   `phi_core::provider::model::ModelConfig` (runtime resolution).
//! - `tools.rs` — `phi_core::types::tool::AgentTool` (compile-time
//!   witness + future trait-object dispatch at M7+).
//!
//! See
//! [`docs/specs/v0/implementation/m5/architecture/session-launch.md`](../../../../../docs/specs/v0/implementation/m5/architecture/session-launch.md).

use domain::model::ids::{AgentId, ModelProviderId, ProjectId, SessionId};
use domain::repository::RepositoryError;

pub mod launch;
pub mod list;
pub mod preview;
pub(crate) mod provider;
pub mod show;
pub mod terminate;
pub mod tools;

pub use launch::{launch_session, LaunchInput, LaunchReceipt};
pub use list::list_sessions_in_project;
pub use preview::{preview_session, PreviewInput, PreviewOutcome};
pub use show::{show_session, SessionView};
pub use terminate::{terminate_session, TerminateInput, TerminateOutcome};
pub use tools::{resolve_agent_tools, ToolSummary};

/// Stable error enum for every session-surface handler. Each variant
/// maps 1:1 to a wire code the web UI + CLI display verbatim; a new
/// variant here requires a matching table-row in
/// `m5/user-guide/troubleshooting.md`.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    // ---- Generic validation + not-found paths (400 / 404) -----------
    #[error("SESSION_INPUT_INVALID: {0}")]
    InputInvalid(String),
    #[error("AGENT_NOT_FOUND: {0}")]
    AgentNotFound(AgentId),
    #[error("PROJECT_NOT_FOUND: {0}")]
    ProjectNotFound(ProjectId),
    #[error("SESSION_NOT_FOUND: {0}")]
    SessionNotFound(SessionId),
    #[error("MODEL_RUNTIME_NOT_FOUND: {0}")]
    ModelRuntimeNotFound(ModelProviderId),

    // ---- Launch gates (409 / 503) -----------------------------------
    /// W2 — the agent is already running `parallelize` sessions.
    #[error("PARALLELIZE_CAP_REACHED: {active}/{cap} active for agent {agent}")]
    ParallelizeCapReached {
        agent: AgentId,
        active: u64,
        cap: u32,
    },
    /// D3 — per-worker `max_concurrent` ceiling hit.
    #[error("SESSION_WORKER_SATURATED: registry at {current}/{cap} live sessions")]
    SessionWorkerSaturated { current: usize, cap: u32 },
    /// Agent profile carries no `model_config_id` — nothing to
    /// launch against.
    #[error("MODEL_RUNTIME_UNRESOLVED: agent {0} has no model_config binding")]
    ModelRuntimeUnresolved(AgentId),
    /// The agent's `model_config_id` points at an archived runtime
    /// row — operator must re-bind.
    #[error("MODEL_RUNTIME_ARCHIVED: {0}")]
    ModelRuntimeArchived(ModelProviderId),
    /// The agent's profile row is missing entirely — launch refuses
    /// rather than synthesising defaults at runtime.
    #[error("AGENT_PROFILE_MISSING: agent {0}")]
    AgentProfileMissing(AgentId),

    // ---- Permission Check gates (403) -------------------------------
    /// `PERMISSION_CHECK_FAILED_AT_STEP_<N>` family. The full trace
    /// is preserved on the response so the UI can render the worked
    /// example.
    #[error("PERMISSION_CHECK_FAILED_AT_STEP_{step}: {reason}")]
    PermissionCheckFailed { step: u8, reason: String },
    /// Agent isn't a member of the project (or the project's owning
    /// org).
    #[error("AGENT_NOT_MEMBER_OF_PROJECT: agent {agent} / project {project}")]
    AgentNotMemberOfProject { agent: AgentId, project: ProjectId },
    /// Viewer isn't allowed to see this session.
    #[error("FORBIDDEN: session {0}")]
    Forbidden(SessionId),

    // ---- Terminate gates (409) --------------------------------------
    #[error("SESSION_ALREADY_TERMINAL: session {0} is not in `running` state")]
    SessionAlreadyTerminal(SessionId),
    #[error("TERMINATE_REASON_REQUIRED")]
    TerminateReasonRequired,
    #[error("TERMINATE_FORBIDDEN: session {0}")]
    TerminateForbidden(SessionId),

    // ---- Recorder / internal runtime errors (500) -------------------
    #[error("RECORDER_FAILURE: {0}")]
    RecorderFailure(String),
    #[error("SESSION_REPLAY_PANIC: {0}")]
    SessionReplayPanic(String),
    #[error("COMPOUND_TX_FAILURE: {0}")]
    CompoundTxFailure(String),

    // ---- Pass-throughs ---------------------------------------------
    #[error("repository error: {0}")]
    Repository(String),
    #[error("audit emit error: {0}")]
    AuditEmit(String),
}

impl From<RepositoryError> for SessionError {
    fn from(e: RepositoryError) -> Self {
        SessionError::Repository(e.to_string())
    }
}

/// Classify a [`SessionError`] into the HTTP status code the
/// handler should return. Keeps the mapping centralised so the
/// troubleshooting doc has a single source of truth.
pub fn http_status_for(err: &SessionError) -> u16 {
    match err {
        SessionError::InputInvalid(_) | SessionError::TerminateReasonRequired => 400,
        SessionError::Forbidden(_)
        | SessionError::AgentNotMemberOfProject { .. }
        | SessionError::PermissionCheckFailed { .. }
        | SessionError::TerminateForbidden(_) => 403,
        SessionError::AgentNotFound(_)
        | SessionError::ProjectNotFound(_)
        | SessionError::SessionNotFound(_)
        | SessionError::ModelRuntimeNotFound(_) => 404,
        SessionError::ParallelizeCapReached { .. }
        | SessionError::ModelRuntimeUnresolved(_)
        | SessionError::ModelRuntimeArchived(_)
        | SessionError::AgentProfileMissing(_)
        | SessionError::SessionAlreadyTerminal(_) => 409,
        SessionError::SessionWorkerSaturated { .. } => 503,
        SessionError::RecorderFailure(_)
        | SessionError::SessionReplayPanic(_)
        | SessionError::CompoundTxFailure(_)
        | SessionError::Repository(_)
        | SessionError::AuditEmit(_) => 500,
    }
}

/// Stable wire code (`UPPER_SNAKE_CASE`) for each [`SessionError`]
/// variant. Used by the HTTP handlers to emit a `code: "..."` field
/// on the 4xx/5xx JSON envelope — the web UI + CLI match on this
/// string verbatim.
pub fn wire_code_for(err: &SessionError) -> &'static str {
    match err {
        SessionError::InputInvalid(_) => "SESSION_INPUT_INVALID",
        SessionError::AgentNotFound(_) => "AGENT_NOT_FOUND",
        SessionError::ProjectNotFound(_) => "PROJECT_NOT_FOUND",
        SessionError::SessionNotFound(_) => "SESSION_NOT_FOUND",
        SessionError::ModelRuntimeNotFound(_) => "MODEL_RUNTIME_NOT_FOUND",
        SessionError::ParallelizeCapReached { .. } => "PARALLELIZE_CAP_REACHED",
        SessionError::SessionWorkerSaturated { .. } => "SESSION_WORKER_SATURATED",
        SessionError::ModelRuntimeUnresolved(_) => "MODEL_RUNTIME_UNRESOLVED",
        SessionError::ModelRuntimeArchived(_) => "MODEL_RUNTIME_ARCHIVED",
        SessionError::AgentProfileMissing(_) => "AGENT_PROFILE_MISSING",
        SessionError::PermissionCheckFailed { .. } => "PERMISSION_CHECK_FAILED",
        SessionError::AgentNotMemberOfProject { .. } => "AGENT_NOT_MEMBER_OF_PROJECT",
        SessionError::Forbidden(_) => "FORBIDDEN",
        SessionError::SessionAlreadyTerminal(_) => "SESSION_ALREADY_TERMINAL",
        SessionError::TerminateReasonRequired => "TERMINATE_REASON_REQUIRED",
        SessionError::TerminateForbidden(_) => "TERMINATE_FORBIDDEN",
        SessionError::RecorderFailure(_) => "RECORDER_FAILURE",
        SessionError::SessionReplayPanic(_) => "SESSION_REPLAY_PANIC",
        SessionError::CompoundTxFailure(_) => "COMPOUND_TX_FAILURE",
        SessionError::Repository(_) => "REPOSITORY_ERROR",
        SessionError::AuditEmit(_) => "AUDIT_EMIT_ERROR",
    }
}
