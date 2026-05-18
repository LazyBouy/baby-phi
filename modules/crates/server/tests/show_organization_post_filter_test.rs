//! CH-18 / ADR-0056 §D56.5 + F3.B.list-filter.a — `GET /api/v0/orgs/:id`
//! silent post-filter test. Asserts that `adopted_template_count`
//! reflects the per-state access matrix filter: a non-requestor
//! non-slot-approver viewer sees `adopted_template_count == 0` even
//! though the org has 1 adoption AR for Template A.
//!
//! Pre-state: `spawn_claimed_with_org` adopts Template A at org
//! creation. The CEO is the requestor on that AR.
//!
//! - CEO viewer → `adopted_template_count == 1`.
//! - Outsider viewer → `adopted_template_count == 0` (silently filtered).

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};
use acceptance_common::owner_grants::seed_owner_grants;
use acceptance_common::TEST_SESSION_SECRET;

use chrono::Utc;
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

async fn get_show(org: &ClaimedOrg, viewer: AgentId) -> reqwest::Response {
    let client = client_authed_as(&viewer.to_string());
    client
        .get(org.url(&format!("/api/v0/orgs/{}", org.org_id)))
        .send()
        .await
        .expect("GET org")
}

#[tokio::test]
async fn show_organization_adopted_template_count_reflects_access_filter() {
    let org = spawn_claimed_with_org(false).await;
    let store: Arc<dyn Repository> = org.admin.acc.store.clone();

    // CEO viewer — requestor on Template A adoption AR — sees count 1.
    let res = get_show(&org, org.ceo_agent_id).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let count = body["adopted_template_count"].as_u64().unwrap();
    assert_eq!(
        count, 1,
        "CEO viewer must see adopted_template_count == 1: got {body}"
    );

    // Outsider viewer — neither requestor nor slot approver — sees 0.
    let outsider = seed_human(&store, org.org_id, "Outsider").await;

    // CH-27 / ADR-0062 §D62.4 (F4.b USER-DIVERGENT) — seed Inspect
    // grant for outsider on the org so the `show_organization`
    // blocking gate (CH-27 / ADR-0062 §D62.1) admits the outsider
    // viewer. The CH-18 silent post-filter on adoption ARs (inside
    // the show body) still applies as defence-in-depth and silently
    // filters the count to 0 for ARs the outsider can't read.
    seed_owner_grants(&store, outsider, vec![org.org_id])
        .await
        .expect("seed Inspect grant for outsider");

    let res = get_show(&org, outsider).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let count = body["adopted_template_count"].as_u64().unwrap();
    assert_eq!(
        count, 0,
        "Outsider viewer must see adopted_template_count == 0 (silent filter): got {body}"
    );
}
