<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin journey overview -->

# Fresh-Install Admin Journey — Overview

> A one-page map of the 9-phase sequence a Human Agent with platform-admin authority walks through to go from fresh install to "an agent can productively run a session." The sequence is also the bootstrap dependency chain, so the phases are strictly ordered.

## Phases

| Phase | Goal | Admin pages | Concept files exercised |
|-------|------|-------------|--------------------------|
| **1** | Claim platform admin role | [01-platform-bootstrap-claim.md](01-platform-bootstrap-claim.md) | [system-agents.md § Bootstrap](../../concepts/system-agents.md#how-system-agents-fit-the-standard-organization-template), [permissions/02 § System Bootstrap Template](../../concepts/permissions/02-auth-request.md#system-bootstrap-template--root-of-the-authority-tree) |
| **2** | Platform-level resource setup | [02-platform-model-providers.md](02-platform-model-providers.md), [03-platform-mcp-servers.md](03-platform-mcp-servers.md), [04-platform-credentials-vault.md](04-platform-credentials-vault.md), [05-platform-defaults.md](05-platform-defaults.md) | [permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue), [organization.md](../../concepts/organization.md), [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md) |
| **3** | Create first organization | [06-org-creation-wizard.md](06-org-creation-wizard.md) | [organization.md](../../concepts/organization.md), [permissions/07 § Standard Organization Template](../../concepts/permissions/07-templates-and-tools.md#standard-organization-template), all 10 org layouts |
| **4** | Organization home | [07-organization-dashboard.md](07-organization-dashboard.md) | [organization.md](../../concepts/organization.md) |
| **5** | Build agent roster | [08-agent-roster-list.md](08-agent-roster-list.md), [09-agent-profile-editor.md](09-agent-profile-editor.md) | [agent.md](../../concepts/agent.md) (Taxonomy, Parallelized Sessions), phi-core `AgentProfile` + `ModelConfig` + `ExecutionLimits` |
| **6** | Create first project | [10-project-creation-wizard.md](10-project-creation-wizard.md), [11-project-detail.md](11-project-detail.md) | [project.md](../../concepts/project.md) (OKR value objects), [permissions/05](../../concepts/permissions/05-memory-sessions.md) |
| **7** | Approve authority-template adoptions | [12-authority-template-adoption.md](12-authority-template-adoption.md) | [permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question), [permissions/07 § Templates Are Pre-Authorized Allocations](../../concepts/permissions/07-templates-and-tools.md#templates-are-pre-authorized-allocations) |
| **8** | Configure system agents | [13-system-agents-config.md](13-system-agents-config.md) | [system-agents.md](../../concepts/system-agents.md) |
| **9** | Launch first session | [14-first-session-launch.md](14-first-session-launch.md) | [agent.md § Experience](../../concepts/agent.md#experience-sessions--memory), [permissions/05 § Sessions as a Tagged Resource](../../concepts/permissions/05-memory-sessions.md#sessions-as-a-tagged-resource) |

## Dependency chain

```
Phase 1 (Bootstrap claim)
  │
  ▼
Phase 2 (Platform settings) — can begin only after Phase 1
  │      model providers, MCP servers, credentials, defaults
  │      each addition is an auditable catalogue-extension Auth Request
  ▼
Phase 3 (Create first org) — needs platform resources from Phase 2 to reference
  │
  ▼
Phase 4 (Org dashboard)
  │
  ▼
Phase 5 (Agent roster) — agents reference platform model providers from Phase 2
  │
  ▼
Phase 6 (First project) — projects reference agents from Phase 5
  │
  ▼
Phase 7 (Template adoption) — requires org + agents + project
  │
  ▼
Phase 8 (System agents config) — fine-tunes memory-extraction-agent + agent-catalog-agent
  │
  ▼
Phase 9 (First session) — needs agent + project; validates the whole stack
```

## Actors

- **Primary** throughout: Human Agent with platform-admin authority (gained in Phase 1). Typically delegates to an org-admin Human Agent from Phase 3 onward, who may be the same Human or a separate one.
- **Secondary** (from Phase 5 onward): Human and LLM Agents added to the roster. They appear as the subjects of rules configured on admin pages and as the actors on the agent-self-service pages.

## Steady-state vs journey

Once Phase 9 is complete the system is usable. The admin pages 07 (Org Dashboard) and 11 (Project Detail) transition from "setup wizard endpoints" to **steady-state operations surfaces**. Other steady-state operations pages (Auth Request approver queue, audit log viewer, grant browser, tenant management, ratings dashboard, memory pool browser, session browser, template management, org chart editor, token budget dashboard, etc.) are not part of this plan's scope — they will be specified in a follow-on requirements plan.

## Cross-references

- [../README.md](../README.md) — top-level requirements README with terminology, ID conventions, coverage goal.
- [../_template/admin-page-template.md](../_template/admin-page-template.md) — the normative 10-section template every page here follows.
- [../cross-cutting/traceability-matrix.md](../cross-cutting/traceability-matrix.md) — concept section → requirement ID coverage index.
