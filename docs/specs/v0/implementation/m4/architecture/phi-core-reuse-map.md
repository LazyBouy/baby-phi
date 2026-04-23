<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — M4 phi-core reuse map

**Status**: [EXISTS] — fully populated at M4/P8 close. This doc is the
durable per-page record of every phi-core reuse decision M4 made +
the close-audit greps that validate each one. Appends to the M3 reuse
map at [`../../m3/architecture/phi-core-reuse-map.md`](../../m3/architecture/phi-core-reuse-map.md).

Legend:
- **✅ direct** — `use phi_core::X` followed by usage of the type as-is.
- **🔌 wrap** — baby-phi composite holds a `phi_core::X` field (e.g. `AgentProfile.blueprint`).
- **♻ inherit** — surface reads a wrap/direct import established at an
  earlier milestone; no new phi-core coupling introduced.
- **🏗 build-native** — pure baby-phi governance primitive, no phi-core
  counterpart exists.

## M4 aggregate table — what each page touches

| Page | Phase | Q1 direct imports | Q2 transitive wraps | Q3 rejections |
|---|---|---|---|---|
| 08 — Agent Roster | P4 | 0 in [`platform/agents/list.rs`](../../../../../../modules/crates/server/src/platform/agents/list.rs) | `Agent.blueprint` (♻ from M3/P1) | `phi_core::Session` (M5), `phi_core::AgentTool` (M5) |
| 09 — Agent Profile Editor | P5 | **4** in [`platform/agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) + [`create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs) | `AgentProfile.blueprint` (🔌 wrap), `AgentExecutionLimitsOverride.limits` (🔌 wrap) | `ContextConfig` / `RetryConfig` per-agent override (stays org-default per ADR-0023) |
| 10 — Project Creation | P6 | 0 in [`platform/projects/create.rs`](../../../../../../modules/crates/server/src/platform/projects/create.rs) | 0 at the wire tier (project is governance-only) | `phi_core::Session` (M5), `AgentTool` (M5), `Usage` (governance-level budget) |
| 11 — Project Detail | P7 | 0 in [`platform/projects/detail.rs`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) | 0 at the wire tier (roster row strips blueprint by design) | `phi_core::Session` / `LoopRecord` / `Turn` (M5 / C-M5-3) |
| 07 — Dashboard retrofit | P8 | 0 (unchanged from M3/P5) | Adds `ProjectShape` + `AgentRole` enums transit; both are baby-phi governance enums | No new rejections |

**Aggregate**: M4 adds 4 direct phi-core imports (all on page 09) + 1
new wrap (`AgentExecutionLimitsOverride`) + zero new transitive
surfaces. Every other M4 surface is phi-core-import-free by design.

## Page 08 — Agent Roster List (P4)

| Surface | phi-core type | Mode |
|---|---|---|
| Agent row wire shape | — | 🏗 build-native (`AgentRosterItemWire` strips `blueprint` at the list tier) |
| Filter query: `list_agents_in_org_by_role` | — | 🏗 build-native |

Positive close-audit grep: `grep -En '^use phi_core::'
modules/crates/server/src/platform/agents/list.rs` → 0 lines.

## Page 09 — Agent Profile Editor (P5) — M4's phi-core-heaviest phase

