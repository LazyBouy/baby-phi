<!-- Last verified: 2026-04-28 by Claude Code (CH-21 user-facing overview — what extraction does at v0, what M6 LLM upgrade adds, how to inspect Memory + Identity counters.) -->

# Memory extraction — operator overview

Each org has a built-in **memory-extraction-agent** system agent. Whenever an LLM agent finishes a session in your org, this agent records what happened by minting a `Memory` row attributed to the LLM agent, and bumping a counter on that agent's `Identity` row.

This page is the operator-facing summary. The architecture detail lives at [`memory-extraction-listener.md`](../architecture/memory-extraction-listener.md); the runbook lives at [`../operations/memory-extraction-operations.md`](../operations/memory-extraction-operations.md).

## What v0 extraction does

When an LLM agent's session ends naturally (not aborted):

- **One Memory row** is minted per session, attributed to the LLM agent that ran the session.
- **The Memory's tags** are derived from the session: `agent:<id>`, `session:<id>`, `project:<id>`, `org:<id>`, plus any tags the originating session already carried (e.g., `#public`).
- **The Identity counter advances**: `Identity.witnessed.memories_extracted` increments by 1, and the matching `extraction_scope_distribution.private` or `.public` bucket increments by 1.
- **Two audit events** are written to your org's audit log: `platform.memory.extracted` (Logged class) and `platform.identity.updated` (Logged class).
- **The runtime-status tile** for the memory-extraction-agent system agent advances (`last_fired_at` updates), visible on Page 13.

When the working agent is a **Human Agent**, the extractor does nothing — Human Agents have no Identity row in this system (per CH-16 / ADR-0038). When a session is **Aborted** (operator-terminated or cancelled), the extractor skips it.

## What v0 does NOT do (deferred to M6)

The concept describes a fully LLM-driven supervisor agent that reads a session transcript, identifies candidate memories with judgment, and routes each memory to one of four pools (`agent:` private / `project:` / `org:` / `#public`) based on content sensitivity. **At v0 this is a deterministic listener, not an LLM agent.** The LLM body lands at **M6-DEFERRED-04**.

Specifically:
- Per-memory pool routing collapses to a binary `private` vs `public` decision (`#public` tag → public; otherwise → private). Project-scoped and org-scoped memories count as `private` at v0.
- One Memory per session (the LLM body may emit N).
- No memory text content (the v0 `Memory` schema has no body field; content is encoded in tags).
- Supervisor-Extraction grants are bypassed; v0 writes via a privileged platform path.

## How to inspect what extraction has produced

### Memories minted for an agent

```sql
SELECT id, owning_agent, tags, created_at FROM memory
WHERE owning_agent = $agent_id ORDER BY created_at DESC;
```

Or via the repo trait: `repo.list_memories_for_agent(agent_id)` (returns newest-first).

### Identity counters for an agent

```sql
SELECT
  agent_id,
  witnessed.memories_extracted AS extracted,
  witnessed.extraction_scope_distribution.private AS private,
  witnessed.extraction_scope_distribution.public AS public,
  updated_at
FROM identity
WHERE agent_id = $agent_id;
```

### Recent extractions in the audit log

```sql
SELECT timestamp, target_entity_id, diff.scope_bucket, diff.session_id
FROM audit_events
WHERE event_type = 'platform.memory.extracted' AND org_scope = $org_id
ORDER BY timestamp DESC LIMIT 50;
```

## How to disable extraction

Disable the org's `memory-extraction-agent` via the system-agent disable surface (Page 13 / `POST /platform/system_agents/{id}/disable` from CH-01). While disabled:

- New `SessionEnded` events are observed but produce **no Memory, no Identity update, no audit, no telemetry tile fire**.
- Past sessions that ended while disabled are NOT replayed when you re-enable.
- The runtime-status tile retains the `last_fired_at` from the last fire while active — useful as forensic context.

Re-enable via the disable surface's `enable` counterpart.

## Concept references

- [`concepts/system-agents.md`](../../../concepts/system-agents.md) § "Memory Extraction Agent".
- [`concepts/agent.md`](../../../concepts/agent.md) § "Two Streams of Experience" — the `witnessed.*` fields this advances.
- [Architecture page](../architecture/memory-extraction-listener.md).
- [Operations runbook](../operations/memory-extraction-operations.md).
