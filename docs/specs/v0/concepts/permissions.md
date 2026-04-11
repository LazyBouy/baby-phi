<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-11 by Claude Code -->

# Permissions Model (Capability-Based)

> Extracted from brainstorm.md Section 4 + permission_model.md. Refined 2026-04-11 with two-tier ontology (fundamentals + composites), `tag` fundamental, `#kind:` identity tags, publish-time manifest validation, Standard Permission Templates, Tool Authority Manifest Examples Catalog, Manifest Authoring Guide, and an end-to-end worked use case.
> See also: [agent.md](agent.md), [organization.md](organization.md) (resolution hierarchy), [project.md](project.md) (project-level rules)

---

## Core Insight

**Permissions are not about tools — they're about actions on resources with constraints.**

This mirrors capability-based security and cloud IAM: authority is tied to a specific object and operation, not ambient possession of a broad tool. Tools are merely *implementations* of actions on resources.

## Canonical Shape

```
Permission = ⟨ subject, action, resource, constraints, provenance ⟩
```

This 5-tuple is the shape of a **Permission Grant** — a capability HELD by a specific principal. Every component answers a distinct question:

| Component | Question | Discussed In |
|-----------|----------|--------------|
| **subject** | WHO holds the capability? | The Five Components → Subject |
| **action** | WHAT operation is permitted? | Standard Action Vocabulary |
| **resource** | ON WHAT is the action performed? | Resource Ontology |
| **constraints** | UNDER WHAT CONDITIONS does the permission apply? | Constraints |
| **provenance** | WHO granted this and HOW? | The Five Components → Provenance |

> **Important distinction:** The 5-tuple describes a **Permission Grant** (what an agent IS allowed to do). It is **not** the same as a **Tool Authority Manifest** (what a tool REQUIRES from its caller). The two are reconciled at runtime by a Permission Check. See the three sections below for each shape and a worked example showing how they interact.

---

## The Five Components

### Subject — WHO

The subject is the **principal** on whom the authority is conferred. In baby-phi, the subject is **implicit in the graph edge** that connects the principal to the Permission node — it is not stored as a property on the Permission node itself.

```
Agent ──HAS_PERMISSION──▶ Permission     -- the source of the edge IS the subject
Project ──HAS_PERMISSION──▶ Permission   -- project-scoped grant
Organization ──HAS_PERMISSION──▶ Permission  -- org-level ceiling
```

**Examples of subjects:**

| Subject | Meaning |
|---------|---------|
| `agent:claude-coder-7` | A specific agent instance |
| `project:website-redesign` | A project (propagates to all member agents) |
| `org:acme-corp` | An organization (topmost ceiling for everything inside) |
| `role:lead@website-redesign` | Anyone holding the lead role within a project |
| `system` | The platform itself (used for non-revocable bootstrap permissions) |

> **Why subject is implicit in the edge:** A Permission node is reusable — the same capability shape can be granted to multiple subjects. Storing subject as a property would force duplication of the Permission node per subject. Modeling subject as the edge source allows one Permission to be referenced by many `HAS_PERMISSION` edges.

### Provenance — WHO GRANTED IT and HOW

Provenance is the **audit trail** of authority. It records who created the grant and how.

**Examples of provenance:**

| Provenance | Meaning |
|------------|---------|
| `system` | Granted by the platform at bootstrap (e.g., default permissions for a System Agent) |
| `agent:human-sarah@2026-04-01` | Granted by a human sponsor at a specific time |
| `agent:admin-bot` | Granted by an administrative agent's explicit action |
| `config:org-default.toml#network-policy` | Declared in a config file at org setup time |
| `inherited_from:org:acme-corp` | Propagated down from an org-level grant |
| `delegated_from:agent:supervisor-3` | Passed down via a delegation chain |
| `contract:bid-4581` | Granted as part of accepting a contract bid (auto-revokes on contract end) |

**What provenance enables:**

1. **Auditing** — "Who decided this agent could read the production database?" is answerable by looking at provenance.
2. **Revocation cascades** — If a parent grant is revoked, all `inherited_from:` and `delegated_from:` grants that descend from it should also be revoked.
3. **Trust assessment** — A permission with provenance `system` is more trusted than one with provenance `agent:random-bot`. This matters for permission-to-grant-permissions decisions.
4. **Time-bounded grants** — Provenance like `contract:bid-4581` tells the system when to auto-revoke (when the contract ends).

---

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

### Composite Classes (5)

Every composite implicitly includes `tag` plus a `#kind:{composite_name}` identity tag — even composites that do not hold `data_object`. This is what makes composites distinguishable from each other at permission check time.

