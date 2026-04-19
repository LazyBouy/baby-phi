<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Traceability Matrix — Concept → Requirements

> Maps each major concept section to the requirement IDs that implement it. Used as the authoritative coverage index — every concept section SHOULD have at least one requirement citing it.
>
> The matrix is **not exhaustive** (every requirement also cites its concept section in Section 12 of its own file); it is a **rolled-up view** that makes coverage gaps easy to spot.
>
> **Legend:**
> - `✓` — concept section has ≥1 requirement implementing it.
> - `–` — concept section is informational / rationale and has no requirement obligation.
> - (empty) — gap. A gap should prompt either a new requirement or an explicit note of why the section is out of scope.

## concepts/agent.md

| Section | Coverage | Requirements citing this section |
|---------|----------|-----------------------------------|
| Grounding Principle | ✓ | [README](../README.md) terminology; [admin/09](../admin/09-agent-profile-editor.md); [a01..a05](../agent-self-service/); all admin pages' actor definitions |
| Agent Taxonomy | ✓ | [admin/08 R1](../admin/08-agent-roster-list.md), [admin/09](../admin/09-agent-profile-editor.md) kind selection |
| LLM Agent Anatomy | ✓ | [admin/09](../admin/09-agent-profile-editor.md) (Soul/Power/Experience); [a05](../agent-self-service/a05-my-profile-and-grants.md) (Identity) |
| Parallelized Sessions | ✓ | [admin/09 W3](../admin/09-agent-profile-editor.md), [admin/08 R1](../admin/08-agent-roster-list.md), [a04 R3](../agent-self-service/a04-my-work.md) |
| Memory Model — Public, Private, Supervisor Extraction | ✓ | [s02](../system/s02-session-end-memory-extraction.md) |
| Identity (Emergent, Event-Driven) | ✓ | [a05 R2](../agent-self-service/a05-my-profile-and-grants.md), [a05 W1](../agent-self-service/a05-my-profile-and-grants.md) |
| Identity Node Content — Provisional Direction | ✓ | [a05 R2](../agent-self-service/a05-my-profile-and-grants.md) |
| Inter-Agent Messaging | ✓ | [a01](../agent-self-service/a01-my-inbox-outbox.md) |
| Agent Categories (System / Standard) | ✓ | [admin/13](../admin/13-system-agents-config.md) (system); [admin/08 R6](../admin/08-agent-roster-list.md) (distinction) |
| Intern Agent / Contract Agent | ✓ | [admin/08 R1, R2](../admin/08-agent-roster-list.md), [admin/09 W3](../admin/09-agent-profile-editor.md) kind enum |
| Mode Transitions | – | (promotion is a separate flow; not in fresh-install journey) |

## concepts/human-agent.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Human Agents | ✓ | [admin/01 W1](../admin/01-platform-bootstrap-claim.md) creates first Human Agent; [admin/06](../admin/06-org-creation-wizard.md) step 7 |
| Channel routing (Slack/email/web) | ✓ | [admin/01 W1](../admin/01-platform-bootstrap-claim.md); [admin/06 step 7](../admin/06-org-creation-wizard.md); [a01 N2](../agent-self-service/a01-my-inbox-outbox.md) |

## concepts/organization.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Organization (Node Type) + Properties | ✓ | [admin/06](../admin/06-org-creation-wizard.md) |
| Organization Edges | ✓ | [admin/06](../admin/06-org-creation-wizard.md); [admin/08 R1](../admin/08-agent-roster-list.md) MEMBER_OF |
| Organization × Project × Agent resolution hierarchy | ✓ | [NFR-security R6](nfr-security.md); implicit in [admin/06](../admin/06-org-creation-wizard.md) |
| Market (Future Concept — Placeholder) | – | marked OUT OF V0 SCOPE in the concept itself |

## concepts/project.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Project (Node Type) + Properties | ✓ | [admin/10](../admin/10-project-creation-wizard.md), [admin/11](../admin/11-project-detail.md) |
| Project Status lifecycle | ✓ | [admin/11 W1, W6](../admin/11-project-detail.md) |
| Project Edges | ✓ | [admin/10 W2](../admin/10-project-creation-wizard.md), [admin/11 R4, R5](../admin/11-project-detail.md) |
| Objectives and Key Results (OKRs) | ✓ | [admin/10 R3, R4 / W5](../admin/10-project-creation-wizard.md), [admin/11 R2, W2](../admin/11-project-detail.md) |
| Task + Task Status Flow | ✓ | [admin/11 R3, W3](../admin/11-project-detail.md), [a04 W1](../agent-self-service/a04-my-work.md) |
| Bid / Rating | – | market-driven; used by [organizations/08-marketplace-gig](../../organizations/08-marketplace-gig.md) but full bidding flow is a downstream concern |

