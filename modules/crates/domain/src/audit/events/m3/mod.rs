//! M3 audit-event builders — one module per admin page.
//!
//! - [`orgs`] — pages 06 (org creation) + 07 (org dashboard).
//!   `platform.organization.created` + `authority_template.adopted`.
//!   Ships with M3/P2.
//!
//! Both events are `org_scope = Some(org_id)` — M3 is the first
//! milestone that chains under a per-org hash chain (M2 wrote every
//! event to the platform root chain). See
//! [`../../../../../docs/specs/v0/implementation/m3/architecture/per-org-audit-chain.md`](../../../../../docs/specs/v0/implementation/m3/architecture/per-org-audit-chain.md).

pub mod orgs;
