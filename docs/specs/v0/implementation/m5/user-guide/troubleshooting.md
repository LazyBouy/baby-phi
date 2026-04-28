<!-- Last verified: 2026-04-28 by Claude Code -->
<!-- CH-01 + CH-22 amendments (2026-04-27): durable disable/archive semantics + agent-catalog audit-mode + new "catalog row stale" symptom. See §"CH-01 + CH-22 amendments" below. Full M5/P9 stable-code table still deferred to M5-tag-close. -->
<!-- CH-06 amendment (2026-04-28): selector parse-error troubleshooting. See §"CH-06 amendment — selector parse errors" below. -->

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

## CH-06 amendment — selector parse errors (2026-04-28)

CH-06 lights up the full PEG tag-predicate DSL on `Selector` (concept-09). When a grant carries a new-grammar selector (e.g. `tags contains org:acme AND tags contains #kind:session`), the parser may surface parse errors. They appear as `Decision::Denied { failed_step: Match, reason: ... }` with stable codes `P-001`–`P-005`:

| Code | Symptom | Fix |
|---|---|---|
| `P-001` | `unexpected token '<X>' at position <N>` | Identifier doesn't match `[a-zA-Z_][a-zA-Z0-9_-]*` (often a value starting with a digit, like `a:1`). Rename to `a:one` or `a:x1`. |
| `P-002` | `unbalanced parens at position <N>` | Count parens; ensure each `(` has a matching `)` at the same depth. |
| `P-003` | `unknown predicate '<name>'` | Use one of the 6 supported predicates: `contains`, `intersects`, `any_match`, `subset_of`, `empty`, `non_empty`. |
| `P-004` | `invalid glob '<pattern>'` | `any_match` glob must contain `*` or `**`. Use `tags contains` for an exact tag match. |
| `P-005` | `unknown set reference '<name>'` (runtime) | Set-ref name not registered. In M5.2 the noop registry returns `false` for every set-ref; CH-15 wires the production registry. |

For full operational guidance see [m5_2/operations/selector-grammar-operations.md](../../m5_2/operations/selector-grammar-operations.md). For grammar syntax see [m5_2/user-guide/selector-syntax-guide.md](../../m5_2/user-guide/selector-syntax-guide.md).

## CH-16 amendment — Identity errors (2026-04-28)

CH-16 lights up the four-field `Identity` node (concept-`agent.md` § "Identity Node Content"). Two common operator-facing errors land at this surface:

| Symptom | Cause | Fix |
|---|---|---|
| `HumanAgentHasNoIdentity { agent_id }` from a handler | Caller tried to write an Identity for a Human-kind agent. Concept-`human-agent.md` § "No Identity" mandates this rejection. | This is by design. Skip the Identity write for Human kind; the create-time path already does this when `payload.identity` is `None`. If a test fixture or REPL probe hits this, change the call to skip Identity entirely for Human-kind agents. See [concept-`human-agent.md` §"No Identity"](../../../concepts/human-agent.md). |
| `apply_agent_creation` fails with `Llm-kind agent requires Some(Identity)` | Server orchestrator forgot to build the Identity row for an LLM-kind payload. | Build via `Identity::default_for_llm(agent_id, now)`. Production handlers at `agents/create.rs` + `system_agents/add.rs` already do this; only test fixtures or future custom orchestrators need to mirror the pattern. |

For full operational guidance see [m5_2/operations/identity-operations.md](../../m5_2/operations/identity-operations.md). For the four-field shape + creation timing see [m5_2/user-guide/identity-overview.md](../../m5_2/user-guide/identity-overview.md).

## CH-21 amendment — Memory-extraction errors (2026-04-28)

CH-21 ships the heuristic memory-extraction listener body (concept-`system-agents.md` § "Memory Extraction Agent"; ADR-0040). Two common operator-facing situations land at this surface:

| Symptom | Cause | Fix |
|---|---|---|
| Extraction not firing — sessions end but no `Memory` rows appear and `Identity.witnessed.memories_extracted` stays at 0 | The org's `memory-extraction-agent` system agent is `active = false` or `archived_at` is populated; per ADR-0040 §D40.3 SKIP-BOTH the listener short-circuits before minting a Memory or firing telemetry. | Re-enable the system agent via `POST /platform/system_agents/{id}/enable` (CH-01 surface). Past sessions that ended while disabled are NOT replayed — extraction is forward-only at v0. See [m5_2/operations/memory-extraction-operations.md](../../m5_2/operations/memory-extraction-operations.md) §"Operator-disabled extractor". |
| Identity counter is stale relative to the Memory rows that exist | `Repository::upsert_identity` errored after `create_memory` succeeded — fail-safe semantics (ADR-0028) leave the Memory durable while the Identity counter has a 1-event gap. | The next successful extraction self-heals (the counter is incremental, not derived). To force-resync, an operator-driven recompute path is M6 / future-CH work (see successor M6-DEFERRED-04). Forensic context: structured logs carry the `event_id` of the failing fire. |

For full operational guidance see [m5_2/operations/memory-extraction-operations.md](../../m5_2/operations/memory-extraction-operations.md). For the v0 heuristic body + what M6 LLM upgrade adds, see [m5_2/user-guide/memory-extraction-overview.md](../../m5_2/user-guide/memory-extraction-overview.md).

## Cross-references

- [Top-level runbook §M5](../../../../../../docs/ops/runbook.md) — operator-facing aggregated index (appended at P9).
- [M4 troubleshooting](../../m4/user-guide/troubleshooting.md) — inherited codes + cross-org isolation invariants.
- [M5 plan §P9 deliverables](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [M5.2 selector-grammar-operations](../../m5_2/operations/selector-grammar-operations.md) — CH-06 selector parse error runbook.
- [M5.2 identity-operations](../../m5_2/operations/identity-operations.md) — CH-16 Identity row runbook.
- [M5.2 memory-extraction-operations](../../m5_2/operations/memory-extraction-operations.md) — CH-21 memory-extraction listener runbook.
