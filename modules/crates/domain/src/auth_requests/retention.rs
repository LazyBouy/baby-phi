//! Active-window math for the two-tier Auth Request retention policy.
//!
//! Per `concepts/permissions/02` §Retention Policy:
//!
//! - **Active** — retained in the hot path; queryable in normal graph
//!   queries. Terminal Auth Requests are active for `active_window_days`
//!   after entering their terminal state. Default 90 days.
//! - **Archived** — after the active window, the request moves to cold
//!   storage. Still auditable via `inspect_archived`; never deleted.
//! - Compliance-only deletion (`delete_after_years`) is off by default
//!   and not modelled in M1.
//!
//! The module is pure — every function takes the inputs it needs and
//! returns `Option<DateTime<Utc>>` / `bool`. P5 bootstrap wires this
//! into the repository.

use chrono::{DateTime, Duration, Utc};

use crate::model::nodes::AuthRequest;

/// Per-request active window in days (wraps `AuthRequest.active_window_days`).
#[derive(Debug, Clone, Copy)]
pub struct ActiveWindow {
    pub days: u32,
}

impl ActiveWindow {
    pub fn from_request(req: &AuthRequest) -> Self {
        Self {
            days: req.active_window_days,
        }
    }
}

/// The moment this request leaves the active window and becomes
/// archive-eligible.
///
/// - Returns `None` for non-terminal requests (no active-window
///   countdown has started).
/// - Returns `Some(terminal_entry + active_window_days)` for terminal
///   requests.
pub fn active_until(req: &AuthRequest) -> Option<DateTime<Utc>> {
    let entered = req.terminal_state_entered_at?;
    Some(entered + Duration::days(req.active_window_days as i64))
}

/// Is this request eligible to move from the active tier to the archived
/// tier at `now`?
///
/// Per the concept doc, a request stays active if any of:
/// - still in a non-terminal state,
/// - has a live grant attached (caller's responsibility to supply via
///   `has_live_grants`),
/// - the active window has not yet elapsed.
pub fn is_archive_eligible(req: &AuthRequest, now: DateTime<Utc>, has_live_grants: bool) -> bool {
    if req.archived {
        // Already archived — not eligible for another move.
        return false;
    }
    if has_live_grants {
        return false;
    }
    let Some(cutoff) = active_until(req) else {
        return false;
    };
    now >= cutoff
}

/// Days remaining in the active window, or `None` for pre-terminal
/// requests. Never returns a negative value — zero once the window has
/// elapsed. Used by observability dashboards and by the monotonicity
/// proptest.
pub fn days_remaining(req: &AuthRequest, now: DateTime<Utc>) -> Option<i64> {
    let cutoff = active_until(req)?;
    let secs = (cutoff - now).num_seconds();
    Some((secs / 86_400).max(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::AuthRequestId;
    use crate::model::nodes::{
        AuthRequest, AuthRequestState, PrincipalRef, ResourceSlot, ResourceSlotState,
    };

    fn terminal_req(state: AuthRequestState, terminal_at: DateTime<Utc>, days: u32) -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor: PrincipalRef::System("system:test".into()),
            kinds: vec![],
            scope: vec![],
            state,
            valid_until: None,
            submitted_at: terminal_at,
            resource_slots: vec![ResourceSlot {
                resource: crate::model::nodes::ResourceRef {
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

    #[test]
    fn active_until_none_for_non_terminal_request() {
        let mut req = terminal_req(AuthRequestState::Approved, Utc::now(), 90);
        req.terminal_state_entered_at = None;
        req.state = AuthRequestState::Pending;
        assert!(active_until(&req).is_none());
    }

    #[test]
    fn active_until_returns_terminal_plus_window() {
        let at = Utc::now();
        let req = terminal_req(AuthRequestState::Approved, at, 90);
        let cutoff = active_until(&req).unwrap();
        let delta = cutoff - at;
        assert_eq!(delta.num_days(), 90);
    }

    #[test]
    fn archive_not_eligible_when_still_within_window() {
        let at = Utc::now();
        let req = terminal_req(AuthRequestState::Approved, at, 90);
        assert!(!is_archive_eligible(&req, at + Duration::days(1), false));
        assert!(!is_archive_eligible(&req, at + Duration::days(89), false));
    }

    #[test]
    fn archive_eligible_after_window() {
        let at = Utc::now();
        let req = terminal_req(AuthRequestState::Approved, at, 90);
        assert!(is_archive_eligible(&req, at + Duration::days(91), false));
    }

    #[test]
    fn archive_not_eligible_when_live_grants_attached() {
        let at = Utc::now();
        let req = terminal_req(AuthRequestState::Approved, at, 90);
        assert!(!is_archive_eligible(&req, at + Duration::days(91), true));
    }

    #[test]
    fn archive_not_eligible_when_already_archived() {
        let at = Utc::now();
        let mut req = terminal_req(AuthRequestState::Approved, at, 90);
        req.archived = true;
        assert!(!is_archive_eligible(&req, at + Duration::days(91), false));
    }

    #[test]
    fn days_remaining_monotonically_non_increasing_over_time() {
        let at = Utc::now();
        let req = terminal_req(AuthRequestState::Approved, at, 90);
        let d0 = days_remaining(&req, at).unwrap();
        let d30 = days_remaining(&req, at + Duration::days(30)).unwrap();
        let d90 = days_remaining(&req, at + Duration::days(90)).unwrap();
        let d120 = days_remaining(&req, at + Duration::days(120)).unwrap();
        assert!(d0 >= d30);
        assert!(d30 >= d90);
        assert!(d90 >= d120);
        assert_eq!(
            d120, 0,
            "past the window the remaining days are clamped to 0"
        );
    }
}
