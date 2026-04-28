<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0040 — Memory-extraction listener heuristic v0; LLM-driven supervisor body deferred

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-21
**Closes:** D6.1 (HIGH, terminal closure) — first call site for `record_system_agent_fire` plus the heuristic listener body that v0 commits to.

---

## Context

[`concepts/system-agents.md`](../../../concepts/system-agents.md) § "Memory Extraction Agent" describes the canonical v0 model: a **supervisor agent** that runs `agent_loop` against the just-ended session's transcript, identifies candidate memories via LLM judgment, decides per-memory pool routing (`agent:` private / `project:` / `org:` / `#public`) by matching the source-session tags to the `Allocation Rules` table, writes each memory via the standard `store_memory` tool, and emits a `MemoryExtracted` audit event per memory.

That full body is a multi-day effort — provider plumbing, prompt design, tool grant resolution, retry semantics, queue management, the "queue saturation / agent disabled / retry exhausted" failure modes the forward-scope row calls out. CH-21's budget is **~1.5 engineer-days**.

The listener was scaffolded at M5/P3 ([`listeners.rs:505-536`](../../../../../../modules/crates/domain/src/events/listeners.rs)) as a stub that logged on `SessionEnded` and otherwise no-op'd. The `record_system_agent_fire` helper landed at M5/P6 with **zero call sites** — drift D6.1. CH-22 shipped the second call site (catalog listener); CH-21 is the closer for the first call site (memory extractor).

CH-16 / ADR-0038 §D38.5 made an explicit commitment that the `IdentityUpdated` `DomainEvent` variant + the `identity_updated` audit emitter would ship variant-only, with **CH-21 lighting the first emitter** when `MemoryExtracted` triggers a `witnessed.memories_extracted` bump.

The chunk discipline forbids "ship a stub for now and fix it later" — concept doc claims must be either honored, marked `silent-in-code`, or transitioned through a documented drift to a future chunk. CH-21 splits the concept's behaviour into:

- **Storage substrate** (one Memory per session, Identity counter advance, audit chain, telemetry tile fire) → honored at v0.
- **LLM judgment, 4-pool routing, grant enforcement, multi-memory-per-session** → deferred to **M6-DEFERRED-04** (NEW marker) with explicit `silent-in-code` status carried in the concept-audit matrix.

## Decision

### D40.1 — Heuristic v0 listener; LLM body deferred to M6-DEFERRED-04

The `MemoryExtractionListener::on_event` body shipped at CH-21 is a **deterministic, synchronous, no-LLM-call** body. On every `DomainEvent::SessionEnded` for an LLM-kind working agent on a non-aborted session, the listener mints exactly one `Memory` row, derives its tags from the source session's tag set, decides a binary `{private, public}` scope, upserts the working agent's `Identity` row (`witnessed.memories_extracted += 1` + the matching scope-bucket counter), emits two audit events (`platform.memory.extracted` + `platform.identity.updated`), and advances the memory-extractor system agent's runtime-status tile.

The full LLM-driven supervisor agent body — running `agent_loop`, identifying candidates, choosing pools per the concept's `Allocation Rules` table, writing via `store_memory` tool — is deferred to **M6-DEFERRED-04 — Memory-extraction LLM supervisor body**. That marker is appended to the forward-scope file at CH-21 seal.

Rationale: heuristic v0 closes drift D6.1 with real call-site behaviour, gives downstream consumers (Identity counters, audit chain, runtime-status tile) a real signal path to verify against, and creates a working substrate the LLM body can replace later. A "stub + telemetry only" alternative (Fork 1 option C) was rejected because it leaves D6.1's concept claim — "agent emits `MemoryExtracted` audit per memory" — formally unsatisfied; we'd close the call-site drift but pretend the concept claim is honored when no Memory rows are minted.

### D40.2 — Tag derivation: union of session tags + governance scope tags