| Class | Explicit fundamentals | Implicit (in every composite) | Notes |
|-------|------------------------|-------------------------------|-------|
| `external_service_object` | `network_endpoint` + `secret/credential` | `tag` + `#kind:external_service` | Covers MCP servers, webhooks, email APIs, Slack, etc. No `data_object` — pure operational composite. Replaces the earlier `communication_object`. |
| `model/runtime_object` | `network_endpoint` + `secret/credential` + `economic_resource` | `tag` + `#kind:model_runtime` | LLM endpoints are external services that also consume token budget. No `data_object`. |
| `control_plane_object` | `data_object` + `identity_principal` | `tag` + `#kind:control_plane` | Managing the policy store means mutating data about principals. |
| `memory_object` | `data_object` | `tag` + `#kind:memory` + {memory tag vocabulary + memory lifecycle rules} | Memory-specific lifecycle rules stay on the composite. See [Memory as a Resource Class](#memory-as-a-resource-class). |
| `session_object` | `data_object` | `tag` + `#kind:session` + {session tag vocabulary + frozen-at-creation + Multi-Scope resolution} | Session-specific lifecycle and resolution rules stay on the composite. See [Sessions as a Tagged Resource](#sessions-as-a-tagged-resource). |

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

## Standard Action Vocabulary

**Reusable** — the same action word means the same thing across the fundamental resource classes where it applies. **Not universal** — each action category applies only to a subset of fundamentals. The matrix below shows which action categories apply to which fundamentals. Composite classes inherit actions by expansion.

| Category | Actions |
|----------|---------|
| **Discovery** | discover, list, inspect |
| **Data** | read, copy, export |
| **Mutation** | create, modify, append, delete |
| **Execution** | execute, invoke, send |
| **Connection** | connect, bind, listen |
| **Authority** | delegate, approve, escalate |
| **Memory** | store, retain, recall |
| **Configuration** | configure, install, enable, disable |
| **Economic** | spend, reserve, exceed |
| **Observability** | observe, log, attest |

### Action × Fundamental Applicability Matrix

This matrix keys on the 9 fundamentals only. Composites inherit their applicable actions by expansion (an action that applies to any of a composite's fundamentals applies to the composite).

| Fundamental | Discovery | Data | Mutation | Execution | Connection | Authority | Memory | Configuration | Economic | Observability |
|---|---|---|---|---|---|---|---|---|---|---|
| `filesystem_object` | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | ✓ |
| `process_exec_object` | ✓ | — | — | ✓ | — | ✓ | — | ✓ | — | ✓ |
| `network_endpoint` | ✓ | ✓ | — | — | ✓ | ✓ | — | — | — | ✓ |
| `secret/credential` | ✓ | ✓ | — | — | — | ✓ | — | — | — | ✓ |
| `data_object` | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | ✓ |
| `tag` | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | — | — | ✓ |
| `identity_principal` | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | — | ✓ |
| `economic_resource` | ✓ | ✓ | — | — | — | ✓ | — | — | ✓ | ✓ |
| `time/compute_resource` | ✓ | ✓ | — | — | — | ✓ | — | ✓ | ✓ | ✓ |

**Composite inheritance:** applicable actions for a composite are the union of its constituents' actions. Example: `memory_object = data_object + tag` — memory supports `recall`/`store`/`retain` (from `tag`) and also `read`/`list`/`inspect`/`create`/`modify`/`delete` (from `data_object`).

**Notes:**
- ✓ means the action category has at least one valid action for that fundamental
- Discovery, Authority, and Observability apply universally (every fundamental has `list`/`inspect`, `delegate`, and `observe`/`log`/`attest`)
- The Memory category (`store`/`recall`/`retain`) is unique to `tag` — this is what makes `tag` a fundamental in its own right
- The Connection category applies only to `network_endpoint`
- Mutation is denied on `secret/credential` because mutating a secret isn't a local operation — credential rotation happens via a separate control-plane flow

---

## Constraints

Each permission carries condition slots that scope or restrict the grant beyond the selector.

### Constraint × Fundamental Applicability Matrix

| Constraint | filesystem | process_exec | network | secret | data | tag | identity | economic | time/compute |
|---|---|---|---|---|---|---|---|---|---|
| `path_prefix` | ✓ | — | — | — | — | — | — | — | — |
| `command_pattern` | — | ✓ | — | — | — | — | — | — | — |
| `domain_allowlist` | — | — | ✓ | — | — | — | — | — | — |
| `tag_predicate` | — | — | — | — | — | ✓ | — | — | — |
| `max_size_bytes` | ✓ | — | ✓ | — | ✓ | — | — | — | — |
| `max_spend` | — | — | — | — | — | — | — | ✓ | ✓ |
| `sandbox_requirement` | — | ✓ | — | — | — | — | — | — | — |
| `output_channel` | — | — | ✓ | — | — | — | — | — | — |
| `timeout_secs` | — | ✓ | ✓ | — | — | — | — | — | ✓ |

**Universal constraints** apply to every fundamental (omitted from the matrix for readability):

| Constraint | Purpose |
|---|---|
| `time_window` | Only during specified hours/dates |
| `approval_requirement` | `auto`, `human_required`, `subordinate_required` |
| `non_delegability` | Prevent grant from flowing through delegation |
| `purpose` | Audit label, not enforced but recorded |

**Critical note on `tag_predicate`:** Tag predicates are a first-class constraint type, owned by the `tag` fundamental. They are the **selector grammar** for `tag` — see the "Selector vs Constraints" subsection below for the precise relationship.

**Every composite implicitly supports `tag_predicate`.** Because every composite pulls in `tag` (to carry its `#kind:` identity tag), every composite's applicable-constraints set includes `tag_predicate` by default — even composites without `data_object`. A grant targeting `external_service_object` can use a tag-predicate selector like `tags contains provider:github` and it will work, because `external_service_object` includes `tag`.

**Composite inheritance:** a composite's applicable constraints are the union of its constituents' constraints. Example: `external_service_object = network_endpoint + secret/credential + (implicit) tag` supports `domain_allowlist` (from network), `tag_predicate` (from the implicit `tag`), and all universal constraints.

### Selector vs Constraints — Formal Distinction

A Permission Grant has two fields that are often confused. This subsection pins down the difference.

| Field | Question it answers | Examples |
|-------|---------------------|----------|
| `resource.selector` | **Which instances of the resource** does this grant cover? | path glob, command regex, tag predicate, domain pattern |
| `constraints` | **Under what additional conditions** does the grant apply? | timeout, sandbox, time window, approval requirement, max spend |

**Key insight:** selector grammar is **resource-class-specific** (filesystem uses path globs, process uses command regex, tag uses tag predicates). Constraints are mostly **resource-class-specific** too, but a subset (`time_window`, `approval_requirement`, `non_delegability`, `purpose`) are **universal** and apply to every fundamental.

**Why they're separate:**
- A **selector** narrows the *target* — "you can read files in /workspace/project-a"
- A **constraint** adds *runtime gating* — "but only during business hours, with sandbox enabled, requiring human approval for writes over 1MB"
- Combining them into one predicate would lose the "targets a specific instance set" vs "gates the runtime conditions" distinction, and would make tag predicates indistinguishable from time windows.

**Tag predicates are selectors, not constraints** — when you write `tags intersects {project:alpha, #public}`, you're saying "this grant targets memories whose tags overlap with this set." That's identifying instances. The fact that the grant might *also* have a `time_window: business_hours` constraint is orthogonal: the constraint applies *after* the selector picks the candidate set.

**In summary:**
- Selector = WHICH instances. Resource-class-specific grammar.
- Constraints = WHEN/HOW the grant applies. Mostly resource-class-specific, with four universal ones.
- Tag predicates belong to the selector side, specifically for the `tag` fundamental (and any composite that includes it, which is every composite).

---

## Per-Resource-Class Reference

This section is the canonical reference for each resource class — fundamental and composite. Each entry lists its selector grammar, applicable actions, applicable constraints, common overlaps, and a worked example grant.

### Fundamentals

#### `filesystem_object`

**Flavor:** Physical/operational
**What it covers:** Files, directories, repos, temp paths on disk
**Examples:** Agent workspaces, skill files, config files, logs, env files

**Selector grammar:** Path glob (e.g., `/workspace/project-a/**`, `*.md`, `!.git/**`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Mutation: `create`, `modify`, `append`, `delete`
- Authority: `delegate`, `approve`, `escalate`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `path_prefix`, `max_size_bytes` + universal constraints

**Common entity overlaps:** `.env` files overlap with `secret/credential`; memory/session files overlap with their respective composites

**Worked example grant:**
```yaml
permission:
  # subject = agent:claude-coder-7
  action: [read, modify]
  resource:
    type: filesystem_object
    selector: "/workspace/project-a/**"
  constraints:
    max_size_bytes: 10485760   # 10MB
  provenance: config:project-a.toml#workspace-policy
  delegable: true
  approval_mode: auto
```

#### `process_exec_object`

**Flavor:** Physical/operational
**What it covers:** Spawning processes, running binaries/scripts, launching containers
**Examples:** BashTool, script interpreters, sandbox runners

**Selector grammar:** Command pattern (regex, e.g., `^cargo (build|test|fmt)$`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Execution: `execute`, `invoke`, `send`
- Authority: `delegate`, `approve`, `escalate`
- Configuration: `configure`, `install`, `enable`, `disable`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `command_pattern`, `sandbox_requirement`, `timeout_secs` + universal constraints

**Common entity overlaps:** A process can transitively reach `filesystem_object`, `network_endpoint`, `secret/credential`, and `time/compute_resource` — these are declared in the tool manifest's `transitive` field

**Worked example grant:**
```yaml
permission:
  # subject = agent:claude-coder-7
  action: [execute]
  resource:
    type: process_exec_object
    selector: "command matches /^cargo (build|test|fmt)$/"
  constraints:
    timeout_secs: 60
    sandbox_requirement: true
  provenance: agent:project-lead-1@2026-04-01
  delegable: false
```

#### `network_endpoint`

**Flavor:** Physical/operational
**What it covers:** Outbound network traffic to hosts, ports, APIs
**Examples:** Provider base_urls, MCP endpoints, webhook URLs

**Selector grammar:** Domain pattern (e.g., `api.anthropic.com`, `*.openrouter.ai`, `https://example.com/**`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Connection: `connect`, `bind`, `listen`
- Authority: `delegate`, `approve`, `escalate`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `domain_allowlist`, `max_size_bytes`, `output_channel`, `timeout_secs` + universal constraints

**Common entity overlaps:** An authenticated network call overlaps with `secret/credential` (the auth token is a separate fundamental)

**Worked example grant:**
```yaml
permission:
  # subject = agent:api-client-3
  action: [read, connect]
  resource:
    type: network_endpoint
    selector: "domain in {api.anthropic.com, *.openrouter.ai}"
  constraints:
    timeout_secs: 30
    max_size_bytes: 10485760
  provenance: config:org-default.toml#network-policy
  delegable: true
```

#### `secret/credential`

**Flavor:** Physical/operational
**What it covers:** API keys, tokens, certificates, SSH keys, env vars holding secrets

**Selector grammar:** Named credential (e.g., `github_token`, `openai_api_key`) or path glob on credential store

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Authority: `delegate`, `approve`, `escalate`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** Universal constraints only (no resource-specific constraints — secrets are atomic)

**Common entity overlaps:** `.env` files (filesystem + secret), MCP auth (external_service composite), LLM API keys (model/runtime composite)

**Worked example grant:**
```yaml
permission:
  # subject = agent:api-client-3
  action: [read]
  resource:
    type: secret/credential
    selector: "name = openai_api_key"
  constraints:
    approval_mode: human_recommended
  provenance: config:org-default.toml#secrets-policy
  delegable: false
  audit_class: logged
```

#### `data_object`

**Flavor:** Data access
**What it covers:** Generic structured data — graph nodes, tables, vectors, arbitrary documents that aren't tagged composites
**Examples:** Message nodes, tabular data, vector stores

**Selector grammar:** Graph query (e.g., `node_type = Message`), table filter, vector similarity

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Mutation: `create`, `modify`, `append`, `delete`
- Authority: `delegate`, `approve`, `escalate`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_size_bytes` + universal constraints

**Common entity overlaps:** Memory and session composites include `data_object`; `control_plane_object` includes `data_object + identity_principal`

**Worked example grant:**
```yaml
permission:
  # subject = agent:analytics-2
  action: [read, export]
  resource:
    type: data_object
    selector: "node_type = Message AND project:alpha"
  provenance: config:project-alpha.toml
  delegable: false
```

#### `tag`

**Flavor:** Data access + composite categorization (operational)
**What it covers:** Tag-predicate access grammar — the three operators `contains`, `intersects`, `subset_of`; the `namespace:value` format; the `#kind:` identity tag machinery that makes composites distinguishable

**Selector grammar:** Tag predicate (e.g., `tags contains project:alpha`, `tags intersects {agent:X, #public}`, `tags subset_of {org:acme, project:beta}`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Mutation: `create`, `modify`, `append`, `delete`
- Authority: `delegate`, `approve`, `escalate`
- Memory: `store`, `retain`, `recall`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `tag_predicate` (the selector form is also the primary constraint-shaped expression) + universal constraints

**Dual role:** `tag` serves two functions:
1. **As a standalone fundamental** for composites that hold data (memory, session)
2. **As the structural substrate for every composite** (via the `#kind:` tag machinery)

**Common entity overlaps:** Every composite implicitly includes `tag` for its `#kind:` identity — even composites without `data_object`

**Worked example grant:**
```yaml
permission:
  # subject = agent:X
  action: [recall]
  resource:
    type: tag
    selector: "tags intersects { agent:X, project:current_project(X), #public }"
  provenance: system:agent-context
  delegable: false
  revocation_scope: dynamic
```

#### `identity_principal`

**Flavor:** Identity
**What it covers:** Agents, users, roles, sessions — the "who" axis of the system

**Selector grammar:** Principal match (e.g., `agent:claude-*`, `role:lead@project:alpha`, `org_member(acme)`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Mutation: `create`, `modify`, `append`, `delete`
- Authority: `delegate`, `approve`, `escalate`
- Configuration: `configure`, `install`, `enable`, `disable`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** Universal constraints only

**Common entity overlaps:** Delegation (spawning a sub-agent) involves `identity_principal` + the sub-agent's transitive reach; `control_plane_object` includes `data_object + identity_principal`

**Worked example grant:**
```yaml
permission:
  # subject = agent:orchestrator-1
  action: [delegate, create]
  resource:
    type: identity_principal
    selector: "principal matches agent:worker-*"
  constraints:
    max_spend: 5000    # per delegation
  provenance: agent:admin-bot
  delegable: false
```

#### `economic_resource`

**Flavor:** Physical/operational
**What it covers:** Token budgets, spend, quotas, rate limits
**Examples:** Per-task token budgets, per-agent spend caps, per-org rate limits

**Selector grammar:** Budget reference (e.g., `budget:task-4581`, `quota:agent:claude-coder-7`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Authority: `delegate`, `approve`, `escalate`
- Economic: `spend`, `reserve`, `exceed`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_spend` + universal constraints

**Common entity overlaps:** `model/runtime_object` includes `economic_resource` because LLM calls cost tokens

**Worked example grant:**
```yaml
permission:
  # subject = agent:contract-agent-7
  action: [spend, reserve]
  resource:
    type: economic_resource
    selector: "budget:task-4581"
  constraints:
    max_spend: 10000   # tokens
  provenance: contract:bid-4581
  delegable: false
  revocation_scope: end_of_task
```

#### `time/compute_resource`

**Flavor:** Physical/operational
**What it covers:** CPU time, wall clock, memory, concurrency slots

**Selector grammar:** Resource scope (e.g., `scope:per_agent`, `scope:per_session`)

**Applicable actions:**
- Discovery: `discover`, `list`, `inspect`
- Data: `read`, `copy`, `export`
- Authority: `delegate`, `approve`, `escalate`
- Configuration: `configure`, `install`, `enable`, `disable`
- Economic: `spend`, `reserve`, `exceed`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_spend`, `timeout_secs` + universal constraints

**Common entity overlaps:** Process execution typically reaches `time/compute_resource` transitively

**Worked example grant:**
```yaml
permission:
  # subject = agent:worker-5
  action: [spend, reserve]
  resource:
    type: time/compute_resource
    selector: "scope:per_session"
  constraints:
    max_spend: 3600   # seconds
    timeout_secs: 300
  provenance: config:org-default.toml#compute-limits
```

### Composites

#### `external_service_object`

**Expansion:** `network_endpoint` + `secret/credential` + (implicit) `tag` + `#kind:external_service`
**What it covers:** Authenticated external services — MCP servers, webhooks, Slack, email APIs, REST APIs
**Examples:** GitHub MCP, Slack webhook, Stripe API
**No `data_object`** — pure operational composite

**Inherited actions:** Discovery + Data + Connection + Authority + Observability (from constituents) + tag-based selectors

**Inherited constraints:** `domain_allowlist`, `max_size_bytes`, `output_channel`, `timeout_secs`, `tag_predicate` (from implicit `tag`) + universal constraints

**Composite-specific rules:** Every instance carries `#kind:external_service`. Tool manifests targeting this composite must declare `#kind: external_service` explicitly (enforced at publish time).

**When to use the composite vs fundamentals directly:** use the composite when the tool is clearly an "external service call" (MCP, webhook, REST API). Use the fundamentals directly if you want to make the network + auth requirements explicit for a tool that just happens to do both (e.g., a network diagnostic tool).

**Worked example grant:**
```yaml
permission:
  # subject = agent:api-client-3
  action: [connect, read]
  resource:
    type: external_service_object
    selector: "tags contains provider:github"
  provenance: agent:human-sarah@2026-04-01
  delegable: false
```

#### `model/runtime_object`

**Expansion:** `network_endpoint` + `secret/credential` + `economic_resource` + (implicit) `tag` + `#kind:model_runtime`
**What it covers:** LLM API calls — the endpoint, the auth token, and the token budget all compose into one unit
**Examples:** Anthropic API, OpenAI API, OpenRouter calls

**Inherited actions:** Discovery + Data + Authority + Configuration + Economic + Observability (includes `execute`/`invoke` from `process_exec_object`-like patterns? No — model calls are network traffic, so just data + connection)

**Inherited constraints:** `domain_allowlist`, `timeout_secs`, `max_spend`, `max_size_bytes`, `tag_predicate` (from implicit `tag`) + universal

**Composite-specific rules:** Every instance carries `#kind:model_runtime`. Manifests must declare `#kind: model_runtime` explicitly at publish time.

**Worked example grant:**
```yaml
permission:
  # subject = agent:worker-5
  action: [invoke, spend]
  resource:
    type: model/runtime_object
    selector: "tags contains provider:anthropic"
  constraints:
    max_spend: 5000    # tokens per call
    timeout_secs: 120
  provenance: contract:bid-4581
  delegable: false
```

#### `control_plane_object`

**Expansion:** `data_object` + `identity_principal` + (implicit) `tag` + `#kind:control_plane`
**What it covers:** Managing the permission/policy/audit layer itself
**Examples:** Reading the tool registry, writing audit logs, modifying the policy store

**Inherited actions:** Discovery + Data + Mutation + Authority + Configuration + Observability

**Inherited constraints:** `max_size_bytes` (from data_object), `tag_predicate` (from implicit `tag`) + universal

**Composite-specific rules:** Every instance carries `#kind:control_plane`. Typically requires `approval_mode: human_required` because modifying the control plane is a sensitive operation.

**Worked example grant:**
```yaml
permission:
  # subject = agent:admin-bot
  action: [read, modify]
  resource:
    type: control_plane_object
    selector: "tags contains scope:tool_registry"
  constraints:
    approval_mode: human_required
  provenance: system:bootstrap
  delegable: false
  audit_class: alerted
```

#### `memory_object`

**Expansion:** `data_object` + (implicit) `tag` + `#kind:memory` + {memory tag vocabulary + memory lifecycle rules}
**What it covers:** Tagged knowledge entries that persist across sessions

**Full details:** see [Memory as a Resource Class](#memory-as-a-resource-class) below for tag vocabulary, lifecycle, default grants, and supervisor extraction.

**Composite-specific rules:** Every Memory node carries `#kind:memory`. Agent chooses tags at creation; tags are mutable within the memory's lifetime. Multi-Scope resolution does NOT apply to memory (that's session-specific).

**Implicit `#kind:memory` tag** is added to every Memory node at creation time. Grants targeting `memory_object` carry an implicit `tags contains #kind:memory` selector refinement. See [Composite Identity Tags](#composite-identity-tags-kind) for the explanation of why implicit kind filters exist.

#### `session_object`

**Expansion:** `data_object` + (implicit) `tag` + `#kind:session` + {session tag vocabulary + frozen-at-creation + Multi-Scope resolution}
**What it covers:** Tagged execution records — sessions, loops, turns, messages (messages inherit session permissions)

**Full details:** see [Sessions as a Tagged Resource](#sessions-as-a-tagged-resource) below for tag vocabulary, lifecycle, Authority Templates, default grants, and Multi-Scope Session Access.

**Composite-specific rules:** Every Session node carries `#kind:session`. Tags are assigned by the system at creation and frozen (except `#archived`/`#active`). Multi-Scope Session Access resolution applies to reads.

**Implicit `#kind:session` tag** is added to every Session node at creation time. Grants targeting `session_object` carry an implicit `tags contains #kind:session` selector refinement. See [Composite Identity Tags](#composite-identity-tags-kind) for the explanation.

---

## Two Shapes: Tool Authority Manifest vs Permission Grant

The doc has been describing two related but distinct shapes. Here is how they differ:

| Shape | What It Describes | Carries Subject? | Carries Provenance? | Where It Lives |
|-------|-------------------|------------------|---------------------|----------------|
| **Tool Authority Manifest** | A tool's REQUIREMENTS — "to use me, you need these capabilities" | No (a tool isn't owned by anyone) | No (it's a static tool spec) | Shipped with the tool definition |
| **Permission Grant** | A capability HELD by a specific principal | Yes (implicit in the edge source) | Yes (audit trail) | Stored as a graph node, attached via `HAS_PERMISSION` |

The runtime reconciles these via a **Permission Check** (see further below).

### Tool Authority Manifest (Tool Requirements)

**Design rule:** Every tool must ship a machine-readable authority manifest declaring what it does, so the system can check whether a caller has the matching grants.

A manifest declares:
- Resource classes touched
- Actions performed
- Transitive resources consumed (e.g., `bash` can reach `network endpoint` transitively)
- Delegation behavior
- Approval defaults
- Constraints that callers must satisfy

> A manifest is **descriptive of the tool**, not prescriptive of any user. It says "I do X" — it does NOT say "Bob is allowed to call me."

Example for `write_file`:
```yaml
tool: write_file
manifest:
  actions: [create, modify]
  resource: filesystem_object
  constraints:
    path_prefix: required     # caller must scope the path
    max_size_bytes: 1048576   # 1MB default limit
  transitive: []              # no transitive access
  delegable: true
  approval: auto              # no human approval needed
```

Example for `bash`:
```yaml
tool: bash
manifest:
  actions: [execute]
  resource: process_exec_object
  constraints:
    command_pattern: required
    sandbox: recommended
    timeout_secs: 120
  transitive:
    - filesystem_object       # can read/write files
    - network_endpoint        # can make HTTP calls
    - secret/credential       # can access env vars
  delegable: false            # too powerful to delegate
  approval: human_recommended
```

#### The Transitive-Grant Match Rule

The `transitive` field is **load-bearing**, not merely documentary. When an agent invokes a tool, the Permission Check must pass for **every fundamental** the manifest implies — derived from its primary class, its transitive list, and the target entity's classification. Missing a grant for any fundamental is a denial.

**The rule, stated plainly:**

> A tool manifest must declare every fundamental the tool touches and every composite `#kind:` value the tool operates on. These declarations are validated at **tool publish time** by the tool registry's manifest validator; inconsistent manifests are rejected and the tool cannot be registered. Once a tool is published, the runtime trusts the manifest and performs Permission Checks against its declarations.
>
> **At runtime** (when an agent invokes an already-published tool), the runtime derives the full set of required **fundamentals** by (a) expanding the manifest's primary class to fundamentals (including the implicit `tag` on any composite), (b) expanding each class in the transitive list to fundamentals, and (c) classifying the target entity (if any) to fundamentals. The Permission Check must pass for **every fundamental in this union**. A missing grant for any fundamental is a **runtime denial** (the agent lacks authorization — not a manifest problem).
>
> **At publish time**, the validator rejects a manifest that (a) declares a composite but omits its `#kind:` value, (b) declares a `#kind:` without the matching fundamentals, or (c) declares fundamentals inconsistent with the composites it names. The error message names the specific missing declaration so the tool creator can fix it and resubmit.
>
> `#kind: *` (blanket) is legal but throws a publish-time warning. The composite label (using `external_service_object` instead of its fundamentals) is optional — a warning at publish time may suggest adding it for readability, but the manifest is accepted either way.

**Worked example:**

An agent invokes `bash` to run `curl https://api.example.com | tee /tmp/response.json`.

The `bash` manifest declares:
- `resource: process_exec_object` (fundamental)
- `transitive: [filesystem_object, network_endpoint, secret/credential]` (all fundamentals)

Runtime derives the required fundamental set: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Runtime runs four parallel Permission Checks:

| Fundamental | Agent grant? | Check result |
|---|---|---|
| `process_exec_object` | ✓ (sandbox execute grant) | ✓ |
| `filesystem_object` | ✓ (workspace read grant) | ✓ |
| `network_endpoint` | ✗ (no network grant) | ✗ |
| `secret/credential` | — (not required for this specific call) | — |

**Result: DENIED** — missing `network_endpoint` is a hard fail, even though the agent has the primary class grant and two of the three transitive classes.

**Worked example with composite shorthand:**

Agent invokes `mcp_github` to read a pull request. `mcp_github` manifest declares `resource: external_service_object` (composite). Runtime expands: `{network_endpoint, secret/credential, tag}` + `#kind:external_service` filter.

If the agent holds a composite grant for `external_service_object`, that grant auto-expands to both fundamentals and satisfies both checks. Alternatively, the agent could hold two separate fundamental grants for `network_endpoint` and `secret/credential` plus a `tag_predicate` selector matching `#kind:external_service`, and both would satisfy the checks. **Both forms are semantically identical** because the runtime normalizes everything to fundamentals.

**The security implication:**

A manifest that under-declares its fundamental reach is a **security bug**, caught at publish time by the manifest validator. If `bash` forgot to declare `secret/credential` in its transitive set, the validator would reject the publish because the declared behavior of `bash` is inconsistent with the declared fundamental set. The validator is the last-line safety net; under-declaration cannot reach production.

**The documentation implication (warning-only rule):**

A manifest that declares the right fundamentals but forgets to use a composite label (e.g., declares `network_endpoint + secret/credential` directly instead of `external_service_object`) is semantically correct. The validator accepts it. The linter may warn that the pattern matches a known composite and suggest the composite form for readability — but this is cosmetic only, never load-bearing.

**Relationship to the entity overlap rule:**

Both the entity-overlap rule (Edit 1c) and the transitive rule converge on the same runtime semantic: **a set of fundamentals must all be satisfied**. They differ only in where the fundamentals come from:

- **Entity overlap**: an entity is classified to multiple fundamentals via `classify_to_fundamentals(entity)`
- **Manifest transitive**: a tool manifest declares multiple fundamentals via `expand_composites(manifest.resource) ∪ expand_composites(manifest.transitive)`
- **Both compose**: a `bash` call operating on `.env` has required fundamentals from both sources. Stage 1 of `check_tool_invocation` unions them.

See the [`check_tool_invocation` pseudocode](#how-the-two-mechanisms-combine) for the full implementation.

### Permission Grant (5-Tuple, Held by a Subject)

A Permission Grant fills in all five components of the canonical shape. It is attached to a subject via a `HAS_PERMISSION` edge.

Example: `claude-coder-7` is allowed to run a narrow set of cargo commands.

```yaml
permission:
  # subject is the source of the HAS_PERMISSION edge → agent:claude-coder-7
  action: [execute]
  resource:
    type: process_exec_object
    selector: "command matches /^cargo (build|test|fmt)$/"
  constraints:
    timeout_secs: 60
    sandbox: true
  provenance: agent:project-lead-1@2026-04-01
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: end_of_session
```

Example: a project-scoped grant that propagates to all project members.

```yaml
permission:
  # subject is the source of the HAS_PERMISSION edge → project:website-redesign
  action: [read, modify]
  resource:
    type: filesystem_object
    selector: "/workspace/website-redesign/**"
  constraints:
    max_size_bytes: 10485760  # 10MB
  provenance: config:org-default.toml#project-workspace-policy
  delegable: true
  approval_mode: auto
  audit_class: silent
  revocation_scope: manual
```

### Permission Check (Runtime Reconciliation)

When an agent invokes a tool, the runtime executes a **Permission Check** that combines:
- The tool's **Authority Manifest** (what the tool requires)
- The agent's **Permission Grants** (what the agent holds)
- The **resolution hierarchy** (org → project → agent — see further below)

#### Worked Example

`claude-coder-7` calls `bash` with the command `cargo build`.

The runtime asks five questions:

| Step | Check | Manifest Side | Grant Side | Result |
|------|-------|---------------|------------|--------|
| 1 | Does the agent hold a grant for the manifest's action? | `actions: [execute]` | grant has `action: [execute]` | ✓ |
| 2 | Does the grant's resource type match the manifest's resource? | `resource: process_exec_object` | `resource.type: process_exec_object` | ✓ |
| 3 | Does the actual call satisfy the grant's resource selector? | call: `cargo build` | selector: `^cargo (build|test|fmt)$` | ✓ |
| 4 | Are the manifest's required constraints satisfied by the grant? | `command_pattern: required`, `sandbox: recommended`, `timeout_secs: 120` | grant provides selector (satisfies pattern), `sandbox: true`, `timeout_secs: 60` (≤ 120) | ✓ |
| 5 | Do org and project ceilings allow this? | — | org allows `process_exec_object`, project allows `bash` | ✓ |
| → | **Decision** | | | **Allowed** |

If the same agent tries to run `rm -rf /`:

| Step | Check | Result |
|------|-------|--------|
| 3 | Does `rm -rf /` match the grant's selector `^cargo (build|test|fmt)$`? | ✗ |
| → | **Decision** | **Denied** (with audit log entry citing the failed selector match) |

#### Mental Model

A manifest and a grant are **two halves of a key**:
- The manifest says *"this is what I need."*
- The grant says *"this is what you can do."*
- A permission check is **set containment**: every requirement on the manifest side must be covered by some grant on the grant side, AND every constraint must be jointly satisfied.

If you remember nothing else: **Manifests describe tools. Grants describe subjects. Permission Checks combine them at runtime.**

---

## Permission Resolution Hierarchy

There are **two distinct mechanisms** at play when resolving a Permission Check, and the doc has historically conflated them. Separating them makes the rules clearer.

### Mechanism 1: Ceiling Enforcement (top-down upper bound)

Organizations and Projects can attach Permission grants to themselves (via `HAS_PERMISSION` edges). These act as **upper bounds** on what any subject within their scope can do. This mechanism enforces "no project within Acme can exceed Acme's policies" and similar containment rules.

```
Organization (highest ceiling)
    │ caps ↓
Project (capped by its owning orgs)
    │ caps ↓
Agent (capped by its project + org)
```

**Rules:**
- **Org caps project:** If an org restricts `network_endpoint` access, no project within it can grant it back to its members.
- **Project caps agent:** If a project restricts `bash` tool, no agent within it can use it via that project's grants.
- **Agent grants are most specific:** Within the bounds set by org and project, the agent's own grants determine fine-grained behavior.

**Delegation:** When Agent A delegates to Agent B, B inherits A's permission *ceiling* (never more than A has), further narrowed by B's own grants.

### Mechanism 2: Scope Resolution (specific-first selection)

When a session has multiple `org:` tags (a joint project) — or, in principle, multiple `project:` tags — the runtime needs to pick **which scope's grants to apply** for a given reader. Resolution cascades from most-specific to most-general, with `base_*` as the tie-breaker:

```
1. Project-level resolution
     - Reader is a member of one of the session's projects?  → use that project's scope
     - Reader is a member of multiple of the session's projects?  → base_project wins
     - Reader is a member of none?  → fall through

2. Org-level resolution
     - Reader's base_org matches one of the session's orgs?  → use that org's scope
     - Reader is a member of multiple of the session's orgs?  → base_org wins
     - Reader is a member of none?  → fall through

3. Intersection fallback (outsider)
     - Apply the intersection of all the session's scope ceilings
```

**Cascade rationale:** Resolution always tries the *narrowest* scope the reader has a legitimate claim to. Outsiders who hold no claim at any level face the strictest treatment (intersection-of-everything), eliminating loopholes where someone could "shop" for a permissive scope.

**Where this matters:** Scope resolution only kicks in when a session has multiple scope tags at some level. The common single-org single-project case bypasses Mechanism 2 entirely. See **[Multi-Scope Session Access](#multi-scope-session-access)** under Sessions for the full rule, worked examples, and the schema constraint that prevents simultaneously-multi-project AND multi-org sessions.

### How the Two Mechanisms Combine

Both mechanisms apply on every Permission Check:

```
allowed = scope_resolution_picks_a_grant_that_matches(reader, session)
        AND ceiling_enforcement_does_not_block(reader, session, picked_scope)
        AND has_matching_subject_grant(reader, session)
```

Scope resolution picks *which* scope's grants apply. Ceiling enforcement bounds those grants from above. The agent's own grants must match within those bounds. All three must hold for the read to succeed.

#### A Possible Refinement

The high-level pseudocode above is intentionally abstract — it states the three required conditions without committing to where each grant lookup happens. A more concrete formulation, useful when reasoning about implementation, makes the two-tier resource ontology and the `#kind:` tag filtering explicit. The concrete version has two stages: derive the required **fundamentals** (from the manifest and entity classification), then run a per-fundamental permission check.

```
// A resolved grant carries its expanded fundamentals AND its effective selector
// (which may include an implicit #kind: refinement if the grant targeted a composite).
struct ResolvedGrant {
    fundamentals: HashSet<Fundamental>,
    effective_selector: Selector,  // explicit selector AND implicit #kind: filter
    constraints: Constraints,
    approval_mode: ApprovalMode,
    // ... other standard fields
}

fn resolve_grant(g: &Grant) -> ResolvedGrant {
    // If the grant targets a composite, expand it to fundamentals AND
    // add the implicit #kind: tag predicate to the effective selector.
    let (fundamentals, extra_selector) = match &g.resource.target {
        ResourceTarget::Fundamental(f) => (HashSet::from([*f]), None),
        ResourceTarget::Composite(c) => {
            let fs = expand_composites(c);   // always includes `tag` implicitly
            let kind_filter = Selector::TagPredicate(
                format!("tags contains #kind:{}", c.name())
            );
            (fs, Some(kind_filter))
        }
    };

    let effective_selector = match extra_selector {
        Some(k) => Selector::And(vec![g.resource.selector.clone(), k]),
        None => g.resource.selector.clone(),
    };

    ResolvedGrant { fundamentals, effective_selector, /* ... */ }
}

fn check_tool_invocation(reader: Agent, tool: Tool, target: Option<Entity>) -> bool {
    // STAGE 1: Derive the required fundamental set.
    //
    // Sources:
    //   (a) The tool manifest's primary class and transitive list.
    //       Composites in the manifest are expanded to their constituents
    //       (including the implicit `tag` on any composite).
    //   (b) The target entity's classification (if the action touches a
    //       specific entity). An entity like `.env` maps to multiple
    //       fundamentals (filesystem_object + secret/credential).
    let mut required: HashSet<Fundamental> = HashSet::new();
    required.extend(expand_composites(tool.manifest.resource));
    required.extend(expand_composites(tool.manifest.transitive));
    if let Some(e) = target {
        required.extend(classify_to_fundamentals(e));
    }

    // STAGE 2: For every required fundamental, run the per-fundamental
    // Permission Check. ALL must pass for the invocation to be allowed.
    required.iter().all(|fundamental| {
        check_action_for_fundamental(reader, tool.manifest.actions, target, fundamental)
    })
}

fn check_action_for_fundamental(
    reader: Agent,
    actions: Vec<Action>,
    target: Option<Entity>,
    fundamental: Fundamental,
) -> bool {
    // Single-fundamental check: scope resolution, candidate grant
    // collection (including composite expansion + #kind: filter), selector
    // matching, ceiling filtering, approval gates, set non-emptiness.

    // 1. Scope resolution — pick which scope's grants apply.
    let scope = resolve_scope(reader, target);

    // 2. Gather candidate grants the reader holds that cover THIS fundamental.
    //    Grants that declare a composite are auto-expanded via resolve_grant().
    //    A composite grant is a candidate for any fundamental in its expansion,
    //    but its effective_selector includes the #kind: filter — so memory-specific
    //    grants will not match session entities even though both grants target
    //    the same fundamentals (data_object + tag).
    let candidates: Vec<ResolvedGrant> = reader.grants()
        .iter()
        .map(resolve_grant)
        .filter(|g| g.fundamentals.contains(&fundamental))
        .filter(|g| g.actions_cover(&actions))
        .collect();

    // 3. Filter by effective selector (includes any implicit #kind: refinement).
    let matching = candidates.iter().filter(|g| g.effective_selector.matches(target));

    // 4. Ceiling enforcement — drop any grant that exceeds an upper bound.
    let allowed_by_ceiling = matching.filter(|g| {
        ceiling_for_scope(scope).bounds(g)
            && all_org_caps_for_target(target).bounds(g)
    });

    // 5. Approval gates — handle subordinate_required, human_required, etc.
    let final_grants = allowed_by_ceiling
        .filter(|g| approval_satisfied(g, reader, target));

    // 6. Decision: non-empty grant set.
    !final_grants.is_empty()
}
```

This concrete version makes six things explicit that the abstract version leaves implicit:

1. **Two-stage structure.** Stage 1 derives what's required; Stage 2 checks each requirement. This separates "what does the operation touch" from "does the agent have permissions for it."
2. **Composite expansion happens in Stage 1.** Both on the manifest side (`expand_composites(tool.manifest.resource)`) and on the grant side inside `resolve_grant` (a composite grant auto-expands to its constituent fundamentals plus an implicit `#kind:` selector refinement).
3. **Entity classification happens in Stage 1 too.** `classify_to_fundamentals(entity)` returns every fundamental the entity belongs to — the overlap rule falls out naturally.
4. **Grants come from multiple sources** — defaults, Authority Templates, Template E, and inherited project/org grants — and they all participate in the same candidate pool.
5. **Ceiling enforcement is a filter on the candidate set** — not a separate step that runs after grant selection.
6. **Approval gates run last** — after scope resolution, after grant matching, after ceiling filtering. This is where Consent Policy fits (`subordinate_required` lives in `approval_satisfied`).

**The key role of `resolve_grant()`:** it expands composite grants by adding an implicit `#kind:` tag predicate to the effective selector. This is how memory-specific grants are prevented from matching session entities even though both share `data_object + tag` fundamentals. The entity's `#kind:` tag is what makes the match possible; the grant's `#kind:` filter is what narrows grant applicability.

**Composites that are missing from the manifest (but whose fundamentals are all present) do NOT cause a hard deny at runtime** — the manifest was already validated at publish time (see "Enforcement Asymmetry"). A missing composite label is a warning at publish time; missing fundamentals or missing `#kind:` is a publish-time rejection. The runtime operates on already-validated manifests and focuses on the agent's authorization.

The abstract version remains the canonical statement of the rule because it's easier to reason about and harder to get wrong. The refinement is a useful pseudocode reference for implementation discussions and edge-case validation.

---

## Permission as a Graph Node

A Permission node stores **four of the five components** of the canonical 5-tuple. The fifth — `subject` — is **not** stored as a property; it is expressed structurally as the source of the `HAS_PERMISSION` edge that points to the node.

```
Permission                       -- the node stores 4 of 5 components
  resource_type: String          -- e.g. "filesystem_object"     [resource]
  resource_selector: String      -- e.g. "/workspace/project-a/**"  [resource]
  action: Vec<String>            -- e.g. ["read", "modify"]      [action]
  constraints: Json              -- condition slots              [constraints]
  delegable: bool                -- can this be passed to sub-agents
  approval_mode: String          -- "auto", "human_required", "human_recommended"
  audit_class: String            -- "silent", "logged", "alerted"
  provenance: String             -- e.g. "system", "agent:sarah", "config:..."  [provenance]
  revocation_scope: String       -- "immediate", "end_of_session", "manual"

  -- subject is NOT a field — it is the source of the HAS_PERMISSION edge
```

**Edges (subject is the source — this is where subject lives):**

| Edge | Subject Type | Meaning |
|------|--------------|---------|
| `Agent ──HAS_PERMISSION──▶ Permission` | An agent | Agent-specific grant |
| `Project ──HAS_PERMISSION──▶ Permission` | A project | Project-scoped grant; propagates to all member agents |
| `Organization ──HAS_PERMISSION──▶ Permission` | An organization | Org-level ceiling; cannot be exceeded by anything inside the org |
| `Role ──HAS_PERMISSION──▶ Permission` | A role | Role-based grant; held by anyone occupying the role |
| `Agent ──GRANTS_PERMISSION──▶ Permission` | (provenance edge, not a holding edge) | Records which agent created this grant — feeds the `provenance` audit trail |

> **Why two edge types involving Agent?** `HAS_PERMISSION` is "the agent HOLDS this capability." `GRANTS_PERMISSION` is "the agent CREATED this capability." The same agent can do both — and a separate agent can be the grantor of a grant held by yet another agent. Provenance is the audit trail; subject is the holder.

### Why subject is structural, not a field

A single Permission node can be referenced by multiple `HAS_PERMISSION` edges from different subjects. Storing subject as a property would force one Permission node per subject, even when the capability shape is identical. Modeling subject as the edge source enables capability *templates* — define the Permission once, attach it to N subjects.

For example, an org-default "read project workspace" grant can be defined once and attached to every project in the org via N `HAS_PERMISSION` edges from the projects to the same Permission node. Provenance still tracks who CREATED the template (`config:org-default.toml`), and the resolution hierarchy still applies.

---

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
permission:
  # subject = agent:X (the source of the HAS_PERMISSION edge)
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
permission:
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
permission:
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
permission:
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
permission:
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
permission:
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

When a relationship is formed in the graph (e.g., a `HAS_LEAD` edge is created, or a `DELEGATES_TO` edge is created), the system optionally auto-issues a Permission grant matching one of the templates below. When the relationship is removed, the grant is revoked (because the grants are tagged with `revocation_scope: revoke_when_edge_removed`).

#### Template A: Project Lead Authority

**Trigger:** A `HAS_LEAD` edge is created from Project P to Agent X.

```yaml
permission:
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
permission:
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
permission:
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
permission:
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
permission:
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
permission:
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

Sessions can have multiple `project:` tags (cross-project work) **or** multiple `org:` tags (joint project), but **not both simultaneously**. This is the only hard schema constraint on session tags. See [Multi-Scope Session Access](#multi-scope-session-access) for the constraint, the resolution rule, and worked examples.

In the common case, a session is born in exactly one project (the agent's `current_project` at creation) and is tagged accordingly. Cross-project work is usually modeled as **multiple sessions**, each tagged with its respective project, which keeps resolution trivial. But the system does not *enforce* single-project sessions — work that legitimately spans projects within the same org may produce a single session with multiple `project:` tags, and the cascading scope resolution rule handles it cleanly.

The forbidden shape is **multi-project AND multi-org on the same session**. When work needs to span multiple orgs *and* multiple projects, the system requires creating a parent project that is itself jointly owned by those orgs — and the session belongs to that parent project. This collapses any would-be multi-project-multi-org session into the joint-project case, which the resolution rule handles natively.

---

## Multi-Scope Session Access

> **Resolves Open Question 2 from earlier drafts.** This section is the canonical home for cross-project and cross-org session resolution. It applies the unified cascading rule from [Permission Resolution Hierarchy → Mechanism 2](#mechanism-2-scope-resolution-specific-first-selection) to the specific case of session reads.

### The Hard Schema Constraint

A Session may have one of these tag shapes:

| Shape | Project tags | Org tags | When it arises |
|-------|--------------|----------|----------------|
| **A** | 1 | 1 | Standard single-org single-project work (the common case) |
| **B** | 1 | 2+ | Joint project — one project owned by multiple orgs |
| **C** | 2+ | 1 | Cross-project work within a single org |
| **D** | 0 | 1 | System session under an org but not in a project |

**Forbidden:** multi-project AND multi-org on the same session (a hypothetical "Shape E"). When work needs to span both axes, the system requires a **joint parent project** that itself is co-owned by the relevant orgs, and the session belongs to that parent. This collapses any would-be Shape E into Shape B.

Why the constraint matters: it ensures that scope resolution **never has to traverse two independent axes simultaneously**. The cascading rule always finds a unique answer because the multi-axis case is structurally impossible.

### The Unified Resolution Rule

When a reader attempts to read a session, the runtime resolves the reader's effective scope by cascading from most-specific to most-general:

```
fn resolve_scope(reader: Agent, session: Session) -> ResolvedScope {
    let session_projects = session.tags.filter(Tag::Project);
    let reader_project_matches = session_projects.intersection(reader.member_projects());

    // Project-level resolution (most specific)
    match reader_project_matches.count() {
        1 => return ResolvedScope::Project(reader_project_matches.first()),
        n if n > 1 => return ResolvedScope::Project(reader.base_project_among(reader_project_matches)),
        0 => {} // fall through to org-level
    }

    // Org-level resolution
    let session_orgs = session.tags.filter(Tag::Org);
    let reader_org_matches = session_orgs.intersection(reader.member_orgs());

    match reader_org_matches.count() {
        1 => ResolvedScope::Org(reader_org_matches.first()),
        n if n > 1 => ResolvedScope::Org(reader.base_org_among(reader_org_matches)),
        0 => ResolvedScope::Intersection(session_orgs),
    }
}
```

The resolved scope's grants apply, **bounded above** by the org caps of all orgs that own the resolved scope (this is Mechanism 1 from the Permission Resolution Hierarchy section).

**Tie-breaker:** When the reader belongs to multiple matching scopes at the same level, `base_project` (for projects) or `base_org` (for orgs) wins. This is consistent across both levels and depends only on stable agent state, not volatile context fields like `current_project`.

**Outsider rule:** A reader who belongs to none of the session's scopes at any level faces the **intersection of all the session's scope ceilings**. This is the strictest possible treatment and prevents loopholes where someone could create a permissive shadow scope to bypass restrictions.

### Worked Examples Across All Shapes

The examples below cover every allowed session shape with multiple reader roles. Each runs the resolution rule above and reports the resolved scope.

#### Shape A: 1 project, 1 org (the common case)

```yaml
session:
  tags: [agent:claude-coder-9, project:acme-internal, org:acme]
```

| Reader | Membership in session scopes | Resolved scope |
|--------|------------------------------|----------------|
| `claude-coder-9` (the owner) | project: acme-internal ✓ | **project:acme-internal** |
| `lead-acme-3` (lead of acme-internal) | project: acme-internal ✓ | **project:acme-internal** (with Template A grant) |
| `lead-other-7` (Acme employee, NOT in acme-internal) | project: ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`, so trivially that) |

#### Shape B: 1 project, 2 orgs (joint project)

```yaml
session:
  tags: [agent:<owner>, project:joint-research, org:acme, org:beta-corp]
```

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `lead-joint-3` (member of joint-research) | project ✓ | **project:joint-research** (project resolution succeeds, org axis irrelevant) |
| `lead-acme-7` (Acme but NOT in joint-research) | project ✗, org: acme ✓ (one match) | **org:acme** |
| `lead-beta-9` (Beta but NOT in joint-research) | project ✗, org: beta-corp ✓ (one match) | **org:beta-corp** |
| `dual-citizen-5` (Acme + Beta member, NOT in joint-research) | project ✗, org: both (two matches) | **base_org wins** — whichever is the agent's primary |
| `external-auditor` (neither org, not in project) | none | **intersection** of `{org:acme, org:beta-corp}` |

> **Why each lead reads under their own org's rules:** This is the contractor model. An Acme lead operates under Acme's rules even when reading joint-project data; a Beta lead operates under Beta's rules. Neither imposes their rules on the other. Only outsiders, who have no organizational citizenship, face the strict intersection.

#### Shape C: 2 projects, 1 org (cross-project within an org)

```yaml
session:
  tags: [agent:claude-coder-9, project:data-pipeline, project:ml-training, org:acme]
```

This shape arises when work legitimately spans two projects within the same org — for example, a workflow that touches both an extraction project and a training project. The system does not enforce single-project sessions, so this shape is allowed.

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `lead-pipeline-3` (member of data-pipeline only) | project: one match | **project:data-pipeline** |
| `lead-ml-7` (member of ml-training only) | project: one match | **project:ml-training** |
| `dual-project-2` (member of both projects) | project: two matches | **base_project wins** |
| `lead-other-9` (Acme, not in either project) | project ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`) |

> **Why each lead reads under their own project's rules:** Symmetric with Shape B at the org level. A pipeline lead reads under pipeline's rules even when the session also touches ml-training. Each project lead has scope authority over their own project's work, even when that work is shared.

#### Shape D: 0 projects, 1 org (system session)

```yaml
session:
  tags: [agent:platform-monitor-1, org:acme]
```

A System Agent doing platform maintenance for Acme, not associated with any user-facing project.

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `platform-monitor-1` (the owner) | project ✗, org: acme ✓ | **org:acme** (or own session via Default Grant 1) |
| `acme-admin-9` (Acme admin) | project ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`) |

System sessions are governed entirely by the org's rules, since they have no project scope to fall back to.

#### Shape E (forbidden) — what the constraint prevents

```yaml
session:
  tags: [agent:..., project:A, project:B, org:acme, org:beta-corp]
                    ^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                    multi-project + multi-org on the same session
```

**Cannot exist.** When work needs to span multiple orgs AND multiple projects, the system either:
- Refuses to create the session, OR
- Requires a joint parent project P co-owned by `{acme, beta-corp}`, and creates the session with `[project:P, org:acme, org:beta-corp]` (Shape B)

The constraint is enforced at session creation. The frozen-at-creation rule then prevents the session from drifting into Shape E later.

### Why the Cascade Stops at the First Match

A reader who is a member of Shape B's joint-research project does **not** also have org-level resolution applied — they get the project-level scope and the org axis is irrelevant for resolution purposes (org caps still apply as upper bounds via Mechanism 1). This is the cascading-resolution principle:

- Project membership is more specific than org membership
- The reader's most-specific claim wins
- The runtime never has to choose between project rules and org rules — project rules take precedence whenever the reader has project-level membership

This avoids ambiguity: there is exactly one resolved scope per (reader, session) pair, and the rule picks it deterministically.

### Subject-Side Reach Is Bounded by Scope Membership

A subtle but important consequence of this rule: an agent's home org (`base_organization`) does **not** reach into sessions belonging to scopes the agent is not a member of. When `claude-coder-9` (base_org = Acme) works in Beta's single-org project, the session is tagged `org:beta-corp` only — no `org:acme` tag. Acme has no scope claim on this session, and Acme's policies cannot constrain `claude-coder-9`'s read of it.

This is the **contractor model** made explicit: when an agent operates in another org's scope, they follow that scope's rules, full stop. Acme governs `claude-coder-9` only when `claude-coder-9` is operating within Acme's scope (i.e., reading sessions tagged `org:acme`).

If Acme wants to impose policies that follow its agents everywhere, those policies are not part of the permission system — they belong to the contract Acme signed when sending its agents to work elsewhere.

---

## Consent Policy (Organizational)

> **Resolves Open Question 1 from earlier drafts.** Subordinate consent is now an organizational policy expressed through the standard `approval_mode` machinery, rather than a separate concept.

### Three Consent Policies

The consent policy is **configurable per organization**, with the default set to **`implicit`**. This keeps the common case frictionless — orgs that need stronger consent guarantees can opt into `one_time` or `per_session` without affecting orgs that don't.

Each Organization picks one consent policy that applies to its Authority Templates (A, B, C, D — Template E is always explicit and doesn't need consent gating):

| Policy | When Authority Template Grants Fire | Storage |
|--------|--------------------------------------|---------|
| **Implicit (default)** | Auto-issue grants immediately when relationship edge is created | Consent is implied by joining the project/org |
| **One-time consent** | Auto-issue grants only after the subordinate signs a one-time consent record | A `Consent` node attached to the Agent, scoped to the org |
| **Per-session consent** | Templates issue grants with `approval_mode: subordinate_required` — every read attempt blocks until the subordinate approves it | Real-time approval, recorded per Permission Check |

The consent policy lives on the Organization node:

```yaml
organization:
  org_id: acme
  consent_policy: one_time   # implicit | one_time | per_session
  consent_scope:
    templates: [A, B, C, D]    # which templates the policy applies to
    excluded_actions: []       # actions exempt from consent (e.g. emergency reads)
```

### Implicit Consent (Default)

```
Edge created (e.g. HAS_LEAD) → grant auto-issued immediately
```

This is the current behavior. No consent step. The act of joining a project/org is taken as implicit consent to standard supervisory access.

### One-Time Consent

```
Edge created (e.g. HAS_LEAD)
    │
    ▼
Does subordinate have a Consent node for this template + org?
    │
    ├── Yes → grant auto-issued
    │
    └── No → grant queued; subordinate prompted to consent
              On consent → grant issued; Consent node created
              On refusal → grant never issued; supervisor sees a "no consent" status
```

The Consent node looks like:

```yaml
consent:
  consent_id: c-4581
  agent_id: claude-coder-9        # the subordinate
  scope:
    org: acme
    templates: [A, B]
    actions: [read, inspect]
  granted_at: 2026-04-09T12:00:00Z
  revocable: true
  provenance: agent:claude-coder-9@onboarding
```

A typical place to collect one-time consent is at agent creation (for a base-org consent record) or at project join (for a project-specific consent record). The org policy decides which.

### Per-Session Consent

The most invasive option. Templates auto-issue grants with `approval_mode: subordinate_required`. When the supervisor invokes `read_session`:

```
Permission check passes (grant exists, selector matches)
    │
    ▼
approval_mode is subordinate_required
    │
    ▼
Notify subordinate via their Channel (or in-band if subordinate is an LLM agent)
    │
    ├── Approved → read proceeds
    ├── Denied → read denied with audit log entry
    └── Timeout reached → default response (deny) applied with audit log entry
```

The pattern reuses `human_approval_required`, just naming a new approval principal: `subordinate_required`. Both go through the same `approval_mode` machinery already documented in the Permission Grant section.

#### Approval Timeout

Each `subordinate_required` approval request carries a **timeout**, configurable per organization. The default timeout is the **remaining duration of the project** that owns the session being read:

```yaml
organization:
  org_id: acme
  consent_policy: per_session
  approval_timeout: project_duration   # default; alternatives: a fixed Duration like "24h"
  approval_timeout_default_response: deny   # default; alternative: allow
```

**Why project duration as the default:**

- Approval requests are meaningful only while the project is active. Once the project closes, supervisor reads on its sessions are typically retrospective audits, which are governed by the archived-project rules (see Open Question on time-bounded authority for archived projects), not by per-session consent.
- It avoids requiring orgs to invent arbitrary timeout numbers ("is 24h right? 7 days?") — the natural lifecycle of the work itself is the answer.
- It scales with project complexity: a one-week sprint gets a one-week timeout; a six-month engagement gets a six-month timeout.
- A subordinate who is genuinely unreachable for the duration of an entire project has effectively withdrawn from the work — the timeout-deny default surfaces that fact rather than blocking the supervisor indefinitely.

**Computing "project duration" across session shapes:** The session's tags determine which project deadline(s) apply.

| Session shape | Project tags | How `project_duration` resolves |
|---------------|--------------|----------------------------------|
| **A** (1 project, 1 org) | exactly one `project:P` | The remaining duration of project `P` |
| **B** (1 project, joint orgs) | exactly one `project:P` (the joint project) | The remaining duration of the joint project `P` |
| **C** (multi-project within an org) | multiple `project:P1, project:P2, ...` | The **maximum** remaining duration among all the session's projects — i.e., the timeout lasts as long as *any* of the projects is still active |
| **D** (no project — system session) | zero `project:` tags | No project to anchor the timeout. Falls back to the org-level default (or a fixed Duration if the org configured one). If neither is set, an explicit fixed timeout is required. |

The "max across projects" rule for Shape C reflects the same intuition as the rest of the Multi-Scope rule: **the timeout is bounded by the most permissive project context that still claims the session**. If any of the session's projects is still active, the work that produced the session is still meaningful, so the supervisor's approval request remains relevant.

**Alternative timeouts:** Orgs that want tighter loops can set a fixed Duration (e.g. `"24h"`). The runtime treats whichever is shorter as the effective timeout — if the org sets `24h` but the resolved project duration is `2h`, the request times out in `2h`.

**Default response on timeout:** `deny`, on the principle that absence of consent is not consent. Orgs that want the opposite (e.g., for low-risk read operations where unblocking the supervisor matters more than strict consent) can set `approval_timeout_default_response: allow`.

### Consent Node (New Node Type)

| Property | Type | Description |
|----------|------|-------------|
| `consent_id` | String | Unique identifier |
| `agent_id` | agent_id | The subordinate giving consent |
| `scope.org` | org_id | The org under whose policy this consent operates |
| `scope.templates` | Vec<TemplateId> | Which Authority Templates this consent covers |
| `scope.actions` | Vec<Action> | Which actions are consented to |
| `granted_at` | DateTime | When the consent was given |
| `revocable` | bool | Whether the subordinate can later withdraw consent |
| `provenance` | String | Audit trail (typically `agent:{subordinate_id}@{event}`) |

**Edges:**
- `Agent ──HAS_CONSENT──▶ Consent` — the subordinate holds the consent record
- `Consent ──SCOPED_TO──▶ Organization` — the org under whose policy it applies

### Interaction with Template E

Template E (manual explicit grants) **does not go through consent**, because it represents a deliberate one-off authorization (e.g., a sponsor granting an auditor read access for a week). The provenance of Template E is always a specific human or admin agent, and the audit class is typically `alerted`. Consent is implicit in the act of issuing the grant.

### Consent Revocation Semantics

> Resolved: revocation applies forward only.

When a subordinate revokes a previously granted `one_time` consent (or denies a `per_session` approval request), the revocation applies to **future actions only**. Reads that have already happened under the consent are not retroactively undone — there is no "unread" operation, and audit logs of past reads remain intact.

This puts the responsibility for both granting and revoking consent squarely on the agent, and avoids two thorny problems:

1. **No retroactive cleanup** — the system doesn't have to track which extracted memories or downstream artifacts originated from a now-revoked consent and try to undo them. That would be an unbounded cascade with no clean stopping point.
2. **No "consent rollback" race conditions** — concurrent reads in flight at the moment of revocation are not interrupted; they complete, and any read attempted *after* the revocation point is denied.

In practical terms:

- For `one_time` consent: revocation deletes the Consent node (or marks it `revoked: true` for audit). The next time the runtime checks for the supervisor's Authority Template grant, the consent precondition fails and the grant is not issued (or is revoked if already issued).
- For `per_session` consent: each approval request is independent. A subordinate who has approved one read can deny the next one with no retroactive effect on the first.

### Open Questions for Consent

- [x] ~~**Default policy**~~ — Resolved: configurable per org, default `implicit`. See [Three Consent Policies](#three-consent-policies) above.
- [x] ~~**Consent revocation cascade**~~ — Resolved: revocation applies forward only. See [Consent Revocation Semantics](#consent-revocation-semantics) above.
- [x] ~~**Per-session consent timeout**~~ — Resolved: configurable per org, default is the remaining project duration; default response on timeout is `deny`. See [Approval Timeout](#approval-timeout) under Per-Session Consent above.

> All consent open questions resolved for v0. Future iterations may revisit these defaults based on real usage patterns.

---

## Open Questions for Session Permissions

- [x] ~~**Subordinate consent**~~ — Resolved by the [Consent Policy](#consent-policy-organizational) section above.
- [x] ~~**Cross-org session ceilings**~~ — Resolved by the [Multi-Scope Session Access](#multi-scope-session-access) section above.
- [ ] **Time-bounded authority for archived projects:** When a project ends and its sessions become `#archived`, do Template A grants survive (so the lead can still review historical work) or are they revoked (forcing re-authorization for any post-mortem)?
- [ ] **Conflict resolution during disputes:** During a rating dispute, can a sponsor override a project lead's authority to read disputed sessions? Probably yes via Template E, but should there be a structured "dispute grant" template?
- [ ] **Granularity escalation:** Should a `list` grant be auto-upgraded to `inspect` after some threshold? (E.g., "if you've listed it 10 times, you probably need to see it.") Probably no — feels like over-engineering.
- [ ] **Sessions with no project:** What about ad-hoc sessions created outside any project (e.g., a system agent doing platform maintenance)? They get an `org:` tag but no `project:` tag. Templates A/B/D don't fire; Templates C/E may.
- [ ] **Re-parenting:** If a session was created under one project and the project is later merged into another, do the tags update? Currently no (frozen rule). Is there a controlled "re-parent" operation that the system performs and audits?
- [ ] **Tag schema extensibility:** Should the tag vocabulary be open (agents can invent new tag namespaces) or closed (only system-defined tags)? Currently closed, which keeps reasoning tractable.

---

## Standard Permission Templates

> **Purpose:** Provide ready-to-use template sets that an org can adopt as a starting point. Without these, every new org has to author every grant from scratch — which is both error-prone and slow. With them, an org can pick a baseline and then customize.
>
> **Framing:** These templates are **explicitly customizable starting points**, not locked-in defaults. The system does NOT auto-apply them — an org admin (or the platform setup process) explicitly opts into a template, then customizes from there.

### Standard Organization Template

A baseline set of org-level grants and ceilings that most orgs can start from.

```yaml
organization_template:
  name: standard
  description: Default baseline for most organizations.

  # ── Tool allowlist ─────────────────────────────────────
  # Tools enabled org-wide. Projects and agents can further narrow.
  tools_allowlist:
    - read_file
    - write_file
    - search
    - list_files
    - recall_memory
    - store_memory
    - bash                 # enabled with sandbox_requirement
    # Network tools NOT enabled by default — orgs explicitly opt in

  tool_constraints:
    bash:
      sandbox_requirement: required
      timeout_secs: 120

  # ── Authority Templates ────────────────────────────────
  authority_templates_enabled: [A, B]   # project lead + direct delegation
  # Templates C (org chart) and D (project role) are opt-in per org
  # Template E (explicit) is always available

  # ── Consent policy ─────────────────────────────────────
  consent_policy: implicit   # configurable: implicit | one_time | per_session
  approval_timeout: project_duration
  approval_timeout_default_response: deny

  # ── Execution limits (default per-agent caps) ─────────
  execution_limits:
    max_turns: 50
    max_tokens: 100000
    max_duration_secs: 3600
    max_cost_usd: 5.00
  # Projects and agents may tighten these but not exceed them.

  # ── Session grants (baseline ceiling) ──────────────────
  session_object_grants:
    - action: [read, list, inspect]
      selector: "tags contains org:{this_org}"
      provenance: template:standard-org
      delegable: false
      approval_mode: auto
      audit_class: silent

  # ── Memory grants (baseline ceiling) ───────────────────
  memory_object_grants:
    - action: [recall]
      selector: "tags contains org:{this_org} OR tags contains #public"
      provenance: template:standard-org
      delegable: false
      approval_mode: auto

  # ── Rating window (from token economy) ─────────────────
  rating_window:
    size: 20
    promotion_threshold_rating: 0.6
    promotion_threshold_jobs: 10
```

**Commonly customized fields:**
- `tools_allowlist` (add/remove tools based on org posture)
- `authority_templates_enabled` (corporate orgs often add C; privacy-focused orgs drop A and use D/E)
- `consent_policy` (compliance-heavy orgs upgrade to `one_time` or `per_session`)
- `execution_limits` (tighter for cost-sensitive orgs)

### Standard Project Template

A baseline set of project-level grants for a typical project.

```yaml
project_template:
  name: standard
  description: Default baseline for most projects.

  # ── Workspace grant (propagated to all project members) ─
  filesystem_object_grants:
    - action: [read, modify]
      selector: "/workspace/{project_id}/**"
      constraints:
        max_size_bytes: 10485760  # 10MB
      provenance: template:standard-project
      delegable: true
      approval_mode: auto
      audit_class: silent

  # ── Project session grants (Default Grant 1 + 2 + Template A for lead) ─
  session_object_grants:
    - action: [read, list, inspect, append]
      selector: "tags contains agent:{self}"  # own sessions
      provenance: template:standard-project
    - action: [list, inspect]
      selector: "tags contains project:{project_id}"
      provenance: template:standard-project

  # ── Project-scoped memory grants ───────────────────────
  memory_object_grants:
    - action: [recall, store]
      selector: "tags contains project:{project_id}"
      provenance: template:standard-project
      delegable: false

  # ── Project execution limits (tighter than org) ────────
  execution_limits:
    max_cost_usd_per_task: 1.00   # override org-level per-task cap
    max_duration_secs_per_task: 1800

  # ── Consent policy (defaults to org's) ─────────────────
  consent_policy: inherit_from_org
```

**Commonly customized fields:**
- Workspace path (auto-filled from `{project_id}` on instantiation)
- `max_cost_usd_per_task` (sensitive projects may tighten further)
- Additional project-specific tool grants (e.g., enabling a project-specific MCP server)

### Standard Agent Templates

Three sub-templates — one per agent kind from the agent taxonomy.

#### System Agent Template

```yaml
agent_template:
  kind: system
  description: Infrastructure agent with broad read access, no economy participation.

  default_grants:
    # Broad read of org's data for monitoring purposes
    - action: [read, list, inspect]
      resource: data_object
      selector: "tags contains org:{base_org}"
      delegable: false
      audit_class: silent

    # Session inspection for monitoring
    - action: [list, inspect]
      resource: session_object
      selector: "tags contains org:{base_org}"
      delegable: false
      audit_class: silent

  token_economy_participation: none   # System agents don't participate
  rating_enabled: false
  worth_value_meaning_enabled: false
```

#### Intern Agent Template

```yaml
agent_template:
  kind: intern
  description: Pre-economy worker agent. Builds track record before market entry.

  default_grants:
    # Own sessions
    - action: [read, list, inspect, append]
      resource: session_object
      selector: "tags contains agent:{self}"
    # Project workspace (inherited from project template)
    # Project memories (inherited from project template)
    # Cannot hold Template A/B grants until promoted

  token_economy_participation: consume_only  # No earning; consumption is tracked
  rating_enabled: true
  worth_value_meaning_enabled: false   # Not calculated until promotion
  promotion_eligible: true
  promotion_criteria:
    jobs_completed_threshold: 10
    rating_threshold: 0.6
```

#### Contract Agent Template

```yaml
agent_template:
  kind: contract
  description: Full market participant. Earns token budgets via bidding.

  default_grants:
    # Own sessions
    - action: [read, list, inspect, append]
      resource: session_object
      selector: "tags contains agent:{self}"
    # Project workspace (inherited)
    # Project memories (inherited)
    # Can be awarded Template B grants when delegating

  token_economy_participation: full   # Earning, spending, savings
  rating_enabled: true
  worth_value_meaning_enabled: true
  rating_window:
    size: 20   # Inherits from org template
```

**How templates work together:**
1. Org admin picks the **Standard Organization Template** at org creation and customizes
2. For each project, the lead picks the **Standard Project Template** and customizes
3. For each agent created in a project, the system applies the appropriate **Standard Agent Template** based on the agent's `kind`
4. Templates produce the baseline permission state; subsequent customization (Template E explicit grants, Authority Template auto-issuance) layers on top

**Non-normative:** these templates are suggestions. An org with unusual requirements can author its own base template set. The templates above are reference points, not requirements.

---

## Tool Authority Manifest Examples

> **Purpose:** Provide a reference catalog of tool authority manifests that tool creators can use as starting points when authoring new tools. Each example illustrates how to use `actions`, `resource`, `constraints`, `transitive`, `#kind`, `delegable`, and `approval` fields for a representative tool category.
>
> **Framing:** These are **reference manifests**, not normative definitions. A tool creator copies the nearest example, edits it to fit their tool, and ships it with the tool. Every example uses fundamental-first declarations; composite shorthand is shown as an optional second form where applicable.

### 1. `read_file` — Safe file-read tool

```yaml
tool: read_file
manifest:
  resource: filesystem_object
  actions: [read, list, inspect]
  constraints:
    path_prefix: required
    max_size_bytes: 10485760   # 10MB
  transitive: []
  delegable: true
  approval: auto
```

**Notes:** Minimal-risk file tool. No transitive reach, safely delegable. Caller must provide a path prefix to scope the read.

### 2. `write_file` — File mutation tool

```yaml
tool: write_file
manifest:
  resource: filesystem_object
  actions: [create, modify]
  constraints:
    path_prefix: required
    max_size_bytes: 1048576   # 1MB default limit
  transitive: []
  delegable: true
  approval: auto
```

**Notes:** Path-scoped writes with size cap. Delegable because the scope is narrow.

### 3. `edit_file` — Targeted file mutation with diff mode

```yaml
tool: edit_file
manifest:
  resource: filesystem_object
  actions: [modify]
  constraints:
    path_prefix: required
    diff_mode: true          # tool-specific constraint
    max_size_bytes: 1048576
  transitive: []
  delegable: true
  approval: auto
```

**Notes:** Demonstrates a tool-specific constraint (`diff_mode`) that goes beyond the universal constraint set.

### 4. `search` — Recursive read with bounded result

```yaml
tool: search
manifest:
  resource: filesystem_object
  actions: [read, list, discover]
  constraints:
    path_prefix: required
    max_matches: 1000        # tool-specific
    max_size_bytes: 10485760
  transitive: []
  delegable: true
  approval: auto
```

**Notes:** Reaches many files but with a bounded result set. The `max_matches` constraint prevents runaway searches.

### 5. `bash` — High-risk shell command

```yaml
tool: bash
manifest:
  resource: process_exec_object
  actions: [execute]
  constraints:
    command_pattern: required
    sandbox_requirement: recommended
    timeout_secs: 120
  transitive:
    - filesystem_object       # can read/write files
    - network_endpoint        # can make HTTP calls
    - secret/credential       # can access env vars
  delegable: false            # too powerful to delegate
  approval: human_recommended
```

**Notes:** The canonical high-risk tool. Declares maximum transitive reach because shell commands can do almost anything. Not delegable. Human approval recommended.

### 6. `http_get` — Pure network tool

```yaml
tool: http_get
manifest:
  resource: network_endpoint
  actions: [read, connect]
  constraints:
    domain_allowlist: required
    timeout_secs: 30
    max_size_bytes: 10485760
  transitive: []
  delegable: true
  approval: auto
```

**Notes:** No transitive reach — HTTP GET is self-contained. Delegable because the domain allowlist is narrow. If the tool needed authentication, `secret/credential` would move to `transitive`.

### 7. `load_env` — Secret-touching tool

```yaml
tool: load_env
manifest:
  resource: secret/credential
  actions: [read]
  constraints:
    env_file_path: required
  transitive:
    - filesystem_object       # env files are on disk
  delegable: false
  approval: human_recommended
```

**Notes:** Demonstrates a secret-primary tool with filesystem as transitive (the env file has to be read from disk). `delegable: false` is the right default for secret access.

### 8. `mcp_github` — MCP adapter tool (composite form)

**Fundamental form:**

```yaml
tool: mcp_github
manifest:
  resource: network_endpoint
  actions: [read, modify, create]
  constraints:
    domain_allowlist: ["api.github.com"]
    timeout_secs: 60
  transitive:
    - secret/credential       # GitHub token
    - tag                     # for #kind filter
  kind: [external_service]    # required: manifest must declare which composite kinds it touches
  delegable: false
  approval: human_recommended
```

**Composite shorthand (equivalent):**

```yaml
tool: mcp_github
manifest:
  resource: external_service_object  # composite: expands to network + secret + tag
  actions: [read, modify, create]
  constraints:
    domain_allowlist: ["api.github.com"]
    timeout_secs: 60
  kind: [external_service]    # still required even with composite label
  delegable: false
  approval: human_recommended
```

**Notes:** Both forms are semantically identical. The composite form is shorter but requires the `kind` declaration anyway. The linter may suggest the composite form for readability.

### 9. `recall_memory` and `store_memory` — Memory tools

```yaml
tool: recall_memory
manifest:
  resource: memory_object   # composite
  actions: [recall]
  constraints:
    tag_predicate: required
  kind: [memory]
  delegable: false
  approval: auto
```

```yaml
tool: store_memory
manifest:
  resource: memory_object
  actions: [store]
  constraints:
    tag_predicate: required
    max_size_bytes: 65536   # 64KB per memory entry
  kind: [memory]
  delegable: false
  approval: auto
```

**Notes:** The caller must provide a tag predicate to scope the recall or store. `#kind: memory` is declared explicitly even when the `resource` is the `memory_object` composite.

### 10. `read_session` — Session-reading tool

```yaml
tool: read_session
manifest:
  resource: session_object
  actions: [read, inspect]
  constraints:
    tag_predicate: required
  kind: [session]
  transitive: []
  delegable: false
  approval_mode: inherit_from_grant   # honors the grant's consent policy
```

**Notes:** Session reads plug into the Consent Policy machinery via `approval_mode: inherit_from_grant`. If the agent's session grant carries `subordinate_required`, each read triggers the consent flow (see [Consent Policy](#consent-policy-organizational)).

### 11. `delegate_task` — Sub-agent spawning tool

```yaml
tool: delegate_task
manifest:
  resource: identity_principal
  actions: [delegate, create]
  constraints:
    subordinate_pattern: required
    max_budget: required
  transitive:
    - economic_resource       # spawns an agent with a token budget
    - time/compute_resource   # spawns an agent that will consume compute
  delegable: false            # no re-delegation of delegation authority
  approval: inherit_from_org  # some orgs require human approval for delegation
```

**Notes:** Delegation itself is a tool manifest. The transitive reach includes economic and compute resources because the sub-agent will consume them. `delegable: false` prevents an agent from passing delegation authority further down the chain.

### 12. `run_python` — High-risk interpreter with tighter constraints

```yaml
tool: run_python
manifest:
  resource: process_exec_object
  actions: [execute]
  constraints:
    sandbox_requirement: required   # stricter than bash
    timeout_secs: 60
    max_memory_mb: 512              # additional constraint
  transitive:
    - filesystem_object
    - network_endpoint
    - secret/credential
  delegable: false
  approval: human_recommended
```

**Notes:** Contrasts with `bash` — same transitive reach profile but tighter constraints (mandatory sandbox, shorter timeout, memory cap). Shows how the same fundamental (`process_exec_object`) can have different risk profiles based on constraint tightness.

---

## Authoring a Tool Manifest: A Guide for Tool Creators

> **Purpose:** Close the authoring gap — a tool creator needs a systematic way to know which fundamentals their tool touches and which `#kind:` values to declare. Under the two-tier ontology, this becomes a single-table lookup: operation → fundamentals + `#kind:`.
>
> **Framing:** This is a mechanical procedure. A tool creator enumerates their tool's side effects, looks each one up in the table, and ships. Publish-time validation is the safety net.

### Why Fundamentals and `#kind:` Matter

Your manifest must declare every fundamental your tool touches and every composite `#kind:` value the tool operates on. These declarations are validated at **tool publish time** — if the manifest is inconsistent or incomplete, the tool registry rejects the publish with a specific error, and no agent can invoke the tool until the manifest is fixed.

- **Missing a fundamental:** publish rejected. Fundamentals are the bedrock; a missing fundamental means the tool has an uncovered capability.
- **Missing a `#kind:`:** publish rejected. Tool must explicitly declare which composite kinds it touches.
- **Missing a composite label (when fundamentals + `#kind:` are present):** warning only. Composite labels are documentation sugar.

The safe strategy is: **always declare fundamentals exhaustively and `#kind:` explicitly**.

### Operation → Fundamentals + `#kind:` Table

For each side effect your tool has, look it up in this table and add both the fundamentals AND the `#kind:` declarations to your manifest:

| If your tool does... | Fundamentals | `#kind:` | Why |
|---|---|---|---|
| Reads a file on disk | `filesystem_object` | — | Not a composite |
| Writes/modifies/deletes a file | `filesystem_object` | — | Same |
| Lists a directory | `filesystem_object` | — | Same |
| Reads an env var holding a secret | `filesystem_object` + `secret/credential` | — | Entity-level overlap, no composite |
| Reads an API key from any source | `secret/credential` | — | Credential material |
| Spawns a shell command | `process_exec_object` + `filesystem_object` + `network_endpoint` + `secret/credential` + `time/compute_resource` | — | Shell commands can reach everything |
| Spawns a sandboxed command (no network, no fs) | `process_exec_object` + `time/compute_resource` | — | Sandbox restricts reach |
| Makes an unauthenticated HTTP call | `network_endpoint` | — | Just network |
| Makes an authenticated HTTP call | `network_endpoint` + `secret/credential` | — | Network + auth |
| Calls an MCP server | `network_endpoint` + `secret/credential` + `tag` | `#kind: external_service` | Composite `external_service_object` |
| Calls an OpenAPI endpoint with a bearer token | `network_endpoint` + `secret/credential` + `tag` | `#kind: external_service` | Same |
| Calls an LLM API | `network_endpoint` + `secret/credential` + `economic_resource` + `tag` | `#kind: model_runtime` | Composite `model/runtime_object` |
| Sends a Slack/email/webhook message | `network_endpoint` + `secret/credential` + `tag` | `#kind: external_service` | Composite `external_service_object` |
| Reads a graph node not tied to memory/session | `data_object` | — | Bare `data_object` |
| Reads a memory entry (tag-filtered) | `data_object` + `tag` | `#kind: memory` | Composite `memory_object` |
| Stores a memory entry | `data_object` + `tag` | `#kind: memory` | Same |
| Reads a session (tag-filtered) | `data_object` + `tag` | `#kind: session` | Composite `session_object` |
| Writes a session log to disk | `data_object` + `tag` + `filesystem_object` | `#kind: session` | Session on disk triples up |
| Reads both memories AND sessions | `data_object` + `tag` | `#kind: [memory, session]` | Must declare both |
| Generic graph backup tool (reads everything) | `data_object` + `tag` + `filesystem_object` | `#kind: *` | Blanket — warning at publish |
| Spawns a sub-agent | `identity_principal` + everything the sub-agent can reach | — | Delegation |
| Reads or charges a token budget | `economic_resource` | — | Budget, no composite |
| Allocates CPU/memory/wall-clock | `time/compute_resource` | — | Compute |
| Reads policy store / tool registry / audit log | `data_object` + `identity_principal` + `tag` | `#kind: control_plane` | Composite `control_plane_object` |
| Configures a model endpoint or prompt template | `network_endpoint` + `secret/credential` + `economic_resource` + `tag` | `#kind: model_runtime` | Composite `model/runtime_object` |
| Manages agents, users, or roles | `identity_principal` | — | Identity ops |

**How to use this table:** For each side effect, look it up and union the results. The resulting manifest is **complete by construction** — if you follow the table honestly, the publish-time validator will accept the manifest.

**Composite shorthand (optional):** you may use composite labels in the `resource` field of your manifest for readability (e.g., `resource: external_service_object`). You must still declare the `#kind:` explicitly — the composite label is documentation sugar, while `#kind:` is security-relevant.

### Self-Check Protocol (6 Steps)

1. **Enumerate every side effect.** List every externally-visible action: disk, network, credentials, process spawns, graph reads/writes, memory, sessions, delegation, budget, compute.
2. **Map each side effect to fundamentals** using the table above. Union the results.
3. **Identify every composite kind your tool touches.** For each composite (memory, session, external_service, model_runtime, control_plane), ask: *does my tool actually operate on that composite kind?* If yes, declare it in `#kind:`. If all kinds, use `#kind: *` (accepts the warning). If none, omit `#kind:`.
4. **Enumerate constraints.** For each fundamental, consult the Constraint × Fundamental matrix and add relevant constraints. Universal constraints can always be added.
5. **Cross-check against the examples catalog.** Find the nearest example above. If your declarations are smaller, ask why. Differences should be explainable by your tool's actual behavior.
6. **Submit to publish-time validation.** Run your manifest through the tool registry's validator. If rejected, fix the named issue and resubmit. You cannot ship a tool with an invalid manifest.

### Conservative Over-Declaration Principle

> When in doubt, **declare more fundamentals and kinds rather than fewer**. Over-declaring is slightly annoying (invoking agents need extra grants) but is always safe. Under-declaring is a **hard publish error** — the validator will reject the manifest.

Worked example:

A pure-read tool `list_files` could technically omit `secret/credential` because it only lists filenames. But declaring it is safe (invoking agents usually have the grant anyway), and it guards against a future refactor that adds content-reading to `list_files` without anyone updating the manifest.

**Exception — do not over-declare `delegable: true` or `approval: auto`.** Over-declaring those fields weakens the permission envelope. Fundamentals and `#kind:` are safe to over-declare; delegation and approval modes are not.

### What v0 Validates vs Future Enhancements

**v0 declaration-only validation (at publish time):**

- Every declared `resource` or `transitive` composite has a matching `#kind:` entry
- Declared fundamentals are a valid superset of what the declared composites expand to
- Declared actions are compatible with declared fundamentals per the Action × Fundamental matrix
- Declared constraints are applicable per the Constraint × Fundamental matrix
- `#kind: *` accepted with warning
- Action/resource combinations that make no sense (e.g., `execute` on `memory_object`) are rejected

**Planned enhancements `[PLANNED]`** (not in v0):

- **Static analysis** of tool source code to verify manifest declarations match actual behavior (per-language tooling required)
- **Test-harness observation** of tool's runtime syscalls in a sandbox (most accurate but heaviest)
- **Continuous validation** on every code change (CI integration)
- **Cross-tool conflict detection** for tools whose combined permissions would exceed org ceilings

For v0, declaration-only validation catches the most common errors (missing `#kind:`, mismatched fundamentals, bad action/resource pairings). Static analysis and test-harness are future enhancements.

---

## Worked End-to-End Use Case: Three Orgs, Twelve Projects

> **Purpose:** Demonstrate every concept in this doc together — org ceilings, project grants, agent defaults, Authority Templates, consent policies, Multi-Scope Session Access, resource overlap, contractor scenarios, and the forbidden Shape E recovery. This is the "tabletop test" of the model.
>
> **Setup:** A fictional but realistic organizational structure. Each org starts from the Standard Organization Template introduced above, then customizes.

### Cast of Organizations and Projects

**Organizations (3):**

- **Acme** — primary tenant; runs three internal projects and co-sponsors one joint project. Adopts the Standard Organization Template as-is.
- **Beta Corp** — secondary tenant; runs three internal projects and co-sponsors the joint project. Customizes: `bash` is entirely disabled (not merely sandbox-required), `consent_policy: one_time`.
- **Gamma Consulting** — small consultancy; runs four internal projects, no joint work. Customizes: tighter execution limits, `consent_policy: per_session`.

**Projects (12, 4 per org):**

| Org | Projects |
|-----|----------|
| Acme | `acme-internal-tools`, `acme-website-redesign`, `acme-api-platform`, `joint-research` (co-owned with Beta) |
| Beta | `beta-data-pipeline`, `beta-mobile-app`, `beta-billing`, `joint-research` (co-owned with Acme) |
| Gamma | `gamma-client-a-audit`, `gamma-client-b-migration`, `gamma-internal-ops`, `gamma-knowledge-base` |

`joint-research` is a **single project node co-owned by Acme and Beta** — Shape B from Multi-Scope Session Access. Sessions in `joint-research` carry both `org:acme` and `org:beta-corp` tags.

### Cast of Agents (~14 total)

**Acme agents (5):**
- `lead-acme-1` — Contract agent, lead of `acme-website-redesign`
- `coder-acme-2` — Intern agent, member of `acme-internal-tools`
- `coder-acme-3` — Contract agent, member of `acme-api-platform`
- `auditor-acme-4` — System agent, platform monitoring across all Acme projects
- `joint-acme-5` — Contract agent, member of both `acme-api-platform` and `joint-research`

**Beta agents (4):**
- `lead-beta-1` — Contract agent, lead of `beta-billing`
- `coder-beta-2` — Intern agent, member of `beta-data-pipeline`
- `joint-beta-3` — Contract agent, member of `joint-research`
- `monitor-beta-4` — System agent, platform monitoring for Beta

**Gamma agents (4):**
- `lead-gamma-1` — Contract agent, lead of `gamma-client-a-audit`
- `coder-gamma-2` — Contract agent, member of `gamma-client-b-migration`
- `auditor-gamma-3` — System agent, internal ops
- `compliance-gamma-5` — Contract agent, read-only auditor on `gamma-client-a-audit` with memory-only access (asymmetric scenario)

**Cross-org agents (1):**
- `contractor-x-9` — Contract agent, base_org = Gamma, currently contracted into `acme-website-redesign`

### Permission Setup Walkthrough

#### Step 0: Template adoption

Each org starts from the Standard Organization Template (see above) and customizes. The deltas are:

**Acme** — adopts as-is:
```yaml
organization: acme
inherits_from: standard
customizations: none
```

**Beta** — disables `bash` and tightens consent:
```yaml
organization: beta
inherits_from: standard
customizations:
  tools_allowlist_remove: [bash]
  consent_policy: one_time
```

**Gamma** — tightest cost caps, per-session consent:
```yaml
organization: gamma
inherits_from: standard
customizations:
  execution_limits:
    max_cost_usd: 2.00          # tighter than standard 5.00
    max_turns: 30
  consent_policy: per_session
  approval_timeout: project_duration
  approval_timeout_default_response: deny
```

This single step replaces dozens of individual grants — the templates handle the defaults.

#### Step 1: Project-level grants for `joint-research`

The joint project inherits from both Acme AND Beta org templates. Both orgs' ceilings apply as upper bounds. The project lead authors project-specific grants:

```yaml
project: joint-research
owned_by: [acme, beta]
permissions:
  # Inherits Standard Project Template
  # PLUS project-specific additions:

  - action: [read, store]
    resource: memory_object
    selector: "tags contains project:joint-research"
    kind: [memory]
    provenance: config:joint-research.toml
    delegable: false

  - action: [read, list, inspect]
    resource: session_object
    selector: "tags contains project:joint-research"
    kind: [session]
    provenance: config:joint-research.toml
    delegable: false

# Effective ceiling: intersection of Acme and Beta org ceilings.
# Since Beta disabled bash entirely, joint-research inherits that — no agent in
# joint-research can use bash, even an Acme agent. Subject-side ceilings from the
# reader's base_org do NOT reach into this project (contractor model), but the
# project's own effective ceiling is the intersection of its owning orgs.
```

#### Step 2: Authority Template auto-issuance

**When `lead-acme-1` is appointed lead of `acme-website-redesign`**, Template A fires:

```yaml
permission:
  # subject = agent:lead-acme-1
  action: [read, inspect, list]
  resource: session_object
  selector: "tags contains project:acme-website-redesign"
  kind: [session]
  constraints: {}
  provenance: system:has_lead@project:acme-website-redesign
  delegable: false
  approval_mode: auto   # Acme uses implicit consent
  audit_class: logged
  revocation_scope: revoke_when_edge_removed
```

**When `contractor-x-9` is brought into `acme-website-redesign`**, project membership grants apply but NO Template A grant fires (they're not a lead). The contractor inherits `acme-website-redesign`'s project template grants for the duration of the contract.

**When `lead-acme-1` delegates work to `contractor-x-9` at loop `L42`**, Template B fires:

```yaml
permission:
  # subject = agent:lead-acme-1
  action: [read, inspect]
  resource: session_object
  selector: "tags contains delegated_from:L42"
  kind: [session]
  constraints: {}
  provenance: system:delegation@L42
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: end_of_delegation_chain
```

#### Step 3: Consent policies in action

**Acme's `implicit` policy:** `lead-acme-1` reads a session owned by `coder-acme-3`. The read proceeds immediately without any consent prompt. Audit log records the read.

**Beta's `one_time` policy:** When `lead-beta-1` was first appointed lead of `beta-billing`, each member signed a one-time consent record:
```yaml
consent:
  agent_id: coder-beta-2
  scope:
    org: beta
    templates: [A, B]
    actions: [read, inspect]
  granted_at: 2026-04-05T10:00:00Z
  revocable: true
```
Subsequent reads by `lead-beta-1` proceed because the consent record exists.

**Gamma's `per_session` policy:** `lead-gamma-1` tries to read a session owned by `coder-gamma-2`. The runtime notifies `coder-gamma-2` via their Channel. The coder approves (or denies, or times out) for this specific read. If approved, the read proceeds; if denied or timed out (after project_duration), the read is denied with an audit entry.

#### Step 4: Multi-Scope resolution for joint-research

Session tagged `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5]`.

**Reader: `joint-acme-5`** (member of joint-research):
- Project resolution: reader is in joint-research ✓
- Resolved scope: `project:joint-research`
- Joint project's rules apply (capped by intersection of Acme + Beta org ceilings — so `bash` denied)
- **Allowed**

**Reader: `lead-acme-1`** (not in joint-research, base_org = Acme):
- Project resolution: reader not in joint-research ✗
- Org resolution: reader's base_org is Acme, session has `org:acme` ✓
- Resolved scope: `org:acme`
- Acme's rules apply
- **Allowed** (subject to grant match)

**Reader: `lead-beta-1`** (not in joint-research, base_org = Beta):
- Project resolution: reader not in joint-research ✗
- Org resolution: reader's base_org is Beta, session has `org:beta-corp` ✓
- Resolved scope: `org:beta-corp`
- Beta's rules apply
- **Allowed** (subject to grant match AND Beta's one_time consent)

**Reader: `lead-gamma-1`** (base_org = Gamma, no membership in Acme or Beta):
- Project resolution: ✗
- Org resolution: Gamma is neither Acme nor Beta ✗
- Resolved scope: `Intersection(org:acme, org:beta-corp)`
- Both ceilings apply
- **Denied** — the intersection is highly restrictive, and Gamma has no grant to read joint-research sessions anyway

#### Step 5: Resource overlap example

`coder-acme-3` runs `cat /workspace/acme-api-platform/.env` via bash.

Entity classification: `/workspace/acme-api-platform/.env` → `{filesystem_object, secret/credential}`

Plus manifest-level requirements from bash: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Union: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Permission check per fundamental:
- `process_exec_object`: agent has bash grant ✓
- `filesystem_object`: agent has workspace grant ✓
- `network_endpoint`: check for network grant... agent has no network grant in its default set, but this specific invocation doesn't actually touch the network. ✗ **Denied**.

Wait — this illustrates a key point. The `bash` manifest declares transitive `network_endpoint`, so even a `cat` invocation that doesn't actually hit the network requires a `network_endpoint` grant. This is the **conservative over-declaration** principle at work. To fix, either:
- Use a more narrow tool (e.g., `read_file` which has no transitive network) for reading files
- Grant `coder-acme-3` a narrow `network_endpoint` grant scoped to a domain allowlist

**If the agent had only `read_file`:** the permission check would be just `{filesystem_object, secret/credential}`. The agent has filesystem read on the project workspace. Does it have `secret/credential`? Under the Standard Project Template, no — secrets require explicit grants. So **denied** because of the `.env` file's entity-level overlap with `secret/credential`, not because of a missing filesystem grant.

#### Step 6: Asymmetric tagged-composite access (the keystone example)

`compliance-gamma-5` is an auditor agent granted **memory_object read only** on `project:gamma-client-a-audit`. Sessions are off-limits. Grant:

```yaml
permission:
  # subject = agent:compliance-gamma-5
  action: [recall, read]
  resource: memory_object   # composite
  selector: "tags contains project:gamma-client-a-audit"
  kind: [memory]
  provenance: agent:human-sarah@2026-04-09
  delegable: false
```

**Scenario A — auditor reads a project memory:**
- Target entity: Memory node with tags `{project:gamma-client-a-audit, agent:coder-gamma-2, #kind:memory}`
- Required fundamentals: `{data_object, tag}`
- Agent's grant resolves to: fundamentals `{data_object, tag}`, effective selector `tags contains project:gamma-client-a-audit AND tags contains #kind:memory`
- Selector match: `project:gamma-client-a-audit` ✓, `#kind:memory` ✓
- **Allowed** ✓

**Scenario B — auditor tries to read a project session:**
- Target entity: Session node with tags `{project:gamma-client-a-audit, agent:coder-gamma-2, #kind:session, org:gamma}`
- Required fundamentals: `{data_object, tag}`
- Agent's grant (same as above) has effective selector requiring `#kind:memory`
- Selector match: `project:gamma-client-a-audit` ✓, but `#kind:memory` ✗ (session has `#kind:session`)
- No grant matches → **Denied** ✗

**Scenario C — what if the sponsor had granted bare fundamentals?**

```yaml
# WRONG grant — too broad
permission:
  action: [read]
  resource: data_object   # fundamental, not composite
  # plus an implicit tag grant
  selector: "tags contains project:gamma-client-a-audit"
  # no kind filter
```

Both memory AND session reads would be allowed because there's no `#kind:` filter. This is why **composite vs bare-fundamental distinction matters** even though the runtime check runs on fundamentals.

#### Step 7: Contractor scenario

`contractor-x-9` (base_org = Gamma) reads sessions in `acme-website-redesign`:

- Session tags: `[project:acme-website-redesign, org:acme, agent:coder-acme-3, #kind:session]`
- Multi-Scope resolution: reader is a member of `acme-website-redesign` (via the contract) → project-level resolution succeeds → `project:acme-website-redesign` applies
- Gamma's subject-side ceilings do **not** reach (the session has no `org:gamma` tag)
- Acme's project rules apply
- The contractor operates entirely under Acme's rules for the duration of the contract
- **Allowed** if the contractor has the project grants

This is the contractor model from Multi-Scope Session Access — the reader's base_org is irrelevant when a project-level resolution succeeds.

#### Step 8: Forbidden Shape E recovery

`joint-acme-5` starts work that spans `acme-api-platform` AND `joint-research`. Attempting to create a single session with tags `[project:acme-api-platform, project:joint-research, org:acme, org:beta-corp]` is **rejected** at session creation time — this is Shape E (multi-project AND multi-org), which is forbidden.

**Valid alternatives:**

1. **Two separate sessions**, one per project:
   - Session 1: `[project:acme-api-platform, org:acme, agent:joint-acme-5]`
   - Session 2: `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5, delegated_from:<session_1_loop>]`
   - Linked via the `delegated_from:` tag

2. **A new joint parent project** (`acme-beta-platform-research`) co-owned by all three scopes, with a single session: `[project:acme-beta-platform-research, org:acme, org:beta-corp, agent:joint-acme-5]` — this is Shape B, which is allowed.

The session creation layer enforces the Shape E constraint; the frozen-at-creation rule then prevents drift into Shape E afterward.

### Summary: Who Can Read What

A condensed access matrix showing which agents can read which session types after all the setup above. This is the "tabletop test" result.

| Agent | Own sessions | Sessions in base project | Sessions in other projects (same org) | joint-research sessions | Cross-org sessions |
|-------|-------------|-------------------------|----------------------------------------|-------------------------|---------------------|
| `lead-acme-1` | ✓ (default) | ✓ (Template A on website-redesign) | list/inspect only | ✓ (via org:acme resolution) | — |
| `coder-acme-2` | ✓ (default) | list/inspect only (intern, no Template A) | list/inspect only | — | — |
| `joint-acme-5` | ✓ (default) | list/inspect only | list/inspect only | ✓ (project member) | — |
| `lead-beta-1` | ✓ (default) | ✓ (Template A on billing) | list/inspect only | ✓ (via org:beta resolution, with one_time consent) | — |
| `coder-beta-2` | ✓ (default) | list/inspect only | — | — | — |
| `lead-gamma-1` | ✓ (default) | ✓ (Template A on client-a-audit, with per_session consent) | — | ✗ (intersection fallback) | — |
| `compliance-gamma-5` | ✓ (default) | memory only on client-a-audit (no session access) | — | — | — |
| `contractor-x-9` | ✓ (default) | ✓ on acme-website-redesign (as contracted member) | — | — | — |
| `auditor-acme-4` (System) | N/A (system) | list/inspect across all Acme projects | list/inspect | list/inspect on joint-research | — |

**What the matrix demonstrates:**

- Default grants cover own-session reads universally
- Templates A and B are what enable reading other agents' sessions within the project
- Interns (like `coder-acme-2`) can't supervise, only work
- Joint project membership wins over org-level resolution
- Consent policies vary per org without affecting the structural rules
- Cross-org read is restricted to project-level participation or explicit Template E grants
- Asymmetric composite access (`compliance-gamma-5`) is expressible via `#kind:memory` scoping

This completes the end-to-end demonstration. Every concept in the doc — from fundamentals and composites through Authority Templates to consent policies — is exercised in a single coherent setup.

---

## phi-core Extension Points

Permissions are a baby-phi concern. phi-core provides the hooks:

| phi-core hook | Permission enforcement |
|---|---|
| `InputFilter` | Check `read` permission on `data_object` before message reaches agent |
| `BeforeToolExecutionFn` | Check tool's authority manifest against agent's permissions |
| `ExecutionLimits` | Enforce `time_compute_resource` and `economic_resource` constraints |
| `BeforeLoopFn` | Check `delegate` permission before sub-agent loop starts |

---

## Open Questions

- [ ] How are permissions bootstrapped? First agent needs permissions to create permissions.
- [ ] Should there be a "root" permission that is non-revocable?
- [ ] How do permissions interact with the Market (can an agent bid on work it doesn't have permissions for yet, with permissions granted on contract acceptance)?
- [ ] Should audit_class be per-permission or per-action?
- [ ] How do MCP server capabilities interact with the resource ontology?