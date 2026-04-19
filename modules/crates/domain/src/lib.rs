//! baby-phi domain layer.
//!
//! This crate is storage-agnostic. It defines:
//!
//! - The graph model (nodes, edges, fundamentals, composites) from
//!   `docs/specs/v0/concepts/ontology.md`.
//! - The Permission Check engine (`permissions/04-manifest-and-resolution.md`).
//! - The Auth Request state machine (`permissions/02-auth-request.md`).
//! - The system-flow state machines (s01–s06 under `concepts/system-flows/`).
//!
//! The `Repository` trait exposes the persistence boundary. Adapters live in
//! the `store` crate.

pub mod model;
pub mod permissions;
pub mod repository;
pub mod state_machines;

pub use repository::Repository;
