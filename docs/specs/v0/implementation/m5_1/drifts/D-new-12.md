<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-12 — AuthRequest per-state Access Control Matrix not enforced at persistence layer

## Identification
- **ID**: D-new-12
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `auth-request-acl`, `security-boundary`
- **Blocks**: none
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix"
- **Concept claim**: Access to the AuthRequest record itself varies by state: e.g. Draft allows requestor read+modify; Pending allows approvers to respond; Approved locks modification but allows owner revocation; Denied/Revoked only allow audit reads.
- **Contradiction**: Repository accepts reads/writes without consulting state-dependent access rules.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Per-state ACL enforced on AuthRequest reads/writes.
- **Reality (shipped state at current HEAD)**: No per-state ACL checks.
- **Root cause**: `concept-doc-not-consulted` at M1.

## Where visible in code
- **File(s)**: AuthRequest persistence sites in `repo_impl.rs`; handlers in `server/src/handlers/` around AR state transitions.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "check_auth_request_access\|auth_request_state_acl" modules/crates/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Define `check_auth_request_access(ar, principal, intended_op) -> Result<(), AccessError>` with state-dependent rules. Wire into every repository read/write and every handler that touches AuthRequest state.
- **Implementation chunk this belongs to**: CH-18
- **Dependencies on other drifts**: none
- **Estimated effort**: 2 engineer-days.
- **Risk to concept alignment if deferred further**: MEDIUM — a requester could modify their own AR after it's Approved; an approver could peek at Draft ARs.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
