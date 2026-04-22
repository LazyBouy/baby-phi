<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — overview

M0 establishes the load-bearing shape of the phi platform. Everything in M1+ plugs into this skeleton; nothing in M0 yet implements a user-visible platform feature.

## System map

```
                    ┌──────────────────┐     ┌──────────────────┐
                    │  CLI  (phi) │     │  Web (Next.js 14)│
                    │  modules/crates/ │     │  modules/web/    │
                    │  cli             │     │                  │
                    └────────┬─────────┘     └────────┬─────────┘
                             │                        │
                             │ HTTP/JSON              │ HTTP/JSON (SSR-proxied via
                             │                        │  next.config.mjs rewrites
                             │                        │  /api/v0/* → PHI_API_URL)
                             ▼                        ▼
                    ┌───────────────────────────────────────────┐
                    │       HTTP API (axum, phi-server)    │
                    │       modules/crates/server               │
                    │                                           │
                    │  /healthz/live   /healthz/ready           │
                    │  /metrics  (Prometheus exposition)        │
                    │  /api/v0/* [PLANNED M1+]                  │
                    └────────────────────┬──────────────────────┘
                                         │
                                         ▼
                    ┌───────────────────────────────────────────┐
                    │          domain   (Rust library)          │
                    │          modules/crates/domain            │
                    │                                           │
                    │  • Repository trait (ping; grows in M1+)  │
                    │  • Permission Check engine [PLANNED M1]   │
                    │  • State machines [PLANNED M1/M7]         │
                    │  • Graph model [PLANNED M1]               │
                    └────────────────────┬──────────────────────┘
                                         │
                                         ▼
                    ┌───────────────────────────────────────────┐
                    │   store (Rust library, SurrealDB adapter) │
                    │   modules/crates/store                    │
                    │   impl Repository for SurrealStore { … }  │
                    └────────────────────┬──────────────────────┘
                                         │
                                         ▼
                    ┌───────────────────────────────────────────┐
                    │        SurrealDB (embedded, RocksDB)      │
                    │        process-local; file-backed         │
                    └───────────────────────────────────────────┘
```

In the default deployment the CLI, server, and SurrealDB all live in the same process tree; the web UI is a separate Next.js process. The web UI never talks to SurrealDB directly — it is always a client of the HTTP API, by design (see [ADR-0002](../decisions/0002-three-parallel-surfaces.md)).

## Dependency flow rule

Strict, downward only:

```
cli ────┐
        ├──▶ domain ──▶ store ──▶ SurrealDB
server ─┤              (plus phi-core for agent/session types)
        │
web ────┘  (HTTP-only; no direct Rust-crate dependency)
```

Enforcement is by construction — the workspace manifest grants `domain` no path dependency on `store`, `store` no dependency on `server`, and so on. Breaking the rule would require editing a Cargo.toml, which code review catches.

See [workspace-layout.md](workspace-layout.md) for per-crate detail.

## What exists in M0

Concrete, code-backed:

- Four Rust crates in a workspace (`cli`, `domain`, `store`, `server`).
- `Repository::ping()` as the only domain-side contract — verified by the real SurrealDB backend and by a fake in tests.
- axum HTTP server with three routes: `/healthz/live`, `/healthz/ready`, `/metrics` (production-only).
- Native TLS via `axum-server` when `[server.tls]` is set; plaintext otherwise.
- Next.js 14 App Router + SSR + a health-probe page.
- Structured logs via `tracing` (JSON in prod, pretty in dev).
- 12-factor layered config: `config/default.toml` → `config/<profile>.toml` → `PHI_*` env.

## What does NOT exist yet in M0

Tagged throughout these docs as `[PLANNED M<n>]`:

- `/api/v0/*` endpoints (bootstrap, orgs, agents, projects, grants, sessions, auth-requests).
- Permission Check engine + Auth Request state machine (M1).
- OAuth 2.0 user login (M3).
- Audit-log hash-chain + OpenTelemetry traces (M7).
- Hardened production posture — load tests, chaos, backup drill, runbook (M7b).

## Reading order

For a first-time reader:

1. [workspace-layout.md](workspace-layout.md) — what the directory tree contains.
2. [server-topology.md](server-topology.md) — how the HTTP server composes routes.
3. [storage-and-repository.md](storage-and-repository.md) — the persistence boundary.
4. [configuration.md](configuration.md) — how config is loaded + overridden.
5. [telemetry-and-metrics.md](telemetry-and-metrics.md) — observability baseline.
6. [web-topology.md](web-topology.md) — Next.js integration.

For rationale-level questions, jump to [../decisions/](../decisions/).
