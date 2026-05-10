//! Per-state Access Control for AuthRequest reads, writes, and lifecycle ops.
//!
//! Source of truth:
//! [`docs/specs/v0/concepts/permissions/02-auth-request.md`](../../../../../docs/specs/v0/concepts/permissions/02-auth-request.md)
//! §"Per-State Access Matrix" lines 130–144 — the full table of (state ×
//! principal-class × allowed ops) for the 9 AuthRequest states.
//!
//! ## Why this module exists
//!
//! Before CH-18 the Repository layer accepted reads + writes against an
//! [`AuthRequest`](crate::model::nodes::AuthRequest) without consulting the
//! per-state matrix from concept doc 02. CH-18 closes drift `D-new-12` by
//! capturing the matrix as a typed pure-function predicate
//! [`check_auth_request_access`] that every user-facing handler invokes
//! upstream of the Repository call.
//!
//! ## Decision references
//!
//! Implementation gated by [`ADR-0056`](../../../../../docs/specs/v0/implementation/m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md):
//!
//! - §D56.1 — pure-function shape (no `&self`, no Repository, no async).
//! - §D56.2 — [`IntendedOp`] enum captures the 11 matrix-column operations
//!   (Read, Modify, Submit, Cancel, Approve, Deny, Reconsider, Revoke,
//!   OverrideApprove, CloseAsDenied, Expire). NOT a member of
//!   `Action::CANONICAL` — closed 34-verb action vocabulary preserved.
//! - §D56.3 — `AuthRequestAccessError` typed-error enum (5 variants); per
//!   F2.A user-locked at gate-1, mirrors the `FrozenTagViolation` shape from
//!   CH-12 ADR-0049 §D49.4.
//! - §D56.5 — wiring depth: F3.B locked at gate-1 (DIVERGENT from planner's
//!   F3.A recommendation) — full Repository + dashboard + resolver +
//!   bootstrap wiring, 17 production callsites consult this predicate.
//! - §D56.7 — Repository trait stays principal-blind; docstring contract
//!   documents the future-callsite invocation requirement.
//! - §D56.8 — 7 kernel-internal callsites are documented as fast-path skips
//!   (event-bus listeners, AR resolvers, cascade-revoke loops, bootstrap
//!   AR creation, `find_adoption_ar` helper).
//! - §D56.9 — sub-fork resolutions under F3.B (role.b / repo-shape.b /
//!   create-side.a / list-filter.a).

use crate::auth_requests::state::is_closed_terminal;
use crate::model::nodes::{ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef};
use crate::permissions::axioms::is_bootstrap_ar;

// ---------------------------------------------------------------------------
// IntendedOp — the 11 matrix-column operations (per ADR-0056 §D56.2)
// ---------------------------------------------------------------------------

/// Closed set of AR access-matrix-column operations.
///
/// Concept doc 02 lines 130–144 names 11 distinct ops across the
/// (state × principal-class × allowed-ops) matrix. This enum captures
/// them as a closed set scoped to AR access semantics.
///
/// **Closed-set invariant.** `IntendedOp` is INTENTIONALLY NOT a member of
/// [`crate::permissions::Action::CANONICAL`]. The two closed sets serve
/// different concerns:
/// - `Action::CANONICAL` is the manifest-resolution action vocabulary
///   (Permission Check engine input — 34 verbs).
/// - `IntendedOp` is the AR per-state matrix-column dimension (this
///   predicate's input — 11 ops).
///
/// Some `IntendedOp` variants (`Submit`, `Cancel`, `OverrideApprove`,
/// `CloseAsDenied`, `Expire`) have no clean `Action` mapping; conflating
/// the two would force a 1:1 column / verb correspondence that doesn't
/// exist. See ADR-0056 §D56.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntendedOp {
    /// Read the AR record itself (e.g., dashboard list, slot-fill read).
    Read,
    /// Mutate the AR's content (Draft-only edits before submit).
    Modify,
    /// Transition the AR from Draft → Pending (or InProgress).
    Submit,
    /// Transition the AR to Cancelled (requestor-only on Draft / Pending /
    /// InProgress).
    Cancel,
    /// Approve a slot the principal owns.
    Approve,
    /// Deny a slot the principal owns.
    Deny,
    /// Reconsider (re-edit) a slot the principal owns and previously filled.
    Reconsider,
    /// Revoke an Approved AR (resource-owner authority; descoped to
    /// `ar.requestor == principal` adoption-AR self-revoke at v2 per
    /// `D-CH18-FOLLOWUP-01`).
    Revoke,
    /// Resource-owner override-approve from `Partial`.
    OverrideApprove,
    /// Resource-owner close-as-denied from `Partial`.
    CloseAsDenied,
    /// Lifecycle expiry transition from Pending / InProgress to Expired.
    Expire,
}

