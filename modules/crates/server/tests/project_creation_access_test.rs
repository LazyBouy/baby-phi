//! CH-18 / ADR-0056 §D56.6 — per-state access-matrix gate fires on the
//! `projects/create::approve_pending_shape_b` mutation when the slot
//! approver already filled their slot. Asserts:
//!  - HTTP returns 403 + `AR_ACCESS_DENIED`.
//!  - An `auth_request.access_denied` Alerted audit event is persisted
//!    against the org's audit log with the AR id as
//!    `provenance_auth_request_id`.
//!
//! The legacy 403 path (`APPROVER_NOT_AUTHORIZED`) handled non-slot
//! agents — that gate sits BEFORE the access check at line 649. The
//! NEW access-matrix gate enforces the unfilled-slot precondition that
//! the (InProgress, Approve) matrix cell requires (concept doc 02 row
//! 3 column 4 — "approve own UNFILLED slot").

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
async fn approve_pending_shape_b_by_already_filled_slot_returns_access_denied_and_emits_audit_event(
) {
    // Build a 2-slot Shape B project-creation AR. First approver fills
    // their slot (the AR moves Pending → InProgress while the second
    // slot is still unfilled). When the SAME approver tries to approve
    // again, `locate_slot` finds their slot, but the per-state access
    // matrix `(InProgress, Approve)` arm sees `SlotApprover { has_unfilled_slot:
    // false, has_filled_slot: true }` → `UnfilledApproverSlotOnly`.
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();
    let (co_owner, _co_ceo) = seed_second_org(&store).await;
    let lead = seed_llm(&store, org.org_id, "lead-llm").await;
    let primary_ceo = org.ceo_agent_id;

    // Submit Shape B.
    let submit: Value = post_create(
        &org,
        shape_b_body("AccessTest", co_owner, lead, ProjectId::new()),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    let ar_id = submit["pending_ar_id"].as_str().unwrap().to_string();

    // First approver — primary org CEO — fills their slot with Approve.
    // The AR moves to InProgress (one slot filled, one unfilled).
    let first: Value = post_approve(
        &org,
        &ar_id,
        json!({ "approver_id": primary_ceo.to_string(), "approve": true }),
    )
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(first["outcome"].as_str(), Some("still_pending"));

    // Now the same approver tries to approve AGAIN. The legacy
    // `locate_slot` gate finds the slot; the new access-matrix gate
    // catches the already-filled-slot state.
    let res = post_approve(
        &org,
        &ar_id,
        json!({ "approver_id": primary_ceo.to_string(), "approve": true }),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "AR_ACCESS_DENIED");

    // Audit event landed.
    let events = store
        .list_recent_audit_events_for_org(org.org_id, 50)
        .await
        .unwrap();
    let evt = events
        .iter()
        .find(|e| e.event_type == "auth_request.access_denied")
        .expect("auth_request.access_denied event should be persisted");
    assert_eq!(evt.audit_class, AuditClass::Alerted);
    assert_eq!(evt.diff["after"]["intended_op"], "approve");
    // The matrix's UnfilledApproverSlotOnly arm fires for this case.
    assert_eq!(
        evt.diff["after"]["error_kind"],
        "unfilled_approver_slot_only"
    );
    // Provenance must point at the AR.
    let ar_uuid = uuid::Uuid::parse_str(&ar_id).unwrap();
    assert_eq!(
        evt.provenance_auth_request_id
            .as_ref()
            .map(|id| id.as_uuid().to_string()),
        Some(ar_uuid.to_string())
    );
}
