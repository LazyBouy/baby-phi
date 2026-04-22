<!-- Last verified: 2026-04-22 by Claude Code -->

# User guide — Troubleshooting (M4)

**Status: [PLANNED M4/P8]** — fleshed out at P8 close with the full stable-code table.

Expected M4 stable codes (populated per-vertical):
- `AGENT_ID_IN_USE` (409)
- `AGENT_IMMUTABLE_FIELD_CHANGED` (409)
- `AGENT_ROLE_INVALID_FOR_KIND` (400)
- `PARALLELIZE_CEILING_EXCEEDED` (400)
- `EXECUTION_LIMITS_EXCEED_ORG_CEILING` (400)
- `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` (409)
- `SYSTEM_AGENT_READ_ONLY` (403)
- `PROJECT_ID_IN_USE` (409)
- `PROJECT_SHAPE_B_COOWNER_MUST_OVERLAP_CATALOGUE` (400)
- `LEAD_AGENT_NOT_IN_ORG` (400)
- `OKR_VALIDATION_FAILED` (400)

See:
- [M4 plan archive §P8](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [M3 troubleshooting](../../m3/user-guide/troubleshooting.md) — inherited codes.
- [Top-level runbook](../../../../../../docs/ops/runbook.md) — gets an M4 section at P8.
