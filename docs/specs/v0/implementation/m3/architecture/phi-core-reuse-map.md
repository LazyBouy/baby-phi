<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — phi-core reuse map (M3)

**Status: [EXISTS]** — durable publication of plan §1.5.

Legend: ✅ direct reuse • 🔌 wrap (phi field holds phi-core type) • 🚫 no phi-core counterpart.

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
| `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `vision`, `mission`, `system_agents`, `default_model_provider` | (none) | Pure phi governance | 🚫 |
| Inbox / Outbox for CEO | (none) | `Composite::InboxObject` / `OutboxObject` — M1 domain | 🚫 |
| Authority Template A/B/C/D constructors | (none) | `domain::templates::{a,b,c,d}` — phi's governance plane | 🚫 |
| Token budget pool | (none) | `TokenBudgetPool` composite in `composites_m3.rs` — phi's economic-resource model | 🚫 |
| `HasLead` edge variant | (none) | New in M3/P1; M5 wires Template A's trigger | 🚫 |

## Page 07 — Org Dashboard (M3/P5)

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| Aggregate reads (agents / projects / pending AR count / alerted event count / budget utilisation / recent events) | (none) | Pure repo traversal | 🚫 |
| Dashboard polling cadence | (none) | 30 s `setInterval` via Server Actions (M7b upgrade to WebSocket) | 🚫 |

**P5 positive close-audit record** (checklist §6):

1. `grep -rn 'use phi_core' modules/crates/server/src/platform/orgs/dashboard.rs` returns 0 lines — verified at close.
2. `DashboardSummary` wire JSON contains **no** `defaults_snapshot` / `execution_limits` / `context_config` / `retry_config` / `default_agent_profile` / `blueprint` keys — pinned by:
   - Unit test `platform::orgs::dashboard::tests::dashboard_summary_wire_shape_excludes_phi_core_fields`.
   - Acceptance test `acceptance_orgs_dashboard::dashboard_wire_json_excludes_phi_core_fields` (walks the real wire JSON recursively).
   - Web test `modules/web/__tests__/orgs-dashboard.test.ts::DashboardSummaryWire contains no phi-core-wrapping keys`.
3. The dashboard handler + orchestrator deliberately **strip** `Organization.defaults_snapshot` when projecting into the dashboard response; full-snapshot drill-down remains available via `GET /orgs/:id` (show.rs). Rationale documented in `org-dashboard.md §phi-core leverage`.
4. New repo methods (`get_token_budget_pool_for_org`, `count_alerted_events_for_org_since`) are pure phi — `TokenBudgetPool` is a phi composite, audit events are orthogonal per `phi/CLAUDE.md`.

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

## Page 06 + 07 compose — M3/P6 cross-page verification

| Surface | phi-core type | M3 use | Mode |
|---|---|---|---|
| `acceptance_m3.rs` — cross-page E2E (wizard → dashboard) | (none) | Drives the compound flow through real HTTP; verifies audit-chain continuity + wire-shape strip invariant | 🚫 (test composition only) |
| `acceptance_metrics.rs` — POST /orgs metric assertion | (none) | Scrapes Prometheus exposition; asserts `/api/v0/orgs` sample exists post-wizard | 🚫 |

**P6 positive close-audit record:**

1. Re-grep on final close: `grep -rEn "^use phi_core::" modules/crates/server/src/platform/orgs/dashboard.rs modules/crates/server/tests/acceptance_m3.rs modules/crates/server/tests/acceptance_orgs_dashboard.rs` returns **0 lines** — every surface added in P5/P6 remains phi-core-import-free.
2. `acceptance_m3::wizard_to_dashboard_phi_core_wire_shape_contracts_stable` — final-tier test asserting page 06's show response DOES carry `defaults_snapshot` AND page 07's dashboard response does NOT (walks both payloads recursively for 6 forbidden keys).
3. Independent 3-agent re-audit (P6.6) confirmed: phi-core leverage correctly enforced across all M3 deliverables; `OrganizationDefaultsSnapshot` wraps 4 phi-core types unchanged; ADR-0023 invariant verified (zero rows on per-agent policy tables after `apply_org_creation`).

## Enforcement

`scripts/check-phi-core-reuse.sh` (hard-gated in CI since M2/P3)
forbids parallel phi redeclarations of phi-core types. Every
M3 phase's close audit re-verifies. Full four-tier enforcement model
documented in [`phi-core-leverage-checklist.md`](phi-core-leverage-checklist.md)
§"Enforcement — four-tier model".
