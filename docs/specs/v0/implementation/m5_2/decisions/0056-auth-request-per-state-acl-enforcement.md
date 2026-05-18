<!-- Last verified: 2026-05-09 by Claude Code (CH-18 gate-3-prep Trivial-multi orchestrator inline patch — §D56.5 header reframed "**8 submit handlers**" → "**8 wired submit handlers + 1 kernel-skip-equivalent**" + adopt.rs:96 bullet annotated KERNEL-SKIP-EQUIVALENT with explicit cross-ref to D-CH18-FOLLOWUP-02; documentary clarification with no semantic change to ADR claims; surfaces the 9-bullet-under-8-header inconsistency the chunk-implementer flagged at P4 close so auditors don't catch it as a hard contradiction.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P4 chunk-seal — Status flipped Proposed → **Accepted** at chunk-close; cycle hex `c77937bc`. ADR §D56.1–§D56.10 sub-decisions all shipped: typed pure-function predicate + 11-variant `IntendedOp` + 5-variant `AuthRequestAccessError` + 4-classifier `classify_principal` + 17 production callsite wiring (4 mutation handlers emit Alerted-class `auth_request.access_denied` audit-event on Err + 5 read-side handlers + 8 submit-side defence-in-depth synthetic-Draft probes) + Repository trait docstring contract on `get_auth_request`/`update_auth_request` + 7 kernel-internal fast-path skip-list documented + `PrincipalRef` `PartialEq, Eq` derive (F1.B). Two follow-up drifts filed: D-CH18-FOLLOWUP-01 (admin/auditor role-discrimination M6+ deferred per F3.B.role.b) + D-CH18-FOLLOWUP-02 (adopt.rs submit-side wiring deferred — admin-on-behalf-of-CEO structural mismatch with F3.B.create-side.a's `requestor == input.actor` precondition). 1529 tests passing within plan §8 band [1526–1542]; 4 CI guards green; phi-core import baseline preserved at 57.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P0 scaffold — ADR drafted as Proposed; sub-decisions §D56.1–§D56.10 populated per plan §5; Forks header captures gate-1 user-lock outcome F1.B / F2.A / F3.B (DIVERGENT from planner-recommended F3.A) / F4.A / F5.B; sub-forks under F3.B all auto-resolved per v2 re-plan: F3.B.role.b / F3.B.repo-shape.b / F3.B.create-side.a / F3.B.list-filter.a; cycle hex `c77937bc`) -->

# ADR-0056 — AuthRequest per-state ACL enforcement

**Status: Accepted**

**Date:** 2026-05-09
**Chunk:** CH-18
**Closes:**
- [`D-new-12`](../../m5_1/drifts/D-new-12.md) (MEDIUM, B) — AuthRequest per-state ACL not enforced. Concept doc `permissions/02-auth-request.md` §"Per-State Access Matrix" lines 130–144 specifies that access to an AuthRequest record itself varies by state (Draft allows requestor read+modify+submit+cancel; Pending allows approvers to respond on their own slot; Approved locks modification but allows owner revocation; Denied/Revoked admit only audit reads). The Repository layer accepts reads + writes today without consulting these state-dependent rules. CH-18 closes the drift via (a) typed pure-function `check_auth_request_access(&AuthRequest, &PrincipalRef, IntendedOp) -> Result<(), AuthRequestAccessError>`; (b) wiring into 17 production AR-touching handlers (8 submit + 5 read + 4 mutation) per F3.B locked path; (c) Alerted-class `auth_request.access_denied` audit-event at the 4 mutation callsites only (per F3.B.list-filter.a silent-post-filter rule for list-side reads); (d) Repository trait docstring contract on `get_auth_request` + `update_auth_request` per §D56.7; (e) NEW drift `D-CH18-FOLLOWUP-01` filing the deferral of admin/auditor role-discrimination to M6+ per F3.B.role.b.

---

## Context

