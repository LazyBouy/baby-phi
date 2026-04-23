<!-- Last verified: 2026-04-23 by Claude Code -->

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
[C-M6-1 carryover](../../../../plan/build/36d0c6c5-build-plan-v01.md).
Draft the `MemoryExtracted` tag shape at P8 + confirm at P8 close
so the audit replay is consumable by M6 without re-extraction.

## Cross-references

- [Event bus M5 extensions](../architecture/event-bus-m5-extensions.md).
- [M5 plan §P8](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [Base plan §M6 §Carryovers from M5](../../../../plan/build/36d0c6c5-build-plan-v01.md).
