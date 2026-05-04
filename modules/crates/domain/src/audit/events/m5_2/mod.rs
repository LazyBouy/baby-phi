//! M5.2 audit event builders.
//!
//! M5.2 is the post-M5 carve-out shipping CH-06 (selector grammar +
//! instance identity tags), CH-16 (Identity node materialization), and
//! CH-21 (memory-extraction listener body — `platform.memory.extracted`
//! audit + first emitter of CH-16's `platform.identity.updated`).
//! Identity- and memory-lifecycle events live here because they
//! overlap the M5 session-end + agent-creation txs but were not in
//! M5's original scope.

pub mod identity;
pub mod memory;
pub mod tool_authority;
