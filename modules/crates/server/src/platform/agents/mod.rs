#![allow(clippy::result_large_err)]

//! Platform-admin agent surfaces.
//!
//! M4 scope (phase-by-phase):
//!
//! - **P4** — page 08 read-only roster: [`list`].
//! - **P5** — page 09 edit/create + per-agent ExecutionLimits override
//!   (lands alongside `create.rs` + `update.rs` + `execution_limits.rs`).
//!
//! ## phi-core leverage
//!
//! Q1 **none** at P4 — the roster payload carries phi governance
//! fields only (`id`, `kind`, `role`, `display_name`, `owning_org`,
//! `created_at`). `AgentProfile` (and therefore the phi-core
//! `blueprint` wrap) lives on a **separate node**; page 09 edits it,
//! page 08 does not surface it.
//!
//! Q2 **none** — no phi-core types transit the list wire shape.
//!
//! Q3 considered-and-rejected: phi-core has no roster / list / filter
//! concept; this module is pure phi governance.

pub mod list;

use domain::model::ids::OrgId;

/// Stable error codes returned by the agent handlers. Every variant
/// maps 1:1 to a wire code the web UI + CLI display verbatim.
#[derive(Debug)]
pub enum AgentError {
    /// Input failed shape validation (bad role string, empty search
    /// after trim, etc.).
    Validation(String),
    /// The requested org does not exist.
    OrgNotFound(OrgId),
    /// Repository returned an error.
    Repository(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::Validation(m) => write!(f, "validation failed: {m}"),
            AgentError::OrgNotFound(id) => write!(f, "org {id} not found"),
            AgentError::Repository(m) => write!(f, "repository: {m}"),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<domain::repository::RepositoryError> for AgentError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        use domain::repository::RepositoryError as E;
        match e {
            E::InvalidArgument(m) => AgentError::Validation(m),
            other => AgentError::Repository(other.to_string()),
        }
    }
}
