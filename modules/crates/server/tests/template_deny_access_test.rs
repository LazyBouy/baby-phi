//! CH-18 / ADR-0056 §D56.6 — per-state access-matrix gate fires on the
//! `templates/deny` mutation when the caller is not authorised for the
//! (state × Deny) cell. Asserts:
//!  - HTTP returns 403 + `AR_ACCESS_DENIED`.
//!  - An `auth_request.access_denied` Alerted audit event is persisted
//!    against the org's audit log with the AR id as `provenance_auth_request_id`.

mod acceptance_common;

use acceptance_common::admin::{authed_client_for, spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, AuthRequestId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef,
    ResourceRef, ResourceSlot, ResourceSlotState, TemplateKind,
};
use domain::Repository;
use serde_json::{json, Value};
use std::sync::Arc;

fn action_url(org: &ClaimedOrg, kind: &str, action: &str) -> String {
    org.url(&format!(
        "/api/v0/orgs/{}/authority-templates/{}/{}",
        org.org_id, kind, action
    ))
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

async fn seed_pending_adoption_ar(
    store: &Arc<dyn Repository>,
    org: OrgId,
    kind: TemplateKind,
    requestor_ceo: AgentId,
) -> AuthRequestId {
    let ar = AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::Agent(requestor_ceo),
        kinds: vec![
            format!("#template:{}", kind.as_str()),
            "#kind:control_plane".to_string(),
        ],
        scope: vec![format!("org:{}", org)],
        state: AuthRequestState::Pending,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: format!("org:{}/template:{}", org, kind.as_str()),
            },
            approvers: vec![ApproverSlot {
                approver: PrincipalRef::Agent(requestor_ceo),
                state: ApproverSlotState::Unfilled,
                responded_at: None,
                reconsidered_at: None,
            }],
            state: ResourceSlotState::InProgress,
        }],
        justification: Some("seed: pending adoption AR for access-test".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
        tags: vec![],
        descends_from_grant: None,
    };
    let ar_id = ar.id;
    store.create_auth_request(&ar).await.unwrap();
    ar_id
}

#[tokio::test]
async fn deny_by_non_authorised_principal_returns_access_denied_and_emits_audit_event() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    let stranger = seed_human(&store, org.org_id, "stranger").await;

    let ar_id =
        seed_pending_adoption_ar(&store, org.org_id, TemplateKind::C, org.ceo_agent_id).await;

    let stranger_client = authed_client_for(&org.admin, stranger).expect("mint stranger session");
    let res = stranger_client
        .post(action_url(&org, "c", "deny"))
        .json(&json!({ "reason": "stranger says no" }))
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
    assert_eq!(evt.provenance_auth_request_id, Some(ar_id));
    assert_eq!(evt.actor_agent_id, Some(stranger));
    assert_eq!(evt.diff["after"]["intended_op"], "deny");
    assert_eq!(evt.diff["after"]["error_kind"], "not_authorised_for_modify");

    // Sanity: existing pre-CH-18 happy-path remains green — the CEO can
    // still deny the AR (or any matrix-permitted reach). Use the
    // pre-existing fixture's `deny_of_unseeded_c_is_not_found_until_adopted`
    // pattern: verify the deny endpoint still returns the standard
    // not-found path on a non-existent (different kind) AR.
    let ceo = authed_client_for(&org.admin, org.ceo_agent_id).expect("mint CEO session");
    let res2 = ceo
        .post(action_url(&org, "d", "deny"))
        .json(&json!({ "reason": "no D AR exists yet" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 404);
    let body2: Value = res2.json().await.unwrap();
    assert_eq!(body2["code"], "TEMPLATE_ADOPTION_NOT_FOUND");
}
