<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 05 — `research-lab-nested`

## Profile

A research lab with **three levels of nested sub-organizations**: Lab → Group → Team. Each sub-org owns its own projects, agents, and resource catalogue; parent orgs set ceilings. Long-duration projects (multi-month or multi-year). Workers run at `parallelize: 4` so one well-performing profile can pursue multiple hypotheses simultaneously. Heavy dependence on the memory-extraction-agent — research sessions produce a lot of insight per run.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | `implicit` (collegial research culture) |
| Agent mix | Lab director + 3 group leads + 9 team leads + ~30 researchers (mostly Contract) |
| Hierarchy | **3 levels of `HAS_SUBORGANIZATION`**: lab → group → team |
| Authority Templates | A + B + C (C essential for nested supervision) |
| Audit posture | `logged` (default) |
| `parallelize` | **4** on researchers; 2 on team leads; 1 on directors |
| Memory extraction | runs at `parallelize: 4` |
| Long-duration projects | yes — project timeouts extend to months |
| Co-ownership | none internally; occasionally joint with other labs via Shape B |

## Narrative

The distinguishing feature is **deep nesting via `HAS_SUBORGANIZATION`**. The top-level Organization is "Phi Research Lab." Under it sit three Groups (Perception, Reasoning, Alignment). Each Group has three Teams. Each Team owns several projects.

**Inheritance through ceilings.** Each layer inherits ceilings from its parent. The Lab sets a ceiling on `token-budget-pool` total; each Group gets a sub-allocation; each Team gets a sub-sub-allocation. A Team cannot exceed its parent Group's ceiling, which cannot exceed the Lab's. The `resources_catalogue` flows downward too — a Team's catalogue is a subset of its Group's, which is a subset of the Lab's.

**`parallelize: 4` on researchers** is the most distinctive setting. Research workflows involve many independent hypotheses: an agent might run one session exploring approach A while another session pursues approach B and a third does a literature scan. The memory-extraction-agent pools learnings back into the researcher's `agent:` scope (and into `team:` and `group:` scopes) so that insights from one session inform the next.

**Long-duration projects.** The Auth Request `valid_until` defaults are extended (90 days → 1 year) for project-lifetime grants. Consent policy is `implicit` because the lab's culture trusts peer supervision; shifting to `one_time` or `per_session` would be friction without benefit at the collegial scale.

## Full YAML Config

