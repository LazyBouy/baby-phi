<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 10 reference organization layouts — see README.md -->

# 09 — `education-org`

## Profile

An educational organization built around **system agents for review and grading** and **learners as Interns with privacy-focused consent**. The work pattern is: instructors post assignments → learners (Intern agents) work on them → system agents review work and route feedback → learners iterate. `per_session` consent protects learner privacy; each instructor-review session requires fresh consent. Four System Agents (beyond the default two) automate grading, plagiarism checks, feedback generation, and progress tracking.

## Knobs Summary

| Knob | Choice |
|------|--------|
| `consent_policy` | **`per_session`** (learner privacy) |
| Agent mix | Program director + 3 Instructors (Human) + ~20 Learners (Intern) + 4 extra System Agents |
| Hierarchy | Flat — instructor sponsors the class; no standing supervision beyond that |
| Authority Templates | D (project-scoped, per-class) + E (explicit assignments) |
| Audit posture | `logged`; `alerted` on any cross-class read |
| `parallelize` | **1 on learners** (focus); 4 on grading agents |
| System agents | default two + **grading, plagiarism, feedback, progress-tracker** |
| Co-ownership | none |

## Narrative

**Learners as Interns with `per_session` consent** is the core privacy posture. Each learner's session is private to them by default. Instructor review requires a new Consent record per session; the learner can decline without consequence (the instructor gets a `Pending` permission-check result, not a `Denied`). This protects learning-in-progress from ambient supervisor access.

**Grading-agent as a System Agent** replaces the per-instructor "grade by hand" workflow. When a learner marks an assignment complete, the grading-agent reads the session, applies the rubric, and emits feedback into the learner's inbox. The grading is auditable — the rubric is versioned in the `control_plane_object` catalogue; every grade references the rubric version it was scored against.

**Four extra System Agents** is more than most orgs have, appropriate for an education context where automated review is load-bearing:

- **grading-agent** — applies rubrics to learner submissions.
- **plagiarism-check-agent** — scans submissions against prior sessions and `#public` memory for reuse.
- **feedback-generation-agent** — generates structured feedback from grading agent's output.
- **progress-tracker-agent** — maintains per-learner progress dashboards, flags at-risk learners.

**`parallelize: 1` on learners** is intentional: learning is easier when focused; concurrency is a productivity optimization that education doesn't need.

## Full YAML Config

