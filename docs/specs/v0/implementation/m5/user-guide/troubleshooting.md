<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-01 + CH-22 amendments (2026-04-27): durable disable/archive semantics + agent-catalog audit-mode + new "catalog row stale" symptom. See §"CH-01 + CH-22 amendments" below. Full M5/P9 stable-code table still deferred to M5-tag-close. -->

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

## CH-01 + CH-22 amendments — symptoms & fixes (2026-04-27)

| Symptom | Likely cause | Fix |
|---|---|---|
| `phi system-agent disable` returns `200` but the agent still appears "active" in old monitoring tooling | Pre-CH-01 tooling reads only the audit log (which has always recorded the disable) but not the durable `agent.active` column | Update the tool to read the new `active` field on the agent row. Both `phi agent show --json` and `GET /api/v0/orgs/:org/agents/:id` now include `{active, archived_at}`. |
| `AUDIT_EMIT_ERROR` after a disable / archive request | Audit-emit path failed AFTER the durable column flip | Durable state IS correct — replay the audit chain rather than re-issuing the request. (CH-01 / ADR-0034 §D34.4 ordering rule.) |
| `agent_catalog_entry` row missing for a known agent | Listener never fired for this agent (likely created via test fixture / direct repo call that bypasses production HTTP handlers) | See [system-agents operations §"Catalog row missing"](../operations/system-agents-operations.md). |
| Audit log flooded with `agent_catalog_refreshed` events in production | `[listeners.catalog] audit_mode = "debug"` left enabled (typically post-investigation) | Set `PHI_LISTENERS__CATALOG__AUDIT_MODE=silent` and restart. Old debug-mode rows are `AuditClass::Silent` (30-day retention) and will age out. |
| Catalog system agent's runtime-status tile shows stale `last_fired_at` | Either (a) no agent-lifecycle events being emitted from production handlers (run a test write to verify), or (b) the catalog system agent isn't resolvable from `org.system_agents` (verify exactly one entry has `display_name == "agent-catalog"`) | See [system-agents operations §"Runtime-status tile stale"](../operations/system-agents-operations.md). |

## Cross-references

- [Top-level runbook §M5](../../../../../../docs/ops/runbook.md) — operator-facing aggregated index (appended at P9).
- [M4 troubleshooting](../../m4/user-guide/troubleshooting.md) — inherited codes + cross-org isolation invariants.
- [M5 plan §P9 deliverables](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
