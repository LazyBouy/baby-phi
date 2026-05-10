<!-- Last verified: 2026-05-09 by Claude Code (filed by CH-18 P3; cycle hex `c77937bc`) -->

# D-CH18-FOLLOWUP-02 — adopt.rs submit-side gating deferred — adoption AR's `requestor != input.actor` structural mismatch with F3.B.create-side.a

## Identification
- **ID**: D-CH18-FOLLOWUP-02
- **Phase of origin**: CH-18 P3 (cycle hex `c77937bc`, 2026-05-09)
- **Discovery source**: `cycle-implementation-deferral` (CH-18 P2b synthetic-Draft probe wiring at the 8 production submit sites uncovered the 9th site `templates/adopt.rs` does not satisfy F3.B.create-side.a's `ar.requestor == input.actor` precondition)
- **Date discovered**: 2026-05-09
- **Status**: `discovered`
- **Bucket**: B — concept-doc-fidelity gap; one of the 9 submit sites does not match the F3.B.create-side.a probe pattern shipped at the other 8
- **Severity**: LOW–MEDIUM (operator-not-impacting; structural cleanliness gap — the adopt.rs handler's existing first-Human-in-org gate at line 79–83 fills the equivalent defence-in-depth role)
- **Tags**: `auth-request-acl`, `adoption-ar`, `admin-on-behalf-of-ceo`, `submit-side-wiring`, `forward-defensive`, `structural-mismatch`
- **Blocks**: nothing today (the F3.B.create-side.a defence-in-depth INTENT is fulfilled via the handler's existing first-Human-in-org gate; the structural mismatch only matters for typed-error consistency across all 9 submit sites)
- **Blocked-by**: future M6+ chunk introducing typed admin-on-behalf-of-CEO authorisation as a typed-principal class; once that exists, the matrix's Submit cell can admit `OnBehalfOfRequestor { delegated_via: ... }` and the kernel-skip can be removed

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" lines 130–144 — the `Draft × Submit by requestor` cell (line 136). Plus the matrix's structural assumption that `Submit` is a requestor-only operation (no admin-on-behalf-of column).
- **Concept claim**: F3.B.create-side.a (user-locked at gate-1) prescribes `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)` BEFORE every `create_auth_request` call as defence-in-depth. The check is definitionally redundant at construction (`ar.requestor == input.actor` by construction at all production sites except adopt.rs); it serves as a typed-error tripwire if a future refactor decouples requestor from input.actor.
- **Contradiction**: 8 of 9 production submit-sites satisfy `ar.requestor == PrincipalRef::Agent(input.actor)` by construction; CH-18 P2b shipped a synthetic-Draft probe pattern (`let probe = AuthRequest { state: Draft, ..ar.clone() };`) at all 8. The 9th site — `server::platform::templates::adopt::create_adoption` — sets `ar.requestor = PrincipalRef::Agent(ceo)` via `build_adoption_request:90`, while `input.actor` is the platform-admin (distinct agent IDs). Even with synthetic-Draft, the matrix returns `RequestorOnlyOperation` because actor != ceo, breaking the existing `acceptance_authority_templates::adopt_c_creates_approved_ar_and_blocks_duplicate` test if naively wired. P2b shipped a 14-line kernel-skip-equivalent comment at `adopt.rs:111` documenting the structural mismatch. The F3.B.create-side.a defence-in-depth coverage is therefore 8/9 callsites at CH-18 close.
- **Classification**: `partially-honored` (8 of 9 submit sites honour F3.B.create-side.a's defence-in-depth wiring at CH-18; the 9th is documented as a kernel-skip-equivalent structural mismatch with the matrix shape, NOT a missed wiring step).
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (CH-18 plan §3 Per-file callsite breakdown row 12 — `templates/adopt.rs:96`):
  > *"F3.B.create-side.a: insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)?;` BEFORE the `create_auth_request` call. Definitionally redundant (`ar.requestor == input.actor` by construction); defence-in-depth."*
- **Plan said** (CH-18 plan §5 §D56.5 sub-decision):
  > *"(c) 8 submit sites at `templates/adopt.rs:96`, `projects/create.rs:470`, `defaults/put.rs:88`, `secrets/add.rs:107`, `mcp_servers/{register,patch_tenants,archive}.rs`, `model_providers/{register,archive}.rs`."*
- **Reality**: P2b discovered that `templates/adopt.rs` does NOT satisfy the plan's "by construction `ar.requestor == input.actor`" assumption. The adoption AR is built at `build_adoption_request:90` with `requestor = PrincipalRef::Agent(ceo)` (the CEO of the org being adopted into), not with `requestor = PrincipalRef::Agent(input.actor)` (the platform-admin invoking the endpoint). Even the synthetic-Draft probe pattern (forcing `state: Draft` so the matrix Submit cell is reachable) returns `Err(RequestorOnlyOperation { state: Draft, intended_op: Submit })` because `input.actor != ceo`. The adoption flow is intentionally **admin-on-behalf-of-CEO**: the platform-admin runs the endpoint to seed the org's bootstrap CEO, but the AR records the CEO as the requestor for downstream provenance. This pattern is structurally outside the matrix's concept (concept doc 02 line 136 names "Requestor" as the only Draft-Submit principal-class). P2b shipped a 14-line kernel-skip-equivalent comment at `adopt.rs:111` documenting the structural mismatch + cross-referencing this drift; the comment notes that the existing first-Human-in-org gate at `adopt.rs:79-83` (admin must be in org with a Human CEO present) fills the equivalent defence-in-depth role.

## Where visible in code (post-CH-18)
- **Files**:
  - `server/src/platform/templates/adopt.rs:111` — 14-line kernel-skip-equivalent comment documenting the structural mismatch (AR.requestor = ceo, input.actor = platform-admin); cross-references this drift.
  - `server/src/platform/templates/adopt.rs:79-83` — existing first-Human-in-org gate that fills the equivalent defence-in-depth role (admin must be in org with a Human CEO present).
  - `server/src/platform/templates/adopt.rs:90` — call to `build_adoption_request` which sets `ar.requestor = PrincipalRef::Agent(ceo)`.
  - `server/src/platform/templates/builders.rs::build_adoption_request:90` — the AR construction site that sets requestor = ceo.
  - `domain/src/auth_requests/access.rs::check_auth_request_access` — predicate body; matrix's `Draft × Submit by Requestor` cell returns `RequestorOnlyOperation` when principal != ar.requestor.
- **Test evidence**:
  - `acceptance_authority_templates::adopt_c_creates_approved_ar_and_blocks_duplicate` (existing test) passes because the kernel-skip-equivalent path at `adopt.rs:111` does not invoke `check_auth_request_access`. Adding the F3.B.create-side.a wiring without this drift would break the test.
  - No new test ships at CH-18 P2b for the adopt.rs site — the structural mismatch precludes a "submit by actor → 200" assertion at this handler. The other 8 submit-side sites have `*_submit_access_test.rs` files per plan §8 MUST-SHIP.
- **Grep for regression**: `grep -n "kernel-skip-equivalent\|D-CH18-FOLLOWUP-02" /root/projects/phi/baby-phi/modules/crates/server/src/platform/templates/adopt.rs` — at CH-18 close, expect ≥ 1 hit on the 14-line comment block; at M6+ remediation, expect 0 hits when typed admin-on-behalf-of-CEO authorisation lands.

## Required follow-up
- **What needs to happen**: a future M6+ chunk introduces typed admin-on-behalf-of-CEO authorisation as a typed-principal class. Two viable approaches:
  - **Approach 1** — extend `IntendedOp` matrix with an `OnBehalfOfRequestor { delegated_via: AgentId }` variant on the Submit cell. Concept doc 02 §"Per-State Access Matrix" line 136 would gain a new column for "Delegated Submitter (admin-on-behalf-of-CEO)" — concept-doc refresh required. The `classify_principal` body would gain a branch matching the delegation-relationship; the matrix's Submit cell would return Ok for delegated submitters.
  - **Approach 2** — extend `PrincipalRef` with a new variant `PrincipalRef::OnBehalfOf { delegating: PrincipalRef, actor: AgentId }`. The classifier maps this to `PrincipalClass::Requestor` if `delegating == ar.requestor`. Heavier than Approach 1 (touches every PrincipalRef consumer); more general — supports admin-on-behalf-of for arbitrary delegations beyond CEO bootstrap.
- **Tests required**:
  - Unit test in `auth_requests::access::tests`: `admin_on_behalf_of_ceo_submit_returns_ok` — fixture builds a Draft adoption AR with `requestor = ceo` + admin-on-behalf-of relationship; assert `check_auth_request_access(&probe, &admin_with_obo, IntendedOp::Submit) == Ok(())`.
  - Integration test in `server/tests/template_adopt_submit_access_test.rs` — admin invokes adopt endpoint; the F3.B.create-side.a wiring is now active; the synthetic-Draft probe returns Ok; the adoption AR creates successfully.
  - Regression test: the existing `acceptance_authority_templates::adopt_c_creates_approved_ar_and_blocks_duplicate` test continues to pass.
- **Acceptance**: the F3.B.create-side.a defence-in-depth INTENT covers all 9 production submit sites uniformly; the kernel-skip-equivalent comment at `adopt.rs:111` is removed; the adopt.rs handler invokes `check_auth_request_access` (or its successor admin-on-behalf-of-aware variant) BEFORE every `create_auth_request` call.

## Closing chunk
- **TBD** — likely a future M6+ chunk that introduces typed admin-on-behalf-of authorisation as a first-class principal-class. Not yet allocated in forward-scope. Candidate names: `CH-NN-admin-on-behalf-of-typed-principal`, `CH-NN-delegated-requestor-matrix-extension`. Either Approach 1 (matrix-extension) or Approach 2 (PrincipalRef variant) is acceptable per CH-18 ADR-0056 §D56.9 sub-fork resolution shape.

## Lifecycle
- **2026-05-09 — `discovered`** — filed by CH-18 P3 under cycle hex `c77937bc`. Discovered during P2b synthetic-Draft probe wiring at the 8 production submit sites; the 9th site `templates/adopt.rs` was found to have a structural `requestor != input.actor` mismatch that does not match F3.B.create-side.a's by-construction precondition. Orchestrator-approved at gate-2 P2b review; user accepted the synthetic-Draft probe pattern at the 8 sites + the kernel-skip-equivalent comment at the 9th + this drift filing as the path forward. Mirrors CH-13's `D-CH13-FOLLOWUP-01`, CH-14's `D-CH14-FOLLOWUP-01`, and CH-18's `D-CH18-FOLLOWUP-01` patterns: chunk closes one axis of a multi-axis concept-doc claim; the deferred axis tracked here.

## Cross-references
- CH-18 plan: [`baby-phi/docs/specs/plan/build/ch-18-authrequest-per-state-acl-enforcement-c77937bc/plan.md`](../../../../plan/build/ch-18-authrequest-per-state-acl-enforcement-c77937bc/plan.md) §3 Per-file callsite breakdown row 12 + §5 §D56.5 sub-decision (c) + §D56.9 sub-fork resolutions under F3.B.
- ADR-0056: [`m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md`](../../m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.5 (8 submit sites) + §D56.8 (kernel-internal fast-path skip-list) + §D56.9 (sub-fork resolutions under F3.B).
- Architecture doc: [`m5_2/architecture/auth-request-access-acl.md`](../../m5_2/architecture/auth-request-access-acl.md) §4 17-callsite wiring map (the templates/adopt.rs row notes the kernel-skip-equivalent) + §6 forward-defensive descope (lists this drift as a deferred axis) + §8 synthetic-Draft probe pattern (closing subsection on the templates/adopt.rs exception).
- D-new-12 (parent drift, closed at CH-18 P4): [`D-new-12.md`](D-new-12.md).
- Sister drifts (one-axis-deferred-by-followup-drift): [`D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md`](D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md), [`D-CH14-FOLLOWUP-01.md`](D-CH14-FOLLOWUP-01.md), [`D-CH13-FOLLOWUP-01.md`](D-CH13-FOLLOWUP-01.md), [`D-CH12-FOLLOWUP-01.md`](D-CH12-FOLLOWUP-01.md), [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md), [`D-CH07-FOLLOWUP-01.md`](D-CH07-FOLLOWUP-01.md).
- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" line 136 (the `Draft × Submit by requestor` cell).
- Affected files (post-remediation candidate edit sites): `server/src/platform/templates/adopt.rs:96-112` (replace kernel-skip-equivalent comment with F3.B.create-side.a wiring), `domain/src/auth_requests/access.rs::classify_principal` (add admin-on-behalf-of branch), potentially `domain/src/model/nodes.rs::PrincipalRef` (if Approach 2 is chosen — new variant).
