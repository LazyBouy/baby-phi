<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-10 — Action × Fundamental applicability matrix not enforced (grants can pair any action with any fundamental)

## Identification
- **ID**: D-new-10
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `permission-engine`, `applicability-matrix`
- **Blocks**: D-new-07 (validator uses the matrix)
- **Blocked-by**: D-new-09 (typed actions needed)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"Action × Fundamental Applicability Matrix"
- **Concept claim**: Each fundamental has a defined set of applicable actions; actions outside the set are invalid. E.g., `send` applies to `message` fundamental; `recall` applies to `tag + data_object` composite.
- **Contradiction**: No matrix defined in code. Grant `(action, fundamental)` pairs are not validated for compatibility at publish or grant-mint time.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Matrix enforced at publish + grant-mint time.
- **Reality (shipped state at current HEAD)**: No enforcement.
- **Root cause**: `concept-doc-not-consulted`.

## Where visible in code
- **File(s)**: No matrix file; no validator.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "ACTION_FUNDAMENTAL_MATRIX\|is_action_applicable" modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Define `permissions::action::APPLICABILITY_MATRIX: &[(Action, &[Fundamental])]` or equivalent. Add `is_action_applicable(action, fundamental) -> bool`. Wire into D-new-07 validator + grant-mint paths.
- **Implementation chunk this belongs to**: CH-04
- **Dependencies on other drifts**: D-new-09 (actions typed first)
- **Estimated effort**: 1 engineer-day.
- **Risk to concept alignment if deferred further**: MEDIUM — grants can declare nonsense (e.g., `send` on `tool_object`); permission checks silently fail or succeed unexpectedly.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
