//! Consent state machine — CH-10 / ADR-0047.
//!
//! Source of truth:
//! `docs/specs/v0/concepts/permissions/06-multi-scope-consent.md` §"Consent
//! Lifecycle" + §"Forward-only revocation" + §"Default response on timeout".
//!
//! ## Shape
//!
//! A [`Consent`](crate::model::nodes::Consent) carries a
//! [`ConsentState`](crate::model::nodes::ConsentState) field that walks the
//! 6-state lifecycle:
//!
//! ```text
//!   Requested ──ack──▶ Acknowledged ──revoke (if revocable)──▶ Revoked
//!       │
//!       ├─decline──▶ Declined
//!       │
//!       └─timeout──▶ TimedOut
//!
//!   any non-terminal ──natural-scope-end──▶ Expired
//! ```
//!
//! ## Layers
//!
//! - [`state`] — pure predicates (`is_terminal`, `legal_transition`) that
//!   carry the legal-transition table from the concept doc verbatim.
//! - [`transitions`] — five pure-fn helpers (`acknowledge`, `decline`,
//!   `revoke`, `mark_timed_out`, `mark_expired`) returning
//!   `Result<Consent, ConsentTransitionError>`. Each clones the input,
//!   validates legality, stamps the appropriate timestamp field, and
//!   returns the new row.
//!
//! ## Invariants
//!
//! - **Forward-only revocation**: `Revoked → Acknowledged` is rejected.
//!   Forward-only revocation is enforced at the `legal_transition` level
//!   so every code path that builds on the table inherits the invariant.
//! - **Revocable gate**: `Acknowledged → Revoked` is rejected when
//!   `revocable == false` on the source row. The pure-fn checks this
//!   AFTER the state-machine legality check.
//! - **Terminal stickiness**: `Revoked` / `Declined` / `TimedOut` /
//!   `Expired` reject every outbound transition.
//!
//! ## phi-core leverage
//!
//! Zero. phi-core has no governance-consent concept. This module is
//! baby-phi-native.

pub mod minters;
pub mod state;
pub mod transitions;

pub use state::{is_terminal, legal_transition};
pub use transitions::{
    acknowledge, decline, mark_expired, mark_timed_out, revoke, ConsentTransitionError,
};
