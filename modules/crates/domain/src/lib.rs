//! baby-phi domain layer.
//!
//! This crate is storage-agnostic. It defines:
//!
//! - The graph model (9 fundamentals + 8 composites + 37 node types + 66
//!   edge types) from `docs/specs/v0/concepts/ontology.md`.
//! - The audit-event framework (`crate::audit`), including the hash-chain
//!   seed that M7b promotes to a tamper-evident off-site stream.
//! - The Permission Check engine (lands in P3 —
//!   `permissions/04-manifest-and-resolution.md`).
//! - The Auth Request state machine (`crate::auth_requests` —
//!   `permissions/02-auth-request.md`).
//! - The system-flow state machines (s01 in P5; s02–s06 in later milestones).
//!
//! The `Repository` trait exposes the persistence boundary. Adapters live in
//! the `store` crate.

pub mod audit;
pub mod auth_requests;
pub mod model;
pub mod permissions;
pub mod repository;
pub mod templates;

#[cfg(any(test, feature = "in-memory-repo"))]
pub mod in_memory;

pub use repository::Repository;
