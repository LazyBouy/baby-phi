<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — M4 phi-core reuse map

**Status: [PLANNED M4/P1]** — populated as each phase ships. See the
durable M3 reuse map at [m3/architecture/phi-core-reuse-map.md](../../m3/architecture/phi-core-reuse-map.md)
for the cumulative platform + M3 leverage table; this doc appends
M4's per-page additions.

## Page 08 — Agent Roster List

Per-page table populated at P4 close.

## Page 09 — Agent Profile Editor (M4's phi-core-heaviest phase)

Expected direct imports at P5 close:
- `use phi_core::agents::profile::AgentProfile;`
- `use phi_core::context::execution::ExecutionLimits;`
- `use phi_core::provider::model::ModelConfig;`
- `use phi_core::types::ThinkingLevel;`

Per-page table populated at P5 close.

## Page 10 — Project Creation Wizard

Expected direct imports: **0** (pure phi governance composition).
Per-page table populated at P6 close.

## Page 11 — Project Detail

Expected direct imports: **0** (pure phi governance reads; "Recent
sessions" panel reads phi's governance `Session` node populated
at M5, not `phi_core::Session`).

Per-page table populated at P7 close.

## `AgentExecutionLimitsOverride` (M4/P1)

New wrap on top of M3's `OrganizationDefaultsSnapshot.execution_limits`
pattern. Opt-in per-agent override; default path = no row = inherit
from snapshot per ADR-0023. ADR-0027 pins the layered design.

Per the leverage checklist's §6 positive close-audit discipline, a
compile-time coercion test (`fn is_phi_core_execution_limits(_:
&phi_core::context::execution::ExecutionLimits) {}`) applies to
`AgentExecutionLimitsOverride.limits` at P1 close.

## Domain event bus

Expected direct imports: **0** — orthogonal to `phi_core::AgentEvent`
(agent-loop telemetry). phi's `DomainEvent` is a governance
reactive-trigger primitive with no phi-core counterpart.

See:
- [M4 plan archive §Part 1.5](../../../../plan/build/a634be65-m4-agents-and-projects.md) — phi-core reuse map up-front.
- [phi-core leverage checklist](../../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
- [M3 reuse map](../../m3/architecture/phi-core-reuse-map.md) — cumulative platform + M3 table.
