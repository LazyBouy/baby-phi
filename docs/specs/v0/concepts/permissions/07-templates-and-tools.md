<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Standard Permission Templates

> **Purpose:** Provide ready-to-use template sets that an org can adopt as a starting point. Without these, every new org has to author every grant from scratch — which is both error-prone and slow. With them, an org can pick a baseline and then customize.
>
> **Framing:** These templates are **explicitly customizable starting points**, not locked-in defaults. The system does NOT auto-apply them — an org admin (or the platform setup process) explicitly opts into a template, then customizes from there.

### Templates Are Pre-Authorized Allocations

Templates and the Auth Request mechanism are not two separate things — every template is **a pre-authorized allocation** expressed through the Auth Request mechanism itself. At template adoption time, an Auth Request with `scope: [allocate]` is created and approved (by the org admin for org-level templates, by the project lead for project-level templates). This single Auth Request covers **all future Grants the template will issue**, without requiring per-grant approval.

Every Grant fired by a template references the template's adoption Auth Request as its `provenance`. The authority chain traces:

```
Template-fired Grant (e.g., a Template A grant for a new project lead)
  │ provenance
  ▼
Template's adoption Auth Request (created when the org adopted Template A)
  │ approved by
  ▼
Org admin at adoption time
  │ whose own authority traces back to...
  ▼
... eventually the System Bootstrap Template at the root.
```

This has three important consequences:

1. **Efficiency** — per-event grants (e.g., every `HAS_LEAD` creation fires Template A) don't require new approvals. Template adoption is the approval.
2. **Auditability** — every Grant is still traceable to a human decision (the org admin adopting the template), even though individual Grants don't each carry a unique approval event.
3. **Revocation** — revoking a template's adoption Auth Request cascade-revokes every Grant the template ever fired. This is how an org can cleanly disable a template.

**Default Grants** (those issued to every new agent at creation) are a special case of this: they are pre-authorized at platform setup time, via an adoption Auth Request for the "Default Grants template" held by the platform admin.

**Example — Standard Organization Template adoption:**

```yaml
# The adoption Auth Request — created when the org admin opts in
auth_request:
  request_id: req-adopt-std-org@acme
  requestor: agent:admin-acme
  kinds: [every registered composite]
  scope: [allocate]
  state: Approved
  resource_slots:
    - resource: org:acme
      approvers:
        - approver: agent:admin-acme
          state: Approved
          responded_at: 2026-04-01T09:00:00Z
  valid_until: null
  provenance: template:standard_organization@acme
  audit_class: logged
```

Every subsequent Template A, Template B, etc. Grant that fires in Acme references this adoption record as its `provenance`. No per-Grant approval is needed at runtime.

#### `audit_class` Composition Through Templates

When a template fires a Grant, three potential sources of `audit_class` can apply: (a) the **org-level default**, (b) the **template adoption Auth Request's** `audit_class`, and (c) any **per-Grant override** the template specifies. The composition rule:

1. **Strictest wins.** The effective `audit_class` is the strictest of (a), (b), and (c). The ordering from loosest to strictest is `none < logged < alerted`.
2. **Per-Grant overrides may only go stricter.** A template's per-Grant override can escalate to a stricter class (e.g., `logged → alerted` for sensitive sub-grants) but cannot loosen (e.g., it cannot set `alerted → logged` even if the org default is `logged`).
3. **Operators can always see what was applied.** The resolved `audit_class` is recorded on the Grant at issuance time, along with which of (a)/(b)/(c) supplied the winning value. This lets operators reason about why a given Grant shows up on their alerted queue even when they didn't explicitly configure it.

