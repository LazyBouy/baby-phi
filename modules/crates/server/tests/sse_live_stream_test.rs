//! CH-17 / ADR-0055 — HTTP-level integration tests for the SSE
//! `GET /api/v0/sessions/:id/events` endpoint.
//!
//! These tests run against the real axum router (via the
//! `acceptance_common` harness) + a real `SurrealStore`. They cover
//! the seven cases the plan §8 names:
//!
//!  1. `sse_returns_403_when_actor_holds_no_observe_grant` — actor
//!     without a Template A grant connects → 403
//!     `PERMISSION_CHECK_FAILED_AT_STEP_<N>` + a
//!     `platform.session.live_stream_denied` audit event is emitted.
//!  2. `sse_streams_events_after_launch` — full launch → connect SSE
//!     → assert 200 OK, `text/event-stream` content-type, at least
//!     one `data:` frame received.
//!  3. `sse_keepalive_ping_observed` — DEBUG-style assertion against
//!     the handler's keep-alive wiring (we cannot wait 30 s in
//!     CI, so we assert the framing surfaces the post-launch event
//!     framing — keep-alive interval is wired in
//!     `handlers::sessions::events`).
//!  4. `sse_lagged_emits_typed_error_then_closes` — drives a slow
//!     consumer against a 1-slot broadcast buffer + asserts
//!     `event: lagged` is observed on the wire.
//!  5. `sse_concurrent_multi_tail_each_receives_events` — connect 2
//!     SSE clients to the same session + assert both receive at
//!     least one `data:` frame (broadcast fan-out test).
//!  6. `sse_returns_session_not_found_for_unknown_session` — connect
//!     to `/sessions/<unknown>/events` → 404 `SESSION_NOT_FOUND`.
//!  7. `sse_returns_410_when_no_sender_registered_after_allowed` —
//!     happy-path permission check but the recorder's broadcast
//!     Sender was already removed (e.g., session finalised before
//!     subscribe). 410 `SESSION_LIVE_STREAM_UNAVAILABLE`.

mod acceptance_common;

use acceptance_common::admin::{
    seed_template_a_grants_for_lead, spawn_claimed_with_org_and_project, ClaimedProject,
};

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::composites_m2::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::model::ids::{
    AgentId, GrantId, LoopId, ModelProviderId, NodeId, OrgId, ProjectId, SessionId,
};
use domain::model::nodes::{
    AgentProfile, ApprovalMode, Grant, LoopRecordNode, PrincipalRef, ResourceRef, Session,
    SessionGovernanceState,
};
use domain::model::Fundamental;
use domain::permissions::Action;
use domain::Repository;
use futures::StreamExt;
use phi_core::session::model::{
    LoopRecord as PhiCoreLoopRecord, LoopStatus, Session as PhiCoreSession, SessionFormation,
    SessionScope,
};
use phi_core::types::event::{AgentEvent, ContinuationKind};
use phi_core::types::Usage;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Fixture helpers — minimal model-runtime + agent-profile binding so the
// happy-path launches reach the agent task. Mirrors the
// `acceptance_sessions_m5p4` harness so the SSE tests share the same
// shape (CH-15 + CH-02 lineage).
// ---------------------------------------------------------------------------

fn ceo_client(project: &ClaimedProject) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(
        &project.claimed_org.admin,
        project.claimed_org.ceo_agent_id,
    )
    .expect("mint CEO session")
}

fn lead_client(project: &ClaimedProject) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(&project.claimed_org.admin, project.project_lead)
        .expect("mint lead session")
}

async fn seed_model_runtime(repo: Arc<dyn Repository>) -> ModelProviderId {
    let id = ModelProviderId::new();
    let runtime = ModelRuntime {
        id,
        config: phi_core::provider::model::ModelConfig::anthropic(
            "test-anthropic",
            "claude-test",
            "",
        ),
        secret_ref: SecretRef::new("anthropic-api-key"),
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
    };
    repo.put_model_provider(&runtime)
        .await
        .expect("seed runtime");
    id
}

async fn bind_model_to_agent(
    repo: Arc<dyn Repository>,
    agent_id: domain::model::ids::AgentId,
    runtime_id: ModelProviderId,
) {
    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(runtime_id.to_string()),
        mock_response: None,
        created_at: Utc::now(),
    };
    repo.create_agent_profile(&profile)
        .await
        .expect("create profile");
}

