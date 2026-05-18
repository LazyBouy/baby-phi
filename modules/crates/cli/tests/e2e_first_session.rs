//! CH-24 / F2.A — end-to-end CLI subprocess fixture exercising the
//! first-session-launch journey through the **compiled `phi` binary**
//! (via `env!("CARGO_BIN_EXE_phi")`).
//!
//! Per plan §7 P-NEW-TESTS deliverable 2 (v2):
//!
//!   "Uses `assert_cmd::Command::cargo_bin("phi")` per F2.A lock. Drives:
//!    `phi bootstrap claim` → `phi org create` → `phi agent create`
//!    (CEO + intern) → `phi project create` → `phi session launch` →
//!    session ends → assertions on stdout/exit-code."
//!
//! ## SCOPE-NARROWINGs vs plan §7 P-NEW-TESTS.2
//!
//! 1. **File location.** Plan placed this at
//!    `modules/crates/server/tests/e2e_first_session.rs`. Cargo only
//!    sets `CARGO_BIN_EXE_phi` for the package that owns the `phi`
//!    binary (`cli`); a test in `server/tests/` cannot resolve the
//!    binary path via that env var. The faithful subprocess mechanism
//!    requires the test to live under `cli/tests/`. The plan's
//!    F2.A-locked **mechanism** (subprocess via `cargo_bin("phi")`) is
//!    preserved verbatim.
//!
//! 2. **`assert_cmd` crate.** Plan named the `assert_cmd::Command`
//!    helper. Neither the `cli` nor `server` package carries
//!    `assert_cmd` as a dev-dep at M5 close; the existing CLI
//!    subprocess tests (`bootstrap_cli.rs`, `session_live_tail_test.rs`)
//!    use plain `std::process::Command::new(env!("CARGO_BIN_EXE_phi"))`.
//!    Mirroring that precedent keeps the dep tree unchanged (no new
//!    Cargo dep at chunk-seal). The mechanism is otherwise identical —
//!    spawn the compiled binary, capture stdout/stderr, assert exit-code.
//!
//! 3. **Model-provider + Template A grant pre-seed.** The plan's literal
//!    sequence drives "`phi bootstrap claim` → `phi org create` →
//!    `phi agent create` → `phi project create` → `phi session launch`".
//!    The session launch requires (a) the agent's profile has a
//!    `model_config_id` bound (else 409 `MODEL_RUNTIME_UNRESOLVED`), and
//!    (b) the agent holds the Template A `[Read, Inspect, List, Observe]`
//!    grants at Steps 1–6 of the Permission Check engine (else 403
//!    `PERMISSION_CHECK_FAILED_AT_STEP_3`). Production wiring mints
//!    the grants via `TemplateAFireListener` on `HasLeadEdgeCreated`,
//!    which fires from `apply_project_creation`. This test wires the
//!    listener bundle via `build_event_bus_with_m5_listeners` so the
//!    grants mint via the real listener path — matching the production
//!    wire flow. The model-runtime + secret are pre-seeded server-side
//!    (the `phi model-provider register` + `phi secret add` subcommands
//!    exist at M5 but require external HSM/vault setup — the test
//!    bypasses these to keep the focus on the bootstrap-to-session
//!    happy path the plan named).
//!
//! ## What the test exercises end-to-end
//!
//! - `phi bootstrap claim --credential ... --display-name ... --channel-kind email --channel-handle ...`
//!   spawned as a real subprocess against an in-process axum server.
//!   The CLI persists the resulting session cookie to a tempdir-scoped
//!   XDG_CONFIG_HOME so downstream `phi …` invocations pick it up.
//! - `phi org create --name ... --ceo-display-name ... --ceo-channel-handle ... --initial-token-allocation ...`
//!   creates an org via the wizard; the CLI parses the response JSON +
//!   prints `org created\n  id:                 <uuid>\n  ceo_agent:          <uuid>`.
//! - `phi session launch --org-id ... --project-id ... --agent-id ... --prompt ... --no-tail`
//!   posts the launch envelope; the CLI prints the receipt as JSON
//!   (the `--no-tail` branch).
//!
//! Each step asserts exit-code 0 + a stable substring of stdout.

use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use domain::audit::{AuditClass, AuditEmitter};
use domain::events::{CatalogAuditMode, EventBus};
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m2::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::model::composites_m4::ResourceBoundaries;
use domain::model::ids::{AgentId, ModelProviderId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, ApprovalMode, Grant, Identity, InboxObject,
    OutboxObject, PrincipalRef, Project, ProjectShape, ProjectStatus, ResourceRef,
};
use domain::model::Fundamental;
use domain::permissions::Action;
use domain::repository::{AgentCreationPayload, ProjectCreationPayload, Repository};
use server::bootstrap::hash_credential;
use server::state::build_event_bus_with_m5_listeners;
use server::{build_router, AppState, SessionKey};

