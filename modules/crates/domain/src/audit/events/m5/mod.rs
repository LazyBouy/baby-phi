//! M5 audit event builders.
//!
//! At P3, the governance-plane event-bus wiring lands for Templates
//! C and D (mirrors the M4 Template A path). Additional M5 builders
//! (session lifecycle, system-agent tuning, memory extraction) join
//! this module as their phases open.

pub mod templates;
