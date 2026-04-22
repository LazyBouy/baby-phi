<!-- Last verified: 2026-04-22 by Claude Code -->

# Operations — Org Creation (page 06)

**Status: [EXISTS]** — landed at M3/P4.

## Error codes

| HTTP | `code` | Meaning | Operator action |
|---|---|---|---|
| 400 | `VALIDATION_FAILED` | Input failed shape validation (empty `display_name`, empty `ceo_display_name`, empty `ceo_channel_handle`, `token_budget == 0`, duplicate template kinds). | Check the wizard form; server validation mirrors the web UI's client checks. |
| 400 | `TEMPLATE_NOT_ADOPTABLE` | Caller requested Template `E`, `F`, or `SystemBootstrap` at creation. E is platform-level; F is reserved for M6; SystemBootstrap is the genesis template. | Remove the offending kind from `authority_templates_enabled`. |
| 401 | `UNAUTHENTICATED` | No valid session cookie. | Re-authenticate (bootstrap claim / login). |
| 404 | `ORG_NOT_FOUND` | `GET /api/v0/orgs/:id` for a non-existent id. | Verify the id; check `GET /api/v0/orgs` for the full list. |
| 409 | `ORG_ID_IN_USE` | Attempted to persist an org with an id that already exists — typically a client retry after a successful response. | The original create succeeded; fetch via `GET /api/v0/orgs/:id` instead. |
| 500 | `AUDIT_EMIT_FAILED` | The compound tx succeeded but one of the follow-up audit-event writes failed. | The org IS persisted (durable). Check server logs for the emit error; the hash chain is consistent up to the last successfully-emitted event. Chain-repair runbook lands with M7b durable-audit work. |
| 500 | `INTERNAL_ERROR` | Unhandled repository error. | Inspect server logs; typical cause is SurrealDB storage backend issues. |

## Failure-rollback playbook

The `apply_org_creation` repo method uses a single
`BEGIN TRANSACTION` / `COMMIT TRANSACTION` envelope. Any statement
error rolls back the whole batch — **no partial state survives**.
Verification:

```sql
-- After a failed create, every new-org table must remain at pre-
-- attempt row counts. Quick sanity query:
SELECT count() FROM organization;
SELECT count() FROM agent_profile;
SELECT count() FROM token_budget_pool;
```

Integration tests that pin this invariant:
- `modules/crates/store/tests/apply_org_creation_tx_test.rs::duplicate_org_id_is_conflict_with_no_partial_state`
- `modules/crates/server/tests/acceptance_orgs_create.rs::create_respects_adr_0023_no_per_agent_policy_nodes`

## Template-adoption audit trail

Each successful create emits `1 + N` audit events per org, in this
order:

1. `platform.organization.created` (Alerted, `org_scope =
   Some(new_org_id)`) — diff carries full org identity + CEO + system
   agent ids.
2. `authority_template.adopted` (Alerted, same `org_scope`) — one
   per enabled template. Diff carries `template_kind` + the
   adoption AR's id.

All events chain on the new org's per-org hash chain. Query them via
`Repository::list_recent_audit_events_for_org(org_id, limit)` or the
M3/P5 dashboard's `RecentAuditEvents` panel.

## CEO-invite inbox row

Per plan §D9, M3 writes the CEO's `InboxObject` row at creation time
but does NOT deliver a real invite message — channel delivery
(Slack/email webhook) lands with M7b's notification infrastructure.
To verify the row landed:

```sql
SELECT * FROM inbox_object WHERE agent_id = <ceo_agent_id>;
```

When M7b wires delivery, the hook reads the inbox row + the CEO's
`Channel.handle` and dispatches; the M3 row shape is forward-
compatible (no schema migration required).

## Reference layouts

Three fixtures ship at `modules/crates/cli/fixtures/reference_layouts/`:

- `minimal-startup.yaml` — 1 adopted template, email CEO, 500k token
  budget.
- `mid-product-team.yaml` — 3 adopted templates (A/B/C), Slack CEO,
  5M token budget.
- `regulated-enterprise.yaml` — 4 adopted templates (A/B/C/D),
  per-session consent, alerted audit default, 50M token budget.

Consumed via `phi org create --from-layout <path>`. The CLI
deserialises YAML → JSON and POSTs verbatim. Full 10-layout parity
lands with M8 per plan §G18.

## phi-core leverage notes

`Organization.defaults_snapshot` carries 4 phi-core-wrapped fields.
Operators inspecting the audit diff or
`GET /api/v0/orgs/:id` response will see phi-core field shapes
(`execution_limits.max_turns`, `retry_config.max_retries`, etc.)
verbatim. The M3 dashboard's drill-down renders these under a
collapsed `<details>` panel. Future phi-core schema evolution does
not require a phi migration because the SurrealDB
`defaults_snapshot` column is `FLEXIBLE TYPE object`.

## References

- [Org creation architecture](../architecture/org-creation.md)
- [ADR-0022 — compound transaction](../decisions/0022-org-creation-compound-transaction.md)
- [ADR-0023 — inherit-from-snapshot](../decisions/0023-system-agents-inherit-from-org-snapshot.md)
- [Per-org audit chain](../architecture/per-org-audit-chain.md)
