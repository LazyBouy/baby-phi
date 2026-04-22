<!-- Last verified: 2026-04-22 by Claude Code -->

# Operations — Org Dashboard

**Status: [EXISTS]** — shipped at M3/P5.

Ops runbook for admin page 07 (organisation dashboard).

## Endpoint + cadence

- **HTTP**: `GET /api/v0/orgs/:id/dashboard`.
- **Poll**: 30 s client-side `setInterval` wrapped around a Next.js
  Server Action that re-issues the GET. No server-push at M3.
- **Cache headers**: `no-store` (both Server Action path and the
  handler). The polling loop is authoritative; intermediate caches
  would mask staleness bugs we care about.

## Response-code reference

| Status | Code | Trigger | Operator action |
|---|---|---|---|
| 200 | — | Happy path | — |
| 404 | `ORG_NOT_FOUND` | Org id unknown | Verify id via `phi org list` |
| 403 | `ORG_ACCESS_DENIED` | Viewer has no `MEMBER_OF` to the org | Check `list_agents_in_org` membership; the admin is **not** a member of any org they didn't nominate themselves to |
| 500 | `REPOSITORY_ERROR` | Repo read failed mid-aggregate | Check server logs + SurrealDB health; the token-budget-pool read is the only read that propagates a guaranteed-exists assumption (the compound tx creates one per org) |
| 500 | `AUDIT_EMIT_FAILED` | Not applicable — dashboard is read-only, never emits audit events | — |

## Data-freshness SLOs

| Metric | Target | Alert |
|---|---|---|
| Dashboard handler `p95` latency | `< 500 ms` | `> 1 s` for 3 consecutive samples |
| Polling cadence (client) | 30 s ± 2 s | N/A (client-timer; no server SLO) |
| Steady-state staleness | `< 30 s` | N/A (derived from cadence) |
| Counter drift (audit-event count) | `== 0` vs backing store | Never. Any drift is a bug in `count_alerted_events_for_org_since` |

## Incident playbooks

### "Dashboard shows stale data"

1. Client-side: open DevTools network tab, confirm 30 s polling
   fires. If the `setInterval` halted, reload the page (the client
   component re-arms on mount).
2. Server-side: tail `phi-server` logs for `orgs::dashboard`
   handler errors. The handler logs every non-200 response via
   `error!`.
3. If the handler is 200 but counters look wrong, verify against the
   CLI: `phi org dashboard --id <uuid> --json` calls the same
   handler — mismatched counts between CLI and web would indicate a
   Next.js caching misconfiguration (not expected given `no-store`).
4. Worst case, bounce the server — the SurrealDB-side aggregates
   (`count_alerted_events_for_org_since`) are pushdowns; no cache
   layer to flush.

### "Polling returns 403 unexpectedly"

The admin cookie is for the platform admin, **not** the org's CEO.
Dashboard access requires `MEMBER_OF`. Operators expecting to see
every org's dashboard should surface that filter — route them to
`GET /api/v0/orgs` (list endpoint) which has no membership check.

### "Token budget tile shows 0 / 0"

`get_token_budget_pool_for_org` returned `None`. The compound
`apply_org_creation` transaction always creates exactly one pool per
org (ADR-0022 invariant); a `None` here means:

- The org predates the compound-tx migration (pre-M3) — not
  expected in v0, but check `SELECT FROM token_budget_pool WHERE
  owning_org = $org` directly.
- The org was created outside the wizard flow (manual SurrealDB
  insert) — operator mistake; rerun with `phi org create`.

The handler returns 500 `REPOSITORY_ERROR` with a human-readable
message rather than rendering the 0/0 tile; log the message and file
a ticket.

## M7b upgrade path — WebSocket push

R-ADMIN-07-N1 allows "30s OR WebSocket push". Migration criteria:

- Dashboard shows 1000+ orgs and polling produces load spikes at the
  30 s boundary.
- Operator feedback requests sub-second freshness for
  pending-approvals tile.

When the threshold is crossed, the upgrade:
1. Add a pub/sub channel per org (the per-org audit-chain already
   partitions write events; leverage that boundary).
2. Swap `setInterval` for a WebSocket subscription in the client
   component — the wire shape stays identical (`DashboardSummary`).
3. Keep the 30 s fallback for clients that can't open sockets (e.g.
   behind strict egress policies).

The handler contract is **stable across this migration** — the
dashboard-shape test in `acceptance_orgs_dashboard.rs` pins it.

## Cross-links

- [Architecture — Org Dashboard](../architecture/org-dashboard.md)
- [M3 plan §P5](../../../../plan/build/563945fe-m3-organization-creation.md)
- [Requirements: admin page 07](../../../requirements/admin/07-organization-dashboard.md)
- [phi-core leverage checklist](../architecture/phi-core-leverage-checklist.md)
