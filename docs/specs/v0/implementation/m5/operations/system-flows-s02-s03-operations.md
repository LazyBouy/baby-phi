<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-22 amendment (2026-04-27): s03 (agent-catalog) listener body shipped — no longer P8-stub. See §"CH-22 amendment — s03 status update" below. CH-21 (s02 memory-extraction) still pending. -->

# Operations — System flows s02 (memory extraction) + s03 (agent catalog) + s05 (template C/D fires)

**Status**: [PLANNED M5/P8] — stub seeded at M5/P0; filled at P8
when listener bodies + acceptance suites ship.

Scope at M5/P8:

- `MemoryExtractionListener` — runs supervisor `agent_loop` per
  `SessionEnded`, emits `MemoryExtracted` audit with structured
  tag list + session reference.
- `AgentCatalogListener` — upserts `AgentCatalogEntry` per 8
  trigger variants.
- `TemplateCFireListener` / `TemplateDFireListener` — already
  bodied at P3; P8 confirms via s05 acceptance.

## Failure modes (land at P8)

- **Extraction queue saturated** → skip with
  `MemoryExtractionSkipped { reason: queue_saturated }`.
- **Extraction agent disabled** → skip with
  `MemoryExtractionSkipped { reason: agent_disabled }`.
- **LLM API error** → retry 3× with exponential backoff → final
  failure as `MemoryExtractionFailed`.
- **Catalog upsert on stale edge** — idempotent (upsert, not
  insert); safe to replay.

## M6 carryover — C-M6-1

M5 emits `MemoryExtracted` audit events with full structured tag
list (agent / group / project / org + custom `#tags`). M6
materialises `Memory` nodes from the audit stream per the
[C-M6-1 carryover](../../../../plan/build/build-plan-v01-36d0c6c5.md).
Draft the `MemoryExtracted` tag shape at P8 + confirm at P8 close
so the audit replay is consumable by M6 without re-extraction.

## CH-22 amendment — s03 status update (2026-04-27)

s03 (agent-catalog) is no longer P8-stubbed. The listener body landed at CH-22 and is wired into the production HTTP path via 6 emit sites (ADR-0035 §D35.5). Catalog rows now mutate on every agent-lifecycle event from `agents/create`, `agents/update`, `system_agents/{add,disable,archive}`, and `orgs/create`. Runtime-status tile for the catalog system agent advances on every fire (drift D6.1 second call site; CH-21 ships the first call site for memory-extraction).

s02 (memory-extraction) and s05 (Template C/D fires) status is unchanged from M5/P8 plan:

- **s02** — Listener body still pending; CH-21 owns. Listener subscription exists; body is a stub that logs on `SessionEnded`.
- **s05** — Listener bodies shipped at M5/P3 (Template C + D fire-listeners). CH-22 did not touch.

### Failure modes (s03, post-CH-22)

The placeholder s03 failure modes from the M5/P8 stub are superseded by the production-grade playbooks in [system-agents operations](system-agents-operations.md) §"CH-22 amendment". Specifically:

- "Catalog upsert on stale edge — idempotent" → confirmed by ADR-0035 §D35.5 (upsert keyed on `agent_id`'s UNIQUE INDEX from migration 0005).
- "Catalog row missing for known agent" → see system-agents-operations.md playbook.
- "Runtime-status tile stale" → see system-agents-operations.md playbook.

### s02 failure modes

Still planned at CH-21 close — placeholder list (queue saturation, agent disabled, LLM API error → 3× backoff retry, idempotent re-replay) carries forward unchanged.

## Cross-references

- [Event bus M5 extensions](../architecture/event-bus-m5-extensions.md).
- [System agents operations](system-agents-operations.md) — production playbooks for s03 catalog refresh.
- [ADR-0035](../../m5_2/decisions/0035-agent-catalog-listener-audit-mode.md) — CH-22 audit-mode + emit-site wiring.
- [M5 plan §P8](../../../../plan/build/m5-templates-system-agents-sessions-01710c13.md).
- [CH-22 plan](../../../../plan/build/ch-22-agent-catalog-listener-body-c5f201bb.md).
- [Base plan §M6 §Carryovers from M5](../../../../plan/build/build-plan-v01-36d0c6c5.md).
