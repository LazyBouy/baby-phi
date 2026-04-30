<!-- Last verified: 2026-04-28 by Claude Code -->

# D-new-23 — Human Agents have no guard preventing Identity-node assignment (concept mandates "Human Agents have no system-computed Identity")

## Identification
- **ID**: D-new-23
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: B — underspecified shape choice
- **Severity**: LOW
- **Tags**: `human-agent-model`, `invariant-enforcement`

## Concept alignment
- **Concept doc(s)**: [`concepts/human-agent.md`](../../../concepts/human-agent.md) §"No Identity"
- **Contradiction**: No runtime guard preventing `Agent(kind=Human) → HAS_IDENTITY → Identity`. Not exploitable at M5 since D-new-01 puts Identity in scaffold-only state (no writers yet), but when D-new-01 closes the guard must exist.
- **Classification**: `silent-in-code`

## Plan vs. reality
- **Reality**: No guard.
- **Root cause**: `cascading-upstream-deferral` (Identity itself deferred per D-new-01)

## Where visible in code
- **File(s)**: No guard code; would live at Identity-creation handler.

## Remediation
- **Approach**: When D-new-01 lands, add guard in identity-creation/update path rejecting Human agent_id. ~0.5 day.
- **Impl chunk**: CH-01 + CH-16
- **Dependencies**: D-new-01

## Lifecycle
- 2026-04-24 — `discovered` — surfaced during concept-vs-code audit at M5.1/P2
- 2026-04-24 — `classified` — Bucket B LOW silent-in-code; cascading from D-new-01 (Identity has no writers at M5) (backfill)
- 2026-04-24 — `scoped` — partially assigned to CH-01 (review only) + finalisation by CH-16 (backfill)
- 2026-04-27 — `review at CH-01 plan-open` — CH-01 acknowledges scope; no code action this chunk because Identity has no writers at M5 (D-new-01 deferred to CH-16); full closure ownership stays with CH-16. Status held at `scoped`. Plan: [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](../../../../plan/build/ch-01-agent-durable-lifecycle-2aa37c80.md) §4.
- 2026-04-28 — `in-chunk-plan` — CH-16 plan approved ([`build/ch-16-identity-node-materialization-2ae4fabe.md`](../../../../plan/build/ch-16-identity-node-materialization-2ae4fabe.md)); BOTH-guards approach: defensive at `Repository::upsert_identity` (typed `RepositoryError::HumanAgentHasNoIdentity { agent_id }` rejects every caller) + preventive at `apply_agent_creation` (skips Identity insertion entirely for `AgentKind::Human`); 3 unit tests pin (defensive rejection, preventive skip, list excludes Human kind); ADR-0039 records the design decisions.
- 2026-04-28 — `remediated` — CH-16 chunk-seal; BOTH guards live: defensive at `Repository::upsert_identity` (in-memory + SurrealDB layers both check `Agent.kind` and return typed `RepositoryError::HumanAgentHasNoIdentity`); preventive at `apply_agent_creation` (rejects Human-with-Some + Llm-with-None mismatches; commits with `identity: None` for Human kind). Tests: 2 in-memory defensive + 1 SurrealDB defensive + 1 list-excludes-Human invariant + 2 archive-orphan regression = 6 guard-related tests pin the closure. ADR-0039 §D39.1–D39.5 documents the design.
