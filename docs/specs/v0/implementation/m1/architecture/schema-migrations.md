<!-- Last verified: 2026-04-20 by Claude Code -->

# Architecture — schema migrations

M1 ships a forward-only SurrealDB migration runner with a startup-gate
fail-safe. Rationale is in
[ADR-0012](../decisions/0012-forward-only-migrations.md); this page covers
mechanics.

## Pieces

| Piece | Location |
|---|---|
| Runner | [`store::migrations`](../../../../../../modules/crates/store/src/migrations.rs) |
| Embedded migration list | `EMBEDDED_MIGRATIONS` (in `migrations.rs`) |
| First migration's DDL | [`modules/crates/store/migrations/0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql) |
| Startup gate | `SurrealStore::open_embedded` in [`store::lib`](../../../../../../modules/crates/store/src/lib.rs) |
| `_migrations` ledger | created by `bootstrap_ledger_table` before migrations run |

## How it runs at startup

```
SurrealStore::open_embedded(path, ns, db)
  │
  ├─ Surreal::new::<RocksDb>(path) .await
  ├─ .use_ns(ns).use_db(db)
  ├─ migrations::run_migrations(&db, EMBEDDED_MIGRATIONS) .await
  │     │
  │     ├─ validate_ordering()   ─── reject out-of-order versions
  │     ├─ bootstrap_ledger_table() ─ DEFINE TABLE IF NOT EXISTS _migrations
  │     ├─ read_applied_versions() ── SELECT version FROM _migrations
  │     ├─ for each migration where version not in applied:
  │     │    ├─ db.query(migration.sql).check()    (DDL)
  │     │    └─ CREATE _migrations row
  │     └─ return list of newly-applied versions
  │
  └─ returns SurrealStore (or StoreError::Migration if anything failed)
```

Any failure surfaces as `StoreError::Migration(MigrationError)`, and
`main.rs` aborts before serving traffic — **fail-safe**. This matches the
build plan's production-readiness row for "schema migrations".

## Adding a new migration

1. Create `modules/crates/store/migrations/NNNN_{slug}.surql` where `NNNN`
   is the next four-digit version (`0002`, `0003`, …).
2. Append the new `Migration { version, slug, sql: include_str!(...) }`
   entry to `EMBEDDED_MIGRATIONS` in
   [`migrations.rs`](../../../../../../modules/crates/store/src/migrations.rs).
3. Write the DDL using `DEFINE TABLE … SCHEMAFULL` and `DEFINE FIELD` /
   `DEFINE INDEX`. Avoid destructive DDL — M1's policy is forward-only.
4. Add a test to
   [`modules/crates/store/tests/`](../../../../../../modules/crates/store/tests/)
   that exercises the new schema.

## Idempotency semantics

- Migrations that succeed are recorded in `_migrations` and never re-run.
- A migration that fails mid-way is **not** recorded; the next startup
  retries the full SQL.
- The initial migration's DDL is authored so it's safe to re-run on a
  partially-applied database (non-idempotent fragments are called out in
  the file). The runner does not wrap the DDL in a transaction because the
  embedded SurrealDB backend does not support DDL-in-transactions; relying
  on the ledger-row write as the final barrier is the chosen alternative.
- `_migrations` itself is created by the runner via
  `DEFINE TABLE IF NOT EXISTS`, so it never conflicts with the initial
  migration's schema.

## Tests

| Test | File | What it asserts |
|---|---|---|
| `runs_embedded_migrations_from_empty_db` | `migrations.rs` | Fresh DB → migration 1 applied, ledger has exactly one row |
| `is_idempotent_across_successive_runs` | `migrations.rs` | Second call to `run_migrations` is a no-op |
| `rejects_out_of_order_migrations` | `migrations.rs` | `Migration::version` must increase; misorder → `MigrationError::OutOfOrder` |
| `broken_migration_surfaces_apply_error_without_ledger_row` | `migrations.rs` | Invalid SQL → `MigrationError::Apply`, no row in `_migrations` |
| `open_embedded_applies_initial_migration_and_creates_schema` | `tests/migrations_test.rs` | End-to-end: fresh tempdir → schema live + `agent.kind` ASSERT rejects invalid values |

## Concept references

- Build plan row: `docs/specs/plan/build/36d0c6c5-build-plan-v01.md`
  §Production-readiness commitments / "Schema migrations".
- ADR: [0012 Forward-only migrations](../decisions/0012-forward-only-migrations.md).
