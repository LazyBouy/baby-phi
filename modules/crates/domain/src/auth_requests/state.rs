//! Aggregation + terminal-state predicates + the legal-transition table.
//!
//! These are the **pure** building blocks the rest of the module composes:
//! every `transition_*` function in [`super::transitions`] ends by calling
//! [`aggregate_request_state`], and every guard consults
//! [`is_terminal`] / [`is_closed_terminal`] /
//! [`legal_request_transition`].

use crate::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, ResourceSlotState,
};

// ---------------------------------------------------------------------------
// Per-resource aggregation (slots → resource-level state)
// ---------------------------------------------------------------------------

/// Combine the approver-slot states of a single resource into the
/// resource-level state.
///
/// Table from `concepts/permissions/02` §Per-resource state derivation:
///
/// | Slot configuration                         | Resource state |
/// |--------------------------------------------|----------------|
/// | All slots `Approved`                       | `Approved`     |
/// | All slots `Denied`                         | `Denied`       |
/// | Any slot `Denied`, any slot `Approved`     | `Partial`      |
/// | Any slot `Unfilled`, no slots `Approved`   | `InProgress`   |
/// | Any slot `Unfilled`, ≥1 `Approved` slot    | `InProgress`   |
///
/// Empty slot list returns [`ResourceSlotState::InProgress`] — the
/// resource has no approvers yet (a degenerate but well-defined shape).
pub fn aggregate_resource_state(slots: &[ApproverSlot]) -> ResourceSlotState {
    if slots.is_empty() {
        return ResourceSlotState::InProgress;
    }
    let has_approved = slots.iter().any(|s| s.state == ApproverSlotState::Approved);
    let has_denied = slots.iter().any(|s| s.state == ApproverSlotState::Denied);
    let has_unfilled = slots.iter().any(|s| s.state == ApproverSlotState::Unfilled);

    if !has_unfilled && has_approved && !has_denied {
        return ResourceSlotState::Approved;
    }
    if !has_unfilled && has_denied && !has_approved {
        return ResourceSlotState::Denied;
    }
    if has_approved && has_denied {
        return ResourceSlotState::Partial;
    }
    ResourceSlotState::InProgress
}

// ---------------------------------------------------------------------------
// Request-level aggregation (resources → request-level state)
// ---------------------------------------------------------------------------

/// Derive the request-level state from the slots alone, respecting the
/// "sticky" terminal states set by explicit transitions.
///
/// Rules:
/// - If the current state is a **closed** terminal (Approved / Denied /
///   Expired / Revoked / Cancelled), or is `Draft`, aggregation is a
///   no-op — caller must invoke the explicit transition to move out.
/// - Otherwise (Pending / InProgress / Partial), recompute from slots:
///   - All resources `Approved` → `Approved`
///   - All resources `Denied` → `Denied`
///   - Any resource still `InProgress` AND no slots filled anywhere → `Pending`
///   - Any resource still `InProgress` AND ≥1 slot filled → `InProgress`
///   - Mixed `Approved`/`Denied` with no `InProgress` → `Partial`
///   - Any resource `Partial` (co-owner split) AND no `InProgress` →
///     `Partial` (the request carries the split until the owner overrides).
pub fn aggregate_request_state(req: &AuthRequest) -> AuthRequestState {
    // Sticky states — aggregation does not move out of these.
    match req.state {
        AuthRequestState::Draft
        | AuthRequestState::Approved
        | AuthRequestState::Denied
        | AuthRequestState::Expired
        | AuthRequestState::Revoked
        | AuthRequestState::Cancelled => return req.state,
        AuthRequestState::Pending | AuthRequestState::InProgress | AuthRequestState::Partial => {}
    }

    let resource_states: Vec<ResourceSlotState> = req
        .resource_slots
        .iter()
        .map(|rs| aggregate_resource_state(&rs.approvers))
        .collect();

    if resource_states.is_empty() {
        // Degenerate: no resource slots on the request. Leave state as-is.
        return req.state;
    }

    let all_approved = resource_states
        .iter()
        .all(|s| *s == ResourceSlotState::Approved);
    let all_denied = resource_states
        .iter()
        .all(|s| *s == ResourceSlotState::Denied);
    let any_in_progress = resource_states.contains(&ResourceSlotState::InProgress);
    let any_approved = resource_states.contains(&ResourceSlotState::Approved);
    let any_denied = resource_states.contains(&ResourceSlotState::Denied);
    let any_partial = resource_states.contains(&ResourceSlotState::Partial);
    let any_slot_filled = req.resource_slots.iter().any(|rs| {
        rs.approvers
            .iter()
            .any(|a| a.state != ApproverSlotState::Unfilled)
    });

    if all_approved {
        return AuthRequestState::Approved;
    }
    if all_denied {
        return AuthRequestState::Denied;
    }
    if any_in_progress {
        if any_slot_filled {
            return AuthRequestState::InProgress;
        }
        return AuthRequestState::Pending;
    }
    if any_partial || (any_approved && any_denied) {
        return AuthRequestState::Partial;
    }
    // Fallback: shouldn't be reachable given the shape of the 5 resource
    // states, but stay state-as-is rather than panicking.
    req.state
}

