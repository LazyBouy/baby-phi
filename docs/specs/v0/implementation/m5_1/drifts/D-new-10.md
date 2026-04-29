<!-- Last verified: 2026-04-29 by Claude Code (CH-04: terminally remediated. The 9×10 applicability matrix from `concepts/permissions/03-action-vocabulary.md` lines 27–37 is encoded as `Action::applies_to(Fundamental) -> bool` + `Action::applies_to_composite(Composite) -> bool` (composite verdict derives via constituents() per concept-doc §"Composite inheritance" line 39). Exhaustive 306-cell test transcribes the matrix verbatim from the concept doc. CH-04 ships the matrix as a queryable function; CH-05 wires it into the publish-time validator (deferred per ADR-0043 §D43.8). ADR-0043 ratifies. Status flipped to `remediated`.) -->

# D-new-10 — Action × Fundamental applicability matrix not enforced (grants can pair any action with any fundamental)

## Identification
- **ID**: D-new-10
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
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
- 2026-04-28 — `in-chunk-plan` — assigned to CH-04 (paired with D-new-09 — neither closes without the other).
- 2026-04-29 — `remediated` — CH-04 chunk-seal — `Action::applies_to(Fundamental)` encodes the 9×10 matrix verbatim; `Action::applies_to_composite(Composite)` derives via constituents(). Exhaustive 306-cell test transcribes the matrix from concept-doc lines 27–37 — a divergence between the concept doc and the code surfaces as a test failure. CH-04 ships the matrix as a queryable function; per ADR-0043 §D43.8, CH-05's publish-time validator wires it in as a real rejection rule. [ADR-0043](../../m5_2/decisions/0043-typed-action-vocabulary.md) Accepted ratifies.
