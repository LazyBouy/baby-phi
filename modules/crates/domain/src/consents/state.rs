//! Pure predicates for the Consent state machine — terminal-state checks +
//! the legal-transition table.
//!
//! Mirrors the `auth_requests::state` precedent. Every transition pure-fn
//! in [`super::transitions`] consults [`legal_transition`] before mutating;
//! the [`is_terminal`] predicate gates the higher-level "callers want to
//! check before invoking" question.

use crate::model::nodes::ConsentState;

/// True for every terminal state.
///
/// Per concept doc 06 §"Consent Lifecycle" lines 397–404, the terminal
/// states are `Declined`, `Revoked`, `TimedOut`, and `Expired`. None of
/// them have outbound arrows (forward-only revocation, no
/// reconsideration). `Requested` and `Acknowledged` are non-terminal.
pub fn is_terminal(state: ConsentState) -> bool {
    matches!(
        state,
        ConsentState::Declined
            | ConsentState::Revoked
            | ConsentState::TimedOut
            | ConsentState::Expired,
    )
}

/// Is the direct transition `from → to` legal?
///
/// The legal-transition table from concept doc 06 §"Consent Lifecycle"
/// transcribed verbatim:
///
/// ```text
///   Requested    → { Acknowledged, Declined, TimedOut, Expired }
///   Acknowledged → { Revoked, Expired }
///   Declined     → {}
///   Revoked      → {}
///   TimedOut     → {}
///   Expired      → {}
/// ```
///
/// **Self-transitions** (`from == to`) return `false` — re-entering the
/// same state is not a meaningful operation for Consent (unlike
/// `AuthRequest` aggregation, where re-aggregation is idempotent).
/// Callers that want to no-op on a self-transition check `from == to`
/// before invoking.
///
/// **Forward-only revocation**: `Revoked → Acknowledged` (or any other
/// outbound transition from `Revoked`) returns `false`. The invariant
/// is enforced at the table level so every code path that consults
/// `legal_transition` inherits it.
pub fn legal_transition(from: ConsentState, to: ConsentState) -> bool {
    use ConsentState::*;
    matches!(
        (from, to),
        (Requested, Acknowledged)
            | (Requested, Declined)
            | (Requested, TimedOut)
            | (Requested, Expired)
            | (Acknowledged, Revoked)
            | (Acknowledged, Expired)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_set_matches_concept_doc() {
        // Per concept doc 06 §"Consent Lifecycle":
        // Terminal: Declined, Revoked, TimedOut, Expired.
        // Non-terminal: Requested, Acknowledged.
        assert!(!is_terminal(ConsentState::Requested));
        assert!(!is_terminal(ConsentState::Acknowledged));
        assert!(is_terminal(ConsentState::Declined));
        assert!(is_terminal(ConsentState::Revoked));
        assert!(is_terminal(ConsentState::TimedOut));
        assert!(is_terminal(ConsentState::Expired));
    }

    #[test]
    fn legal_transitions_match_concept_doc_verbatim() {
        use ConsentState::*;
        // Requested has 4 outbound arrows.
        assert!(legal_transition(Requested, Acknowledged));
        assert!(legal_transition(Requested, Declined));
        assert!(legal_transition(Requested, TimedOut));
        assert!(legal_transition(Requested, Expired));
        // Acknowledged has 2 outbound arrows.
        assert!(legal_transition(Acknowledged, Revoked));
        assert!(legal_transition(Acknowledged, Expired));
    }

    #[test]
    fn terminal_states_have_zero_outbound_arrows() {
        // Forward-only revocation + terminal stickiness combined.
        for from in [
            ConsentState::Declined,
            ConsentState::Revoked,
            ConsentState::TimedOut,
            ConsentState::Expired,
        ] {
            for to in ConsentState::ALL {
                assert!(
                    !legal_transition(from, to),
                    "{from:?} → {to:?} must be illegal (terminal stickiness / forward-only)"
                );
            }
        }
    }

    #[test]
    fn self_transitions_are_illegal() {
        // Per the doc-comment: `from == to` returns false.
        for s in ConsentState::ALL {
            assert!(
                !legal_transition(s, s),
                "{s:?} → {s:?} must be illegal (self-transition)"
            );
        }
    }

    #[test]
    fn requested_cannot_revoke_directly() {
        // Concept-doc: Revoked is reachable ONLY via Acknowledged.
        assert!(!legal_transition(
            ConsentState::Requested,
            ConsentState::Revoked
        ));
    }

    #[test]
    fn acknowledged_cannot_decline_or_time_out() {
        // Once Acknowledged, the only forward paths are Revoked / Expired.
        assert!(!legal_transition(
            ConsentState::Acknowledged,
            ConsentState::Declined
        ));
        assert!(!legal_transition(
            ConsentState::Acknowledged,
            ConsentState::TimedOut
        ));
        assert!(!legal_transition(
            ConsentState::Acknowledged,
            ConsentState::Requested
        ));
    }

    #[test]
    fn full_legal_table_has_exactly_six_arrows() {
        // 4 outbound from Requested + 2 outbound from Acknowledged = 6.
        // Asserted by enumerating the 36-cell matrix.
        let mut count = 0;
        for from in ConsentState::ALL {
            for to in ConsentState::ALL {
                if legal_transition(from, to) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 6, "legal transition count must be exactly 6");
    }
}
