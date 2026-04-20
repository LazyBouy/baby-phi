<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0012: Forward-only embedded migrations with a startup-gate fail-safe

## Status

Accepted — 2026-04-20 (M1 / P1).

## Context

The v0.1 build plan commits to "schema migrations" as a production-readiness
row owned by M1: *"Versioned SurrealDB schema with forward-only migrations.
Every migration tested in CI against a representative dataset. Migration
runs on startup; failed migrations refuse to serve (fail-safe)."*

We need to choose: where do migration files live, who runs them, what
happens on a partial failure.

## Decision

1. **Forward-only.** Migrations are additive or rewriting, never deleting
   or reverting. The schema grows over time; rollbacks are expressed as
   follow-on compensating migrations, not undo scripts.
2. **Embedded via `include_str!`.** Migration DDL files
   (`modules/crates/store/migrations/NNNN_{slug}.surql`) are baked into the
   binary at compile time. The release binary never needs filesystem
   access to a `migrations/` directory — reduces the surface for
   filesystem-permission bugs in deploy environments.
3. **Run at store-open.** `SurrealStore::open_embedded` calls
   `migrations::run_migrations` automatically. Callers don't have to
   remember to run migrations themselves.
4. **Fail-safe startup gate.** If any migration errors, `open_embedded`
   returns `StoreError::Migration(MigrationError::Apply{...})` and the
   server's `main.rs` aborts before binding the listener. A half-migrated
   DB never serves traffic.
5. **Ledger as the record of truth.** Applied versions are recorded in the
   `_migrations` table **after** the DDL succeeds. A migration whose DDL
   succeeded but whose ledger write failed will re-run on next startup —
   all DDL is authored to be safe-to-retry (either inherently idempotent,
   or wrapped in `IF NOT EXISTS`).
6. **Monotonically-increasing version numbers.** The runner rejects an
   out-of-order migration list at startup — guards against a future
   `0005_a.surql` being accidentally placed before `0004_b.surql`.

## Consequences

Positive:

- **Deterministic behaviour.** The embedded migration list is the exact
  set of DDL that was shipped; what the test harness sees is what prod
  sees.
- **No half-migrated DB in production.** Fail-safe. If something went
  wrong, the server refuses to start; an operator investigates. Silent
  partial-apply is the worst outcome; this rules it out.
- **Schema evolution is a PR.** A new migration is a new file + one line
  in `EMBEDDED_MIGRATIONS`. Reviewers see exactly what changed.

Negative:

- **No built-in rollback.** Mistakes are fixed by forward migrations. In
  practice this is fine for an append-mostly schema; it's awkward for a
  schema that needs genuine structural revisions.
- **Migration ordering is Rust-source-level.** You cannot re-order the
  array without breaking existing databases.
- **Release binary carries every migration's text forever.** Negligible
  for v0.1 (~260 lines); if it balloons in a later version, we revisit.

## Alternatives considered

- **`sqlx-migrate`-style file-based with filesystem at runtime.** Rejected:
  adds a runtime-filesystem dependency that doesn't buy us anything; the
  Rust binary is the right trust boundary.
- **Refinery / Diesel migrations.** Rejected: SurrealDB-specific DDL
  doesn't map cleanly to those tools' abstractions, and we'd add a
  dependency to save writing ~150 lines of runner code.
- **Transactions-around-DDL.** Rejected: the embedded SurrealDB backend
  does not support DDL inside transactions. We rely on the ledger-row
  write as the "final barrier" and author DDL to be safely retryable.
- **Down-migrations.** Rejected for v0.1: encourages reversible thinking
  that doesn't match our additive-only policy. M7b revisits if a real
  need surfaces.

## References

- Implementation: [`modules/crates/store/src/migrations.rs`](../../../../../../modules/crates/store/src/migrations.rs)
- Schema: [`modules/crates/store/migrations/0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql)
- Architecture page: [schema-migrations.md](../architecture/schema-migrations.md)
- Plan row: `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §Production-readiness commitments / "Schema migrations".
