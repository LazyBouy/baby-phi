<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-07 — Publish-time manifest validator missing (rejects missing fundamentals / #kind: / invalid action-fundamental pairs / reserved-namespace writes)

## Identification
- **ID**: D-new-07
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `tool-registry`, `publish-time-validation`, `security-boundary`
- **Blocks**: Tool safety guarantee (malformed manifests can't be caught); D-new-10 (action-fundamental applicability enforcement); D-new-27 (reserved-namespace write rejection)
- **Blocked-by**: D-new-09 (action vocabulary needs constants before validator can check)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Manifest Validation at Publish Time"; [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"What v0 Validates vs Future Enhancements"
- **Concept claim**: Publish-time validator rejects manifests missing fundamentals, missing `#kind:` for composites, declaring inconsistent (fundamental, composite) pairs, or writing to reserved namespaces. Blanket `#kind:*` accepted with warning.
- **Contradiction**: No validator exists. `ToolAuthorityManifest` nodes can be stored with any shape.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Publish-time validation gates enforce concept invariants.
- **Reality (shipped state at current HEAD)**: `ToolAuthorityManifest` struct exists; no validator function walks submitted manifests.
- **Root cause**: `concept-doc-not-consulted` — manifest persistence shipped before validation.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/permissions/manifest.rs`](../../../../../../modules/crates/domain/src/permissions/manifest.rs) (Manifest/ManifestEntry); no validator module.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "validate_manifest\|ManifestValidator" modules/crates/domain/src/` — expect 0 hits while drift open.

## Remediation scope (estimate only)
- **Approach (sketch)**: New `permissions::manifest::validator` module with `validate_published_manifest(manifest) -> Result<(), ValidationError>`. Checks: (a) every declared composite has matching `#kind:` selector, (b) action-fundamental compatibility against matrix (D-new-10), (c) reserved-namespace writes rejected. Wire into ToolDefinition publish path.
- **Implementation chunk this belongs to**: CH-05
- **Dependencies on other drifts**: D-new-09 (action vocab constants); D-new-10 (fundamental matrix)
- **Estimated effort**: 2 engineer-days.
- **Risk to concept alignment if deferred further**: HIGH — tool manifests can declare anything, including dangerous reserved-tag writes; concept-specified security boundary absent.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agents 2 + 3 both flagged; merged)
