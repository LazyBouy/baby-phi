<!-- Last verified: 2026-04-22 by Claude Code -->

# Operations — Agent Roster (page 08)

**Status: [EXISTS]** — landed at M4/P4.

## Endpoint

`GET /api/v0/orgs/:org_id/agents`

Query parameters (all optional):

| Name | Values | Effect |
|---|---|---|
| `role` | `executive`, `admin`, `member`, `intern`, `contract`, `system` | Filter to agents whose `role` matches exactly. Absent → no role filter. |
| `search` | free text | Case-insensitive substring over `display_name`. Whitespace is trimmed; empty-after-trim returns 400 `VALIDATION_FAILED`. |

Response (`200 OK`):

```json
{
  "org_id": "<uuid>",
  "agents": [
    {
      "id": "<uuid>",
      "kind": "human" | "llm",
      "display_name": "...",
      "owning_org": "<uuid>" | null,
      "role": "executive" | ... | "system" | null,
      "created_at": "2026-04-22T…Z"
    }
  ]
}
```

The payload is deliberately thin: no `AgentProfile`, no `ExecutionLimits`, no `blueprint`. Detail drill-down lives on page 09 (`GET /api/v0/agents/:id/profile`, M4/P5).

## Error codes

| HTTP | `code` | Meaning | Operator action |
|---|---|---|---|
| 400 | `VALIDATION_FAILED` | `search` was supplied but empty-after-trim, or `role` was supplied with an unknown value. | Remove the offending param or supply a valid value. |
| 401 | `UNAUTHENTICATED` | No valid session cookie. | Re-authenticate (bootstrap claim / login). |
| 404 | `ORG_NOT_FOUND` | The `org_id` path segment doesn't match any persisted org. | Verify via `GET /api/v0/orgs`; typical cause is a typo in the URL. |
| 500 | `INTERNAL_ERROR` | Unhandled repository error. | Inspect server logs; typical cause is SurrealDB storage-backend issues. |

## Playbook — empty list on a populated org

Symptoms: operator knows agents exist (dashboard shows `agents_summary.total > 0`) but the roster page returns `agents: []`.

1. **Check filter state.** Open the URL bar — if `?role=intern` is set, the filter is narrowing. Clear the filter chip ("all") and retry.
2. **Confirm org membership.** Query the repo directly:
   ```sql
   SELECT id, display_name, kind, role, owning_org FROM agent
     WHERE owning_org = organization:<org_id>;
   ```
   If this returns rows but the API returns empty, the roster query + repo-read are out of sync — file a bug.
3. **Check for a stale dashboard cache.** The dashboard summary is aggregated separately (M3/P5); if the dashboard says `total = N` but the roster is 0, the dashboard may be stale. Force refresh (dashboard polls every 30s per R-ADMIN-07-N1).

## Playbook — role filter not taking effect

Symptoms: operator clicks `intern` chip, URL updates, but the table shows agents of other roles.

1. **Verify the `role` query param arrived.** In server logs, the handler logs `role=Some(Intern)` on dispatch — if the server sees `None`, the web UI isn't forwarding the chip. Typical cause: browser caching an older page build; hard-reload.
2. **Verify agent rows have `role` set.** Agents created before M4 migration 0004 have `role = None` and **never match** a `Some(role)` filter. Fix: either assign a role on page 09 edit (M4/P5) or migrate the row via SurrealQL:
   ```sql
   UPDATE agent:<id> SET role = 'intern';
   ```

## Playbook — slow full-org listing (> 500 ms)

M4 volume expectation: tens of agents per org, sub-100ms per request. If a full-org listing exceeds 500ms:

1. **Row count.** `SELECT count() FROM agent WHERE owning_org = organization:<org_id>;` — if this exceeds ~1k, M4's linear scan is not the right fit.
2. **Upgrade path.** M7+ adds a cursor-based `list_agents_in_org_paginated` repo method. The wire-shape stays additive: existing fields unchanged, `next_cursor` added. Current-consumer code (CLI table + web page) stays operational during the upgrade.

## CLI equivalent

```bash
# full roster
phi agent list --org-id <uuid>

# filter to interns
phi agent list --org-id <uuid> --role intern

# search for "bot"
phi agent list --org-id <uuid> --search bot

# combine
phi agent list --org-id <uuid> --role contract --search alpha

# machine-readable
phi agent list --org-id <uuid> --json | jq '.agents | length'
```

Exit codes follow the standard phi set: `0` on success, `65` (`EXIT_REJECTED`) when the server returns 4xx, `68` (`EXIT_TRANSPORT`) on connection failure.

## phi-core leverage notes

This page surfaces **zero phi-core fields** — roster rows carry only phi governance data (`id`, `kind`, `display_name`, `owning_org`, `role`, `created_at`). `AgentProfile` (which wraps `phi_core::AgentProfile`) lives on a separate node and is surfaced by page 09's editor endpoints (M4/P5). Keeping `blueprint` off the list response preserves the "index view is a summary, detail view carries phi-core shape" split established by M3/P4 (`orgs/list` vs `orgs/show`).

## References

- [Agent roster architecture](../architecture/agent-roster.md)
- [Repository method `list_agents_in_org_by_role`](../../../../../../modules/crates/domain/src/repository.rs) — M4/P2 trait surface.
- [M4 plan archive §P4](../../../../plan/build/a634be65-m4-agents-and-projects.md)
