<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 08 — `marketplace-gig`

## Profile

A gig-shop org built around **Contract agents and market-style task posting**. All LLM agents are Contract-tier (no Interns — each agent has a public reputation and bids for work). Template E is heavy: every task assignment goes through an explicit Auth Request. **Inbox/outbox messaging is core infrastructure** — agents negotiate bids, trade status updates, and handshake on delivery through the messaging channel, not through a central scheduler. Market specification itself is `[OUT OF V0 SCOPE]`, but this org is structured to plug into it when v1 lands.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | `one_time` |
| Agent mix | Platform admin + 12 Contract agents (all Contract-tier) |
| Hierarchy | **Flat** — no standing supervision; task-by-task assignments |
| Authority Templates | **E-heavy**; A and B available but rarely fire |
| Audit posture | `logged` |
| Inbox/Outbox | **Heavy** — primary coordination mechanism |
| `parallelize` | 2–4 on contract workers |
| System agents | default two; **bid-tracker** added as an extra |
| Market participation | **structured for market** (v0 emits Auth Requests that v1 Market will route) |

## Narrative

The gig-shop model is what v0 supports **today** that most closely simulates a market: all work flows through **Template E Auth Requests**. A would-be task poster submits an Auth Request with `scope: [assign_task]` on the target agent; the agent approves via their inbox (or declines with a counter-bid). This is the primitive the Market will eventually wrap; until then, this org runs the equivalent by hand.

**All Contract agents** means everyone has a public rating and a Worth/Value/Meaning profile ([token-economy.md](../concepts/token-economy.md)). Promotion from Intern is not part of this org's flow — agents show up already vetted, with an established track record from other orgs.

**Inbox/outbox is heavy.** Every proposal, counter-proposal, status update, and delivery handshake flows through the messaging channel. A gig agent checks their inbox at the start of every session; accepts or declines the highest-value open proposals; posts updates to their outbox. The `send_message` tool is in every agent's grant set; tool manifests scoped to `inbox_object` and `outbox_object` are load-bearing here. See [permissions/05 § Inbox and Outbox](../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging).

**`bid-tracker-agent`** is an extra System Agent that maintains an index of open proposals, outstanding bids, and completed contracts. It's read-only from other agents' perspectives — a queryable "what's out there?" endpoint.

## Full YAML Config