This mirrors the ceiling-intersection rule used elsewhere in the model: an org that opts into `alerted` for compliance reasons is guaranteed that adopting a template can never silently downgrade its audit posture. Conversely, an org with a loose `logged` default can adopt a stricter template (e.g., a "privileged-operations" template with `alerted`) and trust that its Grants will be audited accordingly. Either direction, the operator gets the tighter of the two, never the looser.

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

  # ── Resource Catalogue ─────────────────────────────────
  # The set of resource instances this org makes available to its projects
  # and agents. A resource must be in this catalogue to be referenced anywhere.
  # See 01-resource-ontology.md § Resource Catalogue for the full rule.
  resources_catalogue:
    # === Fundamentals (9) ===
    filesystem_objects:
      - path: /workspace/{project}/**
        default_owner: project
        description: Per-project workspace tree
      - path: /home/{agent}/**
        default_owner: agent
        description: Per-agent home directory
    process_exec_objects:
      - id: sandboxed-shell
        sandbox_requirement: required
    network_endpoints:
      - domain: api.anthropic.com
        purpose: LLM inference
      - domain: api.openai.com
        purpose: LLM inference
    secrets:
      - id: anthropic-api-key
        custodian: agent:platform-admin
      - id: openai-api-key
        custodian: agent:platform-admin
    data_objects: []                    # registered as needed; graph nodes
    tags: []                            # reserved + org-defined vocabularies
    identity_principals: []             # agents, users enumerated in agent_roster
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 0           # orgs configure their own budget
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 8
    # === Composites (8) ===
    memory_objects:
      - scope: per-agent
      - scope: per-project
      - scope: per-org
      - scope: '#public'
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []               # MCP servers etc; added via Template E
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
      - id: gpt-4o-default
        provider: openai
    control_plane_objects:
      - id: tool-registry
      - id: agent-catalogue              # used by agent-catalog-agent (see system_agents below)
      - id: policy-store
    auth_request_objects: []            # catalogue auto-contains all auth_requests
    inbox_objects: []                   # auto-created per agent at agent creation
    outbox_objects: []                  # same

  # ── System Agents ──────────────────────────────────────
  # Standard infrastructure agents instantiated at org adoption time.
  # See concepts/system-agents.md for full definitions.
  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 2                    # up to 2 concurrent session-end extractions
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change

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

#### Opt-in Example: Templates C, D, and E

The Standard Organization Template enables Templates A and B by default. Templates C, D, and E are **opt-in** — they fire only when an org explicitly adopts them. Full grant YAML for each of A–E lives in [Authority Templates (formerly the Authority Question)](05-memory-sessions.md#authority-templates-formerly-the-authority-question); this subsection shows the adoption-time config and when each is the right choice.

**Enabling Template C — Hierarchical Org Chart:**

```yaml
organization_template:
  name: acme-corporate
  inherits_from: standard
  authority_templates_enabled: [A, B, C]   # add C
  # Requires the org to model its hierarchy explicitly as org tree nodes
  # (e.g., org:acme/eng/web/lead, org:acme/eng/platform/lead, ...)
```

When to adopt Template C: the org wants **multi-level supervision without per-project role assignments** — a VP automatically sees their whole subtree. Use C when an explicit org chart exists and the CEO/VP/Director tier needs visibility by title rather than by per-project ask.

**Enabling Template D — Project-Scoped Role:**

```yaml
organization_template:
  name: privacy-focused-consultancy
  inherits_from: standard
  authority_templates_enabled: [D, E]   # drop A and B; use D + E only
  # Requires each project's HAS_AGENT edges to carry an explicit role property
  # (role: worker | supervisor | auditor | ...)
```

When to adopt Template D: the org wants **project-boundary-respecting supervision**. A supervisor on Project P sees P's worker sessions but *not* other supervisors' sessions in P or anything in other projects. This is the canonical choice for orgs with strong per-project boundaries (consultancies, regulated workloads, privacy-first teams).

**Enabling Template E — Explicit Capability:**

Always available; no opt-in needed. Use Template E for **one-off or time-bounded** grants: an external auditor reviewing a specific project for a week, a break-glass read during an incident, a specific cross-project handover. The flow is: requestor submits an Auth Request → approver (typically the owner of the target resource) approves → Grant issues with the same five-tuple shape as any other Grant, with `approval_mode: human_approved` and `audit_class: alerted` by default.

```yaml
# Example: time-bounded external auditor
grant:
  # subject = agent:external-auditor-9
  action: [read, inspect]
  resource:
    type: session_object
    selector: "tags contains project:website-redesign"
  constraints:
    purpose: compliance_audit
  provenance: auth_request:req-9231     # approved via the standard Auth Request flow
  delegable: false
  approval_mode: human_approved
  audit_class: alerted
  expires_at: 2026-04-22T00:00:00Z
  revocation_scope: manual
```

When to adopt Template E: every org. It's the escape hatch for cases the standing templates don't cover. Expect it to handle 1–5% of grants; anything more suggests a standing template would do the job better.

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

**Notes:** Session reads plug into the Consent Policy machinery via `approval_mode: inherit_from_grant`. If the agent's session grant carries `subordinate_required`, each read triggers the consent flow (see [Consent Policy](06-multi-scope-consent.md#consent-policy-organizational)).

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

### 13. `request_grant` — Tool for creating Auth Requests

```yaml
tool: request_grant
manifest:
  resource: auth_request_object   # composite: expands to data_object + tag
  actions: [create]
  constraints:
    tag_predicate: required       # caller specifies the target resources via tag predicates
  kind: [auth_request]
  target_kinds: [auth_request]    # this tool creates a single Auth Request instance per call
  transitive: []                  # creating an Auth Request does not itself reach other resources
  delegable: true                 # any agent can request permissions for themselves
  approval: auto                  # creating the request is auto-approved; the request itself
                                  # then goes through the normal approval flow
```

**Notes:** The `request_grant` tool lets an agent initiate an Auth Request workflow. Creating the request is distinct from approving it — this tool's invocation is always allowed for agents with a basic default grant (since any agent should be able to ask for more access); the substantive authorization question is whether the request is then approved by the resource owner. This is the standard tool an agent uses when it needs access beyond what it currently holds.

### 14. `workspace_snapshot` — Multi-composite reader

Tools often touch more than one composite in a single invocation. `workspace_snapshot` is the canonical example: it builds a point-in-time snapshot of an agent's current workspace by reading the project's files, the agent's recent memories, and the active session's transcript — three different composites in one call.

```yaml
tool: workspace_snapshot
manifest:
  resource: [filesystem_object, memory_object, session_object]
  actions: [read, list, inspect]
  constraints:
    tag_predicate: required         # caller specifies the project/session/agent scopes
    path_prefix: "/workspace/{project_id}/**"   # applies to the filesystem_object reach only
  kind: [memory, session]           # union of composite kinds this tool touches
  target_kinds: [memory, session]   # this tool resolves specific memory/session instances
                                    # via {kind}:{id} selectors
  transitive: []                    # no indirect reach beyond the three composites above
  delegable: false                  # snapshots carry sensitive aggregate info; not re-grantable
  approval: auto
```

**Notes on multi-composite manifests:**

- **`resource` lists all the composite resource types the tool can reach.** `filesystem_object` is a bare fundamental (no composite overlay), while `memory_object` and `session_object` are composites defined in [01-resource-ontology.md](01-resource-ontology.md#composite-classes-5). The validator expands each listed composite into its fundamentals and checks that the tool's actions are applicable to every expanded fundamental.
- **`kind` unions the composite kinds the tool operates on.** Here `[memory, session]` — `filesystem_object` has no `#kind:` because it's a bare fundamental, not a composite.
- **`target_kinds` lists only the composites the tool resolves to specific instances.** The filesystem reach is selector-based (`path_prefix`), so filesystem is absent from `target_kinds`.
- **Constraints can be per-fundamental.** `path_prefix` only applies to the filesystem reach; the tag predicate applies to the memory and session reaches. The manifest does not need to repeat a constraint that naturally applies to only one fundamental — the validator knows which constraint attaches to which fundamental from the Constraint × Fundamental matrix.

**What the caller's grant set must contain** for this tool to pass the Permission Check at invocation time:

| Reach | Required grant shape |
|-------|----------------------|
| Filesystem | A `[read, list]` grant on `filesystem_object` whose selector covers the path under `/workspace/{project_id}/**` |
| Memory | A `[read]` (a.k.a. `recall`) grant on `memory_object` whose tag predicate covers the memories the snapshot will include |
| Session | A `[read, inspect]` grant on `session_object` whose tag predicate covers the specific session instance (`session:{id}`) |

If any one of the three is missing, the Permission Check denies the invocation and the caller sees which reach failed. This illustrates why multi-composite tools require **every** declared reach to be separately granted — there is no "partial execute." The tool is all-or-nothing.

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
| Creates an Auth Request | `data_object` + `tag` | `#kind: auth_request` | Composite `auth_request_object` |
| Reads or inspects an Auth Request | `data_object` + `tag` | `#kind: auth_request` | Same |
| Transfers ownership of a resource | `data_object` + `tag` | `#kind: auth_request` (the TransferRecord rides an Auth Request) | Ownership change |

**How to use this table:** For each side effect, look it up and union the results. The resulting manifest is **complete by construction** — if you follow the table honestly, the publish-time validator will accept the manifest.

**Composite shorthand (optional):** you may use composite labels in the `resource` field of your manifest for readability (e.g., `resource: external_service_object`). You must still declare the `#kind:` explicitly — the composite label is documentation sugar, while `#kind:` is security-relevant.

### Self-Check Protocol (6 Steps)

1. **Enumerate every side effect.** List every externally-visible action: disk, network, credentials, process spawns, graph reads/writes, memory, sessions, delegation, budget, compute.
2. **Map each side effect to fundamentals** using the table above. Union the results.
3. **Identify every composite kind your tool touches.** For each composite (memory, session, external_service, model_runtime, control_plane, auth_request), ask: *does my tool actually operate on that composite kind?* If yes, declare it in `#kind:`. If all kinds, use `#kind: *` (accepts the warning). If none, omit `#kind:`.
3a. **Identify whether your tool addresses single instances or sets of instances.**
   - **Single instance** (e.g., `read_session(session_id)`, `inspect_auth_request(request_id)`): declare `target_kinds: [...]` listing the composite kinds the tool resolves to a specific instance via the `{kind}:{id}` selector convention (see [Instance Identity Tags](01-resource-ontology.md#instance-identity-tags-kindid)).
   - **Set of instances** (e.g., a bulk scanner that operates on whatever the grant covers): leave `target_kinds` absent; the tool operates on whatever set the caller's grant covers via scope tags.
   - **Both**: list the kinds in `target_kinds`; the tool may handle either selector shape.
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
- **Reserved tag namespaces**: manifests that declare `[modify]` on `tag` with a selector matching `#kind:*`, `{kind}:*` (for any registered composite kind, e.g. `session:*`, `memory:*`, `auth_request:*`), `delegated_from:*`, or `derived_from:*` are **rejected**. These namespaces are runtime-assigned and cannot be written by tools.
- **`target_kinds` consistency warning**: a manifest that declares `[create]` on a composite without a corresponding `target_kinds:` entry triggers a warning — this usually indicates the tool creates instances of that kind, in which case the runtime needs to know which kind to assign the self-identity tag correctly.

**Planned enhancements `[PLANNED]`** (not in v0):

- **Static analysis** of tool source code to verify manifest declarations match actual behavior (per-language tooling required)
- **Test-harness observation** of tool's runtime syscalls in a sandbox (most accurate but heaviest)
- **Continuous validation** on every code change (CI integration)
- **Cross-tool conflict detection** for tools whose combined permissions would exceed org ceilings

For v0, declaration-only validation catches the most common errors (missing `#kind:`, mismatched fundamentals, bad action/resource pairings). Static analysis and test-harness are future enhancements.

---
