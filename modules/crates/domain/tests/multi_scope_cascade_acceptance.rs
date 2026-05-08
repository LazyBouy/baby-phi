//! CH-07 / ADR-0051 — Multi-scope cascade acceptance tests.
//!
//! Reproduces the worked-example scenarios from
//! `concepts/permissions/08-worked-example.md` end-to-end via real
//! [`CheckContext`] + [`StaticCatalogue`] + [`ConsentIndex`] (no
//! engine-internal `Fixture`). Each test pins one scenario or one
//! regression-protection slice.
//!
//! Scenarios covered (per plan §7 P3 deliverable 1):
//!
//! 1. Scenario 4 — `lead-acme-1` reading joint-research session →
//!    `org:acme` resolution → `Allowed`.
//! 2. Scenario 5 — `lead-beta-1` reading joint-research session →
//!    `org:beta-corp` resolution → `Allowed`.
//! 3. Scenario 6 — `lead-gamma-1` reading joint-research session →
//!    `IntersectionEmpty` `Denied`.
//! 4. Scenario 7 — `contractor-x-9` reading `acme-website-redesign`
//!    session, base_org=Gamma; session tagged `[org:acme]` only →
//!    project-tier resolution succeeds → `Allowed`.
//! 5. Shape A baseline regression — single-project single-org session,
//!    reader is owner → project-tier picks (regression protection for
//!    the empty-tags / pre-CH-07 path).
//! 6. Shape D system session — 0 projects, 1 org, system reader →
//!    org-tier picks.
//!
//! These scenarios are the "tabletop test" against concept-doc 08; their
//! purpose is to surface any divergence between concept-doc semantics
//! and the typed-Rust cascade body shipped at
//! `domain::permissions::engine::{step_5_scope_resolution,
//! step_2a_ceiling}`.

use std::collections::HashSet;

use chrono::{TimeZone, Utc};
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, AuthRequestId, GrantId, OrgId, ProjectId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::permissions::{
    catalogue::StaticCatalogue,
    check,
    decision::{DeniedReason, FailedStep},
    manifest::{CheckContext, ConsentIndex, Manifest, ToolCall},
    metrics::NoopMetrics,
    Action, Decision,
};

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

fn grant(holder: PrincipalRef, actions: &[Action], resource_uri: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: actions.to_vec(),
        resource: ResourceRef {
            uri: resource_uri.into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        revoked_at: None,
        approval_mode: domain::model::ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    }
}

fn read_filesystem_manifest() -> Manifest {
    Manifest {
        actions: vec![Action::Read],
        resource: vec!["filesystem_object".into()],
        ..Default::default()
    }
}

/// Owns the `Vec<Grant>`s + catalogue + consent index so a
/// `CheckContext<'a>` can borrow into them. Mirrors `tests/common`'s
/// `OwnedCtx` shape but keeps the new CH-07 multi-scope tag slices
/// first-class so the tests below can populate them per-scenario.
struct AcceptanceCtx {
    agent: AgentId,
    current_org: Option<OrgId>,
    current_project: Option<ProjectId>,
    agent_grants: Vec<Grant>,
    project_grants: Vec<Grant>,
    org_grants: Vec<Grant>,
    ceiling_grants: Vec<Grant>,
    catalogue: StaticCatalogue,
    consents: ConsentIndex,
    template_gated: HashSet<AuthRequestId>,
    session_org_tags: Vec<OrgId>,
    session_project_tags: Vec<ProjectId>,
}

impl AcceptanceCtx {
    fn new() -> Self {
        Self {
            agent: AgentId::new(),
            current_org: None,
            current_project: None,
            agent_grants: vec![],
            project_grants: vec![],
            org_grants: vec![],
            ceiling_grants: vec![],
            catalogue: StaticCatalogue::empty(),
            consents: ConsentIndex::empty(),
            template_gated: HashSet::new(),
            session_org_tags: vec![],
            session_project_tags: vec![],
        }
    }

    fn borrow<'a>(&'a self, call: ToolCall) -> CheckContext<'a> {
        CheckContext {
            agent: self.agent,
            current_org: self.current_org,
            current_project: self.current_project,
            current_session: None,
            agent_grants: &self.agent_grants,
            project_grants: &self.project_grants,
            org_grants: &self.org_grants,
            ceiling_grants: &self.ceiling_grants,
            catalogue: &self.catalogue,
            consents: &self.consents,
            timeout_default_response: domain::model::TimeoutResponse::Deny,
            template_gated_auth_requests: &self.template_gated,
            set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
            session_org_tags: &self.session_org_tags,
            session_project_tags: &self.session_project_tags,
            call,
        }
    }
}

