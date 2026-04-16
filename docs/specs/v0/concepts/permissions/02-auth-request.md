<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->

## Auth Request Lifecycle

> **Status:** `[CONCEPTUAL]`

**Purpose.** An **Auth Request** is the structural workflow record that mediates Grant creation. Every Grant in the system (except those descending directly from the [System Bootstrap Template](#system-bootstrap-template-root-of-the-authority-tree)) traces back to an Auth Request. This section specifies the Auth Request composite: its schema, state machine, per-state access matrix, routing table, approval dynamics, the `allocate` scope semantics, retention policy, and interaction with Authority Templates.

An Auth Request makes the authority chain explicit: who requested what, who approved, when, and under what conditions. It is the foundation for auditability, revocation cascades, and accountability.

### Schema — Per-Resource Slot Model

Each Auth Request is a list of **per-resource slots**. Each slot corresponds to a **(resource, approver)** pair. Slots are **independent and atomic**: the decision on one slot does not affect any other slot. A single resource with multiple co-owners has multiple slots (one per co-owner) that all must approve for that resource. The final Grant covers exactly the set of resources for which **all required approvers approved**.

```yaml
auth_request:
  request_id: req-4581
  requestor: agent:claude-coder-7
  kinds: [session]                                # composites touched
  scope: [read, list]                             # actions requested (Standard Action Vocabulary)
                                                  # or [allocate] for delegation authority
  state: Pending                                  # see state machine below
  valid_until: 2026-05-01T00:00:00Z
  submitted_at: 2026-04-11T14:23:00Z

  # Per-resource slots. Each resource gets one or more slots (one per required approver).
  # Slots are INDEPENDENT — partial approvals produce partial grants scoped to approved resources.
  resource_slots:
    - resource: session:s-9831
      approvers:
        - approver: agent:lead-acme-1
          state: Approved
          responded_at: 2026-04-11T15:00:00Z
      # resource-level state: Approved (all approvers said Approved)

    - resource: session:s-9832
      approvers:
        - approver: agent:lead-acme-1
          state: Approved
          responded_at: 2026-04-11T15:05:00Z
        - approver: agent:co-owner-beta-3    # this session is co-owned; both must approve
          state: Unfilled
          responded_at: null
      # resource-level state: In Progress (one slot unfilled)

  routing_override: null                         # see Routing Table section
  justification: "Need access to review Q1 session logs for retrospective."
  audit_class: logged
```

**Per-resource state derivation rules:**

Resource-level state is computed from its slots:

| Slot configuration | Resource-level state |
|--------------------|----------------------|
| All slots `Approved` | `Approved` |
| All slots `Denied` | `Denied` |
| Any slot `Unfilled`, no slots `Denied` | `In Progress` |
| Any slot `Denied`, any slot `Approved` | `Partial` (mixed co-owner opinions) |
| Any slot `Denied`, rest `Unfilled` | `In Progress` heading toward `Denied` |

The Grant produced by approval covers only the `Approved` resources. Resources in `Denied`, `Partial`, or `Expired` states are excluded from the Grant.

### State Machine

Request-level states aggregate from resource-level states. The key insight is **slots are atomic and independent** — the Grant produced covers exactly the approved-resources subset, nothing more.

**Resource-level states** (one per resource in the request):
- `In Progress` — at least one required approver slot is `Unfilled`
- `Approved` — every required approver approved this resource
- `Denied` — any required approver denied this resource (co-owner denial is enough)
- `Expired` — `valid_until` hit before all required slots were filled
- `Partial` — co-owners disagree (one approves, one denies); requires owner override or escalation to resolve

**Request-level states** (aggregated from resource-level states):

| State | Meaning |
|-------|---------|
| `Draft` | Pre-submission; only the requestor has edit access |
| `Pending` | All resources are `In Progress`, no slot filled yet |
| `In Progress` | Some resources have fills, some still have unfilled slots |
| `Approved` | Every resource reached `Approved` — Grant covers all listed resources |
| `Denied` | Every resource reached `Denied` — no Grant |
| `Partial` | Resources split between `Approved` and `Denied` — Grant covers only the Approved subset |
| `Expired` | `valid_until` hit; Grant covers whichever resources had reached `Approved` by then (if any) |
| `Revoked` | Owner revoked the request after approval; Grant auto-revokes (forward-only) |
| `Cancelled` | Requestor cancelled before full resolution; no Grant |

```
Draft ──submit──▶ Pending ──any slot filled──▶ In Progress
                                                     │
                                                     ├─ every resource Approved ─▶ Approved
                                                     │                             (Grant covers all resources)
                                                     │
                                                     ├─ every resource Denied ───▶ Denied
                                                     │                             (no Grant)
                                                     │
                                                     ├─ some Approved, some ─────▶ Partial
                                                     │  Denied, none pending        (Grant covers Approved subset)
                                                     │
                                                     ├─ valid_until hits ────────▶ Expired
                                                     │  (some resources still       (Grant covers whichever
                                                     │   In Progress)                resources reached Approved)
                                                     │
                                                     └─ slot-holder reconsiders ─▶ stays In Progress
                                                        their slot                 (slot returns to Unfilled)

Approved / Partial ──owner revokes──▶ Revoked
Approved / Partial ──valid_until hits──▶ Expired
Draft ──cancel──▶ Cancelled
Pending / In Progress ──cancel (by requestor)──▶ Cancelled
```

**Partial-outcome rule (atomic slots).** When the request reaches `Partial`, `Expired`, or `Revoked` with a non-empty set of `Approved` resources, **the Grant covers exactly the approved-resources subset**. This is the individual-slot atomicity semantic:
- `resource_slots` are independent; each resource is resolved by its own slot set
- The Grant is the **union** of approved resources, produced when the request reaches a terminal state
- Denied or expired resources are simply absent from the Grant
- The audit record preserves the full per-resource outcome

There is no collective-significance requirement across resources. Each stands on its own.

**Why keep resource-level `Partial`?** For co-owned resources: if two co-owners disagree (one approves, one denies), the resource ITSELF is unresolved at the slot level. We represent this as resource-level `Partial`, and the request-level state reflects it. The requestor can escalate to the resource owner (or platform admin) to break the tie, recorded as an `override-approve` or `close-as-denied` action at the resource level.

### Per-State Access Matrix

Who has what access to the Auth Request record itself at each state:

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

**Row notes:**
- `In Progress` is the single interim state — slots may include any mix of Unfilled, Approved, Denied.
- `Partial` is a terminal state for unanimous-approval failure with preserved mixed outcomes. From `Partial`, the owner can still exercise override authority, or leave it in `Partial` as the final audit record.

**Open vs closed terminal states:**
- `Approved`, `Denied`, `Expired`, `Revoked`, `Cancelled` are **closed** — slots are locked; no more edits except revoke/re-grant paths.
- `Partial` is **semi-open** — the record is terminal (no new slot fills can change the unanimous-approval outcome), but slot-holders can still reconsider their own slots, and the owner can override. This lets a slow denying approver change their mind even after the record reached `Partial`. Once the owner explicitly closes the `Partial` record (via `override-approve` or `close-as-denied`), slots lock fully.

### Routing Table on the Resource

Every ownable resource may have a **routing table** that delegates approval routing for specific (scope, action) patterns to designated approvers. This lets owners delegate routing without giving up ownership.

```yaml
resource:
  resource_id: file:/workspace/project-a/docs/**
  owner: agent:lead-acme-1
  routing_table:
    - match: { scope: [read], kind: filesystem_object }
      route_to: agent:deputy-2
    - match: { scope: [allocate] }
      route_to: agent:lead-acme-1       # explicit: owner handles these
    # Fallback: any request not matching goes to the owner
```

- Routing is **optional**; without a routing table, all requests go to the owner.
- Routing overrides are **auditable** — every request records the route it took (via `routing_override` on the Auth Request).
- The delegated router has `[approve]` (a refinement of `allocate`) on the resource, but **not** full `[allocate]` authority — they can approve requests on behalf of the owner but cannot further delegate routing themselves without the owner's consent.
- **Routing conflicts:** if both the owner and a delegated router have claims on the same (scope, action) pattern, **the owner always wins**.

### Multi-Approver Dynamics

- While a slot is `Unfilled`, only that approver can modify it (to fill with Approved or Denied).
- Once filled, the slot is read-only to other approvers, but the slot-holder can **reconsider** (unfill) their own slot until the request reaches a **closed** terminal state.
- The resource owner can override any slot (fill or reconsider) — this is universal owner authority.

### `allocate` Scope Semantics

`allocate` is the scope value that expresses **Shared Allocation** — the third pillar of the [Resource Ownership](01-resource-ontology.md#resource-ownership) model, alongside Creation and Transfer. Holding `[allocate]` on a resource means the holder has been granted a share of ownership authority over that resource. Multiple principals may hold `[allocate]` on the same resource simultaneously — this is precisely how **co-ownership** is expressed in the graph (see [Co-Ownership](01-resource-ontology.md#co-ownership-shared-resources)).

> **Frame of reference.** `allocate` is not a separable "governance token" that lives apart from ownership. A principal holding `[allocate]` on a resource is, for the scope of that grant, a shareholder of ownership authority over that resource. When the original owner allocates `[allocate]` to a second principal, ownership becomes shared — not handed off (that would be [Transfer](01-resource-ontology.md#ownership-philosophy--rust-style)) and not transitively delegated (the allocator keeps their own share).

Because a shareholder of ownership authority can further extend that share, `allocate` **at-least** entails the following mechanical capabilities on the covered resource:

- Issue sub-grants of the held scope (or any sub-scope) to other principals
- Allocate-with-delegation — issue sub-grants that are themselves further delegable (the recipient becomes another shareholder)
- Approve Auth Requests on this resource (fulfil approver slots — this is the mechanism by which routing authority is exercised)
- Escalate stuck Auth Requests to higher authority
- Revoke previously approved requests (forward-only cascade per the [Revocation Cascades](#auth-request-lifecycle) rule)

These are not separate capabilities layered on top of `allocate` — they *are* what it means to hold a share of ownership.

**What `allocate` alone does NOT grant.** Holding `[allocate]` is ownership-sharing for *delegation purposes*; it does not by itself carry operational scopes like `[read]`, `[write]`, `[execute]`, or `[create]` on the resource. A co-owner who wants to operate on the resource directly must also hold the relevant operational scope (either issued to themselves or inherited through another grant). This keeps "authority to govern the resource" cleanly distinct from "authority to act on the resource." In practice, creators and template-issued leads typically receive both bundled (via Template A and similar), so the separation is usually invisible — but it matters for deputies, auditors, and routing delegates who should be able to approve access without themselves reading the content.

**Narrowing the umbrella.** Specific refinements can be expressed as constraints on the Grant. For example, `allocate: no_further_delegation` as a constraint removes the allocate-with-delegation sub-capability, producing a "one-level" shareholder who can issue operational sub-grants but cannot create further shareholders. See [Standard Action Vocabulary](03-action-vocabulary.md#standard-action-vocabulary) for the action placement and the Authority-category companion actions (`delegate`, `approve`, `escalate`, `transfer`).

**`allocate` vs `transfer`.** Both are ownership operations and both live in the Authority category of the Standard Action Vocabulary, but they differ in cardinality:

| Operation | Cardinality | Effect on sender | Rust analogue |
|-----------|-------------|------------------|---------------|
| `allocate` | **additive** | Retains full share | `Arc::clone(&x)` — multiple owners, reference-counted |
| `transfer` | **exclusive** | Loses all authority on the resource | `let y = x;` — move; `x` is no longer valid |

Use `allocate` when you want to **share** ownership (the sender remains a co-shareholder — this is how [Co-Ownership](01-resource-ontology.md#co-ownership-shared-resources) arises). Use `transfer` when you want to **hand off** ownership entirely (the sender loses authority; the receiver becomes the sole owner for that scope). An Auth Request with `scope: [transfer]`, on approval, rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership — past actions stand in the audit log, but future actions on the resource require new authority.

### Interaction with Authority Templates

Authority Templates (A–E from the [Standard Permission Templates](07-templates-and-tools.md#standard-permission-templates) section) are **pre-authorized allocations** expressed through the Auth Request mechanism itself:

- When an org adopts Template A, the adoption creates an Auth Request with `scope: [allocate]` granted to the template itself.
- Every subsequent Template A grant that fires uses this pre-allocated authority; no per-grant Auth Request is needed.
- The Grant's provenance chain traces: Grant → Template's Auth Request → Org adoption → human admin decision.

This gives a clean **authority chain** traceable from any Grant back to a specific human decision. Templates provide efficiency (pre-approval covers many fired grants); explicit Auth Requests provide granularity (per-grant approval for unusual cases).

### Worked Examples

**Example 1 — Ad-hoc read access.**

`agent:auditor-9` requests read access to two Sessions in `project:alpha`. They submit an Auth Request. The routing table for the project workspace routes the request to `agent:lead-acme-1`. The lead approves both slots. The Auth Request reaches `Approved`. A Grant is issued to `agent:auditor-9` covering both Sessions. The auditor reads. Later, the lead revokes the Auth Request (retrospective access is no longer needed). The Grant auto-revokes; past reads remain in the audit log; future reads denied.

**Example 2 — Allocation delegation.**

`agent:lead-acme-1` submits an Auth Request with `scope: [allocate]` on `project:alpha/**` files, targeting `agent:deputy-2`. The lead is the resource owner, so the request approval is self-resolving (owner can approve their own delegation requests). Upon approval, an `ALLOCATED_TO` edge is recorded: lead → deputy. Now `agent:deputy-2` can approve team-member read requests for `project:alpha/**` files without further involvement from the lead.

### How Auth Request Approval Maps to a Grant

Concrete YAML walkthrough showing the before/after data flow.

**Step 1 — Auth Request reaches `Approved`:**

```yaml
auth_request:
  request_id: req-4581
  requestor: agent:auditor-9
  kinds: [session]
  scope: [read, list]
  state: Approved                         # all slots filled Approved
  valid_until: 2026-05-01T00:00:00Z
  submitted_at: 2026-04-11T14:23:00Z
  resource_slots:
    - resource: session:s-9831
      approvers:
        - approver: agent:lead-acme-1
          state: Approved
          responded_at: 2026-04-11T15:00:00Z
    - resource: session:s-9832
      approvers:
        - approver: agent:lead-acme-1
          state: Approved
          responded_at: 2026-04-11T15:05:00Z
  justification: "Q1 retrospective review"
  audit_class: logged
```

**Step 2 — Grant is automatically issued:**

```yaml
grant:
  grant_id: grant-7101
  # subject = source of HOLDS_GRANT edge → agent:auditor-9
  action: [read, list]
  resource:
    type: session_object
    # Selector uses the instance-identity tag convention from "Composite Identity Tags".
    # session:{id} is the runtime-assigned self-identity tag every session carries.
    selector: "tags intersects {session:s-9831, session:s-9832}"
  constraints: {}
  provenance: auth_request:req-4581        # STRUCTURAL reference to the approved request
  approval_mode: auto
  audit_class: logged                      # inherited from auth_request
  expires_at: 2026-05-01T00:00:00Z         # inherited from valid_until
  revocation_scope: tied_to_auth_request   # grant is revoked if auth_request is revoked
```

Key points:
- The `grant.provenance` field is a **structural reference** (`auth_request:req-4581`), not a loose string. The authority chain is traversable.
- `expires_at` on the Grant mirrors `valid_until` on the Auth Request. When the Auth Request expires, the Grant auto-expires.
- `audit_class` is inherited by default; per-Grant overrides are possible if more/less sensitive than the Auth Request default.
- `revocation_scope: tied_to_auth_request` binds the Grant's lifetime to the Auth Request. If the Auth Request is revoked, the Grant is revoked. This is how cascading revocation is implemented.

**Step 3 — Owner reconsiders and revokes the Auth Request:**

```yaml
auth_request:
  request_id: req-4581
  # ... (fields unchanged) ...
  state: Revoked                           # state transition
  revoked_at: 2026-04-18T09:00:00Z
  revoked_by: agent:lead-acme-1
  revocation_reason: "Project scope change; auditor access no longer needed."
```

The Grant is **automatically revoked** (via the `revocation_scope: tied_to_auth_request` coupling):

```yaml
grant:
  grant_id: grant-7101
  # ... (fields unchanged) ...
  state: Revoked                           # derived from auth_request state
  revoked_at: 2026-04-18T09:00:00Z         # inherited
  revocation_provenance: auth_request:req-4581@revoked
```

Per the forward-only revocation rule: past reads by `auditor-9` remain in audit logs, but no new reads are permitted. The Grant is kept as a record (with `state: Revoked`) rather than deleted — matching the Auth Request retention rule.

**Step 4 — A new, narrower Auth Request comes in:**

`agent:auditor-9` needs access to just `session:s-9831` (not both). They create a new Auth Request. The previous Grant is already revoked; a new Grant is issued from the new Auth Request:

```yaml
auth_request:
  request_id: req-4699                     # new ID; not an update of the old
  requestor: agent:auditor-9
  kinds: [session]
  scope: [read]                            # also narrower
  state: Approved
  resource_slots:
    - resource: session:s-9831
      approvers:
        - approver: agent:lead-acme-1
          state: Approved
  # ... approval process identical to req-4581 ...
  supersedes: null                         # optional pointer to req-4581 for audit clarity

grant:
  grant_id: grant-7255                     # new grant
  # ... fields similar to grant-7101 but narrower ...
  provenance: auth_request:req-4699
  supersedes: grant-7101                   # optional pointer to revoked predecessor
```

Key points:
- Auth Requests are **immutable after submission** in terms of what they authorize. An "update" is always a new Auth Request with an optional `supersedes` pointer for audit readability.
- Grants similarly are immutable after issuance — modifications happen via new Grants that supersede old ones.
- The `supersedes` fields are optional convenience pointers; the authority chain is already traversable via provenance.

**Step 5 — An `In Progress` Auth Request with a slot reconsideration:**

```yaml
auth_request:
  request_id: req-5012
  state: In Progress
  resource_slots:
    - resource: session:s-7701
      approvers:
        - approver: agent:sponsor-1
          state: Approved
          responded_at: 2026-04-12T10:00:00Z
        - approver: agent:lead-acme-3
          state: Denied                        # filled with deny
          responded_at: 2026-04-12T11:00:00Z
  # Resource-level state: Partial (mixed co-owner opinions)
  # Request-level state: In Progress (waiting for any slot-holder to reconsider, or the owner to override)
```

`agent:lead-acme-3` reconsiders (unfills their slot):

```yaml
auth_request:
  request_id: req-5012
  state: In Progress                       # unchanged — still interim
  resource_slots:
    - resource: session:s-7701
      approvers:
        - approver: agent:sponsor-1
          state: Approved
          responded_at: 2026-04-12T10:00:00Z
        - approver: agent:lead-acme-3
          state: Unfilled                      # back to unfilled
          responded_at: null
          reconsidered_at: 2026-04-12T13:00:00Z
```

The Grant does NOT exist yet (request hasn't reached `Approved`). The slot-holder can now re-fill with a different decision.

**Summary of the Grant-vs-AuthRequest data contract:**

| Lifecycle event | Auth Request state | Grant state |
|-----------------|---------------------|-------------|
| Request submitted | Draft → Pending | (no Grant yet) |
| Some slots filled | In Progress | (no Grant yet) |
| All slots Approved | Approved | **Grant issued** (state: Active, covers all resources) |
| All slots Denied | Denied | (no Grant) |
| Mixed outcomes | Partial | Grant issued covering Approved resources only |
| Owner overrides Partial | → Approved or Denied | Grant issued (if Approved) or revoked (if closed as Denied) |
| Owner revokes approved | Approved → Revoked | Grant auto-revoked |
| valid_until passes | → Expired | Grant auto-expired |
| Requestor cancels | → Cancelled | (no Grant) |

The Grant is always **downstream** of the Auth Request state — the Auth Request is authoritative; the Grant is a derived artifact that mirrors the Auth Request's approval status.

### Retention Policy

Terminal-state Auth Requests accumulate and need a clear retention policy. The policy rests on two independent principles:

#### Principle 1 — Never Deleted

Auth Requests are **never purged**. They are the audit trail of every authorization decision in the system. Losing them loses accountability — there would be no way to answer "who approved access to X on date Y?" after the fact. This principle holds regardless of storage tier, compliance regime, or how old the record is.

Compliance regimes that legally require deletion after some period are an explicit exception; orgs may set `delete_after_years` to enable scheduled deletion, but doing so is a deliberate trade-off (auditability for compliance) and the policy itself is auditable.

#### Principle 2 — Two-Tier Storage (Active vs Archived)

Storage tiering is a separate concern from retention. It exists to keep the hot path fast without losing records.

- **Active** — retained in the hot path (queryable in normal graph queries). Terminal Auth Requests are active for a configurable window after entering their terminal state. Default window: **90 days**.
- **Archived** — after the active window expires, the Auth Request moves to cold storage. Still auditable, but retrieval requires an explicit `inspect_archived` operation with `approval: human_required` to prevent casual browsing of historical authorization records.

Tiering is purely a **storage optimization**, not a deletion policy. Archived records remain part of the system; they're just slower and gated to access.

#### Configuration

```yaml
organization:
  auth_request_retention:
    # Two-tier storage configuration
    active_window_days: 90          # default; tighter orgs may use 30, audit-heavy orgs may use 365
    archived_retrieval_approval: human_required   # who can recover archived records

    # Compliance-only deletion (very rare; audited)
    delete_after_years: null        # null = never; some compliance regimes may set this
```

**What triggers archival:**
- Active window expires based on `submitted_at` or `terminal_state_entered_at`
- Explicit archive action by an admin
- Terminal state + no related active Grants (an Auth Request whose Grant is still live stays active regardless of window)

**What affects the active window:**
- Grant still live → Auth Request stays active regardless of the window (authoritative record for the Grant)
- Grant revoked / expired → window countdown starts
- Denied / Cancelled requests → immediate window countdown

**Query semantics:**
- Normal permission checks and provenance traversal operate on **active records only**
- Historical audits explicitly include archived records via `scope: [inspect_archived]` (a refinement of Discovery actions on `auth_request_object`)
- Archived retrieval is itself an auditable event (every retrieval is logged)

### Approval UX — Async with Notification

The approval flow is **asynchronous by default**. When an approver fills a slot, the requestor receives a notification through their Channel. The requestor also has **direct read access to the auth_request record**, so they can check status at any time — the record is its own dashboard.

Polling (periodic automated checks) and block-and-wait (synchronous call) are **implementation patterns available to specific tool integrations** but are not the default UX — they'd only be appropriate for tools where the approval is expected to happen in seconds (e.g., a tool that explicitly says "wait up to 30 seconds for human approval, else deny"). The standard case is **notify-on-decision + auth_request record read**.

### System Bootstrap Template — Root of the Authority Tree

Every Grant and Auth Request in the system traces back through the authority tree. There has to be a **root** — some initial authority that isn't derived from anything else. We define this as a **hardcoded template**, not a special record.

**The hardcoded template definition** (the axiom):

```yaml
template:
  name: system_bootstrap
  is_hardcoded: true                   # cannot be modified at runtime; defined in code
  adoption_approver: system:genesis    # the one axiomatic principal
  emits_auth_request_on_adoption:
    requestor: system:genesis
    kinds: [every registered composite]
    scope: [allocate]
    resource_slots:
      - resource: system:root
        approvers:
          - approver: system:genesis
            state: Approved            # auto-approved at adoption time
  fires_on: system_init                # triggers once, at system initialization
```

**The `system:genesis` principal** (the other axiom):
- A hardcoded principal with no session, no profile, no agent behavior
- Exists only to approve the System Bootstrap Template's adoption
- Cannot be deleted, cannot be impersonated
- Audit records that mention it are alerted as a signal that they touch the root of the authority tree

**Properties of the bootstrap adoption record:**

The record produced by adoption is a **regular Auth Request** in `Approved` state — the only thing distinguishing it is its provenance (`template:system_bootstrap`) and the fact that its approver is `system:genesis`. It uses the standard state machine. It is naturally immutable because no path exists to revoke it (no principal exists above `system:genesis` to authorize revocation). Audit traversal terminates cleanly when it reaches this record — the chain stops at the hardcoded template definition.

**Why this is cleaner than a special genesis record:**
- No special `Bootstrapped` state in the Auth Request state machine
- No special `non_revocable` flag — immutability follows naturally from the fact that no authority exists upstream to revoke it
- The pattern "template definition → adoption → fired grants" is uniform across bootstrap and every other template
- The axiom is a **static template definition + one principal name** (both living in code) rather than a runtime record with special rules

### Escalation When Routing Fails

If the designated router is unavailable (offline, revoked, the named principal no longer exists) or does not respond within the Auth Request's `valid_until` window, the request **automatically escalates** up the resource owner's chain:

1. **Direct owner first.** The principal recorded on the resource's `OWNED_BY` edge receives the request in their approval queue.
2. **Co-owners next.** If the direct owner is also unavailable, the request fans out to any co-owners (principals holding `[allocate]` on the same resource). Any one of them may approve.
3. **Platform admin last.** If neither the owner nor co-owners respond, the request escalates to the platform admin (the principal holding `[allocate]` on `control_plane_object`).

**Audit trail.** Each escalation step emits an audit event `AuthRequestEscalated{ from_router, to_principal, reason, timestamp }`. The request's `resource_slots[].approvers` list is **appended**, not rewritten — the original router's slot stays in place, marked `Unfilled` with a `skipped_due_to_timeout` flag for audit clarity, while the escalation target's slot is added. A request that escalates twice therefore shows three approver slots (original router + owner + platform admin), with two `skipped_due_to_timeout` flags.

**Timeout triggers.** The escalation fires at the first of:
- `valid_until` elapsed with the request still in `Pending` or `In Progress`
- Explicit `skip_router` signal (owner-initiated, e.g. known-offline router)
- Routing rule points to a principal that has been revoked or deleted (static validation at submission time catches this; routing-time recheck catches drift)

**What does not escalate.** Partial approvals (some slots `Approved`, others `Unfilled`) do not auto-escalate the `Approved` slots — they stay `Approved`. Only unfilled slots with timeouts escalate. This keeps the per-resource atomicity rule intact.

### Open Questions for Auth Request

- Cross-org requests where co-owners are in different orgs with different consent policies — partially resolved; see [06 § Co-Ownership × Multi-Scope rule 6](06-multi-scope-consent.md#co-ownership--multi-scope-session-access) for the per-co-owner independent evaluation rule. Open edge cases: trust relationships where one org refuses to recognise another org's consent machinery at all.

---

