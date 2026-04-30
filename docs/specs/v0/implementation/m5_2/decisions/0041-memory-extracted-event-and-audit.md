<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0041 — `DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit class

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-21
**Closes:** (none directly; pairs with ADR-0040 to close D6.1)

---

## Context

CH-21's memory-extraction listener body needs two things to slot into phi's existing reactive substrate cleanly:

1. A **`DomainEvent` variant** to represent "an extraction happened". Other listeners (now or future) may want to react — a memory-rebalancer in M6+, an embedding-similarity dashboard later. Without a typed variant the only reactive surface is the audit log, which has different latency + ordering semantics than the in-process bus.

2. An **audit-event class + emitter**. Concept-`system-agents.md` § "Memory Extraction Agent — Behaviour 5" commits to a `MemoryExtracted` audit event per memory. The CH-16 carry-forward (`platform.identity.updated` first emitter) is already wired by CH-21's listener; the matching `platform.memory.extracted` event needs an emitter helper alongside.

CH-21's plan §1 fork 3 user-selected DEFINE BOTH (DomainEvent variant + audit helper). This ADR ratifies the surface shape.

## Decision

### D41.1 — `DomainEvent::MemoryExtracted` variant

A new variant is added to `DomainEvent` at [`modules/crates/domain/src/events/mod.rs`](../../../../../../modules/crates/domain/src/events/mod.rs):

```rust
MemoryExtracted {
    memory_id: MemoryId,
    session_id: SessionId,
    owning_agent: AgentId,
    org_scope: Option<OrgId>,
    tags: Vec<String>,
    extracted_at: DateTime<Utc>,
    event_id: AuditEventId,
}
```

Match arms updated:
- `kind() == "memory_extracted"`.
- `event_id()` returns the variant's `event_id` field via the existing or-pattern arm.

Round-trip serde tests in `events::tests` cover the variant alongside the existing 10 variants.

The catalog listener's `agent_id_and_timestamp_for` resolver gets a new `None` arm for `MemoryExtracted` (the catalog row caches `display_name` / `kind` / `role` — none touched by extraction).

### D41.2 — Audit emitter helper at `audit/events/m5_2/memory.rs`

A new module ships at [`modules/crates/domain/src/audit/events/m5_2/memory.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/memory.rs) with the `memory_extracted` builder:

```rust
pub enum ExtractionScope { Private, Public }
impl ExtractionScope { pub fn as_str(&self) -> &'static str { … } }

pub fn memory_extracted(
    actor: AgentId,
    memory: &Memory,
    session_id: SessionId,
    scope_bucket: ExtractionScope,
    org: OrgId,
    timestamp: DateTime<Utc>,
) -> AuditEvent
```

Returned `AuditEvent` shape:
- `event_type = "platform.memory.extracted"`.
- `audit_class = AuditClass::Logged`.
- `actor_agent_id = Some(actor)` — the working agent at v0 (D40.5).
- `target_entity_id = Some(NodeId::from_uuid(*memory.id.as_uuid()))`.
- `org_scope = Some(org)`.
- `provenance_auth_request_id = None` — the heuristic listener has no AR (D40.4).
- `diff` = `{memory_id, session_id, owning_agent, tags, scope_bucket: "private"|"public"}`.

Module declared via `pub mod memory;` in [`audit/events/m5_2/mod.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/mod.rs) alongside the existing `identity` module.

### D41.3 — Audit class is Logged (not Alerted)

Memory extraction is **routine bookkeeping**, not a security-relevant or operator-action event. `Logged` (mid-tier 365-day retention; structured-log delivery; no alerting) matches:

- Concept-`system-agents.md` § "Memory Extraction Agent — Behaviour 5" treats the audit emission as observability data, not alerting.
- Pairs symmetrically with `platform.identity.updated` (CH-16 / ADR-0038) which is also Logged — both events are part of the same reactive update chain.
- Alerted class (60s delivery to org alert channel) is reserved for security-affecting writes (grants issued, permissions changed). Memory extraction does not move governance state in a way that warrants that envelope.

### D41.4 — First emitter wires at `MemoryExtractionListener`; ADR-0028 fail-safe

Per ADR-0040 §D40.6, bus re-emission of `DomainEvent::MemoryExtracted` is deferred at v0 (no consumer; Arc cycle concern). The audit emit is the only durable signal CH-21 ships.

The audit emit is **post-commit** of the underlying `Memory` row (ADR-0028 fail-safe semantics): `Repository::create_memory` succeeds first; only then does `audit.emit(memory_extracted(...))` fire. If the audit emit fails, the Memory is durable and the audit trail has a gap that operators can replay; the listener logs `error!` and continues to the next step (Identity update).

The emit is **paired** with `identity_updated` from CH-16's `audit/events/m5_2/identity.rs`. The listener emits `memory_extracted` first, then `identity_updated` — `scenario_6_audit_chain_orders_memory_extracted_before_identity_updated` pins this ordering.

## Conforming criteria

- `DomainEvent::MemoryExtracted` variant exists with the 7-field payload shape.
- `kind()` returns `"memory_extracted"` for the variant.
- `event_id()` accessor matches the emitted value (verified by `event_id_accessor_matches_emitted_value_for_every_variant`).
- Serde round-trip pinned (`memory_extracted_roundtrips`).
- `audit::events::m5_2::memory::memory_extracted` helper returns `event_type = "platform.memory.extracted"` + `audit_class = AuditClass::Logged`.
- `audit::events::m5_2::mod.rs` declares `pub mod memory;`.
- `MemoryExtractionListener::on_event` calls `memory_extracted` after `Repository::create_memory` succeeds and before `identity_updated`.
- Three audit-helper unit tests: Logged class + private bucket, public bucket, empty tag list.
- One acceptance test pins ordering against the same org's audit log.

## Alternatives considered

- **Audit-only (no DomainEvent variant).** Rejected per Fork 3 — leaves no in-process reactive surface; future memory-rebalancer or dashboard-emitter listeners would need to scrape the audit log.
- **Alerted audit class.** Rejected — extraction is observability data, not security-affecting. Pairs symmetrically with `platform.identity.updated` (Logged) which fires alongside it.
- **`Silent` audit class.** Rejected — extraction is the load-bearing signal that downstream operators need to debug (concept-`system-agents.md` § "Behaviour 5" pins this as a first-class event); 30-day retention loses ops fidelity.

## Out of scope

- Bus re-emission of `DomainEvent::MemoryExtracted` from inside the listener (ADR-0040 §D40.6 deferred to a future chunk with a clean `Weak<dyn EventBus>` design).

## References

- Concept docs: [`system-agents.md`](../../../concepts/system-agents.md) § "Memory Extraction Agent — Behaviour 5".
- ADRs cross-referenced: ADR-0028 (fail-safe listener semantics), ADR-0038 (CH-16 first-emitter commitment for `IdentityUpdated`), ADR-0040 (heuristic v0 listener).
- Plan archive: [`build/ch-21-memory-extraction-listener-body-bb95cd12.md`](../../../../plan/build/ch-21-memory-extraction-listener-body-bb95cd12.md).
