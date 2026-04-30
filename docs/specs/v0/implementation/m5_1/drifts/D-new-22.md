<!-- Last verified: 2026-04-27 by Claude Code -->

# D-new-22 — Agent role immutability post-creation not enforced at handler layer

## Identification
- **ID**: D-new-22
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `agent-lifecycle`, `immutability-enforcement`

## Concept alignment
- **Concept doc(s)**: [`concepts/agent.md`](../../../concepts/agent.md) §"Agent Roles" — *"Role is immutable post-creation; role transitions go through separate flows."*
- **Contradiction**: `Agent.role: Option<AgentRole>` at [`nodes.rs:199`](../../../../../../modules/crates/domain/src/model/nodes.rs#L199). No explicit guard in `agents/update.rs` rejecting role patch on existing agents. Need to verify handler-layer enforcement.
- **Classification**: `partially-honored`

## Plan vs. reality
- **Reality**: Needs audit of `agents/update.rs` UpdateAgentProfileBody — does it accept `role`? If yes, does it reject? If neither, this drift is real.
- **Root cause**: `concept-doc-not-consulted` at M4 agent-editor handler design.

## Where visible in code
- **File(s)**: [`modules/crates/server/src/platform/agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs)

## Remediation
- **Approach**: Audit update handler; add explicit rejection for `role` field in PATCH body. ~0.5 day.
- **Impl chunk**: CH-01
- **Risk**: LOW (exploitable only via direct API call with role field).

## Lifecycle
- 2026-04-24 — `discovered` — surfaced during concept-vs-code audit at M5.1/P2
- 2026-04-24 — `classified` — Bucket B MEDIUM; partially-honored classification; needs handler audit (backfill)
- 2026-04-24 — `scoped` — assigned to CH-01 in forward-scope inventory §1 at M5.1/P3 close (backfill)
- 2026-04-27 — `in-chunk-plan` — CH-01 plan approved ([`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](../../../../plan/build/ch-01-agent-durable-lifecycle-2aa37c80.md)); plan-time investigation confirmed existing handler-layer enforcement at [`update.rs:133-141`](../../../../../../modules/crates/server/src/platform/agents/update.rs#L133); ratification path: P4 adds explicit acceptance test pinning the rule
- 2026-04-27 — `remediated` — via CH-01 chunk-seal; existing rust-level enforcement at [`update.rs:133-141`](../../../../../../modules/crates/server/src/platform/agents/update.rs#L133) ratified by new acceptance test [`update_rejects_role_change_with_immutable_field_changed`](../../../../../../modules/crates/server/tests/acceptance_agents_profile.rs) which calls `update_agent_profile` directly with `new_role: Some(AgentRole::Admin)` and asserts `Err(AgentError::ImmutableFieldChanged("role"))`; pinned for future regression prevention. Note: HTTP wire format `UpdateAgentProfileRequest` does not include a `role` field, so the rule is also enforced at the wire layer by silent-drop. Ratification documented in [ADR-0034](../../m5_2/decisions/0034-agent-durable-lifecycle.md) §Context
