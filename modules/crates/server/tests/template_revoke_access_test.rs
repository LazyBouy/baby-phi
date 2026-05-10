//! CH-18 / ADR-0056 §D56.6 — per-state access-matrix gate fires on the
//! `templates/revoke` mutation when the caller is not authorised for the
//! (Approved × Revoke) cell. At v2, F3.B.role.b descopes Revoke to
//! `ar.requestor == principal` (adoption-AR self-revoke stub); other
//! principals receive `NotAuthorisedForModify`.
//!
//! Asserts:
//!  - HTTP returns 403 + `AR_ACCESS_DENIED`.
//!  - An `auth_request.access_denied` Alerted audit event is persisted
//!    against the org's audit log.

mod acceptance_common;

use acceptance_common::admin::{authed_client_for, spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind};
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

#[tokio::test]
async fn revoke_by_non_requestor_returns_access_denied_and_emits_audit_event() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // Org-creation auto-adopts Template A in Approved state with the CEO
    // as requestor + sole approver. A non-CEO Human in the same org
    // attempting to revoke hits the matrix's `(Approved, Revoke)` arm —
    // which at v2 (F3.B.role.b descope) admits only the requestor.
    let stranger = seed_human(&store, org.org_id, "stranger").await;
    let stranger_client = authed_client_for(&org.admin, stranger).expect("mint stranger session");

    let res = stranger_client
        .post(action_url(&org, "a", "revoke"))
        .json(&json!({ "reason": "stranger trying to revoke" }))
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
    assert_eq!(evt.actor_agent_id, Some(stranger));
    assert_eq!(evt.diff["after"]["intended_op"], "revoke");
    assert_eq!(evt.diff["after"]["error_kind"], "not_authorised_for_modify");
    // Provenance must point at *some* AR (the Template-A adoption AR);
    // the field MUST be populated for the AR-access event class.
    assert!(
        evt.provenance_auth_request_id.is_some(),
        "auth_request.access_denied must carry provenance_auth_request_id",
    );

    // Sanity: the existing happy-path revoke as CEO still works. Run
    // it after the failed attempt to confirm the fixture's Template-A
    // AR is still in Approved state (the failed revoke must not have
    // mutated state).
    let ceo = authed_client_for(&org.admin, org.ceo_agent_id).expect("mint CEO session");
    let res2 = ceo
        .post(action_url(&org, "a", "revoke"))
        .json(&json!({ "reason": "ceo revoking" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 200);
}
