<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Org Dashboard (page 07)

**Status: [EXISTS]** — shipped at M3/P5.

## End-to-end flow

```
 Client (Web / CLI)   Server                          SurrealDB
 ──────────────────   ──────                          ─────────
       │                    │                              │
       │  GET /dashboard →  │                              │
       │                    │  dashboard_summary(org, viewer, now)
       │                    │    get_organization ────────▶│
       │                    │    list_agents_in_org ──────▶│
       │                    │    list_projects_in_org ────▶│
       │                    │    list_active_auth_requests_for_org ─▶│
       │                    │    list_recent_audit_events_for_org(5) ─▶│
       │                    │    count_alerted_events_for_org_since ─▶│ (24h)
       │                    │    get_token_budget_pool_for_org ──────▶│
       │                    │    list_adoption_auth_requests_for_org ─▶│
       │                    │                              │
       │                    │  project to DashboardSummary │
       │                    │    (phi-core-stripped)       │
       │   200 JSON       ◀─│                              │
       │                    │                              │
  30 s timer → GET /dashboard (repeat)                     │
```

## Code map

- **Business logic**: [`modules/crates/server/src/platform/orgs/dashboard.rs`](../../../../../../modules/crates/server/src/platform/orgs/dashboard.rs)
  - `dashboard_summary(repo, org_id, viewer_agent_id, now) -> Result<DashboardOutcome, OrgError>`
  - `DashboardOutcome::{Found(Box<DashboardSummary>), NotFound, AccessDenied}`
  - Wire shapes: `DashboardSummary`, `OrganizationDashboardHeader`,
    `ViewerContext`, `AgentsSummary`, `ProjectsSummary`,
    `TokenBudgetView`, `RecentEventSummary`, `EmptyStateCtaCards`.
- **HTTP handler**: [`modules/crates/server/src/handlers/orgs.rs::dashboard`](../../../../../../modules/crates/server/src/handlers/orgs.rs) — maps the three outcomes to 200/404/403.
- **Router**: [`modules/crates/server/src/router.rs`](../../../../../../modules/crates/server/src/router.rs) — `GET /api/v0/orgs/:id/dashboard`.
- **Repository surface** (M3/P5 additions):
  [`Repository::get_token_budget_pool_for_org`](../../../../../../modules/crates/domain/src/repository.rs) +
  [`Repository::count_alerted_events_for_org_since`](../../../../../../modules/crates/domain/src/repository.rs).
- **CLI**: [`modules/crates/cli/src/commands/org.rs::dashboard_impl`](../../../../../../modules/crates/cli/src/commands/org.rs).
- **Web**: `modules/web/app/(admin)/organizations/[id]/dashboard/` —
  `page.tsx` (SSR entry) + `DashboardClient.tsx` (30 s polling +
  8 panels). Next.js route-group parentheses confuse relative
  Markdown links, so navigate manually.

## phi-core leverage (Q1/Q2/Q3)

Per the [leverage-checklist](phi-core-leverage-checklist.md), the
dashboard surface is **deliberately phi-core-light**:

- **Q1 direct**: 0 new `use phi_core::…` imports across
  `server/src/platform/orgs/dashboard.rs`,
  `server/src/handlers/orgs.rs` (dashboard handler),
  `cli/src/commands/org.rs` (dashboard subcommand), and the web
  dashboard page.
- **Q2 transitive**: the `DashboardSummary` wire shape **strips**
  `Organization.defaults_snapshot` (which wraps 4 phi-core types) and
  uses a compact `OrganizationDashboardHeader` instead. Rationale:
  the dashboard is a polling surface (30 s cadence per D4); coupling
  its JSON shape to phi-core schema evolution would force a
  polling-contract rev on every phi-core release. Drill-down to the
  full snapshot is available via `GET /orgs/:id` (the `show.rs`
  endpoint), which does carry it verbatim, per D11/Q6.
