<!-- Last verified: 2026-04-27 by Claude Code -->

# ADR-0034 — Agent durable lifecycle state + governance-vs-runtime boundary

**Status: Accepted**
**Decided at:** CH-01 chunk-seal (P5), 2026-04-27
**Chunk plan:** [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](../../../../plan/build/ch-01-agent-durable-lifecycle-2aa37c80.md)
**Closes drifts:** [D6.5](../../m5_1/drifts/D6.5.md) (HIGH) + [D-new-22](../../m5_1/drifts/D-new-22.md) (MEDIUM); reviews [D-new-23](../../m5_1/drifts/D-new-23.md) (LOW, stays `scoped` to CH-16)
**Concepts touched:** [`agent.md`](../../../concepts/agent.md) §Roles + §Lifecycle, [`system-agents.md`](../../../concepts/system-agents.md) §Operator-can-disable + §Archive-flow, [`human-agent.md`](../../../concepts/human-agent.md) §No-Identity, [`phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §agents/

## Context

M4/P0 added the `agent.role` column (migration 0004) but stopped short of the durable lifecycle fields the concept docs mandate: `active: bool` and `archived_at: Option<DateTime>`. The existing system-agent disable / archive handlers ([`disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs), [`archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs)) emitted audit events but did not mutate durable state — there was no column to mutate. AgentCatalogListener (M5/P3 stub at [`listeners.rs:497`](../../../../../../modules/crates/domain/src/events/listeners.rs#L497)) cannot light up the M5.2/P8 body that drives `SystemAgentRuntimeStatus.is_paused` because the durable disable flag doesn't exist. This is drift D6.5 (HIGH).

The user's plan-time review additionally surfaced two architectural questions:
- **Why is `domain::Agent` orthogonal to phi-core's `Agent` trait?** — answered in D34.6 below.
- **Why is `domain::Agent` ↔ `domain::AgentProfile` 1:1 instead of N:1 template-sharing?** — answered as a forward-scope `M6+-OPEN-01` open question (not in CH-01's scope; recoverable at M6 plan-open).

D-new-22 (role immutability not pinned by an acceptance test) was opportunistically closed alongside since the rust-level enforcement at [`update.rs:133-141`](../../../../../../modules/crates/server/src/platform/agents/update.rs#L133) was already shipping. D-new-23 (Human Agent Identity guard) cannot land at M5 because Identity has no writers; it stays `scoped` to CH-16.

## Decision

### D34.1 — `agent.active: bool DEFAULT true`

Migration [`0007_agent_active_archived.surql`](../../../../../../modules/crates/store/migrations/0007_agent_active_archived.surql) adds the column with `DEFAULT true`. Rationale for `true` default: pre-CH-01 rows materialise as active (existing fleet keeps running unchanged); the only way to land at `false` is via the disable handler. Schema choice (`bool`, not `option<bool>`) matches the concept claim — every agent has a defined active state, never "unknown".

The domain struct [`Agent`](../../../../../../modules/crates/domain/src/model/nodes.rs) carries `active: bool` with `#[serde(default = "default_agent_active")]` (the helper returns `true`) so deserialised rows that lack the column round-trip cleanly.

### D34.2 — `agent.archived_at: option<string>`

Same migration adds the column as `option<string>` storing RFC3339 strings, matching the project convention used by `grant.revoked_at` and `consent.revoked_at` (migration 0001). Rust-side: `Option<DateTime<Utc>>` with `#[serde(default)]`. chrono's serde feature handles RFC3339 round-trip automatically.

`None` = not archived; `Some(t)` = archived at time `t`. The archive handler writes `Utc::now()` at the moment of archive.

### D34.3 — Repository methods `set_agent_active` + `set_agent_archived_at`

Two new methods on the `Repository` trait at [`repository.rs:357-394`](../../../../../../modules/crates/domain/src/repository.rs):

```rust
async fn set_agent_active(&self, agent_id: AgentId, active: bool) -> RepositoryResult<()>;
async fn set_agent_archived_at(&self, agent_id: AgentId, archived_at: Option<DateTime<Utc>>) -> RepositoryResult<()>;
```

Both return `RepositoryError::NotFound` if the agent row is missing (existence check is explicit because SurrealDB's `UPDATE` on a missing record silently returns empty). Both backends ([`store::SurrealStore`](../../../../../../modules/crates/store/src/repo_impl.rs), [`domain::InMemoryRepository`](../../../../../../modules/crates/domain/src/in_memory.rs)) implement them; tests at [`store/tests/repo_agent_lifecycle_test.rs`](../../../../../../modules/crates/store/tests/repo_agent_lifecycle_test.rs) cover both backends' contract.

### D34.4 — System-agent disable + archive handlers flip durable state BEFORE audit emit

[`disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs) calls `repo.set_agent_active(agent_id, false).await?` after validation passes and before the audit event is emitted. [`archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs) calls `repo.set_agent_archived_at(agent_id, Some(now)).await?` similarly.

**Ordering rationale.** Durable state is authoritative; audit is replayable. If audit emit fails after the durable flip, the persisted state is still correct (the next audit query can re-derive the event). If we flipped order — audit first, then durable state — a failed durable write after a successful audit would leave the audit log "ahead" of reality, requiring a complicated reconciliation step. Writing durable state first eliminates that class of bug.

### D34.5 — Conforming criteria for CH-22 (AgentCatalogListener body)

When CH-22 ships the listener body that materialises `SystemAgentRuntimeStatus.is_paused`:

1. The listener MUST consult `Agent.active` via `repo.get_agent(agent_id)` when computing `is_paused`. The audit log is NOT the source of truth for current state.
2. The listener MUST treat any agent with `Agent.archived_at = Some(_)` as terminally paused (`is_paused = true`) regardless of `active`. Archive is a stronger statement than disable.
3. If both fields disagree (e.g., `active = true` but `archived_at = Some(_)` from an earlier archive call), the archive wins — `is_paused = true`. Concept docs treat archive as terminal soft-deletion.
4. The listener body MUST NOT write to `agent.active` / `agent.archived_at` itself; only the disable / archive handlers own those columns. The listener is read-only with respect to lifecycle state.

### D34.6 — Governance-vs-runtime boundary: `domain::Agent` is orthogonal to `phi_core::Agent`

baby-phi's [`domain::Agent`](../../../../../../modules/crates/domain/src/model/nodes.rs) is a **governance node** (identity, kind, role, owning_org, lifecycle: `active`, `archived_at`, `created_at`). It does NOT wrap, inherit from, or implement phi-core's `Agent` trait or `BasicAgent` struct. The decision: keep them in different layers because they serve different purposes.

**Three different `Agent` types, three different jobs:**

| Type | Layer | Job | Persisted? | Used by baby-phi? |
|---|---|---|---|---|
| `domain::Agent` (struct) | baby-phi governance | Identity / role / org membership / lifecycle | Yes (DB row) | Yes |
| [`phi_core::Agent`](../../../../../../../phi-core/src/agents/agent.rs) (trait) | phi-core runtime interface | Anything that can be prompted + continued | No (trait, no state) | **No** (baby-phi calls `phi_core::agent_loop()` directly — a free function — bypassing the trait) |
| `phi_core::BasicAgent` (struct) | phi-core runtime impl | Stateful long-lived in-memory wrapper for callers who want a chat-style agent | No (in-memory only) | **No** (baby-phi is per-request stateless) |
| [`phi_core::agents::profile::AgentProfile`](../../../../../../../phi-core/src/agents/profile.rs) | phi-core blueprint | Execution recipe (system_prompt, tools, model) | Caller persists | Yes (wrapped at `domain::AgentProfile.blueprint`) |

**Why baby-phi doesn't use the `Agent` trait.** phi-core's `agent_loop()` is a free function that doesn't require the caller to instantiate `Agent` / `BasicAgent`. baby-phi uses **per-request statelessness**: each session is a fresh `agent_loop()` invocation; state persists to SurrealDB via `BabyPhiSessionRecorder`; nothing lives in process memory between requests. The trait + `BasicAgent` exist for callers who want a stateful long-lived in-memory wrapper (e.g., a CLI chat REPL) — baby-phi has no such caller today.

**The connection point.** [`sessions/provider.rs::build_agent_context`](../../../../../../modules/crates/server/src/platform/sessions/provider.rs) at session-launch time wires `domain::AgentId.to_string()` into `phi_core::types::context::AgentContext.agent_id` — **ID-only delegation**, no struct-level reuse. phi-core never sees `domain::Agent`.

**Review trigger.** If a future milestone introduces *long-lived in-memory chat agents* (state persists in process memory across HTTP requests instead of round-tripping through SurrealDB on every turn), the `phi_core::Agent` trait becomes a leverage candidate. At that point this ADR's D34.6 decision is re-opened. **No such feature is planned through M7b.**

## Conforming-criteria for CH-22 (binding contract for the listener body)

CH-22's plan §5 ADR section MUST cite this ADR. CH-22's plan §7 phase deliverables MUST reflect §D34.5 above (consult `Agent.active`/`archived_at` via `repo.get_agent`; archive wins ties; listener is read-only on lifecycle).

## Ratification evidence

| Sub-decision | Evidence |
|---|---|
| D34.1 (`active` column) | [`migrations/0007_agent_active_archived.surql`](../../../../../../modules/crates/store/migrations/0007_agent_active_archived.surql); `domain::Agent.active` at [`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) |
| D34.2 (`archived_at` column) | Same migration; `domain::Agent.archived_at` |
| D34.3 (repo methods) | [`repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) trait additions; [`repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) Surreal impl; [`in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs) impl; [`migrations_0007_test.rs`](../../../../../../modules/crates/store/tests/migrations_0007_test.rs) (5 tests) + [`repo_agent_lifecycle_test.rs`](../../../../../../modules/crates/store/tests/repo_agent_lifecycle_test.rs) (5 tests) |
| D34.4 (handler wiring) | [`disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs) + [`archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs); acceptance tests `disable_with_confirm_succeeds_and_surfaces_was_standard_flag` + `archive_with_confirm_succeeds_and_flips_durable_archived_at` at [`acceptance_system_agents.rs`](../../../../../../modules/crates/server/tests/acceptance_system_agents.rs) |
| D34.5 (CH-22 conforming criteria) | This ADR §D34.5; CH-22's plan-time §5 must cite this ADR |
| D34.6 (governance/runtime boundary) | This ADR §D34.6; structural docs at [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) §"Orthogonal surfaces" + [`phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §"Connection point" |

## Consequences

**Positive:**
- Concept-doc claim about agent lifecycle (active vs disabled vs archived) is honored at runtime, not just on paper.
- AgentCatalogListener (CH-22) gets a deterministic source of truth for tile state.
- Audit log + durable state cannot diverge (durable write precedes audit emit per D34.4).
- Governance/runtime separation (D34.6) is explicit + reviewable; future milestones can re-open the boundary if requirements change.

**Negative:**
- Pre-CH-01 rows need first-read deserialisation through serde defaults; the migration's `DEFAULT true` covers this on the schema side too.
- Schema grows by 2 columns (modest cost).

**Neutral:**
- D-new-23 (Human Identity guard) stays open at `scoped` for CH-16; this is per the upstream cascade (D-new-01 deferred).
- The `M6+-OPEN-01` AgentProfile-cardinality question is captured in forward-scope §3 but is NOT a commitment to redesign — the user decides at M6 plan-open.

## Alternatives considered

**(A) Add `active` to `domain::AgentProfile` instead of `domain::Agent`.** Rejected: profile is the execution blueprint; lifecycle is governance. Conflating them would make AgentCatalogListener (which reads governance state) depend on the profile path.

**(B) Use `option<bool>` for `active` instead of `bool DEFAULT true`.** Rejected: every agent has a defined active state; "unknown" is not a meaningful concept-doc state. `bool DEFAULT true` keeps semantics precise.

**(C) Wrap `phi_core::Agent` trait in `domain::Agent`.** Rejected per D34.6 — the two serve different layers and have no field overlap. Wrapping would force baby-phi to instantiate runtime in-memory agents on every request, contradicting the per-request-stateless architecture.

**(D) Add `disable_at` instead of just flipping `active = false`.** Rejected at this chunk; could revisit if disable-event timestamps become important downstream. The audit log already records the flip event with timestamp, so a separate column is duplicative for now.

## Review trigger

- CH-22 plan-open — verify §D34.5 conforming criteria are operationalised in the listener body.
- Any future milestone introducing long-lived in-memory chat agents — re-evaluate §D34.6 (potential phi-core trait leverage).
- M7b plan-open — `Agent.archived_at` may be relevant for retention-tier policy (cross-ref `M7b-DEFERRED-01` AuthRequest retention).

## References

- Chunk plan: [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](../../../../plan/build/ch-01-agent-durable-lifecycle-2aa37c80.md)
- Drifts closed: [D6.5](../../m5_1/drifts/D6.5.md), [D-new-22](../../m5_1/drifts/D-new-22.md)
- Drift reviewed (stays scoped): [D-new-23](../../m5_1/drifts/D-new-23.md)
- Concept docs touched: [`agent.md`](../../../concepts/agent.md), [`system-agents.md`](../../../concepts/system-agents.md), [`human-agent.md`](../../../concepts/human-agent.md), [`phi-core-mapping.md`](../../../concepts/phi-core-mapping.md)
- Sibling ADR: [ADR-0033](./0033-k8s-prep-refactors.md) (CH-K8S-PREP — D33.1–D33.4 conforming criteria still satisfied; CH-01 is K8s-neutral per its §3.B 7-axis evaluation)
- Forward-scope: [`forward-scope/22035b2a-...md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §3 `M6+-OPEN-01` (AgentProfile cardinality re-evaluation — open question surfaced during CH-01 plan review)
