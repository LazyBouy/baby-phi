<!-- Status: CONCEPTUAL -->

# Permissions Model (Capability-Based)

> Extracted from brainstorm.md Section 4 + permission_model.md.
> See also: [agent.md](agent.md), [organization.md](organization.md) (resolution hierarchy), [project.md](project.md) (project-level rules)

---

## Core Insight

**Permissions are not about tools — they're about actions on resources with constraints.**

This mirrors capability-based security and cloud IAM: authority is tied to a specific object and operation, not ambient possession of a broad tool. Tools are merely *implementations* of actions on resources.

## Canonical Shape

```
Permission = ⟨ subject, action, resource, constraints, provenance ⟩
```

---

## Resource Ontology

Every authority surface in the system maps to one of these resource families:

| Resource Class | What It Covers | baby-phi Mapping |
|---|---|---|
| **Filesystem object** | Files, directories, repos, temp paths | Agent workspace, skill files |
| **Process/exec object** | Shell commands, binaries, containers | BashTool, script execution |
| **Network endpoint** | Domains, URLs, IPs, ports, APIs | Provider base_urls, MCP endpoints |
| **Data object** | Documents, tables, vector stores, transcripts | Sessions, Messages, Memory nodes |
| **Secret/credential** | API keys, tokens, certificates | ModelConfig.api_key, MCP auth |
| **Identity principal** | User identity, service account, role, session | Agent, HumanAgent, Organization |
| **External service object** | GitHub, Slack, Jira, cloud bucket | MCP servers, OpenAPI specs |
| **Model/runtime object** | Model endpoint, prompt templates, policies | ModelConfig, SystemPrompt, AgentProfile |
| **Control-plane object** | Tool registry, policy store, audit log | Schema registry, Permission nodes |
| **Communication object** | Email, chat thread, webhook, MCP channel | Channels (Human Agent routing) |
| **Economic resource** | Token budget, spend budget, rate limit | Token economy (Contract agents) |
| **Time/compute resource** | CPU time, duration, concurrency, memory | ExecutionLimits |

**Rule:** Every new integration must project its operations into this schema before it can be enabled.

---

## Standard Action Vocabulary

Reusable across all resource classes:

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

---

## Constraints

Each permission carries condition slots:

| Constraint | Example |
|------------|---------|
| Path prefix | `/workspace/project-a/**` |
| Command pattern | `cargo *`, `npm test` |
| Domain allowlist | `api.anthropic.com`, `*.openrouter.ai` |
| Data label | `own_sessions`, `project_shared` |
| Max spend | `10000 tokens per task` |
| Time window | `working_hours_only` |
| Approval requirement | `human_approval_required` |
| Sandbox requirement | `sandboxed_execution` |
| Non-delegability | `cannot_delegate` |
| Output channel | `slack_only`, `no_email` |

---

## Tool Authority Manifest

**Design rule:** Every tool must ship a machine-readable authority manifest declaring:
- Resource classes touched
- Actions performed
- Transitive resources consumed (e.g., `bash` can reach `network endpoint` transitively)
- Delegation behavior
- Approval defaults
- Required constraints

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
    - secret_credential       # can access env vars
  delegable: false            # too powerful to delegate
  approval: human_recommended
```

---

## Permission Resolution Hierarchy

When an agent operates within a project in an organization, permissions resolve top-down:

```
Organization config (highest authority)
    │ overrides ↓
Project config
    │ overrides ↓
Agent config (most specific)
```

**Rules:**
- **Org overrides project:** If an org restricts `network_endpoint` access, no project within it can grant it back.
- **Project overrides agent:** If a project restricts `bash` tool, no agent within it can use it.
- **Agent config is most specific:** Within the bounds set by org and project, the agent's own config determines fine-grained behavior.
- **Cross-org projects:** When a project spans multiple orgs, each org's restrictions apply independently — the *intersection* of all org policies is the effective ceiling.

**Delegation:** When Agent A delegates to Agent B, B inherits A's permission *ceiling* (never more than A has), further narrowed by B's own config.

---

## Permission as a Graph Node

```
Permission
  resource_type: String       -- e.g. "filesystem_object"
  resource_selector: String   -- e.g. "/workspace/project-a/**"
  action: Vec<String>         -- e.g. ["read", "modify"]
  constraints: Json           -- condition slots
  delegable: bool             -- can this be passed to sub-agents
  approval_mode: String       -- "auto", "human_required", "human_recommended"
  audit_class: String         -- "silent", "logged", "alerted"
  provenance: String          -- who granted this (agent_id or "system")
  revocation_scope: String    -- "immediate", "end_of_session", "manual"
```

**Edges:**
- `Agent ──HAS_PERMISSION──▶ Permission`
- `Project ──HAS_PERMISSION──▶ Permission` (project-level rules)
- `Organization ──HAS_PERMISSION──▶ Permission` (org-level ceiling)
- `Agent ──GRANTS_PERMISSION──▶ Permission` (provenance: who created it)

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