```yaml
organization:
  org_id: education-org
  name: "Phi Academy"
  vision: "Learners progress at their own pace, with honest feedback."
  mission: "System agents handle scale; humans handle nuance."
  created_at: 2026-04-15T09:00:00Z

  inherits_from: standard
  consent_policy: per_session
  audit_class_default: logged
  audit_class_on_cross_class_read_attempt: alerted

  resources_catalogue:
    filesystem_objects:
      - path: /workspace/class-{class_id}/**
        default_owner: project
      - path: /submissions/{class_id}/{learner_id}/**
        default_owner: agent                # learner owns their submissions
      - path: /home/{agent}/**
        default_owner: agent
    process_exec_objects:
      - id: sandboxed-shell
        sandbox_requirement: required
    network_endpoints:
      - domain: api.anthropic.com
      - domain: api.openai.com
      - domain: public-research-apis.org
    secrets:
      - id: anthropic-api-key
        custodian: agent:program-director
    data_objects:
      - id: grading-rubrics
        description: Versioned rubrics per class
    tags: [class-cohort-1, class-cohort-2, class-cohort-3, '#at-risk']
    identity_principals: []
    economic_resources:
      - id: token-budget-pool
        units: tokens
        initial_allocation: 60_000_000
    compute_resources:
      - id: default-compute-pool
        max_concurrent_sessions: 24
    memory_objects:
      - scope: per-agent                    # learner private
      - scope: per-project                  # per-class
      - scope: per-org                      # program-wide (rubrics, FAQ)
      - scope: '#public'
    session_objects:
      - scope: per-project
      - scope: per-agent
    external_services: []
    model_runtime_objects:
      - id: claude-sonnet-default
        provider: anthropic
    control_plane_objects:
      - id: tool-registry
      - id: agent-catalogue
      - id: policy-store
      - id: grading-rubrics
      - id: progress-dashboards
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

    # Four education-specific system agents:
    - id: grading-agent
      profile_ref: edu-grading
      parallelize: 4                        # handles peaks at assignment deadlines
      trigger: submission_complete
      profile:
        name: grading-agent
        system_prompt: "Apply the current rubric to the submitted session. Score each criterion. Note weaknesses and strengths."
        thinking_level: high
        temperature: 0.1
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 8192 }
      execution_limits: { max_turns: 30, max_tokens: 100_000, max_duration_secs: 900, max_cost_usd: 2.00 }

    - id: plagiarism-check-agent
      profile_ref: edu-plagiarism
      parallelize: 2
      trigger: submission_complete
      profile:
        name: plagiarism-check-agent
        system_prompt: "Scan submissions for reuse from prior sessions, public memory, and known public corpora."
        thinking_level: medium
        temperature: 0.0
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 20, max_tokens: 50_000, max_duration_secs: 600, max_cost_usd: 1.00 }

    - id: feedback-generation-agent
      profile_ref: edu-feedback
      parallelize: 2
      trigger: grading_complete
      profile:
        name: feedback-generation-agent
        system_prompt: "Given a grading-agent output, generate clear, actionable feedback. Preserve learner dignity while being honest."
        thinking_level: medium
        temperature: 0.4
        personality: supportive
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 15, max_tokens: 40_000, max_duration_secs: 300, max_cost_usd: 0.80 }

    - id: progress-tracker-agent
      profile_ref: edu-progress
      parallelize: 1
      trigger: grading_complete
      profile:
        name: progress-tracker-agent
        system_prompt: "Maintain per-learner progress state; flag at-risk learners with '#at-risk' tag."
        thinking_level: low
        temperature: 0.0
      model_config: { provider: anthropic, model: claude-haiku-default, max_tokens: 4096 }
      execution_limits: { max_turns: 10, max_tokens: 30_000, max_duration_secs: 300, max_cost_usd: 0.30 }

  authority_templates_enabled: [D, E]       # D for instructor→learner scoped access within a class

  tools_allowlist:
    - read_file
    - write_file
    - edit_file
    - search
    - recall_memory
    - store_memory
    - http_get
    - request_grant
    - send_message
    - submit_assignment                     # edu-specific tool
  tool_constraints:
    http_get:
      allowed_domains: [public-research-apis.org]

  execution_limits:
    max_turns: 40
    max_tokens: 80_000
    max_duration_secs: 2400
    max_cost_usd: 1.50

  agent_roster:
    - id: program-director
      kind: Human
      role: sponsor
      channels: [slack, email]

    - id: instructor-alpha
      kind: Human
      role: sponsor                          # instructors sponsor their own classes
      channels: [slack]
      project_role: { class-cohort-1: instructor }

    - id: instructor-beta
      kind: Human
      role: sponsor
      channels: [slack]
      project_role: { class-cohort-2: instructor }

    - id: instructor-gamma
      kind: Human
      role: sponsor
      channels: [slack]
      project_role: { class-cohort-3: instructor }

    # ~20 learner Intern agents. Sample shown; identical shape for the rest.
    - id: learner-01
      kind: Intern
      profile: { name: learner-01, system_prompt: "You are a student in cohort 1. Work through each assignment carefully.", thinking_level: medium, temperature: 0.4, personality: curious }
      model_config: { provider: anthropic, model: claude-sonnet-default, max_tokens: 4096 }
      execution_limits: { max_turns: 40, max_tokens: 80_000, max_duration_secs: 2400, max_cost_usd: 1.50 }
      parallelize: 1
      project_role: { class-cohort-1: learner }

    - id: learner-02
      kind: Intern
      parallelize: 1
      project_role: { class-cohort-1: learner }

    # ... 18 more learners across the three cohorts

  hierarchy:
    task:
      assigns:
        - from: instructor-alpha
          to: learners-in-cohort-1           # logical group, Auth Request per assignment
        - from: instructor-beta
          to: learners-in-cohort-2
        - from: instructor-gamma
          to: learners-in-cohort-3
    sponsorship:
      sponsors_projects:
        - sponsor: program-director
          projects: [phi-academy]
        - sponsor: instructor-alpha
          projects: [class-cohort-1]
        - sponsor: instructor-beta
          projects: [class-cohort-2]
        - sponsor: instructor-gamma
          projects: [class-cohort-3]

  owned_projects: [phi-academy, class-cohort-1, class-cohort-2, class-cohort-3]
```

## Cross-References

- [concepts/permissions/06 § Per-Session Consent](../concepts/permissions/06-multi-scope-consent.md#per-session-consent) — the policy protecting learner privacy.
- [concepts/permissions/05 § Template D](../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) — instructor → learner scope.
- [concepts/system-agents.md § Other System Agents (Future)](../concepts/system-agents.md#other-system-agents-future--out-of-v0-scope) — where the four edu-specific system agents point; in practice these are org-specific extensions beyond v0's standard two.
- [concepts/agent.md § Intern Agent](../concepts/agent.md#intern-agent-pre-economy) — learners are Interns.
