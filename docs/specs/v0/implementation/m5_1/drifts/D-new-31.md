<!-- Last verified: 2026-04-29 by Claude Code (CH-05: terminally remediated. Rule C of `validate_published_manifest` rejects `[Modify]` on the bare `tag` fundamental — the cleanest publish-time discriminator that catches "tool intends to write reserved tag namespaces" without false positives on legitimate composite-modify tools (memory, session, etc., which all internally include Tag but route writes through composite data). The full reserved-namespace prefix list — `#kind:`, `delegated_from:`, `derived_from:`, plus auto-generated `{kind}:` per `Composite::ALL` — ships via `reserved_namespace_prefixes()` for downstream consumers (CH-12 frozen-tag enforcement). ADR-0044 §D44.6 ratifies. Status flipped to `remediated`.) -->

# D-new-31 — Reserved-namespace write rejection at tool-publish time (separate from D-new-07 general validator scope)

## Identification
- **ID**: D-new-31
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
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
- 2026-04-29 — `in-chunk-plan` — assigned to CH-05 as a sub-case of D-new-07 (paired closure).
- 2026-04-29 — `remediated` — CH-05 chunk-seal — Rule C of `validate_published_manifest` rejects `[Modify]` on bare `tag` resource. The full reserved-namespace prefix list ships via `reserved_namespace_prefixes()`. [ADR-0044](../../m5_2/decisions/0044-publish-time-manifest-validator.md) §D44.6 ratifies the design choice (bare-tag interpretation over a more granular target_kinds-based check, which would surface false positives).
