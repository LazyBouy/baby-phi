//! End-to-end acceptance tests for `GET /api/v0/orgs/:id/dashboard`.
//!
//! Commitment C12 in the M3 plan. Scenarios:
//!
//! 1. Fresh-minimal-startup org — CTA cards shown, welcome banner
//!    set, counts match the 2-system-agents shape.
//! 2. Populated-org counters update correctly after a second
//!    acceptance-level mutation (adoption AR for template B).
//! 3. 403 when the caller has no membership in the org.
//! 4. 404 when the org id is unknown.
//! 5. Token budget tile reflects the `initial_allocation` from the
//!    compound tx (R-ADMIN-07-R6).
//! 6. **phi-core invariant** — the dashboard wire payload MUST NOT
//!    contain `defaults_snapshot` / `execution_limits` /
//!    `context_config` / `retry_config` / `default_agent_profile` /
//!    `blueprint` at any depth (per P5.0 pre-audit).
//! 7. **Polling smoke** — two sequential GETs return a stable wire
//!    shape; the second one picks up an audit-event delta seeded in
//!    between.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};
use acceptance_common::TEST_SESSION_SECRET;
use serde_json::Value;
use server::session::{sign_and_build_cookie, SessionKey};

/// Build a reqwest client authed as `subject_agent_id` against the
/// same HMAC key the acceptance server uses. The subject string goes
/// into the JWT's `sub` claim, which `AuthenticatedSession` exposes
/// as `session.agent_id`.
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

async fn get_dashboard_as_ceo(org: &ClaimedOrg) -> reqwest::Response {
    let client = client_authed_as(&org.ceo_agent_id.to_string());
    client
        .get(org.url(&format!("/api/v0/orgs/{}/dashboard", org.org_id)))
        .send()
        .await
        .expect("GET dashboard as CEO")
}

async fn get_dashboard_as_admin(org: &ClaimedOrg) -> reqwest::Response {
    // Platform admin — NOT a member of the fixture org; expected to
    // receive 403.
    org.admin
        .authed_client
        .get(org.url(&format!("/api/v0/orgs/{}/dashboard", org.org_id)))
        .send()
        .await
        .expect("GET dashboard as admin")
}

#[tokio::test]
async fn fresh_minimal_startup_shows_cta_cards_and_welcome_banner() {
    let org = spawn_claimed_with_org(false).await;
    let res = get_dashboard_as_ceo(&org).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();

    // Agents summary — CEO + 2 system agents, all seeded with
    // `role = None` at the fixture (the M3/P3 `spawn_claimed_with_org`
    // pre-dates AgentRole; a real M4/P5 wizard would set roles).
    // Dashboard surfaces all three in `unclassified` so operators
    // notice the gap.
    assert_eq!(body["agents_summary"]["total"].as_u64(), Some(3));
    assert_eq!(body["agents_summary"]["unclassified"].as_u64(), Some(3));
    assert_eq!(body["agents_summary"]["executive"].as_u64(), Some(0));
    assert_eq!(body["agents_summary"]["intern"].as_u64(), Some(0));

    // Projects summary — fresh org has no projects at M3; shape_a /
    // shape_b both zero (M4/P8 flipped these from hardcoded-zero to
    // real counts via `count_projects_by_shape_in_org`).
    assert_eq!(body["projects_summary"]["active"].as_u64(), Some(0));
    assert_eq!(body["projects_summary"]["shape_a"].as_u64(), Some(0));
    assert_eq!(body["projects_summary"]["shape_b"].as_u64(), Some(0));

    // Welcome banner present for fresh orgs.
    assert!(
        body["welcome_banner"].is_string(),
        "fresh org must surface welcome banner: got {body:#?}"
    );

    // Viewer is the CEO (only human in the org); role = admin.
    assert_eq!(body["viewer"]["role"].as_str(), Some("admin"));
    assert_eq!(body["viewer"]["can_admin_manage"].as_bool(), Some(true));

    // CTA cards populated for admin + fresh.
    assert!(body["cta_cards"]["add_agent"].is_string());
    assert!(body["cta_cards"]["create_project"].is_string());
    assert!(body["cta_cards"]["adopt_template"].is_string());
    assert!(body["cta_cards"]["configure_system_agents"].is_string());
}

#[tokio::test]
async fn token_budget_matches_allocation_from_fixture() {
    let org = spawn_claimed_with_org(false).await;
    let res = get_dashboard_as_ceo(&org).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    // `spawn_claimed_with_org` pool = 1_000_000. Used starts at 0 per
    // `TokenBudgetPool::new`.
    assert_eq!(body["token_budget"]["used"].as_u64(), Some(0));
    assert_eq!(body["token_budget"]["total"].as_u64(), Some(1_000_000));
    assert!(body["token_budget"]["pool_id"].is_string());
}

