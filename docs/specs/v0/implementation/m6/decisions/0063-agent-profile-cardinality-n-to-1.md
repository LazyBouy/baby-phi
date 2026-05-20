<!-- Last verified: 2026-05-19 by Claude Code (CH-28-0412eb06 P-SEAL — Status flipped Proposed → Accepted; 16 sub-decisions §D63.1..§D63.16 ratified — iter-5 added §D63.13 in-process struct preservation + §D63.14 write-boundary strip via AgentProfileWireRow + §D63.15 read-path synthesis + composite-write semantics + §D63.16 listener template-tier fan-out scope-narrowing → D-CH28-FOLLOWUP-01 M6-DEFERRED-04; iter-4 cross-ref annotations preserved on §D63.1 + §D63.11; §"Consequences ### For CH-36" body amended per §D63.16; 3 user-locked DIVERGENT forks F1.c hybrid blueprint table + F2.b USES_PROFILE rename + F3.b split migrations preserved verbatim) -->
<!-- Last verified: 2026-05-19 by Claude Code (CH-28-0412eb06 P0 draft, Proposed; v22 plan §5 12 sub-decisions across 3 user-locked DIVERGENT forks F1.c hybrid blueprint table + F2.b USES_PROFILE rename + F3.b split migrations) -->

# ADR-0063 — AgentProfile cardinality 1:1 → N:1 redesign (hybrid Blueprint table + edge rename + split migrations)

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v22 + chunk-implementer v14)

**Chunk**: CH-28 (cycle hex `0412eb06`)

**Milestone**: M6 (foundation tier; first M6-tier ADR)

**Decision-summary** (one line, iter-5 update): AgentProfile cardinality flips from 1:1 to N:1 via hybrid `Blueprint` table (F1.c); HAS_PROFILE → USES_PROFILE rename (F2.b); split migrations 0019 + 0020 (F3.b); EDGE_KIND_NAMES.len() bumps 72 → 74; AgentProfile in-process struct field-set preserved as synthesized projection (§D63.13); write-boundary strip via `AgentProfileWireRow` (§D63.14); composite-write at `upsert_agent_profile` + read-path synthesis at `get_agent_profile_for_agent` wired at P1.5-READ-BRIDGE (§D63.15); listener template-tier fan-out deferred to CH-36 / `M6-DEFERRED-04` per §D63.16.

---

## Forks

| Fork | Locked option | Path | Pros | Cons | Status |
|---|---|---|---|---|---|
| **F1** (CRITICAL) | **F1.c USER-DIVERGENT** | Hybrid `blueprint` table shared across BOTH `agent_profile` rows AND per-agent overrides; both AgentProfile + Agent carry RELATION edges to blueprint row(s); per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) live in the blueprint table per-row keyed by `agent_id` | Maximum auditability; maximum wire-format-explicit separation; downstream chunks inherit a single canonical "override-bearing row" abstraction | Maximum schema complexity; ~3× the cascade vs F1.a; introduces a per-blueprint-row decision (template vs override) | **LOCKED DIVERGENT** from planner-rec F1.a |
| **F2** (HIGH) | **F2.b USER-DIVERGENT** | Rename `Edge::HasProfile` → `Edge::UsesProfile` across all enumerated sites + serde rename-aware deserialization for back-compat | Reflects template-sharing semantic accurately ("uses" implies non-ownership); single canonical edge name; ADR-0057 §D57.7 EDGE_KIND_NAMES cardinality invariant preserved | Cascade through 43 sites; serde back-compat alias required | **LOCKED DIVERGENT** from planner-rec F2.a |
| **F3** (HIGH stakes) | **F3.b USER-DIVERGENT** | Split: `0019_agent_profile_n_to_1_schema.surql` (schema-only) + `0020_agent_profile_n_to_1_backfill.surql` (data-only) | Operator can inspect schema before backfill triggers; cleaner rollback semantics (rollback 0020 leaves 0019 schema intact); 2 atomic units smaller than single composite | First-apply at 0019 leaves DB in a half-migrated state; handled at 0019 by making 0020's backfill idempotent + at the runtime level by reads being tolerant of either pre-0020 or post-0020 shapes | **LOCKED DIVERGENT** from planner-rec F3.a |

**Cross-cycle divergence pattern**: ALL 3 forks locked DIVERGENT at gate-1 — cumulative cross-cycle divergent forks 14-of-19 (~74%) at CH-28 close. F<X>.b/F<X>.c expansion-divergence pattern structurally durable (10 cycles of evidence now).

---

## Context

**Why this chunk.** CH-28 closes the long-open `M6+-OPEN-01` concept re-evaluation marker (surfaced at CH-01 plan-review 2026-04-27; brought in-milestone for M6 per Q9 user-lock 2026-05-18) that asks: *should `domain::Agent` ↔ `domain::AgentProfile` cardinality flip from 1:1 ("profile-as-genetics") to N:1 ("profile-as-template, shared across agents")?* The user has decided to **pursue the flip** at M6 plan-open.

**Concept-doc precedence.** The pre-existing 1:1 + HAS_PROFILE framing was concept-mandated (per `concepts/agent.md` §Soul L160–169 + `concepts/ontology.md` L98). Quality-over-speed principle: *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-28 amends the concept docs FIRST then aligns code.

