<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-26 — Task node fully scaffolded (missing name/description/token_budget/status/deadline/estimation/created_by + 7-state status enum)

## Identification
- **ID**: D-new-26
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: C — convention/pattern decision (scope deferral)
- **Severity**: LOW
- **Tags**: `ontology-gap`, `deferred-scope`

## Concept alignment
- **Concept doc(s)**: [`concepts/project.md`](../../../concepts/project.md) §"Task (Node Type)"
- **Concept claim**: Task carries name, description, token_budget, tokens_spent, status (7 states), deadline, estimation, created_by.
- **Contradiction**: `Task` id-only scaffold; no TaskStatus enum.
- **Classification**: `concept-aspirational` (Task out-of-scope for M4/M5)

## Remediation
- **Approach**: When task/bidding flow enters scope, materialize per concept. ~2 days.
- **Impl chunk**: M7-DEFERRED-02
- **Risk**: LOW at M5.

## Lifecycle
- 2026-04-24 — `discovered`
