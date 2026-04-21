use axum::{
    routing::{get, post},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use tower_cookies::CookieManagerLayer;

use crate::{handlers, health, state::AppState};

/// Build the base application router. This is the routes an **integration
/// test** exercises — it does NOT install a global Prometheus recorder, so
/// many tests can construct their own app in parallel.
///
/// Mounts:
/// - `/healthz/live`                                  (M0)
/// - `/healthz/ready`                                 (M0)
/// - `GET  /api/v0/bootstrap/status`                  (M1/P6)
/// - `POST /api/v0/bootstrap/claim`                   (M1/P6)
/// - `GET  /api/v0/platform/secrets`                  (M2/P4)
/// - `POST /api/v0/platform/secrets`                  (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/rotate`    (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/reveal`    (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/reassign-custody` (M2/P4)
///
/// The `CookieManagerLayer` is applied once here so every handler that
/// pulls `Cookies` from the extractor gets a working jar.
pub fn build_router(state: AppState) -> Router {
    let api_v0 = Router::new()
        .route("/bootstrap/status", get(handlers::bootstrap::status))
        .route("/bootstrap/claim", post(handlers::bootstrap::claim))
        .route(
            "/platform/secrets",
            get(handlers::platform_secrets::list).post(handlers::platform_secrets::add),
        )
        .route(
            "/platform/secrets/:slug/rotate",
            post(handlers::platform_secrets::rotate),
        )
        .route(
            "/platform/secrets/:slug/reveal",
            post(handlers::platform_secrets::reveal),
        )
        .route(
            "/platform/secrets/:slug/reassign-custody",
            post(handlers::platform_secrets::reassign),
        );

    Router::new()
        .route("/healthz/live", get(health::live))
        .route("/healthz/ready", get(health::ready))
        .nest("/api/v0", api_v0)
        .layer(CookieManagerLayer::new())
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
