<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-31 — Reserved-namespace write rejection at tool-publish time (separate from D-new-07 general validator scope)

## Identification
- **ID**: D-new-31
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: C — convention/pattern decision
- **Severity**: LOW
- **Tags**: `reserved-namespace`, `publish-time-gate`
- **Blocks**: D-new-08 enforcement completeness (reserved-namespace coverage is the pub-time half of the runtime gate)
- **Blocked-by**: D-new-07 (general manifest validator)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) §"Reserved Namespace Enforcement"
- **Concept claim**: Publish-time manifest validator rejects tool manifests that declare `[modify]` on reserved namespaces (`#kind:*`, `{kind}:*`, `delegated_from:*`, `derived_from:*`).
- **Contradiction**: No validator exists (D-new-07 general case); no specific reserved-namespace denylist.
- **Classification**: `silent-in-code`

## Remediation
- **Approach**: As part of D-new-07, define `RESERVED_NAMESPACES: &[&str]` constant + validator checks every tool manifest's action/resource pair against it.
- **Impl chunk**: CH-05
- **Risk**: LOW.

## Lifecycle
- 2026-04-24 — `discovered`
