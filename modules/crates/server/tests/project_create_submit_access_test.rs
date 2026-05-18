//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for Shape B project creation
//! (`projects/create.rs:472`).
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor at `build_shape_b_auth_request`), preserving the existing
//! Shape B happy-path of submitting a 2-slot AR.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, OrgId, ProjectId};
use domain::model::nodes::{Agent, AgentKind, Organization};
use domain::Repository;
use serde_json::{json, Value};
use std::sync::Arc;

fn post_create(org: &ClaimedOrg, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/orgs/{}/projects", org.org_id));
    org.admin.authed_client.post(url).json(&body)
}

async fn seed_llm(store: &Arc<dyn Repository>, org: OrgId, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: name.into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let id = agent.id;
    store.create_agent(&agent).await.unwrap();
    id
}

async fn seed_human(store: &Arc<dyn Repository>, org: OrgId, name: &str) -> AgentId {
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: name.into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    };
    let id = agent.id;
    store.create_agent(&agent).await.unwrap();
    id
}

async fn seed_second_org(store: &Arc<dyn Repository>) -> (OrgId, AgentId) {
    let id = OrgId::new();
    let org = Organization {
        id,
        display_name: "Co-Owner Org".into(),
        vision: None,
        mission: None,
        consent_policy: domain::model::composites_m3::ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        approval_timeout: domain::model::ApprovalTimeout::ProjectDuration,
        approval_timeout_default_response: domain::model::TimeoutResponse::Deny,
        tags: vec![format!("organization:{}", id)],
        created_at: Utc::now(),
    };
    store.create_organization(&org).await.unwrap();
    let ceo = seed_human(store, id, "CoOwnerCEO").await;
    (id, ceo)
}

#[tokio::test]
async fn shape_b_project_submit_passes_synthetic_draft_submit_check() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, _co_ceo) = seed_second_org(&store).await;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;

    let body = json!({
        "project_id": ProjectId::new().to_string(),
        "name": "ch18-shape-b-submit",
        "description": "ch18 submit-access happy path",
        "goal": null,
        "shape": "shape_b",
        "co_owner_org_id": co_owner.to_string(),
        "lead_agent_id": lead.to_string(),
        "member_agent_ids": [],
        "sponsor_agent_ids": [],
        "token_budget": null,
        "objectives": [],
        "key_results": [],
    });
    let r = post_create(&org, body).send().await.unwrap();
    // Shape B is a pending 2-slot AR — handler returns 202 Accepted with
    // a `pending_ar_id`.
    assert_eq!(r.status().as_u16(), 202);
    let body: Value = r.json().await.unwrap();
    assert!(body["pending_ar_id"].as_str().is_some());
}
