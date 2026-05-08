<!-- Last verified: 2026-05-07 by Claude Code (CH-08-7cbe74a4 P4 chunk-seal: row Status flipped from `in-flight` to `ready-for-audit`; Auditors set to "2 (audit envelope: medium)" per plan §11; iteration count bumped from 0 → 1 for initial sub-agent dispatch; ADR-0052 flipped Proposed → Accepted; D-new-13 + D-new-29 lifecycle entries appended with CH-08 P4 remediation evidence; drifts/README.md rows 112 + 128 Status cells flipped to **remediated** with CH-08 ✓ marker.) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-08-7cbe74a4 P3 chunk-progress: P3 deliverables shipped — new architecture page `m5_2/architecture/allocate-transfer-cardinality.md` + new operations runbook `m5_2/operations/allocate-transfer-cardinality-operations.md` + `_concept-audit-matrix.md` rows for `permissions/02` "allocate vs transfer cardinality" (D-new-13) and `permissions/03` "`allocate` umbrella" (D-new-29) flipped silent-in-code → **honored** with letter-for-letter copy from plan §2 target column; concept-doc 02 + 03 verified-headers prepended with CH-08 amendment notes; ADR-0052 broken-link fixes for ADR-0022/0023/0028 paths (P0 paperwork bug surfaced at P3 doc-link guard run); 1408 tests passing (unchanged from P2 — pure docs phase); doc-link + ops-doc-headers CI guards both green) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-08-7cbe74a4 status flipped in-flight → retro-complete after Audit A iter 1 GREEN (11/12 substantive PASS, 1 sandbox-deferred) + Audit B iter 1 GREEN (8/8 PASS, observed 4 CI guards executing in-shell — PASS-with-caveat) + orchestrator final cycle re-audit GREEN + retrospective written + 8 standards updates applied (chunk-planner v5→v6; chunk-auditor v4→v5; chunk-retrospector v2→v3; per-chunk-planning-template §3 cascade-discipline extended; settings.json bash-check rule broadened mid-cycle by user; granular-bash-discipline doc §2.5 added; permissions-audit skill matcher-semantics-investigation escalation); 0 new follow-up drifts (first cycle in 4 to close cleanly without CH-NN-FOLLOWUP-01); iteration count stays 1 — orchestrator courtesy correction at gate 4 NOT a Trivial-1L FAIL per CLAUDE.md; cycle held during user halt window then resumed) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-08-7cbe74a4 row appended at chunk-open / P0; status `in-flight`; folder `ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/` already created at chunk-archive-plan; planner draft archived to plan.md verbatim 2026-05-07; ADR-0052 scaffold drafted as Proposed; 5 forks user-locked at plan-approval gate to F1.A / F2.A / F3.A / F4.A / F5.A; all-A path holds Direct-approval criteria) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-07-cc912d07 status flipped ready-for-audit → retro-complete after Audit A iter 1 GREEN (with PASS-with-note on `expansion.rs:55→56` line drift) + Audit B iter 1 GREEN + orchestrator final cycle re-audit GREEN + retrospective written + 7 standards updates applied (chunk-planner v4→v5; per-chunk-planning-template §3 cascade-artifact discipline; forward-scope row CH-07 retroactive informal-labeling parenthetical; settings.json bash-check rule paired colon-form + 2>&1 prefix; granular-bash-discipline doc §2.4 redirect+pipe quirk; permissions-audit skill §B regression-protection step) + D-CH07-FOLLOWUP-01 filed; iteration count stays 1 — orchestrator courtesy correction at gate 4 NOT a Trivial-1L FAIL per CLAUDE.md) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-07-cc912d07 row Status flipped from `in-flight` to `ready-for-audit` at P4 close; chunk implementation complete (P0+P1+P2+P3+P4 deliverables shipped per plan §7); ADR-0051 flipped Proposed → Accepted; D-new-06 + D-new-20 remediated; D-CH07-FOLLOWUP-01 filed; auditors set to "2 (audit envelope: medium)" per plan §11; iteration count bumped from 0 to 1 for initial sub-agent dispatch.) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-07-cc912d07 row appended at chunk-open / P0; status `in-flight`; folder `ch-07-multi-scope-cascade-contractor-model-cc912d07/` already created at chunk-archive-plan; planner draft archived to plan.md verbatim 2026-05-07; ADR-0051 scaffold drafted as Proposed) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-13-d4fe1b7c status flipped ready-for-audit → retro-complete after Audit A iter 1 GREEN + Audit B iter 1 TRIVIAL FAIL→ 2 Trivial-1L orchestrator inline patches at gate 4 + orchestrator final cycle re-audit GREEN + retrospective written + 6 standards updates applied + D-CH13-FOLLOWUP-01 filed; iteration count stays 1 — Trivial-1L per CLAUDE.md does NOT re-spawn auditors) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-13-d4fe1b7c row status flipped from `in-flight` to `ready-for-audit` at P3 close; iteration count remains 1; P0+P1+P2+P3 deliverables shipped per plan; chunk-implementer handed off to orchestrator for sub-agent audit dispatch.) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-13-d4fe1b7c row appended at chunk-open; status `in-flight`; folder `ch-13-audit-class-composition-strictest-wins-d4fe1b7c/` created via chunk-archive-plan skill; planner draft archived to plan.md verbatim) -->