| Surface | phi-core type | Mode |
|---|---|---|
| Create body field `blueprint` | `phi_core::agents::profile::AgentProfile` | ✅ direct |
| Create body field `initial_execution_limits_override` | `phi_core::context::execution::ExecutionLimits` | ✅ direct |
| Update patch body field `blueprint` | `phi_core::agents::profile::AgentProfile` | ✅ direct (full replacement, no diff structure at wire layer) |
| Update patch body field `execution_limits` | `phi_core::context::execution::ExecutionLimits` via `ExecutionLimitsPatchWire` | ✅ direct |
| `ThinkingLevel` form dropdown | `phi_core::types::ThinkingLevel` | ✅ direct |
| `ModelConfig.id` validation | `phi_core::provider::model::ModelConfig` | ✅ direct (id-field check only; shape transit happens via `AgentProfile.blueprint`) |
| `AgentExecutionLimitsOverride.limits` storage | `phi_core::context::execution::ExecutionLimits` | 🔌 wrap (via `AgentExecutionLimitsOverride` composite in `domain/src/model/composites_m4.rs`) |
| `resolve_effective_limits` return | `phi_core::context::execution::ExecutionLimits` | ♻ direct return (phi-core's type is the return contract) |

Compile-time coercion witnesses (in [`update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs)):

```rust
#[allow(dead_code)] fn _is_phi_core_agent_profile(_: &PhiCoreAgentProfile) {}
#[allow(dead_code)] fn _is_phi_core_execution_limits(_: &ExecutionLimits) {}
#[allow(dead_code)] fn _is_phi_core_thinking_level(_: &ThinkingLevel) {}
```

Positive close-audit grep: `grep -En '^use phi_core::'
modules/crates/server/src/platform/agents/{create,update,execution_limits}.rs`
→ 7 lines (4 unique types, duplicated across files by import scope).

## Page 10 — Project Creation (P6)

| Surface | phi-core type | Mode |
|---|---|---|
| `CreateProjectRequest` wire | — | 🏗 build-native |
| Shape B Auth Request | — | 🏗 build-native (phi governance AR) |
| `HasLeadEdgeCreated` domain event | — | 🏗 build-native (orthogonal to `phi_core::AgentEvent` per `phi/CLAUDE.md`) |
| Template A grant issuance | — | 🏗 build-native |

Positive close-audit grep: `grep -En '^use phi_core::'
modules/crates/server/src/platform/projects/` → 0 lines.

Schema-snapshot invariant: `acceptance_projects_create` tests the wire
contract; no phi-core-wrapping fields leak at any depth.

## Page 11 — Project Detail (P7)

| Surface | phi-core type | Mode |
|---|---|---|
| `ProjectDetail` wire | — | 🏗 build-native (roster row deliberately strips `blueprint`) |
| `RosterMember` | — | 🏗 build-native |
| `RecentSessionStub` placeholder | — | 🏗 build-native (M4 always empty; M5/C-M5-3 flips to real rows of baby-phi's governance `Session` — NOT `phi_core::Session` per D11) |
| OKR patch shape | — | 🏗 build-native |
| `platform.project.okr_updated` audit | — | 🏗 build-native |

Positive close-audit grep: `grep -En '^use phi_core::'
modules/crates/server/src/platform/projects/detail.rs` → 0 lines.

Schema-snapshot invariant: `detail::tests::wire_shape_strips_phi_core`
+ `acceptance_projects_detail::show_happy_path` assert the serialised
JSON contains no `blueprint` / `execution_limits` / `defaults_snapshot`
/ `context_config` / `retry_config` keys at any depth.

## Page 07 retrofit — Dashboard (P8)

| Surface | phi-core type | Mode |
|---|---|---|
| `AgentsSummary` 6 + unclassified buckets | — | 🏗 build-native (powered by `AgentRole` enum, governance-only) |
| `ProjectsSummary.shape_a/b` counters | — | 🏗 build-native (`count_projects_by_shape_in_org` repo method) |
| `ViewerRole::ProjectLead` | — | 🏗 build-native (`list_projects_led_by_agent` + org-intersection in-process) |

Positive close-audit grep: `grep -En '^use phi_core::'
modules/crates/server/src/platform/orgs/dashboard.rs` → 0 lines.

Schema-snapshot invariant: `dashboard_summary_wire_shape_excludes_phi_core_fields`
covers every release; adding a phi-core-wrapping field here breaks the
test loudly.

## `AgentExecutionLimitsOverride` composite (M4/P1)

Layered on top of M3's `OrganizationDefaultsSnapshot.execution_limits`
wrap pattern. Opt-in per-agent override; default path = no row =
inherit from snapshot per ADR-0023. ADR-0027 pins the layered design.

```rust
// domain/src/model/composites_m4.rs
pub struct AgentExecutionLimitsOverride {
    pub id: NodeId,
    pub owning_agent: AgentId,
    pub limits: phi_core::context::execution::ExecutionLimits, // 🔌 wrap
    pub created_at: DateTime<Utc>,
}
```

Invariant: `limits.<every_field> <= org_snapshot.<every_field>`.
Enforced at the repository layer (`set_agent_execution_limits_override`
precondition).

## Domain event bus (M4/P1 + P3)

```rust
// domain/src/events/{mod,bus,listeners}.rs
pub enum DomainEvent {
    HasLeadEdgeCreated { project: ProjectId, lead: AgentId, at: DateTime<Utc>, event_id: AuditEventId },
}

pub trait EventBus: Send + Sync {
    async fn emit(&self, ev: DomainEvent);
    fn subscribe(&self, handler: Arc<dyn EventHandler>);
}
```

Zero phi-core imports. Orthogonal to `phi_core::types::event::AgentEvent`
per `phi/CLAUDE.md` §Orthogonal surfaces:

> - `domain::audit::AuditEvent` (governance write log, hash-chain,
>   retention tier) vs `phi_core::types::event::AgentEvent`
>   (agent-loop telemetry stream)

A `DomainEvent` fires exactly once per governance state transition; an
`AgentEvent` fires many times per session-loop iteration. Different
consumers, different lifecycles, different retention policies.

## Four-tier enforcement — P8 close-audit record

Per the [phi-core leverage checklist §6](../../m3/architecture/phi-core-leverage-checklist.md):

1. **CI grep (`check-phi-core-reuse.sh`)** — green at P8 close. 0
   forbidden redeclarations under `modules/crates/`.
2. **Q1/Q2/Q3 structural** — tables above cover every M4 surface.
3. **ADRs** — 0024 (Project+AgentRole typing), 0025 (Shape B 2-approver),
   0027 (per-agent ExecutionLimits override), 0028 (event bus). All
   Accepted at their respective phase closes.
4. **Compile-time coercion witnesses** — 3 in `update.rs` + 1 in
   `composites_m4::tests`.

## What M4 deliberately DID NOT import

Walked at P0.5 per the checklist's Q3 discipline + re-confirmed at
every phase close:

- `phi_core::agent_loop::*` — runtime, not governance. M5+ wires this at session launch.
- `phi_core::agents::{Agent, BasicAgent}` — runtime agent trait. baby-phi persists governance metadata in its own `Agent` node; runtime instantiation happens at session-start.
- `phi_core::config::{parser, schema::AgentConfig}` — external YAML blueprint parsing. baby-phi uses direct CRUD via page 09 form.
- `phi_core::context::ContextConfig` — stays org-default per ADR-0023.
- `phi_core::mcp::*` / `phi_core::openapi::*` / `phi_core::tools::*` — runtime tools. Per-agent tool binding deferred to M5 / C-M5-4.
- `phi_core::provider::retry::RetryConfig` — stays org-default per ADR-0023.
- `phi_core::session::{Session, LoopRecord, Turn}` — M5 / C-M5-3.
- `phi_core::types::{AgentEvent, Usage, ToolResult}` — orthogonal surfaces per `phi/CLAUDE.md`.

Each rejection is documented both here AND in the relevant page's
architecture doc (§Q3 rejections).

## Cross-references

- [M4 plan archive §Part 1.5](../../../../plan/build/a634be65-m4-agents-and-projects.md) — phi-core reuse map up-front.
- [phi-core leverage checklist](../../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
- [M3 reuse map](../../m3/architecture/phi-core-reuse-map.md) — cumulative platform + M3 table.
- [ADR-0027](../decisions/0027-per-agent-execution-limits-override.md) — per-agent override.
- [ADR-0028](../decisions/0028-domain-event-bus.md) — domain event bus.