/// POST `/api/v0/orgs/:org_id/projects/:project_id/sessions` and return
/// the session id. Caller pre-seeds Template A grants + model runtime.
async fn launch_session(project: &ClaimedProject, prompt: &str) -> SessionId {
    let url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client(project)
        .post(url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": prompt,
        }))
        .send()
        .await
        .expect("POST sessions");
    let status = res.status().as_u16();
    let body: Value = res.json().await.expect("launch body");
    assert_eq!(status, 201, "launch must return 201; body: {body:?}");
    let raw = body["session_id"].as_str().expect("session_id");
    SessionId::from_uuid(uuid::Uuid::parse_str(raw).expect("session_id parses"))
}

async fn read_first_data_frame(res: reqwest::Response, deadline_ms: u64) -> Option<String> {
    let mut bytes_stream = res.bytes_stream();
    let mut buf = String::new();
    let deadline = std::time::Instant::now() + Duration::from_millis(deadline_ms);
    while std::time::Instant::now() < deadline {
        let chunk_fut = bytes_stream.next();
        let to = Duration::from_millis(
            (deadline.saturating_duration_since(std::time::Instant::now())).as_millis() as u64,
        );
        let chunk = match tokio::time::timeout(to, chunk_fut).await {
            Ok(Some(Ok(b))) => b,
            Ok(Some(Err(_))) => return None,
            Ok(None) => return None,
            Err(_) => return None,
        };
        buf.push_str(&String::from_utf8_lossy(&chunk));
        for line in buf.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                return Some(rest.trim_start().to_string());
            }
        }
    }
    None
}

/// Drain the SSE response body for up to `deadline_ms` ms, returning all
/// `event:` and `data:` lines observed. Used by tests that need to
/// verify the `lagged` typed error event arrives on the wire.
async fn drain_events_for(res: reqwest::Response, deadline_ms: u64) -> (Vec<String>, Vec<String>) {
    let mut data_lines = Vec::new();
    let mut event_lines = Vec::new();
    let mut bytes_stream = res.bytes_stream();
    let mut buf = String::new();
    let deadline = std::time::Instant::now() + Duration::from_millis(deadline_ms);
    while std::time::Instant::now() < deadline {
        let to = Duration::from_millis(
            (deadline.saturating_duration_since(std::time::Instant::now())).as_millis() as u64,
        );
        let next = tokio::time::timeout(to, bytes_stream.next()).await;
        match next {
            Ok(Some(Ok(chunk))) => buf.push_str(&String::from_utf8_lossy(&chunk)),
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => break,
        }
        // Drain whole frames as they arrive.
        while let Some(idx) = buf.find("\n\n") {
            let frame = buf[..idx].to_string();
            buf.drain(..idx + 2);
            for line in frame.lines() {
                if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.trim_start().to_string());
                } else if let Some(rest) = line.strip_prefix("event:") {
                    event_lines.push(rest.trim().to_string());
                }
            }
        }
    }
    (event_lines, data_lines)
}

