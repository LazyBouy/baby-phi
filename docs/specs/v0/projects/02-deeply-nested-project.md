<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 5 reference project layouts — see README.md -->

# 02 — `deeply-nested-project`

## Profile

A quarterly project with **two levels of `HAS_SUBPROJECT` nesting**. The parent project has two sub-projects; each sub-project has two sub-sub-projects. Each level has its own lead and its own OKRs. The parent's Key Results are **aggregated from the children** — the parent declares how aggregation rolls up. Owned by [05-research-lab-nested.md](../organizations/05-research-lab-nested.md) as the `vision-benchmark-study` project.

## Knobs Summary

| Knob | Choice |
|------|--------|
| Shape | A (one owning org) |
| Sub-projects | **2 levels deep**: parent → 2 sub-projects → each with 2 sub-sub-projects (7 total projects) |
| Owning org | `phi-research-lab` |
| OKRs | Parent: 3 Objectives, KRs aggregated from children |
| Task flow | Direct assignment at each level by that level's lead |
| Duration | ~12 weeks |
| Audit posture | `logged` |

## Narrative

The parent project **`vision-benchmark-study`** is a research effort that decomposes naturally into two workstreams: dataset construction and model evaluation. Each workstream further decomposes. The **`HAS_SUBPROJECT` edge** is the structural primitive; each level is a complete Project node with its own fields.

**OKR aggregation** is the distinctive pattern. The parent declares:

- `Objective: "Publish benchmark paper"` — aggregates KRs from both sub-projects.
- `KeyResult: "Overall benchmark complete"` — composite, computed as `AND(sub-1.complete, sub-2.complete)` (boolean AND of child completion KRs).
- `KeyResult: "Total compute budget under ceiling"` — composite, computed as sum of children's compute spend.

Each sub-project has its **own** OKRs. The parent's KRs reference children's KRs by `kr_id` (the `key_result_ids` field on the parent objective includes child KR IDs).

**Leads at each level.** Parent lead is a Team Lead (`team-lead-vision` in `phi-research-lab`). Each sub-project lead is a researcher acting as a sub-lead for that scope. Template A fires **per level** — the parent lead sees all parent-level sessions; a sub-project lead sees that sub-project's sessions. Cross-level reads require Template E (or Template C if the org chart is enabled, which `phi-research-lab` does).

## Full YAML Config