Memory tags are the union of:
- `Session.tags` (the CH-06 / D-new-11 instance-identity tag set: `#kind:session`, `session:{id}`, plus any user-supplied tags on the originating session).
- `agent:{owning_agent}` — the working agent (session's `started_by`).
- `session:{session_id}` — already present from CH-06; included idempotently.
- `project:{project_id}`.
- `org:{owning_org}`.

De-duplicated; tag order is deterministic (session.tags first, governance tags appended in fixed order).

The binary `{private, public}` scope decision is driven by the resulting tag set:
- Tag set contains `#public` → `ExtractionScope::Public`.
- Otherwise → `ExtractionScope::Private`.

The 4-pool routing concept-`system-agents.md` § "Allocation Rules" specifies (`agent:{owner}` / `project:{id}` / `org:{id}` / `#public`) is **NOT** implemented at v0. The `Identity.witnessed.extraction_scope_distribution` carrier is a binary `{private, public}` field per CH-16 / ADR-0038; CH-21 maps the concept's 4 pools onto this binary surface — project-scoped and org-scoped memories count as `private` at v0.

### D40.3 — Disabled-state behavior: SKIP BOTH (extraction + telemetry)

When the org's `memory-extraction-agent` system agent has `active = false` or `archived_at = Some(_)`, the listener logs at `warn!` level and returns. No Memory row, no Identity update, no audit emission, **and no `record_system_agent_fire` call**.

The "skip extraction but record telemetry fire" alternative was rejected: recording a fire-event for a disabled agent would mislead the runtime-status dashboard into showing recent activity from a system the operator deliberately silenced. The "continue extraction regardless" alternative defeats the operator-disable contract.

### D40.4 — Heuristic listener bypasses Supervisor Extraction grants

[`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) § "Supervisor Extraction as Two Standard Grants" specifies the LLM-driven supervisor agent holds two standard grants — read sessions + store memories — and that extraction is permission-checked. At v0 the heuristic listener writes `Memory` rows via direct `Repository::create_memory` calls, **bypassing** any grant check.

This is a deliberate carry-forward to **M6-DEFERRED-04**. The listener is a privileged platform-level path; treating it as if it held the supervisor grants would be a fiction (no `actor` identity on the call). When the LLM body lands, extraction flows through the agent's tool grants the same way every other tool call does, and the permission-check engine governs writes.

### D40.5 — v0 simplification: working agent = its own extractor

Concept-`agent.md` § "Two Streams of Experience" frames extraction as a supervisor-as-actor pattern: a separate supervisor agent extracts memories from a subordinate session's transcript, and that supervisor's `witnessed.memories_extracted` advances. At v0 the heuristic listener has no agent identity — the working agent (session's `started_by`) is treated as **its own extractor**: the Memory's `owning_agent` is the working agent, and `Identity.witnessed.memories_extracted` advances on the working agent's own row.

The supervisor-as-actor pattern lands with the LLM body in M6-DEFERRED-04. At v0, this simplification keeps the substrate working without inventing a phantom supervisor identity.

### D40.6 — Single-pod assumption + bus re-emission deferred

If multiple pods subscribe to the same EventBus, the same `SessionEnded` would fire multiple extractions — drift potential under K8s fan-out. v0 assumes **single-pod** listener semantics; the M7b broker carve-out (per ADR-0033) covers cross-pod dedup with a delivery-once-per-broker-group contract.

Bus re-emission of the new `DomainEvent::MemoryExtracted` + the existing `DomainEvent::IdentityUpdated` from inside the listener is **deferred at v0**:
- No current consumer subscribes to either variant (catalog listener short-circuits both).
- Holding `Arc<dyn EventBus>` in the listener creates a cycle (bus → listener → bus) that leaks the bus on shutdown.
- A clean fix uses `Weak<dyn EventBus>` upgrade-on-emit, which is meaningful design effort better solved when there's a real subscriber to drive it.

The audit log captures all reactive state needed by downstream observers; the variant exists for forward-compat.

### D40.7 — Out of Scope (carve-out)

The following concept claims are explicitly **out of scope** at CH-21 and routed to M6-DEFERRED-04:

- LLM-driven candidate selection from the session transcript (concept-`system-agents.md` § Behaviour 2).
- 4-pool memory routing per the `Allocation Rules` table (§ Behaviour 3).
- Tool-grant-mediated `store_memory` writes (§ Behaviour 4).
- Supervisor Extraction grants enforced (concept-`permissions/05-memory-sessions.md` § "Supervisor Extraction as Two Standard Grants").
- Multi-memory-per-session extraction.
- Per-memory-text content. The v0 `Memory` struct (`{id, owning_agent, tags, created_at}`) carries no body field — content is encoded entirely in tags. M6 schema-evolution may extend.
- `parallelize`-driven concurrent extractions per concept-`system-agents.md`'s `parallelize: 2` defaults block; the listener serialises through the bus's snapshot-and-release semantics at v0.

## Conforming criteria

- `MemoryExtractionListener::on_event` body lives at [`modules/crates/domain/src/events/listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs) and is fully exercised by 8 unit tests + 6 acceptance scenarios + 1 5-listener-invariant regression.
- `record_system_agent_fire` is called from at least 2 sites under `modules/crates/` (catalog from CH-22, memory from CH-21).
- `MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME` constant + `resolve_memory_extraction_system_agent` helper exist on the listener.
- Disabled-state path covered by `memory_extraction_listener_skips_when_extractor_disabled` (unit) + `scenario_3_disabled_extractor_skips_both_extraction_and_telemetry` (acceptance).
- Aborted-session path covered by `memory_extraction_listener_skips_aborted_session` (unit) + `scenario_2_aborted_session_skips_extraction` (acceptance).
- Human-kind path covered by `memory_extraction_listener_skips_human_kind_working_agent` (unit) + `scenario_4_human_working_agent_skips_extraction` (acceptance).
- Audit chain covered by `scenario_6_audit_chain_orders_memory_extracted_before_identity_updated` (acceptance).
- D6.1 lifecycle entry transitions to `remediated` at chunk seal.

## Alternatives considered

- **Full LLM-driven supervisor body at CH-21.** Rejected — exceeds the 1.5d budget; the provider plumbing alone is a chunk. Defers to M6-DEFERRED-04.
- **Stub + telemetry only.** Rejected — closes D6.1 syntactically but leaves the concept's "agent emits `MemoryExtracted` audit per memory" claim formally unsatisfied. Heuristic v0 satisfies the storage-substrate clauses honestly.
- **All-private at v0 (skip the binary scope decision).** Rejected — Fork 2 user-selected DERIVE FROM SESSION TAGS to keep the `Identity.witnessed.extraction_scope_distribution.{private,public}` carrier from CH-16 honest. Tracking only `private` at v0 would leave the `public` counter dead until the LLM body lands.
- **Bus re-emission at v0.** Rejected — Arc cycle + no subscribers; deferred to a future chunk that can design the `Weak`-based break-cycle pattern alongside the first real consumer.

## Out of scope

See D40.7. Tracked successor: **M6-DEFERRED-04 — Memory-extraction LLM supervisor body** (NEW deferral marker).

## References

- Concept docs: [`system-agents.md`](../../../concepts/system-agents.md) § Memory Extraction Agent; [`permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) § Supervisor Extraction; [`agent.md`](../../../concepts/agent.md) § Two Streams of Experience.
- Drift: [D6.1.md](../../m5_1/drifts/D6.1.md) (terminally closed at CH-21 seal).
- Plan archive: [`build/bb95cd12-ch-21-memory-extraction-listener-body.md`](../../../../plan/build/bb95cd12-ch-21-memory-extraction-listener-body.md).
- ADRs cross-referenced: ADR-0028 (fail-safe listener semantics), ADR-0033 (K8s prep), ADR-0034 (Agent.active discriminator), ADR-0035 (catalog listener audit-mode + production emit sites), ADR-0038 (Identity materialisation + first-emitter commitment), ADR-0041 (`MemoryExtracted` event + audit class).
