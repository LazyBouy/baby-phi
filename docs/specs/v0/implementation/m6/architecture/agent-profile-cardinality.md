<!-- Last verified: 2026-05-19 by Claude Code (CH-28 P-DOCS — NEW architecture doc authored at P-DOCS deliverable 3 per plan §7. Authoritative architecture doc for the AgentProfile N:1 + hybrid Blueprint redesign landed by ADR-0063 (3 user-locked DIVERGENT forks F1.c hybrid blueprint table + F2.b USES_PROFILE rename + F3.b split migrations). The 7 sections below cover: §1 cardinality model; §2 motivation + rationale; §3 hybrid Blueprint storage architecture; §4 edge rename rationale; §5 split-migration walkthrough; §6 pre/post-CH-28 graph comparison; §7 references. Cycle hex `0412eb06`.) -->

# Architecture — AgentProfile cardinality (N:1) + hybrid Blueprint table

CH-28 flips the `domain::Agent` ↔ `domain::AgentProfile` cardinality from
**1:1** ("profile-as-genetics") to **N:1** ("profile-as-template, shared
across agents") and introduces a NEW hybrid `Blueprint` table to carry
per-agent governance overrides without bloating either side of the relation.
This is the first M6-tier ADR (ADR-0063) and the first split-migration in
the workspace (0019 schema + 0020 backfill).

Source-of-truth concept docs:

- [`concepts/agent.md`](../../../concepts/agent.md) §"Soul (Template Blueprint + per-agent overrides)" lines 160–178.
- [`concepts/ontology.md`](../../../concepts/ontology.md) §"Edge Types (74 total)" + §"Agent-Centric (first-order)" line 98 + the two NEW rows below it.

Authoritative ADR: [`m6/decisions/0063-agent-profile-cardinality-n-to-1.md`](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §1 — Cardinality model (pre-CH-28 vs post-CH-28)

| Aspect | Pre-CH-28 (M5 / CH-01 model) | Post-CH-28 (M6 / CH-28 model) |
|---|---|---|
| Cardinality | `Agent | HAS_PROFILE | AgentProfile | 1:1` | `Agent | USES_PROFILE | AgentProfile | N:1` |
| UNIQUE constraint | `DEFINE INDEX agent_profile_agent_id ... FIELDS agent_id UNIQUE` at [`migrations/0001_initial.surql:131`](../../../../../../modules/crates/store/migrations/0001_initial.surql) | UNIQUE on `agent_profile.agent_id` DROPPED in [`migrations/0019_agent_profile_n_to_1_schema.surql`](../../../../../../modules/crates/store/migrations/) |
| Per-agent overrides | Fields directly on `AgentProfile` literal (`parallelize`, `model_config_id`, `mock_response`) | Moved to NEW `blueprint` table per-agent override row, reached via `AGENT_USES_BLUEPRINT_OVERRIDE` edge |
| Soul framing | "Agent's genetics — defined at creation, never mutated" | "Template Blueprint (shareable across N agents) + per-agent override Blueprint (auditable mutation)" |
| Audit-event | `HasProfileEdgeChanged` (per agent) | `UsesProfileEdgeChanged` (per agent) + `BlueprintUpserted` (per override write) |

The semantic shift: a profile is now a *template* identity (think
"intern coder", "research assistant" — a named role-shape) rather than the
private genome of one particular agent. The immutability discipline that
held for the whole 1:1 Soul now applies at two distinct levels:

1. The **template Blueprint** (the shared `agent_profile` row + its template
   `blueprint` row) is write-once. Re-authoring a template is a new template,
   not a mutation.
2. The **per-agent override Blueprint** (the per-agent `blueprint` row
   reached via `AGENT_USES_BLUEPRINT_OVERRIDE`) IS mutable, but every mutation
   is an explicit governance event with its own audit trail per
   [ADR-0063 §D63.7](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §2 — Motivation + rationale (why N:1 over 1:1)

The 1:1 model was concept-mandated at v0 plan-time
([`concepts/agent.md`](../../../concepts/agent.md) §"Soul"
+ [`concepts/ontology.md`](../../../concepts/ontology.md) line 98 pre-CH-28)
and made literal sense when "profile" was framed as "the agent's DNA".

But three forces have pushed the model toward template-sharing:

1. **Profiles are role-shapes, not private state.** An "intern coder" or
   "research assistant" profile describes a *kind* of agent; an org that
   wants 5 interns of the same kind ends up with 5 identical
   `AgentProfile` rows (and 5 update sites every time the template is
   refined). Template-sharing eliminates the duplication.
2. **Infrastructure pattern alignment.** Kubernetes ConfigMaps, Helm
   values, container-image tags, and every other modern config-deployment
   primitive uses N:1 template-sharing. The 1:1 model is
   infrastructurally idiosyncratic.
3. **Ephemeral `AgentContext`.** Per [ADR-0034 §D34.6](../../m5_2/decisions/0034-agent-durable-lifecycle.md),
   baby-phi never instantiates `phi_core::Agent` / `BasicAgent` — the
   runtime trait is reified per-request from the `domain::Agent` ID plus
   a freshly-resolved profile. This makes profile sharing structurally
   feasible (there's no in-memory state to migrate); the cardinality was
   the last 1:1 obstruction.

The decision was deferred at CH-01 (M5 close was preserving 1:1 to align
with concept docs) and brought in-milestone for M6 per the user-lock at
2026-05-18 (`Q9` of the M6 forward-scope review). See
[ADR-0063 §"Context"](../decisions/0063-agent-profile-cardinality-n-to-1.md)
for the full deferral history.

**Rationale citation**: [ADR-0063 §D63.1](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §3 — Hybrid Blueprint storage architecture

The F1.c user-locked design is a **hybrid** because the NEW `blueprint`
table serves two distinct roles via a single uniform shape:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  agent_profile                                                  │
│  ┌────────────────┐                                             │
│  │ id             │  ← template identity (e.g. "intern_coder")  │
│  │ blueprint_id   │──┐                                          │
│  │ created_at     │  │                                          │
│  └────────────────┘  │                                          │
│         ↑            │ AGENT_PROFILE_USES_BLUEPRINT             │
│         │ USES       │                                          │
│         │ PROFILE    ▼                                          │
│         │            ┌────────────────────────────────────┐     │
│      ┌──┴──┐         │ blueprint                          │     │
│      │     │         │ ┌────────────────┐                 │     │
│      │ N   │         │ │ id             │  template-row   │     │
│      │     │         │ │ agent_id: NONE │  (shared)       │     │
│      └─────┘         │ │ parallelize    │                 │     │
│        agent         │ │ model_config_id│                 │     │
│         ↓            │ │ mock_response  │                 │     │
│         │            │ │ created_at     │                 │     │
│         │            │ └────────────────┘                 │     │
│         │ AGENT_USES_BLUEPRINT_OVERRIDE                    │     │
│         │ (zero-or-one)                                    │     │
│         ▼            │ ┌────────────────┐                 │     │
│                      │ │ id             │  override-row   │     │
│                      │ │ agent_id: Some │  (per-agent)    │     │
│                      │ │ parallelize    │                 │     │
│                      │ │ model_config_id│                 │     │
│                      │ │ mock_response  │                 │     │
│                      │ │ created_at     │                 │     │
│                      │ └────────────────┘                 │     │
│                      └────────────────────────────────────┘     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Template-vs-override distinction** is encoded via the `agent_id` field:

| Row type | `agent_id` | Reachability | Mutability |
|---|---|---|---|
| Template Blueprint | `NONE` | via `AGENT_PROFILE_USES_BLUEPRINT` from `AgentProfile` | write-once |
| Per-agent override Blueprint | `Some(<id>)` | via `AGENT_USES_BLUEPRINT_OVERRIDE` from `Agent` | mutable (auditable) |

A **UNIQUE-WHERE-agent_id-NOT-NONE** index enforces "at most one override
row per agent" without preventing template rows from being shared:

```surql
DEFINE INDEX blueprint_agent_id_unique
  ON TABLE blueprint
  FIELDS agent_id
  UNIQUE
  WHERE agent_id != NONE;
```

**Resolution semantic at read time** (effective override-field value for
a given agent):

1. Try the per-agent override Blueprint row first
   (`get_agent_blueprint_override_for_agent(agent_id)`).
2. If absent, fall back to the template Blueprint row reached via
   `AGENT_PROFILE_USES_BLUEPRINT` from the agent's `AgentProfile`
   (`get_blueprint_for_agent_profile(agent_profile_id)`).

This two-layer resolution is encapsulated in the NEW
`read_agent_profile_via_blueprint_or_fallback` repository helper
(see [ADR-0063 §D63.12](../decisions/0063-agent-profile-cardinality-n-to-1.md)
for the additional half-migrated-state handling the helper performs).

**Why one table instead of two (`blueprint_template` + `blueprint_override`)**:
operating on two parallel tables would double the read paths + double the
constructor boilerplate + double the Repository methods. The single-table
+ nullable `agent_id` shape captures the template-vs-override distinction
in a single field that's already audit-traced; the UNIQUE-WHERE index
delivers the per-agent override-row uniqueness without sacrificing
template-row reuse.

**Rationale citation**: [ADR-0063 §D63.1](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §4 — Edge rename: `HAS_PROFILE` → `USES_PROFILE`

The pre-CH-28 edge name `HAS_PROFILE` reflects the 1:1
"profile-as-genetics" semantic (an agent *has* its own profile, exclusively).
The post-CH-28 semantic is template-sharing: an agent *uses* a profile that
may be shared across N agents. The verb rename `HAS` → `USES` matches the
new semantic accurately ("uses" implies non-ownership).

**Cascade through 43 sites** (verified at planner iter-2 cascade-grep):

| Tier | Files | Sites |
|---|---|---|
| PRODUCTION | `domain/src/model/edges.rs` (variant + name + EDGE_KIND_NAMES + tests) | 5 |
| PRODUCTION | `domain/src/events/mod.rs` (DomainEvent variant + kind + event_id + roundtrip) | ~7 |
| PRODUCTION | `domain/src/events/listeners.rs` (match arms) | 5 |
| PRODUCTION | `domain/src/composites_m5.rs` (doc-comments) | 2 |
| PRODUCTION | `domain/src/repository.rs` (doc-comments) | 3 |
| PRODUCTION | `store/src/repo_impl.rs` (RELATE statements) | 4 |
| PRODUCTION | `server/src/platform/agents/update.rs` (event emission) | 1 |
| TEST | `acceptance_common/*.rs` + test files | ~12 |
| TOTAL | | ~43 |

**Serde rename-aware deserialization** absorbs the wire-format
back-compat cost. The renamed variant carries:

```rust
#[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]
UsesProfile { id: EdgeId, from: AgentId, to: NodeId },
```

so EXISTING serialized edge rows stored as JSON with `"edge": "HasProfile"`
in SurrealDB deserialize cleanly into the renamed variant during the
0020 backfill window. After 0020 backfill rewrites all existing rows to
the new edge-name, the `alias` can stay (no harm) or be removed in a
future cycle.

**Protected sites that do NOT rename**:

| Site | Reason |
|---|---|
| `audit/events/m4/agents.rs:78` JSON key `"has_profile"` | snake_case wire-key for the m4 agent-creation audit-event, NOT the edge variant name; protected per [ADR-0033 §D33.5](../../m5_2/decisions/0033-k8s-prep-refactors.md) hash-chain canonical-bytes stability |
| `migrations/0001_initial.surql:314` source-comment + `migrations/0005_*.surql:182` reference | migration files are immutable post-apply; migration 0019 ADDs the new table `DEFINE TABLE uses_profile TYPE RELATION FROM agent TO agent_profile` |

**Audit-event semantic rename** ([ADR-0063 §D63.7](../decisions/0063-agent-profile-cardinality-n-to-1.md)):
`DomainEvent::HasProfileEdgeChanged` renames to
`DomainEvent::UsesProfileEdgeChanged`. The kind-string changes from
`has_profile_edge_changed` → `uses_profile_edge_changed`. Pre-rename
audit-event rows remain valid in the hash-chain (each row chains from
the prev regardless of kind-name); post-rename rows have the new
kind-name in their canonical-bytes — hash-chain is byte-stable per
ADR-0033 §D33.5.

**Rationale citation**: [ADR-0063 §D63.2](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §5 — Split-migration walkthrough (0019 schema + 0020 backfill)

The F3.b user-locked design splits the migration into two atomic units:

### Migration 0019 — schema-only

[`modules/crates/store/migrations/0019_agent_profile_n_to_1_schema.surql`](../../../../../../modules/crates/store/migrations/) — body:

1. `REMOVE INDEX IF EXISTS agent_profile_agent_id ON TABLE agent_profile` (drop UNIQUE constraint that enforced 1:1).
2. `DEFINE TABLE blueprint SCHEMAFULL` + the 6 fields (`id`, `agent_id`, `parallelize`, `model_config_id`, `mock_response`, `created_at`).
3. `DEFINE INDEX blueprint_agent_id_unique ON TABLE blueprint FIELDS agent_id UNIQUE WHERE agent_id != NONE` (UNIQUE on override-row keying only).
4. `DEFINE TABLE uses_profile TYPE RELATION FROM agent TO agent_profile` (renamed from `has_profile`).
5. `DEFINE TABLE agent_profile_uses_blueprint TYPE RELATION FROM agent_profile TO blueprint` (NEW template pointer).
6. `DEFINE TABLE agent_uses_blueprint_override TYPE RELATION FROM agent TO blueprint` (NEW per-agent override pointer).
7. `ALTER TABLE agent_profile REMOVE FIELD parallelize, model_config_id, mock_response` (override-field migration: data migration in 0020 preserves the values via the template Blueprint).

All DDL uses `IF [NOT] EXISTS` so re-applying 0019 on a post-apply database is a no-op (idempotency contract per [ADR-0012](../../m1/decisions/0012-forward-only-migrations.md)).

### Migration 0020 — data backfill

[`modules/crates/store/migrations/0020_agent_profile_n_to_1_backfill.surql`](../../../../../../modules/crates/store/migrations/) — body:

1. For each existing `agent_profile` row: `CREATE blueprint:<template_id> SET agent_id = NONE, parallelize = ..., model_config_id = ..., mock_response = ...` (copy the pre-0019 override values onto a NEW template `blueprint` row).
2. `RELATE` the existing `agent_profile` row to the new template `blueprint` via `agent_profile_uses_blueprint`.
3. For each existing `has_profile` edge row: `RELATE` the corresponding `agent → agent_profile` via `uses_profile`; `DELETE` the old `has_profile` row.
4. **Per-agent override rows are NOT created at backfill.** Each agent inherits its template's defaults; per-agent overrides are written lazily by application code when an agent's profile is mutated.

**Idempotency**: re-running 0020 on a post-apply database is a no-op
(template `blueprint` rows already exist; `has_profile` rows already
removed; `uses_profile` rows already created). Each backfill statement
is guarded by an `IF NOT EXISTS` / `WHERE NOT EXISTS` clause.

### Split-migration pattern (NEW precedent — codified at CH-28)

This is the **first split-migration in the baby-phi workspace** and
introduces a NEW reusable precedent for future cycles facing the
"operator wants inspection window between schema and data" decision:

| When to split | When to ship single composite |
|---|---|
| Schema change is load-bearing AND data backfill is non-trivial | Schema + data fit in one ≤ 200 LOC migration |
| Operator wants to inspect schema between schema and data | Schema + data are atomically coupled |
| Rollback semantic is cleaner with two units (rollback 0020 leaves 0019 schema intact) | Rollback is unlikely to be needed at finer granularity than the whole migration |

The split-migration pattern is documented in
[`m1/architecture/schema-migrations.md`](../../m1/architecture/schema-migrations.md)
§"Split-migration pattern (NEW at CH-28)". Future cycles facing the same
decision cite CH-28 / [ADR-0063 §D63.3](../decisions/0063-agent-profile-cardinality-n-to-1.md)
as precedent.

### Half-migrated-state runtime tolerance

Between 0019 first-apply and 0020 first-apply (the operator-inspection
window), the runtime read of `get_agent_profile_for_agent` must tolerate
either (a) pre-0020 shape (override fields STILL on `agent_profile`
row) OR (b) post-0020 shape (template `blueprint` row + override
`blueprint` rows). The implementer ships a runtime-level helper
`read_agent_profile_via_blueprint_or_fallback` at
`store::repo_impl::SurrealStore` + `domain::in_memory::InMemoryRepository`
that tries the new path first + falls back to the old path if the
blueprint table is empty. This helper is shipped at CH-28 and removed in
a future M7 NFR-observability cycle once the migration is universally
applied. See [ADR-0063 §D63.12](../decisions/0063-agent-profile-cardinality-n-to-1.md).

**Rationale citation**: [ADR-0063 §D63.3 + §D63.5 + §D63.12](../decisions/0063-agent-profile-cardinality-n-to-1.md).

---

## §6 — Pre-CH-28 vs post-CH-28 graph comparison

### Pre-CH-28 (M5 / CH-01 model)

```
   ┌─────────┐                              ┌──────────────────────┐
   │  Agent  │ ───HAS_PROFILE (1:1)──────▶ │   AgentProfile       │
   │ (id A1) │                              │ (id P1)              │
   │         │                              │   parallelize: 4     │
   │         │                              │   model_config_id: M1│
   │         │                              │   mock_response: ... │
   └─────────┘                              │   blueprint:         │
                                            │     phi_core::AgentProfile│
                                            └──────────────────────┘

   ┌─────────┐                              ┌──────────────────────┐
   │  Agent  │ ───HAS_PROFILE (1:1)──────▶ │   AgentProfile       │
   │ (id A2) │                              │ (id P2, identical    │
   │         │                              │    fields as P1)     │
   └─────────┘                              └──────────────────────┘

   2 agents = 2 AgentProfile rows. Sharing the same role-shape
   creates duplicate AgentProfile rows that must be kept in sync
   by update-time discipline.
```

### Post-CH-28 (M6 / CH-28 model)

```
                                  ┌──────────────────────────────┐
                                  │  blueprint (template-row)    │
                              ┌──▶│  (id BT1)                    │
                              │   │    agent_id: NONE            │
                              │   │    parallelize: 4 (default)  │
                              │   │    model_config_id: M1       │
                              │   │    mock_response: ...        │
                              │   └──────────────────────────────┘
                              │            ▲
                              │            │ AGENT_PROFILE_USES_BLUEPRINT
                              │            │
   ┌─────────┐    ┌─────────────────────────────────────┐
   │  Agent  │ ──▶│ AgentProfile (id P1, "intern_coder") │
   │ (id A1) │    │   blueprint_id: BT1 (template)       │
   │         │    └──────────────────────────────────────┘
   │         │           ▲
   │         │           │ USES_PROFILE (N:1)
   │         │           │
   ┌─────────┐           │
   │  Agent  │ ──────────┘
   │ (id A2) │
   └─────────┘

   2 agents share 1 AgentProfile (template). The shared template
   points at 1 template Blueprint row carrying default override values.

   If Agent A2 needs an override (e.g. parallelize: 8 specifically
   for A2 only):

                                  ┌──────────────────────────────┐
                                  │  blueprint (override-row)    │
                              ┌──▶│  (id BO_A2)                  │
                              │   │    agent_id: A2              │
                              │   │    parallelize: 8 (override) │
                              │   │    model_config_id: NONE     │
                              │   │    mock_response: NONE       │
                              │   └──────────────────────────────┘
                              │            ▲
                              │            │ AGENT_USES_BLUEPRINT_OVERRIDE
                              │            │  (zero-or-one)
                              │            │
   ┌─────────┐                 │
   │  Agent  │ ────────────────┘
   │ (id A2) │
   └─────────┘

   Effective override value for A2 = override-row.parallelize (8);
   model_config_id + mock_response fall back to template-row defaults.
   Effective override values for A1 = all from template-row.
```

**Net** (per the graph comparison): 1 fewer `AgentProfile` row per
sharing-cluster (the duplicate-template case) + 1 NEW template Blueprint
row per template + 0 NEW override Blueprint rows by default (lazy
creation on first override). Storage cost scales with **distinct
templates** (not distinct agents); update cost scales with **template
re-authoring** (not agent count).

---

## §7 — References

### ADRs

- **[ADR-0063 — AgentProfile cardinality 1:1 → N:1 redesign](../decisions/0063-agent-profile-cardinality-n-to-1.md)** (this chunk's authoritative decision; 12 sub-decisions §D63.1–§D63.12)
- [ADR-0034 §D34.6](../../m5_2/decisions/0034-agent-durable-lifecycle.md) — wrap-vs-runtime separation; preserved at ADR-0063 §D63.8
- [ADR-0033 §D33.5](../../m5_2/decisions/0033-k8s-prep-refactors.md) — audit hash-chain symmetry; preserved at ADR-0063 §D63.7
- [ADR-0057 §D57.7](../../m5_2/decisions/0057-bucket-b-convention-ratification.md) — EDGE_KIND_NAMES cardinality invariant; bumped 72 → 74 under F1.c at ADR-0063 §D63.10
- [ADR-0038 §D38.1](../../m5_2/decisions/0038-identity-node-materialization.md) — eager creation pattern at `apply_agent_creation` (carry-forward)
- [ADR-0012](../../m1/decisions/0012-forward-only-migrations.md) — migration runner + idempotency contract (carry-forward; split-migration adds precedent)

### Concept docs

- [`concepts/agent.md`](../../../concepts/agent.md) §"Soul (Template Blueprint + per-agent overrides)" lines 160–178 + §"Parallelized Sessions" lines 209–221
- [`concepts/ontology.md`](../../../concepts/ontology.md) §"Edge Types (74 total)" + §"Agent-Centric (first-order)" line 98

### Forward-scope + planning artifacts

- `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 41–48 (CH-28 row)
- `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §3 lines 356–370 (M6+-OPEN-01 origin marker; CH-28 ratifies the redesign-pursuit)
- [Plan archive: `build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md`](../../../../plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md)

### Downstream consumers

- **CH-29 (M6-DEFERRED-02 messaging substrate)** — sender/recipient FK semantic unchanged; Blueprint pointers are orthogonal.
- **CH-36 (a04 My Work / M6-DEFERRED-04 supervisor body)** — supervisor body references template-Blueprint by id + per-agent override-Blueprint by agent-id.
- **CH-37 (a05 My Profile + Grants)** — profile editor renders template-Blueprint (read-only for non-CEO) + per-agent override-Blueprint (writable); `PATCH /api/v0/agents/:id/profile/*` endpoint patches the override row.
- **M7 NFR-observability** — multi-agent audit-event enrichment for `UsesProfileEdgeChanged` + `BlueprintUpserted` (when N agents share a template Blueprint and one is changed); removal of `read_agent_profile_via_blueprint_or_fallback` helper once migration is universally applied.
