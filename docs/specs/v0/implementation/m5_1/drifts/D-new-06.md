<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-06 — Multi-scope cascade resolution (Project → Org → base_project → base_org → intersection fallback) not fully implemented

## Identification
- **ID**: D-new-06
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `permission-engine`, `scope-resolution`, `multi-scope`
- **Blocks**: Worked-example scenarios 4–6 (cross-project, cross-org sessions); contractor-model logic (D-new-20)
- **Blocked-by**: D-new-03 (full cascade uses tag-predicate selectors); D4.1 advisory-only currently masks the issue

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Step 5 — Scope Resolution"; [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Unified Resolution Rule"; [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) scenarios 4–6
- **Concept claim (paraphrase)**: Reader's effective scope resolves by cascading project → org (most specific wins); ties break on reader's base_project/base_org; outsider rule applies intersection of scopes.
- **Contradiction**: [`permissions/engine.rs:311+`](../../../../../../modules/crates/domain/src/permissions/engine.rs#L311) `step_5_scope_resolution` exists but implements simpler logic per the Agent 3 audit — no 5-tier cascade, no tie-breaker using base_project/base_org, no intersection fallback for outsider scenarios.
- **Classification**: `partially-honored` → drift since step exists but doesn't honor full concept algorithm
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: 5-tier cascade per Figure 4 in concept 04 + worked examples in concept 08.
- **Reality (shipped state at current HEAD)**: Simple match against grant candidates; no cascade.
- **Root cause**: `concept-doc-not-consulted` at M1 permission-engine design.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/permissions/engine.rs:311`](../../../../../../modules/crates/domain/src/permissions/engine.rs#L311) `step_5_scope_resolution`; `cascade_resolve` if called.
- **Test evidence**: M1 permission-check acceptance tests cover simple cases (reader grants explicit scope). Worked-example scenarios 4–6 (multi-scope) have no acceptance coverage.
- **Grep for regression**: `grep -n "cascade_resolve\|base_project.*base_org" modules/crates/domain/src/permissions/` — expect expanded impl post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Implement full 5-tier cascade per concept Figure 4. Add base_project/base_org tie-breaker. Implement intersection fallback. Add acceptance tests per worked-example scenarios 4–6. Pair with D-new-03 (selector grammar) since cascade uses rich predicates.
- **Implementation chunk this belongs to**: CH-07
- **Dependencies on other drifts**: D-new-03 (tag-predicate selectors); D4.1 (advisory→hard gate flip makes cascade results actionable)
- **Estimated effort**: 3 engineer-days (cascade logic + tie-breaker + fallback + tests).
- **Risk to concept alignment if deferred further**: HIGH — multi-scope sessions (Shapes B/C/D) can't be correctly resolved; contractor-model security property broken.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none flagging the gap
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agents 2 + 3 both flagged; merged)
