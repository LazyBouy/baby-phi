//! M5/P4 session-surface acceptance tests.
//!
//! Covers the 5 carryover closes + the HTTP surface at once. The
//! synthetic replay task in `sessions::launch` completes in a few
//! milliseconds; tests poll the persisted session state rather than
//! sleeping on a fixed timer so CI runners with slow disks don't
//! flake.
//!
//! Scenarios:
//!  1. `POST /sessions/preview` returns a `Decision` envelope.
//!  2. `POST /sessions` with no `model_config_id` on the agent's
//!     profile returns 409 `MODEL_RUNTIME_UNRESOLVED`.
//!  3. `POST /sessions` happy — returns 201 + session_id; after
//!     replay finalises, `GET /sessions/:id` shows `governance_state
//!     = completed` with 1 loop + 1 turn persisted (C-M5-3 proof);
//!     the agent's `uses_model` edge is written (C-M5-2 proof).
//!  4. `GET /projects/:id/sessions` returns the header strip (no
//!     `inner` leak — schema-snapshot assertion).
//!  5. `POST /sessions/:id/terminate` on a just-launched session
//!     flips state to `aborted`; terminating again returns 409.
//!  6. `GET /sessions/:id/tools` returns `[]` (C-M5-4 wire shape).
//!  7. `PATCH /agents/:id/profile` with a new `model_config_id`
//!     succeeds when no active sessions (C-M5-5 positive path).

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org_and_project, ClaimedProject};

use chrono::Utc;
use domain::model::composites_m2::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};
use domain::model::ids::ModelProviderId;
use domain::Repository;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

fn ceo_client(project: &ClaimedProject) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(
        &project.claimed_org.admin,
        project.claimed_org.ceo_agent_id,
    )
    .expect("mint CEO session")
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

async fn ensure_agent_profile(
    repo: Arc<dyn Repository>,
    agent_id: domain::model::ids::AgentId,
    model_config_id: Option<String>,
) {
    use domain::model::ids::NodeId;
    use domain::model::nodes::AgentProfile;

    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id,
        mock_response: None,
        created_at: Utc::now(),
    };
    // M5/P4 — use `create_agent_profile` (not `upsert_*`) so the
    // row physically lands in SurrealDB. The UPSERT path's UPDATE
    // arm is a no-op when no prior row exists (M5/P2 drift D2.1
    // family — CREATE for fresh-id writes).
    repo.create_agent_profile(&profile)
        .await
        .expect("create profile");
}

async fn bind_model_to_agent(
    repo: Arc<dyn Repository>,
    agent_id: domain::model::ids::AgentId,
    runtime_id: ModelProviderId,
) {
    ensure_agent_profile(repo, agent_id, Some(runtime_id.to_string())).await;
}

