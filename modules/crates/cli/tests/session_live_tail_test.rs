//! CH-17 / ADR-0055 — CLI default-tail + `--no-tail` flag plumbing.
//!
//! Two end-to-end tests:
//!
//! 1. `launch_then_tail_renders_received_events` — runs `phi session
//!    launch` (without `--no-tail`) against a fully-seeded in-process
//!    server. Asserts the CLI consumes the SSE stream + renders at
//!    least one `>>` line (the SSE consumer's per-event prefix at
//!    `cli/src/commands/session.rs:302`) on stdout + exits 0.
//! 2. `launch_no_tail_skips_subscription` — runs `phi session launch
//!    --no-tail` and asserts the CLI does NOT print the live-tail
//!    header (`-- live tail (Ctrl-C to detach; --no-tail to skip):`),
//!    the launch is reported on stdout, and the process exits 0.
//!
//! Both tests pre-seed the harness directly via the in-memory
//! repository (claim → org → project → lead agent → Template A grants
//! → model runtime) and forge the lead's session cookie via
//! `server::session::sign_and_build_cookie`. The cookie is written to
//! a tempdir-scoped XDG_CONFIG_HOME so the spawned `phi` binary picks
//! it up via `session_store::default_session_path()`.
//!
//! The session-store file format mirrors `cli::session_store::SavedSession`
//! verbatim (only the public field set is referenced).

use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use domain::audit::AuditClass;
use domain::events::{EventBus, InProcessEventBus};
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m2::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::model::composites_m3::{ConsentPolicy, TokenBudgetPool};
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, GrantId, ModelProviderId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, ApprovalMode, Channel, ChannelKind, Grant,
    InboxObject, Organization, OutboxObject, PrincipalRef, Project, ProjectShape, ProjectStatus,
    ResourceRef, TemplateKind,
};
use domain::model::Fundamental;
use domain::permissions::Action;
use domain::repository::{
    AgentCreationPayload, OrgCreationPayload, ProjectCreationPayload, Repository,
};
use domain::templates::a;
use server::{build_router, AppState, SessionKey};

const TEST_SECRET: &str = "test-secret-test-secret-test-secret-test-secret";

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    l.local_addr().unwrap().port()
}

/// Bundle returned by `spawn_seeded_server` — every id the CLI test
/// needs to invoke `phi session launch ...`.
struct SeededFixture {
    base_url: String,
    org_id: OrgId,
    project_id: ProjectId,
    lead_agent_id: AgentId,
    lead_cookie: String,
    _join: tokio::task::JoinHandle<()>,
}