**Forward-scope reference.** `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 41–48. Origin marker: `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370. CH-01 plan-review provenance: `build/ch-01-agent-durable-lifecycle-2aa37c80.md`.

**Downstream consumers.** CH-29 (M6-DEFERRED-02 messaging substrate), CH-36 (a04 My Work / M6-DEFERRED-04 supervisor body), CH-37 (a05 My Profile + Grants), and M7 NFR-observability all inherit the hybrid-Blueprint shape established here.

---

## Sub-decisions

### §D63.1 — F1.c lock outcome + hybrid Blueprint table structure

**Decision**: Introduce a NEW `blueprint` table (SCHEMAFULL) shared across both `agent_profile` rows AND per-agent overrides. Both `AgentProfile` and `Agent` carry RELATION edges to blueprint row(s). The `blueprint` row carries 6 fields: `id` (string, primary key), `agent_id` (option<string>, optional — None means "this is a shared-template row, not an agent-keyed override row"), `parallelize` (option<int>, default NONE), `model_config_id` (option<string>, default NONE), `mock_response` (option<string>, default NONE), `created_at` (datetime).

A NEW `Blueprint` struct + `BlueprintId` newtype is added to `domain/src/model/nodes.rs` adjacent to `AgentProfile`. Per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) move OFF `AgentProfile` literal entirely and live on the per-agent `Blueprint` row via `AgentUsesBlueprintOverride`; the AgentProfile row keeps only `id`, `agent_id`, `blueprint`, `created_at`.

Template-vs-override distinction is encoded via the `agent_id` field on `blueprint`: template-rows have `agent_id = None`; per-agent override-rows have `agent_id = Some(<id>)`. Override-row uniqueness ("at most one override row per agent" without preventing template rows from being shared) is enforced at the **repository tier** — see §D63.5 for the SurrealDB 2.6.5 partial-UNIQUE limitation + Option E enforcement narrative (Repository::upsert_agent_blueprint_override wraps SELECT-count + INSERT/UPDATE in a SurrealDB transaction). The `blueprint_agent_id` index defined in migration 0019 is a NON-UNIQUE read-side lookup supporting efficient per-agent override-row retrieval; it does NOT enforce uniqueness.

**Pre-existing behaviour preserved**: per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) survive the cardinality flip; they MOVE from the `agent_profile` row to the NEW `blueprint` table per-agent override row (keyed by `agent_id`); their per-agent semantic is identical to pre-CH-28 (see `nodes.rs:314-340` pre-CH-28 for the pre-CH-28 implementation; CH-28 relocates the fields per F1.c lock outcome via NEW `Blueprint` struct + override-table backfill).

**Iter-4 cross-ref annotation (preserved at iter-5)**: per §D63.13 (NEW iter-4), the in-process `AgentProfile` struct field-set is **preserved** post-CH-28 — the 3 override fields (`parallelize`, `model_config_id`, `mock_response`) stay on the `AgentProfile` Rust struct as a **synthesized projection** of the per-agent override-Blueprint row. The persistence-of-record moves to the `blueprint` table; the in-process projection preserves the public API contract for `AgentProfile` consumers. See §D63.13 + §D63.14 (write-boundary strip via `AgentProfileWireRow`) + §D63.15 (read-path synthesis wiring) for the layered separation.

### §D63.2 — F2.b lock outcome + edge rename HAS_PROFILE → USES_PROFILE

**Decision**: Rename the existing `Edge::HasProfile` variant to `Edge::UsesProfile`. The variant body (`id: EdgeId, from: AgentId, to: NodeId`) stays identical. The `name()` impl returns `"USES_PROFILE"`. The `EDGE_KIND_NAMES` array entry flips from `"HAS_PROFILE"` to `"USES_PROFILE"`. The `DomainEvent::HasProfileEdgeChanged` variant renames to `DomainEvent::UsesProfileEdgeChanged` (variant body + `kind()` string + `event_id()` + serde roundtrip test all rename together).

Listener match-arms, doc-comments, RELATE statements, and test fixtures all rename in sympathy (43 cascade sites total — see Audit-A claims 5+6).

**Serde rename-aware deserialization**: the renamed variant carries `#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]` so EXISTING serialized edge rows (stored as JSON with `"edge": "HasProfile"` in SurrealDB) deserialize cleanly into the renamed variant during the 0020 backfill window. After 0020 backfill rewrites all existing rows to the new edge-name, the `alias` can stay (no harm) or be removed in a future cycle.

**Protected sites that do NOT rename**:
- `audit/events/m4/agents.rs:78` — JSON wire-key `"has_profile": profile.is_some()` for the m4 agent-creation audit-event; protected per ADR-0033 §D33.5 hash-chain canonical-bytes stability.
- `migrations/0001_initial.surql:314` + `migrations/0005_*.surql:182` — immutable migration source-comments.

**Pre-existing behaviour preserved**: the edge semantic (an Agent points at an AgentProfile) is unchanged; only the verb (`HAS` → `USES`) renames per F2.b lock outcome. Serde rename-aware deserialization (`#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]`) preserves wire-format back-compat with existing serialized edge rows during the 0020 backfill window. Audit-event canonical-bytes evolve per ADR-0033 §D33.5 — pre-rename hash-chain entries remain valid; post-rename entries chain from them.

