<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-15 — AuthRequest 2-tier retention (90-day active + archived tier) defined mathematically but not wired (no archival action, no retrieval gate)

## Identification
- **ID**: D-new-15
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `retention-policy`, `audit-archival`
- **Blocks**: nothing runtime; concept compliance at M7b operations
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Retention Policy — Two-Tier Storage"
- **Concept claim**: Active window (90 days default) then archived tier. Archived ARs require explicit `inspect_archived` approval (human_required) to retrieve.
- **Contradiction**: [`domain/src/auth_requests/retention.rs`](../../../../../../modules/crates/domain/src/auth_requests/retention.rs) defines `active_until()` + `is_archive_eligible()` math, but no archival transition code or retrieval-gating logic.
- **Classification**: `partially-honored`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Full 2-tier storage with retrieval gating.
- **Reality (shipped state at current HEAD)**: Math helpers present; no archival action; no `inspect_archived` approval path.
- **Root cause**: `cascading-upstream-deferral` (retention at M7b ops per base plan).

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/auth_requests/retention.rs`](../../../../../../modules/crates/domain/src/auth_requests/retention.rs) — math helpers only.
- **Test evidence**: Unit tests for `active_until()` but no end-to-end archival test.
- **Grep for regression**: `grep -rn "archive_auth_request\|inspect_archived" modules/crates/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Scheduled job archives eligible ARs (separate table or flag). Retrieval handler gates on `inspect_archived` grant. Audit event on archival transition.
- **Implementation chunk this belongs to**: M7b-DEFERRED-01
- **Dependencies on other drifts**: none
- **Estimated effort**: 2 engineer-days at M7b.
- **Risk to concept alignment if deferred further**: LOW at M5; MEDIUM at M7b production.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: retention.rs module doc
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
