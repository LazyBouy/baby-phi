<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Agent Roster List (page 08)

**Status: [EXISTS]** — landed at M4/P4.

Read-only list of agents in an org with optional role-chip + free-text search. First consumer of the 6-variant `AgentRole` enum (`Executive` / `Admin` / `Member` / `Intern` / `Contract` / `System`).

## Surfaces

| Tier | Path | Entry point |
|---|---|---|
| HTTP | `GET /api/v0/orgs/:org_id/agents?role=…&search=…` | `server/src/handlers/agents.rs::list` |
| CLI | `phi agent list --org-id <uuid> [--role <r>] [--search <s>] [--json]` | `cli/src/commands/agent.rs::list_impl` |
| Web | `(admin)/organizations/[id]/agents/page.tsx` | SSR; filters via `<form method="get">` |

Business logic: `server/src/platform/agents/list.rs::list_agents`. The orchestrator:

1. Pre-checks the `org_id` exists (short-circuit 404 via `AgentError::OrgNotFound` → `ApiError` 404).
2. Validates `search` (trim + reject empty-after-trim → 400 `VALIDATION_FAILED`).
3. Delegates to `Repository::list_agents_in_org_by_role(org, role)` (M4/P2 surface).
4. Applies a case-insensitive substring filter on `display_name` when `search` is set.
5. Returns `Vec<AgentRosterRow>`.

## Wire shape

The roster response carries **phi governance fields only**:

```json
{
  "org_id": "<uuid>",
  "agents": [
    {
      "id": "<uuid>",
      "kind": "human" | "llm",
      "display_name": "...",
      "owning_org": "<uuid>" | null,
      "role": "executive" | "admin" | "member" | "intern" | "contract" | "system" | null,
      "created_at": "<rfc3339>"
    }
  ]
}
```

No `AgentProfile`, no `ExecutionLimits`, no `blueprint`. This split is intentional — follows M3/P4's convention that **index views are thin summaries, detail views carry phi-core shape** (`orgs/list.rs` omits `defaults_snapshot` while `orgs/show.rs` includes it).

## phi-core leverage (Q1 / Q2 / Q3)

- **Q1 direct imports: 0.** `list.rs` imports only `domain::*` types.
- **Q2 transitive: 0.** The wire response has no phi-core-wrapping field at any depth. Contrast with page 09 (M4/P5) which surfaces `AgentProfile.blueprint` directly.
- **Q3 rejections:** phi-core has no roster / list / filter concept. Nothing to reuse. Page 08 is pure phi governance.

Positive close-audit greps:

```bash
grep -En '^use phi_core::' modules/crates/server/src/handlers/agents.rs        # → 0 lines
grep -En '^use phi_core::' modules/crates/server/src/platform/agents/          # → 0 lines
```

The CLI `agent list` impl likewise has 0 phi-core imports; the only phi-core references in `cli/src/commands/agent.rs` stay in the pre-M1 `demo` subcommand.

## Filter semantics

Role filter:

- `role=None` → every agent returned (including `role == None` pre-M4 rows).
- `role=Some(X)` → only agents whose `role == Some(X)`. Pre-M4 rows (`role == None`) **do not** match any `Some(role)` filter. Operator UX: the dashboard's "Unclassified" bucket surfaces those rows; pattern matches M4/P8 dashboard rewrite commitment.

Search filter:

- Case-insensitive substring over `display_name`.
- Trimmed before matching.
- Empty-after-trim is rejected with `VALIDATION_FAILED` — prevents "I hit the search button with no input, why did everything come back?" confusion.

Combined filters intersect (AND), not union.

## Pagination

Not at M4. Orgs hold tens of agents on page 08 (M4 scope); full list loads fit in a single HTTP response comfortably. Upgrade path: add `next_cursor` + `limit` query params in a later milestone. The existing response shape stays compatible — new fields are additive.

## References

- [Repository method `list_agents_in_org_by_role`](../../../../../../modules/crates/domain/src/repository.rs) — M4/P2 trait surface.
- [Agent roster operations runbook](../operations/agent-roster-operations.md)
- [phi-core-reuse-map.md §Page 08](phi-core-reuse-map.md)
- [Requirements admin/08](../../../requirements/admin/08-agent-roster-list.md)
- [M4 plan archive §P4](../../../../plan/build/a634be65-m4-agents-and-projects.md)