### §D63.3 — F3.b lock outcome + split migrations 0019 + 0020

**Decision**: Ship TWO migrations instead of one composite migration:

(i) `0019_agent_profile_n_to_1_schema.surql` — schema-only. DROPs UNIQUE on `agent_profile.agent_id`; DEFINEs `blueprint` table + index; DEFINEs `uses_profile` RELATION (renamed from `has_profile`); DEFINEs 2 NEW RELATION edges; ALTERs `agent_profile` to remove override fields.

(ii) `0020_agent_profile_n_to_1_backfill.surql` — data-only. Backfills template blueprint rows from existing AgentProfile data; rewrites `has_profile` edges to `uses_profile`; preserves per-agent override values by re-keying onto override blueprint rows.

Both migrations are idempotent under re-run. Operator inspection window between 0019 and 0020 is permitted; runtime reads tolerate the half-migrated state via `read_agent_profile_via_blueprint_or_fallback` (see §D63.12).

**Pre-existing behaviour preserved**: migration runner semantics (per ADR-0012 + CHK8S-D-05) — migrations apply in numerical order; idempotency contract preserved for both 0019 (schema-only no-op on re-apply) and 0020 (data-only no-op on re-apply). The split (vs single composite) is a NEW precedent per F3.b lock; ADR-0063 §D63.3 codifies the split-pattern for future cycles.

### §D63.4 — Concept-doc amendment scope

**Decision**: At P-DOCS, amend the following concept-doc bodies:

- `concepts/agent.md` §"Soul (Immutable Born Structure)" lines 160–169 — reframe Soul as "template-Blueprint, shareable across agent instances + per-agent override-Blueprint for governance fields"; immutability semantic preserved at the template-row level.
- `concepts/ontology.md` line 98 — flip `Agent | HAS_PROFILE | AgentProfile | 1:1 | Blueprint identity` to `Agent | USES_PROFILE | AgentProfile | N:1 | Blueprint identity (shared template)`; add NEW rows for `AgentProfile | AGENT_PROFILE_USES_BLUEPRINT | Blueprint | N:1 | Template Blueprint pointer` + `Agent | AGENT_USES_BLUEPRINT_OVERRIDE | Blueprint | 1:N (zero-or-one) | Per-agent override Blueprint pointer`.

NEW `m6/architecture/agent-profile-cardinality.md` doc authored with 7 sections capturing the full redesign rationale.

**Pre-existing scaffold preserved**: the open-question marker M6+-OPEN-01 (chunk-assignment unchanged at M6 plan-open until CH-28); CH-28 ratifies the redesign-pursuit, implements the flip.

### §D63.5 — Schema migration body + idempotency contract

**Decision**: Both migrations adhere to the idempotency contract codified in ADR-0012.

- 0019 schema migration: `DEFINE TABLE IF NOT EXISTS`, `DEFINE FIELD IF NOT EXISTS`, `REMOVE INDEX IF EXISTS`, `DEFINE INDEX IF NOT EXISTS`. Re-applying 0019 on a post-apply database is a no-op.
- 0020 data migration: each backfill body checks for pre-existing target rows before writing (`IF NOT EXISTS` semantics via UPSERT or pre-check). Re-applying 0020 on a post-apply database is a no-op.

Half-migrated-state tolerance is handled at the runtime layer (see §D63.12), not in the migration body itself.

**SurrealDB 2.6.5 partial-UNIQUE limitation + app-tier uniqueness enforcement (NEW iter-2 retry, Option E user-locked 2026-05-19)**: the iter-2 plan §F3.b lock body originally prescribed `DEFINE INDEX blueprint_agent_id ON blueprint FIELDS agent_id UNIQUE WHERE agent_id != NONE` to enforce per-agent override-row uniqueness only on rows where `agent_id` is set (template rows with `agent_id = NONE` must be allowed to multiply). The first P-MIGRATION-SCHEMA attempt landed this clause verbatim and the migration failed at first-apply with a SurrealDB parser error — **SurrealDB 2.6.5 does NOT support filtered/partial UNIQUE indexes** (`DEFINE INDEX ... UNIQUE WHERE ...` is rejected by the parser; the `WHERE` clause is not a supported index-definition fragment in the 2.6.x grammar). The user-locked resolution (Option E, 2026-05-19) drops the `UNIQUE` keyword AND the `WHERE` clause from the index definition — `blueprint_agent_id` becomes a NON-UNIQUE read-side lookup index (supporting efficient `SELECT * FROM blueprint WHERE agent_id = $aid` queries used by the per-agent override read path). Override-row uniqueness moves to the **REPOSITORY TIER**: the `upsert_agent_blueprint_override` method (added at P1-BLUEPRINT-STRUCT per §D63.6) wraps its write path in a SurrealDB `BEGIN; ... COMMIT;` transaction that first runs `SELECT count() FROM blueprint WHERE agent_id = $aid AND id != $row_id` and rejects with a `RepositoryError::UniqueViolation` if the count is > 0; otherwise it INSERTs (or UPDATEs the matched row). The race window between SELECT and INSERT exists WITHIN the transaction but is minimal under SurrealDB's serializable transaction isolation. If SurrealDB adds partial-index support in a later major version (3.x candidate), a follow-on migration may re-tighten the constraint to DB-tier UNIQUE; this is tracked here as an informational note — no drift filed at CH-28 because the app-tier enforcement is correct + sufficient for the M6 cardinality flip.

