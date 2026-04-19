<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# baby-phi v0 Requirements

> Derived from the v0 concept docs. Framed as an **admin-UI-first journey**: requirements are grouped by the pages a Human Agent with admin authority walks through to go from fresh install to "an agent can productively run a session." That sequence is also the bootstrap dependency chain — requirements therefore order themselves.

## What this folder is

- **Admin pages** (`admin/`) — a fixed 9-phase fresh-install journey with 14 pages + one overview. Primary actors are Human Agents with platform-admin or org-admin authority.
- **Agent self-service pages** (`agent-self-service/`) — surfaces that any Agent (Human or LLM) uses to manage their own participation: inbox/outbox, consent records, Auth Requests they filed or must approve, assigned tasks, profile and grants.
- **System flows** (`system/`) — headless, reactive behaviours triggered by events (session end, edge change, periodic timers, etc.). Admin pages *provision* these; this folder captures their runtime contract.
- **Cross-cutting** (`cross-cutting/`) — NFRs (see below) + traceability matrix.
- **Template** (`_template/admin-page-template.md`) — the normative 10-section template every page in `admin/` and `agent-self-service/` follows.

## Terminology

- **FR — Functional Requirement.** What the system *does*. Example: "The admin page SHALL allow creating an Agent with a supplied AgentProfile."
- **NFR — Non-Functional Requirement.** Quality attributes: performance, observability, security, cost. Example: "The Permission Check SHALL complete in ≤20ms p95."
- **"Admin"** is **not a distinct entity kind** — per [concepts/agent.md § Grounding Principle](../concepts/agent.md#grounding-principle), everything is an Agent, including humans. "Admin" is a **role** that a Human Agent holds by virtue of the grants they carry:
  - **Platform admin** = Human Agent holding `[allocate]` on `system:root`.
  - **Org admin** = Human Agent holding `[allocate]` on the org's `agent-catalogue` / `resources_catalogue` `control_plane_object` instances (typically the CEO).

  Every permission rule on every page resolves to concrete grants on a concrete Human Agent. No ambient "admin" privilege exists.

## Requirement ID conventions

All requirements carry a short unique ID:

| Prefix | Domain | Example |
|--------|--------|---------|
| `R-ADMIN-<page>-<n>` | An admin page requirement. `<page>` is the two-digit page number (e.g. `08`); `<n>` is the sequence within that page, optionally prefixed with a category letter (`R<n>` for reads, `W<n>` for writes, `N<n>` for notifications). | `R-ADMIN-08-R3`, `R-ADMIN-08-W1`, `R-ADMIN-08-N2` |
| `R-AGENT-<page>-<n>` | An agent-self-service page requirement. `<page>` is `a01`–`a05`. | `R-AGENT-a01-R1` |
| `R-SYS-<flow>-<n>` | A system flow requirement. `<flow>` is `s01`–`s06`. | `R-SYS-s02-3` |
| `R-NFR-<area>-<n>` | A cross-cutting NFR. `<area>` is `performance`, `observability`, `security`, or `cost`. | `R-NFR-security-5` |

Requirement phrasing uses **`SHALL`** (binding) or **`SHOULD`** (recommended). `MAY` is avoided to reduce ambiguity.

## Coverage goal

Every concept rule in [concepts/agent.md](../concepts/agent.md), [concepts/organization.md](../concepts/organization.md), [concepts/project.md](../concepts/project.md), [concepts/permissions/](../concepts/permissions/README.md), [concepts/system-agents.md](../concepts/system-agents.md), [concepts/token-economy.md](../concepts/token-economy.md), and the decided sections of [concepts/coordination.md](../concepts/coordination.md) maps to **at least one requirement** in this folder. The [cross-cutting/traceability-matrix.md](cross-cutting/traceability-matrix.md) is the authoritative coverage index.

## Folder map

| Path | Purpose |
|------|---------|
| [admin/00-fresh-install-journey-overview.md](admin/00-fresh-install-journey-overview.md) | One-pager mapping the 9 phases to the 14 admin pages. |
| [admin/01-platform-bootstrap-claim.md](admin/01-platform-bootstrap-claim.md) | **Phase 1.** First Human Agent claims the bootstrap credential; System Bootstrap Template adoption completes. |
| [admin/02-platform-model-providers.md](admin/02-platform-model-providers.md) — [05-platform-defaults.md](admin/05-platform-defaults.md) | **Phase 2.** Platform-level resource setup: model providers, MCP servers, credentials vault, system defaults. |
| [admin/06-org-creation-wizard.md](admin/06-org-creation-wizard.md) | **Phase 3.** Multi-step wizard creating the first Organization. |
| [admin/07-organization-dashboard.md](admin/07-organization-dashboard.md) | **Phase 4.** Post-creation home; empty state with CTAs, steady-state once populated. |
| [admin/08-agent-roster-list.md](admin/08-agent-roster-list.md) — [09-agent-profile-editor.md](admin/09-agent-profile-editor.md) | **Phase 5.** Agent roster management. |
| [admin/10-project-creation-wizard.md](admin/10-project-creation-wizard.md) — [11-project-detail.md](admin/11-project-detail.md) | **Phase 6.** Project creation + detail with OKRs, tasks, sub-projects. |
| [admin/12-authority-template-adoption.md](admin/12-authority-template-adoption.md) | **Phase 7.** Review and approve authority-template adoption Auth Requests. |
| [admin/13-system-agents-config.md](admin/13-system-agents-config.md) | **Phase 8.** Confirm standard system agents; add org-specific ones. |
| [admin/14-first-session-launch.md](admin/14-first-session-launch.md) | **Phase 9.** "Hello world" validation — launch first session. |
| [agent-self-service/README.md](agent-self-service/README.md) + `a01`–`a05` | Agent-facing surfaces (inbox/outbox, Auth Requests, consent, work, profile+grants). |
| [system/README.md](system/README.md) + `s01`–`s06` | Reactive headless behaviours. |
| [cross-cutting/README.md](cross-cutting/README.md) + NFRs + traceability matrix | Quality attributes + coverage index. |
| [_template/admin-page-template.md](_template/admin-page-template.md) | The 10-section template every admin / agent-self-service page follows. |

## See also

- [concepts/README.md](../concepts/README.md) — the concept spec these requirements derive from.
- [organizations/README.md](../organizations/README.md) — 10 reference organization layouts used as acceptance scenarios.
- [projects/README.md](../projects/README.md) — 5 reference project layouts used as acceptance scenarios.
