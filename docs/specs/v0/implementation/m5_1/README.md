<!-- Last verified: 2026-04-24 by Claude Code -->

# M5.1 — Drift Discovery & Forward-Scope Cataloguing

**Status:** SEALED · 2026-04-24
**Milestone type:** documentation-only (no `modules/` code changes)
**Parent plan:** [`plan/build/01710c13-m5-templates-system-agents-sessions.md`](../../../plan/build/01710c13-m5-templates-system-agents-sessions.md) (M5 proper)
**Successor:** M5.2 (P8 memory-extraction listener + P9 seal; scope now planned chunk-by-chunk per the per-chunk template introduced here)

## Purpose

M5.1 was opened after M5/P7 close when the accumulating drift ledger (24→29 drifts) signalled that the implementation model had outrun the concept-doc source-of-truth. Rather than paper over the gap with ad-hoc remediation, the milestone was reframed: surface every drift, quantify remaining scope, and codify the discipline under which every future implementation chunk proceeds — before any more code moves. No code was written during M5.1.

## Phase summary

| Phase | Goal | Effort | Status |
|---|---|---|---|
| P0 | Folder scaffolding + drift-file schema + index skeleton | 0.5d | SEALED |
| P1 | Migrate 29 existing drifts from plan-archive ledger to per-file catalogue | 1.5d | SEALED |
| P2 | Concept-vs-implementation audit across 20 concept docs; discover new drifts | 2.5d | SEALED |
| P3 | Forward-scope inventory: drift remediation + P8+P9 carryover + M6+ scope broken into implementation chunks | 1.5d | SEALED |
| P4 | Per-chunk planning process: template + lifecycle checklist + drift-lifecycle doc | 0.5d | SEALED |
| P5 | Seal: 3 independent agent audits + cross-reference check + summary report + 3-aspect confidence close | 1d | SEALED (this doc) |

## Artefacts produced

### Drift catalogue — [`drifts/`](./drifts/)

- [`drifts/README.md`](./drifts/README.md) — index table (60 drifts).
- [`drifts/_schema.md`](./drifts/_schema.md) — canonical template every drift file instantiates.
- [`drifts/_ledger-migration-log.md`](./drifts/_ledger-migration-log.md) — P1 migration record (7 discrepancies noted, including the 24-vs-29 meta-drift where planning said 24 but the actual ledger entry count was 29).
- [`drifts/_concept-audit-matrix.md`](./drifts/_concept-audit-matrix.md) — concept-doc × claim matrix (~95 rows) covering all 20 concept docs.
- **60 drift files** — 29 migrated (D1.1 / D1.2 / D1.3 + D2.x–D7.x) + 31 newly discovered (D-new-01 through D-new-31).

### Forward-scope inventory — [`plan/forward-scope/`](../../../plan/forward-scope/)

- [`22035b2a-remaining-scope-post-m5-p7.md`](../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md)
  - §1 — 24 remediation chunks CH-01 to CH-24 + 5 deferred-markers (M6-DEFERRED-01/02, M6-or-M7-DEFERRED, M7-DEFERRED-01, M7b-DEFERRED-01).
  - §2 — M5 P8 + P9 scope restated as chunks.
  - §3 — M6+ scope at chunk granularity.
  - §4 — chunk dependency graph (critical path: CH-02 real `agent_loop` unblocks CH-15/16/17/21/24).
  - §5 — per-chunk summary table.
  - §6 — open questions.
  - §7 — planning decisions Q1–Q7 (all answered 2026-04-24; binding for P4 template + all future chunks).

### Process docs — [`process/`](./process/)

- [`process/per-chunk-planning-template.md`](./process/per-chunk-planning-template.md) — canonical 12-section template every chunk plan copies; Q1–Q7 decisions baked in.
- [`process/chunk-lifecycle-checklist.md`](./process/chunk-lifecycle-checklist.md) — 8-step execution recipe with entry/exit criteria.
- [`process/drift-lifecycle.md`](./process/drift-lifecycle.md) — 7-state machine (`discovered → classified → scoped → in-chunk-plan → remediated | renegotiated | accepted-as-is`) + update discipline.

## Catalogue statistics

### Drift count

- **Total drift files:** 60
- **Migrated from ledger (P1):** 29 (D1.1, D1.2, D1.3, D2.x–D7.x)
- **Newly discovered (P2):** 31 (D-new-01 through D-new-31)
- **Verified-false & discarded during P2:** 2 (system-agent-creation proposal; permission-ceiling-enforcement proposal — both verified-false by reading current code)

