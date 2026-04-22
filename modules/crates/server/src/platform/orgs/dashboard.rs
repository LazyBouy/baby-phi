#![allow(clippy::result_large_err)]

//! Org-dashboard aggregate-read orchestrator — the M3/P5 business
//! logic behind `GET /api/v0/orgs/:id/dashboard`.
//!
//! ## phi-core leverage (per-deliverable, pre-audit pinned P5.0)
//!
//! Q1 / Q2 / Q3 walk — the dashboard surface is **deliberately
//! phi-core-light** per the leverage checklist §5:
//!
//! - **Q1 — Direct imports**: none. Zero `use phi_core::…` lines in
//!   this file. The dashboard aggregates counters + governance-plane
//!   reads only.
//! - **Q2 — Transitive payload**: the [`DashboardSummary`] wire shape
//!   **strips** `Organization.defaults_snapshot` (which wraps 4
//!   phi-core types — `ExecutionLimits` / `ContextConfig` /
//!   `RetryConfig` / `AgentProfile`). The header uses
//!   [`OrganizationDashboardHeader`] which carries only the
//!   operator-facing fields. Rationale: the dashboard is a polling
//!   surface (30 s cadence per D4) — coupling its JSON shape to
//!   phi-core schema evolution would force a polling-contract rev on
//!   every phi-core release. Drill-down to the full snapshot is
//!   available via `GET /orgs/:id` (the `show.rs` endpoint) which
//!   does carry it verbatim, per D11/Q6.
//! - **Q3 — Candidates rejected** (explicit module-walk per checklist
//!   §2): `phi_core::Session`/`LoopRecord`/`Turn` — deferred to M5+
//!   (D11: dashboard rows at M3 link to the audit log, not session
//!   traces). `phi_core::Usage` — the token budget is a
//!   governance-level economic resource, not per-loop usage.
//!   `phi_core::AgentEvent` — orthogonal surface per `baby-phi/CLAUDE.md`
//!   (governance audit log ≠ agent-loop telemetry stream).
//!   `phi_core::context::*` + `phi_core::provider::*` — live only on
//!   `defaults_snapshot`, which the dashboard deliberately strips.
//!
//! Positive-close assertions enforced in P5.5 acceptance tests:
//! - `grep -n 'use phi_core' modules/crates/server/src/platform/orgs/dashboard.rs`
//!   returns 0 lines (verified by `check-phi-core-reuse.sh` during CI).
//! - A schema-snapshot test asserts the serialised `DashboardSummary`
//!   JSON has no `defaults_snapshot` / `execution_limits` /
//!   `context_config` / `retry_config` / `blueprint` keys at any
//!   depth.
//!
//! ## Read topology
//!
//! Four repo reads + two counters per request:
//!
//! 1. `get_organization(org)` — header fields.
//! 2. `list_agents_in_org(org)` — `agents_summary` + viewer role.
//! 3. `list_projects_in_org(org)` — `projects_summary` (count only
//!    at M3; shape breakdown materialises with M4's Project struct).
//! 4. `list_active_auth_requests_for_org(org)` — filtered in-process
//!    to count only those where the caller holds an unfilled approver
//!    slot (R-ADMIN-07-R4).
//! 5. `list_recent_audit_events_for_org(org, 5)` — `recent_events`.
//! 6. `count_alerted_events_for_org_since(org, now - 24h)` —
//!    `alerted_events_24h` (R-ADMIN-07-R5).
//! 7. `get_token_budget_pool_for_org(org)` — `token_budget`.
//! 8. `list_adoption_auth_requests_for_org(org)` — `templates_adopted`.
//!
//! The reads are mostly sequential (the repo trait isn't batchable at
//! M3), which is acceptable for a 30 s polling cadence on tens of
//! orgs. M4+ may coalesce into a single repo method if metrics show
//! the fan-out is a hot path.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use domain::audit::AuditEvent;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, AuthRequest, AuthRequestState, Organization, PrincipalRef, TemplateKind,
};
use domain::repository::Repository;

use super::OrgError;

// ---------------------------------------------------------------------------
// Wire shapes — deliberately phi-core-stripped.
// ---------------------------------------------------------------------------

/// Organization header used by the dashboard — drops
/// `defaults_snapshot` and every other phi-core-wrapping field. The
/// full snapshot is available via `GET /orgs/:id` (show.rs).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDashboardHeader {
    pub id: OrgId,
    pub display_name: String,
    pub vision: Option<String>,
    pub mission: Option<String>,
    pub consent_policy: ConsentPolicy,
}

