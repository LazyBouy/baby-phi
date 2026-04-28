<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0038 — Identity node materialized as 4-field struct; eager creation per LLM agent in `apply_agent_creation`; embedding population deferred

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-16
**Closes:** D-new-01 (HIGH) — 4-field Identity materialization

---

## Context

[`concepts/agent.md`](../../../concepts/agent.md) § "Identity (Emergent, Event-Driven)" + § "Identity Node Content — Provisional Direction" labels the four-field Identity node (`self_description` / `lived` / `witnessed` / `embedding`) as **"the v0 commitment — implementations should code against it"**. The implementation at [`modules/crates/domain/src/model/nodes.rs:852-857`](../../../../../../modules/crates/domain/src/model/nodes.rs) was `scaffold_node!(Identity, NodeId)` — an id-only stub with a `[PLANNED M5]` doc-comment that never landed.

[`concepts/ontology.md`](../../../concepts/ontology.md) lines 24–29 commits to Identity as a stored node (not computed-on-demand) with exactly four persistent fields, reactively updated. CH-21 (memory-extraction listener body) cannot land without a load-bearing `Repository::upsert_identity` to call.

## Decision

### D38.1 — Eager creation in `apply_agent_creation` for every LLM agent

`Identity` is materialized as a full struct (Fork 1 EAGER) — one row per LLM agent, written inside the existing `apply_agent_creation` compound tx (atomic with `Agent` + `InboxObject` + `OutboxObject`). Server orchestrator builds the row via `Identity::default_for_llm(agent.id, now)` for `AgentKind::Llm`; passes `None` for `AgentKind::Human`.

Lazy creation (on first `SessionEnded`) was rejected. Lazy means every Identity reader needs a "row absent vs row present-but-empty" branch; eager keeps the state model uniform.

### D38.2 — Four content fields per concept

`Identity` carries the four concept-mandated fields plus governance fields (`id`, `agent_id`, `created_at`, `updated_at`):

| Field | Type | Default at create |
|---|---|---|
| `self_description` | `String` | `""` (empty; agent-authored later) |
| `lived` | `LivedExperience` | zeroed struct |
| `witnessed` | `WitnessedExperience` | zeroed struct |
| `embedding` | `Vec<f32>` | `vec![]` |

`LivedExperience` field set matches concept-agent.md line 326 verbatim: `sessions_completed`, `sessions_successful`, `ratings_window: Vec<RatingPoint>`, `skills: Vec<SkillRef>`, `specializations: Vec<String>`.

`WitnessedExperience` field set matches concept-agent.md line 327 verbatim: `memories_extracted`, `subordinates_observed: Vec<AgentId>`, `extraction_scope_distribution`.

### D38.3 — Embedding population deferred to M6-DEFERRED-03

