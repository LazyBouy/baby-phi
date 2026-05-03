//! Transition operations on a [`Consent`].
//!
//! Every public function here takes an immutable `&Consent` and returns
//! `Result<Consent, ConsentTransitionError>` — state never mutates in
//! place. That keeps the API pure, proptest-friendly, and trivially
//! rollbackable when the caller hits a constraint violation downstream.
//!
//! All transitions respect the legal-transition table in
//! [`super::state::legal_transition`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::ids::ConsentId;
use crate::model::nodes::{Consent, ConsentState};

use super::state::legal_transition;

/// Structured error for rejected Consent transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConsentTransitionError {
    /// The requested transition is not in the legal-transition table.
    /// Covers terminal stickiness + forward-only revocation + every
    /// other illegal pair.
    #[error("illegal consent transition {from:?} → {to:?}")]
    IllegalTransition {
        from: ConsentState,
        to: ConsentState,
    },
    /// `Acknowledged → Revoked` was requested on a Consent whose
    /// `revocable == false`. Per concept doc 06 §"Revocation semantics":
    /// the subordinate cannot withdraw a non-revocable consent.
    #[error("consent {consent_id} is not revocable")]
    NotRevocable { consent_id: ConsentId },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn enter_state(consent: &Consent, to: ConsentState) -> Result<Consent, ConsentTransitionError> {
    if !legal_transition(consent.state, to) {
        return Err(ConsentTransitionError::IllegalTransition {
            from: consent.state,
            to,
        });
    }
    let mut next = consent.clone();
    next.state = to;
    Ok(next)
}

// ---------------------------------------------------------------------------
// acknowledge — Requested → Acknowledged
// ---------------------------------------------------------------------------

/// Move a `Requested` consent to `Acknowledged`. Stamps `responded_at`
/// to `at` (concept doc 06 line 360 — the timestamp is "when the
/// subordinate acknowledged or declined").
pub fn acknowledge(
    consent: &Consent,
    at: DateTime<Utc>,
) -> Result<Consent, ConsentTransitionError> {
    let mut next = enter_state(consent, ConsentState::Acknowledged)?;
    next.responded_at = Some(at);
    Ok(next)
}

// ---------------------------------------------------------------------------
// decline — Requested → Declined
// ---------------------------------------------------------------------------

