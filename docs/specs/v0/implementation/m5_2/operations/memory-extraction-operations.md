<!-- Last verified: 2026-04-28 by Claude Code (CH-21 ships the listener body — runbook covers operator-disable behaviour, fail-safe semantics, multi-pod caveat, and how to re-trigger after disable.) -->

# Operations — memory-extraction listener

Operational runbook for the memory-extraction listener body shipped at CH-21 ([`memory-extraction-listener.md`](../architecture/memory-extraction-listener.md)).

## What the listener produces per session

For every non-aborted `SessionEnded` for an LLM-kind working agent in an org whose `memory-extraction-agent` system agent is active:
- 1 row in the `memory` table (`owning_agent = session.started_by`, tags derived from session).
- `Identity.witnessed.memories_extracted += 1` on the working agent's row.
- `Identity.witnessed.extraction_scope_distribution.{private | public} += 1` (bucket from `#public` tag presence).
- 1 `platform.memory.extracted` audit event (Logged class).
- 1 `platform.identity.updated` audit event (Logged class).
- `system_agent_runtime_status.last_fired_at = ended_at` for the memory-extraction system agent in this org.

## Operator-disabled extractor

When an operator disables the `memory-extraction-agent` system agent (CH-01 surface — see [`m5/operations/system-agents-operations.md`](../../m5/operations/system-agents-operations.md)), the listener honors that intent immediately. Per ADR-0040 §D40.3:

- No Memory row is minted.
- No Identity counter advance.
- No audit emission.
- **No telemetry-tile fire** (the `system_agent_runtime_status` row stays at the last value from when the agent was active — useful as forensic context).

The listener still observes every `SessionEnded` event; it just short-circuits at the disabled-state guard. To **re-enable extraction**: re-activate the agent via `POST /platform/system_agents/{id}/enable` (CH-01 wiring). Past sessions that ended during the disabled window are NOT replayed — extraction is forward-only at v0.

## Operator playbook entries

### "Memory rows are not appearing for LLM agents"

1. Check the org's `memory-extraction-agent` is `active = true` and `archived_at IS NULL`:
   ```sql
   SELECT id, display_name, active, archived_at FROM agent
   WHERE display_name = 'memory-extraction-agent' AND owning_org = $org_id;
   ```
   If `active = false` or `archived_at` populated, follow "Operator-disabled extractor" above to re-enable.

2. Check the `system_agent_runtime_status` tile last_fired_at — confirms the listener has been triggered recently:
   ```sql
   SELECT * FROM system_agent_runtime_status
   WHERE owning_org = $org_id AND agent_id = $extractor_agent_id;
   ```
   If `last_fired_at` is stale relative to the last `SessionEnded`, the listener body bailed out before step 10 — check structured logs for `MemoryExtractionListener:` `warn` or `error` lines on the relevant `event_id`.

3. Confirm the working agent is `kind = 'llm'`. Per ADR-0040 §D40.5, Human-kind agents do NOT trigger extraction (they have no Identity row at v0 per ADR-0038/0039).

### "Memory row minted but Identity counter is stale"

Per ADR-0040 §D40.6 fail-safe semantics, if `Repository::upsert_identity` errors after `create_memory` succeeds, the Memory is durable and the Identity counter has a gap. The next successful extraction self-heals (the gap is one fire). To force-resync, run the operator-driven recompute path (M6 / future-CH; not exposed at v0 — see successor marker M6-DEFERRED-04).

### "Identity bumped but no Memory row"

This should not happen — the listener mints Memory before Identity update. If observed, check the `audit_events` table for `platform.memory.extracted` events corresponding to the `platform.identity.updated` events; a missing pair indicates a manually-inserted Identity row outside the listener's path (operator-driven repair, NOT extraction-driven).

## Multi-pod / EventBus caveat

Per ADR-0040 §D40.6, v0 assumes **single-pod** listener semantics. If the EventBus is replicated across pods (M7b broker carve-out per ADR-0033), the same `SessionEnded` could fire multiple extractions, producing duplicate Memory rows + double-counted Identity. The M7b broker design owns the cross-pod dedup contract; do NOT enable multi-pod EventBus until M7b ships.

## Audit chain ordering

For every fire, the audit log records the events in this order within the same `org_scope` hash chain:
1. `platform.session.ended` (emitted by `BabyPhiSessionRecorder` — pre-existing CH-02 surface).
2. `platform.memory.extracted` (CH-21 first emitter).
3. `platform.identity.updated` (CH-21 first emitter — ADR-0038 §D38.5 honored).

Acceptance test `scenario_6_audit_chain_orders_memory_extracted_before_identity_updated` pins this within a single fire; cross-fire ordering is by `timestamp` ascending.

## Concept refs

- [`concepts/system-agents.md`](../../../concepts/system-agents.md) § "Memory Extraction Agent".
- [ADR-0040](../decisions/0040-memory-extraction-listener-heuristic-v0.md).
- [ADR-0041](../decisions/0041-memory-extracted-event-and-audit.md).
- Drift [D6.1](../../m5_1/drifts/D6.1.md) (remediated at CH-21).
