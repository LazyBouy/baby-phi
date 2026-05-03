//! Per-policy [`Consent`] minter helpers (CH-11 / ADR-0048 §D48.8).
//!
//! Pure-fn constructors that build a freshly-stamped `Requested` consent
//! row for the launch handler to persist. Two minters cover the two
//! policies the engine's Step 6 surfaces a `Pending` decision for:
//!
//! - [`request_one_time`] — `(subordinate, org, None)` axis. One-time
//!   consent applies across every session in the org until revoked /
//!   expired.
//! - [`request_per_session`] — `(subordinate, org, Some(session_id))`
//!   axis. Per-session consent applies only to reads on the named
//!   session.
//!
//! ## Provenance string
//!
//! Each minter stamps `provenance = "engine:step_6@<RFC-3339>"` so audit
//! reviewers can trace the row back to a Step 6 surfacing event. The
//! timestamp is the same `requested_at` used on the row + the engine's
//! event clock (the launch handler passes `Utc::now()` — minters
//! materialise both in lock-step).
//!
//! ## phi-core leverage
//!
//! Zero. Consent is a phi-only governance primitive (see
//! `concepts/phi-core-mapping.md`).

use chrono::{DateTime, Utc};

use crate::model::ids::{AgentId, ConsentId, OrgId, SessionId};
use crate::model::nodes::{Consent, ConsentScope, ConsentState};

/// Mint a `Requested` `OneTime`-policy [`Consent`] row.
///
/// `scope.session_id == None` — the consent is not session-scoped (it
/// applies to every session under the org until terminal).
///
/// `revocable = true` per concept doc 06 §"Revocation semantics" — a
/// freshly-requested consent the subordinate has not yet acted on is
/// always revocable.
///
/// `deadline_at` is the caller-computed timeout boundary (CH-10's
/// sweeper auto-flips `Requested` rows whose `deadline_at <= now` to
/// `TimedOut`). `None` means no auto-timeout — the row sits as
/// `Requested` until the subordinate acks/declines or the row is
/// otherwise terminal.
pub fn request_one_time(
    subordinate: AgentId,
    org: OrgId,
    deadline_at: Option<DateTime<Utc>>,
) -> Consent {
    let now = Utc::now();
    Consent {
        id: ConsentId::new(),
        agent_id: subordinate,
        scope: ConsentScope {
            org,
            templates: Vec::new(),
            actions: Vec::new(),
            session_id: None,
        },
        state: ConsentState::Requested,
        requested_at: now,
        responded_at: None,
        revoked_at: None,
        revocable: true,
        provenance: format!("engine:step_6@{}", now.to_rfc3339()),
        deadline_at,
    }
}

/// Mint a `Requested` `PerSession`-policy [`Consent`] row.
///
/// `scope.session_id == Some(session_id)` — the consent applies only to
/// reads on the named session. A second session under the same
/// `(subordinate, org)` requires its own ack.
///
/// `revocable = true` per concept doc 06 §"Revocation semantics".
///
/// `deadline_at` is the caller-computed timeout boundary (see
/// [`request_one_time`] for the sweeper semantics).
pub fn request_per_session(
    subordinate: AgentId,
    org: OrgId,
    session_id: SessionId,
    deadline_at: Option<DateTime<Utc>>,
) -> Consent {
    let now = Utc::now();
    Consent {
        id: ConsentId::new(),
        agent_id: subordinate,
        scope: ConsentScope {
            org,
            templates: Vec::new(),
            actions: Vec::new(),
            session_id: Some(session_id),
        },
        state: ConsentState::Requested,
        requested_at: now,
        responded_at: None,
        revoked_at: None,
        revocable: true,
        provenance: format!("engine:step_6@{}", now.to_rfc3339()),
        deadline_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn request_one_time_carries_requested_state_and_no_session_axis() {
        let sub = AgentId::new();
        let org = OrgId::new();
        let consent = request_one_time(sub, org, None);
        assert_eq!(consent.state, ConsentState::Requested);
        assert_eq!(consent.agent_id, sub);
        assert_eq!(consent.scope.org, org);
        assert_eq!(consent.scope.session_id, None);
        assert!(consent.revocable);
        assert!(consent.responded_at.is_none());
        assert!(consent.revoked_at.is_none());
        assert!(consent.deadline_at.is_none());
        assert!(consent.provenance.starts_with("engine:step_6@"));
    }

    #[test]
    fn request_per_session_carries_session_axis_in_scope() {
        let sub = AgentId::new();
        let org = OrgId::new();
        let session = SessionId::new();
        let consent = request_per_session(sub, org, session, None);
        assert_eq!(consent.state, ConsentState::Requested);
        assert_eq!(consent.scope.org, org);
        assert_eq!(consent.scope.session_id, Some(session));
        assert_eq!(consent.agent_id, sub);
        assert!(consent.provenance.starts_with("engine:step_6@"));
    }

    #[test]
    fn minters_propagate_caller_supplied_deadline_at() {
        let sub = AgentId::new();
        let org = OrgId::new();
        let session = SessionId::new();
        let deadline = Utc::now() + Duration::hours(24);
        let one_time = request_one_time(sub, org, Some(deadline));
        let per_session = request_per_session(sub, org, session, Some(deadline));
        assert_eq!(one_time.deadline_at, Some(deadline));
        assert_eq!(per_session.deadline_at, Some(deadline));
    }

    #[test]
    fn minters_emit_unique_consent_ids_per_call() {
        let sub = AgentId::new();
        let org = OrgId::new();
        let a = request_one_time(sub, org, None);
        let b = request_one_time(sub, org, None);
        assert_ne!(a.id, b.id, "consent ids must be unique per mint");
    }
}