// ---------------------------------------------------------------------------
// AuthRequestAccessError — typed deny classes (per ADR-0056 §D56.3)
// ---------------------------------------------------------------------------

/// Typed-error enum for matrix-cell DENY classes.
///
/// Mirrors [`crate::permissions::FrozenTagViolation`] (CH-12 ADR-0049
/// §D49.4) structurally — typed enum at the predicate's home module;
/// callers `match`-arm the variants for handler-side error mapping. Each
/// variant carries enough data to (a) format a useful 4xx response body
/// and (b) populate the `auth_request.access_denied` audit event per
/// §D56.6.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthRequestAccessError {
    /// The principal is not authorised to read the AR at this state.
    /// Carries the AR's state and the principal's wire-form classification
    /// (`"agent"`, `"user"`, `"organization"`, `"project"`, `"system"`)
    /// for audit-event composition.
    #[error("principal kind {principal_kind:?} is not authorised to read auth request in state {state:?}")]
    NotAuthorisedForRead {
        state: AuthRequestState,
        principal_kind: String,
    },

    /// The principal is not authorised to modify the AR at this state.
    /// Carries the AR's state, the principal's wire-form classification,
    /// and the attempted op for audit-event composition.
    #[error("principal kind {principal_kind:?} is not authorised to perform {intended_op:?} on auth request in state {state:?}")]
    NotAuthorisedForModify {
        state: AuthRequestState,
        principal_kind: String,
        intended_op: IntendedOp,
    },

    /// The intended op is forbidden in the current state regardless of
    /// principal — e.g., `Submit` on an Approved AR, `Modify` on a
    /// closed-terminal AR. Distinguished from [`Self::NotAuthorisedForModify`]
    /// because the cell is structurally empty in the matrix, not principal-gated.
    #[error("operation {intended_op:?} is forbidden on auth request in state {state:?}")]
    OperationForbiddenInState {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },

    /// The intended op requires `principal == ar.requestor` (e.g., `Submit`,
    /// `Cancel`, `Modify` on Draft).
    #[error("operation {intended_op:?} on auth request in state {state:?} is permitted only to the requestor")]
    RequestorOnlyOperation {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },

    /// The intended op requires the principal to own an *unfilled* approver
    /// slot — `Approve` / `Deny` on Pending / InProgress. A principal who
    /// owns a slot they have already filled hits this error rather than
    /// the catch-all `NotAuthorisedForModify`.
    #[error("operation {intended_op:?} on auth request in state {state:?} requires the principal to own an unfilled approver slot")]
    UnfilledApproverSlotOnly {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },
}

// ---------------------------------------------------------------------------
// PrincipalClass — internal classification (per ADR-0056 §D56.5)
// ---------------------------------------------------------------------------

/// Internal classification of a principal relative to an AR.
///
/// At v2 the classifier recognises four classes:
/// - [`Self::Requestor`] — `principal == ar.requestor` via `==`.
/// - [`Self::SlotApprover`] — the principal owns at least one approver slot
///   on the AR. Carries the slot's filled/unfilled state for the
///   `Approve` / `Deny` / `Reconsider` discrimination.
/// - [`Self::Bootstrap`] — `is_bootstrap_ar(ar) == true` AND principal is
///   the `system:genesis` axiom — system-internal AR fast-path. Only
///   permits Read at v2 per the matrix's "Observer reads at every state"
///   row; mutation paths on the bootstrap AR go through the kernel-internal
///   skip-list per §D56.8, not through this predicate.
/// - [`Self::Other`] — every other principal (admin/auditor/CEO/non-slot
///   agents). Per F3.B.role.b's M6+ deferral, the "Observer" matrix column
///   is partially honoured at v2: Read by Other returns
///   `Err(NotAuthorisedForRead)`. M6+ wires admin/auditor classification
///   per `D-CH18-FOLLOWUP-01`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrincipalClass {
    Requestor,
    SlotApprover {
        has_unfilled_slot: bool,
        has_filled_slot: bool,
    },
    Bootstrap,
    Other,
}

