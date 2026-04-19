<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0007: Embedded SurrealDB for v0.1; standalone + TiKV as scaling tiers

## Status
Accepted — 2026-04-19 (M0).

## Context

[ADR-0001](0001-surrealdb-over-memgraph.md) chose SurrealDB as the database. Separately, we need to decide **how** to deploy it for v0.1:

- **Embedded** — SurrealDB runs as a Rust library inside `baby-phi-server`'s process, using the RocksDB backend to write an on-disk file tree.
- **Sidecar** — SurrealDB runs as a separate process (standalone server mode), and `baby-phi-server` connects over a WebSocket or HTTP client.

This decision is separate from the database choice itself because SurrealDB supports both modes with the same query language; only the connection setup differs.

## Decision

**Ship embedded (in-process) for v0.1. Document the migration path to standalone-server and TiKV-cluster deployments without query-code changes.**

v0.1 `config/default.toml`:
```toml
[storage]
data_dir = "data/baby-phi.db"
namespace = "baby-phi"
database = "v0"
```

Implementation at [`modules/crates/store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs):
```rust
Surreal::new::<RocksDb>(path).await?;
```

## Consequences

### Positive

- **One binary, one process, one volume.** Local dev is `cargo run -p server` — no docker-compose required. Production deploys are a single container with a single PVC. Backup is "tar up `data_dir`".
- **Zero network hop** between server and DB. Every query is an in-process Rust function call; no serialization overhead, no connection-pool tuning, no DNS lookups.
- **Simpler failure modes.** If the DB is unreachable, the server is also down — there's no split-brain scenario where the API is up but the DB is gone.
- **Migration path is clean.** Moving to standalone-server is a connection-string change; SurrealQL and every domain-level query are unchanged.

### Negative

- **Cannot scale `baby-phi-server` horizontally.** Multiple server processes would each own a private RocksDB file; they'd be isolated databases. Horizontal scaling requires moving to standalone or TiKV.
- **Single point of failure.** Process crash = DB unavailable until restart. For v0.1's target (single-tenant, one org, internal use) this is acceptable; HA is a v0.2+ concern.
- **Resource contention.** The server process and the DB share CPU, memory, and the same event loop for I/O. For v0's load (~50 agents, ~10 projects, ≤100 concurrent sessions) this is fine; comfortable ceiling per build-plan §"Scaling path" is ~100k nodes / 1M edges.
- **Harder to pin DB version separately.** The embedded crate version is locked to the server build. Upgrading SurrealDB requires re-compiling and re-deploying the server.

## Alternatives considered

- **Standalone from day one.** Valid; industry-typical. Rejected for v0 because the operational complexity (second container, network config, connection health monitoring, cert management if encrypted in-transit) is wasted effort at single-tenant scale. We get the same effect by being ready to move to standalone when needed.
- **Embed forever, avoid sidecar tier.** Rejected — some deployments will need horizontal scale. The `store` crate's use of SurrealDB's dual-mode API keeps the door open.
- **SQLite (embedded) instead of SurrealDB (embedded).** Rejected — SQLite is not a graph database; the graph model would leak into every query. See ADR-0001.
- **Run SurrealDB in a sidecar container via docker-compose in dev.** Rejected — adds infrastructure-knowledge overhead for every contributor just to run the server locally. `cargo run -p server` is the dev experience we want.

## Scaling tiers (migration path)

All three tiers share the same SurrealQL and the same `Repository` trait implementation — switching tiers changes only the connection setup and deployment topology.

| Tier | When | Connection | Ops cost |
|---|---|---|---|
| **Embedded + RocksDB** (v0.1) | Single-tenant, ~50 agents, ~10 projects, ≤100 concurrent sessions | `Surreal::new::<RocksDb>("data/baby-phi.db")` | Zero extra infra. |
| **Standalone SurrealDB server** | Need horizontal `baby-phi-server` scale, or independent DB lifecycle | `surreal start --bind 0.0.0.0:8000 file:…` + `Surreal::new::<Client>("ws://host:8000")` | One extra process/container. Day of ops work. |
| **SurrealDB cluster with TiKV** | Multi-region, HA, >100 GB data | TiKV cluster backs SurrealDB | Standard distributed-system ops. |

The build plan at [`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../../plan/build/36d0c6c5-build-plan-v01.md) §"Scaling path" documents each tier's tradeoffs in more depth.

## Follow-up milestones

- **M7** — `surreal export` backup tooling + restore script (embedded mode only for v0.1).
- **M7b** — backup & restore drill on populated DB; verified equivalence after restore.
- **v0.2** — evaluate standalone-server tier based on real load data from v0.1 production.
- **v1** — if usage reveals the need, TiKV cluster deployment story.

The migration commitment is explicit in the build plan: if embedded isn't enough, we move to standalone before considering a non-SurrealDB target. That's a "free to bounded-effort" spectrum, which is the right shape for a v0 storage decision.
