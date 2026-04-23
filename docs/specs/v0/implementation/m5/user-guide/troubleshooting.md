<!-- Last verified: 2026-04-23 by Claude Code -->

# User guide — Troubleshooting (M5)

**Status**: [PLANNED M5/P9] — stub seeded at M5/P0; full stable-code
table + CLI exit codes + cross-org isolation invariants land at
P9 close, mirroring the M4/P8 troubleshooting pattern.

Every M5 HTTP error carries a JSON body `{ "code": "<STABLE_CODE>",
"message": "..." }`. The CLI surfaces the `code` verbatim via
`phi: rejected (<CODE>): <message>` + maps it to one of the exit
codes pinned in [`cli-reference-m5.md`](cli-reference-m5.md).

## Session surface (page 14) — placeholder

Full table lands at P9. Codes to expect (per plan §G12-G18):

- `PARALLELIZE_CAP_REACHED` (409)
- `SESSION_WORKER_SATURATED` (503)
- `AGENT_NOT_FOUND` (404)
- `PROJECT_NOT_FOUND` (404)
- `PERMISSION_CHECK_FAILED_AT_STEP_N` (403 / 400)
- `MODEL_RUNTIME_UNRESOLVED` (400)
- `SESSION_NOT_FOUND` (404)
- `SESSION_ALREADY_TERMINAL` (409)
- `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` (409) — M4 code, becomes reachable at M5/P4 (C-M5-5 flip)

## Authority templates surface (page 12) — placeholder

Full table lands at P9.

## System agents surface (page 13) — placeholder

Full table lands at P9.

## Cross-cutting codes (inherited)

Full table in [`../../m4/user-guide/troubleshooting.md`](../../m4/user-guide/troubleshooting.md):

- `VALIDATION_FAILED` (400)
- `AUDIT_EMIT_FAILED` (500)
- `UNAUTHENTICATED` (401)
- `INTERNAL` (500)

## Cross-references

- [Top-level runbook §M5](../../../../../../docs/ops/runbook.md) — operator-facing aggregated index (appended at P9).
- [M4 troubleshooting](../../m4/user-guide/troubleshooting.md) — inherited codes + cross-org isolation invariants.
- [M5 plan §P9 deliverables](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
