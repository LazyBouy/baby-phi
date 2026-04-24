use axum::{
    routing::{delete, get, patch, post},
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
/// - `POST  /api/v0/orgs/:org_id/agents`                      (M4/P5)
/// - `PATCH /api/v0/agents/:id/profile`                       (M4/P5)
/// - `DELETE /api/v0/agents/:id/execution-limits-override`    (M4/P5)
/// - `POST  /api/v0/orgs/:org_id/projects`                    (M4/P6)
/// - `POST  /api/v0/projects/_pending/:ar_id/approve`         (M4/P6)
/// - `GET   /api/v0/projects/:id`                              (M4/P7)
/// - `PATCH /api/v0/projects/:id/okrs`                         (M4/P7)
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
        .route(
            "/orgs/:org_id/agents",
            get(handlers::agents::list).post(handlers::agents::create),
        )
        .route("/agents/:id/profile", patch(handlers::agents::update))
        .route(
            "/agents/:id/execution-limits-override",
            delete(handlers::agents::revert_execution_limits_override),
        )
        .route("/orgs/:org_id/projects", post(handlers::projects::create))
        .route(
            "/projects/_pending/:ar_id/approve",
            post(handlers::projects::approve_pending),
        )
        .route("/projects/:id", get(handlers::projects::show))
        .route("/projects/:id/okrs", patch(handlers::projects::update_okrs))
        // M5/P4 — session surface (6 routes).
        .route(
            "/orgs/:org_id/projects/:project_id/sessions",
            post(handlers::sessions::launch),
        )
        .route(
            "/orgs/:org_id/projects/:project_id/sessions/preview",
            post(handlers::sessions::preview),
        )
        .route(
            "/projects/:project_id/sessions",
            get(handlers::sessions::list_in_project),
        )
        .route("/sessions/:id", get(handlers::sessions::show))
        .route(
            "/sessions/:id/terminate",
            post(handlers::sessions::terminate),
        )
        .route("/sessions/:id/tools", get(handlers::sessions::tools))
        // M5/P5 — authority-template surface (5 routes).
        .route(
            "/orgs/:org_id/authority-templates",
            get(handlers::templates::list),
        )
        .route(
            "/orgs/:org_id/authority-templates/:kind/approve",
            post(handlers::templates::approve),
        )
        .route(
            "/orgs/:org_id/authority-templates/:kind/deny",
            post(handlers::templates::deny),
        )
        .route(
            "/orgs/:org_id/authority-templates/:kind/adopt",
            post(handlers::templates::adopt),
        )
        .route(
            "/orgs/:org_id/authority-templates/:kind/revoke",
            post(handlers::templates::revoke),
        )
        // M5/P6 — system-agent surface (5 routes).
        .route(
            "/orgs/:org_id/system-agents",
            get(handlers::system_agents::list).post(handlers::system_agents::add),
        )
        .route(
            "/orgs/:org_id/system-agents/:agent_id",
            patch(handlers::system_agents::tune),
        )
        .route(
            "/orgs/:org_id/system-agents/:agent_id/disable",
            post(handlers::system_agents::disable),
        )
        .route(
            "/orgs/:org_id/system-agents/:agent_id/archive",
            post(handlers::system_agents::archive),
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
