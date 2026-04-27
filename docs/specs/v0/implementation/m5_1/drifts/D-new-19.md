<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-19 — `audit_class` composition rule (strictest-of org-default / template-AR / per-grant override) not enforced

## Identification
- **ID**: D-new-19
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `audit-composition`, `security-policy`
- **Blocks**: Templates issuing grants with loosened audit_class can bypass operator intent
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates"
- **Concept claim**: Strictest of (org default, template AR, per-grant override) wins. Override can only escalate, not loosen. Composed result recorded on Grant at issuance.
- **Contradiction**: No composition logic in template-builder code; grants stamp audit_class directly without composing against org default or template AR.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Compose strictest-wins at mint time.
- **Reality (shipped state at current HEAD)**: Field present on Grant; no composer.
- **Root cause**: `concept-doc-not-consulted` during M2/M3 template + audit work.

## Where visible in code
- **File(s)**: template builders in `domain/src/templates/*.rs`; Grant mint path.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "compose_audit_class\|strictest_audit_class" modules/crates/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Helper `compose_audit_class(org_default, template_ar, override) -> AuditClass` using AuditClass ordering (Logged < Alerted < Elevated). Wire into grant-mint. Reject overrides that loosen.
- **Implementation chunk this belongs to**: CH-13
- **Dependencies on other drifts**: none
- **Estimated effort**: 1 engineer-day.
- **Risk to concept alignment if deferred further**: MEDIUM — audit integrity compromised; mis-configured templates silently downgrade audit class.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
