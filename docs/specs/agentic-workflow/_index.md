<!-- Last verified: 2026-05-02 by Claude Code -->

# Agentic-workflow design docs

Index of meta-design documents that govern how baby-phi (and adjacent projects) decompose work into chunks, run chunks through multi-agent cycles, and refine the process via retrospectives. These are **process** docs — they describe how work happens, not the work itself.

## Active design

| Hex | Title | Date | Status |
|---|---|---|---|
| `0853574c` | [Multi-agent baby-phi chunk pipeline — agent set, skills, orchestration](multi-agent-chunk-pipeline-0853574c.md) | 2026-05-02 | Accepted |

## Conventions

- **Naming**: `<slug>-<8hex>.md` per memory `feedback_plan_archive_naming.md` (slug-first, hex suffix). Hex generated via `openssl rand -hex 4`.
- **Single-file or folder**: meta-plans land as a single flat file here. Per-cycle artifacts (plan + audit logs + cycle-audit + retrospective) live in `baby-phi/docs/specs/plan/build/<slug>-<8hex>/` folders.
- **Index discipline**: every new meta-design doc gets a row here at landing-time.
- **Status**: `Proposed` → `Accepted` (after user approval) → `Superseded` (when replaced; link replacement in row).
