<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-29 — `allocate` refinement constraints (`no_further_delegation` etc.) not implemented

## Identification
- **ID**: D-new-29
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: LOW
- **Tags**: `action-refinement`, `allocate-umbrella`

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"`allocate` as the Umbrella Action"
- **Concept claim**: `allocate` can be refined by constraints (e.g., `allocate: no_further_delegation`) to restrict sub-capabilities (delegate / approve / escalate / revoke).
- **Contradiction**: Grant.`constraints` is `Vec<String>`; no typed refinement for `allocate` sub-capabilities.
- **Classification**: `silent-in-code`

## Remediation
- **Approach**: Depends on D-new-09 (typed actions). Add `AllocateRefinement { no_further_delegation, max_depth, etc. }` structured constraint. ~1 day.
- **Impl chunk**: CH-08
- **Risk**: LOW (feature-gap, not security-breach).

## Lifecycle
- 2026-04-24 — `discovered`
