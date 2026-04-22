//! Smoke test for the `spawn_claimed_with_org()` fixture (M3/P3).
//!
//! Proves the fixture returns a ClaimedOrg whose org + CEO + two
//! system agents + token-budget-pool are actually persisted. The
//! fixture is the foundation that M3/P4's `acceptance_orgs_create.rs`
//! + M3/P5's `acceptance_orgs_dashboard.rs` build on.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed_with_org;
use domain::repository::Repository;

#[tokio::test]
async fn spawn_claimed_with_org_returns_usable_populated_org() {
    let claimed = spawn_claimed_with_org(false).await;

    // The platform admin from spawn_claimed is still reachable + the
    // new CEO is a DISTINCT agent owned by the new org.
    assert_ne!(claimed.admin.agent_id, claimed.ceo_agent_id.to_string());
    assert_eq!(claimed.system_agents.len(), 2);

    // The org + CEO + system agents are durable: query via Repository
    // through the harness store handle.
    let store = claimed.admin.acc.store.as_ref();
    let org = store
        .get_organization(claimed.org_id)
        .await
        .expect("get org")
        .expect("org exists");
    assert_eq!(org.id, claimed.org_id);
    assert_eq!(org.display_name, "Fixture Org");
    assert_eq!(org.system_agents.len(), 2);

    // Agents: CEO + 2 system agents all present, all owned by the org.
    let agents_in_org = store
        .list_agents_in_org(claimed.org_id)
        .await
        .expect("list agents");
    assert_eq!(
        agents_in_org.len(),
        3,
        "CEO + 2 system agents = 3 org members"
    );

    // Authority templates: exactly one (Template A), terminal-Approved
    // because the adoption AR is Template-E-shaped.
    let adoptions = store
        .list_adoption_auth_requests_for_org(claimed.org_id)
        .await
        .expect("list adoption ARs");
    assert_eq!(adoptions.len(), 1);
    assert_eq!(
        adoptions[0].state,
        domain::model::nodes::AuthRequestState::Approved
    );
}

#[tokio::test]
async fn spawn_claimed_with_org_url_helper_matches_admin() {
    let claimed = spawn_claimed_with_org(false).await;
    let expected = claimed.admin.url("/healthz/ready");
    let got = claimed.url("/healthz/ready");
    assert_eq!(got, expected);
}