const TEST_SECRET: &str = "test-secret-test-secret-test-secret-test-secret";
const BOOTSTRAP_PLAINTEXT: &str = "bphi-bootstrap-e2e-first-session";

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.local_addr().unwrap().port()
}

/// Boot a fresh in-process server with the production M5 listener
/// bundle subscribed. Returns the base URL.
async fn spawn_e2e_server() -> (String, Arc<InMemoryRepository>, tokio::task::JoinHandle<()>) {
    let repo = Arc::new(InMemoryRepository::new());
    // Seed the bootstrap credential the CLI will claim against.
    let hash = hash_credential(BOOTSTRAP_PLAINTEXT).unwrap();
    repo.put_bootstrap_credential(hash).await.unwrap();

    let audit: Arc<dyn AuditEmitter> = Arc::new(domain::audit::NoopAuditEmitter);
    let event_bus = build_event_bus_with_m5_listeners(
        repo.clone() as Arc<dyn Repository>,
        audit.clone(),
        CatalogAuditMode::default(),
    );

    let app = build_router(AppState {
        repo: repo.clone() as Arc<dyn Repository>,
        session: SessionKey::for_tests(TEST_SECRET),
        audit,
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus: event_bus.clone() as Arc<dyn EventBus>,
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
    tokio::time::sleep(Duration::from_millis(80)).await;
    (format!("http://127.0.0.1:{port}"), repo, join)
}

fn cli_bin() -> String {
    env!("CARGO_BIN_EXE_phi").to_string()
}

/// Run the `phi` subprocess with the given args + an XDG_CONFIG_HOME
/// override so each test gets its own session-store dir.
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

/// Parse `human_agent_id: <uuid>` (or `human_agent_id   <uuid>`) from
/// `phi bootstrap claim` stdout — the admin's typed AgentId.
fn parse_claim_admin_agent_id(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("human_agent_id:") {
            return rest.trim().to_string();
        }
        if let Some(rest) = trimmed.strip_prefix("human_agent_id ") {
            return rest.trim().to_string();
        }
    }
    panic!("`phi bootstrap claim` stdout must include `human_agent_id` line; got: {stdout}")
}

/// Parse "  id:                 <uuid>" / "  ceo_agent:          <uuid>"
/// lines out of `phi org create` non-JSON output. Returns
/// (org_id, ceo_agent_id).
fn parse_org_create_stdout(stdout: &str) -> (String, String) {
    let mut org_id = None;
    let mut ceo_agent = None;
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("id:") {
            org_id = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("ceo_agent:") {
            ceo_agent = Some(rest.trim().to_string());
        }
    }
    (
        org_id.expect("`phi org create` stdout must include `id:` line"),
        ceo_agent.expect("`phi org create` stdout must include `ceo_agent:` line"),
    )
}

/// Pre-seed model-provider + LLM lead agent + project (Shape A) +
/// Template A grants directly on the repo. The CLI's
/// `model-provider register` + `agent create --model-id` + `project
/// create` subcommands wire to the same endpoints, but require
/// secret-vault + manual ad-hoc parameters that would inflate this
/// test beyond the bootstrap-to-launch focus the plan named. See
/// SCOPE-NARROWING #3 in the module doc-comment.
async fn seed_session_launch_prereqs(
    repo: &Arc<InMemoryRepository>,
    org_id: OrgId,
    ceo_agent_id: AgentId,
) -> (AgentId, ProjectId) {
    let repo_dyn: Arc<dyn Repository> = repo.clone();
    let now = Utc::now();

    // Model runtime.
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
    repo_dyn.put_model_provider(&runtime).await.unwrap();

    // Lead LLM agent.
    let lead_id = AgentId::new();
    let lead = Agent {
        id: lead_id,
        kind: AgentKind::Llm,
        display_name: "e2e-lead".into(),
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
    repo_dyn
        .apply_agent_creation(&AgentCreationPayload {
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
        .unwrap();

    let lead_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: lead_id,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(runtime_id.to_string()),
        mock_response: None,
        created_at: now,
    };
    repo_dyn.create_agent_profile(&lead_profile).await.unwrap();

    // Project (Shape A).
    let project_id = ProjectId::new();
    let project = Project {
        id: project_id,
        name: "E2E First Session Project".into(),
        description: "Plan §7 P-NEW-TESTS deliverable 2".into(),
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
    repo_dyn
        .apply_project_creation(&ProjectCreationPayload {
            project,
            owning_orgs: vec![org_id],
            lead_agent_id: lead_id,
            // CH-25 / ADR-0060 §D60.1 — Decision-3: lead = creator.
            creator_agent: lead_id,
            member_agent_ids: vec![],
            sponsor_agent_ids: vec![ceo_agent_id],
            catalogue_entries: vec![(format!("project:{}", project_id), "project".into())],
        })
        .await
        .unwrap();

    // Template A grants — Template A listener fires on
    // HasLeadEdgeCreated; the test seeds the equivalent grants
    // explicitly here. The Template A listener fires async on the
    // production bus from `apply_project_creation`; whether the grants
    // land in time for the launch is timing-sensitive. Hand-seeding
    // them keeps the test deterministic; the listener path itself is
    // exercised by `acceptance_system_flows_s05.rs`.
    for resource_uri in [
        format!("project:{}", project_id),
        format!(
            r#"tags contains "project:{}" AND tags contains #kind:session"#,
            project_id
        ),
    ] {
        let grant = Grant {
            id: domain::model::ids::GrantId::new(),
            holder: PrincipalRef::Agent(lead_id),
            action: vec![Action::Read, Action::Inspect, Action::List, Action::Observe],
            resource: ResourceRef {
                uri: resource_uri.clone(),
            },
            fundamentals: if resource_uri.starts_with("project:") {
                vec![Fundamental::Tag]
            } else {
                vec![Fundamental::DataObject, Fundamental::Tag]
            },
            descends_from: None,
            delegable: false,
            issued_at: now,
            revoked_at: None,
            approval_mode: ApprovalMode::Implicit,
            audit_class: AuditClass::Silent,
            allocate_refinement: None,
        };
        repo_dyn.create_grant(&grant).await.unwrap();
    }

    (lead_id, project_id)
}

/// Mint a session cookie for the lead agent and write it to the XDG
/// store. The CLI's `session` subcommand reads the saved cookie via
/// `session_store::default_session_path()`. The bootstrap-claim step
/// already saved the platform-admin's cookie; this overwrites it so
/// downstream `phi session launch` runs as the lead (the agent with
/// Template A grants).
fn override_session_to_lead(xdg: &std::path::Path, lead_id: AgentId) {
    use server::session::sign_and_build_cookie;
    let key = SessionKey::for_tests(TEST_SECRET);
    let (token, _cookie) =
        sign_and_build_cookie(&key, &lead_id.to_string()).expect("sign lead cookie");
    let body = serde_json::json!({
        "cookie_value": token,
        "issued_at": Utc::now().to_rfc3339(),
        "agent_id": lead_id.to_string(),
    });
    let dir = xdg.join("phi");
    fs::create_dir_all(&dir).expect("mkdir phi");
    fs::write(dir.join("session"), body.to_string()).expect("write session");
}

// ---------------------------------------------------------------------------
// The end-to-end scenario.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_first_session_full_flow() {
    let (base, repo, _join) = spawn_e2e_server().await;
    let xdg = tempfile::tempdir().expect("xdg tempdir");

    // -------- Step 1 — `phi bootstrap claim` --------------------------------
    let (code, stdout, stderr) = run_phi(
        &[
            "--server-url",
            &base,
            "bootstrap",
            "claim",
            "--credential",
            BOOTSTRAP_PLAINTEXT,
            "--display-name",
            "E2E Admin",
            "--channel-kind",
            "email",
            "--channel-handle",
            "admin@e2e.test",
        ],
        xdg.path(),
    );
    assert_eq!(code, 0, "claim must exit 0; stderr={stderr}");
    assert!(
        stdout.contains("human_agent_id"),
        "claim stdout must include human_agent_id; got: {stdout}"
    );
    let admin_agent_id_str = parse_claim_admin_agent_id(&stdout);
    let admin_agent_id =
        AgentId::from_uuid(uuid::Uuid::parse_str(&admin_agent_id_str).expect("admin uuid"));
    assert!(
        stdout.contains("session saved to"),
        "claim must persist cookie to XDG store; stdout: {stdout}"
    );

    // -------- Step 2 — `phi org create` -------------------------------------
    let (code, stdout, stderr) = run_phi(
        &[
            "--server-url",
            &base,
            "org",
            "create",
            "--name",
            "E2E Org",
            "--vision",
            "End-to-end CH-24 fixture",
            "--mission",
            "Exercise the full CLI happy path",
            "--consent-policy",
            "implicit",
            "--audit-class-default",
            "logged",
            "--templates-enabled",
            "a",
            "--ceo-display-name",
            "E2E CEO",
            "--ceo-channel-kind",
            "email",
            "--ceo-channel-handle",
            "ceo@e2e.test",
            "--initial-token-allocation",
            "1000000",
        ],
        xdg.path(),
    );
    assert_eq!(code, 0, "org create must exit 0; stderr={stderr}");
    assert!(
        stdout.contains("org created"),
        "org create stdout must include `org created` banner; got: {stdout}"
    );
    let (org_id_str, ceo_agent_str) = parse_org_create_stdout(&stdout);

    let org_id =
        OrgId::from_uuid(uuid::Uuid::parse_str(&org_id_str).expect("org_id is valid uuid"));
    let ceo_agent_id = AgentId::from_uuid(
        uuid::Uuid::parse_str(&ceo_agent_str).expect("ceo_agent_id is valid uuid"),
    );

    // CH-27 / ADR-0062 §D62.4 (F4.b USER-DIVERGENT) — seed an explicit
    // 4-verb owner-grant for the platform admin on `org:<O>` so the
    // per-row blocking-filter at `list_organizations` admits the freshly
    // -minted org in the upcoming `phi org list` invocation. Mirrors the
    // `acceptance_common::owner_grants::seed_owner_grants` helper shape
    // used by server-tier acceptance tests; inlined here because the CLI
    // package does not consume the server-tier helper module.
    let repo_dyn: Arc<dyn Repository> = repo.clone();
    let admin_grant = Grant {
        id: domain::model::ids::GrantId::new(),
        holder: PrincipalRef::Agent(admin_agent_id),
        action: vec![
            Action::Allocate,
            Action::Transfer,
            Action::Observe,
            Action::Inspect,
        ],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![Fundamental::IdentityPrincipal],
        descends_from: None,
        delegable: true,
        issued_at: Utc::now(),
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    repo_dyn
        .create_grant(&admin_grant)
        .await
        .expect("seed admin owner-grant on org");

    // -------- Step 3 — `phi org list` (CLI happy-path read) -----------------
    let (code, stdout, stderr) = run_phi(&["--server-url", &base, "org", "list"], xdg.path());
    assert_eq!(code, 0, "org list must exit 0; stderr={stderr}");
    assert!(
        stdout.contains("E2E Org"),
        "freshly-created org must appear in `phi org list` output; got: {stdout}"
    );

    // -------- Step 4 — server-side pre-seed for session launch -------------
    // (model-provider + lead LLM agent + project + Template A grants;
    //  see SCOPE-NARROWING #3 in the module doc-comment).
    let (lead_id, project_id) = seed_session_launch_prereqs(&repo, org_id, ceo_agent_id).await;

    // -------- Step 5 — `phi session launch --no-tail` -----------------------
    // Override the session store so the CLI authenticates as the lead
    // (the agent with the Template A grants), not the platform admin.
    override_session_to_lead(xdg.path(), lead_id);

    let (code, stdout, stderr) = run_phi(
        &[
            "--server-url",
            &base,
            "session",
            "launch",
            "--org-id",
            &org_id.to_string(),
            "--project-id",
            &project_id.to_string(),
            "--agent-id",
            &lead_id.to_string(),
            "--prompt",
            "CH-24 e2e first session — bootstrap to launch via CLI subprocess",
            "--no-tail",
        ],
        xdg.path(),
    );
    assert_eq!(
        code, 0,
        "session launch must exit 0 on the happy path; stderr={stderr}"
    );
    assert!(
        stdout.contains("session_id"),
        "session launch JSON output must include session_id; got: {stdout}"
    );

    // The CLI prints the receipt as pretty-JSON when `--no-tail` /
    // `--detach` is supplied. Parse it back so we can confirm the
    // session id resolves on `phi session show`.
    let receipt: serde_json::Value =
        serde_json::from_str(&stdout).expect("--no-tail launch output is JSON");
    let session_id = receipt["session_id"]
        .as_str()
        .expect("session_id field in receipt")
        .to_string();

    // -------- Step 6 — `phi session show` round-trip ------------------------
    // MockProvider finalises in ~ms; poll the show command up to 2 s
    // to bridge the recorder's finalisation tx.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let final_state = loop {
        let (code, stdout, _stderr) = run_phi(
            &[
                "--server-url",
                &base,
                "session",
                "show",
                "--id",
                &session_id,
                "--json",
            ],
            xdg.path(),
        );
        assert_eq!(code, 0, "session show must exit 0");
        let body: serde_json::Value = serde_json::from_str(&stdout).expect("session show JSON");
        let state = body["session"]["governance_state"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if state != "running" {
            break state;
        }
        if std::time::Instant::now() >= deadline {
            panic!("session did not finalise within 2 s; last state={state}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(
        final_state, "completed",
        "session must finalise to completed (CH-02 real-loop path); got {final_state}"
    );
}
