<!-- Last verified: 2026-05-03 by Claude Code (added row for 18564835; 478b9384 status flipped to "Accepted (extended by 18564835)") -->

# Permissions design docs

Index of permission-policy / hook design documents that govern how Claude Code interacts with the `/root/projects/phi/` project. These are **policy** docs — they describe how the agent's tool calls are gated (allow rules, deny rules, defaultMode, hooks, telemetry, audits).

## Active design

| Hex | Title | Date | Status |
|---|---|---|---|
| `18564835` | [Tool-use logging + permissions-audit skill — closing the feedback loop](tool-use-logging-and-permissions-audit-skill-18564835.md) | 2026-05-03 | Accepted |
| `478b9384` | [Project permissions hardening — settings.json + hooks](project-permissions-hardening-478b9384.md) | 2026-05-03 | Accepted (extended by `18564835`) |

## Conventions

- **Naming**: `<slug>-<8hex>.md` per memory `feedback_plan_archive_naming.md` (slug-first, hex suffix). Hex generated via `openssl rand -hex 4`.
- **Single-file or folder**: policy plans land as a single flat file here. Per-revision iterations get fresh hex tokens (no in-place edits — older iterations stay for history).
- **Index discipline**: every new policy doc gets a row here at landing-time.
- **Status**: `Proposed` → `Accepted` (after user approval) → `Superseded` (when replaced; link replacement in row).

## Companion docs

- **Multi-agent workflow** at `baby-phi/docs/specs/agentic-workflow/` — the agent-pipeline meta-design that this permissions config supports.
- **CLAUDE.md** at the repo root + baby-phi root — operational guidance for the orchestrator + agents.
- Live config at `/root/projects/phi/.claude/settings.json` + `/root/projects/phi/.claude/hooks/*.sh`.

## When to revise

- After 2–3 multi-agent cycles, retrospective should compare prompts-per-cycle vs the pre-config baseline and flag any false-positive denials or false-negative escapes.
- When a new tool / agent / domain is added that needs explicit allow rules.
- When a sensitive new file path needs deny coverage.

Each revision generates a fresh hex token + adds a row above + flips the prior row's Status to `Superseded`.
