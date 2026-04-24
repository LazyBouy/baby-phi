<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Page 13 system agents config

**Status**: `[EXISTS]` as of M5/P6.

Scope:

- List / tune / add / disable / archive handlers.
- `SystemAgentRuntimeStatus` runtime-status row + shared helper.
- Strong-warning audit class on disable-standard.

## Error-code reference

| HTTP | Code | Meaning | Fix |
|---|---|---|---|
| 400 | `SYSTEM_AGENT_INPUT_INVALID` | Empty display_name / profile_ref, or out-of-range value | Re-submit with valid shape |
| 400 | `DISABLE_CONFIRMATION_REQUIRED` | POST disable with `confirm: false` | Re-post with `confirm: true` |
| 400 | `TRIGGER_TYPE_INVALID` | Unknown trigger slug in add payload | Use `session_end`/`edge_change`/`periodic`/`explicit`/`custom_event` |
| 404 | `ORG_NOT_FOUND` | Unknown `org_id` | Verify id |
| 404 | `SYSTEM_AGENT_NOT_FOUND` | Agent id doesn't exist in the org | Verify id |
| 409 | `SYSTEM_AGENT_WRONG_KIND` | Target agent is not a system agent | Operate on `AgentRole::System` agents only |
| 409 | `PARALLELIZE_CEILING_EXCEEDED` | Requested parallelize > 32 (M5 cap) | Lower value; or raise cap post-M5 |
| 409 | `SYSTEM_AGENT_ID_IN_USE` | Add clashes with existing agent id | Choose a new display_name |
| 409 | `SYSTEM_AGENT_PROFILE_REF_UNKNOWN` | profile_ref doesn't resolve | Seed profile row first |
| 409 | `STANDARD_SYSTEM_AGENT_NOT_ARCHIVABLE` | Archive attempted on canonical memory-extraction / agent-catalog agent | Use disable instead |
| 500 | `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` | Internal failure | Check server logs |

## Audit event dictionary

| Event | Class | Triggered by | Fields |
|---|---|---|---|
| `platform.system_agent.reconfigured` | Alerted | tune | `before.parallelize`, `after.parallelize` |
| `platform.system_agent.added` | Logged | add | `profile_ref`, `parallelize`, `trigger` |
| `platform.system_agent.disabled` | Alerted | disable | `was_standard`, `profile_ref` |
| `platform.system_agent.archived` | Logged | archive | `profile_ref` |

## Incident playbooks

- **Queue runaway** — `SystemAgentRuntimeStatus.queue_depth`
  climbs without bound. At M5/P6 the helper seeds `queue_depth: 0`
  on every upsert — a climbing value means the M5/P8 memory /
  catalog listener bodies (when they ship) aren't keeping up OR
  the listener body is crashing before clearing the queue. Debug
  via listener log + the listener's `last_error` field.
- **Standard system agent disabled** — hard-blocked on
  archive but accepted on disable. Audit emits
  `platform.system_agent.disabled { was_standard: true, ... }`
  — the Alerted class surfaces it in the audit trail; operators
  re-enable by submitting a fresh `POST /system-agents` with the
  same profile_ref slug.
- **Tune rejected with PARALLELIZE_CEILING_EXCEEDED** — M5 cap is
  32. Raise via config at M7b when we add per-org overrides;
  don't patch the hard-coded constant without a corresponding
  migration + ADR.
- **Add rejected with TRIGGER_TYPE_INVALID** — operator typo'd
  the trigger slug. The five valid values are
  `session_end` / `edge_change` / `periodic` / `explicit` /
  `custom_event`.
- **Listener upsert stale** — the shared
  `record_system_agent_fire` helper logs `runtime-status tile
  stale` on upsert failure. At M5/P6 no listener calls this
  helper (deferred to P8 bodies per D6.1) so this log line should
  NOT appear at M5; if it does, a future phase has wired the
  helper into a listener without the matching fail-safe path.

## Metrics (M7 observability extensions)

At M5/P6 the routes emit through `axum-prometheus` standard HTTP
counters. M7b adds per-agent:

- `phi_system_agent_queue_depth{agent_id, org_id}` gauge.
- `phi_system_agent_fires_total{agent_id, outcome}` counter.
- `phi_system_agent_last_error{agent_id}` gauge/bool.

## Cross-references

- [System agents architecture](../architecture/system-agents.md).
- [System flows s02 + s03 operations](system-flows-s02-s03-operations.md).
