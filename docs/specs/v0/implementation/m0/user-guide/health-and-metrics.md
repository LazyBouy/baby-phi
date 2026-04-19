<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — health and metrics

`baby-phi-server` exposes three observability endpoints in M0. Liveness and readiness are distinct on purpose; metrics is Prometheus-format text.

See also the architecture details at [`../architecture/server-topology.md`](../architecture/server-topology.md) and [`../architecture/telemetry-and-metrics.md`](../architecture/telemetry-and-metrics.md).

## `/healthz/live`

**Meaning:** is the process alive and responsive?

**Response:** always `200 OK` with JSON body.

```bash
curl -i http://127.0.0.1:8080/healthz/live
```

```
HTTP/1.1 200 OK
content-type: application/json
content-length: 15

{"status":"ok"}
```

**Used by:** container orchestrators as the liveness probe. Failure (no response or non-2xx) ⇒ orchestrator **restarts the container**.

**What this tells you if it fails:** the server process is either dead, deadlocked, or not listening. Check the container status and logs.

## `/healthz/ready`

**Meaning:** can the server serve real traffic?

**Response:** `200 OK` if storage is reachable; `503 Service Unavailable` otherwise.

```bash
# Healthy
curl -i http://127.0.0.1:8080/healthz/ready
```

```
HTTP/1.1 200 OK
content-type: application/json

{"status":"ok","storage":"ok"}
```

```bash
# Unhealthy (storage failed to open, or has become unreachable)
```

```
HTTP/1.1 503 Service Unavailable
content-type: application/json

{"status":"unavailable","storage":"unreachable"}
```

**Used by:** container orchestrators as the readiness probe. Failure ⇒ orchestrator **routes traffic away** from this instance but does **not** restart it. Also used by load balancers and service meshes for the same purpose.

**What this tells you if it fails:** storage (SurrealDB embedded RocksDB) has a problem. Disk full? Permission error on `data_dir`? Corrupted DB files? Check server logs for the corresponding `tracing::error!` entry.

## `/metrics`

**Meaning:** Prometheus-format counters, histograms, and gauges for every HTTP request.

**Response:** `200 OK` with `Content-Type: text/plain; version=0.0.4`.

```bash
curl http://127.0.0.1:8080/metrics | head -30
```

Sample output (truncated):

```
# HELP axum_http_requests_duration_seconds Latency of HTTP requests
# TYPE axum_http_requests_duration_seconds histogram
axum_http_requests_duration_seconds_bucket{endpoint="/healthz/live",method="GET",status="200",le="0.005"} 3
axum_http_requests_duration_seconds_bucket{endpoint="/healthz/live",method="GET",status="200",le="0.01"} 3
…
axum_http_requests_duration_seconds_count{endpoint="/healthz/live",method="GET",status="200"} 3
axum_http_requests_duration_seconds_sum{endpoint="/healthz/live",method="GET",status="200"} 0.00064

# HELP axum_http_requests_total Total number of HTTP requests
# TYPE axum_http_requests_total counter
axum_http_requests_total{endpoint="/healthz/live",method="GET",status="200"} 3
axum_http_requests_total{endpoint="/healthz/ready",method="GET",status="200"} 2

# HELP axum_http_requests_pending Number of currently in-flight HTTP requests
# TYPE axum_http_requests_pending gauge
axum_http_requests_pending 0
```

**Used by:** a Prometheus server scraping on a fixed interval (typically 15–60 s). Configure your Prometheus with:

```yaml
# prometheus.yml snippet
scrape_configs:
  - job_name: baby-phi
    scrape_interval: 30s
    static_configs:
      - targets: ['<baby-phi-host>:8080']
    metrics_path: /metrics
```

**What this tells you:**

- Request rate (`rate(axum_http_requests_total[5m])`) by endpoint / status.
- Latency percentiles (`histogram_quantile(0.95, rate(axum_http_requests_duration_seconds_bucket[5m]))`).
- Concurrent load (`axum_http_requests_pending`).

## Cardinality

`axum-prometheus`'s `endpoint` label is the **matched route pattern**, not the raw URL. `/api/v0/agents/{id}` (when M1+ lands) appears as `endpoint="/api/v0/agents/{id}"`, not `endpoint="/api/v0/agents/abc123"`. This keeps metric cardinality bounded.

## Exposing via reverse proxy

The recommended production posture has a reverse proxy in front. `/metrics` is internal — do **not** expose it to the public internet. Scraping config for an nginx + Prometheus setup:

- `nginx` serves `https://baby-phi.example.com/` publicly, proxying to `baby-phi-server:8080`.
- `/metrics` is blocked from public ingress via `location /metrics { allow 10.0.0.0/8; deny all; }`.
- Prometheus scrapes directly against `http://baby-phi-server-internal:8080/metrics` on the internal network.

Alternative: expose `/metrics` on a second, metrics-only port bound to the internal network — `[PLANNED M7b]`.

## Observability beyond M0

| Surface | M0 | Later |
|---|---|---|
| JSON structured logs | ✓ | — |
| Prometheus metrics (HTTP-level) | ✓ | Custom domain metrics (M7) |
| OpenTelemetry traces | — | M7 |
| SLO dashboard (Grafana JSON) | — | M7b |
| Audit-log hash-chain | — | M7 |
| On-call paging integration | — | Per-deployment runbook (M7b) |

Custom domain metrics to watch for in M7 per the build plan:

- `permission_check_latency_seconds{decision, failed_step}`
- `auth_request_state_transitions_total{from, to}`
- `session_active`
- `audit_events_total{class}`
- `surreal_query_duration_seconds`
