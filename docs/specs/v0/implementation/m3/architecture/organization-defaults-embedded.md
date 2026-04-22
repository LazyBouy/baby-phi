<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — OrganizationDefaults embedded on Organization

**Status: [EXISTS]** — landed in M3/P1 per [ADR-0020](../decisions/0020-organization-defaults-embedded.md).

Each org's governance snapshot (execution limits / agent blueprint /
context config / retry config + baby-phi retention + alert channels)
is a field on the Organization node, NOT a sibling composite. This
page publishes the rationale for grep-ability; the ADR holds the
Accepted decision.

See [`../decisions/0020-organization-defaults-embedded.md`](../decisions/0020-organization-defaults-embedded.md) for the durable ADR. Code
references:

- [`../../../../../../modules/crates/domain/src/model/composites_m3.rs`](../../../../../../modules/crates/domain/src/model/composites_m3.rs) — `OrganizationDefaultsSnapshot` + `ConsentPolicy` + `TokenBudgetPool`.
- [`../../../../../../modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) — `Organization.defaults_snapshot` + sibling governance fields.
- [`../../../../../../modules/crates/store/migrations/0003_org_creation.surql`](../../../../../../modules/crates/store/migrations/0003_org_creation.surql) — `FLEXIBLE TYPE option<object>` column for the embedded snapshot.

## Non-retroactive invariant

Captured at creation time, frozen thereafter. A later
`PlatformDefaults` PUT does **not** mutate any org's snapshot. M2/P7
pinned the invariant with the `platform_defaults_non_retroactive_
props` proptest; M3 re-verifies it by using the same proptest
(unchanged after M3/P1's Organization struct extension).

## phi-core leverage

Four phi-core types wrapped directly (no parallel baby-phi layer);
same pattern as M2/P7's `PlatformDefaults`:

- `phi_core::context::execution::ExecutionLimits`
- `phi_core::agents::profile::AgentProfile`
- `phi_core::context::config::ContextConfig`
- `phi_core::provider::retry::RetryConfig`

Baby-phi adds only `default_retention_days` + `default_alert_channels`
— governance concerns with no phi-core counterpart.
