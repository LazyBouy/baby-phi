<!-- iter-5 re-author (2026-05-19): Second Architectural-FAIL re-spawn driven by user-routed Option B. P1-BLUEPRINT-STRUCT iter-4 ADDITIVE-only landed correctly (7/7 deliverables; 11/11 new tests green) but workspace `cargo test --no-fail-fast --workspace` is RED at P1 close (9 failing test targets). Root cause: migration 0019 `REMOVE FIELD` on `agent_profile.{parallelize,model_config_id,mock_response}` shipped at P-MIGRATION-SCHEMA close + AgentProfile struct still carries the 3 fields (correctly preserved per iter-4 §D63.13); but the existing `create_agent_profile` + `upsert_agent_profile` bodies in `store/src/repo_impl.rs:635,675` serialize the WHOLE AgentProfile JSON via `serde_json::to_value(profile)` and CREATE/UPDATE `agent_profile` with `CONTENT $body` — SurrealDB SCHEMAFULL after `REMOVE FIELD` rejects writes carrying the now-undefined fields. The synthesis read-path helper `read_agent_profile_via_blueprint_or_fallback` shipped at P1 (repo_impl.rs:491-528) but is NOT wired into `get_agent_profile_for_agent` body. Iter-5 INSERTS a NEW phase P1.5-READ-BRIDGE between P1 and P-EDGE-RENAME to: (a) wire the read-helper into the surreal + in-memory `get_agent_profile_for_agent` bodies; (b) introduce an `AgentProfileWireRow` intermediate struct that strips the 3 override fields at the SurrealDB write boundary in `create_agent_profile` + `upsert_agent_profile` (composite-write: surreal row WITHOUT overrides + override-Blueprint row WITH overrides); (c) add the deferred `half_migrated_state_runtime_reads_via_fallback_helper` test (Tier A); (d) acceptance criterion: `cargo test --workspace --no-fail-fast` GREEN at P1.5 close. P2-HANDLERS renamed to P2-ACCEPTANCE and SCOPE-COLLAPSED — under iter-4 §D63.13 the 91-site cascade dissolves (struct field-set preserved → no mechanical rewrite; write-path strip already in P1.5 → no per-handler edits); P2-ACCEPTANCE retains ONLY: 7 NEW acceptance tests in `acceptance_m6_agent_profile_cardinality.rs` + OPTIONAL factory helper at `acceptance_common/blueprint_fixture.rs`. ADR §D63.13 sub-decisions extended to 16: NEW §D63.14 (write-boundary strip via AgentProfileWireRow), NEW §D63.15 (read-path synthesis wiring + composite-write transaction semantics), NEW §D63.16 (listener template-tier fan-out scope-narrowing → D-CH28-FOLLOWUP-01 M6-DEFERRED). All 3 locked forks F1.c + F2.b + F3.b preserved verbatim. Audit envelope LARGE preserved (P-DOCS + P-MIGRATION-SCHEMA + P-MIGRATION-BACKFILL + P1-BLUEPRINT-STRUCT [DONE] + P1.5-READ-BRIDGE + P-EDGE-RENAME + P2-ACCEPTANCE = 7 substantive phases). §8 band [1590, 1604] preserved (P1.5 adds ~3-4 tests within band). Recommended gate: gate-1.5 approval → resume at P1.5-READ-BRIDGE. -->
<!-- Last verified: 2026-05-19 by Claude Code (chunk-planner v22 iter-5 re-spawn; CH-28 cycle hex 0412eb06; second Architectural-FAIL re-spawn — P1.5-READ-BRIDGE phase INSERTED between P1-BLUEPRINT-STRUCT [DONE 7/7 deliverables] and P-EDGE-RENAME; user-routed Option B 2026-05-19; chosen write-boundary strip approach: AgentProfileWireRow intermediate struct (approach b per re-spawn mandate point 1); P2-HANDLERS scope-collapsed → renamed P2-ACCEPTANCE; ADR sub-decisions 13 → 16 NEW §D63.14 + §D63.15 + §D63.16; all 3 locked forks F1.c + F2.b + F3.b preserved verbatim; iter-4 §D63.13 in-process struct preservation invariant carried forward unchanged; phases 7 → 7 (P1.5 replaces P2-HANDLERS substantive-tier scope; P2-ACCEPTANCE is a thin tests-only phase). -->

# CH-28 — AgentProfile cardinality 1:1 → N:1 redesign (hybrid blueprint table + edge rename + split migrations)

> **8hex token**: `0412eb06`
> **Slug**: `ch-28-agentprofile-cardinality-redesign`
> **Forward-scope row**: [`m6-forward-scope-8b7a8bcd.md` §1 "Foundation tier" lines 41–48](../../forward-scope/m6-forward-scope-8b7a8bcd.md#§1)
> **Drift closed**: `M6+-OPEN-01` (open-question marker; brought in-milestone per Q9 user-lock 2026-05-18)
> **Severity / Effort**: ⚠HIGH · 5–7 ed
> **Prerequisites**: none (M5.3 closed at CH-27 `0edcaba9`); first chunk of M6
> **K8s posture**: A1 re-evaluated; A4 gains 2 new migrations (0019 schema + 0020 data backfill) — idempotency verified for both; both migrations already first-apply green at chunk entry per iter-3 P-MIGRATION-BACKFILL close (5/5 migration tests green)
> **phi-core leverage delta**: 0 (no new phi-core types consumed; AgentProfile wrap continues to embed `phi_core::agents::profile::AgentProfile` unchanged)
> **Audit envelope**: **LARGE (3 auditors A + B + C)** — tier preserved from iter-2/iter-3/iter-4; see §11
> **Confidence target**: 9/10 at chunk close

---

## Forks for orchestrator

> ⚠ **CROSS-CYCLE DIVERGENCE PATTERN — 3-OF-3 LOCKED DIVERGENT THIS CYCLE**: at gate-1 the user locked **all 3 forks DIVERGENT** from planner-rec (F1.c hybrid blueprint table over F1.a Agent columns; F2.b USES_PROFILE rename over F2.a keep HAS_PROFILE; F3.b split migrations over F3.a single composite). Cumulative cross-cycle divergent-fork count was 11-of-16 (~69%) at CH-27 close; CH-28 adds 3 more → **14-of-19 (~74%) cumulative**. The F<X>.b/F<X>.c expansion-divergence pattern is structurally durable (10 cycles of evidence now: baby-phi CH-15/17/18/20/24/25/28 + i-phi CH-02a/02b/02c). User systematically prefers tighter / more-fragmented / more-defensive / wire-format-explicit options at gate-1. **Treat divergence as the modal outcome; v9 surfacing-not-suppressing recommendation framing held at iter-1; per v13 parity-weighting applies for tighter/richer/more-defensive options.** Iter-2 absorbed the 3 locks; iter-3 swapped phase positions 5 + 6 (P1-BLUEPRINT-STRUCT and P-EDGE-RENAME); iter-4 narrowed P1-BLUEPRINT-STRUCT to ADDITIVE-only + relocated the field-removal+synthesis cascade into P2-HANDLERS; iter-5 INSERTS P1.5-READ-BRIDGE between P1 [DONE] and P-EDGE-RENAME and SCOPE-COLLAPSES P2-HANDLERS to P2-ACCEPTANCE. The 3 locks F1.c + F2.b + F3.b stay verbatim at iter-5.

Three gate-1 forks. **All three locked DIVERGENT at gate-1 (2026-05-19)**. Plan approval was via AskUserQuestion + ExitPlanMode (criterion 1 fails because of the locks; criterion 7 fails because 2 new migrations ship).

### F1 — Per-agent override storage location — **LOCKED at gate-1, DIVERGENT (planner-rec F1.a → user-lock F1.c)**

| Option | Shape | Pros | Cons | Status |
|---|---|---|---|---|
| **F1.a** (planner-rec, parity-weighted) | Move per-agent overrides to `Agent` node (3 NEW nullable fields on `agent` table); `AgentProfile` keeps blueprint + 1:N back-edge | Minimal schema disruption; per-agent governance lives WITH the per-agent identity | Mixes governance concerns with identity row | NOT chosen |
| **F1.b** (cross-cycle divergent candidate iter-1) | NEW separate `agent_profile_override` table; AgentProfile keeps blueprint-only; Agent node unchanged | Cleanest separation (blueprint vs override vs identity) | NEW table + cascade through 5 read sites | NOT chosen |
| **F1.c** (cross-cycle-pattern divergent — **USER-LOCKED**) | **NEW `blueprint` table shared across BOTH `agent_profile` rows AND per-agent overrides; both `AgentProfile` and `Agent` carry RELATION edges to the blueprint row(s); per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) live in the blueprint table per-row keyed by `agent_id`** | Maximum auditability (every state change traceable to a blueprint row); maximum wire-format-explicit separation; downstream chunks (CH-36 supervisor, CH-37 profile editor) inherit a single canonical "override-bearing row" abstraction; future blueprint-time-travel / blueprint-version-pinning becomes natural | Maximum schema complexity (NEW table + 2 RELATION edge variants + per-blueprint-row scope of overrides); ~3× the cascade vs F1.a; introduces a per-blueprint-row decision (template-row vs override-row distinguished by an `is_template: bool` discriminant OR by edge type — sub-decision §D63.1.a per ADR draft) | **LOCKED** |

**Locked rationale**: maximum auditability + wire-format-explicit separation; aligns with the 10-cycle F<X>.b/c divergence pattern (user systematically prefers wire-format-explicit options). See **Locked fork details — F1** below.

### F2 — Edge naming — **LOCKED at gate-1, DIVERGENT (planner-rec F2.a → user-lock F2.b)**

| Option | Shape | Pros | Cons | Status |
|---|---|---|---|---|
| **F2.a** (planner-rec) | Keep `Edge::HasProfile` variant; relax cardinality only | Zero rename cascade through ~43 sites; preserves `EDGE_KIND_NAMES` invariant | "HAS" verb subtly implies ownership/uniqueness | NOT chosen |
| **F2.b** (**USER-LOCKED**) | **Rename `Edge::HasProfile` → `Edge::UsesProfile` across all enumerated sites in `EDGE_KIND_NAMES` + `Edge` enum + edge-construction sites + serde rename-aware deserialization for back-compat with existing serialized edges + migration to rewrite existing edges from `HAS_PROFILE` → `USES_PROFILE`. `EDGE_KIND_NAMES.len()` stays at 72 (rename, not add).** | Reflects template-sharing semantic accurately ("uses" implies non-ownership); single canonical edge name; ADR-0057 §D57.7 EDGE_KIND_NAMES cardinality invariant preserved | Cascade through 43 sites (verified at iter-2 plan-draft via `git -C ... grep -nE 'HasProfile\|HAS_PROFILE\|has_profile'`): 18 in `domain` (edges.rs + events/mod.rs + events/listeners.rs + composites_m5.rs + repository.rs doc-comments) + 11 in `store` (repo_impl.rs RELATE statements + 2 migration source-comments) + 1 in `server/platform/agents/update.rs` + 12 test-fixture/source-comment sites; serde back-compat alias required (`#[serde(rename = "HAS_PROFILE", alias = "USES_PROFILE")]` initially during 0020 backfill window OR `#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]` if F-serde-direction.a locks) | **LOCKED** |
| **F2.c** (open at iter-1, NOT considered at gate-1) | Keep HasProfile AND add UsesProfile | Forward-flexible | Two-edge surface confusion | NOT chosen |

**Locked rationale**: verb-name matches template-sharing semantic; serde rename-aware deserialization absorbs the back-compat cost. See **Locked fork details — F2** below.

### F3 — Migration ordering — **LOCKED at gate-1, DIVERGENT (planner-rec F3.a → user-lock F3.b)**

| Option | Shape | Pros | Cons | Status |
|---|---|---|---|---|
| **F3.a** (planner-rec) | Single composite 0019 migration (schema + backfill atomic) | Single atomic unit; matches CH-16/CH-25 precedent | Larger migration body | NOT chosen |
| **F3.b** (**USER-LOCKED**) | **Split: `0019_agent_profile_n_to_1_schema.surql` (schema-only: DROP UNIQUE on agent_profile.agent_id + NEW blueprint table + NEW RELATION edges UsesProfile + edge rename body + per-agent override fields on blueprint) + `0020_agent_profile_n_to_1_backfill.surql` (data-only: re-key existing per-agent agent_profile rows; rewrite existing HAS_PROFILE edges to USES_PROFILE; populate blueprint rows from current AgentProfile + Agent override values)** | Operator can inspect schema before backfill triggers; cleaner rollback semantics (rollback 0020 leaves 0019 schema intact); 2 atomic units smaller than single composite | First-apply at 0019 leaves DB in a half-migrated state (schema flipped, data not yet keyed) — handled at 0019 by making 0020's backfill idempotent + at the runtime level by reads being tolerant of EITHER pre-0020 OR post-0020 shapes (the synthesised AgentProfile read continues to work because the new fields are `option<...>` with sensible defaults) | **LOCKED** |

**Locked rationale**: split-migration cleaner rollback + operator inspection window. See **Locked fork details — F3** below.

**Implementer-side phase choice (per chunk-implementer R3 P-FIXTURES actuals snapshot rule + this plan's authority)**: the iter-2 plan **splits the migration work into TWO phases — P-MIGRATION-SCHEMA (0019) and P-MIGRATION-BACKFILL (0020)**. This is the planner's call per the F3.b locked-fork text. Two-phase migration aligns with the per-cycle P-FIXTURES actuals snapshot discipline (each migration phase reports its actuals; orchestrator confirms before the next migration phase opens). See §7 phase enumeration.

---

### Locked fork details — what each lock actually means

**v22 P13 self-check loop applied (iter-2)**: this section is mandatory because ≥ 1 fork is gate-1-locked. The chunk-archive-plan skill v3 hard-asserts the heading exists when any `LOCKED at gate-1` token is present in the plan. Planner self-emits the appendix at iter-2 archive (NOT punted to orchestrator post-draft cleanup). **This is the inaugural empirical validation of v22 P13** — iter-2 ships the appendix natively, closing the 2-of-2-cycle regression (CH-03-i-phi + CH-04-i-phi) where iter-2 planners had to be patched post-draft.

**iter-3 preservation note**: this appendix was preserved verbatim from iter-2.

**iter-4 preservation note**: this appendix continues PRESERVED VERBATIM at iter-4. The Outcome-1 narrow-P1-cascade-relocate-to-P2 fix does NOT change WHAT each lock means — it changes the implementation phase scoping. The locked fork bodies + Option E amendments (SurrealDB partial-UNIQUE limitation; repository-tier enforcement; non-UNIQUE index) are unchanged.

**iter-5 preservation note**: this appendix continues PRESERVED VERBATIM at iter-5 per re-spawn mandate point 7 + memory `feedback_locked_fork_details_appendix.md`. The P1.5-READ-BRIDGE insertion + P2 scope-collapse to P2-ACCEPTANCE do NOT change WHAT each lock means — they change the implementation phase scoping. The locked fork bodies + Option E amendments stay intact.

#### F1 = F1.c — Hybrid blueprint table shared across AgentProfile rows AND per-agent overrides

**What the lock means for the implementer.** Implementer creates a NEW `blueprint` table (SCHEMAFULL) in migration 0019 with 6 fields: `id` (string, primary key), `agent_id` (string, optional — None means "this is a shared-template row, not an agent-keyed override row"), `parallelize` (option<int>, default NONE), `model_config_id` (option<string>, default NONE), `mock_response` (option<string>, default NONE), `created_at` (datetime). The implementer adds a NEW `Blueprint` struct + `BlueprintId` newtype in `domain/src/model/nodes.rs` adjacent to `AgentProfile` (around lines 295-340). The `blueprint` rows are reached via NEW Edge variants: `Edge::AgentProfileUsesBlueprint { id, from: NodeId (agent_profile id), to: BlueprintId }` (template-row pointer; emitted at AgentProfile-creation time) AND `Edge::AgentUsesBlueprintOverride { id, from: AgentId, to: BlueprintId }` (override-row pointer; emitted at per-agent override-write time). Both Edge variants are NEW additions to `EDGE_KIND_NAMES` — cardinality bumps **72 → 74** (NOT 72 — the iter-1 banner saying "EDGE_KIND_NAMES stays 72" referred to F2-only; the F1.c lock independently adds 2 edge variants). Per-agent override values (`parallelize`, `model_config_id`, `mock_response`) move OFF `AgentProfile` literal entirely and live on the per-agent `Blueprint` row via `AgentUsesBlueprintOverride`; the AgentProfile row keeps only `id`, `blueprint_id` (FK to template `Blueprint`), `created_at`. (Note: the rationale for keeping override fields on the `AgentProfile` literal with `#[serde(default)]` for back-compat — the v19 P2 scaffold — is **NOT applicable here** because F1.c removes the fields from the struct entirely; the test-fixture cascade is 30 sites that NOW need explicit migration via re-routing through the new `Blueprint` writer helper.)

**What it implies for downstream consumers.** CH-36 (M6-DEFERRED-04 supervisor body) consumes the `Blueprint` row pattern when modeling supervisor tuning: the supervisor inherits a template-`Blueprint` reference + has its own per-agent `Blueprint` override row. CH-37 (a05 profile editor) surfaces the `Blueprint` row distinction at the UI tier — the editor renders TWO surfaces: (i) template-`Blueprint` (read-only for non-CEO agents) (ii) per-agent `Blueprint` (writable for the agent + the agent's CEO). The `PATCH /api/v0/agents/:id/profile/*` endpoint patches the per-agent `Blueprint` override row (NEW Repository methods `get_agent_blueprint_override_for_agent` + `upsert_agent_blueprint_override`). CH-29 (M6-DEFERRED-02 messaging substrate) is unblocked: sender/recipient FK semantic remains at the agent-id layer, unchanged. The hybrid pattern is forward-stable: future per-agent governance fields land on the `Blueprint` row (one new column) rather than the Agent row (which stays minimal per ADR-0034 §D34.6 wrap-vs-runtime separation).

**Which open-questions it closes.** Closes `M6+-OPEN-01` (open-question marker at `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370 — "AgentProfile cardinality 1:1 → N:1 redesign"). Closes forward-scope `m6-forward-scope-8b7a8bcd.md` §6 Q9 (user-lock 2026-05-18 to pursue the flip). Does NOT touch concept-doc `agent.md` §"Soul" L160–169's open invariants beyond the redesign — the Soul body is amended to describe template-Blueprint sharing in P-DOCS.

#### F2 = F2.b — Rename HAS_PROFILE → USES_PROFILE across all enumerated sites + serde back-compat aliases + migration edge rewrite

**What the lock means for the implementer.** Implementer renames the existing `Edge::HasProfile` variant to `Edge::UsesProfile` in `domain/src/model/edges.rs:33-37` (the variant body — `id: EdgeId, from: AgentId, to: NodeId` — stays identical). The `name()` impl at `edges.rs:455` returns `"USES_PROFILE"` (was `"HAS_PROFILE"`). The `EDGE_KIND_NAMES` array entry at `edges.rs:547` flips from `"HAS_PROFILE"` to `"USES_PROFILE"`. `EDGE_KIND_NAMES.len()` STAYS at 72 (rename, not add — F1.c's NEW edge variants bump it to 74 independently; F2.b alone is 72→72). The `DomainEvent::HasProfileEdgeChanged` variant at `events/mod.rs:185-191` renames to `DomainEvent::UsesProfileEdgeChanged` (the variant body + `kind()` string at `:255` + `event_id()` at `:273` + serde roundtrip test at `:537-550` all rename together — ~7 sites in `events/mod.rs`). Listener match-arms at `events/listeners.rs:1063, 1149, 2656-2657, 2818-2854` rename (5 sites). `composites_m5.rs:121, 139` doc-comments rename (2 sites). `repository.rs:468, 1409, 1479` doc-comments rename (3 sites). `store/src/repo_impl.rs:2399-2402, 2729-2730` RELATE statements flip from `RELATE $a -> has_profile -> $prof` to `RELATE $a -> uses_profile -> $prof` (4 sites). `server/src/platform/agents/update.rs:449,456` event emission renames (1 site). `apply_org_creation_tx_test.rs:234` counts table rename (1 site). The audit-events JSON key at `audit/events/m4/agents.rs:78` `"has_profile": profile.is_some()` STAYS as `"has_profile"` — it's a snake_case JSON wire-format key for the audit event, not the edge variant name; renaming this would break audit-event hash-chain byte-stability per ADR-0033 §D33.5 (audit-event canonical-bytes are append-only-stable). The `0001_initial.surql:314` migration source-comment `DEFINE TABLE has_profile TYPE RELATION FROM agent TO agent_profile` STAYS (migration files are immutable post-apply); migration 0019 ADDS the new table `DEFINE TABLE uses_profile TYPE RELATION FROM agent TO blueprint` (per F1.c). **Serde rename-aware deserialization**: the `Edge` enum has `#[serde(tag = "edge")]` at line 30; the renamed variant carries `#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]` so EXISTING serialized edge rows (stored as JSON with `"edge": "HasProfile"` in SurrealDB) deserialize cleanly into the renamed variant during the 0020 backfill window. After 0020 backfill rewrites all existing rows to the new edge-name, the `alias` can stay (no harm) or be removed in a future cycle.

**What it implies for downstream consumers.** Concept doc `ontology.md` L98 updates to `Agent | USES_PROFILE | AgentProfile | N:1 | Blueprint identity (shared template)`. ADR-0057 §D57.7 invariant docstring updates to reference the renamed edge name. Future chunks reading edge-name from `EDGE_KIND_NAMES` see `USES_PROFILE`. Audit-event-name `has_profile_edge_changed` (kind string at `events/mod.rs:255`) renames to `uses_profile_edge_changed` — this is a load-bearing audit-event-name change per ADR-0033 §D33.5; the canonical-bytes for new audit-event rows post-flip differ from pre-flip rows; the hash-chain is preserved (each new entry chains from the previous regardless of name); old audit-event rows with `kind = "has_profile_edge_changed"` remain valid in the chain. (Note: this is the same audit-event-name evolution pattern applied at every cardinality-tightening cycle; ADR-0063 §D63.7 documents the rename precedent.)

**Which open-questions it closes.** Closes forward-scope literal "uses_profile" wording at `m6-forward-scope-8b7a8bcd.md` §1 line 43 (the author's prose at 2026-05-18) — ratified as a binding rename rather than scoping-gloss (the opposite of iter-1 §3.D's F2.a re-interpretation). Closes ADR-0057 §D57.7 EDGE_KIND_NAMES invariant cardinality-pinning at 72 (preserved via rename-not-add).

#### F3 = F3.b — Split migrations 0019 (schema) + 0020 (data backfill) + dual P-MIGRATION phases

**What the lock means for the implementer.** Implementer writes **TWO migrations**:

(i) `modules/crates/store/migrations/0019_agent_profile_n_to_1_schema.surql` — schema-only. Body: DROP INDEX `agent_profile_agent_id` (the UNIQUE on the existing 1:1 lock); DEFINE TABLE `blueprint SCHEMAFULL`; DEFINE FIELD `agent_id` on it (option<string>, default NONE), `parallelize` (option<int>, default NONE), `model_config_id` (option<string>, default NONE), `mock_response` (option<string>, default NONE), `created_at` (datetime); DEFINE INDEX `blueprint_agent_id ON blueprint FIELDS agent_id UNIQUE WHERE agent_id != NONE` (UNIQUE only for override rows, NOT template rows); DEFINE TABLE `uses_profile TYPE RELATION FROM agent TO agent_profile` (renamed from `has_profile`); DEFINE TABLE `agent_profile_uses_blueprint TYPE RELATION FROM agent_profile TO blueprint`; DEFINE TABLE `agent_uses_blueprint_override TYPE RELATION FROM agent TO blueprint`; ALTER `agent_profile` table to remove the per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) — these fields are migrated to the `blueprint` table at 0020. **Idempotency**: re-running 0019 on a post-apply database is a no-op (UNIQUE already dropped; tables already defined; ALTER already applied).

(ii) `modules/crates/store/migrations/0020_agent_profile_n_to_1_backfill.surql` — data-only. Body: for each existing `agent_profile` row, CREATE a `blueprint` template-row (`id = blueprint_<existing_profile_id>`; `agent_id = NONE`; copy `parallelize` / `model_config_id` / `mock_response` from the old `agent_profile` row); RELATE the existing `agent_profile` row to the new template `blueprint` via `agent_profile_uses_blueprint`; for each existing `has_profile` edge row, REMOVE the row and RELATE the corresponding `agent -> agent_profile` via `uses_profile` (the renamed edge); the per-agent override rows are NOT created at backfill (each agent inherits its template's defaults); per-agent overrides are written lazily by application code when an agent's profile is mutated. **Idempotency**: re-running 0020 on a post-apply database is a no-op (template `blueprint` rows already exist; `has_profile` rows already removed; `uses_profile` rows already created). **Half-migrated state tolerance**: between 0019 first-apply and 0020 first-apply (which the operator might pause between for inspection per F3.b's design intent), the runtime reads of `get_agent_profile_for_agent` must tolerate either (a) pre-0020 shape (`agent_profile` row STILL has the override fields per F-half-migrated-tolerance.a) OR (b) post-0020 shape (`blueprint` template-row + edges). The implementer adds a runtime-level `read_agent_profile_via_blueprint_or_fallback` helper at `repo_impl.rs` that tries the new path first + falls back to the old path if the blueprint table is empty. This helper is removed in a future cycle once the migration is universally applied (M7 NFR-observability cycle).

**What it implies for downstream consumers.** Two cycle-index migration entries instead of one; two ledger entries for K8s A4 axis (both fall under the existing CHK8S-D-05 leader-election-lock cover). Future chunks that touch the migration list table (e.g., M7 NFR-observability) inherit the 0019+0020 precedent for splitting schema-vs-data migrations when the operator-inspection-window matters. The 0020 backfill is run automatically by the migration runner on first-boot; operators don't need to take manual action.

**Which open-questions it closes.** Sets a NEW precedent (the first split-migration in the workspace) — codified in `m1/architecture/schema-migrations.md` §"Split-migration pattern (NEW)" P-DOCS deliverable. Future cycles facing the same "operator wants inspection window between schema and data" decision can cite CH-28 / ADR-0063 §D63.3 as precedent.

---

## §1 — Context & principle

**Why this chunk.** CH-28 closes the long-open M6+-OPEN-01 concept re-evaluation marker (surfaced at CH-01 plan-review 2026-04-27; brought in-milestone for M6 per Q9 user-lock 2026-05-18) that asks: *should `domain::Agent` ↔ `domain::AgentProfile` cardinality flip from 1:1 ("profile-as-genetics") to N:1 ("profile-as-template, shared across agents")?* The user has decided to **pursue the flip** at M6 plan-open. The iter-2 re-spawn absorbs the user's gate-1 user-lock of F1.c (hybrid blueprint table) + F2.b (rename HAS_PROFILE → USES_PROFILE) + F3.b (split migrations). The downstream consequence is two-fold: (a) **CH-37 a05** needs the shared-Blueprint shape to render the profile-editor surface honestly; (b) **CH-36 a04 / M6-DEFERRED-04** supervisor body references Blueprint by id and must distinguish template-Blueprint from per-agent override-Blueprint.

**Quality-over-speed restatement.** *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* The CH-28 chunk-specific application: the concept-doc amendment (`agent.md` §Soul + `ontology.md` L98 cardinality row + edge name rename) lands FIRST, then code aligns. The pre-existing 1:1 + HAS_PROFILE framing was concept-mandated; the N:1 + USES_PROFILE + hybrid-Blueprint framing requires concept-doc body amendment, which this chunk authors in P-DOCS BEFORE code lands in P1/P1.5/P-EDGE-RENAME/P2.

**Forward-scope reference.** [`m6-forward-scope-8b7a8bcd.md` §1 "Foundation tier" lines 41–48](../../forward-scope/m6-forward-scope-8b7a8bcd.md#L41-L48). Origin marker: [`remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md#L356-L370). CH-01 plan-review provenance: [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](../ch-01-agent-durable-lifecycle-2aa37c80.md#L206).

**Iter-2 deviation note**: this plan was iter-2 of a re-spawn cycle. Iter-1 (planner-rec F1.a + F2.a + F3.a, audit envelope MEDIUM, 5 phases, ~9 production-tier deliverables) was reviewed by the user at gate-1 (2026-05-19); user locked 3-of-3 forks DIVERGENT. Iter-2 re-derived §3.B pause-thresholds, §5 ADR sub-decisions (9 → 12), §7 phase count (5 → 7), §11 audit envelope (MEDIUM → LARGE), §8 test-count band ([1584, 1592] → [1590, 1604]).

**Iter-3 deviation note**: this plan was iter-3 of a re-spawn cycle (gate-2.5 phase-order correction). Iter-2's phase ordering placed P-EDGE-RENAME (step 5) BEFORE P1-BLUEPRINT-STRUCT (step 6). Iter-3 swapped the two phases. The iter-3 §7.0 rationale framed the swap as closing a **compile-time** struct-schema lockstep gap. **This framing was empirically incorrect** — see iter-4 deviation note below.

**Iter-4 deviation note**: this plan was iter-4 of a re-spawn cycle (Architectural-FAIL re-spawn, gate-2.5 P1-narrowing per user-routed Option C). Chunk-implementer surfaced (i) empirical falsification of iter-3 §7.0 compile-time root-cause (the cascade was RUNTIME-test, not compile-time); (ii) internal inconsistency between iter-3 P1 deliverable 1 ("remove 3 fields from AgentProfile struct"), P2 deliverable 6 ("body synthesises returned struct"), and §8 line 660. Iter-4 chose Outcome-1: keep iter-3 phase ordering; narrow P1-BLUEPRINT-STRUCT to ADDITIVE-only; relocate field-removal+91-site cascade into P2-HANDLERS; add NEW §D63.13 sub-decision (AgentProfile in-process struct field-set preservation: persistence-of-record moves to Blueprint; in-process struct preserved as synthesized projection).

**Iter-5 deviation note (THIS ITERATION; second Architectural-FAIL re-spawn, user-routed Option B 2026-05-19)**: this plan is iter-5 of a re-spawn cycle. P1-BLUEPRINT-STRUCT (iter-4 ADDITIVE-only) shipped 7/7 deliverables at P1 close (Blueprint struct + BlueprintId + 4 trait methods + 2 NEW Edge variants + DomainEvent::BlueprintUpserted + listener arm + 11 NEW inline tests all green; EDGE_KIND_NAMES.len() == 74 invariant test passes; clippy + CI guards green). **However**, workspace `cargo test --no-fail-fast --workspace` is RED at P1 close — 9 failing test targets:

- `-p server --test acceptance_agents_profile`, `acceptance_m4`, `acceptance_m5_memory`, `acceptance_m5_sessions`, `acceptance_sessions_m5p4`, `acceptance_system_agents`, `acceptance_system_flows_s03`, `sse_live_stream_test`.
- `-p store --test repository_test` — 2 isolated failures: `create_agent_profile_persists_row` (SCHEMAFULL rejects unknown fields on the `agent_profile` row write because migration 0019 `REMOVE FIELD parallelize / model_config_id / mock_response` already shipped) + `agent_profile_mock_response_roundtrips_through_repo` (deserialization fails on missing field).

**Root cause analysis**: migration 0019 `REMOVE FIELD` on `agent_profile.{parallelize, model_config_id, mock_response}` already shipped at P-MIGRATION-SCHEMA close. Per iter-4 §D63.13, the in-process AgentProfile struct correctly retains these 3 fields (synthesized projection). But the existing `create_agent_profile` body at `store/src/repo_impl.rs:635-645` calls `let body = strip_id(serde_json::to_value(profile)?)` → CREATE `agent_profile` CONTENT $body — the body includes `parallelize`, `model_config_id`, `mock_response`. SurrealDB SCHEMAFULL on `agent_profile` table rejects writes carrying these (now undefined) fields. Same root cause for `upsert_agent_profile` (line 675-696). The bridge code that would route these field values into the `blueprint` table at write time + synthesize them from the `blueprint` table at read time was scoped into P2-HANDLERS per iter-4, but the bridge MUST land BEFORE P-EDGE-RENAME / P2-* to close the workspace-RED window. The implementer's P1 close report frames this as the second Architectural-FAIL escalation; user routed Option B (re-spawn planner for iter-5 formal phase re-author).

**Iter-5 user-routed Option B fix**:

- **Insert NEW phase P1.5-READ-BRIDGE** between P1-BLUEPRINT-STRUCT (DONE) and P-EDGE-RENAME. Mandate:
  - **Read-path wiring**: wire the existing `read_agent_profile_via_blueprint_or_fallback` helper (already shipped at `repo_impl.rs:491-528`) INTO the `get_agent_profile_for_agent` body for the SurrealDB impl. Mirror the helper logic into the in-memory impl at `domain/src/in_memory.rs:351-361`.
  - **Write-path strip**: introduce a NEW `AgentProfileWireRow` intermediate struct at `store/src/repo_impl.rs` (approach **(b)** per re-spawn mandate point 1 — chosen by planner for type-safety + no runtime branches). The wire-row mirrors AgentProfile field-set MINUS the 3 override fields (`parallelize`, `model_config_id`, `mock_response`). `create_agent_profile` + `upsert_agent_profile` serialize via `AgentProfileWireRow` at the SurrealDB boundary; the 3 override values are written to the `blueprint` table via composite-write call to `upsert_agent_blueprint_override` wrapped in a single SurrealDB `BEGIN; ... COMMIT;` transaction.
  - **Composite-write semantics**: after the `agent_profile` row write succeeds, internally call `upsert_agent_blueprint_override` with a Blueprint row carrying `agent_id = Some(profile.agent_id)` + the 3 override values (if any are non-default OR always for round-trip integrity per the implementer's choice). The composite-write is atomic via SurrealDB transaction; if the second write fails, the first is rolled back.
  - **Add deferred Tier-A test** `half_migrated_state_runtime_reads_via_fallback_helper` (the helper itself shipped at P1; this test exercises the fallback path when only 0019 is applied — not 0020).
  - **Acceptance criterion (LOAD-BEARING)**: `cargo test --workspace --no-fail-fast` returns GREEN at P1.5 close. If RED, PAUSE via AskUserQuestion; do NOT proceed to P-EDGE-RENAME.

- **Rename P2-HANDLERS → P2-ACCEPTANCE + SCOPE-COLLAPSE**: under iter-4 §D63.13 (AgentProfile in-process struct field-set preserved as synthesized projection), the 91-site cascade DISSOLVES:
  - 39 PRODUCTION + TEST-FIXTURE struct-literal sites STAY as struct-literal construction — NO mechanical rewrite.
  - 22 field-read call-sites STAY compilable — the synthesis at P1.5-wired `get_agent_profile_for_agent` populates the 3 fields from the override-Blueprint (or template defaults).
  - The 9 PRODUCTION write-path sites STAY as direct AgentProfile struct-literal construction; the composite-write at `upsert_agent_profile` body (landing at P1.5) absorbs the echo into the Blueprint table transparently.
  - Net P2 scope: 7 NEW acceptance tests in `acceptance_m6_agent_profile_cardinality.rs` + OPTIONAL factory helper `make_test_profile_with_overrides` at `acceptance_common/blueprint_fixture.rs`. NO production-tier code changes.

- **§5 ADR sub-decisions 13 → 16** (iter-5 expansion):
  - **§D63.14 (NEW iter-5)** — Write-boundary strip via `AgentProfileWireRow` intermediate struct (approach b chosen over a/c per planner type-safety analysis).
  - **§D63.15 (NEW iter-5)** — Read-path synthesis wiring at `get_agent_profile_for_agent` + composite-write transaction semantics at `upsert_agent_profile` (atomicity + failure-mode discussion).
  - **§D63.16 (NEW iter-5)** — Listener template-tier fan-out scope-narrowing (BlueprintUpserted template-tier fan-out across all agents whose AgentProfile points at the upserted template) — DEFERRED to follow-up drift `D-CH28-FOLLOWUP-01` with `M6-DEFERRED-04` (a04 supervisor) allocation; requires NEW Repository method `list_agents_using_blueprint_template(BlueprintId)`; rare migration-time operation; not load-bearing for CH-28 close.

- **§D63.13 + §D63.1 + §D63.11 preserved verbatim** from iter-4. iter-5 builds atop the §D63.13 in-process-struct-preservation invariant.

- **§3 phi-core leverage map preserved verbatim** (no phi-core delta).

- **§8 chunk-close prediction band preserved [1590, 1604]** (P1.5 adds ~3-4 tests; total NEW MUST-SHIP 21 → 24-25; still within band's upper bound). Per-Tier breakdown updated below in §8.

- **§11 audit envelope LARGE preserved**: 7 substantive phases (P0 + P-DOCS + P-MIGRATION-SCHEMA + P-MIGRATION-BACKFILL + P1-BLUEPRINT-STRUCT [DONE] + P1.5-READ-BRIDGE + P-EDGE-RENAME + P2-ACCEPTANCE + P-SEAL = 9 phase headers; orchestrator counts 7 substantive). Audit prompts updated (Audit-A claim 8 + 11 wording updated; Audit-A NEW claim 18 verifies wire-row strip; Audit-B claim 1 sub-decision count 13 → 16; Audit-C NEW claim 15 verifies workspace `cargo test --no-fail-fast` GREEN).

- **Recommended gate**: gate-1.5 approval → resume Phase 2f (P1.5-READ-BRIDGE).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/agent.md` | §"Soul (Immutable Born Structure)" lines 160–169 | *"The Soul is the agent's **genetics** — defined at creation, never mutated. Immutability: The Soul node is write-once. If you need to change an agent's fundamental nature, you create a new agent."* + table at L164–168 mapping Profile snapshot → `AgentProfile (frozen)` | concept-aspirational (pre-CH-28: code enforces 1:1 schema UNIQUE; concept frames as "genetics, never mutated"; but user has chosen to amend the concept-doc to "template-Blueprint sharing") | **honored** — concept-doc body amended at P-DOCS to reframe Soul as "template-Blueprint, shareable across agent instances + per-agent override-Blueprint for governance fields"; immutability semantic preserved (the template Blueprint is still write-once; sharing is the new semantic; per-agent override-Blueprints are per-agent mutable) |
| `concepts/agent.md` | §"Parallelized Sessions" lines 209–221 | *"An AgentProfile carries a `parallelize: u32` field... Why parallelize per profile, not per agent instance: the profile defines what the agent *is*; `parallelize` is about the agent's concurrency capacity under that profile. An org with 5 agents sharing a profile has 5 × `parallelize` total concurrent sessions possible from that profile family."* | partially-honored | **honored** — concept body amended at P-DOCS: `parallelize` lives on the per-agent override-Blueprint row (NOT on the shared template Blueprint); the "5 × parallelize" arithmetic continues to hold because each agent has its own override-Blueprint with its own `parallelize` value, defaulting to the template's value when unset |
| `concepts/ontology.md` | §"Edge Types" → §"Agent-Centric (first-order)" line 98 | *"Agent | `HAS_PROFILE` | AgentProfile | 1:1 | Blueprint identity"* | contradicted | **honored** — line 98 amended to `Agent | USES_PROFILE | AgentProfile | N:1 | Blueprint identity (shared template)`; NEW rows added below it for `AgentProfile | AGENT_PROFILE_USES_BLUEPRINT | Blueprint | N:1 | Template Blueprint pointer` + `Agent | AGENT_USES_BLUEPRINT_OVERRIDE | Blueprint | 1:N (zero-or-one) | Per-agent override Blueprint pointer` |
| `concepts/phi-core-mapping.md` | §"AgentProfile" table row line 82 | *"`AgentProfile` (struct) | **Node** | **AgentProfile** (wrapped at `domain::AgentProfile.blueprint`)"* | honored | **honored** — no change to the wrap. The hybrid-Blueprint cardinality lives at the baby-phi-domain layer, NOT in phi-core. The wrap's `blueprint` field on the baby-phi `AgentProfile` struct continues to be the single source of truth for `system_prompt`, `thinking_level`, etc.; the per-agent override fields persistence-of-record moves to the NEW `Blueprint` row (iter-4 §D63.13 clarification); the in-process AgentProfile struct still carries the 3 fields as a synthesised projection |
| `concepts/permissions/README.md` | (entry invariants) | (N/A — this chunk does not touch the permissions subtree; no Permission Check engine surface affected) | n/a | n/a — chunk skips the permissions subtree |

**Coverage check**: every concept doc whose claims the chunk's code will touch is in the table.

**phi-core-mapping hook**: cited above. The chunk does NOT consume any new phi-core types; the AgentProfile wrap stays intact.

**Permissions subtree hook**: N/A — chunk skips `permissions/01`–`permissions/09` docs.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::agents::profile::AgentProfile` | wrapped at `domain::AgentProfile.blueprint` (per `nodes.rs:322`) | wrap | no change — wrap embedding stays identical; CH-28 changes the cardinality at the baby-phi-domain layer, NOT phi-core itself |
| `phi_core::context::execution::ExecutionLimits` | wrapped at `AgentCreationPayload.initial_execution_limits_override` (per `repository.rs:472-474`) | wrap | no change — orthogonal to the cardinality fork |
| (no other phi-core types touched) | n/a | n/a | n/a |

**Expected import-count delta at chunk close**: **+0 leverage-sites** (v20 P4 leverage-site methodology; tolerance ±3). The chunk does not introduce, consume, or remove any phi-core import. Baseline phi-core import count: **57 lines** at `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` (verified 2026-05-19 at HEAD). Predicted at chunk-close: **57** (zero new imports; zero removed imports).

**v22 P2 proc-macro decorator prediction**: not applicable here. No phi-core trait being newly implemented in baby-phi at CH-28; no proc-macro decorators implied; no new dev-deps required.

**Positive close-audit greps** (the exact commands the post-chunk audit will run):

```bash
# (1) AgentProfile wrap continues to embed phi_core::AgentProfile
grep -n "phi_core::agents::profile::AgentProfile" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs
# expect: ≥ 1 hit at the blueprint field

# (2) AgentProfile struct-literal construction sites continue to set `blueprint:` field
grep -rn "blueprint: phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: SAME count as baseline (the 39 AgentProfile struct-literal sites preserved per iter-4 §D63.13)
# NOTE (iter-5): the AgentProfileWireRow intermediate struct introduced at P1.5 (per §D63.14) sits inside store/src/repo_impl.rs and is NOT a public AgentProfile alternative — it's a write-boundary serialization shim. The grep above continues to count the AgentProfile struct-literal sites unchanged.
```

**Forbidden-duplication greps** (must return 0 hits):

```bash
# (1) No NEW phi-core type duplicated under baby-phi
grep -rnE "^pub struct AgentProfile\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::"
# expect: exactly 1 hit (the wrap at domain/src/model/nodes.rs:314); not 2 or more
# NOTE (iter-5): AgentProfileWireRow is `pub(crate) struct` (NOT pub) — does not show under this grep.

# (2) check-phi-core-reuse.sh green
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
# expect: exit 0
```

### §3 cascade-artifact discipline (chunk-planner v4/v8 cascade-grep + v22 P1 per-Tier breakdown; iter-5 collapse-update applied)

The iter-2/iter-3/iter-4 cascade was the **sum of THREE independent cascades**. **Iter-5 update**: under §D63.13 in-process struct preservation (carried forward from iter-4) AND the P1.5-READ-BRIDGE absorbing the write-boundary strip + read-path synthesis, the iter-4 P2 cascade FURTHER COLLAPSES — the 9 PRODUCTION write-path sites stay unchanged (composite-write happens inside `upsert_agent_profile`'s body at P1.5; per-handler code is agnostic). Cascade 3 (NEW Blueprint table sites) shipped at P1 close. Cascade 1 (struct-literal sites) + Cascade 2 (HAS_PROFILE → USES_PROFILE renames) land at P-EDGE-RENAME (cascade 2 only; cascade 1 fully dissolves per §D63.13).

**Cascade 1 — AgentProfile struct-literal sites** (F1-driven; iter-5 final form):

Raw count at plan-draft (re-verified 2026-05-19): **39 sites** (9 PRODUCTION + 30 TEST-FIXTURE per iter-1 per-file breakdown). **iter-5 final**: under §D63.13 + P1.5 composite-write absorption, **0 sites** are mechanically rewritten. All 39 sites stay as direct struct-literal construction; the `upsert_agent_profile` body at P1.5 echoes the override values into the Blueprint table transparently.

**Cascade 2 — HAS_PROFILE / HasProfile / has_profile rename sites** (NEW iter-2 cascade, F2.b-driven):

```bash
git -C /root/projects/phi/baby-phi grep -nE 'HasProfile|HAS_PROFILE|has_profile' modules/crates/
```

Raw count at plan-draft (verified 2026-05-19): **44 lines** — **43 lines after excluding `audit/events/m4/agents.rs:78`** (audit JSON wire-key STAYS per ADR-0033 §D33.5). Per-file breakdown unchanged from iter-2/iter-4:

| File | Count | Site type |
|---|---|---|
| `modules/crates/domain/src/audit/events/m4/agents.rs` | 1 | EXCLUDED (audit JSON wire-key) |
| `modules/crates/domain/src/events/listeners.rs` | 10 | 5 PRODUCTION + 5 TEST-FIXTURE |
| `modules/crates/domain/src/events/mod.rs` | 9 | 7 PRODUCTION + 2 TEST-FIXTURE |
| `modules/crates/domain/src/model/composites_m5.rs` | 2 | PRODUCTION doc-comments |
| `modules/crates/domain/src/model/edges.rs` | 3 | PRODUCTION |
| `modules/crates/domain/src/repository.rs` | 3 | PRODUCTION doc-comments |
| `modules/crates/server/src/platform/agents/update.rs` | 2 | PRODUCTION |
| `modules/crates/store/migrations/0001_initial.surql` | 1 | EXCLUDED (immutable) |
| `modules/crates/store/migrations/0005_sessions_templates_system_agents.surql` | 1 | EXCLUDED (immutable) |
| `modules/crates/store/src/repo_impl.rs` | 4 | PRODUCTION RELATE statements |
| `modules/crates/store/tests/apply_org_creation_tx_test.rs` | 1 | TEST-FIXTURE |
| **Total relevant** | **35** | **24 PRODUCTION + 7 TEST-FIXTURE + 4 EXCLUDED** |

**Cascade 3 — NEW Blueprint table + RELATION edges sites** (F1.c-driven; **P1 DONE**):

P1-BLUEPRINT-STRUCT shipped these sites at close (2026-05-19):
- `domain/src/model/nodes.rs`: NEW `Blueprint` struct (lines 402-424) + `BlueprintId` newtype (lines 349-375). AgentProfile struct fields PRESERVED.
- `domain/src/model/edges.rs`: NEW `Edge::AgentProfileUsesBlueprint` + `Edge::AgentUsesBlueprintOverride` variants + 2 NEW `name()` arms + 2 NEW `EDGE_KIND_NAMES` entries. `EDGE_KIND_NAMES: [&str; 74]` (was 72). Test `edge_kind_names_cardinality_is_74_pinned_at_compile_time` renamed + green.
- `domain/src/repository.rs`: 4 NEW trait methods at lines 632-680.
- `domain/src/in_memory.rs`: 4 method impls at lines 388-465.
- `store/src/repo_impl.rs`: 4 method impls at lines 700-831 + half-migrated helper at lines 491-528.
- `domain/src/events/mod.rs`: NEW `DomainEvent::BlueprintUpserted` variant + emission body + kind() arm + serde roundtrip test.
- `domain/src/events/listeners.rs`: NEW override-tier listener match-arm at lines 1089-1106 + 1191-1196. **SCOPE-NARROWING**: template-tier fan-out NOT shipped (requires `list_agents_using_blueprint_template` Repository method; deferred per §D63.16 → `D-CH28-FOLLOWUP-01`).

**P1 close actuals (P-FIXTURES snapshot)**: 11/11 new tests green; clippy + 4 CI guards green; ~440 LOC across 9 files; EDGE_KIND_NAMES.len() == 74; BlueprintUpserted event roundtrip test green. **Workspace `cargo test --no-fail-fast`**: **RED** (9 failing test targets — root cause: 0019 REMOVE FIELD vs `create_agent_profile`/`upsert_agent_profile` writing the full AgentProfile JSON; bridge code MUST land before P-EDGE-RENAME). **Closure path**: P1.5-READ-BRIDGE.

**Iter-5 P1.5-READ-BRIDGE cascade (NEW)**:
- 1 NEW struct `AgentProfileWireRow` at `store/src/repo_impl.rs` (~20 LOC + custom `From<&AgentProfile>` impl).
- 2 PRODUCTION body rewrites at `store/src/repo_impl.rs`: `create_agent_profile` (lines 635-646) + `upsert_agent_profile` (lines 675-696) — write-boundary strip + composite-write to Blueprint via `upsert_agent_blueprint_override`.
- 2 PRODUCTION body rewrites at `store/src/repo_impl.rs` + `domain/src/in_memory.rs`: `get_agent_profile_for_agent` — call `read_agent_profile_via_blueprint_or_fallback` (already shipped at P1).
- 1 in-memory mirror of the synthesis logic at `domain/src/in_memory.rs:351-361`.
- 3 NEW Tier-A/B tests + the deferred `half_migrated_state_runtime_reads_via_fallback_helper`.

**Iter-5 P2-ACCEPTANCE cascade (collapsed from iter-4 P2-HANDLERS)**:
- 7 NEW acceptance tests in `acceptance_m6_agent_profile_cardinality.rs`.
- OPTIONAL factory helper at `acceptance_common/blueprint_fixture.rs` (planner-recommendation: ship it; needed for the 7 acceptance tests' explicit override-write exercises).
- **0 PRODUCTION code changes** (all production-tier work absorbed by P1 + P1.5).

**Combined cascade-band (v21 R1; iter-5 final form)**:

| Cascade | Lower-bound | Upper-bound | Pause-trip (1.5×) |
|---|---|---|---|
| Cascade 1 — AgentProfile literal sites (DISSOLVED per §D63.13 + §D63.14/§D63.15) | 0 | 0 | n/a |
| Cascade 2 — HAS_PROFILE → USES_PROFILE rename (F2.b; lands at P-EDGE-RENAME) | 24 PRODUCTION | 31 (24 PRODUCTION + 7 TEST-FIXTURE) | 47 |
| Cascade 3 — NEW Blueprint table + edges (P1 DONE) | 9 files, ~440 LOC | 9 files | LOC bounded (660 = 1.5× 440) — closed actual ~440 |
| Cascade 4 (NEW iter-5) — Wire-row + bridge wiring at P1.5 | 5 PRODUCTION sites (1 NEW struct + 2 write rewrites + 2 read rewrites) | 7 (incl. in-memory mirror + the deferred test) | 11 (1.5× 7) |
| **Combined edit-site band (iter-5)** | **33** | **38** | **57** |

**Pause-discipline (iter-5 re-derived)**: PAUSE via AskUserQuestion if **actual cascade > 57 edit sites** (1.5× upper-bound 38). Threshold tightens from iter-4's 105 because the dissolved Cascade 1 + P1-DONE Cascade 3 are removed from the live band.

**iter-5 P1.5-specific pause-discipline**: at P1.5-READ-BRIDGE close, workspace `cargo test --workspace --no-fail-fast` MUST be GREEN. If RED, PAUSE immediately via AskUserQuestion; do NOT proceed to P-EDGE-RENAME. This is the LOAD-BEARING acceptance criterion for P1.5.

**Caveat (CH-07 retro)**: per-file edit-count predictions are approximate; the aggregate band [33, 38] is the load-bearing prediction; pause threshold (1.5× upper bound = 57) is enforced on the aggregate, not per-file.

### Per-fork pause-threshold table (iter-5 preserved from iter-2/iter-4)

| Fork | Locked outcome | Δ file-count cap | Δ key-file LOC cap | Δ Cargo.lock cap |
|---|---|---|---|---|
| F1.c (LOCKED) | NEW `blueprint` table + struct + 4 methods + 2 Edge variants + DomainEvent + override-tier listener (DONE at P1); read-path + write-path bridge at P1.5 | +0 files at P1 + 0 NEW files at P1.5 (wire-row lives inside `repo_impl.rs`); +1 helper file at P2-ACCEPTANCE (OPTIONAL) | `nodes.rs` ≤ 1500 (current ~1480 post-P1 close); `edges.rs` ≤ 950; `repo_impl.rs` ≤ 3200 (current ~3000 post-P1 close + ~80 LOC for wire-row + bridges at P1.5 → ~3080); `in_memory.rs` ≤ 2500 | +0 |
| F2.b (LOCKED) | Edge rename + serde alias + DomainEvent rename — lands at P-EDGE-RENAME | +0 files | `edges.rs` unchanged for F2.b alone; `events/mod.rs` unchanged | +0 |
| F3.b (LOCKED) | 2 NEW migrations 0019 + 0020 + half-migrated runtime helper (already shipped at iter-3 P-MIGRATION-* + P1) | +0 NEW files at P1.5 | each migration ≤ 200 LOC | +0 |

### §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | No new in-process state. iter-5 P1.5 adds `AgentProfileWireRow` (stack-only intermediate struct; no caching). | **no** | n/a |
| **A2** | New IPC channel | No new IPC channels | **no** | n/a |
| **A3** | New pod-local resource | No new pod-local resources | **no** | n/a |
| **A4** | Migration runner / first-apply race | 2 NEW migrations (0019 + 0020) — already shipped + first-apply green per iter-3. Ride existing CHK8S-D-05. | **no** | n/a |
| **A5** | Trait-shape requirement | Repository trait gained 4 NEW methods at P1. iter-5 P1.5 modifies existing `get_agent_profile_for_agent` + `create_agent_profile` + `upsert_agent_profile` BODIES (NOT signatures); trait-shape unchanged. | **no** | n/a |
| **A6** | Cross-pod state sharing | SurrealDB-durable storage; no in-process cache. iter-5 P1.5 composite-write is a single transactional unit — atomicity preserved across pods (BEGIN/COMMIT). | **no** | n/a |
| **A7** | Audit hash-chain symmetry | F2.b renames `DomainEvent::HasProfileEdgeChanged` → `UsesProfileEdgeChanged`; hash-chain canonical-bytes preserved per ADR-0033 §D33.5. iter-5 introduces `DomainEvent::BlueprintUpserted` at P1 (also covered by D33.5). | **no** | n/a |

**Conforming-criteria check against ADR-0033**: unchanged from iter-4.

**Conclusion**: **K8s-neutral**. No new K8s blocker class. Both migrations 0019 + 0020 ride existing CHK8S-D-05.

### §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m6/architecture/agent-profile-cardinality.md` (NEW; shipped at P-DOCS iter-3) | YES (DONE) | shipped |
| **Architecture** | `docs/specs/v0/implementation/m1/architecture/graph-model.md` | YES (DONE at iter-3 P-DOCS) | shipped |
| **Architecture** | `docs/specs/v0/implementation/m1/architecture/schema-migrations.md` | YES (DONE at iter-3 P-DOCS) | shipped |
| **Operations** | `docs/specs/v0/implementation/m6/operations/agent-profile-operations.md` | NO | **(b) defer** — successor CH-37 |
| **User-guide** | `docs/specs/v0/implementation/m6/user-guide/agent-profile-walkthrough.md` | NO | **(b) defer** — successor CH-37 |

### §3.D — Forward-scope-vs-concept-doc precedence

**iter-5 update**: unchanged from iter-2/iter-4. F2.b lock makes "uses_profile" wording binding; F1.c adds 2 NEW edge variants; ADR-0057 §D57.7 invariant text updates to `EDGE_KIND_NAMES.len() == 74 post-CH-28`.

### §3.E — Anticipated gate-2.5 candidates

Per chunk-planner v13 §3.E:

- **Candidate 1 — AgentCreationPayload adoption-flow** (iter-2 forward-looking): defer to CH-37 a05.
- **Candidate 2 — UsesProfileEdgeChanged multi-agent enrichment**: defer to M7 NFR-observability (drift D-CH28-FOLLOWUP-02 with `M7-DEFERRED-AUDIT-MULTI-AGENT-LIST` allocation if surfaced).
- **Candidate 3 — Half-migrated-state runtime tolerance test**: NOW LANDING at P1.5-READ-BRIDGE per re-spawn mandate point 1.d (the deferred `half_migrated_state_runtime_reads_via_fallback_helper` test).
- **Candidate 4 (iter-4)** — AgentProfile in-process struct projection cascade: preserved at iter-5; if P1.5 implementation surfaces a wire-format roundtrip divergence between the AgentProfile struct field values and the override-Blueprint row column values, route to (a) close-in-chunk refinement of the synthesis body.
- **Candidate 5 (NEW iter-5) — Composite-write atomicity defect at upsert_agent_profile**: if the implementer discovers that wrapping `upsert_agent_profile` body's two writes (`agent_profile` row + `agent_uses_blueprint_override` edge + `blueprint` row) in a single SurrealDB `BEGIN; ... COMMIT;` transaction surfaces a parser limitation or rollback semantic edge-case in SurrealDB 2.6.5, route to:
  - (a) close-in-chunk: split into 2 separate transactions with explicit error-cleanup OR rely on the existing `upsert_agent_blueprint_override` body's own internal transaction (it already wraps DELETE + UPDATE + DELETE-edge + RELATE in a BEGIN/COMMIT).
  - (b) defer to follow-up drift `D-CH28-FOLLOWUP-03` + ship best-effort composite-write at P1.5 with a clarifying note that full atomicity awaits M7.
  - **Planner-recommendation: (a) close-in-chunk** — the existing `upsert_agent_blueprint_override` already does a transactional inner write; the composite at `upsert_agent_profile` can call it sequentially after the agent_profile row write succeeds. If the second write fails after the first succeeds, the inconsistency is documentable as a §D63.15 caveat (M7 NFR-observability cleanup).
- **Candidate 6 (NEW iter-5) — Listener template-tier fan-out at BlueprintUpserted**: shipped as DEFERRED at P1 per implementer SCOPE-NARROWING; iter-5 codifies the deferral via §D63.16 + filing `D-CH28-FOLLOWUP-01` with `M6-DEFERRED-04` allocation. If P-EDGE-RENAME or P2-ACCEPTANCE authoring surfaces a load-bearing template-tier fan-out scenario (e.g., a test requiring template-Blueprint mutation visibility across siblings), route to:
  - (a) close-in-chunk: add `list_agents_using_blueprint_template(BlueprintId)` Repository method + extend the BlueprintUpserted listener arm.
  - (b) defer per §D63.16 (current planner choice).
  - **Planner-recommendation: (b) defer** — template-tier mutations are migration-time-rare; CH-36 a04 supervisor will need this anyway.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `M6+-OPEN-01` | `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370 | HIGH | open-question → **resolved-by-redesign** | At P-SEAL, the marker is amended with a `Resolution` line citing CH-28 / ADR-0063 (cycle hex `0412eb06`). |

**NEW iter-5 follow-up drifts filed at P-SEAL** (per chunk-implementer SCOPE-NARROWING from P1):

| Drift ID | File | Severity | Status | Notes |
|---|---|---|---|---|
| `D-CH28-FOLLOWUP-01` | `docs/specs/v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md` (NEW; filed at P-SEAL) | LOW | scoped → `M6-DEFERRED-04` | Listener template-tier fan-out NOT shipped at P1 (requires NEW Repository method `list_agents_using_blueprint_template(BlueprintId)`). Per §D63.16, route to CH-36 a04 supervisor body which inherits the same fan-out requirement. `Impl chunk`: `M6-DEFERRED-04` (CH-36). Override-tier listener IS shipped at P1 (lines 1089-1106 + 1191-1196). |

**v13 Row 4 drift-allocation rule check**: M6+-OPEN-01 transitions to terminal; D-CH28-FOLLOWUP-01 is filed NEW with explicit `M6-DEFERRED-04` allocation (NOT `TBD`). No `TBD` markers introduced.

---

## §5 — ADRs drafted

**ADR-0063** at `docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-n-to-1.md`.

**Iter-5 preservation note**: §5 preserved verbatim from iter-4 with THREE additions — NEW sub-decisions §D63.14 + §D63.15 + §D63.16 (sub-decision count 13 → 16). §D63.1 + §D63.11 + §D63.13 bodies unchanged from iter-4.

**P7 ADR-location lookup discipline (v22 P7)**: ADR-0063 lands under `m6/decisions/`. Path verified at iter-3 P0.

**Decision-summary** (one line, iter-5 update): *"AgentProfile cardinality flips 1:1 → N:1 via hybrid `Blueprint` table (F1.c); HAS_PROFILE → USES_PROFILE rename (F2.b); split migrations 0019 + 0020 (F3.b); EDGE_KIND_NAMES.len() bumps 72 → 74; AgentProfile in-process struct field-set preserved as synthesized projection (§D63.13); write-boundary strip via `AgentProfileWireRow` (§D63.14); composite-write at `upsert_agent_profile` + read-path synthesis at `get_agent_profile_for_agent` wired at P1.5-READ-BRIDGE (§D63.15); listener template-tier fan-out deferred to CH-36 / `M6-DEFERRED-04` per §D63.16."*

**Status at plan draft**: `Proposed`. **Flip to `Accepted`**: at P-SEAL.

**Drafted-at-phase**: P0 (scaffold) → drafted as Proposed; flipped Accepted at P-SEAL.

**ADR top-level section enumeration (v17 mandatory; preserved at iter-5)** — the implementer authors ALL SEVEN sections:

1. **`## Forks`** — header table; F1 + F2 + F3 all DIVERGENT.
2. **`## Context`** — chunk-graph + forward-scope citations.
3. **`## Sub-decisions`** — **SIXTEEN** `### §D63.<M>` sub-decisions (iter-5 expanded from iter-4's 13):
   - §D63.1 — F1.c hybrid Blueprint table (iter-4 annotation cross-ref to §D63.13 preserved).
   - §D63.2 — F2.b edge rename.
   - §D63.3 — F3.b split migrations.
   - §D63.4 — Concept-doc amendment scope.
   - §D63.5 — Schema migration body + idempotency.
   - §D63.6 — Repository trait method additions (4 NEW methods; shipped at P1).
   - §D63.7 — Audit-event semantics rename + hash-chain preservation.
   - §D63.8 — Wrap-vs-runtime separation preserved.
   - §D63.9 — Cross-chunk consequences.
   - §D63.10 — EDGE_KIND_NAMES cardinality bump 72 → 74.
   - §D63.11 — Test-fixture cascade migration via helper (iter-4 annotation preserved).
   - §D63.12 — Half-migrated-state runtime tolerance.
   - §D63.13 — AgentProfile in-process struct field-set preservation (iter-4; preserved).
   - **§D63.14 (NEW iter-5) — Write-boundary strip via `AgentProfileWireRow` intermediate struct**. The `create_agent_profile` + `upsert_agent_profile` SurrealDB-tier bodies serialize through a `pub(crate) struct AgentProfileWireRow` at `store/src/repo_impl.rs` that mirrors `domain::AgentProfile`'s field-set MINUS the 3 override fields (`parallelize`, `model_config_id`, `mock_response`). The wire-row is constructed via `From<&AgentProfile>` impl that picks the SurrealDB-defined fields. **Rationale for approach (b) over (a) custom serde with `#[serde(skip_serializing_if)]` + (c) JSON `.remove()`**: (b) gives compile-time guarantees that the right set of fields lands at SurrealDB (any future AgentProfile field-addition that should NOT persist to `agent_profile` table is caught at compile time via a missing field on the wire-row mapping); (a) requires per-field annotations + a runtime flag; (c) is string-typed + error-prone. The wire-row pattern is forward-extensible — if M7 adds more fields to the projection, the wire-row mapping is the single source of truth.
   - **§D63.15 (NEW iter-5) — Read-path synthesis wiring at `get_agent_profile_for_agent` + composite-write transaction semantics at `upsert_agent_profile`**. At P1.5-READ-BRIDGE the SurrealDB impl of `get_agent_profile_for_agent` is rewritten to: (1) call `read_agent_profile_via_blueprint_or_fallback` (helper shipped at P1 / `repo_impl.rs:491-528`); (2) return the synthesized AgentProfile with 3 override fields populated from the override-Blueprint (or template defaults if no override exists, or legacy `agent_profile` un-asserted-property values if half-migrated). The in-memory impl mirrors the same logic at `in_memory.rs:351-361`. At P1.5 the SurrealDB impl of `upsert_agent_profile` is rewritten to: (a) STRIP the 3 override fields via `AgentProfileWireRow` (§D63.14); (b) write the `agent_profile` row via `UPDATE type::thing(...) CONTENT $wire_body`; (c) call `upsert_agent_blueprint_override(&Blueprint { agent_id: Some(...), parallelize: Some(profile.parallelize), model_config_id: profile.model_config_id.clone(), mock_response: profile.mock_response.clone(), ... })` to persist the override row. The two writes are sequential (NOT in a single outer transaction wrapper) per Candidate 5 (a) — `upsert_agent_blueprint_override` already wraps its own internal SurrealDB `BEGIN; ... COMMIT;`. **Failure-mode**: if the second write fails after the first succeeds, the `agent_profile` row exists WITHOUT the override-Blueprint — `get_agent_profile_for_agent` will synthesize using `read_agent_profile_via_blueprint_or_fallback` which falls back to either template-Blueprint defaults OR (during half-migrated window) the un-asserted legacy properties. Net effect: at-rest semantic is "missing override row → inherit template" which is the same semantic as freshly-created agents. **The composite-write is best-effort atomic; full-atomic outer transaction is documentable as a §D63.15 caveat for M7 NFR-observability cleanup**.
   - **§D63.16 (NEW iter-5) — Listener template-tier fan-out scope-narrowing**. The BlueprintUpserted listener arm at `events/listeners.rs:1089-1106 + 1191-1196` covers the OVERRIDE-tier (per-agent override Blueprint upserts trigger profile snapshot refresh for the SINGLE owning agent). The TEMPLATE-tier fan-out (template Blueprint upserts trigger profile snapshot refresh for ALL agents whose AgentProfile points at the upserted template) is NOT shipped. Implementing template-tier fan-out requires a NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` that traverses `Blueprint → reverse AGENT_PROFILE_USES_BLUEPRINT → AgentProfile → reverse USES_PROFILE → Agent`. The narrowing is documented at `listeners.rs:1094-1124` per the implementer's P1 close report. Deferred to `D-CH28-FOLLOWUP-01` (filed at P-SEAL) with `M6-DEFERRED-04` allocation — CH-36 a04 supervisor body inherits the same template-tier fan-out requirement (supervisors share tuning templates across pools of supervised agents).
4. **`## Cross-references`** — 4 categories (preserved verbatim from iter-4; iter-5 adds `D-CH28-FOLLOWUP-01` to category (b) `closed drifts` register — actually as NEW-filed drift, NOT closed; tracked in §4 above).
5. **`## Consequences`** — `### For CH-36 / CH-37 / CH-29 / M7-NFR-observability / future M7 cleanup` subsections preserved. **iter-5 ADD**: `### For CH-36` body amends to call out `list_agents_using_blueprint_template` as a NEW requirement inherited via `D-CH28-FOLLOWUP-01` (CH-36 implements the method + uses it for supervisor-tier fan-out per §D63.16).
6. **`## Revisit triggers`** — 9 bullets (iter-4's 8 + 1 new iter-5):
   - (preserved 8 bullets from iter-4)
   - **NEW iter-5**: If the §D63.15 composite-write best-effort atomicity surfaces a production-tier inconsistency (e.g., `agent_profile` row exists without override-Blueprint after a crash mid-`upsert_agent_profile`), §D63.15 reopens — the M7 NFR-observability cleanup tightens the composite-write to a single outer SurrealDB `BEGIN; ... COMMIT;`.
7. **`## Verification`** — commands from §12.

**ADR-body checklist (v11 strict-or-variation):**
- All 13 iter-4 preservation notes preserved.
- **§D63.14 strict (NEW iter-5)**: *"Pre-existing behaviour preserved: AgentProfile struct field-set unchanged (per §D63.13); the wire-row pattern is a NEW serialization shim shipped at CH-28 at the SurrealDB write boundary only; it does NOT affect the AgentProfile public API; in-memory impl is unaffected (in-memory uses direct struct insertion into its `agent_profiles` HashMap)."*
- **§D63.15 strict (NEW iter-5)**: *"Pre-existing behaviour preserved: `get_agent_profile_for_agent` trait contract unchanged at the public surface (returns `Option<AgentProfile>` with the same field-set per §D63.13); the body changes are at the SurrealDB impl tier only; in-memory impl receives mirrored synthesis logic for tier-symmetry. `upsert_agent_profile` trait contract unchanged (takes `&AgentProfile`); the body now invokes `upsert_agent_blueprint_override` internally to persist the override-Blueprint row."*
- **§D63.16 variation (a) (NEW iter-5)**: *"Pre-existing scaffold preserved: BlueprintUpserted listener arm at `events/listeners.rs:1089-1106` ships the override-tier coverage at CH-28; template-tier fan-out coverage routed to `D-CH28-FOLLOWUP-01` with `M6-DEFERRED-04` allocation; CH-36 a04 supervisor body inherits the requirement."*

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| CH-01 (M5.2 agent-durable-lifecycle) | `Agent` struct shape carries `active` + `archived_at` | `grep -n "active: bool\|archived_at: Option<DateTime" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` — expect ≥ 2 hits |
| CH-02 (M5.2 MockProvider) | `AgentProfile.mock_response: Option<String>` PRESERVED per §D63.13 | `grep -n "pub mock_response: Option<String>" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` — at chunk-close: expect 2 hits (1 on AgentProfile + 1 on Blueprint) |
| CH-08 (cardinality) | `EDGE_KIND_NAMES.len() == 74` (bumped per F1.c) | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain edge_kind_names_cardinality_is_74_pinned_at_compile_time` — GREEN at P1 close |
| CH-16 (Identity node) | `apply_agent_creation` compound tx atomicity preserved | `grep -n "fn apply_agent_creation" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs` — expect 2 hits |
| CH-19 (Bucket B) | ADR-0057 §D57.7 EDGE_KIND_NAMES amendment 72 → 74 | `grep -n "EDGE_KIND_NAMES.len() == 74" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs` — expect ≥ 1 hit |
| CH-25 (Owns) | `Edge::Owns` variant + emission | `grep -n "Edge::Owns" /root/projects/phi/baby-phi/modules/crates/ -r | wc -l` — expect ≥ 3 hits |
| CH-27 (blocking-gate) | `check_permission` invocations at 7 admin handlers | `grep -rn "check_permission" /root/projects/phi/baby-phi/modules/crates/server/src/platform/orgs/ /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/ | wc -l` — expect ≥ 7 hits |
| Schema migration runner | All 0001..0020 first-apply green | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test` — expect green |
| **iter-5 NEW** — P1 BLUEPRINT additive surface | Blueprint struct + 4 trait methods + BlueprintUpserted + 11/11 inline tests green | `grep -n "pub struct Blueprint\b\|fn get_blueprint_for_agent_profile\|fn upsert_blueprint_template\|fn upsert_agent_blueprint_override" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs` — expect ≥ 4 hits |

**Carry-forward invariants** (verified at chunk open; iter-5 update):
- `cargo build --workspace --all-targets -j 4` GREEN.
- `cargo test -p store --test migrations_test` 5/5 green.
- `cargo test --workspace --no-fail-fast` **RED** at P1 close (9 failing test targets — root cause documented in §1 iter-5 deviation note; CLOSED at P1.5).
- `bash scripts/check-phi-core-reuse.sh` green.
- `bash scripts/check-doc-links.sh` green.
- `bash scripts/check-ops-doc-headers.sh` green.
- `bash scripts/check-spec-drift.sh` green.

**v19 P2 back-compat decision-prompt (iter-5 update)**: under iter-4 §D63.13 + iter-5 §D63.14/§D63.15, the in-process AgentProfile struct field-set is preserved as synthesized projection AND the SurrealDB write boundary strips via `AgentProfileWireRow`. The 30 TEST-FIXTURE struct-literal sites do NOT require mechanical rewrite. Resolution:
- (a) NEW factory helper `make_test_profile_with_overrides` OFFERED at `acceptance_common/blueprint_fixture.rs` — used by the 7 NEW acceptance tests.
- (b) preserve the fields on `AgentProfile` with `#[serde(default)]` markers — **chosen** (§D63.13).
- (c) defer carry-forward test migrations — NOT applicable.

---

## §7 — Phases within the chunk

The chunk has **7 substantive phases** at iter-5 (was 7 at iter-2/iter-4; iter-5 keeps count at 7 because P1.5 insertion is paired with P2-HANDLERS → P2-ACCEPTANCE scope-collapse). Ordering:

`P0 → P-DOCS → P-MIGRATION-SCHEMA → P-MIGRATION-BACKFILL → P1-BLUEPRINT-STRUCT (DONE) → P1.5-READ-BRIDGE (NEW iter-5) → P-EDGE-RENAME → P2-ACCEPTANCE (re-scoped from P2-HANDLERS) → P-SEAL`

Phase headers: 9 (P0 + 7 substantive + P-SEAL). **Audit envelope = LARGE** (7 substantive phases ≥ 6 per per-chunk-template §11).

### §7.0 — Phase ordering rationale (iter-5 re-cast)

**Why P1.5-READ-BRIDGE is INSERTED** between P1-BLUEPRINT-STRUCT (DONE) and P-EDGE-RENAME.

**What was empirically falsified at iter-5**: the iter-4 §7.0 rationale claimed "P1-BLUEPRINT-STRUCT ADDITIVE-only ⇒ workspace stays GREEN by construction". This claim was empirically INCORRECT — workspace `cargo test --no-fail-fast` is RED at P1 close (9 failing test targets). The flaw in the iter-4 rationale: "additive-only" applied to the Rust struct layer (AgentProfile struct field-set preserved per §D63.13) but did NOT account for the SurrealDB layer where migration 0019 had ALREADY shipped `REMOVE FIELD` on `agent_profile.{parallelize, model_config_id, mock_response}`. The schema-vs-struct mismatch is INHERENT between the migration apply (already done at P-MIGRATION-SCHEMA) AND any bridge code (NOT yet written). No phase between those two boundaries can be RED-free without inserting the bridge code.

**Why P1.5-READ-BRIDGE is THE correct phase** (NOT routing the bridge into P-EDGE-RENAME or P2-HANDLERS):
- The bridge is small, tightly scoped, and verifiably closes the RED window in a single phase. Routing it into P-EDGE-RENAME would conflate the F2.b mechanical rename (orthogonal concern) with the load-bearing read/write synthesis; routing it into P2-HANDLERS would push the RED window through P-EDGE-RENAME, leaving the workspace RED for a longer interval.
- P1.5 ships BEFORE P-EDGE-RENAME so that P-EDGE-RENAME starts from a GREEN baseline and its sole responsibility is the mechanical HAS_PROFILE → USES_PROFILE rename. The serde alias decorator + 43-site rename can land cleanly without entanglement.
- P1.5 ALSO lands the deferred `half_migrated_state_runtime_reads_via_fallback_helper` test, closing P-MIGRATION-BACKFILL's deferred test obligation.

**Net effect at iter-5**:
- P1-BLUEPRINT-STRUCT close (DONE 2026-05-19): ADDITIVE surface in place; workspace `cargo test --no-fail-fast` **RED** (9 failing targets; documented root cause).
- P1.5-READ-BRIDGE close: workspace `cargo test --no-fail-fast` **GREEN**. **LOAD-BEARING ACCEPTANCE CRITERION**.
- P-EDGE-RENAME close: mechanical rename complete; workspace GREEN.
- P2-ACCEPTANCE close: 7 NEW acceptance tests verify the cardinality flip end-to-end + OPTIONAL factory helper landed; workspace GREEN.

This sequence minimises the "red workspace" window to **EXACTLY ONE PHASE** (P1 close → P1.5 close), and the P1.5 phase is small (~80 LOC + 3-4 NEW tests + 2-3 body rewrites).

### P0 — Scaffolding + ADR-0063 Proposed draft (DONE iter-3)

**Status at iter-5**: shipped at iter-3 P0 close. iter-5 amendment: ADR-0063 sub-decision count updated 13 → 16 at P-SEAL (add §D63.14, §D63.15, §D63.16).

### P-DOCS — Concept-doc amendments (DONE iter-3)

**Status at iter-5**: shipped at iter-3 P-DOCS close (7 doc files modified).

### P-MIGRATION-SCHEMA — Migration 0019 (DONE iter-3)

**Status at iter-5**: shipped at iter-3 P-MIGRATION-SCHEMA close. 5/5 migration tests green.

### P-MIGRATION-BACKFILL — Migration 0020 (DONE iter-3)

**Status at iter-5**: shipped at iter-3 P-MIGRATION-BACKFILL close. The `half_migrated_state_runtime_reads_via_fallback_helper` test was deferred from this phase → NOW LANDS at P1.5-READ-BRIDGE.

### P1-BLUEPRINT-STRUCT — NEW Blueprint struct + 4 Repository methods + 2 NEW Edge variants + DomainEvent + listener + half-migrated helper (DONE iter-4 ADDITIVE-only)

**Status at iter-5**: shipped at iter-4-P1 close 2026-05-19. **7/7 deliverables landed**:

| # | Deliverable | Location | Status |
|---|---|---|---|
| 1 | NEW `BlueprintId` newtype + `Blueprint` struct (AgentProfile fields PRESERVED per §D63.13) | `domain/src/model/nodes.rs:349-424` | ✅ |
| 2 | 2 NEW Edge variants + `EDGE_KIND_NAMES: [&str; 74]` + test renamed/assert 74 | `domain/src/model/edges.rs` | ✅ |
| 3 | 4 NEW Repository trait methods | `domain/src/repository.rs:632-680` | ✅ |
| 4 | In-memory impl of 4 methods | `domain/src/in_memory.rs:388-465` | ✅ |
| 5 | SurrealDB impl of 4 methods + `read_agent_profile_via_blueprint_or_fallback` helper | `store/src/repo_impl.rs:491-528, 700-831` | ✅ |
| 6 | `DomainEvent::BlueprintUpserted` variant + kind/event_id/roundtrip test | `domain/src/events/mod.rs` | ✅ |
| 7 | Override-tier listener arm (template-tier fan-out NARROWED per §D63.16) | `domain/src/events/listeners.rs:1089-1106, 1191-1196` | ✅ (with SCOPE-NARROWING) |

**P1 close P-FIXTURES actuals snapshot**: 11/11 new inline tests green; clippy + 4 CI guards green; `cargo build --workspace` green; ~440 LOC across 9 files; EDGE_KIND_NAMES.len() == 74 invariant green; BlueprintUpserted event roundtrip green. **Workspace `cargo test --no-fail-fast --workspace`**: **RED** (9 failing test targets — root cause documented in §1 iter-5 deviation note; CLOSED at P1.5).

### P1.5-READ-BRIDGE — Wire AgentProfileWireRow strip + composite-write at upsert_agent_profile + synthesis read-path wiring at get_agent_profile_for_agent + half-migrated test (NEW iter-5)

**Goal.** Close the workspace-RED window opened at P1 close. Land the SurrealDB-tier write boundary strip + read-path synthesis wiring + composite-write at `upsert_agent_profile` + the deferred half-migrated test. **Load-bearing acceptance criterion: workspace `cargo test --workspace --no-fail-fast` returns GREEN at P1.5 close.**

**Deliverables.**

1. **NEW `AgentProfileWireRow` intermediate struct** at `store/src/repo_impl.rs` (placed near the existing `read_agent_profile_via_blueprint_or_fallback` helper; ~20 LOC). Shape:
   ```rust
   /// CH-28 / ADR-0063 §D63.14 — SurrealDB write-boundary strip for
   /// AgentProfile rows. Mirrors `domain::AgentProfile`'s field-set
   /// MINUS the 3 per-agent override fields (`parallelize`,
   /// `model_config_id`, `mock_response`) whose persistence-of-record
   /// moved to the `blueprint` table at migration 0019. The override
   /// values for each agent persist via `upsert_agent_blueprint_override`
   /// in a composite-write paired with the `agent_profile` row write
   /// (per §D63.15).
   #[derive(Debug, Clone, Serialize)]
   pub(crate) struct AgentProfileWireRow<'a> {
       pub id: NodeId,
       pub agent_id: AgentId,
       pub blueprint: &'a phi_core::agents::profile::AgentProfile,
       pub created_at: DateTime<Utc>,
   }

   impl<'a> From<&'a AgentProfile> for AgentProfileWireRow<'a> {
       fn from(p: &'a AgentProfile) -> Self {
           Self {
               id: p.id,
               agent_id: p.agent_id,
               blueprint: &p.blueprint,
               created_at: p.created_at,
           }
       }
   }
   ```

2. **Rewrite `create_agent_profile` body** at `store/src/repo_impl.rs:635-646`:
   - Replace `let body = strip_id(serde_json::to_value(profile)?)` with `let wire = AgentProfileWireRow::from(profile); let body = strip_id(serde_json::to_value(&wire)?)`.
   - After the `agent_profile` row write succeeds, call `self.upsert_agent_blueprint_override(&Blueprint { id: BlueprintId::new(), agent_id: Some(profile.agent_id), parallelize: Some(profile.parallelize), model_config_id: profile.model_config_id.clone(), mock_response: profile.mock_response.clone(), created_at: profile.created_at })`. The Blueprint construction populates ALL 3 override fields (preserving round-trip integrity per §D63.15).
   - **Note**: AgentProfile's `parallelize: u32` is non-optional; map to `Some(profile.parallelize)`. The Blueprint's `parallelize: Option<u32>` accepts this.

3. **Rewrite `upsert_agent_profile` body** at `store/src/repo_impl.rs:675-696`:
   - Replace `let body = strip_id(serde_json::to_value(profile)?)` with `let wire = AgentProfileWireRow::from(profile); let body = strip_id(serde_json::to_value(&wire)?)`.
   - The existing BEGIN/COMMIT transaction with DELETE-then-UPDATE stays (preserves the 1:1 invariant for the `agent_profile` table side).
   - After the transaction commits successfully, call `self.upsert_agent_blueprint_override(&Blueprint { ... })` mirroring deliverable 2's composite-write body. (Sequential composite per Candidate 5 (a); `upsert_agent_blueprint_override` has its own internal BEGIN/COMMIT.)

4. **Rewrite `get_agent_profile_for_agent` SurrealDB body** at `store/src/repo_impl.rs:648-673`:
   - REPLACE the existing direct SELECT with `read_agent_profile_via_blueprint_or_fallback(self, agent).await` (the helper shipped at P1; lines 491-528). Body shrinks to ~5 lines.

5. **Mirror in-memory synthesis** at `domain/src/in_memory.rs:351-361`:
   - Modify `get_agent_profile_for_agent` body to: (a) look up the AgentProfile from the `agent_profiles` map; (b) call `get_agent_blueprint_override_for_agent` to retrieve any override-Blueprint; (c) layer override values onto the returned AgentProfile (mirroring the helper's priority: override > template > stored). For the in-memory case the template-Blueprint lookup via `get_blueprint_for_agent_profile` is also wired.

6. **Mirror in-memory composite-write** at `domain/src/in_memory.rs:344-349 (create_agent_profile)` + `:363-384 (upsert_agent_profile)`:
   - After the AgentProfile insert/upsert, also insert/upsert an override-Blueprint row via `upsert_agent_blueprint_override`. Note: in-memory `create_agent_profile` currently just inserts into the `agent_profiles` HashMap — no SCHEMAFULL constraint applies. The mirror is for symmetry + so the in-memory tests of the composite path stay consistent with the SurrealDB tier.

7. **NEW Tier-A test** `half_migrated_state_runtime_reads_via_fallback_helper` at `store/tests/migrations_test.rs` (deferred from P-MIGRATION-BACKFILL per re-spawn mandate point 1.d). Exercises the helper's fallback when only 0019 is applied and the override-Blueprint table is empty — the helper falls back to reading legacy `agent_profile` un-asserted property values.

8. **NEW Tier-B/C test** `agent_profile_wire_row_strips_override_fields_at_surreal_boundary` at `store/tests/repository_test.rs`. Constructs an AgentProfile with `parallelize: 4, mock_response: Some("X")`; calls `create_agent_profile`; queries `SELECT * FROM agent_profile WHERE id = ...` and verifies the SurrealDB row does NOT carry the 3 override fields; queries `SELECT * FROM blueprint WHERE agent_id = ...` and verifies the override-Blueprint row carries them.

9. **NEW Tier-B/C test** `upsert_agent_profile_composite_write_persists_override_blueprint` at `store/tests/repository_test.rs`. Calls `upsert_agent_profile` with an existing-agent AgentProfile; verifies (a) `agent_profile` row updated; (b) `blueprint` row keyed by `agent_id` updated; (c) `get_agent_profile_for_agent` returns the synthesized AgentProfile with the new override values.

10. **(Indirectly) repair existing tests**: `create_agent_profile_persists_row` + `agent_profile_mock_response_roundtrips_through_repo` at `store/tests/repository_test.rs` — these tests query for `parallelize` + `mock_response` directly on the `agent_profile` row. With the wire-row strip in place, the assertions need adjustment OR the tests need to query the synthesized AgentProfile (via `get_agent_profile_for_agent`) instead of the raw row. **Planner-recommendation**: rewrite the assertions to query through `get_agent_profile_for_agent` (the synthesis surface; preserves test intent at the public API contract) — this aligns with §D63.15 + §D63.13. The raw-row query at `repository_test.rs:144-152` becomes a separate "the wire-row strip is in effect" assertion: assert the raw row has no `parallelize` column.

**Tests.** Deliverables 7, 8, 9 (3 NEW tests). Plus indirect repairs to deliverable 10's existing 2 tests (these are NOT counted as NEW; they're carry-forward repairs).

**Acceptance criterion (LOAD-BEARING).** `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --no-fail-fast` returns GREEN at P1.5 close. The 9 previously-RED test targets must ALL flip to GREEN.

**Concept-alignment check.** §2 row "AgentProfile" target status confirmed honored — the synthesized projection invariant ships at P1.5.
**phi-core leverage check.** Baseline 57 (unchanged; the wire-row references `phi_core::agents::profile::AgentProfile` via a borrow — not a NEW import).
**User-facing doc updates.** None at P1.5 (P-DOCS already shipped at iter-3).
**Confidence target.** ≥ 97%.

**Pause discipline.**
- **PRIMARY** (LOAD-BEARING): PAUSE via AskUserQuestion if workspace `cargo test --workspace --no-fail-fast` is RED at P1.5 close. Do NOT proceed to P-EDGE-RENAME.
- Secondary: PAUSE if Candidate 5 surfaces (composite-write atomicity defect — see §3.E).
- Tertiary: PAUSE if cascade-4 actual sites > 11 (1.5× upper-bound 7).

**P-FIXTURES actuals snapshot**: implementer reports AgentProfileWireRow LOC + create_agent_profile body LOC + upsert_agent_profile body LOC + get_agent_profile_for_agent body LOC + in-memory mirror LOC + 3 NEW tests result + 2 indirect-repair tests result + workspace `cargo test --workspace --no-fail-fast` GREEN confirmation + which-of-9-previously-RED-targets flipped GREEN. Orchestrator confirms before P-EDGE-RENAME opens.

### P-EDGE-RENAME — Rename Edge::HasProfile → Edge::UsesProfile across 43 cascade sites + DomainEvent rename + serde alias (preserved from iter-4)

**Goal.** Mechanical rename of all 43 HAS_PROFILE / HasProfile / has_profile sites per F2.b lock. Add serde rename-aware alias for back-compat.

**Iter-5 ordering note**: P-EDGE-RENAME runs IMMEDIATELY after P1.5-READ-BRIDGE closes (iter-5 step 7). At this phase entry: `EDGE_KIND_NAMES.len() == 74` ✅; workspace `cargo test --workspace --no-fail-fast` ✅ GREEN; AgentProfile struct field-set unchanged from chunk-open.

**Deliverables.** (Preserved verbatim from iter-4.)

1. `domain/src/model/edges.rs`: rename `Edge::HasProfile` → `Edge::UsesProfile`; `name()` + `EDGE_KIND_NAMES` entries; `#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]`.
2. `domain/src/events/mod.rs`: rename `DomainEvent::HasProfileEdgeChanged` → `UsesProfileEdgeChanged` (variant + kind() + event_id() + doc-comments + serde tests).
3. `domain/src/events/listeners.rs`: rename 5 sites.
4. `domain/src/model/composites_m5.rs`: 2 doc-comments.
5. `domain/src/repository.rs`: 3 doc-comments.
6. `server/src/platform/agents/update.rs`: 2 event emission sites.
7. `store/src/repo_impl.rs`: 4 RELATE statements.
8. `store/tests/apply_org_creation_tx_test.rs`: 1 site `count_rows("has_profile")` → `count_rows("uses_profile")`.

**Tests.** Carry-forward green + NEW inline `uses_profile_edge_changed_deserializes_from_legacy_has_profile_via_alias`.

**Concept-alignment / phi-core / docs / confidence / pause-discipline.** Unchanged from iter-4 (PAUSE if cascade-2 actual sites > 47 OR if serde alias roundtrip fails).

**P-FIXTURES actuals snapshot**: as iter-4. Orchestrator confirms before P2-ACCEPTANCE opens.

### P2-ACCEPTANCE — 7 NEW acceptance tests + OPTIONAL factory helper (re-scoped from iter-4 P2-HANDLERS; scope-collapse per §D63.13 + §D63.14 + §D63.15)

**Goal.** Verify the cardinality flip end-to-end with 7 acceptance scenarios. Ship the OPTIONAL factory helper used by those tests. **NO PRODUCTION-TIER CODE CHANGES** — all production work absorbed by P1 + P1.5.

**Iter-5 scope-collapse note**: this phase was P2-HANDLERS at iter-4 with 8 deliverables spanning 9 PRODUCTION write-path re-routes + factory helper + 7 acceptance tests + struct-literal cascade re-routing. Under iter-5 the production-tier work fully dissolves:
- The 9 PRODUCTION write-path sites (`server/src/platform/agents/create.rs:160`, `update.rs:288,333,360`, `orgs/create.rs:533,560`, `system_agents/add.rs:147`) STAY as direct AgentProfile struct-literal construction. The composite-write happens inside `upsert_agent_profile`'s body at P1.5 — handler code is unchanged.
- The 30 TEST-FIXTURE sites STAY as struct-literal construction per §D63.13.
- The 22 field-read call-sites STAY compilable via the synthesis surface at `get_agent_profile_for_agent` (wired at P1.5).
- Net P2-ACCEPTANCE scope: 7 NEW acceptance tests + OPTIONAL factory helper. Total ~150-200 LOC of test code.

**Deliverables.**

1. **NEW `server/tests/acceptance_common/blueprint_fixture.rs`** (OPTIONAL but planner-recommended): factory helper `make_test_profile_with_overrides(agent_id, parallelize, model_config_id, mock_response) -> (AgentProfile, Blueprint)`. Used by the 7 NEW acceptance tests to exercise the explicit override-Blueprint write path.
2. **NEW `server/tests/acceptance_m6_agent_profile_cardinality.rs`** (Tier F). 7 tests:
   - `test_two_agents_can_share_one_template_blueprint_via_uses_profile_edge`
   - `test_per_agent_override_blueprint_parallelize_does_not_affect_sibling_agent_sharing_same_template`
   - `test_get_agent_profile_for_agent_synthesises_template_blueprint_plus_per_agent_override`
   - `test_upsert_agent_profile_no_longer_deletes_sibling_rows_for_same_agent_id` (verifies the 1:1 → N:1 flip semantic)
   - `test_drop_unique_constraint_post_migration_allows_two_rows_with_same_agent_id`
   - `test_uses_profile_edge_emits_single_agent_at_ch28_per_adr_0063_d63_7`
   - `test_blueprint_upserted_event_emits_when_override_blueprint_written`

**Tests.** Deliverable 2 = 7 NEW Tier-F tests. Full workspace `cargo test --workspace --no-fail-fast` GREEN at P2-ACCEPTANCE close.

**Concept-alignment / phi-core / docs / confidence.** Concept §2 row "Edge Types" target satisfied; phi-core baseline 57; no doc updates; confidence ≥ 97%.

**Pause discipline.** PAUSE if any of the 7 acceptance tests reveals a §D63.15 synthesis invariant violation (e.g., wire-format roundtrip surfaces a field-value mismatch between AgentProfile struct field and Blueprint row column) — that signals §D63.13 or §D63.15 needs amendment.

**P-FIXTURES actuals snapshot**: 7 acceptance test count + factory helper LOC + workspace test count + clippy result. Orchestrator confirms before P-SEAL opens.

### P-SEAL — ADR-0063 Accepted (16 sub-decisions) + paperwork + cycle-index flip + cargo-clean

**Goal.** Close the chunk: ADR-0063 flipped Proposed → Accepted with all 16 sub-decisions (iter-5 expanded from iter-4's 13); concept-audit matrix flipped; cycle-index row flipped `in-flight` → `ready-for-audit`; file `D-CH28-FOLLOWUP-01`; cargo-clean placement-1 applied.

**Deliverables.**
1. ADR-0063 Status: Accepted. All 7 sections + **16 sub-decisions §D63.1..§D63.16**. §D63.14 + §D63.15 + §D63.16 NEW iter-5 bodies authored. §D63.1 + §D63.11 + §D63.13 iter-4 cross-ref annotations preserved. §"Consequences ### For CH-36" body amended per §D63.16.
2. Concept-audit matrix amendment verified.
3. Forward-scope M6+-OPEN-01 Resolution line authored.
4. **NEW iter-5**: file `D-CH28-FOLLOWUP-01` at `docs/specs/v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md` with `Impl chunk: M6-DEFERRED-04` (CH-36).
5. M6 forward-scope verified-header bumped.
6. Cycle-index row flipped to `ready-for-audit`; Iterations stays `pending` and Status stays `in-flight` per cycle-index lifecycle paragraph.
7. cargo-clean placement-1: after final `cargo test --workspace` in P2-ACCEPTANCE, run `cargo clean`.
8. Verified-header on every doc touched (11 files per §3.C + §4 + §5 + NEW D-CH28-FOLLOWUP-01).

**Tests.** Final workspace test + clippy + 4 CI guards all green.
**Concept-alignment / phi-core / docs / confidence.** All §2 targets reached; phi-core 57; tier-1 docs shipped; confidence ≥ 99%.
**Pause discipline.** None.

---

## §8 — Tests summary

**Expected total test count at chunk close**: **1576 baseline + 24-25 NEW MUST-SHIP + 0 MAY-COVER (predicted)** = **[1600, 1601]** point estimate within band.

**Plan §8 chunk-close prediction band**: `[1590, 1604]` (preserved from iter-2/iter-3/iter-4 at iter-5).

**Per-Tier breakdown (v22 P1 applied; iter-5 update for P1.5)**:

| Tier | Tests | Notes |
|---|---|---|
| **A — schema/migration** | 5 | 4 already shipped at P-MIGRATION-* (0019/0020 first-apply + idempotency); 1 NEW `half_migrated_state_runtime_reads_via_fallback_helper` AT P1.5 (deferred from P-MIGRATION-BACKFILL). |
| **B — domain model/types** | 8 | All 8 already shipped at P1 close (3 inline at nodes.rs for Blueprint, 3 inline at edges.rs for edge variants + EDGE_KIND_NAMES cardinality bump, 2 inline at events/mod.rs for BlueprintUpserted). |
| **C — repository (engine)** | 2 (NEW iter-5) | `agent_profile_wire_row_strips_override_fields_at_surreal_boundary` + `upsert_agent_profile_composite_write_persists_override_blueprint` AT P1.5. Validates §D63.14 + §D63.15 invariants. |
| **D — server/HTTP** | 0 | No NEW handlers; production code unchanged at P2-ACCEPTANCE. Handler-tier behaviour validated end-to-end at Tier F. |
| **E — listener/event** | 1 | `uses_profile_edge_changed_deserializes_from_legacy_has_profile_via_alias` at P-EDGE-RENAME. |
| **F — integration/acceptance** | 7 | NEW `acceptance_m6_agent_profile_cardinality.rs` (7 scenarios) AT P2-ACCEPTANCE. |
| **Total NEW MUST-SHIP** | **23-25** | 11 (P1 DONE) + 3 (P1.5 NEW: 1 Tier-A + 2 Tier-C) + 1 (P-EDGE-RENAME) + 7 (P2-ACCEPTANCE) + 0-2 (P1.5 indirect-repair adjustments to existing 2 RED tests; counted as repairs not NEW). |
| **MAY-COVER inline unit-extras** | 0-3 | Implementer may add round-trip unit tests on the wire-row pattern OR the composite-write transaction; these would land in Tier C. |

**MUST-SHIP** (named test files / test fns):
- Already shipped at P1 close: 11 inline tests (Blueprint struct + Edge variants + EDGE_KIND_NAMES + BlueprintUpserted).
- At P1.5: `store/tests/migrations_test.rs::half_migrated_state_runtime_reads_via_fallback_helper` + `store/tests/repository_test.rs::agent_profile_wire_row_strips_override_fields_at_surreal_boundary` + `store/tests/repository_test.rs::upsert_agent_profile_composite_write_persists_override_blueprint`.
- At P-EDGE-RENAME: `domain/src/events/mod.rs::uses_profile_edge_changed_deserializes_from_legacy_has_profile_via_alias` (inline).
- At P2-ACCEPTANCE: `server/tests/acceptance_m6_agent_profile_cardinality.rs` 7 tests.

**Named expected-still-green tests** (v17 grep-verify discipline applied 2026-05-19; iter-5 update):
- `domain/src/model/nodes.rs::agent_profile_model_config_id_defaults_to_none_for_pre_m5_rows` — STAYS GREEN per §D63.13.
- `domain/src/model/nodes.rs::agent_profile_mock_response_defaults_to_none_for_pre_ch02_rows` — STAYS GREEN per §D63.13.
- `domain/src/model/nodes.rs::agent_profile_mock_response_roundtrip_preserves_some_value` — STAYS GREEN per §D63.13.
- `domain/src/model/edges.rs::edge_kind_names_cardinality_is_74_pinned_at_compile_time` — RENAMED + asserts 74 (shipped at P1).
- **CRITICAL iter-5**: `store/tests/repository_test.rs::create_agent_profile_persists_row` + `::agent_profile_mock_response_roundtrips_through_repo` — currently RED at P1 close; ADJUSTED at P1.5 (deliverable 10) to query the synthesized surface; flip to GREEN at P1.5 close.
- All 9 server-tier RED acceptance suites at P1 close (`acceptance_agents_profile`, `acceptance_m4`, `acceptance_m5_memory`, `acceptance_m5_sessions`, `acceptance_sessions_m5p4`, `acceptance_system_agents`, `acceptance_system_flows_s03`, `sse_live_stream_test`) — **FLIP TO GREEN at P1.5 close** via the synthesis read-path wiring (these tests previously read AgentProfile via the trait and consumed the 3 override fields; after P1.5 wiring, the synthesis populates the fields transparently).
- Migration runner test suite: `store/tests/migrations_test.rs` 0001..0020 — 5/5 confirmed green.

---

## §9 — Pre-chunk gate

**Reading list (mandatory) — iter-5 entry into P1.5:**
1. **iter-4 plan §1 + §7 P1-BLUEPRINT-STRUCT body** at the same plan path (DONE iter-4 P1 close report context).
2. **Current source**: 
   - `modules/crates/store/src/repo_impl.rs:459-528` (the half-migrated helper shipped at P1).
   - `modules/crates/store/src/repo_impl.rs:635-696` (the failing `create_agent_profile` + `upsert_agent_profile` bodies).
   - `modules/crates/store/src/repo_impl.rs:700-831` (the 4 NEW Blueprint methods at P1 — `upsert_agent_blueprint_override` is invoked from P1.5's composite-write).
   - `modules/crates/domain/src/in_memory.rs:344-468` (in-memory impls).
   - `modules/crates/domain/src/model/nodes.rs:314-424` (AgentProfile + Blueprint structs).
   - `modules/crates/store/tests/repository_test.rs:118-201` (the 2 currently-RED unit tests).
   - `modules/crates/store/migrations/0019_agent_profile_n_to_1_schema.surql:152-172` (the REMOVE FIELD body causing the SCHEMAFULL rejection).
3. **ADR-0063** at `docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-n-to-1.md` (the iter-4 §D63.13 body + iter-5 §D63.14 + §D63.15 sub-decisions to be authored at P-SEAL).

**Carry-forward invariants** (verified at chunk open):
- `cargo build --workspace --all-targets -j 4` GREEN.
- `cargo test -p store --test migrations_test` 5/5 green.
- `cargo test --workspace --no-fail-fast` **RED** at P1 close (the 9 failing targets list) — P1.5 closes this.
- `bash scripts/check-phi-core-reuse.sh` green.
- `bash scripts/check-doc-links.sh` green.
- `bash scripts/check-ops-doc-headers.sh` green.
- `bash scripts/check-spec-drift.sh` green.

**Pending decisions carried into this chunk:**
- F1 + F2 + F3 — LOCKED at gate-1 (DIVERGENT).
- Iter-4 §D63.13 + iter-5 §D63.14 + §D63.15 + §D63.16 — APPLIED.

---

## §10 — Close criteria

**4 aspects (each graded pass/fail):**
- **Code aspect** — P0 + P-DOCS + P-MIGRATION-SCHEMA + P-MIGRATION-BACKFILL + P1-BLUEPRINT-STRUCT (DONE) + **P1.5-READ-BRIDGE** + P-EDGE-RENAME + P2-ACCEPTANCE + P-SEAL deliverables shipped; **`cargo test --workspace --no-fail-fast` GREEN at chunk close** (LOAD-BEARING); `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` green; `cargo fmt --all -- --check` green; all 4 CI guards green.
- **Docs aspect** —
  - *Governance tier*: ADR-0063 Status=Accepted with all **16 sub-decisions** + Pre-existing-behaviour preservation notes; M6+-OPEN-01 marker Resolution line; concept-audit matrix amended; ADR-0057 §D57.7 amendment; `D-CH28-FOLLOWUP-01` filed with `M6-DEFERRED-04` allocation; verified-headers on all touched files.
  - *User-facing tier*: §3.C tier-1 (architecture) all 3 rows shipped in-chunk; tier-2 + tier-3 deferred to CH-37.
- **phi-core leverage aspect** — §3 import-count delta = +0; forbidden-duplication greps return 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's target-status at chunk-close achieved.

**2 confidence % (each with named numerator/denominator):**
- **Implementation confidence %** = `(claims-honored / total-claims-in-scope-for-chunk)`. Target: **≥ 9/10 (90%)**. Anticipated: **10/10** at chunk close.
- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check / doc-pages-touched-in-chunk)`. Target: **10/10**.

**Composite = min(...).** Target: **9/10**. Aim: **10/10**.

**P4 chunk-seal paperwork checklist:**
- Every modified doc verified-header description matches body diff.
- `_concept-audit-matrix.md` rows copied letter-for-letter from §2 target column.
- P-SEAL cycle-index row update: `grep -n 0412eb06 .../_cycle-index.md` MUST return ≥ 1 hit.
- Cargo-clean placement-1 + placement-2 applied per CLAUDE.md.
- `D-CH28-FOLLOWUP-01` filed with explicit `Impl chunk: M6-DEFERRED-04` allocation (NOT `TBD`).

---

## §11 — Post-chunk independent audit plan

**Audit envelope size**: phase count = 7 substantive phases → **LARGE (3 auditors)** → A + B + C. Tier preserved at iter-5.

**iter-5 audit-prompt note**: prompts updated for the P1.5-READ-BRIDGE insertion + §D63.14/§D63.15/§D63.16 sub-decisions + workspace-RED-window closure verification. Audit-A claim 8 wording flips (composite-write at upsert_agent_profile is now CONCRETE behaviour at P1.5, not a future P2 deliverable); Audit-A NEW claim 18 verifies AgentProfileWireRow; Audit-B claim 1 sub-decision count 13 → 16; Audit-C NEW claim 15 verifies workspace `cargo test --no-fail-fast` GREEN at close.

### Audit A (code + phi-core + K8s) — ≤ 600 words

```
You are auditing CH-28 in baby-phi at /root/projects/phi/baby-phi/. Read-only on source.
Plan at /root/projects/phi/baby-phi/docs/specs/plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md.
ADR at /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-n-to-1.md.

Verify each numbered claim with file:line citation; PASS/FAIL each.

1. Migration 0019_agent_profile_n_to_1_schema.surql exists + body conformant.
2. Migration 0020_agent_profile_n_to_1_backfill.surql exists + body backfills correctly.
3. NEW Blueprint struct + BlueprintId newtype at nodes.rs adjacent to AgentProfile.
   AgentProfile struct fields PRESERVED on-struct as in-process projection per
   iter-4 §D63.13 (parallelize / model_config_id / mock_response STAY).
4. NEW Edge variants AgentProfileUsesBlueprint + AgentUsesBlueprintOverride.
   EDGE_KIND_NAMES.len() == 74. Test edge_kind_names_cardinality_is_74_pinned_at_compile_time
   passes at edges.rs.
5. Edge::HasProfile renamed → Edge::UsesProfile + serde alias decorator.
6. DomainEvent::HasProfileEdgeChanged renamed → UsesProfileEdgeChanged.
7. 4 NEW Repository methods exist + in-memory + SurrealDB impls (shipped at P1).
8. upsert_agent_profile body (P1.5):
   - DELETE-then-UPDATE BEGIN/COMMIT preserved for agent_profile table.
   - Wire-body via AgentProfileWireRow (§D63.14) STRIPS the 3 override fields
     before serde_json::to_value.
   - After agent_profile row write succeeds, composite-write calls
     upsert_agent_blueprint_override with a Blueprint row carrying the 3
     override values (§D63.15).
9. AgentProfile continues to embed phi_core::AgentProfile at nodes.rs:322.
10. apply_agent_creation compound tx atomicity preserved.
11. get_agent_profile_for_agent body (P1.5):
    - Surreal impl delegates to read_agent_profile_via_blueprint_or_fallback
      (the helper shipped at P1 / repo_impl.rs:491-528).
    - In-memory impl mirrors the synthesis logic at in_memory.rs.
    - Returned AgentProfile has 3 fields populated per §D63.15 projection.
12. cargo test --workspace --no-fail-fast GREEN at expected count [1590, 1604].
    NOTE: NOT-EXECUTED-IN-AUDIT (sandbox-blocked); orchestrator gate-4 runs MUST-RUN.
13. CI guards green; no new use phi_core:: imports beyond §3 (0).
    grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l — expect 57.
14. NEW acceptance test file at acceptance_m6_agent_profile_cardinality.rs with 7 scenarios.
15. NEW BlueprintUpserted event variant + 2 tests at events/mod.rs.
16. K8s posture: all 7 axes resolved (no new blocker; CHK8S-D ledger not appended).
17. Half-migrated-state runtime helper read_agent_profile_via_blueprint_or_fallback
    present at repo_impl.rs; test half_migrated_state_runtime_reads_via_fallback_helper
    green (shipped at P1.5 per iter-5 deferred-test resolution).
18. (NEW iter-5) AgentProfileWireRow at store/src/repo_impl.rs:
    - pub(crate) struct (not pub).
    - Mirrors AgentProfile field-set MINUS the 3 override fields.
    - From<&AgentProfile> impl wires the field mapping.
    - Used by create_agent_profile + upsert_agent_profile via wire-row construction
      before serde_json::to_value.
    - SurrealDB row body does NOT carry the 3 override fields (verified by
      agent_profile_wire_row_strips_override_fields_at_surreal_boundary test).
19. (NEW iter-5) Composite-write at upsert_agent_profile invokes
    upsert_agent_blueprint_override sequentially after the agent_profile row write
    succeeds (NOT in a single outer transaction wrapper; per §D63.15 + Candidate 5 (a)).
20. (NEW iter-5) §D63.16 SCOPE-NARROWING — listener template-tier fan-out NOT shipped
    at P1; D-CH28-FOLLOWUP-01 filed at P-SEAL with M6-DEFERRED-04 allocation.

PASS/FAIL each. ≤ 600 words. After completion run cargo clean per v9 placement-1.
```

### Audit B (concept + docs + ADR) — ≤ 600 words

```
You are auditing CH-28's concept-fidelity + docs-fidelity. Read-only.
Plan at /root/projects/phi/baby-phi/docs/specs/plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md.

Verify each numbered claim; PASS/FAIL each.

1. ADR-0063 Accepted at .../m6/decisions/0063-agent-profile-cardinality-n-to-1.md
   with ALL 7 top-level sections: ## Forks, ## Context, ## Sub-decisions
   (§D63.1..§D63.16 — SIXTEEN sub-decisions, iter-5 expanded from iter-4's 13 to add
   §D63.14 + §D63.15 + §D63.16), ## Cross-references (a/b/c/d),
   ## Consequences (### For CH-29, CH-36, CH-37, M7-NFR-observability, future M7 cleanup;
   ### For CH-36 body amended per §D63.16 with list_agents_using_blueprint_template),
   ## Revisit triggers (9 bullets — iter-5 adds §D63.15 atomicity bullet),
   ## Verification.
   Each sub-decision ends with Pre-existing-behaviour preservation note.
   §D63.1 + §D63.11 carry iter-4 cross-ref annotations to §D63.13 (preserved).

2. M6+-OPEN-01 marker has Resolution line citing CH-28 / ADR-0063 (cycle hex 0412eb06).
3. concepts/agent.md §"Soul" amended; verified-header bumped.
4. concepts/ontology.md line 98 amended; NEW rows added; verified-header bumped.
5. NEW m6/architecture/agent-profile-cardinality.md exists.
6. m1/architecture/graph-model.md amended.
7. m1/architecture/schema-migrations.md migration-list table includes 0019 + 0020.
   NEW §"Split-migration pattern" subsection present.
8. m5_1/drifts/_concept-audit-matrix.md amended.
9. m5_2/decisions/0057-bucket-b-convention-ratification.md §D57.7 amendment line citing 72 → 74.
10. Plan archive at cycle folder exists with cycle hex 0412eb06.
11. _cycle-index.md row has CH-28 status `ready-for-audit` at P-SEAL.
12. Verified-header on every doc touched carries this cycle hex.
13. Prior-chunk doc invariants intact:
    - ADR-0057 §D57.7 EDGE_KIND_NAMES amendment (72 → 74).
    - ADR-0034 §D34.6 wrap-vs-runtime separation referenced in §D63.8.
    - phi-core-mapping.md AgentProfile row unchanged.
14. F1 + F2 + F3 ALL marked DIVERGENT in ADR-0063 ## Forks header table.
15. §D63.13 sub-decision body preserved verbatim from iter-4.
16. (NEW iter-5) §D63.14 sub-decision body covers AgentProfileWireRow
    write-boundary strip + rationale for approach (b) over (a) + (c).
17. (NEW iter-5) §D63.15 sub-decision body covers read-path synthesis wiring
    at get_agent_profile_for_agent + composite-write at upsert_agent_profile
    (sequential composite per Candidate 5 (a); failure-mode caveat documented).
18. (NEW iter-5) §D63.16 sub-decision body covers listener template-tier fan-out
    scope-narrowing + D-CH28-FOLLOWUP-01 routing to M6-DEFERRED-04 (CH-36).
19. (NEW iter-5) D-CH28-FOLLOWUP-01 filed at
    docs/specs/v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md
    with `Impl chunk: M6-DEFERRED-04` (NOT `TBD`; v13 Row 4 rule).

PASS/FAIL each. ≤ 600 words. After completion run cargo clean per v9 placement-1.
```

### Audit C (carry-forward regression — LARGE envelope only) — ≤ 600 words

```
You are auditing CH-28's carry-forward regression posture. Read-only.
Plan at /root/projects/phi/baby-phi/docs/specs/plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md.

Verify each numbered claim; PASS/FAIL each.

1. acceptance_sessions_m5p4 GREEN (was RED at P1 close; flipped at P1.5 via
   read-path synthesis wiring per §D63.15). 30 TEST-FIXTURE AgentProfile
   struct-literal sites PRESERVED as-is per iter-4 §D63.13.
2. acceptance_agents_profile GREEN (was RED at P1 close; flipped at P1.5).
3. acceptance_per_session_consent_gating GREEN.
4. sse_live_stream_test GREEN (was RED at P1 close; flipped at P1.5).
5. e2e_first_session.rs + session_live_tail_test.rs (cli tests) GREEN.
6. acceptance_common/admin.rs + acceptance_common/m5_bootstrap.rs fixtures GREEN.
7. domain/tests/in_memory_ch25_owns_edges.rs GREEN.
8. Migration runner test suite green (0001..0020 first-apply; 5/5 confirmed).
9. (P1.5 NEW) half_migrated_state_runtime_reads_via_fallback_helper GREEN.
10. (P-EDGE-RENAME NEW) uses_profile_edge_changed_deserializes_from_legacy_has_profile_via_alias
    GREEN.
11. EDGE_KIND_NAMES cardinality test: edge_kind_names_cardinality_is_74_pinned_at_compile_time
    passes. ADR-0057 §D57.7 amendment internally consistent.
12. Audit hash-chain symmetry preserved per ADR-0063 §D63.7.
13. Workspace test count at [1590, 1604]: NOT-EXECUTED-IN-AUDIT; orchestrator gate-4 runs.
14. iter-4 §D63.13 projection invariant verified:
    - get_agent_profile_for_agent returns AgentProfile with 3 fields populated.
    - upsert_agent_profile composite-write writes both agent_profile row + override-Blueprint.
    - 22 field-read call-sites continue to compile and pass.
15. (NEW iter-5 — LOAD-BEARING) Workspace `cargo test --workspace --no-fail-fast`
    GREEN at chunk close (was RED at P1 close per 9 failing test targets:
    acceptance_agents_profile, acceptance_m4, acceptance_m5_memory,
    acceptance_m5_sessions, acceptance_sessions_m5p4, acceptance_system_agents,
    acceptance_system_flows_s03, sse_live_stream_test, repository_test).
    ALL 9 previously-RED targets confirm GREEN at P1.5 close + remain GREEN
    through P-EDGE-RENAME + P2-ACCEPTANCE + P-SEAL.
16. (NEW iter-5) repository_test::create_agent_profile_persists_row +
    ::agent_profile_mock_response_roundtrips_through_repo GREEN at chunk close
    (adjusted at P1.5 per deliverable 10 to query the synthesized surface).
17. (NEW iter-5) P1.5 composite-write integration tests GREEN:
    - agent_profile_wire_row_strips_override_fields_at_surreal_boundary GREEN.
    - upsert_agent_profile_composite_write_persists_override_blueprint GREEN.

PASS/FAIL each. ≤ 600 words. After completion run cargo clean per v9 placement-1.
```

**Audit pass criteria:**
- Any new drift → its own drift file created BEFORE chunk seals.
- Audit-flagged concept contradiction → either fixed in-chunk, renegotiated, or converted to drift.
- Chunk seal blocked until all 3 audits return clean.

---

## §12 — Verification section (end-to-end recipe)

```bash
# Working directory: any (commands use absolute paths)

# 1. CI guards
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (LOAD-BEARING at P1.5 close)
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --no-fail-fast -- --test-threads=1

# 3. Chunk-specific
# (a) Migration 0019 + 0020 first-apply + idempotency
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test

# (b) Half-migrated-state runtime tolerance (NEW at P1.5)
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_test half_migrated_state

# (c) AgentProfile wire-row strip + composite-write (NEW at P1.5)
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test repository_test agent_profile_wire_row_strips_override_fields_at_surreal_boundary
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test repository_test upsert_agent_profile_composite_write_persists_override_blueprint

# (d) Previously-RED tests now GREEN
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test repository_test create_agent_profile_persists_row
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test repository_test agent_profile_mock_response_roundtrips_through_repo

# (e) AgentProfile cardinality acceptance suite
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p server --test acceptance_m6_agent_profile_cardinality

# (f) Edge rename + Blueprint inline tests
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain edge_kind_names_cardinality_is_74
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain blueprint_struct
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain uses_profile_edge_changed_deserializes_from_legacy

# (g) phi-core leverage delta (expect 57)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l

# (h) Forbidden-duplication grep (expect 1)
grep -rnE "^pub struct AgentProfile\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::" | wc -l

# (i) AgentProfileWireRow is pub(crate) — NOT pub (iter-5 §D63.14)
grep -nE "pub\(crate\) struct AgentProfileWireRow" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs
# Expect: 1 hit

# (j) EDGE_KIND_NAMES invariant (expect 74)
grep -n "EDGE_KIND_NAMES.len() ==" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs

# (k) Concept-doc amendment citations
grep -n "USES_PROFILE" /root/projects/phi/baby-phi/docs/specs/v0/concepts/ontology.md | head -3
grep -n "N:1" /root/projects/phi/baby-phi/docs/specs/v0/concepts/ontology.md | head -3

# (l) Edge rename verification
git -C /root/projects/phi/baby-phi grep -nE 'HasProfile|HAS_PROFILE' modules/crates/domain/src/ modules/crates/store/src/ | wc -l
# Expect: 0 hits in production code (allowed in audit JSON wire-key + immutable migration source-comments + serde alias decorator)

# (m) iter-4 §D63.13 projection invariant + iter-5 §D63.14 wire-row verification
grep -n "pub parallelize\|pub model_config_id\|pub mock_response" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs
# Expect: 6 hits (3 on AgentProfile preserved per §D63.13 + 3 on NEW Blueprint struct)

# (n) Composite-write wiring at upsert_agent_profile (iter-5 §D63.15)
grep -n "upsert_agent_blueprint_override" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs
# Expect: ≥ 2 hits (1 trait method impl + 1 call from upsert_agent_profile + 1 call from create_agent_profile)

# (o) Read-path synthesis wiring at get_agent_profile_for_agent (iter-5 §D63.15)
grep -n "read_agent_profile_via_blueprint_or_fallback" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs
# Expect: ≥ 2 hits (1 helper definition + 1 call from get_agent_profile_for_agent SurrealDB impl)

# 4. Drift-file status
grep -n "Resolution.*CH-28" /root/projects/phi/baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01*.md
# Expect: 1 file (D-CH28-FOLLOWUP-01 filed at P-SEAL per iter-5 §D63.16)

# 5. ADR-0063 status
grep -n "Status: Accepted" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-n-to-1.md
grep -cE "^### §D63\." /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-n-to-1.md
# Expect: 16 (sub-decisions §D63.1..§D63.16 per iter-5)

# 6. Cargo-clean v9 placement-1
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml
```

---

## Plan-iteration banner

- **Iter 1** (2026-05-19, planner v22): inaugural; planner-rec F1.a + F2.a + F3.a; envelope MEDIUM; 5 phases.
- **Iter 2** (2026-05-19, planner v22 re-spawn): 3-of-3 gate-1 forks user-locked DIVERGENT; envelope MEDIUM → LARGE; phases 5 → 7; ADR sub-decisions 9 → 12; §3 cascade-band [8, 20] → [42, 70]; §8 [1584, 1592] → [1590, 1604]. v22 P13 appendix self-emitted.
- **Iter 3** (2026-05-19, planner v22 gate-2.5): phase-ordering swap — P1-BLUEPRINT-STRUCT to step 5, P-EDGE-RENAME to step 6. §7.0 rationale (compile-time framing) was empirically incorrect.
- **Iter 4** (2026-05-19, planner v22 gate-2.5 Architectural-FAIL re-spawn): Outcome-1 narrowing — P1 ADDITIVE-only; cascade relocated to P2-HANDLERS; §D63.13 NEW (in-process struct field-set preservation); sub-decisions 12 → 13; §7.0 re-cast to attribute cascade to runtime-test not compile-time.
- **Iter 5** (2026-05-19, planner v22 second Architectural-FAIL re-spawn, user-routed Option B): P1-BLUEPRINT-STRUCT iter-4 ADDITIVE-only landed correctly (7/7 deliverables; 11/11 inline tests green) but workspace `cargo test --no-fail-fast` is RED at P1 close (9 failing test targets: 7 server-tier acceptance suites + sse_live_stream_test + 2 repository_test units). Root cause: migration 0019 REMOVE FIELD already shipped at P-MIGRATION-SCHEMA close + create_agent_profile/upsert_agent_profile bodies still serialize the WHOLE AgentProfile JSON → SurrealDB SCHEMAFULL rejects the now-undefined fields. Iter-5 INSERTS NEW phase **P1.5-READ-BRIDGE** between P1 [DONE] and P-EDGE-RENAME with: (a) NEW `AgentProfileWireRow` intermediate struct at `store/src/repo_impl.rs` (chosen approach (b) over (a) custom serde + (c) JSON.remove for compile-time type-safety); (b) write-path body rewrites at create_agent_profile + upsert_agent_profile via wire-row strip + composite-write to upsert_agent_blueprint_override (sequential composite per Candidate 5 (a)); (c) read-path body rewrite at get_agent_profile_for_agent (SurrealDB + in-memory) via the half-migrated helper shipped at P1; (d) deferred half_migrated_state_runtime_reads_via_fallback_helper test landing; (e) 2 NEW Tier-B/C tests for wire-row strip + composite-write invariants. **Acceptance criterion (LOAD-BEARING)**: `cargo test --workspace --no-fail-fast` GREEN at P1.5 close. P2-HANDLERS scope-collapses → renamed P2-ACCEPTANCE (7 NEW acceptance tests + OPTIONAL factory helper; NO production code changes — all production work absorbed by P1 + P1.5). ADR sub-decisions 13 → 16: §D63.14 NEW (write-boundary strip via AgentProfileWireRow), §D63.15 NEW (read-path synthesis wiring + composite-write transaction semantics), §D63.16 NEW (listener template-tier fan-out scope-narrowing → `D-CH28-FOLLOWUP-01` filed at P-SEAL with `M6-DEFERRED-04` allocation; CH-36 a04 supervisor body inherits the requirement). §3.E adds Candidate 5 (composite-write atomicity) + Candidate 6 (template-tier fan-out). §6 carry-forward table adds new row asserting P1 BLUEPRINT additive surface invariants. §11 audit prompts updated: Audit-A claim 8 + 11 + NEW 18 + 19 + 20; Audit-B claim 1 (count 13 → 16) + NEW 16 + 17 + 18 + 19; Audit-C NEW 15 + 16 + 17 (LOAD-BEARING workspace-GREEN verification). §8 per-Tier breakdown updated; total NEW MUST-SHIP 21 → 23-25 within band [1590, 1604]. §7.0 re-cast to explain the iter-4 "additive-only ⇒ green by construction" empirical falsification (the schema-vs-struct mismatch at the SurrealDB boundary is INHERENT between migration 0019 apply and bridge-code wiring; P1.5 is THE bridge phase). All 3 locked forks F1.c + F2.b + F3.b preserved verbatim; iter-4 §D63.13 in-process struct preservation invariant carried forward unchanged. Recommended gate: gate-1.5 approval → resume Phase 2f (P1.5-READ-BRIDGE).

**Direct-approval criteria check** (unchanged from iter-2/iter-3/iter-4):
- ✗ Criterion 1 (no locked forks): FAILS — F1.c + F2.b + F3.b user-locked DIVERGENT.
- ✗ Criterion 2 (scope ≤ 1.5× forward-scope): FAILS marginally.
- ✓ Criterion 3 (zero phi-core leverage delta): holds; +0.
- ✓ Criterion 4 (no new K8s blocker class): holds.
- ✗ Criterion 5 (audit envelope ≤ medium): FAILS — LARGE envelope.
- ✓ Criterion 6 (confidence ≥ 9/10): holds.
- ✗ Criterion 7 (no new migration): FAILS — 2 NEW migrations ship.
