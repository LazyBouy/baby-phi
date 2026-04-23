//! `spawn_claimed()` — end-to-end harness fixture that stands up the
//! acceptance server AND drives the bootstrap-claim flow, then hands
//! the caller a [`ClaimedAdmin`] bundle with a pre-cookied reqwest
//! client ready to call M2+ endpoints.
//!
//! Every M2/P4+ page acceptance test starts here: secret ops, model
//! provider registration, MCP server patches, platform-defaults
//! writes. Centralising the claim dance avoids copy/pasting 30 lines
//! of set-up into each test file.

// Cargo compiles each file under tests/ as a separate test binary and
// flags code not reached from that specific binary as dead. The
// functions here are exercised by acceptance_bootstrap.rs's sibling
// tests (starting at M2/P4); suppress the lint during P3 while the
// first real consumer is still being built.
#![allow(dead_code)]

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::time::Duration;

use super::{claim_body, mint_credential, spawn, Acceptance};

/// Everything a page-test needs after a successful bootstrap claim:
/// the acceptance server, the admin's `agent_id`, the signed-cookie
/// value, and a reqwest client that automatically sends the cookie
/// on every request.
pub struct ClaimedAdmin {
    pub acc: Acceptance,
    /// The human admin's agent_id (UUID string).
    pub agent_id: String,
    /// The raw `phi_kernel_session` cookie value (the JWT). Useful for
    /// tests that want to sign a forged cookie variant.
    pub session_cookie: String,
    /// reqwest client preconfigured with `Cookie: phi_kernel_session=<jwt>`
    /// on every request.
    pub authed_client: reqwest::Client,
}

impl ClaimedAdmin {
    /// Convenience — absolute URL for a path on the acceptance server.
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.acc.base_url, path)
    }
}

/// Mint a reqwest client that authenticates as `subject_agent_id`.
///
/// Signs a fresh JWT with the same key the acceptance server uses and
/// pre-installs the `phi_kernel_session` cookie as a default header.
/// Used by page-test scenarios that need a viewer distinct from the
/// bootstrap admin (e.g. the page-11 403 access-gate test).
pub fn authed_client_for(
    _admin: &ClaimedAdmin,
    subject_agent_id: domain::model::ids::AgentId,
) -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    use server::SessionKey;
    let key = SessionKey::for_tests(super::TEST_SESSION_SECRET);
    let (token, _cookie) =
        server::session::sign_and_build_cookie(&key, &subject_agent_id.to_string())?;
    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("phi_kernel_session={token}"))?,
    );
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(default_headers)
        .build()?)
}

/// Boot a fresh acceptance server, mint a bootstrap credential, POST
/// `/api/v0/bootstrap/claim` with sensible defaults, and capture the
/// resulting session cookie into a preconfigured reqwest client.
///
/// `with_metrics` controls whether the Prometheus layer + `/metrics`
/// route are installed — only one caller per process may pass `true`
/// (see the `OnceLock` note in `acceptance_common::spawn`).
pub async fn spawn_claimed(with_metrics: bool) -> ClaimedAdmin {
    let acc = spawn(with_metrics).await;
    let credential = mint_credential(&acc).await;

    // Use the non-cookied client to run the claim — the response
    // carries the `Set-Cookie` header we capture below.
    let bootstrap_client = acc.client();
    let res = bootstrap_client
        .post(format!("{}/api/v0/bootstrap/claim", acc.base_url))
        .json(&claim_body(
            &credential,
            "Acceptance Admin",
            "web",
            "https://example.com/admin",
        ))
        .send()
        .await
        .expect("post claim");
    assert_eq!(
        res.status().as_u16(),
        201,
        "claim must return 201; body was {:?}",
        res.text().await
    );

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("claim response must set session cookie")
        .to_str()
        .expect("set-cookie value is ASCII")
        .to_string();
    let session_cookie = set_cookie
        .split("phi_kernel_session=")
        .nth(1)
        .expect("phi_kernel_session= prefix present")
        .split(';')
        .next()
        .expect("cookie value present")
        .to_string();

    let body: serde_json::Value = res.json().await.expect("decode claim body");
    let agent_id = body["human_agent_id"]
        .as_str()
        .expect("human_agent_id in claim response")
        .to_string();

    // Pre-cookied client — every request automatically carries the
    // session. reqwest doesn't expose a "set a default cookie" knob,
    // so we install a default `Cookie` header.
    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("phi_kernel_session={session_cookie}"))
            .expect("cookie value is a valid header"),
    );
    let authed_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(default_headers)
        .build()
        .expect("build authed reqwest client");

    ClaimedAdmin {
        acc,
        agent_id,
        session_cookie,
        authed_client,
    }
}

// ============================================================================
// M3/P3 — `spawn_claimed_with_org` fixture
// ============================================================================

