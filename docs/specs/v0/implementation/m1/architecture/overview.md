<!-- Last verified: 2026-04-20 by Claude Code -->

# M1 architecture — overview

M1 turns the M0 scaffolding into a functional Permission Check spine. This
page is the system map; depth pages cover each subsystem.

## System map (P1–P3 landed)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          web (Next.js 14 / SSR)                         │
│                       [P8]  /bootstrap page                             │
├─────────────────────────────────────────────────────────────────────────┤
│                       cli (clap; baby-phi binary)                       │
│                       [P7]  baby-phi bootstrap ...                      │
├─────────────────────────────────────────────────────────────────────────┤
│                 server (axum; baby-phi-server binary)                   │
│   existing: /healthz/live /healthz/ready /metrics                       │
│     [P6]: /api/v0/bootstrap/{status,claim} + session cookie             │
├─────────────────────────────────────────────────────────────────────────┤
│                              domain                                     │
│   ✓ model/   — 9 fundamentals, 8 composites, 37 nodes, 66 edges         │
│   ✓ model/principal_resource — sealed Principal/Resource marker traits  │
│   ✓ audit    — event shape, class tiers, hash-chain helper              │
│   ✓ repository — 33-method trait + 3 typed free-function wrappers       │
│   ✓ in_memory — HashMap-backed Repository fake (feature-gated)          │
│   ✓ permissions — 6-step (+2a) engine, pure fn, metric-instrumented     │
│   ⏳ auth_requests — 9-state machine (P4)                               │
├─────────────────────────────────────────────────────────────────────────┤
│                               store                                     │
│   ✓ SurrealStore::open_embedded runs migrations on startup              │
│   ✓ migrations — forward-only runner with startup-gate fail-safe        │
│   ✓ crypto — AES-GCM envelope for secrets_vault                         │
│   ✓ repo_impl — full SurrealDB impl of all 33 Repository methods        │
├─────────────────────────────────────────────────────────────────────────┤
│                  SurrealDB (embedded RocksDB backend)                   │
│   ✓ schema 0001_initial: 37 node tables + 66 edge relations             │
│                          + bootstrap_credentials + secrets_vault        │
│                          + audit_events + resources_catalogue           │
└─────────────────────────────────────────────────────────────────────────┘
```

**Dependency flow** (strict, downward):

```
cli              ┐
server          ─┼─▶ domain ─▶ store ─▶ SurrealDB
web (Next.js)  ──┘             (plus phi-core for agent/session types)
```

The `domain` crate does **not** depend on `store`; `Repository` is a trait
defined in `domain` and implemented in `store`. This keeps the crate DAG
downward-only and lets domain tests use an in-memory fake.

## What P1 + P2 + P3 delivered

**P1 foundation:**

1. **Graph model** (`modules/crates/domain/src/model/`). See
   [graph-model.md](graph-model.md) for the full inventory.
2. **Audit-event skeleton** (`modules/crates/domain/src/audit.rs`). See
   [audit-events.md](audit-events.md).
3. **Schema + migration runner** (`modules/crates/store/migrations/` +
   `modules/crates/store/src/migrations.rs`). See
   [schema-migrations.md](schema-migrations.md).
4. **At-rest encryption layer** (`modules/crates/store/src/crypto.rs`). See
   [at-rest-encryption.md](at-rest-encryption.md).
5. **Startup gate**: `SurrealStore::open_embedded` runs every embedded
   migration automatically; a failed migration surfaces as
   [`StoreError::Migration`](../../../../../../modules/crates/store/src/lib.rs)
   and the server refuses to start — fail-safe.

**P2 repository + type safety:**

6. **Type-safe ownership edges** (`modules/crates/domain/src/model/principal_resource.rs`):
   sealed `Principal` / `Resource` marker traits + typed
   `Edge::new_owned_by` / `new_created` / `new_allocated_to` constructors
   + 6 `trybuild` compile-fail fixtures. See
   [ADR-0015](../decisions/0015-type-safe-ownership-edges.md).
7. **Repository trait expansion** (`modules/crates/domain/src/repository.rs`):
   33 object-safe methods + 3 typed free-function wrappers. See
   [storage-and-repository.md](storage-and-repository.md).
8. **SurrealStore implementation** (`modules/crates/store/src/repo_impl.rs`):
   full CRUD against embedded SurrealDB via `type::thing(...)` record ids
   + `CONTENT $body` pattern + per-type row translators for rich types
   (Grant, AuthRequest).
9. **In-memory Repository fake**
   (`modules/crates/domain/src/in_memory.rs`): feature-gated HashMap
   impl. Used by the M0 server health tests and the P3/P4 proptests to
   follow.

**P3 Permission Check engine:**

10. **Permission Check engine** (`modules/crates/domain/src/permissions/`):
    8-file module implementing the full 6-step (+Step 2a) pipeline as a
    **pure** `check()` free function. Decision shape is a three-valued
    enum (`Allowed` / `Denied { failed_step }` / `Pending`); the engine
    records latency + result label via a caller-supplied
    `PermissionCheckMetrics` trait (no `prometheus` dependency in
    domain). See [permission-check-engine.md](permission-check-engine.md)
    + [ADR-0008](../decisions/0008-permission-check-as-pipeline.md).
11. **Engine proptest coverage**: 6 files under
    `modules/crates/domain/tests/permission_check_*_props.rs` +
    `permission_check_worked_trace.rs` — 14 invariants × 256 cases
    default ≈ 3,584 random-input branches per CI run. Canonical
    coverage: Step-0 catalogue precondition, Step-2 empty-grants,
    Step-3 no-matching-grant, Step-4 constraint satisfaction, Step-6
    consent gating, pipeline monotonicity (adding unrelated grants /
    ceilings / revoked grants never widens Allowed), and the
    concept-doc worked trace (`bash cargo build` → Allowed;
    `bash rm -rf /` without filesystem grant → Step 3 denial).

## Crate DAG

| Crate | Purpose | P1 additions |
|---|---|---|
| [`domain`](../../../../../../modules/crates/domain/) | Graph model + Permission Check engine + Auth Request state machine | `model/` submodule (5 files), `audit.rs`, blake3 dep |
| [`store`](../../../../../../modules/crates/store/) | SurrealDB (RocksDB) adapter | `migrations.rs`, `crypto.rs`, `migrations/0001_initial.surql`, aes-gcm + base64 + rand deps |
| [`server`](../../../../../../modules/crates/server/) | axum HTTP surface | _Not in P1_ — P6 extends. |
| [`cli`](../../../../../../modules/crates/cli/) | clap CLI | _Not in P1_ — P7 rewrites. |

## Testing posture (after P3 + post-audit P2 widening)

| Layer | P1 | P2 | P3 | P3+ (post-audit widening) |
|---|---|---|---|---|
| Domain unit (model + audit + marker-trait impls + typed constructors + permissions helpers) | 27 | 36 | 91 | 91 |
| Domain proptest (permission-check invariants, 6 files × ~2.3 invariants) | 0 | 0 | 14 | 14 |
| Domain compile-fail (`trybuild`, wrong-pair endpoint rejection) | 0 | 6 | 6 | 6 |
| Store unit (crypto + migrations) | 13 | 13 | 13 | 13 |
| Store integration (migrations + crypto vault + repository) | 2 | 29 | 29 | **59** |
| Server integration (M0 health + TLS, now using `InMemoryRepository`) | 4 | 4 | 4 | 4 |
| **Runnable total** | **46** | **82** | **151** | **186** |

P2 shipped 29 store integration tests; a post-P3 independent re-audit
identified the P2 repository surface as under-tested relative to the
plan's ≈92-test budget. The **P3+ widening pass** progressed in two
stages:

1. **Breadth pass** — 23 new integration tests covering: error paths
   (duplicate-id rejection, missing-id no-op semantics, idempotent
   revocation), multi-field round-trips (every `PrincipalRef` variant
   as a grant holder, multi-action grants, grants with `descends_from`
   provenance, multi-slot / multi-approver Auth Requests, full-field
   audit events), cross-cutting semantics (case-sensitive catalogue
   lookup, kind-metadata persistence, archive-flip drops from active
   listings, edge upserts produce distinct ids).
2. **Audit follow-up** — an Explore-agent re-audit flagged two
   remaining weaknesses: (a) two bulk "all create green" tests that
   did not verify persistence via `get_*`, and (b) the `ping` health
   surface was untested. Both were addressed: the bulk tests were
   split into 9 focused tests that each assert row persistence and
   key field preservation via direct SurrealDB `count()` /
   field-projection queries, and `ping_returns_ok_on_fresh_store`
   was added.

Total P2 store integration count now **59** (57 repository + 1
migrations + 1 crypto vault) — substantive coverage across every one
of the 33 trait methods + 3 free-function wrappers + the `ping`
surface, with strong error-path parity on the critical paths
(revocation, update, list filters, cross-scope isolation).

## What to read next

- [graph-model.md](graph-model.md) — the 9/8/37/66 inventory and how it's
  organised in Rust.
- [schema-migrations.md](schema-migrations.md) — the forward-only runner.
- [audit-events.md](audit-events.md) — the audit event shape and hash-chain
  seed.
- [at-rest-encryption.md](at-rest-encryption.md) — envelope encryption for
  the secrets vault.
- [permission-check-engine.md](permission-check-engine.md) — P3's
  6-step (+2a) pipeline with ASCII diagram, module layout, and
  proptest coverage map.
- The M0 companion pages (one folder up,
  [`../../m0/`](../../m0/README.md)) for everything P1 builds upon.