**Pre-existing behaviour preserved**: migration runner semantics (apply-once-or-noop) per ADR-0012. The 0019/0020 split adds a NEW precedent of explicit schema-vs-data atomicity boundary while preserving the per-migration idempotency contract.

### §D63.6 — Repository trait method additions

**Decision**: Add 4 NEW methods to the `Repository` trait at `domain/src/repository.rs`:

- `async fn get_blueprint_for_agent_profile(&self, profile_id: NodeId) -> Result<Option<Blueprint>, RepositoryError>`
- `async fn get_agent_blueprint_override_for_agent(&self, agent_id: AgentId) -> Result<Option<Blueprint>, RepositoryError>`
- `async fn upsert_blueprint_template(&self, blueprint: &Blueprint) -> Result<(), RepositoryError>`
- `async fn upsert_agent_blueprint_override(&self, override: &Blueprint) -> Result<(), RepositoryError>`

In-memory + SurrealDB implementations follow the existing async-trait method pattern. Trait-shape conformance with ADR-0033 §D33.1 (trait-object-friendly) preserved.

**Pre-existing behaviour preserved**: trait-method-shape conventions (async-trait, Result return, RepositoryError error type) per ADR-0033 §D33.1.

### §D63.7 — Audit-event semantics rename + hash-chain preservation

**Decision**: `DomainEvent::HasProfileEdgeChanged` renames to `DomainEvent::UsesProfileEdgeChanged`. The audit-event kind-string changes from `has_profile_edge_changed` → `uses_profile_edge_changed`. Pre-rename audit-event rows remain valid in the hash-chain (each row chains from the prev regardless of kind-name); post-rename rows have the new kind-name in their canonical-bytes — hash-chain is byte-stable per ADR-0033 §D33.5.

Emission cardinality stays SINGLE-agent at CH-28 (was single-agent per pre-CH-28 `events/mod.rs:185-191`; CH-28 renames the variant but does not change emission cardinality). Multi-agent enrichment (one Blueprint change visible to N agents sharing the template) is deferred to M7 NFR-observability per forward-scope §7.6.

The audit-event JSON wire-key `"has_profile": profile.is_some()` at `audit/events/m4/agents.rs:78` STAYS unchanged (snake_case wire-key for the m4 agent-creation audit-event, NOT the edge variant name).

**Pre-existing behaviour preserved**: `HasProfileEdgeChanged` event (now renamed `UsesProfileEdgeChanged`) emits SINGLE-agent at CH-28 (was single-agent per `events/mod.rs:185-191`; CH-28 renames the variant but does not change emission cardinality; multi-agent enrichment deferred to M7 NFR-observability per forward-scope §7.6).

### §D63.8 — Wrap-vs-runtime separation preserved

**Decision**: The hybrid-Blueprint cardinality is a baby-phi-domain-layer decision. phi-core's `AgentProfile` type is unchanged. The wrap at `domain::AgentProfile.blueprint: phi_core::agents::profile::AgentProfile` (per `nodes.rs:322`) continues to be the single source of truth for `system_prompt`, `thinking_level`, etc. The NEW baby-phi `Blueprint` struct (introduced per F1.c) is orthogonal to phi-core; lives only at the baby-phi-domain layer.

**Pre-existing behaviour preserved**: `domain::AgentProfile.blueprint: phi_core::agents::profile::AgentProfile` wrap (see `nodes.rs:322`; ADR-0034 §D34.6 wrap-vs-runtime separation; CH-28 does not change this). NEW baby-phi `Blueprint` struct (introduced per F1.c) is orthogonal to phi-core; lives only at baby-phi-domain layer.

### §D63.9 — Cross-chunk consequences

**Decision**: The hybrid-Blueprint shape unblocks downstream M6 chunks:

- CH-29 (M6-DEFERRED-02 messaging substrate): sender/recipient FK semantic remains at the agent-id layer; Blueprint pointers are orthogonal.
- CH-36 (a04 My Work / M6-DEFERRED-04 supervisor body): supervisor body references template-Blueprint by id + per-agent override-Blueprint by agent-id.
- CH-37 (a05 My Profile + Grants): profile editor renders template-Blueprint (read-only for non-CEO) + per-agent override-Blueprint (writable). The `PATCH /api/v0/agents/:id/profile/*` endpoint patches the per-agent override-Blueprint row.

**Pre-existing scaffold preserved**: forward-scope §1 chunk dependency graph at `m6-forward-scope-8b7a8bcd.md` §4; CH-28 is the gate to all M6 foundation-tier work.

### §D63.10 — EDGE_KIND_NAMES cardinality bump 72 → 74

**Decision**: Per F1.c lock outcome, 2 NEW edge variants are added to `Edge` enum + `EDGE_KIND_NAMES` array:

