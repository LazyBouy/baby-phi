//! M4 audit-event builders — one module per vertical slice.
//!
//! - [`agents`] — pages 08 + 09 (agent roster list + profile editor).
//!   `platform.agent.created` + `platform.agent_profile.updated`.
//!   Ships with M4/P2.
//! - [`projects`] — page 10 (project creation wizard) + page 11
//!   (project detail). `platform.project.created` +
//!   `platform.project_creation.pending` +
//!   `platform.project_creation.denied`. Ships with M4/P2.
//! - [`templates`] — s05 (Template A firing on lead assignment).
//!   `template.a.grant_fired`. Ships with M4/P2, wired to the
//!   event-listener path at M4/P3.
//!
//! Every M4 event is `org_scope = Some(org_id)` — M4 continues the
//! per-org audit chain opened at M3. See
//! [`../../../../../docs/specs/v0/implementation/m3/architecture/per-org-audit-chain.md`](../../../../../docs/specs/v0/implementation/m3/architecture/per-org-audit-chain.md).
//!
//! ## phi-core leverage
//!
//! None — audit events are phi governance write log (orthogonal
//! surface per `phi/CLAUDE.md`). `phi_core::AgentEvent` is agent-
//! loop telemetry, not a hash-chained audit trail.

pub mod agents;
pub mod projects;
pub mod templates;