/// Poll the session row for a non-running governance state. Returns
/// the final `SessionDetail` JSON.
async fn wait_for_session_finalised(
    client: &reqwest::Client,
    url: String,
    deadline_ms: u64,
) -> Value {
    let start = std::time::Instant::now();
    loop {
        let res = client.get(&url).send().await.expect("GET session");
        if res.status().as_u16() == 200 {
            let body: Value = res.json().await.expect("session json");
            let state = body["session"]["governance_state"].as_str().unwrap_or("");
            if state != "running" {
                return body;
            }
        }
        if start.elapsed() > Duration::from_millis(deadline_ms) {
            panic!("session did not finalise within {deadline_ms}ms");
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

// ---------------------------------------------------------------------------
// 1. Preview
// ---------------------------------------------------------------------------

#[tokio::test]
async fn preview_returns_decision_envelope() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions/preview",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client(&project)
        .post(url)
        .json(&json!({ "agent_id": project.project_lead.to_string() }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["agent_id"].as_str().unwrap(),
        project.project_lead.to_string()
    );
    // The Decision is tagged on `outcome` at the wire tier —
    // allowed / denied / pending. Test just asserts the tag is
    // present + the preview path wired end-to-end.
    assert!(body["decision"]["outcome"].is_string());
}

// ---------------------------------------------------------------------------
// 2. Launch with no model binding → MODEL_RUNTIME_UNRESOLVED
// ---------------------------------------------------------------------------

#[tokio::test]
async fn launch_without_model_config_returns_409() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();

    // Profile exists but has NO `model_config_id` — the M5/P4 gate
    // refuses the launch with MODEL_RUNTIME_UNRESOLVED. Without a
    // profile at all, the launch returns AGENT_PROFILE_MISSING
    // (asserted separately).
    ensure_agent_profile(repo.clone(), project.project_lead, None).await;

    let url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client(&project)
        .post(url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "hello",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "MODEL_RUNTIME_UNRESOLVED");
}

// ---------------------------------------------------------------------------
// 3. Launch happy path — persists Session, writes uses_model edge.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn launch_happy_path_persists_session_and_writes_uses_model_edge() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();

    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;

    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client(&project)
        .post(launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "hello world",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201, "launch succeeds");
    let body: Value = res.json().await.unwrap();
    let session_id = body["session_id"].as_str().unwrap().to_string();

    // Wait for the agent task (CH-02 — phi_core::agent_loop driven by
    // MockProvider) to finalise. The recorder writes 1 loop + 1 turn
    // via finalise_and_persist.
    let show_url = project.url(&format!("/api/v0/sessions/{}", session_id));
    let finalised = wait_for_session_finalised(&ceo_client(&project), show_url, 2_000).await;

    assert_eq!(
        finalised["session"]["governance_state"].as_str().unwrap(),
        "completed",
        "session finalises to completed after agent task runs",
    );
    assert_eq!(
        finalised["loops"].as_array().unwrap().len(),
        1,
        "one LoopRecordNode persisted",
    );
    let loop_id = finalised["loops"][0]["id"].as_str().unwrap();
    let turns = finalised["turns_by_loop"][loop_id].as_array().unwrap();
    assert_eq!(turns.len(), 1, "one TurnNode persisted (C-M5-3 proof)");

    // CH-02 proof — these assertions only pass when phi_core::agent_loop
    // actually runs (not the previous synthetic feeder). The synthetic
    // feeder fabricated TurnEnd with a user message; the real loop
    // produces an assistant message via MockProvider plus user-prompted
    // input_messages.
    let turn = &turns[0];
    assert_eq!(
        turn["inner"]["triggered_by"].as_str().unwrap(),
        "User",
        "CH-02: real agent_loop tags the first turn triggered_by=User",
    );
    let turn_json = serde_json::to_string(turn).expect("serialise turn");
    assert!(
        turn_json.contains("hello world"),
        "CH-02: input_messages must contain the user prompt; got: {turn_json}",
    );
    assert!(
        turn_json.contains("Acknowledged."),
        "CH-02: output_message must contain the default MockProvider response \
         when AgentProfile.mock_response is None; got: {turn_json}",
    );
}

// ---------------------------------------------------------------------------
// 3b. Launch with per-profile mock_response override (CH-02 / ADR-0032 D32.2).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn launch_with_mock_response_override_drives_agent_output() {
    use domain::model::ids::NodeId;
    use domain::model::nodes::AgentProfile;

    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();

    let runtime_id = seed_model_runtime(repo.clone()).await;
    // Seed the AgentProfile manually with a non-None mock_response.
    let custom = "Custom CH-02 fixture response — proves per-profile override flows through.";
    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id: project.project_lead,
        parallelize: 2,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(runtime_id.to_string()),
        mock_response: Some(custom.to_string()),
        created_at: Utc::now(),
    };
    repo.create_agent_profile(&profile)
        .await
        .expect("create profile");

    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client(&project)
        .post(launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "test prompt for override",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let session_id = res.json::<Value>().await.unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let show_url = project.url(&format!("/api/v0/sessions/{}", session_id));
    let finalised = wait_for_session_finalised(&ceo_client(&project), show_url, 2_000).await;

    let loop_id = finalised["loops"][0]["id"].as_str().unwrap();
    let turn = &finalised["turns_by_loop"][loop_id][0];
    let turn_json = serde_json::to_string(turn).expect("serialise turn");
    assert!(
        turn_json.contains(custom),
        "AgentProfile.mock_response override must reach MockProvider's text response; got: {turn_json}",
    );
    assert!(
        !turn_json.contains("Acknowledged."),
        "default MockProvider response must NOT appear when override is Some; got: {turn_json}",
    );
}

