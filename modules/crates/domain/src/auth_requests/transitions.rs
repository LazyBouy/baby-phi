//! Transition operations on an [`AuthRequest`].
//!
//! Every public function here takes an immutable `&AuthRequest` and
//! returns `Result<AuthRequest, TransitionError>` — state never mutates
//! in place. That keeps the API pure, proptest-friendly, and trivially
//! rollbackable when the caller hits a constraint violation downstream.
//!
//! All transitions respect the legal-transition table in
//! [`super::state::legal_request_transition`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::nodes::{ApproverSlotState, AuthRequest, AuthRequestState, ResourceSlotState};

use super::state::{
    aggregate_request_state, aggregate_resource_state, is_closed_terminal,
    legal_request_transition, legal_slot_transition,
};

/// Structured error for rejected transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransitionError {
    /// The request is in a terminal state that forbids this operation.
    #[error("request in closed terminal state {state:?} rejects {attempted}")]
    ClosedTerminal {
        state: AuthRequestState,
        attempted: String,
    },
    /// The requested request-level transition is not in the legal table.
    #[error("illegal request transition {from:?} → {to:?}")]
    IllegalRequestTransition {
        from: AuthRequestState,
        to: AuthRequestState,
    },
    /// The requested approver-slot transition is not in the legal table.
    #[error("illegal slot transition {from:?} → {to:?}")]
    IllegalSlotTransition {
        from: ApproverSlotState,
        to: ApproverSlotState,
    },
    /// The slot indices do not point into the request.
    #[error("slot index out of bounds: resource {resource_idx}, slot {slot_idx}")]
    SlotOutOfBounds {
        resource_idx: usize,
        slot_idx: usize,
    },
    /// An action (`override_approve`, `close_as_denied`) requires the
    /// request to currently be in `Partial`.
    #[error("operation {attempted} only valid from Partial (got {state:?})")]
    NotInPartial {
        state: AuthRequestState,
        attempted: String,
    },
    /// `submit` only applies to `Draft`.
    #[error("submit only valid from Draft (got {state:?})")]
    NotInDraft { state: AuthRequestState },
    /// `cancel` only applies to active (non-terminal) states.
    #[error("cancel only valid from active states (got {state:?})")]
    NotCancellable { state: AuthRequestState },
    /// `expire` requires `valid_until` to be present and `<= now`.
    #[error("expire requires valid_until ≤ now (valid_until = {valid_until:?}, now = {now})")]
    ExpiryNotDue {
        valid_until: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn enter_state(
    req: &AuthRequest,
    to: AuthRequestState,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    if !legal_request_transition(req.state, to) {
        return Err(TransitionError::IllegalRequestTransition {
            from: req.state,
            to,
        });
    }
    let mut next = req.clone();
    let now_terminal = super::state::is_terminal(to);
    next.state = to;
    // Stamp the terminal-entry time on every transition INTO a terminal
    // state. On re-entry (Partial→Approved→Revoked chains) we update to
    // the most recent terminal transition so retention math tracks the
    // latest closed timestamp.
    if now_terminal {
        next.terminal_state_entered_at = Some(at);
    }
    Ok(next)
}

// ---------------------------------------------------------------------------
// submit — Draft → Pending
// ---------------------------------------------------------------------------

/// Move a `Draft` request to `Pending`. Also stamps `submitted_at`.
pub fn submit(
    req: &AuthRequest,
    submitted_at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    if req.state != AuthRequestState::Draft {
        return Err(TransitionError::NotInDraft { state: req.state });
    }
    let mut next = enter_state(req, AuthRequestState::Pending, submitted_at)?;
    next.submitted_at = submitted_at;
    Ok(next)
}

// ---------------------------------------------------------------------------
// transition_slot — fill / change / reconsider one approver slot
// ---------------------------------------------------------------------------

/// Change approver slot `(resource_idx, slot_idx)` to `new_state`.
///
/// Filling an unfilled slot stamps `responded_at`; reconsidering
/// (filled → `Unfilled`) stamps `reconsidered_at` and clears
/// `responded_at`. Re-aggregates the request state at the end.
///
/// Refuses to modify slots when the request is in a closed terminal
/// state (Denied / Expired / Revoked / Cancelled). Partial is permitted
/// per the concept doc's "semi-open" semantics.
pub fn transition_slot(
    req: &AuthRequest,
    resource_idx: usize,
    slot_idx: usize,
    new_state: ApproverSlotState,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    if is_closed_terminal(req.state) {
        return Err(TransitionError::ClosedTerminal {
            state: req.state,
            attempted: "transition_slot".into(),
        });
    }
    let resource = req.resource_slots.get(resource_idx).ok_or({
        TransitionError::SlotOutOfBounds {
            resource_idx,
            slot_idx,
        }
    })?;
    let slot = resource.approvers.get(slot_idx).ok_or({
        TransitionError::SlotOutOfBounds {
            resource_idx,
            slot_idx,
        }
    })?;
    if !legal_slot_transition(slot.state, new_state) {
        return Err(TransitionError::IllegalSlotTransition {
            from: slot.state,
            to: new_state,
        });
    }

    let mut next = req.clone();
    {
        let slot = &mut next.resource_slots[resource_idx].approvers[slot_idx];
        let was_filled =
            slot.state == ApproverSlotState::Approved || slot.state == ApproverSlotState::Denied;
        let now_filled =
            new_state == ApproverSlotState::Approved || new_state == ApproverSlotState::Denied;
        slot.state = new_state;
        if now_filled {
            slot.responded_at = Some(at);
        }
        if was_filled && !now_filled {
            // Reconsideration: stamp reconsidered_at, null the response.
            slot.reconsidered_at = Some(at);
            slot.responded_at = None;
        }
    }
    // Recompute the resource state.
    next.resource_slots[resource_idx].state =
        aggregate_resource_state(&next.resource_slots[resource_idx].approvers);

    // Re-derive the request state.
    let aggregated = aggregate_request_state(&next);
    if aggregated != next.state {
        // Use enter_state so we stamp terminal entry and validate the transition.
        next = enter_state(&next, aggregated, at)?;
    }
    Ok(next)
}

/// Convenience — reconsidering unconditionally moves a slot to `Unfilled`.
pub fn reconsider_slot(
    req: &AuthRequest,
    resource_idx: usize,
    slot_idx: usize,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    transition_slot(req, resource_idx, slot_idx, ApproverSlotState::Unfilled, at)
}

// ---------------------------------------------------------------------------
// cancel — Draft / Pending / InProgress → Cancelled
// ---------------------------------------------------------------------------

/// Requestor-initiated cancellation. Only valid from the three active
/// states; everything else returns [`TransitionError::NotCancellable`].
pub fn cancel(req: &AuthRequest, at: DateTime<Utc>) -> Result<AuthRequest, TransitionError> {
    if !super::state::is_active(req.state) {
        return Err(TransitionError::NotCancellable { state: req.state });
    }
    enter_state(req, AuthRequestState::Cancelled, at)
}

// ---------------------------------------------------------------------------
// override_approve / close_as_denied — owner actions from Partial
// ---------------------------------------------------------------------------

/// Owner override: Partial → Approved. Every resource slot that was
/// `Denied` is re-stamped `Approved`; the owner's decision propagates
/// through to the derived resource + request states.
pub fn override_approve(
    req: &AuthRequest,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    if req.state != AuthRequestState::Partial {
        return Err(TransitionError::NotInPartial {
            state: req.state,
            attempted: "override_approve".into(),
        });
    }
    let mut next = req.clone();
    // Force every approver slot that was blocking to Approved. The
    // override is an owner-authority action, so non-Approved slots
    // become Approved.
    for rs in &mut next.resource_slots {
        for ap in &mut rs.approvers {
            if ap.state != ApproverSlotState::Approved {
                ap.state = ApproverSlotState::Approved;
                ap.responded_at = Some(at);
            }
        }
        rs.state = ResourceSlotState::Approved;
    }
    enter_state(&next, AuthRequestState::Approved, at)
}

/// Owner override: Partial → Denied. Symmetric to [`override_approve`].
pub fn close_as_denied(
    req: &AuthRequest,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    if req.state != AuthRequestState::Partial {
        return Err(TransitionError::NotInPartial {
            state: req.state,
            attempted: "close_as_denied".into(),
        });
    }
    let mut next = req.clone();
    for rs in &mut next.resource_slots {
        for ap in &mut rs.approvers {
            if ap.state != ApproverSlotState::Denied {
                ap.state = ApproverSlotState::Denied;
                ap.responded_at = Some(at);
            }
        }
        rs.state = ResourceSlotState::Denied;
    }
    enter_state(&next, AuthRequestState::Denied, at)
}

// ---------------------------------------------------------------------------
// Internal helper consumed by the revocation module (keeps the
// legal-transition table in one place).
// ---------------------------------------------------------------------------

pub(super) fn transition_to_revoked(
    req: &AuthRequest,
    at: DateTime<Utc>,
) -> Result<AuthRequest, TransitionError> {
    enter_state(req, AuthRequestState::Revoked, at)
}

// ---------------------------------------------------------------------------
// expire — valid_until elapsed
// ---------------------------------------------------------------------------

/// Mark the request `Expired` when `valid_until` has elapsed. Works from
/// Pending / InProgress / Approved / Partial. A request with no
/// `valid_until` set is never expireable.
pub fn expire(req: &AuthRequest, now: DateTime<Utc>) -> Result<AuthRequest, TransitionError> {
    let Some(valid_until) = req.valid_until else {
        return Err(TransitionError::ExpiryNotDue {
            valid_until: None,
            now,
        });
    };
    if valid_until > now {
        return Err(TransitionError::ExpiryNotDue {
            valid_until: Some(valid_until),
            now,
        });
    }
    enter_state(req, AuthRequestState::Expired, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::AuthRequestId;
    use crate::model::nodes::{
        ApproverSlot, AuthRequest, PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState,
    };
    use chrono::Duration;

    fn slot(state: ApproverSlotState) -> ApproverSlot {
        ApproverSlot {
            approver: PrincipalRef::System("system:test".into()),
            state,
            responded_at: None,
            reconsidered_at: None,
        }
    }

    fn draft_req(slot_states_per_resource: Vec<Vec<ApproverSlotState>>) -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor: PrincipalRef::System("system:test".into()),
            kinds: vec![],
            scope: vec!["read".into()],
            state: AuthRequestState::Draft,
            valid_until: Some(Utc::now() + Duration::days(7)),
            submitted_at: Utc::now(),
            resource_slots: slot_states_per_resource
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

    // ---- submit ----------------------------------------------------------

    #[test]
    fn submit_draft_goes_to_pending() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let out = submit(&req, Utc::now()).expect("submit");
        assert_eq!(out.state, AuthRequestState::Pending);
    }

    #[test]
    fn submit_from_non_draft_fails() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let err = submit(&req, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::NotInDraft { .. }));
    }

    // ---- transition_slot ------------------------------------------------

    #[test]
    fn transition_slot_stamps_responded_at() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let at = Utc::now();
        let out = transition_slot(&req, 0, 0, ApproverSlotState::Approved, at).unwrap();
        assert_eq!(
            out.resource_slots[0].approvers[0].state,
            ApproverSlotState::Approved
        );
        assert_eq!(out.resource_slots[0].approvers[0].responded_at, Some(at));
    }

    #[test]
    fn transition_slot_reaggregates_to_approved_when_all_slots_filled() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let out = transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap();
        assert_eq!(out.state, AuthRequestState::Approved);
        assert!(out.terminal_state_entered_at.is_some());
    }

    #[test]
    fn transition_slot_reconsider_clears_response_and_marks_reconsidered() {
        let req = draft_req(vec![vec![
            ApproverSlotState::Unfilled,
            ApproverSlotState::Unfilled,
        ]]);
        let req = submit(&req, Utc::now()).unwrap();
        let approved =
            transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap();
        // Now reconsider.
        let reconsidered =
            reconsider_slot(&approved, 0, 0, Utc::now() + Duration::minutes(5)).unwrap();
        assert_eq!(
            reconsidered.resource_slots[0].approvers[0].state,
            ApproverSlotState::Unfilled
        );
        assert!(reconsidered.resource_slots[0].approvers[0]
            .responded_at
            .is_none());
        assert!(reconsidered.resource_slots[0].approvers[0]
            .reconsidered_at
            .is_some());
    }

    #[test]
    fn transition_slot_reject_direct_approved_denied_flip() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let approved =
            transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap();
        // Direct flip back to Denied is illegal (must reconsider first).
        // But this test triggers: terminal-state-guard rejects it first
        // because `approved` request is now in Approved (closed-ish).
        let err = transition_slot(&approved, 0, 0, ApproverSlotState::Denied, Utc::now());
        assert!(err.is_err());
    }

    #[test]
    fn transition_slot_rejects_closed_terminal() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let mut req = submit(&req, Utc::now()).unwrap();
        req.state = AuthRequestState::Revoked;
        let err = transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::ClosedTerminal { .. }));
    }

    #[test]
    fn transition_slot_out_of_bounds_errors() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let err =
            transition_slot(&req, 42, 0, ApproverSlotState::Approved, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::SlotOutOfBounds { .. }));
    }

    // ---- cancel ---------------------------------------------------------

    #[test]
    fn cancel_from_pending_succeeds() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let out = cancel(&req, Utc::now()).unwrap();
        assert_eq!(out.state, AuthRequestState::Cancelled);
        assert!(out.terminal_state_entered_at.is_some());
    }

    #[test]
    fn cancel_from_approved_fails() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let approved =
            transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap();
        let err = cancel(&approved, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::NotCancellable { .. }));
    }

    // ---- override_approve / close_as_denied ---------------------------

    fn partial_req() -> AuthRequest {
        // Two resources, one Approved, one Denied → request is Partial.
        let req = draft_req(vec![
            vec![ApproverSlotState::Unfilled],
            vec![ApproverSlotState::Unfilled],
        ]);
        let req = submit(&req, Utc::now()).unwrap();
        let req = transition_slot(&req, 0, 0, ApproverSlotState::Approved, Utc::now()).unwrap();
        let req = transition_slot(&req, 1, 0, ApproverSlotState::Denied, Utc::now()).unwrap();
        assert_eq!(req.state, AuthRequestState::Partial);
        req
    }

    #[test]
    fn override_approve_from_partial_forces_approved() {
        let req = partial_req();
        let out = override_approve(&req, Utc::now()).unwrap();
        assert_eq!(out.state, AuthRequestState::Approved);
        for rs in &out.resource_slots {
            assert_eq!(rs.state, ResourceSlotState::Approved);
            for ap in &rs.approvers {
                assert_eq!(ap.state, ApproverSlotState::Approved);
            }
        }
    }

    #[test]
    fn close_as_denied_from_partial_forces_denied() {
        let req = partial_req();
        let out = close_as_denied(&req, Utc::now()).unwrap();
        assert_eq!(out.state, AuthRequestState::Denied);
        for rs in &out.resource_slots {
            assert_eq!(rs.state, ResourceSlotState::Denied);
        }
    }

    #[test]
    fn override_approve_not_in_partial_errors() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let err = override_approve(&req, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::NotInPartial { .. }));
    }

    // ---- expire ---------------------------------------------------------

    #[test]
    fn expire_with_past_valid_until_succeeds() {
        let mut req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        req.valid_until = Some(Utc::now() - Duration::days(1));
        let req = submit(&req, Utc::now()).unwrap();
        let out = expire(&req, Utc::now()).unwrap();
        assert_eq!(out.state, AuthRequestState::Expired);
    }

    #[test]
    fn expire_with_future_valid_until_rejects() {
        let req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        let req = submit(&req, Utc::now()).unwrap();
        let err = expire(&req, Utc::now()).unwrap_err();
        assert!(matches!(err, TransitionError::ExpiryNotDue { .. }));
    }

    #[test]
    fn expire_with_no_valid_until_rejects() {
        let mut req = draft_req(vec![vec![ApproverSlotState::Unfilled]]);
        req.valid_until = None;
        let req = submit(&req, Utc::now()).unwrap();
        let err = expire(&req, Utc::now()).unwrap_err();
        assert!(matches!(
            err,
            TransitionError::ExpiryNotDue {
                valid_until: None,
                ..
            }
        ));
    }
}
