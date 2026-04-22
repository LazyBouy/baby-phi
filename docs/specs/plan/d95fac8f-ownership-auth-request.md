# Plan: Add Resource Ownership + Auth Request Model to permissions.md

> **Legend for header annotations:**
> - `[PLAN: unchanged]` — this section of the plan file is unchanged since Phases A–C were authored
> - `[PLAN: new in Phase D]` — this section was added on 2026-04-15 as part of the gap-closure pass
> - `[DOCS: ✅ implemented]` — the plan for this section was faithfully executed in the concept docs
> - `[DOCS: ⚠️ underimplemented]` — executed but does not fully match plan intent; needs revision
> - `[DOCS: ❌ not implemented]` — not present in the docs at all
> - `[DOCS: ⏳ pending]` — Phase D work, not yet executed (plan mode is active)
> - `[DOCS: n/a]` — reference/meta section; nothing to implement

## Context  `[PLAN: unchanged]` `[DOCS: n/a]`

The current [permissions.md](/root/projects/phi/phi/docs/specs/v0/concepts/permissions.md) has a real gap that emerged during review: **provenance is a string, and the authority chain behind every grant is implicit.** A grant like "agent X can read /workspace/project-a/**" currently has no structural account of who owned those files or what gave the grantor authority to issue the grant. Revocation cascades, audit trails, and accountability are all hand-wavy.

The fix is to make ownership and the grant-creation workflow first-class, while preserving everything we've already decided. The model gains:

- **Explicit ownership** (Rust-style): resources are owned through creation, transfer, or shared allocation
- **Auth Request as a composite** (`auth_request_object`): a first-class workflow node that mediates grant creation
- **`allocate` scope value**: the authority to issue sub-grants, expressed through the same Auth Request mechanism (no separate delegation-authority concept needed)
- **Routing table on resources**: owners can delegate approval routing without giving up ownership
- **Traceable authority chain**: every grant points back to an Auth Request, which points to an approver, ownership, and ultimately a human decision at bootstrap

This is a **conceptual deepening, not a replacement**. The two-tier resource ontology, the `tag` fundamental, `#kind:` identity tags, selector vs constraints, Multi-Scope Session Access, Consent Policy, and all existing worked examples stay as they are. Terminology tightens to disambiguate the overloaded use of "permission."

