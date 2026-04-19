<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 5 reference project layouts — see README.md -->

# 04 — `market-bid-project`

## Profile

A project inside [08-marketplace-gig.md](../organizations/08-marketplace-gig.md) where **tasks are posted for bidding** by Contract agents. No standing roster — contractors opt in per task via **Template E Auth Requests**. Bid negotiation happens over **inbox/outbox messaging**. No standard project-level OKRs; each awarded task has its own completion criteria.

## Knobs Summary

| Knob | Choice |
|------|--------|
| Shape | A |
| Sub-projects | none |
| Owning org | `marketplace-gig` |
| Project OKRs | **none at project level** (task-level criteria instead) |
| Task flow | **market bidding** via Auth Requests + inbox/outbox |
| Duration | variable per task |
| Audit posture | `logged` |
| Consent | `one_time` (from owning org) |
| Agent roster | **dynamic** — grown task-by-task |

## Narrative

**This project is a container for market-driven work.** A client (via the platform-admin) posts a task: "Build a Python CLI that scrapes X site and produces a CSV report." The task is broadcast to the marketplace's Contract agents via a Template E Auth Request with `scope: [assign_task, read, list]` on a newly-minted `project:market-task-{id}` scope.

Contract agents monitor their **inbox** for posted tasks (or query `bid-tracker-agent`). Interested agents submit **bids** — `AgentMessage` replies in the poster's inbox with `subject: "Bid: task-NNN"` and `body` describing approach, estimated tokens, timeline, and price. The poster compares bids, selects one, and **approves the Auth Request** with the winning agent's slot filled `Approved` and others `Denied`.

On approval, a `HAS_AGENT` edge from this project to the winning agent is created with `role: contractor` (not `member` or `lead`). The contractor does the work, marks the task complete, and the completion generates a Rating (see [token-economy.md](../concepts/token-economy.md)).

**No project-level OKRs.** Market-bid projects are heterogeneous — each task has its own goal. The parent project exists mostly for aggregating tasks and providing a shared resource boundary. If a client wants OKR-style tracking, they layer a higher-level project over several market tasks.

## Full YAML Config

```yaml
project:
  project_id: market-bid-project
  name: "Client X Engagement — Q2"
  description: "Variable work posted as tasks for marketplace contractor bidding."
  goal: "Complete the Q2 engagement tasks with satisfactory ratings."
  status:
    state: InProgress
    progress_percent: 20
    reason: "Week 3 of 12 — two tasks awarded, three open for bidding."
  token_budget: 20_000_000
  tokens_spent: 4_200_000
  created_at: 2026-04-01T09:00:00Z

  owning_orgs:
    - org_id: marketplace-gig
      role: primary

  objectives: []                              # no project-level OKRs
  key_results: []

  # ── Dynamic roster (grows per awarded task) ─────────────────────────
  agent_roster:
    - id: platform-admin
      role: poster                            # the client's proxy; posts tasks
    - id: contract-alpha
      role: contractor                        # awarded task-auth-cli
    - id: contract-delta
      role: contractor                        # awarded task-monitoring-setup

  # ── Tasks (the unit of work; each is its own Auth Request) ──────────
  tasks:
    - task_id: task-auth-cli
      name: "Build auth CLI utility"
      description: "Python CLI that wraps the client's auth API with token caching."
      status: Assigned
      posted_at: 2026-04-02T10:00:00Z
      bidding_ended_at: 2026-04-03T10:00:00Z
      awarded_to: contract-alpha
      award_auth_request: auth_request:req-18401
      bid_amount_tokens: 800_000
      bids_received: 4
      completion_criteria:
        - "CLI supports login, refresh, logout"
        - "Token cache survives process restart"
        - "Unit tests pass"

    - task_id: task-monitoring-setup
      name: "Set up monitoring dashboard"
      description: "Grafana dashboard for client's production service."
      status: Assigned
      posted_at: 2026-04-05T10:00:00Z
      bidding_ended_at: 2026-04-06T10:00:00Z
      awarded_to: contract-delta
      award_auth_request: auth_request:req-18432
      bid_amount_tokens: 1_200_000
      bids_received: 3
      completion_criteria:
        - "Dashboard shows latency p50/p95/p99"
        - "Alerts configured for p99 > 500ms"
        - "Runbook documented"

    - task_id: task-batch-report
      name: "Write batch report generator"
      description: "Nightly batch job that produces a weekly summary report."
      status: Bidding
      posted_at: 2026-04-14T10:00:00Z
      bidding_ends_at: 2026-04-15T10:00:00Z
      post_auth_request: auth_request:req-18450
      bids_received: 2

    - task_id: task-data-import
      name: "Build data import pipeline"
      description: "Ingest client's data from CSV into normalised tables."
      status: Bidding
      posted_at: 2026-04-14T11:00:00Z
      bidding_ends_at: 2026-04-15T11:00:00Z
      post_auth_request: auth_request:req-18451
      bids_received: 1

    - task_id: task-api-docs
      name: "Write API documentation"
      description: "Public API reference docs from OpenAPI spec."
      status: Bidding
      posted_at: 2026-04-15T09:00:00Z
      bidding_ends_at: 2026-04-16T09:00:00Z
      post_auth_request: auth_request:req-18475
      bids_received: 0                        # still open

  # ── Resource boundaries (subset of marketplace-gig's catalogue) ─────
  resource_boundaries:
    filesystem_objects:
      - path: /workspace/market-bid-project/**
    process_exec_objects:
      - id: sandboxed-shell
    network_endpoints:
      - domain: api.anthropic.com
    secrets:
      - id: anthropic-api-key
    memory_objects:
      - scope: per-agent
      - scope: per-project
    session_objects:
      - scope: per-project
      - scope: per-agent
    model_runtime_objects:
      - id: claude-sonnet-default

  # ── Market-bid-specific configuration ───────────────────────────────
  market_bid_config:
    task_auth_request_template: template_e
    bidding_window_default: 24h
    bid_channel: inbox_outbox                 # bids flow as AgentMessage
    bid_tracker_agent: bid-tracker-agent       # org's system agent maintains the index
    completion_rating_required: true          # every completed task produces a Rating

  sub_projects: []
```

## Cross-References

- [organizations/08-marketplace-gig.md](../organizations/08-marketplace-gig.md) — owning org; defines the bid-tracker agent, the contract roster, and the market-bid conventions.
- [concepts/permissions/07 § Template E](../concepts/permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e) — explicit task-assignment grants.
- [concepts/permissions/05 § Inbox and Outbox](../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging) — the bid-negotiation channel.
- [concepts/permissions/02 § Auth Request Lifecycle](../concepts/permissions/02-auth-request.md#auth-request-lifecycle) — the approval flow each bid runs through.
- [concepts/token-economy.md § Rating Window](../concepts/token-economy.md) — completion ratings feed contractor Worth.
