//! Smoke test for the server's health endpoints with a faked Repository.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use domain::repository::{Repository, RepositoryError, RepositoryResult};
use server::{build_router, AppState};
use tower::ServiceExt;

struct FakeRepo {
    healthy: bool,
}

#[async_trait]
impl Repository for FakeRepo {
    async fn ping(&self) -> RepositoryResult<()> {
        if self.healthy {
            Ok(())
        } else {
            Err(RepositoryError::Backend("fake down".into()))
        }
    }
}

fn app(healthy: bool) -> axum::Router {
    build_router(AppState {
        repo: Arc::new(FakeRepo { healthy }),
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
