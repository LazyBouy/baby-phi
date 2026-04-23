<!-- Last verified: 2026-04-23 by Claude Code -->

# M5 architecture overview

**Status**: [PLANNED M5/P1] — stub seeded at M5/P0; filled as each
surface lands. See the [plan archive](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md)
for the full P0–P9 scope.

M5 layers three verticals on the M4 foundation:

- **Session persistence** — governance `Session` / `LoopRecordNode`
  / `TurnNode` wrapping the three `phi_core::session::model::*`
  types per [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md).
- **Authority template adoption** — pages 12 drives approve /
  deny / adopt-inline / revoke-cascade across Templates A–E.
  Migration 0005 flips UNIQUE(name) → UNIQUE(kind) per
  [ADR-0030](../decisions/0030-template-node-uniqueness.md).
- **First session launch** — page 14 wires
  `phi_core::agent_loop` with cancellation + bounded concurrency
  per [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).

Plus three reactive supervisors — `memory-extraction-agent` (s02),
`agent-catalog-agent` (s03), and the extended Template-A fire
listener (s05).

## Commitment ledger

See the plan archive §Part 2 for the 27-commitment ledger. As each
phase closes, the corresponding commitment row flips to ✅ in the
[README phase status table](../README.md#phase-status).

## Cross-references

- [m4-postflight-delta.md](m4-postflight-delta.md) — the P0 verification audit.
- [phi-core-reuse-map.md](phi-core-reuse-map.md) — per-page leverage table.
- [session-persistence.md](session-persistence.md) — 3-wrap pattern.
- [session-launch.md](session-launch.md) — page 14 flow.
- [authority-templates.md](authority-templates.md) — page 12 flow.
- [system-agents.md](system-agents.md) — page 13 flow.
- [shape-b-materialisation.md](shape-b-materialisation.md) — C-M5-6 pre/post.
- [event-bus-m5-extensions.md](event-bus-m5-extensions.md) — 8 new `DomainEvent` variants.
