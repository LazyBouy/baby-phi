<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 01 — `minimal-startup`

## Profile

A solo founder working with two LLM intern agents. Simplest possible org under v0. Single human sponsor; all work funnels through one project. Implicit consent, no market, no co-ownership. The purpose of this layout is to serve as the **minimum viable baseline** — every other layout can be explained as "start from this, add X."

## Knobs Summary

| Knob | Choice | Default? |
|------|--------|----------|
| `consent_policy` | `implicit` | ✓ default |
| Agent mix | 1 Human (sponsor) + 2 Intern LLM Agents | — |
| Hierarchy | Flat (sponsor → interns) | — |
| Audit posture | `logged` (default) | ✓ default |
| Co-ownership | none | ✓ |
| Market participation | none | ✓ |
| Inbox/Outbox usage | light (just shared notes) | — |
| `parallelize` | `1` on all agents | ✓ default |
| System agents | default two (memory-extraction + agent-catalog) | ✓ default |

## Narrative

The minimal-startup org exists to show that the v0 model **does not require elaborate configuration to be useful**. A single human sponsor starts the org, declares a small resource catalogue (a project workspace, an LLM credential, a token budget pool), adopts the Standard Organization Template unchanged, and spawns two Intern LLM Agents to do the work.

Because there are no co-owners, no sub-projects, and no second human, the per-co-owner consent rule from [permissions/06](../concepts/permissions/06-multi-scope-consent.md) never fires; the implicit consent policy handles everything without explicit acknowledgement. Because all agents are Interns (below the 10-job, 0.6-rating promotion threshold), no one participates in the token economy yet — the org absorbs all token costs as overhead.

The two System Agents (`memory-extraction-agent` and `agent-catalog-agent`) are included at their default `parallelize` values. Even in an org this small they earn their keep: the memory extraction agent routes end-of-session learnings into the appropriate pool, and the agent catalog agent means the founder can ask "show me agent activity this week" without querying the graph by hand.

**Scenarios this layout handles well:**
- Prototyping an agent-assisted product.
- Personal research where one human drives two parallel agent workstreams.
- First adoption of the baby-phi model ("what's the smallest thing I can get running?").

**Scenarios this layout does NOT handle** and that require upgrading to a larger layout:
- Multiple simultaneous sponsors or co-owners (see `06`, `07`).
- Structured supervision or multi-level hierarchy (see `02`, `05`).
- Regulated data flows or compliance auditing (see `04`).
- High concurrency (see `05`, `10`).

## Full YAML Config

```yaml
organization:
  org_id: minimal-startup
  name: "Founder's Workshop"
  vision: "Ship one agent-assisted product in a quarter."
  mission: "Two interns plus the founder, short feedback loops."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard                 # Standard Organization Template, unchanged
  customizations: none

  consent_policy: implicit
  audit_class_default: logged

  # ── Resource Catalogue ──────────────────────────────────────────────
  resources_catalogue:
    filesystem_objects:
      - path: /workspace/solo-project/**
        default_owner: project
        description: Single-project workspace
      - path: /home/founder/**
        default_owner: agent
      - path: /home/intern-a/**
        default_owner: agent
      - path: /home/intern-b/**
        default_owner: agent
    process_exec_objects:
      - id: sandboxed-shell
        sandbox_requirement: required
    network_endpoints:
      - domain: api.anthropic.com
        purpose: LLM inference
    secrets:
      - id: anthropic-api-key
        custodian: agent:founder
    data_objects: []
    tags: []
    identity_principals: []               # populated by agent_roster below
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 5_000_000     # modest quarterly budget
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 4
    memory_objects:
      - scope: per-agent
      - scope: per-project
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []                 # no MCP servers in v0.1
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
    control_plane_objects:
      - id: tool-registry
      - id: agent-catalogue
      - id: policy-store
    auth_request_objects: []
    inbox_objects: []                     # auto-created per agent
    outbox_objects: []

  # ── System Agents ───────────────────────────────────────────────────
  system_agents:
    - id: memory-extraction-agent
      profile_ref: system-memory-extraction
      parallelize: 1
      trigger: session_end
    - id: agent-catalog-agent
      profile_ref: system-agent-catalog
      parallelize: 1
      trigger: edge_change

  # ── Authority Templates ─────────────────────────────────────────────
  authority_templates_enabled: [A, B]

  # ── Tool allowlist ──────────────────────────────────────────────────
  tools_allowlist:
    - read_file
    - write_file
    - edit_file
    - search
    - recall_memory
    - store_memory
    - bash                                # sandboxed
  tool_constraints:
    bash:
      sandbox_requirement: required
      timeout_secs: 120

  # ── Execution limits (per-agent default) ────────────────────────────
  execution_limits:
    max_turns: 50
    max_tokens: 100_000
    max_duration_secs: 3600
    max_cost_usd: 2.00

  # ── Agent Roster ────────────────────────────────────────────────────
  # Uses phi-core types: AgentProfile, ModelConfig, ExecutionLimits.
  agent_roster:
    - id: founder
      kind: Human
      role: sponsor
      channels: [slack, email]
      # No AgentProfile / ModelConfig / ExecutionLimits — humans aren't LLM-backed

    - id: intern-a
      kind: Intern                        # Standard Agent, pre-economy
      profile:
        name: intern-a
        system_prompt: "General-purpose intern agent. Focus on the founder's top priority."
        thinking_level: medium
        temperature: 0.3
        personality: curious
      model_config:
        provider: anthropic
        model: claude-sonnet-default
        max_tokens: 8192
      execution_limits:
        max_turns: 50
        max_tokens: 100_000
        max_duration_secs: 3600
        max_cost_usd: 2.00
      parallelize: 1

    - id: intern-b
      kind: Intern
      profile:
        name: intern-b
        system_prompt: "General-purpose intern agent. Focus on the founder's second priority."
        thinking_level: medium
        temperature: 0.3
        personality: meticulous
      model_config:
        provider: anthropic
        model: claude-sonnet-default
        max_tokens: 8192
      execution_limits:
        max_turns: 50
        max_tokens: 100_000
        max_duration_secs: 3600
        max_cost_usd: 2.00
      parallelize: 1

  # ── Hierarchy (task + sponsorship) ──────────────────────────────────
  hierarchy:
    task:
      # founder assigns tasks to both interns; no lead-of-leads
      assigns:
        - from: founder
          to: [intern-a, intern-b]
    sponsorship:
      # founder sponsors the sole project
      sponsors_projects:
        - sponsor: founder
          projects: [solo-project]

  # ── Projects owned by this org ──────────────────────────────────────
  owned_projects: [solo-project]
```

## Cross-References

- [concepts/organization.md](../concepts/organization.md) — Organization node model.
- [concepts/permissions/07 § Standard Organization Template](../concepts/permissions/07-templates-and-tools.md#standard-organization-template) — the template this inherits from.
- [concepts/permissions/06 § Consent Policy (Organizational)](../concepts/permissions/06-multi-scope-consent.md#consent-policy-organizational) — `implicit` consent semantics.
- [concepts/system-agents.md](../concepts/system-agents.md) — the two System Agents included by default.
- [concepts/agent.md § Intern Agent](../concepts/agent.md#intern-agent-pre-economy) — why all LLM agents here are Interns.
- [projects/01-flat-single-project.md](../projects/01-flat-single-project.md) — the sole project this org owns (`solo-project` in this config ↔ `flat-single-project` in the projects catalogue).
