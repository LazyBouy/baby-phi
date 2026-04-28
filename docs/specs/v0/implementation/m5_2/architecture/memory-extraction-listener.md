<!-- Last verified: 2026-04-28 by Claude Code (CH-21 lands the body — heuristic v0 per ADR-0040 + new `DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit per ADR-0041; first emitter of CH-16's `platform.identity.updated`.) -->

# Architecture — memory-extraction listener

CH-21 lands the body of the `MemoryExtractionListener` shipped at M5/P3 as a stub. The listener is one of two system-agent listeners that
cover concept-[`coordination.md`](../../../concepts/coordination.md) §"event-driven reactivity"; the other is the catalog listener
(CH-22, see [`m1/architecture/audit-events.md`](../../m1/architecture/audit-events.md) cross-ref). At v0 the body is **heuristic — no LLM call** — per ADR-[0040](../decisions/0040-memory-extraction-listener-heuristic-v0.md).

## What the listener does on `SessionEnded`

Pseudocode (real source: [`modules/crates/domain/src/events/listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs)):

```text
on SessionEnded { session_id, agent_id: working_agent, ended_at, … }:
  1. read working agent row; bail if missing or kind != Llm
  2. read SessionDetail; bail if missing or governance_state == Aborted
  3. resolve memory-extraction-agent system agent in working agent's org
  4. if extractor !active or archived → log warn + return (SKIP BOTH)
  5. mint Memory: tags = session.tags ∪ {agent:X, session:Y, project:Z, org:W}
  6. scope = if tags contains "#public" then Public else Private
  7. read working agent's Identity (CH-16 row); upsert with:
        witnessed.memories_extracted += 1
        witnessed.extraction_scope_distribution.{private|public} += 1
        updated_at = ended_at
  8. emit platform.memory.extracted (Logged) audit
  9. emit platform.identity.updated (Logged) audit  -- CH-16 first emitter
 10. record_system_agent_fire(extractor)            -- D6.1 first call site
```

Each step's failure logs and continues where coherent (ADR-0028 fail-safe semantics). Bailing out of step 7 (Identity-missing for an LLM
agent) still ships steps 5/6/8/10 — the Memory + memory-extracted audit + telemetry tile fire are all durable; only the Identity counter has a gap that the next extraction self-heals.

## Tag derivation (D40.2 DERIVE FROM SESSION TAGS)

The Memory's `tags` field is the deterministic union of:
- `Session.tags` — the CH-06 instance-identity set (`#kind:session`, `session:{id}`, plus user-supplied tags on the originating session).
- `agent:{owning_agent}` — the working agent (session's `started_by`).
- `session:{session_id}` — already present from CH-06; included idempotently.
- `project:{project_id}`.
- `org:{owning_org}`.

De-duplicated; tag order is deterministic (session.tags first, governance tags appended in fixed order).

The binary `{private, public}` scope decision is driven by the resulting tag set: `#public` present → `Public`, otherwise `Private`. The 4-pool routing concept-`system-agents.md` § "Allocation Rules" specifies (`agent:` / `project:` / `org:` / `#public`) is **NOT** implemented at v0 — the `Identity.witnessed.extraction_scope_distribution` carrier is a binary `{private, public}` field per CH-16 / ADR-0038. Project-scoped and org-scoped memories count as `private` at v0; LLM body lands the 4-pool routing in M6-DEFERRED-04.

## Disabled-state behaviour (D40.3 SKIP BOTH)

When the org's `memory-extraction-agent` system agent has `active = false` or `archived_at = Some(_)`, the listener logs `warn!` and returns. **Both** the extraction and the telemetry-tile fire are skipped — no `record_system_agent_fire` call, no Memory row, no Identity update, no audit emission.

The runbook entry is at [`m5_2/operations/memory-extraction-operations.md`](../operations/memory-extraction-operations.md) §"Operator-disabled extractor".

## Failure modes

| Step | Failure | Behavior |
|---|---|---|
| 1 | working agent missing | warn + return (no Memory minted) |
| 1 | working agent is Human-kind | debug + return (CH-16: no Identity to bump) |
| 2 | session not fetchable | warn + return |
| 2 | governance_state == Aborted | debug + return |
| 3 | extractor unresolvable in org | warn + return |
| 4 | extractor disabled/archived | warn + return (SKIP BOTH) |
| 5 | `Repository::create_memory` errors | error + abort the rest of the fire |
| 7 | Identity row missing for LLM agent | warn + skip Identity update; Memory + audit still ship |
| 7 | `Repository::upsert_identity` errors | error + skip Identity audit; Memory + memory.extracted audit + telemetry fire still ship |
| 8/9 | audit emit errors | error + continue (audit gap; row durable) |
| 10 | `record_system_agent_fire` errors | logged + swallowed inside the helper (telemetry tile stale) |

## Out of scope at v0 (M6-DEFERRED-04)

Per ADR-0040 §D40.7 — full LLM-driven supervisor agent body, 4-pool routing, Supervisor Extraction grant enforcement, multi-memory-per-session extraction, per-memory text content, `parallelize`-driven concurrent extractions.

## Concept references

- [`concepts/system-agents.md`](../../../concepts/system-agents.md) § "Memory Extraction Agent" — substrate clauses honored at v0; LLM-body clauses preserved silent-in-code.
- [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) § "Supervisor Extraction" — preserved silent-in-code; permission-checked extraction lands with M6-DEFERRED-04.
- [`concepts/agent.md`](../../../concepts/agent.md) § "Two Streams of Experience" — `witnessed.memories_extracted` + `extraction_scope_distribution.{private,public}` counters now reactively update.
- [`concepts/coordination.md`](../../../concepts/coordination.md) § "event-driven reactivity / runtime-status telemetry" — fully honored (both call sites in production).

## ADRs + drift

- [ADR-0040](../decisions/0040-memory-extraction-listener-heuristic-v0.md) — heuristic v0 listener body; LLM body deferred.
- [ADR-0041](../decisions/0041-memory-extracted-event-and-audit.md) — `DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit class.
- [Drift D6.1](../../m5_1/drifts/D6.1.md) — terminally remediated at CH-21 seal.
- [ADR-0038](../decisions/0038-identity-node-materialization.md) §D38.5 — first-emitter commitment for `IdentityUpdated` honoured here.