```yaml
project:
  project_id: vision-benchmark-study
  name: "Vision Benchmark Study (Q2)"
  description: "Publish a vision benchmark paper; decomposes into dataset and evaluation workstreams."
  goal: "Benchmark paper published to ArXiv by end of quarter."
  status:
    state: InProgress
    progress_percent: 55
    reason: "Week 7 of 12 — dataset complete, evaluation in progress."
  token_budget: 100_000_000
  tokens_spent: 48_000_000
  created_at: 2026-04-01T09:00:00Z

  owning_orgs:
    - org_id: phi-research-lab
      role: primary

  objectives:
    - objective_id: obj-publish
      name: "Publish the paper"
      description: "Submit to ArXiv with full benchmark results."
      status: Active
      owner: team-lead-vision
      deadline: 2026-07-01T00:00:00Z
      key_result_ids: [kr-paper-draft, kr-benchmark-complete, kr-compute-under-ceiling]

    - objective_id: obj-benchmark
      name: "Establish the benchmark"
      description: "Curated dataset + rigorous evaluation across candidate models."
      status: Active
      owner: team-lead-vision
      deadline: 2026-06-15T00:00:00Z
      key_result_ids: [kr-benchmark-complete, kr-dataset-curated, kr-eval-complete]

    - objective_id: obj-reproducibility
      name: "Make it reproducible"
      description: "Open data, open code, versioned runs."
      status: Active
      owner: team-lead-vision
      deadline: 2026-06-20T00:00:00Z
      key_result_ids: [kr-open-data, kr-open-code]

  key_results:
    - kr_id: kr-paper-draft
      name: "Paper draft ready for internal review"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: team-lead-vision
      deadline: 2026-06-20T00:00:00Z
      status: NotStarted

    - kr_id: kr-benchmark-complete
      name: "Benchmark complete"
      description: "Aggregated: AND(sub-1.dataset-complete, sub-2.eval-complete)"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: team-lead-vision
      deadline: 2026-06-15T00:00:00Z
      status: InProgress
      aggregation: "AND(sub-project:dataset-construction.kr-dataset-complete, sub-project:model-evaluation.kr-eval-complete)"

    - kr_id: kr-compute-under-ceiling
      name: "Total compute spend under ceiling"
      description: "Aggregated: sum of children's compute spend ≤ 100M tokens"
      measurement_type: Count
      target_value: 100_000_000
      current_value: 48_000_000                 # equals tokens_spent so far
      owner: team-lead-vision
      deadline: 2026-07-01T00:00:00Z
      status: InProgress
      aggregation: "sum(sub-project:dataset-construction.tokens_spent, sub-project:model-evaluation.tokens_spent)"

    - kr_id: kr-dataset-curated
      name: "Dataset curated"
      description: "References sub-project:dataset-construction.kr-dataset-complete directly"
      measurement_type: Boolean
      target_value: true
      current_value: true
      owner: team-lead-vision
      status: Achieved
      aggregation: "sub-project:dataset-construction.kr-dataset-complete"

    - kr_id: kr-eval-complete
      name: "Evaluation complete"
      description: "References sub-project:model-evaluation.kr-eval-complete"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: team-lead-vision
      status: InProgress
      aggregation: "sub-project:model-evaluation.kr-eval-complete"

    - kr_id: kr-open-data
      name: "Dataset published with DOI"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: researcher-vision-1
      deadline: 2026-06-18T00:00:00Z
      status: NotStarted

    - kr_id: kr-open-code
      name: "Code published with reproducibility instructions"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: researcher-vision-1
      deadline: 2026-06-20T00:00:00Z
      status: NotStarted

  agent_roster:
    - id: team-lead-vision
      role: lead
    - id: researcher-vision-1
      role: member
    - id: researcher-vision-2
      role: member
    - id: researcher-vision-3
      role: member

  tasks:
    - task_id: task-finalise-paper
      name: "Finalise paper draft"
      assigned_to: team-lead-vision
      status: NotStarted
      linked_kr: kr-paper-draft

  # ── Sub-projects (level 1) ──────────────────────────────────────────
  sub_projects:
    - project_id: dataset-construction
      name: "Dataset Construction"
      description: "Curate, label, and version the vision benchmark dataset."
      goal: "Dataset v1.0 published."
      status: { state: InProgress, progress_percent: 95 }
      token_budget: 40_000_000
      tokens_spent: 18_000_000
      owning_orgs: [{ org_id: phi-research-lab, role: primary }]
      parent_project: vision-benchmark-study

      objectives:
        - objective_id: obj-dataset-done
          name: "Complete the dataset"
          status: Active
          owner: researcher-vision-1
          deadline: 2026-05-30T00:00:00Z
          key_result_ids: [kr-dataset-complete, kr-labels-verified]

      key_results:
        - kr_id: kr-dataset-complete
          name: "Dataset curated, labelled, versioned"
          measurement_type: Boolean
          target_value: true
          current_value: true
          owner: researcher-vision-1
          status: Achieved

        - kr_id: kr-labels-verified
          name: "Labels verified by second annotator"
          measurement_type: Percentage
          target_value: 1.0
          current_value: 0.98
          owner: researcher-vision-1
          status: InProgress

      agent_roster:
        - id: researcher-vision-1
          role: lead
        - id: researcher-vision-2
          role: member

      # Sub-sub-projects (level 2) ────────────────────────────────────
      sub_projects:
        - project_id: dataset-curation
          name: "Curation"
          description: "Select candidate images from source corpora."
          status: { state: Finished, progress_percent: 100 }
          token_budget: 15_000_000
          tokens_spent: 14_200_000
          parent_project: dataset-construction
          owning_orgs: [{ org_id: phi-research-lab, role: primary }]
          agent_roster:
            - id: researcher-vision-1
              role: lead

        - project_id: dataset-labelling
          name: "Labelling"
          description: "Annotate curated images per taxonomy."
          status: { state: InProgress, progress_percent: 98 }
          token_budget: 25_000_000
          tokens_spent: 3_800_000
          parent_project: dataset-construction
          owning_orgs: [{ org_id: phi-research-lab, role: primary }]
          agent_roster:
            - id: researcher-vision-2
              role: lead

    - project_id: model-evaluation
      name: "Model Evaluation"
      description: "Run candidate models against the benchmark; analyse results."
      goal: "Full evaluation report ready."
      status: { state: InProgress, progress_percent: 35 }
      token_budget: 60_000_000
      tokens_spent: 30_000_000
      owning_orgs: [{ org_id: phi-research-lab, role: primary }]
      parent_project: vision-benchmark-study

      objectives:
        - objective_id: obj-eval-done
          name: "Complete the evaluation"
          status: Active
          owner: researcher-vision-3
          deadline: 2026-06-15T00:00:00Z
          key_result_ids: [kr-eval-complete, kr-analysis-ready]

      key_results:
        - kr_id: kr-eval-complete
          name: "All candidate models evaluated"
          measurement_type: Count
          target_value: 6
          current_value: 2
          owner: researcher-vision-3
          status: InProgress

        - kr_id: kr-analysis-ready
          name: "Results analysis complete"
          measurement_type: Boolean
          target_value: true
          current_value: false
          owner: researcher-vision-3
          status: NotStarted

      agent_roster:
        - id: researcher-vision-3
          role: lead

      sub_projects:
        - project_id: eval-candidate-models
          name: "Evaluate candidate models"
          description: "Run the benchmark against the 6 candidate models."
          status: { state: InProgress, progress_percent: 33 }
          token_budget: 50_000_000
          tokens_spent: 28_000_000
          parent_project: model-evaluation
          owning_orgs: [{ org_id: phi-research-lab, role: primary }]
          agent_roster:
            - id: researcher-vision-3
              role: lead

        - project_id: eval-statistical-analysis
          name: "Statistical analysis"
          description: "Significance testing and result interpretation."
          status: { state: NotStarted, progress_percent: 0 }
          token_budget: 10_000_000
          tokens_spent: 2_000_000
          parent_project: model-evaluation
          owning_orgs: [{ org_id: phi-research-lab, role: primary }]
          agent_roster:
            - id: researcher-vision-3
              role: lead

  resource_boundaries:
    # Subset of phi-research-lab's catalogue
    filesystem_objects:
      - path: /workspace/vision-benchmark-study/**
      - path: /datasets/perception/**           # group-scoped; accessible to this project
    process_exec_objects:
      - id: sandboxed-shell
      - id: gpu-compute
    network_endpoints:
      - domain: api.anthropic.com
      - domain: api.openai.com
      - domain: arxiv.org
    secrets:
      - id: anthropic-api-key
      - id: openai-api-key
    memory_objects:
      - scope: per-agent
      - scope: per-project
      - scope: per-team                         # team-vision pool
    session_objects:
      - scope: per-project
      - scope: per-agent
    model_runtime_objects:
      - id: claude-sonnet-default
      - id: gpt-4o-default
    compute_resources:
      - id: gpu-compute-pool
```

## Cross-References

- [concepts/project.md § Objectives and Key Results](../concepts/project.md#objectives-and-key-results-okrs) — OKR aggregation via `key_result_ids` and `aggregation` expressions.
- [concepts/project.md § Project Edges](../concepts/project.md#project-edges) — `HAS_SUBPROJECT` cardinality 1:N, chainable.
- [organizations/05-research-lab-nested.md](../organizations/05-research-lab-nested.md) — the owning org with matching nested sub-org shape.