- `Edge::AgentProfileUsesBlueprint { id, from: NodeId (agent_profile id), to: BlueprintId }` — template-row pointer; emitted at AgentProfile-creation time.
- `Edge::AgentUsesBlueprintOverride { id, from: AgentId, to: BlueprintId }` — override-row pointer; emitted at per-agent override-write time.

`EDGE_KIND_NAMES.len()` bumps **72 → 74** (NOT 72 — the F2.b rename keeps the count flat but F1.c independently adds 2 NEW variants). ADR-0057 §D57.7 invariant docstring updated to reflect the new cardinality. The compile-time test at `edges.rs:704` renames from `edge_kind_names_cardinality_is_72_pinned_at_compile_time` to `..._is_74_pinned_...` and asserts `74`.

**Pre-existing behaviour preserved**: EDGE_KIND_NAMES sortedness + uniqueness invariants (per `edges.rs:704-712` test bodies); the cardinality bumps 72 → 74 per F1.c lock outcome adding 2 NEW edge variants; ADR-0057 §D57.7 invariant docstring updated to reflect the new cardinality.

### §D63.11 — Test-fixture cascade migration via helper

**Decision**: Per F1.c lock, the 3 override fields (`parallelize`, `model_config_id`, `mock_response`) are REMOVED from `AgentProfile` struct entirely. The 30 TEST-FIXTURE struct-literal sites that currently construct `AgentProfile { ..., parallelize, model_config_id, mock_response, ... }` must migrate.

Migration is via a NEW helper at `server/tests/acceptance_common/blueprint_fixture.rs`:

```rust
pub fn make_test_profile_with_overrides(
    blueprint: phi_core::agents::profile::AgentProfile,
    agent_id: AgentId,
    overrides: BlueprintOverrides,
) -> (AgentProfile, Option<Blueprint>);
```

Returns the template-AgentProfile + optional per-agent override-Blueprint row. Test-fixture call-sites call this helper instead of struct-literal construction.

The v19 P2 back-compat scaffold option (b) — keep override fields on AgentProfile with `#[serde(default)]` — is NOT applicable under F1.c (the fields are REMOVED from the struct entirely).

**Pre-existing scaffold REPLACED**: the v19 P2 back-compat scaffold option (b) — keep override fields on AgentProfile with `#[serde(default)]` — is NOT applicable under F1.c (the fields are REMOVED from the struct entirely). The 30 TEST-FIXTURE AgentProfile struct-literal sites migrate to a NEW factory helper `make_test_profile_with_overrides` at `acceptance_common/blueprint_fixture.rs`. Net migration cost: ~30 sites edited mechanically via helper-call rewrite.

**Iter-4/iter-5 cross-ref annotation (preserved at iter-5)**: per §D63.13 (NEW iter-4), §D63.14 (NEW iter-5 write-boundary strip), and §D63.15 (NEW iter-5 read-path synthesis), the test-fixture cascade **dissolves** — the in-process `AgentProfile` struct field-set is preserved, so the 39 raw struct-literal sites (9 PRODUCTION + 30 TEST-FIXTURE) stay as direct struct-literal construction. The `make_test_profile_with_overrides` factory helper at `acceptance_common/blueprint_fixture.rs` ships at P2-ACCEPTANCE as an **OPTIONAL** convenience used by the 7 NEW acceptance tests; the 30 TEST-FIXTURE sites do NOT mechanically rewrite. Net migration cost at iter-5: **0 sites** rewritten (vs ~30 sites at iter-3).

### §D63.12 — Half-migrated-state runtime tolerance

**Decision**: Per F3.b lock, the 0019↔0020 inter-migration window is permitted (operator inspection use-case). A NEW runtime helper `read_agent_profile_via_blueprint_or_fallback` at `repo_impl.rs` bridges the window: tries the new path (read blueprint + synthesise) first; falls back to the old path (pre-0020 shape, override fields still on agent_profile row) if the blueprint table is empty.

The helper is shipped at CH-28 and removed in a future M7 NFR-observability cycle once the migration is universally applied.

**Cross-reference to §D63.5 SurrealDB partial-UNIQUE limitation (NEW iter-2 retry, Option E user-locked 2026-05-19)**: §D63.5 documents that override-row uniqueness is enforced at the repository tier (`upsert_agent_blueprint_override`) rather than via a DB-tier filtered UNIQUE index (SurrealDB 2.6.5 grammar limitation). The half-migrated-state runtime tolerance helper here (`read_agent_profile_via_blueprint_or_fallback`) continues to work cleanly under the app-tier enforcement model — the helper reads from the `blueprint` table without relying on DB-tier UNIQUE constraints. The 0019-applied-but-not-0020 window remains tolerable: the helper's "blueprint table empty" fallback path is independent of the uniqueness-enforcement layer. The two concerns (uniqueness enforcement vs half-migrated tolerance) are orthogonal — §D63.5 addresses *which row wins when two override candidates exist for the same agent*; §D63.12 addresses *which path serves the read when the blueprint table is empty post-0019-pre-0020*.

