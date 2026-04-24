#![allow(clippy::result_large_err)]

//! Platform-admin project surfaces — pages 10 (creation — M4/P6) + 11
//! (detail — M4/P7).
//!
//! ## phi-core leverage
//!
//! **Q1 direct imports: 0** — project creation is pure phi governance
//! (Project + OKRs + ProjectShape are baby-phi composites; phi-core
//! has no Project / OKR / planning concept — see M4 plan Part 1.5
//! Page 10).
//!
//! **Q2 transitive: 0 at the P6 wire tier** — the request body carries
//! `{project_id, name, shape, lead_agent_id, member_agent_ids,
//! sponsor_agent_ids, objectives, key_results}`, all phi governance.
//! Lead-picker drill-down into phi-core blueprints happens via the
//! agent roster (page 08, M4/P4) — a different endpoint.
//!
//! **Q3 rejections**: `phi_core::Session` (M5), `phi_core::AgentEvent`
//! (agent-loop telemetry — orthogonal to `DomainEvent`),
//! `ContextConfig` / `RetryConfig` (inherit-from-snapshot per
//! ADR-0023).
//!
//! ## M4/P3 + P6 scope
//!
//! - M4/P3 shipped the two repo-backed resolvers the Template A
//!   fire-listener needs — see [`resolvers`].
//! - M4/P6 (this phase) ships [`create`] — orchestrator for
//!   `POST /api/v0/orgs/:org_id/projects` + `POST
//!   /api/v0/projects/_pending/:ar_id/approve`.

pub mod create;
pub mod detail;
pub mod resolvers;

pub use resolvers::{
    RepoActorResolver, RepoAdoptionArResolver, RepoTemplateCAdoptionArResolver,
    RepoTemplateDAdoptionArResolver,
};

use domain::model::ids::{AgentId, AuthRequestId, OrgId, ProjectId};

/// Stable error codes returned by the project-creation handlers. Every
/// variant maps 1:1 to a wire code the web UI + CLI display verbatim.
#[derive(Debug)]
pub enum ProjectError {
    /// `400` — input failed shape validation (bad `project_id` regex,
    /// empty `name`, OKR measurement-type mismatch, etc.).
    Validation(String),
    /// `400` — OKR payload is invalid (measurement-type vs value
    /// mismatch, empty name, missing objective for key_result).
    OkrValidation(String),
    /// `404` — the owning org id does not exist.
    OrgNotFound(OrgId),
    /// `400` — Shape B co-owner org id does not exist OR is the same
    /// as the primary owner.
    CoOwnerInvalid(String),
    /// `404` — the supplied `lead_agent_id` does not exist.
    LeadNotFound(AgentId),
    /// `400` — the supplied lead agent does not belong to any of the
    /// owning orgs (Shape A: must be in the single owning org; Shape
    /// B: must be in one of the two co-owning orgs).
    LeadNotInOwningOrg,
    /// `400` — a member / sponsor agent id does not exist OR does not
    /// belong to an owning org.
    MemberInvalid(String),
    /// `400` — Shape B submit was missing a co-owner org id.
    ShapeBMissingCoOwner,
    /// `400` — Shape A submit supplied a co-owner org id (not allowed).
    ShapeAHasCoOwner,
    /// `409` — `project_id` already exists in the target org.
    ProjectIdInUse(ProjectId),
    /// `404` — pending-approval handler called with an unknown AR id.
    PendingArNotFound(AuthRequestId),
    /// `400` — pending-approval handler called on an AR that is NOT a
    /// Shape B project-creation AR.
    PendingArNotShapeB,
    /// `409` — pending-approval handler called on an AR that is in a
    /// terminal state (already decided).
    PendingArAlreadyTerminal,
    /// `403` — caller approving a Shape B AR is not listed as one of
    /// the two approver slots.
    ApproverNotAuthorized,
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
    /// Auth-request state-machine transition returned an error.
    Transition(String),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Validation(m) => write!(f, "validation failed: {m}"),
            ProjectError::OkrValidation(m) => write!(f, "OKR validation failed: {m}"),
            ProjectError::OrgNotFound(id) => write!(f, "org {id} not found"),
            ProjectError::CoOwnerInvalid(m) => write!(f, "co-owner invalid: {m}"),
            ProjectError::LeadNotFound(id) => write!(f, "lead agent {id} not found"),
            ProjectError::LeadNotInOwningOrg => {
                write!(f, "lead agent does not belong to an owning org")
            }
            ProjectError::MemberInvalid(m) => write!(f, "member/sponsor invalid: {m}"),
            ProjectError::ShapeBMissingCoOwner => {
                write!(f, "Shape B requires `co_owner_org_id` in the payload")
            }
            ProjectError::ShapeAHasCoOwner => {
                write!(f, "Shape A must not supply `co_owner_org_id`")
            }
            ProjectError::ProjectIdInUse(id) => {
                write!(f, "project id {id} already in use in the target org")
            }
            ProjectError::PendingArNotFound(id) => {
                write!(f, "pending auth request {id} not found")
            }
            ProjectError::PendingArNotShapeB => {
                write!(f, "auth request is not a Shape B project-creation AR")
            }
            ProjectError::PendingArAlreadyTerminal => {
                write!(f, "pending AR is already in a terminal state")
            }
            ProjectError::ApproverNotAuthorized => {
                write!(f, "caller is not a designated approver for this AR")
            }
            ProjectError::Repository(m) => write!(f, "repository: {m}"),
            ProjectError::AuditEmit(m) => write!(f, "audit emit: {m}"),
            ProjectError::Transition(m) => write!(f, "state transition: {m}"),
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<domain::repository::RepositoryError> for ProjectError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        use domain::repository::RepositoryError as E;
        match e {
            E::Conflict(m) if m.to_lowercase().contains("project") && m.contains("already") => {
                ProjectError::ProjectIdInUse(ProjectId::new()) // placeholder — handler re-maps to the real id
            }
            E::InvalidArgument(m) => ProjectError::Validation(m),
            other => ProjectError::Repository(other.to_string()),
        }
    }
}