(Body fills at P1–P3 — context paragraph will pin concept doc `permissions/02-auth-request.md` §"Per-State Access Matrix" lines 130–144 as canonical for the (state × principal-class × allowed-ops) matrix; §"Multi-Approver Dynamics" lines 175–179 as canonical for per-slot independence + slot-holder reconsider window + resource-owner override; §"State Machine" lines 70–129 as canonical for the 9-state state-machine the matrix is indexed by; concept doc `permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" lines 7–22 as canonical for the closed 34-verb invariant rationale supporting D56.2's separate `IntendedOp` enum.)

## Forks

Forks user-locked at gate-1 (plan approval, 2026-05-09) to **F1.B / F2.A / F3.B (DIVERGENT from planner-recommended F3.A) / F4.A / F5.B**. The diverging fork — F3.B — explicitly broadens scope from F3.A's mutation-handler-only wiring to full Repository + dashboard + resolver + bootstrap wiring (17 callsites). v2 re-plan applied. Sub-forks under F3.B all auto-resolved at v2 to **F3.B.role.b / F3.B.repo-shape.b / F3.B.create-side.a / F3.B.list-filter.a** (planner-recommendations under the locked-path's spirit; no additional gate-1 user-lock required).

- F1 → **F1.B** (additive `PartialEq, Eq` derive on `PrincipalRef` — mechanical trait-derive change; cascade 0; closes CH-14 retro Row 5 type-derive precedent) — planner-recommended; auto-approved at gate-1.
- F2 → **F2.A** (NEW `AuthRequestAccessError` thiserror enum at `domain/src/auth_requests/access.rs`, mirrors `FrozenTagViolation` shape from CH-12 ADR-0049 §D49.4) — user-locked at gate-1.
- F3 → **F3.B** (full Repository + dashboard + resolver + bootstrap wiring — 17 production callsites) — **user-locked at gate-1, DIVERGENT from planner-recommended F3.A.**
- F4 → **F4.A** (NEW property-test file `domain/tests/auth_request_access_props.rs` per per-concern-per-file convention; CH-09/CH-10/CH-11 precedent) — planner-recommended; auto-approved at gate-1.
- F5 → **F5.B** (NEW Alerted-class `auth_request.access_denied` audit-event in `domain/src/audit/events/m5_2/auth_request_access.rs`; follows CH-15 `session.launch_denied` precedent and CH-12 `frozen_tag_write_rejected` shape) — user-locked at gate-1.

**F3.B sub-fork resolutions** (planner-auto-resolved at v2 re-plan; no additional user-lock required):

- F3.B.role → **F3.B.role.b** (defer admin/auditor role-discrimination to M6+ via NEW drift `D-CH18-FOLLOWUP-01`; current implementation classifies all non-requestor / non-slot-approver / non-bootstrap principals as "Other Agent" → DENY). Concept doc 02 line 134 names "Observer (admin/auditor) — read at every state"; that column is partial-honoured at v2.
- F3.B.repo-shape → **F3.B.repo-shape.b** (Repository trait stays principal-blind; access-checks live at the handler boundary above Repository — same pattern as CH-12 ADR-0049 §D49.5/§D49.7 tag-write contract). K8s A5 axis stays neutral; no `CHK8S-D-NN` ledger entry filed. F3.B.repo-shape.a (Repository methods gain `principal: PrincipalRef` parameter) was rejected on grounds of (i) parameter cascade through 2 Repository implementations + ~30 callsites + (ii) system-internal listeners + resolvers having no caller principal context, forcing synthetic-principal noise.
- F3.B.create-side → **F3.B.create-side.a** (wire `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)` BEFORE every `create_auth_request` call at all 8 submit-side handlers — defence-in-depth; the check is definitionally redundant at the construction site but provides a typed-error tripwire if a future refactor decouples `ar.requestor` from `input.actor`).
- F3.B.list-filter → **F3.B.list-filter.a** (silent post-filter at the 5 list-side reads — `dashboard.rs:273,293`, `show.rs:63`, `projects/create.rs:636` slot-fill read, plus the slot-fill principal assertion. NO audit-event emission per filtered entry — would multiply audit-event volume by ~10× per non-admin viewer per dashboard render).

All forks at v2-re-plan; F3 user-locked DIVERGENT. v2 re-plan honours F3.B's tightened scope; no additional gate-1 user-lock required for the four F3.B sub-forks.

---

## Decision

### D56.1 — Typed pure-function `check_auth_request_access` (F2.A consequence)

A pure function ships at `domain/src/auth_requests/access.rs`:

```rust
pub fn check_auth_request_access(
    ar: &AuthRequest,
    principal: &PrincipalRef,
    intended_op: IntendedOp,
) -> Result<(), AuthRequestAccessError>;
```

- No `&self`, no Repository, no async — domain-layer purity per the existing `auth_requests::transitions` precedent (`submit`, `transition_slot`, `reconsider_slot`, `cancel`, `override_approve`, `close_as_denied`).
- Inputs are an immutable `&AuthRequest` snapshot already loaded by the handler, a `&PrincipalRef` from the request context, and an `IntendedOp` capturing the matrix-column op.
- Output is `Ok(())` for matrix-allowed cells; typed `Err(AuthRequestAccessError::*)` for matrix-disallowed cells per §D56.3.
- Body switches on `(ar.state, classify_principal(principal, ar), intended_op)` and matches concept doc 02 lines 130–144 verbatim.

**Rejected alternatives:**
- Method on `AuthRequest` (`fn check_access(&self, principal, op) -> Result<...>`) — couples a domain rule to the data type's surface; violates the `auth_requests::transitions` free-fn precedent.
- Method on `Repository` (gated read/write) — would force the principal-plumbing F3.B.repo-shape.a path that v2 explicitly rejects.

### D56.2 — `IntendedOp` enum captures matrix-column ops (closed-set invariant preserved)

A new `IntendedOp` enum at `domain/src/auth_requests/access.rs`:

```rust
pub enum IntendedOp {
    Read, Modify, Submit, Cancel,
    Approve, Deny, Reconsider, Revoke,
    OverrideApprove, CloseAsDenied, Expire,
}
```

11 variants matching the 11 matrix columns of concept doc 02 lines 130–144. **NOT a member of `Action::CANONICAL`** — the closed 34-verb action vocabulary defined in concept doc 03 §"Standard Action Vocabulary" lines 7–22 is preserved; `Action::CANONICAL.len() == 34` invariant unbroken.

`IntendedOp` is a SEPARATE closed set scoped to AR access semantics. The two closed sets serve different concerns: `Action::CANONICAL` is the manifest-resolution action vocabulary (engine input); `IntendedOp` is the AR per-state matrix-column dimension (predicate input). This separation mirrors CH-15 ADR-0054 §D54.2's re-interpretation precedent — closed-set invariants compose by orthogonal scope, not by embedding.

**Rejected alternatives:**
- Reuse `Action` variants — would force a 1:1 mapping between matrix columns and `Action::CANONICAL` verbs; some matrix ops (Submit, Cancel, OverrideApprove, CloseAsDenied) have no clean Action mapping; would also conflate two scopes.
- 11 boolean flags on a `IntendedOpFlags` struct — verbose at every callsite; not exhaustive against future matrix-column additions.

### D56.3 — `AuthRequestAccessError` typed-error enum (F2.A user-lock)

```rust
pub enum AuthRequestAccessError {
    NotAuthorisedForRead { state, principal_kind },
    NotAuthorisedForModify { state, principal_kind },
    OperationForbiddenInState { state, intended_op },
    RequestorOnlyOperation { ... },
    UnfilledApproverSlotOnly { ... },
}
```

5 variants discriminating the matrix-cell DENY classes. Mirrors `FrozenTagViolation` (CH-12 ADR-0049 §D49.4) structurally — typed enum at the predicate's home module, callers `match`-arm the variants for handler-side error mapping. Each variant carries enough data to (a) format a useful 4xx response body and (b) populate the `auth_request.access_denied` audit event per §D56.6.

**Rejected alternatives:**
- Single `AuthRequestAccessError::Denied(String)` — collapses semantics; harder to test cell-class coverage.
- Re-use `TransitionError` from `auth_requests::transitions` — orthogonal concerns (transition-legality vs principal-authorisation); would conflate them.

### D56.4 — `PrincipalRef` derives `PartialEq, Eq` (F1.B planner-recommended)

`#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` on `pub enum PrincipalRef` at `domain/src/model/nodes.rs:797`. Additive trait-derive change; cascade 0 (no method consumes `==` on `PrincipalRef` today; the derive enables `classify_principal` to compare `&PrincipalRef` against `ar.requestor` via `==` rather than via destructure-match). Closes CH-14 retro Row 5 type-derive precedent.

