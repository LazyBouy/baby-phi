<!-- Last verified: 2026-04-27 by Claude Code -->

# D-new-23 — Human Agents have no guard preventing Identity-node assignment (concept mandates "Human Agents have no system-computed Identity")

## Identification
- **ID**: D-new-23
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `scoped`
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
- 2026-04-27 — `review at CH-01 plan-open` — CH-01 acknowledges scope; no code action this chunk because Identity has no writers at M5 (D-new-01 deferred to CH-16); full closure ownership stays with CH-16. Status held at `scoped`. Plan: [`build/2aa37c80-ch-01-agent-durable-lifecycle.md`](../../../../plan/build/2aa37c80-ch-01-agent-durable-lifecycle.md) §4.
