<!-- Last verified: 2026-05-09 by Claude Code (filed by CH-18 P0 scaffold; cycle hex `c77937bc`) -->

# D-CH18-FOLLOWUP-01 — Admin/auditor role-discrimination deferred from CH-18 (per F3.B.role.b)

## Identification
- **ID**: D-CH18-FOLLOWUP-01
- **Phase of origin**: CH-18 P0 scaffold (cycle hex `c77937bc`, 2026-05-09)
- **Discovery source**: `cycle-plan-deferral` (plan §3 Forks F3.B.role sub-fork — planner-auto-resolved at v2 re-plan to F3.B.role.b under user-locked F3.B path)
- **Date discovered**: 2026-05-09
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap; one column of the per-state matrix partial-honoured at CH-18; admin/auditor classifier deferred to M6+
- **Severity**: MEDIUM
- **Tags**: `auth-request-acl`, `admin-classifier`, `observer-role`, `forward-defensive`, `role-discrimination`
- **Blocks**: nothing today (the four shipped principal classifiers — `Requestor`, `SlotApprover`, `Bootstrap/SystemGenesis`, `OtherAgent` — give a complete partition over `PrincipalRef` for matrix evaluation; the matrix's "admin/auditor read-allow" cell is collapsed into "OtherAgent → DENY" but no shipped flow needs it because the ceo IS the AR-creator/slot-approver in the typical fixture)
- **Blocked-by**: future M6+ chunk wiring `Agent.role` (`domain::model::nodes::AgentRole`) lookup OR a Permission Check delegation path; either approach changes the classifier shape

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" line 134 — *"Observer (admin/auditor) — read at every state"*. Also lines 136–144 for every state row's last column showing `read` for the Observer principal-class.
- **Concept claim**: An admin or auditor (the "Observer" principal-class) holds **read access at every state** of an AuthRequest, including ARs they did not submit and slots they do not own. This is a cross-cutting read-allow column on the per-state matrix orthogonal to the requestor / slot-approver / resource-owner classifiers.
- **Contradiction**: NONE today on the typical-fixture shape — the ceo (Admin role) in dashboard tests is also the AR-creator or slot-approver, so the requestor / slot-approver classifier returns Ok before the missing admin classifier matters. The contradiction surfaces only when an Admin viewer reads ANOTHER agent's AR — at which point CH-18's `classify_principal` returns `OtherAgent` and `check_auth_request_access` returns `Err(NotAuthorisedForRead)`. The matrix says `read` should be `Ok` for the admin column; CH-18 ships `Err`.
- **Classification**: `partially-honored` (4 of 5 principal-class columns honoured at CH-18: requestor, slot-approver own/other slot, resource-owner-as-requestor stub, bootstrap/system-genesis fast-path; admin/auditor column collapsed into OtherAgent DENY).
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (CH-18 plan §F3.B.role.b — planner recommendation auto-applied at v2 re-plan):
  > *"F3.B.role.b — Treat all non-requestor / non-slot-approver / non-bootstrap principals as 'Other Agent' (DENY for read-at-every-state); flag as a known gap and file `D-CH18-FOLLOWUP-01` drift to wire admin/auditor classification at M6+."*
- **Plan said** (CH-18 plan §3 sub-fork F3.B.role rationale):
  > *"F3.B.role.c (resolve admin-class via `OrgMembership.role` lookup) would require either (i) wiring a new `repo.get_agent_role_in_org(viewer, org)` method (Repository-shape change → A5 K8s axis flip), OR (ii) loading the full `Agent` struct in every read handler and inspecting `agent.role`. Both are heavier than (b)'s defer-via-followup-drift."*
- **Plan said** (CH-18 plan §1):
  > *"The matrix's 'Observer (admin/auditor) — read at every state' column is partial-honoured: requestor reads are permitted; bootstrap/system-genesis reads are permitted; slot-approvers read their own AR; **all other principals get `Err(NotAuthorisedForRead)`** — including the org's CEO when reading another agent's AR (a known gap; documented as `D-CH18-FOLLOWUP-01`)."*
- **Reality**: matches the plan exactly. CH-18 ships the four-classifier partition (`Requestor`, `SlotApprover`, `Bootstrap`, `OtherAgent`); admin/auditor classification is collapsed into `OtherAgent → DENY`. The dashboard's silent post-filter at `dashboard.rs:273,293` (per F3.B.list-filter.a) further hides ARs from the typical Admin viewer when they are not the requestor or a slot-approver. No production flow today depends on the admin-read-bypass; the chunk's existing dashboard happy-path tests pass because the typical-fixture ceo is also the AR-creator/slot-approver.

## Where visible in code (post-CH-18)
- **Files**:
  - `domain/src/auth_requests/access.rs` — `classify_principal` returns `OtherAgent` for any `PrincipalRef::Agent(...)` whose id is neither `ar.requestor` (Agent variant) nor any `slot.approver` across `ar.resource_slots[*].approver_slots[*]`, and the AR is not a bootstrap AR (`is_bootstrap_ar(ar) == false`).
  - `server/src/platform/orgs/dashboard.rs:273,293` — silent post-filter applies `check_auth_request_access(.., IntendedOp::Read)` to every list entry; non-admin/non-requestor/non-slot-approver viewers see strictly fewer ARs.
  - `server/src/platform/orgs/show.rs:63` — count-aggregate uses the same post-filter.
- **Test evidence**: `server/tests/dashboard_silent_filter_test.rs` (NEW at CH-18 P2b) asserts the post-filter behaviour for non-requestor / non-slot-approver viewer; the test does NOT exercise an Admin-role viewer because the matrix-bypass is not yet implemented.
- **Grep for regression**: `grep -n "OtherAgent\|admin_class\|AgentRole::Admin" /root/projects/phi/baby-phi/modules/crates/domain/src/auth_requests/access.rs` — at CH-18 close, expect 0 hits on `AgentRole::Admin` (deferred); at M6+ remediation, expect ≥ 1 hit when admin-classifier wires in.

## Required follow-up
- **What needs to happen**: when a future M6+ chunk extends AR observability to admin/auditor viewers (e.g., a compliance-audit dashboard that surfaces ALL org ARs to the org's CEO regardless of slot-membership), the `classify_principal` body MUST gain an additional admin-classifier branch. Two viable approaches per plan §F3.B.role:
  - **Approach 1** — wire `Agent.role: AgentRole` lookup. Either (i) add a Repository method `repo.get_agent_role_in_org(viewer: AgentId, org: OrgId) -> Option<AgentRole>` (Repository-shape change → A5 K8s axis flip; would require a `CHK8S-D-NN` ledger entry), or (ii) load the full `Agent` struct via the existing `repo.get_agent(viewer)` in every AR read handler before invoking `check_auth_request_access`, then pass `agent.role` as a new `intended_op_principal_role: Option<AgentRole>` parameter to the predicate.
  - **Approach 2** — Permission Check delegation. Treat "admin/auditor reads ANY org AR" as a permission-check-gated reach (e.g., `Action::Inspect` on `auth_request_object` per concept doc 03 §"Standard Action Vocabulary" — `Inspect` already exists in `Action::CANONICAL`). The admin's role-defaults Template grant (per concept doc 07 §"Templates" — admins get a baseline org-Inspect grant) would carry the necessary descends-from chain to the Permission Check engine; the AR-touching handler would call `engine.evaluate(...)` BEFORE invoking the per-state matrix and short-circuit on a positive Inspect-on-AR decision.
- **Tests required**:
  - Unit test in `auth_requests::access::tests`: `admin_role_reads_any_org_ar_returns_ok` — fixture builds an Admin agent + a non-admin agent's AR; assert `check_auth_request_access(&ar, &admin_principal, IntendedOp::Read) == Ok(())`.
  - Integration test in `server/tests/dashboard_admin_view_test.rs`: an Admin viewer sees ALL org ARs in the dashboard list (post-filter is bypassed for admin-role viewers).
- **Acceptance**: the admin/auditor "Observer" column of concept doc 02 lines 130–144 is honoured end-to-end — Admin viewer reads any AR at any state without DENY; auditor (a separate AgentRole if introduced at M6+) likewise reads at every state.

## Closing chunk
- **TBD** — likely a future M6+ chunk that introduces a compliance-audit dashboard OR an explicit admin-role read-bypass surface. Not yet allocated in forward-scope. Candidate names: `CH-NN-admin-auditor-ar-read-bypass`, `CH-NN-org-compliance-audit-dashboard`. Either approach (AgentRole lookup vs Permission Check delegation) is acceptable per CH-18 ADR-0056 §D56.9 sub-fork resolution.

## Lifecycle
- **2026-05-09 — `discovered`** — filed by CH-18 P0 scaffold under cycle hex `c77937bc` per plan §F3.B.role.b auto-resolution. CH-18 ships the four-classifier partition (`Requestor`, `SlotApprover`, `Bootstrap`, `OtherAgent`) and silently denies admin/auditor reads on other-agent ARs; the chunk's dashboard happy-path tests still pass because the typical-fixture ceo is the AR-creator/slot-approver. Mirrors CH-13's `D-CH13-FOLLOWUP-01` and CH-14's `D-CH14-FOLLOWUP-01` patterns: chunk closes one axis of a multi-axis concept-doc claim; the deferred axis tracked here.

## Cross-references
- CH-18 plan: [`baby-phi/docs/specs/plan/build/ch-18-authrequest-per-state-acl-enforcement-c77937bc/plan.md`](../../../../plan/build/ch-18-authrequest-per-state-acl-enforcement-c77937bc/plan.md) §F3.B.role + §3 sub-fork F3.B.role rationale + §1 known-gap restatement.
- ADR-0056: [`m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md`](../../m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.5 (wiring depth + descoped resource-owner-lookup-helper note) + §D56.9 (sub-fork resolutions under F3.B).
- D-new-12 (parent drift, closed at CH-18 P4): [`D-new-12.md`](D-new-12.md).
- Sister patterns (one-axis-deferred-by-followup-drift): [`D-CH14-FOLLOWUP-01.md`](D-CH14-FOLLOWUP-01.md), [`D-CH13-FOLLOWUP-01.md`](D-CH13-FOLLOWUP-01.md), [`D-CH12-FOLLOWUP-01.md`](D-CH12-FOLLOWUP-01.md), [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md), [`D-CH07-FOLLOWUP-01.md`](D-CH07-FOLLOWUP-01.md).
- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" line 134.
- Affected files (post-remediation candidate edit sites): `domain/src/auth_requests/access.rs` (classify_principal body), `server/src/platform/orgs/dashboard.rs:273,293` (post-filter shape), `server/src/platform/orgs/show.rs:63` (count-aggregate post-filter), potentially new Repository method `domain/src/repository.rs` (if Approach 1.i is chosen).