**Pre-existing absence preserved**: the half-migrated-state runtime tolerance is a NEW concern introduced at CH-28 per F3.b lock; the `read_agent_profile_via_blueprint_or_fallback` helper is a NEW scaffold shipped at CH-28 (no prior behaviour to preserve); planned removal at M7 NFR-observability cycle (forward-routing note: track helper removal in M7 prep).

### §D63.13 — AgentProfile in-process struct field-set preservation (NEW iter-4; preserved verbatim at iter-5)

**Decision**: Per the iter-4 Outcome-1 re-spawn (gate-2.5 P1-narrowing per user-routed Option C), the in-process `domain::AgentProfile` Rust struct field-set is **PRESERVED** post-CH-28 — the 3 override fields (`parallelize: u32`, `model_config_id: Option<String>`, `mock_response: Option<String>`) stay on the `AgentProfile` Rust struct as a **synthesized projection** of the per-agent override-Blueprint row. The persistence-of-record for these fields moves to the NEW `blueprint` table per F1.c lock outcome; the in-process projection preserves the public API contract for `AgentProfile` consumers (39 struct-literal construction sites + all downstream readers of `profile.parallelize`, `profile.model_config_id`, `profile.mock_response`).

The persistence-vs-projection split is a load-bearing invariant of the iter-4 phase decomposition: P1-BLUEPRINT-STRUCT lands the NEW `Blueprint` struct + `BlueprintId` newtype as **ADDITIVE-only** (no struct field-set removal from `AgentProfile`); P1.5-READ-BRIDGE (NEW iter-5) wires the write-boundary strip (§D63.14) + read-path synthesis (§D63.15) so the schema-vs-struct mismatch between migration 0019 `REMOVE FIELD` (shipped at P-MIGRATION-SCHEMA) and the preserved struct field-set closes cleanly at the SurrealDB write/read boundary.

