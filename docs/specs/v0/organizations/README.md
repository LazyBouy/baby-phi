<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Organization Layouts — Reference Catalogue

> Ten **illustrative, not normative** organization layouts. Each demonstrates one valid configuration under the v0 concept spec. The goal is to make the design space concrete and to give new org authors a starting point to fork from.
>
> **See also:** [concepts/organization.md](../concepts/organization.md), [concepts/permissions/07 § Standard Organization Template](../concepts/permissions/07-templates-and-tools.md#standard-organization-template), [concepts/system-agents.md](../concepts/system-agents.md).
>
> **The admin pages that create these shapes live in [requirements/admin/](../requirements/admin/00-fresh-install-journey-overview.md).** The layouts here are acceptance-scenario fodder: almost every admin page's Section 11 (Acceptance Scenarios) grounds its requirements in one or more of the 10 org layouts below.

## Index

| # | File | Domain | In one line |
|---|------|--------|-------------|
| 01 | [01-minimal-startup.md](01-minimal-startup.md) | 1 sponsor + 2 interns | Simplest possible org. |
| 02 | [02-mid-product-team.md](02-mid-product-team.md) | Mid-sized product team (3 leads + ~10 workers) | Flat two-level hierarchy with mixed intern / contract agents. |
| 03 | [03-consultancy-strict.md](03-consultancy-strict.md) | Privacy-focused consultancy | per_session consent, Template D only, bash disabled. |
| 04 | [04-regulated-enterprise.md](04-regulated-enterprise.md) | Compliance-heavy enterprise | `alerted` audit default, `#sensitive` catalogue, compliance auditor system agents. |
| 05 | [05-research-lab-nested.md](05-research-lab-nested.md) | Research lab with nested sub-orgs | Deep `HAS_SUBORGANIZATION`, long-duration projects, `parallelize: 4`. |
| 06 | [06-joint-venture-acme.md](06-joint-venture-acme.md) | Acme side of a joint venture | Co-owns `joint-research` with Beta. Implicit consent. |
| 07 | [07-joint-venture-beta.md](07-joint-venture-beta.md) | Beta side of the same joint venture | Co-owns with #06. one_time consent — exercises per-co-owner rule. |
| 08 | [08-marketplace-gig.md](08-marketplace-gig.md) | Contract-agent-first gig shop | All contract agents, Template E-heavy, market-ready. Inbox/outbox for bidding. |
| 09 | [09-education-org.md](09-education-org.md) | Education / learning org | System-agent-heavy, learners as interns, per_session consent. |
| 10 | [10-platform-infra.md](10-platform-infra.md) | Platform infrastructure custodian | Manages MCP servers, credentials, token economy for other orgs. |

## Knobs Matrix

A compact view of which knobs each layout exercises differently.

| # | Consent | Agent mix | Hierarchy | Audit | Co-own | Market | Inbox/Outbox usage | `parallelize` |
|---|---------|-----------|-----------|-------|--------|--------|---------------------|-----------------|
| 01 | implicit | 1 human + 2 interns | flat | silent | — | — | minimal | 1 |
| 02 | one_time | interns + contracts | flat (2 levels) | logged | — | — | light | 2 on interns |
| 03 | per_session | mostly contract | flat, strict project boundaries | logged | — | — | medium | 1 |
| 04 | per_session | mixed + compliance auditors | deep (Template C) | alerted (default) | — | — | medium | 1–2 |
| 05 | implicit | mix + system-heavy | nested sub-orgs (3 levels) | logged | — | — | heavy (cross-team refs) | **4** |
| 06 | implicit | mixed | flat | logged | **yes** (with 07) | — | medium | 2 |
| 07 | one_time | mixed | flat | logged | **yes** (with 06) | — | medium | 2 |
| 08 | one_time | **all contract** | flat, market-ready | logged | — | **structured for market** | **heavy** (bid negotiation) | 2–4 |
| 09 | per_session | **system-heavy** + learner-interns | flat-with-system-layer | logged | — | — | light (feedback loops) | 1 |
| 10 | implicit | **system-heavy** custodians | flat | alerted (default) | — | — | medium (cross-org announcements) | **8** on platform agents |

## Recommended Reading Order

- **Learning the model the first time:** `01` → `02` → `06`+`07` (to see co-ownership) → `10` (platform custodian).
- **Designing your own org:** pick the nearest example by knobs matrix, fork it, customise.
- **Understanding nested structure:** `05-research-lab-nested.md`.
- **Understanding co-ownership interactions:** `06-joint-venture-acme.md` + `07-joint-venture-beta.md` together, then [projects/03-joint-research.md](../projects/03-joint-research.md).
- **Understanding market and messaging-heavy orgs:** `08-marketplace-gig.md`.
- **Understanding infrastructure orgs:** `10-platform-infra.md`.

## Structure of Each Layout File

Every layout file follows the same six-section template:

1. **Header stub** (Status, Last verified, cross-links).
2. **Profile** — one-paragraph description of the domain, size, and distinguishing feature.
3. **Knobs Summary Table** — single table with each knob showing this org's choice.
4. **Narrative Walkthrough** — 2–4 paragraphs explaining the distinguishing choices and the scenarios where they matter.
5. **Full YAML Config** — complete `organization:` declaration with `resources_catalogue`, `system_agents`, `authority_templates_enabled`, `consent_policy`, `execution_limits`, `agent_roster` (using phi-core `AgentProfile` + `ModelConfig` + `ExecutionLimits`), and hierarchy (task + sponsorship).
6. **Cross-references** — links to the concept docs each choice exercises.
