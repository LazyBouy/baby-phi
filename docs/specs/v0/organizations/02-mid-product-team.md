<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 02 — `mid-product-team`

## Profile

A mid-sized product team: 1 CEO (Human), 3 Team Leads (Contract agents), ~10 Workers (mix of Intern and Contract LLM Agents). Each lead owns one project stream; workers are assigned to leads. `one_time` consent policy reduces per-session friction while still getting explicit subordinate acknowledgement once per org. Interns run at `parallelize: 2` so one promising agent can support two concurrent streams.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | `one_time` |
| Agent mix | 1 Human CEO + 3 Contract Leads + 10 Workers (5 Intern + 5 Contract) |
| Hierarchy | Two-level: CEO → 3 Leads → Workers |
| Audit posture | `logged` (default) |
| Co-ownership | none |
| Market participation | none (workers assigned, not bid) |
| Inbox/Outbox usage | light — peer messaging between leads |
| `parallelize` | 2 on interns; 1 on contracts; 1 on CEO (human) |
| System agents | default two (memory extraction at `parallelize: 2`) |
| Authority Templates | A + B (standard) |

## Narrative

`mid-product-team` is the canonical "productive team" org. The CEO sponsors three projects, each with a dedicated lead. Leads own task hierarchies within their projects; workers receive tasks from their lead and don't cross streams except via inbox/outbox messaging.

**`one_time` consent** is the right fit: the team is big enough that `per_session` consent creates too much paperwork, but small enough that `implicit` (auto-acknowledged) consent feels too light. Each worker acknowledges once at onboarding; their consent then applies org-wide until revoked.

**Interns at `parallelize: 2`** lets a promising new agent work on two concurrent tasks without waiting for one to close. Contracts stay at `parallelize: 1` by default — each contract agent is a billed participant, so the org keeps their focus singular unless they explicitly negotiate higher concurrency. The memory extraction agent runs at `parallelize: 2` to keep up with end-of-session events across the ~13 agents.

**Template A + B** is the standard combination: leads see their team's sessions (Template A), and delegated sub-sessions are readable by the delegator (Template B). No Template C (no org chart tree deep enough to warrant it) and no Template D (project-role-based supervision is covered by Template A at this scale).

## Full YAML Config

```yaml
organization:
  org_id: mid-product-team
  name: "Acme Product Team"
  vision: "Ship three product streams concurrently in six-week cycles."
  mission: "Three leads, one team per lead, outcome-focused."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: one_time
  audit_class_default: logged

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/stream-a/**
        default_owner: project
      - path: /workspace/stream-b/**
        default_owner: project
      - path: /workspace/stream-c/**
        default_owner: project
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
        initial_allocation: 50_000_000
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
    control_plane_objects:
      - id: tool-registry
      - id: agent-catalogue
      - id: policy-store
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
    bash:
      sandbox_requirement: required
      timeout_secs: 120

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

    - id: lead-stream-a
      kind: Contract
      profile:
        name: lead-stream-a
        system_prompt: "Lead agent for Stream A. Own end-to-end delivery; delegate tactical tasks to workers."
        thinking_level: high
        temperature: 0.3
        personality: directive
      model_config:
        provider: anthropic
        model: claude-sonnet-default
        max_tokens: 8192
      execution_limits:
        max_turns: 80
        max_tokens: 200_000
        max_duration_secs: 7200
        max_cost_usd: 8.00
      parallelize: 1

    - id: lead-stream-b
      kind: Contract
      profile:
        name: lead-stream-b
        system_prompt: "Lead agent for Stream B."
        thinking_level: high
        temperature: 0.3
        personality: directive
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 1

    - id: lead-stream-c
      kind: Contract
      profile:
        name: lead-stream-c
        system_prompt: "Lead agent for Stream C."
        thinking_level: high
        temperature: 0.3
        personality: directive
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 1

    # Workers: 5 Intern + 5 Contract. Interns are parallelize:2 (eager but unproven);
    # contracts run parallelize:1 (billed focus).
    - id: intern-a1
      kind: Intern
      profile: { name: intern-a1, system_prompt: "Worker on Stream A.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: intern-a2
      kind: Intern
      profile: { name: intern-a2, system_prompt: "Worker on Stream A.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: intern-b1
      kind: Intern
      profile: { name: intern-b1, system_prompt: "Worker on Stream B.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: intern-c1
      kind: Intern
      profile: { name: intern-c1, system_prompt: "Worker on Stream C.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: intern-c2
      kind: Intern
      profile: { name: intern-c2, system_prompt: "Worker on Stream C.", thinking_level: medium, temperature: 0.3, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: contract-a1
      kind: Contract
      profile: { name: contract-a1, system_prompt: "Experienced contract worker for Stream A.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 4000, max_cost_usd: 4.00 }
      parallelize: 1

    - id: contract-a2
      kind: Contract
      profile: { name: contract-a2, system_prompt: "Experienced contract worker for Stream A.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 4000, max_cost_usd: 4.00 }
      parallelize: 1

    - id: contract-b1
      kind: Contract
      profile: { name: contract-b1, system_prompt: "Experienced contract worker for Stream B.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 4000, max_cost_usd: 4.00 }
      parallelize: 1

    - id: contract-b2
      kind: Contract
      profile: { name: contract-b2, system_prompt: "Experienced contract worker for Stream B.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 4000, max_cost_usd: 4.00 }
      parallelize: 1

    - id: contract-c1
      kind: Contract
      profile: { name: contract-c1, system_prompt: "Experienced contract worker for Stream C.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 4000, max_cost_usd: 4.00 }
      parallelize: 1

  hierarchy:
    task:
      assigns:
        - from: ceo
          to: [lead-stream-a, lead-stream-b, lead-stream-c]
        - from: lead-stream-a
          to: [intern-a1, intern-a2, contract-a1, contract-a2]
        - from: lead-stream-b
          to: [intern-b1, contract-b1, contract-b2]
        - from: lead-stream-c
          to: [intern-c1, intern-c2, contract-c1]
    sponsorship:
      sponsors_projects:
        - sponsor: ceo
          projects: [stream-a, stream-b, stream-c]

  owned_projects: [stream-a, stream-b, stream-c]
```

## Cross-References

- [concepts/permissions/06 § One-Time Consent](../concepts/permissions/06-multi-scope-consent.md#one-time-consent).
- [concepts/permissions/05 § Authority Templates](../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) — Templates A + B usage.
- [concepts/agent.md § Parallelized Sessions](../concepts/agent.md#parallelized-sessions) — why interns get `parallelize: 2`.
- [concepts/agent.md § Intern Agent (Pre-Economy)](../concepts/agent.md#intern-agent-pre-economy) and [§ Contract Agent](../concepts/agent.md#contract-agent-token-economy-participant) — the two agent kinds in the mix.
