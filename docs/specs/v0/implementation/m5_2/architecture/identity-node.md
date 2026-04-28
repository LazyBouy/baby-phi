<!-- Last verified: 2026-04-28 by Claude Code -->

# Identity node — design page

> **Status:** [EXISTS] as of CH-16 (M5.2). The four-field Identity node ships in [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); migration 0009 adds the `identity` table; eager creation lands inside `apply_agent_creation` for every LLM agent. For the normative concept-doc reference, read [`concepts/agent.md`](../../../concepts/agent.md) §"Identity (Emergent, Event-Driven)" + §"Identity Node Content".

---

## What this page covers

`Identity` is the emergent self of an LLM agent. Concept-`agent.md` calls it the "v0 commitment" — a stored node, reactively updated by session-end / memory-extraction / skill-change / rating-received triggers. CH-16 turns the id-only scaffold (from migration 0001) into the full four-field shape.

This page describes:

- The struct shape + supporting inline types.
- How creation timing works (eager via `apply_agent_creation`).
- The Human-Agent guard pair (defensive at repo + preventive at call site).
- The `IdentityUpdated` event surface and how CH-21 will use it.
- The orphan-on-archive policy.

ADR-0038 + ADR-0039 record the design decisions; this page is the operator-facing description.

---

## Struct shape

```rust
pub struct Identity {
    pub id: NodeId,
    pub agent_id: AgentId,            // UNIQUE per migration 0009; one row per LLM agent
    pub self_description: String,      // ≤500 tokens, agent-authored (CH-21 synthesises)
    pub lived: LivedExperience,        // direct-doing metrics
    pub witnessed: WitnessedExperience, // supervised-doing metrics
    pub embedding: Vec<f32>,           // cosine-similarity vector (M6-DEFERRED-03)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,     // bumped on every reactive update
}

pub struct LivedExperience {
    pub sessions_completed: u64,
    pub sessions_successful: u64,
    pub ratings_window: Vec<RatingPoint>, // last 20 per token-economy.md
    pub skills: Vec<SkillRef>,
    pub specializations: Vec<String>,     // top tags by frequency
}

pub struct WitnessedExperience {
    pub memories_extracted: u64,
    pub subordinates_observed: Vec<AgentId>,
    pub extraction_scope_distribution: ExtractionScopeDistribution,
}
```

Field names match concept-`agent.md` lines 326–328 and concept-`ontology.md` lines 24–29 verbatim — the migration matches them byte-for-byte so a future query maps directly from concept-doc to schema.

---

## Creation timing — eager via `apply_agent_creation`

ADR-0038 §D38.1 (Fork 1 EAGER). Every LLM agent gets one Identity row, written inside the same compound tx as the Agent + InboxObject + OutboxObject. Server orchestrator builds via `Identity::default_for_llm(agent.id, now)`; defaults are zeroed content, both timestamps set to `now`.

