<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0001: SurrealDB over Memgraph

## Status
Accepted — 2026-04-19 (M0).

## Context

phi's domain model is a typed graph (nodes + edges from [`../../../concepts/ontology.md`](../../../concepts/ontology.md)) with document-shaped payloads on nodes (e.g. agent profiles carry nested phi-core structs) and time-series audit events. v0.1 needed an embedded-friendly graph-capable database that could scale from a single-machine deploy to a clustered one without rewriting queries.

The two finalists under "embedded graph DB" in the build plan were **SurrealDB** and **Memgraph**.

## Decision

**Use SurrealDB with the RocksDB backend, embedded in the `phi-server` process.**

See the build plan's comparison table at [`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../../plan/build/36d0c6c5-build-plan-v01.md) §"Storage choice: SurrealDB vs Memgraph" for the full matrix.

## Consequences

### Positive

- **Embedded in-process.** `surrealdb = "2.x"` with `kv-rocksdb` feature — no sidecar DB container. Local dev is `cargo run`.
- **Rust-native.** Zero FFI; Rust types + `serde` flow naturally into and out of SurrealQL. Memgraph is C++, reached via the Bolt network protocol even for local dev.
- **Multi-model fit.** SurrealDB handles documents (agent profiles), graph edges (MEMBER_OF, HAS_LEAD, DESCENDS_FROM), and time-series (audit events) in one store. Memgraph is graph-only; document fields would be encoded as property maps and time-series would need a separate store.
- **Scaling path with zero query rewrites.** Embedded RocksDB → standalone SurrealDB server → TiKV cluster share the same SurrealQL. Only the connection string changes.
- **Migrating out is bounded.** SurrealQL is SQL-flavoured; if SurrealDB is ever the wrong tool, the `store` crate is the sole adapter to rewrite.

### Negative

- **Youth.** SurrealDB v2 is less battle-tested at scale than Memgraph. v0 target (one org, ~50 agents, ~10 projects) is well within SurrealDB's proven envelope.
- **SurrealQL learning curve.** A custom query language; ~1 day ramp for a developer comfortable with SQL.
- **Backup/restore tooling is less mature** than Postgres/MySQL. M7 writes a scheduled `surreal export` + restore script; full drill lands in M7b.
- **Query optimiser is less mature.** Complex multi-hop traversals may need hand-tuning. v0's traversals are shallow (authority chains ≤6 hops) — not a near-term blocker.
- **Fewer third-party tools** (visualisers, GUIs) than Memgraph. Not critical; the admin UI is our primary inspection surface.

## Alternatives considered

- **Memgraph.** Stronger graph analytics (PageRank, community detection) out of the box. v0 does not need those — the traversal patterns we need ("walk DESCENDS_FROM to root", "list grants matching a selector") are trivial in SurrealQL. Operational overhead of a second process tips the balance to SurrealDB.
- **SQLite** (the build plan's original default before the decision to go graph-native). Rejected because the graph model is first-class in phi's concept docs; encoding edges as join tables would drag query complexity into the domain layer and obscure the ontology.
- **Postgres + Apache AGE extension.** Mature substrate, graph-capable via AGE. Rejected for v0.1: operational complexity of Postgres + AGE exceeds SurrealDB's embedded-single-binary story, and AGE is a Cypher wrapper so we'd still pay a query-language learning cost.
- **Neo4j.** License constraints (GPLv3 community edition or commercial) complicate redistribution. Server-only, similar ergonomics to Memgraph.

## Safety valve

If v1 scale or usage patterns reveal SurrealDB's weaknesses are load-bearing for us, the `store` crate (the sole adapter) is rewritten against a different backend and the domain crate is unchanged. This is a deliberate, documented escape hatch. See [ADR-0007](0007-embedded-vs-sidecar-database.md) for the embedded-vs-sidecar sub-decision.