/// Viewer-role decision. Derived from the caller's membership in the
/// org + whether they hold `allocate` on `org:<id>` (the CEO signal).
///
/// `ProjectLead` is defined but **not populated** at M3 — project
/// surface + `HAS_LEAD` edge wiring land in M4 per the plan's Q2
/// open question. Until then the dashboard returns `Admin` for CEO
/// viewers and `Member` for ordinary org members.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewerRole {
    /// Holds `allocate` on `org:<id>` — typically the CEO. Sees every
    /// tile including pending-AR count and alerted-events count.
    Admin,
    /// Lead of at least one Project in this org — filtered view
    /// (project-scoped agent list, "View Projects" instead of
    /// "Create Project", alerted-events tile hidden). **M4**.
    ProjectLead,
    /// Has `MEMBER_OF` to the org but no admin grant. Read-only
    /// summary; no pending-AR tile, no alerted-events tile, no CTA
    /// cards.
    Member,
    /// Viewer has no relation to the org. The handler maps this to
    /// a 403 before the orchestrator would ever produce it; the
    /// variant exists so the orchestrator can signal the decision
    /// without an in-band `Option`.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewerContext {
    pub agent_id: AgentId,
    pub role: ViewerRole,
    pub can_admin_manage: bool,
}

/// Count of agents grouped by kind. M3's domain model has two
/// `AgentKind` variants (`Human`, `Llm`); the requirements doc
/// mentions a future four-way split (`Human / Intern / Contract /
/// System`) that needs an `AgentRole` field — carryover for M4/P?.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgentsSummary {
    pub total: u32,
    pub human: u32,
    /// System / LLM agents — at M3 the only `AgentKind::Llm` agents
    /// per org are the 2 system agents provisioned at creation time.
    pub llm: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProjectsSummary {
    /// Total count of active projects. M3 returns `0` until M4 wires
    /// project persistence.
    pub active: u32,
    /// Shape A count — reserved for M4 (the Project struct carries
    /// `shape: ProjectShape` then). `0` at M3.
    pub shape_a: u32,
    /// Shape B count — reserved for M4. `0` at M3.
    pub shape_b: u32,
}

/// Token-budget utilisation tile (R-ADMIN-07-R6).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenBudgetView {
    pub used: u64,
    pub total: u64,
    pub pool_id: NodeId,
}

/// A single row in the `recent_events` panel. **Deliberately compact**
/// — the full audit event (including `diff`, `prev_event_hash`) is
/// available via the audit-log detail page that each row links to at
/// M3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentEventSummary {
    pub id: AuditEventId,
    pub kind: String,
    pub actor: Option<AgentId>,
    pub timestamp: DateTime<Utc>,
    /// One-line human summary — derived from `event_type` + target
    /// entity id. Frontend renders this verbatim; no client-side
    /// formatting required.
    pub summary: String,
}

