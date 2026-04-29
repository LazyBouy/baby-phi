<!-- Last verified: 2026-04-28 by Claude Code -->

# CH-04 — Typed action vocabulary + Action × Fundamental matrix

**Plan file token:** `3a65a2fc` (generated via `openssl rand -hex 4` at chunk-open 2026-04-29).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/3a65a2fc-ch-04-typed-action-vocabulary.md`.
**Chunk ID:** CH-04 (forward-scope §1 lines 63–68; §5 inventory row line 412).
**Severity:** MED.
**Expected effort:** ~2.5 engineer-days.
**Hard prerequisites:** none.
**Chunks unblocked at close:** CH-05 (publish-time validator), CH-08 (allocate/transfer cardinality), CH-15 (real permission manifest at session launch).

---

## Context

### The simple version

Today, when the code talks about *what an agent is allowed to do* — read a file, send a message, allocate a resource — it stores those words as plain strings: `vec!["read", "inspect", "list"]`. That's it. There's no list of valid actions, no compiler check, no enforcement of "this action doesn't apply to this resource type". Anything you type goes.

CH-04 fixes this by introducing a Rust enum. The 33 actions named in the concept doc become 33 enum variants (plus one `Wildcard` for the existing `"*"` escape hatch — total 34). Once that lands:

- A typo (`raed` instead of `read`) won't compile.
- The grant `vec![Action::Read, Action::Inspect]` is type-safe.
- A new compile-time matrix says *"`recall` only applies to memory; you can't use it on a filesystem"* — and the validator can reject grants that violate it before they ever hit the database.

That's the chunk. Add the enum, add the matrix, migrate the ~51 places in the code that build action lists, update the tests. Three days of code work, all behind a stable wire format (action strings still serialize the same way — operators see no difference).

### What this chunk does NOT do

- Does NOT change SurrealDB schema. Wire format stays `Vec<String>` over JSON; the Rust type just becomes `Vec<Action>` with a serde rename to snake_case.
- Does NOT add new actions to the vocabulary. The 33 verbs come from the concept doc, untouched.
- Does NOT change Permission Check engine semantics. The check still does the same job; it just operates on typed values now.
- Does NOT enforce the matrix at every callsite — only at the publish-time validator (which actually exists at CH-05). At CH-04 the matrix is a queryable function `Action::applies_to(Fundamental) -> bool`, ready to use; CH-05 wires it into the validator.
- Does NOT introduce per-resource action enums (the more rigorous design — `enum FilesystemObjectAction`, `enum MemoryObjectAction`, …). That alternative gives compile-time enforcement of "this action is valid for this resource type" but requires ~2-3× the engineering effort and restructuring of the concept doc into per-resource sections. **Deferred to v1** with explicit notes added in both `concepts/permissions/03-action-vocabulary.md` § "Future work" and ADR-0043 §D43.8. CH-04's flat-enum + matrix design matches the concept doc's current structure 1:1 and is sufficient for v0.1.

### Why expand from "just rename strings"?

Two reasons:

1. **The concept doc already specifies a closed set of 33 actions and an applicability matrix.** Today, that's documentation; CH-04 makes it executable Rust. The drift D-new-09 calls this out.
2. **Three downstream chunks are blocked.** CH-05's validator can't reject "unknown action" without a typed enum to compare against. CH-08's allocate/transfer cardinality rules can't pattern-match on string actions safely. CH-15's manifest typing depends on this.

Locking it down now means the next three chunks each save half a day of "first, we need the enum…" prologue.

---

## §1 — Why this chunk (one paragraph)

Action strings are the foot-gun version of a model that the concept doc already specifies as a closed set. Drift D-new-09 records the gap (no enum, no constants); D-new-10 records that the concept doc's 9-fundamental × 33-action applicability matrix isn't enforced anywhere. CH-04 closes both: introduces the `Action` enum + the applicability matrix as code, migrates the three carrier types (Grant, Manifest, ToolAuthorityManifest) and ~51 callsites to use it, and ships acceptance tests showing wire format is preserved end-to-end. ADR-0043 captures the design decisions (wildcard handling, the synthetic-action cleanup, atomic vs hybrid migration choice — all locked at plan-review).

### User-decided forks (locked at plan-review, 2026-04-28)

1. **Wildcard handling**: add `Action::Wildcard` as a 34th variant; serialized as `"*"` for wire compat. Permission Check engine matches it as "covers all actions". Documented in ADR-0043 as a privileged escape hatch.
2. **`launch_session` synthetic action**: replace with canonical `Action::Invoke`. The concept doc's 33-verb vocabulary stays unchanged; the one outlier callsite (`sessions/preview.rs:86`) updates to use the canonical verb.
3. **Migration scope**: atomic. Carriers become `Vec<Action>`; all ~51 callsites migrate in this chunk. Wire format preserved via `#[serde(rename_all = "snake_case")]` on the enum — `Action::Read` round-trips as `"read"`, identical to today.

