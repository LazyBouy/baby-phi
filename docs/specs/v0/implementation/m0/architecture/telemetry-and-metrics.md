<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — telemetry and metrics

M0 ships a production-grade baseline for observability: structured JSON logs via `tracing`, a Prometheus `/metrics` endpoint via `axum-prometheus`, and config-driven log filtering. OpenTelemetry traces and custom domain metrics land later (M7).

## Logging

Initialised once at startup by [`telemetry::init()`](../../../../../../modules/crates/server/src/telemetry.rs):

```
┌──────────────────────────────┐
│   tracing_subscriber         │
│   ┌──────────────────────┐   │
│   │ EnvFilter            │   │   ← from cfg.telemetry.log_filter
│   │ (directive parser)   │   │     e.g. "info,server=debug"
│   └──────────────────────┘   │
│              │                │
│              ▼                │
│   ┌──────────────────────┐   │
│   │ fmt::layer()         │   │   ← cfg.telemetry.json_logs:
│   │   .json()  OR        │   │       true  → JSON to stdout
│   │   .pretty()          │   │       false → pretty to stdout
│   └──────────────────────┘   │
└──────────────────────────────┘
```

The subscriber is installed via `try_init()` which is a no-op on subsequent calls — the server only calls it once in M0; tests do not install a subscriber at all.

### Log format

- **Pretty (dev default)** — colourised, one-event-per-multi-line-block. Easy to read on a local terminal.
- **JSON (prod default)** — one JSON object per line with `timestamp`, `level`, `fields`, `target`, `span`. Directly ingestable by Loki, Elasticsearch, Cloud Logging, or anything else that speaks JSON lines.

Sample JSON line (from `BABY_PHI_PROFILE=prod cargo run -p server`):

```json
{"timestamp":"2026-04-19T14:03:17.294Z","level":"INFO","fields":{"message":"baby-phi-server listening (plaintext HTTP — terminate TLS at reverse proxy in prod)","addr":"0.0.0.0:8080"},"target":"server"}
```

### Filter directives

`cfg.telemetry.log_filter` is a standard [`tracing-subscriber` `EnvFilter`](https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/filter/struct.EnvFilter.html) directive. Examples:

| Directive | Effect |
|---|---|
| `"info"` | All `info!` and above everywhere. |
| `"info,server=debug"` | `debug!` and above in the `server` crate; `info!` elsewhere. |
| `"warn,store=trace"` | Very quiet everywhere except store, where everything is visible. |

`dev.toml` sets a verbose filter (`"info,server=debug,domain=debug,store=debug"`) so developers see internal events. Prod runs `"info"`.

## Metrics

Wired by [`with_prometheus`](../../../../../../modules/crates/server/src/router.rs) at router assembly time. This is a **production-only** code path — tests call `build_router` directly and skip metrics.

Under the hood:

1. `axum_prometheus::PrometheusMetricLayer::pair()` builds a `metrics` recorder and a `PrometheusHandle`.
2. The recorder is installed as the global `metrics::Recorder` for the process.
3. The returned `axum` layer instruments every request with:
   - `axum_http_requests_total` (counter, labeled by method / endpoint / status)
   - `axum_http_requests_duration_seconds` (histogram)
   - `axum_http_requests_pending` (gauge)
4. The `/metrics` endpoint returns the handle's rendered Prometheus-format text.

### Global-recorder caveat

`PrometheusMetricLayer::pair()` installs a **process-global** metrics recorder. Calling it twice in the same process panics:

```
Failed to build metrics recorder: FailedToCreateHTTPListener("Address already in use (os error 98)")
```
or
```
Failed to set global recorder: SetRecorderError { .. }
```

This is why the server separates [`build_router`](../../../../../../modules/crates/server/src/router.rs) (pure route wiring) from [`with_prometheus`](../../../../../../modules/crates/server/src/router.rs) (global install). Tests use `build_router`; `main.rs` uses `with_prometheus(build_router(state))`. See [ADR-0005](../decisions/0005-metrics-layer-separation.md).

### Scraping

A Prometheus server scrapes `/metrics` on a configurable cadence (typically 15–60 s). The Dockerfile exposes port 8080; operators point Prometheus at `http://<baby-phi-host>:8080/metrics`. The ingress/reverse proxy should route this path without authentication on an internal-only interface, or with a scraper-only auth token — see [`operations/tls-and-transport-security.md`](../operations/tls-and-transport-security.md).

### Cardinality

`axum-prometheus`'s default labels include `endpoint` (the matched route pattern, not the raw URL), so path parameters don't explode cardinality. When M1+ adds dynamic routes like `/api/v0/agents/{id}`, they appear as `endpoint="/api/v0/agents/{id}"` in the metrics, not `endpoint="/api/v0/agents/abc123"`. This is the correct shape.

## OpenTelemetry traces — `[PLANNED M7]`

M7 adds:

- `tracing-opentelemetry` bridging — every `tracing` span becomes an OTEL span.
- OTLP exporter → collector (configurable endpoint via env var).
- Sample JSON of a bootstrap → org-create → session-launch trace checked in as a fixture.

Current `tracing` instrumentation is already structured-span-shaped (every request carries an implicit request-id span via tower-http). M7's upgrade is wiring, not instrumentation.

## Domain metrics — `[PLANNED M7]`

The plan commits custom metrics in M7, per [`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../../plan/build/36d0c6c5-build-plan-v01.md) §M7 and §NFR-observability. Expected additions include:

- `permission_check_latency_seconds` (histogram, labeled by decision and failed_step).
- `auth_request_state_transitions_total` (counter, labeled by from-state and to-state).
- `session_active` (gauge).
- `audit_events_total` (counter, labeled by event class).
- `surreal_query_duration_seconds` (histogram).

These are declared in `domain` and `store` via the `metrics` crate's recording macros (already present in the workspace dependencies). Because the recorder is process-global, the macros Just Work once `with_prometheus` has installed it.
