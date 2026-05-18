//! CH-18 / ADR-0056 §D56.5 + F3.B.list-filter.a — dashboard silent
//! post-filter test. Asserts that ARs the viewer is NOT authorised to
//! read are silently filtered from the dashboard's `templates_adopted`
//! list, and that NO `auth_request.access_denied` audit event is
//! emitted for filtered entries (silent post-filter — same UX as
//! `archived_at IS NOT NULL` filtering).
//!
//! Pre-state: `spawn_claimed_with_org` adopts Template A at org
//! creation. The CEO is the requestor on the Template A AR (Template-E
//! shape, CEO self-approve). The CEO viewer is authorised by the
//! matrix's Approved × Read by Requestor cell → Ok.
//!
//! Test scenario: a non-CEO non-slot-approver agent invokes the
//! dashboard endpoint. The matrix's Approved × Read by Other cell
//! returns Err → AR is filtered out → `templates_adopted` is empty for
//! that viewer.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};
use acceptance_common::owner_grants::seed_owner_grants;
use acceptance_common::TEST_SESSION_SECRET;

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind};
use domain::Repository;
use serde_json::Value;
use server::session::{sign_and_build_cookie, SessionKey};
use std::sync::Arc;

fn client_authed_as(subject_agent_id: &str) -> reqwest::Client {
    let key = SessionKey::for_tests(TEST_SESSION_SECRET);
    let (jwt, _cookie) =
        sign_and_build_cookie(&key, subject_agent_id).expect("sign session cookie");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("phi_kernel_session={jwt}"))
            .expect("cookie header"),
    );
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .build()
        .unwrap()
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

async fn get_dashboard(org: &ClaimedOrg, viewer: AgentId) -> reqwest::Response {
    let client = client_authed_as(&viewer.to_string());
    client
        .get(org.url(&format!("/api/v0/orgs/{}/dashboard", org.org_id)))
        .send()
        .await
        .expect("GET dashboard")
}

#[tokio::test]
async fn dashboard_silently_filters_adoption_ars_for_unauthorised_viewer() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // CEO is the requestor on the Template A adoption AR — authorised
    // to read it. Sanity-check via the dashboard.
    let res = get_dashboard(&org, org.ceo_agent_id).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let adopted = body["templates_adopted"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        adopted.iter().any(|v| v == "a"),
        "CEO viewer must see Template A: got {adopted:?}"
    );

    // A second human agent in the org (NOT a slot approver, NOT the
    // requestor) — under the matrix's Approved × Read by Other cell,
    // the Template A adoption AR is filtered out silently.
    let outsider = seed_human(&store, org.org_id, "Outsider").await;

    // CH-27 / ADR-0062 §D62.4 (F4.b USER-DIVERGENT) — seed Observe
    // grant for outsider on the org so the dashboard endpoint's
    // blocking gate (CH-27 / ADR-0062 §D62.1) admits the outsider
    // viewer. The CH-18 silent post-filter on adoption ARs (inside
    // the dashboard body) still applies as defence-in-depth and
    // silently filters out ARs the outsider is not authorised to read.
    seed_owner_grants(&store, outsider, vec![org.org_id])
        .await
        .expect("seed Observe grant for outsider");

    // Snapshot audit events BEFORE the outsider's dashboard read.
    let events_before = store
        .list_recent_audit_events_for_org(org.org_id, 100)
        .await
        .unwrap();
    let count_before = events_before
        .iter()
        .filter(|e| e.event_type == "auth_request.access_denied")
        .count();

    let res = get_dashboard(&org, outsider).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let adopted = body["templates_adopted"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        adopted.is_empty(),
        "Outsider viewer must see EMPTY templates_adopted (silent filter): got {adopted:?}"
    );

    // Snapshot audit events AFTER. F3.B.list-filter.a — silent filter
    // implies NO new `auth_request.access_denied` events emitted for
    // filtered entries.
    let events_after = store
        .list_recent_audit_events_for_org(org.org_id, 100)
        .await
        .unwrap();
    let count_after = events_after
        .iter()
        .filter(|e| e.event_type == "auth_request.access_denied")
        .count();
    assert_eq!(
        count_after, count_before,
        "silent filter — no auth_request.access_denied event expected for filtered list entries"
    );
    // Sanity — the AuditClass on any pre-existing access-denied events
    // remains Alerted (no regression on the audit-class invariant).
    for e in events_after
        .iter()
        .filter(|e| e.event_type == "auth_request.access_denied")
    {
        assert_eq!(e.audit_class, AuditClass::Alerted);
    }
}
