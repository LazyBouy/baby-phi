//! System Agents configuration — admin page 13 (M5/P6).
//!
//! Each submodule owns one handler path:
//! - [`list`] — GET `/api/v0/orgs/:org/system-agents` returns
//!   `{standard, org_specific, recent_events}`.
//! - [`tune`] — PATCH `/api/v0/orgs/:org/system-agents/:agent_id`
//!   adjusts `parallelize` (R-ADMIN-13-W1).
//! - [`add`] — POST `/api/v0/orgs/:org/system-agents` creates a
//!   new org-specific System agent (R-ADMIN-13-W2).
//! - [`disable`] — POST `.../disable` marks active=false
//!   (R-ADMIN-13-W3).
//! - [`archive`] — POST `.../archive` archives an org-specific
//!   system agent (R-ADMIN-13-W4).
//!
//! ## phi-core leverage
//!
//! **One new direct import at P6** — `AgentProfile` in
//! [`add`] for profile_ref validation. Matches Part 1.5
//! prediction.
//!
//! ## Runtime status
//!
//! `SystemAgentRuntimeStatus` tiles (per-agent queue-depth +
//! last-fired-at) are upserted by the event-bus listeners via
//! the shared helper
//! [`domain::events::listeners::record_system_agent_fire`] (M5/P6
//! wiring). The `list` endpoint reads + merges them with the
//! agent-catalog rows.

use domain::model::ids::{AgentId, OrgId};
use domain::repository::RepositoryError;

pub mod add;
pub mod archive;
pub mod audit_events;
pub mod disable;
pub mod list;
pub mod tune;

pub use add::{add_system_agent, AddInput, AddOutcome};
pub use archive::{archive_system_agent, ArchiveInput, ArchiveOutcome};
pub use disable::{disable_system_agent, DisableInput, DisableOutcome};
pub use list::{list_system_agents, SystemAgentRow, SystemAgentsListing};
pub use tune::{tune_system_agent, TuneInput, TuneOutcome};

/// The two platform-standard system agents every org provisions.
///
/// Profile-ref slugs correspond to the `AgentProfile.config_id`
/// values seeded at M3 org-creation. Agents with `kind: System` +
/// one of these slugs sort into the "standard" bucket on page 13;
/// anything else is "org-specific".
pub const STANDARD_SYSTEM_AGENT_PROFILES: &[&str] =
    &["system-memory-extraction", "system-agent-catalog"];

/// Stable error enum for every system-agent handler.
#[derive(Debug, thiserror::Error)]
pub enum SystemAgentError {
    #[error("SYSTEM_AGENT_INPUT_INVALID: {0}")]
    InputInvalid(String),

    #[error("SYSTEM_AGENT_NOT_FOUND: org {org} / agent {agent}")]
    NotFound { org: OrgId, agent: AgentId },

    #[error("ORG_NOT_FOUND: {0}")]
    OrgNotFound(OrgId),

    /// The target agent exists but isn't `kind: System`.
    #[error("SYSTEM_AGENT_WRONG_KIND: agent {0} is not a system agent")]
    WrongKind(AgentId),

    /// Tune / add `parallelize` exceeds the org cap or blueprint
    /// declaration.
    #[error("PARALLELIZE_CEILING_EXCEEDED: requested {requested}, ceiling {ceiling}")]
    ParallelizeCeilingExceeded { requested: u32, ceiling: u32 },

    /// Add rejected because the id clashes with an existing agent.
    #[error("SYSTEM_AGENT_ID_IN_USE: {0}")]
    IdInUse(AgentId),

    /// `profile_ref` didn't resolve to a known AgentProfile.
    #[error("SYSTEM_AGENT_PROFILE_REF_UNKNOWN: {0}")]
    ProfileRefUnknown(String),

    /// Archive attempted on a standard system agent (R-ADMIN-13-W4
    /// — archive path is org-specific only).
    #[error("STANDARD_SYSTEM_AGENT_NOT_ARCHIVABLE: {0}")]
    StandardNotArchivable(AgentId),

    /// Disable attempted without the `confirm: true` body.
    #[error("DISABLE_CONFIRMATION_REQUIRED")]
    DisableConfirmationRequired,

    /// Already-terminal archive / disable retry.
    #[error("SYSTEM_AGENT_ALREADY_TERMINAL: agent {0}")]
    AlreadyTerminal(AgentId),

    #[error("TRIGGER_TYPE_INVALID: {0}")]
    TriggerTypeInvalid(String),

    /// Pass-throughs.
    #[error("repository error: {0}")]
    Repository(String),
    #[error("audit emit error: {0}")]
    AuditEmit(String),
}

impl From<RepositoryError> for SystemAgentError {
    fn from(e: RepositoryError) -> Self {
        SystemAgentError::Repository(e.to_string())
    }
}

pub fn http_status_for(err: &SystemAgentError) -> u16 {
    match err {
        SystemAgentError::InputInvalid(_)
        | SystemAgentError::DisableConfirmationRequired
        | SystemAgentError::TriggerTypeInvalid(_) => 400,
        SystemAgentError::OrgNotFound(_) | SystemAgentError::NotFound { .. } => 404,
        SystemAgentError::WrongKind(_)
        | SystemAgentError::ParallelizeCeilingExceeded { .. }
        | SystemAgentError::IdInUse(_)
        | SystemAgentError::ProfileRefUnknown(_)
        | SystemAgentError::StandardNotArchivable(_)
        | SystemAgentError::AlreadyTerminal(_) => 409,
        SystemAgentError::Repository(_) | SystemAgentError::AuditEmit(_) => 500,
    }
}

pub fn wire_code_for(err: &SystemAgentError) -> &'static str {
    match err {
        SystemAgentError::InputInvalid(_) => "SYSTEM_AGENT_INPUT_INVALID",
        SystemAgentError::NotFound { .. } => "SYSTEM_AGENT_NOT_FOUND",
        SystemAgentError::OrgNotFound(_) => "ORG_NOT_FOUND",
        SystemAgentError::WrongKind(_) => "SYSTEM_AGENT_WRONG_KIND",
        SystemAgentError::ParallelizeCeilingExceeded { .. } => "PARALLELIZE_CEILING_EXCEEDED",
        SystemAgentError::IdInUse(_) => "SYSTEM_AGENT_ID_IN_USE",
        SystemAgentError::ProfileRefUnknown(_) => "SYSTEM_AGENT_PROFILE_REF_UNKNOWN",
        SystemAgentError::StandardNotArchivable(_) => "STANDARD_SYSTEM_AGENT_NOT_ARCHIVABLE",
        SystemAgentError::DisableConfirmationRequired => "DISABLE_CONFIRMATION_REQUIRED",
        SystemAgentError::AlreadyTerminal(_) => "SYSTEM_AGENT_ALREADY_TERMINAL",
        SystemAgentError::TriggerTypeInvalid(_) => "TRIGGER_TYPE_INVALID",
        SystemAgentError::Repository(_) => "REPOSITORY_ERROR",
        SystemAgentError::AuditEmit(_) => "AUDIT_EMIT_ERROR",
    }
}

/// Decide whether an [`AgentProfile`] profile_ref slug is one of
/// the two platform-standard system agents.
///
/// [`AgentProfile`]: domain::model::nodes::AgentProfile
pub fn is_standard_system_agent(profile_config_id: Option<&str>) -> bool {
    match profile_config_id {
        Some(slug) => STANDARD_SYSTEM_AGENT_PROFILES.contains(&slug),
        None => false,
    }
}