fn classify_principal(p: &PrincipalRef, ar: &AuthRequest) -> PrincipalClass {
    // Bootstrap fast-path: system-genesis principal on a bootstrap AR.
    if is_bootstrap_ar(ar) {
        if let PrincipalRef::System(s) = p {
            if s == crate::permissions::axioms::SYSTEM_GENESIS_PRINCIPAL {
                return PrincipalClass::Bootstrap;
            }
        }
    }

    // Requestor classification — relies on F1.B PartialEq derive.
    if &ar.requestor == p {
        return PrincipalClass::Requestor;
    }

    // Slot-approver classification — scan every (resource_slot, approver_slot)
    // pair for ownership matches. A principal may own multiple slots; we
    // record whether any are unfilled and whether any are filled so the
    // matrix arms can distinguish the two sub-classes.
    let mut has_unfilled_slot = false;
    let mut has_filled_slot = false;
    for resource_slot in &ar.resource_slots {
        for approver_slot in &resource_slot.approvers {
            if &approver_slot.approver == p {
                match approver_slot.state {
                    ApproverSlotState::Unfilled => has_unfilled_slot = true,
                    ApproverSlotState::Approved | ApproverSlotState::Denied => {
                        has_filled_slot = true
                    }
                }
            }
        }
    }
    if has_unfilled_slot || has_filled_slot {
        return PrincipalClass::SlotApprover {
            has_unfilled_slot,
            has_filled_slot,
        };
    }

    PrincipalClass::Other
}

/// Wire-form classification string for [`AuthRequestAccessError`] payloads.
fn principal_kind_str(p: &PrincipalRef) -> String {
    match p {
        PrincipalRef::Agent(_) => "agent".into(),
        PrincipalRef::User(_) => "user".into(),
        PrincipalRef::Organization(_) => "organization".into(),
        PrincipalRef::Project(_) => "project".into(),
        PrincipalRef::System(_) => "system".into(),
    }
}

// ---------------------------------------------------------------------------
// check_auth_request_access — the per-state matrix predicate (§D56.1)
// ---------------------------------------------------------------------------

