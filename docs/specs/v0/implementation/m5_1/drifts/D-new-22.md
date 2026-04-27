<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-22 — Agent role immutability post-creation not enforced at handler layer

## Identification
- **ID**: D-new-22
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
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
- 2026-04-24 — `discovered`
