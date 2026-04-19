<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Project, Task, Bid, Rating

> Extracted from brainstorm.md Sections 3.6-3.9, refined 2026-04-09.
> See also: [token-economy.md](token-economy.md) (bidding, Worth/Value/Meaning, rating window), [organization.md](organization.md) (project ownership), [permissions.md](permissions/README.md) (project-level permissions and the Multi-Scope Session Access rule for sessions belonging to multiple projects).
>
> **Five worked examples:** see [`../projects/`](../projects/README.md) for five reference project layouts spanning Shape A vs B, flat vs nested sub-projects, standing vs market-bid task flows, and sprint vs long-duration cadences.

---

## Project (Node Type)

A Project is a **container for work** with a goal, agents, and governance.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `project_id` | String | Unique identifier |
| `name` | String | Human-readable name |
| `description` | String | What this project aims to achieve |
| `goal` | Option<String> | Specific measurable goal |
| `status` | ProjectStatus | Planned(0%), InProgress(%), OnHold(reason), Finished(100%) |
| `token_budget` | Option<u64> | Total tokens allocated for this project |
| `tokens_spent` | u64 | Running total of tokens consumed |
| `created_at` | DateTime | When the project was created |
| `objectives` | Option<Vec<Objective>> | High-level goals the project pursues. Optional — projects without explicit Objectives use the simpler `goal` field. Projects may mix (set both). See [Objectives and Key Results](#objectives-and-key-results-okrs) below. |
| `key_results` | Option<Vec<KeyResult>> | Measurable outcomes that indicate progress toward Objectives. Each KR is tracked by its own schedule and status. See [Objectives and Key Results](#objectives-and-key-results-okrs) below. |
| `resource_boundaries` | Option<ResourceBoundaries> | A subset of the owning org(s)' `resources_catalogue` that this project operates within. Narrows the available resource set for project-scoped grants. |

### Project Status

```
Planned (0%) ──▶ InProgress (with %) ──▶ Finished (100%)
                      │         ▲
                      ▼         │
                 OnHold (with/without reason)
```

All status transitions carry a reason. OnHold captures ALL suspension scenarios — no separate "blocked", "waiting", "paused" states.

### Project Edges

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Project | `HAS_SPONSOR` | Agent (Human) | 1:N | role: primary/secondary |
| Project | `HAS_AGENT` | Agent | 1:N | role: member/lead |
| Project | `HAS_LEAD` | Agent | 1:1 | — (shortcut for HAS_AGENT where role=lead) |
| Project | `HAS_TASK` | Task | 1:N | order: u32 |
| Project | `HOLDS_GRANT` | Grant | 1:N | project-scoped rules |
| Project | `HAS_CONFIG` | AgentConfig | 1:1 | project-level config |
| Project | `HAS_SUBPROJECT` | Project | 1:N | — |
| Project | `BELONGS_TO` | Organization | N:N | role: primary/secondary |

### Objectives and Key Results (OKRs)

Projects that need structured goal tracking declare **Objectives** (what the project is trying to achieve) and **Key Results** (measurable outcomes that indicate progress). Both are optional; a project without explicit OKRs continues to use the simpler `goal` field. Projects may mix — declare a top-level `goal` *and* specific OKRs — when the `goal` is a one-line framing and the OKRs are the measurement.

Objectives and Key Results are **value objects embedded on the Project node**, not independent graph nodes. This keeps the shape small and scoped to the project that owns them.

**`Objective`** value object:

```
Objective {
  objective_id:   String
  name:           String
  description:    String
  status:         enum { Draft, Active, Achieved, Missed, Cancelled }
  owner:          agent_id                      // the agent accountable for the objective
  deadline:       Option<DateTime>
  key_result_ids: Vec<String>                   // links to this project's KeyResults
}
```

**`KeyResult`** value object:

```
KeyResult {
  kr_id:             String
  name:              String
  description:       String
  measurement_type:  enum { Count, Boolean, Percentage, Custom }
  target_value:      Value                      // typed by measurement_type
  current_value:     Value                      // updated as the project progresses
  owner:             agent_id
  deadline:          Option<DateTime>
  status:            enum { NotStarted, InProgress, Achieved, Missed, Cancelled }
}
```

**Notes:**

- **Measurement types:** `Count` uses integer target/current (e.g., "ship 5 features"); `Boolean` uses true/false (e.g., "security review passed"); `Percentage` uses 0.0–1.0 (e.g., "coverage ≥ 0.85"); `Custom` allows a free-form `Value` for domain-specific metrics (e.g., a SLA score composite).
- **Aggregation across sub-projects:** a parent project with `HAS_SUBPROJECT` sub-projects may reference sub-project OKRs in its own Key Results (by `kr_id`) to roll up progress. The aggregation rule is explicit at the parent (sum / max / weighted average) rather than inferred.
- **OKR vs Task distinction:** Objectives are goals; Tasks are units of work. A KR may be achieved by completing several Tasks; one Task may contribute to multiple KRs. The Project's `HAS_TASK` edges and `key_results` field are complementary views.
- **Status lifecycle:** Objectives and KRs follow an explicit status lifecycle and are expected to transition deliberately (a KR doesn't silently expire; it moves to `Missed` or `Cancelled` with a reason). Status transitions are auditable events.

---

## Task (Node Type — Optional Decomposition)

A Task is the **biddable unit of work** within a Project. Simple projects can skip Tasks entirely and go straight to Sessions.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `task_id` | String | Unique identifier |
| `name` | String | Task title |
| `description` | String | What needs to be done |
| `token_budget` | Option<u64> | Tokens allocated (for contract bidding) |
| `tokens_spent` | u64 | Running total |
| `status` | TaskStatus | Open, Bidding, Assigned, InProgress, Review, Completed, Cancelled |
| `deadline` | Option<DateTime> | When this task should be completed |
| `estimation` | Option<u64> | Estimated tokens (from estimation skill) |
| `created_by` | agent_id | Who created this task |

### Task Status Flow

```
Open ──▶ Bidding ──▶ Assigned ──▶ InProgress ──▶ Review ──▶ Completed
  │                                  │                        │
  ▼                                  ▼                        ▼
Cancelled                        OnHold                   Cancelled
```

### Task Edges

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Task | `ASSIGNED_TO` | Agent | N:1 | — (the winning bidder or assigned agent) |
| Task | `HAS_BID` | Bid | 1:N | — |
| Task | `PRODUCES_SESSION` | Session | 1:N | — (execution of the task) |
| Task | `HAS_SUBTASK` | Task | 1:N | — |
| Task | `CREATED_BY` | Agent | N:1 | — |

### Who Creates Tasks

Either sponsors (Human Agents) or lead agents can create biddable Tasks for their subordinates. In the future, Tasks may also be posted to a Market (see [organization.md](organization.md)).

---

## Bid (Node Type)

A Bid is an agent's **proposal** for a Task.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `bid_id` | String | Unique identifier |
| `token_amount` | u64 | How many tokens the agent requests |
| `approach` | String | Brief description of how the agent will do the work |
| `estimated_turns` | Option<u32> | Estimated number of turns |
| `status` | BidStatus | Submitted, Accepted, Rejected, Withdrawn |
| `submitted_at` | DateTime | When the bid was submitted |

### Bid Edges

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Bid | `SUBMITTED_BY` | Agent | N:1 | — |
| Bid | `FOR_TASK` | Task | N:1 | — |
| Bid | `APPROVED_BY` | Agent | N:1 | — (sponsor or lead who approved) |

> When a Bid is accepted, the Task status moves to Assigned, and a **Contract** relationship is implicitly formed (Task ASSIGNED_TO Agent with the bid's token_amount as the budget).

---

## Rating (Node Type)

A Rating is a **quality assessment** given to an agent after completing work.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `rating_id` | String | Unique identifier |
| `score` | f32 | Numeric score in `0.0 – 1.0` (normalized; see [token-economy.md](token-economy.md) for the rating window and Worth formula) |
| `dimensions` | Option<Json> | Optional multi-dimensional scores { quality, speed, efficiency, communication } — each dimension also in `0.0 – 1.0` |
| `comment` | Option<String> | Free-text feedback |
| `rated_at` | DateTime | When the rating was given |

### Rating Edges

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Rating | `RATES` | Agent | N:1 | — (the agent being rated) |
| Rating | `GIVEN_BY` | Agent | N:1 | — (the rater: human or agent) |
| Rating | `FOR_TASK` | Task | N:1 | — (what work was this for) |
| Rating | `FOR_PROJECT` | Project | N:1 | — (project-level rating) |

> **Rating triggers Identity update.** When a Rating is created, the rated agent's Identity node is reactively updated.