// ----------------------------------------------------------------------------
// Concept-08 §"Step 4: Multi-Scope resolution for joint-research"
// (lines 192–222) — Scenarios 4 / 5 / 6.
// ----------------------------------------------------------------------------

/// Scenario 4 — `lead-acme-1` reads joint-research session.
///
/// The session is jointly tagged `[org:acme, org:beta-corp]` (Shape B);
/// `lead-acme-1` is a member of Acme only. Cascade falls through the
/// (empty) project tier into the org tier; `reader_org_matches = {acme}`
/// → `count == 1` → pick the Acme grant. Result: `Allowed`.
#[test]
fn scenario_4_lead_acme_reads_joint_research_via_org_acme() {
    let acme = OrgId::new();
    let beta = OrgId::new();
    let mut ctx = AcceptanceCtx::new();
    let acme_grant = grant(
        PrincipalRef::Organization(acme),
        &[Action::Read],
        "filesystem_object",
    );
    ctx.org_grants.push(acme_grant.clone());
    // Session is owned jointly by acme + beta-corp (Shape B).
    ctx.session_org_tags = vec![acme, beta];

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    let resolved = match d {
        Decision::Allowed { resolved_grants } => resolved_grants,
        other => panic!("Scenario 4 expected Allowed, got {:?}", other),
    };
    assert_eq!(
        resolved[0].grant_id, acme_grant.id,
        "Scenario 4: lead-acme-1's Acme org grant must win"
    );
}

/// Scenario 5 — `lead-beta-1` reads the same joint-research session.
///
/// Same Shape B session as Scenario 4, but the reader is a Beta
/// member only. `reader_org_matches = {beta}` → `count == 1` → pick the
/// Beta grant. Result: `Allowed`.
#[test]
fn scenario_5_lead_beta_reads_joint_research_via_org_beta() {
    let acme = OrgId::new();
    let beta = OrgId::new();
    let mut ctx = AcceptanceCtx::new();
    let beta_grant = grant(
        PrincipalRef::Organization(beta),
        &[Action::Read],
        "filesystem_object",
    );
    ctx.org_grants.push(beta_grant.clone());
    ctx.session_org_tags = vec![acme, beta];

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    let resolved = match d {
        Decision::Allowed { resolved_grants } => resolved_grants,
        other => panic!("Scenario 5 expected Allowed, got {:?}", other),
    };
    assert_eq!(
        resolved[0].grant_id, beta_grant.id,
        "Scenario 5: lead-beta-1's Beta org grant must win"
    );
}

/// Scenario 6 — `lead-gamma-1` reads the same joint-research session.
///
/// Reader is a Gamma member only — Gamma is NOT in the session's tag
/// set. `reader_org_matches = {}` → both tiers fall through to the
/// intersection fallback. The reader's Gamma grant is not covered by any
/// session-org ceiling → `IntersectionEmpty` `Denied`.
#[test]
fn scenario_6_lead_gamma_reads_joint_research_intersection_denied() {
    let acme = OrgId::new();
    let beta = OrgId::new();
    let gamma = OrgId::new();
    let mut ctx = AcceptanceCtx::new();
    // Reader's only org grant is at Gamma — no acme/beta ceiling
    // available in `org_grants` → intersection fallback fires empty.
    ctx.org_grants.push(grant(
        PrincipalRef::Organization(gamma),
        &[Action::Read],
        "filesystem_object",
    ));
    ctx.session_org_tags = vec![acme, beta];

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    match d {
        Decision::Denied {
            failed_step: FailedStep::Scope,
            reason:
                DeniedReason::IntersectionEmpty {
                    session_scope_count,
                    ..
                },
        } => {
            assert_eq!(
                session_scope_count, 2,
                "Scenario 6: session_scope_count counts both org tags"
            );
        }
        other => panic!(
            "Scenario 6 expected Denied(IntersectionEmpty), got {:?}",
            other
        ),
    }
}

// ----------------------------------------------------------------------------
// Concept-08 §"Step 7: Contractor scenario" (lines 287–298) — Scenario 7.
// ----------------------------------------------------------------------------

