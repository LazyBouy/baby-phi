use axum::{
    routing::{get, patch, post},
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
/// - `/healthz/live`                                        (M0)
/// - `/healthz/ready`                                       (M0)
/// - `GET  /api/v0/bootstrap/status`                        (M1/P6)
/// - `POST /api/v0/bootstrap/claim`                         (M1/P6)
/// - `GET  /api/v0/platform/secrets`                        (M2/P4)
/// - `POST /api/v0/platform/secrets`                        (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/rotate`          (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/reveal`          (M2/P4)
/// - `POST /api/v0/platform/secrets/{slug}/reassign-custody`(M2/P4)
/// - `GET  /api/v0/platform/model-providers`                (M2/P5)
/// - `POST /api/v0/platform/model-providers`                (M2/P5)
/// - `POST /api/v0/platform/model-providers/{id}/archive`   (M2/P5)
/// - `GET  /api/v0/platform/provider-kinds`                 (M2/P5)
/// - `GET   /api/v0/platform/mcp-servers`                    (M2/P6)
/// - `POST  /api/v0/platform/mcp-servers`                    (M2/P6)
/// - `PATCH /api/v0/platform/mcp-servers/{id}/tenants`       (M2/P6)
/// - `POST  /api/v0/platform/mcp-servers/{id}/archive`       (M2/P6)
/// - `GET   /api/v0/platform/defaults`                        (M2/P7)
/// - `PUT   /api/v0/platform/defaults`                        (M2/P7)
/// - `POST  /api/v0/orgs`                                     (M3/P4)
/// - `GET   /api/v0/orgs`                                     (M3/P4)
/// - `GET   /api/v0/orgs/:id`                                 (M3/P4)
/// - `GET   /api/v0/orgs/:id/dashboard`                       (M3/P5)
/// - `GET   /api/v0/orgs/:org_id/agents`                      (M4/P4)
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
        )
        .route(
            "/platform/model-providers",
            get(handlers::platform_model_providers::list)
                .post(handlers::platform_model_providers::register),
        )
        .route(
            "/platform/model-providers/:id/archive",
            post(handlers::platform_model_providers::archive),
        )
        .route(
            "/platform/provider-kinds",
            get(handlers::platform_model_providers::provider_kinds),
        )
        .route(
            "/platform/mcp-servers",
            get(handlers::platform_mcp_servers::list)
                .post(handlers::platform_mcp_servers::register),
        )
        .route(
            "/platform/mcp-servers/:id/tenants",
            patch(handlers::platform_mcp_servers::patch_tenants),
        )
        .route(
            "/platform/mcp-servers/:id/archive",
            post(handlers::platform_mcp_servers::archive),
        )
        .route(
            "/platform/defaults",
            get(handlers::platform_defaults::get).put(handlers::platform_defaults::put),
        )
        .route(
            "/orgs",
            get(handlers::orgs::list).post(handlers::orgs::create),
        )
        .route("/orgs/:id", get(handlers::orgs::show))
        .route("/orgs/:id/dashboard", get(handlers::orgs::dashboard))
        .route("/orgs/:org_id/agents", get(handlers::agents::list));

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
