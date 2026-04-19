use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct LiveResponse {
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub storage: &'static str,
}

/// Liveness: is the process up? Always 200 OK when reachable.
pub async fn live() -> Json<LiveResponse> {
    Json(LiveResponse { status: "ok" })
}

/// Readiness: can we serve traffic? Probes the storage backend.
pub async fn ready(
    State(state): State<AppState>,
) -> Result<Json<ReadyResponse>, (StatusCode, Json<ReadyResponse>)> {
    match state.repo.ping().await {
        Ok(()) => Ok(Json(ReadyResponse {
            status: "ok",
            storage: "ok",
        })),
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "unavailable",
                storage: "unreachable",
            }),
        )),
    }
}
