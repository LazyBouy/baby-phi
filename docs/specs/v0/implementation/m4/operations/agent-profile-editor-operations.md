<!-- Last verified: 2026-04-22 by Claude Code -->

# Operations — Agent Profile Editor (page 09)

**Status: [PLANNED M4/P5]** — fleshed out at P5 close.

Ops runbook covering: create/edit/revert-limits HTTP flows, the three `ExecutionLimits` paths (inherit / override / revert), ADR-0027 invariant enforcement, active-session guard (409 `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`), audit event diff shape for `AgentProfileUpdated`.

See:
- [agent-profile-editor.md](../architecture/agent-profile-editor.md).
- [ADR-0027](../decisions/0027-per-agent-execution-limits-override.md).
- [M4 plan archive §P5](../../../../plan/build/a634be65-m4-agents-and-projects.md).