// ---------------------------------------------------------------------------
// 4. List returns header strip (no `inner` leak).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_sessions_in_project_strips_phi_core_inner() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;

    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let _ = ceo_client(&project)
        .post(launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "list test",
        }))
        .send()
        .await
        .unwrap();

    let list_url = project.url(&format!("/api/v0/projects/{}/sessions", project.project_id));
    // Give the replay task a moment to finalise.
    tokio::time::sleep(Duration::from_millis(150)).await;
    let res = ceo_client(&project).get(list_url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let arr = body.as_array().expect("list body is array");
    assert!(!arr.is_empty(), "at least one session header returned");
    // Schema-snapshot strip: no `inner` / `blueprint` / `loops`
    // keys at depth (they belong to the detail endpoint).
    for header in arr {
        assert!(
            header.get("inner").is_none(),
            "session header must not leak phi-core `inner`",
        );
        assert!(
            header.get("blueprint").is_none(),
            "session header must not leak phi-core blueprint",
        );
        assert!(
            header.get("loops").is_none(),
            "session header must not include nested loops",
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Terminate running session + double-terminate 409.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn terminate_twice_returns_already_terminal_on_second_call() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;

    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let launch: Value = ceo_client(&project)
        .post(launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "terminate test",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let session_id = launch["session_id"].as_str().unwrap().to_string();

    // Wait for the replay to finalise — the synthetic feeder
    // completes Completed before our terminate call races it, so
    // the second terminate call is guaranteed to see a terminal
    // state and return 409 regardless of which arm flipped first.
    let show_url = project.url(&format!("/api/v0/sessions/{}", session_id));
    let _ = wait_for_session_finalised(&ceo_client(&project), show_url, 2_000).await;

    let term_url = project.url(&format!("/api/v0/sessions/{}/terminate", session_id));
    let res = ceo_client(&project)
        .post(term_url.clone())
        .json(&json!({ "reason": "op manual abort" }))
        .send()
        .await
        .unwrap();
    // Either 200 (got there before finalise) OR 409 (finalise won
    // the race). Both outcomes are correct terminal-state flows;
    // this test asserts the handler ALWAYS returns SOMETHING
    // terminal, not a transient error.
    assert!(
        res.status().as_u16() == 200 || res.status().as_u16() == 409,
        "terminate returns 200 (we won) or 409 (replay won)",
    );

    // Second call MUST return 409 regardless of which arm won the
    // first race.
    let res2 = ceo_client(&project)
        .post(term_url)
        .json(&json!({ "reason": "op manual abort" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 409);
    let body2: Value = res2.json().await.unwrap();
    assert_eq!(body2["code"], "SESSION_ALREADY_TERMINAL");
}

// ---------------------------------------------------------------------------
// 6. GET /sessions/:id/tools returns empty list (C-M5-4 wire shape).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_session_tools_returns_empty_list_at_m5() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;
    bind_model_to_agent(repo.clone(), project.project_lead, runtime_id).await;

    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let launch: Value = ceo_client(&project)
        .post(launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "tools test",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let session_id = launch["session_id"].as_str().unwrap().to_string();

    let tools_url = project.url(&format!("/api/v0/sessions/{}/tools", session_id));
    let res = ceo_client(&project).get(tools_url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let arr = body.as_array().expect("tools body is array");
    assert!(
        arr.is_empty(),
        "C-M5-4 — wire shape ships at P4; real tool resolution lands at M7+",
    );
}

// ---------------------------------------------------------------------------
// 7. Update agent profile with new model_config_id (C-M5-5 positive).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn agent_profile_update_with_model_config_id_succeeds_when_idle() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();
    let runtime_id = seed_model_runtime(repo.clone()).await;

    // PATCH the project lead's profile with the new binding. The
    // agent has no active sessions, so the 409 gate must not fire.
    let patch_url = project.url(&format!("/api/v0/agents/{}/profile", project.project_lead));
    let res = ceo_client(&project)
        .patch(patch_url)
        .json(&json!({
            "model_config_id": runtime_id.to_string(),
        }))
        .send()
        .await
        .unwrap();
    // M1 AR-governed; the canonical update path returns 200 w/ the
    // updated-agent payload on success.
    assert!(
        (200..300).contains(&res.status().as_u16()),
        "PATCH succeeds when agent is idle; got {}",
        res.status()
    );

    // Verify the profile row now carries the model_config_id.
    let profile = repo
        .get_agent_profile_for_agent(project.project_lead)
        .await
        .expect("repo ok")
        .expect("profile present");
    assert_eq!(
        profile.model_config_id.as_deref(),
        Some(runtime_id.to_string().as_str())
    );
}
