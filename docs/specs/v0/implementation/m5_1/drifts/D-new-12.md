<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P3: Status flipped `discovered` → `remediated` with CH-18 ✓ marker; cycle hex `c77937bc`. ADR-0056 §D56.1–§D56.10 ships typed `check_auth_request_access` predicate + 17 production callsite wiring + 7 kernel-internal fast-path skip-list + Alerted-class `auth_request.access_denied` audit-event + Repository trait docstring contract. Two follow-up drifts filed: D-CH18-FOLLOWUP-01 (admin/auditor role-discrimination deferred to M6+ per F3.B.role.b) + D-CH18-FOLLOWUP-02 (adopt.rs submit-side wiring deferred — admin-on-behalf-of-CEO structural mismatch).) -->
<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-12 — AuthRequest per-state Access Control Matrix not enforced at persistence layer

## Identification
- **ID**: D-new-12
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
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
- 2026-05-09 — `remediated` — CH-18 ✓ (cycle hex `c77937bc`; ADR-0056 Accepted). Typed pure-function predicate `domain::auth_requests::access::check_auth_request_access(&AuthRequest, &PrincipalRef, IntendedOp) -> Result<(), AuthRequestAccessError>` at `domain/src/auth_requests/access.rs` captures concept doc 02 §"Per-State Access Matrix" lines 130–144 verbatim; 17 production callsites consult the predicate (4 mutation handlers at `templates/{approve,deny,revoke}.rs` + `projects/create.rs` slot-fill mutation; 5 read-side handlers at `dashboard.rs:273,293`, `show.rs:63`, `projects/create.rs:636` slot-fill read, plus the slot-fill principal assertion; 8 submit-side defence-in-depth synthetic-Draft probe sites at `projects/create.rs:470`, `defaults/put.rs`, `secrets/add.rs`, `mcp_servers/{register,patch_tenants,archive}.rs`, `model_providers/{register,archive}.rs`); 7 kernel-internal callsites documented as fast-path skips per ADR-0056 §D56.8 (events listener + cascade-revoke loop + bootstrap claim + find_adoption_ar helper + 3 Template A/C/D AR resolvers). New Alerted-class `auth_request.access_denied` audit-event builder at `domain/src/audit/events/m5_2/auth_request_access.rs` emits at the 4 mutation callsites only (per F3.B.list-filter.a — silent post-filter rule for list-side reads). Repository trait docstring contract on `get_auth_request` + `update_auth_request` documents the future-callsite invocation requirement per §D56.7. Two follow-up drifts filed: `D-CH18-FOLLOWUP-01` (admin/auditor role-discrimination deferred to M6+ per F3.B.role.b — concept doc 02 line 134 "Observer (admin/auditor) — read at every state" column partial-honoured) + `D-CH18-FOLLOWUP-02` (adopt.rs submit-side wiring deferred — admin-on-behalf-of-CEO structural mismatch with F3.B.create-side.a's `requestor == input.actor` precondition; 14-line kernel-skip-equivalent comment shipped at `adopt.rs:111`).
