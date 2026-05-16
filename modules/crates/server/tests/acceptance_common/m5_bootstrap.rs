//! Shared bootstrap helpers for the 5 per-page CH-24 milestone-rollup
//! acceptance suites (`acceptance_m5_{orgs,projects,agents,sessions,memory}.rs`).
//!
//! Why this module exists (CH-24 / F1.B duplication-risk mitigation per
//! plan §7 P-NEW-TESTS): the 5 per-page split risks setup-code
//! duplication across the files (each file needs its own bootstrap →
//! claim CEO → create org preamble). This helper centralises the four
//! shapes the per-page files reach for so the bootstrap preamble is
//! defined exactly once.
//!
//! Naming mirrors the existing `acceptance_common::admin::spawn_*`
//! family. Each variant returns a different fixture struct so the
//! per-page test gets exactly what it needs (e.g. the orgs file does
//! not need an agent created; the sessions file needs a project +
//! template adopted + agent + model runtime + grants).
//!
//! ## Variant map
//!
//! - [`bootstrap_with_two_orgs`] — boots a claimed admin + creates **two**
//!   Shape A orgs via the real `POST /api/v0/orgs` wizard endpoint. Used
//!   by `acceptance_m5_orgs.rs` for the page-1 list visibility + cross-org
//!   isolation scenarios.
//! - [`bootstrap_with_two_orgs_and_projects`] — extends the two-orgs
//!   shape with a project under Org-A. Used by `acceptance_m5_projects.rs`
//!   for the project isolation scenario.
//! - [`bootstrap_with_org_and_two_agents`] — boots one org + seeds a
//!   CEO + LLM intern agent pair. Used by `acceptance_m5_agents.rs`.
//! - [`bootstrap_to_session_endable`] — boots one org + project + lead
//!   agent + model runtime + Template A grants + memory-extraction
//!   wiring. Returns the project bundle ready for a `POST /sessions`
//!   call that will succeed end-to-end (CH-15 hard-deny gate + C-M5-3
//!   loop-record persistence + CH-17 SSE Observe gate). Used by
//!   `acceptance_m5_sessions.rs` + `acceptance_m5_memory.rs`.
//!
//! ## phi-core leverage (CH-24 §3 forbidden-duplication grep)
//!
//! This module adds zero direct phi-core import lines (no `use
//! phi_core` statement here). The existing per-feature fixtures
//! (`spawn_claimed_with_org`, `spawn_claimed_with_org_and_project`,
//! `seed_template_a_grants_for_lead`) already wrap whatever phi-core
//! surface they need; this file delegates to them. The two inline
//! references to `phi_core::provider::model::ModelConfig` and
//! `phi_core::agents::profile::AgentProfile` below are path-qualified
//! one-shot usages (not new top-of-file imports) mirroring the
//! pattern already in
//! `acceptance_common/admin.rs::spawn_claimed_with_org`.

#![allow(dead_code)]

use std::sync::Arc;

use chrono::Utc;
use domain::model::composites_m2::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, ModelProviderId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, Identity, InboxObject, OutboxObject, Project,
    ProjectShape, ProjectStatus,
};
use domain::repository::{AgentCreationPayload, ProjectCreationPayload, Repository};

use super::admin::{
    spawn_claimed_with_org, spawn_claimed_with_org_and_project, ClaimedAdmin, ClaimedOrg,
    ClaimedProject,
};

// ---------------------------------------------------------------------------
// `bootstrap_with_two_orgs` — page-1/page-2 (orgs file)
// ---------------------------------------------------------------------------

/// Two fully-seeded orgs (each with their own CEO + 2 system agents)
/// sharing a single claimed-admin session. Each org is created via the
/// real `POST /api/v0/orgs` wizard endpoint so the per-page test
/// exercises the same HTTP path operators do.
pub struct TwoOrgs {
    pub admin: ClaimedAdmin,
    pub org_a_id: OrgId,
    pub org_b_id: OrgId,
    pub org_a_ceo: AgentId,
    pub org_b_ceo: AgentId,
}

/// Create two orgs via the real HTTP wizard. Each org carries its own
/// CEO (distinct uuid) + 2 system agents per the minimal-startup shape.
pub async fn bootstrap_with_two_orgs() -> TwoOrgs {
    use super::admin::spawn_claimed;

    let admin = spawn_claimed(false).await;

    let org_a =
        create_org_via_http(&admin, "Atlas Research", "Atlas vision", "ceo-a@atlas.test").await;
    let org_b = create_org_via_http(&admin, "Beta Labs", "Beta vision", "ceo-b@beta.test").await;

    TwoOrgs {
        admin,
        org_a_id: org_a.org_id,
        org_b_id: org_b.org_id,
        org_a_ceo: org_a.ceo_agent_id,
        org_b_ceo: org_b.ceo_agent_id,
    }
}

