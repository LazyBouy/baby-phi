<!-- Last verified: 2026-05-17 by Claude Code (CH-26 cycle-audit gate-4 — orchestrator final re-audit GREEN; F1.b + F2.b USER-DIVERGENT user-locks (both DIVERGENT from planner-recommended F1.a + F2.a); cumulative cross-cycle divergent forks now 10-of-12 (~83%); 3-auditor envelope per F3 large; Audit-A iter 1 PASS 24/24 clean; Audit-B iter 1 PASS 15/15 with 2 orchestrator-applied Trivial patches (1L on `_concept-audit-matrix.md:25` Composite cardinality + multi on `permissions/01-resource-ontology.md:189` enumeration + anchor); Audit-C iter 1 PASS 9/9 clean; advisory-only F1.b revision per implementer scope-revision + orchestrator-approval (ADR-0061 §D61.5 + §D61.7 amendments document + cross-ref to CH-27 carve-out); 2 acceptance tests action-verb-renamed (Inspect/Observe → Allocate per CH-25 synth-grant scope [Allocate, Transfer]); cargo test 1568/0/2 within plan §8 v2 band [1568, 1574] at lower bound; phi-core baseline 57 preserved (Δ +0); 4 CI guards green orchestrator-side; ADR-0061 Accepted; D-philosophy-02 remediated; D-CH26-FOLLOWUP-01 filed (Closing chunk: CH-27 per user-routed M5.3 carve-out extension); CH-27 row appended to forward-scope §2.5; M5.3 carve-out scope now 3-chunk {CH-25, CH-26, CH-27}; cycle hex `d1cb9e1f`.) -->

# CH-26 cycle audit — Org/Project as Composite resources (closes D-philosophy-02 via ADR-0061; advisory-only F1.b with CH-27 blocking-gate carve-out)

**Cycle hex:** `d1cb9e1f`
**Date:** 2026-05-17
**Author:** Claude Code (orchestrator)
**Plan:** [plan.md](./plan.md) (v2 — F1.b + F2.b USER-DIVERGENT re-plan after gate-1 user-lock)
**Resume note** (mid-cycle scope-revision): [RESUME-NOTE.md](./RESUME-NOTE.md)
**Audit logs:** [audit-A-iter1.md](./audit-A-iter1.md), [audit-B-iter1.md](./audit-B-iter1.md), [audit-C-iter1.md](./audit-C-iter1.md)
**Verdict:** GREEN

---

## §1 — Audit pipeline summary

