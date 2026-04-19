<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 03 — `consultancy-strict`

## Profile

A privacy-focused consultancy with hard project boundaries. Every project serves a different client; cross-project visibility is a bug, not a feature. `per_session` consent means every access to a worker's session is logged and (if configured) requires the worker to acknowledge. `bash` is **entirely removed** from the tool allowlist — not sandboxed, not restricted, *removed*. Templates A and B are disabled in favour of Template D (project-scoped role), so supervisors see only their own project's workers.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | **`per_session`** |
| Agent mix | 1 Managing Partner (Human) + 4 Senior Contractors + 8 Contract Workers |
| Hierarchy | Flat inside each project; hard boundaries between projects |
| Audit posture | `logged` default, but `alerted` on any cross-project read attempt |
| Authority Templates | **D only** (A, B, C disabled; E available as always) |
| Tools | **`bash` removed**; `http_get` scoped to an allowlist |
| `ExecutionLimits` | Tighter than default (cost caps per session) |
| Co-ownership | none |
| `parallelize` | `1` everywhere |

## Narrative

The defining characteristic is **strict project isolation**. A worker assigned to Client A's project has zero visibility into Client B's project — no shared resources, no session cross-reads, no inbox messages from anyone outside their project.

**`per_session` consent** means the worker explicitly grants supervision rights for each session, or the supervisor's Template D grant is denied. The administrative overhead is deliberate: clients pay for this posture. See [permissions/06 § Per-Session Consent](../concepts/permissions/06-multi-scope-consent.md#per-session-consent) and the `project_duration` timeout rule.

**Template D only** (project-scoped role) scopes supervisor access to worker sessions *within the same project*. Templates A (project lead → all sessions in their project) and B (delegator → delegated sessions) are disabled because they tempt supervisors into ambient access patterns that violate client boundaries. Template E (explicit manual grant) remains available for the rare break-glass case, but those always go through an Auth Request and carry `audit_class: alerted`.

**`bash` removed** is the one policy that surprises newcomers most. It is removed not sandboxed. A consultancy that runs arbitrary shell commands on its workers' environments is one `rm` from losing client data; the cleaner answer is to never make `bash` available. Workers get `read_file`, `write_file`, `edit_file`, `search`, `http_get` (allowlisted), and `recall_memory` / `store_memory`. Anything `bash` could do is either done via a specific narrower tool or is not done.

## Full YAML Config

```yaml
organization:
  org_id: consultancy-strict
  name: "Meridian Consulting"
  vision: "Trusted confidential advisor to our clients."
  mission: "Client isolation is the core service."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: per_session
  audit_class_default: logged
  audit_class_on_cross_project_read_attempt: alerted

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/client-{client_id}/**
        default_owner: project
        description: Per-client workspace; strict boundary
      - path: /home/{agent}/**
        default_owner: agent
    process_exec_objects: []              # no process execution (bash removed)
    network_endpoints:
      - domain: api.anthropic.com
      - domain: public-research-apis.org  # allowlisted client-agnostic research
    secrets:
      - id: anthropic-api-key
        custodian: agent:managing-partner
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 30_000_000
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 16
    memory_objects:
      - scope: per-agent
      - scope: per-project                # memory pooled at project, never org-wide
      # NO per-org memory — no cross-client pooling
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
    control_plane_objects: [tool-registry, agent-catalogue, policy-store]
    auth_request_objects: []
    inbox_objects: []
    outbox_objects: []

  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 1                      # per-session consent + per-project scope reduces throughput need
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change

  authority_templates_enabled: [D]        # D only; E always available, A/B/C disabled
  authority_template_notes:
    A_disabled: "Project leads do NOT get ambient session access — must use Template E or D."
    B_disabled: "Delegated sessions are not auto-readable by the delegator."
    C_disabled: "No org chart — no cross-project supervision."

  tools_allowlist:
    - read_file
    - write_file
    - edit_file
    - search
    - http_get
    - recall_memory
    - store_memory
    - send_message
    - request_grant
    # bash EXPLICITLY NOT IN ALLOWLIST
  tool_constraints:
    http_get:
      allowed_domains: [public-research-apis.org]

  execution_limits:
    max_turns: 40
    max_tokens: 80_000
    max_duration_secs: 2400
    max_cost_usd: 2.50

  agent_roster:
    - id: managing-partner
      kind: Human
      role: sponsor
      channels: [slack, email]

    - id: senior-1
      kind: Contract
      profile: { name: senior-1, system_prompt: "Senior consultant. Lead of Client A engagement.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 3600, max_cost_usd: 4.00 }
      parallelize: 1
      project_role: { client-a: supervisor }

    - id: senior-2
      kind: Contract
      profile: { name: senior-2, system_prompt: "Senior consultant. Lead of Client B engagement.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 3600, max_cost_usd: 4.00 }
      parallelize: 1
      project_role: { client-b: supervisor }

    - id: senior-3
      kind: Contract
      profile: { name: senior-3, system_prompt: "Senior consultant. Lead of Client C engagement.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 3600, max_cost_usd: 4.00 }
      parallelize: 1
      project_role: { client-c: supervisor }

    - id: senior-4
      kind: Contract
      profile: { name: senior-4, system_prompt: "Senior consultant. Lead of Client D engagement.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 60, max_tokens: 120_000, max_duration_secs: 3600, max_cost_usd: 4.00 }
      parallelize: 1
      project_role: { client-d: supervisor }

    # 8 Contract workers, each bound to exactly one project.
    # (profile/model/limits identical shape; showing one in full, rest as refs)
    - id: worker-a1
      kind: Contract
      profile: { name: worker-a1, system_prompt: "Consultant worker on Client A.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.50 }
      parallelize: 1
      project_role: { client-a: worker }

    - id: worker-a2
      kind: Contract
      project_role: { client-a: worker }
      # ... identical profile/model/limits to worker-a1 except the assigned project

    - id: worker-b1
      kind: Contract
      project_role: { client-b: worker }

    - id: worker-b2
      kind: Contract
      project_role: { client-b: worker }

    - id: worker-c1
      kind: Contract
      project_role: { client-c: worker }

    - id: worker-c2
      kind: Contract
      project_role: { client-c: worker }

    - id: worker-d1
      kind: Contract
      project_role: { client-d: worker }

    - id: worker-d2
      kind: Contract
      project_role: { client-d: worker }

  hierarchy:
    task:
      assigns:
        - from: managing-partner
          to: [senior-1, senior-2, senior-3, senior-4]
        - from: senior-1
          to: [worker-a1, worker-a2]
        - from: senior-2
          to: [worker-b1, worker-b2]
        - from: senior-3
          to: [worker-c1, worker-c2]
        - from: senior-4
          to: [worker-d1, worker-d2]
    sponsorship:
      sponsors_projects:
        - sponsor: managing-partner
          projects: [client-a, client-b, client-c, client-d]

  owned_projects: [client-a, client-b, client-c, client-d]
```

## Cross-References

- [concepts/permissions/06 § Per-Session Consent](../concepts/permissions/06-multi-scope-consent.md#per-session-consent).
- [concepts/permissions/05 § Template D — Project-Scoped Role](../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question).
- [concepts/permissions/07 § Opt-in Example: Template D](../concepts/permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e) — template adoption shape.
- [concepts/agent.md § Contract Agent](../concepts/agent.md#contract-agent-token-economy-participant).