### Forward-scope reference

[CH-04 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 63–68) + [§5 inventory](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 412).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Status at chunk-close |
|---|---|---|---|---|
| [`permissions/03-action-vocabulary.md`](baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md) | § "Standard Action Vocabulary" (lines 10–21) | 33 named verbs in 10 categories | silent-in-code (vocab is informally enforced via prose; no closed set exists) | honored — `Action` enum has 33 named variants matching the doc 1:1 |
| `permissions/03-action-vocabulary.md` | § "Action × Fundamental Matrix" (lines 27–37) | Each action applies to a defined subset of the 9 fundamentals | silent-in-code (matrix exists in the doc; not in code) | honored — `Action::applies_to(Fundamental) -> bool` encodes the matrix exactly |
| `permissions/03-action-vocabulary.md` | § "Composite Inheritance" (line 39) | Composite resources inherit the union of their constituents' actions | partial (composites have `constituents()` but no action-applicability check derived from it) | honored — `applies_to_composite(Composite) -> bool` derives via `Composite::constituents()` |
| `permissions/03-action-vocabulary.md` | § "`allocate` as Umbrella Action" (lines 48–54) | `allocate` is a single action covering all ownership-share semantics | honored (already a string `"allocate"`) | honored (now `Action::Allocate`) |

The vocabulary doc itself is the source of truth; CH-04 doesn't *modify* it, only wires it into code.

---

## §3 — phi-core leverage map

| phi-core type | Action in this chunk |
|---|---|
| (none) | — |

