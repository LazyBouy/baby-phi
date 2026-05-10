<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 — NEW convention doc per v0/conventions/ peer tier, cycle hex 240616a4) -->

# Persistence conventions

Reviewer-tier guidance for SurrealDB schema mechanics + write-verb idioms shipped through M5/P1–P4. Governance authority: [ADR-0058](../implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md) §D58.1 (schema mechanics) + §D58.3 (write verbs). Concept docs ([`ontology.md`](../concepts/ontology.md)) are silent below this granularity.

## Schema mechanics

### 1. DEFINE FIELD on pre-existing scaffolds (never fresh DEFINE TABLE)

When migration 0005 layers governance columns onto pre-existing `session` / `turn` scaffolds (defined in 0001), use `DEFINE FIELD` only — do NOT issue a fresh `DEFINE TABLE`. SurrealDB DDL `DEFINE TABLE` is destructive on existing rows.

- **Regression grep:** `grep -nE "DEFINE TABLE session SCHEMAFULL" modules/crates/store/migrations/0005_*.surql` → 0 hits.
- **Closes:** D1.1. **Cross-ref:** ADR-0058 §D58.1.

### 2. REMOVE + DEFINE retype on no-writer-scaffold

For edge-direction retype (e.g. `runs_session` reverse `agent → session` → forward `session → project`), use `REMOVE TABLE ... ; DEFINE TABLE ...` only when the scaffold has no live writers (M1-era reverse direction never had a writer). Tables with live writers require a paired migration + writer change.

- **Regression grep:** `grep -nE "REMOVE TABLE runs_session" modules/crates/store/migrations/0005_*.surql` → 1 hit.
- **Closes:** D1.2. **Cross-ref:** ADR-0058 §D58.1.

## Write verbs

### 1. CREATE-not-UPDATE-as-upsert

`persist_session` / `append_loop_record` / `append_turn` use `CREATE` (not `UPDATE` as upsert). UPDATE on a non-existent SCHEMAFULL+FLEXIBLE row is silent no-op; CREATE surfaces duplicate-violation mapping to `RepositoryError::Conflict`.

- **Regression grep:** `grep -nE "CREATE type::thing" modules/crates/store/src/repo_impl.rs` → ≥ 1 hit.
- **Closes:** D2.1. **Cross-ref:** ADR-0058 §D58.3.

### 2. LET-first RELATE

`RELATE` requires LET-bound endpoints: `LET $f = type::thing(...); LET $t = type::thing(...); RELATE $f -> edge -> $t`. SurrealDB parser rejects inline `type::thing(...)` in FROM/TO slots.

- **Reviewer rule:** reject any RELATE with inline `type::thing(...)` in FROM/TO.
- **Closes:** D2.2. **Cross-ref:** ADR-0058 §D58.3.

### 3. Branch-on-existence for upsert-vs-create

When a handler must upsert-or-create (agent profile rebind), branch on `current_profile.is_some()` at the handler — call `upsert_agent_profile` when prior row exists, `create_agent_profile` otherwise. Same root-cause as subsection 1.

- **Regression grep:** `grep -nE "current_profile\.is_some\(\)" modules/crates/server/src/platform/agents/update.rs` → ≥ 1 hit.
- **Closes:** D4.4. **Cross-ref:** ADR-0058 §D58.3.
