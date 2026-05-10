<!-- Last verified: 2026-05-10 by Claude Code (CH-19 P2: Status flipped `discovered` → `accepted-as-is`; ratified via CH-19 / ADR-0057 §D57.9; cycle hex `2c520ba7`.) -->
<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-27 — Token-economy fields missing on Agent (rating_window, total_tokens_earned, total_tokens_consumed + Worth formula)

## Identification
- **ID**: D-new-27
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `accepted-as-is`
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
- 2026-05-10 — `accepted-as-is` — ratified via CH-19 (cycle hex `2c520ba7`) / ADR-0057 §D57.9; the `accepted-as-is` is for the **deferral itself**, not the implementation surface; review trigger: M6-or-M7-DEFERRED (token-economy / contracts-bidding milestone); the drift's `Implementation chunk this belongs to: M6-or-M7-DEFERRED` field stays as future-remediation marker (CH-19 ratifies the deferral; M6/M7 deferred-token-economy chunk is when `rating_window`, `total_tokens_earned/consumed`, Worth-formula fields land); concept-doc `token-economy.md` carries 1-line deferred-state footnote at the §"Worth" preamble referencing CH-19 + M6-or-M7-DEFERRED.