Writers: [`server/src/platform/agents/create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs) (LLM agents only); [`server/src/platform/system_agents/add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs) (system agents are LLM-kind by definition). The bootstrap CEO path in [`bootstrap/claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs) is Human-kind — Identity stays `None`.

Lazy creation (on first `SessionEnded`) was rejected. Lazy means every Identity reader needs a "row absent vs row present-but-empty" branch; eager keeps the state model uniform.

---

## Human-Agent guard pair

ADR-0039 §D39.1 — defense in depth.

**Defensive guard at [`Repository::upsert_identity`](../../../../../../modules/crates/domain/src/repository.rs):**

- Reads `Agent.kind` for `identity.agent_id`.
- Returns `RepositoryError::HumanAgentHasNoIdentity { agent_id }` if `kind == AgentKind::Human`.
- Implemented at both `InMemoryRepository` and `SurrealRepository` layers; SurrealQL fast-path reads via `SELECT kind FROM type::thing('agent', $aid)`.

**Preventive guard at [`Repository::apply_agent_creation`](../../../../../../modules/crates/domain/src/repository.rs):**

- Pre-flight match on `(payload.agent.kind, &payload.identity)`:
  - `(Human, Some(_))` → `Err(HumanAgentHasNoIdentity)`
  - `(Llm, None)` → `Err(InvalidArgument)` (orchestrator must build via `default_for_llm`)
  - `(Human, None)` → OK, Identity insertion skipped
  - `(Llm, Some(_))` → OK, Identity insertion proceeds

A Human-kind agent never has an Identity row; the absence is the invariant.

---

## `IdentityUpdated` event

```rust
pub enum DomainEvent {
    IdentityUpdated {
        agent_id: AgentId,
        trigger: IdentityUpdateTrigger,
        at: DateTime<Utc>,
        event_id: AuditEventId,
    },
}

pub enum IdentityUpdateTrigger {
    SessionEnded, MemoryExtracted, SkillChanged, RatingReceived,
}
```

CH-16 ships the variant + bus dispatch arms; **no production emitter** at v0.1. CH-21 (memory-extraction listener body) lights the first emitter when `MemoryExtracted` triggers a `witnessed` update. Future writers (skill-change, rating-received in M6+) plug into the same event without touching CH-21's call site.

---

## Audit events

Two audit-event builders ship at [`domain/src/audit/events/m5_2/identity.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/identity.rs):

| Event type | Class | Emitted when | Payload summary |
|---|---|---|---|
| `platform.identity.created` | Alerted | After `apply_agent_creation` commits an Identity row for an LLM agent | `{agent_id, self_description_len: 0, lived_sessions_completed: 0, witnessed_memories_extracted: 0, embedding_dim: 0}` |
| `platform.identity.updated` | Logged | Future reactive updates (CH-21+) | `{trigger: <kind>, before: {…summary}, after: {…summary}}` |

The `created` event is paired 1:1 with `platform.agent.created` in the same hash-chain segment. Operators can join the two on `actor_agent_id` + `target_entity_id` (the Identity row's `id`, which differs from the agent id but is connected via the `agent_id` field).

---

## Orphan-on-archive policy

ADR-0038 §D38.6 (Fork 4 LEAVE QUERYABLE). Archiving an LLM agent flips `Agent.active = false` but the Identity row stays queryable. Concept-`agent.md` § "Materialization" treats Identity as a continuously-updated record; preserving it post-archive supports forensic / hiring / evaluation queries (e.g. "who used to lead the website-redesign project before they retired?").

`Repository::delete_identity` exists for operator-driven cleanup (GDPR erasure scripts, etc.) but no production handler calls it. Two regression tests pin the policy:

- `archive_does_not_delete_identity_row` — archive leaves the row queryable.
- `operator_driven_delete_identity_after_archive_succeeds` — explicit `delete_identity` after archive succeeds.

---

## What's deferred

Per ADR-0038 §D38.3 + §D38.7:

- **Embedding population** → M6-DEFERRED-03 (Identity embedding provider integration). v0.1 has no embedding provider; populating `embedding: Vec<f32>` requires platform-level model config + a re-embed batch path for the "Model change is an admin event" case.
- **`tags` field on Identity** → no — Identity is keyed-by `agent_id` and never a permission-check selector target.
- **`self_description` synthesiser** → CH-21. CH-16 ships only the storage substrate.
- **CLI surface** (`phi identity show <agent_id>`) → CH-19 / M6.

---

## Cross-References

- [`concepts/agent.md`](../../../concepts/agent.md) — normative spec (§"Identity (Emergent, Event-Driven)" + §"Identity Node Content")
- [`concepts/human-agent.md`](../../../concepts/human-agent.md) §"No Identity"
- [`concepts/ontology.md`](../../../concepts/ontology.md) §"Node Types — Core Identity"
- ADR-0038 — design decisions
- ADR-0039 — Human-Agent guard
- [`m5_2/operations/identity-operations.md`](../operations/identity-operations.md) — runbook
- [`m5_2/user-guide/identity-overview.md`](../user-guide/identity-overview.md) — operator reference
