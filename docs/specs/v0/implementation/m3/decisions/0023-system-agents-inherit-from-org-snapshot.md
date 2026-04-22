<!-- Last verified: 2026-04-22 by Claude Code -->

# ADR-0023 — System agents inherit execution context from the org defaults snapshot

**Status: [ACCEPTED M3/P3]** — P3 shipped; invariant verified by
[`modules/crates/store/tests/apply_org_creation_tx_test.rs::adr_0023_invariant_no_per_agent_policy_nodes_materialised`](../../../../../../modules/crates/store/tests/apply_org_creation_tx_test.rs)
(asserts 0 rows on `execution_limits` / `retry_policy` /
`cache_policy` / `compaction_policy` tables after
`apply_org_creation` runs). P4's `acceptance_orgs_create.rs` will
re-run the invariant on the HTTP path when that phase ships.

## Context

M3/P3 materialises two system agents per organisation at creation time
(`apply_org_creation` compound SurrealQL tx). Each system agent needs an
execution budget (`phi_core::context::execution::ExecutionLimits`), a
context policy (`phi_core::context::config::ContextConfig`), and a retry
policy (`phi_core::provider::retry::RetryConfig`) to be invokable.

The `NodeKind` enum already reserves variants for per-agent
`ExecutionLimits`, `RetryPolicy`, `CachePolicy`, and `CompactionPolicy`
nodes (see [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs)
`NodeKind`). M3 could materialise those nodes per system agent — but
[`Organization.defaults_snapshot`](../../../../../../modules/crates/domain/src/model/composites_m3.rs)
already **freezes** every one of those phi-core types at org-creation
time per ADR-0019's non-retroactive invariant.

**The question is**: for M3's 2 system agents per org, do we:

1. **Materialise per-agent `ExecutionLimits` / `ContextConfig` /
   `RetryConfig` nodes** (copy the snapshot at creation into 2 more
   phi-core-wrap instances per org), OR
2. **Inherit at invocation time** — system agents read their execution
   context from the owning_org's snapshot when they're invoked; no
   per-agent node materialisation.

## Decision

**Inherit from snapshot.** M3 does **not** create per-system-agent
`ExecutionLimits` / `ContextConfig` / `RetryConfig` / `CompactionPolicy`
nodes. System agents read from `Organization.defaults_snapshot` at
invoke time.

Per-agent node materialisation is **deferred to M5 session launch**, and
only when a specific per-agent override is required. Until then, the
invocation path walks the `MemberOf` edge from agent to org, reads the
snapshot, and uses those values.

## Consequences

**Positive:**
- Each phi-core type lives in **one place per org** — the phi/CLAUDE.md §phi-core Leverage mandate satisfied
  in the strongest form. Materialising per-agent copies would put
  `phi_core::ExecutionLimits` in 1 org + 2 system agents = 3 copies
  per org, amplifying every subsequent phi-core schema change 3×.
- Non-retroactive semantics stay coherent: the snapshot is the one
  source of truth; no "what happens to the per-agent copy when the org
  snapshot is re-frozen?" ambiguity.
- M3/P3 compound tx is smaller — fewer INSERTs, fewer edges — which
  tightens the rollback correctness proof.
- `AgentProfile.blueprint` (which wraps `phi_core::AgentProfile`) stays
  the only phi-core-wrapping node per system agent. Clear mental model:
  "agent → profile (blueprint) → org (execution/context/retry)".

**Negative:**
- Invocation path reads two nodes (agent + org) instead of one (agent).
  Mitigated: `Organization` is already a hot lookup on every permission
  check (consent_policy, audit_class_default); the snapshot fields
  ride in the same row.
- When M5 needs a per-agent override, it must introduce the node **and**
  the "precedence = per-agent overrides snapshot" resolution rule.
  Documented here so M5 planning doesn't rediscover it.

**Neutral:**
- Human agents (CEO) never had `ExecutionLimits` per
  [`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) line 183's
  invariant; this decision doesn't change that.

## phi-core leverage

This ADR is, at its core, a decision about **where** phi-core types
live to avoid duplication. After M3/P3 ships, each org holds:

- **1** `phi_core::ExecutionLimits` (inside `Organization.defaults_snapshot`)
- **1** `phi_core::ContextConfig` (inside `Organization.defaults_snapshot`)
- **1** `phi_core::RetryConfig` (inside `Organization.defaults_snapshot`)
- **1** `phi_core::AgentProfile` (default, inside `Organization.defaults_snapshot`)
- **2** `phi_core::AgentProfile` (one per system agent, inside `AgentProfile.blueprint` — reasonable because each system agent genuinely differs in `name` / `system_prompt`)
- **1 or more** `phi_core::ModelConfig` (inside `ModelRuntime.config`, shared platform-wide; orgs reference via `default_model_provider`)

That is the minimum-replication count compatible with per-system-agent
identity (name/prompt) while honouring one-source-of-truth for
execution policy.

## Enforcement

P3's `acceptance_orgs_create.rs` (P4) adds a post-creation assertion
querying the SurrealDB schema for node counts per org — must show **0**
rows on `execution_limits` / `retry_policy` / `cache_policy` /
`compaction_policy` tables when only M3-flow writes have run. An
additional row signals a plan regression.

`scripts/check-phi-core-reuse.sh` continues to hard-gate duplicated
phi-core type definitions in CI.

## References

- [M3 plan §D12](../../../../plan/build/563945fe-m3-organization-creation.md)
- [ADR-0019 — Platform defaults are non-retroactive](../../m2/decisions/0019-platform-defaults-non-retroactive.md)
- [phi/CLAUDE.md §phi-core Leverage](../../../../../../CLAUDE.md)
- [`modules/crates/domain/src/model/composites_m3.rs`](../../../../../../modules/crates/domain/src/model/composites_m3.rs) — `OrganizationDefaultsSnapshot`
