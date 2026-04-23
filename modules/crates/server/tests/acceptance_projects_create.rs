//! End-to-end acceptance tests for the M4/P6 project-creation surface —
//! `POST /api/v0/orgs/:org_id/projects` plus
//! `POST /api/v0/projects/_pending/:ar_id/approve` (commitment C15).
//!
//! Scenarios (11):
//!
//! Shape A:
//!  1. Happy path — project materialises; response carries
//!     `outcome: "materialised"` + a fresh `has_lead_edge_id`.
//!  2. Validation — empty name returns 400.
//!  3. Lead-not-in-owning-org returns 400.
//!  4. OKR shape mismatch returns 400.
//!  5. 409 on duplicate project id.
//!
//! Shape B:
//!  6. Submit happy path — returns 202 + pending AR id + two approver ids.
//!  7. Both-approve terminal — second approval flips AR to `approved`.
//!  8. Both-deny terminal — AR → `denied`.
//!  9. Mixed A/D terminal — AR → `partial`, audit emitted.
//! 10. Mixed D/A terminal — AR → `partial`.
//!
//! Other:
//! 11. 403 on approve by an agent not listed as an approver slot.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::model::ids::{AgentId, OrgId, ProjectId};
use domain::model::nodes::{Agent, AgentKind, Organization};
use domain::Repository;
use serde_json::{json, Value};
use std::sync::Arc;

fn post_create(org: &ClaimedOrg, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/orgs/{}/projects", org.org_id));
    org.admin.authed_client.post(url).json(&body)
}

fn post_approve(org: &ClaimedOrg, ar_id: &str, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/projects/_pending/{ar_id}/approve"));
    org.admin.authed_client.post(url).json(&body)
}

/// Seed a fresh LLM agent owned by `org` — used as a lead.
async fn seed_llm(store: &Arc<dyn Repository>, org: OrgId, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: name.into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
    };
    let id = agent.id;
    store.create_agent(&agent).await.unwrap();
    id
}

/// Seed a fresh Human agent owned by `org` — used as a sponsor or
/// Shape B approver.
async fn seed_human(store: &Arc<dyn Repository>, org: OrgId, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: name.into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
    };
    let id = agent.id;
    store.create_agent(&agent).await.unwrap();
    id
}

/// Seed a second fully-shaped org alongside the fixture org (Shape B
/// needs two). Includes a CEO-like Human agent so
/// `first_human_agent_in_org` returns a real id.
async fn seed_second_org(
    store: &Arc<dyn Repository>,
    admin_acc_store: &Arc<dyn Repository>,
) -> (OrgId, AgentId) {
    let id = OrgId::new();
    let org = Organization {
        id,
        display_name: "Co-Owner Org".into(),
        vision: None,
        mission: None,
        consent_policy: domain::model::composites_m3::ConsentPolicy::Implicit,
        audit_class_default: domain::audit::AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        created_at: Utc::now(),
    };
    store.create_organization(&org).await.unwrap();
    admin_acc_store.create_organization(&org).await.ok(); // same handle; idempotent insertion through the second ref
    let ceo = seed_human(store, id, "CoOwnerCEO").await;
    (id, ceo)
}

fn shape_a_body(name: &str, lead: AgentId, project_id: ProjectId) -> Value {
    json!({
        "project_id": project_id.to_string(),
        "name": name,
        "description": "seed",
        "goal": "ship",
        "shape": "shape_a",
        "co_owner_org_id": null,
        "lead_agent_id": lead.to_string(),
        "member_agent_ids": [],
        "sponsor_agent_ids": [],
        "token_budget": null,
        "objectives": [],
        "key_results": [],
    })
}

fn shape_b_body(name: &str, co_owner: OrgId, lead: AgentId, project_id: ProjectId) -> Value {
    json!({
        "project_id": project_id.to_string(),
        "name": name,
        "description": "co-owned project",
        "goal": null,
        "shape": "shape_b",
        "co_owner_org_id": co_owner.to_string(),
        "lead_agent_id": lead.to_string(),
        "member_agent_ids": [],
        "sponsor_agent_ids": [],
        "token_budget": null,
        "objectives": [],
        "key_results": [],
    })
}

// ---------------------------------------------------------------------------
// Shape A scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shape_a_happy_path() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let body = shape_a_body("Atlas", lead, ProjectId::new());
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["outcome"].as_str(), Some("materialised"));
    assert!(body["project_id"].as_str().is_some());
    assert!(body["has_lead_edge_id"].as_str().is_some());
    assert!(body["audit_event_id"].as_str().is_some());
}

#[tokio::test]
async fn shape_a_empty_name_rejected() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let mut body = shape_a_body("Atlas", lead, ProjectId::new());
    body["name"] = json!("");
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("VALIDATION_FAILED"));
}