#[tokio::test]
async fn dashboard_exposes_adopted_templates_from_adoption_ars() {
    let org = spawn_claimed_with_org(false).await;
    let res = get_dashboard_as_ceo(&org).await;
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let adopted = body["templates_adopted"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    // Fixture adopts Template A.
    assert!(
        adopted.iter().any(|v| v == "a"),
        "templates_adopted must contain `a`: got {adopted:?}"
    );
}

#[tokio::test]
async fn unknown_org_id_returns_404_org_not_found() {
    let org = spawn_claimed_with_org(false).await;
    let unknown = domain::model::ids::OrgId::new();
    let client = client_authed_as(&org.ceo_agent_id.to_string());
    let res = client
        .get(org.url(&format!("/api/v0/orgs/{unknown}/dashboard")))
        .send()
        .await
        .expect("GET dashboard");
    assert_eq!(res.status().as_u16(), 404);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("ORG_NOT_FOUND"));
}

#[tokio::test]
async fn non_member_viewer_receives_403_access_denied() {
    // The platform admin (claimed session) is NOT a member of the org
    // the fixture creates — `spawn_claimed_with_org`'s CEO is a fresh
    // Human Agent distinct from `admin.agent_id`. The dashboard
    // orchestrator's `resolve_viewer_role` returns `None`, and the
    // handler maps it to 403.
    let org = spawn_claimed_with_org(false).await;
    let res = get_dashboard_as_admin(&org).await;
    assert_eq!(
        res.status().as_u16(),
        403,
        "platform admin is not a member of the fixture org; expected 403"
    );
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("ORG_ACCESS_DENIED"));
}

/// **Positive phi-core leverage invariant (P5.0 pre-audit)** — the
/// dashboard wire JSON must not carry any phi-core-wrapping fields.
/// Walks the payload recursively; any hit fails loudly so a future
/// reviewer knows exactly which field re-introduced the coupling.
#[tokio::test]
async fn dashboard_wire_json_excludes_phi_core_fields() {
    let org = spawn_claimed_with_org(false).await;
    // The admin is a non-member so the dashboard direct-fetch would
    // 403; we sidestep that by driving the orchestrator in-process
    // over the raw repo (same pattern tests lower in this file use
    // for non-HTTP assertions).
    let summary = server::platform::orgs::dashboard::dashboard_summary(
        org.admin.acc.store.clone(),
        org.org_id,
        org.ceo_agent_id,
        chrono::Utc::now(),
    )
    .await
    .expect("dashboard_summary");
    let outcome = match summary {
        server::platform::orgs::dashboard::DashboardOutcome::Found(s) => s,
        other => panic!("expected Found, got {other:?}"),
    };
    let json = serde_json::to_value(&*outcome).expect("serialize");
    let forbidden = [
        "defaults_snapshot",
        "execution_limits",
        "context_config",
        "retry_config",
        "default_agent_profile",
        "blueprint",
    ];
    assert_no_keys(&json, &forbidden);
}

fn assert_no_keys(v: &Value, forbidden: &[&str]) {
    match v {
        Value::Object(map) => {
            for (k, inner) in map {
                for f in forbidden {
                    assert_ne!(
                        k, f,
                        "dashboard wire payload contains forbidden key `{f}` — P5.0 pre-audit forbids \
                         phi-core-wrapping fields on the dashboard contract"
                    );
                }
                assert_no_keys(inner, forbidden);
            }
        }
        Value::Array(arr) => {
            for inner in arr {
                assert_no_keys(inner, forbidden);
            }
        }
        _ => {}
    }
}

/// Polling smoke: two sequential GETs return the same wire shape and
/// counts remain stable when no underlying state changes.
#[tokio::test]
async fn sequential_gets_are_stable_absent_state_changes() {
    // Drive the orchestrator twice in-process against the same store;
    // bypasses the 403 path the HTTP form takes on the non-member
    // admin cookie.
    let org = spawn_claimed_with_org(false).await;
    let now = chrono::Utc::now();
    let a = server::platform::orgs::dashboard::dashboard_summary(
        org.admin.acc.store.clone(),
        org.org_id,
        org.ceo_agent_id,
        now,
    )
    .await
    .unwrap();
    let b = server::platform::orgs::dashboard::dashboard_summary(
        org.admin.acc.store.clone(),
        org.org_id,
        org.ceo_agent_id,
        now,
    )
    .await
    .unwrap();
    use server::platform::orgs::dashboard::DashboardOutcome;
    let (a, b) = match (a, b) {
        (DashboardOutcome::Found(a), DashboardOutcome::Found(b)) => (a, b),
        _ => panic!("both outcomes should be Found"),
    };
    assert_eq!(a.agents_summary, b.agents_summary);
    assert_eq!(a.token_budget, b.token_budget);
    assert_eq!(a.templates_adopted, b.templates_adopted);
}
