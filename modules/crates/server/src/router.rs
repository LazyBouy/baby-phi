use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;

use crate::{health, state::AppState};

/// Build the base application router. This is the routes an **integration
/// test** exercises — it does NOT install a global Prometheus recorder, so
/// many tests can construct their own app in parallel.
///
/// API endpoints (bootstrap, orgs, agents, projects, grants, sessions,
/// auth-requests) are added in subsequent milestones.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz/live", get(health::live))
        .route("/healthz/ready", get(health::ready))
        .with_state(state)
}

/// Wrap a router with the Prometheus metrics layer and expose `/metrics`.
///
/// Call this **once** from the production binary. `axum-prometheus` installs
/// a process-global recorder; calling this twice in a single process (or
/// concurrently from multiple tests) will panic with "Address already in use"
/// or `SetRecorderError`.
pub fn with_prometheus(router: Router) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    router
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
}
