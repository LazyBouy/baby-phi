//! CH-18 / ADR-0056 §D56.5 — slot-fill read access gate fires when a
//! non-slot-approver-non-requestor agent tries to drive a Shape B
//! pending AR through the `approve_pending_shape_b` endpoint. The READ
//! gate (`projects/create.rs:638`) catches the unauthorised reader
//! BEFORE the existing `locate_slot` gate at line 696.
//!
//! Asserts:
//!  - HTTP returns 403 + `AR_ACCESS_DENIED`.
//!  - An `auth_request.access_denied` Alerted audit event is persisted
//!    against the AR with `intended_op == "read"`.

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

fn post_approve(org: &ClaimedOrg, ar_id: &str, body: Value) -> reqwest::RequestBuilder {
    let url = org.url(&format!("/api/v0/projects/_pending/{ar_id}/approve"));
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

#[tokio::test]
async fn approve_pending_shape_b_by_non_slot_non_requestor_returns_access_denied_on_read() {
    // Build a 2-slot Shape B project-creation AR. A non-slot-approver-
    // non-requestor agent tries to drive the slot-fill via the
    // `/_pending/{ar_id}/approve` endpoint. The new READ gate at
    // `projects/create.rs:638` fires BEFORE `locate_slot` and emits an
    // `auth_request.access_denied` Alerted event with `intended_op == "read"`.
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, _co_ceo) = seed_second_org(&store).await;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;

    // Submit Shape B FIRST — slots are (primary CEO, co-owner CEO).
    // Adding extra Humans in the primary org BEFORE submit might make
    // them the "first Human" via iteration order; we seed stranger in a
    // THIRD fresh org to guarantee non-slot non-requestor classification.
    let submit: Value = post_create(
        &org,
        shape_b_body("AccessReadTest", co_owner, lead, ProjectId::new()),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    let ar_id = submit["pending_ar_id"].as_str().unwrap().to_string();

    // A stranger agent in a THIRD org — guaranteed not the requestor
    // (org.admin), not the primary-org CEO (slot 0), not the co-owner
    // CEO (slot 1).
    let (_third_org, stranger) = seed_second_org(&store).await;

    // Stranger tries to call `_pending/{ar_id}/approve` — the slot-fill
    // READ gate catches this BEFORE the `locate_slot` gate.
    let res = post_approve(
        &org,
        &ar_id,
        json!({ "approver_id": stranger.to_string(), "approve": true }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "AR_ACCESS_DENIED");

    // Audit event with intended_op = "read" landed.
    let events = store
        .list_recent_audit_events_for_org(org.org_id, 50)
        .await
        .unwrap();
    let evt = events
        .iter()
        .find(|e| {
            e.event_type == "auth_request.access_denied" && e.diff["after"]["intended_op"] == "read"
        })
        .expect("auth_request.access_denied with intended_op=read should be persisted");
    assert_eq!(evt.audit_class, AuditClass::Alerted);
    // Provenance points at the AR.
    let ar_uuid = uuid::Uuid::parse_str(&ar_id).unwrap();
    assert_eq!(
        evt.provenance_auth_request_id
            .as_ref()
            .map(|id| id.as_uuid().to_string()),
        Some(ar_uuid.to_string())
    );
}