```yaml
organization:
  org_id: phi-research-lab
  name: "Phi Research Lab"
  vision: "Understand and advance capable, aligned agent systems."
  mission: "Deep research with long horizons; trust researchers to run parallel hypotheses."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: implicit
  audit_class_default: logged

  # Sub-organization hierarchy (three levels).
  sub_organizations:
    - org_id: phi-perception
      parent: phi-research-lab
      sub_organizations:
        - org_id: phi-perception-vision-team
          parent: phi-perception
        - org_id: phi-perception-audio-team
          parent: phi-perception
        - org_id: phi-perception-multimodal-team
          parent: phi-perception
    - org_id: phi-reasoning
      parent: phi-research-lab
      sub_organizations:
        - org_id: phi-reasoning-planning-team
          parent: phi-reasoning
        - org_id: phi-reasoning-math-team
          parent: phi-reasoning
        - org_id: phi-reasoning-theory-team
          parent: phi-reasoning
    - org_id: phi-alignment
      parent: phi-research-lab
      sub_organizations:
        - org_id: phi-alignment-interpretability-team
          parent: phi-alignment
        - org_id: phi-alignment-evaluation-team
          parent: phi-alignment
        - org_id: phi-alignment-governance-team
          parent: phi-alignment

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/{project}/**
        default_owner: project
      - path: /datasets/{group}/**
        default_owner: group                # group-level shared datasets
      - path: /home/{agent}/**
        default_owner: agent
    process_exec_objects:
      - id: sandboxed-shell
        sandbox_requirement: required
      - id: gpu-compute
        sandbox_requirement: recommended
    network_endpoints:
      - domain: api.anthropic.com
      - domain: api.openai.com
      - domain: arxiv.org
      - domain: scholar.google.com
    secrets:
      - id: anthropic-api-key
        custodian: agent:lab-director
      - id: openai-api-key
        custodian: agent:lab-director
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 500_000_000    # lab-level; sub-allocated to groups
    compute_resources:
      - id: cpu-compute-pool
        max_concurrent_sessions: 128
      - id: gpu-compute-pool
        max_concurrent_sessions: 32
    memory_objects:
      - scope: per-agent
      - scope: per-team
      - scope: per-group
      - scope: per-lab
      - scope: '#public'
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services:
      - id: arxiv-mcp
        kind: external_service
        endpoint: mcp://arxiv
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
      parallelize: 4                       # research generates a lot of extractable insight
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change

  authority_templates_enabled: [A, B, C]   # C essential for nested supervision
  org_chart:
    - path: org:phi-research-lab/director
      occupant: lab-director
    - path: org:phi-research-lab/group-perception
      occupant: group-lead-perception
      reports_to: org:phi-research-lab/director
    - path: org:phi-research-lab/group-reasoning
      occupant: group-lead-reasoning
      reports_to: org:phi-research-lab/director
    - path: org:phi-research-lab/group-alignment
      occupant: group-lead-alignment
      reports_to: org:phi-research-lab/director

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
    - mcp_arxiv
  tool_constraints:
    bash: { sandbox_requirement: required, timeout_secs: 600 }

  execution_limits:
    max_turns: 100
    max_tokens: 400_000
    max_duration_secs: 14400
    max_cost_usd: 20.00

  auth_request_retention:
    active_window_days: 365                # long projects need long-active records
    archived_retrieval_approval: human_required

  agent_roster:
    - id: lab-director
      kind: Human
      role: sponsor
      channels: [slack, email]

    - id: group-lead-perception
      kind: Contract
      profile: { name: group-lead-perception, system_prompt: "Group lead, Perception.", thinking_level: high, temperature: 0.3, personality: directive }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 100, max_tokens: 250_000, max_duration_secs: 14400, max_cost_usd: 25.00 }
      parallelize: 1
      org_chart_path: org:phi-research-lab/group-perception

    - id: group-lead-reasoning
      kind: Contract
      parallelize: 1
      org_chart_path: org:phi-research-lab/group-reasoning

    - id: group-lead-alignment
      kind: Contract
      parallelize: 1
      org_chart_path: org:phi-research-lab/group-alignment

    # 9 team leads (3 per group). Identical profile shape; listed by id.
    - id: team-lead-vision
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-perception/team-vision
    - id: team-lead-audio
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-perception/team-audio
    - id: team-lead-multimodal
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-perception/team-multimodal
    - id: team-lead-planning
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-reasoning/team-planning
    - id: team-lead-math
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-reasoning/team-math
    - id: team-lead-theory
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-reasoning/team-theory
    - id: team-lead-interp
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-alignment/team-interpretability
    - id: team-lead-eval
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-alignment/team-evaluation
    - id: team-lead-governance
      kind: Contract
      parallelize: 2
      org_chart_path: org:phi-research-lab/group-alignment/team-governance

    # ~30 researchers — sample shown; rest follow the same shape.
    - id: researcher-vision-1
      kind: Contract
      profile: { name: researcher-vision-1, system_prompt: "Vision researcher; pursue hypotheses in parallel.", thinking_level: high, temperature: 0.5, personality: exploratory }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 100, max_tokens: 400_000, max_duration_secs: 14400, max_cost_usd: 20.00 }
      parallelize: 4                       # <-- the distinctive setting
      org_chart_path: org:phi-research-lab/group-perception/team-vision/researcher-1

    # ... 29 more researchers, similarly configured, under their respective teams.

  hierarchy:
    task:
      assigns:
        - from: lab-director
          to: [group-lead-perception, group-lead-reasoning, group-lead-alignment]
        - from: group-lead-perception
          to: [team-lead-vision, team-lead-audio, team-lead-multimodal]
        # ... team leads assign to their researchers
    sponsorship:
      sponsors_projects:
        - sponsor: lab-director
          projects: [vision-benchmark-study, interp-circuits-mapping, math-reasoning-probe]

  owned_projects: [vision-benchmark-study, interp-circuits-mapping, math-reasoning-probe]
```

## Cross-References

- [concepts/organization.md § Organization Edges](../concepts/organization.md) — `HAS_SUBORGANIZATION` for nested structure.
- [concepts/permissions/05 § Template C](../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question).
- [concepts/agent.md § Parallelized Sessions](../concepts/agent.md#parallelized-sessions) — why researchers run at `parallelize: 4`.
- [concepts/system-agents.md § Memory Extraction Agent](../concepts/system-agents.md#memory-extraction-agent) — tuned to `parallelize: 4` for this org's throughput.
- [projects/02-deeply-nested-project.md](../projects/02-deeply-nested-project.md) — an example project that exercises sub-project nesting parallel to this org's nested structure.