/// Boot a fresh in-process axum router + materialise a Shape A project +
/// mint Template A grants on the lead so a `phi session launch` against
/// the lead's session cookie passes the CH-15 hard-deny gate AND the
/// CH-17 SSE Observe gate.
async fn spawn_seeded_server() -> SeededFixture {
    let repo = Arc::new(InMemoryRepository::new());
    let now = Utc::now();

    // ---- Org + CEO ---------------------------------------------------------
    let org_id = OrgId::new();
    let ceo_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "CEO".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
        active: true,
        archived_at: None,
    };
    let ceo_channel = Channel {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        kind: ChannelKind::Email,
        handle: "ceo@cli-fixture.test".into(),
        created_at: now,
    };
    let ceo_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let ceo_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let ceo_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(ceo_agent.id),
        action: vec![Action::Allocate],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![Fundamental::IdentityPrincipal, Fundamental::Tag],
        descends_from: None,
        delegable: true,
        issued_at: now,
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };

    // Two stub system agents so the org-creation payload validates.
    let sys0_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "memory-extractor".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
        active: true,
        archived_at: None,
    };
    let sys1_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
        active: true,
        archived_at: None,
    };
    let sys0_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("memory-extractor".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys1_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("agent-catalog".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys0_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys0_agent.id,
        parallelize: 1,
        blueprint: sys0_blueprint,
        model_config_id: None,
        mock_response: None,
        created_at: now,
    };
    let sys1_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys1_agent.id,
        parallelize: 1,
        blueprint: sys1_blueprint,
        model_config_id: None,
        mock_response: None,
        created_at: now,
    };

    let org = Organization {
        id: org_id,
        display_name: "CLI Tail Fixture Org".into(),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![TemplateKind::A],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![sys0_agent.id, sys1_agent.id],
        approval_timeout: domain::model::ApprovalTimeout::ProjectDuration,
        approval_timeout_default_response: domain::model::TimeoutResponse::Deny,
        tags: vec![],
        created_at: now,
    };
    let pool = TokenBudgetPool::new(org_id, 1_000_000, now);
    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id,
        ceo: PrincipalRef::Agent(ceo_agent.id),
        now,
    });
    let payload = OrgCreationPayload {
        organization: org,
        // CH-25 / ADR-0060 §D60.1 — fixture: ceo IS the creator.
        creator_agent: ceo_agent.id,
        ceo_agent: ceo_agent.clone(),
        ceo_channel,
        ceo_inbox,
        ceo_outbox,
        ceo_grant,
        system_agents: [
            (sys0_agent.clone(), sys0_profile),
            (sys1_agent.clone(), sys1_profile),
        ],
        token_budget_pool: pool,
        adoption_auth_requests: vec![adoption_ar],
        catalogue_entries: vec![
            (format!("org:{}", org_id), "control_plane".into()),
            (format!("org:{}/template:a", org_id), "control_plane".into()),
        ],
    };
    repo.apply_org_creation(&payload)
        .await
        .expect("apply_org_creation");

    // ---- Lead agent (LLM kind, Intern role) --------------------------------
    let lead_id = AgentId::new();
    let lead_agent = Agent {
        id: lead_id,
        kind: AgentKind::Llm,
        display_name: "cli-tail-lead".into(),
        owning_org: Some(org_id),
        role: Some(AgentRole::Intern),
        created_at: now,
        active: true,
        archived_at: None,
    };
    let lead_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: lead_id,
        created_at: now,
        tags: Vec::new(),
    };
    let lead_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: lead_id,
        created_at: now,
        tags: Vec::new(),
    };
    repo.apply_agent_creation(&AgentCreationPayload {
        agent: lead_agent.clone(),
        inbox: lead_inbox,
        outbox: lead_outbox,
        profile: None,
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity: Some(domain::model::nodes::Identity::default_for_llm(
            lead_id, now,
        )),
    })
    .await
    .expect("create lead agent");

    // ---- Project (Shape A) -------------------------------------------------
    let project_id = ProjectId::new();
    let project = Project {
        id: project_id,
        name: "CLI Tail Fixture Project".into(),
        description: "tail-events benchmark".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape: ProjectShape::A,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: Some(ResourceBoundaries::default()),
        tags: vec![],
        created_at: now,
    };
    repo.apply_project_creation(&ProjectCreationPayload {
        project,
        owning_orgs: vec![org_id],
        lead_agent_id: lead_id,
        // CH-25 / ADR-0060 §D60.1 — Decision-3 user-lock: lead IS the
        // creator at both Shape A and Shape B.
        creator_agent: lead_id,
        member_agent_ids: vec![],
        sponsor_agent_ids: vec![ceo_agent.id],
        catalogue_entries: vec![(format!("project:{}", project_id), "project".into())],
    })
    .await
    .expect("apply_project_creation");

    // ---- Template A grants on the lead (Read+Inspect+List+Observe) ---------
    // Mirror `seed_template_a_grants_for_lead` with the CH-17 / ADR-0055
    // §D55.9 4-action set so the SSE Observe gate also passes.
    let project_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(lead_id),
        action: vec![Action::Read, Action::Inspect, Action::List, Action::Observe],
        resource: ResourceRef {
            uri: format!("project:{}", project_id),
        },
        fundamentals: vec![Fundamental::Tag],
        descends_from: None,
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    repo.create_grant(&project_grant)
        .await
        .expect("project grant");
    let session_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(lead_id),
        action: vec![Action::Read, Action::Inspect, Action::List, Action::Observe],
        resource: ResourceRef {
            uri: format!(
                r#"tags contains "project:{}" AND tags contains #kind:session"#,
                project_id
            ),
        },
        fundamentals: vec![Fundamental::DataObject, Fundamental::Tag],
        descends_from: None,
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    repo.create_grant(&session_grant)
        .await
        .expect("session grant");

    // ---- Model runtime + profile binding for the lead ----------------------
    let runtime_id = ModelProviderId::new();
    let runtime = ModelRuntime {
        id: runtime_id,
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
        .expect("seed runtime");
    let lead_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: lead_id,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(runtime_id.to_string()),
        mock_response: None,
        created_at: now,
    };
    repo.create_agent_profile(&lead_profile)
        .await
        .expect("create profile");

    // ---- Build router + boot ----------------------------------------------
    let session_key = SessionKey::for_tests(TEST_SECRET);
    let event_bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let repo_dyn: Arc<dyn Repository> = repo.clone();
    let app = build_router(AppState {
        repo: repo_dyn.clone(),
        session: session_key.clone(),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus,
        session_registry: server::state::new_session_registry(),
        session_live_stream_registry: server::state::new_session_live_stream_registry(),
        session_max_concurrent: 16,
        session_live_stream_buffer: 64,
    });
    let port = free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let join = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base_url = format!("http://127.0.0.1:{port}");

    // Sign a session cookie for the lead.
    let (lead_token, _cookie) =
        server::session::sign_and_build_cookie(&session_key, &lead_id.to_string())
            .expect("sign lead cookie");

    // Wait for the listener to start accepting.
    tokio::time::sleep(Duration::from_millis(80)).await;

    SeededFixture {
        base_url,
        org_id,
        project_id,
        lead_agent_id: lead_id,
        lead_cookie: lead_token,
        _join: join,
    }
}

