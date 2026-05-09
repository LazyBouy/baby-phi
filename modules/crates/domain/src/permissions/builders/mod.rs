//! Manifest builders — pure-fn constructors that produce the engine's
//! [`super::manifest::Manifest`] input shape for synthetic call sites
//! (session launch, preview, SSE live-event tail, future per-tool
//! runtime invocations, memory recall, etc.).
//!
//! Builders live here rather than in `manifest/` because `manifest/`
//! hosts the engine-input contract type itself; constructors fan out
//! per call-site domain. As of CH-17 two builders ship:
//! - `session_launch::build_session_launch_manifest` —
//!   `[Read, Inspect, List]` on `session_object` (ADR-0054 §D54.1).
//! - `session_observe::build_session_observe_manifest` —
//!   `[Observe]` on `session_object` (ADR-0055 §D55.5). Sibling to
//!   the launch builder; the SSE handler uses this for the live-event
//!   tail gate.
//!
//! M6+ per-tool runtime + memory-recall builders co-locate here when
//! they land.
//!
//! Source of truth: ADR-0054 §D54.1, ADR-0055 §D55.5.

pub mod session_launch;
pub mod session_observe;

pub use session_launch::build_session_launch_manifest;
pub use session_observe::build_session_observe_manifest;
