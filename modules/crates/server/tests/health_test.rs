//! Smoke test for the server's health endpoints with the shared
//! `domain::in_memory::InMemoryRepository` (enabled via
//! `features = ["in-memory-repo"]` on the dev-dep).

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use domain::in_memory::InMemoryRepository;
use server::{build_router, AppState, SessionKey};
use tower::ServiceExt;

fn app(healthy: bool) -> axum::Router {
    let repo = InMemoryRepository::new();
    repo.set_unhealthy(!healthy);
    build_router(AppState {
        repo: Arc::new(repo),
        session: SessionKey::for_tests("test-secret-test-secret-test-secret-test-secret"),
        audit: Arc::new(domain::audit::NoopAuditEmitter),
        master_key: Arc::new(store::crypto::MasterKey::from_bytes([7u8; 32])),
        event_bus: Arc::new(domain::events::InProcessEventBus::new()),
        session_registry: server::state::new_session_registry(),
        session_max_concurrent: 16,
    })
}

#[tokio::test]
async fn live_is_always_ok() {
    let res = app(false)
        .oneshot(
            Request::builder()
                .uri("/healthz/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn ready_reports_storage_up() {
    let res = app(true)
        .oneshot(
            Request::builder()
                .uri("/healthz/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn ready_reports_storage_down() {
    let res = app(false)
        .oneshot(
            Request::builder()
                .uri("/healthz/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// /metrics is exercised by the production binary (see `with_prometheus`
// in router.rs). It cannot be covered here because `axum-prometheus`
// installs a process-global recorder that conflicts across parallel tests.
// An end-to-end smoke test against the running binary covers it in M7b.
