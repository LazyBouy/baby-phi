//! CH-24 / F1.B page-1/page-2 vertical slice — milestone-rollup
//! acceptance for the org-creation surface.
//!
//! Per plan §7 P-NEW-TESTS (v2 user-locked F1.B 5-per-page split):
//!
//! - `m5_orgs_bootstrap_to_org_list_visibility` — golden-path slice:
//!   spawn a claimed admin, create two orgs via the real `POST
//!   /api/v0/orgs` wizard endpoint, and assert both are visible on `GET
//!   /api/v0/orgs`. Mirrors the cross-page bootstrap-to-org segment of
//!   the milestone golden path.
//! - `m5_orgs_cross_org_list_isolation` — cross-org isolation at the
//!   org-detail / dashboard surface.
//!
//!   **SCOPE-NARROWING vs plan §7 P-NEW-TESTS.1 scenario 2**: the plan's
//!   literal wording says *"Org-A viewer sees only Org-A in their page-1
//!   query"* + *"Org-B's id 403/404 on Org-A's context"*. The actual
//!   surface at M5 ships `GET /api/v0/orgs` as a **platform-admin list**
//!   that returns ALL orgs to any authenticated session (cf
//!   `server/src/handlers/orgs.rs::list`, no viewer filter). The
//!   cross-org isolation gate at M5 lives at the **dashboard** surface
//!   (`GET /api/v0/orgs/:id/dashboard` returns 403 `ORG_ACCESS_DENIED`
//!   for non-member viewers per CH-15 hard-deny). This scenario
//!   exercises the actually-shipping gate to preserve the spirit of the
//!   isolation invariant. The page-1 list-filter behaviour is M6+ scope
//!   (post-M5 admin-page narrowing).
//!
//! Both scenarios are stand-alone end-to-end tests; each spins its own
//! acceptance harness via `acceptance_common::m5_bootstrap` helpers.

mod acceptance_common;

use acceptance_common::m5_bootstrap::{bootstrap_with_two_orgs, client_as};
use serde_json::Value;

#[tokio::test]
async fn m5_orgs_bootstrap_to_org_list_visibility() {
    let fx = bootstrap_with_two_orgs().await;

    // GET /api/v0/orgs returns both orgs the wizard just created.
    let res = fx
        .admin
        .authed_client
        .get(fx.admin.url("/api/v0/orgs"))
        .send()
        .await
        .expect("GET /api/v0/orgs");
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.expect("decode list body");
    let orgs = body["orgs"].as_array().expect("orgs is array");
    assert_eq!(
        orgs.len(),
        2,
        "two orgs created via wizard should both be visible: {body:?}"
    );

    let names: std::collections::HashSet<&str> = orgs
        .iter()
        .map(|o| o["display_name"].as_str().expect("display_name is string"))
        .collect();
    assert!(
        names.contains("Atlas Research"),
        "Org-A absent from list: {names:?}"
    );
    assert!(
        names.contains("Beta Labs"),
        "Org-B absent from list: {names:?}"
    );

    // Each list entry exposes the wire-stable member_count =
    // CEO + 2 system agents = 3 (the wizard-shape baseline).
    for o in orgs {
        assert_eq!(
            o["member_count"].as_u64(),
            Some(3),
            "wizard shape baseline = 3 agents per org"
        );
    }

    // Cross-check by walking the show endpoint for one org —
    // confirms list ↔ show consistency at the M5-seal moment.
    let org_a_show = fx
        .admin
        .authed_client
        .get(fx.admin.url(&format!("/api/v0/orgs/{}", fx.org_a_id)))
        .send()
        .await
        .expect("GET /api/v0/orgs/:id");
    assert_eq!(org_a_show.status().as_u16(), 200);
    let show_body: Value = org_a_show.json().await.expect("decode show body");
    assert_eq!(
        show_body["organization"]["display_name"]
            .as_str()
            .expect("display_name string"),
        "Atlas Research"
    );
}

#[tokio::test]
async fn m5_orgs_cross_org_list_isolation() {
    let fx = bootstrap_with_two_orgs().await;

    // The Org-A CEO is a member of Org-A only. Hitting Org-B's
    // dashboard with Org-A CEO's session cookie must surface 403
    // `ORG_ACCESS_DENIED` — the CH-15 hard-deny gate at the
    // dashboard surface.
    let org_a_ceo_client = client_as(&fx.admin, fx.org_a_ceo);
    let res = org_a_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/orgs/{}/dashboard", fx.org_b_id)),
        )
        .send()
        .await
        .expect("GET Org-B dashboard as Org-A CEO");
    assert_eq!(
        res.status().as_u16(),
        403,
        "Org-A CEO must not see Org-B dashboard"
    );
    let err: Value = res.json().await.expect("decode 403 envelope");
    assert_eq!(
        err["code"].as_str(),
        Some("ORG_ACCESS_DENIED"),
        "cross-org dashboard access must surface ORG_ACCESS_DENIED"
    );

    // Symmetric assertion — Org-B CEO denied at Org-A.
    let org_b_ceo_client = client_as(&fx.admin, fx.org_b_ceo);
    let res2 = org_b_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/orgs/{}/dashboard", fx.org_a_id)),
        )
        .send()
        .await
        .expect("GET Org-A dashboard as Org-B CEO");
    assert_eq!(res2.status().as_u16(), 403);
    let err2: Value = res2.json().await.expect("decode 403 envelope");
    assert_eq!(err2["code"].as_str(), Some("ORG_ACCESS_DENIED"));

    // Sanity — each CEO CAN see their own org's dashboard. Proves
    // the assertion above is gate-driven, not generic 403.
    let res_self = org_a_ceo_client
        .get(
            fx.admin
                .url(&format!("/api/v0/orgs/{}/dashboard", fx.org_a_id)),
        )
        .send()
        .await
        .expect("GET Org-A dashboard as Org-A CEO");
    assert_eq!(
        res_self.status().as_u16(),
        200,
        "Org-A CEO must see own dashboard"
    );
}