### Severity distribution

- **HIGH:** 18 drifts (all targeted to close before M5 tag per Q5)
- **MEDIUM:** 16 drifts (case-by-case defer decision at each chunk-open)
- **LOW:** 26 drifts (all close at M5 via CH-19 + CH-20 pure-doc chunks)

### Current status distribution

- **discovered:** 58 drift files
- **remediated:** 2 drift files (D5.1 closed at M5/P5-scope-deferred-to-P7-ship; D6.2 closed at M5/P7-ship; both confirmed at P1 migration)
- **classified / scoped / in-chunk-plan / renegotiated / accepted-as-is:** 0 each (no chunk has opened yet under the new process)

### Concept-doc coverage

- **Top-level concept docs audited:** 11/11 (`agent`, `coordination`, `human-agent`, `ontology`, `organization`, `phi-core-mapping`, `project`, `system-agents`, `token-economy` + `README` + implicit cross-refs)
- **Permissions sub-docs audited:** 9/9 (`permissions/01` through `permissions/09`)
- **Total concept docs:** 20/20 (100%)

### Chunk inventory

- **Total chunks in forward-scope §5:** 24 CH-NN + 5 deferred markers = 29
- **HIGH-severity-closing chunks:** CH-01 through ~CH-16 (mapping varies per drift severity)
- **Pure-doc chunks:** CH-19 (LOW-doc ratification) + CH-20 (retired-drift LOW sweep)

## P5 seal — independent audit results

Three fresh `Explore` subagents ran in parallel. None had produced the artefacts they audited.

### Audit 1 — Coverage audit: PASS

- Ledger → files: **26/26** (all D2.1–D7.6 ledger entries present as files; D1.1/1.2/1.3 predate the ledger range but are present in catalogue)
- Concept-doc matrix coverage: **20/20** (every concept doc has ≥1 matrix row)
- Chunk → drift/commitment references: **29/29 chunks anchored** (no chunk references nothing)
- Orphan drifts: **0** (every drift file is cited from README, matrix, OR forward-scope)

### Audit 2 — Concept-alignment spot-check: PASS (3/3 confirms-matrix)

Agent picked 3 stratified claims and independently verified each against shipped code:

1. **`ontology.md` §Node Types — 37 NodeKind variants** (status: honored)
   → Confirmed: [`domain/src/model/nodes.rs:41-136`](../../../../../modules/crates/domain/src/model/nodes.rs) declares exactly 37 variants; `NodeKind::ALL` array matches.

2. **`permissions/05-memory-sessions.md` §Memory selector predicates — full tag-predicate DSL** (status: contradicted → D-new-03)
   → Confirmed: [`domain/src/permissions/selector.rs`](../../../../../modules/crates/domain/src/permissions/selector.rs) implements only 4 variants (`Any`, `Exact`, `Prefix`, `KindTag`); no composition operators. Drift D-new-03 correctly classifies.

3. **`permissions/03-action-vocabulary.md` §Conservative Over-Declaration** (status: honored)
   → Confirmed: [`domain/src/permissions/manifest.rs:34-50`](../../../../../modules/crates/domain/src/permissions/manifest.rs) + [`domain/src/permissions/engine.rs:153-171`](../../../../../modules/crates/domain/src/permissions/engine.rs) implement resource ∪ transitive expansion exactly as specified.

**No matrix-wrong / matrix-stale findings.**

### Audit 3 — phi-core leverage audit: PASS

- `scripts/check-phi-core-reuse.sh`: **green** (exit 0).
- Forbidden-duplication greps (8 phi-core type names): **0 hits** across all patterns.
- Wrapper patterns verified: [`domain/src/model/nodes.rs:296`](../../../../../modules/crates/domain/src/model/nodes.rs) `AgentProfile` wraps `phi_core::agents::profile::AgentProfile` via `blueprint` field; `nodes.rs:841` `Session` wraps `phi_core::session::model::Session` via `inner` field. Both follow rule 1 of [`baby-phi/CLAUDE.md`](../../../../../CLAUDE.md) §phi-core Leverage.
- `leverage-violation`-tagged drifts: **1** (D4.2 — synthetic event feeder bypasses `phi_core::agent_loop()`); concrete code locations named; properly mapped to `contradicts-concept` classification.
- No forbidden hits outside drift coverage.

## P5 seal — cross-reference + carry-forward checks

### CI guards (all green at P5 seal)

