<!-- Last verified: 2026-05-19 by Claude Code (CH-28 P-DOCS deliverable 5 — appended 0019_agent_profile_n_to_1_schema + 0020_agent_profile_n_to_1_backfill rows to the NEW Migration list table per ADR-0063 §D63.3; NEW §"Split-migration pattern (NEW at CH-28)" subsection codifies the schema-vs-data split as a reusable precedent (first split-migration in the workspace; future cycles facing "operator wants inspection window between schema and data" cite CH-28). Cycle hex `0412eb06`.) -->

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

## Migration list (post-CH-28)

| Version | Slug | Tier | What it ships |
|---|---|---|---|
| 0001 | `initial` | M1 schema | Full v0 ontology table set (37 nodes + 66 edges) |
| 0002 | (M2 grants extension) | M2 | Grant-table evolution |
| 0003 | (M2 templates) | M2 | Authority template scaffold |
| 0004 | (M3 K8s prep) | M3 | K8s-prep refactors per ADR-0033 |
| 0005 | (M4 page extensions) | M4 | M4 page-driven schema additions |
| 0006 | (M4 tags + composite extensions) | M4 | Composite/node `tags: Vec<String>` per ADR-0037 |
| 0007 | (CH-01 agent lifecycle) | M5 | `agent.active` + `agent.archived_at` durable lifecycle fields |
| 0008 | (CH-02 system agent listing) | M5 | System-agent-list query support |
| 0009 | (CH-16 identity) | M5.2 | `identity` table + 4-field shape + UNIQUE-on-`agent_id` |
| 0010 | (CH-21 memory extraction) | M5.2 | Memory-extraction counter fields |
| 0011 | (CH-23 manages + has_agent_supervisor) | M5.2 | Template C/D HTTP edge tables |
| 0012–0017 | (M5.2 / M5.3 follow-ons) | M5.2/M5.3 | Various per-chunk schema additions through CH-25 close |
| 0018 | (CH-25 OWNS edge) | M5.3 | Agent ownership edge for org + project per ADR-0060 |
| **0019** | **`agent_profile_n_to_1_schema`** | **M6 (CH-28)** | **Schema-only: DROP UNIQUE on `agent_profile.agent_id`; DEFINE `blueprint` table (SCHEMAFULL, 6 fields, UNIQUE-WHERE-agent_id-NOT-NONE index); DEFINE `uses_profile` RELATION (renamed from `has_profile`); DEFINE `agent_profile_uses_blueprint` + `agent_uses_blueprint_override` RELATION; ALTER `agent_profile` to remove override fields (`parallelize`, `model_config_id`, `mock_response`)** |
| **0020** | **`agent_profile_n_to_1_backfill`** | **M6 (CH-28)** | **Data-only: CREATE template `blueprint` rows from existing AgentProfile override values; RELATE existing `agent_profile` rows to template blueprints via `agent_profile_uses_blueprint`; rewrite `has_profile` edges to `uses_profile` (renamed); per-agent override rows are NOT created at backfill — they're written lazily by application code on first override** |

**Narrative for 0019 + 0020 (per ADR-0063 §D63.3 + §D63.5).**

**0019** is schema-only: it makes the N:1 cardinality structurally
possible (by dropping the UNIQUE constraint on `agent_profile.agent_id`)
and defines the NEW `blueprint` table with its UNIQUE-WHERE-agent_id-NOT-NONE
index that distinguishes template vs override rows. All DDL uses
`IF [NOT] EXISTS` so re-applying 0019 on a post-apply database is a
no-op. The override-field removal from `agent_profile` (`parallelize`,
`model_config_id`, `mock_response`) is an `ALTER … REMOVE FIELD` that
loses the values temporarily — 0020 preserves them by re-creating them on
the corresponding template `blueprint` rows.

**0020** is data-only: for each existing `agent_profile` row, it CREATEs
a template `blueprint` row (`agent_id = NONE`, copying the pre-0019
override values) and RELATEs the profile to the template via
`agent_profile_uses_blueprint`. It then rewrites each existing
`has_profile` edge row to `uses_profile` (the renamed edge). Per-agent
override rows are NOT created at backfill — each agent inherits its
template's defaults; per-agent overrides are written lazily by
application code when an agent's profile is mutated. Idempotency is
preserved: re-running 0020 on a post-apply database is a no-op (template
`blueprint` rows already exist; `has_profile` rows already removed;
`uses_profile` rows already created — each backfill statement is guarded
by an `IF NOT EXISTS` / `WHERE NOT EXISTS` clause).

