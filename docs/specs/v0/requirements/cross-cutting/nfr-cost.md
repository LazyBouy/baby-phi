<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# NFR — Cost

> Token-budget enforcement and cost-accounting fidelity. Token costs translate to real money spent with LLM providers; the system MUST enforce configured budgets accurately and produce audit-grade cost attribution.

## Token budget enforcement

- **R-NFR-cost-1 — Per-org budget caps.** Each org's `token-budget-pool` economic_resource SHALL have an `initial_allocation` (set on [admin/06-org-creation-wizard.md](../admin/06-org-creation-wizard.md)) and a running `tokens_spent`. When `tokens_spent >= initial_allocation`, new session launches from this org's agents SHALL be denied with `TOKEN_BUDGET_EXHAUSTED`. See [system/s06-periodic-triggers.md R-SYS-s06-10](../system/s06-periodic-triggers.md).
- **R-NFR-cost-2 — Thresholds emit audit events.** Crossing 50% / 75% / 90% / 100% of budget SHALL emit `TokenBudgetThresholdCrossed` audit events (50/75 logged, 90/100 alerted). Thresholds are checked at snapshot time (hourly) and again on session launch.
- **R-NFR-cost-3 — Per-project caps.** A project MAY declare its own `token_budget`; if so, session launches on that project SHALL respect the project budget AS WELL AS the org budget. The stricter of the two binds.
- **R-NFR-cost-4 — Per-agent ExecutionLimits bound per-session.** An Agent's `ExecutionLimits.max_cost_usd` (phi-core type; see [concepts/phi-core-mapping.md](../../concepts/phi-core-mapping.md)) bounds a single session's cost. A session approaching the per-session cap SHALL be terminated with reason `execution_limit_cost` and a `SessionAbortedExecutionLimit` audit event.

## Cost accounting

- **R-NFR-cost-5 — Per-session cost recorded.** Every session SHALL have a recorded `token_cost` and `usd_cost` on session end. Attribution: costs aggregate up to the owning project's `tokens_spent` and the owning org's `tokens_spent`.
- **R-NFR-cost-6 — For Shape B co-owned projects**, costs SHALL be split per the project's declared `cost_split` field (default 50/50) between the co-owners' org budgets. Session-end accounting records the split.
- **R-NFR-cost-7 — Contract-agent tokens-kept savings.** Per [concepts/agent.md § Contract Agent](../../concepts/agent.md#contract-agent-token-economy-participant), a Contract agent that completes work under their bid budget retains the savings as accrued Worth. The accounting flow SHALL:
  - Record the contracted token amount (from the winning Bid).
  - Record actual consumption.
  - Credit `(contracted - actual)` to the Contract agent's accumulated savings — a field tracked on the Agent's Identity node's `lived` struct.
- **R-NFR-cost-8 — Auditable reconciliation.** At month-end (per [s06 § billing-aggregator-agent](../system/s06-periodic-triggers.md)) an org-level reconciliation SHALL produce `{sum_of_session_costs_this_period} == {org_budget_delta_this_period}` within ±0.01 USD. Failure to reconcile emits alerted `TokenAccountingReconciliationFailed` event.

## Cost attribution

- **R-NFR-cost-9 — Cost events structurally reference the session.** Every cost-related audit event SHALL include `session_id`, `agent_id`, `project_id?`, `org_id`, `tokens`, `usd_cost`. No dangling charges — every token spent resolves to a session.
- **R-NFR-cost-10 — Model provider pricing is configurable per `model_runtime_object`.** The `model/runtime_object` entry ([admin/02-platform-model-providers.md](../admin/02-platform-model-providers.md)) SHALL carry the pricing rule (per-input-token / per-output-token / per-request). The accounting flow applies this rule at session-end to produce `usd_cost`.

## Cost transparency

- **R-NFR-cost-11 — Org dashboard shows current burn.** The [admin/07-organization-dashboard.md](../admin/07-organization-dashboard.md) SHALL display `tokens_spent / initial_allocation` with current rate (per hour/day trailing). Rate exceeds sustainable burn (configurable threshold) → warning badge.
- **R-NFR-cost-12 — Agent self-visibility.** An Agent SHALL be able to see their own session cost history via [a04-my-work.md](../agent-self-service/a04-my-work.md).

## Cross-references

- [concepts/token-economy.md](../../concepts/token-economy.md) — the economic model.
- [concepts/agent.md § Contract Agent](../../concepts/agent.md#contract-agent-token-economy-participant) — the savings-retention rule.
- [system/s06-periodic-triggers.md](../system/s06-periodic-triggers.md) — billing-aggregator + threshold triggers.
