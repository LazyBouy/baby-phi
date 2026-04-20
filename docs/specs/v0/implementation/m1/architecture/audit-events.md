<!-- Last verified: 2026-04-20 by Claude Code -->

# Architecture — audit events

M1 lands the audit-event framework: a base schema, three retention class
tiers, and a per-org hash-chain seed. The schema matches
[`docs/specs/v0/requirements/cross-cutting/nfr-observability.md`](../../../requirements/cross-cutting/nfr-observability.md).
Rationale for the hash-chain seed is in
[ADR-0013](../decisions/0013-audit-events-class-and-chain.md).

## Shape

Defined at [`domain::audit`](../../../../../../modules/crates/domain/src/audit.rs):

```rust
pub struct AuditEvent {
    pub event_id: AuditEventId,
    pub event_type: String,                 // "platform_admin.claimed", ...
    pub actor_agent_id: Option<AgentId>,
    pub target_entity_id: Option<NodeId>,
    pub timestamp: DateTime<Utc>,
    pub diff: serde_json::Value,            // before/after, event-specific
    pub audit_class: AuditClass,            // Silent / Logged / Alerted
    pub provenance_auth_request_id: Option<AuthRequestId>,
    pub org_scope: Option<OrgId>,
    pub prev_event_hash: Option<[u8; 32]>,  // BLAKE3-256 of the previous event
}
```

## Class tiers

| `AuditClass` | Default retention | Delivery |
|---|---|---|
| `Silent` | 30 days | No delivery — still written to the log for forensics |
| `Logged` | 365 days | Streamed to structured log sink |
| `Alerted` | 365+ days | Delivered to the org's alert channel within 60 s |

Retention is enforced downstream — M1 writes events with the tier set; M7b
wires the retention sweep + off-site stream.

## Hash chain

Every event has `prev_event_hash` = BLAKE3-256 of the previous event within
the same `org_scope`. The helper
[`hash_event`](../../../../../../modules/crates/domain/src/audit.rs) computes
a digest over the event's canonical bytes (see below); emitters look up
the last-recorded event for the org and populate `prev_event_hash` before
writing.

**Canonical bytes** exclude the `prev_event_hash` field itself — otherwise a
circular dependency would form. The exclusion is enforced by a unit test:
`canonical_bytes_excludes_prev_event_hash`.

In M1, the chain is local — `audit_events.prev_event_hash_b64` sits next to
the event row. In M7b it's extended to an append-only off-site S3/GCS
stream with object-lock, making tampering detectable end-to-end.

## Emitter contract

[`AuditEmitter`](../../../../../../modules/crates/domain/src/audit.rs) is the
write-side trait. Implementations (landing in P2/P6) must:

1. Look up the last `prev_event_hash_b64` for `event.org_scope` and set
   `event.prev_event_hash` before persisting.
2. Write the event to the `audit_events` table.
3. Append the serialized event to the shadow NDJSON log at
   `{data_dir}/audit.log` — cheap recoverability if the DB is corrupted.
4. For `Alerted` events, schedule delivery to the org's alert channel.

P1 defines the trait only; the concrete implementation follows repository
expansion in P2 and server wiring in P6.

## Storage format

In SurrealDB (per
[`0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql)):

```surql
DEFINE TABLE audit_events SCHEMAFULL;
DEFINE FIELD event_type            ON audit_events TYPE string;
DEFINE FIELD actor_agent_id        ON audit_events TYPE option<string>;
DEFINE FIELD target_entity_id      ON audit_events TYPE option<string>;
DEFINE FIELD timestamp             ON audit_events TYPE datetime;
DEFINE FIELD diff                  ON audit_events TYPE object;
DEFINE FIELD audit_class           ON audit_events TYPE string
    ASSERT $value INSIDE ["silent", "logged", "alerted"];
DEFINE FIELD provenance_auth_request_id ON audit_events TYPE option<string>;
DEFINE FIELD org_scope             ON audit_events TYPE option<string>;
DEFINE FIELD prev_event_hash_b64   ON audit_events TYPE option<string>;
DEFINE INDEX audit_events_org_timestamp ON audit_events FIELDS org_scope, timestamp;
```

The `audit_class` string ASSERT enforces the tier at write time.
`prev_event_hash_b64` is base64 (standard, no-padding) — see the rationale
in [at-rest-encryption.md](at-rest-encryption.md) for why bytes-via-base64
rather than native `bytes`.

## Tests (P1)

| Test | File | Asserts |
|---|---|---|
| `canonical_bytes_excludes_prev_event_hash` | `audit.rs` | Changing `prev_event_hash` leaves canonical bytes unchanged |
| `hash_is_deterministic_for_same_content` | `audit.rs` | Identical events hash identically |
| `hash_changes_when_content_changes` | `audit.rs` | Flipping a field flips the hash |
| `audit_class_serde_roundtrip` | `audit.rs` | Wire format stable for the three tiers |

Hash-chain walk proptests + emitter integration land in P2 / P4 when the
repository grows `write_audit_event` + `last_event_hash_for_org`.

## Concept references

- NFR: `docs/specs/v0/requirements/cross-cutting/nfr-observability.md`.
- ADR: [0013 audit events — class + chain](../decisions/0013-audit-events-class-and-chain.md).
- Build plan: "Audit-log tamper resistance" row — M1 seeds, M7b completes.
