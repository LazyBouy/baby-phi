<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# System — Reactive & Headless Behaviours

> Headless behaviours the system performs without direct UI interaction. They are **provisioned** by admin pages (an admin page sets up which agents, which triggers, which retention windows); the files here capture the **runtime contract** of each resulting behaviour — trigger conditions, preconditions, side effects, observability, and failure modes.
>
> Each flow file follows a focused template: Purpose → Trigger → Preconditions → Behaviour → Side Effects → Failure Modes → Observability → Cross-References. Sections are tighter than the admin-page template because system flows have no UI surface and their own reduced set of concerns.

## Flows

| File | Trigger | Provisioned by |
|------|---------|-----------------|
| [s01-bootstrap-template-adoption.md](s01-bootstrap-template-adoption.md) | System init / first platform admin claim | [admin/01-platform-bootstrap-claim.md](../admin/01-platform-bootstrap-claim.md) |
| [s02-session-end-memory-extraction.md](s02-session-end-memory-extraction.md) | `AgentEvent::SessionEnd` | [admin/13-system-agents-config.md](../admin/13-system-agents-config.md) |
| [s03-edge-change-catalog-update.md](s03-edge-change-catalog-update.md) | graph edge add/remove on MEMBER_OF / HAS_AGENT / HAS_LEAD / HAS_PROFILE | [admin/13-system-agents-config.md](../admin/13-system-agents-config.md) |
| [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md) | slot-fill events / timeouts / owner actions | [admin/12-authority-template-adoption.md](../admin/12-authority-template-adoption.md) and most other admin pages |
| [s05-template-adoption-grant-fires.md](s05-template-adoption-grant-fires.md) | matching graph events per adopted Authority Template | [admin/12-authority-template-adoption.md](../admin/12-authority-template-adoption.md) |
| [s06-periodic-triggers.md](s06-periodic-triggers.md) | cron / periodic schedule | multiple — see per-flow detail |

## Requirement ID convention

All requirements in `system/` carry `R-SYS-<flow>-<n>`:

- `<flow>` is one of `s01`..`s06`.
- `<n>` is a sequence inside that flow (no R/W/N prefix, since system flows have no UI reads/writes/notifications — everything is behaviour).

## Cross-reference policy

Every system flow file MUST name the admin page(s) that provision it. Every admin page with a W-section that triggers a system flow MUST link back to the relevant system file. This keeps the admin → system coupling traceable in both directions.