#[tokio::test]
async fn shape_a_lead_not_in_owning_org_rejected() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    // Stand up an unrelated org + put the "lead" in it.
    let (other_org, _) = seed_second_org(&store, &store).await;
    let lead_in_other_org = seed_llm(&store, other_org, "wrong-org-lead").await;
    let body = shape_a_body("Atlas", lead_in_other_org, ProjectId::new());
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("LEAD_NOT_IN_OWNING_ORG"));
}

#[tokio::test]
async fn shape_a_okr_shape_mismatch_rejected() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    // `measurement_type=count` with a Bool target_value — shape mismatch.
    let mut body = shape_a_body("Atlas", lead, ProjectId::new());
    body["objectives"] = json!([{
        "objective_id": "obj-1",
        "name": "goal",
        "description": "",
        "status": "draft",
        "owner": AgentId::new().to_string()
    }]);
    body["key_results"] = json!([{
        "kr_id": "kr-1",
        "objective_id": "obj-1",
        "name": "kr",
        "description": "",
        "measurement_type": "count",
        "target_value": { "kind": "bool", "value": true },
        "owner": AgentId::new().to_string(),
        "status": "not_started"
    }]);
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("OKR_VALIDATION_FAILED"));
}

#[tokio::test]
async fn shape_a_duplicate_project_id_rejected() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let pid = ProjectId::new();
    post_create(&org, shape_a_body("Atlas", lead, pid))
        .send()
        .await
        .unwrap();
    // Second submit with the same project id.
    let res = post_create(&org, shape_a_body("Atlas-2", lead, pid))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 409);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("PROJECT_ID_IN_USE"));
}

// ---------------------------------------------------------------------------
// Shape B scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shape_b_submit_happy_path() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, _co_ceo) = seed_second_org(&store, &store).await;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let body = shape_b_body("CoAtlas", co_owner, lead, ProjectId::new());
    let res = post_create(&org, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 202);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["outcome"].as_str(), Some("pending"));
    assert!(body["pending_ar_id"].as_str().is_some());
    let approvers = body["approver_ids"].as_array().unwrap();
    assert_eq!(approvers.len(), 2);
}

/// Drive both slots through the approval matrix. Returns the final
/// terminal AR state string after the second decision.
async fn drive_shape_b_matrix(
    org: &ClaimedOrg,
    approver_a_decision: bool,
    approver_b_decision: bool,
) -> String {
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, co_ceo) = seed_second_org(&store, &store).await;
    // The primary org's CEO is spawn_claimed_with_org's ceo_agent_id.
    let primary_ceo = org.ceo_agent_id;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let body = shape_b_body("Matrix", co_owner, lead, ProjectId::new());
    let submit: Value = post_create(org, body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ar_id = submit["pending_ar_id"].as_str().unwrap().to_string();

    // First approver — the primary org's CEO.
    let first: Value = post_approve(
        org,
        &ar_id,
        json!({ "approver_id": primary_ceo.to_string(), "approve": approver_a_decision }),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(first["outcome"].as_str(), Some("still_pending"));

    // Second approver — the co-owner org's CEO.
    let second: Value = post_approve(
        org,
        &ar_id,
        json!({ "approver_id": co_ceo.to_string(), "approve": approver_b_decision }),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(second["outcome"].as_str(), Some("terminal"));
    second["state"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn shape_b_both_approve_terminal_state_is_approved() {
    let org = spawn_claimed_with_org(false).await;
    let state = drive_shape_b_matrix(&org, true, true).await;
    assert_eq!(state, "approved");
    // NOTE: at M4 project_id is null per C-M5-6 deferral. The state
    // machine invariant is what this test pins.
}

#[tokio::test]
async fn shape_b_both_deny_terminal_state_is_denied() {
    let org = spawn_claimed_with_org(false).await;
    let state = drive_shape_b_matrix(&org, false, false).await;
    assert_eq!(state, "denied");
}

#[tokio::test]
async fn shape_b_approve_then_deny_terminal_state_is_partial() {
    let org = spawn_claimed_with_org(false).await;
    let state = drive_shape_b_matrix(&org, true, false).await;
    assert_eq!(state, "partial");
}

#[tokio::test]
async fn shape_b_deny_then_approve_terminal_state_is_partial() {
    let org = spawn_claimed_with_org(false).await;
    let state = drive_shape_b_matrix(&org, false, true).await;
    assert_eq!(state, "partial");
}

// ---------------------------------------------------------------------------
// Other
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shape_b_approve_by_non_slot_agent_is_403() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, _co_ceo) = seed_second_org(&store, &store).await;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let submit: Value = post_create(
        &org,
        shape_b_body("Matrix", co_owner, lead, ProjectId::new()),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    let ar_id = submit["pending_ar_id"].as_str().unwrap();
    // The lead-llm is an LLM agent, not a slot approver; the AR was
    // minted with the two CEO Human agents. Approval attempt must 403.
    let res = post_approve(
        &org,
        ar_id,
        json!({ "approver_id": lead.to_string(), "approve": true }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("APPROVER_NOT_AUTHORIZED"));
}
