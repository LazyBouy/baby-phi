<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 06 — `joint-venture-acme`

## Profile

The **Acme half** of a joint venture with Beta (`07-joint-venture-beta.md`). The two orgs co-own the project `joint-research` (Shape B in [permissions/06 § Multi-Scope Session Access](../concepts/permissions/06-multi-scope-consent.md#multi-scope-session-access)). Acme contributes its agents, its resources, its consent policy. Acme runs `consent_policy: implicit` on its own projects — so when Acme agents work inside `joint-research`, Acme's side of the per-co-owner consent check is auto-satisfied, but Beta's (`one_time`) is not. This layout exercises the per-co-owner consent rule from one side; layout 07 exercises it from the other.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | `implicit` |
| Agent mix | CEO + 2 Leads + 6 Workers (mix) + `joint-acme-5` (co-owned project member) |
| Hierarchy | Flat (two-level: CEO → Leads → Workers) |
| Authority Templates | A + B |
| Audit posture | `logged` |
| Co-ownership | **`joint-research` co-owned with `joint-venture-beta`** |
| `parallelize` | 2 on workers, 1 on leads |
| System agents | default two |

## Narrative

Acme is a standalone mid-size org **plus** a co-ownership relationship with Beta on one project. Everything about Acme's own projects (`acme-internal-tools`, `acme-website-redesign`, `acme-api-platform`) is self-contained: `implicit` consent, Templates A + B, agents assigned by Acme's CEO.

The interesting part is `joint-research`. From Acme's side:

- The project is in Acme's `owned_projects` list **with a `co_owner` annotation**.
- `joint-acme-5` is Acme's agent assigned to this joint project. Their `base_organization` is Acme.
- When `joint-acme-5` runs a session tagged `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5]`, Shape B resolution from [permissions/06](../concepts/permissions/06-multi-scope-consent.md#co-ownership-multi-scope-session-access) applies.
- The ceiling intersection of Acme's + Beta's policies is the effective ceiling.
- Per-co-owner consent is evaluated independently: Acme's `implicit` → auto-acknowledged; Beta's `one_time` (see `07`) → must have a valid Consent record in `Acknowledged` state.

**This layout is the "cooperative partner" side** — Acme's own shape is simple and permissive. The complexity surfaces only at the joint project boundary, handled by the multi-scope machinery.

## Full YAML Config

```yaml
organization:
  org_id: acme
  name: "Acme Corporation"
  vision: "Ship useful products; partner well."
  mission: "Simple, productive, collaborative."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: implicit
  audit_class_default: logged

  # Co-ownership declaration — Acme declares it shares joint-research with Beta.
  co_owned_resources:
    - resource_ref: project:joint-research
      co_owners: [acme, beta-corp]
      notes: "Joint venture; see permissions/06 § Co-Ownership × Multi-Scope Session Access"

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/acme-internal-tools/**
        default_owner: project
      - path: /workspace/acme-website-redesign/**
        default_owner: project
      - path: /workspace/acme-api-platform/**
        default_owner: project
      - path: /workspace/joint-research/**
        default_owner: project              # co-owned project
      - path: /home/{agent}/**
        default_owner: agent
    process_exec_objects:
      - id: sandboxed-shell
        sandbox_requirement: required
    network_endpoints:
      - domain: api.anthropic.com
      - domain: api.openai.com
    secrets:
      - id: anthropic-api-key
        custodian: agent:ceo
      - id: openai-api-key
        custodian: agent:ceo
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 80_000_000
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 24
    memory_objects:
      - scope: per-agent
      - scope: per-project
      - scope: per-org
      - scope: '#public'
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
      - id: gpt-4o-default
        provider: openai
    control_plane_objects: [tool-registry, agent-catalogue, policy-store]
    auth_request_objects: []
    inbox_objects: []
    outbox_objects: []

  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 2
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change

  authority_templates_enabled: [A, B]

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
  tool_constraints:
    bash: { sandbox_requirement: required, timeout_secs: 120 }

  execution_limits:
    max_turns: 60
    max_tokens: 150_000
    max_duration_secs: 5400
    max_cost_usd: 5.00

  agent_roster:
    - id: ceo
      kind: Human
      role: sponsor
      channels: [slack, email]

    - id: lead-acme-1
      kind: Contract
      profile: { name: lead-acme-1, system_prompt: "Lead of acme-website-redesign.", thinking_level: high, temperature: 0.3, personality: directive }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 1

    - id: lead-acme-3
      kind: Contract
      profile: { name: lead-acme-3, system_prompt: "Lead of acme-api-platform.", thinking_level: high, temperature: 0.3, personality: directive }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 1

    - id: coder-acme-2
      kind: Intern
      profile: { name: coder-acme-2, system_prompt: "Coder on acme-internal-tools.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: coder-acme-3
      kind: Contract
      parallelize: 2

    - id: coder-acme-4
      kind: Contract
      parallelize: 2

    - id: coder-acme-5
      kind: Contract
      parallelize: 2

    - id: auditor-acme-1
      kind: System                          # platform monitoring agent beyond the default two
      profile: { name: auditor-acme-1, system_prompt: "Acme-internal platform monitoring. Not compliance — just uptime and cost.", thinking_level: low, temperature: 0.0 }
      model_config: { provider: anthropic, model: claude-haiku-default, max_tokens: 4096 }
      execution_limits: { max_turns: 10, max_tokens: 40_000, max_duration_secs: 600, max_cost_usd: 0.50 }
      parallelize: 1
      trigger: periodic

    # joint-acme-5: Acme's contribution to joint-research.
    - id: joint-acme-5
      kind: Contract
      profile: { name: joint-acme-5, system_prompt: "Contract agent on the joint-research project. base_org = acme.", thinking_level: high, temperature: 0.3, personality: cooperative }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 2
      base_organization: acme
      current_project: joint-research
      co_owned_project_member: true         # signals Shape B participation

  hierarchy:
    task:
      assigns:
        - from: ceo
          to: [lead-acme-1, lead-acme-3, joint-acme-5]
        - from: lead-acme-1
          to: [coder-acme-2, coder-acme-3]
        - from: lead-acme-3
          to: [coder-acme-4, coder-acme-5]
    sponsorship:
      sponsors_projects:
        - sponsor: ceo
          projects:
            - acme-internal-tools
            - acme-website-redesign
            - acme-api-platform
            - joint-research           # co-owned — see layout 07 for Beta's side

  owned_projects: [acme-internal-tools, acme-website-redesign, acme-api-platform, joint-research]
```

## Cross-References

- [07-joint-venture-beta.md](07-joint-venture-beta.md) — **required companion reading** to see the other side of the joint venture.
- [concepts/permissions/06 § Co-Ownership × Multi-Scope Session Access](../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access) — the six rules that govern joint projects.
- [concepts/permissions/06 § Worked Example — Intersection on Ceilings, Union on Grants](../concepts/permissions/06-multi-scope-consent.md#worked-example--intersection-on-ceilings-union-on-grants) — a concrete bash-allowed-vs-forbidden scenario across this org and Beta.
- [projects/03-joint-research.md](../projects/03-joint-research.md) — the actual co-owned project.
