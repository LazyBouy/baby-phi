<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Memory as a Resource Class

> Memory access fits naturally into the standard permission model — it does not need a parallel system. This section shows how the canonical (subject, action, resource, constraints, provenance) shape applies to Memory, and why earlier framings of "tag matching" turn out to be an instance of standard resource selectors.
>
> The conceptual home for the Memory model is [agent.md](agent.md). This section specifies *how* memory access is enforced as a normal permission check.

### Resource Class: `memory_object`

A Memory node is a `memory_object` resource. Unlike `filesystem_object` (path-prefix selectors) or `process_exec_object` (command-pattern selectors), `memory_object` uses **tag predicates** as its selector grammar.

A tag predicate is one of:
- `tags contains T` — the memory must carry tag T
- `tags intersects {T1, T2, ...}` — at least one of the listed tags must be present
- `tags subset_of {T1, T2, ...}` — all of the memory's tags must come from this set
- compositions via `AND` / `OR` / `NOT`

This is the only thing that distinguishes memory permissions from any other resource permission — the **selector grammar**. Everything else (action, constraints, provenance, the runtime check) is the same as the rest of the permission model.

### Tag Vocabulary

A Memory node is tagged with zero or more of:

| Tag | Source | Meaning |
|-----|--------|---------|
| `agent:{agent_id}` | Authoring agent | Author or owner of the memory |
| `project:{project_id}` | Authoring agent | Project the memory belongs to |
| `org:{org_id}` | Authoring agent | Organization the memory belongs to |
| `#public` | Authoring agent | Open to all agents in the system |

### Standard Actions Applied to Memory

The standard action vocabulary maps directly — no new actions are needed:

| Standard Action | Memory Operation |
|-----------------|------------------|
| `recall` (Memory category) | Retrieve a memory matching a tag predicate |
| `store` (Memory category) | Create a new memory with a chosen tag set |
| `delete` (Mutation category) | Revoke a memory the agent owns |
| `read` (Data category) | Read a Session in order to extract memories from it (Supervisor case — operates on `session_object`, not `memory_object`) |

### Default Grant Derived from Agent Context

Every agent automatically receives a system-provenance grant whose selector is computed from the agent's **current context** (current project, base project, current org, etc.):

```yaml
grant:
  # subject = agent:X (the source of the HOLDS_GRANT edge)
  action: [recall]
  resource:
    type: memory_object
    selector: "tags intersects {
      agent:X,                       # own memories
      project:current_project(X),    # current project
      project:base_project(X),       # base project
      org:current_organization(X),   # current org
      org:base_organization(X),      # base org
      #public                        # globally public
    }"
  constraints: {}
  provenance: system:agent-context
  delegable: false
  approval_mode: auto
  audit_class: silent
  revocation_scope: dynamic   # selector is recomputed when context changes
```

Plus extra tags from **secondary memberships** — an agent participating in multiple projects gets the project tag for each.

The selector is **dynamic**: when the agent changes its `current_project` or joins a new org, the system recomputes the allowable set. This is what `revocation_scope: dynamic` means in this context.

The system also issues a parallel `[store]` grant on `memory_object` so the agent can write memories under tags it is allowed to use.

### Why "Private Memory" Is Not a Special Case

Earlier drafts of this doc had a special rule: *"if the memory's only tag is an agent tag, it's private."* Under the resource-class model, this rule **disappears** — the same outcome falls out of plain set intersection between the memory's tags and the asking agent's selector.

| Memory tags | Asking agent's allowable tags | Intersection | Outcome |
|-------------|-------------------------------|--------------|---------|
| `{agent:X}` | agent X's selector (contains `agent:X`) | `{agent:X}` ≠ ∅ | X reads it ✓ |
| `{agent:X}` | agent Y's selector (no `agent:X`) | ∅ | Y cannot read it — this *is* "private to X" ✓ |
| `{agent:X, project:alpha}` | agent Y's selector (contains `project:alpha`) | `{project:alpha}` ≠ ∅ | Y reads it; `agent:X` is interpreted as authorship, not restriction ✓ |
| `{#public}` | any agent's selector (always contains `#public`) | `{#public}` ≠ ∅ | All agents read it ✓ |
| `{org:acme}` | agent in org acme | `{org:acme}` ≠ ∅ | Visible to org members ✓ |

