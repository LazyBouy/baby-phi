//! Smoke test for the `spawn_claimed()` acceptance fixture.
//!
//! The fixture is consumed by every M2/P4+ page-acceptance test. This
//! test just exercises the bootstrap-claim dance + confirms the
//! pre-cookied client actually carries the session cookie on a
//! follow-up request.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;

#[tokio::test]
async fn spawn_claimed_returns_a_usable_admin_session() {
    let admin = spawn_claimed(false).await;

    // agent_id looks like a UUID.
    assert!(
        uuid::Uuid::parse_str(&admin.agent_id).is_ok(),
        "agent_id must be a UUID; got {}",
        admin.agent_id
    );

    // Cookie is a JWT — three dot-separated segments.
    assert_eq!(
        admin.session_cookie.matches('.').count(),
        2,
        "session cookie must be a JWT: {}",
        admin.session_cookie
    );

    // Follow-up request: `/healthz/ready` is unauthenticated but
    // reachable — ensures the pre-cookied client actually calls the
    // server.
    let res = admin
        .authed_client
        .get(admin.url("/healthz/ready"))
        .send()
        .await
        .expect("authed GET");
    assert!(res.status().is_success());
}

#[tokio::test]
async fn second_claim_on_same_server_is_refused() {
    // `spawn_claimed` claims once. A second POST to the same
    // `/api/v0/bootstrap/claim` must return 409 PLATFORM_ADMIN_CLAIMED.
    let admin = spawn_claimed(false).await;
    let res = admin
        .acc
        .client()
        .post(admin.url("/api/v0/bootstrap/claim"))
        .json(&serde_json::json!({
            "bootstrap_credential": "bphi-bootstrap-second",
            "display_name": "Second",
            "channel": { "kind": "email", "handle": "second@example.com" }
        }))
        .send()
        .await
        .expect("second claim");
    assert_eq!(res.status().as_u16(), 409);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("PLATFORM_ADMIN_CLAIMED"));
}
