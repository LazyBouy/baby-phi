//! Thin wrapper around `domain::permissions::check` that every M2+
//! write handler calls before touching the repository.
//!
//! Three responsibilities:
//!   1. Invoke the engine with the caller's manifest + context.
//!   2. Record the per-check timing sample via `PermissionCheckMetrics`.
//!   3. Map the [`Decision`] to either `Ok(Vec<ResolvedReach>)` or an
//!      [`ApiError`] — the HTTP mapping table is pinned by plan decision
//!      D10 in the archived M2 plan.
//!
//! Exhaustive mapping (D10):
//!
//! | `Decision` | HTTP | `code` |
//! |---|---|---|
//! | `Allowed { resolved_grants }` | — (returns `Ok`) | — |
//! | `Pending { awaiting_consent }` | 202 | `AWAITING_CONSENT` |
//! | `Denied { Catalogue }` | 403 | `CATALOGUE_MISS` |
//! | `Denied { Expansion }` | 400 | `MANIFEST_EMPTY` |
//! | `Denied { Resolution }` | 403 | `NO_GRANTS_HELD` |
//! | `Denied { Ceiling }` | 403 | `CEILING_EMPTIED` |
//! | `Denied { Match }` | 403 | `NO_MATCHING_GRANT` |
//! | `Denied { Constraint }` | 403 | `CONSTRAINT_VIOLATION` |
//! | `Denied { Scope }` | 403 | `SCOPE_UNRESOLVABLE` |
//! | `Denied { Consent }` | 202 | `AWAITING_CONSENT` |
//!
//! Stable codes land in [`modules/web/lib/api/errors.ts`] so the UI
//! can render operator-friendly hints.

use axum::http::StatusCode;
use domain::permissions::{
    check,
    decision::{Decision, FailedStep, ResolvedReach},
    manifest::{CheckContext, Manifest},
    metrics::PermissionCheckMetrics,
};

use super::errors::ApiError;

/// Invoke the engine and map the decision.
///
/// Returns:
/// - `Ok(resolved_grants)` on `Decision::Allowed` — callers typically
///   need the grant ids to carry forward as `descends_from` on any
///   grants they issue.
/// - `Err(ApiError)` on `Denied` or `Pending`, with HTTP status +
///   stable code per the mapping table above.
pub fn check_permission(
    ctx: &CheckContext<'_>,
    manifest: &Manifest,
    metrics: &dyn PermissionCheckMetrics,
) -> Result<Vec<ResolvedReach>, ApiError> {
    let decision = check(ctx, manifest, metrics);
    match decision {
        Decision::Allowed { resolved_grants } => Ok(resolved_grants),
        Decision::Pending {
            awaiting_consent: _,
        } => Err(ApiError::new(
            StatusCode::ACCEPTED,
            "AWAITING_CONSENT",
            "subordinate consent required before this write can proceed",
        )),
        Decision::Denied {
            failed_step,
            reason,
        } => Err(denial_to_api_error(failed_step, &reason)),
    }
}

/// Map a `(FailedStep, DeniedReason)` pair to its stable `ApiError`.
/// Kept as a dedicated function so the `handler_support_test` suite
/// can assert the mapping is exhaustive (one distinct code per step).
///
/// The `reason` is used only to produce a more informative human
/// message — the stable `code` is driven by `failed_step` alone so the
/// UI's hint table stays stable.
pub fn denial_to_api_error(
    failed_step: FailedStep,
    reason: &domain::permissions::decision::DeniedReason,
) -> ApiError {
    use domain::permissions::decision::DeniedReason;
    let message = match reason {
        DeniedReason::CatalogueMiss { resource_uri } => {
            format!("resource `{resource_uri}` is not in the catalogue")
        }
        DeniedReason::ManifestEmpty => "manifest declared no resources or actions".to_string(),
        DeniedReason::NoGrantsHeld => "no grants are held against this resource".to_string(),
        DeniedReason::CeilingEmptied => {
            "every candidate grant was clamped out by the resource ceiling".to_string()
        }
        DeniedReason::NoMatchingGrant {
            fundamental,
            action,
        } => {
            format!("no grant covers `{fundamental:?}` for action `{action}`")
        }
        DeniedReason::ConstraintViolation { constraint, .. } => {
            format!("constraint `{constraint}` was not satisfied")
        }
        DeniedReason::ScopeUnresolvable {
            fundamental,
            action,
        } => {
            format!("scope cascade could not pick a winner for `{fundamental:?}`/`{action}`")
        }
    };

    match failed_step {
        FailedStep::Catalogue => ApiError::new(StatusCode::FORBIDDEN, "CATALOGUE_MISS", message),
        FailedStep::Expansion => ApiError::new(StatusCode::BAD_REQUEST, "MANIFEST_EMPTY", message),
        FailedStep::Resolution => ApiError::new(StatusCode::FORBIDDEN, "NO_GRANTS_HELD", message),
        FailedStep::Ceiling => ApiError::new(StatusCode::FORBIDDEN, "CEILING_EMPTIED", message),
        FailedStep::Match => ApiError::new(StatusCode::FORBIDDEN, "NO_MATCHING_GRANT", message),
        FailedStep::Constraint => {
            ApiError::new(StatusCode::FORBIDDEN, "CONSTRAINT_VIOLATION", message)
        }
        FailedStep::Scope => ApiError::new(StatusCode::FORBIDDEN, "SCOPE_UNRESOLVABLE", message),
        // `Consent` only surfaces via `Decision::Pending` in practice;
        // the branch is here for exhaustiveness so adding a new
        // FailedStep variant breaks compilation rather than producing a
        // silent default.
        FailedStep::Consent => ApiError::new(StatusCode::ACCEPTED, "AWAITING_CONSENT", message),
    }
}
