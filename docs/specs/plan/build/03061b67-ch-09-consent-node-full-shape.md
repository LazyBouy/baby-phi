<!-- Last verified: 2026-04-30 by Claude Code -->

# CH-09 — Consent node full shape (11 fields per concept doc)

**Plan file token:** `03061b67` (generated 2026-04-30 at chunk-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/03061b67-ch-09-consent-node-full-shape.md`.
**Chunk ID:** CH-09 (forward-scope §1 lines 100–105; §5 inventory row line 417).
**Severity:** ⚠ HIGH.
**Expected effort:** ~1 engineer-day.
**Hard prerequisites:** none.
**Chunks unblocked at close:** CH-10 (Consent lifecycle state machine — needs the `state` field + enum), CH-11 (Per-Session consent gating — needs full struct + state).

---

## Context

### The simple version

The concept doc (`permissions/06-multi-scope-consent.md` §"Consent Node") specifies a Consent record with **11 fields** carrying everything needed to gate Authority Template grants: who consented, what scope (org + templates + actions), what state (requested/acknowledged/declined/revoked/timed-out/expired), three timestamps, a revocability flag, and a provenance string.

Today's `Consent` struct ships only **5 fields** — `id`, `subordinate`, `scoped_to`, `granted_at`, `revoked_at`. Missing: the entire lifecycle state, the per-template / per-action scope, the response timestamp, the revocability flag, and provenance. Drift D-new-04 (HIGH) records this gap. The downstream consent state machine (CH-10) and per-session gating (CH-11) cannot function without these fields.

That's the chunk. Extend `Consent` to the concept-doc shape, define the `ConsentState` enum + nested `ConsentScope` struct, ship migration 0010 to redefine the SurrealDB table, update the one test fixture that constructs a `Consent` today. ~1 engineer-day.

### What this chunk does NOT do

- Does NOT ship the consent **state machine** (transition function from `Requested` → `Acknowledged` / `Declined` / `TimedOut`, etc.). That's CH-10 (drift D-new-05). CH-09 ships the enum + the field; CH-10 wires the transitions.
- Does NOT change the **Permission Check engine's `ConsentIndex`** projection. The index still consumes `(AgentId, OrgId)` pairs at this chunk; CH-11 evolves it to include scope/state filtering.
- Does NOT add `Repository::get_consent` / `list_consents_for_subordinate` / `update_consent_state`. The trait surface stays at the existing single method `create_consent`. CH-10 adds `update_consent_state`; CH-11 adds the read methods. Each downstream chunk owns its own repo additions.
- Does NOT add `ApprovalMode` enum on Grant (per-session approval principal — `auto` / `human_required` / `subordinate_required`). That lives on Grant, not Consent — CH-11's job.
- Does NOT migrate existing production data. There is **no production data** — only one test fixture at `repository_test.rs:359` constructs a Consent today. Migration 0010 redefines the consent table cleanly without back-compat shims.

### User-decided forks (locked at plan-review, 2026-04-29)

1. **Field naming** — Rename to match concept doc verbatim: `subordinate` → `agent_id`, flatten `scoped_to: OrgId` into nested `scope: ConsentScope { org, templates, actions }`, rename `granted_at` → `responded_at` (with semantic shift: now `Option<DateTime<Utc>>`, set when subordinate responds Acknowledged or Declined). Migration 0010 redefines the consent table cleanly. Best concept-doc fidelity; one test fixture changes.

2. **`ConsentState` enum scope** — Ships in CH-09 with all 6 variants (Requested, Acknowledged, Declined, Revoked, TimedOut, Expired). `#[serde(rename_all = "snake_case")]` for wire form. Default `Acknowledged` for back-compat (matches the implicit-policy default — when no consent flow is required, the consent record is auto-acknowledged at creation). CH-10 ships ONLY the transition function + `update_consent_state` repo method — no struct changes. Cleaner separation; the struct lands complete in one chunk.

3. **Repository surface** — Stays minimal. Only `create_consent` exists; CH-09 keeps it that way. CH-10 adds `update_consent_state`; CH-11 adds read methods. Each chunk owns its own repo additions.

### Forward-scope reference

[CH-09 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 100–105) + [§5 inventory](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 417).

---

## §1 — Why this chunk (one paragraph)

The concept doc has specified an 11-field Consent record since M0; the implementation shipped 5 fields at M2. Drift D-new-04 (HIGH) has been tracking the gap since 2026-04-24. CH-10 (consent lifecycle state machine) and CH-11 (per-session gating) are blocked on this — neither makes sense without the per-template / per-action scope and the lifecycle state field. CH-09 closes the gap by extending `Consent` to the concept-doc shape, defining the supporting `ConsentScope` nested struct + `ConsentState` enum, shipping migration 0010 to redefine the SurrealDB table cleanly, and updating the one test fixture that constructs a Consent today. The chunk is small (~1 engineer-day) because there are zero production callsites — Consent creation has been test-only territory until now — and the repo trait surface stays minimal at this chunk.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Status at chunk-close |
|---|---|---|---|---|
| [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | § "Consent Node (New Node Type)" lines 349–367 | 11-field Consent: `consent_id`, `agent_id`, `scope.org`, `scope.templates`, `scope.actions`, `state`, `requested_at`, `responded_at`, `revoked_at`, `revocable`, `provenance` | contradicted (5 fields shipped) | honored — Consent struct rewritten with all 11 fields per concept doc; field names match verbatim |
| `permissions/06-multi-scope-consent.md` | § "Consent Lifecycle" lines 369–414 | `ConsentState` enum with 6 variants: Requested, Acknowledged, Declined, Revoked, TimedOut, Expired | silent-in-code (enum doesn't exist) | partially-honored — enum defined with all 6 variants + serde snake_case + Default::Acknowledged. **Transition function deferred to CH-10** (D-new-05) per locked Q2; only the type lands at this chunk |
| `permissions/06-multi-scope-consent.md` | § "Edges" line 366 | `Agent ──HAS_CONSENT──▶ Consent` + `Consent ──SCOPED_TO──▶ Organization` | partially-honored (existing `subordinate: AgentId` + `scoped_to: OrgId` already encode the relationships) | partially-honored — same encoding via the new `agent_id` + `scope.org` fields. Typed edges are a separate concern (out of scope at this chunk) |

The three concept-doc sections collectively define CH-09's spec. The chunk doesn't *modify* the concept doc — it lifts the Consent shape into Rust 1:1.

---

## §3 — phi-core leverage map

| phi-core type | Action in this chunk |
|---|---|
| (none) | — |

Consent is baby-phi-native. phi-core has no concept of consent records, supervisor approval, or organizational consent policy (it's the agent-loop library; consent gating lives one layer up, with the permission-check engine). Zero phi-core imports added or removed.

**Positive close-audit greps**:
```bash
grep -n "pub struct Consent\b" modules/crates/domain/src/model/nodes.rs                 # 1
grep -n "pub struct ConsentScope\b" modules/crates/domain/src/model/nodes.rs           # 1
grep -n "pub enum ConsentState\b" modules/crates/domain/src/model/nodes.rs             # 1
grep -c "^    [A-Z][a-zA-Z]*,$" modules/crates/domain/src/model/nodes.rs | head -1     # ConsentState has 6 variants (asserted via test, not grep)
ls modules/crates/store/migrations/0010_*.surql                                         # exists
grep -c "DEFINE FIELD" modules/crates/store/migrations/0010_*.surql                     # ≥ 9 (one per new/changed field on consent table)
```

**Forbidden-duplication / regression greps**:
```bash
grep -rn "use phi_core::" modules/crates/domain/src/model/nodes.rs | grep -i consent    # 0 (no phi-core overlap on Consent)
grep -rn 'pub subordinate: AgentId' modules/crates/                                     # 0 (renamed to agent_id)
grep -rn 'pub scoped_to: OrgId' modules/crates/                                         # 0 (moved into nested scope)
grep -rn 'pub granted_at: DateTime' modules/crates/ | grep -i consent                   # 0 (renamed to responded_at)
```

---

## §3.B — K8s readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | None. Struct shape change only. | No |
| **A2** IPC channels | The Consent wire format changes (new fields, renamed fields). No production producer/consumer exists today, so this is the inaugural wire format. | No (no consumers yet) |
| **A3** pod-local resources | None. | No |
| **A4** migration runner | New migration 0010 (`0010_consent_full_shape.surql`) redefines the consent table. Migration is idempotent (per existing `_migrations` ledger pattern). Empty production tables make the redefine zero-risk. | No |
| **A5** trait-shape requirement | `Repository::create_consent` signature unchanged (still takes `&Consent`). | No |
| **A6** cross-pod state sharing | Consent rows would gossip identically post-migration. Pre-CH-09 rows would deserialize cleanly via `#[serde(default)]` on new fields IF any existed; since none do, no real concern. | No |
| **A7** audit hash-chain symmetry | No new audit-event variant. The eventual CH-10 state-machine transitions WILL emit audit events (e.g., `consent.acknowledged`, `consent.revoked`); CH-09 ships the data shape only, no event emission. The hash chain stays untouched. | No |

**Conclusion:** K8s-neutral. No M7b ledger entry added.

---

## §3.C — User-facing documentation impact

| Tier | File | Action |
|---|---|---|
| Concept | [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | Verified-header bump noting CH-09 lifts the §"Consent Node" 11-field spec into typed Rust at `domain::model::nodes::Consent` + `ConsentScope` + `ConsentState`. State-machine transitions remain CH-10. Doc body UNCHANGED. |
| Decision | `m5_2/decisions/0045-consent-node-full-shape.md` (NEW) | Full ADR — see §5. |
| Architecture | (none) | Consent struct lives in the model layer; no architecture-doc bump needed. The relevant architecture doc (`m1/architecture/graph-model.md`) is a high-level overview that already lists Consent as a node type without enumerating fields. |

2 file touches (one new ADR, one concept-doc header bump). No ops doc, no user-guide doc.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition |
|---|---|---|---|
| **D-new-04** | [`m5_1/drifts/D-new-04.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-04.md) | HIGH | `discovered → in-chunk-plan → remediated` |

D-new-05 (Consent state machine) **stays open** at this chunk — CH-09 ships the enum type + the `state` field, but CH-10 owns the transition function + `update_consent_state` repo method. The drift's lifecycle entry will be added by CH-10.

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — D-new-04 row Status flipped to `remediated`; "Closes at" → `CH-09 ✓`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip row for "Consent node 11 fields" from `contradicted` to `honored`.

---

## §5 — ADR drafted

ADR numbering: highest issued = ADR-0044 (CH-05). Next-free = **ADR-0045**.

| ADR | Title | Decision summary |
|---|---|---|
| **ADR-0045** | Consent node full shape (11 fields + ConsentScope + ConsentState enum) | **D45.1** Consent struct rewritten at `modules/crates/domain/src/model/nodes.rs` with 11 fields matching `concepts/permissions/06-multi-scope-consent.md` lines 351–363 verbatim: `id: ConsentId`, `agent_id: AgentId`, `scope: ConsentScope`, `state: ConsentState`, `requested_at: DateTime<Utc>`, `responded_at: Option<DateTime<Utc>>`, `revoked_at: Option<DateTime<Utc>>`, `revocable: bool`, `provenance: String`. Derives `Debug + Clone + Serialize + Deserialize`. **D45.2** New nested struct `ConsentScope` at the same module: `org: OrgId`, `templates: Vec<TemplateId>`, `actions: Vec<crate::permissions::Action>`. Cross-layer dep on `Action` from `permissions::action` is intentional and consistent with `Grant.action` / `ToolAuthorityManifest.actions` precedent. **D45.3** New `ConsentState` enum with 6 variants matching concept doc 06 §"Consent Lifecycle" lines 397–404: `Requested`, `Acknowledged`, `Declined`, `Revoked`, `TimedOut`, `Expired`. Derives `Debug + Clone + Copy + PartialEq + Eq + Hash + Serialize + Deserialize`. `#[serde(rename_all = "snake_case")]` for wire form. **D45.4** Implements `Default for ConsentState` returning `Acknowledged` — back-compat for serde rows that don't carry the field (none exist in production today; future-proofing). The default also matches the `implicit` consent policy semantic per concept doc line 410 ("Consent is auto-Acknowledged at agent creation"). **D45.5** New migration `modules/crates/store/migrations/0010_consent_full_shape.surql` redefines the consent table cleanly: `REMOVE TABLE IF EXISTS consent` then `DEFINE TABLE consent SCHEMAFULL` with all 11 fields. Safe because there is no production data (only test fixtures construct Consent today). The `_migrations` ledger applies it once at startup per the existing forward-only idempotent runner pattern (per ADR-0042 §D42.3 conforming-criteria #5). **D45.6** Field-rename rationale: the existing struct's `subordinate: AgentId` becomes `agent_id: AgentId` (concept-doc-verbatim), `scoped_to: OrgId` flattens into `scope.org: OrgId`, `granted_at: DateTime<Utc>` becomes `responded_at: Option<DateTime<Utc>>` with semantic shift (the timestamp is now nullable and set when the subordinate Acknowledges or Declines, not at record creation). Test fixture at `store/tests/repository_test.rs:359` updates accordingly. Per locked Q1 (2026-04-29). **D45.7** Repository trait surface unchanged: only `create_consent(&Consent) -> RepositoryResult<()>`. Per locked Q3 (2026-04-29) — CH-10 adds `update_consent_state`; CH-11 adds read methods. **D45.8** `ConsentIndex` projection unchanged at this chunk — still consumes `(AgentId, OrgId)` pairs. CH-11 evolves it to include scope + state filtering once the per-session gating logic lands. **D45.9** Out of scope at this chunk: per locked Q2 (2026-04-29), `ConsentState` ships as a TYPE only — CH-10 ships the transition function + lifecycle invariants (forward-only revocation, timeout handling, Requested → Acknowledged paths). The audit-event emission for consent transitions is also CH-10's job. **D45.10** `provenance: String` stays untyped at v0 — the concept doc shows examples like `"agent:claude-coder-9@onboarding"` which are conventional audit labels, not a structured type. A typed Provenance enum is deferred to v1 (would bundle with a broader audit-event schema overhaul). The string field is sufficient to record the trail per concept doc line 363. |

ADR file: [`m5_2/decisions/0045-consent-node-full-shape.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0045-consent-node-full-shape.md) (NEW).

---

## §6 — Prior-chunk regression re-verification

| Upstream | Invariant | Verification |
|---|---|---|
| Post-CH-05 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1187; 4 CI guards green; clippy clean under `-Dwarnings` | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh`<br>`cargo test -j 4 --workspace -- --test-threads=1` |
| CH-04 / ADR-0043 | `Action` enum reachable from `crate::permissions::Action` for use in `ConsentScope.actions: Vec<Action>` | New ConsentScope construction sites import + use `Action`; matrix tests still green |
| CH-05 / ADR-0044 | Manifest validator + Repository guard untouched | `cargo test -j 4 -p server --test acceptance_manifest_validator -- --test-threads=1` |
| CH-21 / ADR-0040 + 0041 | Audit hash chain byte-stable | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |
| Migration runner | Migration 0010 applies once + is safe to re-run | `cargo test -j 4 -p store -- --test-threads=1` (existing migration tests cover idempotency) |

The Consent struct change must not affect any audit-event canonical bytes (the BLAKE3 chain depends on byte-stable serialization). No audit events touch Consent today; CH-10 is what introduces consent-related audit events. The hash chain stays untouched at CH-09.

---

## §7 — Phases

**Phase count: 3** → audit envelope = **2 agents** (medium chunk).

### P1 — Land Consent + ConsentScope + ConsentState + migration 0010 (~0.5d)

**Goal.** Ship the new struct shape + supporting types + migration. End-state: workspace builds + all existing tests pass + the one Consent test fixture is updated to construct the new shape.

**Deliverables.**

1. **Replace `Consent` struct** at [`modules/crates/domain/src/model/nodes.rs:759`](baby-phi/modules/crates/domain/src/model/nodes.rs):
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Consent {
       pub id: ConsentId,
       pub agent_id: AgentId,
       pub scope: ConsentScope,
       pub state: ConsentState,
       pub requested_at: DateTime<Utc>,
       #[serde(default)]
       pub responded_at: Option<DateTime<Utc>>,
       #[serde(default)]
       pub revoked_at: Option<DateTime<Utc>>,
       pub revocable: bool,
       pub provenance: String,
   }
   ```

2. **New nested struct** `ConsentScope` at the same module:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ConsentScope {
       pub org: OrgId,
       #[serde(default)]
       pub templates: Vec<TemplateId>,
       #[serde(default)]
       pub actions: Vec<crate::permissions::Action>,
   }
   ```

3. **New `ConsentState` enum** at the same module:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
   #[serde(rename_all = "snake_case")]
   pub enum ConsentState {
       Requested,
       #[default]
       Acknowledged,
       Declined,
       Revoked,
       TimedOut,
       Expired,
   }
   ```

4. **New migration** at [`modules/crates/store/migrations/0010_consent_full_shape.surql`](baby-phi/modules/crates/store/migrations/0010_consent_full_shape.surql):
   - `REMOVE TABLE IF EXISTS consent;`
   - `DEFINE TABLE consent SCHEMAFULL;`
   - `DEFINE FIELD agent_id ON consent TYPE string ASSERT $value != NONE;`
   - `DEFINE FIELD scope ON consent FLEXIBLE TYPE object ASSERT $value != NONE;`
   - `DEFINE FIELD state ON consent TYPE string DEFAULT "acknowledged";`
   - `DEFINE FIELD requested_at ON consent TYPE string ASSERT $value != NONE;`
   - `DEFINE FIELD responded_at ON consent TYPE option<string>;`
   - `DEFINE FIELD revoked_at ON consent TYPE option<string>;`
   - `DEFINE FIELD revocable ON consent TYPE bool DEFAULT true;`
   - `DEFINE FIELD provenance ON consent TYPE string DEFAULT "";`

5. **Update existing test fixture** at [`modules/crates/store/tests/repository_test.rs:359`](baby-phi/modules/crates/store/tests/repository_test.rs) — `create_consent_persists_row_with_subordinate_and_org` test. Construct the new 11-field shape using sensible defaults: `agent_id: AgentId::new()`, `scope: ConsentScope { org: OrgId::new(), templates: vec![], actions: vec![] }`, `state: ConsentState::Acknowledged`, `requested_at: Utc::now()`, `responded_at: Some(Utc::now())`, `revoked_at: None`, `revocable: true`, `provenance: "test:fixture".into()`. Rename the test if needed to reflect the new field names.

6. **In-memory repo unchanged** — `InMemoryRepository::create_consent` already takes `&Consent` and inserts into `HashMap`; the new struct shape passes through transparently.

7. **SurrealDB repo unchanged** — `SurrealStore::create_consent` serializes via `serde_json::to_value` and writes via `CREATE type::thing(...)`. The new fields serialize naturally; the migration ensures the table accepts them.

**Tests.** ~8 unit tests in `nodes::tests` (or a new `consent_tests` module):
- `consent_struct_has_eleven_fields` — sanity check via Rust's reflective struct tests (or a const-asserted field count via macro).
- `consent_state_has_six_variants` — `ConsentState::Requested..Expired` enumerated.
- `consent_state_default_is_acknowledged` — pins the back-compat default.
- `consent_scope_serde_round_trip` — a populated `ConsentScope` survives JSON round-trip.
- `consent_full_serde_round_trip` — full Consent struct round-trips.
- `consent_state_serializes_as_snake_case` — `Acknowledged` → `"acknowledged"`, `TimedOut` → `"timed_out"`.
- `consent_state_legacy_string_round_trip` — pre-CH-09 wire format (no `state` field) deserializes with `state = Acknowledged` via the `Default` derive.
- `consent_with_action_in_scope_serde` — `ConsentScope.actions` carries `Action::Read`, round-trips as `"read"`.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- The migration runner errors on `REMOVE TABLE IF EXISTS consent` because of an unanticipated foreign-key edge from another table to consent. (Should not — current edges are `Agent ──HAS_CONSENT──▶ Consent` and `Consent ──SCOPED_TO──▶ Organization`, both source-side, not pointing INTO consent.)
- Any other test (beyond the one repository_test.rs:359 fixture) constructs a Consent record and breaks. The grep confirmed only one callsite, but if more surface, list them and resolve before proceeding.
- The `#[default]` attribute on the `Acknowledged` variant of `ConsentState` doesn't compile under the workspace's serde version. (Would require `#[derive(Default)]` to use `#[default]` — verify before relying on it.)

---

### P2 — Workspace tests + acceptance test for new shape (~0.2d)

**Goal.** Confirm zero regressions across the workspace + add one acceptance test for the new shape.

**Deliverables.**

1. **Workspace test pass** — `cargo test -j 4 --workspace -- --test-threads=1` should pass at ~1195 (1187 baseline + 8 new unit tests).
2. **One acceptance test** at [`modules/crates/server/tests/acceptance_consent_node_shape.rs`](baby-phi/modules/crates/server/tests/acceptance_consent_node_shape.rs) (NEW): construct a full 11-field Consent record, persist via `Repository::create_consent` against `InMemoryRepository`, confirm successful persistence + that subsequent operations on the same id work (since today's trait has no `get_consent`, the test confirms the create-path doesn't error). Cross-impl: same with `SurrealStore::open_embedded` + tempfile.
3. **fmt + clippy clean** under `RUSTFLAGS="-Dwarnings"`.

**Tests.** ~8 unit (P1) + ~2 acceptance (P2) = ~10 new tests. Workspace total: ~1187 + 10 = **~1197**.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- Any pre-existing test regresses unexpectedly (P1 only changed Consent shape; widening regressions would indicate a leak).
- The migration 0010 fails on first run against a fresh embedded SurrealDB.

---

### P3 — ADR Accepted + drift closed + concept-doc bump + audit + seal (~0.3d)

**Goal.** Ratify ADR-0045. Close D-new-04. Spawn 2 audit agents. Seal.

**Deliverables.**

1. ADR-0045 flipped from `Proposed` → `Accepted` at chunk seal.
2. D-new-04 Status flipped to `remediated`. Lifecycle entry appended:
   - `2026-04-XX — remediated — CH-09 chunk-seal — Consent struct extended to 11 concept-doc-mandated fields. ConsentScope nested struct + ConsentState enum (6 variants) shipped with serde snake_case wire form. Migration 0010 redefines the consent table cleanly. Test fixture updated. State-machine transitions remain CH-10 (D-new-05).`
3. `drifts/README.md` row flipped + `Closes at` column updated.
4. `_concept-audit-matrix.md` row flipped from `contradicted` to `honored`.
5. Concept doc `permissions/06-multi-scope-consent.md`: verified-header bump (CH-09 amendment line). Doc body UNCHANGED.
6. Spawn 2 audit agents per §11.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit reports a finding.

---

## §8 — Tests summary

- **Expected total at chunk close:** post-CH-05 baseline (1187) + ~8 P1 unit tests + ~2 P2 acceptance tests = **~1197 tests**.
- **New test files:** unit tests live in `nodes::tests` (or a `consent_tests` sub-module); acceptance test at `server/tests/acceptance_consent_node_shape.rs`.
- **Wire-format checks:** `consent_state_serializes_as_snake_case` + `consent_state_legacy_string_round_trip` pin the serde shape.
- **Cross-impl consistency:** Acceptance test runs the same Consent against both `InMemoryRepository` and `SurrealStore` (via `open_embedded`).
- **Migration safety:** Migration 0010 idempotency is covered by the existing `_migrations` ledger pattern; no new test needed since the runner is already proptest-asserted.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4`.
2. Copy plan verbatim to `baby-phi/docs/specs/plan/build/<8hex>-ch-09-consent-node-full-shape.md`.
3. Update placeholders in lines 4–5 of the archived copy.
4. `bash scripts/check-doc-links.sh`.

### Reading list (mandatory)

1. [`concepts/permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) — § "Consent Node (New Node Type)" (lines 349–367) + § "Consent Lifecycle" (lines 369–414) + § "Three Consent Policies" (lines 232–245) for context.
2. [`drifts/D-new-04.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-04.md) — full.
3. [`drifts/D-new-05.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-05.md) — for context on what CH-10 owns (state machine).
4. [`modules/crates/domain/src/model/nodes.rs`](baby-phi/modules/crates/domain/src/model/nodes.rs) lines 757–765 — current Consent struct.
5. [`modules/crates/domain/src/model/composites_m3.rs`](baby-phi/modules/crates/domain/src/model/composites_m3.rs) lines 43–47 — existing `ConsentPolicy` enum (Implicit, OneTime, PerSession) for pattern reference.
6. [`modules/crates/domain/src/permissions/manifest/mod.rs`](baby-phi/modules/crates/domain/src/permissions/manifest/mod.rs) lines 169–191 — `ConsentIndex` (the read-side projection that stays unchanged at this chunk).
7. [`modules/crates/store/migrations/0009_identity_node.surql`](baby-phi/modules/crates/store/migrations/0009_identity_node.surql) — most recent migration, pattern reference.
8. [`modules/crates/store/tests/repository_test.rs`](baby-phi/modules/crates/store/tests/repository_test.rs) lines 350–380 — the one existing Consent test fixture.
9. [ADR-0044](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0044-publish-time-manifest-validator.md) — most recent ADR for header/format precedent.
10. [ADR-0042](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) §D42.3 — the conforming-backend criteria (migration #5) the new migration satisfies.

### Carry-forward invariants (verified at chunk-open)

- `cargo test --workspace -- --test-threads=1` ≈ 1187.
- 4 CI guards green.
- D-new-04 status `discovered`; D-new-05 status `discovered` (stays open).
- ADR-0034..0044 Accepted; next-free = 0045.
- `git diff --stat HEAD -- modules/` empty (or tracking only intentional pending work).

---

## §10 — Close criteria (5-aspect)

- **Code aspect** — workspace builds; clippy under `RUSTFLAGS="-Dwarnings"` clean; `cargo test --workspace -- --test-threads=1` green at ~1197; no callsite still uses `subordinate` / `scoped_to` / `granted_at` field names on Consent.
- **Docs aspect** — D-new-04 lifecycle remediated; concept-audit matrix row flipped honored; ADR-0045 Accepted; concept-doc verified-header bumped.
- **phi-core leverage** — import-count delta = 0; positive/forbidden greps all match expected.
- **Concept alignment** — every §2 row at target status (D-new-05 row stays partially-honored per locked Q2).
- **K8s readiness** — neutral; ledger unchanged.

**Implementation confidence** = `claims-honored / claims-in-scope` = target **8/8**:
1. `Consent` struct has 11 fields matching concept doc verbatim.
2. `ConsentScope { org, templates, actions }` nested struct exists.
3. `ConsentState` enum has 6 variants + serde snake_case + Default::Acknowledged.
4. Migration 0010 redefines the consent table cleanly.
5. Test fixture at `repository_test.rs:359` updated for new shape.
6. Repository trait surface unchanged (`create_consent` only).
7. `ConsentIndex` projection unchanged.
8. Acceptance test pins cross-impl behaviour for the new shape.

---

## §11 — Audit plan

**2 agents** (medium chunk per per-chunk-template).

### Audit A — Code correctness + phi-core leverage

> You are auditing CH-09 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan: `docs/specs/plan/build/<8hex>-ch-09-consent-node-full-shape.md`.
>
> 1. `Consent` struct at `modules/crates/domain/src/model/nodes.rs` has exactly 11 fields matching concept doc 06 lines 351–363: `id`, `agent_id`, `scope`, `state`, `requested_at`, `responded_at`, `revoked_at`, `revocable`, `provenance` (`scope` is nested = 3 logical sub-fields, total 11 leaf fields per concept doc table).
> 2. `ConsentScope` struct exists at the same module with fields `org: OrgId`, `templates: Vec<TemplateId>`, `actions: Vec<crate::permissions::Action>`.
> 3. `ConsentState` enum has exactly 6 variants: `Requested`, `Acknowledged`, `Declined`, `Revoked`, `TimedOut`, `Expired`.
> 4. `ConsentState` derives `Debug + Clone + Copy + PartialEq + Eq + Hash + Serialize + Deserialize + Default`. `#[serde(rename_all = "snake_case")]`. `#[default]` on `Acknowledged`.
> 5. Field renames complete: `subordinate` → `agent_id`; `scoped_to: OrgId` flattened into `scope.org`; `granted_at` → `responded_at: Option<DateTime<Utc>>`. Negative greps (`pub subordinate: AgentId` / `pub scoped_to: OrgId` / `pub granted_at: DateTime` in consent context) all return 0.
> 6. Migration `modules/crates/store/migrations/0010_consent_full_shape.surql` exists with `REMOVE TABLE IF EXISTS consent` + `DEFINE TABLE consent SCHEMAFULL` + `DEFINE FIELD` for each of the 9 leaf fields.
> 7. Test fixture at `modules/crates/store/tests/repository_test.rs` is updated to construct the new 11-field shape; the test name may have been updated to reflect the new field names.
> 8. New unit tests in `nodes::tests` (or a `consent_tests` sub-module): ≥ 8 tests covering struct shape, ConsentState variants/default, serde round-trip (full + ConsentScope-only + state snake_case + legacy default), Action import.
> 9. New acceptance test `modules/crates/server/tests/acceptance_consent_node_shape.rs` exists with cross-impl coverage (InMemoryRepository + SurrealStore::open_embedded).
> 10. `cargo test --workspace -- --test-threads=1` green at ~1197.
> 11. CI guards green; `check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports.
> 12. CH-04 + CH-05 invariants intact: `Action::applies_to` matrix tests still green; `acceptance_manifest_validator` still green; CH-21 `acceptance_memory_extraction` still green (audit hash chain byte-stable).
> 13. `Repository::create_consent` signature unchanged; both `SurrealStore` + `InMemoryRepository` impls compile + persist the new shape correctly.

PASS/FAIL each. ≤ 600 words.

### Audit B — Concept fidelity + docs fidelity

> You are auditing CH-09's concept-fidelity + docs-fidelity. Read-only.
>
> 1. ADR-0045 Accepted at `m5_2/decisions/0045-consent-node-full-shape.md` with sub-decisions D45.1–D45.10.
> 2. ADR-0045 Status field reads exactly `**Status: Accepted**` (one line, bold).
> 3. ADR-0045 documents the 3 locked forks: Q1 (concept-doc-verbatim field renames), Q2 (ConsentState enum ships at CH-09; transitions deferred to CH-10), Q3 (Repository surface stays minimal — only create_consent).
> 4. ADR-0045 cross-references concept doc 06 (lines 351–414), drift D-new-04 (closed) + D-new-05 (stays open), ADR-0043 (Action enum dependency), ADR-0042 §D42.3 (migration runner conforming criteria), and downstream chunks CH-10 + CH-11.
> 5. D-new-04 Status = `remediated`; lifecycle entry for CH-09 chunk-seal present.
> 6. D-new-05 Status = `discovered` (UNCHANGED — CH-10 owns it).
> 7. `drifts/README.md` row for D-new-04 flipped; "Closes at" → CH-09 ✓.
> 8. `_concept-audit-matrix.md` row for "Consent node 11 fields" flipped contradicted → honored.
> 9. Concept doc `permissions/06-multi-scope-consent.md` verified-header bumped (CH-09 amendment line). Doc body UNCHANGED.
> 10. Plan archive at `plan/build/<8hex>-ch-09-...md`.
> 11. CH-04 invariants intact: ADR-0043 still Accepted; D-new-09 + D-new-10 still remediated.
> 12. CH-05 invariants intact: ADR-0044 still Accepted; D-new-07 + D-new-31 still remediated.
> 13. CH-21 + CH-22 + CH-16 invariants intact (file-existence smoke test).

PASS/FAIL each. ≤ 600 words.

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1197 passed / 0 failed

# 3. Positive greps
grep -n "pub struct Consent\b" modules/crates/domain/src/model/nodes.rs                  # 1
grep -n "pub struct ConsentScope\b" modules/crates/domain/src/model/nodes.rs            # 1
grep -n "pub enum ConsentState\b" modules/crates/domain/src/model/nodes.rs              # 1
grep -n "pub agent_id: AgentId" modules/crates/domain/src/model/nodes.rs                # ≥ 1 (new Consent.agent_id field)
ls modules/crates/store/migrations/0010_consent_full_shape.surql                        # exists
grep -c "DEFINE FIELD" modules/crates/store/migrations/0010_consent_full_shape.surql    # ≥ 9
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0045-consent-node-full-shape.md  # 1

# 4. Negative greps (no leftover old field names)
grep -rn 'pub subordinate: AgentId' modules/crates/                                     # 0
grep -rn 'pub scoped_to: OrgId' modules/crates/                                         # 0
grep -rn 'pub granted_at: DateTime' modules/crates/ | grep -i consent                   # 0
grep -rn 'use phi_core::' modules/crates/domain/src/model/nodes.rs | grep -i consent    # 0

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-04.md  # 1
grep -c '^- \*\*Status\*\*: `discovered`' docs/specs/v0/implementation/m5_1/drifts/D-new-05.md  # 1 (UNCHANGED)

# 6. Consent unit tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib model::nodes::tests
# Expect: existing tests + ~8 new consent tests pass.

# 7. Acceptance suite for new shape
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_consent_node_shape -- --test-threads=1
# Expect: ~2 tests (in-memory + cross-impl with SurrealStore).

# 8. Carry-forward sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_manifest_validator -- --test-threads=1   # CH-05 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::action::tests                          # CH-04 matrix still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1   # CH-21 still green
```

---

## What this plan does NOT do

- Doesn't ship the consent state-machine transition function (CH-10).
- Doesn't add `update_consent_state` / `get_consent` / `list_consents_for_subordinate` repo methods (CH-10 / CH-11).
- Doesn't change the Permission Check engine's `ConsentIndex` (CH-11).
- Doesn't add `ApprovalMode` enum on Grant (CH-11).
- Doesn't migrate existing production Consent data (none exists).
- Doesn't emit consent-related audit events (CH-10).
- Doesn't type the `provenance` field (deferred to v1).

---

## Critical files

**New:**
- `modules/crates/store/migrations/0010_consent_full_shape.surql` — table redefine
- `modules/crates/server/tests/acceptance_consent_node_shape.rs` — cross-impl persistence test
- `docs/specs/v0/implementation/m5_2/decisions/0045-consent-node-full-shape.md` — ADR

**Modified (light):**
- `modules/crates/domain/src/model/nodes.rs` — Consent struct rewrite + ConsentScope + ConsentState
- `modules/crates/store/tests/repository_test.rs:359` — update existing test fixture
- Drift files: `D-new-04.md`, `drifts/README.md`, `_concept-audit-matrix.md`
- Concept doc: `permissions/06-multi-scope-consent.md` — header bump only

**Unchanged (verified by close-audit):**
- `modules/crates/domain/src/repository.rs` — `create_consent` signature unchanged
- `modules/crates/store/src/repo_impl.rs` — `create_consent` impl unchanged (serde handles new fields transparently)
- `modules/crates/domain/src/in_memory.rs` — `create_consent` impl unchanged
- `modules/crates/domain/src/permissions/manifest/mod.rs` — `ConsentIndex` unchanged

---

## Estimated effort

~1 engineer-day:
- 0.5d — P1 struct rewrite + ConsentScope + ConsentState + migration 0010 + test fixture update + ~8 unit tests.
- 0.2d — P2 ~2 acceptance tests + workspace clippy/test green.
- 0.3d — P3 ADR Accepted + drift closure + concept-doc header bump + matrix flip + 2 audit agents + seal.
