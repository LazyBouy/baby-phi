<!-- Last verified: 2026-04-20 by Claude Code -->

# Schema migrations — operations

M1 ships the forward-only embedded migration runner described in
[architecture/schema-migrations.md](../architecture/schema-migrations.md)
and [ADR-0012](../decisions/0012-forward-only-migrations.md). This
page covers the operator-facing lifecycle: applying, auditing, and
dealing with a broken migration.

## How migrations are applied

`SurrealStore::open_embedded` walks every file under
[`modules/crates/store/migrations/`](../../../../../../modules/crates/store/migrations/),
sorted by filename, and applies any that are not yet recorded in the
`_migrations` meta-table. Applied migrations land inside a single
SurrealDB transaction per file — partial application is impossible.

The sequence on `phi-server` start-up:

1. `open_embedded(data_dir, ns, db)` opens the embedded RocksDB
   store at `data_dir`.
2. The runner queries `SELECT * FROM _migrations` to see what's
   already applied.
3. For each on-disk file whose sorted name comes after the most
   recent applied version, the file is parsed and executed as a
   single multi-statement query inside a `BEGIN TRANSACTION …
   COMMIT TRANSACTION` envelope.
4. Success → a row `{ version, applied_at }` is inserted into
   `_migrations`.
5. Failure → the transaction rolls back, the server returns
   `StoreError::Migration`, and `phi-server` aborts with a
   non-zero exit code.

This is the **startup gate** from the plan — a broken migration
cannot corrupt a production database because the process refuses to
start until the migration issue is resolved.

## Auditing the current schema version

There is no dedicated admin command in M1; operators query the
`_migrations` table directly:

```bash
# Assuming the server is NOT running (RocksDB is single-writer).
/root/rust-env/cargo/bin/cargo run -q -p server --bin phi-server -- \
  bootstrap-init   # any subcommand that opens the store will first run migrations
```

A dedicated `phi admin migrations status` subcommand lands in M2+
as part of the `admin/` surface work.

## Adding a new migration

1. Create `modules/crates/store/migrations/000N_<slug>.surql` where
   `N` is the next integer in sequence.
2. Write **additive** SurrealDB DDL only — `DEFINE TABLE`,
   `DEFINE FIELD`, `DEFINE INDEX`. Forward-only means no `REMOVE`
   statements and no destructive `ALTER`.
3. Include a `--- Down migration: …` ASCII note at the top of the
   file describing how a future operator would manually reverse the
   change if required (the plan does not implement automatic `down`
   migrations; "down" is a rebuild + restore operation — see the
   backup/restore runbook stub planned for M7b).
4. Run the full workspace test suite locally; the `SurrealStore`
   integration tests boot a fresh store per test, so they exercise
   every migration on every CI run.
5. Commit the `.surql` file in the same commit as the code that
   depends on the new schema shape.

## Recovering from a broken migration

"Broken" means SurrealDB rejected the statement. The server refused
to start; no data has been corrupted. Recovery steps:

1. Read the `StoreError::Migration` message — it echoes the failing
   statement and SurrealDB's error.
2. Fix the `.surql` file. Push a new commit (or amend locally if
   it's a dev machine).
3. Restart the server. The runner re-attempts the same version
   (it's **not** yet in `_migrations` since the previous run rolled
   back).
4. Once the migration succeeds, `_migrations` records it and future
   start-ups skip it.

Never edit `_migrations` by hand on a production database. If a
migration applied successfully but its effect is undesirable, the
right fix is a new, additive migration that modifies the shape —
never a rewrite of an applied-and-committed migration.

## What M1 does NOT ship (deferred to M7b)

- **Dry-run mode.** A `phi admin migrations dry-run` command
  that loads the `.surql` file into an in-memory store and reports
  what it would do. Planned in M7b production-hardening.
- **Schema diffing.** A tool that diffs the running schema against
  a reference dump, so operators can detect drift. Planned in M7b.
- **Down migrations.** Deliberate omission — see
  [ADR-0012 §Consequences](../decisions/0012-forward-only-migrations.md).

## Cross-references

- [architecture/schema-migrations.md](../architecture/schema-migrations.md)
  — the runner's design and the `_migrations` schema.
- [ADR-0012](../decisions/0012-forward-only-migrations.md) — why
  forward-only with a startup gate.
- [modules/crates/store/migrations/](../../../../../../modules/crates/store/migrations/)
  — the current migration set (only `0001_initial.surql` in M1).
- [modules/crates/store/src/migrations.rs](../../../../../../modules/crates/store/src/migrations.rs)
  — the runner implementation.
