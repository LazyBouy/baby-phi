<!-- Last verified: 2026-04-22 by Claude Code -->

# M4 — Agents + Projects

Ships admin pages 08 (agent roster list), 09 (agent profile editor),
10 (project creation wizard — Shape A + Shape B), 11 (project detail).
First milestone to materialise the `Project` node + `HAS_LEAD` edge
in production. First milestone to introduce a two-approver Auth
Request flow (Shape B). First milestone to extend the agent surface
with the 6-variant `AgentRole` spanning both Human and LLM kinds.
First milestone to ship a domain event bus (`TemplateAFireListener`
subscribes to `HasLeadEdgeCreated` and fires the Template A grant
automatically).

Plan archive: [`../../../plan/build/a634be65-m4-agents-and-projects.md`](../../../plan/build/a634be65-m4-agents-and-projects.md).

## Phase status

| Phase | Status | Scope |
|---|---|---|
| P0 — Post-flight delta + concept doc + base plan amendment + docs tree seed | [PLANNED M4/P0] | archive plan; 10-item audit; `concepts/agent.md` 6-variant `AgentRole` amendment; base plan M5/M8 carryover entries; ontology audit; docs tree seed |
| P1 — Foundation: types, migration, web primitives, CLI scaffold | [PLANNED M4/P1] | `Project` / `ProjectShape` / `ProjectStatus` / `AgentRole` / `AgentExecutionLimitsOverride` / OKR value objects; migration 0004; `EventBus` trait + `DomainEvent`; ADRs 0024/0025/0027/0028 |
| P2 — Repository expansion + Template A pure-fn + M4 audit events | [PLANNED M4/P2] | 10 repo methods including `resolve_effective_execution_limits`; `fire_grant_on_lead_assignment` pure fn; 5 M4 audit builders |
| P3 — Compound tx + fixture + Shape B proptest + event bus wiring + TemplateAFireListener | [PLANNED M4/P3] | `apply_project_creation` + `apply_agent_creation`; 50-case Shape B matrix proptest; `InProcessEventBus` + listener wired into `AppState` |
| P4 — Page 08 vertical (Agent Roster List) | [PLANNED M4/P4] | Business logic + handler + CLI + Web + ops doc |
| P5 — Page 09 vertical (Agent Profile Editor + ExecutionLimits override) | [PLANNED M4/P5] | M4's phi-core-heaviest phase — 4 direct imports (AgentProfile, ExecutionLimits, ModelConfig, ThinkingLevel) |
| P6 — Page 10 vertical (Project Creation Wizard) | [PLANNED M4/P6] | Shape A + Shape B + 6-step web wizard; Template A fires automatically via subscription |
| P7 — Page 11 vertical (Project Detail) | [PLANNED M4/P7] | Read-only detail + in-place OKR edit |
| P8 — Seal: dashboard rewrite + cross-page acceptance + CI + runbook + independent 3-agent re-audit | [PLANNED M4/P8] | M3 dashboard counters reactivated; `acceptance_m4.rs`; target ≥99% composite |

## ADRs

| # | Title | Status |
|---|---|---|
| [0024](decisions/0024-project-and-agent-role-typing.md) | Project + AgentRole (6-variant) typing decisions | Proposed (→ Accepted at P1 close) |
| [0025](decisions/0025-shape-b-two-approver-flow.md) | Shape B two-approver flow | Proposed (→ Accepted at P6 close) |
| [0027](decisions/0027-per-agent-execution-limits-override.md) | Per-agent ExecutionLimits override (layers on ADR-0023) | Proposed (→ Accepted at P5 close) |
| [0028](decisions/0028-domain-event-bus.md) | In-process domain event bus + Template A subscription | Proposed (→ Accepted at P3 close) |

## phi-core leverage (per-phase)

Per the [leverage checklist](../m3/architecture/phi-core-leverage-checklist.md)'s
four-tier enforcement model. M4 adds one new phi-core wrap
(`AgentExecutionLimitsOverride` wraps `phi_core::context::execution::ExecutionLimits`)
and four new direct imports on page 09 (agent profile editor —
M4's phi-core-heaviest phase). Every other M4 surface is
phi-core-import-free by design. See [phi-core-reuse-map.md](architecture/phi-core-reuse-map.md)
for the durable table.

## Testing posture (plan §5)

Target: M3 close 633 Rust + 55 Web = **688** → M4 close **~865**
combined (+~140 Rust / +~38 Web). Per-phase close audit runs the same
3-aspect check (code correctness + docs accuracy + phi-core leverage)
with explicit % target; confidence reported before each next phase
opens.

## Cross-references

- [Base build plan §M4](../../../plan/build/36d0c6c5-build-plan-v01.md) — upstream scope definition.
- [M3/P6 close architecture doc](../m3/architecture/org-dashboard.md) §Deviations — the M4 carryovers M4 closes.
- [phi-core leverage checklist](../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
