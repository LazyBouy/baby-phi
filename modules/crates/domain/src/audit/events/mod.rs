//! Per-milestone audit-event builder functions. Each builder returns a
//! fully-formed [`crate::audit::AuditEvent`] with `prev_event_hash =
//! None`; the [`crate::audit::AuditEmitter`] implementation populates
//! the chain link at emit time.
//!
//! Pinning the diff shape in a constructor (rather than letting each
//! handler author its own JSON literal) keeps the wire format stable
//! across handler + test authors — handlers call the builder; tests
//! assert on the same shape. See decision D12 in the archived M2 plan.

pub mod m2;
pub mod m3;
pub mod m4;