**Rejected alternatives:**
- Manual `impl PartialEq for PrincipalRef` — boilerplate; trait-derive is the idiomatic Rust path for plain ADTs.
- `PartialEq` only (no `Eq`) — `PrincipalRef`'s contained `AgentId` / `OrgId` / `ProjectId` newtypes are all `Eq`; reflexivity holds; deriving `Eq` is correct.

### D56.5 — Wiring depth: F3.B locked at gate-1 (DIVERGENT from planner's F3.A)

**User-locked at gate-1, DIVERGENT from planner-recommended F3.A.** v2 re-plan honours F3.B's tightened scope.

Wiring spans **17 production callsites** plus **7 documented kernel-internal fast-path skips** (per §D56.8):

- **8 wired submit handlers + 1 kernel-skip-equivalent** (per F3.B.create-side.a defence-in-depth wiring; the 9th site (`adopt.rs`) is a kernel-skip per `D-CH18-FOLLOWUP-02`):
  - `server/src/platform/templates/adopt.rs:96` (Template adoption submit) — **KERNEL-SKIP-EQUIVALENT** per `D-CH18-FOLLOWUP-02`: `build_adoption_request:90` sets `requestor: PrincipalRef::Agent(ceo)` while `input.actor` is the platform-admin (distinct agent IDs); 14-line code comment at `adopt.rs:111` documents the structural mismatch + names the follow-up drift; M6+ chunk wires admin-on-behalf-of-CEO authorisation as a typed-principal class
  - `server/src/platform/projects/create.rs:470` (Project Shape B 2-slot AR submit)
  - `server/src/platform/defaults/put.rs:88` (org defaults change submit)
  - `server/src/platform/secrets/add.rs:107` (secret submit)
  - `server/src/platform/mcp_servers/register.rs:92`
  - `server/src/platform/mcp_servers/patch_tenants.rs:98`
  - `server/src/platform/mcp_servers/archive.rs:54`
  - `server/src/platform/model_providers/register.rs:124`
  - `server/src/platform/model_providers/archive.rs:60`