| Guard | Result |
|---|---|
| `scripts/check-doc-links.sh` | ✅ all markdown under `docs/specs/v0/implementation` valid |
| `scripts/check-ops-doc-headers.sh` | ✅ all 25 ops docs carry `Last verified` header |
| `scripts/check-phi-core-reuse.sh` | ✅ no forbidden redeclarations |
| `scripts/check-spec-drift.sh` | ✅ 29 referenced IDs all present |

### Workspace health

- `cargo test --workspace -- --test-threads=1`: **966 passed, 0 failed, 1 ignored** (serialised to avoid port-bind race in acceptance harness — baseline unchanged from M5/P7 close).
- `git diff --stat HEAD -- modules/`: **empty** (no code shifted in all of M5.1).

## 3-aspect close + 2 confidence %

**Aspect 1 — Coverage** ✅
Independent Audit 1 verified 26/26 ledger→file + 20/20 concept-doc matrix + 29/29 chunk anchors + 0 orphans.

**Aspect 2 — Concept fidelity** ✅
Independent Audit 2 spot-checked 3 stratified matrix rows against current code; 3/3 confirms-matrix. No matrix-wrong / matrix-stale findings.

**Aspect 3 — Actionability** ✅
Independent Audit 3 confirmed phi-core leverage stays green; drift files name concrete code locations; forward-scope §4 dependency graph identifies CH-02 as the critical-path unblock; process docs provide copy-ready template + checklist for any future chunk.

### Documentation confidence % (source of truth: concept docs)

`Doc confidence = (pages cross-checkable unambiguously) / (pages produced in M5.1)`

| Denominator breakdown | Count |
|---|---|
| 60 drift files | 60 |
| `_schema.md` + `_ledger-migration-log.md` + `_concept-audit-matrix.md` + `drifts/README.md` | 4 |
| 3 process docs (template + checklist + drift-lifecycle) | 3 |
| 1 forward-scope inventory | 1 |
| 1 M5.1 summary README (this doc) | 1 |
| **Total** | **69** |

Numerator: pages where an independent reader can cross-check against code + concept docs + ADRs without ambiguity. Audits 1+2+3 validated coverage, concept fidelity, and phi-core leverage on a sampled basis. Audit 2's 3/3 confirmation on stratified matrix rows is the strongest signal. No pages flagged by any audit as unclear or stale.

`Doc confidence = 69/69 = **100%**`

### Implementation confidence % — N/A

M5.1 ships no code changes. Implementation-vs-concept confidence applies at per-chunk close (per template §10), not at a doc-only milestone close. `git diff --stat HEAD -- modules/` verifies empty diff.

### Composite

`Composite = min(coverage✅, concept-fidelity✅, actionability✅, doc-confidence 100%) = **100%**`

Target for P5 was ≥99% with independent-audit sign-off. Target exceeded.

## What M5.1 did NOT do

- No `modules/` code changes.
- No concept doc amendments (concept docs remain authoritative exactly as they were at P5 open).
- No ADRs flipped to Accepted (ADR authoring now happens per-chunk at chunk-plan draft time per Q6).
- No remediation of any drift via code.
- No M5.2 (P8 / P9) scope execution.

## What happens next

1. **User selects the first chunk** from [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 (per Q4 — user-decided at each chunk-open).
2. **Drafter copies [`process/per-chunk-planning-template.md`](./process/per-chunk-planning-template.md) structure** into a new plan file at `plan/build/<8hex>-<chunk-name>.md`; fills 12 sections; allocates ADR number(s) per Q6.
3. **Pre-chunk gate walked** per template §9: reading list, carry-forward invariants, chunk-specific re-verification.
4. **`ExitPlanMode` approval** per Q7 (uniform ritual — same for doc-only chunks).
5. **Chunk execution** per [`process/chunk-lifecycle-checklist.md`](./process/chunk-lifecycle-checklist.md) Steps 4–8.
6. **Drift files transition** per [`process/drift-lifecycle.md`](./process/drift-lifecycle.md) state machine.
7. **Repeat** for subsequent chunks. M5 tag ships when all HIGH-severity chunks close (Q5).

## Guardrails (binding beyond M5.1)

The 9 guardrails from the M5.1 plan remain binding for every subsequent chunk — concept docs authoritative, no silent drift, no code without plan, mid-flight pause, drift lifecycle tracked, phi-core leverage non-negotiable, post-chunk independent audit mandatory, no assumption of prior-phase correctness, close-time confidence pinned to concepts with min-composite + no rounding.
