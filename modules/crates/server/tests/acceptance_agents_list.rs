//! End-to-end acceptance tests for `GET /api/v0/orgs/:org_id/agents`.
//!
//! Commitment C13 in the M4 plan (admin page 08 — agent roster).
//! Scenarios:
//!
//! 1. Fresh-minimal-startup org lists exactly 3 agents (CEO +
//!    memory-extractor + agent-catalog) at their baseline.
//! 2. Role filter — seeding a handful of role-classified agents into
//!    the fixture org returns only the filtered rows.
//! 3. Text-search substring (case-insensitive) narrows results.
//! 4. Combined role + search intersects (AND).
//! 5. Empty `search` param returns 400 `VALIDATION_FAILED`.
//! 6. Unknown org id returns 404 `ORG_NOT_FOUND`.
//! 7. No session cookie returns 401 `UNAUTHENTICATED`.
//! 8. **phi-core invariant** — roster wire payload must not carry
//!    `blueprint` / `execution_limits` / `model_config` / `defaults_snapshot`
//!    keys at any depth.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentRole};
use domain::Repository;
use serde_json::Value;

async fn seed_agent(org: &ClaimedOrg, kind: AgentKind, role: AgentRole, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind,
        display_name: name.into(),
        owning_org: Some(org.org_id),
        role: Some(role),
        created_at: Utc::now(),
    };
    let id = agent.id;
    org.admin
        .acc
        .store
        .create_agent(&agent)
        .await
        .expect("create_agent seed");
    id
}

fn agent_names(body: &Value) -> Vec<String> {
    body["agents"]
        .as_array()
        .expect("agents array")
        .iter()
        .map(|a| {
            a["display_name"]
                .as_str()
                .expect("display_name string")
                .to_string()
        })
        .collect()
}

fn contains_any_blueprint_like_key(v: &Value) -> bool {
    let forbidden = [
        "blueprint",
        "execution_limits",
        "model_config",
        "defaults_snapshot",
        "context_config",
        "retry_config",
    ];
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                if forbidden.contains(&k.as_str()) {
                    return true;
                }
                if contains_any_blueprint_like_key(child) {
                    return true;
                }
            }
            false
        }
        Value::Array(arr) => arr.iter().any(contains_any_blueprint_like_key),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fresh_org_lists_baseline_three_agents() {
    let org = spawn_claimed_with_org(false).await;
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let names = agent_names(&body);
    assert_eq!(names.len(), 3, "baseline has CEO + 2 system agents");
    assert!(names.contains(&"CEO".to_string()));
    assert!(names.contains(&"memory-extractor".to_string()));
    assert!(names.contains(&"agent-catalog".to_string()));
}

#[tokio::test]
async fn role_filter_returns_only_matching_rows() {
    let org = spawn_claimed_with_org(false).await;
    let _i1 = seed_agent(&org, AgentKind::Llm, AgentRole::Intern, "iris-intern").await;
    let _i2 = seed_agent(&org, AgentKind::Llm, AgentRole::Intern, "ivy-intern").await;
    let _c = seed_agent(&org, AgentKind::Llm, AgentRole::Contract, "alpha-contract").await;

    let url = org.url(&format!("/api/v0/orgs/{}/agents?role=intern", org.org_id));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let names = agent_names(&body);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"iris-intern".to_string()));
    assert!(names.contains(&"ivy-intern".to_string()));
    assert!(!names.contains(&"alpha-contract".to_string()));
}

#[tokio::test]
async fn text_search_is_case_insensitive_substring() {
    let org = spawn_claimed_with_org(false).await;
    seed_agent(&org, AgentKind::Llm, AgentRole::Intern, "iris-bot").await;
    seed_agent(&org, AgentKind::Llm, AgentRole::Contract, "alpha-bot").await;
    seed_agent(&org, AgentKind::Human, AgentRole::Member, "Alice").await;

    let url = org.url(&format!("/api/v0/orgs/{}/agents?search=BOT", org.org_id));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let names = agent_names(&body);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"iris-bot".to_string()));
    assert!(names.contains(&"alpha-bot".to_string()));
}

#[tokio::test]
async fn combined_role_and_search_intersects() {
    let org = spawn_claimed_with_org(false).await;
    seed_agent(&org, AgentKind::Llm, AgentRole::Intern, "iris-bot").await;
    seed_agent(&org, AgentKind::Llm, AgentRole::Contract, "alpha-bot").await;
    seed_agent(&org, AgentKind::Llm, AgentRole::Contract, "beta-gadget").await;

    let url = org.url(&format!(
        "/api/v0/orgs/{}/agents?role=contract&search=bot",
        org.org_id
    ));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let names = agent_names(&body);
    assert_eq!(names, vec!["alpha-bot".to_string()]);
}

#[tokio::test]
async fn empty_search_is_validation_error() {
    let org = spawn_claimed_with_org(false).await;
    let url = org.url(&format!(
        "/api/v0/orgs/{}/agents?search=%20%20%20",
        org.org_id
    ));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("VALIDATION_FAILED"));
}

#[tokio::test]
async fn unknown_org_returns_404() {
    let org = spawn_claimed_with_org(false).await;
    let bogus = OrgId::new();
    let url = org.url(&format!("/api/v0/orgs/{}/agents", bogus));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("ORG_NOT_FOUND"));
}

#[tokio::test]
async fn unauthenticated_returns_401() {
    let org = spawn_claimed_with_org(false).await;
    let anon = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = anon.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 401);
}

#[tokio::test]
async fn response_carries_no_phi_core_fields_at_any_depth() {
    let org = spawn_claimed_with_org(false).await;
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = org.admin.authed_client.get(&url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(
        !contains_any_blueprint_like_key(&body),
        "roster payload leaked a phi-core-wrapped key: {body}"
    );
}