- **Q3 rejections** (all phi-native):
  `phi_core::Session`/`LoopRecord`/`Turn` — deferred to M5+ (D11:
  dashboard rows at M3 link to audit log, not session traces).
  `phi_core::Usage` — token budget is a governance-level economic
  resource, not per-loop Usage. `phi_core::AgentEvent` — orthogonal
  per CLAUDE.md (governance audit log ≠ agent-loop telemetry).
  `phi_core::context::*` + `phi_core::provider::*` — live only on
  `defaults_snapshot`, which the dashboard strips.

**Positive close-audit assertions (shipped, verified in CI):**

1. `scripts/check-phi-core-reuse.sh` reports 0 hits on the dashboard
   surfaces.
2. [`server/src/platform/orgs/dashboard.rs::tests::dashboard_summary_wire_shape_excludes_phi_core_fields`](../../../../../../modules/crates/server/src/platform/orgs/dashboard.rs)
   asserts the serialised `DashboardSummary` JSON has no
   `defaults_snapshot` / `execution_limits` / `context_config` /
   `retry_config` / `default_agent_profile` / `blueprint` keys.
3. [`server/tests/acceptance_orgs_dashboard.rs::dashboard_wire_json_excludes_phi_core_fields`](../../../../../../modules/crates/server/tests/acceptance_orgs_dashboard.rs)
   walks the real wire JSON recursively and fails loudly if any
   phi-core-wrapping key re-appears.
4. [`modules/web/__tests__/orgs-dashboard.test.ts`](../../../../../../modules/web/__tests__/orgs-dashboard.test.ts)
   pins the invariant on the web tier.

## Viewer-role resolution

M3 implements two of the three roles R-ADMIN-07-§7 describes:

- `Admin` — the first-created Human agent in the org (the CEO at
  org-creation time). Sees every panel plus the 4 CTA cards on a
  fresh org.
- `Member` — any other Agent with `owning_org == Some(org_id)`.
  Read-only summary. No CTA cards, no pending-approvals tile, no
  alerted-events tile.
- `ProjectLead` — **carryover to M4** (requires `HAS_LEAD` edge
  wiring + persisted Project struct; the edge exists since M3/P1,
  the Project surface arrives at M4). The enum variant exists so the
  dashboard contract is stable across the M3→M4 boundary.

Non-members receive **403 `ORG_ACCESS_DENIED`** at the handler layer
before any aggregate read runs.

## Polling cadence

30-second client-side `setInterval` wrapped around a Server Action
call to the same `GET /dashboard` handler (plan D4). See the M7b
upgrade path in the operations doc for WebSocket push.

## CTA-card visibility (R-ADMIN-07-R8)

Cards surface only when the viewer is `Admin` **and** the org is
fresh — operationally: `projects_summary.active == 0` or
`agents_summary.total <= 3` (CEO + 2 system agents). Populated orgs
hide the cards. Project leads will see a filtered subset at M4.

## Deviations from requirements doc §10

| Requirement field | Shipped shape | Reason |
|---|---|---|
| `agents_summary.by_kind: { Human, Intern, Contract, System }` | `{ total, human, llm }` | Domain has only 2 `AgentKind` variants. Intern/Contract are role concepts that need an `AgentRole` field — carryover to M4+ when the roster page (08) needs the distinction. |
| `projects_summary.shape_a` / `shape_b` | `0` at M3 | `Project` struct + shape discriminator land at M4. |
| `viewer.role: ProjectLead` filtered view | `Member` fallback at M3 | `HAS_LEAD` wiring + project surface are M4. |
| Welcome banner | Server-computed string | Same copy across CLI + Web; no client-side l10n at M3. |

## References

- [M3 plan §P5](../../../../plan/build/563945fe-m3-organization-creation.md)
- [ADR-0022 — compound transaction](../decisions/0022-org-creation-compound-transaction.md)
- [ADR-0023 — inherit-from-snapshot](../decisions/0023-system-agents-inherit-from-org-snapshot.md)
- [phi-core leverage checklist](phi-core-leverage-checklist.md)
- [Requirements: admin page 07](../../../requirements/admin/07-organization-dashboard.md)