- **5 read-side sites** (per F3.B.list-filter.a silent post-filter):
  - `server/src/platform/orgs/dashboard.rs:273` (`list_active_auth_requests_for_org`)
  - `server/src/platform/orgs/dashboard.rs:293` (`list_adoption_auth_requests_for_org`)
  - `server/src/platform/orgs/show.rs:63` (count-aggregate; cascades `viewer: AgentId` parameter through `show_organization`)
  - `server/src/platform/projects/create.rs:636` (slot-fill read-before-mutate)
  - principal assertion at the slot-fill flow's caller boundary
- **4 mutation handlers** (per F5.B audit-event emission on Err):
  - `server/src/platform/templates/approve.rs:80`
  - `server/src/platform/templates/deny.rs:85`
  - `server/src/platform/templates/revoke.rs:84`
  - `server/src/platform/projects/create.rs:658` (slot-fill mutation in `approve_pending_shape_b`)

The "resource-owner-lookup helper" is **DESCOPED from CH-18**. The matrix cell `Approved × Revoke by resource-owner` is approximated via `ar.requestor == principal` for adoption-AR self-revoke and accepts as stub for general owner-class until a resource-owner-lookup helper lands in M6+ (per `D-CH18-FOLLOWUP-01` cross-reference + the deferred user-fork at gate-1).