**Rationale for preservation**: the alternative (removing the 3 override fields from the `AgentProfile` struct outright per iter-3's original framing) would cascade through 91 sites (39 struct-literal construction + 52 field-access readers) — a mechanical-rewrite cost orders of magnitude larger than the synthesis-shim approach. The synthesis approach also preserves CH-02 (M5.2 MockProvider) test coverage at zero churn (e.g., `agent_profile_mock_response_roundtrip_preserves_some_value` STAYS green via the synthesized projection).

**Pre-existing behaviour preserved**: the `AgentProfile` Rust struct field-set is **preserved** verbatim from pre-CH-28 (per `nodes.rs:314-340`); the override fields stay on the in-process struct as a synthesized projection; the persistence-of-record moves to the per-agent override-Blueprint row per F1.c. The public API contract for `AgentProfile` consumers is unchanged. The 39 struct-literal construction sites (9 PRODUCTION + 30 TEST-FIXTURE per the iter-1 per-file breakdown) stay as direct struct-literal construction (no mechanical rewrite).

### §D63.14 — Write-boundary strip via `AgentProfileWireRow` intermediate struct (NEW iter-5)

**Decision**: The `create_agent_profile` + `upsert_agent_profile` SurrealDB-tier bodies serialize through a `pub(crate) struct AgentProfileWireRow` at `store/src/repo_impl.rs` that mirrors `domain::AgentProfile`'s field-set MINUS the 3 override fields (`parallelize`, `model_config_id`, `mock_response`). The wire-row is constructed via a `From<&AgentProfile>` impl that picks the SurrealDB-defined fields.

**Rationale for approach (b) over alternatives (a) custom serde with `#[serde(skip_serializing_if)]` + (c) JSON `.remove()`**:
- (b) gives compile-time guarantees that the right set of fields lands at SurrealDB. Any future `AgentProfile` field-addition that should NOT persist to the `agent_profile` table is caught at compile time via a missing field on the wire-row mapping. The wire-row pattern is forward-extensible — if M7 adds more fields to the projection, the wire-row mapping is the single source of truth.
- (a) requires per-field annotations + a runtime flag; brittle under refactors; the `#[serde(skip_serializing_if)]` decorator approach couples projection-vs-persistence at the serde layer, which conflates concerns.
- (c) is string-typed + error-prone; relies on stringly-typed JSON field-name removal which is undisciplined and untestable at compile time.

**Pre-existing behaviour preserved**: AgentProfile struct field-set unchanged (per §D63.13); the wire-row pattern is a NEW serialization shim shipped at CH-28 at the SurrealDB write boundary only; it does NOT affect the AgentProfile public API; in-memory impl is unaffected (in-memory uses direct struct insertion into its `agent_profiles` HashMap).

### §D63.15 — Read-path synthesis wiring at `get_agent_profile_for_agent` + composite-write transaction semantics at `upsert_agent_profile` (NEW iter-5)

**Decision**: At P1.5-READ-BRIDGE the SurrealDB impl of `get_agent_profile_for_agent` is rewritten to: (1) call `read_agent_profile_via_blueprint_or_fallback` (helper shipped at P1 / `repo_impl.rs:491-528`); (2) return the synthesized `AgentProfile` with 3 override fields populated from the override-Blueprint (or template defaults if no override exists, or legacy `agent_profile` un-asserted-property values if half-migrated per §D63.12). The in-memory impl mirrors the same logic at `in_memory.rs:351-361` for tier-symmetry.

At P1.5 the SurrealDB impl of `upsert_agent_profile` is rewritten to: (a) STRIP the 3 override fields via `AgentProfileWireRow` (§D63.14); (b) write the `agent_profile` row via `UPDATE type::thing(...) CONTENT $wire_body`; (c) call `upsert_agent_blueprint_override(&Blueprint { agent_id: Some(...), parallelize: Some(profile.parallelize), model_config_id: profile.model_config_id.clone(), mock_response: profile.mock_response.clone(), ... })` to persist the override row. The two writes are sequential (NOT in a single outer transaction wrapper) per planner-recommendation — `upsert_agent_blueprint_override` already wraps its own internal SurrealDB `BEGIN; ... COMMIT;`.

**Failure-mode**: if the second write fails after the first succeeds, the `agent_profile` row exists WITHOUT the override-Blueprint — `get_agent_profile_for_agent` will synthesize using `read_agent_profile_via_blueprint_or_fallback` which falls back to either template-Blueprint defaults OR (during half-migrated window) the un-asserted legacy properties. Net effect: at-rest semantic is "missing override row → inherit template", which is the same semantic as freshly-created agents. **The composite-write is best-effort atomic**; full-atomic outer transaction is documentable as a §D63.15 caveat for M7 NFR-observability cleanup.

**Pre-existing behaviour preserved**: `get_agent_profile_for_agent` trait contract unchanged at the public surface (returns `Option<AgentProfile>` with the same field-set per §D63.13); the body changes are at the SurrealDB impl tier only; in-memory impl receives mirrored synthesis logic for tier-symmetry. `upsert_agent_profile` trait contract unchanged (takes `&AgentProfile`); the body now invokes `upsert_agent_blueprint_override` internally to persist the override-Blueprint row.

### §D63.16 — Listener template-tier fan-out scope-narrowing (NEW iter-5; deferred to D-CH28-FOLLOWUP-01 / M6-DEFERRED-04 / CH-36)

**Decision**: The `BlueprintUpserted` listener arm at `events/listeners.rs:1089-1106 + 1191-1196` covers the **OVERRIDE-tier** (per-agent override Blueprint upserts trigger profile snapshot refresh for the SINGLE owning agent). The **TEMPLATE-tier fan-out** (template Blueprint upserts trigger profile snapshot refresh for ALL agents whose AgentProfile points at the upserted template) is **NOT shipped** at CH-28.

Implementing template-tier fan-out requires a NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` that traverses `Blueprint → reverse AGENT_PROFILE_USES_BLUEPRINT → AgentProfile → reverse USES_PROFILE → Agent`. The narrowing is documented in-source at `listeners.rs:1094-1124` per the implementer's P1 close report.

Deferred to **`D-CH28-FOLLOWUP-01`** (filed at P-SEAL) with **`M6-DEFERRED-04`** allocation — **CH-36 a04 supervisor body** inherits the same template-tier fan-out requirement (supervisors share tuning templates across pools of supervised agents). CH-36 implements the `list_agents_using_blueprint_template` method + wires the template-tier fan-out arm.

**Pre-existing scaffold preserved**: BlueprintUpserted listener arm at `events/listeners.rs:1089-1106` ships the override-tier coverage at CH-28; template-tier fan-out coverage routed to `D-CH28-FOLLOWUP-01` with `M6-DEFERRED-04` allocation; CH-36 a04 supervisor body inherits the requirement.

---

## Cross-references

**(a) Originating concept docs + section + line ranges**:
- `docs/specs/v0/concepts/agent.md` §"Soul (Immutable Born Structure)" lines 160–169
- `docs/specs/v0/concepts/agent.md` §"Parallelized Sessions" lines 209–221
- `docs/specs/v0/concepts/ontology.md` §"Edge Types" line 98

**(b) Closed drift(s) by ID**: `M6+-OPEN-01` (open-question marker at `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370; NOT a D-* drift; brought in-milestone per Q9 user-lock 2026-05-18)

