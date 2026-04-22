<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s02 — Session-End Memory Extraction

## Purpose

The `memory-extraction-agent` (one of the two standard System Agents) reads the transcript of a just-ended session and produces Memory nodes routed to the appropriate pool (agent-private / project-public / org-public / #public) based on the source session's tags and content classification. This is how organisational learning accrues without per-working-agent overhead.

## Trigger

`AgentEvent::SessionEnd { session_id, agent_id, project_id?, ended_at, reason }` on the event stream.

## Preconditions

- The `memory-extraction-agent` is provisioned for the owning org (added at org creation; tunable on [admin/13-system-agents-config.md](../admin/13-system-agents-config.md)) and is in `running` state.
- The extraction agent holds Grants A (`[read]` on `session_object` scoped to this org) and B (`[store]` on `memory_object` scoped to this org) per [concepts/permissions/05 § Supervisor Extraction as Two Standard Grants](../../concepts/permissions/05-memory-sessions.md#supervisor-extraction-as-two-standard-grants).
- The extraction agent has available `parallelize` capacity (configurable; default 2).

## Behaviour

- **R-SYS-s02-1:** On receiving a `SessionEnd` event, the flow SHALL enqueue the session for extraction. If `parallelize` capacity is saturated, the session queues and is processed in FIFO order.
- **R-SYS-s02-2:** The extraction agent SHALL read the session's transcript (using Grant A), analyse for candidate memories per its AgentProfile system prompt, and for each candidate choose an allocation pool per the rule table in [concepts/system-agents.md § Memory Extraction Agent § Allocation Rules](../../concepts/system-agents.md#allocation-rules-where-a-memory-goes).
- **R-SYS-s02-3:** Each chosen pool produces a Memory node tagged appropriately (`agent:{id}`, `project:{id}`, `org:{id}`, `#public`) plus a `derived_from:session:{source_id}` tag for traceability.
- **R-SYS-s02-4:** The flow SHALL emit one audit event `MemoryExtracted { session_id, memory_id, pool, extractor_agent }` per extracted memory (a session may produce 0..N memories).
- **R-SYS-s02-5:** If the session carries multiple `org:` tags (Shape B — see [concepts/permissions/06 § Multi-Scope Session Access](../../concepts/permissions/06-multi-scope-consent.md#multi-scope-session-access)), extraction SHALL respect per-co-owner authority: the agent extracts only to pools authorised by Grants it holds under the relevant co-owner. In the canonical [projects/03-joint-research.md](../../projects/03-joint-research.md) case, the extraction agent belongs to the owning org that contains it, and its grants are scoped accordingly.
- **R-SYS-s02-6:** For a Shape E forbidden session (multi-project AND multi-org), extraction is skipped with a `MemoryExtractionSkipped { session_id, reason: shape_e }` audit event (should not occur, but is defended against).

## Side effects

- 0..N `Memory` nodes created with appropriate tag sets.
- 0..N `MemoryExtracted` audit events (logged unless the containing org has alerted default).
- Queue depth metric updates.
- Supervisor's Witnessed Experience struct updated per [concepts/agent.md § Witnessed Experience Is Mediated by Extraction](../../concepts/agent.md#witnessed-experience-is-mediated-by-extraction) — concurrent per-extraction updates with LWW consistency.

## Failure modes

- **Extraction agent disabled** ([admin/13 W3](../admin/13-system-agents-config.md#6-write-requirements)) → extraction is skipped; a `MemoryExtractionSkipped { reason: agent_disabled }` event fires per session end; admin can re-enable to restart.
- **Queue saturation** (`parallelize` cap) → sessions queue; metric `memory_extraction_queue_depth` rises; alert threshold configurable.
- **LLM API error on extraction** → retry with exponential backoff up to 3 attempts; after final failure, emit `MemoryExtractionFailed` alerted event; session moves to a dead-letter queue reviewable by the admin.
- **Pool permission denied** → if the agent lacks Grant B on a pool the content would naturally belong in, that specific memory is dropped; `MemoryExtractionPoolDenied { session_id, pool, reason }` event emitted (not fatal).

## Observability

- Audit events: `MemoryExtracted`, `MemoryExtractionSkipped`, `MemoryExtractionFailed`, `MemoryExtractionPoolDenied`.
- Metrics: `phi_memory_extraction_queue_depth`, `phi_memory_extraction_duration_seconds`, `phi_memory_extraction_memories_per_session`, `phi_memory_extraction_failures_total{reason=...}`.

## Cross-References

**Concept files:**
- [concepts/system-agents.md § Memory Extraction Agent](../../concepts/system-agents.md#memory-extraction-agent).
- [concepts/permissions/05 § Supervisor Extraction as Two Standard Grants](../../concepts/permissions/05-memory-sessions.md#supervisor-extraction-as-two-standard-grants).
- [concepts/agent.md § Memory Model — Public, Private, and Supervisor Extraction](../../concepts/agent.md#memory-model--public-private-and-supervisor-extraction).

**Admin page provisioning this flow:**
- [admin/13-system-agents-config.md](../admin/13-system-agents-config.md) — configures, tunes, disables.

**Related flows:**
- [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md) — extraction agent's grants were themselves issued through Auth Requests.
