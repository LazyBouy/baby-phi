//! Manifest builders — pure-fn constructors that produce the engine's
//! [`super::manifest::Manifest`] input shape for synthetic call sites
//! (session launch, preview, future per-tool runtime invocations,
//! memory recall, etc.).
//!
//! Builders live here rather than in `manifest/` because `manifest/`
//! hosts the engine-input contract type itself; constructors fan out
//! per call-site domain. As of CH-15 only the session-launch builder
//! ships; the M6+ per-tool runtime + memory-recall builders co-locate
//! here when they land.
//!
//! Source of truth: ADR-0054 §D54.1.

pub mod session_launch;

pub use session_launch::build_session_launch_manifest;
