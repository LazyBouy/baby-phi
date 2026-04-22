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
    /// The raw `baby_phi_session` cookie value (the JWT). Useful for
    /// tests that want to sign a forged cookie variant.
    pub session_cookie: String,
    /// reqwest client preconfigured with `Cookie: baby_phi_session=<jwt>`
    /// on every request.
    pub authed_client: reqwest::Client,
}

impl ClaimedAdmin {
    /// Convenience — absolute URL for a path on the acceptance server.
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.acc.base_url, path)
    }
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
        .split("baby_phi_session=")
        .nth(1)
        .expect("baby_phi_session= prefix present")
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
        HeaderValue::from_str(&format!("baby_phi_session={session_cookie}"))
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
        created_at: Utc::now(),
    };
    let sys1_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
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
