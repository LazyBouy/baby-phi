<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 5 reference project layouts — see README.md -->

# 03 — `joint-research`

## Profile

The canonical **Shape B (co-owned) project**. Owned jointly by [06-joint-venture-acme.md](../organizations/06-joint-venture-acme.md) and [07-joint-venture-beta.md](../organizations/07-joint-venture-beta.md). Mixed-org agent roster: `joint-acme-5` from Acme and `joint-beta-3` from Beta. Sessions carry both `org:acme` and `org:beta-corp` tags. This project is the reference exercise for:

- Per-co-owner consent (Acme `implicit` + Beta `one_time` evaluated independently)
- Ceiling intersection (Acme allows `bash`, Beta forbids it → `bash` forbidden in `joint-research`)
- Union-on-grants (both orgs can issue grants on the same resource; grants are additive)
- Auth Request per-resource slot model (requests touching co-owned resources get one slot per co-owner)

## Knobs Summary

| Knob | Choice |
|------|--------|
| Shape | **B — two-org co-owned** |
| Sub-projects | none |
| Owning orgs | **`acme` + `beta-corp`** (both primary) |
| OKRs | 3 Objectives × 2 KRs each = 6 total |
| Task flow | mixed — each agent receives tasks from their own org's lead; joint tasks require Auth Request with slots for both leads |
| Duration | open-ended (ongoing research partnership) |
| Audit posture | `logged` |
| Consent | **per-co-owner independent** — Acme `implicit` auto-acks; Beta `one_time` requires Consent record |

## Narrative

**Co-ownership declared symmetrically.** Both `acme` and `beta-corp` list `project:joint-research` in their `co_owned_resources`. At runtime, the same Project node has `BELONGS_TO` edges to both Organizations with `role: primary`.

**Agent roster is mixed.** `joint-acme-5` (Acme's agent, `base_organization: acme`) and `joint-beta-3` (Beta's, `base_organization: beta-corp`) both work on the project. Each agent's `current_project` is `joint-research`. When either runs a session, the session carries tags `[project:joint-research, org:acme, org:beta-corp, agent:{that_agent}]`.

**Per-co-owner consent example.** `lead-acme-1` (Acme lead) wants to read `joint-beta-3`'s session inside `joint-research`. Permission Check at [permissions/04](../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode) Step 6 evaluates consent per co-owner:

- Acme's `implicit` policy → auto-acknowledged for `joint-beta-3` under Acme's authority.
- Beta's `one_time` policy → must have a Consent record for `joint-beta-3` under Beta's policy in `Acknowledged` state.

If Beta's Consent is not yet acknowledged, the read returns `Pending(awaiting_consent=(joint-beta-3, beta-corp))`. If acknowledged, the read proceeds.