/// Move a `Requested` consent to `Declined`. Stamps `responded_at` to
/// `at` (same field as `acknowledge`; the concept doc reuses it for
/// both directions of the response).
pub fn decline(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError> {
    let mut next = enter_state(consent, ConsentState::Declined)?;
    next.responded_at = Some(at);
    Ok(next)
}

// ---------------------------------------------------------------------------
// revoke — Acknowledged → Revoked (gated by `revocable`)
// ---------------------------------------------------------------------------

/// Move an `Acknowledged` consent to `Revoked`. Stamps `revoked_at` to
/// `at`. Rejects with `NotRevocable` when `consent.revocable == false`.
///
/// Forward-only revocation: once `Revoked`, no transition is legal.
/// `legal_transition` enforces this — `Revoked` has zero outbound arrows.
pub fn revoke(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError> {
    // Check the state-machine legality first (this catches Requested →
    // Revoked attempts, terminal stickiness, etc.). The `?` propagates
    // the IllegalTransition error before the revocable gate kicks in,
    // so the caller sees `IllegalTransition` (more informative) for an
    // already-Revoked / Declined / etc. consent rather than a misleading
    // `NotRevocable`.
    let mut next = enter_state(consent, ConsentState::Revoked)?;
    if !consent.revocable {
        return Err(ConsentTransitionError::NotRevocable {
            consent_id: consent.id,
        });
    }
    next.revoked_at = Some(at);
    Ok(next)
}

// ---------------------------------------------------------------------------
// mark_timed_out — Requested → TimedOut
// ---------------------------------------------------------------------------

/// Move a `Requested` consent to `TimedOut`. Called by the sweeper when
/// `consent.deadline_at <= now`. Does not stamp any timestamp field —
/// `requested_at` is immutable, `responded_at` stays `None` (the
/// subordinate did not respond, by definition), and `revoked_at` is for
/// revocation only.
pub fn mark_timed_out(
    consent: &Consent,
    _at: DateTime<Utc>,
) -> Result<Consent, ConsentTransitionError> {
    enter_state(consent, ConsentState::TimedOut)
}

// ---------------------------------------------------------------------------
// mark_expired — any non-terminal → Expired
// ---------------------------------------------------------------------------

/// Move a `Requested` or `Acknowledged` consent to `Expired`. The
/// natural-scope-end transition (concept doc 06 line 404 — "the Consent's
/// natural scope ended — e.g. a `per_session` consent outlives its
/// session"). Does not stamp any timestamp field.
pub fn mark_expired(
    consent: &Consent,
    _at: DateTime<Utc>,
) -> Result<Consent, ConsentTransitionError> {
    enter_state(consent, ConsentState::Expired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, ConsentId, OrgId};
    use crate::model::nodes::ConsentScope;

    fn fresh_consent(state: ConsentState, revocable: bool) -> Consent {
        let now = Utc::now();
        Consent {
            id: ConsentId::new(),
            agent_id: AgentId::new(),
            scope: ConsentScope {
                org: OrgId::new(),
                templates: vec![],
                actions: vec![],
                session_id: None,
            },
            state,
            requested_at: now,
            responded_at: None,
            revoked_at: None,
            revocable,
            provenance: "agent:test@unit".into(),
            deadline_at: None,
        }
    }

    // ---- acknowledge --------------------------------------------------------

    #[test]
    fn acknowledge_from_requested_sets_state_and_responded_at() {
        let c = fresh_consent(ConsentState::Requested, true);
        let at = Utc::now();
        let next = acknowledge(&c, at).expect("legal transition");
        assert_eq!(next.state, ConsentState::Acknowledged);
        assert_eq!(next.responded_at, Some(at));
        assert_eq!(next.revoked_at, None);
        // Original is unchanged (pure-fn invariant).
        assert_eq!(c.state, ConsentState::Requested);
        assert_eq!(c.responded_at, None);
    }

    #[test]
    fn acknowledge_from_non_requested_is_illegal() {
        for from in [
            ConsentState::Acknowledged,
            ConsentState::Declined,
            ConsentState::Revoked,
            ConsentState::TimedOut,
            ConsentState::Expired,
        ] {
            let c = fresh_consent(from, true);
            let err = acknowledge(&c, Utc::now()).expect_err("must reject");
            match err {
                ConsentTransitionError::IllegalTransition { from: f, to } => {
                    assert_eq!(f, from);
                    assert_eq!(to, ConsentState::Acknowledged);
                }
                other => panic!("expected IllegalTransition, got {other:?}"),
            }
        }
    }

    // ---- decline ------------------------------------------------------------

    #[test]
    fn decline_from_requested_sets_state_and_responded_at() {
        let c = fresh_consent(ConsentState::Requested, true);
        let at = Utc::now();
        let next = decline(&c, at).expect("legal transition");
        assert_eq!(next.state, ConsentState::Declined);
        assert_eq!(next.responded_at, Some(at));
    }

    #[test]
    fn decline_from_acknowledged_is_illegal() {
        let c = fresh_consent(ConsentState::Acknowledged, true);
        let err = decline(&c, Utc::now()).expect_err("must reject");
        assert!(matches!(
            err,
            ConsentTransitionError::IllegalTransition { .. }
        ));
    }

    // ---- revoke -------------------------------------------------------------

    #[test]
    fn revoke_from_acknowledged_revocable_sets_state_and_revoked_at() {
        let c = fresh_consent(ConsentState::Acknowledged, true);
        let at = Utc::now();
        let next = revoke(&c, at).expect("legal transition");
        assert_eq!(next.state, ConsentState::Revoked);
        assert_eq!(next.revoked_at, Some(at));
    }

    #[test]
    fn revoke_from_acknowledged_non_revocable_returns_not_revocable() {
        let c = fresh_consent(ConsentState::Acknowledged, false);
        let err = revoke(&c, Utc::now()).expect_err("must reject");
        match err {
            ConsentTransitionError::NotRevocable { consent_id } => {
                assert_eq!(consent_id, c.id);
            }
            other => panic!("expected NotRevocable, got {other:?}"),
        }
    }

    #[test]
    fn revoke_from_requested_is_illegal_not_not_revocable() {
        // Even with `revocable = false`, the state-machine legality check
        // happens first — the caller sees IllegalTransition (more
        // informative) for a Requested-state consent, not NotRevocable.
        let c = fresh_consent(ConsentState::Requested, false);
        let err = revoke(&c, Utc::now()).expect_err("must reject");
        assert!(matches!(
            err,
            ConsentTransitionError::IllegalTransition { .. }
        ));
    }

    #[test]
    fn revoke_from_revoked_is_illegal_forward_only() {
        // Forward-only revocation invariant.
        let c = fresh_consent(ConsentState::Revoked, true);
        let err = revoke(&c, Utc::now()).expect_err("must reject");
        assert!(matches!(
            err,
            ConsentTransitionError::IllegalTransition { .. }
        ));
    }

    // ---- mark_timed_out -----------------------------------------------------

    #[test]
    fn mark_timed_out_from_requested_succeeds() {
        let c = fresh_consent(ConsentState::Requested, true);
        let next = mark_timed_out(&c, Utc::now()).expect("legal transition");
        assert_eq!(next.state, ConsentState::TimedOut);
        // No timestamp field is updated by mark_timed_out.
        assert_eq!(next.responded_at, None);
        assert_eq!(next.revoked_at, None);
    }

    #[test]
    fn mark_timed_out_from_acknowledged_is_illegal() {
        // Per the legal table: Acknowledged → TimedOut is NOT legal.
        // The subordinate has already responded; timeout doesn't apply.
        let c = fresh_consent(ConsentState::Acknowledged, true);
        let err = mark_timed_out(&c, Utc::now()).expect_err("must reject");
        assert!(matches!(
            err,
            ConsentTransitionError::IllegalTransition { .. }
        ));
    }

    // ---- mark_expired -------------------------------------------------------

    #[test]
    fn mark_expired_from_requested_succeeds() {
        let c = fresh_consent(ConsentState::Requested, true);
        let next = mark_expired(&c, Utc::now()).expect("legal transition");
        assert_eq!(next.state, ConsentState::Expired);
    }

    #[test]
    fn mark_expired_from_acknowledged_succeeds() {
        let c = fresh_consent(ConsentState::Acknowledged, true);
        let next = mark_expired(&c, Utc::now()).expect("legal transition");
        assert_eq!(next.state, ConsentState::Expired);
    }

    #[test]
    fn mark_expired_from_terminal_is_illegal() {
        // Already-terminal states reject every transition (including
        // Expired itself, since self-transitions are illegal).
        for from in [
            ConsentState::Declined,
            ConsentState::Revoked,
            ConsentState::TimedOut,
            ConsentState::Expired,
        ] {
            let c = fresh_consent(from, true);
            let err = mark_expired(&c, Utc::now()).expect_err("must reject");
            assert!(matches!(
                err,
                ConsentTransitionError::IllegalTransition { .. }
            ));
        }
    }

    // ---- exhaustive matrix --------------------------------------------------

    #[test]
    fn exhaustive_36_cell_matrix_matches_legal_transition_table() {
        // For every (from, to) pair in the 36-cell matrix, the result of
        // attempting the transition (via the matching pure-fn when one
        // exists for `to`) matches `legal_transition(from, to)`.
        for from in ConsentState::ALL {
            for to in ConsentState::ALL {
                let c = fresh_consent(from, true);
                let pure_fn_result = match to {
                    ConsentState::Acknowledged => acknowledge(&c, Utc::now()).is_ok(),
                    ConsentState::Declined => decline(&c, Utc::now()).is_ok(),
                    ConsentState::Revoked => revoke(&c, Utc::now()).is_ok(),
                    ConsentState::TimedOut => mark_timed_out(&c, Utc::now()).is_ok(),
                    ConsentState::Expired => mark_expired(&c, Utc::now()).is_ok(),
                    // No pure-fn moves INTO Requested (it's the entry
                    // state, set at consent creation, never via
                    // transition).
                    ConsentState::Requested => false,
                };
                let table_says = legal_transition(from, to) && to != ConsentState::Requested;
                assert_eq!(
                    pure_fn_result, table_says,
                    "{from:?} → {to:?}: pure_fn={pure_fn_result}, table={table_says}"
                );
            }
        }
    }
}
