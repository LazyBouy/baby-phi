<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 07 — `joint-venture-beta`

## Profile

The **Beta half** of the joint venture with Acme (`06-joint-venture-acme.md`). Same co-owned project (`joint-research`), **different consent policy**: Beta runs `one_time`. This is what makes the per-co-owner consent rule interesting — when an Acme supervisor wants to read `joint-acme-5`'s session inside `joint-research`, Acme's `implicit` side is auto-satisfied but Beta's `one_time` side requires an explicit `Consent` record. Beta also forbids `bash` entirely (different tool posture from Acme), so the ceiling intersection forbids `bash` inside `joint-research`.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | **`one_time`** (vs Acme's `implicit`) |
| Agent mix | CEO + 1 Lead + 3 Workers + `joint-beta-3` (co-owned project member) |
| Hierarchy | Flat |
| Authority Templates | A + B |
| Audit posture | `logged` |
| Co-ownership | **`joint-research` co-owned with `joint-venture-acme`** |
| Tools | **`bash` not in Beta's allowlist** (Beta-wide policy, not just joint project) |
| `parallelize` | 2 on workers, 1 on lead |
| System agents | default two |

## Narrative

Beta is a smaller org than Acme — one lead, three workers, one joint-project agent. The important differences from Acme:

1. **`consent_policy: one_time`**. Every Beta subordinate acknowledges once at onboarding. For agents working inside `joint-research`, this acknowledgement applies equally. Acme supervisors reading Beta's joint-project sessions are gated by the existence of a valid Beta-scoped Consent record on the target agent — they don't bypass Beta's rules just because Acme's are more permissive. See [permissions/06 § rule 6](../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access).

2. **`bash` entirely removed**. Beta's security posture forbids shell access. This is a Beta-wide choice, but the effect on the joint project is the interesting bit: the ceiling intersection rule (see [permissions/06 § Worked Example](../concepts/permissions/06-multi-scope-consent.md#worked-example--intersection-on-ceilings-union-on-grants)) means `bash` is forbidden in `joint-research` too, even though Acme allows it on its own projects. `joint-acme-5` gets `bash` in Acme's own projects but not in `joint-research`.

3. **`joint-beta-3`** is Beta's contribution to the joint project, mirroring `joint-acme-5`. Their `base_organization` is `beta-corp`.

**Reading `06` and `07` together** is the right way to understand joint projects: the co-owned resources live in both orgs' catalogues (atomically — the same resource, one declaration per owner); the Auth Request slot machinery handles approval from each side; the per-co-owner consent and ceiling-intersection rules kick in when sessions carry both orgs' tags.

## Full YAML Config

```yaml
organization:
  org_id: beta-corp
  name: "Beta Corporation"
  vision: "Small, focused, secure."
  mission: "Quality over quantity. Every access is logged and acknowledged."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: one_time               # <-- distinguishing
  audit_class_default: logged

  co_owned_resources:
    - resource_ref: project:joint-research
      co_owners: [acme, beta-corp]
      notes: "Joint venture; mirror declaration from 06-joint-venture-acme.md"

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/beta-data-pipeline/**
        default_owner: project
      - path: /workspace/beta-mobile-app/**
        default_owner: project
      - path: /workspace/beta-billing/**
        default_owner: project
      - path: /workspace/joint-research/**
        default_owner: project             # co-owned
      - path: /home/{agent}/**
        default_owner: agent
    process_exec_objects: []               # NO sandboxed-shell — Beta forbids shell entirely
    network_endpoints:
      - domain: api.anthropic.com
    secrets:
      - id: anthropic-api-key
        custodian: agent:beta-ceo
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 30_000_000
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 12
    memory_objects:
      - scope: per-agent
      - scope: per-project
      - scope: per-org
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
      parallelize: 1                        # smaller org
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
    - http_get
    - delegate_task
    - request_grant
    - send_message
    # bash DELIBERATELY NOT IN ALLOWLIST — Beta-wide policy
  tool_constraints:
    http_get:
      allowed_domains: [api.anthropic.com]

  execution_limits:
    max_turns: 50
    max_tokens: 120_000
    max_duration_secs: 4500
    max_cost_usd: 4.00

  agent_roster:
    - id: beta-ceo
      kind: Human
      role: sponsor
      channels: [slack, email]

    - id: lead-beta-1
      kind: Contract
      profile: { name: lead-beta-1, system_prompt: "Lead of beta-billing.", thinking_level: high, temperature: 0.3, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 7.00 }
      parallelize: 1

    - id: coder-beta-2
      kind: Intern
      profile: { name: coder-beta-2, system_prompt: "Coder on beta-data-pipeline.", thinking_level: medium, temperature: 0.3, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 2.00 }
      parallelize: 2

    - id: coder-beta-4
      kind: Contract
      parallelize: 2

    - id: coder-beta-5
      kind: Contract
      parallelize: 2

    - id: monitor-beta-1
      kind: System                          # Beta-internal platform monitor
      profile: { name: monitor-beta-1, system_prompt: "Beta-internal platform monitor. Cost + uptime.", thinking_level: low, temperature: 0.0 }
      model_config: { provider: anthropic, model: claude-haiku-default, max_tokens: 4096 }
      execution_limits: { max_turns: 10, max_tokens: 40_000, max_duration_secs: 600, max_cost_usd: 0.50 }
      parallelize: 1
      trigger: periodic

    # joint-beta-3: Beta's contribution to joint-research.
    - id: joint-beta-3
      kind: Contract
      profile: { name: joint-beta-3, system_prompt: "Contract agent on the joint-research project. base_org = beta-corp.", thinking_level: high, temperature: 0.3, personality: cooperative }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 7.00 }
      parallelize: 2
      base_organization: beta-corp
      current_project: joint-research
      co_owned_project_member: true

  hierarchy:
    task:
      assigns:
        - from: beta-ceo
          to: [lead-beta-1, joint-beta-3]
        - from: lead-beta-1
          to: [coder-beta-2, coder-beta-4, coder-beta-5]
    sponsorship:
      sponsors_projects:
        - sponsor: beta-ceo
          projects:
            - beta-data-pipeline
            - beta-mobile-app
            - beta-billing
            - joint-research           # co-owned — see 06 for Acme's side

  owned_projects: [beta-data-pipeline, beta-mobile-app, beta-billing, joint-research]
```

## Cross-References

- [06-joint-venture-acme.md](06-joint-venture-acme.md) — the partner side of this joint venture.
- [concepts/permissions/06 § rule 6 — per-co-owner consent](../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access) — the rule Beta's `one_time` policy exercises.
- [projects/03-joint-research.md](../projects/03-joint-research.md) — the joint project.
