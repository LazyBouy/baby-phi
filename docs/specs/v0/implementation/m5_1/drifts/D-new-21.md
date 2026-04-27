<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-21 — Edge-count documentation mismatch (docstring claims 69, actual count ambiguous)

## Identification
- **ID**: D-new-21
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice (doc/code count drift)
- **Severity**: LOW
- **Tags**: `documentation-counting`, `ontology-headcount`

## Concept alignment
- **Concept doc(s)**: [`concepts/ontology.md`](../../../concepts/ontology.md) §"Edge Types" — claims 69 (67 at M3 + 2 M4/P1).
- **Contradiction**: Agent 1's inspection counted 67 enum variants; my `grep -cE` heuristic returned 71. The actual canonical count needs an authoritative manual audit. Docstring and concept don't match code exactly.
- **Classification**: `partially-honored` (count off by some delta)

## Plan vs. reality
- **Reality**: `edges.rs:25` header says "69"; enum variant count uncertain without line-by-line count.
- **Root cause**: `reviewer-gap-at-phase-close` — nobody has manually recounted since M4/P1.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) line 25 header

## Remediation
- **Approach**: Manual line-by-line recount of Edge enum variants. Fix either the docstring or the enum. ~0.25 day.
- **Impl chunk**: CH-19
- **Risk**: low.

## Lifecycle
- 2026-04-24 — `discovered`