/// Pure-function gate over the per-state access matrix at concept doc 02
/// lines 130–144.
///
/// Returns `Ok(())` when the matrix admits the (state × principal-class ×
/// intended_op) cell; returns a typed [`AuthRequestAccessError`] otherwise.
/// No I/O — callers are responsible for fetching `ar` from the Repository
/// before invoking this predicate.
///
/// ## Bootstrap fast-path
///
/// When [`is_bootstrap_ar(ar)`](is_bootstrap_ar) is true and the principal
/// is the `system:genesis` axiom, the principal is classified as
/// [`PrincipalClass::Bootstrap`] and Read is permitted at every state.
/// Mutation on the bootstrap AR goes through the kernel-internal skip-list
/// per §D56.8, not through this predicate.
///
/// ## Resource-owner descope
///
/// Per F3.B.role.b at gate-1, the "Resource Owner" matrix column is
/// partially honoured at v2: [`IntendedOp::Revoke`] is permitted only when
/// `ar.requestor == principal` (adoption-AR self-revoke), per
/// `D-CH18-FOLLOWUP-01`. M6+ wires the full resource-owner classification.
///
/// ## Observer descope
///
/// Per F3.B.role.b at gate-1, the "Observer (admin/auditor)" matrix column
/// is partially honoured at v2: only the requestor, slot approvers, and
/// the bootstrap principal can Read; every other principal receives
/// `Err(NotAuthorisedForRead)` — including the org's CEO when reading
/// another agent's AR. M6+ wires admin/auditor classification per
/// `D-CH18-FOLLOWUP-01`.
pub fn check_auth_request_access(
    ar: &AuthRequest,
    principal: &PrincipalRef,
    intended_op: IntendedOp,
) -> Result<(), AuthRequestAccessError> {
    let class = classify_principal(principal, ar);
    let state = ar.state;

    // Bootstrap fast-path: any-state Read by `system:genesis` on the
    // bootstrap AR is always permitted. Mutation paths on the bootstrap
    // AR are kernel-internal (§D56.8) and never reach this predicate.
    if class == PrincipalClass::Bootstrap {
        return match intended_op {
            IntendedOp::Read => Ok(()),
            _ => Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op }),
        };
    }

    // Closed terminal states (Denied / Cancelled / Revoked / Expired):
    // Read permitted to requestor + slot approvers (matrix rows 5/8/9 +
    // Revoked); every modify is forbidden per `is_closed_terminal`.
    if is_closed_terminal(state) {
        return match intended_op {
            IntendedOp::Read => match class {
                PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
                PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                    state,
                    principal_kind: principal_kind_str(principal),
                }),
                PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
            },
            _ => Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op }),
        };
    }

    // Per-state arms. Below this line `state` is one of:
    // Draft, Pending, InProgress, Approved, Partial.
    match (state, intended_op) {
        // --- Draft row ---------------------------------------------------
        (AuthRequestState::Draft, IntendedOp::Read) => match class {
            PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
            PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                state,
                principal_kind: principal_kind_str(principal),
            }),
            PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
        },
        (AuthRequestState::Draft, IntendedOp::Modify | IntendedOp::Submit | IntendedOp::Cancel) => {
            match class {
                PrincipalClass::Requestor => Ok(()),
                _ => Err(AuthRequestAccessError::RequestorOnlyOperation { state, intended_op }),
            }
        }
        (AuthRequestState::Draft, _) => {
            Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op })
        }

        // --- Pending row -------------------------------------------------
        (AuthRequestState::Pending, IntendedOp::Read) => match class {
            PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
            PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                state,
                principal_kind: principal_kind_str(principal),
            }),
            PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
        },
        (AuthRequestState::Pending, IntendedOp::Cancel) => match class {
            PrincipalClass::Requestor => Ok(()),
            _ => Err(AuthRequestAccessError::RequestorOnlyOperation { state, intended_op }),
        },
        (AuthRequestState::Pending, IntendedOp::Approve | IntendedOp::Deny) => match class {
            PrincipalClass::SlotApprover {
                has_unfilled_slot: true,
                ..
            } => Ok(()),
            PrincipalClass::SlotApprover {
                has_unfilled_slot: false,
                has_filled_slot: true,
            } => Err(AuthRequestAccessError::UnfilledApproverSlotOnly { state, intended_op }),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        (AuthRequestState::Pending, _) => {
            Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op })
        }

        // --- InProgress row ----------------------------------------------
        (AuthRequestState::InProgress, IntendedOp::Read) => match class {
            PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
            PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                state,
                principal_kind: principal_kind_str(principal),
            }),
            PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
        },
        (AuthRequestState::InProgress, IntendedOp::Cancel) => match class {
            PrincipalClass::Requestor => Ok(()),
            _ => Err(AuthRequestAccessError::RequestorOnlyOperation { state, intended_op }),
        },
        (AuthRequestState::InProgress, IntendedOp::Approve | IntendedOp::Deny) => match class {
            PrincipalClass::SlotApprover {
                has_unfilled_slot: true,
                ..
            } => Ok(()),
            PrincipalClass::SlotApprover {
                has_unfilled_slot: false,
                has_filled_slot: true,
            } => Err(AuthRequestAccessError::UnfilledApproverSlotOnly { state, intended_op }),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        (AuthRequestState::InProgress, IntendedOp::Reconsider) => match class {
            PrincipalClass::SlotApprover {
                has_filled_slot: true,
                ..
            } => Ok(()),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        (AuthRequestState::InProgress, _) => {
            Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op })
        }

        // --- Approved row ------------------------------------------------
        (AuthRequestState::Approved, IntendedOp::Read) => match class {
            PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
            PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                state,
                principal_kind: principal_kind_str(principal),
            }),
            PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
        },
        // `Reconsider` on an Approved AR by a filled slot approver is
        // matrix-permitted (row 4 col 4); other classes are denied.
        (AuthRequestState::Approved, IntendedOp::Reconsider) => match class {
            PrincipalClass::SlotApprover {
                has_filled_slot: true,
                ..
            } => Ok(()),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        // `Revoke` on an Approved AR is descoped per F3.B.role.b: only
        // permitted when `ar.requestor == principal` (adoption-AR
        // self-revoke). M6+ wires the full resource-owner classification.
        (AuthRequestState::Approved, IntendedOp::Revoke) => match class {
            PrincipalClass::Requestor => Ok(()),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        // `Expire` is a lifecycle transition; permitted on Approved /
        // Pending / InProgress regardless of principal (kernel-internal
        // schedulers; surfaced here so the predicate is total).
        (AuthRequestState::Approved, IntendedOp::Expire) => Ok(()),
        // Modify on Approved is the closed-terminal modify class — denied
        // to every principal.
        (AuthRequestState::Approved, _) => {
            Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op })
        }

        // --- Partial row -------------------------------------------------
        // Partial is semi-open per concept doc 02 line 152: Read by
        // requestor + slot approvers; Reconsider by filled slot approvers
        // (until owner closes the record); OverrideApprove / CloseAsDenied
        // are owner-class ops, descoped per F3.B.role.b like Revoke
        // (requestor-only stub).
        (AuthRequestState::Partial, IntendedOp::Read) => match class {
            PrincipalClass::Requestor | PrincipalClass::SlotApprover { .. } => Ok(()),
            PrincipalClass::Other => Err(AuthRequestAccessError::NotAuthorisedForRead {
                state,
                principal_kind: principal_kind_str(principal),
            }),
            PrincipalClass::Bootstrap => unreachable!("bootstrap handled above"),
        },
        (AuthRequestState::Partial, IntendedOp::Reconsider) => match class {
            PrincipalClass::SlotApprover {
                has_filled_slot: true,
                ..
            } => Ok(()),
            _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                state,
                principal_kind: principal_kind_str(principal),
                intended_op,
            }),
        },
        (AuthRequestState::Partial, IntendedOp::OverrideApprove | IntendedOp::CloseAsDenied) => {
            match class {
                PrincipalClass::Requestor => Ok(()),
                _ => Err(AuthRequestAccessError::NotAuthorisedForModify {
                    state,
                    principal_kind: principal_kind_str(principal),
                    intended_op,
                }),
            }
        }
        (AuthRequestState::Partial, IntendedOp::Expire) => Ok(()),
        (AuthRequestState::Partial, _) => {
            Err(AuthRequestAccessError::OperationForbiddenInState { state, intended_op })
        }

        // Closed-terminal arms unreachable here — handled above the
        // per-state match.
        (AuthRequestState::Denied, _)
        | (AuthRequestState::Expired, _)
        | (AuthRequestState::Revoked, _)
        | (AuthRequestState::Cancelled, _) => {
            unreachable!("closed terminals handled by is_closed_terminal arm above")
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — per ADR-0056 §D56.5 cell-class coverage
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::{AgentId, AuthRequestId};
    use crate::model::nodes::{ApproverSlot, ResourceRef, ResourceSlot, ResourceSlotState};
    use crate::permissions::axioms::{system_bootstrap_template_id, system_genesis_principal};
    use chrono::Utc;

    fn ar_with_state_and_slots(
        requestor: PrincipalRef,
        state: AuthRequestState,
        resource_slots: Vec<ResourceSlot>,
    ) -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor,
            kinds: vec!["filesystem_object".into()],
            scope: vec!["read".into()],
            state,
            valid_until: None,
            submitted_at: Utc::now(),
            resource_slots,
            justification: None,
            audit_class: AuditClass::Logged,
            terminal_state_entered_at: None,
            archived: false,
            active_window_days: 90,
            provenance_template: None,
            tags: vec![],
            descends_from_grant: None,
        }
    }

    fn slot_with_approver(approver: PrincipalRef, state: ApproverSlotState) -> ResourceSlot {
        ResourceSlot {
            resource: ResourceRef {
                uri: "filesystem:/workspace/x".into(),
            },
            approvers: vec![ApproverSlot {
                approver,
                state,
                responded_at: None,
                reconsidered_at: None,
            }],
            state: ResourceSlotState::InProgress,
        }
    }

    // ---- Draft row ------------------------------------------------------

    #[test]
    fn draft_read_by_requestor_ok() {
        let requestor = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Draft,
            vec![],
        );
        assert!(
            check_auth_request_access(&ar, &PrincipalRef::Agent(requestor), IntendedOp::Read,)
                .is_ok()
        );
    }

    #[test]
    fn draft_read_by_non_requestor_err() {
        let requestor = AgentId::new();
        let other = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Draft,
            vec![],
        );
        let err = check_auth_request_access(&ar, &PrincipalRef::Agent(other), IntendedOp::Read)
            .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::NotAuthorisedForRead { .. }
        ));
    }

    #[test]
    fn draft_modify_by_requestor_ok() {
        let requestor = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Draft,
            vec![],
        );
        assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor),
            IntendedOp::Modify,
        )
        .is_ok());
    }

    #[test]
    fn draft_modify_by_non_requestor_err() {
        let requestor = AgentId::new();
        let other = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Draft,
            vec![],
        );
        let err = check_auth_request_access(&ar, &PrincipalRef::Agent(other), IntendedOp::Modify)
            .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::RequestorOnlyOperation {
                intended_op: IntendedOp::Modify,
                ..
            }
        ));
    }

    #[test]
    fn draft_submit_by_requestor_ok() {
        let requestor = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Draft,
            vec![],
        );
        assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor),
            IntendedOp::Submit,
        )
        .is_ok());
    }

    // ---- Pending row ----------------------------------------------------

    #[test]
    fn pending_approve_by_unfilled_slot_approver_ok() {
        let requestor = AgentId::new();
        let approver = AgentId::new();
        let slot = slot_with_approver(PrincipalRef::Agent(approver), ApproverSlotState::Unfilled);
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Pending,
            vec![slot],
        );
        assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(approver),
            IntendedOp::Approve,
        )
        .is_ok());
    }

    #[test]
    fn pending_approve_by_filled_slot_approver_err() {
        let requestor = AgentId::new();
        let approver = AgentId::new();
        let slot = slot_with_approver(PrincipalRef::Agent(approver), ApproverSlotState::Approved);
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Pending,
            vec![slot],
        );
        let err =
            check_auth_request_access(&ar, &PrincipalRef::Agent(approver), IntendedOp::Approve)
                .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::UnfilledApproverSlotOnly {
                intended_op: IntendedOp::Approve,
                ..
            }
        ));
    }

    #[test]
    fn pending_approve_by_non_slot_agent_err() {
        let requestor = AgentId::new();
        let approver = AgentId::new();
        let stranger = AgentId::new();
        let slot = slot_with_approver(PrincipalRef::Agent(approver), ApproverSlotState::Unfilled);
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Pending,
            vec![slot],
        );
        let err =
            check_auth_request_access(&ar, &PrincipalRef::Agent(stranger), IntendedOp::Approve)
                .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::NotAuthorisedForModify {
                intended_op: IntendedOp::Approve,
                ..
            }
        ));
    }

    // ---- InProgress row -------------------------------------------------

    #[test]
    fn in_progress_reconsider_by_filled_slot_approver_own_slot_ok() {
        let requestor = AgentId::new();
        let approver = AgentId::new();
        let slot = slot_with_approver(PrincipalRef::Agent(approver), ApproverSlotState::Approved);
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::InProgress,
            vec![slot],
        );
        assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(approver),
            IntendedOp::Reconsider,
        )
        .is_ok());
    }

    #[test]
    fn in_progress_reconsider_by_filled_slot_approver_other_slot_err() {
        // Two-slot AR. `approver_a` filled their slot; `approver_b`'s slot
        // is still unfilled. `approver_b` is a slot approver but has no
        // FILLED slot — `Reconsider` is not permitted (matrix row 3 col 4
        // requires "re-edit own slot"; the predicate's filled-slot
        // sub-class is the test for "own filled slot").
        let requestor = AgentId::new();
        let approver_a = AgentId::new();
        let approver_b = AgentId::new();
        let slot_a =
            slot_with_approver(PrincipalRef::Agent(approver_a), ApproverSlotState::Approved);
        let slot_b =
            slot_with_approver(PrincipalRef::Agent(approver_b), ApproverSlotState::Unfilled);
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::InProgress,
            vec![slot_a, slot_b],
        );
        let err = check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(approver_b),
            IntendedOp::Reconsider,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::NotAuthorisedForModify {
                intended_op: IntendedOp::Reconsider,
                ..
            }
        ));
    }

    // ---- Approved row ---------------------------------------------------

    #[test]
    fn approved_modify_by_anyone_err() {
        let requestor = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Approved,
            vec![],
        );
        // Even the requestor cannot Modify a closed-terminal-side AR.
        let err =
            check_auth_request_access(&ar, &PrincipalRef::Agent(requestor), IntendedOp::Modify)
                .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::OperationForbiddenInState {
                intended_op: IntendedOp::Modify,
                ..
            }
        ));
    }

    #[test]
    fn approved_revoke_by_resource_owner_ok() {
        // F3.B.role.b descope: `Revoke` is permitted when
        // `ar.requestor == principal` (adoption-AR self-revoke stub).
        let requestor = AgentId::new();
        let ar = ar_with_state_and_slots(
            PrincipalRef::Agent(requestor),
            AuthRequestState::Approved,
            vec![],
        );
        assert!(check_auth_request_access(
            &ar,
            &PrincipalRef::Agent(requestor),
            IntendedOp::Revoke,
        )
        .is_ok());
    }

    // ---- Closed terminal rows (Denied / Cancelled / Revoked / Expired) --

    #[test]
    fn closed_terminals_read_by_any_ok() {
        let requestor = AgentId::new();
        let approver = AgentId::new();
        let slot = slot_with_approver(PrincipalRef::Agent(approver), ApproverSlotState::Approved);
        for state in [
            AuthRequestState::Denied,
            AuthRequestState::Cancelled,
            AuthRequestState::Revoked,
            AuthRequestState::Expired,
        ] {
            let ar =
                ar_with_state_and_slots(PrincipalRef::Agent(requestor), state, vec![slot.clone()]);
            // Requestor reads.
            assert!(
                check_auth_request_access(&ar, &PrincipalRef::Agent(requestor), IntendedOp::Read,)
                    .is_ok(),
                "requestor read on closed terminal {:?}",
                state,
            );
            // Filled slot approver reads.
            assert!(
                check_auth_request_access(&ar, &PrincipalRef::Agent(approver), IntendedOp::Read,)
                    .is_ok(),
                "approver read on closed terminal {:?}",
                state,
            );
        }
    }

    #[test]
    fn closed_terminals_modify_by_any_err() {
        let requestor = AgentId::new();
        for state in [
            AuthRequestState::Denied,
            AuthRequestState::Cancelled,
            AuthRequestState::Revoked,
            AuthRequestState::Expired,
        ] {
            let ar = ar_with_state_and_slots(PrincipalRef::Agent(requestor), state, vec![]);
            let err =
                check_auth_request_access(&ar, &PrincipalRef::Agent(requestor), IntendedOp::Modify)
                    .unwrap_err();
            assert!(
                matches!(
                    err,
                    AuthRequestAccessError::OperationForbiddenInState { .. }
                ),
                "closed-terminal {:?} must reject Modify",
                state,
            );
        }
    }

    // ---- Bootstrap (system-genesis) row --------------------------------

    #[test]
    fn system_genesis_bootstrap_ar_any_op_by_any_non_system_principal_err() {
        // Bootstrap-AR Read is permitted to the system-genesis principal
        // only. A non-system principal asking to Read the bootstrap AR
        // hits the Other classification → NotAuthorisedForRead.
        let stranger = AgentId::new();
        let mut ar = ar_with_state_and_slots(
            system_genesis_principal(),
            AuthRequestState::Approved,
            vec![],
        );
        ar.provenance_template = Some(system_bootstrap_template_id());
        // Bootstrap principal Read works.
        assert!(
            check_auth_request_access(&ar, &system_genesis_principal(), IntendedOp::Read,).is_ok()
        );
        // Non-system principal Read on the bootstrap AR — `classify_principal`
        // sees the principal as Other (the bootstrap fast-path gate is
        // SYSTEM_GENESIS-only) → NotAuthorisedForRead.
        let err = check_auth_request_access(&ar, &PrincipalRef::Agent(stranger), IntendedOp::Read)
            .unwrap_err();
        assert!(matches!(
            err,
            AuthRequestAccessError::NotAuthorisedForRead { .. }
        ));
    }
}