/// Every page-test at M3/P4+ that needs a populated org starts here:
/// the acceptance server + the platform-admin-claimed environment +
/// one minimal-startup-shaped org (CEO, 2 system agents, 1 adopted
/// template, 1 token budget pool).
pub struct ClaimedOrg {
    pub admin: ClaimedAdmin,
    pub org_id: domain::model::ids::OrgId,
    pub ceo_agent_id: domain::model::ids::AgentId,
    pub system_agents: [domain::model::ids::AgentId; 2],
}

impl ClaimedOrg {
    pub fn url(&self, path: &str) -> String {
        self.admin.url(path)
    }
}

/// Boot a claimed environment + create a minimal-startup-shaped
/// organization via the compound `apply_org_creation` transaction.
///
/// **M3/P3 stub-body note**: the plan commits to driving the org
/// through the real `POST /api/v0/orgs` wizard flow at P4. The P3
/// fixture instead reaches into the harness's `Arc<SurrealStore>` and
/// calls [`domain::Repository::apply_org_creation`] directly — this
/// lets downstream P3 tests (e.g. the two-orgs audit-chain proptest)
/// use a real, populated org without waiting for the HTTP endpoint.
/// P4 replaces the body with the wizard submission without breaking
/// the public signature.
pub async fn spawn_claimed_with_org(with_metrics: bool) -> ClaimedOrg {
    use chrono::Utc;
    use domain::audit::AuditClass;
    use domain::model::composites_m3::{ConsentPolicy, TokenBudgetPool};
    use domain::model::ids::{AgentId, GrantId, NodeId, OrgId};
    use domain::model::nodes::{
        Agent, AgentKind, AgentProfile, Channel, ChannelKind, Grant, InboxObject, Organization,
        OutboxObject, PrincipalRef, ResourceRef, TemplateKind,
    };
    use domain::model::Fundamental;
    use domain::repository::{OrgCreationPayload, Repository};
    use domain::templates::a;

    let admin = spawn_claimed(with_metrics).await;

    let org_id = OrgId::new();
    // CEO is a FRESH Human agent owned by the new org — distinct from
    // `admin.agent_id` (the platform admin). A real wizard at M3/P4
    // may have the admin nominate themselves or a different human.
    let ceo_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "CEO".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
    };
    let ceo_channel = Channel {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        kind: ChannelKind::Email,
        handle: "ceo@fixture.test".into(),
        created_at: Utc::now(),
    };
    let ceo_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: Utc::now(),
    };
    let ceo_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: Utc::now(),
    };
    let ceo_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(ceo_agent.id),
        action: vec!["allocate".into()],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![Fundamental::IdentityPrincipal, Fundamental::Tag],
        descends_from: None,
        delegable: true,
        issued_at: Utc::now(),
        revoked_at: None,
    };
    // Two system agents with phi-core blueprints — the same role
    // defaults M3/P4 will assign at the real wizard.
    let sys0_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "memory-extractor".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
    };
    let sys1_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
    };
    let sys0_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("memory-extractor".into()),
        system_prompt: Some("You distill agent memories.".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys1_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("agent-catalog".into()),
        system_prompt: Some("You maintain the org's agent catalogue.".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys0_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys0_agent.id,
        parallelize: 1,
        blueprint: sys0_blueprint,
        created_at: Utc::now(),
    };
    let sys1_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys1_agent.id,
        parallelize: 1,
        blueprint: sys1_blueprint,
        created_at: Utc::now(),
    };
    let sys_ids = [sys0_agent.id, sys1_agent.id];
    let organization = Organization {
        id: org_id,
        display_name: "Fixture Org".into(),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![TemplateKind::A],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: sys_ids.to_vec(),
        created_at: Utc::now(),
    };
    let token_budget_pool = TokenBudgetPool::new(org_id, 1_000_000, Utc::now());
    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id,
        ceo: PrincipalRef::Agent(ceo_agent.id),
        now: Utc::now(),
    });

    let payload = OrgCreationPayload {
        organization,
        ceo_agent: ceo_agent.clone(),
        ceo_channel,
        ceo_inbox,
        ceo_outbox,
        ceo_grant,
        system_agents: [(sys0_agent, sys0_profile), (sys1_agent, sys1_profile)],
        token_budget_pool,
        adoption_auth_requests: vec![adoption_ar],
        catalogue_entries: vec![
            (format!("org:{}", org_id), "control_plane".into()),
            (format!("org:{}/template:a", org_id), "control_plane".into()),
        ],
    };

    let ceo_agent_id = ceo_agent.id;
    admin
        .acc
        .store
        .apply_org_creation(&payload)
        .await
        .expect("spawn_claimed_with_org: apply_org_creation should succeed on a fresh store");

    ClaimedOrg {
        admin,
        org_id,
        ceo_agent_id,
        system_agents: sys_ids,
    }
}

