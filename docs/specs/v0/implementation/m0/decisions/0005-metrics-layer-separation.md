<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0005: Separate `build_router` from `with_prometheus`

## Status
Accepted — 2026-04-19 (M0).

## Context

The M0 axum server exposes a Prometheus `/metrics` endpoint wired via `axum-prometheus` 0.7. Initial implementation baked the metrics layer straight into `build_router`:

```rust
pub fn build_router(state: AppState) -> Router {
    let (prom_layer, metric_handle) = PrometheusMetricLayer::pair();
    Router::new()
        .route("/healthz/live", get(health::live))
        .route("/healthz/ready", get(health::ready))
        .route("/metrics", get(move || async move { metric_handle.render() }))
        .layer(prom_layer)
        .with_state(state)
}
```

This caused three of the four integration tests to fail on the second test run with:

```
Failed to build metrics recorder: FailedToCreateHTTPListener("Address already in use (os error 98)")
Failed to set global recorder: SetRecorderError { .. }
```

Root cause: `PrometheusMetricLayer::pair()` registers a **process-global** `metrics` recorder via `metrics::set_global_recorder`, which panics if called twice. Each integration test that constructed an app called `pair()`, and the `cargo test` harness runs tests in parallel by default — the first test wins, the rest panic.

The global-recorder design is the library's choice, not ours: `axum-prometheus` deliberately uses a process-level singleton so any piece of code in any crate can record a metric that shows up in `/metrics`. That design is correct for production; it just doesn't compose well with parallel tests.

## Decision

**Split router assembly into two functions:**

- `build_router(state) -> Router` in [`router.rs`](../../../../../../modules/crates/server/src/router.rs) — pure route wiring. No global installs. Tests use this.
- `with_prometheus(router) -> Router` in [`router.rs`](../../../../../../modules/crates/server/src/router.rs) — adds the metrics layer and `/metrics` route. Call exactly once per process. The production binary uses it via `with_prometheus(build_router(state))` in [`main.rs`](../../../../../../modules/crates/server/src/main.rs).

## Consequences

### Positive

- **Tests can run in parallel.** The four integration tests (3 health + 1 TLS) pass reliably without `--test-threads=1`.
- **Production wiring is explicit.** Reading `main.rs` makes it immediately clear that metrics are a production concern, not a core-server concern. The separation documents the global-recorder caveat in the type-level structure.
- **Easier to swap out.** A future milestone that replaces `axum-prometheus` with OpenTelemetry metrics-export changes only `with_prometheus`; `build_router` is untouched. This matters for M7 when custom domain metrics arrive.

### Negative

- **Two functions to remember.** Callers that add routes in M1+ must add them to `build_router`, not `with_prometheus`. Unclear to a new contributor at first glance; mitigated by the doc comments on both functions and this ADR.
- **`/metrics` is invisible in tests.** The route is only defined in `with_prometheus`; tests never see it. Acceptable — no test currently asserts `/metrics` content, and a standalone smoke test against the running binary covers it post-M7b.
- **Two router layers instead of one.** Negligible overhead (one additional `tower::Layer` clone on startup).

## Alternatives considered

- **Use `std::sync::Once` to call `PrometheusMetricLayer::pair()` once.** Rejected because the first test to call it captures the handle, but subsequent tests don't see the metrics layer on their routers. Results in inconsistent state across tests.
- **Force `--test-threads=1`.** Serialising tests masks the issue without fixing it; long test suites become slow. Rejected.
- **Fork `axum-prometheus` to not use a global recorder.** Massive yak-shave for a problem that's clean to solve at the application boundary.
- **Use a different metrics library** (e.g. `prometheus` crate with per-registry handles). Rejected for v0 — `axum-prometheus` is the best-maintained axum-specific option, and the global-recorder pattern is actually correct for production.
- **Install the global recorder in `main` once, have `build_router` attach the layer only.** Viable but more moving parts — the layer and the `/metrics` handler need to share the handle returned by `pair()`, so pair() still needs to be called somewhere that has lifecycle ownership. `with_prometheus` owns that lifecycle cleanly.

## How this appears

In [`router.rs:12-17`](../../../../../../modules/crates/server/src/router.rs):
```rust
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz/live", get(health::live))
        .route("/healthz/ready", get(health::ready))
        .with_state(state)
}
```

In [`router.rs:25-33`](../../../../../../modules/crates/server/src/router.rs):
```rust
pub fn with_prometheus(router: Router) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    router
        .route("/metrics", get(move || async move { metric_handle.render() }))
        .layer(prometheus_layer)
}
```

In [`main.rs`](../../../../../../modules/crates/server/src/main.rs):
```rust
let app = with_prometheus(build_router(state));
```

In [`tests/health_test.rs`](../../../../../../modules/crates/server/tests/health_test.rs):
```rust
fn app(healthy: bool) -> axum::Router {
    build_router(AppState { repo: Arc::new(FakeRepo { healthy }) })
}
```
