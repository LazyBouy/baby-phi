<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — phi-core reuse map (M3)

**Status: [EXISTS]** — durable publication of plan §1.5.

Legend: ✅ direct reuse • 🔌 wrap (baby-phi field holds phi-core type) • 🚫 no phi-core counterpart.

## Page 06 — Org Creation Wizard (M3/P4)

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| System agent blueprint | `phi_core::agents::profile::AgentProfile` | Both system agents' `blueprint` field (M2/P0 already wraps) | ✅ (inherited) |
| System agent execution budget | `phi_core::context::execution::ExecutionLimits` | Defaults to org's `defaults_snapshot.execution_limits` | ✅ |
| System agent model binding | `phi_core::provider::model::ModelConfig` via M2's `ModelRuntime` | Bound via `secret_ref` + registered provider | ✅ |
| Org defaults snapshot — execution | `phi_core::context::execution::ExecutionLimits` | `OrganizationDefaultsSnapshot.execution_limits` field | 🔌 |
| Org defaults snapshot — context | `phi_core::context::config::ContextConfig` | `OrganizationDefaultsSnapshot.context_config` field | 🔌 |
| Org defaults snapshot — retry | `phi_core::provider::retry::RetryConfig` | `OrganizationDefaultsSnapshot.retry_config` field | 🔌 |
| Org defaults snapshot — agent blueprint | `phi_core::agents::profile::AgentProfile` | `OrganizationDefaultsSnapshot.default_agent_profile` field | 🔌 |

## Organization node — governance fields

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `vision`, `mission`, `system_agents`, `default_model_provider` | (none) | Pure baby-phi governance | 🚫 |
| Inbox / Outbox for CEO | (none) | `Composite::InboxObject` / `OutboxObject` — M1 domain | 🚫 |
| Authority Template A/B/C/D constructors | (none) | `domain::templates::{a,b,c,d}` — baby-phi's governance plane | 🚫 |
| Token budget pool | (none) | `TokenBudgetPool` composite in `composites_m3.rs` — baby-phi's economic-resource model | 🚫 |
| `HasLead` edge variant | (none) | New in M3/P1; M5 wires Template A's trigger | 🚫 |

## Page 07 — Org Dashboard (M3/P5)

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| Aggregate reads (agents / projects / pending AR count / alerted event count / budget utilisation / recent events) | (none) | Pure repo traversal | 🚫 |
| Dashboard polling cadence | (none) | 30 s `setInterval` via Server Actions (M7b upgrade to WebSocket) | 🚫 |

## CLI / Seal (M3/P4–P6)

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| Shell completion generator | `clap_complete` (non-phi-core; M2/P8 wired) | New `org` subcommand auto-surfaces | ✅ (zero net work) |

## Why `Organization` is NOT a wrap of `phi_core::session::model::Session`

Summarised here; full 4-reason argument lives in
[`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md)
§1.5 and is pinned as D11 in Part 3:

1. Different lifetimes (years vs hours).
2. Different level in hierarchy (Org → Agent → Session is three levels; a wrap collapses two).
3. Zero field overlap — `turns` / `loop_records` / `status` / `ended_at` don't make sense on an Organization.
4. `phi_core::Session` becomes load-bearing at M5 via a **containment** FK relationship, not a wrap. Org-level drill-down into individual sessions is a navigation pattern across the FK graph at M5+; M3's dashboard link targets upgrade from audit-log pages to session-trace URLs when events carry session provenance. No wrap needed.

## Enforcement

`scripts/check-phi-core-reuse.sh` (hard-gated in CI since M2/P3)
forbids parallel baby-phi redeclarations of phi-core types. Every
M3 phase's close audit re-verifies.
