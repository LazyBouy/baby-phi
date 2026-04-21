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
//! M2 ships:
//! - [`e::build_auto_approved_request`] — Template E, the "self-interested
//!   auto-approve" pattern every page-02–05 write uses (the platform
//!   admin is both requestor and approver).
//!
//! Templates A / B / C / D / F arrive with their owning milestones
//! (M3+ — full permission-system delivery).

pub mod e;
