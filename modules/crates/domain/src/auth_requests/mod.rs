//! Auth Request state machine.
//!
//! Source of truth:
//! `docs/specs/v0/concepts/permissions/02-auth-request.md` §Auth Request
//! Lifecycle, §State Machine, §Retention Policy.
//!
//! ## Shape
//!
//! An [`AuthRequest`](crate::model::nodes::AuthRequest) carries a list of
//! [`ResourceSlot`](crate::model::nodes::ResourceSlot)s. Each slot has an
//! ordered list of [`ApproverSlot`](crate::model::nodes::ApproverSlot)s
//! (one per required approver). State **aggregates upward**:
//!
//! ```text
//!   ApproverSlot.state  ──aggregate──▶  ResourceSlot.state  ──aggregate──▶  AuthRequest.state
//!   (3 variants)                         (5 variants)                       (9 variants)
//! ```
//!
//! The 9 request-level states (Draft / Pending / InProgress / Approved /
//! Denied / Partial / Expired / Revoked / Cancelled) are named by
//! [`AuthRequestState`](crate::model::nodes::AuthRequestState); the 5
//! resource-level states by
//! [`ResourceSlotState`](crate::model::nodes::ResourceSlotState); the 3
//! approver-level states by
//! [`ApproverSlotState`](crate::model::nodes::ApproverSlotState).
//!
//! ## Module layout
//!
//! | File | Purpose |
//! |---|---|
//! | [`state`] | Aggregation helpers (`aggregate_resource_state`, `aggregate_request_state`) + terminal-state predicates + the legal-transition table. |
//! | [`transitions`] | The public transition surface: [`transitions::submit`], [`transitions::transition_slot`], [`transitions::reconsider_slot`], [`transitions::cancel`], [`transitions::override_approve`], [`transitions::close_as_denied`]. Every op returns a new `AuthRequest` or a typed `TransitionError`. |
//! | [`revocation`] | Forward-only revocation ([`revocation::revoke`]) — Approved/Partial → Revoked. Emits the `auth_request.revoked` audit-event template consumed by P5. |
//! | [`retention`] | Active-window math (`active_until`, `is_archive_eligible`) for the two-tier storage policy (default 90-day active window; compliance purge off by default). |
//!
//! ## Invariants shipped in M1/P4
//!
//! 1. Illegal transitions never succeed ([`transitions::TransitionError`]).
//! 2. Aggregation matches `concepts/permissions/02` §State Machine tables.
//! 3. Revocation is forward-only — once a slot is marked Revoked it stays
//!    Revoked.
//! 4. Closed terminal states (Approved/Denied/Expired/Revoked/Cancelled)
//!    admit no further transitions except `revoke`/`expire` from Approved.
//! 5. Slot independence — one approver's state change never mutates
//!    another approver's state.
//! 6. Active-window countdown is monotonically non-increasing.
//!
//! Each invariant is exercised by a proptest file under
//! `modules/crates/domain/tests/auth_request_*_props.rs`.

pub mod retention;
pub mod revocation;
pub mod state;
pub mod transitions;

pub use retention::{active_until, is_archive_eligible, ActiveWindow};
pub use revocation::{revoke, RevocationError};
pub use state::{
    aggregate_request_state, aggregate_resource_state, is_closed_terminal, is_terminal,
    legal_request_transition,
};
pub use transitions::{
    cancel, close_as_denied, expire, override_approve, reconsider_slot, submit, transition_slot,
    TransitionError,
};