/// The 4 empty-state call-to-action cards (R-ADMIN-07-R8). Each is
/// `Some(url)` when visible to the viewer, `None` otherwise (respects
/// the viewer-role filter per §7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EmptyStateCtaCards {
    pub add_agent: Option<String>,
    pub create_project: Option<String>,
    pub adopt_template: Option<String>,
    pub configure_system_agents: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardSummary {
    pub org: OrganizationDashboardHeader,
    pub viewer: ViewerContext,
    pub agents_summary: AgentsSummary,
    pub projects_summary: ProjectsSummary,
    pub pending_auth_requests_count: u32,
    pub alerted_events_24h: u32,
    pub token_budget: TokenBudgetView,
    pub recent_events: Vec<RecentEventSummary>,
    pub templates_adopted: Vec<TemplateKind>,
    pub cta_cards: EmptyStateCtaCards,
    /// First-visit welcome banner copy (R-ADMIN-07-N3). `Some` only
    /// when the org is fresh (every counter ≈ 0); `None` otherwise.
    pub welcome_banner: Option<String>,
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Three-state outcome: the org may not exist (→ 404), the viewer may
/// have no relation to it (→ 403), or we return the payload (→ 200).
/// Typed out so the handler's branches are unambiguous and a future
/// reviewer can't accidentally conflate "missing" with "access denied".
#[derive(Debug)]
pub enum DashboardOutcome {
    Found(Box<DashboardSummary>),
    NotFound,
    AccessDenied,
}

pub async fn dashboard_summary(
    repo: Arc<dyn Repository>,
    org_id: OrgId,
    viewer_agent_id: AgentId,
    now: DateTime<Utc>,
) -> Result<DashboardOutcome, OrgError> {
    let org = match repo.get_organization(org_id).await? {
        Some(o) => o,
        None => return Ok(DashboardOutcome::NotFound),
    };

    let agents = repo.list_agents_in_org(org_id).await?;
    let viewer_role = resolve_viewer_role(&agents, viewer_agent_id);
    if matches!(viewer_role, ViewerRole::None) {
        return Ok(DashboardOutcome::AccessDenied);
    }

    let projects = repo.list_projects_in_org(org_id).await?;
    let active_ars = repo.list_active_auth_requests_for_org(org_id).await?;
    let pending_for_viewer = count_pending_with_slot_for(&active_ars, viewer_agent_id);

    let recent_events_raw = repo
        .list_recent_audit_events_for_org(org_id, RECENT_EVENT_LIMIT)
        .await?;
    let alerted_24h = repo
        .count_alerted_events_for_org_since(org_id, now - Duration::hours(24))
        .await?;

    let pool = match repo.get_token_budget_pool_for_org(org_id).await? {
        Some(p) => p,
        None => {
            return Err(OrgError::Repository(format!(
                "token_budget_pool missing for org {org_id} — org was not created via the wizard \
                 or migration state is broken"
            )));
        }
    };

    let adoption_ars = repo.list_adoption_auth_requests_for_org(org_id).await?;
    let templates_adopted = templates_from_adoption_uris(&adoption_ars, org_id);

    let agents_summary = count_agents(&agents);
    let projects_summary = ProjectsSummary {
        active: projects.len() as u32,
        ..Default::default()
    };

    let cta_cards = build_cta_cards(org_id, viewer_role, &agents_summary, &projects_summary);
    let welcome_banner = build_welcome_banner(&org, &agents_summary, &projects_summary);

    let recent_events = recent_events_raw
        .iter()
        .map(RecentEventSummary::from)
        .collect();

    Ok(DashboardOutcome::Found(Box::new(DashboardSummary {
        org: OrganizationDashboardHeader {
            id: org.id,
            display_name: org.display_name,
            vision: org.vision,
            mission: org.mission,
            consent_policy: org.consent_policy,
        },
        viewer: ViewerContext {
            agent_id: viewer_agent_id,
            role: viewer_role,
            can_admin_manage: matches!(viewer_role, ViewerRole::Admin),
        },
        agents_summary,
        projects_summary,
        pending_auth_requests_count: pending_for_viewer,
        alerted_events_24h: alerted_24h,
        token_budget: TokenBudgetView {
            used: pool.used,
            total: pool.initial_allocation,
            pool_id: pool.id,
        },
        recent_events,
        templates_adopted,
        cta_cards,
        welcome_banner,
    })))
}

const RECENT_EVENT_LIMIT: usize = 5;

// ---------------------------------------------------------------------------
// Pure helpers — unit-tested exhaustively below.
// ---------------------------------------------------------------------------

fn resolve_viewer_role(agents: &[Agent], viewer_agent_id: AgentId) -> ViewerRole {
    // Membership is derived from Agent.owning_org being the org in
    // question — the `list_agents_in_org` filter already narrowed
    // the list. Admin detection at M3 is "first Human agent in the
    // org" — the CEO created at org-creation time. A proper grant
    // walk lands once M5's permission-check wiring is dashboard-
    // aware; noting this narrowness explicitly here rather than
    // hiding it in a comment.
    let in_org = agents.iter().any(|a| a.id == viewer_agent_id);
    if !in_org {
        return ViewerRole::None;
    }
    let is_ceo = agents
        .iter()
        .filter(|a| matches!(a.kind, AgentKind::Human))
        .min_by_key(|a| a.created_at)
        .map(|a| a.id == viewer_agent_id)
        .unwrap_or(false);
    if is_ceo {
        ViewerRole::Admin
    } else {
        ViewerRole::Member
    }
}

fn count_agents(agents: &[Agent]) -> AgentsSummary {
    let mut out = AgentsSummary::default();
    for a in agents {
        out.total += 1;
        match a.kind {
            AgentKind::Human => out.human += 1,
            AgentKind::Llm => out.llm += 1,
        }
    }
    out
}

fn count_pending_with_slot_for(active: &[AuthRequest], viewer: AgentId) -> u32 {
    active
        .iter()
        .filter(|ar| {
            matches!(
                ar.state,
                AuthRequestState::Pending | AuthRequestState::InProgress
            )
        })
        .filter(|ar| {
            ar.resource_slots.iter().any(|s| {
                s.approvers.iter().any(|a| {
                    matches!(
                        &a.approver,
                        PrincipalRef::Agent(id) if *id == viewer
                    )
                })
            })
        })
        .count() as u32
}

fn templates_from_adoption_uris(adoption_ars: &[AuthRequest], org_id: OrgId) -> Vec<TemplateKind> {
    let prefix = format!("org:{org_id}/template:");
    let mut out = Vec::new();
    for ar in adoption_ars {
        for slot in &ar.resource_slots {
            if let Some(kind_str) = slot.resource.uri.strip_prefix(&prefix) {
                if let Some(kind) = parse_template_kind(kind_str) {
                    if !out.contains(&kind) {
                        out.push(kind);
                    }
                }
            }
        }
    }
    out
}

fn parse_template_kind(s: &str) -> Option<TemplateKind> {
    match s {
        "a" => Some(TemplateKind::A),
        "b" => Some(TemplateKind::B),
        "c" => Some(TemplateKind::C),
        "d" => Some(TemplateKind::D),
        _ => None,
    }
}

fn build_cta_cards(
    org_id: OrgId,
    role: ViewerRole,
    agents: &AgentsSummary,
    projects: &ProjectsSummary,
) -> EmptyStateCtaCards {
    // Cards surface only when the viewer can actually complete the
    // target page's write. Admin sees every card. Member sees no
    // cards (they can't create agents / projects / adopt templates).
    // ProjectLead is M4.
    let show = matches!(role, ViewerRole::Admin);
    if !show {
        return EmptyStateCtaCards::default();
    }
    // The cards surface on a freshly-created org (agents.human == 0
    // and projects.active == 0). Counts matching the "2 system agents
    // only" shape still count as fresh — the CEO is human=1; at that
    // point the CTAs still make sense.
    let is_fresh = projects.active == 0
        || agents.total <= 3 /* CEO + 2 system */;
    if !is_fresh {
        return EmptyStateCtaCards::default();
    }
    EmptyStateCtaCards {
        add_agent: Some("/organizations/".to_string() + &org_id.to_string() + "/agents/new"),
        create_project: Some("/organizations/".to_string() + &org_id.to_string() + "/projects/new"),
        adopt_template: Some("/organizations/".to_string() + &org_id.to_string() + "/templates"),
        configure_system_agents: Some(
            "/organizations/".to_string() + &org_id.to_string() + "/system-agents",
        ),
    }
}

fn build_welcome_banner(
    org: &Organization,
    agents: &AgentsSummary,
    projects: &ProjectsSummary,
) -> Option<String> {
    // First-visit-fresh banner (R-ADMIN-07-N3). The banner is server-
    // computed rather than client-driven so the copy is identical
    // across CLI + Web (both consume the same field).
    if projects.active == 0 && agents.total <= 3 {
        Some(format!(
            "Welcome to {}. Start with Phase 5 to build your agent roster.",
            org.display_name
        ))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// AuditEvent → RecentEventSummary projection
// ---------------------------------------------------------------------------

impl From<&AuditEvent> for RecentEventSummary {
    fn from(e: &AuditEvent) -> Self {
        RecentEventSummary {
            id: e.event_id,
            kind: e.event_type.clone(),
            actor: e.actor_agent_id,
            timestamp: e.timestamp,
            summary: format_summary(e),
        }
    }
}

fn format_summary(e: &AuditEvent) -> String {
    let target = e
        .target_entity_id
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "-".into());
    format!("{}: {}", e.event_type, target)
}

// ---------------------------------------------------------------------------
// Tests — pure helpers + phi-core invariant (schema snapshot).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use domain::model::ids::AgentId;

    fn agent(kind: AgentKind, created_at: DateTime<Utc>) -> Agent {
        Agent {
            id: AgentId::new(),
            kind,
            display_name: "test".into(),
            owning_org: Some(OrgId::new()),
            created_at,
        }
    }

    #[test]
    fn resolve_viewer_role_non_member_is_none() {
        let agents = vec![agent(AgentKind::Human, Utc::now())];
        let outsider = AgentId::new();
        assert_eq!(resolve_viewer_role(&agents, outsider), ViewerRole::None);
    }

    #[test]
    fn resolve_viewer_role_first_human_is_admin() {
        let t0 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let t1 = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
        let ceo = agent(AgentKind::Human, t0);
        let later_human = agent(AgentKind::Human, t1);
        let sys = agent(AgentKind::Llm, t0);
        let agents = vec![ceo.clone(), later_human.clone(), sys];
        assert_eq!(resolve_viewer_role(&agents, ceo.id), ViewerRole::Admin);
        assert_eq!(
            resolve_viewer_role(&agents, later_human.id),
            ViewerRole::Member
        );
    }

    #[test]
    fn count_agents_splits_human_and_llm() {
        let agents = vec![
            agent(AgentKind::Human, Utc::now()),
            agent(AgentKind::Llm, Utc::now()),
            agent(AgentKind::Llm, Utc::now()),
        ];
        let s = count_agents(&agents);
        assert_eq!(s.total, 3);
        assert_eq!(s.human, 1);
        assert_eq!(s.llm, 2);
    }

    #[test]
    fn welcome_banner_fresh_org_shows_banner() {
        let org = org_with_name("Acme");
        let agents = AgentsSummary {
            total: 3,
            human: 1,
            llm: 2,
        };
        let projects = ProjectsSummary::default();
        let banner = build_welcome_banner(&org, &agents, &projects);
        assert!(banner.is_some());
        assert!(banner.unwrap().contains("Acme"));
    }

    #[test]
    fn welcome_banner_populated_org_hides_banner() {
        let org = org_with_name("Acme");
        let agents = AgentsSummary {
            total: 10,
            human: 3,
            llm: 2,
        };
        let projects = ProjectsSummary {
            active: 2,
            ..Default::default()
        };
        assert!(build_welcome_banner(&org, &agents, &projects).is_none());
    }

    #[test]
    fn cta_cards_hidden_for_members() {
        let cards = build_cta_cards(
            OrgId::new(),
            ViewerRole::Member,
            &AgentsSummary::default(),
            &ProjectsSummary::default(),
        );
        assert!(cards.add_agent.is_none());
        assert!(cards.create_project.is_none());
    }

    #[test]
    fn cta_cards_shown_for_admin_on_fresh_org() {
        let org_id = OrgId::new();
        let agents = AgentsSummary {
            total: 3,
            human: 1,
            llm: 2,
        };
        let projects = ProjectsSummary::default();
        let cards = build_cta_cards(org_id, ViewerRole::Admin, &agents, &projects);
        assert!(cards.add_agent.is_some());
        assert!(cards.create_project.is_some());
        assert!(cards.adopt_template.is_some());
        assert!(cards.configure_system_agents.is_some());
    }

    /// **Positive phi-core invariant — schema-snapshot test.**
    ///
    /// The dashboard wire shape MUST NOT leak phi-core-wrapping
    /// fields (`defaults_snapshot`, `execution_limits`,
    /// `context_config`, `retry_config`, `blueprint`). If a future
    /// refactor swaps in `Organization` for `OrganizationDashboardHeader`
    /// (or otherwise re-adds the snapshot), this test breaks
    /// immediately and forces a reviewer to re-read the pre-audit.
    #[test]
    fn dashboard_summary_wire_shape_excludes_phi_core_fields() {
        let sample = DashboardSummary {
            org: OrganizationDashboardHeader {
                id: OrgId::new(),
                display_name: "Acme".into(),
                vision: None,
                mission: None,
                consent_policy: ConsentPolicy::Implicit,
            },
            viewer: ViewerContext {
                agent_id: AgentId::new(),
                role: ViewerRole::Admin,
                can_admin_manage: true,
            },
            agents_summary: AgentsSummary::default(),
            projects_summary: ProjectsSummary::default(),
            pending_auth_requests_count: 0,
            alerted_events_24h: 0,
            token_budget: TokenBudgetView {
                used: 0,
                total: 1,
                pool_id: NodeId::new(),
            },
            recent_events: vec![],
            templates_adopted: vec![],
            cta_cards: EmptyStateCtaCards::default(),
            welcome_banner: None,
        };
        let json = serde_json::to_string(&sample).expect("serialize");
        for forbidden in &[
            "defaults_snapshot",
            "execution_limits",
            "context_config",
            "retry_config",
            "default_agent_profile",
            "blueprint",
        ] {
            assert!(
                !json.contains(forbidden),
                "DashboardSummary wire JSON must not contain `{forbidden}` — that would \
                 reintroduce phi-core-wrapping transit (see P5.0 pre-audit)"
            );
        }
    }

    fn org_with_name(name: &str) -> Organization {
        Organization {
            id: OrgId::new(),
            display_name: name.into(),
            vision: None,
            mission: None,
            consent_policy: ConsentPolicy::Implicit,
            audit_class_default: domain::audit::AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        }
    }
}
