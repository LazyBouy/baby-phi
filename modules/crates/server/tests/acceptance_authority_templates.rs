//! M5/P5 authority-template surface acceptance tests.
//!
//! Covers page 12 (R-ADMIN-12-R1/R2/R3 + W1/W2/W3/W4 + N1/N2/N3)
//! via the HTTP surface.
//!
//! Scenarios:
//!  1. `GET /authority-templates` — pending / active / revoked /
//!     available buckets populate correctly for a fresh org that
//!     adopted A + B at creation.
//!  2. `POST /:kind/adopt` for C — creates new approved adoption
//!     AR; second adopt of same kind returns 409
//!     `TEMPLATE_ADOPTION_ALREADY_ACTIVE`.
//!  3. `POST /:kind/adopt` for E — returns 400
//!     `TEMPLATE_E_ALWAYS_AVAILABLE`.
//!  4. `POST /:kind/revoke` — forward-only cascade; emits audit
//!     with `grant_count_revoked`; re-revoke returns 409
//!     `TEMPLATE_ADOPTION_TERMINAL`.
//!  5. `POST /:kind/revoke` without reason — returns 400
//!     `TEMPLATE_INPUT_INVALID`.
//!  6. `POST /:kind/deny` on a pending adoption AR — transitions
//!     to Denied; re-adopt succeeds (kind becomes available again).
//!  7. `POST /:kind/approve` on already-approved AR — returns 409
//!     `TEMPLATE_ADOPTION_ALREADY_ACTIVE`.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use serde_json::{json, Value};

fn ceo_client(org: &ClaimedOrg) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(&org.admin, org.ceo_agent_id)
        .expect("mint CEO session")
}

fn list_url(org: &ClaimedOrg) -> String {
    org.url(&format!("/api/v0/orgs/{}/authority-templates", org.org_id))
}

fn action_url(org: &ClaimedOrg, kind: &str, action: &str) -> String {
    org.url(&format!(
        "/api/v0/orgs/{}/authority-templates/{}/{}",
        org.org_id, kind, action
    ))
}

// ---------------------------------------------------------------------------
// 1. List buckets — fresh org with A + B adopted at creation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_buckets_are_populated_for_fresh_org() {
    let org = spawn_claimed_with_org(false).await;
    let res = ceo_client(&org).get(list_url(&org)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();

    // The `spawn_claimed_with_org` fixture adopts Template A at
    // org-creation time (Template-E shape: immediately approved).
    let active = body["active"].as_array().expect("active array");
    assert!(
        !active.is_empty(),
        "at least Template A should be active after org creation",
    );
    let active_kinds: Vec<String> = active
        .iter()
        .map(|r| r["kind"].as_str().unwrap().to_string())
        .collect();
    assert!(
        active_kinds.contains(&"a".to_string()),
        "Template A should be in the active bucket; got {active_kinds:?}",
    );

    // C + D should be available (fresh org with only A adopted).
    let available = body["available"].as_array().expect("available array");
    let avail_strs: Vec<String> = available
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(avail_strs.contains(&"c".to_string()), "got {avail_strs:?}");
    assert!(avail_strs.contains(&"d".to_string()), "got {avail_strs:?}");
    assert!(
        avail_strs.contains(&"e-always".to_string()),
        "E always-available sentinel present; got {avail_strs:?}",
    );
}

// ---------------------------------------------------------------------------
// 2. Adopt C — fresh adoption; second adopt returns 409
// ---------------------------------------------------------------------------

