<!-- Last verified: 2026-04-20 by Claude Code -->

# Audit log — retention

M1 lays the audit-event foundation: the three-tier class model, the
per-org hash chain, and the shadow NDJSON append. The retention
lifecycle — how long each class stays queryable, how events age out,
where the off-site copy lives — is the operator-facing concern this
page covers.

## Class tiers

Every audit event is tagged with an `AuditClass`
([`domain/src/audit.rs`](../../../../../../modules/crates/domain/src/audit.rs)):

| Class | Retention | Delivery | Example |
|---|---|---|---|
| `Silent` | 30 days | None | State transitions that change nothing important (an admin views a page) |
| `Logged` | 365 days | Structured log sink | Most write operations |
| `Alerted` | 365+ days | Alert channel within 60 s | Security-sensitive events (platform-admin claimed, grant revoked, master-key touched) |

The tier is set at emit time — the handler that writes the event
decides. In M1 the only classes in active use are `Alerted` (bootstrap
claim) and `Logged` (auth-request transitions, grant issuance).
`Silent` is defined but unused until M2+ page-view tracking lands.

## What M1 persists

Every emitted event lands in the SurrealDB `audit_events` table
([schema in `0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql))
with these fields:

- `event_id`, `event_type` (dotted, e.g. `platform_admin.claimed`).
- `actor_agent_id`, `target_entity_id` (both nullable).
- `timestamp`, `diff` (free-form JSON before/after).
- `audit_class`, `provenance_auth_request_id`, `org_scope`.
- `prev_event_hash_b64` — the chain linkage, computed by the
  emitter from the previous event in the same `org_scope`.

## What M1 does NOT yet ship

- **Retention enforcement**. No background job deletes `Silent`
  events after 30 days or `Logged` events after 365. The counts
  grow unbounded. Planned in M7b.
- **Off-site hash-chain stream** (S3 object-lock or equivalent).
  The plan's tamper-evidence story relies on shipping the chain
  to an append-only external store, which is an M7b item. M1's
  hash chain is therefore "tamper-evident for a cooperative
  operator" rather than "tamper-evident against an attacker with
  write access to the DB".
- **Alerted-class delivery**. The `Alerted` tier is recognised in
  the schema; the actual per-org delivery channel (Slack webhook /
  email / PagerDuty) is wired in M2's admin pages. M1 persists
  `Alerted` events in the DB but does not push them anywhere.

## Querying the audit log (M1-compatible recipes)

There is no admin UI in M1. Operators query via the repository
trait or by embedding a SurrealDB shell at the `data_dir`:

```sql
-- Recent Alerted events across the whole install
SELECT * FROM audit_events
WHERE audit_class = 'alerted'
ORDER BY timestamp DESC
LIMIT 20;

-- Chain of events for a specific organisation
SELECT event_id, event_type, timestamp, prev_event_hash_b64
FROM audit_events
WHERE org_scope = $org_id
ORDER BY timestamp ASC;

-- "Which events have NO predecessor in their org?" (genesis events)
SELECT event_id, event_type, org_scope
FROM audit_events
WHERE prev_event_hash_b64 IS NONE;
```

Exactly **one** platform-scope genesis event exists per install —
`platform_admin.claimed`. Each organisation also has a genesis
event when it's first created (M3 feature).

## Verifying the hash chain

The chain is a unidirectional linked list of events within each
`org_scope`. Given the latest event, a verifier can re-hash each
predecessor and confirm the stored `prev_event_hash_b64` field
matches. Four proptest invariants in
[`audit_hash_chain_props.rs`](../../../../../../modules/crates/domain/tests/audit_hash_chain_props.rs)
pin the guarantees:

1. `hash_event` is independent of the `prev_event_hash` field (no
   self-reference).
2. Changing any captured field changes the hash.
3. `org_scope` is in the hash preimage (no cross-tenant merges).
4. Two-event chain detects a tampered predecessor.

An operator-facing verify tool (`baby-phi admin verify-audit-chain
--org <id>`) is an M7b item.

## Shadow NDJSON log

ADR-0013 names a shadow NDJSON append at `{data_dir}/audit.log` for
cheap recoverability if SurrealDB is lost. M1's implementation
scope: the shape is defined, the emitter's contract requires it,
but the concrete writer lands alongside the M2 audit-event emitter
work. Operators should NOT rely on the shadow log in M1 — the
SurrealDB table is the source of truth.

## What M1 does NOT ship (deferred)

- Retention-enforcement background job (M7b).
- Off-site tamper-evident stream (M7b — S3 object-lock).
- Alerted-tier delivery channels (M2 admin pages).
- Shadow NDJSON writer implementation (M2).
- Admin UI for audit-log browsing (M2).

## Cross-references

- [architecture/audit-events.md](../architecture/audit-events.md)
  — the event shape + chain design.
- [ADR-0013](../decisions/0013-audit-events-class-and-chain.md) —
  the class-tier + hash-chain decision.
- [requirements/cross-cutting/nfr-observability.md](../../../requirements/cross-cutting/nfr-observability.md)
  — retention rules per class.
- [modules/crates/domain/src/audit.rs](../../../../../../modules/crates/domain/src/audit.rs)
  — the `AuditEvent` + `AuditClass` + `hash_event` types.
- [modules/crates/domain/tests/audit_hash_chain_props.rs](../../../../../../modules/crates/domain/tests/audit_hash_chain_props.rs)
  — the chain invariants exercised in CI.