# Cycle index — baby-phi multi-agent chunk pipeline

Pointer index for hex-tagged cycles under this directory. Each row points to a cycle folder; click through to the plan, audit logs, cycle audit, and retrospective.

**Convention**: cycles started under the multi-agent system (CH-11 onward) live in folders `<slug>-<8hex>/` with files `plan.md`, `audit-A-iter<N>.md`, `audit-B-iter<N>.md` (when applicable), `cycle-audit.md`, `retrospective.md`. Pre-existing cycles (CH-09, CH-10, CH-23, ...) are flat-file legacy and listed in a separate section below.

## Active cycles (folder-style, multi-agent system)

| Hex | Slug | Phases | Auditors | Iterations | Status | Retro |
|---|---|---|---|---|---|---|
| [`d5428c43`](ch-11-per-session-consent-gating-d5428c43/plan.md) | CH-11 — Per-Session consent gating | 4 | 2 (audit envelope: medium) | 1 (Trivial-1L orchestrator inline patch) | `retro-complete` (cycle GREEN; awaiting user commit) | [retrospective](ch-11-per-session-consent-gating-d5428c43/retrospective.md) |
| [`6a748175`](ch-12-frozen-session-tag-immutability-6a748175/plan.md) | CH-12 — Frozen session-tag immutability | 3 | 2 (audit envelope: medium) | 2 (Trivial-multi audit-fix re-spawn for `_concept-audit-matrix.md` row 191) | `retro-complete` (cycle GREEN; awaiting user commit) | [retrospective](ch-12-frozen-session-tag-immutability-6a748175/retrospective.md) |
| [`d4fe1b7c`](ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md) | CH-13 — `audit_class` composition (strictest wins) | 4 | 2 (audit envelope: medium) | 1 (2 Trivial-1L orchestrator inline patches on ADR-0050; no auditor re-spawn per CLAUDE.md trivial split) | `retro-complete` (cycle GREEN; awaiting user commit) | [retrospective](ch-13-audit-class-composition-strictest-wins-d4fe1b7c/retrospective.md) |
| [`cc912d07`](ch-07-multi-scope-cascade-contractor-model-cc912d07/plan.md) | CH-07 — Multi-scope cascade + contractor model | 5 | 2 (audit envelope: medium) | 1 (orchestrator courtesy-corrected `expansion.rs:55→56` line drift at gate 4 — NOT a Trivial-1L FAIL; no auditor re-spawn) | `retro-complete` (cycle GREEN; 7 standards updates landed; awaiting user commit) | [retrospective](ch-07-multi-scope-cascade-contractor-model-cc912d07/retrospective.md) |
| [`7cbe74a4`](ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/plan.md) | CH-08 — `allocate` / `transfer` cardinality + refinements | 5 | 2 (audit envelope: medium) | 1 (no re-spawns; cycle held during user halt window then resumed; auditor B observed 4 CI guards executing in-shell — PASS-with-caveat) | `retro-complete` (cycle GREEN; 8 standards updates landed; 0 new follow-up drifts; awaiting user commit) | [retrospective](ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/retrospective.md) |

## Legacy cycles (flat-file, pre-multi-agent)

These chunks were planned + executed before the multi-agent system landed. Their plan archives stay where they are — no churn.

| Hex | Slug | Plan file |
|---|---|---|
| varies | CH-01 through CH-10, CH-21, CH-22, CH-23 | see `*.md` files in this directory |

(Run `ls baby-phi/docs/specs/plan/build/*.md` to enumerate.)

## Closed cycles

(empty — populated as cycles complete)

## How to read this index

- **Status values**: `in-flight` (planning or implementation in progress) | `audit` (sub-agent audits running) | `final-audit` (orchestrator's final cycle re-audit) | `retro` (retrospective being written) | `closed` (committed, retro applied) | `paused` (escalated to user, no progress).
- **Iteration count**: highest audit iteration number observed (1 = no re-spawns; ≥ 2 = at least one re-audit).
- **Retro link**: `[hex](slug-hex/retrospective.md)` once retrospective exists.

## Maintenance

- **Add row** at chunk-open: planner agent or orchestrator appends.
- **Update row** at status transitions: orchestrator updates as cycle progresses.
- **Move to "Closed"** after user commits the cycle.
- **Never delete** — this is durable history.
