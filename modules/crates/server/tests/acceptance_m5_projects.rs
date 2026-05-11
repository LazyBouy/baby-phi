//! CH-24 / F1.B page-3/page-4 vertical slice — milestone-rollup
//! acceptance for the project-creation surface.
//!
//! Per plan §7 P-NEW-TESTS (v2 user-locked F1.B 5-per-page split):
//!
//! - `m5_projects_create_under_org_and_assert_isolation` — bootstrap
//!   two orgs, materialise a Shape A project under Org-A, and assert
//!   the project is visible to Org-A's CEO and invisible to Org-B's
//!   CEO at the project-detail surface (`GET /api/v0/projects/:id`).
//!
//!   **SCOPE-NARROWING vs plan §7 P-NEW-TESTS.1 (acceptance_m5_projects.rs)**:
//!   the plan's literal wording reads *"assert P-A1 invisible on Org-B's
//!   page-3 project list → assert page-3 query for P-A1 from Org-B context
//!   returns 403/404"*. There is **no `GET /api/v0/orgs/:org_id/projects`
//!   list endpoint at M5** (cf `server/src/router.rs:114` — only `POST`
//!   for project create); the page-3 project-list page is served via
//!   `GET /api/v0/orgs/:id/dashboard` (`projects_summary.{shape_a,
//!   shape_b}` counts). This scenario asserts cross-org isolation at
//!   the existing project-detail surface (`GET /api/v0/projects/:id` →
//!   403 `PROJECT_ACCESS_DENIED` for non-member viewers per the
//!   `DetailOutcome::AccessDenied` arm at
//!   `server/src/platform/projects/detail.rs:215`) AND verifies via the
//!   dashboard counts that Org-A surfaces the project. A future M6+
//!   project-list-endpoint would let this assertion split across the
//!   two surfaces; at M5 the gate-driven access path IS the page-3
//!   isolation invariant.

mod acceptance_common;

use acceptance_common::m5_bootstrap::{bootstrap_with_two_orgs_and_projects, client_as};
use serde_json::Value;

#[tokio::test]
async fn m5_projects_create_under_org_and_assert_isolation() {
    let fx = bootstrap_with_two_orgs_and_projects().await;

    // -------- Visibility from Org-A side ------------------------------------
    let org_a_ceo_client = client_as(&fx.admin, fx.org_a_ceo);

    // Org-A's dashboard surface (the page-3 backing query at M5) shows
    // the freshly-materialised project under shape_a.
    let dash_a = org_a_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/orgs/{}/dashboard", fx.org_a_id)),
        )
        .send()
        .await
        .expect("GET Org-A dashboard");
    assert_eq!(dash_a.status().as_u16(), 200);
    let body_a: Value = dash_a.json().await.expect("decode Org-A dashboard");
    assert_eq!(
        body_a["projects_summary"]["shape_a"].as_u64(),
        Some(1),
        "Org-A dashboard must surface the materialised Shape A project: {body_a:?}"
    );

    // Project-detail surface — Org-A's lead is on the roster + Org-A
    // is the owning org. 200 with the full ProjectDetail body.
    let lead_client = client_as(&fx.admin, fx.project_a_lead);
    let detail_a = lead_client
        .get(
            fx.admin
                .url(&format!("/api/v0/projects/{}", fx.project_a_id)),
        )
        .send()
        .await
        .expect("GET project detail as Org-A lead");
    assert_eq!(detail_a.status().as_u16(), 200);
    let detail_body: Value = detail_a.json().await.expect("decode project detail");
    let owning = detail_body["owning_org_ids"]
        .as_array()
        .expect("owning_org_ids array");
    assert_eq!(owning.len(), 1);
    assert_eq!(
        owning[0].as_str().expect("owning org is string"),
        fx.org_a_id.to_string(),
        "Shape A owns under Org-A only"
    );

    // -------- Isolation from Org-B side -------------------------------------
    // Org-B's CEO is a member of Org-B only — has no relation to
    // Project-A (not on its roster, not in its owning_orgs). The
    // detail handler returns 403 PROJECT_ACCESS_DENIED.
    let org_b_ceo_client = client_as(&fx.admin, fx.org_b_ceo);
    let detail_b = org_b_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/projects/{}", fx.project_a_id)),
        )
        .send()
        .await
        .expect("GET project detail as Org-B CEO");
    assert!(
        matches!(detail_b.status().as_u16(), 403 | 404),
        "cross-org viewer must be denied at project detail; got {}",
        detail_b.status()
    );

    // Org-B's dashboard surface must NOT count Org-A's project.
    let dash_b = org_b_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/orgs/{}/dashboard", fx.org_b_id)),
        )
        .send()
        .await
        .expect("GET Org-B dashboard");
    assert_eq!(dash_b.status().as_u16(), 200);
    let body_b: Value = dash_b.json().await.expect("decode Org-B dashboard");
    assert_eq!(
        body_b["projects_summary"]["shape_a"].as_u64(),
        Some(0),
        "Org-B has no projects; cross-org leak would surface as a non-zero count"
    );
    assert_eq!(
        body_b["projects_summary"]["shape_b"].as_u64(),
        Some(0),
        "Org-B has no Shape B projects either"
    );
}