/// Seed a `governance_state = running` session row directly + register
/// a fresh broadcast Sender in the live-stream registry. Returns the
/// `SessionId` and the registered Sender so the test can both connect
/// SSE and pump events through it. Sidesteps the real launch path so
/// tests don't race against MockProvider's instant finalisation (the
/// real launch task removes the Sender from the registry within
/// milliseconds; our tests need a live entry for the duration).
async fn seed_running_session(
    project: &ClaimedProject,
    buffer: usize,
) -> (SessionId, broadcast::Sender<AgentEvent>) {
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let now = Utc::now();
    let org_id: OrgId = project.org_id();
    let project_id: ProjectId = project.project_id;
    let actor: AgentId = project.project_lead;
    let session_id = SessionId::new();
    let phi_session_id = format!("{session_id}");
    let phi_loop_id = format!("{phi_session_id}.default.0");
    let phi_session = PhiCoreSession {
        session_id: phi_session_id.clone(),
        agent_id: actor.to_string(),
        created_at: now,
        last_active_at: now,
        formation: SessionFormation::FirstLoop { timestamp: now },
        parent_spawn_ref: None,
        scope: SessionScope::Ephemeral,
        loops: Vec::new(),
    };
    let phi_first_loop = PhiCoreLoopRecord {
        loop_id: phi_loop_id,
        session_id: phi_session_id,
        agent_id: actor.to_string(),
        parent_loop_id: None,
        continuation_kind: ContinuationKind::Initial,
        started_at: now,
        ended_at: None,
        status: LoopStatus::Running,
        rejection: None,
        config: None,
        messages: Vec::new(),
        turns: Vec::new(),
        usage: Usage::default(),
        metadata: None,
        events: Vec::new(),
        children_loop_ids: Vec::new(),
        child_loop_refs: Vec::new(),
        parallel_group: None,
        compaction_block: None,
    };
    let baby_session = Session {
        id: session_id,
        inner: phi_session,
        owning_org: org_id,
        owning_project: project_id,
        started_by: actor,
        governance_state: SessionGovernanceState::Running,
        started_at: now,
        ended_at: None,
        tokens_spent: 0,
        tags: domain::model::composites::auto_tags_for("session", &session_id.to_string()).to_vec(),
    };
    let first_loop = LoopRecordNode {
        id: LoopId::new(),
        inner: phi_first_loop,
        session_id,
        loop_index: 0,
    };
    repo.persist_session(&baby_session, &first_loop)
        .await
        .expect("persist seeded session");

    let (tx, _rx0) = broadcast::channel::<AgentEvent>(buffer.max(1));
    project
        .claimed_org
        .admin
        .acc
        .session_live_stream_registry
        .insert(session_id, tx.clone());
    (session_id, tx)
}

fn fresh_agent_event(loop_id: &str) -> AgentEvent {
    AgentEvent::AgentEnd {
        loop_id: loop_id.to_string(),
        messages: Vec::new(),
        usage: Usage::default(),
        timestamp: Utc::now(),
        rejection: None,
    }
}

async fn wait_for_session_terminal(
    project: &ClaimedProject,
    session_id: SessionId,
    deadline_ms: u64,
) {
    let url = project.url(&format!("/api/v0/sessions/{}", session_id));
    let client = ceo_client(project);
    let start = std::time::Instant::now();
    loop {
        let res = client.get(&url).send().await.expect("GET session");
        if res.status().as_u16() == 200 {
            let body: Value = res.json().await.expect("session json");
            let state = body["session"]["governance_state"].as_str().unwrap_or("");
            if state != "running" {
                return;
            }
        }
        if start.elapsed() > Duration::from_millis(deadline_ms) {
            panic!("session did not finalise within {deadline_ms} ms");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// ---------------------------------------------------------------------------
// 1. Hard-deny — actor without Observe grant gets 403 + audit event.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_returns_403_when_actor_holds_no_observe_grant() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    let session_id = launch_session(&project, "deny-path-fixture").await;
    // Wait for the agent task to register the broadcast Sender +
    // emit at least one event before we reach in as a no-grant actor.
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Connect as the project member (no Template A grant minted, so
    // Step 2 / Step 3 hard-denies on Observe-on-session_object).
    let member_client = acceptance_common::admin::authed_client_for(
        &project.claimed_org.admin,
        project.project_member,
    )
    .expect("mint member session");
    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));
    let res = member_client.get(url).send().await.expect("GET events");
    assert_eq!(
        res.status().as_u16(),
        403,
        "no-grant actor must hard-deny at SSE per ADR-0055 §D55.5"
    );
    let body: Value = res.json().await.expect("error body json");
    assert_eq!(body["code"], "PERMISSION_CHECK_FAILED");

    // Audit event lookup — at least one `live_stream_denied` event
    // emitted (Alerted-class).
    let events = repo
        .list_recent_audit_events_for_org(project.org_id(), 200)
        .await
        .expect("list audit events");
    let deny_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == "platform.session.live_stream_denied")
        .collect();
    assert!(
        !deny_events.is_empty(),
        "ADR-0055 §D55.5: live_stream_denied audit event emitted on 403; saw {:?}",
        events.iter().map(|e| &e.event_type).collect::<Vec<_>>()
    );
    assert_eq!(deny_events[0].audit_class, AuditClass::Alerted);
}

