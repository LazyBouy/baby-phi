<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-17 — Per-Session consent gating (`subordinate_required` approval-mode flow) incomplete; Step 6 is stub-only

## Identification
- **ID**: D-new-17
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `consent-policy`, `permission-gating`
- **Blocks**: Per-Session consent policy is non-functional at runtime
- **Blocked-by**: D-new-04 + D-new-05 (Consent struct + state machine)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Per-Session Consent"
- **Concept claim**: Templates auto-issue grants with `approval_mode:subordinate_required`; each read blocks until subordinate approves; default timeout = project_duration. Decision = Pending(AwaitingConsent) until responded.
- **Contradiction**: [`permissions/engine.rs:106`](../../../../../../modules/crates/domain/src/permissions/engine.rs#L106) `step_6_consent_gating` exists but is stub-only (checks for pre-existing Consent, doesn't dispatch subordinate_required requests). Grant struct has no `approval_mode` field. No timeout logic.
- **Classification**: `partially-honored`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Real-time subordinate approval flow.
- **Reality (shipped state at current HEAD)**: Stub check + decision returns Pending if Consent absent; no request dispatch; no timeout.
- **Root cause**: `cascading-upstream-deferral` (needs Consent fields D-new-04 + state machine D-new-05).

## Where visible in code
- **File(s)**: [`engine.rs:106+`](../../../../../../modules/crates/domain/src/permissions/engine.rs#L106); Grant struct missing approval_mode.
- **Test evidence**: Limited — existing acceptance exercises Implicit/One-Time paths, not Per-Session.
- **Grep for regression**: `grep -rn "subordinate_required\|ApprovalMode::" modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Add `ApprovalMode` enum to Grant. Extend templates that should use Per-Session policy to mint with `subordinate_required`. Step 6 body: if grant requires subordinate_required, look up Consent for (subordinate, scope, action); dispatch request + return Pending. Timeout via scheduled expiration.
- **Implementation chunk this belongs to**: CH-11 (part of CH-09/10/11 consent-model triad)
- **Dependencies on other drifts**: D-new-04, D-new-05
- **Estimated effort**: 2 engineer-days (after Consent base).
- **Risk to concept alignment if deferred further**: HIGH — Per-Session policy is conceptually available to operators but cannot be configured effectively.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: engine.rs Step 6 comment
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
