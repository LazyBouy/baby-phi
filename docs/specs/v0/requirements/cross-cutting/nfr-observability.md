<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# NFR — Observability

> Audit events, metrics, and logs the system MUST emit. The observability surface is load-bearing for the permission model — without it, the authority chain cannot be reconstructed and revocation cascades cannot be validated.

## Audit events

- **R-NFR-observability-1:** Every state change on a `Grant`, `AuthRequest`, `Consent`, `Agent`, `Project`, `Organization`, or `Resource` node SHALL emit an audit event. The event SHALL record: `event_type`, `actor_agent_id`, `target_entity_id`, `timestamp`, `diff` (old/new field values where applicable), `audit_class` (`silent` / `logged` / `alerted`), and `provenance_auth_request_id` if applicable.
- **R-NFR-observability-2:** `alerted` audit events SHALL be delivered to the org's designated alert channel within 60s of occurrence. The alerting mechanism is the audit log feed, not email/SMS — integrations with external alert systems are downstream concerns.
- **R-NFR-observability-3:** `logged` events SHALL be retained in active audit storage for at least 365 days (regulated orgs configure longer via [admin/05-platform-defaults.md](../admin/05-platform-defaults.md)).
- **R-NFR-observability-4:** `silent` events SHALL be retained at shorter durations (default 30 days) and are intended for routine, high-volume telemetry where individual records are rarely consulted.
- **R-NFR-observability-5:** Every audit event SHALL have a unique `event_id` that other events can reference. Revocation cascade events SHALL reference the triggering event's `event_id`.

## Authority-chain traversal

- **R-NFR-observability-6:** Given any `Grant` id, the system SHALL be able to produce the full authority chain (`Grant → Auth Request → Approver → ... → System Bootstrap Template`) per [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain) in a single query completing within 500ms p95.
- **R-NFR-observability-7:** Given any audit `event_id`, the system SHALL be able to produce the forward-looking **cascade fan-out** — events causally triggered by this one (e.g., revoking an Auth Request → cascaded grant revocations → cascaded downstream-session access denials) within 1s p95.

## Metrics

- **R-NFR-observability-8:** The system SHALL expose metrics in a Prometheus-compatible format (or equivalent scrapeable interface). At minimum, metrics listed in each system flow file's Observability section SHALL be present.
- **R-NFR-observability-9:** Every API endpoint SHALL export per-endpoint latency histograms (p50 / p95 / p99) and request-count counters keyed by status code.
- **R-NFR-observability-10:** Domain metrics SHALL include at minimum:
  - `phi_permission_check_duration_seconds{result=allowed|denied|pending, failed_step=N?}`
  - `phi_grants_active{org_id}`
  - `phi_auth_requests_pending{org_id}`
  - `phi_kernel_sessions_active{org_id}`
  - `phi_token_budget_usage_ratio{org_id}`
  - `phi_memory_extraction_queue_depth{org_id}`
  - `phi_agent_catalog_heartbeat_lag_seconds{org_id}`.

## Logs

- **R-NFR-observability-11:** Application logs SHALL be structured (JSON) with required fields: `timestamp`, `severity`, `trace_id`, `agent_id?`, `org_id?`, `event_id?`, `message`.
- **R-NFR-observability-12:** Every Permission Check SHALL produce a trace log line on denial. On allow, a trace line is produced at `debug` level (filterable).
- **R-NFR-observability-13:** Secret material SHALL NEVER appear in logs. The vault API ([admin/04-platform-credentials-vault.md](../admin/04-platform-credentials-vault.md)) masks secrets at edge; any accidental inclusion is a P0 security bug.

## Storage tiering (observability data)

- **R-NFR-observability-14:** Active audit events SHALL be queryable from the hot path (see Auth Request retention's active window). Archived events remain retrievable with an admin-level query with explicit approval per [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy).
- **R-NFR-observability-15:** Metric histograms SHALL be retained at full resolution for 7 days, then down-sampled.

## Cross-references

- [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy).
- [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain).
- [system/ flows](../system/README.md) — each flow file lists its specific observability metrics.
