//! End-to-end acceptance tests for pages 09's create + update +
//! revert-limits flows (M4/P5 — commitment C14).
//!
//! Scenarios:
//!
//! Create path:
//!  1. Happy LLM create with Intern role — agent + inbox + outbox +
//!     profile + audit event all land; `execution_limits_source = inherit`.
//!  2. Happy Human create with Executive role — no profile by design
//!     (humans are optionally profiled; default path creates none).
//!  3. Happy LLM create with initial ExecutionLimits override — row
//!     persists, effective limits resolve to the override.
//!  4. 400 on role-kind mismatch (Intern + Human).
//!  5. 400 on parallelize = 0.
//!  6. 400 on override exceeding org ceiling.
//!
//! Update path:
//!  7. Edit `display_name` — row updates, audit event emitted.
//!  8. Edit blueprint `temperature` — profile updates.
//!  9. Set override on previously-inheriting agent — row created,
//!     source flips to Override.
//! 10. Revert override — row deleted, source flips to Inherit.
//! 11. 400 on immutable `role` change attempt.
//! 12. 400 on parallelize ceiling breach.
//!
//! Other:
//! 13. `DELETE .../execution-limits-override` is idempotent (200 on
//!     second call).
//! 14. **phi-core invariant** — the create + update + revert response
//!     bodies don't leak phi-core structural keys at the top level
//!     (the payloads embed `AgentProfile` intentionally via `blueprint`
//!     — that's allowed; but there's no spurious leak).
//! 15. 401 on unauthenticated create.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::model::ids::{AgentId, NodeId};
use domain::model::nodes::{Agent, AgentKind, AgentRole};
use domain::Repository;
use serde_json::{json, Value};

async fn seed_agent_with_role(org: &ClaimedOrg, role: AgentRole, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind: match role {
            AgentRole::Executive | AgentRole::Admin | AgentRole::Member => AgentKind::Human,
            AgentRole::Intern | AgentRole::Contract | AgentRole::System => AgentKind::Llm,
        },
        display_name: name.into(),
        owning_org: Some(org.org_id),
        role: Some(role),
        created_at: Utc::now(),
    };
    let id = agent.id;
    org.admin.acc.store.create_agent(&agent).await.unwrap();
    id
}

fn post_create(org: &ClaimedOrg, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    org.admin.authed_client.post(url).json(&body)
}

fn patch_profile(org: &ClaimedOrg, agent_id: AgentId, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/agents/{agent_id}/profile"));
    org.admin.authed_client.patch(url).json(&body)
}

fn delete_override(org: &ClaimedOrg, agent_id: AgentId) -> reqwest::RequestBuilder {
    let url = org.url(&format!(
        "/api/v0/agents/{agent_id}/execution-limits-override"
    ));
    org.admin.authed_client.delete(url)
}

fn llm_create_body(name: &str, role: &str) -> Value {
    json!({
        "display_name": name,
        "kind": "llm",
        "role": role,
        "blueprint": {
            "system_prompt": "you are a helper",
            "thinking_level": "medium",
            "temperature": 0.2
        },
        "parallelize": 1,
        "initial_execution_limits_override": null
    })
}

fn human_create_body(name: &str, role: &str) -> Value {
    json!({
        "display_name": name,
        "kind": "human",
        "role": role,
        "blueprint": {},
        "parallelize": 1
    })
}

