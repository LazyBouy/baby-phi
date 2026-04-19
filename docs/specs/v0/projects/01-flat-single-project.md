<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 5 reference project layouts — see README.md -->

# 01 — `flat-single-project`

## Profile

The simplest project under the v0 model. Shape A: one project, one owning org. One lead + three workers. Six-week sprint with a lightweight OKR (2 objectives, 4 key results). Owned by [01-minimal-startup.md](../organizations/01-minimal-startup.md) (where it is named `solo-project`). The purpose of this layout is the **minimum viable project shape** — every other project layout adds structure on top of this one.

## Knobs Summary

| Knob | Choice |
|------|--------|
| Shape | A (one project, one org) |
| Sub-projects | none |
| Owning org | `minimal-startup` |
| OKRs | 2 Objectives × 2 KRs each = 4 total |
| Task flow | direct assignment (lead → workers) |
| Duration | ~6 weeks |
| Audit posture | `logged` (inherits org default) |
| Consent | `implicit` (inherits org) |

## Narrative

A founder wants to ship one product over a quarter. The single project `solo-project` is owned by `minimal-startup`. The founder (Human) is the sponsor; two Intern LLM agents (`intern-a`, `intern-b`) are the workers. The founder assigns tasks directly; the workers execute; OKRs track the sprint goal.

**Why minimal OKRs are enough.** At this scale, heavy OKR machinery is overhead. Two Objectives ("ship MVP" and "validate with 3 users") with two Key Results each give just enough structure to measure progress without forcing the founder to write rubrics and reviews.

**Everything else follows from the org.** No sub-projects, no special resource boundaries (the project inherits `minimal-startup`'s whole catalogue), no extra system agents, no joint ownership. Shape A, Template A from the owning org's enabled set governs the lead-sees-worker-sessions relationship.

## Full YAML Config

```yaml
project:
  project_id: solo-project
  name: "Solo Sprint MVP"
  description: "Founder's six-week sprint to ship the v0.1 MVP and validate with three users."
  goal: "MVP shipped and validated."
  status:
    state: InProgress
    progress_percent: 35
    reason: "Week 2 of 6 — ahead of schedule on features, behind on validation."
  token_budget: 3_000_000
  tokens_spent: 820_000
  created_at: 2026-04-01T09:00:00Z

  owning_orgs:
    - org_id: minimal-startup
      role: primary

  # ── OKRs ────────────────────────────────────────────────────────────
  objectives:
    - objective_id: obj-ship
      name: "Ship the MVP"
      description: "Deploy a working v0.1 to production with core feature set."
      status: Active
      owner: founder
      deadline: 2026-05-15T00:00:00Z
      key_result_ids: [kr-features-shipped, kr-deploy]

    - objective_id: obj-validate
      name: "Validate with users"
      description: "Confirm product-market fit signal with three paying or paying-intent users."
      status: Active
      owner: founder
      deadline: 2026-05-20T00:00:00Z
      key_result_ids: [kr-user-interviews, kr-users-committed]

  key_results:
    - kr_id: kr-features-shipped
      name: "Core features shipped"
      description: "Auth, core workflow, billing integration all working end-to-end."
      measurement_type: Count
      target_value: 3
      current_value: 2
      owner: founder
      deadline: 2026-05-12T00:00:00Z
      status: InProgress

    - kr_id: kr-deploy
      name: "Production deployment"
      description: "Service running on production infra with monitoring."
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: founder
      deadline: 2026-05-15T00:00:00Z
      status: NotStarted

    - kr_id: kr-user-interviews
      name: "User interviews completed"
      description: "Talked to five candidate users with structured discovery interviews."
      measurement_type: Count
      target_value: 5
      current_value: 3
      owner: founder
      deadline: 2026-05-10T00:00:00Z
      status: InProgress

    - kr_id: kr-users-committed
      name: "Paying/paying-intent users"
      description: "Three users have committed to pay or signed LOI."
      measurement_type: Count
      target_value: 3
      current_value: 0
      owner: founder
      deadline: 2026-05-20T00:00:00Z
      status: NotStarted

  # ── Agent roster (references owning org's agents) ───────────────────
  agent_roster:
    - id: founder
      role: sponsor
    - id: intern-a
      role: worker
    - id: intern-b
      role: worker
  # The lead role is filled by the founder informally; no contract lead at this scale.

  # ── Tasks ───────────────────────────────────────────────────────────
  tasks:
    - task_id: task-auth
      name: "Implement auth flow"
      assigned_to: intern-a
      status: InProgress
      linked_kr: kr-features-shipped

    - task_id: task-billing
      name: "Integrate Stripe billing"
      assigned_to: intern-b
      status: InProgress
      linked_kr: kr-features-shipped

    - task_id: task-workflow
      name: "Core workflow screens"
      assigned_to: intern-a
      status: Completed
      linked_kr: kr-features-shipped

    - task_id: task-deploy
      name: "Set up production deployment"
      assigned_to: intern-b
      status: NotStarted
      linked_kr: kr-deploy

    - task_id: task-interviews
      name: "Run user interviews"
      assigned_to: founder                # the founder does this; agents support
      status: InProgress
      linked_kr: kr-user-interviews

  # ── Resource boundaries (subset of owning org's catalogue) ──────────
  resource_boundaries:
    filesystem_objects:
      - path: /workspace/solo-project/**
    process_exec_objects:
      - id: sandboxed-shell
    network_endpoints:
      - domain: api.anthropic.com
      - domain: stripe.com            # task-billing needs this; added via Template E when task starts
    secrets:
      - id: anthropic-api-key
      - id: stripe-test-key           # project-scoped; added via Template E when task-billing starts
    memory_objects:
      - scope: per-agent
      - scope: per-project
    session_objects:
      - scope: per-project
      - scope: per-agent
    model_runtime_objects:
      - id: claude-sonnet-default

  sub_projects: []                    # flat — no sub-projects
```

## Cross-References

- [concepts/project.md § Objectives and Key Results (OKRs)](../concepts/project.md#objectives-and-key-results-okrs) — the OKR shape.
- [concepts/permissions/05 § Sessions as a Tagged Resource](../concepts/permissions/05-memory-sessions.md#sessions-as-a-tagged-resource) — Shape A session semantics.
- [organizations/01-minimal-startup.md](../organizations/01-minimal-startup.md) — the owning org.