The phrase "private memory" is now just **shorthand for a memory whose tag set has no overlap with any other agent's selector** — which happens to mean "tagged only with the owner's `agent:` tag." It is an emergent property of set intersection, not a separate rule that needs to be hard-coded into the runtime.

### Reference Implementation (Set Intersection)

The runtime check for memory recall is the standard Permission Check with set intersection as the selector evaluator:

```rust
// Selector evaluation for memory_object resources.
// This is what the standard Permission Check delegates to when the
// resource type is memory_object.
fn selector_matches_memory(grant: &Grant, memory: &Memory) -> bool {
    let allowable = grant.resource_selector.as_tag_set();
    !allowable.is_disjoint(&memory.tags)
}
```

**Properties of this algorithm:**
- **Set intersection** is the core operation — fast, predictable, easy to index
- **No explicit ACLs** — access is implicit in the tag overlap
- **Composable** — adding a project tag instantly grants access to all current project participants
- **Authorship preserved** — `agent:X` tags survive into public memories without restricting them
- **No special cases** — the same code path handles private memories, project memories, public memories, and supervisor-extracted memories

### Worked Example: Memory Recall Permission Check

Agent `claude-coder-7` calls the `recall_memory` tool with a query.

**Tool Manifest** (the `recall_memory` tool):

```yaml
tool: recall_memory
manifest:
  actions: [recall]
  resource: memory_object
  constraints:
    tag_predicate: required   # caller must scope the recall
```

**Agent's Grant** (the default agent-context grant from above):

```yaml
grant:
  # subject = agent:claude-coder-7
  action: [recall]
  resource:
    type: memory_object
    selector: "tags intersects { agent:claude-coder-7, project:website-redesign, org:acme, #public }"
  provenance: system:agent-context
```

**Runtime Permission Check:**

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a grant for `[recall]` on `memory_object`? | ✓ (default agent-context grant) |
| 2 | Resource type matches? | ✓ (both `memory_object`) |
| 3 | Tool's required `tag_predicate` constraint satisfied? | ✓ (grant supplies the tag predicate) |
| 4 | For each candidate memory in the store: does its tag set intersect the grant's selector? | per-memory filter |
| 5 | Org and project ceilings allow this? | ✓ (no restriction on memory recall within org) |
| → | **Allowed**, returning only memories whose tags intersect the agent's allowable set | |

This is the same Permission Check used for `bash` and `write_file` — there is no memory-specific code path. The only memory-specific element is the **selector evaluator** that knows how to interpret tag predicates.

### Supervisor Extraction as Two Standard Grants

