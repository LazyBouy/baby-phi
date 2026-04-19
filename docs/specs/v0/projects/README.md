<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Project Layouts — Reference Catalogue

> Five **illustrative, not normative** project layouts. Each demonstrates one valid configuration under the v0 concept spec. Together they span the design space of Shape A vs B, flat vs deeply nested, formal OKRs vs lightweight goal tracking, standing task assignment vs market bidding, and short sprint vs long-duration project.
>
> **See also:** [concepts/project.md](../concepts/project.md), [concepts/permissions/05 § Sessions as a Tagged Resource](../concepts/permissions/05-memory-sessions.md#sessions-as-a-tagged-resource), [concepts/permissions/06 § Multi-Scope Session Access](../concepts/permissions/06-multi-scope-consent.md#multi-scope-session-access).
>
> **The admin pages that create these project shapes are [requirements/admin/10-project-creation-wizard.md](../requirements/admin/10-project-creation-wizard.md) and [requirements/admin/11-project-detail.md](../requirements/admin/11-project-detail.md).** The layouts below are used as acceptance-scenario grounding in both pages' Section 11.

## Index

| # | File | Shape | In one line |
|---|------|-------|-------------|
| 01 | [01-flat-single-project.md](01-flat-single-project.md) | Shape A | Simplest: 1 lead + 3 workers, sprint-length OKR. |
| 02 | [02-deeply-nested-project.md](02-deeply-nested-project.md) | Shape A + 2 levels of sub-projects | Deep `HAS_SUBPROJECT` nesting with aggregated OKRs. |
| 03 | [03-joint-research.md](03-joint-research.md) | Shape B (two-org co-owned) | Acme + Beta joint research. Exercises per-co-owner consent rule. |
| 04 | [04-market-bid-project.md](04-market-bid-project.md) | Shape A, market-bid flow | Tasks posted for bidding. Template E + inbox/outbox negotiation. |
| 05 | [05-compliance-audit-project.md](05-compliance-audit-project.md) | Shape A, long duration | Quarterly compliance audit. 4 objectives × 3 KRs each. 7-year retention. |

## Knobs Matrix

| # | Shape | Sub-projects | Ownership | OKRs | Task flow | Duration | Audit posture |
|---|-------|-------------|-----------|------|-----------|----------|---------------|
| 01 | A | none | single-org | 2 objectives / 4 KRs | direct assignment | sprint (~6 weeks) | logged |
| 02 | A | **2 levels deep** | single-org | rolled up from sub-projects | direct assignment | quarter (~12 weeks) | logged |
| 03 | **B** | none | **two orgs co-own** | 3 objectives / 6 KRs (shared) | mixed | open-ended | logged |
| 04 | A | none | single-org | none (task-level, not project-level) | **market bidding** | per-task (variable) | logged |
| 05 | A | none | single-org | **4 objectives / 12 KRs** | direct assignment | **long** (quarterly, recurring) | **alerted** |

## Recommended Reading Order

- **Learning the model the first time:** `01` → `02` → `03`.
- **Understanding co-ownership end-to-end:** `03` (paired with orgs `06-joint-venture-acme` + `07-joint-venture-beta`).
- **Understanding market-bid flows:** `04` (paired with org `08-marketplace-gig`).
- **Understanding compliance-grade projects:** `05` (paired with org `04-regulated-enterprise`).

## Structure of Each Layout File

Every project layout file follows the same six-section template:

1. **Header stub** (Status, Last verified, cross-links).
2. **Profile** — one-paragraph description of the project type, size, and distinguishing feature.
3. **Knobs Summary Table** — shape, nesting, ownership, OKR cadence, task flow, duration.
4. **Narrative Walkthrough** — 2–3 paragraphs explaining the distinguishing choices and how they compose with the owning org's config.
5. **Full YAML Config** — complete `project:` declaration with `owning_orgs:`, `objectives`, `key_results`, `agent_roster` (referencing the owning org's catalogue), `tasks`, `resource_boundaries`, `sub_projects` (where relevant).
6. **Cross-references** — links to the concept docs and the owning org(s) in `/organizations/`.