**Rejected alternatives:**
- F3.A (mutation-handlers-only forward-defensive ship — 4 callsites) — planner-recommended; user explicitly chose F3.B's broader scope.
- F3.C (Repository methods gain `principal: PrincipalRef` parameter) — see F3.B.repo-shape rejection below.

### D56.6 — Audit-event emission `auth_request.access_denied` (F5.B user-lock)

A new audit-event builder lives at `domain/src/audit/events/m5_2/auth_request_access.rs`:

```
auth_request.access_denied {
  actor*: AgentId,
  target_ar*: AuthRequestId,
  org*: OrgId,
  error_kind*: String,             // AuthRequestAccessError variant tag (snake_case)
  attempted_op*: String,           // IntendedOp variant tag (snake_case)
  state_at_check*: String,         // ar.state name at predicate evaluation
  attempted_at*: DateTime<Utc>,
}
```

Asterisked fields contribute to `canonical_bytes`. Audit class: **`Alerted`** (concept doc 04 invariant 5: "audit trail on every outcome"; deny is alert-worthy per concept doc 07 §"audit_class composition" — failed permission checks default to `Alerted`). Mirrors CH-15 `platform.session.launch_denied` (ADR-0054 §D54.5) and CH-12 `frozen_tag_write_rejected` (ADR-0049 §D49.4) shapes.

**Emission frequency** — per F3.B.list-filter.a (planner-auto-resolved at gate-1):
- Emit at every Err return at the **4 mutation callsites only** (templates/{approve,deny,revoke}.rs + projects/create.rs:658 slot-fill mutation).
- Do NOT emit at the 5 list-side read callsites (silent post-filter — would multiply audit-event volume by ~10× per non-admin viewer per dashboard render; matches the existing `agent.archived_at IS NOT NULL` filter UX).
- Do NOT emit at the 8 submit-side callsites unless they fail (they will not fail under current code paths because `ar.requestor == input.actor` by construction; the check is definitionally redundant defence-in-depth — emission would be unreachable under normal callsites).

**Pre-existing-behaviour preservation** (CH-14 retro Row 10 / chunk-planner v7 pattern):
- The 4 mutation handlers' existing typed-error → HTTP 4xx behaviour is **unchanged**. Each handler preserves its current happy-path emission (`template.approved` / `template.denied` / `template.revoked` / project-creation events) — CH-18 does NOT alter existing audit-event emission for the success path.
- The new `auth_request.access_denied` event is **purely additive** — emitted only on the new Err path. No existing test assertion is broken.
- The 4 handlers gain a new `AccessDenied(AuthRequestAccessError)` variant on their respective `*Error` enums; existing `*Error` consumers' `match` arms continue to compile (additive enum cascade per chunk-planner cascade discipline).

**Hash-chain symmetry** — additive event-type (per CH-12 plan §3.B A7 + CH-15 plan §3.B A7 precedent). `canonical_bytes` excludes `prev_event_hash`; single-writer guarantee preserved.

**Rejected alternatives:**
- F5.A (re-use `permission_check_decision` field on receipt; no new audit event) — there is no AR-touching equivalent of the launch receipt; the deny path returns `Err` before receipt construction.
- Emit per-filtered-list-entry (rejected as F3.B.list-filter.c) — high-frequency audit-event noise at dashboard render rate.

### D56.7 — Repository trait docstring contract (F3.B.repo-shape.b consequence)

`get_auth_request` + `update_auth_request` on the `Repository` trait at `domain/src/repository.rs:792–797` gain a contract-sentence docstring:

> *"Callers in handler-layer surfaces MUST pair their consumption with `domain::auth_requests::access::check_auth_request_access` per ADR-0056 §D56.5 future-callsite contract; system-internal kernel paths (event-bus listeners, AR resolvers, cascade-revoke loops) are explicit fast-path skips per ADR-0056 §D56.8."*

Mirrors CH-12 ADR-0049 §D49.5 + §D49.7 Repository docstring tag-write contract precedent. The docstring is the single canonical pointer for any future M6+ chunk that adds a new AR-touching path; it tells the implementer where to find the predicate AND when the kernel-skip pattern applies.

