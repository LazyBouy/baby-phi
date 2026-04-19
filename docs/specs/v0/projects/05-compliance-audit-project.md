<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- One of 5 reference project layouts — see README.md -->

# 05 — `compliance-audit-project`

## Profile

A **long-duration quarterly compliance audit** inside [04-regulated-enterprise.md](../organizations/04-regulated-enterprise.md). Formal OKRs with 4 Objectives × 3 Key Results each (12 KRs total). Auditor roles with `#sensitive` grants. All Auth Requests retained under `delete_after_years: 7` per regulatory mandate. `audit_class: alerted` on every grant-issued action — standard for this org but especially load-bearing here because every auditor read is a reviewable event.

## Knobs Summary

| Knob | Choice |
|------|--------|
| Shape | A |
| Sub-projects | none (flat; separate audit scopes are tracked as Objectives) |
| Owning org | `regulated-enterprise` |
| OKRs | **4 Objectives × 3 KRs each = 12 KRs** |
| Task flow | direct assignment to auditors |
| Duration | **full quarter (13 weeks); recurring quarterly** |
| Audit posture | **`alerted`** |
| Consent | `per_session` (inherits org) |
| Retention | `delete_after_years: 7` |
| Special roles | compliance auditor (read-only into `#sensitive` resources) |

## Narrative

A quarterly compliance audit is the textbook **formal, long-duration project**. Four Objectives cover the regulatory pillars: data handling, access control review, incident documentation, and third-party risk. Each has three Key Results — a mix of Boolean completions and Percentage thresholds.

**Auditors are a distinct role.** An auditor's grant set is **read-only** on `#sensitive` resources, scoped to the audit period. They cannot modify or delete; they can `read`, `list`, `inspect`. Their grants carry `audit_class: alerted` and `purpose: compliance_audit` constraints. When the audit period ends, the grants are revoked forward-only — the auditor's past reads remain in the audit log, but no new reads are permitted under the expired grants.

**`delete_after_years: 7`** on all Auth Requests in this project's scope satisfies the regulatory mandate. The Auth Request retention policy applies org-wide (inherited from `regulated-enterprise`), but this project is explicit about it because compliance auditors are the first readers to rely on it during future retrospective investigations.

**Sub-projects intentionally not used.** Unlike `02-deeply-nested-project.md` where sub-projects carry sub-OKRs, this project's Objectives are the top-level organising unit. The rationale: compliance audits are deliberately flat so that every KR is traceable to the single project identity at report time.

## Full YAML Config

