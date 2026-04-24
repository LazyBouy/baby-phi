//! Integration-level tests for the session cookie layer.
//!
//! Unit-level sign/verify lives inside `src/session.rs`; these tests
//! exercise the full HTTP wiring: a successful claim sets the cookie,
//! the cookie is a valid HS256 JWT, and its `sub` matches the newly
//! minted admin agent id.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use domain::in_memory::InMemoryRepository;
use domain::Repository;
use http_body_util::BodyExt;
use server::bootstrap::hash_credential;
use server::session::{verify_token, SessionKey};
use server::{build_router, AppState};
use tower::ServiceExt;

const SECRET: &str = "test-secret-test-secret-test-secret-test-secret";

fn session_key() -> SessionKey {
    SessionKey::for_tests(SECRET)
}

async fn seed(repo: &InMemoryRepository, plain: &str) {
    let hash = hash_credential(plain).unwrap();
    repo.put_bootstrap_credential(hash).await.unwrap();
}

#[tokio::test]
async fn cookie_on_success_is_signed_with_app_secret() {
    let repo = Arc::new(InMemoryRepository::new());
    seed(&repo, "bphi-bootstrap-sess").await;
    let app = build_router(AppState {
        repo,
        session: session_key(),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus: Arc::new(domain::events::InProcessEventBus::new()),
        session_registry: server::state::new_session_registry(),
        session_max_concurrent: 16,
    });

    let body = serde_json::json!({
        "bootstrap_credential": "bphi-bootstrap-sess",
        "display_name": "Sess Tester",
        "channel": { "kind": "web", "handle": "https://example.com/sess" }
    })
    .to_string();

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header")
        .to_str()
        .unwrap()
        .to_string();

    assert!(set_cookie.starts_with("phi_kernel_session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));

    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let expected_sub = json["human_agent_id"].as_str().unwrap().to_string();

    let token = set_cookie
        .split("phi_kernel_session=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let claims = verify_token(&session_key(), token).expect("verifies");
    assert_eq!(claims.sub, expected_sub);
    assert!(claims.exp > claims.iat);
}

#[tokio::test]
async fn cookie_from_a_different_secret_does_not_verify() {
    let repo = Arc::new(InMemoryRepository::new());
    seed(&repo, "bphi-bootstrap-wrongsig").await;
    let app = build_router(AppState {
        repo,
        session: session_key(),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus: Arc::new(domain::events::InProcessEventBus::new()),
        session_registry: server::state::new_session_registry(),
        session_max_concurrent: 16,
    });

    let body = serde_json::json!({
        "bootstrap_credential": "bphi-bootstrap-wrongsig",
        "display_name": "Alex",
        "channel": { "kind": "slack", "handle": "@alex" }
    })
    .to_string();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    let token = cookie
        .split("phi_kernel_session=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();

    let wrong_key = SessionKey::for_tests("wrong-secret-wrong-secret-wrong-secret-0123456789");
    assert!(verify_token(&wrong_key, token).is_err());
}

#[tokio::test]
async fn status_endpoint_does_not_set_cookie() {
    let repo = Arc::new(InMemoryRepository::new());
    let app = build_router(AppState {
        repo,
        session: session_key(),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus: Arc::new(domain::events::InProcessEventBus::new()),
        session_registry: server::state::new_session_registry(),
        session_max_concurrent: 16,
    });
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v0/bootstrap/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("set-cookie").is_none());
}
