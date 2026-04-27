<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-27 — Token-economy fields missing on Agent (rating_window, total_tokens_earned, total_tokens_consumed + Worth formula)

## Identification
- **ID**: D-new-27
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: C — convention/pattern decision (scope deferral)
- **Severity**: MEDIUM
- **Tags**: `token-economy`, `deferred-scope`, `rolling-window`

## Concept alignment
- **Concept doc(s)**: [`concepts/token-economy.md`](../../../concepts/token-economy.md) §"Worth Formula", §"Rating Window", §"Intern→Contract Carry-Forward"
- **Concept claim**: Agent carries rating_window (Vec<f32>, size N=20 default), rating_history_avg, rating_history_count, total_tokens_earned, total_tokens_consumed. Worth = avg_rating × (earned − consumed) / consumed.
- **Contradiction**: Agent struct has none of these fields.
- **Classification**: `concept-aspirational` (token-economy deferred until contracts/bidding milestone)

## Remediation
- **Approach**: When contracts/bidding milestone opens, add fields. Migration adds columns. Worth computed from them. ~3 days.
- **Impl chunk**: M6-or-M7-DEFERRED
- **Risk**: MEDIUM — rolling-window calculations important for Worth; blocks Intern→Contract promotion logic.

## Lifecycle
- 2026-04-24 — `discovered`
