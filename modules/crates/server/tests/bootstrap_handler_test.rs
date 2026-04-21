//! HTTP-layer tests for `/api/v0/bootstrap/*`.
//!
//! Exercises the router end-to-end via `tower::ServiceExt::oneshot`:
//! - `GET /api/v0/bootstrap/status` before + after a claim.
//! - `POST /api/v0/bootstrap/claim` for 201 / 400 / 403 / 409.
//! - Session cookie is set on a successful claim and verifies.
//!
//! The P9 acceptance suite will cover the `/metrics` surface end-to-end
//! (the global Prometheus recorder conflicts with parallel tests here —
//! see the note in `router.rs`).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use domain::in_memory::InMemoryRepository;
use domain::Repository;
use http_body_util::BodyExt;
use server::bootstrap::hash_credential;
use server::session::{verify_token, SessionKey};
use server::{build_router, AppState};
use tower::ServiceExt;

// ---- fixtures --------------------------------------------------------------

const TEST_SECRET: &str = "test-secret-test-secret-test-secret-test-secret";

fn test_session() -> SessionKey {
    SessionKey::for_tests(TEST_SECRET)
}

async fn seed_credential(repo: &InMemoryRepository, plaintext: &str) -> String {
    let hash = hash_credential(plaintext).unwrap();
    let row = repo.put_bootstrap_credential(hash).await.unwrap();
    row.record_id
}

fn app_with(repo: Arc<InMemoryRepository>) -> Router {
    build_router(AppState {
        repo,
        session: test_session(),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
    })
}

async fn body_json(res: axum::response::Response) -> serde_json::Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_claim_body(credential: &str, display_name: &str, handle: &str) -> String {
    serde_json::json!({
        "bootstrap_credential": credential,
        "display_name": display_name,
        "channel": { "kind": "slack", "handle": handle }
    })
    .to_string()
}

// ---- GET /api/v0/bootstrap/status -----------------------------------------

#[tokio::test]
async fn status_reports_unclaimed_on_fresh_install() {
    let repo = Arc::new(InMemoryRepository::new());
    let res = app_with(repo)
        .oneshot(
            Request::builder()
                .uri("/api/v0/bootstrap/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["claimed"], false);
    assert_eq!(json["awaiting_credential"], true);
    assert!(json.get("admin_agent_id").is_none());
}

#[tokio::test]
async fn status_reports_claimed_after_successful_claim() {
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-status-check").await;

    // POST claim first.
    let app = app_with(repo.clone());
    let _ = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-status-check",
                    "Alex Chen",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    // New router instance for the GET call (InMemoryRepository is shared
    // via Arc so the admin is visible).
    let app2 = app_with(repo);
    let res = app2
        .oneshot(
            Request::builder()
                .uri("/api/v0/bootstrap/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["claimed"], true);
    assert!(json["admin_agent_id"].is_string());
}

// ---- POST /api/v0/bootstrap/claim — success path --------------------------

#[tokio::test]
async fn claim_happy_path_returns_201_and_sets_session_cookie() {
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-happy").await;

    let app = app_with(repo);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-happy",
                    "Alex Chen",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    // Set-Cookie must contain the session cookie.
    let set_cookie = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        set_cookie.contains("baby_phi_session="),
        "expected baby_phi_session cookie, got: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));

    // Extract the cookie value and verify it.
    let cookie_line = set_cookie
        .lines()
        .find(|l| l.contains("baby_phi_session="))
        .unwrap();
    let value = cookie_line
        .split("baby_phi_session=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let claims = verify_token(&test_session(), value).expect("session verifies");
    // sub is the admin's agent_id UUID string.
    let json = body_json(res).await;
    assert_eq!(claims.sub, json["human_agent_id"].as_str().unwrap());
    // Full response shape.
    for key in [
        "human_agent_id",
        "inbox_id",
        "outbox_id",
        "grant_id",
        "bootstrap_auth_request_id",
        "audit_event_id",
    ] {
        assert!(json.get(key).is_some(), "missing {key} in response");
    }
}

// ---- POST /api/v0/bootstrap/claim — error paths ---------------------------

#[tokio::test]
async fn claim_with_empty_display_name_returns_400() {
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-400").await;

    let res = app_with(repo)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-400",
                    "   ",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let json = body_json(res).await;
    assert_eq!(json["code"], "VALIDATION_FAILED");
}

#[tokio::test]
async fn claim_with_wrong_credential_returns_403_bootstrap_invalid() {
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-real").await;

    let res = app_with(repo)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-wrong",
                    "Alex Chen",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let json = body_json(res).await;
    assert_eq!(json["code"], "BOOTSTRAP_INVALID");
}

#[tokio::test]
async fn claim_with_consumed_credential_returns_403_already_consumed() {
    let repo = Arc::new(InMemoryRepository::new());
    let record_id = seed_credential(&repo, "bphi-bootstrap-orphan").await;
    // Simulate a consumed-but-no-admin state (corner case worth pinning):
    // mark the credential consumed without running the full claim.
    repo.consume_bootstrap_credential(&record_id).await.unwrap();

    let res = app_with(repo)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-orphan",
                    "Alex Chen",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let json = body_json(res).await;
    assert_eq!(json["code"], "BOOTSTRAP_ALREADY_CONSUMED");
}

#[tokio::test]
async fn claim_after_admin_exists_returns_409() {
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-first").await;

    // First claim → 201.
    let app1 = app_with(repo.clone());
    let first = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-first",
                    "Alex Chen",
                    "@alex",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    // Second claim attempt (even with a different credential) → 409.
    seed_credential(&repo, "bphi-bootstrap-second").await;
    let app2 = app_with(repo);
    let res = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from(post_claim_body(
                    "bphi-bootstrap-second",
                    "Someone Else",
                    "@other",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let json = body_json(res).await;
    assert_eq!(json["code"], "PLATFORM_ADMIN_CLAIMED");
}

#[tokio::test]
async fn claim_with_malformed_json_returns_4xx() {
    // Axum's Json extractor returns 400/415/422 on malformed JSON. We
    // don't care which 4xx exactly (axum can version-bump it); we care
    // that the handler doesn't panic and that no admin is created.
    let repo = Arc::new(InMemoryRepository::new());
    let res = app_with(repo.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v0/bootstrap/claim")
                .header("content-type", "application/json")
                .body(Body::from("{not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res.status().is_client_error());
    assert!(repo.get_admin_agent().await.unwrap().is_none());
}

#[tokio::test]
async fn claim_with_missing_channel_returns_4xx() {
    // Missing `channel` field — the Json extractor rejects before
    // execute_claim runs.
    let repo = Arc::new(InMemoryRepository::new());
    seed_credential(&repo, "bphi-bootstrap-missing-ch").await;
    let body = serde_json::json!({
        "bootstrap_credential": "bphi-bootstrap-missing-ch",
        "display_name": "Alex",
    })
    .to_string();
    let res = app_with(repo.clone())
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
    assert!(res.status().is_client_error());
    assert!(repo.get_admin_agent().await.unwrap().is_none());
}