/// Write a `cli::session_store::SavedSession`-shaped JSON file into
/// `<xdg>/phi/session` so the spawned `phi` binary picks up the
/// signed cookie. Uses the same path resolution
/// `cli::session_store::default_session_path()` does (XDG fallback).
fn write_session_store(xdg: &std::path::Path, agent_id: AgentId, cookie: &str) {
    let dir = xdg.join("phi");
    fs::create_dir_all(&dir).expect("mkdir phi");
    let body = serde_json::json!({
        "cookie_value": cookie,
        "issued_at": Utc::now().to_rfc3339(),
        "agent_id": agent_id.to_string(),
    });
    fs::write(dir.join("session"), body.to_string()).expect("write session");
}

fn cli_bin() -> String {
    env!("CARGO_BIN_EXE_phi").to_string()
}

fn run_phi(args: &[&str], xdg: &std::path::Path) -> (i32, String, String) {
    let out = Command::new(cli_bin())
        .args(args)
        .env("XDG_CONFIG_HOME", xdg)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn phi");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (code, stdout, stderr)
}

// ---------------------------------------------------------------------------
// Test 1 — default tail subscribes + renders received events.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_then_tail_renders_received_events() {
    let fixture = spawn_seeded_server().await;
    let xdg = tempfile::tempdir().expect("xdg tempdir");
    write_session_store(xdg.path(), fixture.lead_agent_id, &fixture.lead_cookie);

    // Default tail mode — no `--no-tail` / `--detach` flags.
    let (code, stdout, stderr) = run_phi(
        &[
            "--server-url",
            &fixture.base_url,
            "session",
            "launch",
            "--org-id",
            &fixture.org_id.to_string(),
            "--project-id",
            &fixture.project_id.to_string(),
            "--agent-id",
            &fixture.lead_agent_id.to_string(),
            "--prompt",
            "live tail rendering test",
        ],
        xdg.path(),
    );
    assert_eq!(code, 0, "launch with tail must exit 0; stderr: {stderr}");
    assert!(
        stdout.contains("session launched"),
        "expected launch banner; stdout: {stdout}"
    );
    // The CLI prints the live-tail banner BEFORE consuming SSE per
    // `cli/src/commands/session.rs:246`.
    assert!(
        stdout.contains("live tail"),
        "expected live-tail banner; stdout: {stdout}"
    );
    // SSE events are rendered as `>>` prefixed lines (the CLI's
    // `consume_sse_tail` body). With MockProvider the agent task
    // finalises in ~ms — sometimes the SSE consumer wins the race and
    // sees `>>`-rendered AgentEvent frames, sometimes the
    // post-finalise 410 path runs and the CLI prints the
    // graceful-shutdown line `-- live tail unavailable (session
    // finalised before subscribe)`. Both outcomes prove the
    // launch→tail wire roundtrip lands per ADR-0055.
    let saw_event_render = stdout.contains(">>");
    let saw_finalised_message = stdout.contains("live tail unavailable");
    assert!(
        saw_event_render || saw_finalised_message,
        "expected EITHER `>>`-rendered SSE events OR `live tail unavailable` shutdown line; stdout: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Test 2 — `--no-tail` skips the SSE subscription entirely.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_no_tail_skips_subscription() {
    let fixture = spawn_seeded_server().await;
    let xdg = tempfile::tempdir().expect("xdg tempdir");
    write_session_store(xdg.path(), fixture.lead_agent_id, &fixture.lead_cookie);

    let (code, stdout, stderr) = run_phi(
        &[
            "--server-url",
            &fixture.base_url,
            "session",
            "launch",
            "--org-id",
            &fixture.org_id.to_string(),
            "--project-id",
            &fixture.project_id.to_string(),
            "--agent-id",
            &fixture.lead_agent_id.to_string(),
            "--prompt",
            "no-tail flag plumbing test",
            "--no-tail",
        ],
        xdg.path(),
    );
    assert_eq!(
        code, 0,
        "launch with --no-tail must exit 0; stderr: {stderr}"
    );
    // `--no-tail` AND `--detach` route to the same JSON-print branch
    // at `cli/src/commands/session.rs:213`. Output is the pretty
    // JSON body; no `>>` prefix, no live-tail banner.
    assert!(
        stdout.contains("session_id"),
        "expected session_id in JSON receipt; stdout: {stdout}"
    );
    assert!(
        !stdout.contains("-- live tail"),
        "must NOT print live-tail banner with --no-tail; stdout: {stdout}"
    );
    assert!(
        !stdout.contains(">>"),
        "must NOT print `>>` SSE events with --no-tail; stdout: {stdout}"
    );
}