**(c) Prior ADRs cited as precedent**:
- `m5_2/decisions/0034-agent-durable-lifecycle.md` §D34.6 (wrap-vs-runtime separation; preserved at §D63.8)
- `m4/decisions/0023-org-defaults-snapshot.md` (per-agent ExecutionLimits override precedent)
- `m5_2/decisions/0038-identity-node-materialization.md` §D38.1 (eager creation pattern at `apply_agent_creation`)
- `m5_1/decisions/0057-bucket-b-ratification.md` §D57.7 (EDGE_KIND_NAMES cardinality invariant; bumped 72 → 74 under F1.c — see §D63.10)
- `m5_2/decisions/0033-k8s-prep-refactors.md` §D33.1 + §D33.2 + §D33.4 + §D33.5 (trait-shape conformance + audit hash-chain symmetry)
- `m1/decisions/0012-forward-only-migrations.md` (forward-only migration-runner contract — referenced as ADR-0012 in §D63.3 / §D63.5 bodies; gate-2 patch corrected the implementer's draft "ADR-0029" references, which mis-identified the migration ADR — ADR-0029 at `m5/decisions/0029-session-persistence-and-recorder-wrap.md` is the session-persistence/recorder-wrap ADR, not the migration-runner contract)

**(d) Forward-scope row**:
- `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 41–48
- Origin marker: `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370

---

## Consequences

### For CH-29 (M6-DEFERRED-02 messaging substrate)

Sender/recipient FK semantic unchanged at agent-id layer; under F1.c the Blueprint row pointers are orthogonal. No CH-29 plumbing required for the cardinality flip.

### For CH-36 (a04 My Work / M6-DEFERRED-04 supervisor body)

Supervisor body references template-Blueprint by id + per-agent override-Blueprint by agent-id. CH-28's hybrid Blueprint shape enables both abstractions natively. CH-36 plumbing: supervisor inherits a template-`Blueprint` reference + has its own per-agent `Blueprint` override row.

**NEW iter-5 inherited requirement (per §D63.16 + D-CH28-FOLLOWUP-01)**: CH-36 implements a NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` and wires the template-tier fan-out arm in the `BlueprintUpserted` listener at `events/listeners.rs:1089-1124`. The method traverses `Blueprint → reverse AGENT_PROFILE_USES_BLUEPRINT → AgentProfile → reverse USES_PROFILE → Agent`. CH-28 ships the override-tier listener arm only; template-tier fan-out is scope-narrowed and routed via `D-CH28-FOLLOWUP-01` with `M6-DEFERRED-04` allocation. Supervisor-tier fan-out (supervisors share tuning templates across pools of supervised agents) is the load-bearing CH-36 surface that consumes the same `list_agents_using_blueprint_template` traversal.

### For CH-37 (a05 My Profile + Grants)

Profile editor surfaces template-Blueprint (read-only for non-CEO) + per-agent override-Blueprint (writable). The `PATCH /api/v0/agents/:id/profile/*` endpoint patches the per-agent override-Blueprint row (NEW Repository methods `get_agent_blueprint_override_for_agent` + `upsert_agent_blueprint_override`). CH-37 also surfaces the adoption-flow (an agent adopting an existing template Blueprint vs creating a new one); this surface is deferred to CH-37 design via §3.E candidate 1.

### For M7 NFR-observability (deferred via §7.6)

Multi-agent audit-event enrichment for `UsesProfileEdgeChanged` + `BlueprintUpserted`. When N agents share a template Blueprint and one is changed, the audit-event should enumerate all affected agents. Deferred per forward-scope §7.6.

### For future M7 cleanup

Removal of `read_agent_profile_via_blueprint_or_fallback` helper once migration is universally applied. Track helper removal in M7 prep.

---

## Revisit triggers

1. If F1.c hybrid pattern proves operationally complex (e.g., template-vs-override distinction confuses operators), §D63.1 reopens.
2. If F2.b rename causes a runtime serde-back-compat regression in production data, §D63.2 + §D63.7 reopen.
3. If F3.b split-migration causes a first-apply ordering bug (e.g., 0020 applies before 0019 in a fresh cluster), §D63.3 + §D63.5 reopen.
4. If multi-agent audit-event enrichment becomes load-bearing at M7, §D63.7 reopens.
5. If a per-agent override field grows to ≥ 5 (i.e., CH-36 supervisor body adds 2 more overrides), §D63.1 + §D63.6 reopen (Blueprint row column-count growth re-evaluated).
6. If phi-core's AgentProfile gains an inner field overlapping with baby-phi's override surface, §D63.8 reopens (wrap-vs-runtime boundary re-evaluated).
7. If the half-migrated-state runtime tolerance helper is exercised in production (suggests migration ordering or operator-pause-pattern issue), §D63.12 reopens.
8. **NEW iter-5**: If the §D63.15 composite-write best-effort atomicity surfaces a production-tier inconsistency (e.g., `agent_profile` row exists without override-Blueprint after a crash mid-`upsert_agent_profile`), §D63.15 reopens — the M7 NFR-observability cleanup tightens the composite-write to a single outer SurrealDB `BEGIN; ... COMMIT;`.
9. **NEW iter-5**: If CH-36 a04 supervisor body discovers a template-tier fan-out scenario blocking on the deferred `list_agents_using_blueprint_template` method (e.g., template-Blueprint mutation visibility across sibling agents becomes load-bearing), §D63.16 reopens — CH-36 implements the method per `D-CH28-FOLLOWUP-01` allocation.

---

## Verification

```bash
# (a) Migration 0019 schema first-apply + idempotency
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test migration_0019

# (b) Migration 0020 backfill first-apply + idempotency
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test migration_0020

# (c) Half-migrated-state runtime tolerance
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test half_migrated_state

# (d) AgentProfile cardinality acceptance suite
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p server --test acceptance_m6_agent_profile_cardinality

# (e) EDGE_KIND_NAMES cardinality (expect 74 post-CH-28)
grep -n "EDGE_KIND_NAMES.len() ==" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs

# (f) phi-core leverage delta (expect 57, unchanged)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l

# (g) Edge rename completeness
git -C /root/projects/phi/baby-phi grep -nE 'HasProfile|HAS_PROFILE' modules/crates/domain/src/ modules/crates/store/src/ | wc -l
# Expect: 0 hits in production code (rename complete); allowed hits only at protected sites
```