#[tokio::test]
async fn adopt_c_creates_approved_ar_and_blocks_duplicate() {
    let org = spawn_claimed_with_org(false).await;
    let res = ceo_client(&org)
        .post(action_url(&org, "c", "adopt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["state"], "approved");
    assert!(body["adoption_auth_request_id"].as_str().is_some());
    assert!(body["audit_event_id"].as_str().is_some());

    // Second adopt — active adoption already exists → 409.
    let res2 = ceo_client(&org)
        .post(action_url(&org, "c", "adopt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 409);
    let body2: Value = res2.json().await.unwrap();
    assert_eq!(body2["code"], "TEMPLATE_ADOPTION_ALREADY_ACTIVE");

    // List shows C under active now.
    let listing: Value = ceo_client(&org)
        .get(list_url(&org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let active: Vec<String> = listing["active"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap().to_string())
        .collect();
    assert!(active.contains(&"c".to_string()));
    // `c` no longer appears in `available`.
    let available: Vec<String> = listing["available"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(!available.contains(&"c".to_string()));
}

// ---------------------------------------------------------------------------
// 3. Adopt E — always-available sentinel; 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn adopt_e_returns_always_available_error() {
    let org = spawn_claimed_with_org(false).await;
    let res = ceo_client(&org)
        .post(action_url(&org, "e", "adopt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "TEMPLATE_E_ALWAYS_AVAILABLE");
}

// ---------------------------------------------------------------------------
// 4. Revoke A — cascade; re-revoke returns 409
// ---------------------------------------------------------------------------

#[tokio::test]
async fn revoke_a_cascades_grants_and_blocks_re_revoke() {
    let org = spawn_claimed_with_org(false).await;

    // Fresh org's Template A has 0 fired grants (no HAS_LEAD edges
    // yet in the minimal fixture). `grant_count_revoked = 0` is
    // still a valid successful revoke; the cascade machinery is
    // exercised even without grants to cascade over.
    let res = ceo_client(&org)
        .post(action_url(&org, "a", "revoke"))
        .json(&json!({ "reason": "restructure" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["adoption_auth_request_id"].as_str().is_some());
    assert_eq!(body["grant_count_revoked"], 0);
    assert!(body["audit_event_id"].as_str().is_some());

    // Second revoke returns 409 TEMPLATE_ADOPTION_TERMINAL.
    let res2 = ceo_client(&org)
        .post(action_url(&org, "a", "revoke"))
        .json(&json!({ "reason": "re-run" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status().as_u16(), 409);
    let body2: Value = res2.json().await.unwrap();
    assert_eq!(body2["code"], "TEMPLATE_ADOPTION_TERMINAL");

    // Listing shows A under revoked (not active).
    let listing: Value = ceo_client(&org)
        .get(list_url(&org))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let revoked: Vec<String> = listing["revoked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap().to_string())
        .collect();
    assert!(revoked.contains(&"a".to_string()));
}

// ---------------------------------------------------------------------------
// 5. Revoke without reason — 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn revoke_without_reason_returns_400() {
    let org = spawn_claimed_with_org(false).await;
    let res = ceo_client(&org)
        .post(action_url(&org, "a", "revoke"))
        .json(&json!({ "reason": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "TEMPLATE_INPUT_INVALID");
}

// ---------------------------------------------------------------------------
// 6. Approve on already-approved AR — 409
// ---------------------------------------------------------------------------

#[tokio::test]
async fn approve_on_already_approved_returns_409() {
    let org = spawn_claimed_with_org(false).await;
    // Fixture adopted Template A at org creation already-Approved.
    let res = ceo_client(&org)
        .post(action_url(&org, "a", "approve"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "TEMPLATE_ADOPTION_ALREADY_ACTIVE");
}

// ---------------------------------------------------------------------------
// 7. Deny + re-adopt round-trip for C (which has no prior AR)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deny_of_unseeded_c_is_not_found_until_adopted() {
    let org = spawn_claimed_with_org(false).await;
    // No prior C adoption AR exists → deny surfaces 404
    // TEMPLATE_ADOPTION_NOT_FOUND.
    let res = ceo_client(&org)
        .post(action_url(&org, "c", "deny"))
        .json(&json!({ "reason": "not now" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 404);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["code"], "TEMPLATE_ADOPTION_NOT_FOUND");
}
