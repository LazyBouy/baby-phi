//! CH-18 / ADR-0056 §D56.6 — per-state access-matrix gate fires on the
//! `templates/approve` mutation when the caller is not authorised for the
//! (state × Approve) cell. Asserts:
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
use serde_json::Value;
use std::sync::Arc;

fn action_url(org: &ClaimedOrg, kind: &str, action: &str) -> String {
    org.url(&format!(
        "/api/v0/orgs/{}/authority-templates/{}/{}",
        org.org_id, kind, action
    ))
}

/// Seed a fresh Human agent owned by `org` — used as a non-CEO actor.
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

/// Seed a Pending adoption AR for `kind` whose requestor + sole approver
/// is the given CEO. Mirrors the Template-E shape (one resource, one
/// approver) with `state = Pending` so the approve handler does not
/// short-circuit on `AdoptionAlreadyActive`.
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
async fn approve_by_non_authorised_principal_returns_access_denied_and_emits_audit_event() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // Seed a fresh Human agent in the org who is NOT the CEO and not on
    // any approver slot — classified as `Other` by the matrix.
    let stranger = seed_human(&store, org.org_id, "stranger").await;

    // Seed a Pending adoption AR for Template C whose requestor + sole
    // approver is the CEO. Approve from the stranger must fail the
    // access matrix `(Pending, Approve)` arm with NotAuthorisedForModify.
    let ar_id =
        seed_pending_adoption_ar(&store, org.org_id, TemplateKind::C, org.ceo_agent_id).await;

    let stranger_client = authed_client_for(&org.admin, stranger).expect("mint stranger session");
    let res = stranger_client
        .post(action_url(&org, "c", "approve"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "AR_ACCESS_DENIED");

    // Audit-event sanity: an Alerted `auth_request.access_denied` row
    // landed in the org's audit log, citing the seeded AR.
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
    assert_eq!(evt.diff["after"]["intended_op"], "approve");
    // The non-CEO classifies as Other → NotAuthorisedForModify variant.
    assert_eq!(evt.diff["after"]["error_kind"], "not_authorised_for_modify");

    // Negative-side invariant: the failed approve must NOT have mutated
    // the AR's state — the access check is upstream of `transition_slot`,
    // and the audit event is the ONLY side effect. Verify by re-fetching
    // the AR.
    let ar = store
        .get_auth_request(ar_id)
        .await
        .unwrap()
        .expect("seeded AR is still present");
    assert_eq!(
        ar.state,
        AuthRequestState::Pending,
        "AR state must be unchanged after failed access check"
    );
}
