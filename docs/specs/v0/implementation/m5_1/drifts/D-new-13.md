<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-13 — `allocate` vs `transfer` cardinality semantics not enforced (sender authority preservation vs revocation)

## Identification
- **ID**: D-new-13
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `permission-model`, `authority-cardinality`, `security-boundary`
- **Blocks**: Correct accounting of grant cardinality (who still holds authority after a delegation)
- **Blocked-by**: D-new-09 (typed actions must distinguish allocate vs transfer)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics"
- **Concept claim**: `allocate` is additive — sender retains full share; `transfer` is exclusive — sender loses authority. Arc::clone semantics for allocate (multiple holders); move semantics for transfer (sole holder).
- **Contradiction**: Grant action is `String`; no cardinality distinction. Both allocate and transfer appear the same at persistence; transfer doesn't revoke sender's grant.
- **Classification**: `silent-in-code` — cardinality not enforced
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: allocate preserves sender; transfer revokes sender. Different cardinality at runtime.
- **Reality (shipped state at current HEAD)**: Both just add a grant for the recipient; sender's grant stays.
- **Root cause**: `concept-doc-not-consulted` at M1 grant-mint logic.

## Where visible in code
- **File(s)**: Grant-mint paths; grep `Grant { .. action:` in templates or auth_requests.
- **Test evidence**: None.
- **Grep for regression**: Check transfer-path revokes sender's grant post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Distinguish allocate vs transfer in mint logic. Transfer path must revoke sender's matching grant atomically with issuing recipient's grant (compound tx). Acceptance tests cover both cardinalities.
- **Implementation chunk this belongs to**: CH-08
- **Dependencies on other drifts**: D-new-09 (typed actions)
- **Estimated effort**: 2 engineer-days.
- **Risk to concept alignment if deferred further**: HIGH — transfer-scope bugs compound silently; authority over-accounted across multiple holders.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
