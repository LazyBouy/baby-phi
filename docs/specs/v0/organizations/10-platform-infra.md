<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 10 — `platform-infra`

## Profile

A **platform infrastructure organization** — the custodian of shared infrastructure that other orgs depend on: MCP server registrations, external-service credentials, the token-economy operations, cross-org billing, and monitoring. Its resource catalogue **includes composites that other orgs reference** — specifically, `external_service_object` instances (registered MCP servers), `model/runtime_object` instances (managed LLM provider bindings), and a platform-wide `economic_resource` (the aggregate token pool). Agents run at **`parallelize: 8`** because infrastructure work is high-throughput and mostly independent events.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | `implicit` (custodial agents don't need per-session consent for infra ops) |
| `audit_class_default` | **`alerted`** on any modification; `logged` on reads |
| Agent mix | 1 Platform CEO (Human) + 4 Platform Engineers (Contract) + 6 System Agents (extras) |
| Hierarchy | Flat |
| Authority Templates | A + E (A for platform engineers seeing platform sessions; E for every cross-org grant) |
| Inbox/Outbox | medium — announcements to tenant orgs |
| `parallelize` | **8** on platform engineers; 4 on high-throughput system agents |
| Shared resources | external_service_objects + model_runtime_objects + token pool |
| Tenants | other orgs consume this org's catalogue via cross-org grants |

## Narrative

The platform-infra org is structurally unlike the others: its **value is the resource catalogue itself**. Other orgs in the system **reference** its declared `external_service_object` instances (an org that wants to use `mcp-github` adds it to their own catalogue via a Template E Auth Request that references platform-infra's original registration as its provenance). Same for LLM provider bindings: a tenant org that wants `claude-sonnet-default` doesn't re-register with Anthropic — it references platform-infra's `model/runtime_object` entry.

**`audit_class: alerted` on modifications** is non-negotiable here. Every change to a shared resource (a new MCP server, a secret rotation, a token-pool rebalance) pages the platform on-call. Reads are `logged` (there's too much read traffic to alert on it).

**`parallelize: 8` on platform engineers** reflects the nature of the work: most operations are independent events (register a new MCP for tenant A, rotate a key for tenant B, adjust a rate limit for tenant C). Parallel throughput is the point.

**Agent catalog agent is cross-org here**. Unlike other orgs where agent-catalog-agent tracks the org's own agents, platform-infra's agent-catalog-agent also maintains a catalogue of **tenant-visible service agents** — the agents other orgs can query for platform services.

## Full YAML Config

```yaml
organization:
  org_id: platform-infra
  name: "Phi Platform Infrastructure"
  vision: "Boring, reliable infrastructure so tenant orgs can focus on their work."
  mission: "Every shared resource is auditable, revocable, and replaceable."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: implicit
  audit_class_default_reads: logged
  audit_class_default_writes: alerted       # non-standard split — writes get alerted

  resources_catalogue:
    filesystem_objects:
      - path: /platform/operations/**
        default_owner: org
    process_exec_objects:
      - id: platform-admin-shell
        sandbox_requirement: required
    network_endpoints:
      - domain: api.anthropic.com
      - domain: api.openai.com
      - domain: github.com
      - domain: slack.com
      - domain: platform-admin.phi-platform.internal
    secrets:
      - id: anthropic-platform-master-key
        custodian: agent:platform-admin
        tags: ['#sensitive']
      - id: openai-platform-master-key
        custodian: agent:platform-admin
        tags: ['#sensitive']
      - id: github-mcp-client-secret
        custodian: agent:platform-admin
        tags: ['#sensitive']
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: platform-wide-token-pool
        units: tokens
        initial_allocation: 10_000_000_000   # aggregate pool; tenants draw from this
        notes: "Tenant orgs reference this via cross-org grants; platform-infra rebalances."
    compute_resources:
      - id: shared-compute-pool
        max_concurrent_sessions: 512
    memory_objects:
      - scope: per-agent
      - scope: per-org                       # platform-internal notes
      - scope: '#public'                     # things tenants can read (docs, announcements)
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services:                       # CORE OFFERING — tenants reference these
      - id: mcp-github
        kind: external_service
        endpoint: mcp://github
        tenants_allowed: '*'                 # any tenant can reference with an approved Auth Request
      - id: mcp-slack
        kind: external_service
        endpoint: mcp://slack
        tenants_allowed: '*'
      - id: mcp-linear
        kind: external_service
        endpoint: mcp://linear
        tenants_allowed: '*'
      - id: mcp-jira
        kind: external_service
        endpoint: mcp://jira
        tenants_allowed: '*'
    model_runtime_objects:                   # CORE OFFERING — tenants reference these
      - id: claude-sonnet-default
        provider: anthropic
        tenants_allowed: '*'
      - id: claude-opus-default
        provider: anthropic
        tenants_allowed: '*'                 # premium
      - id: claude-haiku-default
        provider: anthropic
        tenants_allowed: '*'
      - id: gpt-4o-default
        provider: openai
        tenants_allowed: '*'
    control_plane_objects:
      - id: tool-registry
      - id: agent-catalogue
      - id: policy-store
      - id: tenant-registry                  # platform-infra-specific
      - id: quota-registry
    auth_request_objects: []
    inbox_objects: []
    outbox_objects: []

  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 4
      trigger: session_end

    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change
      notes: "Also maintains a cross-org index of tenant-visible service agents."

    # Six platform-specific system agents beyond the default two:
    - id: mcp-onboarding-agent
      profile_ref: platform-mcp-onboarding
      parallelize: 2
      trigger: auth_request_state_change
      profile:
        name: mcp-onboarding-agent
        system_prompt: "Handle tenant requests to reference a platform MCP server. Validate, approve or escalate, register in tenant's catalogue."
        thinking_level: medium
        temperature: 0.1
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 20, max_tokens: 50_000, max_duration_secs: 600, max_cost_usd: 1.00 }

    - id: secret-rotation-agent
      profile_ref: platform-secret-rotation
      parallelize: 4
      trigger: periodic
      notes: "Rotates shared credentials on a schedule; emits alerted audit events."

    - id: quota-enforcer-agent
      profile_ref: platform-quota-enforcer
      parallelize: 4
      trigger: token_pool_change
      notes: "Monitors per-tenant token burn; alerts on anomalies; can throttle."

    - id: billing-aggregator-agent
      profile_ref: platform-billing
      parallelize: 2
      trigger: periodic
      notes: "Aggregates per-tenant token spend monthly; produces invoices."

    - id: uptime-monitor-agent
      profile_ref: platform-uptime
      parallelize: 8                         # high-throughput event processing
      trigger: periodic
      notes: "Polls platform endpoints, emits alerts on degradation."

    - id: announcement-agent
      profile_ref: platform-announcement
      parallelize: 1
      trigger: explicit
      notes: "Sends platform-wide announcements to tenant org inboxes (maintenance windows, deprecations)."

  authority_templates_enabled: [A, E]
  # A for platform engineers seeing platform sessions; E for every cross-org grant to tenants.

  tools_allowlist:
    - read_file
    - write_file
    - edit_file
    - search
    - recall_memory
    - store_memory
    - bash
    - http_get
    - delegate_task
    - request_grant
    - send_message
    - register_mcp                           # platform-specific
    - rotate_secret                          # platform-specific
    - adjust_quota                           # platform-specific
    - announce                               # platform-specific
  tool_constraints:
    bash: { sandbox_requirement: required, timeout_secs: 300 }

  execution_limits:
    max_turns: 100
    max_tokens: 300_000
    max_duration_secs: 7200
    max_cost_usd: 15.00

  agent_roster:
    - id: platform-ceo
      kind: Human
      role: sponsor
      channels: [slack, email, pager]

    # 4 Platform Engineers at parallelize: 8
    - id: platform-eng-alpha
      kind: Contract
      profile: { name: platform-eng-alpha, system_prompt: "Platform engineer. Handle infrastructure tickets, tenant requests, and on-call.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 100, max_tokens: 300_000, max_duration_secs: 7200, max_cost_usd: 15.00 }
      parallelize: 8

    - id: platform-eng-beta
      kind: Contract
      parallelize: 8

    - id: platform-eng-gamma
      kind: Contract
      parallelize: 8

    - id: platform-eng-delta
      kind: Contract
      parallelize: 8

  hierarchy:
    task:
      assigns:
        - from: platform-ceo
          to: [platform-eng-alpha, platform-eng-beta, platform-eng-gamma, platform-eng-delta]
      notes: |
        Most 'tasks' at platform-infra arrive as Auth Requests from tenant orgs
        (e.g., "please register mcp-github for us"). mcp-onboarding-agent processes
        routine cases; exceptions escalate to a platform engineer via inbox.
    sponsorship:
      sponsors_projects:
        - sponsor: platform-ceo
          projects: [platform-ops, platform-onboarding, platform-incident-response]

  owned_projects: [platform-ops, platform-onboarding, platform-incident-response]

  tenants:
    # List of orgs that reference this org's shared resources.
    # Each tenancy is established via a Template E Auth Request.
    - tenant_org: minimal-startup
      allowed_references: [claude-sonnet-default]
    - tenant_org: mid-product-team
      allowed_references: [claude-sonnet-default, gpt-4o-default]
    - tenant_org: consultancy-strict
      allowed_references: [claude-sonnet-default]
    - tenant_org: regulated-enterprise
      allowed_references: [claude-sonnet-default, mcp-slack]
    # ... etc for all tenant orgs that take the platform's catalogue
```

## Cross-References

- [concepts/permissions/01 § Resource Catalogue](../concepts/permissions/01-resource-ontology.md#resource-catalogue) — the rule that lets tenant orgs reference platform-infra's catalogue entries.
- [concepts/permissions/07 § Template E](../concepts/permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e) — how cross-org grants work.
- [concepts/system-agents.md](../concepts/system-agents.md) — the default two + the six platform-specific extensions.
- [concepts/agent.md § Parallelized Sessions](../concepts/agent.md#parallelized-sessions) — why `parallelize: 8` is appropriate for platform engineers.
