//! Authority Template adoption — admin page 12 (M5/P5).
//!
//! Each submodule owns one handler path:
//! - [`list`] — GET `/api/v0/orgs/:org/authority-templates`
//!   returns `{pending, active, revoked, available}`.
//! - [`approve`] — POST `/api/v0/orgs/:org/authority-templates/:kind/approve`
//!   transitions the pending adoption AR for `kind` to Approved.
//! - [`deny`] — POST `.../:kind/deny` transitions to Denied.
//! - [`adopt`] — POST `.../:kind/adopt` mints a fresh
//!   CEO-self-approved adoption AR for `kind`. Used by R-ADMIN-12-W3
//!   when an org adopts a template that wasn't enabled at
//!   org-creation (e.g. Template C / D).
//! - [`revoke`] — POST `.../:kind/revoke` transitions the active
//!   adoption AR to Revoked + **cascade-revokes every grant**
//!   whose `descends_from == adoption_ar.id` (forward-only per
//!   R-ADMIN-12-W4 + [system/s04-auth-request-state-transitions.md]).
//!
//! ## phi-core leverage
//!
//! **Zero new phi-core imports at P5** (pure phi governance
//! plane). The Permission Check preview surface from M5/P4 is
//! reused as a read-only input to `adopt`'s pre-check when future
//! work introduces adoption-time permission gates; for M5/P5 the
//! handlers delegate to the existing repo surface only.
//!
//! ## P5 advisory — D4.1 carry-forward
//!
//! Adopted templates mint grants at trigger-fire time (Template A
//! at `HAS_LEAD`, Template C at `MANAGES`, Template D at
//! `HAS_AGENT_SUPERVISOR`). Those grants are then used as
//! Permission-Check inputs at session-launch time. At M5 the
//! launch chain gates on Step 0 (Catalogue) only; steps 1-6 are
//! advisory (drift D4.1 in the plan archive). So even a fully
//! adopted Template C / D won't change the *launch* outcome at
//! M5 — it changes the *trace*. M6+ tightens the gate.

use domain::model::ids::{AuthRequestId, OrgId};
use domain::model::nodes::TemplateKind;
use domain::repository::RepositoryError;

pub mod adopt;
pub mod approve;
pub mod audit_events;
pub mod deny;
pub mod list;
pub mod revoke;

pub use adopt::{adopt_template_inline, AdoptInput, AdoptOutcome};
pub use approve::{approve_adoption_ar, ApproveInput, ApproveOutcome};
pub use deny::{deny_adoption_ar, DenyInput, DenyOutcome};
pub use list::{list_templates_for_org, TemplatesListing};
pub use revoke::{revoke_template, RevokeInput, RevokeOutcome};

/// Stable error enum for every template-surface handler. Each
/// variant maps 1:1 to a wire code.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("TEMPLATE_INPUT_INVALID: {0}")]
    InputInvalid(String),

    /// The requested `:kind` is not adoptable through this surface
    /// (e.g. `SystemBootstrap` is claim-flow only; `F` is
    /// break-glass only and reserved for M6).
    #[error("TEMPLATE_KIND_NOT_ADOPTABLE: {0:?}")]
    KindNotAdoptable(TemplateKind),

    /// `:kind` = E — Template E is "always available on demand"
    /// per R-ADMIN-12-R3 and has no adoption AR; adopt/approve/
    /// deny/revoke all refuse.
    #[error("TEMPLATE_E_ALWAYS_AVAILABLE")]
    TemplateEAlwaysAvailable,

    /// No adoption AR exists for (org, kind). Surfaces on approve /
    /// deny / revoke — the adopt path returns this as a green path
    /// (creates a new AR).
    #[error("TEMPLATE_ADOPTION_NOT_FOUND: org {org} / kind {kind:?}")]
    AdoptionNotFound { org: OrgId, kind: TemplateKind },

    /// An adoption AR for (org, kind) already exists in a non-terminal
    /// state; caller should approve/deny it rather than adopt fresh.
    #[error("TEMPLATE_ADOPTION_ALREADY_PENDING: {0}")]
    AdoptionAlreadyPending(AuthRequestId),

    /// An adoption AR for (org, kind) is already Active (Approved).
    /// Adopt refuses — the caller should revoke first if they want
    /// to re-adopt.
    #[error("TEMPLATE_ADOPTION_ALREADY_ACTIVE: {0}")]
    AdoptionAlreadyActive(AuthRequestId),

    /// Adoption AR is already terminal (Approved / Denied /
    /// Revoked / Expired / Cancelled / Partial) — approve / deny
    /// refuse with 409. Revoke accepts only Approved as input.
    #[error("TEMPLATE_ADOPTION_TERMINAL: adoption {ar} is in terminal state {state}")]
    AdoptionTerminal {
        ar: AuthRequestId,
        state: &'static str,
    },

    /// Caller is not the org's CEO / admin — R-ADMIN-12 §7.
    #[error("TEMPLATE_ADOPT_FORBIDDEN: org {0}")]
    Forbidden(OrgId),

    /// Org not found.
    #[error("ORG_NOT_FOUND: {0}")]
    OrgNotFound(OrgId),

    /// Underlying AR state-machine rejected the transition (race;
    /// another admin flipped the state between read and update).
    #[error("AR_STATE_TRANSITION_FAILED: {0}")]
    StateTransitionFailed(String),

    /// Pass-throughs.
    #[error("repository error: {0}")]
    Repository(String),
    #[error("audit emit error: {0}")]
    AuditEmit(String),
}