The Repository itself does **not** invoke the predicate — it stays principal-blind per F3.B.repo-shape.b. Comprehensive coverage is achieved at the handler boundary (the 17 enumerated callsites in §D56.5).

**Rejected alternatives:**
- F3.B.repo-shape.a (Repository methods gain `principal: PrincipalRef`) — parameter cascade through 2 implementations (`InMemoryRepository`, `SurrealRepository`) + ~30 callsites; system-internal listeners + resolvers (5 sites without caller principal) would have to pass synthetic `PrincipalRef::System(...)` — strictly more code, strictly more confusion. K8s A5 axis would flip from neutral to "new blocker introduced", requiring a `CHK8S-D-11` ledger entry; explicitly rejected.

### D56.8 — Kernel-internal fast-path skip-list (NEW at v2 re-plan)

The following **7 callsites do NOT invoke `check_auth_request_access`** because they execute as system-internal paths with no caller principal context:

1. `domain/src/events/listeners.rs:170` — kernel listener composes `audit_class` from AR's tier; runs after `events.publish_*`; no user-facing principal in scope.
2. `server/src/platform/templates/revoke.rs:115` — cascade-revoke loop `get_auth_request(cascaded_ar_id)` (kernel BFS over revoke graph; the parent revoke at `revoke.rs:84` gates the user-facing entry).
3. `server/src/platform/templates/revoke.rs:131` — cascade-revoke `update_auth_request(&next_cascaded)` (same cascade loop body).
4. `server/src/bootstrap/claim.rs:307` — bootstrap AR creation `apply_bootstrap_claim`; `is_bootstrap_ar(&ar)` returns true; the predicate's system-genesis fast-path would trivially pass anyway.
5. `server/src/platform/templates/mod.rs:178` — `find_adoption_ar` helper; consumed by approve/deny/revoke handlers BEFORE the principal context becomes meaningful; the mutation callsite gates AFTER the helper returns (CH-12 precedent — domain helpers stay principal-blind; handlers gate).
6. `server/src/platform/projects/resolvers.rs:55` — Template A AR resolver (kernel event-bus path).
7. `server/src/platform/projects/resolvers.rs:122` — Template C AR resolver (kernel event-bus path).
8. `server/src/platform/projects/resolvers.rs:161` — Template D AR resolver (kernel event-bus path).

(The numbering above lists 8 line-references against 7 logical sites — `revoke.rs:115`/`:131` are the same cascade loop pair.)

**Rationale for the skip-list as an explicit ADR sub-decision** — CH-12 ADR-0049 §D49.5 + §D49.7 set the precedent that kernel-internal helpers stay principal-blind; access-checks live above the kernel boundary. The §D56.7 Repository docstring + the NEW `m5_2/architecture/auth-request-access-acl.md` §6 (shipping at P3) explicitly enumerate this skip-list to prevent future drift: a successor chunk that adds a new kernel listener consulting AR's `audit_class` is told here that it does NOT need to gate; a successor chunk that adds a new user-facing HTTP handler IS told that it DOES need to gate.

### D56.9 — Sub-fork resolutions under F3.B (NEW at v2 re-plan)

All four F3.B sub-forks auto-resolved at v2 re-plan (no additional gate-1 user-lock required):

