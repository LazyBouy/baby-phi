<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Standard Action Vocabulary

**Reusable** — the same action word means the same thing across the fundamental resource classes where it applies. **Not universal** — each action category applies only to a subset of fundamentals. The matrix below shows which action categories apply to which fundamentals. Composite classes inherit actions by expansion.

| Category | Actions |
|----------|---------|
| **Discovery** | discover, list, inspect |
| **Data** | read, copy, export |
| **Mutation** | create, modify, append, delete |
| **Execution** | execute, invoke, send |
| **Connection** | connect, bind, listen |
| **Authority** | delegate, approve, escalate, allocate, transfer |
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
- Discovery, Authority, and Observability apply universally (every fundamental has `list`/`inspect`, `delegate`/`allocate`/`transfer`, and `observe`/`log`/`attest`)
- The Memory category (`store`/`recall`/`retain`) is unique to `tag` — this is what makes `tag` a fundamental in its own right
- The Connection category applies only to `network_endpoint`
- Mutation is denied on `secret/credential` because mutating a secret isn't a local operation — credential rotation happens via a separate control-plane flow

### `allocate` as the Umbrella Action

`allocate` (in the **Authority** category, alongside `delegate`/`approve`/`escalate`) is the scope value that expresses **Shared Allocation** — the ownership-sharing mechanism from the [Resource Ownership](01-resource-ontology.md#resource-ownership) model. A principal holding `[allocate]` on a resource holds a share of ownership authority; multiple principals may hold `[allocate]` concurrently, and that is precisely how [Co-Ownership](01-resource-ontology.md#co-ownership-shared-resources) is represented in the graph.

The detailed semantics — what shareholders can mechanically do, how sub-grants trace their provenance, how revocation cascades — live in the [Auth Request Lifecycle](02-auth-request.md#auth-request-lifecycle) section, specifically under [`allocate` Scope Semantics](02-auth-request.md#allocate-scope-semantics). This subsection only names the action's placement in the vocabulary.

**Why `allocate` is umbrella rather than split:** we considered expressing each sub-capability as a distinct action value (`allocate_with_delegation`, `approve_request`, `escalate`, `revoke`). The umbrella approach wins on simplicity: one action word that names the ownership-sharing fact, with refinements expressed as constraints where they matter. A grant holder with `[allocate]` gets the full shareholder bundle by default; orgs that want to restrict specific sub-capabilities (e.g., `allocate: no_further_delegation`) use constraints. This preserves the clean "actions describe what authority; constraints describe how it's narrowed" distinction.

### Conservative Over-Declaration on Tool Manifests

When authoring a tool manifest, **err on the side of declaring more fundamentals and actions than strictly required**. A `bash` tool whose most common use is `cargo build` should still declare `network_endpoint` in its manifest, because a shell command *could* reach the network — and the manifest describes the tool's **maximum reach**, not its common-case reach. The runtime's Permission Check then limits each invocation to what the caller's grants authorise.

**The security philosophy:**

- **Over-declaration is safe.** Callers without the extra grants get predictable denials on reaches they weren't authorised for. The envelope is honest about what the tool could do.
- **Under-declaration is unsafe.** A tool that silently reaches a fundamental it didn't declare can succeed at something no grant authorised — a capability leak.
- **Manifests are the tool's upper bound** on capability; **grants are the caller's upper bound** on authority. The Permission Check returns the intersection.

**Exception — do not over-declare `delegable: true` or `approval: auto`.** Those fields weaken the permission envelope (one allows sub-delegation, the other removes the human-in-the-loop). Fundamentals and `#kind:` values are safe to over-declare; `delegable` and `approval` are not.

This principle is applied in practice throughout the [Tool Authority Manifest Examples](07-templates-and-tools.md#tool-authority-manifest-examples) catalog (see the `bash` example for a canonical over-declaration pattern) and restated in the [Authoring Guide](07-templates-and-tools.md#conservative-over-declaration-principle) for tool authors.

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

A Grant has two fields that are often confused. This subsection pins down the difference. (For the formal grammar of the selector DSL itself, see [09-selector-grammar.md](09-selector-grammar.md).)

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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `path_prefix`, `max_size_bytes` + universal constraints

**Common entity overlaps:** `.env` files overlap with `secret/credential`; memory/session files overlap with their respective composites

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Configuration: `configure`, `install`, `enable`, `disable`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `command_pattern`, `sandbox_requirement`, `timeout_secs` + universal constraints

**Common entity overlaps:** A process can transitively reach `filesystem_object`, `network_endpoint`, `secret/credential`, and `time/compute_resource` — these are declared in the tool manifest's `transitive` field

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `domain_allowlist`, `max_size_bytes`, `output_channel`, `timeout_secs` + universal constraints

**Common entity overlaps:** An authenticated network call overlaps with `secret/credential` (the auth token is a separate fundamental)

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** Universal constraints only (no resource-specific constraints — secrets are atomic)

**Common entity overlaps:** `.env` files (filesystem + secret), MCP auth (external_service composite), LLM API keys (model/runtime composite)

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_size_bytes` + universal constraints

**Common entity overlaps:** Memory and session composites include `data_object`; `control_plane_object` includes `data_object + identity_principal`

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Memory: `store`, `retain`, `recall`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `tag_predicate` (the selector form is also the primary constraint-shaped expression) + universal constraints

**Dual role:** `tag` serves two functions:
1. **As a standalone fundamental** for composites that hold data (memory, session)
2. **As the structural substrate for every composite** (via the `#kind:` tag machinery)

**Common entity overlaps:** Every composite implicitly includes `tag` for its `#kind:` identity — even composites without `data_object`

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Configuration: `configure`, `install`, `enable`, `disable`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** Universal constraints only

**Common entity overlaps:** Delegation (spawning a sub-agent) involves `identity_principal` + the sub-agent's transitive reach; `control_plane_object` includes `data_object + identity_principal`

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Economic: `spend`, `reserve`, `exceed`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_spend` + universal constraints

**Common entity overlaps:** `model/runtime_object` includes `economic_resource` because LLM calls cost tokens

**Worked example grant:**
```yaml
grant:
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
- Authority: `delegate`, `approve`, `escalate`, `allocate`, `transfer`
- Configuration: `configure`, `install`, `enable`, `disable`
- Economic: `spend`, `reserve`, `exceed`
- Observability: `observe`, `log`, `attest`

**Applicable constraints:** `max_spend`, `timeout_secs` + universal constraints

**Common entity overlaps:** Process execution typically reaches `time/compute_resource` transitively

**Worked example grant:**
```yaml
grant:
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
grant:
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
grant:
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
grant:
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

**Full details:** see [Memory as a Resource Class](05-memory-sessions.md#memory-as-a-resource-class) below for tag vocabulary, lifecycle, default grants, and supervisor extraction.

**Composite-specific rules:** Every Memory node carries `#kind:memory`. Agent chooses tags at creation; tags are mutable within the memory's lifetime. Multi-Scope resolution does NOT apply to memory (that's session-specific).

**Implicit `#kind:memory` tag** is added to every Memory node at creation time. Grants targeting `memory_object` carry an implicit `tags contains #kind:memory` selector refinement. See [Composite Identity Tags](01-resource-ontology.md#composite-identity-tags-kind) for the explanation of why implicit kind filters exist.

#### `session_object`

**Expansion:** `data_object` + (implicit) `tag` + `#kind:session` + {session tag vocabulary + frozen-at-creation + Multi-Scope resolution}
**What it covers:** Tagged execution records — sessions, loops, turns, messages (messages inherit session permissions)

**Full details:** see [Sessions as a Tagged Resource](05-memory-sessions.md#sessions-as-a-tagged-resource) below for tag vocabulary, lifecycle, Authority Templates, default grants, and Multi-Scope Session Access.

**Composite-specific rules:** Every Session node carries `#kind:session`. Tags are assigned by the system at creation and frozen (except `#archived`/`#active`). Multi-Scope Session Access resolution applies to reads.

**Implicit `#kind:session` tag** is added to every Session node at creation time. Grants targeting `session_object` carry an implicit `tags contains #kind:session` selector refinement. See [Composite Identity Tags](01-resource-ontology.md#composite-identity-tags-kind) for the explanation.

---