/// Scenario 7 — `contractor-x-9` (base_org = Gamma) reads
/// `acme-website-redesign` session. The session is tagged
/// `[project:acme-website-redesign, org:acme]` (Shape A from the
/// session's perspective). The contractor IS a member of the project (a
/// project grant is held); the project-tier resolution succeeds and the
/// contractor's Gamma base_org ceiling is structurally excluded by the
/// `step_2a_ceiling` membership bound (concept-doc 06 line 162).
///
/// The contractor's Gamma ceiling carries a strict action set
/// (`Connect` only) that would NOT admit a `Read` candidate; without
/// the membership bound, the candidate would be filtered to empty. With
/// the bound, Gamma is excluded from the applicable-ceilings list →
/// candidate survives → cascade returns the project-tier grant. Result:
/// `Allowed`.
#[test]
fn scenario_7_contractor_reads_acme_session_project_tier_wins() {
    let acme = OrgId::new();
    let gamma = OrgId::new();
    let acme_project = ProjectId::new();
    let mut ctx = AcceptanceCtx::new();

    // Contractor's project-tier grant (issued under the
    // `acme-website-redesign` contract).
    let project_grant = grant(
        PrincipalRef::Project(acme_project),
        &[Action::Read],
        "filesystem_object",
    );
    ctx.project_grants.push(project_grant.clone());

    // Contractor's home-org Gamma ceiling — NOT a session-tagged org.
    // Carries `Connect` so it would clamp out a `Read` candidate if the
    // membership bound did not exclude it.
    ctx.ceiling_grants.push(grant(
        PrincipalRef::Organization(gamma),
        &[Action::Connect],
        "filesystem_object",
    ));

    // Session tags: project:acme-website-redesign + org:acme.
    ctx.session_project_tags = vec![acme_project];
    ctx.session_org_tags = vec![acme];

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    let resolved = match d {
        Decision::Allowed { resolved_grants } => resolved_grants,
        other => panic!(
            "Scenario 7 expected Allowed (project-tier picks), got {:?}",
            other
        ),
    };
    assert_eq!(
        resolved[0].grant_id, project_grant.id,
        "Scenario 7: contractor's project-tier grant must win; Gamma base_org ceiling is excluded by the membership bound"
    );
}

// ----------------------------------------------------------------------------
// Regression-protection: empty session-tag slices preserve M1 behaviour.
// ----------------------------------------------------------------------------

/// Shape A baseline — single-project single-org session, reader is the
/// project's owner. Both `session_org_tags` and `session_project_tags`
/// are empty (the legacy / pre-CH-07 single-scope path) → cascade
/// reduces to today's tier-sort → project-tier wins. This regression
/// test pins the back-compat invariant the plan relies on (Artifact D
/// callsite-cascade defensive default — empty slices = M1 behaviour).
#[test]
fn shape_a_baseline_empty_session_tags_project_tier_wins() {
    let org = OrgId::new();
    let proj = ProjectId::new();
    let mut ctx = AcceptanceCtx::new();
    ctx.current_org = Some(org);
    ctx.current_project = Some(proj);
    let project_grant = grant(
        PrincipalRef::Project(proj),
        &[Action::Read],
        "filesystem_object",
    );
    let org_grant = grant(
        PrincipalRef::Organization(org),
        &[Action::Read],
        "filesystem_object",
    );
    ctx.project_grants.push(project_grant.clone());
    ctx.org_grants.push(org_grant);
    // Empty session-tag slices = single-scope back-compat path.

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    let resolved = match d {
        Decision::Allowed { resolved_grants } => resolved_grants,
        other => panic!(
            "Shape A baseline expected Allowed, got {:?} — empty session-tag slices must preserve single-scope behaviour",
            other
        ),
    };
    assert_eq!(
        resolved[0].grant_id, project_grant.id,
        "Shape A baseline: project-tier must beat org-tier in single-scope path (M1 invariant)"
    );
}

/// Shape D system session — 0 projects, 1 org. The session is owned by
/// a single org (no project tags), and a system-tier reader holds an
/// org-tier grant. Cascade: project tier is empty (no candidates AND no
/// session_project_tags) → fall through to org tier; org tier picks the
/// single matching grant. Result: `Allowed`.
#[test]
fn shape_d_system_session_org_tier_wins() {
    let system_org = OrgId::new();
    let mut ctx = AcceptanceCtx::new();
    let org_grant = grant(
        PrincipalRef::Organization(system_org),
        &[Action::Read],
        "filesystem_object",
    );
    ctx.org_grants.push(org_grant.clone());
    // Shape D: 1 org tag, 0 project tags.
    ctx.session_org_tags = vec![system_org];

    let d = check(
        &ctx.borrow(ToolCall::default()),
        &read_filesystem_manifest(),
        &NoopMetrics,
    );
    let resolved = match d {
        Decision::Allowed { resolved_grants } => resolved_grants,
        other => panic!("Shape D expected Allowed, got {:?}", other),
    };
    assert_eq!(
        resolved[0].grant_id, org_grant.id,
        "Shape D: org-tier must pick the single session-tagged-org grant"
    );
}
