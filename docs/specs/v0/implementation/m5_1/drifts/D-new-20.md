<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-20 — Contractor-model logic: reader's base_org ceiling does NOT reach into sessions of scopes they aren't a member of

## Identification
- **ID**: D-new-20
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `multi-scope`, `contractor-model`, `security-boundary`
- **Blocks**: Worked-example contractor scenarios; correct multi-org trust model
- **Blocked-by**: D-new-06 (multi-scope cascade)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §"Contractor scenario"; [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Subject-Side Reach Is Bounded by Scope Membership"
- **Concept claim**: When a reader operates in another org's scope, they follow that scope's rules — their base_org ceiling does NOT reach into sessions belonging to scopes they aren't a member of.
- **Contradiction**: No contractor-specific logic at Step 2a (ceiling) or Step 5 (scope resolution). The reader's base_org ceiling applies uniformly; no membership check.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Contractor bounds enforced at Step 2a/5.
- **Reality (shipped state at current HEAD)**: Uniform ceiling.
- **Root cause**: `concept-doc-not-consulted`.

## Where visible in code
- **File(s)**: `permissions/engine.rs` Step 2a + Step 5.
- **Test evidence**: None (no contractor-scenario acceptance).
- **Grep for regression**: `grep -rn "contractor\|scope_membership_check" modules/crates/domain/src/permissions/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: At Step 2a, scope ceiling to only those scopes where reader is a member. At Step 5, exclude ceilings from scopes outside membership. Add acceptance tests per worked-example contractor scenario.
- **Implementation chunk this belongs to**: CH-07
- **Dependencies on other drifts**: D-new-06
- **Estimated effort**: 1.5 engineer-days.
- **Risk to concept alignment if deferred further**: MEDIUM — multi-org trust model broken; contractor scenarios not supported.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