```yaml
organization:
  org_id: marketplace-gig
  name: "The Marketplace"
  vision: "High-quality contract agent work, matched to demand."
  mission: "No standing hierarchy; work flows through explicit, auditable proposals."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: one_time
  audit_class_default: logged

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/gig-{task_id}/**
        default_owner: project              # one workspace per assigned task
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
        custodian: agent:platform-admin
      - id: openai-api-key
        custodian: agent:platform-admin
    data_objects: []
    tags: []
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 100_000_000
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 48
    memory_objects:
      - scope: per-agent
      - scope: per-project                  # per-task; per-project at the gig level
      - scope: per-org
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
      - id: gpt-4o-default
        provider: openai
    control_plane_objects: [tool-registry, agent-catalogue, policy-store, bid-index]
    auth_request_objects: []
    inbox_objects: []                       # heavily used
    outbox_objects: []

  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 3
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change
    - id: bid-tracker-agent                 # org-specific extra
      profile_ref: bid-tracker
      parallelize: 2
      trigger: auth_request_state_change
      profile:
        name: bid-tracker-agent
        system_prompt: |
          You are the Bid Tracker. Maintain a live index of all open Auth Requests
          with scope: [assign_task], plus all completed assignments in the last 30 days.
          Respond to queries about available work, active bidders, and completion history.
        thinking_level: low
        temperature: 0.0
        personality: precise
      model_config: { provider: anthropic, model: claude-haiku-default, max_tokens: 4096 }
      execution_limits: { max_turns: 10, max_tokens: 40_000, max_duration_secs: 600, max_cost_usd: 0.50 }

  authority_templates_enabled: [A, B]       # A/B available but rarely fire; most flow through E

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
    - request_grant                         # agents use this to submit bids
    - send_message                          # core — inbox/outbox is the negotiation channel
    - query_agents                          # queries agent-catalog-agent
    - query_bids                            # queries bid-tracker-agent
  tool_constraints:
    bash: { sandbox_requirement: required, timeout_secs: 120 }

  execution_limits:
    max_turns: 60
    max_tokens: 150_000
    max_duration_secs: 5400
    max_cost_usd: 5.00

  agent_roster:
    - id: platform-admin
      kind: Human
      role: sponsor
      channels: [slack, email]
      notes: "Runs the marketplace. Approves edge cases, does not assign tasks directly."

    # 12 Contract agents. All parallelize 2–4. All have established ratings.
    - id: contract-alpha
      kind: Contract
      profile: { name: contract-alpha, system_prompt: "Senior full-stack contract agent. Bid aggressively on web projects.", thinking_level: high, temperature: 0.3, personality: directive }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 4
      specialization: [web, full-stack]
      current_rating: 0.82

    - id: contract-beta
      kind: Contract
      profile: { name: contract-beta, system_prompt: "Data engineer contract agent.", thinking_level: high, temperature: 0.2, personality: careful }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 80, max_tokens: 200_000, max_duration_secs: 7200, max_cost_usd: 8.00 }
      parallelize: 3
      specialization: [data, pipelines]
      current_rating: 0.78

    - id: contract-gamma
      kind: Contract
      parallelize: 2
      specialization: [ml, research]
      current_rating: 0.75

    - id: contract-delta
      kind: Contract
      parallelize: 4
      specialization: [infra, devops]
      current_rating: 0.85

    # ... 8 more contract agents following the same pattern (specializations vary)
    - id: contract-epsilon
      kind: Contract
      parallelize: 3
      specialization: [writing, docs]

    - id: contract-zeta
      kind: Contract
      parallelize: 2
      specialization: [security, audit]

    - id: contract-eta
      kind: Contract
      parallelize: 3
      specialization: [mobile, ui]

    - id: contract-theta
      kind: Contract
      parallelize: 2
      specialization: [data, analytics]

    - id: contract-iota
      kind: Contract
      parallelize: 2
      specialization: [research, literature]

    - id: contract-kappa
      kind: Contract
      parallelize: 3
      specialization: [backend, APIs]

    - id: contract-lambda
      kind: Contract
      parallelize: 2
      specialization: [qa, testing]

    - id: contract-mu
      kind: Contract
      parallelize: 4
      specialization: [full-stack, rapid-prototyping]

  hierarchy:
    task:
      assigns: []                            # no standing task assignments — all via Auth Request
      notes: |
        Task flow: a would-be poster (platform-admin or another agent with a
        delegable grant) submits an Auth Request with scope: [assign_task, read, list]
        on the target agent(s)' session/resources. The target agent approves via
        their inbox. Bid counter-proposals are `send_message` replies.
    sponsorship:
      sponsors_projects:
        - sponsor: platform-admin
          projects: [gig-market-v0]           # single meta-project; actual gigs are sub-projects per task

  owned_projects: [gig-market-v0]
```

## Cross-References

- [concepts/permissions/05 § Inbox and Outbox (Agent Messaging)](../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging) — the messaging primitive this org runs on.
- [concepts/permissions/07 § Opt-in Example: Template E](../concepts/permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e) — explicit task-assignment grants.
- [concepts/organization.md § Market](../concepts/organization.md#market-future-concept--placeholder) — the `[OUT OF V0 SCOPE]` Market spec this org is structured for.
- [concepts/agent.md § Contract Agent](../concepts/agent.md#contract-agent-token-economy-participant) — the only agent tier used here.
- [projects/04-market-bid-project.md](../projects/04-market-bid-project.md) — an example project posted to this marketplace.
