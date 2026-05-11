<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — verified-header re-stamp per plan §3.C; body content unchanged from CH-22 close. CH-24 milestone-seal cycle — page 13 ops surface re-verified PASS against current HEAD; symptoms + remediations + CH-01/CH-21/CH-22 amendments all still correct; cross-refs to ADR-0034/ADR-0035/ADR-0040 intact. Cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-04-28 by Claude Code (CH-21 amendment: drift D6.1 terminally remediated — memory-extraction-agent runtime-status tile now advances on every non-aborted SessionEnded; disabling the agent skips BOTH extraction AND telemetry fire per ADR-0040 §D40.3 SKIP-BOTH. For the memory-extraction listener runbook see `m5_2/operations/memory-extraction-operations.md`.) -->
<!-- CH-01 amendment (2026-04-27): disable / archive flip durable agent-row state — disable-then-bug-out semantics no longer "audit-only". See §"CH-01 amendment" below. -->
<!-- CH-22 amendment (2026-04-27): new `[listeners.catalog] audit_mode` config + `agent_catalog_refreshed` audit event (Debug mode only). New "catalog row stale" + "runtime-status tile not advancing" playbooks. See §"CH-22 amendment" below. -->

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

## CH-01 amendment — durable disable/archive (2026-04-27)

Pre-CH-01: `disable.rs` + `archive.rs` emitted audit events but did not mutate any column on the agent row. CH-01 ships migration 0007 with `agent.active: bool` + `agent.archived_at: option<string>` and rewires both handlers to flip those columns BEFORE the audit emit (ADR-0034 §D34.4 ordering rule).

Operator-visible consequences:

- A successful `POST /system-agents/:id/disable` is now idempotent at the durable level — re-issuing returns `200` and re-writes `active = false` (no state conflict).
- `POST /system-agents/:id/archive` writes `archived_at = Some(Utc::now())` and is also idempotent (later archive overwrites the timestamp). Standard system agents (memory-extraction + agent-catalog) still hard-fail with `409 STANDARD_SYSTEM_AGENT_NOT_ARCHIVABLE`.
- Audit-emit failures **do not roll back** the durable flip. If you see `AUDIT_EMIT_ERROR` after a successful disable/archive, the durable state IS correct — replay the audit chain rather than re-running the handler.
- The `agent_catalog_entry.active` column is the consumer of these durable fields (see CH-22 amendment + ADR-0034 §D34.5).

## CH-22 amendment — agent-catalog listener body shipped (2026-04-27)

### New config block

```toml
[listeners.catalog]
audit_mode = "silent"   # default; "debug" emits per-fire audit events
```

Override via `PHI_LISTENERS__CATALOG__AUDIT_MODE=debug`. Debug mode is intended for dev / acceptance investigations — production should leave it Silent (catalog refresh fires up to 8× per session and would 10–100× audit-log volume).

### New audit-event row

| Event | Class | Triggered by | Fields |
|---|---|---|---|
| `agent_catalog_refreshed` | Silent | `AgentCatalogListener` per fire (Debug mode only) | `refreshed_agent_id`, `triggering_event_id`, `triggering_event_kind` |

### New incident playbooks

- **Catalog row missing for a known agent.** Hit `GET /api/v0/orgs/:org/agents/:id` then check `repo.get_agent_catalog_entry(agent_id)` — if `None`, the listener never fired for this agent. Likely causes: (a) the agent was created via a code path that bypasses the production HTTP handlers (test fixture / direct repo call); (b) `state.event_bus` had no `AgentCatalogListener` subscribed at boot — verify [`build_event_bus_with_m5_listeners`](../../../../../../modules/crates/server/src/state.rs) ran. Recovery: emit a synthetic `DomainEvent::AgentCreated` from a maintenance task (M7b retry fabric will formalise this).
- **Catalog row's `active` column out of sync with durable agent state.** The listener computes `catalog_active = agent.active && agent.archived_at.is_none()` per fire. If the catalog row says `active = true` but the agent row's durable `active = false` or `archived_at = Some`, a fire was missed. Re-trigger by issuing a no-op profile patch (which emits `HasProfileEdgeChanged`) or by emitting `DomainEvent::AgentArchived` from a maintenance task.
- **Runtime-status tile for the catalog system agent stale.** `record_system_agent_fire` updates the tile on every catalog-listener fire. If `last_fired_at` lags real activity, either (a) no agent-lifecycle events are being emitted from the production handlers (confirm via debug-mode audit emit), or (b) the catalog system agent isn't resolvable from `org.system_agents` — verify exactly one entry has `display_name == "agent-catalog"`.
- **`agent_catalog_refreshed` flooding the audit log.** Operator left `audit_mode = "debug"` in production. Flip it back to `silent` via `PHI_LISTENERS__CATALOG__AUDIT_MODE=silent` + restart. Existing debug-mode rows are `AuditClass::Silent` (30-day retention) so they will age out.

## Cross-references

- [System agents architecture](../architecture/system-agents.md).
- [System flows s02 + s03 operations](system-flows-s02-s03-operations.md).
- [ADR-0034](../../m5_2/decisions/0034-agent-durable-lifecycle.md) — durable lifecycle (CH-01).
- [ADR-0035](../../m5_2/decisions/0035-agent-catalog-listener-audit-mode.md) — audit-mode + emit-site wiring (CH-22).