impl From<RepositoryError> for TemplateError {
    fn from(e: RepositoryError) -> Self {
        TemplateError::Repository(e.to_string())
    }
}

pub fn http_status_for(err: &TemplateError) -> u16 {
    match err {
        TemplateError::InputInvalid(_) | TemplateError::TemplateEAlwaysAvailable => 400,
        TemplateError::Forbidden(_) => 403,
        TemplateError::OrgNotFound(_) | TemplateError::AdoptionNotFound { .. } => 404,
        TemplateError::KindNotAdoptable(_)
        | TemplateError::AdoptionAlreadyPending(_)
        | TemplateError::AdoptionAlreadyActive(_)
        | TemplateError::AdoptionTerminal { .. }
        | TemplateError::StateTransitionFailed(_) => 409,
        TemplateError::Repository(_) | TemplateError::AuditEmit(_) => 500,
    }
}

pub fn wire_code_for(err: &TemplateError) -> &'static str {
    match err {
        TemplateError::InputInvalid(_) => "TEMPLATE_INPUT_INVALID",
        TemplateError::KindNotAdoptable(_) => "TEMPLATE_KIND_NOT_ADOPTABLE",
        TemplateError::TemplateEAlwaysAvailable => "TEMPLATE_E_ALWAYS_AVAILABLE",
        TemplateError::AdoptionNotFound { .. } => "TEMPLATE_ADOPTION_NOT_FOUND",
        TemplateError::AdoptionAlreadyPending(_) => "TEMPLATE_ADOPTION_ALREADY_PENDING",
        TemplateError::AdoptionAlreadyActive(_) => "TEMPLATE_ADOPTION_ALREADY_ACTIVE",
        TemplateError::AdoptionTerminal { .. } => "TEMPLATE_ADOPTION_TERMINAL",
        TemplateError::Forbidden(_) => "TEMPLATE_ADOPT_FORBIDDEN",
        TemplateError::OrgNotFound(_) => "ORG_NOT_FOUND",
        TemplateError::StateTransitionFailed(_) => "AR_STATE_TRANSITION_FAILED",
        TemplateError::Repository(_) => "REPOSITORY_ERROR",
        TemplateError::AuditEmit(_) => "AUDIT_EMIT_ERROR",
    }
}

/// Check whether `kind` is adoptable via the page-12 surface.
/// Adoptable kinds: A, B, C, D. Non-adoptable: SystemBootstrap
/// (claim flow), E (always available), F (break-glass; M6).
pub fn is_adoptable_kind(kind: TemplateKind) -> bool {
    matches!(
        kind,
        TemplateKind::A | TemplateKind::B | TemplateKind::C | TemplateKind::D
    )
}

/// Find the most-recent adoption AR for (org, kind) via the
/// standard `#template:<kind>` tag filter.
///
/// Multiple ARs can exist for a kind over an org's lifetime
/// (adopt → revoke → re-adopt). This returns the most recently
/// `submitted_at` entry. Terminal states are included — the caller
/// disambiguates by inspecting `ar.state`.
pub async fn find_adoption_ar(
    repo: &dyn domain::Repository,
    org: OrgId,
    kind: TemplateKind,
) -> Result<Option<domain::model::nodes::AuthRequest>, TemplateError> {
    let ars = repo.list_adoption_auth_requests_for_org(org).await?;
    let expected_tag = format!("#template:{}", kind.as_str());
    let mut matches: Vec<_> = ars
        .into_iter()
        .filter(|ar| ar.kinds.iter().any(|k| k == &expected_tag))
        .collect();
    matches.sort_by_key(|ar| std::cmp::Reverse(ar.submitted_at));
    Ok(matches.into_iter().next())
}
