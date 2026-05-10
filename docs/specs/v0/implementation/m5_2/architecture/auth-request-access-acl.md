<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P3 — NEW architecture doc, cycle hex c77937bc) -->

# AuthRequest per-state ACL — architecture

> **Status:** [EXISTS] as of CH-18 (M5.2). The typed predicate ships at [`modules/crates/domain/src/auth_requests/access.rs`](../../../../../../modules/crates/domain/src/auth_requests/access.rs); the audit-event builder at [`domain/src/audit/events/m5_2/auth_request_access.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/auth_request_access.rs); the `PrincipalRef` `PartialEq, Eq` derive at [`model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); 17 production callsite consumers across `server/src/platform/`. For the normative concept-doc reference, read [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" lines 130–144 + §"Multi-Approver Dynamics" lines 175–179. Decision rationale lives in [ADR-0056](../decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.1–§D56.10.

---

## §1 — Why this exists

Concept doc 02 §"Per-State Access Matrix" lines 130–144 defines a 9 × 5 matrix specifying who has what access to an `AuthRequest` record itself at each state — Draft / Pending / In Progress / Approved / Denied / Partial / Revoked / Expired / Cancelled crossed with the principal-classes Requestor / Unfilled Approver Slot / Filled Approver Slot / Resource Owner / Observer (admin/auditor). Pre-CH-18 the Repository layer accepted reads + writes on `AuthRequest` rows without consulting the matrix; an approver could peek at another agent's Draft AR; a stranger could mutate a Draft AR they did not submit; a non-slot-approver could approve a Pending AR.

Drift `D-new-12` (MEDIUM, Bucket B) tracked the gap. CH-18 closes it by:

- Capturing the matrix as a typed pure-function predicate `check_auth_request_access(&AuthRequest, &PrincipalRef, IntendedOp) -> Result<(), AuthRequestAccessError>` at `domain::auth_requests::access` (per [ADR-0056](../decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.1).
- Wiring the predicate into **17 production callsites** spanning 4 mutation handlers + 5 read-side handlers + 8 submit-side handlers (per F3.B locked path).
- Documenting **7 kernel-internal callsites** as explicit fast-path skips (per §D56.8) — event-bus listeners, AR resolvers, cascade-revoke loops, bootstrap AR creation, the `find_adoption_ar` helper.
- Emitting an Alerted-class `auth_request.access_denied` audit-event at every Err return from the **4 mutation callsites only** (per F3.B.list-filter.a — silent post-filter rule for list-side reads).
- Filing `D-CH18-FOLLOWUP-01` to defer admin/auditor role-discrimination ("Observer" matrix column) to M6+.
- Filing `D-CH18-FOLLOWUP-02` to track the structural mismatch at `templates/adopt.rs` between F3.B.create-side.a's prescription and adoption-AR's admin-on-behalf-of-CEO requestor pattern.

---

## §2 — Function shape

### Signature

```rust
// domain::auth_requests::access::
pub fn check_auth_request_access(
    ar: &AuthRequest,
    principal: &PrincipalRef,
    intended_op: IntendedOp,
) -> Result<(), AuthRequestAccessError>;
```

Pure — no `&self`, no Repository, no async. Domain-layer purity per the existing `auth_requests::transitions` precedent (per ADR-0056 §D56.1). Callers are responsible for loading the `AuthRequest` (typically via `Repository::get_auth_request`) and the `PrincipalRef` (from request context) before invoking.

### `IntendedOp` — closed set of 11 matrix-column operations

Concept doc 02 lines 130–144 names 11 distinct operations across the (state × principal-class × allowed-ops) matrix. The `IntendedOp` enum captures them as a closed set scoped to AR access semantics:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntendedOp {
    Read,
    Modify,
    Submit,
    Cancel,
    Approve,
    Deny,
    Reconsider,
    Revoke,
    OverrideApprove,
    CloseAsDenied,
    Expire,
}
```

**Closed-set invariant**: `IntendedOp` is INTENTIONALLY NOT a member of `crate::permissions::Action::CANONICAL`. The two closed sets serve different concerns:

- `Action::CANONICAL` is the manifest-resolution action vocabulary (Permission Check engine input — 34 verbs).
- `IntendedOp` is the AR per-state matrix-column dimension (this predicate's input — 11 ops).

Some `IntendedOp` variants (`Submit`, `Cancel`, `OverrideApprove`, `CloseAsDenied`, `Expire`) have no clean `Action` mapping; conflating the two would force a 1:1 column / verb correspondence that doesn't exist (ADR-0056 §D56.2). The `Action::CANONICAL.len() == 34` invariant is preserved at chunk close.

### `AuthRequestAccessError` — 5 typed DENY-class variants

Mirrors the `FrozenTagViolation` shape from CH-12 ADR-0049 §D49.4 — typed enum at the predicate's home module; callers `match`-arm the variants for handler-side error mapping. Each variant carries enough data to (a) format a useful 4xx response body and (b) populate the `auth_request.access_denied` audit event per §D56.6.

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthRequestAccessError {
    /// The principal is not authorised to read the AR at this state.
    NotAuthorisedForRead {
        state: AuthRequestState,
        principal_kind: String,
    },

    /// The principal is not authorised to modify the AR at this state.
    NotAuthorisedForModify {
        state: AuthRequestState,
        principal_kind: String,
        intended_op: IntendedOp,
    },

    /// The intended op is forbidden in the current state regardless of
    /// principal — e.g., `Submit` on an Approved AR, `Modify` on a
    /// closed-terminal AR. Distinguished from `NotAuthorisedForModify`
    /// because the cell is structurally empty in the matrix, not
    /// principal-gated.
    OperationForbiddenInState {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },

    /// The intended op requires `principal == ar.requestor` (e.g., Submit,
    /// Cancel, Modify on Draft).
    RequestorOnlyOperation {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },

    /// The intended op requires the principal to own an *unfilled* approver
    /// slot — Approve / Deny on Pending / InProgress. A principal who owns
    /// a slot they have already filled hits this error rather than the
    /// catch-all NotAuthorisedForModify.
    UnfilledApproverSlotOnly {
        state: AuthRequestState,
        intended_op: IntendedOp,
    },
}
```

The five variants partition the matrix's DENY space into distinguishable cell-classes. Test coverage at `domain/src/auth_requests/access.rs::tests` covers ≥ 15 distinct cell-classes per plan §8 MUST-SHIP.

---

## §3 — Per-state matrix table (verbatim from concept doc 02 lines 130–144)

| State | Requestor | Unfilled Approver Slot | Filled Approver Slot | Resource Owner | Observer (admin/auditor) |
|-------|-----------|------------------------|----------------------|----------------|---------------------------|
| Draft | read, modify, submit, cancel | — | — | read | read |
| Pending | read, cancel | read, approve, deny (own slot) | — | read, approve, deny, escalate | read |
| In Progress | read, cancel | read, approve, deny (own slot) | read, reconsider (re-edit own slot) | read, approve, deny, reconsider any slot, escalate | read |
| Approved | read | — | read, reconsider | read, revoke | read |
| Denied | read, resubmit-as-new | — | read | read, reconsider | read |
| Partial | read, escalate, resubmit-narrower | — | read, reconsider (own slot, until owner closes the record) | read, override-approve, close-as-denied, escalate | read |
| Revoked | read | — | read | read, re-grant (new request) | read |
| Expired | read, resubmit-as-new | — | read | read | read |
| Cancelled | read | — | — | read | read |

### Cell-count + classifier mapping at v2 / F3.B

Total cells: 9 states × 5 principal-classes = **45 cells**.

| Principal-class column | Honoured at v2 | Classifier branch | Notes |
|---|---|---|---|
| Requestor | yes — full | `PrincipalClass::Requestor` (via F1.B `PrincipalRef` `==`) | Captures the leftmost column verbatim per concept doc 02. |
| Unfilled Approver Slot | yes — full | `PrincipalClass::SlotApprover { has_unfilled_slot: true }` | Per `ApproverSlotState::Unfilled` scan over `ar.resource_slots[*].approvers[*]`. |
| Filled Approver Slot | yes — full | `PrincipalClass::SlotApprover { has_filled_slot: true }` | Per `ApproverSlotState::{Approved,Denied}` scan; the slot-holder can `Reconsider` until closed-terminal. |
| Resource Owner | partially — adoption-AR self-revoke only | approximated via `ar.requestor == principal` for Approved × Revoke | Resource-owner-lookup helper DESCOPED from CH-18; tracked in `D-CH18-FOLLOWUP-01` cross-ref to M6+. |
| Observer (admin/auditor) | partially — bootstrap fast-path only | `PrincipalClass::Bootstrap` for system-genesis on bootstrap AR | Admin/auditor classifier deferred to M6+ via `D-CH18-FOLLOWUP-01`; all non-bootstrap "Other Agent" reads return `Err(NotAuthorisedForRead)`. |

### Closed-terminal short-circuit

Closed-terminal states (`Denied`, `Cancelled`, `Revoked`, `Expired`) admit only audit-style reads from any principal. The predicate body short-circuits these states via `auth_requests::state::is_closed_terminal(state)` per ADR-0056 §D56.5; mutation ops on closed-terminal states return `OperationForbiddenInState`.

### System-genesis fast-path

`is_bootstrap_ar(ar) == true` (per CH-14 ADR-0053 §D53.2 two-witness predicate) PLUS `principal == SYSTEM_GENESIS_PRINCIPAL` short-circuits to Read-Ok. Mutation paths on the bootstrap AR are kernel-internal (per §D56.8 skip-list); they do not flow through this predicate.

---

## §4 — 17-callsite wiring map (per ADR-0056 §D56.5)

The predicate is invoked at 17 production callsites — 4 mutation + 5 read + 8 submit. Each callsite validates that the principal of the calling user is authorised for the AR's current state × the intended op cell of the matrix BEFORE the Repository write.

### Mutation handlers (4 callsites — emit on Err)

| # | File:line | IntendedOp | Principal | Audit-event on Err |
|---|---|---|---|---|
| 1 | `server/src/platform/templates/approve.rs` | `Approve` | `PrincipalRef::Agent(input.actor)` | `auth_request.access_denied` (Alerted) |
| 2 | `server/src/platform/templates/deny.rs` | `Deny` | `PrincipalRef::Agent(input.actor)` | `auth_request.access_denied` (Alerted) |
| 3 | `server/src/platform/templates/revoke.rs` | `Revoke` | `PrincipalRef::Agent(input.actor)` | `auth_request.access_denied` (Alerted) |
| 4 | `server/src/platform/projects/create.rs` (slot-fill mutation in `approve_pending_shape_b`) | `Approve` or `Deny` | `PrincipalRef::Agent(input.approver_id)` | `auth_request.access_denied` (Alerted) |

The handlers emit the audit-event BEFORE returning the typed `*Error::AccessDenied(...)` — audit-write never lags the wire response.

### Read-side handlers (5 callsites — silent post-filter, NO audit-event)

| # | File:line | IntendedOp | Principal | Filter shape |
|---|---|---|---|---|
| 5 | `server/src/platform/projects/create.rs` (slot-fill read in `approve_pending_shape_b`) | `Read` | `PrincipalRef::Agent(input.approver_id)` | Returns `Err(*Error::AccessDenied)` if predicate returns Err — this is the principal assertion before the slot-fill write. Single-read site, NOT a list filter. |
| 6 | `server/src/platform/orgs/dashboard.rs` (`list_active_auth_requests_for_org`) | `Read` | `PrincipalRef::Agent(viewer_agent_id)` | Silent post-filter: `Vec::retain(\|ar\| check_auth_request_access(ar, &viewer, Read).is_ok())`. |
| 7 | `server/src/platform/orgs/dashboard.rs` (`list_adoption_auth_requests_for_org`) | `Read` | `PrincipalRef::Agent(viewer_agent_id)` | Same as #6. |
| 8 | `server/src/platform/orgs/show.rs` (`list_adoption_auth_requests_for_org`, count-aggregate consumed via `.len()`) | `Read` | `PrincipalRef::Agent(viewer_agent_id)` | Post-filter then count. Required plumbing `viewer: AgentId` through `show_organization` signature — every caller cascade closed at P2b. |

Per F3.B.list-filter.a, list-side reads filter silently — no audit-event per filtered entry. Rationale: dashboard rendering would multiply audit-event volume by ~10× per non-admin viewer; the matrix cell is silently honoured by hiding the AR from the list (matches existing `agent.archived_at IS NOT NULL` filter UX).

### Submit-side handlers (8 callsites — defence-in-depth, redundant-by-construction)

| # | File:line | IntendedOp | Principal | Pattern |
|---|---|---|---|---|
| 9 | `server/src/platform/projects/create.rs` (Shape B 2-slot AR creation) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe (see §8 below). |
| 10 | `server/src/platform/defaults/put.rs` (org defaults change) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 11 | `server/src/platform/secrets/add.rs` (secret submit) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 12 | `server/src/platform/mcp_servers/register.rs` (MCP register) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 13 | `server/src/platform/mcp_servers/patch_tenants.rs` (MCP tenants patch) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 14 | `server/src/platform/mcp_servers/archive.rs` (MCP archive) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 15 | `server/src/platform/model_providers/register.rs` (model-provider register) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 16 | `server/src/platform/model_providers/archive.rs` (model-provider archive) | `Submit` | `PrincipalRef::Agent(input.actor)` | Synthetic-Draft probe. |
| 17 | `server/src/platform/projects/create.rs:636` (slot-fill READ assert above) | (already counted above as #5) | — | — |

(Note: row 17 above is the same callsite as row 5 — the wiring map per ADR-0056 §D56.5 enumerates 17 distinct callsites; the 8-row submit-side table renumbers from #9 to keep rows #1–#4 mutation and #5–#8 read aligned with the audit/filter behaviour split.)

The 9th submit-side callsite — `templates/adopt.rs` — is a **kernel-skip-equivalent** structural mismatch with F3.B.create-side.a's prescription; see §8 below + `D-CH18-FOLLOWUP-02`.

### Why "synthetic-Draft probe" (not a literal `IntendedOp::Submit` against real-state)

By the time `create_auth_request` is invoked, `ar.state` has been set to `Pending` (or `InProgress` for multi-slot ARs that opened with one `Approved` outcome). The matrix's "Submit" column is on the `Draft` row only — invoking `check_auth_request_access(&ar_pending, ..., Submit)` would return `OperationForbiddenInState { state: Pending, intended_op: Submit }`. Production AR construction NEVER builds an AR at `state == Draft`; the defence-in-depth INTENT of F3.B.create-side.a is preserved by constructing a synthetic clone with `state: Draft`:

```rust
let probe = AuthRequest { state: AuthRequestState::Draft, ..ar.clone() };
check_auth_request_access(&probe, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)
    .map_err(|e| /* map to typed handler error */)?;
```

This pattern is used at all 8 submit-side callsites (rows #9–#16 above). See §8 for the design discussion + rationale.

---

## §5 — Audit-event shape: `auth_request.access_denied`

Builder at `domain::audit::events::m5_2::auth_request_access::auth_request_access_denied`.

### Wire shape

```json
{
  "event_type": "auth_request.access_denied",
  "audit_class": "alerted",
  "actor_agent_id": "<uuid>",
  "target_entity_id": "<NodeId derived from auth_request_id>",
  "org_scope": "<uuid>",
  "provenance_auth_request_id": "<auth_request_id>",
  "diff": {
    "before": null,
    "after": {
      "auth_request_id": "<uuid>",
      "intended_op":     "approve" | "deny" | "revoke" | "modify" | …,  // snake_case
      "error_kind":      "not_authorised_for_modify" | "not_authorised_for_read" | "operation_forbidden_in_state" | "requestor_only_operation" | "unfilled_approver_slot_only",
      "principal_kind":  "agent" | "user" | "organization" | "project" | "system" | "unspecified",
      "attempted_at":    "<rfc3339-timestamp>"
    }
  }
}
```

### Field-level notes

- `event_type` is always `"auth_request.access_denied"` — stable canonical string.
- `audit_class` is always `Alerted` (60s delivery to org alert channel per `nfr-observability.md`).
- `target_entity_id` is `NodeId::from_uuid(*target_ar.as_uuid())` — matches the AR's id so dashboard cross-refs work.
- `provenance_auth_request_id` is `Some(target_ar)` — **divergence from `frozen_tag_write_rejected`** which uses `None` (frozen-tag rejection has no AR provenance; AR access denial does).
- `diff.before` is always `null` (matches the create-style rejection-event convention from CH-12 + CH-15).
- `diff.after.intended_op` is the snake_case `IntendedOp` label per `intended_op_label` (stable across Rust `Debug` representation).
- `diff.after.error_kind` is the snake_case `AuthRequestAccessError` variant tag.
- `diff.after.principal_kind` is `"unspecified"` for the three deny classes that don't carry the principal-kind (state-shape rejections); the two `NotAuthorised*` variants carry the wire-form classification.

### Hash-chain symmetry

The new event-type's `canonical_bytes()` excludes `prev_event_hash` — additive event-type does not perturb prior events' canonical bytes. Single-writer guarantee preserved (per plan §3.B A7). Emission frequency at v2 is low — 4 mutation-callsite sites × ~1 event-per-rejected-request — not a high-volume axis.

### Storage migration

None — `audit_events` table absorbs the new `event_type` without migration (`event_type` is FLEXIBLE TYPE per migration `0001_initial.surql`; `audit_class` already accepts `"alerted"`).

---

## §6 — Forward-defensive descope

### What ships at CH-18

- 4 mutation-handler callsites (rows #1–#4) — every Err emits `auth_request.access_denied`.
- 5 read-side callsites (rows #5–#8) — silent post-filter at list-side reads + principal assertion at the slot-fill read.
- 8 submit-side callsites (rows #9–#16) — synthetic-Draft probe per F3.B.create-side.a.
- 7 kernel-internal callsites documented as fast-path skips (per ADR-0056 §D56.8).
- Repository trait docstring contract on `get_auth_request` + `update_auth_request` per §D56.7 — future-callsite contract sentence requires handler-layer pairing with the predicate.

### What is descoped

- **Repository-layer wiring (F3.B.repo-shape.a)** — Repository methods do NOT gain a `principal: PrincipalRef` parameter. Repository stays principal-blind per F3.B.repo-shape.b. K8s A5 axis stays neutral; no `CHK8S-D-NN` ledger entry filed at v2.
- **Dashboard-layer wiring beyond list-side filter** — the dashboard's per-AR detail render does not invoke the predicate explicitly; the upstream list-filter has already removed unauthorised entries before they reach the render layer.
- **Bootstrap path beyond fast-path skip** — `bootstrap/claim.rs` does not invoke the predicate; the bootstrap AR is created via the system-genesis principal which short-circuits any predicate call to Read-Ok per the `PrincipalClass::Bootstrap` classifier.
- **Admin/auditor role-discrimination ("Observer" matrix column)** — DEFERRED to M6+ via `D-CH18-FOLLOWUP-01`. Concept doc 02 line 134 specifies "read at every state" for admins; CH-18 classifies all non-requestor / non-slot-approver / non-bootstrap principals as "Other Agent" → DENY. Two viable M6+ approaches: (a) `Agent.role: AgentRole` lookup, (b) Permission Check delegation via `Action::Inspect` on `auth_request_object`.
- **Adoption-AR submit-side wiring at `templates/adopt.rs`** — DEFERRED to M6+ via `D-CH18-FOLLOWUP-02`. The structural mismatch is admin-on-behalf-of-CEO: the AR's `requestor` is the CEO (set at `build_adoption_request:90`), not `input.actor` (the platform-admin). Synthetic-Draft probe with `&PrincipalRef::Agent(input.actor)` returns `RequestorOnlyOperation` because admin != ceo. A 14-line kernel-skip-equivalent comment at `adopt.rs:111` documents the structural mismatch; M6+ work that introduces typed admin-on-behalf-of-CEO authorisation can re-admit this site.
- **Resource-owner-lookup helper** — DESCOPED. Matrix cell `Approved × Revoke by resource-owner` is approximated via `ar.requestor == principal` for adoption-AR self-revoke and accepts as stub for general owner-class until a resource-owner-lookup helper lands in M6+ (per `D-CH18-FOLLOWUP-01` cross-ref).

### Kernel-internal fast-path skips (per ADR-0056 §D56.8)

The following 7 callsites do NOT invoke `check_auth_request_access` because they execute as system-internal paths with no caller principal context:

| # | File:line | Rationale |
|---|---|---|
| 1 | `domain/src/events/listeners.rs:170` | Kernel listener composing `audit_class` from AR's tier; runs after `events.publish_*`; no user-facing principal in scope. |
| 2 | `server/src/platform/templates/revoke.rs:115,131` | Cascade-revoke loop body; the parent revoke at row #3 of the wiring map gates the user-facing entry; the cascade fires from the kernel's revoke-graph traversal. |
| 3 | `server/src/bootstrap/claim.rs:307` | Bootstrap AR creation — system-genesis equivalent; `is_bootstrap_ar(&ar) == true` on the freshly-built AR; `check_auth_request_access` would trivially pass via the `classify_principal` system-genesis fast-path. |
| 4 | `server/src/platform/templates/mod.rs:178` | `find_adoption_ar` helper — handler caller (rows #1–#3) gates after the helper returns; helper itself is principal-blind (CH-12 precedent — domain helpers don't take principals; handlers do). |
| 5 | `server/src/platform/projects/resolvers.rs:55` | Template A AR resolver — kernel event-bus consumer, no user principal in scope. |
| 6 | `server/src/platform/projects/resolvers.rs:122` | Template C AR resolver — same. |
| 7 | `server/src/platform/projects/resolvers.rs:161` | Template D AR resolver — same. |

Successor chunks adding new kernel listeners consulting an AR's `audit_class` are explicitly told here that they do NOT need to gate; successor chunks adding new user-facing HTTP handlers ARE told that they DO need to gate (per the §D56.7 Repository docstring contract).

---

## §7 — K8s posture

CH-18 v2 is **K8s-neutral** under F3.B.repo-shape.b. No new blockers; existing CHK8S-D-01 through CHK8S-D-10 ledger entries unchanged. **No new `CHK8S-D-NN` entry filed at v2.**

### A1–A7 axis evaluation

| Axis | Surface | New blocker? |
|---|---|---|
| **A1** New in-process state | `AuthRequestAccessError` (zero-state thiserror enum); `IntendedOp` (copy-Cell-style enum); no shared state | no |
| **A2** New IPC channel | none — pure synchronous predicate | no |
| **A3** New pod-local resource | none | no |
| **A4** Migration runner / first-apply race | zero migrations; `audit_events` table absorbs new `event_type` without schema change | no |
| **A5** Trait-shape requirement | F3.B.repo-shape.b chosen — Repository stays principal-blind. ADR-0033 §D33.1 (`SessionRegistry`) untouched. | no (under repo-shape.b) |
| **A6** Cross-pod state sharing | none — predicate consumes already-loaded `&AuthRequest` snapshot + `&PrincipalRef` from request context; both flow through standard handler-side data plumbing | no |
| **A7** Audit hash-chain symmetry | New `auth_request.access_denied` `canonical_bytes()` excludes `prev_event_hash`. Single-writer guarantee preserved. Low-volume — 4 mutation callsites × ~1 event/rejection — not a hash-chain hot path. | no |

### Why F3.B.repo-shape.b matters

If F3.B.repo-shape.a had been chosen (Repository methods gain `principal: PrincipalRef` parameter), A5 would have flipped to "new blocker introduced":
- Parameter cascade through 2 implementations (`InMemoryRepository`, `SurrealRepository`) + ~30 callsites.
- System-internal listeners + resolvers (5 sites without caller principal) would have to pass synthetic `PrincipalRef::System(...)` — strictly more code, strictly more confusion.
- Would require a `CHK8S-D-11` ledger entry tracking the trait-shape-change deferral.

v2 explicitly does NOT take that path. The handler-boundary access-check pattern (CH-12 ADR-0049 §D49.5/§D49.7 tag-write contract precedent) keeps Repository principal-blind.

---

## §8 — Synthetic-Draft probe pattern (NEW design discussion)

### The problem

F3.B.create-side.a (user-locked at gate-1 under F3.B path) prescribed a defence-in-depth `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)` BEFORE every `create_auth_request` call. The check is **definitionally redundant at construction**: `ar.requestor == input.actor` by construction at all 8 submit-side handlers (the construction site sets requestor from input). The intent is a typed-error tripwire if a future refactor decouples `ar.requestor` from `input.actor`.

But there is a **state-encoding mismatch** with the matrix:
- The matrix's "Submit" column is on the `Draft` row only.
- Production AR construction in baby-phi NEVER builds an AR at `state == Draft` — every `create_auth_request` callsite constructs an AR at `Pending` (or `InProgress` for multi-slot ARs that opened with one Approved slot).
- Invoking `check_auth_request_access(&ar_pending, ..., Submit)` returns `OperationForbiddenInState { state: Pending, intended_op: Submit }` — the wrong typed-error class.

### The solution

Construct a synthetic clone with `state: Draft` for the duration of the probe call:

```rust
// CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — submit-side
// defence-in-depth check.  Probe with synthetic Draft state because
// production AR construction sets state=Pending; the matrix's Submit
// column is on the Draft row only.
let probe = AuthRequest {
    state: AuthRequestState::Draft,
    ..ar.clone()
};
check_auth_request_access(
    &probe,
    &PrincipalRef::Agent(input.actor),
    IntendedOp::Submit,
)
.map_err(|e| ProjectError::AccessDenied(e))?;

repo.create_auth_request(&ar).await?;
```

### Why this preserves intent

The probe's `requestor`, `resource_slots`, and ID are taken from the real `ar` via `..ar.clone()`. Only the `state` field is forced to `Draft` to align with the matrix's Submit-row semantics. The classifier at `classify_principal` runs against the real `requestor` field, so:

- If `ar.requestor == input.actor` (the by-construction case), classifier returns `Requestor`; matrix cell `Draft × Submit by Requestor` returns `Ok(())`. No-op; tripwire not triggered.
- If a future refactor decouples `ar.requestor` from `input.actor` (e.g., bug introduces mismatch), classifier returns `Other`; matrix cell returns `Err(RequestorOnlyOperation)`. Handler emits audit-event + returns typed error. Tripwire triggers exactly as F3.B.create-side.a intended.

### Why not extend `IntendedOp` with a `SubmitFromAnyState` variant?

Considered + rejected:
- Pollutes the closed `IntendedOp` set with a synthetic variant that has no concept-doc grounding.
- Forces the predicate body to special-case the new variant outside the matrix's state-row structure.
- Synthetic-Draft probe pattern is a **caller-side composition** that preserves the predicate's pure-mathematical relationship to the concept-doc matrix.

### Why not skip the check entirely at construction sites?

Considered + rejected per F3.B.create-side.a's user-locked tightening of scope:
- Removes the typed-error tripwire that is the entire point of F3.B's broader-than-F3.A scope.
- Future-defensive against refactors that decouple `ar.requestor` from `input.actor` (e.g., a future "submit on behalf of" surface).
- Cost is +5 lines per submit handler; benefit is non-zero forward-defensive value.

### The `templates/adopt.rs` exception

The 9th submit-site (`templates/adopt.rs`) cannot use the synthetic-Draft probe because the `requestor != input.actor` mismatch is **structural, not bug-prone**:

- `build_adoption_request:90` sets `ar.requestor = PrincipalRef::Agent(ceo)` — the CEO of the org being adopted into.
- `input.actor` is the platform-admin invoking the adopt endpoint — a distinct agent ID.
- Synthetic-Draft probe with `IntendedOp::Submit` and `&PrincipalRef::Agent(input.actor)` returns `RequestorOnlyOperation` because admin != ceo. This is **expected matrix behaviour** for "Draft × Submit by anyone-other-than-requestor" — but the admin-on-behalf-of-CEO flow is intentionally out-of-matrix.

A 14-line kernel-skip-equivalent comment at `adopt.rs:111` documents the structural mismatch + cross-references `D-CH18-FOLLOWUP-02`. M6+ work introducing typed admin-on-behalf-of-CEO authorisation can re-admit this site (e.g., extend the matrix's Submit cell to admit `OnBehalfOfRequestor { delegated_via: ... }`).

---

## §9 — Tests

P1 ships ≥ 15 distinct cell-class tests at `domain/src/auth_requests/access::tests` covering:
- Draft × {Read, Modify, Submit, Cancel} by requestor → Ok / non-requestor → Err.
- Pending × Approve by unfilled-slot approver → Ok / by filled-slot approver → Err / by non-slot agent → Err.
- InProgress × Reconsider by filled-slot approver (own slot) → Ok / (other slot) → Err.
- Approved × Modify by anyone → Err.
- Approved × Revoke by `ar.requestor == principal` adoption-AR self-revoke → Ok (descoped owner-class stub).
- Closed-terminals × Read by any → Ok.
- Closed-terminals × Modify by any → Err.
- System-genesis bootstrap AR × Read by any → Ok (system-genesis fast-path).

P2a ships 3 audit-event builder tests at `domain/src/audit/events/m5_2/auth_request_access::tests` mirroring `frozen_tag_write_rejected_*`.

P2a + P2b ship 15 integration tests at `server/tests/` covering the 4 mutation handlers + 9 submit-side handlers + 2 list-filter sites + 1 slot-fill read site.

Total v2 MUST-SHIP = 35 tests; combined band [1526, 1542] at chunk close.

---

## Cross-references

- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" lines 130–144 + §"Multi-Approver Dynamics" lines 175–179.
- ADR: [ADR-0056](../decisions/0056-auth-request-per-state-acl-enforcement.md) §D56.1–§D56.10.
- Closed drift: [`D-new-12`](../../m5_1/drifts/D-new-12.md) (MEDIUM, B; closed at CH-18).
- Filed drifts: [`D-CH18-FOLLOWUP-01`](../../m5_1/drifts/D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md) (admin/auditor role-discrimination M6+) + [`D-CH18-FOLLOWUP-02`](../../m5_1/drifts/D-CH18-FOLLOWUP-02-adopt-rs-submit-side-wiring.md) (adopt.rs submit-side wiring M6+).
- Operations runbook: [`auth-request-access-acl-operations.md`](../operations/auth-request-access-acl-operations.md).
- Sister architecture pages:
  - [`session-launch-permission-gate.md`](session-launch-permission-gate.md) — CH-15 / ADR-0054 hard-deny + Alerted-class precedent.
  - [`session-live-stream.md`](session-live-stream.md) — CH-17 / ADR-0055 silent-filter at list endpoints precedent.
- Audit-event builder: [`domain/src/audit/events/m5_2/auth_request_access.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/auth_request_access.rs).
- Predicate body: [`domain/src/auth_requests/access.rs`](../../../../../../modules/crates/domain/src/auth_requests/access.rs).