## Decisions Captured (from the discussion)  `[PLAN: unchanged]` `[DOCS: see Impl column below]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Ownership philosophy** | Rust-style: creation, transfer, shared allocation. | ✅ implemented — permissions.md §Resource Ownership, Ownership Philosophy table |
| **Creation rule** | Most-specific scope at creation time wins. If agent has `current_project`, project owns; else `current_organization` owns; else agent owns. Override possible via explicit assignment. | ✅ implemented — §Creation Ownership Rules |
| **Transfer semantics** | Automatic for allocation authority (via Template A grants when leads change). Explicit `TransferRecord` for true ownership changes (rarer, more consequential). | ✅ implemented — §Worked Example step 4 |
| **Co-ownership for joint resources** | Multiple principals hold `scope: [allocate]`. Either can allocate within their allocated authority. Matches Multi-Scope base-org rule. | ⚠️ underimplemented — §Co-Ownership exists but the interaction with Multi-Scope Session Access is not spelled out (Phase D → Edit D4) |
| **Auth Request as a composite** | Yes, first-class: `auth_request_object = data_object + tag + #kind:auth_request + {lifecycle rules}`. | ✅ implemented — added to Composite Classes table + full Auth Request Lifecycle section |
| **Auth Request scope field** | Uses the Standard Action Vocabulary. The new value `allocate` is the umbrella for sub-grant authority. | ⚠️ underimplemented — the field is in the schema, but the framing of `allocate` itself is too narrow (Phase D → Edit D1) |
| **Actions on auth_request itself** | Separate from the `scope` field (which describes what the request asks for). Operations ON the auth_request record use a specific set of actions: `create`, `read`, `inspect`, `modify` (Draft only), `submit`, `cancel`, `approve` (fill a slot with Approved), `deny` (fill a slot with Denied), `reconsider` (unfill one's own slot), `escalate` (raise to higher authority), `revoke` (owner-only, post-approval), `override-approve` (owner-only, Partial state), `close-as-denied` (owner-only, Partial state), `archive`, `inspect_archived`. Full matrix of who can invoke which action at which state is in the per-state access matrix (Edit A2 step 4). | ✅ implemented — §Per-State Access Matrix |
| **`allocate` "at-least" semantics** | `allocate` includes at minimum: issue sub-grants of same scope, allocate-with-delegation, approve Auth Requests, escalate stuck requests, revoke previously approved requests. Specific refinements may be expressed as constraints on the grant. | ⚠️ underimplemented — bullets are correct but the framing leads with "delegation authority" instead of the Rust-style Shared Allocation / ownership-sharing framing the user requested (Phase D → Edit D1) |
| **Instance-identity tags** | Every composite instance carries a self-identity tag of the form `{kind}:{instance_id}` (e.g. `session:s-9831`, `memory:m-4581`, `auth_request:req-4581`) in addition to its `#kind:` type tag. This is what lets a grant or auth request address a **specific** instance via the selector (e.g., `tags contains session:s-9831`). Without this convention, selectors could only address sets of instances by category tag (`project:alpha`), never single instances. | ✅ implemented in permissions.md §Instance Identity Tags; ❌ not mirrored in ontology.md (Phase D → Edit D2 step 4) |
| **Approval routing** | Every ownable resource has an optional routing table. By default, Auth Requests go to the resource owner. Owner may route specific (action, scope) combinations to designated delegates. Routing overrides are auditable. | ✅ implemented — §Routing Table on the Resource |
| **What a "Template" is** | A **Template** is a reusable pattern that auto-issues grants when a specific structural event occurs in the graph (edge creation, membership change, session creation, etc.). Templates are authored once (in permissions.md's "Standard Permission Templates" section or a custom equivalent), adopted at the org level, and then fire automatically. The existing doc defines five Authority Templates (A–E) for session access based on relationship edges (project lead, delegation, org chart, project role, explicit manual). Default Grants are a special case of template (issued at agent creation). | ✅ implemented — preamble in §Standard Permission Templates |
| **Templates as pre-authorizations** | Authority Templates (A–E) and Default Grants are pre-authorized allocations. At template adoption time, one Auth Request covers all future grants the template will issue — the org admin approves the template *pattern*, not each individual fired grant. Every grant the template emits later references the template's adoption Auth Request as its provenance. | ✅ implemented — §Templates Are Pre-Authorized Allocations, with `req-adopt-std-org@acme` worked example |
| **Per-state access on Auth Requests** | Lifecycle rules on the composite. Requestor: read + submit/cancel in Draft. Approvers: edit own slot while unfilled. Owner: universal authority. Detailed matrix in Edit A2. | ✅ implemented — 9 states × 5 roles matrix present |
| **Multi-approver dynamics** | Each approver has edit access to their own slot only. Filled slots can be reconsidered (unfilled) by the slot-holder until full resolution. | ✅ implemented — §Multi-Approver Dynamics, including open/semi-open/closed terminal distinction |
| **Revocation cascades** | Forward-only (matches Consent Revocation rule). Downstream sub-allocations that descended from a revoked grant are also revoked; past reads stand; future reads denied. | ✅ implemented — state machine + Worked E2E Step 9 |
| **Auth Request retention (TTL / archival)** | Terminal-state Auth Requests (Approved, Denied, Revoked, Expired, Cancelled) are retained for audit, not deleted. Retention policy is **configurable per org** with sensible defaults. Archival is a two-tier policy: "active" (queryable in the hot path) vs "archived" (moved to cold storage, still auditable via explicit retrieval). See Edit A2 for defaults. | ✅ implemented — §Retention Policy, restructured with Principle 1 (Never Deleted) + Principle 2 (Two-Tier Storage) per user request |
| **Terminology renames** | Permission (the 5-tuple) → **Grant**. Permission Request → **Auth Request**. `HAS_PERMISSION` edge → **`HOLDS_GRANT`**. `GRANTS_PERMISSION` edge → **`ISSUED_GRANT`**. The doc can still be called `permissions.md` — it describes the permission system — but the specific overloaded nouns are disambiguated. | ✅ implemented — grep-verified zero stale references across all concept files |

## Scope of Change  `[PLAN: unchanged]` `[DOCS: n/a]`

**Primary file (all Phase A and C edits + B1):**
- `/root/projects/phi/phi/docs/specs/v0/concepts/permissions.md`

**Secondary files (terminology propagation, Phase B2):**
- `/root/projects/phi/phi/docs/specs/v0/concepts/agent.md`
- `/root/projects/phi/phi/docs/specs/v0/concepts/organization.md`
- `/root/projects/phi/phi/docs/specs/v0/concepts/project.md`
- `/root/projects/phi/phi/docs/specs/v0/concepts/ontology.md`
- Any other concept file referencing `HAS_PERMISSION` / `GRANTS_PERMISSION` or "permission" in the 5-tuple sense

## Edit Plan (phased)  `[PLAN: unchanged]` `[DOCS: n/a]`

Phases are ordered by dependency. Within a phase, edits can land together.

### Phase A — Foundational new concepts  `[PLAN: unchanged]` `[DOCS: ✅ all four edits implemented; Edit A3 has a framing issue tracked in Phase D]`

#### Edit A1: Add **Resource Ownership** section  `[PLAN: unchanged]` `[DOCS: ✅ implemented]`

**Where:** New top-level section inserted right after "Resource Ontology — Two Tiers" (around current line 160, before "Composite Identity Tags").

**Content:**

- **Why ownership matters** — motivates the section. Current provenance is a string; authority chain is implicit; this section makes it explicit.
- **Rust-style ownership philosophy**:
  - **Creation** — the creator's context establishes the initial owner. Rules: most-specific scope wins (`current_project` > `current_organization` > agent itself). Override available.
  - **Transfer** — one-time, exclusive ownership handoff. Captured as a `TransferRecord` node (who, when, from, to). Old owner loses all authority; new owner inherits.
  - **Shared Allocation** — additive. Allocator retains authority; additional holders gain authority. Allocator can revoke allocation (forward-only).
- **New nodes and edges:** (edge directions matter — read the arrow as "is _edge-name_ <target>")
  - Edge: `Resource ──OWNED_BY──▶ Principal` — the resource is owned by the principal
  - Edge: `Principal ──CREATED──▶ Resource` — the principal created the resource (creation provenance)
  - Node: `TransferRecord { transfer_id, resource_id, from_principal, to_principal, timestamp, requestor, approver }` — the requestor (typically the current owner) specifies the approver, not the system; this keeps system load minimal
  - Edge: `Resource ──TRANSFERRED_VIA──▶ TransferRecord` — history of ownership changes for this resource
  - Edge: `Principal ──ALLOCATED_TO──▶ Principal` (scoped by resource, with an Auth Request as provenance) — records that A has allocated some scope of authority over a specific resource to B
- **Ownership table by resource class**:
  | Resource | Default owner at creation | Transferable? | Allocatable? |
  |----------|---------------------------|---------------|--------------|
  | filesystem_object in `/workspace/{project}/**` | project | yes | yes |
  | filesystem_object in `/home/{agent}/**` | agent | yes | yes |
  | session_object | project (if session has `project:` tag) else agent | no (frozen-at-creation) | yes (via grants) |
  | memory_object | project (if tagged) else agent | no | yes |
  | external_service_object (registered) | org platform admin | yes | yes |
  | network_endpoint (domain) | org (via org-wide network policy) | no | yes |
  | secret/credential | org platform admin (or a specifically assigned custodian) | yes | yes (carefully) |
- **Co-ownership / joint resources** — multiple principals hold `scope: [allocate]`. Consistent with Multi-Scope base-org rule. Each co-owner allocates within their authority; the effective allocation set is the union.
- **Worked example** — a project workspace with lead, deputy (allocated deputy authority), and team member (allocated read-only).

#### Edit A1.5: Extend "Composite Identity Tags" with **Instance Identity** convention  `[PLAN: unchanged]` `[DOCS: ✅ implemented in permissions.md; ❌ not mirrored in ontology.md — addressed by Phase D Edit D2 step 4]`

**Where:** Existing "Composite Identity Tags (`#kind:`)" subsection in permissions.md. Add a new subsection **"Instance Identity Tags"** right after the kind-tag explanation, before "Tool Creator Responsibility for `#kind:` Declarations".

**Why this is needed:** the doc establishes `#kind:{composite_name}` as the identity-namespace tag that distinguishes composite **types** at permission check time. But it never specifies the convention for addressing a **single instance** within a kind. Examples throughout the doc use ad-hoc patterns like `tags contains session:s-9831` without ever pinning down that this is the canonical way to address one specific session by ID. Tool creators and grant authors are left to guess.

This edit makes the convention canonical and consistent with the existing tag namespace approach.

**Content of the new subsection:**

1. **The rule, stated plainly:**
   > Every composite instance carries a **self-identity tag** of the form `{kind}:{instance_id}` in addition to its `#kind:{kind}` type tag. The runtime auto-adds this tag at instance creation time (just like the `#kind:` tag), and it cannot be set or modified by agents or tools.

2. **Examples:**
   | Instance | `#kind:` tag | Self-identity tag |
   |----------|--------------|-------------------|
   | A Session with id `s-9831` | `#kind:session` | `session:s-9831` |
   | A Memory with id `m-4581` | `#kind:memory` | `memory:m-4581` |
   | An Auth Request with id `req-7102` | `#kind:auth_request` | `auth_request:req-7102` |
   | An MCP server registration with id `mcp-github-7` | `#kind:external_service` | `external_service:mcp-github-7` |

3. **How this enables single-instance vs set-of-instances selectors:**

   The same tag-predicate grammar handles both via inclusion or omission of the self-identity tag:

   | Selector pattern | What it addresses |
   |------------------|-------------------|
   | `tags contains session:s-9831` | The single Session with id `s-9831` |
   | `tags contains session:s-9831 OR tags contains session:s-9832` | Two specific Sessions |
   | `tags contains project:alpha AND tags contains #kind:session` | All Sessions belonging to project alpha (any session in the project) |
   | `tags contains agent:claude-coder-7 AND tags contains #kind:memory` | All Memories authored by claude-coder-7 |
   | `tags contains #kind:session` | All Sessions, period (typically only used by System Agents) |

4. **Why a separate self-identity tag instead of just letting the runtime use the entity's `id` field?**
   - **Consistency**: every selector uses the same tag-predicate grammar; no special-cased "id" field syntax
   - **Composability**: instance-identity tags compose naturally with scope tags (`tags contains session:s-9831 AND tags contains #public`)
   - **Indexability**: tag-based indexes work uniformly for all selectors
   - **Audit symmetry**: when a memory is extracted from a session, the resulting memory can carry `derived_from:session:s-9831` as a tag — same grammar, same machinery

5. **Reserved namespaces:** the runtime owns these tag namespaces and rejects manual writes to them at publish/creation time:
   - `#kind:*` — composite type identity
   - `{kind}:*` for any registered composite — instance identity (e.g., `session:*`, `memory:*`, `auth_request:*`)
   - `delegated_from:*` — lineage tag (already used on sessions)
   - `derived_from:*` — derivation tag (e.g., memory extracted from a session)

   All other namespaces (`agent:*`, `project:*`, `org:*`, `task:*`, `role_at_creation:*`, `#public`, `#sensitive`, etc.) follow the lifecycle rules of their respective composites.

6. **Cross-references:**
   - Auth Request examples in this doc use `auth_request:{request_id}` for self-identity
   - Grant examples that target a specific instance use the relevant `{kind}:{id}` tag in their selector
   - The Worked End-to-End Use Case examples will be updated to use this convention consistently

**Impact on existing examples:** YAML examples throughout the doc that already use `session:s-9831` etc. are now made canonical rather than ad-hoc. No example needs to change syntactically — they were already consistent with the convention; the convention just needed to be named.

**Impact on Tool Authoring (also requires updating Edit C4 — Authoring Guide):**

Tool creators don't write grant selectors in their manifests (selectors live on grants, which are written by admins, the system, or Auth Request approvers). But the instance-identity convention does affect tool authoring in three specific ways:

1. **Tools that create composite instances** must NOT declare `[modify]` actions on `tag` for the reserved namespaces (`#kind:*`, `{kind}:*`, `delegated_from:*`, `derived_from:*`). These tags are runtime-assigned. A tool that creates a session declares `[create]` on `session_object`; the runtime assigns `#kind:session` and `session:{new_id}` automatically. Attempting to set them via the tool's API is a publish-time rejection.

2. **Tools that accept a target instance as a parameter** should document this in their manifest under a new optional field `target_kinds:`. For example, a `read_session` tool that takes a `session_id` parameter declares:

   ```yaml
   tool: read_session
   manifest:
     resource: session_object
     actions: [read, inspect]
     constraints:
       tag_predicate: required
     kind: [session]
     target_kinds: [session]   # NEW: this tool resolves a single instance via a {kind}:{id} predicate
   ```

   The `target_kinds` field tells the manifest validator that this tool's `tag_predicate` constraint will be satisfied at runtime by an instance-identity tag (e.g., `tags contains session:s-9831`). The validator can verify the tool's runtime call shape matches this expectation. If absent, the runtime falls back to scope-only selectors.

3. **Tools that create cross-instance references** (e.g., a memory extracted from a session that gets a `derived_from:session:{source_id}` tag) need to declare `read` access on the source kind, plus `create` on the destination kind. The `derived_from:` tag's runtime assignment is automatic, but the read of the source instance (to extract from it) is the tool creator's responsibility to declare in the manifest.

**Updated self-check protocol step (in Edit C4 / 6c.3):**

The existing self-check protocol gets one new step:

> **3a. Identify whether your tool addresses single instances or sets of instances.**
> - Single instance: declare `target_kinds: [...]` listing the composite kinds the tool resolves to a specific instance via the `{kind}:{id}` selector convention.
> - Set of instances: leave `target_kinds` absent; the tool operates on whatever set the caller's grant covers via scope tags.
> - Both: list the kinds in `target_kinds`; the tool may handle either selector shape.

**Updated rejection rules in the publish-time validator (in Edit 6a content):**

The validator gains two new checks:

- **Reject** any manifest that declares `[modify]` on `tag` with a selector matching reserved namespaces (`#kind:*`, `{kind}:*` for any registered kind, `delegated_from:*`, `derived_from:*`).
- **Warn** any manifest that declares `[create]` on a composite without a corresponding `target_kinds:` entry — this usually indicates the tool creates instances of that kind, in which case the runtime needs to know which kind so it can assign the self-identity tag correctly.

These are minor additions to the Authoring Guide and the publish-time validator scope, but they keep the instance-identity convention safe by construction.

#### Edit A2: Add **Auth Request** as a composite + Auth Request Lifecycle section  `[PLAN: unchanged]` `[DOCS: ✅ implemented — schema, state machine, per-state access matrix, routing, multi-approver dynamics, retention, bootstrap, approval UX all present; `allocate` framing inside step 7 is the one ⚠️ underimplemented piece, tracked in Phase D Edit D1]`

**Where:**
- Update the Composite Classes table (in current Resource Ontology — Two Tiers) to add `auth_request_object`
- New top-level section "Auth Request Lifecycle" placed after the Resource Ownership section

**Composite entry (added to the composites table):**

| Class | Explicit fundamentals | Implicit | Notes |
|-------|------------------------|----------|-------|
| `auth_request_object` | `data_object` | `tag` + `#kind:auth_request` + {workflow lifecycle rules} | First-class workflow node that mediates grant creation. See [Auth Request Lifecycle](#auth-request-lifecycle). |

**New top-level section content:**

1. **Purpose & rationale** — one-paragraph opener explaining why this exists (authority chain, audit, revocation cascades, accountability).

2. **Schema (YAML) — per-resource slot model:**

   Each slot corresponds to a **(resource, approver) pair**. Slots are **independent**: the decision on one slot does not affect any other slot. A single resource with multiple co-owners has multiple slots (one per co-owner) that all must approve for that resource. The final grant covers exactly the set of resources for which **all required approvers approved**.

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

     routing_override: null   # see routing table section below
     justification: "Need access to review Q1 session logs for retrospective."
     audit_class: logged
   ```

   **Per-resource state derivation rules:**
   - Resource-level state is computed from its slots:
     - All slots `Approved` → resource is `Approved`
     - All slots `Denied` → resource is `Denied`
     - Any slot `Unfilled` and no slots `Denied` → resource is `In Progress`
     - Any slot `Denied` and any slot `Approved` → resource is `Partial` (mixed co-owner opinions)
     - Any slot `Denied` and rest `Unfilled` → resource is `In Progress` heading toward `Denied`
   - Request-level state is an aggregation of resource-level states (see state machine below).

   **The grant produced by approval covers only the `Approved` resources.** Resources in `Denied` / `Partial` / `Expired` states are excluded from the grant.

3. **State machine diagram** (ASCII) with transitions:

   The request-level state aggregates resource-level states. The key insight is that **slots are atomic and independent** — the grant produced covers exactly the approved-resources subset, nothing more.

   **Resource-level states** (one per resource in the request):
   - `In Progress` — at least one required approver slot is `Unfilled`
   - `Approved` — every required approver approved this resource
   - `Denied` — any required approver denied this resource (co-owner denial is enough)
   - `Expired` — `valid_until` hit before all required slots were filled

   **Request-level state** (aggregated from resource-level states):
   - `Draft` — pre-submission
   - `Pending` — all resources are `In Progress`, no slot filled yet
   - `In Progress` — some resources have fills, some still have unfilled slots
   - `Approved` — every resource reached `Approved` (Grant covers all listed resources)
   - `Denied` — every resource reached `Denied` (no Grant)
   - `Partial` — resources split between `Approved` and `Denied` (Grant covers only the Approved subset)
   - `Expired` — `valid_until` hit; Grant covers whichever resources had reached `Approved` by then (if any)
   - `Revoked` — owner revoked the request after approval; Grant auto-revokes (forward-only)
   - `Cancelled` — requestor cancelled before full resolution; no Grant

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
                                                        │  Denied, none pending       (Grant covers Approved subset)
                                                        │
                                                        ├─ valid_until hits ────────▶ Expired
                                                        │  (some resources still       (Grant covers whichever
                                                        │   In Progress)                resources reached Approved)
                                                        │
                                                        └─ slot-holder reconsiders ─▶ stays In Progress
                                                           their slot                (slot returns to Unfilled)

   Approved / Partial ──owner revokes──▶ Revoked
   Approved / Partial ──valid_until hits──▶ Expired
   Draft ──cancel──▶ Cancelled
   Pending / In Progress ──cancel (by requestor)──▶ Cancelled
   ```

   **Partial-outcome rule (revised — atomic slots):** when the request reaches `Partial`, `Expired`, or `Revoked` with a non-empty set of `Approved` resources, **the Grant covers exactly the approved-resources subset**. This is the "individual slot atomicity" semantic:
   - `resource_slots` are independent; each resource is resolved by its own slot set
   - The Grant is the **union** of approved resources, produced when the request reaches a terminal state
   - Denied or expired resources are simply absent from the Grant
   - The audit record preserves the full per-resource outcome

   This replaces the earlier "unanimous required" assumption. There is no collective-significance requirement across resources — each one stands on its own.

   **Request-level `Partial` vs `Approved`:**
   - `Approved` (all resources approved) is the fully-successful terminal state.
   - `Partial` signals to the requestor that some resources were denied. The request was still useful (some resources granted), but the requestor may want to escalate or re-request the denied subset. The Grant produced is valid immediately; `Partial` is about informing the requestor, not blocking the grant.

   **Why keep resource-level `Partial` even though slots are atomic?** For co-owned resources: if two co-owners disagree (one approves, one denies), the resource ITSELF is unresolved at the slot level. We represent this with resource-level `Partial`, and the request-level state reflects it too. The requestor can escalate to the resource owner (or platform admin) to break the tie, which is captured as an `override-approve` or `close-as-denied` action at the resource level.

4. **Per-state access matrix** (who has what access to the Auth Request record itself at each state):

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
   - `In Progress` replaces the earlier `Partially Approved` interim state — a more accurate name given that interim slots may include both Approved and Denied fills.
   - `Partial` is a new terminal state for unanimous-approval failure with preserved mixed outcomes. From `Partial`, the owner can still exercise override authority (`override-approve` closes the record as `Approved`; `close-as-denied` closes as `Denied`), or leave it in `Partial` as the final audit record.

5. **Routing table on the resource** — explains the optional routing mechanism:
   - Every ownable resource may have a `routing_table` attached:
     ```yaml
     resource:
       resource_id: file:/workspace/project-a/docs/**
       owner: agent:lead-acme-1
       routing_table:
         - match: { scope: [read], kind: filesystem_object }
           route_to: agent:deputy-2
         - match: { scope: [allocate] }
           route_to: agent:lead-acme-1        # explicit: owner handles these
         # Fallback: any request not matching goes to owner
     ```
   - Routing is **optional**; without a routing table, all requests go to the owner.
   - Routing overrides are **auditable** — every request records the route it took.
   - The delegated router has `[approve]` (a refinement of `allocate`) on the resource, but **not** full `[allocate]` authority — they can approve requests on behalf of the owner but cannot further delegate routing themselves without the owner's consent.

6. **Multi-approver dynamics** — explains why each approver has edit access to their own slot only, and the reconsideration flow:
   - While a slot is `Unfilled`, only that approver can modify it (to fill with Approved or Denied).
   - Once filled, the slot is read-only to other approvers but the slot-holder can **reconsider** (unfill) their own slot until the request reaches a **closed** terminal state (see below).
   - The resource owner can override any slot (fill or reconsider) — this is universal owner authority.

   **Open vs closed terminal states:**
   - `Approved`, `Denied`, `Expired`, `Revoked`, `Cancelled` are **closed** — slots are locked; no more edits except revoke/re-grant paths.
   - `Partial` is a **semi-open** terminal state — the record is terminal (no new slot fills can change the unanimous-approval outcome), but slot-holders can still reconsider their own slots, and the owner can override. This lets a slow denying approver change their mind even after the record reached `Partial`. Once the owner explicitly closes the `Partial` record (via `override-approve` or `close-as-denied`), slots lock fully.

7. **`allocate` scope semantics** — dedicated subsection explaining that `allocate` is the umbrella value with "at least" the following meanings:
   - Issue sub-grants of the same scope on this resource
   - Allocate-with-delegation (sub-grants that are themselves further delegable)
   - Approve Auth Requests on this resource (routing authority)
   - Escalate stuck requests to higher authority
   - Revoke previously approved requests
   - Specific refinements can be expressed as constraints on the grant (e.g., `allocate: no_further_delegation` as a constraint narrows the umbrella).

8. **Interaction with Authority Templates** — short subsection explaining that templates are **pre-authorized allocations**:
   - When an org adopts Template A, the adoption creates an Auth Request with `scope: [allocate]` granted to the template itself.
   - Every subsequent Template A grant that fires uses this pre-allocated authority; no per-grant request is needed.
   - The grant's provenance chain traces: Grant → Template's Auth Request → Org adoption → human admin decision.

9. **Worked examples (2 examples):**
   - Example 1: Ad-hoc read access. `agent:auditor-9` requests read access to `project:alpha` session logs. Owner `agent:lead-acme-1` approves. Grant is issued.
   - Example 2: Allocation delegation. `agent:lead-acme-1` grants `scope: [allocate]` on `project:alpha/**` files to `agent:deputy-2`. Deputy can now approve team member read requests.

9a. **How Auth Request approval maps to a Grant** — concrete YAML walkthrough showing the before/after data flow:

    **Step 1 — Auth Request reaches `Approved`:**

    ```yaml
    auth_request:
      request_id: req-4581
      requestor: agent:auditor-9
      resources: [session:s-9831, session:s-9832]
      kinds: [session]
      scope: [read, list]
      state: Approved                         # all slots filled Approved
      valid_until: 2026-05-01T00:00:00Z
      submitted_at: 2026-04-11T14:23:00Z
      approver_slots:
        - approver: agent:lead-acme-1
          state: Approved
          responded_at: 2026-04-11T15:00:00Z
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
      provenance: auth_request:req-4581        # REFERENCES the approved request
      approval_mode: auto
      audit_class: logged                      # inherited from auth_request
      expires_at: 2026-05-01T00:00:00Z         # inherited from valid_until
      revocation_scope: tied_to_auth_request   # grant is revoked if auth_request is revoked
    ```

    Key points:
    - The `grant.provenance` field is now a **structural reference** (`auth_request:req-4581`), not a loose string. The authority chain is traversable.
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
      resources: [session:s-9831]              # narrower scope
      kinds: [session]
      scope: [read]                            # also narrower
      state: Approved
      # ... (approval process identical to req-4581) ...
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

    For completeness, here's what happens when an approver reconsiders while the request is still interim:

    ```yaml
    auth_request:
      request_id: req-5012
      state: In Progress
      approver_slots:
        - approver: agent:sponsor-1
          state: Approved
          responded_at: 2026-04-12T10:00:00Z
        - approver: agent:lead-acme-3
          state: Denied                        # filled with deny
          responded_at: 2026-04-12T11:00:00Z
      # Request is currently In Progress, heading toward Partial
    ```

    `agent:lead-acme-3` reconsiders (unfills their slot):

    ```yaml
    auth_request:
      request_id: req-5012
      state: In Progress                       # unchanged — still interim
      approver_slots:
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
    | All slots Approved | Approved | **Grant issued** (state: Active) |
    | All slots Denied | Denied | (no Grant) |
    | Mixed outcomes | Partial | (no Grant; owner can override) |
    | Owner overrides Partial | → Approved or Denied | Grant issued (if Approved) |
    | Owner revokes approved | Approved → Revoked | Grant auto-revoked |
    | valid_until passes | → Expired | Grant auto-expired |
    | Requestor cancels | → Cancelled | (no Grant) |

    The Grant is always **downstream** of the Auth Request state — the Auth Request is authoritative; the Grant is a derived artifact that mirrors the Auth Request's approval status.

10. **Retention Policy** — terminal-state Auth Requests accumulate and need a clear retention policy. The policy rests on two independent principles:

    ### Principle 1 — Never Deleted

    Auth Requests are **never purged**. They are the audit trail of every authorization decision in the system. Losing them loses accountability — there would be no way to answer "who approved access to X on date Y?" after the fact. This principle holds regardless of storage tier, compliance regime, or how old the record is.

    Compliance regimes that legally require deletion after some period are an explicit exception; orgs may set `delete_after_years` to enable scheduled deletion, but doing so is a deliberate trade-off (auditability for compliance) and the policy itself is auditable.

    ### Principle 2 — Two-Tier Storage (Active vs Archived)

    Storage tiering is a separate concern from retention. It exists to keep the hot path fast without losing records.

    - **Active** — retained in the hot path (queryable in normal graph queries). Terminal Auth Requests are active for a configurable window after entering their terminal state. Default window: **90 days**.
    - **Archived** — after the active window expires, the Auth Request moves to cold storage. Still auditable, but retrieval requires an explicit `inspect_archived` operation with `approval: human_required` to prevent casual browsing of historical authorization records.

    Tiering is purely a **storage optimization**, not a deletion policy. Archived records remain part of the system; they're just slower and gated to access.

    ### Configuration

    ```yaml
    organization:
      auth_request_retention:
        # Two-tier storage configuration
        active_window_days: 90          # default; tighter orgs may use 30, audit-heavy orgs may use 365
        archived_retrieval_approval: human_required   # who can recover archived records

        # Compliance-only deletion (very rare; audited)
        delete_after_years: null        # null = never; some compliance regimes may set this
    ```

    ### What triggers archival

    - Active window expires based on `submitted_at` or `terminal_state_entered_at`
    - Explicit archive action by an admin
    - Terminal state + no related active Grants (an Auth Request whose Grant is still live stays active regardless of window)

    ### What affects the active window

    - Grant still live → Auth Request stays active regardless of the window (authoritative record for the Grant)
    - Grant revoked / expired → window countdown starts
    - Denied / Cancelled requests → immediate window countdown

    ### Query semantics

    - Normal permission checks and provenance traversal operate on **active records only**
    - Historical audits explicitly include archived records via `scope: [inspect_archived]` (a refinement of Discovery actions on `auth_request_object`)
    - Archived retrieval is itself an auditable event (every retrieval is logged)

11. **System Bootstrap Template (root of the authority tree):**

    Bootstrap is unified with the template mechanism. At system initialization, the system "adopts" a hardcoded **System Bootstrap Template**. Adoption produces a regular Auth Request in `Approved` state — no special record type, no special state.

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
    - It is a **regular Auth Request** in `Approved` state — the only thing distinguishing it is its provenance (`template:system_bootstrap`) and the fact that its approver is `system:genesis`
    - It is naturally immutable because there is no path by which it could be revoked (no principal exists above `system:genesis` to authorize revocation)
    - Audit traversal terminates cleanly when it reaches this record — the chain stops at the hardcoded template definition
    - The Grant it produces allocates `system:root` to the platform admin (or to whichever principal the platform setup designates), and every downstream authority traces back here

    **Why this is cleaner than a special genesis record:**
    - No special `Bootstrapped` state in the Auth Request state machine
    - No special `non_revocable` flag — immutability follows naturally from the fact that no authority exists upstream to revoke it
    - The pattern "template definition → adoption → fired grants" is uniform across bootstrap and every other template
    - The axiom is a **static template definition + one principal name** (both living in code) rather than a runtime record with special rules
    - Matches the user's intuition that bootstrap "feels like" a template

    See the "Bootstrap as a Template — Unified Design" section below for the design comparison and trade-offs.

12. **Other open questions for Auth Request:**
    - Escalation path when routing fails (approver unavailable, stuck request)
    - Cross-org requests where co-owners are in different orgs with different consent policies

#### Edit A3: Add `allocate` to the Standard Action Vocabulary  `[PLAN: unchanged]` `[DOCS: ⚠️ mechanically implemented — `allocate` is in the Authority row, in the Action×Fundamental matrix, and in all per-resource-class Authority lists; the "`allocate` as Umbrella Action" subsection mirrors Edit A2 step 7 but leads with "delegation authority" instead of Shared-Allocation / ownership-sharing framing. Phase D Edit D1 fixes both occurrences.]`

**Where:** Update existing "Standard Action Vocabulary" section (Action × Fundamental Matrix) + all Per-Resource-Class Reference subsections.

**Changes:**
- Add `allocate` to the **Authority** category: `delegate, approve, escalate, allocate`
- Update Action × Fundamental Applicability Matrix: `allocate` applies universally to every fundamental (same as `delegate`, `approve`, `escalate`)
- Update every Per-Resource-Class subsection to list `allocate` among applicable Authority actions
- Add a subsection **"`allocate` as umbrella action"** that restates the at-least semantics from A2 (the Auth Request section owns the detailed semantics; the Action Vocabulary just references it)

### Phase B — Terminology migration  `[PLAN: unchanged]` `[DOCS: ⚠️ terminology rename complete; but the plan's B2 did not task ontology.md with absorbing the new ownership edges — that gap is picked up in Phase D Edit D2]`

#### Edit B1: Rename Permission → Grant in permissions.md  `[PLAN: unchanged]` `[DOCS: ✅ implemented — grep audit shows zero stale references]`

**Where:** Every occurrence in permissions.md.

**Changes:**

- Section "Canonical Shape": update wording — "This 5-tuple is the shape of a **Grant** — a capability HELD by a specific principal."
- Section "Permission as a Graph Node" → rename to "**Grant as a Graph Node**"
- Every YAML example `permission: ...` → `grant: ...`
- Every `HAS_PERMISSION` edge → `HOLDS_GRANT`
- Every `GRANTS_PERMISSION` edge → `ISSUED_GRANT`
- Every prose reference to "the permission" in the 5-tuple sense → "the Grant"
- Keep the word "permission" when referring to the overall system or concept (e.g., "permission check," "permission resolution hierarchy")

**Not renamed:**
- Document filename stays `permissions.md`
- Section headers referring to the system (e.g., "Permissions Model", "Permission Resolution Hierarchy", "Permission Check") stay
- `permission_request` would have been renamed but becomes `auth_request_object` as a composite name per A2

#### Edit B2: Cross-file terminology updates  `[PLAN: unchanged — but scope was narrow]` `[DOCS: ⚠️ terminology rename done across agent.md, ontology.md, project.md, coordination.md, organization.md; structural additions for ownership were out of this edit's scope and remain missing — Phase D Edit D2 closes the gap]`

**Where:**
- agent.md
- organization.md
- project.md
- ontology.md
- Any other concept file referencing `HAS_PERMISSION` / `GRANTS_PERMISSION`

**Changes:**
- `HAS_PERMISSION` → `HOLDS_GRANT` in edge tables
- `GRANTS_PERMISSION` → `ISSUED_GRANT`
- Update any prose that says "Permission" in the 5-tuple sense
- Bump verification header on each touched file

### Phase C — Integration  `[PLAN: unchanged]` `[DOCS: ✅ all 7 edits implemented faithfully]`

#### Edit C1: Update Permission Resolution Hierarchy  `[PLAN: unchanged]` `[DOCS: ✅ implemented — §The Authority Chain, with rooted tree visualization]`

**Where:** Existing "Permission Resolution Hierarchy" section.

**Changes:**
- Add a subsection **"Authority Chain"** after "How the Two Mechanisms Combine":
  - Every grant points to an Auth Request (its provenance)
  - Auth Request points to the approver(s) and resource owner(s)
  - Owner's authority traces back to their own Auth Request (from an allocator) or to bootstrap (system root)
  - This forms a tree rooted at bootstrap
  - Revocation cascades forward through this tree
- Update the pseudocode: in `resolve_grant()`, add a note that the `ResolvedGrant.provenance` now references an Auth Request node, not just a string.

#### Edit C2: Update Standard Permission Templates  `[PLAN: unchanged]` `[DOCS: ✅ implemented — §Templates Are Pre-Authorized Allocations preamble + example]`

**Where:** "Standard Permission Templates" section.

**Changes:**
- Add a preamble: "Templates are **pre-authorized allocations**. At adoption time, an Auth Request with `scope: [allocate]` is created and approved, covering all future grants the template will issue."
- Show one example: Standard Organization Template's adoption as an Auth Request
- Reframe the template YAML to note that each template grant it generates will reference the template's pre-authorization as provenance

#### Edit C3: Update Tool Authority Manifest Examples  `[PLAN: unchanged]` `[DOCS: ✅ implemented — §13 `request_grant` added]`

**Where:** "Tool Authority Manifest Examples" section.

**Changes:**
- Add a new example: `request_grant` — a tool that an agent uses to create Auth Requests. Declares `resource: auth_request_object`, `actions: [create]`, `#kind: auth_request`.
- No changes to existing examples (their fundamentals/kinds are unchanged).

#### Edit C4: Update Authoring Guide  `[PLAN: unchanged]` `[DOCS: ✅ implemented — auth-request row added to ops table; self-check step 3a for `target_kinds`; validator rejection for reserved namespaces + warning for `create`-without-`target_kinds`]`

**Where:** "Authoring a Tool Manifest: A Guide for Tool Creators" section.

**Changes:**
- Add to the Operation → Fundamentals + `#kind:` table: the row "Creates an Auth Request" → fundamentals `data_object + tag`, `#kind: auth_request`.
- Minor note in the self-check protocol that tools creating Auth Requests must declare `#kind: auth_request`.

#### Edit C5: Update the Worked End-to-End Use Case  `[PLAN: unchanged]` `[DOCS: ✅ implemented — Step 0 augmented with ownership setup; new Step 9 walks through ad-hoc Auth Request → partial approval → revocation cascade with authority-chain trace]`

**Where:** "Worked End-to-End Use Case" section.

**Changes:**
- Add to the setup: establish resource ownership for each project and the joint project.
- Add a new step **9. Ad-hoc Auth Request** — walk through a scenario where an auditor outside the project requests access, owner approves, grant is issued, auditor reads, then owner revokes (showing cascade).
- The existing 8 steps remain unchanged; just terminology updates (Permission → Grant, Permission Request → Auth Request) where applicable.

#### Edit C6: Update Open Questions  `[PLAN: unchanged]` `[DOCS: ✅ implemented — bootstrap, root permission, Market, audit_class moved to Resolved; 4 new open questions added for Phase D follow-ups]`

**Where:** Both "Open Questions for Session Permissions" and the global "Open Questions" section.

**Changes:**
- Resolve: "How are permissions bootstrapped?" → answered by Ownership + Auth Request model (system root Auth Request is non-revocable at bootstrap)
- Resolve: "Should there be a 'root' permission that is non-revocable?" → yes; system-root Auth Request
- Resolve: "How do permissions interact with the Market?" → market postings are Auth Requests with specific routing; approval by the poster
- Resolve: "Should audit_class be per-permission or per-action?" → per-grant, inherited from the Auth Request (can be overridden)
- Add new open questions: Auth Request TTL, bulk requests, routing conflict resolution

#### Edit C7: Update verification header on permissions.md  `[PLAN: unchanged]` `[DOCS: ✅ implemented — all 6 touched files (permissions.md, agent.md, ontology.md, project.md, organization.md, coordination.md) bumped to 2026-04-15]`

**Where:** Top of permissions.md.

**Changes:**
- Bump `<!-- Last verified: 2026-04-11 by Claude Code -->` to today's date
- Expand the subtitle note to mention the new ownership + Auth Request model

## Critical Files to Modify  `[PLAN: unchanged]` `[DOCS: n/a — reference list]`

| File | Scope of change |
|------|-----------------|
| `phi/docs/specs/v0/concepts/permissions.md` | Primary — all Phase A, B1, C edits |
| `phi/docs/specs/v0/concepts/agent.md` | B2 terminology migration |
| `phi/docs/specs/v0/concepts/organization.md` | B2 terminology migration |
| `phi/docs/specs/v0/concepts/project.md` | B2 terminology migration |
| `phi/docs/specs/v0/concepts/ontology.md` | B2 terminology migration (edge names in Governance Wiring table) |

Other concept files (human-agent.md, coordination.md, token-economy.md, phi-core-mapping.md) will need a quick scan for `HAS_PERMISSION` / `GRANTS_PERMISSION` references. If none exist, no changes needed.

## What Stays Unchanged  `[PLAN: unchanged]` `[DOCS: n/a — scope guard]`

- Two-tier resource ontology (fundamentals + composites)
- The 9 fundamentals and 5 existing composites (`auth_request_object` becomes the 6th composite)
- Tag fundamental and `#kind:` identity mechanism
- Composite Identity Tags rule (extended in A1.5 to cover instance identity, but the existing rule for type identity is unchanged)
- Publish-time validator + runtime Permission Check split
- Tool Authority Manifest structure and all existing manifest examples
- Selector vs Constraints formal distinction
- Authority Templates A–E (just reframed as pre-authorizations)
- Multi-Scope Session Access rule (four shapes, cascade resolution, base-org tie-breaker)
- Consent Policy section (implicit / one-time / per-session)
- Memory as a Resource Class / Sessions as a Tagged Resource
- Agent taxonomy (System / Intern / Contract) and the rest of agent.md content
- Token economy, Worth/Value/Meaning formulas
- All composite lifecycle rules for memory and session

## Verification  `[PLAN: unchanged]` `[DOCS: ✅ read-through + terminology audit + cross-file grep done on 2026-04-15; confidence 72% pre-Phase-D, target 97% post-Phase-D]`

1. **Read-through pass on permissions.md** — end-to-end, check internal consistency between the new ownership model and every existing section.
2. **Terminology audit (grep)** — confirm no stale `HAS_PERMISSION`, `GRANTS_PERMISSION`, or "Permission" (the 5-tuple) references remain.
3. **Cross-file check** — grep across all concept files for old edge names.
4. **Example walkthroughs** — mentally run:
   - Project created → ownership established at the `project:` level
   - Template A adopted → pre-authorization recorded; Template A grant fires → uses pre-auth
   - Ad-hoc Auth Request → Draft → submit → approver fills slot → Approved → Grant issued
   - Owner revokes → Grant revoked forward, downstream sub-allocations cascade-revoked
   - Expired Auth Request → grant auto-revoked
5. **Update verification header** on all touched files with today's date.
6. **Status tags** — stay as `CONCEPTUAL` across all files (no code has landed).

## Resolved Questions (previously open)  `[PLAN: unchanged]` `[DOCS: ✅ all four resolutions reflected in permissions.md §Open Questions → Resolved subsection]`

- [x] **Bulk Auth Requests** — **Resolved:** each slot represents a single (resource, approver) pair. Slots are independent and atomic; partial approvals produce a Grant covering the approved-resources subset only. See the "per-resource slot model" in Edit A2.
- [x] **Routing conflicts** — **Resolved:** the resource owner always wins. If both the owner and a delegated router have claims on the same (scope, action) routing pattern, routing goes to the owner. Delegated routers are strict subordinates of the owner.
- [x] **Transfer approval requirements** — **Resolved:** the requestor (typically the current owner) specifies the approver directly on the `TransferRecord`. No system-level approver resolution; the requestor takes responsibility for naming who must approve. Keeps system load minimal.
- [x] **Sync vs async approval UX** — **Resolved (primary design):** the approval flow is asynchronous. When an approver fills a slot, the requestor receives a notification through their Channel. The requestor also has direct read access to the auth_request record, so they can check status at any time (the record is its own dashboard). Polling (periodic automated checks) and block-and-wait (synchronous call) are **implementation patterns available to specific tool integrations** but are not the default UX — they'd only be appropriate for tools where the approval is expected to happen in seconds (e.g., a tool that explicitly says "wait up to 30 seconds for human approval, else deny"). The standard case is notify-on-decision + auth_request record read.

## Bootstrap as a Template — Unified Design  `[PLAN: unchanged]` `[DOCS: ✅ implemented — §System Bootstrap Template — Root of the Authority Tree]`

The Ownership + Auth Request model creates a tree of authority. Every Grant points to an Auth Request, every Auth Request is approved by an owner, every owner's ownership traces back somewhere. There has to be a **root** — some initial authority that isn't derived from anything else. The question is: how do we name that root cleanly?

**The unified answer: bootstrap is just the first template adoption.**

Rather than introducing a special "genesis record" with unique state (`Bootstrapped`) and unique revocation rules (non-revocable), we define a **System Bootstrap Template** as a regular template hardcoded in the system. At initialization, the system "adopts" this template — and adoption produces a completely normal Auth Request in `Approved` state. No special record type, no special state, no special revocation rules.

**What is axiomatic:**
- The **System Bootstrap Template definition** is hardcoded in the runtime (like a constant). It specifies that adoption creates an Auth Request granting `scope: [allocate]` on `system:root` to the platform admin, approved by `system:genesis`.
- The **`system:genesis` principal** is hardcoded. It is the one principal whose existence doesn't derive from another Auth Request. It has no session, no profile, no agent behavior — it exists only to approve the bootstrap template's adoption.

Everything else is regular. Every Auth Request in the system (including the bootstrap adoption record) uses the standard state machine. Every Grant (including the bootstrap allocation to the platform admin) is a regular Grant.

### Shape of the bootstrap adoption record  `[PLAN: unchanged]` `[DOCS: ✅ implemented — YAML example in §System Bootstrap Template]`

```yaml
auth_request:
  request_id: bootstrap-adoption      # a regular id; not special
  requestor: system:genesis
  kinds: [every registered composite]
  scope: [allocate]
  state: Approved                      # standard state — nothing special
  resource_slots:
    - resource: system:root
      approvers:
        - approver: system:genesis
          state: Approved
          responded_at: <system init time>
  valid_until: null                    # no expiry
  submitted_at: <system init time>
  provenance: template:system_bootstrap@<init time>   # adoption of the bootstrap template
  audit_class: alerted                 # any inspection is audit-alerted
```

It has no upstream provenance because it *is* the adoption of the System Bootstrap Template — the template definition itself is the axiom, not a prior Auth Request.

### How this compares to the earlier "Option A" (special genesis record)  `[PLAN: unchanged]` `[DOCS: ✅ rationale reflected in §"Why this is cleaner than a special genesis record"]`

| Aspect | Option A (special genesis record) | Bootstrap-as-Template (chosen) |
|---|---|---|
| Special states needed | `Bootstrapped` unique to genesis | None — standard states only |
| Special revocation rules | Non-revocable flag on the record | None — no one has a path to revoke it because the template definition is hardcoded |
| Where the axiom lives | In a specific runtime record | In a static template definition + one principal name |
| Pattern consistency | Genesis is a one-off | Genesis is the first instance of the normal pattern |
| Rehydration after data loss | Recreate the specific genesis record | Re-adopt the hardcoded template |
| Functional difference in grants issued | None | None |

The unified design removes two special cases from the runtime (one state, one revocation rule) at the cost of one extra level of indirection at the axiom (the axiom is now a template definition, not a runtime record). The trade is worth it.

### Implication for the plan  `[PLAN: unchanged]` `[DOCS: n/a — plan-structure note]`

The plan's **Edit A2 step 11** (previously titled "Bootstrap Auth Request (genesis node)") is updated to specify the **System Bootstrap Template** rather than a special genesis record. No separate edit is needed — the unified design fits within Edit A2's existing structure for templates.

## Open Questions (still open after this change)  `[PLAN: unchanged]` `[DOCS: superseded — Phase D's gap list enumerates the real remaining work]`

None that block the main plan. The remaining refinements — if any emerge during implementation — can be addressed in a follow-up pass.

---

# Phase D — Plan-Verification Findings and Gap Closure (added 2026-04-15)  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending approval]`

## Context  `[PLAN: new in Phase D]` `[DOCS: n/a]`

After Phases A–C landed, a review pass produced a confidence score of ~72%. The user flagged that a mid-session compaction may have caused substandard edits, and specifically corrected the `allocate` framing: **"allocate is about a transfer of ownership — with possibilities of multiple principals."** This is the Rust-style Shared Allocation meaning from the plan's own line 21/67/24, and the doc's current framing ("delegation authority") underweights it.

The goal of Phase D is to close the real gaps identified by the review and by the compaction audit, so the concept docs reach ≥97% confidence **without** requiring the formalization or pseudocode work (which can wait).

## Verification Against the Plan — What's Faithful  `[PLAN: new in Phase D]` `[DOCS: n/a — audit output]`

These edits landed correctly and match the plan:

| Plan item | Status | Notes |
|-----------|--------|-------|
| A1 Resource Ownership section | ✅ faithful | Rust-style framing clear; all new edges present; worked example included |
| A1.5 Instance Identity Tags | ✅ faithful | `{kind}:{id}` convention canonical; reserved namespaces listed; tool-authoring rules present |
| A2 Auth Request composite entry | ✅ faithful | Added to Composite Classes table |
| A2 Schema — per-resource slot model | ✅ faithful | Atomic independent slots; per-resource state table correct |
| A2 State machine | ✅ faithful | `In Progress` + `Partial` distinction preserved |
| A2 Per-state access matrix | ✅ faithful | All 9 states × 5 roles populated |
| A2 Routing table | ✅ faithful | Owner-default + optional delegate entries |
| A2 Multi-approver dynamics | ✅ faithful | Reconsideration + closed/semi-open terminal states |
| A2 Interaction with Authority Templates | ✅ faithful | Templates-as-pre-authorization framing present |
| A2 Worked examples (including Grant-from-AuthRequest walkthrough) | ✅ faithful | 5-step before/after flow is in the doc |
| A2 Retention policy | ✅ faithful | Two principles (Never Deleted + Two-Tier Storage) separated as user requested |
| A2 System Bootstrap Template | ✅ faithful | Hardcoded template + `system:genesis` axiomatic principal; no special state |
| A2 Approval UX (async default) | ✅ faithful | Notify + auth_request-as-dashboard; polling/block-and-wait as opt-in per tool |
| A3 `allocate` added to Action Vocabulary + matrices + per-resource refs | ✅ faithful | All 9 Authority-action lists updated |
| B1 Permission → Grant rename in permissions.md | ✅ faithful | Terminology audit shows zero stale `HAS_PERMISSION` / `GRANTS_PERMISSION` / `permission_request` references |
| B2 Cross-file terminology migration | ✅ faithful | agent.md, ontology.md, project.md, coordination.md updated |
| C1 Authority Chain subsection | ✅ faithful | Tree visualization present, rooted at `system:genesis` |
| C2 Templates Are Pre-Authorized Allocations preamble + YAML | ✅ faithful | `req-adopt-std-org@acme` example is in the doc |
| C3 Tool Manifest #13 `request_grant` | ✅ faithful | Added after entry 12 |
| C4 Authoring Guide updates | ✅ faithful | Auth-request row added to ops table; step 3a for `target_kinds`; reserved-namespace rejection + `target_kinds` warning added to validator |
| C5 Worked E2E Step 9 (ad-hoc Auth Request + revocation cascade) | ✅ faithful | Step 0 also got ownership setup note |
| C6 Open questions resolution | ✅ faithful | Bootstrap, root permission, Market, audit_class moved to "Resolved"; 4 new open questions added |
| C7 Verification-header bumps | ✅ faithful | All 6 touched files now dated 2026-04-15 |

## Issues Found (Gaps Blocking ≥97%)  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

### D1 — `allocate` framing is too narrow (USER-FLAGGED)  `[PLAN: new in Phase D]` `[DOCS: ⚠️ underimplemented in permissions.md §`allocate` Scope Semantics (line 423) and §`allocate` as the Umbrella Action (line 964)]`

**Current state** (permissions.md lines 423–433 and 964–974):
> `allocate` is the umbrella scope value for **delegation authority**. It has **at-least** the following meanings: …

**Plan intent** (lines 21, 67, 24): `allocate` is the **Rust-style Shared Allocation mechanism** — it is how ownership authority is shared with another principal. Multiple principals holding `[allocate]` on the same resource is precisely how **co-ownership** is expressed. "Delegation authority" is the mechanical consequence (the holder can now sub-grant), but the *primary* framing is ownership-sharing.

**User correction (verbatim):** "allocate is about a transfer of ownership — with possibilities of multiple principals."

**Why this matters.** The current framing reads as if `allocate` were a separable "governance token" that lives apart from ownership. But the plan unifies the two: holding `[allocate]` *is* holding a share of ownership authority, and the "at-least" bullet list is just the concrete enumeration of what a shareholder can do. Readers and future implementers should see allocate as the **third pillar of the ownership model** (Creation / Transfer / **Shared Allocation**), not as a side mechanism.

### D2 — ontology.md missing new ownership edges (from review, also a plan gap)  `[PLAN: new in Phase D]` `[DOCS: ❌ not implemented — ontology.md has AuthRequest + TransferRecord nodes only; edges missing]`

Edit B2 in the original plan only did terminology propagation. The plan never tasked ontology.md with absorbing the new ownership structure. As a result, ontology.md is missing:

- Edge: `Resource ──OWNED_BY──▶ Principal` (the new generic ownership edge — the existing line 103 `Agent ──OWNED_BY──▶ User` is a different, pre-existing edge)
- Edge: `Principal ──CREATED──▶ Resource`
- Edge: `Resource ──TRANSFERRED_VIA──▶ TransferRecord`
- Edge: `Principal ──ALLOCATED_TO──▶ Principal`
- `TransferRecord` schema (`{ transfer_id, resource_id, from_principal, to_principal, timestamp, requestor, approver }`)
- `AuthRequest` node details: state enum, resource_slots, retention fields, cross-reference to Auth Request Lifecycle
- Instance-identity tag convention (at least a pointer to permissions.md)

**Impact:** a developer reading ontology.md alone cannot build the correct graph model. High load-bearing gap.

### D3 — ontology.md governance edge table out of sync with permissions.md  `[PLAN: new in Phase D]` `[DOCS: ❌ not implemented — Grant↔AuthRequest, AuthRequest↔Approver, AuthRequest↔Resource, AuthRequest↔Template edges missing]`

Current ontology.md governance edges (lines 159–167) cover `ISSUED_GRANT`, `APPLIES_TO`, `HOLDS_GRANT` and Consent edges. Missing: approver / ownership / transfer / allocation edges from D2, plus:

- `Grant ──DESCENDS_FROM──▶ AuthRequest` (provenance edge) — or equivalent naming
- `AuthRequest ──APPROVED_BY──▶ Principal` (per-slot approvals)
- `AuthRequest ──REQUESTS──▶ Resource` (what the request targets)
- `AuthRequest ──EMITTED_BY──▶ Template` (for template-adopted requests)

### D4 — Co-ownership × Multi-Scope Session Access interaction not explicit (from review)  `[PLAN: new in Phase D]` `[DOCS: ⚠️ underimplemented — both sections exist in permissions.md but their interaction (base-org tie-breaker, ceiling intersection, approval dynamics) is not spelled out]`

Co-Ownership (permissions.md §222) and Multi-Scope Session Access (§2477) are both present, but the interaction — **which co-owner's grants are checked when a session carries multiple `org:` tags** — is not spelled out. The plan's "base-org tie-breaker" mentions the connection but doesn't close it.

### D5 — Consent Policy lifecycle under-specified (from review)  `[PLAN: new in Phase D]` `[DOCS: ⚠️ underimplemented — permissions.md §Consent Node exists with policies and revocation; request/acknowledge side is light]`

Consent Node exists (§2757); request/acknowledge/revoke workflow needs one concrete paragraph or a small state diagram. The Consent Revocation section (§2778) handles half of this; the request/acknowledge side is light.

### D6 — Identity node content genuinely open (from review; from agent.md)  `[PLAN: new in Phase D]` `[DOCS: ⚠️ underimplemented — agent.md lines 237–242 explicitly list three candidates as an open question]`

agent.md lines 237–242 list three candidates (embedding / structured fields / NL bio) and asks which. This blocks storage-schema work. Picking a direction (even provisionally) would lift the doc out of "open question" status.

### D7 — ToolDefinition ↔ Tool Authority Manifest relationship unlinked in ontology.md  `[PLAN: new in Phase D]` `[DOCS: ❌ not implemented — ontology.md has ToolDefinition and ToolImplementation but no manifest edge; permissions.md has the manifest concept]`

ontology.md lists `ToolDefinition` (line 45) and `ToolImplementation` (line 46). Permissions.md extensively covers Tool Authority Manifests. No edge connects them. Either add `ToolDefinition ──HAS_MANIFEST──▶ ToolAuthorityManifest` as a new edge, or record the manifest as a property of ToolDefinition with a cross-reference to the Authoring Guide.

### Out-of-scope for Phase D (per user direction)  `[PLAN: new in Phase D]` `[DOCS: ⏸️ deferred]`

- Permission Check runtime reconciliation pseudocode — deferred
- Formal selector-grammar spec — deferred
- Market full specification — already acknowledged as deferred in organization.md

## Proposed Phase D Edits  `[PLAN: new in Phase D]` `[DOCS: ⏳ all pending approval]`

### Edit D0: Archive this plan into the repo before execution  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

Before touching any concept file, archive a verbatim copy of this plan into the repo so the decision trail is version-controlled alongside the specs.

**Files:**
- Create directory: `/root/projects/phi/phi/docs/specs/plan/` (if it doesn't exist)
- Write file: `/root/projects/phi/phi/docs/specs/plan/<random>-ownership-auth-request.md`

**Naming rule:** `<random>-ownership-auth-request.md`, where `<random>` is a short randomly-generated token (e.g., an 8-character hex string, or a short word-pair like `sharded-stearns`). The fixed tail `ownership-auth-request.md` is invariant so future plan archives in the same folder can be sorted by topic.

**Content:** a **verbatim copy** of the current plan file at `/root/.claude/plans/sharded-discovering-stearns.md`. No edits, no redactions, no summarization — the whole plan, legend, tables, status annotations, and Phase D edits included. This is a point-in-time snapshot, not a living document.

**Why archive here rather than only keeping it in `~/.claude/plans/`:**
- `~/.claude/plans/` is Claude-Code-local; a teammate cloning the repo would not see it.
- `phi/docs/specs/plan/` is under the project's version control, so the plan travels with the code and can be referenced from commit messages, PRs, and future specs.
- Putting it under `specs/plan/` (rather than `specs/v0/concepts/`) keeps it out of the "living concept docs" namespace — this is process history, not part of the spec proper.

**No cross-references, no bumps to other files** as part of D0. It is purely a snapshot action.

### Edit D1: Reframe `allocate` scope semantics (USER-PRIORITY)  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** permissions.md

**Where:**
- §`allocate` Scope Semantics (line 423)
- §`allocate` as the Umbrella Action (line 964)
- Resource Ownership → Shared Allocation bullet (line 178) — add the back-reference
- Co-Ownership section (line 222) — make the connection explicit

**Changes:**

1. Rewrite the opening of both `allocate` sections to lead with the ownership framing:

   > `allocate` is the scope value that expresses **Shared Allocation** (the third pillar of the [Resource Ownership](#resource-ownership) model, alongside Creation and Transfer). Holding `[allocate]` on a resource means the holder has been granted a share of ownership authority over that resource. Multiple principals may hold `[allocate]` on the same resource simultaneously — this is precisely how **co-ownership** is expressed in the graph.
   >
   > Because a shareholder of ownership authority can further extend that share, `allocate` **at-least** entails the following mechanical capabilities:
   >
   > - Issue sub-grants of the held scope (or any sub-scope) to other principals
   > - Allocate-with-delegation (sub-grants that are themselves further delegable)
   > - Approve Auth Requests on this resource (fulfil approver slots)
   > - Escalate stuck Auth Requests to higher authority
   > - Revoke previously approved requests (forward-only cascade)
   >
   > These are not separate capabilities layered on top of allocate — they *are* what it means to hold a share of ownership.

2. Keep the "constraints can narrow the umbrella" sentence about `allocate: no_further_delegation`.

3. In the Resource Ownership → Shared Allocation bullet (§Ownership Philosophy), add a forward-reference: "see [allocate Scope Semantics](#allocate-scope-semantics) for the action-vocabulary form of this mechanism."

4. In the Co-Ownership section, rename / clarify the first sentence to: "Co-ownership is the state of having two or more principals holding `[allocate]` on the same resource. It is not a separate concept from Shared Allocation — it is the natural outcome when `allocate` is granted to multiple principals."

5. Remove the phrase "delegation authority" as the primary descriptor. Keep "delegation" only in the narrower contexts where it's accurate (e.g., "allocate-with-delegation" as a specific sub-capability).

### Edit D2: Bring ontology.md up to date with ownership model  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** ontology.md

**Where:**
- Governance Wiring edge table (current lines 159–167)
- Governance node table (current lines 53–65)
- Add a new subsection "Tag Fundamentals" or extend an existing one to note the self-identity convention with a cross-reference

**Changes:**

1. Add these rows to the governance edge table:

   | From | Edge | To | Cardinality | Meaning |
   |------|------|----|-------------|---------|
   | Resource | `OWNED_BY` | Principal | N:1 | Current owner (Principal = Agent \| Project \| Organization \| User) |
   | Principal | `CREATED` | Resource | 1:N | Creation provenance |
   | Resource | `TRANSFERRED_VIA` | TransferRecord | 1:N | Ownership change history |
   | Principal | `ALLOCATED_TO` | Principal | N:N | A has allocated scope of authority over a Resource to B (resource & scope carried on edge properties or via Auth Request provenance) |
   | Grant | `DESCENDS_FROM` | AuthRequest | N:1 | Provenance: the Auth Request that produced this Grant |
   | AuthRequest | `APPROVED_BY` | Principal | N:N | Per-slot approval edges (approver, state, responded_at on the edge) |
   | AuthRequest | `REQUESTS` | Resource | N:N | Resources this request targets |
   | AuthRequest | `EMITTED_BY` | Template | N:1 | Present only for template-adopted requests |

2. Expand the **AuthRequest** node row to include the full property set (request_id, requestor, kinds, scope, state, valid_until, submitted_at, resource_slots, routing_override, justification, audit_class, retention fields) with a cross-reference to permissions.md § Auth Request Lifecycle.

3. Expand the **TransferRecord** node row to list its full schema: `{ transfer_id, resource_id, from_principal, to_principal, timestamp, requestor, approver }`.

4. Add a short note in the Value Objects or a new "Tag Conventions" section that tags include `#kind:*` (type identity) and `{kind}:{instance_id}` (self-identity), with reserved namespaces listed and a cross-reference to permissions.md § Instance Identity Tags.

5. Disambiguate the existing `Agent ──OWNED_BY──▶ User` edge (line 103) — either rename to something distinct (e.g., `CONTROLLED_BY`) OR note explicitly that the generic `Resource ──OWNED_BY──▶ Principal` edge is a separate edge family. Pick one to avoid ambiguity.

6. Bump the verification header to 2026-04-15 (already done).

### Edit D3: Link ToolDefinition to its manifest in ontology.md  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** ontology.md

**Change:** In the Capability edges table, add:

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| ToolDefinition | `HAS_MANIFEST` | ToolAuthorityManifest | 1:1 | The authority manifest declared at publish time |

Add `ToolAuthorityManifest` to the Capability node table with a cross-reference to permissions.md § Tool Authority Manifest (Tool Requirements).

### Edit D4: Spell out Co-ownership × Multi-Scope Session Access interaction  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** permissions.md

**Where:** At the end of the Co-Ownership section OR at the top of Multi-Scope Session Access, add one subsection:

> **When a session carries multiple org: tags (Shape B — joint project):** both co-owners hold `[allocate]` on the session. For approval purposes, either co-owner may fulfil the approver slot for that resource. For ceiling purposes, the session is subject to the **intersection** of both orgs' ceilings (the stricter bound wins). For base-org tagging, the `base_org` field on the requesting agent determines whose org context is recorded on derived memories / downstream grants. This is the same base-org tie-breaker used elsewhere in Multi-Scope Session Access.

### Edit D5: Flesh out Consent lifecycle  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** permissions.md

**Where:** §Consent Node, add one paragraph or a mini state diagram:

> **Consent lifecycle.** A Consent node progresses through: `Requested` (created when a subordinate is asked to consent to an upcoming action) → `Acknowledged` (the subordinate explicitly agreed) OR `Declined` (the subordinate refused) → optionally `Revoked` (the subordinate withdraws a prior Acknowledgement). Under `implicit` policy, consent is auto-`Acknowledged`. Under `one_time`, consent is requested once and persists for the scope of the issuing org. Under `per_session`, a new Consent node is created per Session. Revocation follows the forward-only rule: past actions stand; future actions denied until a fresh Consent is Acknowledged.

### Edit D6: Pick a direction for Identity node content (provisional)  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** agent.md

**Where:** §Identity / Materialization

**Change:** Replace the three-way open question with a provisional direction (the user can still revisit later):

> **Provisional content model (pending validation):** the Identity node carries (a) a short natural-language self-description (≤500 tokens, agent-authored), (b) structured sub-fields for Lived Experience metrics (sessions completed, ratings histogram, skill names) and Witnessed Experience metrics (memories extracted from subordinates, distinct subordinates observed), and (c) an embedding vector derived from (a) for similarity queries. The three views are redundant on purpose: the NL text is for humans, the structured fields are for filters, the embedding is for matching. Future revisions may prune to one canonical form once usage patterns are clear.

### Edit D7: Terminology / verification header bumps  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending]`

**Files:** all Phase D files (permissions.md, ontology.md, agent.md)

**Change:** Bump `Last verified` headers to the Phase D implementation date.

## Critical Files for Phase D  `[PLAN: new in Phase D]` `[DOCS: n/a — reference list]`

| File | Edit(s) |
|------|---------|
| `phi/docs/specs/plan/<random>-ownership-auth-request.md` (NEW) | D0 (create) |
| permissions.md | D1, D4, D5, D7 |
| ontology.md | D2, D3, D7 |
| agent.md | D6, D7 |

## Verification for Phase D  `[PLAN: new in Phase D]` `[DOCS: ⏳ pending execution]`

1. **Re-run the terminology audit** — confirm no "delegation authority" framing survives as the primary descriptor of `allocate`.
2. **Read-through ontology.md** — an implementer should be able to build the graph model without cross-referencing permissions.md for missing edges.
3. **Manual walkthrough of the ownership → allocation → co-ownership → Auth-Request chain** — verify the three framings are unified and cross-linked.
4. **Second confidence pass** — target ≥97%. If gaps remain, they should be confined to the explicitly-deferred items (Permission Check pseudocode, formal selector grammar, Market full spec).

## What Stays Unchanged in Phase D  `[PLAN: new in Phase D]` `[DOCS: n/a — scope guard]`

- All existing worked examples and YAML
- The `allocate` "at-least" bullet list (the mechanics are right; only the framing changes)
- All other Phase A–C edits (they were verified faithful)
- Open questions explicitly deferred by the user