struct CreatedOrg {
    org_id: OrgId,
    ceo_agent_id: AgentId,
}

async fn create_org_via_http(
    admin: &ClaimedAdmin,
    display_name: &str,
    vision: &str,
    ceo_channel_handle: &str,
) -> CreatedOrg {
    let body = serde_json::json!({
        "display_name": display_name,
        "vision": vision,
        "mission": "M5 milestone-rollup test fixture",
        "consent_policy": "implicit",
        "audit_class_default": "logged",
        "authority_templates_enabled": ["a"],
        "default_model_provider": null,
        "ceo_display_name": format!("{} CEO", display_name),
        "ceo_channel_kind": "email",
        "ceo_channel_handle": ceo_channel_handle,
        "token_budget": 1_000_000_u64,
    });
    let res = admin
        .authed_client
        .post(admin.url("/api/v0/orgs"))
        .json(&body)
        .send()
        .await
        .expect("POST /api/v0/orgs");
    assert_eq!(
        res.status().as_u16(),
        201,
        "POST /api/v0/orgs must return 201; body={:?}",
        res.text().await
    );
    let receipt: serde_json::Value = res.json().await.expect("decode org-creation receipt JSON");
    let org_id = OrgId::from_uuid(
        uuid::Uuid::parse_str(receipt["org_id"].as_str().expect("org_id string"))
            .expect("org_id is uuid"),
    );
    let ceo_agent_id = AgentId::from_uuid(
        uuid::Uuid::parse_str(
            receipt["ceo_agent_id"]
                .as_str()
                .expect("ceo_agent_id string"),
        )
        .expect("ceo_agent_id is uuid"),
    );
    CreatedOrg {
        org_id,
        ceo_agent_id,
    }
}

// ---------------------------------------------------------------------------
// `bootstrap_with_two_orgs_and_projects` — page-3/page-4 (projects file)
// ---------------------------------------------------------------------------

/// Two orgs + one Shape A project materialised under Org-A. The
/// project-creation path here uses the repository's
/// `apply_project_creation` directly (matching the existing
/// `spawn_claimed_with_org_and_project` precedent) — keeps the surface
/// the test exercises at the project-detail / cross-org-access layer.
pub struct TwoOrgsWithProjectA {
    pub admin: ClaimedAdmin,
    pub org_a_id: OrgId,
    pub org_b_id: OrgId,
    pub org_a_ceo: AgentId,
    pub org_b_ceo: AgentId,
    pub project_a_id: ProjectId,
    pub project_a_lead: AgentId,
}

pub async fn bootstrap_with_two_orgs_and_projects() -> TwoOrgsWithProjectA {
    let two = bootstrap_with_two_orgs().await;
    let repo: Arc<dyn Repository> = two.admin.acc.store.clone();

    // Seed a lead agent under Org-A for the project.
    let now = Utc::now();
    let lead = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "atlas-lead".into(),
        owning_org: Some(two.org_a_id),
        role: Some(AgentRole::Intern),
        created_at: now,
        active: true,
        archived_at: None,
    };
    let lead_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: lead.id,
        created_at: now,
        tags: Vec::new(),
    };
    let lead_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: lead.id,
        created_at: now,
        tags: Vec::new(),
    };
    let lead_id = lead.id;
    repo.apply_agent_creation(&AgentCreationPayload {
        agent: lead,
        inbox: lead_inbox,
        outbox: lead_outbox,
        profile: None,
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity: Some(Identity::default_for_llm(lead_id, now)),
    })
    .await
    .expect("seed Org-A project lead");

    let project_id = ProjectId::new();
    let project = Project {
        id: project_id,
        name: "Atlas Memory Benchmark".into(),
        description: "Page-3 fixture project".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape: ProjectShape::A,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        created_at: now,
    };
    repo.apply_project_creation(&ProjectCreationPayload {
        project,
        owning_orgs: vec![two.org_a_id],
        lead_agent_id: lead_id,
        // CH-25 / ADR-0060 §D60.1 — Decision-3: lead = creator.
        creator_agent: lead_id,
        member_agent_ids: vec![],
        sponsor_agent_ids: vec![two.org_a_ceo],
        catalogue_entries: vec![(format!("project:{}", project_id), "project".into())],
    })
    .await
    .expect("materialise Project A");

    TwoOrgsWithProjectA {
        admin: two.admin,
        org_a_id: two.org_a_id,
        org_b_id: two.org_b_id,
        org_a_ceo: two.org_a_ceo,
        org_b_ceo: two.org_b_ceo,
        project_a_id: project_id,
        project_a_lead: lead_id,
    }
}

