<!-- Last verified: 2026-04-24 by Claude Code -->

# Chunk lifecycle checklist

The repeatable step-by-step that every implementation chunk follows from draft to seal. Pairs with [`per-chunk-planning-template.md`](./per-chunk-planning-template.md) (the what) and [`drift-lifecycle.md`](./drift-lifecycle.md) (the drift-file status transitions triggered at each step).

Eight mandatory steps. Each step has explicit entry + exit criteria. No step skipped. No step backfilled.

---

## Step 1 — Draft the chunk plan per template

**Entry criteria:**
- User has selected the next chunk from [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 (Q4 decision — user-decided per chunk-open).
- No other chunk is mid-flight in this repo.
- CI guards (`check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`) are green at HEAD.

**Actions:**
1. Generate the 8hex token: `openssl rand -hex 4`.
2. Create the plan file at `baby-phi/docs/specs/plan/build/<8hex>-<chunk-name>.md`.
3. Copy the [`per-chunk-planning-template.md`](./per-chunk-planning-template.md) structure — 12 mandatory sections.
4. **ADR numbering (Q6 decision):** Before filling §5 "ADRs drafted":
   ```bash
   ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null \
     | xargs -I{} basename {} .md \
     | grep -oE "ADR-[0-9]+" \
     | sort -u \
     | tail -5
   ```
   Record the next sequential free number in §5. Never allocate mid-chunk.
5. Fill §1 through §12 using the exact field labels from the template.
6. For every drift cited in §4, open its drift file and confirm its current `Status` is `scoped` (or `classified` if newly discovered; transition to `scoped` happens in Step 2).
7. Draft §11 post-chunk audit plan — agent count per chunk size, prompt text, audit aspect coverage.

**Exit criteria:**
- All 12 sections present with non-empty content (or an "N/A + one-line justification" where legitimately absent).
- §9 reading list compiled.
- §5 ADR numbers claimed and recorded.
- No `TBD` / `...` placeholder text anywhere in the plan file.

---

## Step 2 — Walk concept-alignment + phi-core leverage map + post-chunk audit plan

**Entry criteria:**
- Step 1 exit criteria met.

**Actions:**
1. Physically open every concept doc cited in plan §2. Verify each citation's `§anchor` matches the current doc structure.
2. For each drift in plan §4, transition its `Status` from `classified` or `scoped` to `scoped` (if not already) and append a lifecycle entry:
   ```
   - YYYY-MM-DD — `scoped` — included in CH-NN plan draft (file <8hex>-<chunk>.md)
   ```
3. Physically run the §3 phi-core leverage greps against current HEAD. Confirm the "current handling" column is accurate.
4. Physically run `bash scripts/check-phi-core-reuse.sh` and confirm green.
5. Finalise §11 audit prompts — the drafter writes the exact `Agent()` prompt text each auditor will receive. Audit scope fixed here is the scope used at Step 6; no mid-chunk audit-scope drift.

**Exit criteria:**
- §2 citations verified green against live concept docs.
- §3 current-state column matches live grep output.
- Every drift in §4 has its `Status` and lifecycle entries synced.
- §11 audit prompts locked in.

---

## Step 3 — User approval via ExitPlanMode

**Entry criteria:**
- Step 2 exit criteria met.
- `git status --short` for `modules/` returns empty (no code changes prior to approval).

**Actions:**
1. Invoke `ExitPlanMode` referencing the plan file path.
2. Present the plan's §1 context + §4 drifts-closed table + §10 close criteria + §11 audit plan to the user (summary, not full re-dump).
3. Wait for explicit user approval. Do not start code.

**Exit criteria:**
- User approves the plan, OR
- User rejects → drafter revises plan per feedback → return to Step 1 or Step 2.

**No code moves until approval is explicit.** Applies uniformly to all chunks — including doc-only chunks (Q7 decision).

---

## Step 4 — Phase-by-phase execution with mid-phase pause discipline

**Entry criteria:**
- Step 3 user approval received.
- Git working tree clean.

**Actions:**
- Work through plan §7 phases in the stated order.
- Each phase's deliverables shipped BEFORE opening the next.
- **Mid-phase pause rules**: any of the following triggers `AskUserQuestion` before continuing:
  - Concept contradiction not anticipated in §2.
  - phi-core type overlap not in §3.
  - A drift surfaced by test or in-phase audit.
  - Scope change that would breach a §10 confidence target.
  - Test count deviating from §8 prediction by > 10%.
- "Document as drift at close" is retired for mid-flight discoveries. Discoveries surface immediately.

**Exit criteria:**
- All phases in §7 have shipped deliverables.
- No unresolved mid-phase pause.

---

## Step 5 — Per-phase 4-aspect close + 2 confidence %

**Entry criteria:**
- A given phase's deliverables shipped.

**Actions (for EACH phase, not just chunk close):**
1. Run the phase-scoped close checks:
   - Code aspect: test count matches phase prediction; clippy green; fmt green.
   - Docs aspect: every affected doc updated; verified-header dates bumped.
   - phi-core leverage aspect: import-count delta matches phase prediction; forbidden-duplication greps return 0; `check-phi-core-reuse.sh` green.
   - Concept alignment aspect: phase's §2 target-status transitions achieved.
2. Compute **two confidence percentages with named numerator/denominator**:
   - Implementation % = `(phase-claims-verified-honored) / (phase-claims-in-scope)`
   - Documentation % = `(phase-pages-cross-checkable-unambiguously) / (phase-pages-touched)`
3. Composite = **min** of the 5 measures (impl%, doc%, code-aspect-binary, leverage-aspect-binary, alignment-aspect-binary).
4. If composite ≥ target → phase closed; proceed to next phase.
5. If composite < target → do NOT round up. Pause. Surface to user. Fix before proceeding.

**Exit criteria:**
- Every phase has a close report with the five measures named and numerators/denominators explicit.

---

## Step 6 — Post-chunk independent audit

**Entry criteria:**
- All phases in §7 closed per Step 5.
- No unresolved mid-phase pause.

**Actions:**
1. Spawn audit agents per plan §11:
   - 1 agent for chunks ≤ 3 phases.
   - 2 agents for 4–6 phase chunks.
   - 3 agents for 7+ phase chunks.
2. Each agent receives the exact prompt drafted at Step 2 (§11 locked scope).
3. Audit aspects (a–d): code correctness, docs fidelity vs concept docs, concept alignment, phi-core leverage.
4. Auditors MUST be fresh `Explore` or `general-purpose` agents — never the implementer.
5. Collect audit reports.
6. Any new drift discovered → create a new drift file in [`../drifts/`](../drifts/) per the `_schema.md` template BEFORE chunk seal. Assign to a future chunk or fix in-chunk per user direction.
7. Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval via ADR, or converted to a drift file with explicit future-chunk assignment.

**Exit criteria:**
- All agents report clean on 4 aspects, OR
- Every audit-raised issue is explicitly scoped (fixed, renegotiated, or new drift file with assignment).

---

## Step 7 — Prior-chunk regression re-verification

**Entry criteria:**
- Step 6 audit returns clean + any audit-discovered drifts scoped.

**Actions:**
1. Re-run every plan §6 re-verification command (commands that ran at chunk-open — same commands, expected-same outputs).
2. Re-run prior chunks' own §12 verification recipes for upstream chunks this chunk depends on.
3. Any regression → create a drift file naming the broken upstream invariant; surface as an open question to the user BEFORE chunk seal. User decides: fix-in-chunk, new-remediation-chunk, or renegotiate.

**Exit criteria:**
- All upstream invariants re-verified green, OR
- Regressions documented as new drift files + explicit user decision recorded.

---

## Step 8 — Chunk seal with drift-file status updates

**Entry criteria:**
- Step 7 clean.
- Plan §10 composite confidence ≥ target.

**Actions:**
1. Compute chunk-level composite = `min(` all phase composites, all §10 measures `)`. Report with named numerators/denominators.
2. For each drift in plan §4, transition its `Status`:
   - `in-chunk-plan` → `remediated` (default: chunk shipped the remediation).
   - `in-chunk-plan` → `renegotiated` (if an ADR or user decision reframed scope — link the ADR).
   - `in-chunk-plan` → `accepted-as-is` (if user approved keeping the drift; rare; requires explicit ADR).
3. Append a lifecycle entry to each drift file:
   ```
   - YYYY-MM-DD — `remediated` — via CH-NN (plan <8hex>-<chunk>.md); <one-line how>
   ```
4. Update [`../drifts/README.md`](../drifts/README.md) index: status column for each drift refreshed.
5. Flip any `Proposed` ADRs drafted in plan §5 to `Accepted` (if the chunk covers them holistically).
6. Update the concept-audit matrix at [`../drifts/_concept-audit-matrix.md`](../drifts/_concept-audit-matrix.md): any row the chunk transitioned from `contradicted` to `honored` gets its Status column updated + Code-evidence column refreshed.
7. Update `cargo test --workspace` baseline count reference in the top-level chunk-plan verification recipe (if it shifted).
8. Commit with message citing the chunk ID + forward-scope row + list of drift files transitioned.

**Exit criteria:**
- All §4 drifts have updated `Status` + lifecycle entry.
- `drifts/README.md` index reflects current state.
- Concept-audit matrix updated where applicable.
- ADR statuses flipped where applicable.
- Post-seal `cargo test --workspace` + `clippy` + `fmt --check` green.
- Post-seal `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh` all green.

---

## Cross-step invariants

- **Concept docs are source of truth** — any disagreement resolved in favour of concept unless user explicitly renegotiates (guardrail 1 of the M5.1 plan).
- **No silent drift** — every mid-flight discovery gets a drift file; no informal mental carry-forward (guardrail 2).
- **phi-core leverage non-negotiable** — `scripts/check-phi-core-reuse.sh` green at every step that touches code (guardrail 6).
- **Audit mandatory** — every chunk closes with Step 6 audit before Step 8 seal (guardrail 7).
- **Confidence pinned to concepts** — every close reports the two % with named numerators/denominators; composite = min; no rounding (guardrail 9).

## Failure modes & escape hatches

| Failure | Response |
|---|---|
| Step 3 approval rejected | Return to Step 1 or 2 per user feedback. |
| Step 5 composite < target | Pause. Surface to user. Fix root cause. Do NOT proceed to next phase. |
| Step 6 audit finds new drift | Create drift file; user decides fix-in-chunk / defer / renegotiate BEFORE Step 8. |
| Step 7 regression detected | Create drift file for upstream; surface to user; user decides before Step 8. |
| Step 8 can't commit (test/clippy/fmt failure) | Investigate root cause; fix before commit. Never `--no-verify`. |

## Relationship to other process docs

- [`per-chunk-planning-template.md`](./per-chunk-planning-template.md) — the plan structure this checklist executes.
- [`drift-lifecycle.md`](./drift-lifecycle.md) — the drift-file state transitions triggered at Steps 2 and 8.