The Supervisor extraction capability (read a subordinate's session, then create a memory derived from it) is expressed as **two standard grants** — no special "extraction path":

**Grant A: Read subordinate sessions** — operates on `session_object` (see [Sessions as a Tagged Resource](#sessions-as-a-tagged-resource) below for the full session model):

```yaml
grant:
  # subject = agent:supervisor-7
  action: [read]
  resource:
    type: session_object
    selector: "tags contains project:website-redesign"
  constraints:
    purpose: memory_extraction
  provenance: system:has_lead@project:website-redesign
  delegable: false
  revocation_scope: revoke_when_edge_removed
```

This grant is auto-issued by an **Authority Template** — see the Sessions section. The selector is a tag predicate over the session's frozen tags, exactly the same shape as memory selectors.

**Grant B: Store memories derived from those sessions** — operates on `memory_object`:

```yaml
grant:
  # subject = agent:supervisor-7
  action: [store]
  resource:
    type: memory_object
    selector: "tags subset_of supervisors_tagging_scope(supervisor-7)"
  constraints:
    requires_source_session: true   # extracted memories must reference the source
  provenance: system:supervisor-role
  delegable: false
```

The Supervisor's tagging choice (private to themselves, published to project, `#public`, etc.) is unconstrained except by the `subset_of` selector — they can use any tags they themselves are allowed to apply.

### Other Open Questions for Memory Permissions

- [ ] Should agents be able to **revoke** their own published memories? If so, what about copies that other agents have already cached?
- [ ] Can a Supervisor extract memories from sessions of an agent that has since been removed from the project?
- [ ] Do memories have TTL? Should some tags imply different retention policies?

---

## Sessions as a Tagged Resource

> Sessions get the same treatment as Memory. The selector grammar is tag predicates, the runtime check is set intersection, and the question of "who can supervise whom" — previously a structural problem with five candidate models — dissolves into "which grant templates does the system auto-issue when relationships form."
>
> The conceptual home for the Session model is [agent.md](agent.md) and brainstorm.md Section 2 (Loop, Turn, Message hierarchy). This section specifies *how* session access is enforced as a normal permission check.

### Resource Class: `session_object`

A Session node is a `session_object` resource. Like `memory_object`, its selector grammar is **tag predicates**:

- `tags contains T`
- `tags intersects {T1, T2, ...}`
- `tags subset_of {T1, T2, ...}`
- compositions via `AND` / `OR` / `NOT`
- hierarchical match such as `tags any_match org:acme/**`

Loops and Turns inherit their parent Session's tags for permission purposes — if you can read the Session, you can read its Loops and Turns. Messages within a Session also inherit, though they may carry additional content-level tags (e.g. `#sensitive`) for finer filtering.

### Tag Vocabulary for Sessions

Session tags are **assigned automatically by the system at session creation** — agents do not choose them, and they are **frozen at creation** (see "Specific Considerations" below for why).

| Tag | Source | Mutability |
|-----|--------|------------|
| `agent:{owner_agent_id}` | The executing agent | **Immutable** (authorship cannot be reassigned) |
| `project:{project_id}` | Agent's `current_project` at session start | **Frozen at creation** |
| `org:{org_id}` | Agent's `current_organization` at session start | Frozen at creation |
| `task:{task_id}` | If the session was created to execute a specific Task | Immutable |
| `delegated_from:{parent_loop_id}` | If spawned by a sub-agent tool call | Immutable |
| `role_at_creation:{role}` | The owner's role within the project at session start (e.g. `worker`, `lead`) | Frozen at creation |
| `agent_kind:{kind}` | `system` / `intern` / `contract` (from agent taxonomy) | Frozen at creation |
| `#archived` / `#active` | Session lifecycle state | **Mutable** (system-managed) |

> The lifecycle tags (`#archived` / `#active`) are the only mutable tags on a session. Everything else is locked at creation. This is the **frozen-at-creation rule** — see Specific Considerations.

### Standard Actions Applied to Sessions

The standard action vocabulary maps directly:

| Standard Action | Session Operation |
|-----------------|-------------------|
| `read` (Data category) | Retrieve a Session and its contents (Loops, Turns, Messages) |
| `list` (Discovery category) | Enumerate sessions matching a tag predicate without reading content |
| `inspect` (Discovery category) | Read session metadata (status, usage, timing) without message content |
| `append` (Mutation category) | Add a new Loop/Turn to an active session — typically held by the owning agent |
| `delete` (Mutation category) | Remove or archive a session — typically held by the owner or an admin |
| `export` (Data category) | Bulk read for backup, transfer, or analysis (often more restricted than `read`) |

### Default Grants Issued to Every Agent

Every agent receives two system-provenance grants for sessions at creation:

**Default Grant 1: Read your own sessions**

```yaml
grant:
  # subject = agent:claude-coder-7
  action: [read, list, inspect, append]
  resource:
    type: session_object
    selector: "tags contains agent:claude-coder-7"
  constraints: {}
  provenance: system:agent-default
  delegable: false
  approval_mode: auto
  audit_class: silent
  revocation_scope: never
```

**Default Grant 2: List public sessions in current scopes**

```yaml
grant:
  # subject = agent:claude-coder-7
  action: [list, inspect]
  resource:
    type: session_object
    selector: "tags intersects {
      project:current_project(claude-coder-7),
      project:base_project(claude-coder-7),
      org:current_organization(claude-coder-7),
      org:base_organization(claude-coder-7)
    }"
  constraints: {}
  provenance: system:agent-context
  delegable: false
  approval_mode: auto
  audit_class: silent
  revocation_scope: dynamic
```

Note that the default grants give an agent **read** of its own sessions but only **list/inspect** of project/org sessions — full content read of someone else's session requires an Authority Template grant.

### Authority Templates (formerly the Authority Question)

> **Status update:** What was previously an open structural question (5 mutually exclusive models) is now a set of **grant templates**. An organization picks which templates the system should auto-issue when relationships are formed. **Multiple templates can run side-by-side** because they are just Permission grants — the runtime composes them via standard set union of selector matches.

When a relationship is formed in the graph (e.g., a `HAS_LEAD` edge is created, or a `DELEGATES_TO` edge is created), the system optionally auto-issues a Grant matching one of the templates below. When the relationship is removed, the grant is revoked (because the grants are tagged with `revocation_scope: revoke_when_edge_removed`).

#### Template A: Project Lead Authority

**Trigger:** A `HAS_LEAD` edge is created from Project P to Agent X.

```yaml
grant:
  # subject = agent:X
  action: [read, inspect, list]
  resource:
    type: session_object
    selector: "tags contains project:P"
  constraints: {}
  provenance: system:has_lead@project:P
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: revoke_when_edge_removed
```

**Properties:**
- ✅ Simple to compute (one edge lookup at issuance)
- ✅ Maps to "team lead" mental model
- ✅ Naturally cleaned up when leadership changes
- ⚠️ Coarse-grained — a 50-agent project gives the lead a firehose; consider pairing with `#sensitive` content-level filtering

#### Template B: Direct Delegation Authority

**Trigger:** Agent A spawns Agent B via a `DELEGATES_TO` edge at loop `L42`.

```yaml
grant:
  # subject = agent:A
  action: [read, inspect]
  resource:
    type: session_object
    selector: "tags contains delegated_from:L42"
  constraints: {}
  provenance: system:delegation@L42
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: end_of_delegation_chain
```

**Properties:**
- ✅ Fine-grained — only the actual delegation chain matters
- ✅ Naturally scoped to the work that was delegated
- ✅ Composes with Template A (a project lead also gets delegation grants for the work they personally spawn)
- ⚠️ A project lead who didn't personally spawn every team member won't reach all of them via this template alone

#### Template C: Hierarchical Org Chart

**Trigger:** Agent X is appointed to a node in the Organization tree (e.g., `org:acme/eng/web/lead`).

```yaml
grant:
  # subject = agent:X
  action: [read, inspect, list]
  resource:
    type: session_object
    selector: "tags any_match org:acme/eng/web/**"
  constraints: {}
  provenance: system:org_chart@acme/eng/web/lead
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: revoke_when_edge_removed
```

**Properties:**
- ✅ Matches real-world organizational mental models
- ✅ Captures multi-level supervision via subtree matching
- ⚠️ Requires modeling the org tree explicitly with hierarchical names
- ⚠️ Imposes a structure that may not fit all use cases

#### Template D: Project-Scoped Role

**Trigger:** Agent Y is assigned a `supervisor` role on Project P (an edge property on `HAS_AGENT`).

```yaml
grant:
  # subject = agent:Y
  action: [read, inspect]
  resource:
    type: session_object
    selector: "tags contains project:P AND tags contains role_at_creation:worker"
  constraints: {}
  provenance: system:project_role@project:P/supervisor
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: revoke_when_edge_removed
```

**Properties:**
- ✅ Honors project boundaries (no cross-project surveillance)
- ✅ Only sees worker sessions, not other supervisors' sessions
- ⚠️ Requires per-project role assignment

#### Template E: Explicit Capability (no template)

Just a manually issued grant — no template needed. This was always available; it's the escape hatch for unusual cases. Example: a one-time auditor reviewing a specific project for a week.

```yaml
grant:
  # subject = agent:auditor-9
  action: [read, inspect]
  resource:
    type: session_object
    selector: "tags contains project:website-redesign"
  constraints:
    purpose: compliance_audit
  provenance: agent:human-sarah@2026-04-09
  delegable: false
  approval_mode: human_approved
  audit_class: alerted
  expires_at: 2026-04-16T00:00:00Z
  revocation_scope: manual
```

#### Template Composition

The big difference from the old "five models" framing: **templates are not mutually exclusive**. An org can enable any combination, and an agent's effective access is the union of all grants it holds. Common combinations:

| Org style | Templates enabled | Effect |
|-----------|-------------------|--------|
| Small team, flat | A only | Lead reads everything in the project |
| Startup, delegation-driven | A + B | Lead reads project + every agent reads what they delegated |
| Corporate, hierarchical | A + C | Lead reads project + org tree authority above the project |
| High-privacy | D + E | Only explicit role-based access; no automatic grants |
| Maximum control | E only | Every grant is manually issued; no defaults |

**Recommended default for baby-phi:** Templates **A + B**. This gives sensible behavior ("project leads can read their team; agents can read what they delegated") with two cheap-to-compute auto-issued grants.

### Worked Examples

#### Example 1: A worker reads their own session

Agent `claude-coder-7` calls `read_session` on a session it owns.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a grant for `[read]` on `session_object`? | ✓ (Default Grant 1) |
| 2 | Resource type matches? | ✓ |
| 3 | Session tags `{agent:claude-coder-7, project:website-redesign, org:acme}` intersect grant selector `{tags contains agent:claude-coder-7}`? | ✓ |
| 4 | Constraints satisfied? | ✓ (none) |
| → | **Allowed** | |

#### Example 2: A worker tries to read another worker's session

Agent `claude-coder-7` tries to read a session owned by `claude-coder-9`.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a grant for `[read]` on `session_object`? | ✓ (Default Grant 1) |
| 2 | Session tags `{agent:claude-coder-9, project:website-redesign}` intersect grant selector `{tags contains agent:claude-coder-7}`? | ✗ |
| 3 | Any other `[read]` grant on `session_object` for this agent? | ✗ (no Authority Template grant — claude-coder-7 is not a lead, did not delegate to claude-coder-9) |
| → | **Denied** | |

Note that the Default Grant 2 (list/inspect) would still let `claude-coder-7` *see that the session exists* but not read its contents.

#### Example 3: A project lead reads a worker's session under Template A

Agent `lead-3` has the Template A grant from being lead of `project:website-redesign`. They call `read_session` on a session owned by `claude-coder-9`.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a grant for `[read]` on `session_object`? | ✓ (Template A grant) |
| 2 | Session tags `{agent:claude-coder-9, project:website-redesign, role_at_creation:worker}` intersect grant selector `{tags contains project:website-redesign}`? | ✓ |
| 3 | Constraints satisfied? | ✓ (`purpose: memory_extraction` is required by the manifest if they're extracting; otherwise no constraint applies) |
| 4 | Org and project ceilings allow it? | ✓ |
| → | **Allowed**, audit_class `logged` so the read is recorded | |

#### Example 4: A delegation-chain read using Template B

Agent `coordinator-2` delegated work to `claude-coder-9` at loop `L42`. The Template B grant fires. Now `coordinator-2` reads the delegated session.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a grant for `[read]` on `session_object`? | ✓ (Template B grant from `L42`) |
| 2 | Session tags `{agent:claude-coder-9, delegated_from:L42, project:other-project}` intersect grant selector `{tags contains delegated_from:L42}`? | ✓ |
| → | **Allowed** | |

> Notice that `project:other-project` is irrelevant — the delegation grant doesn't care which project the subordinate's session ended up in. This is the value of delegation-scoped authority: it follows the work, not the project.

#### Example 5: Cross-project subordinate (the case Templates A and B handle differently)

Agent `claude-coder-9` is a member of two projects: `project:website-redesign` (where they were delegated work by `coordinator-2`) and `project:internal-tool` (where they work independently).

`coordinator-2` (who has Template B from delegation) tries to read `claude-coder-9`'s session in `project:internal-tool`.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds any `[read]` grant on `session_object`? | ✓ (Template B grant from `L42`) |
| 2 | Session tags `{agent:claude-coder-9, project:internal-tool}` intersect grant selector `{tags contains delegated_from:L42}`? | ✗ (no `delegated_from:L42` tag — this session was not delegated by `coordinator-2`) |
| → | **Denied** | |

This is exactly the right answer: delegating work to an agent does not give you access to their *other* work. The frozen-at-creation rule guarantees this — the session in `project:internal-tool` was tagged when it was created, and `delegated_from` was not part of that tagging.

#### Example 6: Time-bounded auditor under Template E

`agent:auditor-9` has the explicit grant from Example E with `expires_at: 2026-04-16`. On 2026-04-10, they call `read_session` on a session in `project:website-redesign`.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a `[read]` grant on `session_object`? | ✓ (explicit grant) |
| 2 | Selector matches? | ✓ |
| 3 | `expires_at` in the future? | ✓ (2026-04-10 < 2026-04-16) |
| → | **Allowed**, audit_class `alerted` so a notification fires | |

On 2026-04-17:

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds a `[read]` grant on `session_object`? | ✗ (grant has expired and been auto-revoked) |
| → | **Denied** | |

#### Example 7: A worker tries to retag their own session

Agent `claude-coder-9` tries to add a `project:other-project` tag to one of their sessions, hoping to gain access to other-project's lead.

| Step | Check | Result |
|------|-------|--------|
| 1 | Agent holds any grant for `[modify]` on `session_object`? | ✗ — no grant template issues a `modify` action on the project tag (only `#archived`/`#active` lifecycle tags are mutable, and only via system-managed transitions) |
| → | **Denied**, the request never reaches the storage layer | |

This is why the **frozen-at-creation rule** matters: there's no mutation grant for the structural tags, so attempts to retag are denied at the permission layer, not at a separate "validation" step.

### Specific Considerations for Sessions

These are the only places sessions differ meaningfully from memory:

#### 1. Frozen-at-creation tags (immutability)

Memory tags are chosen by the agent at creation; session tags are **assigned by the system from the agent's context** at session creation, and (with the exception of `#archived`/`#active`) **cannot be changed**.

**Why this matters:**

- A worker who moves from `project:A` to `project:B` should not retroactively grant the `project:B` lead access to their old `project:A` sessions.
- A Contract agent that escapes to a different organization cannot exfiltrate session history by retagging.
- The "Authority Template" grants reference *frozen* facts, so they remain consistent over time.

**Implementation:** No grant template issues `[modify]` on the structural tags of `session_object`. The only mutable tags are lifecycle (`#archived`/`#active`), which are managed by system events (e.g., a project closing causes all its sessions to become `#archived`).

#### 2. Session vs Loop vs Turn vs Message granularity

A Session contains Loops, which contain Turns, which contain Messages. The `session_object` resource class covers the whole tree by default — a `read` grant on a session implicitly grants `read` on its loops, turns, and messages.

If finer granularity is needed (e.g., redacting individual messages), Messages can carry **content-level tags** like `#sensitive` or `#secret` that compose with session-level grants:

```yaml
grant:
  # subject = agent:lead-3
  action: [read]
  resource:
    type: session_object
    selector: "tags contains project:website-redesign AND NOT tags contains #sensitive"
  ...
```

This lets a project lead read everything in the project *except* messages flagged sensitive — a common compliance pattern.

#### 3. Lineage-aware tags

`delegated_from:{parent_loop_id}` is special: it captures a relationship that exists at session creation time but references another session/loop. This makes the delegation graph **queryable as tags**, not just as edges. The selector `tags contains delegated_from:L42` is a tag-based way to ask "what work descended from loop L42?" — equivalent to a graph traversal but cheaper.

For deep delegation chains, the system can also write a transitive tag: `delegated_from_chain:L42` (added when the spawning loop itself was descended from L42). This lets a top-level coordinator read all transitively-spawned work without the runtime traversing the chain at check time.

#### 4. Cross-project and cross-org sessions

Sessions can have multiple `project:` tags (cross-project work) **or** multiple `org:` tags (joint project), but **not both simultaneously**. This is the only hard schema constraint on session tags. See [Multi-Scope Session Access](06-multi-scope-consent.md#multi-scope-session-access) for the constraint, the resolution rule, and worked examples.

In the common case, a session is born in exactly one project (the agent's `current_project` at creation) and is tagged accordingly. Cross-project work is usually modeled as **multiple sessions**, each tagged with its respective project, which keeps resolution trivial. But the system does not *enforce* single-project sessions — work that legitimately spans projects within the same org may produce a single session with multiple `project:` tags, and the cascading scope resolution rule handles it cleanly.

The forbidden shape is **multi-project AND multi-org on the same session**. When work needs to span multiple orgs *and* multiple projects, the system requires creating a parent project that is itself jointly owned by those orgs — and the session belongs to that parent project. This collapses any would-be multi-project-multi-org session into the joint-project case, which the resolution rule handles natively.

---
