<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->

## Resource Ontology — Two Tiers

The resource ontology has **two tiers**:

- **Fundamental classes** are atomic capabilities the system exposes. They cannot be decomposed further. Permission Checks are ultimately resolved against fundamentals.
- **Composite classes** are named combinations of fundamentals. They exist as documentation shortcuts and authoring conveniences. At runtime, every composite is normalized (expanded) to its constituent fundamentals before permission checks run.

### Fundamental Classes (9)

Fundamentals group into three flavors: physical/operational, data access, and identity.

| Class | Flavor | What It Covers | baby-phi Mapping |
|-------|--------|----------------|-------------------|
| `filesystem_object` | Physical/operational | Files, directories, paths on disk | Agent workspace, skill files, env files |
| `process_exec_object` | Physical/operational | Spawning processes, running binaries/scripts | BashTool, script execution |
| `network_endpoint` | Physical/operational | Outbound network traffic to hosts/ports | Provider base_urls, MCP endpoints, webhooks |
| `secret/credential` | Physical/operational | API keys, tokens, env vars holding secrets | ModelConfig.api_key, MCP auth |
| `economic_resource` | Physical/operational | Token budgets, spend, quotas | Token economy (Contract agents) |
| `time/compute_resource` | Physical/operational | CPU, memory, wall clock, concurrency | ExecutionLimits |
| `data_object` | Data access | Generic structured data access (graph nodes, tables, vectors) | Messages, arbitrary tabular data, vector stores |
| `tag` | Data access + composite categorization | Tag-predicate access grammar. Owns `contains`/`intersects`/`subset_of` operators and the `namespace:value` format. **Implicitly included in every composite** to carry the `#kind:` identity tag. Dual role: (1) data-access capability for composites that hold data, (2) structural substrate that makes every composite distinguishable via `#kind:` filters. | Tag predicates on memory, session, and future tagged composites |
| `identity_principal` | Identity | Agents, users, roles — the "who" axis | Agent, HumanAgent, Organization, Role |

### Composite Classes (8)

Every composite implicitly **uses** the `tag` fundamental to carry a `#kind:{composite_name}` identity tag — even composites that do not hold `data_object`. The `tag` fundamental itself is defined in §Fundamental Classes above; composites don't create new tags, they rely on the `tag` fundamental's machinery. The `#kind:` tag is what makes composites distinguishable from each other at permission check time.