// ---------------------------------------------------------------------------
// Terminal-state predicates
// ---------------------------------------------------------------------------

/// True for every terminal state — closed (Approved/Denied/Expired/Revoked/
/// Cancelled) or semi-open (Partial).
pub fn is_terminal(state: AuthRequestState) -> bool {
    matches!(
        state,
        AuthRequestState::Approved
            | AuthRequestState::Denied
            | AuthRequestState::Partial
            | AuthRequestState::Expired
            | AuthRequestState::Revoked
            | AuthRequestState::Cancelled
    )
}

/// Closed terminal states reject every transition except revoke / expire
/// from Approved. Partial is **not** closed — the owner may still override.
pub fn is_closed_terminal(state: AuthRequestState) -> bool {
    matches!(
        state,
        AuthRequestState::Denied
            | AuthRequestState::Expired
            | AuthRequestState::Revoked
            | AuthRequestState::Cancelled
    )
}

/// True iff `state` is an active (pre-terminal) state where slots may
/// still change naturally.
pub fn is_active(state: AuthRequestState) -> bool {
    matches!(
        state,
        AuthRequestState::Draft | AuthRequestState::Pending | AuthRequestState::InProgress
    )
}

// ---------------------------------------------------------------------------
// Legal transition table
// ---------------------------------------------------------------------------

/// Is the direct transition `from → to` legal?
///
/// Used as a guard by [`super::transitions`] and by the `illegal_transition_never_succeeds`
/// proptest. The table is the concept doc's §State Machine diagram
/// transcribed into Rust:
///
/// ```text
///   Draft        → { Pending, Cancelled }
///   Pending      → { InProgress, Expired, Cancelled }
///   InProgress   → { Approved, Denied, Partial, Expired, Cancelled }
///   Approved     → { Revoked, Expired }
///   Partial      → { Approved, Denied, Revoked, Expired }
///   Denied       → {}
///   Expired      → {}
///   Revoked      → {}
///   Cancelled    → {}
/// ```
///
/// **Self-transitions** (`from == to`) return `true` — re-aggregation is
/// an idempotent op. Callers that want to reject self-transitions check
/// `from != to` first.
pub fn legal_request_transition(from: AuthRequestState, to: AuthRequestState) -> bool {
    if from == to {
        return true;
    }
    use AuthRequestState::*;
    match (from, to) {
        (Draft, Pending) | (Draft, Cancelled) => true,
        // Pending → InProgress / Expired / Cancelled are the nominal
        // arrows. The direct `Pending → Approved` and `Pending → Denied`
        // transitions cover the degenerate case where a request has
        // only single-slot single-resource configurations and a single
        // `transition_slot` call drives aggregation straight to a
        // terminal outcome. The concept doc's diagram is a state *tour*,
        // not exhaustive of one-step reachability.
        (Pending, InProgress)
        | (Pending, Approved)
        | (Pending, Denied)
        | (Pending, Expired)
        | (Pending, Cancelled) => true,
        (InProgress, Approved)
        | (InProgress, Denied)
        | (InProgress, Partial)
        | (InProgress, Expired)
        | (InProgress, Cancelled)
        // Reconsideration backtrack: when the last filled slot of an
        // InProgress request reconsiders back to Unfilled, the request
        // has effectively no slots filled — the natural state is Pending.
        | (InProgress, Pending) => true,
        (Approved, Revoked) | (Approved, Expired) => true,
        (Partial, Approved)
        | (Partial, Denied)
        | (Partial, Revoked)
        | (Partial, Expired) => true,
        // Additional "re-aggregation" transitions where the request is
        // already in the Partial bucket and further slot changes move it
        // to a closed outcome naturally (e.g. the last denying approver
        // reconsiders, bringing the request back to Approved via the slot
        // path). Not listed above because those are handled via the
        // aggregation fn.
        (Partial, InProgress) => true,
        _ => false,
    }
}