```yaml
project:
  project_id: quarterly-audit
  name: "Q2 2026 Compliance Audit"
  description: "Quarterly compliance audit covering data, access, incidents, and third-party risk."
  goal: "Full audit report submitted to regulators on schedule; no unaddressed findings."
  status:
    state: InProgress
    progress_percent: 30
    reason: "Week 4 of 13; data pillar complete; access pillar in progress."
  token_budget: 30_000_000
  tokens_spent: 8_400_000
  created_at: 2026-04-01T09:00:00Z

  owning_orgs:
    - org_id: regulated-enterprise
      role: primary

  objectives:
    - objective_id: obj-data-handling
      name: "Data handling compliance"
      description: "Verify PII/PHI handling meets regulatory standards across all in-scope systems."
      status: Active
      owner: auditor-data
      deadline: 2026-06-01T00:00:00Z
      key_result_ids: [kr-data-inventory, kr-data-encryption, kr-data-retention-review]

    - objective_id: obj-access-control
      name: "Access control review"
      description: "Verify least-privilege and timely revocation across in-scope systems."
      status: Active
      owner: auditor-access
      deadline: 2026-06-15T00:00:00Z
      key_result_ids: [kr-grants-audited, kr-stale-grants-removed, kr-access-review-signed]

    - objective_id: obj-incidents
      name: "Incident documentation"
      description: "All incidents in the audit period are documented, classified, and remediated or risk-accepted."
      status: Active
      owner: auditor-incidents
      deadline: 2026-06-30T00:00:00Z
      key_result_ids: [kr-incidents-listed, kr-incidents-classified, kr-incidents-remediated]

    - objective_id: obj-third-party
      name: "Third-party risk review"
      description: "All in-scope third-party services (MCP servers, external APIs, vendors) reviewed for risk posture."
      status: Active
      owner: auditor-third-party
      deadline: 2026-06-30T00:00:00Z
      key_result_ids: [kr-vendors-listed, kr-vendors-attested, kr-risk-matrix-updated]

  key_results:
    # Objective 1 — Data handling
    - kr_id: kr-data-inventory
      name: "PII/PHI inventory complete"
      measurement_type: Boolean
      target_value: true
      current_value: true
      owner: auditor-data
      status: Achieved

    - kr_id: kr-data-encryption
      name: "Encryption-at-rest coverage"
      measurement_type: Percentage
      target_value: 1.0
      current_value: 0.98
      owner: auditor-data
      deadline: 2026-05-20T00:00:00Z
      status: InProgress

    - kr_id: kr-data-retention-review
      name: "Retention policies reviewed for all stores"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: auditor-data
      deadline: 2026-06-01T00:00:00Z
      status: InProgress

    # Objective 2 — Access control
    - kr_id: kr-grants-audited
      name: "All active Grants reviewed"
      measurement_type: Percentage
      target_value: 1.0
      current_value: 0.45
      owner: auditor-access
      deadline: 2026-06-10T00:00:00Z
      status: InProgress

    - kr_id: kr-stale-grants-removed
      name: "Stale Grants revoked"
      measurement_type: Count
      target_value: 0                         # target: zero stale grants remain
      current_value: 23                       # initial finding; decreases over audit
      owner: auditor-access
      deadline: 2026-06-15T00:00:00Z
      status: InProgress

    - kr_id: kr-access-review-signed
      name: "VP sign-off on access review report"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: vp-compliance
      deadline: 2026-06-15T00:00:00Z
      status: NotStarted

    # Objective 3 — Incidents
    - kr_id: kr-incidents-listed
      name: "All quarter's incidents listed"
      measurement_type: Boolean
      target_value: true
      current_value: true
      owner: auditor-incidents
      status: Achieved

    - kr_id: kr-incidents-classified
      name: "Incidents classified by severity"
      measurement_type: Percentage
      target_value: 1.0
      current_value: 0.80
      owner: auditor-incidents
      deadline: 2026-06-20T00:00:00Z
      status: InProgress

    - kr_id: kr-incidents-remediated
      name: "Incidents remediated or risk-accepted with sign-off"
      measurement_type: Percentage
      target_value: 1.0
      current_value: 0.60
      owner: auditor-incidents
      deadline: 2026-06-30T00:00:00Z
      status: InProgress

    # Objective 4 — Third-party
    - kr_id: kr-vendors-listed
      name: "Third-party vendor inventory complete"
      measurement_type: Boolean
      target_value: true
      current_value: true
      owner: auditor-third-party
      status: Achieved

    - kr_id: kr-vendors-attested
      name: "Vendors with current attestation"
      measurement_type: Percentage
      target_value: 1.0
      current_value: 0.70
      owner: auditor-third-party
      deadline: 2026-06-25T00:00:00Z
      status: InProgress

    - kr_id: kr-risk-matrix-updated
      name: "Risk matrix updated with Q2 findings"
      measurement_type: Boolean
      target_value: true
      current_value: false
      owner: vp-compliance
      deadline: 2026-06-30T00:00:00Z
      status: NotStarted

  agent_roster:
    - id: vp-compliance
      role: sponsor
    - id: auditor-data
      role: lead                              # auditor lead for the data pillar
    - id: auditor-access
      role: lead
    - id: auditor-incidents
      role: lead
    - id: auditor-third-party
      role: lead
    - id: compliance-audit-agent              # the org-specific system agent contributes read-only analysis
      role: system_support

  tasks:
    - task_id: task-data-inventory
      name: "Complete PII/PHI inventory"
      assigned_to: auditor-data
      status: Completed
      linked_kr: kr-data-inventory

    - task_id: task-grants-sweep
      name: "Sweep all active Grants"
      assigned_to: auditor-access
      status: InProgress
      linked_kr: kr-grants-audited

    - task_id: task-stale-grants-revoke
      name: "Revoke stale Grants"
      assigned_to: auditor-access
      status: InProgress
      linked_kr: kr-stale-grants-removed

    - task_id: task-incident-classify
      name: "Classify incidents by severity"
      assigned_to: auditor-incidents
      status: InProgress
      linked_kr: kr-incidents-classified

    - task_id: task-vendor-attest
      name: "Collect vendor attestations"
      assigned_to: auditor-third-party
      status: InProgress
      linked_kr: kr-vendors-attested

  resource_boundaries:
    filesystem_objects:
      - path: /workspace/quarterly-audit/**
      - path: /workspace/{project}/regulated/**       # #sensitive paths across all projects; auditors read-only
    process_exec_objects:
      - id: sandboxed-shell
    network_endpoints:
      - domain: api.anthropic.com
      - domain: internal-compliance-api.meridian.com
    secrets:
      - id: pii-store-access-key                     # read-only to auditors
      - id: phi-store-access-key                     # read-only to auditors
    memory_objects:
      - scope: per-agent
      - scope: per-project
      - scope: per-org-sensitive                     # the auditors read the sensitive pool
    session_objects:
      - scope: per-project
      - scope: per-agent
    control_plane_objects:
      - id: compliance-log
      - id: policy-store

  compliance_settings:
    audit_class: alerted                             # all grant-issued actions page
    auditor_grants_read_only: true
    auth_request_retention:
      active_window_days: 365
      archived_retrieval_approval: human_required
      delete_after_years: 7                         # regulatory mandate
    third_party_attestation_required: true

  sub_projects: []
```

## Cross-References

- [organizations/04-regulated-enterprise.md](../organizations/04-regulated-enterprise.md) — the owning org with `alerted` default and `delete_after_years: 7`.
- [concepts/permissions/02 § Retention Policy](../concepts/permissions/02-auth-request.md#retention-policy) — the Auth Request retention rules this project depends on.
- [concepts/permissions/07 § `audit_class` Composition Through Templates](../concepts/permissions/07-templates-and-tools.md#audit_class-composition-through-templates) — how `alerted` composes with the auditor grants.
- [concepts/project.md § Objectives and Key Results](../concepts/project.md#objectives-and-key-results-okrs) — the OKR schema this project populates heavily.