The 0019↔0020 inter-migration window is permitted (operator-inspection
use-case). Runtime reads of `get_agent_profile_for_agent` tolerate the
half-migrated state via the NEW
`read_agent_profile_via_blueprint_or_fallback` helper at the repository
layer; see [`m6/architecture/agent-profile-cardinality.md`](../../m6/architecture/agent-profile-cardinality.md) §5 for
the design.

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

## Split-migration pattern (NEW at CH-28)

CH-28 introduced the **first split-migration in the baby-phi workspace**
— `0019_agent_profile_n_to_1_schema.surql` (schema-only) +
`0020_agent_profile_n_to_1_backfill.surql` (data-only) — per ADR-0063
§D63.3 (F3.b user-locked at gate-1). This codifies a reusable pattern
for future cycles facing the same "operator wants inspection window
between schema and data" decision.

**When to split (decision criteria):**

| Use split (0019/0020 style) | Use single composite (default style) |
|---|---|
| Schema change is load-bearing AND data backfill is non-trivial | Schema + data fit in one ≤ 200 LOC migration |
| Operator wants to inspect schema before backfill triggers | Schema + data are atomically coupled |
| Rollback semantic is cleaner with two units | Rollback granularity is at the whole-migration level |
| Half-migrated-state runtime tolerance is shippable | Half-migrated state is unsafe (e.g. data is read by code in flight) |

**Shape of each half:**

- **Schema half (`NNNN_<slug>_schema.surql`)**: only DDL statements
  (`DEFINE TABLE`, `DEFINE FIELD`, `DEFINE INDEX`, `REMOVE INDEX`,
  `ALTER TABLE … REMOVE FIELD`). All DDL is `IF [NOT] EXISTS`. No
  `CREATE` / `UPDATE` / `RELATE` / `DELETE` against application rows.
- **Data half (`NNNN+1_<slug>_backfill.surql`)**: only DML
  (`CREATE`, `UPDATE`, `RELATE`, `DELETE`). Every statement is guarded
  by `IF NOT EXISTS` / `WHERE NOT EXISTS` for idempotency. References to
  schema entities (tables, indexes) added in the schema half are
  resolved at apply time; if the schema half hasn't applied yet, the
  data half fails fast with a schema-not-found error (the migration
  runner catches this and abandons the apply per the fail-safe contract).

**Half-migrated-state runtime tolerance** (required when the inter-migration
window is permitted): the application's repository layer ships a helper
that reads from EITHER the pre-schema-change shape OR the post-schema-change
shape, preferring the new path but falling back gracefully when the new
tables/columns are empty. CH-28's example:
`read_agent_profile_via_blueprint_or_fallback` at
`store::repo_impl::SurrealStore` + `domain::in_memory::InMemoryRepository`.
The helper is removed in a future cycle once the migration is universally
applied (CH-28's helper is slated for removal at M7 NFR-observability).

**Migration runner ordering** (per ADR-0012 forward-only contract): the
runner applies migrations in `version` order; if 0019 is applied but
0020 is not (because the operator paused the deployment between them),
the application starts cleanly + the runtime helper handles reads. If
the operator skips 0019 and tries to apply 0020 first, the runner
rejects the out-of-order apply (`MigrationError::OutOfOrder`) before any
DDL runs.

**Precedent**: future cycles facing the same decision cite CH-28 / ADR-0063
§D63.3 as precedent.

## Tests

| Test | File | What it asserts |
|---|---|---|
| `runs_embedded_migrations_from_empty_db` | `migrations.rs` | Fresh DB → migration 1 applied, ledger has exactly one row |
| `is_idempotent_across_successive_runs` | `migrations.rs` | Second call to `run_migrations` is a no-op |
| `rejects_out_of_order_migrations` | `migrations.rs` | `Migration::version` must increase; misorder → `MigrationError::OutOfOrder` |
| `broken_migration_surfaces_apply_error_without_ledger_row` | `migrations.rs` | Invalid SQL → `MigrationError::Apply`, no row in `_migrations` |
| `open_embedded_applies_initial_migration_and_creates_schema` | `tests/migrations_test.rs` | End-to-end: fresh tempdir → schema live + `agent.kind` ASSERT rejects invalid values |

## Concept references

- Build plan row: `docs/specs/plan/build/build-plan-v01-36d0c6c5.md`
  §Production-readiness commitments / "Schema migrations".
- ADR: [0012 Forward-only migrations](../decisions/0012-forward-only-migrations.md).