/// Legal approver-slot transitions. An `Unfilled` slot can move to
/// `Approved` or `Denied` (fill); a filled slot can move back to
/// `Unfilled` (reconsider). Direct `Approved ↔ Denied` flips are illegal —
/// callers must reconsider first.
pub fn legal_slot_transition(from: ApproverSlotState, to: ApproverSlotState) -> bool {
    use ApproverSlotState::*;
    match (from, to) {
        (Unfilled, Approved) | (Unfilled, Denied) => true,
        (Approved, Unfilled) | (Denied, Unfilled) => true,
        (same1, same2) if same1 == same2 => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::AuthRequestId;
    use crate::model::nodes::{
        ApproverSlot, AuthRequest, PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState,
    };
    use chrono::Utc;

    fn slot(state: ApproverSlotState) -> ApproverSlot {
        ApproverSlot {
            approver: PrincipalRef::System("system:test".into()),
            state,
            responded_at: None,
            reconsidered_at: None,
        }
    }

    fn req_with_slots(
        state: AuthRequestState,
        per_resource: Vec<Vec<ApproverSlotState>>,
    ) -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor: PrincipalRef::System("system:test".into()),
            kinds: vec![],
            scope: vec!["read".into()],
            state,
            valid_until: None,
            submitted_at: Utc::now(),
            resource_slots: per_resource
                .into_iter()
                .map(|slots| ResourceSlot {
                    resource: ResourceRef {
                        uri: "test:r".into(),
                    },
                    approvers: slots.into_iter().map(slot).collect(),
                    state: ResourceSlotState::InProgress,
                })
                .collect(),
            justification: None,
            audit_class: AuditClass::Logged,
            terminal_state_entered_at: None,
            archived: false,
            active_window_days: 90,
            provenance_template: None,
        }
    }

    // ---- aggregate_resource_state ----------------------------------------

    #[test]
    fn resource_empty_slots_is_in_progress() {
        assert_eq!(aggregate_resource_state(&[]), ResourceSlotState::InProgress);
    }

    #[test]
    fn resource_all_approved_is_approved() {
        let slots = [
            slot(ApproverSlotState::Approved),
            slot(ApproverSlotState::Approved),
        ];
        assert_eq!(
            aggregate_resource_state(&slots),
            ResourceSlotState::Approved
        );
    }

    #[test]
    fn resource_all_denied_is_denied() {
        let slots = [slot(ApproverSlotState::Denied)];
        assert_eq!(aggregate_resource_state(&slots), ResourceSlotState::Denied);
    }

    #[test]
    fn resource_mix_of_approved_and_denied_is_partial() {
        let slots = [
            slot(ApproverSlotState::Approved),
            slot(ApproverSlotState::Denied),
        ];
        assert_eq!(aggregate_resource_state(&slots), ResourceSlotState::Partial);
    }

    #[test]
    fn resource_any_unfilled_with_no_denied_is_in_progress() {
        let slots = [
            slot(ApproverSlotState::Unfilled),
            slot(ApproverSlotState::Approved),
        ];
        assert_eq!(
            aggregate_resource_state(&slots),
            ResourceSlotState::InProgress
        );
    }

    #[test]
    fn resource_any_unfilled_with_denied_is_in_progress() {
        // Per concept doc: "Any slot Denied, rest Unfilled → In Progress
        // heading toward Denied".
        let slots = [
            slot(ApproverSlotState::Unfilled),
            slot(ApproverSlotState::Denied),
        ];
        assert_eq!(
            aggregate_resource_state(&slots),
            ResourceSlotState::InProgress
        );
    }

    // ---- aggregate_request_state -----------------------------------------

    #[test]
    fn request_all_unfilled_stays_pending() {
        let req = req_with_slots(
            AuthRequestState::Pending,
            vec![vec![
                ApproverSlotState::Unfilled,
                ApproverSlotState::Unfilled,
            ]],
        );
        assert_eq!(aggregate_request_state(&req), AuthRequestState::Pending);
    }

    #[test]
    fn request_any_filled_becomes_in_progress() {
        let req = req_with_slots(
            AuthRequestState::Pending,
            vec![vec![
                ApproverSlotState::Approved,
                ApproverSlotState::Unfilled,
            ]],
        );
        assert_eq!(aggregate_request_state(&req), AuthRequestState::InProgress);
    }

    #[test]
    fn request_all_resources_approved_becomes_approved() {
        let req = req_with_slots(
            AuthRequestState::InProgress,
            vec![
                vec![ApproverSlotState::Approved],
                vec![ApproverSlotState::Approved],
            ],
        );
        assert_eq!(aggregate_request_state(&req), AuthRequestState::Approved);
    }

    #[test]
    fn request_all_resources_denied_becomes_denied() {
        let req = req_with_slots(
            AuthRequestState::InProgress,
            vec![vec![ApproverSlotState::Denied]],
        );
        assert_eq!(aggregate_request_state(&req), AuthRequestState::Denied);
    }

    #[test]
    fn request_mixed_approved_denied_no_in_progress_becomes_partial() {
        let req = req_with_slots(
            AuthRequestState::InProgress,
            vec![
                vec![ApproverSlotState::Approved],
                vec![ApproverSlotState::Denied],
            ],
        );
        assert_eq!(aggregate_request_state(&req), AuthRequestState::Partial);
    }

    #[test]
    fn request_sticky_terminals_are_unchanged_by_aggregation() {
        for state in [
            AuthRequestState::Approved,
            AuthRequestState::Denied,
            AuthRequestState::Revoked,
            AuthRequestState::Expired,
            AuthRequestState::Cancelled,
        ] {
            let req = req_with_slots(state, vec![vec![ApproverSlotState::Unfilled]]);
            assert_eq!(
                aggregate_request_state(&req),
                state,
                "{state:?} must be sticky under aggregation"
            );
        }
    }

    // ---- predicates + legal transitions ---------------------------------

    #[test]
    fn terminal_predicates_match_concept_doc() {
        for s in AuthRequestState::ALL {
            let active = is_active(s);
            let terminal = is_terminal(s);
            let closed = is_closed_terminal(s);
            assert!(
                active != terminal,
                "{s:?} cannot be both active and terminal"
            );
            if closed {
                assert!(terminal, "closed implies terminal");
            }
        }
        // Specific calls.
        assert!(is_active(AuthRequestState::Draft));
        assert!(is_active(AuthRequestState::Pending));
        assert!(is_active(AuthRequestState::InProgress));
        assert!(is_terminal(AuthRequestState::Partial));
        assert!(!is_closed_terminal(AuthRequestState::Partial));
        assert!(!is_closed_terminal(AuthRequestState::Approved)); // Approved can revoke/expire
        assert!(is_closed_terminal(AuthRequestState::Revoked));
    }

    #[test]
    fn legal_request_transitions_cover_every_concept_doc_arrow() {
        use AuthRequestState::*;
        assert!(legal_request_transition(Draft, Pending));
        assert!(legal_request_transition(Draft, Cancelled));
        assert!(legal_request_transition(Pending, InProgress));
        assert!(legal_request_transition(Pending, Cancelled));
        assert!(legal_request_transition(Pending, Expired));
        assert!(legal_request_transition(InProgress, Approved));
        assert!(legal_request_transition(InProgress, Denied));
        assert!(legal_request_transition(InProgress, Partial));
        assert!(legal_request_transition(InProgress, Expired));
        assert!(legal_request_transition(InProgress, Cancelled));
        assert!(legal_request_transition(Approved, Revoked));
        assert!(legal_request_transition(Approved, Expired));
        assert!(legal_request_transition(Partial, Approved));
        assert!(legal_request_transition(Partial, Denied));
        assert!(legal_request_transition(Partial, Revoked));
        assert!(legal_request_transition(Partial, Expired));
    }

    #[test]
    fn legal_request_transitions_reject_illegal_arrows() {
        use AuthRequestState::*;
        // Terminal closed states have no outbound arrows.
        for from in [Denied, Expired, Revoked, Cancelled] {
            for to in AuthRequestState::ALL {
                if from == to {
                    continue;
                }
                assert!(
                    !legal_request_transition(from, to),
                    "{from:?}→{to:?} must be illegal"
                );
            }
        }
        // Draft cannot jump straight to Approved / Denied / Partial.
        assert!(!legal_request_transition(Draft, Approved));
        assert!(!legal_request_transition(Draft, Denied));
        assert!(!legal_request_transition(Draft, Partial));
        // Pending cannot jump straight to Revoked / Partial (no grant yet,
        // need multi-slot fills to partial-out).
        assert!(!legal_request_transition(Pending, Revoked));
        assert!(!legal_request_transition(Pending, Partial));
    }

    #[test]
    fn legal_slot_transitions_forbid_direct_approved_denied_flip() {
        use ApproverSlotState::*;
        assert!(legal_slot_transition(Unfilled, Approved));
        assert!(legal_slot_transition(Unfilled, Denied));
        assert!(legal_slot_transition(Approved, Unfilled));
        assert!(legal_slot_transition(Denied, Unfilled));
        // Direct flip illegal.
        assert!(!legal_slot_transition(Approved, Denied));
        assert!(!legal_slot_transition(Denied, Approved));
    }
}
