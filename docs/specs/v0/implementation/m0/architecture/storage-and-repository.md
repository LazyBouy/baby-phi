<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — storage and repository

M0 establishes a clean persistence boundary: the `domain` crate owns a `Repository` trait, and the `store` crate provides the sole implementation against an embedded SurrealDB instance backed by RocksDB. The server only ever talks to the trait — the concrete store is pluggable.

## Boundary

```
 ┌─────────────────┐    trait   ┌─────────────────────┐
 │  server, cli    │  ────────▶ │  domain::Repository │
 │  (M1+ handlers) │            └──────────┬──────────┘
 └─────────────────┘                       │ impl
                                           ▼
                                  ┌─────────────────────┐
                                  │  store::SurrealStore│
                                  │  (Surreal + RocksDB)│
                                  └──────────┬──────────┘
                                             │
                                             ▼
                                     ┌──────────────┐
                                     │  RocksDB     │
                                     │  on-disk     │
                                     │  file tree   │
                                     └──────────────┘
```

## The trait

Defined at [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs):

```rust
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Readiness check used by `/healthz/ready`. Returns Ok(()) if the
    /// backend is reachable and usable.
    async fn ping(&self) -> RepositoryResult<()>;
}
```

M0 ships exactly one method. Each milestone adds more:

| Milestone | Methods added | Motivation |
|---|---|---|
| M1 | `write_bootstrap_credential`, `claim_bootstrap_credential`, `create_platform_admin`, `list_grants_for_principal`, `record_audit_event` | Bootstrap flow + Permission Check engine |
| M2 | model-provider, MCP-server, credentials-vault CRUD | Platform-setup admin pages 02–05 |
| M3 | org + system-agent + adoption auth-request CRUD | Organization creation wizard |
| M4 | agent, project, grant, edge CRUD | Agents + Projects pages 08–11 |
| M5 | session + memory + catalog CRUD | First-session launch |

Each addition lands as part of the milestone that needs it; no method is added speculatively.

## The implementation

At [`modules/crates/store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs):

```rust
pub struct SurrealStore {
    db: Surreal<Db>,
}

impl SurrealStore {
    pub async fn open_embedded(
        path: impl AsRef<Path>,
        namespace: &str,
        database: &str,
    ) -> Result<Self, StoreError> { ... }
}

#[async_trait]
impl Repository for SurrealStore {
    async fn ping(&self) -> RepositoryResult<()> {
        self.db.health().await
            .map_err(|e| RepositoryError::Backend(e.to_string()))
    }
}
```

`open_embedded` calls `Surreal::new::<RocksDb>(path).await`, then `use_ns(namespace).use_db(database).await`. The client is held by value and cloned where needed (SurrealDB's `Surreal<T>` is cheap to clone — it's a reference handle).

`client()` exposes the raw `Surreal<Db>` for migrations and ad-hoc queries in M1+. Domain-layer code goes through the trait; the escape hatch is documented as for schema work only.

## Why embedded

v0.1 ships SurrealDB **in the same process** as `baby-phi-server`. No sidecar DB container, no network hop. The RocksDB backend writes an on-disk file tree under `storage.data_dir` (default `data/baby-phi.db`, prod `/var/lib/baby-phi/data`).

Benefits:
- One binary, one process, one volume to back up.
- Zero-FFI integration with Rust-native SurrealDB.
- Local dev is `cargo run` — no docker-compose required.

See [ADR-0001](../decisions/0001-surrealdb-over-memgraph.md) for why SurrealDB over Memgraph, and [ADR-0007](../decisions/0007-embedded-vs-sidecar-database.md) for why embedded over sidecar.

## Scaling path

Embedded is the v0.1 default, **not** a dead end. The connection string is the only thing that changes across tiers:

| Tier | When | Connection | Query-code change? |
|---|---|---|---|
| **Embedded + RocksDB** (v0.1) | Single-node, ≤100k nodes / ≤1M edges / ≤100 concurrent sessions | `Surreal::new::<RocksDb>("data/baby-phi.db")` | — |
| **Standalone SurrealDB server** | Need to scale server horizontally or separate DB lifecycle | `Surreal::new::<Client>("ws://host:8000")` | **None.** Same SurrealQL. |
| **SurrealDB cluster with TiKV** | Multi-region, HA, or >100 GB data | TiKV backend + clustered SurrealDB | **None.** |
| **Swap database entirely** | SurrealDB itself is wrong tool | Rewrite `store` crate to a different DB | Full query rewrite. Domain trait unchanged. |

This is the explicit safety valve: the `store` crate is the *only* place we depend on SurrealDB. Replacing it is bounded work.

Full rationale + honest weak spots are in the archived build plan at [`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../../plan/build/36d0c6c5-build-plan-v01.md) §"Scaling path — from embedded to distributed".

## Data migration (forward-looking)

Three mechanisms available for moving data between SurrealDB instances (from simplest to most portable):

1. **RocksDB file copy** — fastest; same-version upgrades only.
2. **`surreal export` / `surreal import`** — SurrealQL dump; portable across SurrealDB versions.
3. **HTTP API streaming** — `SELECT *` in batches; universal; slower.

v0.1 ships no migration tooling — the backup/restore drill lands in M7b. M1's schema-migration framework (forward-only migrations in `domain::migrations`) is a separate concern: it migrates the *schema* in place, not between instances.

## Test fixture pattern

M0's health tests swap `SurrealStore` for a `FakeRepo` constructed inline at [`modules/crates/server/tests/health_test.rs`](../../../../../../modules/crates/server/tests/health_test.rs):

```rust
struct FakeRepo { healthy: bool }

#[async_trait]
impl Repository for FakeRepo {
    async fn ping(&self) -> RepositoryResult<()> {
        if self.healthy { Ok(()) } else { Err(...) }
    }
}
```

This pattern generalises for M1+: every integration test that doesn't need real persistence instantiates a fake implementing the methods it exercises. Acceptance tests that *do* need real storage will use a temp-directory `SurrealStore::open_embedded` — spin-up cost is milliseconds for a fresh RocksDB.
