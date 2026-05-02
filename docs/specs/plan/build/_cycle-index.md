<!-- Last verified: 2026-05-02 by Claude Code -->

# Cycle index — baby-phi multi-agent chunk pipeline

Pointer index for hex-tagged cycles under this directory. Each row points to a cycle folder; click through to the plan, audit logs, cycle audit, and retrospective.

**Convention**: cycles started under the multi-agent system (CH-11 onward) live in folders `<slug>-<8hex>/` with files `plan.md`, `audit-A-iter<N>.md`, `audit-B-iter<N>.md` (when applicable), `cycle-audit.md`, `retrospective.md`. Pre-existing cycles (CH-09, CH-10, CH-23, ...) are flat-file legacy and listed in a separate section below.

## Active cycles (folder-style, multi-agent system)

| Hex | Slug | Phases | Auditors | Iterations | Status | Retro |
|---|---|---|---|---|---|---|
| _none yet — first folder-style cycle will be the next chunk landed under the new system_ | | | | | | |

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
