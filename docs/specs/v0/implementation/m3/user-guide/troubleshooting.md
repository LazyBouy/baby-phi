<!-- Last verified: 2026-04-22 by Claude Code -->

# User guide — Troubleshooting (M3)

**Status: [EXISTS]** — M3/P6 close.

Stable error codes + recovery flow for admin pages 06 (org creation)
and 07 (org dashboard). M1/M2 codes are inherited — see the cross-link
at the bottom for the carry-over table.

## M3 stable codes

| Code | HTTP | Surface | Cause | Recovery |
|---|---|---|---|---|
| `ORG_ID_IN_USE` | 409 | POST /orgs | Server-minted org_id collides with an existing org. Only observable on a repo-layer retry where the same uuid is forced; normal wizard flow does not hit this. | Re-submit the wizard; a fresh id is minted per request. |
| `TEMPLATE_NOT_ADOPTABLE` | 400 | POST /orgs | Wizard payload lists Template E, F, or SystemBootstrap in `authority_templates_enabled`. Only A, B, C, D are adoptable at creation time; Template E is platform-level (self-approve), F is reserved for M6 break-glass, SystemBootstrap is the platform-genesis template. | Remove the non-adoptable kind from the payload and re-submit. |
| `VALIDATION_FAILED` | 400 | POST /orgs | One of: empty `display_name`, empty `ceo_display_name`, empty `ceo_channel_handle`, `token_budget = 0`, duplicate entry in `authority_templates_enabled`. | Fix the offending field; message names it (`"display_name must not be empty"`, etc.). |
| `ORG_NOT_FOUND` | 404 | GET /orgs/:id, GET /orgs/:id/dashboard | The org id is unknown to the store. Either the org was never created or the id typo'd. | Verify via `phi org list` or `GET /orgs`. |
| `ORG_ACCESS_DENIED` | 403 | GET /orgs/:id/dashboard | Viewer has no `MEMBER_OF` to the org. Platform admin is intentionally NOT a member of orgs they create at M3 — the CEO (a separately-minted Human agent) is. | Log in as the CEO, or fetch via `GET /orgs/:id` (list/show endpoints have no membership check at M3). |
| `AUDIT_EMIT_FAILED` | 500 | POST /orgs (after 201 commit) | Compound tx committed successfully; batch audit emit (organization_created + N authority_template.adopted) then failed. The org is durable but its audit chain may be incomplete. | The compound tx is irreversible at this point. Log the failure + re-emit the missing audit entries manually from the ops side; M7b ships a chain-repair drill. |
| `REPOSITORY_ERROR` | 500 | Any M3 endpoint | Underlying SurrealDB read/write failed — connection pool exhausted, disk full, migration-mismatch, etc. | Check server logs, DB health. Dashboard reads specifically: verify `token_budget_pool` has a row for the org (ADR-0022 invariant — one pool per org). |

## Dashboard-specific signals

Dashboard-only symptoms, not stable-coded (render-side
degradations rather than HTTP errors):

| Symptom | Likely cause | Recovery |
|---|---|---|
| All counters stuck at zero (including `token_budget.total`) | Org exists but `token_budget_pool` row is missing — only possible if the org was created outside `apply_org_creation` (manual SurrealDB insert) | `phi org create` via the wizard, or `INSERT INTO token_budget_pool ...` manually matching the schema |
| `agents_summary.human = 0` despite visible org header | CEO Agent's `owning_org` doesn't match the org id | Verify `Agent.owning_org` column against `Organization.id` |
| `templates_adopted = []` but the wizard POST listed templates | The adoption AR's resource URI doesn't match the `org:<id>/template:<kind>` prefix | Verify in `auth_request.resource_slots[*].resource.uri` |
| Dashboard polling stalls (stale `last_fetched` timestamp) | Client-side `setInterval` halted (e.g. browser throttling an inactive tab) | Reload the page — the client component re-arms the interval on mount |

## CLI exit-code reference

Inherits from M2. M3-relevant subcommands use the same codes:

| Exit | Code name | When |
|---|---|---|
| 0 | `EXIT_OK` | Happy path |
| 2 | `EXIT_PRECONDITION_FAILED` | Missing required flag (`--id`, `--name`, etc.) or no saved session |
| 3 | `EXIT_REJECTED` | Server returned a 4xx with a stable code (`ORG_ID_IN_USE`, etc.) |
| 4 | `EXIT_TRANSPORT` | Network failure (DNS / connection / timeout) |
| 5 | `EXIT_INTERNAL` | Response body wouldn't decode |

## Cross-references

- [M2 troubleshooting](../../m2/user-guide/troubleshooting.md) — inherited codes (`UNAUTHENTICATED`, `AUDIT_EMIT_FAILED`, `PLATFORM_DEFAULTS_STALE_WRITE`, etc.).
- [Top-level runbook — M3 section](../../../../../../docs/ops/runbook.md#m3--organization-creation--dashboard-aggregated-runbook-index).
- [org-creation operations doc](../operations/org-creation-operations.md).
- [org-dashboard operations doc](../operations/org-dashboard-operations.md).
- [M3 plan §P6](../../../../plan/build/563945fe-m3-organization-creation.md).
