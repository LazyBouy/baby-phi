<!-- Last verified: 2026-04-19 by Claude Code -->

# phi — M0 implementation documentation

M0 is the **scaffolding** milestone of the v0.1 build plan: a Cargo workspace with four Rust crates (`cli`, `domain`, `store`, `server`), an embedded SurrealDB (RocksDB) adapter, an axum HTTP server with health + metrics, a Next.js 14 web UI skeleton, a Dockerfile + compose stack, three CI workflows, and a 12-factor layered-config story. Nothing in M0 implements the Permission Check engine, Auth Request state machine, or any admin page — those land in M1+.

These pages describe **what M0 actually shipped**. The archived build plan at [`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../plan/build/36d0c6c5-build-plan-v01.md) describes the intent for M0–M8. The two must stay in sync — intent lives in the plan, actuality lives here.

## Layout

```
m0/
├── architecture/      "how M0 is built"
├── user-guide/        "how to run / develop on M0"
├── operations/        "how to deploy, monitor, and secure M0"
└── decisions/         ADRs — load-bearing choices and their rationale
```

## architecture/

| Page | Purpose |
|---|---|
| [overview.md](architecture/overview.md) | System map: three surfaces, shared domain + storage, dependency flow |
| [workspace-layout.md](architecture/workspace-layout.md) | Every crate, package-name-vs-binary-name, directory convention |
| [server-topology.md](architecture/server-topology.md) | axum router composition, `AppState`, health semantics, handler table |
| [storage-and-repository.md](architecture/storage-and-repository.md) | `Repository` trait boundary, `SurrealStore` adapter, scaling tiers |
| [configuration.md](architecture/configuration.md) | Layered TOML + env-var precedence, schema reference |
| [telemetry-and-metrics.md](architecture/telemetry-and-metrics.md) | `tracing` subscriber, `axum-prometheus`, global-recorder caveat |
| [web-topology.md](architecture/web-topology.md) | Next.js App Router, SSR, `/api/v0/*` proxy, auth placeholder |

## user-guide/

| Page | Purpose |
|---|---|
| [getting-started.md](user-guide/getting-started.md) | Toolchain prerequisites + first build |
| [dev-workflow.md](user-guide/dev-workflow.md) | Daily cargo / npm commands, adding a crate |
| [running-locally.md](user-guide/running-locally.md) | CLI demo, HTTP server, web dev server |
| [docker-compose.md](user-guide/docker-compose.md) | One-command local stack |
| [health-and-metrics.md](user-guide/health-and-metrics.md) | Probing `/healthz/*` and `/metrics` |
| [troubleshooting.md](user-guide/troubleshooting.md) | Known error signatures + fixes |

## operations/

| Page | Purpose |
|---|---|
| [deployment-docker.md](operations/deployment-docker.md) | Dockerfile walk-through, runtime env vars, volumes |
| [configuration-profiles.md](operations/configuration-profiles.md) | `dev` / `staging` / `prod` semantics, adding a profile |
| [tls-and-transport-security.md](operations/tls-and-transport-security.md) | Reverse-proxy pattern (recommended) + native TLS path |
| [ci-pipelines.md](operations/ci-pipelines.md) | `rust.yml`, `web.yml`, `spec-drift.yml` |
| [security-scanning.md](operations/security-scanning.md) | `cargo audit`, `cargo deny`, `npm audit` |

## decisions/

Each ADR follows the Status / Context / Decision / Consequences / Alternatives pattern.

| # | Decision |
|---|---|
| [0001](decisions/0001-surrealdb-over-memgraph.md) | SurrealDB over Memgraph as the embedded graph DB |
| [0002](decisions/0002-three-parallel-surfaces.md) | Ship CLI + HTTP API + Web UI in parallel |
| [0003](decisions/0003-modules-crates-layout.md) | `modules/crates/` + `modules/web/` directory split |
| [0004](decisions/0004-terse-package-names.md) | Package names drop the `phi-` prefix; binary names keep it |
| [0005](decisions/0005-metrics-layer-separation.md) | Separate `build_router` from `with_prometheus` |
| [0006](decisions/0006-twelve-factor-layered-config.md) | TOML layers + `PHI_*` env overrides, secrets env-only |
| [0007](decisions/0007-embedded-vs-sidecar-database.md) | Embedded SurrealDB for v0.1; standalone + TiKV as scaling tiers |

## Conventions

- Every page carries a `<!-- Last verified: YYYY-MM-DD by Claude Code -->` header.
- Feature references are status-tagged: `[EXISTS]`, `[PLANNED M<n>]`, `[CONCEPTUAL]`.
- Code claims link to file + line (e.g. [`modules/crates/server/src/health.rs`](../../../../../modules/crates/server/src/health.rs)), so docs stay discoverable as code evolves.
- Rationale-heavy claims link to the archived plan rather than re-stating it.
- Diagrams are ASCII — diff-able, dependency-free.

## Sibling milestones

M1+ implementation documentation will live in sibling folders: `m1/`, `m2/`, etc. Each follows the same four-folder shape (architecture / user-guide / operations / decisions) so readers can navigate consistently across milestones.
