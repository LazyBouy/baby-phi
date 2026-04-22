//! Smoke test for the `spawn_claimed_with_org_and_project()` fixture
//! (M4/P3 plan commitment C12).
//!
//! Proves the extended fixture lands a Shape A project + its lead /
//! member agents on top of the claimed-org environment. Downstream
//! tests at M4/P4–P7 build on this foundation.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed_with_org_and_project;
use domain::repository::Repository;

#[tokio::test]
async fn spawn_claimed_with_project_persists_project_and_edges() {
    let claimed = spawn_claimed_with_org_and_project(false).await;

    let store = claimed.claimed_org.admin.acc.store.as_ref();

    // The project row is reachable.
    let proj = store
        .get_project(claimed.project_id)
        .await
        .expect("get_project")
        .expect("project persisted");
    assert_eq!(proj.id, claimed.project_id);
    assert_eq!(proj.name, "Fixture Project");
    assert_eq!(
        proj.shape,
        domain::model::nodes::ProjectShape::A,
        "fixture seeds a Shape A project"
    );

    // Belongs-to edge picks it up in the owning-org listing.
    let in_org = store
        .list_projects_in_org(claimed.org_id())
        .await
        .expect("list_projects_in_org");
    assert_eq!(in_org.len(), 1);
    assert_eq!(in_org[0].id, claimed.project_id);

    // `HAS_LEAD` edge is reachable via `list_projects_led_by_agent`.
    let led_by_lead = store
        .list_projects_led_by_agent(claimed.project_lead)
        .await
        .expect("list_projects_led_by_agent");
    assert_eq!(led_by_lead.len(), 1);
    assert_eq!(led_by_lead[0].id, claimed.project_id);

    // The non-lead member is NOT the lead.
    let led_by_member = store
        .list_projects_led_by_agent(claimed.project_member)
        .await
        .expect("list_projects_led_by_agent for member");
    assert!(led_by_member.is_empty(), "member isn't the lead");
}

#[tokio::test]
async fn spawn_claimed_with_project_agents_belong_to_org() {
    let claimed = spawn_claimed_with_org_and_project(false).await;

    let store = claimed.claimed_org.admin.acc.store.as_ref();
    let agents = store
        .list_agents_in_org(claimed.org_id())
        .await
        .expect("list agents");

    // CEO + 2 system + lead + member = 5 agents.
    assert_eq!(
        agents.len(),
        5,
        "claimed org + 2 new project agents = 5 total"
    );
    assert!(agents.iter().any(|a| a.id == claimed.project_lead));
    assert!(agents.iter().any(|a| a.id == claimed.project_member));
}