**Ceiling intersection in action.** When `joint-acme-5` tries to invoke `bash`: Acme's ceiling permits it; Beta's forbids it; intersection → forbidden. See [permissions/06 § Worked Example](../concepts/permissions/06-multi-scope-consent.md#worked-example--intersection-on-ceilings-union-on-grants).

## Full YAML Config

```yaml
project:
  project_id: joint-research
  name: "Acme–Beta Joint Research"
  description: "Multi-org research partnership on agent coordination patterns."
  goal: "Deliver joint research findings both orgs can publish or apply."
  status:
    state: InProgress
    progress_percent: 40
    reason: "Ongoing; re-scoped per quarter."
  token_budget: 80_000_000
  tokens_spent: 32_000_000
  created_at: 2026-01-15T09:00:00Z

  # ── Two owning orgs, both primary (Shape B) ─────────────────────────
  owning_orgs:
    - org_id: acme
      role: primary
    - org_id: beta-corp
      role: primary

  objectives:
    - objective_id: obj-publish
      name: "Publish joint findings"
      description: "Two publications (one Acme-led, one Beta-led) acknowledging joint work."
      status: Active
      owner: joint-acme-5
      deadline: 2026-08-31T00:00:00Z
      key_result_ids: [kr-paper-acme, kr-paper-beta]

    - objective_id: obj-artifacts
      name: "Ship shared artifacts"
      description: "Both orgs can use the resulting agent-coordination library."
      status: Active
      owner: joint-beta-3
      deadline: 2026-09-30T00:00:00Z
      key_result_ids: [kr-library-v1, kr-library-adopted]

    - objective_id: obj-knowledge-flow
      name: "Establish knowledge flow"
      description: "Each org extracts reusable memory from joint sessions."
      status: Active
      owner: joint-acme-5
      deadline: 2026-12-31T00:00:00Z
      key_result_ids: [kr-memories-extracted, kr-cross-references]

  key_results:
    - kr_id: kr-paper-acme
      name: "Acme publishes paper"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: joint-acme-5
      deadline: 2026-08-15T00:00:00Z
      status: InProgress

    - kr_id: kr-paper-beta
      name: "Beta publishes paper"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: joint-beta-3
      deadline: 2026-08-30T00:00:00Z
      status: InProgress

    - kr_id: kr-library-v1
      name: "Coordination library v1 released"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: joint-beta-3
      deadline: 2026-09-15T00:00:00Z
      status: InProgress

    - kr_id: kr-library-adopted
      name: "Both orgs adopt library internally"
      measurement_type: Count
      target_value: 2
      current_value: 0
      owner: joint-beta-3
      deadline: 2026-09-30T00:00:00Z
      status: NotStarted

    - kr_id: kr-memories-extracted
      name: "Memories extracted to both orgs' pools"
      description: "Memory Extraction Agent routes per-co-owner; both orgs see their share."
      measurement_type: Count
      target_value: 50
      current_value: 18
      owner: joint-acme-5
      status: InProgress

    - kr_id: kr-cross-references
      name: "Cross-references between sessions documented"
      measurement_type: Count
      target_value: 20
      current_value: 6
      owner: joint-acme-5
      status: InProgress

  # ── Agent roster (mixed-org) ────────────────────────────────────────
  agent_roster:
    - id: joint-acme-5
      role: member
      base_organization: acme
    - id: joint-beta-3
      role: member
      base_organization: beta-corp
    # Occasional collaborators (not full roster members):
    - id: lead-acme-1
      role: occasional_reviewer
      base_organization: acme
    - id: lead-beta-1
      role: occasional_reviewer
      base_organization: beta-corp

  tasks:
    - task_id: task-design-experiments
      name: "Design joint experiments"
      assigned_to: joint-acme-5
      status: InProgress
      linked_kr: kr-paper-acme

    - task_id: task-library-core
      name: "Build coordination library core"
      assigned_to: joint-beta-3
      status: InProgress
      linked_kr: kr-library-v1

    - task_id: task-weekly-sync
      name: "Weekly status sync across orgs"
      assigned_to: [joint-acme-5, joint-beta-3]
      status: Recurring
      notes: "Coordinated via inbox/outbox messaging; each agent posts weekly summary to the other's inbox."

  # ── Resource boundaries ─────────────────────────────────────────────
  # Resources must be declared in BOTH owning orgs' catalogues (co-owned).
  resource_boundaries:
    filesystem_objects:
      - path: /workspace/joint-research/**
        declared_in: [acme, beta-corp]      # catalogue declared in both orgs
    process_exec_objects: []                # bash forbidden by Beta's ceiling; intersection rule
    network_endpoints:
      - domain: api.anthropic.com
        declared_in: [acme, beta-corp]
    secrets:
      - id: joint-research-shared-key       # created via Template E; both orgs approve its catalogue entry
        declared_in: [acme, beta-corp]
    memory_objects:
      - scope: per-agent
      - scope: per-project                  # joint-research's shared memory pool
      - scope: per-org                      # each agent reads their own org's pool; no cross-org read
    session_objects:
      - scope: per-project
      - scope: per-agent
    model_runtime_objects:
      - id: claude-sonnet-default
        declared_in: [acme, beta-corp]

  # ── Co-owned project rules ──────────────────────────────────────────
  co_ownership:
    consent_evaluation: per_co_owner_independent
    ceiling_evaluation: intersection
    grant_evaluation: union
    revocation: per_co_owner_subtree
    # These four rules are from permissions/06 § Co-Ownership × Multi-Scope Session Access
    # rules 6, 2, 3/5 (union applies to grants), and 5 respectively.

  sub_projects: []
```

## Cross-References

- [organizations/06-joint-venture-acme.md](../organizations/06-joint-venture-acme.md) — Acme's side.
- [organizations/07-joint-venture-beta.md](../organizations/07-joint-venture-beta.md) — Beta's side.
- [concepts/permissions/06 § Co-Ownership × Multi-Scope Session Access](../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access) — the six rules this project exercises.
- [concepts/permissions/06 § Worked Example — Intersection on Ceilings, Union on Grants](../concepts/permissions/06-multi-scope-consent.md#worked-example--intersection-on-ceilings-union-on-grants) — the bash-forbidden-in-intersection scenario.
- [concepts/permissions/02 § Schema — Per-Resource Slot Model](../concepts/permissions/02-auth-request.md#schema--per-resource-slot-model) — how Auth Requests on co-owned resources fan out per co-owner.