| Stage | Auditor | Iter | Verdict | Notes |
|---|---|---|---|---|
| Plan-time gate-1 | self + user | n/a | APPROVED with **F1.b + F2.b DIVERGENT** user-locks | User locked F1.b WIDE handler refactor (~5 ed) + F2.b NEW `tags: Vec<String>` field (~1.5 ed wire-format cascade); both DIVERGENT from planner-recommended narrow-scope F1.a + catalogue-path F2.a. F3 audit envelope (3 auditors / large) aligned. Planner re-spawn v2 absorbed the wide-scope into 7-phase plan with NEW P-FIELD-EXTEND phase. |
| Mid-cycle scope-revision | self + user | n/a | APPROVED **advisory-only F1.b + CH-27 carve-out** | Implementer surfaced at Partial-P2 that F1.b WIDE handler refactor cost ~12-15 ed (5× the planner's ~3 ed underestimate). User routed to hybrid Option-A: ship engine.check_permission INVOCATIONS at 15 sites across 7 handlers consumed ADVISORILY (bespoke gates remain wire-tier rejection); blocking-gate tightening + synth-grant widening + resolvers wiring carve out to NEW CH-27 (kept in M5 carve-out per user direction, NOT M6-DEFERRED). ADR-0061 §D61.5 + §D61.7 amendments + D-CH26-FOLLOWUP-01 + forward-scope CH-27 row durably forward-route. |
| Sub-agent audit | A — Rust correctness | 1 | **PASS** 24/24 clean (re-dispatched after prior session interrupt killed first attempt) | All claims green: Composite cardinality 8→10 with constituents() = [DataObject, IdentityPrincipal, Tag]; `tags: Vec<String>` field on Organization (`nodes.rs:441-442`) + Project (`nodes.rs:513-514`) both `#[serde(default)]`; migration #0018 idempotent + registered at slot 18 (migrations_test.rs len bumped 17→18); InMemoryRepository + SurrealStore materialise paths wired; 15 `check_permission` invocations across 7 handler files (5× the D-philosophy-02:39 ≥3-hit invariant); advisory-only consumption verified at 3 representative handlers (e.g., `orgs/show.rs:87 let _engine_inspect_allowed = ...` discarded with doc-comment citing CH-27 tightening); action-verb-renamed tests use `Action::Allocate`; cargo test 1568/0/2 + clippy + fmt + 4 CI guards GREEN; phi-core baseline 57; ADR-0061 Accepted; D-philosophy-02 remediated; D-CH26-FOLLOWUP-01 filed; cycle-index dual-write verified; forward-scope CH-27 row appended. Cargo-clean reclaimed 92.1 GiB. |
| Sub-agent audit | B — Docs fidelity | 1 | **PASS** 15/15 (2 Trivial orchestrator-applied patches at gate-3) | ADR-0061 + 2 NEW m5_3 docs + concept-doc amendments + drift housekeeping + cycle-index dual-write + forward-scope §2.5 amendments all comprehensive. 4 Side Observations: (1) Trivial-1L on `_concept-audit-matrix.md:25` "8 Composite" → "10 Composite" — orchestrator-applied at gate-3; (2) Trivial-multi on `permissions/01-resource-ontology.md:189` enumeration extended to "10 composites" + section anchor `composite-classes-8` → `composite-classes-10` — orchestrator-applied at gate-3; (3) `agent-ownership-model.md` body subsections deferred per verified-header — deferrable to CH-27 P-DOCS; (4) `traceability-matrix.md:64` — next-docs-sweep deferrable. |
| Sub-agent audit | C — Vertical-slice integrity | 1 | **PASS** 9/9 clean | End-to-end chain (Edge::Owns emit (CH-25) → list_agent_owned_orgs/projects (CH-25) → step_2_resolve_grants synth-loop (CH-25) → CH-26 check_permission invocation at refactored handler → advisory-only consumption) verified via `acceptance_m5_3_composite_resources.rs:332-364`. Stranger-Agent denial via empty `agent_owned_orgs` → no synth-grant Candidate (cross-org isolation invariant). Composite expansion via `constituents()` returns expected [DataObject, IdentityPrincipal, Tag] bundle per ADR-0061 §D61.1. Migration #0018 idempotent + purely additive. CH-27 carve-out durably forward-routed across 5 surfaces (ADR §D61.7 + drift D-CH26-FOLLOWUP-01 + forward-scope CH-27 row + Post-M5.3-actions amendment + cycle-index row). |
| Orchestrator final cycle re-audit | self | n/a | **PASS** (this doc) | MUST-RUN list executed authoritatively; see §3. |

**Iteration accounting:** Audit-fix-loop iteration count for CH-26 = **1**. **2 orchestrator-applied trivial patches** (1 Trivial-1L on `_concept-audit-matrix.md:25` + 1 Trivial-multi on `permissions/01-resource-ontology.md:189`) per CLAUDE.md trivial protocol — neither requires auditor re-spawn. Audit-A re-dispatched once due to session-interrupt killing the original run mid-flight; the re-dispatch is NOT counted as an iteration per CLAUDE.md (resumed audit, not audit-fix-loop iteration).

---

## §2 — User-locked forks (final state)

| Fork | Locked at | Path | Recommendation alignment |
|---|---|---|---|
| F1 — Handler refactor scope | gate-1 plan-approval | **F1.b WIDE (≥7 handlers refactored in-chunk; advisory-only revision at mid-cycle)** | **diverges from planner v1 F1.a NARROW (~3 ed; defer 6-7 handlers to M7-DEFERRED)** — user explicitly chose wide; implementer scope-revised to advisory-only mid-cycle with orchestrator + user approval; CH-27 carve-out completes blocking-gate work in M5 |
| F2 — Backfill mechanism | gate-1 plan-approval | **F2.b NEW `tags: Vec<String>` field on Org/Project + migration #0018** | **diverges from planner v1 F2.a catalogue-entry path (~0.8 ed)** — user chose richer wire-format cascade (~1.5 ed); ~50 fixture-cascade sites updated |
| F3 — Audit envelope | gate-1 plan-approval | F3 default 3 auditors (large) | aligns w/ planner v1 + audit-envelope-size skill |
| Mid-cycle scope-revision | gate-3-prep | **Advisory-only F1.b + NEW CH-27 carve-out** | hybrid Option-A variant per user routing; ratified in ADR-0061 §D61.5 + §D61.7 amendments |

**Cross-cycle divergence pattern note** (per chunk-planner v17 prominent gate-1 callout): F1.b + F2.b lock continues divergence streak. Cumulative cross-cycle divergent forks: **10-of-12 (~83%)** (CH-15 F5.B + CH-17 F5.B + CH-18 F3.B + CH-20 F1.B + CH-24 F1.B + F-D59.2.b + F-D59.3.b + CH-25 F1.b + CH-26 F1.b + F2.b vs lone non-divergent CH-19 + 2 aligned forks). User consistently prefers tighter / richer / more-defensive / more-explicit-visible options. **Pattern is now fully mature at 83% — v17 divergence-aware framing remains default discipline.**

---

## §3 — Orchestrator MUST-RUN list (gate-4)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **PASS** (exit 0; 0 diffs) |
| `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` | **PASS** (exit 0; 0 warnings; fresh build from clean target/) |
| `cargo test --workspace -j 4 -- --test-threads=1` (via canonical `audit-tmp-cargo-counts.sh`) | **PASS** (`passed=1568 failed=0 ignored=2` — exact match to plan §8 v2 band [1568, 1574] at lower bound; +12 net new tests vs CH-25 baseline 1556; matches implementer + Audit-A reports) |
| `cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` (immediate post-test per v9 placement-1) | **92.1 GiB reclaimed** (44,636 files removed) |
| `bash scripts/check-doc-links.sh` | **PASS** ("all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK.") |
| `bash scripts/check-ops-doc-headers.sh` | **PASS** ("all 38 ops doc(s) carry the 'Last verified' header. OK.") |
| `bash scripts/check-phi-core-reuse.sh` | **PASS** ("no forbidden phi-core redeclarations under modules/crates. OK.") |
| `bash scripts/check-spec-drift.sh` | **PASS** ("29 referenced ids all present in docs/specs/v0/requirements. OK.") |

**Audit-A claim count**: 24/24 PASS verified by orchestrator gate-4 cross-check. **Audit-B claim count**: 15/15 PASS + 2 Trivial patches closed inline. **Audit-C claim count**: 9/9 PASS.

---

## §4 — phi-core leverage (gate-4 verification)

| Check | Result |
|---|---|
| Canonical baseline grep `grep -rn "use phi_core" modules/crates/ \| wc -l` | **57** (Δ +0 vs CH-25 close per plan §3 prediction exactly) |
| Forbidden-duplication grep | **0 hits** |
| Composite enum cardinality | **10** (8 → 10 evolution; NEW `OrganizationObject` + `ProjectObject`) |
| `tags: Vec<String>` fields on Org+Project | extant + `#[serde(default)]` |
| `check_permission` invocations across orgs+projects handlers | **15** (≫ D-philosophy-02:39 ≥3-hit invariant) |
| Migration `0018_org_project_tags.surql` | extant + idempotent + registered at slot 18 |

**phi-core leverage aspect: PASS.** Δ +0 imports preserved; forbidden-duplication greps clean; OrganizationObject + ProjectObject + tags field are baby-phi-defined NOT phi-core duplications.

---

## §5 — K8s microservice readiness (gate-4 verification)

| Axis | Result |
|---|---|
| A1 — New in-process state | No |
| A2 — New IPC channel | No |
| A3 — New pod-local resource | No |
| A4 — Migration / first-apply race | NEW migration #0018 — idempotent (DEFINE FIELD on already-defined-field is no-op in SurrealDB 2.x); purely additive; no breaking change to existing tail |
| A5 — Trait-shape requirement | No new trait method (per F2.b `tags` field add is struct-level; engine + handler invocations consume existing Repository methods) |
| A6 — Cross-pod state sharing | No |
| A7 — Audit hash-chain symmetry | No new emitters (advisory-only consumption only) |

**K8s posture: K8s-neutral.** No new `CHK8S-D-XX` ledger entry.

---

## §6 — Concept alignment + paperwork verification

### §6.1 — §2 concept-doc rows (from plan)

All §2 rows `honored` at chunk close. Notable transitions:
- `_concept-audit-matrix.md` "Organization has Projects (Project = Resource Type)" row Status `silent-in-code → honored` with `D-philosophy-02 ✓` Covering-drift cell.
- "Two Types of Resources (Fundamental + Composite)" row updated 8→10 Composite cardinality (orchestrator Trivial-1L patch at gate-3 per Audit-B Side Observation 1).

### §6.2 — Paperwork extants

| Artefact | Path | Status |
|---|---|---|
| ADR-0061 | `v0/implementation/m5_3/decisions/0061-org-project-as-composite-resources.md` | ✅ extant, Status: **Accepted**, 7 sub-decisions §D61.1-§D61.7; §D61.5 + §D61.7 amendments document advisory-only revision + CH-27 carve-out |
| Drift D-philosophy-02 | `v0/implementation/m5_3/drifts/D-philosophy-02.md` | ✅ extant, Status: **remediated** (load-bearing semantic axis), lifecycle entry with cycle hex `d1cb9e1f` |
| NEW drift D-CH26-FOLLOWUP-01 | `v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md` | ✅ filed at Status: `discovered`; Severity LOW; Bucket B; **Closing chunk: CH-27** (M5.3 carve-out extension per user direction, NOT M6-DEFERRED) |
| `m5_3/architecture/composite-resources-model.md` | (existing) | ✅ NEW; 8 sections per plan §7 P-DOCS deliverable |
| `m5_3/operations/composite-resources-operations.md` | (existing) | ✅ NEW; 5 sections per plan |
| `core-philosophy.md` | (existing) | ✅ verified-header bump (§"Organization has Projects" + §"Two Types of Resources" honored) |
| `permissions/01-resource-ontology.md` | (existing) | ✅ AMEND BODY — Composite Classes 8→10 + 2 NEW rows + tags field documented; line 189 enumeration extended at gate-3 Trivial-multi patch per Audit-B Side Observation 2 |
| `m5_3/drifts/README.md` | (existing) | ✅ AMEND — open-drifts + remediated counts updated; D-philosophy-02 row Status flipped; D-CH26-FOLLOWUP-01 row appended |
| `m5_3/drifts/_concept-audit-matrix.md` | (existing) | ✅ AMEND — "Org has Projects" row flipped + "Two Types of Resources" cardinality updated 8→10 (Trivial-1L) |
| Cycle-index row | `_cycle-index.md` | ✅ APPEND + cycle-index top verified-header PREPEND per chunk-implementer v9 R2 dual-write (verified: `head -1 \| grep -c d1cb9e1f` returns 1; `grep -c d1cb9e1f _cycle-index.md` returns ≥ 2) |
| Forward-scope §2.5 | `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` | ✅ CH-26 row CLOSED header amended + NEW CH-27 row appended + §"Post-M5.3 actions" amended (M6 plan-open waits on CH-27 close) |

**Doc-sync widened sweep (per CLAUDE.md gate-2 + CH-15 retro Row 1):** ZERO actionable stale-narrative matches introduced by CH-26.

### §6.3 — Concept-contradiction discoveries → retrospective routing candidates

Audit-A + Audit-B + Audit-C surfaced **6 retrospective routing candidates**:

1. **Mid-cycle scope-revision pattern**: F1.b WIDE → advisory-only path required orchestrator + user routing at gate-3-prep. Pattern emerged from F1.a planner estimate underspecifying CheckContext-build cost (~5 ed predicted vs ~12-15 ed actual). Chunk-planner v17 candidate: extend §3 cascade-discipline to include "CheckContext-build cost" estimation for handler-refactor cascades.

2. **F2.b wire-format cascade vs catalogue-path tradeoff**: planner-recommended F2.a catalogue-path (0.8 ed); user-locked F2.b wire-format (1.5 ed + ~50 fixture sites). Both work; F2.b is more "visible" + serde-strip-correct. Retrospective candidate: document the tradeoff matrix for future composite-resource extensions.

3. **Trivial-1L recurrence on `_concept-audit-matrix.md` cardinality references**: Audit-B Side Observation 1 surfaced "8 Composite" → "10 Composite" — chunk-implementer v9 P-SEAL paperwork checklist already covers cycle-index dual-write; should also cover concept-audit-matrix cardinality references on any chunk that flips an enum/struct cardinality.

4. **Trivial-multi recurrence on concept-doc enumeration + anchor sync**: Audit-B Side Observation 2 surfaced `permissions/01-resource-ontology.md:189` enumeration + anchor. Chunk-implementer v11 candidate: when a concept-doc section anchor changes (e.g., `composite-classes-8` → `composite-classes-10`), grep + update all cross-references to the old anchor in the same chunk.

5. **CH-27 carve-out forward-routing**: CH-26 is the second cycle (after CH-24 / CH-25 R5) to use a user-routed in-M5 carve-out (vs M6-DEFERRED). Pattern is maturing; retrospective candidate to codify "in-M5 carve-out vs M6-DEFERRED" decision criteria.

6. **Permissions-audit skill v4 re-validation**: CH-26 is the 2nd cycle to run the skill post-fix (CH-25 was 1st). Retrospector should re-confirm skill operates correctly on CH-26 cycle window.

**Concept alignment aspect: PASS** (all §2 rows honored; doc-sync widened sweep ZERO actionable matches).

---

## §7 — Cycle metrics

| Metric | Value |
|---|---|
| Cycle hex | `d1cb9e1f` |
| Plan versions | v1 → v2 (1 re-spawn after gate-1 F1.b + F2.b USER-DIVERGENT user-lock) |
| Phases shipped | 7 (P0 + P1 + P-FIELD-EXTEND + Partial-P2 + P2-d6 + P3 + P-DOCS + P-SEAL — counting Partial-P2 + P2-d6 as one phase = 7) |
| Estimated effort | ~3-4 ed planner v1 / ~5 ed planner v2 / **~12-15 ed actual** (F1.b WIDE underestimate surfaced at Partial-P2; advisory-only revision reduced remaining ed budget by ~70%) |
| Files modified | ~50 modified + ~10 new (cycle folder + ADR-0061 + 2 m5_3 doc-tier files + 3 NEW tests + migration + RESUME-NOTE + audit-tmp scripts) |
| New test files | NEW `acceptance_m5_3_composite_resources.rs` (10 scenarios) + harness extensions in `acceptance_common/` |
| Test count delta | **+12** new tests (workspace 1556 → 1568/0/2; within plan §8 v2 band [1568, 1574] at lower bound) |
| phi-core import delta | **0** (baseline 57 preserved) |
| ADRs ratified | **1** (ADR-0061 Proposed → Accepted; sub-decisions §D61.1–§D61.7) |
| Drifts closed | **1** (`D-philosophy-02` discovered → remediated at load-bearing semantic axis) |
| Follow-up drifts filed | **1** (`D-CH26-FOLLOWUP-01` LOW; Closing chunk CH-27; documents advisory→blocking tightening + resolvers passthrough + synth-grant widening + M3/M4/M5 fixture extension) |
| Cargo-clean cumulative | **~400+ GiB reclaimed** across cycle invocations (implementer P1-P-FIELD-EXTEND + P-SEAL + audit-A 92.1 GiB + gate-4 92.1 GiB) |
| Within-cycle divergent forks | **2** (F1.b + F2.b — first cycle with multiple BOTH-divergent forks at gate-1) |
| Cumulative cross-cycle divergent forks | **10-of-12 (~83%)** |
| Sub-agent audit FAILs | **0** (Audit-A 24/24 + Audit-B 15/15 + Audit-C 9/9) |
| Trivial-1L orchestrator inline patches | **1** (`_concept-audit-matrix.md:25` Composite cardinality reference per Audit-B Side Observation 1) |
| Trivial-multi orchestrator inline patches | **1** (`permissions/01-resource-ontology.md:189` enumeration + section anchor per Audit-B Side Observation 2) |
| Mid-cycle disk-pressure incidents | **0** (v9 placement-1 cargo-clean discipline empirically validated for 6th consecutive cycle) |
| Audit iteration count | **1** (Audit-A re-dispatch after session-interrupt kill NOT counted as iteration per CLAUDE.md) |
| Mid-cycle scope-revisions | **1** (F1.b WIDE → advisory-only + CH-27 carve-out per user routing at Partial-P2 boundary) |

---

## §8 — Close criteria (per plan §10)

| Aspect | Status | Evidence |
|---|---|---|
| Code aspect | ✅ PASS | All 7 phase deliverables shipped; cargo test 1568/0/2 within plan §8 v2 band [1568, 1574]; clippy 0 warnings; fmt clean; 4 CI guards green. |
| Docs aspect | ✅ PASS | NEW ADR-0061 (Accepted) + drift remediated + NEW follow-up drift filed + 2 NEW m5_3 doc-tier files + 5 amended docs; doc-sync widened sweep ZERO actionable. |
| phi-core leverage aspect | ✅ PASS | Baseline 57 preserved (Δ +0); forbidden-duplication greps return 0; Composite variants + tags field + check_permission invocations all baby-phi-defined. |
| Concept alignment aspect | ✅ PASS | "Organization has Projects (Project = Resource Type)" transitioned `silent-in-code → honored`; all §2 rows honored at chunk close. |
| Implementation confidence % | **24/24 = 100%** (Audit-A) | All 24 Audit-A claims PASS. |
| Documentation confidence % | **15/15 = 100%** (Audit-B; 2 Trivial patches closed at gate-3) | Audit-B 15 PASS + 2 Trivial resolved. |
| Vertical-slice integrity | **9/9 = 100%** (Audit-C) | Audit-C 9 PASS clean. |
| Composite | **≥99%** | min(100%, 100%, 100%, code, phi-core, concept-alignment) = 100% post-orchestrator-trivial-resolution. |

---

## §9 — Verdict + handoff

**Verdict: GREEN.** CH-26 M5.3-carve-out chunk (2nd of 3-chunk extended carve-out {CH-25, CH-26, CH-27}) passes orchestrator final cycle re-audit at composite ≥99% with zero implementation defects, 1 new follow-up drift filed (D-CH26-FOLLOWUP-01 for CH-27 closure), 2 orchestrator-applied trivial patches (1 Trivial-1L + 1 Trivial-multi), and full cargo-clean discipline honored.

**Cycle-index row Status flip**: `ready-for-audit` → **`audited-pending-retro`** (orchestrator applies post-cycle-audit write).

**Next gate (gate-5):** spawn chunk-retrospector v4 to write `retrospective.md` + invoke `permissions-audit` skill v4 (2nd cycle post-fix) + propose standards updates. The 6 retrospective routing candidates surfaced in §6.3 are pre-loaded for retrospector pickup.

**Cross-cycle implications:**
- **10-of-12 cumulative cross-cycle divergence (83%)** — user pattern continues; chunk-planner v17 divergence-aware framing remains default discipline.
- **M5.3 carve-out is now 3-chunk** {CH-25, CH-26, CH-27}; M6 plan-open waits on CH-27 close.
- **Advisory-only F1.b precedent**: 2nd cycle to use the pattern (CH-24 §D60.4 + CH-26 §D61.5). Establishing the "engine-level load-bearing semantic + wire-tier deferred-to-followup-chunk" idiom as a recurring resolution-shape for handler-refactor cycles.
- **Mid-cycle scope-revision pattern**: CH-26 is the 2nd cycle to route a mid-flight scope adjustment (CH-25 was the 1st with F-D59.2/F-D59.3 mid-cycle re-locks). User-routed in-M5 carve-out (vs M6-DEFERRED) is now the recurring shape.
- **Cargo-clean v9 placement-1 discipline validated for 6th consecutive cycle** (CH-19, CH-20, CH-22-23 grandfathered, CH-24, CH-25, CH-26); NO disk-pressure incidents within-cycle.
- **Permissions-audit skill v4 re-validation pending at retrospector**: CH-26 is the 2nd cycle to run the skill post-fix.

**No milestone tag at CH-26.** M5 was sealed at CH-24 with `v0.1-m5`; M5.3 carve-out is now 3-chunk; M6 plan-open follows CH-27 close.
