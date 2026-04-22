#![allow(clippy::result_large_err)]

//! Page 06 — Organization Creation + list/show business logic.
//!
//! ## phi-core leverage
//!
//! Per the [leverage checklist](../../../../docs/specs/v0/implementation/m3/architecture/phi-core-leverage-checklist.md)
//! Q1/Q2/Q3 split:
//!
//! - **Q1 (direct imports)**: [`create.rs`] imports
//!   [`phi_core::agents::profile::AgentProfile`] to clone + tweak
//!   per-system-agent blueprints. [`list.rs`] / [`show.rs`] have no
//!   direct phi-core imports (governance-plane reads only).
//! - **Q2 (transitive)**: `OrgCreationPayload` carries the full
//!   `OrganizationDefaultsSnapshot` which wraps four phi-core types
//!   (`ExecutionLimits`, `ContextConfig`, `RetryConfig`, `AgentProfile`).
//!   These transit via serde through the HTTP body untouched.
//! - **Q3 (inherit-not-duplicate)**: Per ADR-0023, `ExecutionLimits`
//!   / `ContextConfig` / `RetryConfig` live **only** in the snapshot;
//!   no per-agent duplicates are created. The invariant is enforced
//!   structurally (the compound tx writes zero rows into
//!   `execution_limits` / `retry_policy` / `cache_policy` /
//!   `compaction_policy` tables) and verified by
//!   [`apply_org_creation_tx_test::adr_0023_invariant_no_per_agent_policy_nodes_materialised`].
//!
//! ## Scope at M3
//!
//! - `POST /api/v0/orgs` — wizard submission; compound-tx create.
//! - `GET  /api/v0/orgs` — list orgs the caller can see.
//! - `GET  /api/v0/orgs/:id` — single org detail (dashboard-preview
//!   shape; real dashboard in M3/P5).
//!
//! `UsesModel` edge wiring is **deferred to M5 session launch** per
//! the D12 / ADR-0023 "inherit-from-snapshot" pattern — M3 stores
//! `Organization.default_model_provider: ModelProviderId` only.

pub mod create;
pub mod dashboard;
pub mod list;
pub mod show;

use domain::model::ids::{
    AgentId, AuditEventId, AuthRequestId, GrantId, NodeId, OrgId, TemplateId,
};

/// Stable error codes returned by the `orgs` handlers. Every variant
/// maps 1:1 to a wire code the web UI + CLI display verbatim.
#[derive(Debug)]
pub enum OrgError {
    /// Input failed shape validation (empty display_name, duplicate
    /// system-agent id, invalid template kind, etc.).
    Validation(String),
    /// The requested org_id already exists.
    OrgIdInUse,
    /// The caller requested an adopt-template kind not supported at
    /// creation time — Template E / SystemBootstrap are not
    /// adopt-on-create; Template F is reserved for M6.
    TemplateNotAdoptable(String),
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
}

impl std::fmt::Display for OrgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgError::Validation(m) => write!(f, "validation failed: {m}"),
            OrgError::OrgIdInUse => write!(f, "org id already in use"),
            OrgError::TemplateNotAdoptable(m) => write!(f, "template not adoptable: {m}"),
            OrgError::Repository(m) => write!(f, "repository: {m}"),
            OrgError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for OrgError {}

impl From<domain::repository::RepositoryError> for OrgError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        use domain::repository::RepositoryError as E;
        match e {
            E::Conflict(m) if m.contains("organization already exists") => OrgError::OrgIdInUse,
            E::InvalidArgument(m) => OrgError::Validation(m),
            other => OrgError::Repository(other.to_string()),
        }
    }
}

/// Everything the CREATE handler returns on success. Every id pairs
/// with a row the compound tx persisted; `audit_event_ids` pairs with
/// the `emit_audit_batch` sequence (one `OrganizationCreated` + N
/// `AuthorityTemplateAdopted` events, in input order).
#[derive(Debug, Clone)]
pub struct CreatedOrg {
    pub org_id: OrgId,
    pub ceo_agent_id: AgentId,
    pub ceo_channel_id: NodeId,
    pub ceo_inbox_id: NodeId,
    pub ceo_outbox_id: NodeId,
    pub ceo_grant_id: GrantId,
    pub system_agent_ids: [AgentId; 2],
    pub token_budget_pool_id: NodeId,
    pub adoption_auth_request_ids: Vec<AuthRequestId>,
    /// Audit-event ids emitted in the same input order as the batch:
    /// `[organization_created, template_A_adopted, template_B_adopted, ...]`.
    pub audit_event_ids: Vec<AuditEventId>,
    /// Ids of the `Template` graph nodes persisted alongside the
    /// adoption ARs. Empty for M3 (template nodes are persisted by
    /// the wizard payload when the operator enables each template;
    /// the mapping is surfaced so downstream dashboard queries can
    /// resolve `adoption_ar.provenance_template`).
    pub template_ids: Vec<TemplateId>,
}
