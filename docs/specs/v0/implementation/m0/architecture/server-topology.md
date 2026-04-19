<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — server topology

The HTTP server is an axum application assembled in two stages: a base router that tests can exercise cheaply, and a production wrapper that adds Prometheus metrics. `main.rs` wires storage, telemetry, and the listener (plaintext or TLS).

## File map

| File | Role |
|---|---|
| [`modules/crates/server/src/lib.rs`](../../../../../../modules/crates/server/src/lib.rs) | Public library surface: `ServerConfig`, `AppState`, `build_router`, `with_prometheus`, `telemetry` |
| [`modules/crates/server/src/router.rs`](../../../../../../modules/crates/server/src/router.rs) | `build_router` (base) + `with_prometheus` (wrapper) |
| [`modules/crates/server/src/health.rs`](../../../../../../modules/crates/server/src/health.rs) | `/healthz/live` + `/healthz/ready` handlers |
| [`modules/crates/server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs) | `AppState { repo: Arc<dyn Repository> }` |
| [`modules/crates/server/src/config.rs`](../../../../../../modules/crates/server/src/config.rs) | Config loader; see [configuration.md](configuration.md) |
| [`modules/crates/server/src/telemetry.rs`](../../../../../../modules/crates/server/src/telemetry.rs) | `tracing` subscriber init; see [telemetry-and-metrics.md](telemetry-and-metrics.md) |
| [`modules/crates/server/src/main.rs`](../../../../../../modules/crates/server/src/main.rs) | `#[tokio::main]` — load config, open store, serve plaintext or TLS |

## Two-layer router

Implemented at [`router.rs:12-17`](../../../../../../modules/crates/server/src/router.rs) and [`router.rs:25-33`](../../../../../../modules/crates/server/src/router.rs):

```text
build_router(state)       ← base: pure route wiring, no metrics install
    │
    ├─ GET /healthz/live   handler health::live
    └─ GET /healthz/ready  handler health::ready
    with_state(state)

with_prometheus(router)   ← production wrapper (call once per process)
    │
    ├─ <router from build_router>
    ├─ GET /metrics        renders PrometheusMetricLayer handle
    └─ .layer(prometheus_layer)  instrument every request
```

`PrometheusMetricLayer::pair()` installs a **process-global** `metrics` recorder. Calling `with_prometheus` twice in one process panics; calling it across parallel integration tests panics with `Address already in use` or `SetRecorderError`. Tests therefore use `build_router` directly and skip metrics assertions. See [ADR-0005](../decisions/0005-metrics-layer-separation.md).

## Handler surface (M0)

| Method | Path | Handler | Input | Output | Status codes |
|---|---|---|---|---|---|
| GET | `/healthz/live` | [`health::live`](../../../../../../modules/crates/server/src/health.rs) | — | `{ "status": "ok" }` | **200** always |
| GET | `/healthz/ready` | [`health::ready`](../../../../../../modules/crates/server/src/health.rs) | `State<AppState>` | `{ "status": "ok", "storage": "ok" }` or `{ "status": "unavailable", "storage": "unreachable" }` | **200** if `repo.ping()` succeeds; **503** otherwise |
| GET | `/metrics` | inline closure in `with_prometheus` | — | Prometheus text exposition | **200** |

Everything else under `/api/v0/*` is `[PLANNED M1+]` and will be added as each admin page lands.

## State injection

`AppState` at [`state.rs`](../../../../../../modules/crates/server/src/state.rs):

```rust
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
}
```

Repository is a trait object so integration tests can swap in an in-memory fake without touching handler code. The `health_test.rs` suite does exactly this with a `FakeRepo { healthy: bool }` — see [`modules/crates/server/tests/health_test.rs`](../../../../../../modules/crates/server/tests/health_test.rs).

In production, `main.rs` constructs `AppState { repo: Arc::new(SurrealStore::open_embedded(...).await?) }`. Any handler that writes (M1+) will `.clone()` the `Arc` and call trait methods on it.

## Listener selection (TLS vs plaintext)

`main.rs` at [`main.rs:33-55`](../../../../../../modules/crates/server/src/main.rs) branches on `cfg.server.tls`:

```
cfg.server.tls = Some(tls)   →  axum_server::bind_rustls(addr, RustlsConfig::from_pem_file(tls.cert_path, tls.key_path))
                                  .serve(app.into_make_service())

cfg.server.tls = None        →  tokio::net::TcpListener::bind(addr)
                                  .and_then(|l| axum::serve(l, app))
```

The recommended production posture is **plaintext on an internal interface with TLS terminated at a reverse proxy** — native TLS is provided as an escape hatch for simple single-node deploys. See [tls-and-transport-security.md](../operations/tls-and-transport-security.md) and [ADR-0002](../decisions/0002-three-parallel-surfaces.md).

## Health semantics

- **Liveness (`/healthz/live`)** — "is the process running?" Returns 200 as long as the event loop can respond. Orchestrators (Docker, Kubernetes) restart a container whose liveness probe fails for N consecutive checks.
- **Readiness (`/healthz/ready`)** — "can I serve traffic?" Probes the storage backend via `Repository::ping()`. Orchestrators route requests away from instances whose readiness probe fails, but do not restart them.

The M0 split exists so rolling deploys do the right thing from day one. The Dockerfile HEALTHCHECK at [`Dockerfile`](../../../../../../Dockerfile) hits `/healthz/ready` — if storage is broken, the container is marked unhealthy and replaced.

In M7b, `/healthz/ready` will also verify migration state and any dependency we add in M1+ — the split stays the same, the predicates grow.

## Test coverage (M0)

[`tests/health_test.rs`](../../../../../../modules/crates/server/tests/health_test.rs) covers:

- `live_is_always_ok` — liveness returns 200 even when the fake repo is unhealthy.
- `ready_reports_storage_up` — ready returns 200 when ping succeeds.
- `ready_reports_storage_down` — ready returns 503 when ping errors.

[`tests/tls_test.rs`](../../../../../../modules/crates/server/tests/tls_test.rs) covers the TLS listener end-to-end: rcgen-generated self-signed cert, `axum_server::bind_rustls`, HTTPS request via reqwest succeeds, plaintext HTTP against the same port fails with a protocol error.
