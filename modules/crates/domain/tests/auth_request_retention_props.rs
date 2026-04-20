//! Property tests for [`domain::auth_requests::retention`].
//!
//! Invariants covered:
//!
//! 1. `days_remaining_is_monotonically_non_increasing_over_time` — for
//!    any terminal request, `days_remaining(now=t₀) ≥ days_remaining(now=t₁)`
//!    whenever t₀ ≤ t₁.
//! 2. `active_window_cutoff_equals_terminal_entry_plus_days` — the
//!    cutoff is exactly `terminal_state_entered_at + active_window_days`.
//! 3. `archive_eligible_requires_expired_window_and_no_live_grants` —
//!    `is_archive_eligible` respects the two-part AND: window elapsed
//!    AND no live grants remain.

use chrono::{DateTime, Duration, Utc};
use domain::audit::AuditClass;
use domain::auth_requests::retention::{active_until, days_remaining, is_archive_eligible};
use domain::model::ids::AuthRequestId;
use domain::model::nodes::{
    AuthRequest, AuthRequestState, PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState,
};

use proptest::prelude::*;

fn base_terminal(state: AuthRequestState, terminal_at: DateTime<Utc>, days: u32) -> AuthRequest {
    AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::System("system:test".into()),
        kinds: vec![],
        scope: vec![],
        state,
        valid_until: None,
        submitted_at: terminal_at,
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "test:r".into(),
            },
            approvers: vec![],
            state: ResourceSlotState::Approved,
        }],
        justification: None,
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: Some(terminal_at),
        archived: false,
        active_window_days: days,
        provenance_template: None,
    }
}

proptest! {
    /// For increasing `now`, days_remaining never goes up.
    #[test]
    fn days_remaining_is_monotonically_non_increasing_over_time(
        days in 1u32..400,
        elapsed_a in 0i64..500,
        delta in 0i64..500,
    ) {
        let terminal_at = Utc::now();
        let req = base_terminal(AuthRequestState::Approved, terminal_at, days);
        let t0 = terminal_at + Duration::days(elapsed_a);
        let t1 = t0 + Duration::days(delta);
        let d0 = days_remaining(&req, t0).unwrap();
        let d1 = days_remaining(&req, t1).unwrap();
        prop_assert!(d0 >= d1,
            "expected non-increasing: d0 = {d0}, d1 = {d1}, t0 = {t0}, t1 = {t1}");
        // Also: both values are non-negative.
        prop_assert!(d0 >= 0);
        prop_assert!(d1 >= 0);
    }

    /// The cutoff returned by `active_until` equals
    /// `terminal_state_entered_at + active_window_days`.
    #[test]
    fn active_window_cutoff_equals_terminal_entry_plus_days(
        days in 1u32..400,
    ) {
        let at = Utc::now();
        let req = base_terminal(AuthRequestState::Approved, at, days);
        let cutoff = active_until(&req).unwrap();
        let delta = cutoff - at;
        prop_assert_eq!(delta.num_days(), days as i64);
    }

    /// Archive eligibility is a two-part AND: window elapsed AND no live
    /// grants. Taking out either piece must flip the result to false.
    #[test]
    fn archive_eligible_respects_both_conditions(
        days in 1u32..180,
        elapsed in 0i64..400,
        has_live_grants in prop::bool::ANY,
    ) {
        let terminal_at = Utc::now();
        let req = base_terminal(AuthRequestState::Approved, terminal_at, days);
        let now = terminal_at + Duration::days(elapsed);
        let window_elapsed = elapsed >= days as i64;
        let got = is_archive_eligible(&req, now, has_live_grants);
        let expected = window_elapsed && !has_live_grants;
        prop_assert_eq!(got, expected);
    }

    /// Pre-terminal requests are never archive-eligible.
    #[test]
    fn pre_terminal_request_is_never_archive_eligible(
        elapsed_days in 0i64..365,
        has_live_grants in prop::bool::ANY,
    ) {
        let mut req = base_terminal(AuthRequestState::Approved, Utc::now(), 90);
        // Remove the terminal-entry stamp to simulate a pre-terminal request.
        req.terminal_state_entered_at = None;
        req.state = AuthRequestState::InProgress;
        let now = Utc::now() + Duration::days(elapsed_days);
        let got = is_archive_eligible(&req, now, has_live_grants);
        prop_assert!(!got,
            "pre-terminal requests must never be archive-eligible (elapsed={elapsed_days})");
    }
}