- **F3.B.role.b** chosen — defer admin/auditor role-discrimination to M6+ via NEW drift `D-CH18-FOLLOWUP-01`. Concept doc 02 line 134 names "Observer (admin/auditor) — read at every state"; that column is partial-honoured at v2: requestor reads are permitted; bootstrap/system-genesis reads are permitted; slot-approvers read their own AR; **all other principals get `Err(NotAuthorisedForRead)`** — including the org's CEO when reading another agent's AR. M6+ chunk wires admin classification via either `Agent.role` (`domain::model::nodes::AgentRole`) lookup or via Permission Check delegation. F3.B.role.c (resolve-via-OrgMembership-role lookup) was rejected because it forces either a Repository-shape change → A5 K8s axis flip, or a heavier full-Agent-load in every read handler — both heavier than the defer-via-followup-drift path.
- **F3.B.repo-shape.b** chosen — Repository trait stays principal-blind; docstring contract enforced per §D56.7. K8s A5 axis stays neutral; no `CHK8S-D-NN` ledger entry filed. F3.B.repo-shape.a was rejected per §D56.7 alternatives.
- **F3.B.create-side.a** chosen — wire `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)` BEFORE every `create_auth_request` call at the 8 submit handlers. Per-callsite cost: +1 line per submit handler. The check is definitionally redundant at the construction site (`ar.requestor == input.actor` by construction) — this is **defence-in-depth** in the spirit of F3.B's user-locked tightening of scope. A future refactor where `ar.requestor` is constructed from a different source than `input.actor` would surface as a typed-error here rather than silently submitting a mis-attributed AR.
- **F3.B.list-filter.a** chosen — silent post-filter at the 5 list-side reads. NO audit-event emission per filtered entry — would multiply audit-event volume by ~10× per non-admin viewer per dashboard render. The matrix cell `Pending × Read by Other Agent → Err` is silently honoured by hiding the AR from the list (matches the existing `agent.archived_at IS NOT NULL` filter UX shape). The 4 mutation callsites DO emit on Err per §D56.6 — those are explicit-action paths where the viewer is asserting an action; silent denial there would lose the audit trail.

### D56.10 — Divergent-fork audit-trail note (NEW at v2 re-plan)

Planner v1 recommended F3.A (mutation-handlers-only forward-defensive ship — 4 callsites). User gate-1 locked F3.B (full Repository + dashboard + resolver + bootstrap wiring — 17 callsites + 7 kernel-skip docs). v2 re-plan honours F3.B's tightened scope. The auto-approval criteria re-evaluated under F3.B (per chunk-planner v9 §11) showed:
- Scope ≤ 1.5× forward-scope target — at boundary (3.0–3.5 days vs 1.5× of 2 days = 3 days); orchestrator-discretion at gate-1 explicitly accepted broader scope.
- Zero phi-core leverage delta — clean (+0).
- No new K8s blocker class — clean under F3.B.repo-shape.b.
- Audit envelope ≤ medium — flipped from small (1 auditor, 4 phases) to medium (2 auditors, 5 phases) per audit-envelope-size skill heuristic (5-content-phases threshold).
- Confidence ≥ 9/10 — ~39/43 ≈ 9.07/10 meets the floor.
- No new migration — clean.

**Result**: under F3.B, direct-approval criteria mostly hold with one boundary case (scope ratio at 1.5× boundary). The orchestrator's gate-1 user-lock to F3.B explicitly accepted this. The implementer at P0 records the gate-1 outcome in this ADR's Forks header.

---

## Pre-existing behaviour preservation

**Pre-CH-18 AR-handler behaviour** (preserved at the success-path level; new fail-path adds new typed-error variant):

- Pre-CH-18 the 4 mutation handlers (`templates/{approve,deny,revoke}.rs` + `projects/create.rs:658`) gate state-machine transitions via `auth_requests::transitions::*` and return `TemplateError::*` / `ProjectError::*` on transition-illegality; principal-of-caller is asserted only by the upstream auth middleware (caller is authenticated).
- Pre-CH-18 dashboard reads (`dashboard.rs:273,293`) return ALL active org ARs to any authenticated viewer.
- Pre-CH-18 the 8 submit handlers construct an AR with `requestor: input.actor` and call `repo.create_auth_request(&ar)` without a separate principal assertion.

**Post-CH-18 behaviour** (additive; existing paths preserved):
- The 4 mutation handlers gain an upstream `check_auth_request_access` call. On Err, they emit `auth_request.access_denied` and return `*Error::AccessDenied(AuthRequestAccessError)`. Happy path unchanged.
- The 5 read-side sites apply silent post-filter per F3.B.list-filter.a; non-admin / non-requestor / non-slot-approver viewers see strictly fewer ARs than pre-CH-18. No audit-event per filtered entry; operator-narrative documented in `m5_2/operations/auth-request-access-acl-operations.md` §5 (shipping at P3).
- The 8 submit handlers gain an upstream defence-in-depth `check_auth_request_access(.., IntendedOp::Submit)` call. Happy path unchanged (the check is definitionally redundant at construction).
- Repository trait gains a docstring contract on `get_auth_request` + `update_auth_request`. No method-signature change; no implementation-side change. Existing callsites unchanged.
- `PrincipalRef` derives `PartialEq, Eq` (additive; cascade 0).