| Class | Explicit fundamentals | Implicit (in every composite) | Notes |
|-------|------------------------|-------------------------------|-------|
| `external_service_object` | `network_endpoint` + `secret/credential` | `tag` + `#kind:external_service` | Covers MCP servers, webhooks, email APIs, Slack, etc. No `data_object` — pure operational composite. Replaces the earlier `communication_object`. |
| `model/runtime_object` | `network_endpoint` + `secret/credential` + `economic_resource` | `tag` + `#kind:model_runtime` | LLM endpoints are external services that also consume token budget. No `data_object`. |
| `control_plane_object` | `data_object` + `identity_principal` | `tag` + `#kind:control_plane` | Managing the policy store means mutating data about principals. |
| `memory_object` | `data_object` | `tag` + `#kind:memory` + {memory tag vocabulary + memory lifecycle rules} | Memory-specific lifecycle rules stay on the composite. See [Memory as a Resource Class](05-memory-sessions.md#memory-as-a-resource-class). |
| `session_object` | `data_object` | `tag` + `#kind:session` + {session tag vocabulary + frozen-at-creation + Multi-Scope resolution} | Session-specific lifecycle and resolution rules stay on the composite. See [Sessions as a Tagged Resource](05-memory-sessions.md#sessions-as-a-tagged-resource). |
| `auth_request_object` | `data_object` | `tag` + `#kind:auth_request` + {workflow lifecycle rules, per-state access matrix, routing table} | First-class workflow node that mediates Grant creation. See [Auth Request Lifecycle](02-auth-request.md#auth-request-lifecycle). |
| `inbox_object` | `data_object` | `tag` + `#kind:inbox` + {one-per-Agent lifecycle} | An agent's **received messages** queue. One per Agent, created at Agent creation time and added to the org catalogue atomically. Messages are `AgentMessage` value objects embedded on the inbox. Separate from task queue — messaging is information flow, not control flow. See [Inbox and Outbox (Agent Messaging)](05-memory-sessions.md#inbox-and-outbox-agent-messaging). |
| `outbox_object` | `data_object` | `tag` + `#kind:outbox` + {one-per-Agent lifecycle, append-only from owner} | An agent's **sent messages** log. One per Agent, parallel to inbox. Messages are `AgentMessage` value objects. The owning agent appends (via send); reads are allowed for the owner and for authorised auditors. See [Inbox and Outbox (Agent Messaging)](05-memory-sessions.md#inbox-and-outbox-agent-messaging). |

**Rule:** Every new integration must project its operations into this schema before it can be enabled.

### Resource Class Overlaps (Entity-Level)

Resource classes can overlap at the **entity level**: one underlying entity may belong to multiple fundamentals simultaneously. This is distinct from composite expansion (static, from the ontology).

**Entity overlap examples:**

| Entity | Fundamentals it belongs to | Why |
|--------|----------------------------|-----|
| `/workspace/project-a/README.md` | `filesystem_object` | Standard file, not a composite |
| `/workspace/project-a/.env` | `filesystem_object` + `secret/credential` | File that carries credentials — entity overlap across two fundamentals, no composite involved |
| A memory node in the graph | `data_object` + (implicit) `tag` + `#kind:memory` | Per `memory_object` composition |
| A memory node persisted to disk | `data_object` + `filesystem_object` + (implicit) `tag` + `#kind:memory` | Memory-on-disk overlaps both the composite expansion AND disk storage |
| A session log file | `data_object` + `filesystem_object` + (implicit) `tag` + `#kind:session` | Session persisted to file |
| A GitHub API call | `network_endpoint` + `secret/credential` + (implicit) `tag` + `#kind:external_service` | Per `external_service_object` composition (no `data_object`) |
| Running `cat .env` via bash | `process_exec_object` + `filesystem_object` + `secret/credential` | Process + file + secret, no composite involved |
| An LLM API call | `network_endpoint` + `secret/credential` + `economic_resource` + (implicit) `tag` + `#kind:model_runtime` | Per `model/runtime_object` composition |
| Spawning a sub-agent | `identity_principal` + transitive reach of the sub-agent | Delegation; `identity_principal` is not a composite so no `#kind:` added |

**The rule:** when an entity falls under N fundamentals (whether from composite expansion or from entity-level overlap), the agent must hold permissions covering **ALL N** fundamentals for the operation to be allowed. Set intersection of allowance, not union.

**Two sources of multi-fundamental requirements:**

1. **Composition** (static, from the ontology): a composite expands to multiple fundamentals via the table above.
2. **Entity overlap** (dynamic, from `classify(entity)`): a concrete entity may fall under multiple fundamentals at runtime.

The runtime combines both sources into a required fundamental set:

```
required_fundamentals =
    expand_composites(manifest.resource)
    ∪ expand_composites(manifest.transitive)
    ∪ classify_to_fundamentals(target_entity)
```

The Permission Check must pass for every fundamental in this set.

---

## Resource Ownership

> **Status:** `[CONCEPTUAL]`

**Why ownership matters.** Earlier in this doc, `provenance` on a Grant was described as a string recording "who issued this grant." That string is a documentation annotation — it doesn't structurally anchor the grant to any authority chain. If we ask "why did `admin-bot` have the authority to grant `agent-7` read access to `/workspace/project-a/**`?", the answer today is "because admin-bot is an admin" — which begs the question of what makes admin-bot an admin. There is no structural account of ownership or the authority chain behind each grant.

This section makes ownership explicit and first-class. Every Grant will (as of the [Auth Request Lifecycle](02-auth-request.md#auth-request-lifecycle) section below) carry a structural reference to an Auth Request, which in turn traces back through the approver, the resource owner, and ultimately the bootstrap template. Every grant has a **traceable authority path** rooted at a documented axiom.

### Ownership Philosophy — Rust-Style

baby-phi adopts a Rust-style ownership philosophy for resources, with three distinct modes:

| Mode | Rust parallel | baby-phi parallel |
|------|---------------|-------------------|
| **Creation** | `let x = Value::new()` — creator is the initial owner | When a resource comes into existence, the creator's context determines the initial owner (agent's `current_project`, `current_organization`, or the agent itself for personal resources). |
| **Transfer** | `let y = x;` — move semantics; `x` is no longer valid | Ownership transfers from one principal to another. The old owner's authority on the resource disappears; the new owner inherits. Used for lead handoffs, project-to-project transfer, etc. One-time, exclusive. |
| **Shared Allocation** | `Arc<T>` / `Rc<T>` — multiple owners, tracked | Multiple principals hold `[allocate]` on the resource. The allocator retains their own share; the additional holder becomes a co-shareholder of ownership authority. The allocator can revoke the allocation (forward-only). The scope value that expresses this is `allocate` — see [`allocate` Scope Semantics](02-auth-request.md#allocate-scope-semantics) in the Auth Request Lifecycle section for the mechanical details. |

The distinctions matter at runtime:
- **Transfer** is **one-time and exclusive** — the old owner truly loses authority
- **Allocation** is **additive** — the allocator still retains their own authority, but adds another holder
- **Creation** establishes the initial state, which can then be transferred or allocated

### New Nodes and Edges

Edge directions below are read as "`<source>` is _edge-name_ `<target>`":

- Edge: `Resource ──OWNED_BY──▶ Principal` — the resource is owned by the principal
- Edge: `Principal ──CREATED──▶ Resource` — the principal created the resource (creation provenance)
- Edge: `Principal ──ALLOCATED_TO──▶ Principal` (scoped by resource, with an Auth Request as provenance) — records that A has allocated some scope of authority over a specific resource to B

**Transfer is expressed as an Auth Request, not as a separate node.** When an owner wants to transfer a resource, they submit an Auth Request with `scope: [transfer]`, targeting the resource and naming the receiving principal. On approval, the `Resource ──OWNED_BY──▶ Principal` edge is rewritten to point to the new owner; the old owner loses all authority on the resource (move semantics, matching Rust's `let y = x;`).

A **TransferRecord** value object is embedded on the approving Auth Request as a named shape for readability: `TransferRecord { transfer_id = request_id, resource_id, from_principal, to_principal, timestamp, requestor, approver }`. It has **no independent identity** and **no separate immutability invariant** — it inherits both from the Auth Request's never-deleted retention and post-submission immutability. A correction to a past transfer is a **new** Auth Request (typically a reversing transfer), never an edit of the original.

> **Why no separate node.** Every TransferRecord field is already carried by the approving Auth Request (requestor ↔ from_principal, resource_slots ↔ resource_id and to_principal, responded_at ↔ timestamp, approvers ↔ approver). A separate node would be a denormalised view that could drift from the authoritative Auth Request; eliminating it eliminates the drift risk and the standalone immutability invariant. Transfers and Shared Allocations are now both traversable through the same Auth Request machinery, which also cleans up the prior asymmetry (creation as edge, transfer as dedicated node).

### Creation Ownership Rules

When a resource is created, the initial owner is determined by the **most-specific scope** rule:

| Condition | Initial owner |
|-----------|---------------|
| Creating agent has a `current_project` set | The project (`project:{current_project}`) owns the resource |
| Creating agent has a `current_organization` but no `current_project` | The org (`org:{current_organization}`) owns the resource |
| Creating agent has neither | The agent itself owns the resource |

An **explicit override** is available at creation time: a tool that creates a resource may (subject to its own permissions) specify a different initial owner, as long as the target owner is within the creating agent's allocation authority.

### Ownership by Resource Class

| Resource class | Default owner at creation | Transferable? | Allocatable? |
|----------------|----------------------------|---------------|--------------|
| `filesystem_object` in `/workspace/{project}/**` | project | Yes | Yes |
| `filesystem_object` in `/home/{agent}/**` | agent | Yes | Yes |
| `session_object` | project (if session has a `project:` tag) else agent | No (frozen-at-creation) | Yes (via Grants) |
| `memory_object` | project (if memory is tagged with a project) else agent | No | Yes |
| `external_service_object` (registered) | org platform admin | Yes | Yes |
| `network_endpoint` (domain) | org (via org-wide network policy) | No | Yes |
| `secret/credential` | org platform admin (or a specifically assigned custodian) | Yes | Yes (carefully) |
| `auth_request_object` | the requestor (for editing rights in Draft state; ownership transfers to the system at submission) | No | — |

**Frozen-at-creation composites** (session, memory) carry their tags (including scope tags that imply ownership) immutably. Ownership cannot be moved by retagging — the frozen-at-creation rule prevents it.

### Co-Ownership (Shared Resources)

Co-ownership is the state of having two or more principals holding `[allocate]` on the same resource. It is **not a separate concept** from Shared Allocation — it is the natural outcome when `allocate` has been granted to multiple principals (either by independent creation paths, or by an owner explicitly allocating to a second shareholder). A joint project spanning Acme and Beta is the canonical example: both orgs hold `[allocate]` on the joint-project resources.

The behaviors of co-ownership:

- Each co-owner allocates within their own share, and the effective allocation set across the resource is the **union** of all co-owners' allocations
- Any co-owner may fulfil an approver slot on an incoming Auth Request for the scope they cover (per the [Per-Resource Slot Model](02-auth-request.md#schema--per-resource-slot-model) — if a resource has multiple co-owners, each gets their own slot and *all* must approve for that resource)
- Revocation by one co-owner revokes only what that co-owner allocated; the other co-owners' allocations are unaffected. This is what "forward-only" means in a co-ownership setting: the revocation walks forward through the revoker's own sub-tree only.
- This matches the [Multi-Scope Session Access](06-multi-scope-consent.md#multi-scope-session-access) base-org rule — see [Co-Ownership × Multi-Scope Session Access](06-multi-scope-consent.md#co-ownership--multi-scope-session-access) below for the full interaction.

Co-ownership is therefore a first-class concept consistent with the rest of the model, not a special case.

### Worked Example — Lead, Deputy, Team Member

A project workspace `/workspace/project-a/**`:

1. **Creation.** `agent:lead-acme-1` creates the project. Ownership is established: `Resource(/workspace/project-a/**) ──OWNED_BY──▶ project:project-a`. The lead's ownership authority is derived from the lead's Template A grant, which was itself pre-authorized at template adoption time.

2. **Allocation to deputy.** `agent:lead-acme-1` allocates `scope: [allocate]` on the project workspace to `agent:deputy-2`. This is recorded as:
   - An Auth Request from the lead to the deputy with `scope: [allocate]` on the workspace
   - An `ALLOCATED_TO` edge from lead → deputy, with the Auth Request as provenance
   - The deputy now holds allocation authority (can further grant read access to team members) without holding ownership of the workspace itself

3. **Team-member read access.** `agent:team-3` requests read access to the project docs. Routing to the deputy (per the resource's routing table). Deputy approves. A read Grant is issued to `agent:team-3`.

4. **Transfer.** The project lead changes: `agent:lead-acme-1` is replaced by `agent:new-lead-4`. The lead's Template A grant is revoked (the `HAS_LEAD` edge was removed), and the new lead's Template A grant fires. This is **automatic for allocation authority**. For genuine ownership transfer (moving the project to a different sponsor, for example), an Auth Request with `scope: [transfer]` is submitted by the current owner; on approval, the `OWNED_BY` edge is rewritten to point to the new owner, and a TransferRecord value object is embedded on the Auth Request as the named audit shape.

5. **Revocation cascade.** If `agent:lead-acme-1` revokes the deputy's allocation (say, before the lead change), the deputy's downstream grants to team members cascade-revoke (forward-only): past reads remain in the audit log, but no new reads are permitted under those grants.

### Resource Catalogue

Every Organization maintains a `resources_catalogue` that enumerates the resource instances under its ownership or sub-allocation. A resource — **both primary fundamentals and composites** — can only be referenced by a project, agent, tool manifest, or grant if it is declared in the owning Organization's catalogue. The catalogue is the **single source of truth** for what exists in the org's scope.

**What the catalogue covers:**

- All **9 fundamentals**: `filesystem_object`, `process_exec_object`, `network_endpoint`, `secret/credential`, `data_object`, `tag`, `identity_principal`, `economic_resource`, `time/compute_resource`.
- All **composites** (`memory_object`, `session_object`, `external_service_object`, `model/runtime_object`, `control_plane_object`, `auth_request_object`, plus `inbox_object` and `outbox_object` from [§Composite Classes](#composite-classes-8)).
- **Composite instances constructed by the org's own operations.** Registering a new MCP server creates a new `external_service_object` instance; the registration operation **atomically** adds the new instance to the catalogue. Same for onboarding an LLM provider (`model/runtime_object`), minting a secret (`secret/credential`), or creating an agent inbox at agent-creation time.

**How catalogue entries are added:**

1. **At org setup** — the platform admin declares the initial catalogue as part of adopting the Standard Organization Template (see [07-templates-and-tools.md § Standard Organization Template](07-templates-and-tools.md#standard-organization-template) for the YAML shape).
2. **At runtime** — a schema-extension-style Auth Request (Template E shape — see [07 § Template E](07-templates-and-tools.md#opt-in-example-templates-c-d-and-e)) is submitted by a principal holding `[allocate]` on `control_plane_object`. On approval, the catalogue gains the new entry and the operation that produced it runs. Adding to the catalogue is itself an auditable event; the approving Auth Request is the extension's provenance.

**Effect on the Permission Check:**

The catalogue is the **structural pre-condition** for grant resolution. Before the runtime evaluates grants, constraints, or ceilings, it checks that every resource the tool call reaches is present in the owning org's catalogue. A call that reaches an out-of-catalogue resource is denied at Step 0 — before the grant-matching machinery runs. See [04-manifest-and-resolution.md § Formal Algorithm (Pseudocode)](04-manifest-and-resolution.md#formal-algorithm-pseudocode), Step 0.

**Why the catalogue is load-bearing:**

- **No ambient resources.** An agent cannot invoke a tool that reaches a resource the org has not explicitly declared, even if the tool's manifest looks correct. This prevents "the bash tool happens to resolve some DNS name the org never approved" from becoming an exploit.
- **Composite additions are auditable.** Runtime-constructed composites (new MCP server, new secret, new memory pool) flow through the same Auth Request machinery as everything else. The authority chain stays complete.
- **Projects and agents reference the catalogue, not the universe.** A project's `resource_boundaries` names a subset of the catalogue it operates within. An agent's grants select over the catalogue. This makes resource scoping composable and bounded.

---


## Composite Identity Tags (`#kind:`)

**The problem:** memory and session are both composed from `data_object + tag`, and `external_service_object` and `model/runtime_object` share `network_endpoint + secret/credential`. Without an additional mechanism, a grant for the bare fundamentals would be indistinguishable from a grant for a specific composite kind.

**The solution:** every composite — regardless of whether it holds `data_object` or only operational fundamentals — carries an implicit `#kind:{composite_name}` tag. The runtime auto-adds this tag to every instance of a composite at creation time:

- Every Memory node carries `#kind:memory`
- Every Session node carries `#kind:session`
- Every external-service invocation or handle carries `#kind:external_service`
- Every LLM API call carries `#kind:model_runtime`
- Every control-plane operation carries `#kind:control_plane`
- Future composites (e.g., `note_object`, `document_object`) automatically get their own `#kind:note`, `#kind:document`, etc.

**This is universal.** Composites without `data_object` (like `external_service_object`) still pull in `tag` implicitly, because the `#kind:` tag needs tag-predicate machinery to be queryable at check time. This is why `tag` is described as "the structural substrate for every composite."

### Instance Identity Tags (`{kind}:{id}`)

`#kind:{name}` establishes the **type** of a composite instance. But we also need to address **individual instances** — a specific Session, a specific Memory, a specific Auth Request. The convention:

> Every composite instance carries a **self-identity tag** of the form `{kind}:{instance_id}` in addition to its `#kind:{kind}` type tag. The runtime auto-adds this tag at instance creation time (just like the `#kind:` tag), and it cannot be set or modified by agents or tools.

**Examples:**

| Instance | `#kind:` tag | Self-identity tag |
|----------|--------------|-------------------|
| A Session with id `s-9831` | `#kind:session` | `session:s-9831` |
| A Memory with id `m-4581` | `#kind:memory` | `memory:m-4581` |
| An Auth Request with id `req-7102` | `#kind:auth_request` | `auth_request:req-7102` |
| An MCP server registration with id `mcp-github-7` | `#kind:external_service` | `external_service:mcp-github-7` |

**Single-instance vs set-of-instances selectors:**

The same tag-predicate grammar handles both, via inclusion or omission of the self-identity tag:

| Selector pattern | What it addresses |
|------------------|-------------------|
| `tags contains session:s-9831` | The single Session with id `s-9831` |
| `tags intersects {session:s-9831, session:s-9832}` | Two specific Sessions |
| `tags contains project:alpha AND tags contains #kind:session` | All Sessions belonging to project alpha |
| `tags contains agent:claude-coder-7 AND tags contains #kind:memory` | All Memories authored by `claude-coder-7` |
| `tags contains #kind:session` | All Sessions, period (typically only used by System Agents) |

**Why a separate self-identity tag instead of a dedicated `id` field on the resource?**

- **Consistency.** Every selector uses the same tag-predicate grammar; no special-cased `id` field syntax.
- **Composability.** Instance-identity tags compose naturally with scope tags (`tags contains session:s-9831 AND tags contains #public`).
- **Indexability.** Tag-based indexes work uniformly for all selectors, including single-instance lookups.
- **Audit symmetry.** When a Memory is extracted from a Session, the resulting Memory can carry a `derived_from:session:s-9831` tag — same grammar, same machinery, different meaning.

**Reserved tag namespaces.** The runtime owns these namespaces and rejects manual writes to them at publish/creation time:

- `#kind:*` — composite type identity
- `{kind}:*` for any registered composite — instance identity (e.g., `session:*`, `memory:*`, `auth_request:*`)
- `delegated_from:*` — lineage tag (already used on sessions)
- `derived_from:*` — derivation tag (e.g., a Memory extracted from a Session)

All other namespaces (`agent:*`, `project:*`, `org:*`, `task:*`, `role_at_creation:*`, `#public`, `#sensitive`, etc.) follow the lifecycle rules of their respective composites.

**Implication for tool authoring (see the [Authoring Guide](07-templates-and-tools.md#authoring-a-tool-manifest-a-guide-for-tool-creators) below):**

Tool creators don't write grant selectors (those live on Grants, which are authored by admins, the system, or Auth Request approvers). But three rules flow from this convention:

1. Tools that create composite instances must **not** declare `[modify]` actions on the reserved tag namespaces — the runtime assigns them at creation.
2. Tools that accept a target instance as a parameter should declare a `target_kinds:` field so the publish-time validator knows the tool resolves to a specific instance via a `{kind}:{id}` selector.
3. Tools that create cross-instance references (e.g., a Memory derived from a Session) must declare read access on the source kind as part of their manifest.

### Tool Creator Responsibility for `#kind:` Declarations

The runtime does NOT auto-infer which composite kinds a tool operates on. **Tool creators must declare `#kind:` values explicitly in the manifest.** Three forms:

1. **Single kind:** `#kind: memory` — the tool works on memory entities only
2. **Multiple specific kinds:** `#kind: [memory, note]` — the tool works on memories and notes
3. **Blanket kind:** `#kind: *` — the tool works on any composite, regardless of kind

**Enforcement — at publish time, not at runtime:**

The tool registry runs a **manifest validator** when a tool is published (or re-published after a code change). The validator enforces the `#kind:` rules before the tool can be invoked by any agent:

- **Missing a `#kind:` declaration for a composite the manifest touches → publish-time rejection.** If the manifest's `resource` field is `memory_object` but the manifest omits `#kind:`, the validator rejects the publish with a specific error: `"Manifest declares memory_object but is missing required #kind: memory."` The tool creator fixes the manifest and resubmits. No agent ever sees a broken manifest.
- **`#kind:` set inconsistent with declared fundamentals → publish-time rejection.** If the manifest declares `#kind: external_service` but the resource and transitive list don't contain the fundamentals `external_service_object` expands to, the validator flags the mismatch.
- **`#kind: *` is legal but throws a warning at publish time.** Accepted, but recorded in tool metadata for admin review.
- The validator specifies **which `#kind:` values are missing or inconsistent** in its rejection message, making the error actionable.

**Why publish-time and not runtime:**

- **Shift-left principle.** Catching manifest errors at publish time is standard for declarative systems. Runtime enforcement of manifest structure is a delayed failure mode — the tool would work until it happens to touch the missing kind, then suddenly deny operations.
- **Clear responsibility.** Publish-time rejection puts the error back on the tool creator immediately, while they still have the context to fix it.
- **Runtime simplicity.** Once published, the manifest is trusted. The runtime still performs the Permission Check on every invocation (does the agent hold grants for the manifest's declared fundamentals and `#kind:` values?), but it does NOT re-validate the manifest itself.

### Enforcement Asymmetry

The two-tier design with explicit `#kind:` declarations enables a three-tier enforcement rule, split between **publish time** and **runtime**:

| Issue | When detected | Response | Rationale |
|-------|---------------|----------|-----------|
| Fundamental missing | Publish time via declaration consistency check | **Publish rejected** | Fundamentals are the bedrock. A missing fundamental means the tool has an uncovered capability. |
| `#kind:` declaration missing for a declared composite | Publish time | **Publish rejected** | If the manifest declares `memory_object` but omits `#kind: memory`, the validator flags the inconsistency. |
| `#kind:` set inconsistent with fundamentals | Publish time | **Publish rejected** | Incompatible combinations (e.g., `#kind: external_service` without `network_endpoint`) are rejected. |
| Composite label missing (fundamentals + `#kind:` present) | Publish time | **Warning only** | Composite label is documentation sugar. Correctness unaffected. |
| `#kind: *` declaration (blanket) | Publish time | **Warning, but accepted** | Legitimate for polymorphic tools. Recorded in tool metadata for admin review. |
| Agent lacks required grants for the tool's declared fundamentals/`#kind:` | Runtime (on each invocation) | **Invocation denied** | Normal Permission Check. Manifest validity is assumed; authorization is checked. |

**Worked examples (publish time):**

- `bash` manifest declares `process_exec_object`, `filesystem_object`, `network_endpoint`, `secret/credential`. No composites involved. **Publish accepted.**
- `curl_to_github` manifest declares `external_service_object` AND `#kind: external_service`. Validator expands the composite, sees the `#kind:` matches. **Publish accepted.**
- `curl_to_github` manifest declares `external_service_object` but omits `#kind:`. **Publish rejected** with message: `"Manifest declares external_service_object but is missing required #kind: external_service."`
- A tool reads both memories and sessions and declares `#kind: [memory, session]`. **Publish accepted.**
- A backup tool declares `#kind: *`. **Warning at publish**: `"Tool 'backup_all' uses blanket #kind: * — consider narrowing if possible."` Accepted.

**Worked example (runtime):**

Agent `auditor-3` invokes the already-published `read_memory` tool. Manifest declares `resource: memory_object`, `#kind: memory`, `actions: [read]`. Runtime gathers the agent's grants covering `data_object + tag`, filters by the grant's effective selector (which includes `#kind:memory` if the grant targets `memory_object`), and decides: if no grant matches, **invocation denied**. The manifest itself is trusted; only the agent's authorization is checked.

### Composite Creation Checklist

A new composite class can be added to the ontology at any time — the two-tier model is designed to extend cleanly. But every new composite must declare a specific set of fields so the runtime, tool creators, and permission administrators know how to treat it.

**Why this matters:** an agent's **Power** (in the sense defined in [agent.md](agent.md) — verbs, phrases, and sentences) is ultimately a set of actions over resources. Composites are the reusable unit of "a bundle of actions over a bundle of resources." Having a clean definition of how to create a composite means:

- New agent capabilities can be added systematically without ad-hoc invention
- Agent Power is discoverable through the composite registry
- Scaling to new use cases (document stores, knowledge graphs, project management integrations) becomes mechanical
- The philosophical grounding — that Power is a set of queryable verbs and phrases — is preserved at the permission layer

A composite definition must include 13 fields:

1. **Name** — stable identifier used in manifests and grants. Convention: `{concept}_object` (e.g., `memory_object`, `document_object`).
2. **Identity tag** — the `#kind:{name}` value every instance carries. Convention: `#kind:{name}` without the `_object` suffix (e.g., `#kind:memory`).
3. **Constituent fundamentals** — which fundamentals this composite expands to at runtime. `tag` is always implicitly included.
4. **Action set** — which actions from the Standard Action Vocabulary apply, with definitions specific to this composite.
5. **Constraint set** — which constraints apply, inherited from constituents plus composite-specific ones.
6. **Tag vocabulary** — what namespace tags the composite supports beyond `#kind:` (e.g., `agent:`, `project:`, `org:`).
7. **Lifecycle rules** — how instances are created, modified, and deleted. Who assigns tags? Are tags mutable? Is there a status flow?
8. **Scope resolution rules** — does this composite participate in Multi-Scope Session Access resolution? Or have its own rules?
9. **Default grants** — what grants does the system auto-issue for this composite at agent creation?
10. **Authority templates** — does the composite support auto-issued template grants when relationships form?
11. **Consent policy participation** — does reading instances of this composite go through the Consent Policy machinery?
12. **Relationship to other composites** — does this composite share fundamentals with another composite? Document any semantic overlap.
13. **Example manifest entry** — a minimal working tool manifest that targets this composite.

**Worked example — a hypothetical `document_object` composite:**

```yaml
composite:
  name: document_object
  identity_tag: "#kind:document"
  constituent_fundamentals: [data_object, tag]   # tag is implicit
  actions:
    - read: retrieve document contents matching a tag predicate
    - list: enumerate documents matching a tag predicate (metadata only)
    - create: create a new document with a chosen tag set and initial content
    - modify: replace document content
    - delete: remove a document the caller owns
    - annotate: add a comment or annotation without modifying main content
  constraints:
    - tag_predicate: required for all operations
    - max_size_bytes: 1048576 (1MB per document)
    - universal: time_window, approval, non_delegability, purpose
  tag_vocabulary:
    - agent:{agent_id}: author
    - project:{project_id}: project scope
    - org:{org_id}: organization scope
    - folder:{folder_name}: user-defined folder hierarchy
    - #public
  lifecycle:
    - Creation: agent chooses tags at document creation
    - Tags mutable: agent may retag documents they authored
    - Status flow: #draft → #published → #archived
  scope_resolution:
    - Uses standard tag-intersection access (same as memory)
    - Does not participate in Multi-Scope Session Access
  default_grants:
    - Agent-context grant: [read, list] on documents whose tags intersect
      {agent:X, project:current(X), #public}
  authority_templates:
    - Template A (Project Lead): leads get [read, list, annotate] on all documents in their project
    - Template E only for full [modify, delete] access
  consent_policy:
    - Template A grants honor the org's consent_policy
  relationships_to_other_composites:
    - Shares data_object + tag with memory_object and session_object
    - Distinguished by #kind:document vs #kind:memory vs #kind:session
    - Unlike memories, documents have a status flow; unlike sessions, documents are not frozen at creation
  example_manifest_entry:
    tool: edit_document
    manifest:
      resource: document_object
      actions: [read, modify]
      kind: [document]
      constraints:
        tag_predicate: "tags contains project:{project_id}"
        max_size_bytes: 1048576
      delegable: false
      approval_mode: auto
```

**Review process:** before a new composite is merged into the runtime's composite registry, the 13-field definition is reviewed by whoever owns the permission layer (initially a human; eventually perhaps a System Agent). A composite that declares all 13 fields coherently is safe to deploy; a composite with ambiguous or conflicting fields is rejected until it's tightened.

---