Permissions are baby-phi-native. phi-core has no concept of Grant, Manifest, or action vocabulary (it's the agent-loop library; permissions live one layer up). Zero phi-core imports added or removed.

**Positive close-audit greps**:
```bash
grep -n "pub enum Action\b" modules/crates/domain/src/permissions/action.rs           # 1 (the new enum)
grep -c "^    [A-Z][a-zA-Z]*,$" modules/crates/domain/src/permissions/action.rs       # ≥ 34 (33 + Wildcard)
grep -n "fn applies_to\b" modules/crates/domain/src/permissions/action.rs             # ≥ 1
grep -rn "use.*permissions::action::Action" modules/crates/                            # ≥ 10 (carrier callsites)
ls modules/crates/domain/src/permissions/action.rs                                     # exists
```

**Forbidden-duplication / regression greps**:
```bash
grep -rn 'vec!\["read"\|vec!\["allocate"\|vec!\["transfer"\|vec!\["modify"' modules/crates/ | wc -l   # 0 (all migrated to Action::*)
grep -rn '"launch_session"' modules/crates/                                                            # 0 (replaced with Action::Invoke)
grep -rn "use phi_core::" modules/crates/domain/src/permissions/action.rs                              # 0
```

---

## §3.B — K8s readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | None. Action is a value-type enum (`Copy + Eq + Hash`). | No |
| **A2** IPC channels | None. Wire format unchanged. | No |
| **A3** pod-local resources | None. | No |
| **A4** migration runner | No SurrealDB schema migration; field types stay JSON arrays of strings on the wire. | No |
| **A5** trait-shape requirement | The Permission Check engine's matching loop becomes typed; trait shape unchanged. | No |
| **A6** cross-pod state sharing | Wire format preserved → cross-pod gossip is byte-for-byte identical. | No |
| **A7** audit hash-chain symmetry | Audit events serialize Action via the same snake_case path; hash-chain bytes unchanged. | No |

**Conclusion:** K8s-neutral. No M7b ledger entry added.

---

## §3.C — User-facing documentation impact

| Tier | File | Action |
|---|---|---|
| Concept | [`permissions/03-action-vocabulary.md`](baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md) | (a) Verified-header bump noting CH-04 lifts the doc's 33-verb list + matrix into typed Rust. (b) **NEW § "Future work — v1 revisit: per-resource action enums"** subsection appended at the end of the doc. Captures the design tradeoff (current: flat enum + matrix; v1: per-resource enums for compile-time enforcement) + cross-references ADR-0043 §D43.8. ~150 words. |
| Decision | `m5_2/decisions/0043-typed-action-vocabulary.md` (NEW) | Full ADR — see §5 (now with §D43.8 capturing the v1 deferral). |
| Architecture | [`m1/architecture/permission-check-engine.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/permission-check-engine.md) | Light verified-header bump cross-referencing ADR-0043 (verified 2026-04-29: file exists; canonical name is `permission-check-engine.md`, not `permission-check.md`). |

3 file touches (one new, one concept-doc with both header bump + new subsection, one architecture-doc header bump). No ops doc, no user-guide doc.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition |
|---|---|---|---|
| **D-new-09** | [`m5_1/drifts/D-new-09.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-09.md) | HIGH | `discovered → in-chunk-plan → remediated` |
| **D-new-10** | [`m5_1/drifts/D-new-10.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-10.md) | MED | `discovered → in-chunk-plan → remediated` |

Both close together: D-new-09 ships the typed enum; D-new-10 ships the matrix that consumes it. Neither is meaningful without the other.

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — both row Statuses flipped to `remediated`; "Closes at" → `CH-04 ✓`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip rows for "Action vocabulary closed set" + "Action × Fundamental matrix" from `silent-in-code` to `honored`.

---

## §5 — ADR drafted

ADR numbering: highest issued = ADR-0042 (CH-03). Next-free = **ADR-0043**.

| ADR | Title | Decision summary |
|---|---|---|
| **ADR-0043** | Typed action vocabulary + applicability matrix | **D43.1** `Action` enum at `domain::permissions::action::Action` with 33 canonical variants matching `concepts/permissions/03-action-vocabulary.md` lines 10–21 verbatim, plus a 34th `Wildcard` variant for the existing `"*"` escape hatch. Derives `Copy + Clone + PartialEq + Eq + Hash + Serialize + Deserialize`. `#[serde(rename_all = "snake_case")]` ensures wire-format compatibility with current `Vec<String>` storage — `Action::Read` round-trips as `"read"`, `Action::Wildcard` as `"*"`. **D43.2** `Action::ALL: [Action; 34]` constant for exhaustive iteration. `Action::as_str(&self) -> &'static str` for canonical-string. `TryFrom<&str>` for parsing; rejects unknown verbs with `ParseActionError`. **D43.3** `Action::applies_to(Fundamental) -> bool` encodes the 9×10 applicability matrix from concept doc lines 27–37 verbatim. `Action::applies_to_composite(Composite) -> bool` derives via the composite's `constituents()` — composite applicability is the union of constituent applicability per concept-doc line 39. **D43.4** Storage migration: `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` change from `Vec<String>` to `Vec<Action>`. No SurrealDB schema migration; `#[serde(rename_all = "snake_case")]` preserves the on-disk JSON shape. All ~51 callsites migrate in CH-04 (atomic). **D43.5** Wildcard semantics: `Action::Wildcard` matches any required action in Permission Check engine's `covers()` + ceiling enforcement paths. Documented as a privileged escape hatch — only system:genesis / bootstrap-claim grants should use it. **D43.6** `launch_session` synthetic action (sessions/preview.rs) replaced with `Action::Invoke` at this chunk. The 33-verb concept vocabulary stays unchanged. **D43.7** Out of scope at this chunk: enforcement of the matrix at every callsite. CH-04 ships the matrix as a queryable function; CH-05's publish-time validator wires it in as a real rejection rule. CH-08 uses it for allocate/transfer cardinality. **D43.8** Deferred to v1: a per-resource action enum design (e.g., `enum FilesystemObjectAction`, `enum MemoryObjectAction`, …) where `Grant<R>` is generic over a `ResourceClass` trait with an associated `Action` type. This would give compile-time enforcement of "this action is valid for this resource type" (e.g., `Grant<FilesystemObject> { action: vec![Recall] }` would fail to compile rather than being caught by the publish-time validator). The flat-enum + matrix design shipped in v0.1 (this ADR) is sufficient for the threat model — operators load manifests through the validator, which catches the same class of bugs at effectively the same stage. Rationale for deferral: per-resource enums require restructuring the concept doc into per-resource sections, designing per-Composite enum unions (8 of them, derived from `constituents()`), reworking the `*` wildcard semantics, and changing the `Repository` trait surface to be generic — ~2-3× the engineering effort for marginal compile-time-vs-publish-time safety improvement. The concept doc grows a § "Future work — v1 revisit" subsection cross-referencing this sub-decision. |

ADR file: [`m5_2/decisions/0043-typed-action-vocabulary.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0043-typed-action-vocabulary.md) (NEW).

---

## §6 — Prior-chunk regression re-verification

| Upstream | Invariant | Verification |
|---|---|---|
| Post-CH-03 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1121 (post-CH-21); 4 CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh`<br>`cargo test -j 4 --workspace -- --test-threads=1` |
| CH-21 / ADR-0040 + 0041 | Memory-extraction listener + DomainEvent::MemoryExtracted; D6.1 remediated | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |
| CH-16 / ADR-0038 | Identity materialization | `cargo test -j 4 -p server --test identity_materialization_acceptance -- --test-threads=1` |
| CH-22 / ADR-0035 | Catalog listener body | `cargo test -j 4 -p domain --lib events::listeners::tests` (15+ catalog tests) |
| Wire format compat | Action strings round-trip identically before/after | New round-trip test in `permissions::action::tests` proves `serde_json::to_string(Action::Read) == "\"read\""` matches old `serde_json::to_string("read") == "\"read\""` |

CH-04 must not change any audit event's canonical bytes (the BLAKE3 chain depends on byte-stable serialization). Wire-format compatibility is the load-bearing invariant.

---

## §7 — Phases

**Phase count: 3** → audit envelope = **2 agents** (medium chunk).

### P1 — Land the `Action` enum + applicability matrix (no migration yet) (~0.7d)

**Goal.** Ship the new types as standalone code with full unit-test coverage. No callsite migration in this phase. End-state: `Action` enum exists, but Grant/Manifest/ToolAuthorityManifest still use `Vec<String>`. The new types are imported but not yet load-bearing.

**Deliverables.**

1. **New module** at [`modules/crates/domain/src/permissions/action.rs`](baby-phi/modules/crates/domain/src/permissions/action.rs) (NEW):
   - `pub enum Action` with 34 variants (33 canonical + `Wildcard`), `#[serde(rename_all = "snake_case")]`, custom serialization for `Wildcard` ↔ `"*"`.
   - `impl Action`: `ALL: [Action; 34]`, `as_str() -> &'static str`, `category() -> ActionCategory`, `applies_to(Fundamental) -> bool`, `applies_to_composite(Composite) -> bool`.
   - `pub enum ActionCategory` (10 variants — Discovery / Data / Mutation / Execution / Connection / Authority / Memory / Configuration / Economic / Observability) with `as_str()` + `actions()` returning the slice for the category.
   - `pub enum ParseActionError` for `TryFrom<&str>` failures (with the offending string + a hint listing valid alternatives in the same category).

2. **Wire-format compat test** (proves the migration is safe to flip on later):
   - `Action::Read` serializes to `"read"`.
   - `Action::Wildcard` serializes to `"*"`.
   - Round-trip through `serde_json` for every variant.
   - Round-trip through `Vec<String>` (old shape) → `Vec<Action>` (new shape) → `Vec<String>` produces identical output for any valid input.

3. **Matrix exhaustive test**:
   - For every `(Action, Fundamental)` pair in `Action::ALL × Fundamental::ALL`, assert that `Action::applies_to(F)` matches the concept doc's matrix (lines 27–37) cell-by-cell. 9×34 = 306 assertions, generated via a single proptest or a hand-written exhaustive loop.

4. **Module wiring**: add `pub mod action;` to `modules/crates/domain/src/permissions/mod.rs`. Re-export `Action`, `ActionCategory`, `ParseActionError` via `permissions::*`.

**Tests.** ~35 unit tests in `permissions::action::tests`:
- 1 `ALL.len() == 34`.
- 1 round-trip serde for each variant (34 tests, parameterized).
- 1 `TryFrom<&str>` for each canonical action + 2 negative cases (typo + empty string).
- 1 exhaustive matrix assertion (306 cells covered in one test).
- 1 composite inheritance test per Composite variant (8 tests).
- 1 wildcard `applies_to` returns `true` for all `(Fundamental, Composite)` pairs.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- Any of the 33 concept-doc verbs has ambiguous semantics making the enum-variant naming non-obvious.
- The matrix as documented contains internal contradictions (e.g., a fundamental row claims action X applies, but the action's category contradicts).
- An additional escape-hatch use of `Vec<String>` is discovered that doesn't fit the wildcard model.

---

### P2 — Migrate the 3 carrier types + ~51 callsites (~1.5d)

**Goal.** Flip `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` from `Vec<String>` to `Vec<Action>`. Migrate every callsite. End-state: nowhere in `modules/crates/` constructs an action with a string literal (except the test fixtures that explicitly probe parse-error cases).

**Deliverables (carrier-type changes).**

1. **`Grant.action`** at `modules/crates/domain/src/model/nodes.rs:627`: `Vec<String>` → `Vec<Action>`. The `#[serde(rename_all = "snake_case")]` carries through.
2. **`Manifest.actions`** at `modules/crates/domain/src/permissions/manifest.rs:38`: same change.
3. **`ToolAuthorityManifest.actions`** at `modules/crates/domain/src/model/nodes.rs:775`: same change.
4. **`Manifest::from_node`** at `modules/crates/domain/src/permissions/manifest.rs:85`: clones the typed `Vec<Action>` directly (no parsing needed since `ToolAuthorityManifest.actions` is now also `Vec<Action>`).

**Deliverables (read-side callsites).**

5. **`expansion.rs:106`** (`covers()` method): match against `Action::Wildcard` instead of string `"*"`. Replace `g.action.iter().any(|a| a == action || a == "*")` with `g.action.iter().any(|a| *a == action || *a == Action::Wildcard)`.
6. **`engine.rs:255`** (Step 2a ceiling enforcement): same wildcard replacement.
7. **`engine.rs:164`** (Step 1 reach build): no semantic change, just type adjustment.
8. **`decision.rs:164`** (resolved-grant mapping): `r.action.clone()` already works for `Vec<Action>`.
9. **`manifest.rs:97`** (`is_empty()`): no change needed.

**Deliverables (write-side callsites).**

10. **Templates** (`a.rs:103`, `c.rs:84`, `d.rs:80`): replace `vec!["read", "inspect", "list"]` (etc.) with `vec![Action::Read, Action::Inspect, Action::List]`.
11. **Server handlers**:
    - `secrets/add.rs:136`: `vec![Action::Read]`
    - `model_providers/register.rs:149`: `vec![Action::Invoke]`
    - `mcp_servers/register.rs:120`: `vec![Action::Invoke]`
    - `sessions/preview.rs:86`: `vec![Action::Invoke]` (per Fork 2 — `launch_session` → `invoke`)
    - `orgs/create.rs:181`: `vec![Action::Allocate]`
12. **Bootstrap / claim**:
    - `claim.rs:194` (AuthRequest scope): `vec![Action::Allocate]`
    - `claim.rs:233` (Grant action): `vec![Action::Allocate]`
13. **Auth Request transitions / revocation**:
    - `auth_requests/transitions.rs:330`: `vec![Action::Read]`
    - `auth_requests/revocation.rs:110`: `vec![Action::Read]`

**Deliverables (test fixtures).**

14. Migrate ~15–20 explicit assertions like `assert_eq!(grant.action, vec!["read"])` to `assert_eq!(grant.action, vec![Action::Read])` across:
    - `domain/tests/template_a_firing_props.rs:70`
    - `server/src/bootstrap/claim.rs:348` (in-module test)
    - `server/src/platform/secrets/reveal.rs:287`
    - All other `tests/` files that compare action vectors.

15. **Audit-event diffs** carrying action lists (e.g., `template.a.grant_fired` audit's `action` field): these are `serde_json::Value` constructions today; ensure they continue to serialize as `"read"`-style strings (verified by existing audit-event round-trip tests, which should keep passing without change because of `#[serde(rename_all = "snake_case")]`).

**Tests.** Existing tests should pass after migration. New tests:
- 1 acceptance-level test: end-to-end Permission Check with typed actions, exercising wildcard + matrix-violation rejection (deferred enforcement, but the matrix returns false correctly).
- 1 wire-format compatibility test: serialize a Grant with `Vec<Action>`, deserialize into a struct with `Vec<String>`, confirm bit-for-bit equivalence.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- A test failure surfaces a callsite the explore agents missed (e.g., a string literal action constructed via `format!()` rather than `vec![]`).
- An audit-event hash-chain test fails (would mean canonical bytes shifted — SHOULD NOT happen given snake_case serde, but if it does, this is a load-bearing failure that needs redesign).
- The `launch_session` → `invoke` swap surfaces a behavioural difference I didn't anticipate (e.g., an existing manifest in the resource catalogue blocks `invoke` differently from `launch_session`).

---

### P3 — ADR Accepted + drifts closed + audit + seal (~0.3d)

**Goal.** Ratify ADR-0043. Close D-new-09 + D-new-10. Spawn 2 audit agents. Seal.

**Deliverables.**

1. ADR-0043 flipped from `Proposed` → `Accepted`.
2. D-new-09 + D-new-10 Statuses flipped to `remediated`. Lifecycle entries appended:
   - D-new-09: `2026-04-XX — remediated — CH-04 chunk-seal — Action enum + 34 variants shipped; 51 callsites migrated; wire format preserved.`
   - D-new-10: `2026-04-XX — remediated — CH-04 chunk-seal — APPLICABILITY_MATRIX shipped as Action::applies_to(Fundamental). CH-05 wires into validator.`
3. `drifts/README.md` rows flipped + `Closes at` columns updated.
4. `_concept-audit-matrix.md` 2 rows flipped from `silent-in-code` to `honored`.
5. Concept-`permissions/03-action-vocabulary.md`:
   - Verified-header bump (CH-04 amendment line).
   - **Append new § "Future work — v1 revisit: per-resource action enums"** subsection. ~150 words. Documents the flat-enum + matrix design shipped in v0.1, the per-resource enum alternative considered + deferred, and the rationale (effort tradeoff, concept-doc restructuring needed). Cross-references ADR-0043 §D43.8.
6. Spawn 2 audit agents per §11.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit reports a finding.

---

## §8 — Tests summary

- **Expected total at chunk close:** post-CH-21 baseline (1121) + ~40 new tests = **~1161 tests**. (Net new from P1 ~35 + P2 ~5 acceptance/round-trip; existing tests migrate without count change.)
- **New test files:** none — tests live in `permissions::action::tests` (unit) + extensions to existing `tests/` integration files.
- **Migration touch points:** ~51 construction sites + ~15–20 assertion sites (per Explore B report).
- **Wire-format compat tests:** at least 2 (Vec<Action> ↔ Vec<String> bit-for-bit; per-variant round-trip).
- **Matrix exhaustive coverage:** 9 fundamentals × 34 actions = 306 cells, asserted in one parameterized test.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4`.
2. Copy plan verbatim to `baby-phi/docs/specs/plan/build/<8hex>-ch-04-typed-action-vocabulary.md`.
3. Update placeholders in lines 4–5 of the archived copy.
4. `bash scripts/check-doc-links.sh`.

### Reading list (mandatory)

1. [`concepts/permissions/03-action-vocabulary.md`](baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md) — full. The 33 verbs + the matrix come from here verbatim.
2. [`drifts/D-new-09.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-09.md) + [`D-new-10.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-10.md) — full.
3. [`modules/crates/domain/src/model/fundamentals.rs`](baby-phi/modules/crates/domain/src/model/fundamentals.rs) — for the matrix's column type.
4. [`modules/crates/domain/src/model/composites.rs`](baby-phi/modules/crates/domain/src/model/composites.rs) — for `constituents()` (powers `applies_to_composite`).
5. [`modules/crates/domain/src/permissions/engine.rs`](baby-phi/modules/crates/domain/src/permissions/engine.rs) lines 100–270 — Permission Check matching loop (callsite migration touchpoint).
6. [ADR-0042](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) — most recent ADR for header/format precedent.
7. Existing typed enum example at `nodes.rs:256–290` (`AgentRole`) — convention to mirror.

### Carry-forward invariants (verified at chunk-open)

- `cargo test --workspace -- --test-threads=1` ≈ 1121.
- 4 CI guards green.
- D-new-09 + D-new-10 status `discovered`.
- ADR-0034..0042 Accepted; next-free = 0043.
- `git diff --stat HEAD -- modules/` empty (or tracking only intentional pending work).

---

## §10 — Close criteria (5-aspect)

- **Code aspect** — workspace builds; clippy under `RUSTFLAGS="-Dwarnings"` clean; `cargo test --workspace -- --test-threads=1` green at ~1161; no `vec!["read"]`-style construction outside parse-error tests.
- **Docs aspect** — D-new-09 + D-new-10 lifecycle remediated; matrix flipped honored; ADR-0043 Accepted; concept doc verified-header bumped.
- **phi-core leverage** — import-count delta = 0; positive/forbidden greps all match expected.
- **Concept alignment** — every §2 row at target status.
- **K8s readiness** — neutral; ledger unchanged.

**Implementation confidence** = `claims-honored / claims-in-scope` = target **8/8**:
1. `Action` enum exists with 34 variants (33 canonical + Wildcard).
2. `Action::as_str` + `TryFrom<&str>` + `ALL` constant.
3. `Action::applies_to(Fundamental)` encodes the concept-doc matrix exactly.
4. `Action::applies_to_composite(Composite)` derives via `constituents()`.
5. `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` are `Vec<Action>`.
6. Wildcard semantics (`Action::Wildcard` matches any in `covers()` + ceiling).
7. `launch_session` callsite uses `Action::Invoke`.
8. Wire format byte-stable (audit hash chain unchanged).

---

## §11 — Audit plan

**2 agents** (medium chunk per per-chunk-template; matches CH-16 / CH-21 precedent).

### Audit A — Code correctness + phi-core leverage

> You are auditing CH-04 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan: `docs/specs/plan/build/<8hex>-ch-04-typed-action-vocabulary.md`.
>
> 1. `Action` enum at `modules/crates/domain/src/permissions/action.rs` has exactly 34 variants (33 canonical + Wildcard).
> 2. The 33 canonical variants match concept-doc `permissions/03-action-vocabulary.md` lines 10–21 verbatim (case-insensitive).
> 3. `#[serde(rename_all = "snake_case")]` on the enum.
> 4. `Action::ALL: [Action; 34]` constant exists.
> 5. `Action::as_str(&self) -> &'static str` returns canonical strings (read, inspect, …, *).
> 6. `TryFrom<&str> for Action` accepts canonical + rejects unknown with `ParseActionError`.
> 7. `Action::applies_to(Fundamental) -> bool` matches concept-doc matrix (lines 27–37) exhaustively. Test exists asserting all 9×34 = 306 cells.
> 8. `Action::applies_to_composite(Composite) -> bool` derives via `Composite::constituents()`.
> 9. `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` are `Vec<Action>`.
> 10. `expansion.rs::covers()` + `engine.rs` Step 2a use `Action::Wildcard` for `*` semantics.
> 11. `sessions/preview.rs` uses `Action::Invoke` (no `"launch_session"` string remaining).
> 12. `grep -rn 'vec!\["read"\|vec!\["allocate"\|vec!\["transfer"\|vec!\["modify"' modules/crates/` returns 0.
> 13. `cargo test --workspace -- --test-threads=1` green at ~1161 / 0 failed.
> 14. Wire-format compat: `serde_json::to_string(&Action::Read) == "\"read\""`; `serde_json::from_str::<Action>("\"read\"")` succeeds.
> 15. CI guards green; `check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports.
> 16. Audit-event hash-chain unchanged (run a CH-21 acceptance test; confirm canonical bytes match pre-CH-04 baseline).

PASS/FAIL each. ≤ 600 words.

### Audit B — Concept fidelity + docs fidelity

> You are auditing CH-04's concept-fidelity + docs-fidelity. Read-only.
>
> 1. ADR-0043 Accepted at `m5_2/decisions/0043-typed-action-vocabulary.md` with sub-decisions D43.1–D43.7.
> 2. ADR-0043 documents wildcard handling, launch_session→invoke swap, atomic migration choice (matches plan §1 forks 1+2+3).
> 3. D-new-09 Status = `remediated`; lifecycle entry present.
> 4. D-new-10 Status = `remediated`; lifecycle entry present.
> 5. `drifts/README.md` rows for D-new-09 + D-new-10 flipped.
> 6. `_concept-audit-matrix.md` 2 rows flipped silent-in-code → honored.
> 7. Concept-`permissions/03-action-vocabulary.md` verified-header bumped (CH-04 amendment line).
> 8. Concept doc body: 33-verb list + applicability matrix UNCHANGED (CH-04 doesn't rewrite the canonical vocabulary).
> 8a. Concept doc has a NEW § "Future work — v1 revisit: per-resource action enums" subsection appended at the end. The subsection cross-references ADR-0043 §D43.8 and explains the deferral rationale.
> 9. ADR-0043 cross-references concept-doc, drifts D-new-09 + D-new-10, and downstream chunks CH-05 + CH-08 + CH-15.
> 10. Plan archive at `plan/build/<8hex>-ch-04-...md`.
> 11. CH-21 invariants intact; CH-16 Identity acceptance still green; CH-22 catalog tests still green.
> 12. CH-03 invariants intact; ADR-0042 still Accepted; D-new-02 still remediated.

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
# Expect: ~1161 passed / 0 failed

# 3. Positive greps
grep -n "pub enum Action\b" modules/crates/domain/src/permissions/action.rs              # 1
grep -n "fn applies_to\b" modules/crates/domain/src/permissions/action.rs                # ≥ 1
grep -c "^    [A-Z][a-zA-Z]*,$" modules/crates/domain/src/permissions/action.rs          # ≥ 34
ls modules/crates/domain/src/permissions/action.rs                                       # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0043-typed-action-vocabulary.md  # 1

# 4. Negative greps (no string-action construction left)
grep -rn 'vec!\["read"\|vec!\["allocate"\|vec!\["transfer"\|vec!\["modify"' modules/crates/   # 0
grep -rn '"launch_session"' modules/crates/                                                  # 0
grep -rn '"\*"' modules/crates/domain/src/permissions/  | grep -v "test\|comment"             # 0 (or only Wildcard serialization)

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-09.md   # 1
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-10.md   # 1

# 5a. v1-revisit note in concept doc + ADR
grep -n "Future work\|v1 revisit\|per-resource action" docs/specs/v0/concepts/permissions/03-action-vocabulary.md  # ≥ 1
grep -c '^### D43\.8' docs/specs/v0/implementation/m5_2/decisions/0043-typed-action-vocabulary.md  # 1

# 6. Wire-format sanity
cargo test -j 4 -p domain --lib permissions::action::tests
# Expect: ~35 tests passed including the round-trip + matrix exhaustive tests.

# 7. Carry-forward sanity
cargo test -j 4 -p server --test acceptance_memory_extraction              # CH-21 still green
cargo test -j 4 -p server --test identity_materialization_acceptance       # CH-16 still green
cargo test -j 4 -p domain --lib events::listeners::tests                   # CH-22 catalog tests still green
```

---

## What this plan does NOT do

- Doesn't change SurrealDB schema. Wire format preserved via snake_case serde.
- Doesn't enforce the matrix at every callsite — only at CH-05's publish-time validator. CH-04 ships the matrix as a queryable function.
- Doesn't add new actions. The 33-verb vocabulary stays exactly what the concept doc lists.
- Doesn't refactor Permission Check engine semantics. Same algorithm; typed values now.
- Doesn't migrate audit-event diff JSON. Action serialization is byte-stable.

---

## Critical files

**New:**
- `modules/crates/domain/src/permissions/action.rs` — the `Action` enum + matrix
- `docs/specs/v0/implementation/m5_2/decisions/0043-typed-action-vocabulary.md` — ADR

**Modified (heavy):**
- `modules/crates/domain/src/permissions/mod.rs` — add `pub mod action;`
- `modules/crates/domain/src/model/nodes.rs` — Grant.action + ToolAuthorityManifest.actions field types
- `modules/crates/domain/src/permissions/manifest.rs` — Manifest.actions field type + `from_node`
- `modules/crates/domain/src/permissions/engine.rs` — wildcard match update
- `modules/crates/domain/src/permissions/expansion.rs` — wildcard match update
- `modules/crates/domain/src/permissions/decision.rs` — type adjustment

**Modified (medium):**
- `modules/crates/domain/src/templates/{a,c,d}.rs` — typed action lists
- `modules/crates/server/src/platform/{secrets/add,model_providers/register,mcp_servers/register,sessions/preview,orgs/create}.rs` — typed action constructions
- `modules/crates/server/src/bootstrap/claim.rs` — typed action constructions (×2)
- `modules/crates/domain/src/auth_requests/{transitions,revocation}.rs` — typed action constructions

**Modified (light):**
- Test fixtures: `domain/tests/template_a_firing_props.rs`, `server/tests/*.rs` (~15–20 assertion updates)
- Drift files: `D-new-09.md`, `D-new-10.md`, `drifts/README.md`, `_concept-audit-matrix.md`
- Concept doc: `permissions/03-action-vocabulary.md` header bump
- Architecture doc: `m1/architecture/permission-check-engine.md` header bump

---

## Estimated effort

~2.5 engineer-days:
- 0.5d — chunk-open ritual + P1 enum scaffold + matrix encoding + 35 unit tests.
- 1.5d — P2 carrier-type migration + ~51 callsite updates + test fixture updates + workspace clippy/test green.
- 0.3d — P3 ADR Accepted + drift closure + concept-doc header + matrix flip + drifts/README + 2 audit agents + seal.
- 0.2d — buffer for unexpected callsite discoveries during P2.