**Existing test impact**: Artifact-C cascade per plan §3 — predicted ≤ 4 existing dashboard happy-path test fixtures may need amendment (use slot-approver / requestor as viewer for AR-bearing assertions). The bulk of dashboard tests assert agent-count / project-count integers and continue to pass because the ceo IS the AR-creator/slot-approver in the typical fixture. Test-count delta: any breaking test gets re-stated to assert post-filter behaviour; +0 net tests but +N test fixture amendments at P2b. Pause-discipline at plan §7 P2b lists this as a pause trigger.

---

## Cross-references

### Originating concept doc + section + line range

- [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" lines 130–144 — the canonical (state × principal-class × allowed-ops) matrix; D56.1 + D56.2 + D56.3 + D56.5 + D56.6 + D56.9 all cite specific cells.
- [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Multi-Approver Dynamics" lines 175–179 — per-slot independence + slot-holder reconsider-window + resource-owner override; D56.5 references for the slot-approver classifier.
- [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"State Machine" lines 70–129 — the 9-state enumeration the matrix is indexed against.
- [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"Standard Action Vocabulary" lines 7–22 — closed 34-verb action vocabulary supporting D56.2's separate `IntendedOp` enum + closed-set invariant rationale.

### Closed drifts

- [`m5_1/drifts/D-new-12.md`](../../m5_1/drifts/D-new-12.md) (MEDIUM, B) — primary; transitions `discovered → remediated` at chunk seal (P4).
- [`m5_1/drifts/D-CH18-FOLLOWUP-01.md`](../../m5_1/drifts/D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md) (MEDIUM, B) — NEW at v2; filed at chunk-open (P0); status `discovered`; M6+ deferral confirmed at chunk seal (P4) per F3.B.role.b.

### Prior ADRs cited as precedent (milestone-prefixed)

- [`m1/decisions/0010-per-slot-aggregation.md`](../../m1/decisions/0010-per-slot-aggregation.md) — original AR per-slot aggregation + state-machine ADR; D56.1 builds the new predicate as a sibling free-fn next to `auth_requests::transitions` (the home for `submit`/`transition_slot`/`reconsider_slot`/`cancel`/`override_approve`/`close_as_denied`).
- [`m5_2/decisions/0049-frozen-session-tag-immutability.md`](0049-frozen-session-tag-immutability.md) §D49.4 / §D49.5 / §D49.7 — typed-violation-enum (D56.3 mirror) + Repository docstring contract precedent (D56.7 mirror); kernel-internal-helper-stays-principal-blind precedent (D56.8 mirror).
- [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](0053-system-genesis-authority-chain-revocation-cascade.md) §D53.2 — `is_bootstrap_ar` two-witness predicate (D56.5 fast-path); §D53.3–§D53.5 — system-internal cascade precedent (D56.8 cascade-revoke skip).
- [`m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md`](0054-session-launch-manifest-and-hard-deny-flip.md) §D54.2 — closed-set Action vocabulary preserved verbatim (D56.2 IntendedOp-as-separate-closed-set rationale); §D54.5 — Alerted-class deny audit-event pattern (D56.6 mirror).

### Forward-scope row

- [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §1 lines 169–175 (CH-18 row) + §5 row 18 + §6 line 426 (`MED, 2d, permissions/02, —, yes`) + §7 Q5 line 478 (close-at-M5 binding decision for MED-severity chunks).

---

## Consequences

(Body fills at P1–P3.)

---

## Audit / verification

(Body fills at P3 with the canonical `cargo test` + greps + CI guard list per plan §12.)