// ---------------------------------------------------------------------------
// `bootstrap_with_org_and_two_agents` — page-5/page-6 (agents file)
// ---------------------------------------------------------------------------

/// One org plus a freshly-seeded CEO-like Human agent and LLM intern
/// agent (in addition to the fixture's 3 baseline agents). Both agents
/// are created via the repo's `apply_agent_creation`.
pub struct OrgWithTwoAgents {
    pub claimed_org: ClaimedOrg,
    /// The extra Human agent seeded by this helper (distinct from the
    /// fixture's CEO).
    pub extra_ceo: AgentId,
    /// The LLM intern agent seeded by this helper.
    pub intern: AgentId,
}

pub async fn bootstrap_with_org_and_two_agents() -> OrgWithTwoAgents {
    let claimed_org = spawn_claimed_with_org(false).await;
    let repo: Arc<dyn Repository> = claimed_org.admin.acc.store.clone();
    let now = Utc::now();

    let extra_ceo = create_agent(
        &repo,
        claimed_org.org_id,
        AgentKind::Human,
        AgentRole::Member,
        "atlas-cfo",
        now,
    )
    .await;
    let intern = create_agent(
        &repo,
        claimed_org.org_id,
        AgentKind::Llm,
        AgentRole::Intern,
        "atlas-intern",
        now,
    )
    .await;

    OrgWithTwoAgents {
        claimed_org,
        extra_ceo,
        intern,
    }
}

async fn create_agent(
    repo: &Arc<dyn Repository>,
    org_id: OrgId,
    kind: AgentKind,
    role: AgentRole,
    name: &str,
    now: chrono::DateTime<Utc>,
) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind,
        display_name: name.into(),
        owning_org: Some(org_id),
        role: Some(role),
        created_at: now,
        active: true,
        archived_at: None,
    };
    let inbox = InboxObject {
        id: NodeId::new(),
        agent_id: agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let aid = agent.id;
    let identity = if matches!(kind, AgentKind::Llm) {
        Some(Identity::default_for_llm(aid, now))
    } else {
        None
    };
    repo.apply_agent_creation(&AgentCreationPayload {
        agent,
        inbox,
        outbox,
        profile: None,
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity,
    })
    .await
    .expect("create_agent helper");
    aid
}

// ---------------------------------------------------------------------------
// `bootstrap_to_session_endable` — page-11/page-14 (sessions + memory files)
// ---------------------------------------------------------------------------

/// One org + Shape A project + lead agent (LLM Intern) + bound model
/// runtime + Template A grants + memory-extraction system agent already
/// configured by the fixture. Calling `POST
/// /api/v0/orgs/:org/projects/:project/sessions` on this bundle with a
/// reqwest client authenticated as the CEO will exercise the full
/// CH-02 + CH-15 + CH-17 + CH-21 path and finalise in milliseconds via
/// MockProvider.
pub struct SessionEndable {
    pub project: ClaimedProject,
    pub model_runtime_id: ModelProviderId,
}

pub async fn bootstrap_to_session_endable() -> SessionEndable {
    use super::admin::seed_template_a_grants_for_lead;

    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let now = Utc::now();

    let model_runtime_id = ModelProviderId::new();
    let runtime = ModelRuntime {
        id: model_runtime_id,
        config: phi_core::provider::model::ModelConfig::anthropic(
            "test-anthropic",
            "claude-test",
            "",
        ),
        secret_ref: SecretRef::new("anthropic-api-key"),
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: now,
    };
    repo.put_model_provider(&runtime)
        .await
        .expect("seed model runtime");

    // Bind the model runtime to the project lead's AgentProfile.
    let lead_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: project.project_lead,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(model_runtime_id.to_string()),
        mock_response: None,
        created_at: now,
    };
    repo.create_agent_profile(&lead_profile)
        .await
        .expect("create lead profile");

    // CH-15 / ADR-0054 — seed Template A grants on the lead so the
    // hard-deny launch gate passes at Steps 1–5.
    seed_template_a_grants_for_lead(&project).await;

    SessionEndable {
        project,
        model_runtime_id,
    }
}

// ---------------------------------------------------------------------------
// Misc reusable helpers
// ---------------------------------------------------------------------------

/// Build a reqwest client authenticated as `viewer` against the
/// acceptance harness. Re-exposes the lower-level
/// `acceptance_common::admin::authed_client_for` so per-page files don't
/// need to import the inner module.
pub fn client_as(admin: &ClaimedAdmin, viewer: AgentId) -> reqwest::Client {
    super::admin::authed_client_for(admin, viewer).expect("mint viewer session cookie")
}