## concepts/permissions/01-resource-ontology.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Two-Tier Resource Ontology | ✓ | all system flows assume; [admin/02..05](../admin/02-platform-model-providers.md) populate the catalogue |
| Fundamental Classes (9) | ✓ | [admin/04](../admin/04-platform-credentials-vault.md) (secret/credential), [admin/02](../admin/02-platform-model-providers.md) (various) |
| Composite Classes (8) | ✓ | [admin/02](../admin/02-platform-model-providers.md), [admin/03](../admin/03-platform-mcp-servers.md), [a01](../agent-self-service/a01-my-inbox-outbox.md) (inbox/outbox) |
| Resource Class Overlaps (Entity-Level) | ✓ | [NFR-security R2](nfr-security.md) catalogue precondition |
| Resource Ownership | ✓ | [admin/04 § W2](../admin/04-platform-credentials-vault.md) custody, [admin/06 step 5](../admin/06-org-creation-wizard.md) |
| Resource Catalogue | ✓ | [admin/02](../admin/02-platform-model-providers.md)–[admin/05](../admin/05-platform-defaults.md); [NFR-security R2](nfr-security.md) Step 0 |
| Composite Identity Tags / Instance Identity Tags | ✓ | implicit across admin/* permission rules |

## concepts/permissions/02-auth-request.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Schema — Per-Resource Slot Model | ✓ | [s04 R2, R7](../system/s04-auth-request-state-transitions.md), [a02 R4](../agent-self-service/a02-my-auth-requests.md) |
| State Machine | ✓ | [s04 R1..R6](../system/s04-auth-request-state-transitions.md) |
| Per-State Access Matrix | ✓ | [a02 W1..W5](../agent-self-service/a02-my-auth-requests.md) |
| Routing Table on Resource + Escalation | ✓ | [s04 R5](../system/s04-auth-request-state-transitions.md) |
| `allocate` Scope Semantics | ✓ | [admin/01](../admin/01-platform-bootstrap-claim.md), [admin/06](../admin/06-org-creation-wizard.md), [admin/08 permission rules](../admin/08-agent-roster-list.md) |
| Retention Policy | ✓ | [admin/05](../admin/05-platform-defaults.md), [s06 R1..R3](../system/s06-periodic-triggers.md), [NFR-observability R3, R14](nfr-observability.md) |
| System Bootstrap Template — Root of the Authority Tree | ✓ | [admin/01](../admin/01-platform-bootstrap-claim.md), [s01](../system/s01-bootstrap-template-adoption.md) |

## concepts/permissions/03-action-vocabulary.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Standard Action Vocabulary | ✓ | implicit; every admin page's W-requirements reference specific actions |
| `allocate` umbrella; `transfer` | ✓ | [admin/04 W3](../admin/04-platform-credentials-vault.md) reassign custody; [admin/06](../admin/06-org-creation-wizard.md) |
| Constraints | ✓ | [admin/02 W1](../admin/02-platform-model-providers.md) tenants_allowed; [admin/09 W3](../admin/09-agent-profile-editor.md) |
| Per-Resource-Class Reference | – | informational — implementation-side |

## concepts/permissions/04-manifest-and-resolution.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Two Shapes (Manifest vs Grant) | ✓ | [admin/09 § 9](../admin/09-agent-profile-editor.md) preview |
| Permission Check (Runtime Reconciliation) | ✓ | [admin/14 R3](../admin/14-first-session-launch.md) preview; [NFR-performance R4](nfr-performance.md) |
| Formal Algorithm (Pseudocode) | ✓ | [NFR-security R1, R2](nfr-security.md); [admin/14 R3](../admin/14-first-session-launch.md) |
| The Authority Chain | ✓ | [a05 R4](../agent-self-service/a05-my-profile-and-grants.md); [NFR-observability R6](nfr-observability.md) |

## concepts/permissions/05-memory-sessions.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Memory as a Resource Class | ✓ | [s02](../system/s02-session-end-memory-extraction.md); [a05 R2](../agent-self-service/a05-my-profile-and-grants.md) |
| Sessions as a Tagged Resource | ✓ | [admin/14 W1](../admin/14-first-session-launch.md) session tagging |
| Authority Templates (A–E) | ✓ | [admin/12](../admin/12-authority-template-adoption.md), [s05](../system/s05-template-adoption-grant-fires.md) |
| Default Grants Issued to Every Agent | ✓ | [admin/09 W1 § 9](../admin/09-agent-profile-editor.md), [a01](../agent-self-service/a01-my-inbox-outbox.md) |
| Inbox and Outbox (Agent Messaging) | ✓ | [a01](../agent-self-service/a01-my-inbox-outbox.md) |
| Supervisor Extraction as Two Standard Grants | ✓ | [s02 R2](../system/s02-session-end-memory-extraction.md) |

## concepts/permissions/06-multi-scope-consent.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Multi-Scope Session Access (shapes A–D + forbidden E) | ✓ | [admin/10 W2, W3](../admin/10-project-creation-wizard.md); [s02 R5, R6](../system/s02-session-end-memory-extraction.md) |
| Co-Ownership × Multi-Scope Session Access | ✓ | [admin/10 W3](../admin/10-project-creation-wizard.md); [NFR-security R5, R6](nfr-security.md) |
| Consent Policy (3 policies) | ✓ | [admin/05](../admin/05-platform-defaults.md), [admin/06 step 3](../admin/06-org-creation-wizard.md), [a03](../agent-self-service/a03-my-consent-records.md) |
| Consent Lifecycle | ✓ | [a03 W1–W3](../agent-self-service/a03-my-consent-records.md) |

## concepts/permissions/07-templates-and-tools.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Standard Organization Template | ✓ | [admin/06](../admin/06-org-creation-wizard.md) |
| Templates Are Pre-Authorized Allocations | ✓ | [admin/12](../admin/12-authority-template-adoption.md); [s05](../system/s05-template-adoption-grant-fires.md) |
| `audit_class` Composition Through Templates | ✓ | [admin/05](../admin/05-platform-defaults.md); [NFR-observability R1, R2](nfr-observability.md) |
| Tool Authority Manifest Examples (14 tools) | – | reference catalogue; individual tools are referenced from relevant admin pages |
| Authoring a Tool Manifest | – | authoring-side; out of scope for the journey requirements |

## concepts/permissions/08-worked-example.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Worked End-to-End Use Case | – | informational / acceptance grounding across all pages |
| Step 9 (Ad-hoc Auth Request) | ✓ | [a02 Scenario 1](../agent-self-service/a02-my-auth-requests.md) |

## concepts/permissions/09-selector-grammar.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Normative PEG Grammar | – | reference for implementers; not a page-level requirement |

## concepts/system-agents.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Memory Extraction Agent | ✓ | [s02](../system/s02-session-end-memory-extraction.md); [admin/13](../admin/13-system-agents-config.md) |
| Agent Catalog Agent | ✓ | [s03](../system/s03-edge-change-catalog-update.md); [admin/08 R7](../admin/08-agent-roster-list.md) |
| Bootstrap Template adoption unified with system agents pattern | ✓ | [s01](../system/s01-bootstrap-template-adoption.md) |
| Other System Agents (future stubs) | – | intentionally not in v0 |

## concepts/token-economy.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Worth / Value / Meaning | ✓ | [NFR-cost R7](nfr-cost.md) contract savings; [admin/08 R1](../admin/08-agent-roster-list.md) rating display |
| Rating window | – | downstream per-rating mechanics |
| Bidding process | – | market is deferred |
| Value rolling window (decided) | ✓ | [NFR-cost R7](nfr-cost.md) |

## concepts/coordination.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Design Decisions (v0 defaults) | ✓ | [NFR-performance R11](nfr-performance.md) (LWW); [NFR-observability R4, R14](nfr-observability.md) (hybrid event sourcing); implicit in every system flow |
| Coordination patterns (Blackboard, Event-Driven, etc.) | – | not load-bearing for fresh-install journey |

## concepts/ontology.md

| Section | Coverage | Requirements |
|---------|----------|---------------|
| Node Types + Edge Types | ✓ | implicit — every admin page's W-actions produce node/edge mutations per ontology; [admin/06](../admin/06-org-creation-wizard.md) creates Organization/Agent/etc. |
| Governance Wiring | ✓ | [s04](../system/s04-auth-request-state-transitions.md), [NFR-security R3](nfr-security.md) |
| Tag Conventions | – | reference; implementation-side |
| Schema Registry (Meta-Graph) | – | v0 does not expose schema extension via UI pages (the rule exists in the concept; no admin page adds schema in this journey) |

---

## Coverage summary

- All permission model sections have ≥1 requirement reference. The permissions subsystem is fully covered.
- Token-economy's rating/bidding are deferred (marketplace flow is a follow-on concern).
- Schema-registry extension via UI is not a fresh-install page; the concept rule stands on its own.
- ontology.md's node/edge enumeration is implicit throughout — requirements don't cite each individual node type but they collectively exercise the whole ontology.

**Gaps identified: none load-bearing for the journey.** Areas marked `–` are intentional — informational content in the concept that does not translate to an admin-facing requirement.
