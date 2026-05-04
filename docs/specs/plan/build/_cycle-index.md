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