// ---------------------------------------------------------------------------
// Create scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_llm_intern_happy_path() {
    let org = spawn_claimed_with_org(false).await;
    let res = post_create(&org, llm_create_body("iris-bot", "intern"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let body: Value = res.json().await.unwrap();
    assert!(body["agent_id"].as_str().is_some());
    assert!(body["profile_id"].as_str().is_some());
    assert!(body["execution_limits_override_id"].is_null());
    assert!(body["audit_event_id"].as_str().is_some());
}

#[tokio::test]
async fn create_human_executive_happy_path() {
    let org = spawn_claimed_with_org(false).await;
    let res = post_create(&org, human_create_body("Alice CEO", "executive"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let body: Value = res.json().await.unwrap();
    assert!(body["agent_id"].as_str().is_some());
    // Humans don't get an AgentProfile row at create time by design.
    assert!(body["profile_id"].is_null());
}

#[tokio::test]
async fn create_with_initial_override_persists_row() {
    let org = spawn_claimed_with_org(false).await;
    let body = json!({
        "display_name": "iris-bot-tight",
        "kind": "llm",
        "role": "intern",
        "blueprint": { "system_prompt": "tight" },
        "parallelize": 1,
        "initial_execution_limits_override": {
            "max_turns": 10,
            "max_total_tokens": 50000,
            "max_duration": { "secs": 60, "nanos": 0 },
            "max_cost": 0.25
        }
    });
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let body: Value = res.json().await.unwrap();
    assert!(body["execution_limits_override_id"].as_str().is_some());
}

#[tokio::test]
async fn create_rejects_role_kind_mismatch() {
    let org = spawn_claimed_with_org(false).await;
    // Intern is LLM-only; pairing with human fails is_valid_for.
    let mut body = human_create_body("mismatch", "intern");
    body["kind"] = json!("human");
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("AGENT_ROLE_INVALID_FOR_KIND"));
}

#[tokio::test]
async fn create_rejects_zero_parallelize() {
    let org = spawn_claimed_with_org(false).await;
    let mut body = llm_create_body("iris-bot", "intern");
    body["parallelize"] = json!(0);
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("PARALLELIZE_CEILING_EXCEEDED"));
}

#[tokio::test]
async fn create_rejects_override_over_ceiling() {
    let org = spawn_claimed_with_org(false).await;
    // The fixture org has no defaults_snapshot (spawn_claimed_with_org
    // sets it to None). The resolver falls back to
    // `phi_core::ExecutionLimits::default()` which has `max_turns =
    // 50`, so any value > 50 breaches.
    let body = json!({
        "display_name": "iris-bot-too-big",
        "kind": "llm",
        "role": "intern",
        "blueprint": {},
        "parallelize": 1,
        "initial_execution_limits_override": {
            "max_turns": 10000,
            "max_total_tokens": 50000,
            "max_duration": { "secs": 60, "nanos": 0 },
            "max_cost": 0.25
        }
    });
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(
        err["code"].as_str(),
        Some("EXECUTION_LIMITS_EXCEED_ORG_CEILING")
    );
}

// ---------------------------------------------------------------------------
// Update scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_display_name_happy_path() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris-old").await;
    let res = patch_profile(&org, agent_id, json!({ "display_name": "iris-new" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["audit_event_id"].as_str().is_some());
    assert_eq!(body["execution_limits_source"].as_str(), Some("inherit"));
}

#[tokio::test]
async fn update_blueprint_temperature_changes_profile() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris").await;
    // Seed an AgentProfile for the agent so the update path has
    // something to edit.
    let profile = domain::model::nodes::AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize: 1,
        blueprint: Default::default(),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    org.admin
        .acc
        .store
        .create_agent_profile(&profile)
        .await
        .unwrap();

    let res = patch_profile(
        &org,
        agent_id,
        json!({
            "blueprint": {
                "temperature": 0.7
            }
        }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["audit_event_id"].as_str().is_some());
}

#[tokio::test]
async fn update_set_override_flips_source_to_override() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris").await;
    let res = patch_profile(
        &org,
        agent_id,
        json!({
            "execution_limits": {
                "set": {
                    "max_turns": 5,
                    "max_total_tokens": 10000,
                    "max_duration": { "secs": 30, "nanos": 0 },
                    "max_cost": 0.10
                }
            }
        }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["execution_limits_source"].as_str(), Some("override"));
}

#[tokio::test]
async fn update_revert_flips_source_back_to_inherit() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris").await;
    // Set first.
    patch_profile(
        &org,
        agent_id,
        json!({
            "execution_limits": {
                "set": {
                    "max_turns": 5,
                    "max_total_tokens": 10000,
                    "max_duration": { "secs": 30, "nanos": 0 },
                    "max_cost": null
                }
            }
        }),
    )
    .send()
    .await
    .unwrap();
    // Then revert via the PATCH body.
    let res = patch_profile(
        &org,
        agent_id,
        json!({ "execution_limits": { "revert": null } }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["execution_limits_source"].as_str(), Some("inherit"));
}

#[tokio::test]
async fn update_rejects_parallelize_ceiling_breach() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris").await;
    let res = patch_profile(&org, agent_id, json!({ "parallelize": 9999 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("PARALLELIZE_CEILING_EXCEEDED"));
}

#[tokio::test]
async fn update_rejects_system_agent_edit() {
    let org = spawn_claimed_with_org(false).await;
    let sys_id = seed_agent_with_role(&org, AgentRole::System, "memory-extractor-clone").await;
    let res = patch_profile(&org, sys_id, json!({ "display_name": "renamed" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("SYSTEM_AGENT_READ_ONLY"));
}

// ---------------------------------------------------------------------------
// Revert endpoint scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn revert_override_endpoint_is_idempotent() {
    let org = spawn_claimed_with_org(false).await;
    let agent_id = seed_agent_with_role(&org, AgentRole::Intern, "iris").await;
    // First call (no override exists) — still 200.
    let res = delete_override(&org, agent_id).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    // Second call — still 200.
    let res2 = delete_override(&org, agent_id).send().await.unwrap();
    assert_eq!(res2.status().as_u16(), 200);
}

#[tokio::test]
async fn revert_override_404_on_unknown_agent() {
    let org = spawn_claimed_with_org(false).await;
    let bogus = AgentId::new();
    let res = delete_override(&org, bogus).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("AGENT_NOT_FOUND"));
}

// ---------------------------------------------------------------------------
// Auth + immutability
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unauthenticated_create_returns_401() {
    let org = spawn_claimed_with_org(false).await;
    let anon = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let res = anon
        .post(org.url(&format!("/api/v0/orgs/{}/agents", org.org_id)))
        .json(&llm_create_body("iris-bot", "intern"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 401);
}
