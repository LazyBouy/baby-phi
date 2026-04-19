<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s06 — Periodic Triggers

## Purpose

Time-based, non-event-driven system behaviours. Groups several independent flows under one file for locality: Auth Request retention archival, credential rotation reminders, monitoring heartbeats, and token-budget accounting.

## Triggers

All are time-based, driven by internal scheduler:

| Sub-flow | Cadence (default) | Configurable per org? |
|----------|-------------------|-----------------------|
| Auth Request retention archival | hourly | no (uses org retention policy) |
| Credential rotation reminders | daily at 01:00 UTC | no |
| Agent-catalog-agent heartbeat | every 60s | no |
| Token-budget snapshot | hourly | yes — per-org threshold |
| Auth Request timeout scan (delegated to [s04](s04-auth-request-state-transitions.md)) | every 60s | no |

## Preconditions

- Respective subsystems (Auth Request machinery, vault, agent catalog, token pool, etc.) are running.

## Behaviour — per sub-flow

### Auth Request retention archival

- **R-SYS-s06-1:** Every hour, the flow SHALL scan Auth Requests in terminal states (Approved / Denied / Partial / Expired / Revoked / Cancelled) and move them from **active** to **archived** when the active window has elapsed (default 90 days; org-configurable in [admin/05-platform-defaults.md](../admin/05-platform-defaults.md) and [admin/06-org-creation-wizard.md](../admin/06-org-creation-wizard.md)).
- **R-SYS-s06-2:** Archival does NOT delete. Per [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy), archived records remain queryable via `inspect_archived` (gated by `human_required` approval by default).
- **R-SYS-s06-3:** If the owning org has `delete_after_years` set (compliance scenarios — see [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md)), records older than that limit SHALL be deleted after a final `AuthRequestDeletedPerRetentionPolicy` alerted audit event.

### Credential rotation reminders

- **R-SYS-s06-4:** Every 24h, the flow SHALL check each `secret/credential` entry in every platform and org catalogue. If `last_rotated_at` plus the org's rotation threshold (default 90 days) is approaching (30 / 7 / 1 day before), the custodian's inbox SHALL receive an `AgentMessage` with the upcoming-rotation reminder.
- **R-SYS-s06-5:** On threshold exceeded, the custodian's inbox receives a high-priority message; an alerted audit event `SecretRotationOverdue { secret_id, overdue_by }` is emitted.

### Agent-catalog-agent heartbeat

- **R-SYS-s06-6:** Every 60s, the flow SHALL record the agent-catalog-agent's queue depth and last-updated timestamp to a `control_plane_object:catalog-heartbeat` resource. This powers the [admin/08-agent-roster-list.md R7](../admin/08-agent-roster-list.md#5-read-requirements) and [admin/07-organization-dashboard.md](../admin/07-organization-dashboard.md) status indicators.
- **R-SYS-s06-7:** If no heartbeat is recorded for 5 consecutive minutes, the flow SHALL emit `AgentCatalogAgentUnresponsive` alerted audit event and the admin-dashboard status goes to `error`.

### Token-budget snapshot

- **R-SYS-s06-8:** Every hour, the flow SHALL snapshot each org's `token-budget-pool` consumption vs initial_allocation and write it to a time-series for dashboard display.
- **R-SYS-s06-9:** When an org's consumption crosses 50% / 75% / 90% / 100% of allocation, the flow SHALL emit `TokenBudgetThresholdCrossed` audit events (50/75 logged, 90 alerted, 100 alerted with hard-block semantics).
- **R-SYS-s06-10:** At 100%, new session launches from this org's agents SHALL be rejected by the Permission Check with reason `TOKEN_BUDGET_EXHAUSTED`. The admin must top up the pool before work resumes.

### Auth Request timeout scan

Delegated to [s04-auth-request-state-transitions.md R-SYS-s04-4](s04-auth-request-state-transitions.md). Listed here only for completeness since the periodic scheduler runs it.

## Side effects

- Archival writes on Auth Request storage tier metadata.
- Inbox messages delivered to credential custodians.
- Heartbeat records updated.
- Token budget time-series written.
- Audit events as listed in each sub-flow.

## Failure modes

- **Scheduler downtime** → sub-flows accumulate lag. On resumption, backlog is processed in FIFO order; heartbeat gap triggers `AgentCatalogAgentUnresponsive` if not closed within 5 min; overdue rotation reminders fire immediately for all stale secrets.
- **Retention archival failure** → retried with backoff; after 3 failures, `RetentionArchivalFailed` alerted event per record.
- **Token-budget threshold dispatch failure** (can't deliver the message) → retry, then dead-letter; the threshold-crossed state is still recorded so reconciliation at next snapshot catches it.

## Observability

- Metrics: `baby_phi_retention_archive_total`, `baby_phi_secret_rotation_reminders_sent`, `baby_phi_agent_catalog_heartbeat_lag_seconds`, `baby_phi_token_budget_usage_ratio`, `baby_phi_auth_request_timeout_expirations_total`.
- Audit events: `AuthRequestArchived`, `AuthRequestDeletedPerRetentionPolicy`, `SecretRotationOverdue`, `AgentCatalogAgentUnresponsive`, `TokenBudgetThresholdCrossed`.

## Cross-References

**Concept files:**
- [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy) — archival + `delete_after_years`.
- [concepts/token-economy.md](../../concepts/token-economy.md) — budget and Worth/Value.

**Admin pages referencing this flow:**
- [admin/04-platform-credentials-vault.md](../admin/04-platform-credentials-vault.md) — source of rotation reminders.
- [admin/05-platform-defaults.md](../admin/05-platform-defaults.md) — retention defaults.
- [admin/07-organization-dashboard.md](../admin/07-organization-dashboard.md) — shows token budget + heartbeat status.
- [admin/08-agent-roster-list.md](../admin/08-agent-roster-list.md) — catalog-agent status indicator.

**Related flows:**
- [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md) — the timeout scan sub-flow reuses s04's transition logic.