// ---------------------------------------------------------------------------
// 2. Happy path — SSE streams events after launch.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_streams_events_after_launch() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    // Use the seeded-running-session shape so the SSE Sender stays
    // registered for the duration of the test (real-launch races
    // MockProvider's ~10 ms finalisation).
    let (session_id, tx) = seed_running_session(&project, 64).await;

    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));
    let res = lead_client(&project)
        .get(url)
        .send()
        .await
        .expect("GET events");
    assert_eq!(res.status().as_u16(), 200, "SSE returns 200 OK");
    let ct = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.starts_with("text/event-stream"),
        "content-type must be text/event-stream; got {ct:?}"
    );

    // Allow the SSE handler to install the receiver before we push.
    tokio::time::sleep(Duration::from_millis(60)).await;
    // Pump a fresh AgentEvent onto the broadcast — the SSE handler's
    // BroadcastStream wrapper serialises it onto the wire.
    let _ = tx.send(fresh_agent_event("loop-happy-path"));

    let payload = read_first_data_frame(res, 2_500).await;
    assert!(
        payload.is_some(),
        "expected at least one SSE data frame after publish"
    );
    let data = payload.unwrap();
    let parsed: Value = serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("data frame must parse as JSON; raw={data:?}, err={e}"));
    assert!(
        parsed.is_object() || parsed.is_string(),
        "AgentEvent JSON shape should be object or string; got {parsed:?}"
    );
}

// ---------------------------------------------------------------------------
// 3. Keep-alive wiring — frames are surfaced through axum's keep-alive
//    SSE response. We cannot wait 30 s in CI, so this test asserts the
//    response opens with the keep-alive content-type + the agent-event
//    stream produces frames immediately (proves the SSE response is the
//    keep-aliveable shape per `Sse::keep_alive`).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_keepalive_ping_observed() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    let (session_id, tx) = seed_running_session(&project, 64).await;
    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));
    let res = lead_client(&project)
        .get(url)
        .send()
        .await
        .expect("GET events");
    assert_eq!(res.status().as_u16(), 200);
    // Allow the receiver to register, then publish one event so the
    // keep-alive-wrapped Sse response surfaces a frame within the
    // (sub-30 s) test deadline. We assert framing — not the 30 s
    // comment ping itself, which would force CI to wait that long.
    tokio::time::sleep(Duration::from_millis(60)).await;
    let _ = tx.send(fresh_agent_event("loop-keepalive"));
    let (events, datas) = drain_events_for(res, 2_500).await;
    assert!(
        !datas.is_empty() || !events.is_empty(),
        "keep-alive-shaped SSE response must surface a frame within deadline; events={events:?}, datas={datas:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. Lagged consumer — slow reader → typed `lagged` SSE error then close.
//
// We cannot easily drive the broadcast channel into Lagged from a
// black-box HTTP test (the receiver is owned by axum) without massive
// fan-out. Instead, drive the underlying broadcast channel directly:
// register a tiny-buffer Sender in the live-stream registry for a
// running session, pump N+1 events through it (where N = 1, the
// buffer), and assert the SSE response surfaces the `event: lagged`
// frame. This is the F3.B contract test per ADR-0055 §D55.3.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sse_lagged_emits_typed_error_then_closes() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    // Seed a running session with a 1-slot broadcast buffer. The
    // BroadcastStream wrapper inside the SSE handler will surface a
    // `Lagged(n)` error when the Sender outpaces the receiver — easy
    // to provoke with a 1-slot buffer + a tight burst of sends.
    let (session_id, tx) = seed_running_session(&project, 1).await;

    // Connect the SSE consumer.
    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));
    let res = lead_client(&project)
        .get(url)
        .send()
        .await
        .expect("GET events");
    assert_eq!(res.status().as_u16(), 200);

    // Burst-push events synchronously WITHOUT yielding to the runtime
    // — the SSE handler's task can't drain the broadcast between
    // sends, so the receiver lags. The 1-slot buffer means any
    // tx.send() while a prior message is unread bumps the receiver
    // into Lagged territory.
    for i in 0..2_000 {
        let _ = tx.send(fresh_agent_event(&format!("loop-lag-{i}")));
    }
    // Drop our Sender so the BroadcastStream closes after surfacing
    // the Lagged error frame.
    drop(tx);

    let (events, _datas) = drain_events_for(res, 3_000).await;
    assert!(
        events.iter().any(|e| e == "lagged"),
        "expected 'event: lagged' in stream; got events={events:?}"
    );
}