`embedding: Vec<f32>` defaults to `vec![]` at create (Fork 2 EMPTY). Concept-agent.md ties the embedding to `self_description` re-derivation; with `self_description` empty at create, any placeholder embedding is wasted compute and would produce false-positive similarity hits (the bug class concept-09's "single-model vector space" warning calls out at line 338).

Population is deferred to **M6-DEFERRED-03 — Identity embedding provider integration** (NEW deferral marker). The platform-level embedding model config that concept-agent.md § "Scoping the embedding model" references does not exist at v0.1; populating embeddings requires:

1. An embedding provider trait (akin to phi-core's `StreamProvider`).
2. A platform-bootstrap config field naming the model id + dimension.
3. A re-embed batch path for the "Model change is an admin event" case (concept-agent.md line 340).

CH-16 ships none of this; it ships only the storage substrate.

### D38.4 — `LivedExperience` / `WitnessedExperience` are inline structs

Rather than separate node types with `LIVED_BY` / `WITNESSED_BY` edges, `LivedExperience` and `WitnessedExperience` are inline structs serialised into the `identity` row's `lived` / `witnessed` columns (SurrealDB `FLEXIBLE TYPE object`). They are query-shape detail of the Identity row, not first-class nodes per `ontology.md` lines 24–29 (the table lists Identity as a single node type with these as fields, not as siblings).

### D38.5 — `IdentityUpdated` `DomainEvent` variant ships without a production emitter

CH-16 ships:

- The `DomainEvent::IdentityUpdated { agent_id, trigger, at, event_id }` variant.
- The `IdentityUpdateTrigger` enum with four variants (`SessionEnded`, `MemoryExtracted`, `SkillChanged`, `RatingReceived`).
- `kind() == "identity_updated"` and `event_id()` arms.
- The bus dispatch arms in the `EventBus` infra.

CH-16 does NOT ship a production emitter — the first one lands at CH-21 (memory-extraction listener body) when `MemoryExtracted` triggers a `witnessed` update. Direct repo calls from CH-21 were rejected per Fork 3; event-bus-mediated fan-out matches ADR-0028 fail-safe semantics and lets future writers (skill-change, rating-received in M6+) plug in without touching CH-21.

### D38.6 — Archive does NOT delete Identity (LEAVE QUERYABLE policy)

`AgentArchived` does not delete the Identity row (Fork 4 LEAVE QUERYABLE). Concept-agent.md § "Materialization" treats Identity as a continuously-updated record; preserving it after archive supports forensic / hiring / evaluation queries.

`Repository::delete_identity` exists on the trait for symmetry and operator-driven cleanup (e.g. GDPR erasure scripts) but no handler in the M5/M5.2 codebase calls it. Two regression tests pin the policy: `archive_does_not_delete_identity_row` and `operator_driven_delete_identity_after_archive_succeeds`.

### D38.7 — Identity does not carry a `tags` field

CH-06 / ADR-0037 §D37.3 enumerated the tag-emission scope as composites + `AuthRequest` + `ExternalService` + `InboxObject` + `OutboxObject` + `Session` + (already-existing) Memory. **Identity is NOT in that list**, by design:

- Identity is keyed-by `agent_id` (1:1 with the owning agent), not by its own `NodeId` discoverable via tag query.
- Identity is never a permission-check selector target — it is read by hiring/evaluation queries, not gated by selector match.

If a future query pattern needs tag-based identity discovery, that's a successor M6+ concern; ADR-0038 §D38.7 carries the deferral explicitly.

---

## Consequences

**Positive:**
- Concept-doc claim ("Identity is the v0 commitment") is honored; the four-field shape lands.
- D-new-01 closed at M5.2 (HIGH severity drift terminally remediated before M5 tag).
- CH-21 (memory-extraction listener body) unblocks — has a real `upsert_identity` to call.

**Negative:**
- Schema change at migration 0009: the existing `identity` scaffold (id-only from migration 0001) gets `OVERWRITE`-style field additions; pre-CH-16 rows materialise with empty content + serde defaults.
- The `embedding: Vec<f32>` field is dead weight at v0.1 (always empty); operators reading the row see a wasted column until M6-DEFERRED-03 lands.

**Mitigations:**
- `OVERWRITE` syntax in migration 0009 keeps old rows readable (no DROP).
- ADR-0037 §D37.3 + ADR-0038 §D38.7 explicitly document the no-tags carry-forward.

---

## Cross-References

- [`concepts/agent.md`](../../../concepts/agent.md) §§ "Identity (Emergent, Event-Driven)" / "Identity Node Content"
- [`concepts/ontology.md`](../../../concepts/ontology.md) §"Node Types — Core Identity"
- [`concepts/human-agent.md`](../../../concepts/human-agent.md) §"No Identity"
- [drift D-new-01](../../m5_1/drifts/D-new-01.md)
- ADR-0037 §D37.3 (CH-06) — Identity-without-tags carry-forward
- ADR-0039 (sibling) — Human Agent guard
- CH-16 plan archive: [`baby-phi/docs/specs/plan/build/2ae4fabe-ch-16-identity-node-materialization.md`](../../../../plan/build/2ae4fabe-ch-16-identity-node-materialization.md)
