//! Auth Request **templates** — the named lifecycle patterns from
//! `docs/specs/v0/concepts/permissions/02-auth-request-templates.md`.
//!
//! Each template kind (`TemplateKind::{SystemBootstrap, A, B, C, D, E, F}`)
//! corresponds to one distinct authorisation shape. Rather than scatter
//! the "how do I mint an Auth Request of kind X" logic across every
//! handler that needs it, each variant gets a dedicated **pure helper**
//! in its own sub-module.
//!
//! Helpers return a fully-shaped `AuthRequest` (plus any companion
//! structs like `Grant`s) ready for the caller to persist. The helpers
//! do NOT perform I/O; handlers own transactional composition.
//!
//! ## What ships per milestone
//!
//! - **M2/P3** — [`e::build_auto_approved_request`] (Template E, the
//!   "self-interested auto-approve" pattern every page-02–05 write uses).
//! - **M3/P2** — [`a::build_adoption_request`] /
//!   [`b::build_adoption_request`] / [`c::build_adoption_request`] /
//!   [`d::build_adoption_request`]. At org-creation time the CEO
//!   auto-adopts each enabled template by constructing a
//!   pre-approved adoption AR. **M3 wires adoption only**; the
//!   per-template trigger-fire AR (e.g. one AR per `HAS_LEAD` edge
//!   firing for Template A) is M5 work and gets its own builder set
//!   alongside the event-subscription wiring.
//! - **M6 (reserved)** — Template F (break-glass). Not wired in M3.
//!
//! See [`../../../../docs/specs/v0/implementation/m3/architecture/authority-templates.md`](../../../../docs/specs/v0/implementation/m3/architecture/authority-templates.md)
//! for the semantic recap and the M3→M5 scope boundary.

pub mod a;
pub mod adoption;
pub mod b;
pub mod c;
pub mod d;
pub mod e;
