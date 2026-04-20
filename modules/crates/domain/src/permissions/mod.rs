//! Permission Check engine — the runtime's authorization spine.
//!
//! The engine is a **pure** 6-step (+2a) pipeline. It takes a fully-materialised
//! [`CheckContext`] + a tool's [`Manifest`] and returns a [`Decision`]. Callers
//! (P6 HTTP handlers, the bootstrap flow in P5) are responsible for assembling
//! the context from the `Repository`; the engine itself does no I/O.
//!
//! Source of truth: `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md`.
//!
//! ## Stage overview
//!
//! ```text
//!   ┌─ Step 0 ─ Catalogue precondition (each reach must be declared)
//!   ├─ Step 1 ─ Expand manifest → (fundamental, action) reaches
//!   ├─ Step 2 ─ Gather candidate grants from agent + project + org
//!   ├─ Step 2a ─ Ceiling clamp (top-down bound)
//!   ├─ Step 3 ─ Match each reach → ≥1 candidate grant (else Denied)
//!   ├─ Step 5 ─ Scope resolution (most-specific-first cascade)
//!   ├─ Step 4 ─ Constraint check vs winning grant
//!   └─ Step 6 ─ Consent gating (Template A–D) → Pending if missing
//! ```
//!
//! Steps 4 and 5 appear swapped in source order because the concept doc's
//! pseudocode (`04-manifest-and-resolution.md` §Formal Algorithm) defers the
//! constraint check until after scope resolution picks a winner.
//!
//! ## Metric recording
//!
//! Every call to [`check`] records latency + result via the caller-provided
//! [`PermissionCheckMetrics`] sink (per M1 commitment C10). The domain crate
//! ships a [`NoopMetrics`] sink for tests; the server crate will plug a
//! Prometheus histogram into this trait in P6.

pub mod catalogue;
pub mod decision;
pub mod engine;
pub mod expansion;
pub mod manifest;
pub mod metrics;
pub mod selector;

pub use catalogue::{CatalogueLookup, StaticCatalogue};
pub use decision::{Decision, DeniedReason, FailedStep};
pub use engine::check;
pub use expansion::{expand_resource_to_fundamentals, ResolvedGrant};
pub use manifest::{CheckContext, Manifest, ToolCall};
pub use metrics::{NoopMetrics, PermissionCheckMetrics};
pub use selector::Selector;
