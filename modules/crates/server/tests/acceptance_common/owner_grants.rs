//! CH-27 / ADR-0062 §D62.4 — F4.b USER-DIVERGENT opt-in helper for the
//! M3 + M4 + M5 acceptance fixture extension cascade.
//!
//! ## Purpose
//!
//! At CH-27 the 7 admin handlers in `server::platform::{orgs,projects}::*`
//! flipped from **advisory** `check_permission(...).is_ok()` consumption
//! to **blocking** `?`-propagation via `denial_to_api_error`. Acceptance
//! tests that previously relied on the platform-admin (or another
//! non-owner viewer) reaching the handler body now get HTTP 403
//! NO_GRANTS_HELD unless the viewer satisfies one of:
//!
//! 1. The CH-25 synth-owner-grant rule (owns the org via `Edge::Owns`).
//! 2. An explicit persisted `Grant` against the org URI carrying the
//!    relevant Action verb (`Allocate` / `Transfer` / `Observe` /
//!    `Inspect` per the widened CH-27 / ADR-0062 §D62.2 4-verb scope).
//!
//! This helper provides the **explicit-grant** path. It writes a real
//! persisted `Grant` row via `Repository::create_grant` covering all 4
//! universal-applicability verbs for `agent` on each `org:<O>` URI in
//! `org_ids`, mirroring the CH-25 synth-grant shape but materialised as
//! a persisted grant (audit-visible at the call site, no Edge::Owns
//! seeding needed).
//!
//! ## Wire-format-explicit per F4.b (USER-DIVERGENT)
//!
//! Tests opt in to seeing the org by explicitly invoking
//! `seed_owner_grants(repo, agent, vec![org_id])` after the org is
//! created. This is more verbose than F4.a default-extension (where
//! the helper would extend `spawn_claimed_with_org` to seed by default)
//! but per user-lock 2026-05-18 the explicit per-test call is the
//! audit-visibility-winning path. Cross-cycle precedent: CH-26 F2.b
//! (tag-field-on-struct over catalogue-entry-only) + CH-25 F1.b (NEW
//! `Edge::Owns` variant) + i-phi CH-02b F4.b (thiserror enum).
//!
//! ## SCOPE-NARROWING vs plan §3 Artifact C body
//!
//! The plan's helper-body literal (§3 Artifact C) shows
//! `repo.insert_edge(Edge::Owns { from: agent, to: org_id.into() })`
//! — but the `Repository` trait does **not** expose an
//! `insert_edge(Edge)` method, nor does it expose a single-row
//! `OwnsEdge` writer. The only canonical path to emit `Edge::Owns` is
//! via `apply_org_creation` / `apply_project_creation` (which emits
//! the edge for the CEO/lead atomically with the org/project itself).
//!
//! The narrowing preserves F4.b's spirit (wire-format-explicit per-test
//! grant seeding, audit-visible in test body) by **materialising a
//! persisted `Grant` row directly via `create_grant`** (an existing
//! Repository trait method). The engine's Step 2 (Resolution) picks up
//! this persisted grant in the candidate pool exactly as it would the
//! synth-grant — same per-grant verdict shape downstream. Surfaced for
//! orchestrator review at gate-2; flagged in §"Forks taken" of the
//! P-FIXTURES report.

use std::sync::Arc;

use chrono::Utc;
use domain::model::ids::{AgentId, GrantId, OrgId, ProjectId};
use domain::model::nodes::{ApprovalMode, Grant, PrincipalRef, ResourceRef};
use domain::model::Fundamental;
use domain::permissions::action::Action;
use domain::repository::Repository;

/// Seed an explicit Allocate+Transfer+Observe+Inspect grant for `agent`
/// on each `org:<O>` URI. Mirrors the CH-27 / ADR-0062 §D62.2 widened
/// 4-verb owner-grant scope, materialised as a persisted `Grant` row
/// (engine Step 2 picks it up in the candidate pool exactly like the
/// CH-25 synth-grant).
///
/// Tests should call this helper after the org is created (e.g., after
/// `apply_org_creation` or `bootstrap_org_via_wizard`) and before any
/// handler invocation that the test wants `agent` to pass the blocking
/// gate on.
///
/// Returns `Err` if any `create_grant` call fails.
#[allow(dead_code)] // each test binary that imports acceptance_common::mod re-compiles this; only callers that use it consume it
pub async fn seed_owner_grants(
    repo: &Arc<dyn Repository>,
    agent: AgentId,
    org_ids: Vec<OrgId>,
) -> Result<(), domain::repository::RepositoryError> {
    for org_id in org_ids {
        let uri = format!("org:{}", org_id);
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec![
                Action::Allocate,
                Action::Transfer,
                Action::Observe,
                Action::Inspect,
            ],
            resource: ResourceRef { uri },
            fundamentals: vec![Fundamental::IdentityPrincipal],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
            approval_mode: ApprovalMode::Implicit,
            audit_class: domain::audit::AuditClass::Silent,
            allocate_refinement: None,
        };
        repo.create_grant(&grant).await?;
    }
    Ok(())
}

/// Sibling variant for project URIs. Symmetric to
/// [`seed_owner_grants`] but binds the grant to each `project:<P>`
/// URI in `project_ids`.
#[allow(dead_code)] // not yet used by the M3+M4+M5 acceptance cascade
pub async fn seed_owner_grants_on_projects(
    repo: &Arc<dyn Repository>,
    agent: AgentId,
    project_ids: Vec<ProjectId>,
) -> Result<(), domain::repository::RepositoryError> {
    for project_id in project_ids {
        let uri = format!("project:{}", project_id);
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: vec![
                Action::Allocate,
                Action::Transfer,
                Action::Observe,
                Action::Inspect,
            ],
            resource: ResourceRef { uri },
            fundamentals: vec![Fundamental::IdentityPrincipal],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
            approval_mode: ApprovalMode::Implicit,
            audit_class: domain::audit::AuditClass::Silent,
            allocate_refinement: None,
        };
        repo.create_grant(&grant).await?;
    }
    Ok(())
}

/// Optional variant for non-owner-grant explicit-grant test scenarios
/// — caller picks the action set per (org, actions) pair. Use this for
/// tests that want to seed a narrower scope (e.g., observer-only) than
/// the 4-verb default of [`seed_owner_grants`].
#[allow(dead_code)] // present for plan-§3-Artifact-C-spec parity; M3+M4+M5 cascade uses the 4-verb default
pub async fn seed_owner_grants_with_explicit_grants(
    repo: &Arc<dyn Repository>,
    agent: AgentId,
    grants_to_seed: Vec<(OrgId, Vec<Action>)>,
) -> Result<(), domain::repository::RepositoryError> {
    for (org_id, actions) in grants_to_seed {
        let uri = format!("org:{}", org_id);
        let grant = Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(agent),
            action: actions,
            resource: ResourceRef { uri },
            fundamentals: vec![Fundamental::IdentityPrincipal],
            descends_from: None,
            delegable: true,
            issued_at: Utc::now(),
            revoked_at: None,
            approval_mode: ApprovalMode::Implicit,
            audit_class: domain::audit::AuditClass::Silent,
            allocate_refinement: None,
        };
        repo.create_grant(&grant).await?;
    }
    Ok(())
}