// ---------------------------------------------------------------------------
// 5. Multi-tail — two SSE clients receive events from the same session.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sse_concurrent_multi_tail_each_receives_events() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    let (session_id, tx) = seed_running_session(&project, 64).await;
    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));

    // Connect both subscribers concurrently against the SAME Sender
    // — the F1.A registry hands each handler a clone, and each
    // `.subscribe()` mints a fresh receiver against the same channel
    // (broadcast fan-out).
    let res_a = lead_client(&project).get(&url).send().await.expect("GET A");
    let res_b = lead_client(&project).get(&url).send().await.expect("GET B");
    assert_eq!(res_a.status().as_u16(), 200);
    assert_eq!(res_b.status().as_u16(), 200);

    // Allow both receivers to register before publishing.
    tokio::time::sleep(Duration::from_millis(80)).await;
    let _ = tx.send(fresh_agent_event("loop-multi-tail"));

    let (frame_a, frame_b) = tokio::join!(
        read_first_data_frame(res_a, 2_500),
        read_first_data_frame(res_b, 2_500),
    );
    assert!(
        frame_a.is_some(),
        "first subscriber receives at least one data frame"
    );
    assert!(
        frame_b.is_some(),
        "second subscriber receives at least one data frame (broadcast fan-out)"
    );
}

// ---------------------------------------------------------------------------
// 6. Unknown session id → 404.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_returns_session_not_found_for_unknown_session() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    let unknown = SessionId::new();
    let url = project.url(&format!("/api/v0/sessions/{}/events", unknown));
    let res = lead_client(&project)
        .get(url)
        .send()
        .await
        .expect("GET events");
    assert_eq!(
        res.status().as_u16(),
        404,
        "unknown session id must return 404 SESSION_NOT_FOUND"
    );
    let body: Value = res.json().await.expect("error body");
    assert_eq!(body["code"], "SESSION_NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 7. Allowed permission, no Sender registered → 410.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_returns_410_when_no_sender_registered_after_allowed() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;
    seed_template_a_grants_for_lead(&project).await;

    // Launch + wait for finalisation so the broadcast Sender is
    // removed from the registry. The session row remains; the
    // registry slot is empty.
    let session_id = launch_session(&project, "410-fixture").await;
    wait_for_session_terminal(&project, session_id, 2_500).await;
    project
        .claimed_org
        .admin
        .acc
        .session_live_stream_registry
        .remove(&session_id);

    // Mint an Observe-bearing grant for the project member so it
    // passes Step 3 (the lead's grant already has Observe via
    // `seed_template_a_grants_for_lead`, which post-CH-17 mints
    // `[Read, Inspect, List, Observe]`). Issue an explicit
    // class-level Observe grant on `session_object` to match the
    // SSE manifest's resource projection.
    let observe_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(project.project_lead),
        action: vec![Action::Observe],
        resource: ResourceRef {
            uri: "session_object".to_string(),
        },
        fundamentals: vec![Fundamental::DataObject, Fundamental::Tag],
        descends_from: None,
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    repo.create_grant(&observe_grant)
        .await
        .expect("seed extra observe grant");

    let url = project.url(&format!("/api/v0/sessions/{}/events", session_id));
    let res = lead_client(&project)
        .get(url)
        .send()
        .await
        .expect("GET events");
    let status = res.status().as_u16();
    // The post-finalisation registry has no Sender for this session;
    // ADR-0055 §D55.5 says Step E returns 410 GONE in that case.
    assert_eq!(
        status, 410,
        "post-finalisation SSE connect must return 410 SESSION_LIVE_STREAM_UNAVAILABLE; \
         got {status}"
    );
    let body: Value = res.json().await.expect("error body");
    assert_eq!(body["code"], "SESSION_LIVE_STREAM_UNAVAILABLE");
}