// ============================================================================
// M4/P3 — `spawn_claimed_with_org_and_project` fixture
// ============================================================================

/// Bundle returned by [`spawn_claimed_with_org_and_project`]. Extends
/// [`ClaimedOrg`] with a materialised Shape A project + its
/// lead/member agents + (if Template A's fire-listener ran) the
/// issued lead grant id.
pub struct ClaimedProject {
    pub claimed_org: ClaimedOrg,
    pub project_id: domain::model::ids::ProjectId,
    pub project_lead: domain::model::ids::AgentId,
    pub project_member: domain::model::ids::AgentId,
    /// Present iff the server's `TemplateAFireListener` observed the
    /// `HasLeadEdgeCreated` event and persisted the grant. For the
    /// in-memory acceptance stack (no event emission by the fixture)
    /// this starts `None`; P6 wizard handlers that drive the full
    /// orchestration populate it.
    pub template_a_grant_id: Option<domain::model::ids::GrantId>,
}

impl ClaimedProject {
    pub fn url(&self, path: &str) -> String {
        self.claimed_org.url(path)
    }
    pub fn org_id(&self) -> domain::model::ids::OrgId {
        self.claimed_org.org_id
    }
}

/// Extend [`spawn_claimed_with_org`] with a minimal Shape A project
/// materialised via `apply_project_creation`. Two LLM agents are
/// spawned via `apply_agent_creation` (one lead + one member) so the
/// project's `HAS_LEAD` + `HAS_AGENT` edges have valid referents.
///
/// **M4/P3 scope**: this fixture does NOT emit the
/// `HasLeadEdgeCreated` domain event (the server orchestrator does
/// that at P6). Tests wanting to exercise the listener path construct
/// the event + emit directly on `admin.acc.state.event_bus`.
pub async fn spawn_claimed_with_org_and_project(with_metrics: bool) -> ClaimedProject {
    use chrono::Utc;
    use domain::model::composites_m4::ResourceBoundaries;
    use domain::model::ids::{AgentId, NodeId, ProjectId};
    use domain::model::nodes::{
        Agent, AgentKind, AgentRole, InboxObject, OutboxObject, Project, ProjectShape,
        ProjectStatus,
    };
    use domain::repository::{AgentCreationPayload, ProjectCreationPayload, Repository};

    let claimed_org = spawn_claimed_with_org(with_metrics).await;
    let org_id = claimed_org.org_id;
    let now = Utc::now();

    // Lead + member agents (LLM-kind so the AgentRole::Intern check
    // passes).
    let lead_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "project-lead".into(),
        owning_org: Some(org_id),
        role: Some(AgentRole::Intern),
        created_at: now,
    };
    let lead_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: lead_agent.id,
        created_at: now,
    };
    let lead_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: lead_agent.id,
        created_at: now,
    };
    claimed_org
        .admin
        .acc
        .store
        .apply_agent_creation(&AgentCreationPayload {
            agent: lead_agent.clone(),
            inbox: lead_inbox,
            outbox: lead_outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await
        .expect("spawn_claimed_with_org_and_project: create lead agent");

    let member_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "project-member".into(),
        owning_org: Some(org_id),
        role: Some(AgentRole::Intern),
        created_at: now,
    };
    let member_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: member_agent.id,
        created_at: now,
    };
    let member_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: member_agent.id,
        created_at: now,
    };
    claimed_org
        .admin
        .acc
        .store
        .apply_agent_creation(&AgentCreationPayload {
            agent: member_agent.clone(),
            inbox: member_inbox,
            outbox: member_outbox,
            profile: None,
            default_grants: vec![],
            initial_execution_limits_override: None,
            catalogue_entries: vec![],
        })
        .await
        .expect("spawn_claimed_with_org_and_project: create member agent");

    let project = Project {
        id: ProjectId::new(),
        name: "Fixture Project".into(),
        description: "Atlas-class memory benchmark".into(),
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
    let project_id = project.id;

    claimed_org
        .admin
        .acc
        .store
        .apply_project_creation(&ProjectCreationPayload {
            project,
            owning_orgs: vec![org_id],
            lead_agent_id: lead_agent.id,
            member_agent_ids: vec![member_agent.id],
            sponsor_agent_ids: vec![claimed_org.ceo_agent_id],
            catalogue_entries: vec![(format!("project:{}", project_id), "project".into())],
        })
        .await
        .expect("spawn_claimed_with_org_and_project: apply_project_creation");

    ClaimedProject {
        claimed_org,
        project_id,
        project_lead: lead_agent.id,
        project_member: member_agent.id,
        template_a_grant_id: None,
    }
}
