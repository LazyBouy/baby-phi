//! Exhaustive mapping test for `handler_support::permission::denial_to_api_error`.
//!
//! Every `FailedStep` variant must map to a distinct, stable
//! `ApiError.code`. This asserts the D10 / ADR-0018 contract so the
//! web tier's hint table (lib/api/errors.ts) stays in sync.

use domain::model::ids::GrantId;
use domain::model::Fundamental;
use domain::permissions::decision::{DeniedReason, FailedStep};
use server::handler_support::permission::denial_to_api_error;

/// Every variant of FailedStep — used to assert exhaustive coverage.
/// Adding a new variant breaks this list at compile time.
fn all_failed_steps() -> Vec<FailedStep> {
    vec![
        FailedStep::Catalogue,
        FailedStep::Expansion,
        FailedStep::Resolution,
        FailedStep::Ceiling,
        FailedStep::Match,
        FailedStep::Constraint,
        FailedStep::Scope,
        FailedStep::Consent,
    ]
}

fn sample_reason_for(step: FailedStep) -> DeniedReason {
    match step {
        FailedStep::Catalogue => DeniedReason::CatalogueMiss {
            resource_uri: "system:ghost".into(),
        },
        FailedStep::Expansion => DeniedReason::ManifestEmpty,
        FailedStep::Resolution => DeniedReason::NoGrantsHeld,
        FailedStep::Ceiling => DeniedReason::CeilingEmptied,
        FailedStep::Match => DeniedReason::NoMatchingGrant {
            fundamental: Fundamental::FilesystemObject,
            action: "read".into(),
        },
        FailedStep::Constraint => DeniedReason::ConstraintViolation {
            constraint: "purpose".into(),
            grant_id: GrantId::new(),
        },
        FailedStep::Scope => DeniedReason::ScopeUnresolvable {
            fundamental: Fundamental::DataObject,
            action: "read".into(),
        },
        // There is no `DeniedReason::ConsentMissing` variant — consent
        // flows surface via `Decision::Pending`. Reuse `NoGrantsHeld`
        // as a placeholder for the exhaustiveness probe; the mapping
        // under test only switches on `FailedStep`.
        FailedStep::Consent => DeniedReason::NoGrantsHeld,
    }
}

#[test]
fn every_failed_step_has_a_distinct_code() {
    let mut seen = std::collections::HashSet::new();
    for step in all_failed_steps() {
        let reason = sample_reason_for(step);
        let err = denial_to_api_error(step, &reason);
        assert!(
            !err.code.is_empty(),
            "FailedStep::{:?} must map to a non-empty code",
            step
        );
        assert!(
            seen.insert(err.code),
            "FailedStep::{:?} emitted a duplicate code: {}",
            step,
            err.code
        );
    }
    assert_eq!(seen.len(), all_failed_steps().len());
}

#[test]
fn mapping_is_stable_d10_contract() {
    // Pin the exact codes per plan decision D10. These are the values
    // the web tier's lib/api/errors.ts depends on.
    let table: &[(FailedStep, &str, u16)] = &[
        (FailedStep::Catalogue, "CATALOGUE_MISS", 403),
        (FailedStep::Expansion, "MANIFEST_EMPTY", 400),
        (FailedStep::Resolution, "NO_GRANTS_HELD", 403),
        (FailedStep::Ceiling, "CEILING_EMPTIED", 403),
        (FailedStep::Match, "NO_MATCHING_GRANT", 403),
        (FailedStep::Constraint, "CONSTRAINT_VIOLATION", 403),
        (FailedStep::Scope, "SCOPE_UNRESOLVABLE", 403),
        (FailedStep::Consent, "AWAITING_CONSENT", 202),
    ];
    for (step, expected_code, expected_status) in table {
        let reason = sample_reason_for(*step);
        let err = denial_to_api_error(*step, &reason);
        assert_eq!(
            err.code, *expected_code,
            "FailedStep::{:?} code mismatch",
            step
        );
        assert_eq!(
            err.status.as_u16(),
            *expected_status,
            "FailedStep::{:?} status mismatch",
            step
        );
    }
}

#[test]
fn mapping_messages_include_deniedreason_detail() {
    let err = denial_to_api_error(
        FailedStep::Catalogue,
        &DeniedReason::CatalogueMiss {
            resource_uri: "system:specific-uri".into(),
        },
    );
    assert!(
        err.message.contains("system:specific-uri"),
        "expected the URI in the error message, got: {}",
        err.message
    );
}
