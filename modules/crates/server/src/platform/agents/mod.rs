#![allow(clippy::result_large_err)]

//! Platform-admin agent surfaces.
//!
//! M4 scope (phase-by-phase):
//!
//! - **P4** — page 08 read-only roster: [`list`].
//! - **P5** — page 09 edit/create + per-agent ExecutionLimits override
//!   (ADR-0027): [`create`], [`update`], [`execution_limits`].
//!
//! ## phi-core leverage
//!
//! M4/P5 is **the phi-core-heaviest phase of M4**. Four direct imports
//! land in the sibling modules:
//!
//! - [`phi_core::agents::profile::AgentProfile`] — form binding in
//!   [`create`] + [`update`].
//! - [`phi_core::context::execution::ExecutionLimits`] — writeable per
//!   [ADR-0027](../../../../../../docs/specs/v0/implementation/m4/decisions/0027-per-agent-execution-limits-override.md)
//!   opt-in override path.
//! - [`phi_core::provider::model::ModelConfig`] — `model_config.id`
//!   validation against the org's model catalogue.
//! - [`phi_core::types::ThinkingLevel`] — 5-variant dropdown.
//!
//! The roster list ([`list`], M4/P4) retains **zero** phi-core imports
//! by design — the list response is a governance-only summary.

pub mod create;
pub mod execution_limits;
pub mod list;
pub mod update;

use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{AgentKind, AgentRole};

/// Stable error codes returned by the agent handlers. Every variant
/// maps 1:1 to a wire code the web UI + CLI display verbatim.
#[derive(Debug)]
pub enum AgentError {
    /// Input failed shape validation (bad role string, empty search
    /// after trim, `parallelize == 0`, etc.).
    Validation(String),
    /// The requested org does not exist.
    OrgNotFound(OrgId),
    /// The requested agent does not exist.
    AgentNotFound(AgentId),
    /// `409` — attempted to create an agent whose id already exists
    /// (typically a client retry after a successful response).
    AgentIdInUse,
    /// `400` — create/update carries a `role` incompatible with `kind`
    /// per [`AgentRole::is_valid_for`].
    RoleInvalidForKind { role: AgentRole, kind: AgentKind },
    /// `400` — edit attempted to change an immutable field (`id`,
    /// `kind`, `role`, `owning_org`). The inner string names the
    /// offending field so the wizard can surface a specific message.
    ImmutableFieldChanged(&'static str),
    /// `400` — `parallelize` outside the supported range
    /// `1..=PARALLELIZE_MAX_CAP` (see [`create::PARALLELIZE_MAX_CAP`]).
    ParallelizeCeilingExceeded { requested: u32, ceiling: u32 },
    /// `400` — per-agent `ExecutionLimits` override would raise some
    /// field above the org snapshot's corresponding ceiling. Opt-in
    /// overrides can only tighten, never loosen, per ADR-0027.
    ExecutionLimitsExceedOrgCeiling(String),
    /// `409` — `model_config.id` change requested while the agent has
    /// non-zero active sessions. Per D-M4-3 the operator must terminate
    /// sessions first. M4 stub: always returns 0 sessions (M5 flips it
    /// to a real query against the Session table).
    ActiveSessionsBlockModelChange,
    /// `403` — attempted to edit a `System`-role agent. These are
    /// platform-provisioned and lifecycle-managed; their blueprints
    /// are read-only on page 09.
    SystemAgentReadOnly,
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::Validation(m) => write!(f, "validation failed: {m}"),
            AgentError::OrgNotFound(id) => write!(f, "org {id} not found"),
            AgentError::AgentNotFound(id) => write!(f, "agent {id} not found"),
            AgentError::AgentIdInUse => write!(f, "agent id already in use"),
            AgentError::RoleInvalidForKind { role, kind } => {
                write!(f, "role {} is not valid for kind {:?}", role.as_str(), kind)
            }
            AgentError::ImmutableFieldChanged(field) => {
                write!(f, "cannot change immutable field `{field}`")
            }
            AgentError::ParallelizeCeilingExceeded { requested, ceiling } => write!(
                f,
                "parallelize {requested} exceeds per-org ceiling {ceiling}"
            ),
            AgentError::ExecutionLimitsExceedOrgCeiling(m) => {
                write!(f, "execution limits exceed org ceiling: {m}")
            }
            AgentError::ActiveSessionsBlockModelChange => {
                write!(f, "cannot change model_config while active sessions exist")
            }
            AgentError::SystemAgentReadOnly => {
                write!(f, "system agents are read-only on page 09")
            }
            AgentError::Repository(m) => write!(f, "repository: {m}"),
            AgentError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<domain::repository::RepositoryError> for AgentError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        use domain::repository::RepositoryError as E;
        match e {
            E::Conflict(m) if m.to_lowercase().contains("agent") && m.contains("already") => {
                AgentError::AgentIdInUse
            }
            E::InvalidArgument(m) => AgentError::Validation(m),
            other => AgentError::Repository(other.to_string()),
        }
    }
}
