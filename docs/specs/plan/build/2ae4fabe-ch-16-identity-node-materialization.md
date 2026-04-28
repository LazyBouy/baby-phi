<!-- Last verified: 2026-04-28 by Claude Code -->

# CH-16 — Identity node materialization

**Plan file token:** `2ae4fabe`.
**Plan archive path (verbatim copy from `/root/.claude/plans/sharded-discovering-stearns.md`):** `baby-phi/docs/specs/plan/build/2ae4fabe-ch-16-identity-node-materialization.md`. Archived at chunk-open Step 0 on 2026-04-28.
**Chunk ID:** CH-16 (forward-scope §1 lines 155–160; §4 dependency edge `CH-02 → CH-15/16/17/21/24` line 314; §5 inventory row line 348).
**Severity:** HIGH.
**Expected effort:** ~3 engineer-days.
**Hard prerequisites:** CH-01 (`Agent.active` / `archived_at` + `AgentKind::Human/Llm` discriminator — DONE), CH-02 (real `agent_loop` + MockProvider for SessionEnded payloads — DONE).
**Chunks unblocked at close:** CH-21 (memory-extraction listener body — gains a real `Repository::upsert_identity` to call when `MemoryExtracted` triggers a reactive update); softer downstream: CH-24 carryover seal (Identity is a v0 commitment that must materialize before M5 tag).

---

## Context

CH-16 closes the largest single concept-vs-code drift in the M5 catalogue:

- `concepts/agent.md` § "Identity (Emergent, Event-Driven)" + § "Identity Node Content — Provisional Direction" labels the four-field Identity node (`self_description` / `lived` / `witnessed` / `embedding`) as **"the v0 commitment — implementations should code against it"**.
- The implementation at [`modules/crates/domain/src/model/nodes.rs:852-857`](baby-phi/modules/crates/domain/src/model/nodes.rs) is `scaffold_node!(Identity, NodeId)` — an id-only stub.

Two governance jobs land in this chunk, physically inseparable:

1. **D-new-01 (HIGH, Bucket A) closure** — materialize the four-field Identity struct, ship migration 0009 with the `identity` table, add 4 repo methods, wire eager creation in `apply_agent_creation` for every LLM agent (default-empty content), define `DomainEvent::IdentityUpdated` for future CH-21 reactive emission.
2. **D-new-23 (LOW, Bucket B) closure** — finalise the Human Agent guard scoped at CH-01: defensive guard at `Repository::upsert_identity` (rejects all callers with `RepositoryError::HumanAgentHasNoIdentity`); preventive guard at `apply_agent_creation` (skips Identity insertion entirely for Human kind).

The two halves are inseparable because: the guard is the protective rail around the writer; shipping the writer without the guard violates `concepts/human-agent.md` lines 16–17 ("*Human Agents do **not** have a system-computed Identity*"); shipping the guard without the writer is meaningless.

**User-decided forks (locked at plan-review via AskUserQuestion):**

1. **Creation timing: EAGER** (in `apply_agent_creation` for every LLM agent). One row per LLM agent matches `ontology.md` line 29's "stored node" framing and removes "row absent vs row present-but-empty" ambiguity.
2. **Embedding init: EMPTY `Vec<f32>`**. Concept-agent.md ties embedding to `self_description`; with self_description empty at create, any placeholder embedding is wasted compute. Population deferred to M6.
3. **Update signalling: NEW `DomainEvent::IdentityUpdated`** + `IdentityUpdateTrigger` enum. Variant + bus dispatch ship in CH-16; first production emitter lands at CH-21. Matches ADR-0028 fail-safe semantics.
4. **Archive policy: LEAVE Identity queryable**. Archive flips `Agent.active = false` but the row stays. Supports forensic/hiring/evaluation queries. `delete_identity` exists for symmetry but is operator-driven only (no handler calls it).
5. **Human Agent guard: BOTH** (defensive at repo + preventive at call site).
6. **Initial `self_description`: EMPTY string**. Concept-agent.md line 325 commits to agent-authored content; blueprint copy would be a stale snapshot.

**Outcome:** D-new-01 + D-new-23 → `remediated`; concept-`agent.md` § "Identity (Emergent)" + § "Identity Node Content" + § "Two Streams of Experience" + § "Materialization" + concept-`ontology.md` § "Node Types — Identity" + concept-`human-agent.md` § "No Identity" all flip to `honored`; downstream CH-21 unblocks; M5 v0-commitment count flips from 1-violated to 0-violated.

---

## §1 — Context & principle

### Why this chunk

CH-16 is the materialization of a v0 commitment. The four-field shape in `agent.md` IS the spec — not three-fields-with-embedding-deferred, not "store self_description as a column on Agent". The `LivedExperience` and `WitnessedExperience` structs have field names and types pinned in `ontology.md` lines 24–29 and `agent.md` lines 326–328; the migration must match those names byte-for-byte.

The Human Agent guard is not an afterthought — `human-agent.md` line 16 spends a full paragraph explaining *why* a human's identity exists outside the system; shipping a writer without the guard reproduces the silent-drift pattern the chunk discipline forbids.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* Applied: every concept-doc field name is mirrored verbatim in the struct + migration; the guard fires at TWO layers (defensive + preventive) so a future direct-repo caller cannot bypass the concept invariant.

### Forward-scope reference

[§1 CH-16 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 155–160) + [§4 critical-path graph](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 314 — `CH-02 → CH-15/16/17/21/24`) + [§5 inventory row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 348).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Identity (Emergent, Event-Driven)" lines 266–315 | Identity is a stored node, updated reactively. **LLM agents only.** | contradicted (scaffold-only) | honored |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Identity Node Content" lines 317–344 | Four fields: `self_description` (≤500 tokens, agent-authored), `lived: LivedExperience`, `witnessed: WitnessedExperience`, `embedding: Vec<f32>`. | contradicted | honored |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Two Streams of Experience" lines 276–304 | LivedExperience: sessions_completed/ratings_window/skills/specializations. WitnessedExperience: memories_extracted/subordinates_observed/extraction_scope_distribution. | silent-in-code | honored (struct fields named per concept) |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Materialization" lines 308–315 | "Not computed from scratch each time — incrementally updated by the triggering event"; "Queryable: returns the materialized node". | contradicted | honored (eager create + reactive `IdentityUpdated` event) |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Scoping the embedding model" line 338 | Embedding model is platform-level config fixed at org-bootstrap. | silent-in-code | preserved silent-in-code; deferred to **M6-DEFERRED-03 (NEW)** per ADR-0038 §D38.3 + § Out-of-Scope. CH-16 ships a placeholder `embedding_dim: Option<u32>` carrier. |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § "Model change is an admin event" line 340 | Switching providers triggers batch re-embed. | silent-in-code | preserved silent-in-code; documented as out-of-scope in ADR-0038 § Out-of-Scope. |
| [`human-agent.md`](baby-phi/docs/specs/v0/concepts/human-agent.md) | § "No Identity" lines 16–17 | Human Agents do **not** have a system-computed Identity node. | silent-in-code (no guard) | honored (defensive guard at `upsert_identity` + preventive guard at `apply_agent_creation`) |
| [`ontology.md`](baby-phi/docs/specs/v0/concepts/ontology.md) | § "Node Types — Core Identity" lines 24–29 | Identity carries the 4-field shape; updated reactively; LLM Agents only. | contradicted | honored |
| [`README.md`](baby-phi/docs/specs/v0/concepts/README.md) | (entry invariants) | Concepts subtree invariants. | honored | honored (re-verified post-materialization) |
| [`phi-core-mapping.md`](baby-phi/docs/specs/v0/concepts/phi-core-mapping.md) | (phi-core surfaces) | No phi-core overlap for Identity / LivedExperience / WitnessedExperience. | honored | honored (declared in §3; phi-core has no Identity tier) |

---

## §3 — phi-core leverage map

| phi-core type | Current handling | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Rationale:** Identity / `LivedExperience` / `WitnessedExperience` / Identity-`embedding` are phi-only governance primitives. phi-core ships no Identity tier. Closest neighbour `phi_core::session::model::Session` is execution telemetry — orthogonal per `baby-phi/CLAUDE.md` § "Orthogonal surfaces that are NOT phi-core duplicates".

**Expected import-count delta at chunk close:** **0 phi-core imports added or removed.**

**Positive close-audit greps** (must pass at seal):
```bash
grep -n "pub struct Identity\b" modules/crates/domain/src/model/nodes.rs                       # 1
grep -n "pub struct LivedExperience\b" modules/crates/domain/src/model/nodes.rs                # 1
grep -n "pub struct WitnessedExperience\b" modules/crates/domain/src/model/nodes.rs            # 1
grep -n "fn upsert_identity\b\|fn get_identity\b\|fn delete_identity\b\|fn list_identities_for_org\b" \
  modules/crates/domain/src/repository.rs                                                       # 4
grep -n "DomainEvent::IdentityUpdated\b\|IdentityUpdateTrigger\b" modules/crates/domain/src/events/mod.rs  # ≥ 3
grep -n "HumanAgentHasNoIdentity" modules/crates/domain/src/repository.rs                      # ≥ 1
ls modules/crates/store/migrations/0009_identity_node.surql                                    # exists
```

**Forbidden-duplication greps** (must return 0):
```bash
grep -rn "use phi_core::" modules/crates/domain/src/model/nodes.rs | grep -i identity         # 0
grep -rn "^pub struct Identity\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"        # 0
bash scripts/check-phi-core-reuse.sh                                                            # exit 0
grep -rn "scaffold_node!.*Identity" modules/crates/domain/src/model/nodes.rs                   # 0 (scaffold removed)
```

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker? |
|---|---|---|---|
| **A1** in-process state | `Identity` is a SurrealDB row; `LivedExperience` / `WitnessedExperience` are inline structs serialized into the row. No `OnceCell`/`Mutex`/process-globals. The future platform-level embedding model config is per-pod read of bootstrap config, not in-process mutable state. | No |
| **A2** IPC channels | `DomainEvent::IdentityUpdated` rides existing `EventBus`. No new IPC. | No |
| **A3** pod-local resources | None. `Vec<f32>` is in-row; not file-backed. | No |
| **A4** migration runner / first-apply race | Migration **0009** is additive (new `identity` table + UNIQUE-on-`agent_id` index). Cross-ref existing CHK8S-D-05 (lock-missing). Additive table-add migrations are not aggravated. | No (existing concern preserved) |
| **A5** trait-shape requirement | 4 new methods ride existing `Repository` trait. Object-safe. Future remote/Redis-backed Identity store drops in by implementing the trait. | No (trait-shaped from day one) |
| **A6** cross-pod state sharing | Identity rows in durable SurrealDB; visible across pods via `SurrealStore::open_remote`. No in-process cache. `IdentityUpdated` event fan-out across pods rides whatever EventBus the M7b broker carve-out lands. | No |
| **A7** audit hash-chain symmetry | New audit event types: `platform.identity.created` (Alerted) + `platform.identity.updated` (Routine). Both go through existing `AuditEmitter::emit` → `prev_event_hash` chain. | No |

**Conforming criteria for ADR-0033 (CH-K8S-PREP) preserved.** **Conclusion: K8s-neutral.** Ledger stays at 8 (same baseline post-CH-06).

---

## §3.C — User-facing documentation impact map (post-Q9 / CH-22 binding)

| Tier | File | Touched? | Action |
|---|---|---|---|
| Architecture | `m5_2/architecture/identity-node.md` (NEW) | yes — design page: 4-field shape, eager-create timing, `IdentityUpdated` event, struct field reference, embedding-dim deferral note | (a) create in-chunk |
| Architecture | [`m1/architecture/graph-model.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/graph-model.md) | yes — Identity node section currently shows scaffold-only; rewrite to 4-field struct + `HAS_IDENTITY` edge + `agent_id` UNIQUE index | (a) update in-chunk |
| Architecture | [`m1/architecture/audit-events.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/audit-events.md) | yes — add `platform.identity.created` (Alerted) + `platform.identity.updated` (Routine) rows | (a) update in-chunk |
| Operations | `m5_2/operations/identity-operations.md` (NEW) | yes — runbook: orphan identity rows (LLM archived without identity delete — soft); embedding re-population batch (deferred); `HumanAgentHasNoIdentity` operator action | (a) create in-chunk |
| Operations | [`m5/operations/agent-creation-runbook.md`](baby-phi/docs/specs/v0/implementation/m5/operations/agent-creation-runbook.md) (verify exists) | yes if exists — note that LLM-kind agent creation now writes identity row in same compound tx; Human-kind unchanged | (a) update in-chunk if file present; otherwise (b) defer with successor `CH-19` |
| User-guide | `m5_2/user-guide/identity-overview.md` (NEW) | yes — operator-facing: what Identity is, why LLM-only, how `self_description` gets written by the agent over time, what the four fields mean for hiring/evaluation queries | (a) create in-chunk |
| User-guide | [`m5/user-guide/troubleshooting.md`](baby-phi/docs/specs/v0/implementation/m5/user-guide/troubleshooting.md) | yes — add "Tried to write Identity to a Human Agent" entry mapping `HumanAgentHasNoIdentity` to operator action | (a) update in-chunk |

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D-new-01** | [`m5_1/drifts/D-new-01.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-01.md) | HIGH | `discovered → in-chunk-plan → remediated` | Identity struct shipped with 4 concept-mandated fields + governance fields; migration 0009 adds `identity` table with UNIQUE-on-`agent_id`; eager creation in `apply_agent_creation` for every LLM agent; 4 repo methods + `DomainEvent::IdentityUpdated` variant. |
| **D-new-23** | [`m5_1/drifts/D-new-23.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-23.md) | LOW | `scoped → in-chunk-plan → remediated` | Defensive guard at `Repository::upsert_identity` + preventive guard at `apply_agent_creation` skip; 3 unit tests pin (defensive rejection, preventive skip, list excludes Human kind). |

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — D-new-01 + D-new-23 row Status → `remediated`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip 3 rows: "Identity (Emergent)" 4-field (`contradicted → honored`); "No Identity" Human guard (`silent-in-code → honored`); "Identity ontology" 4-field shape (`contradicted → honored`).

**Mid-flight discovery hook:** if a phase reveals a third drift (e.g., `AgentArchived` event handler must also delete Identity but currently has no path; or a server-side handler instantiates a `LivedExperience` literal that would silently crash on the new schema), surface via `AskUserQuestion` and add a row before phase close.

---

## §5 — ADRs drafted

ADR numbering check (run at draft time): `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE '^[0-9]{4}' | sort -u | tail -5` → currently `0033 0034 0035 0036 0037` → **next free = 0038**, then 0039.

| ADR | Title | Drafted at phase | Decision summary | Flip-to-Accepted phase |
|---|---|---|---|---|
| **ADR-0038** | Identity node materialized as 4-field struct; eager creation per LLM agent in `apply_agent_creation`; embedding population deferred | P1 | **D38.1** Identity is a full struct (Fork 1 EAGER); one row per LLM agent inside `apply_agent_creation` compound tx (atomic with Agent+Inbox+Outbox). **D38.2** Four fields per concept-agent.md lines 323–328: `self_description: String` (default `""`), `lived: LivedExperience` (default zeroed), `witnessed: WitnessedExperience` (default zeroed), `embedding: Vec<f32>` (default `vec![]`). **D38.3** Embedding population deferred (Fork 2 EMPTY); platform-level model config does not exist at v0.1; placeholder `embedding_dim: Option<u32>` carrier reserved on `OrganizationDefaultsSnapshot` (#[serde(default)]); deferred to **M6-DEFERRED-03** (NEW). **D38.4** `LivedExperience` / `WitnessedExperience` are inline structs (not separate node types) — they are query-shape detail of the Identity row, not first-class nodes. **D38.5** `IdentityUpdated` `DomainEvent` variant ships with bus dispatch arms (Fork 3) but **no production emitter** (CH-21 lights the first emitter). **D38.6** Archive does NOT delete Identity (Fork 4 LEAVE QUERYABLE); `delete_identity` shipped for symmetry but operator-driven only. **D38.7** Identity does not carry a `tags` field — keyed-by-`agent_id`, never a permission-check selector target (cross-ref ADR-0037 §D37.3). | Chunk seal (P3) |
| **ADR-0039** | Human Agent Identity guard: defensive (Repository) + preventive (call-site) | P2 | **D39.1** Fork 5 BOTH guards. Defensive at `Repository::upsert_identity` returns typed `RepositoryError::HumanAgentHasNoIdentity { agent_id }` (NEW variant). **D39.2** Preventive at `apply_agent_creation` matches `payload.agent.kind` and skips Identity insertion entirely for Human kind. **D39.3** 3 unit tests pin: defensive rejection, preventive skip, `list_identities_for_org` excludes Human kind. **D39.4** No migration to clean up pre-existing Human Identity rows (none can exist — D-new-01 means Identity has zero rows pre-CH-16). **D39.5** Concept-`human-agent.md` § "No Identity" stays at "external to system" framing — guard is enforcement, not concept change. | Chunk seal (P3) |

ADR file paths:
- [`m5_2/decisions/0038-identity-node-materialization.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0038-identity-node-materialization.md)
- [`m5_2/decisions/0039-human-agent-identity-guard.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0039-human-agent-identity-guard.md)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant relied on | Re-verification command |
|---|---|---|
| Post-CH-06 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1067 (CH-06 shipped ~52 new) | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` |
| CH-01 / ADR-0034 | `Agent.kind: AgentKind` discriminator + `Agent.active`/`archived_at` columns | `grep -n "AgentKind::Human\|AgentKind::Llm" modules/crates/domain/src/model/nodes.rs` ≥ 4 |
| CH-02 / ADR-0032 | `BabyPhiSessionRecorder` + `DomainEvent::SessionEnded` payload (CH-21 will consume; CH-16 verifies shape) | `grep -n "SessionEnded {" modules/crates/domain/src/events/mod.rs` ≥ 1 |
| CH-06 / ADR-0036 + 0037 | Selector grammar + instance identity tags. Identity is NOT in CH-06 tag-emission list (ADR-0037 §D37.3); ADR-0038 §D38.7 carries forward. | `grep -n "tags" modules/crates/domain/src/model/nodes.rs` (Identity struct should not carry the field) |
| CH-22 / ADR-0035 | `AgentCatalogListener::on_event` reads `Agent` directly. CH-16 adds Identity creation in same compound tx but does NOT touch catalog shape or listener body. | `cargo test -p domain --lib events::listeners::tests` (15+ catalog tests) |
| M1 permission-check spine | `apply_agent_creation` compound tx is still atomic — all rows commit or none. | `cargo test -p store --test apply_agent_creation_tx_test` (4 existing scenarios + 1 NEW LLM-with-identity scenario) |
| All chunks | 4 CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` |

---

## §7 — Phases within the chunk

**Phase count: 3** → audit envelope = **2 agents** (medium chunk per per-chunk-template guardrail; matches CH-22 + CH-06 precedent).

### P1 — Identity struct + migration 0009 + repo methods + Human guard (~1.5d)

**Goal.** Land the four-field Identity struct, ship migration 0009, implement the 4 repo methods on both `InMemoryRepository` and `SurrealRepository`, add `RepositoryError::HumanAgentHasNoIdentity` + defensive guard, define `DomainEvent::IdentityUpdated` variant.

**Deliverables.**

1. **Concept-aligned struct definitions** in `modules/crates/domain/src/model/nodes.rs` — replace `scaffold_node!(Identity, NodeId)` (line 852–857) with full 4-field struct + supporting types:
   ```rust
   pub struct Identity {
       pub id: NodeId,
       pub agent_id: AgentId,            // UNIQUE per migration 0009; one row per LLM agent
       #[serde(default)] pub self_description: String,
       #[serde(default)] pub lived: LivedExperience,
       #[serde(default)] pub witnessed: WitnessedExperience,
       #[serde(default)] pub embedding: Vec<f32>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,    // bumped on every reactive update
   }

   pub struct LivedExperience {
       sessions_completed: u64, sessions_successful: u64,
       ratings_window: Vec<RatingPoint>, skills: Vec<SkillRef>,
       specializations: Vec<String>,
   }

   pub struct WitnessedExperience {
       memories_extracted: u64,
       subordinates_observed: Vec<AgentId>,
       extraction_scope_distribution: ExtractionScopeDistribution,
   }

   pub struct ExtractionScopeDistribution { private: u64, public: u64 }
   pub struct RatingPoint { rater: AgentId, score: f32, at: DateTime<Utc> }
   pub struct SkillRef { name: String }    // M6 swaps to typed SkillId
   ```
   Plus `Identity::default_for_llm(agent_id, now)` constructor.

2. **Migration** `modules/crates/store/migrations/0009_identity_node.surql`:
   ```sql
   DEFINE TABLE identity SCHEMAFULL;
   DEFINE FIELD agent_id          ON identity TYPE string ASSERT $value != NONE;
   DEFINE FIELD self_description  ON identity TYPE string DEFAULT "";
   DEFINE FIELD lived             ON identity FLEXIBLE TYPE object;
   DEFINE FIELD witnessed         ON identity FLEXIBLE TYPE object;
   DEFINE FIELD embedding         ON identity TYPE array DEFAULT [];
   DEFINE FIELD embedding.*       ON identity TYPE float;
   DEFINE FIELD created_at        ON identity TYPE datetime;
   DEFINE FIELD updated_at        ON identity TYPE datetime;
   DEFINE INDEX identity_agent_id ON identity FIELDS agent_id UNIQUE;
   ```

3. **Repository trait extensions** in `modules/crates/domain/src/repository.rs`:
   ```rust
   async fn upsert_identity(&self, identity: &Identity) -> RepositoryResult<()>;
   async fn get_identity(&self, agent_id: AgentId) -> RepositoryResult<Option<Identity>>;
   async fn delete_identity(&self, agent_id: AgentId) -> RepositoryResult<()>;
   async fn list_identities_for_org(&self, org: OrgId) -> RepositoryResult<Vec<Identity>>;
   ```
   Plus `RepositoryError::HumanAgentHasNoIdentity { agent_id: AgentId }` variant.

4. **`InMemoryRepository` impl** (`domain/src/in_memory.rs`) — adds `identities: RwLock<HashMap<AgentId, Identity>>`; `upsert_identity` reads `Agent` row, checks `kind != Human`, then writes. `get_identity` / `delete_identity` are direct map ops. `list_identities_for_org` joins through `agents` map.

5. **`SurrealRepository` impl** (`store/src/repo_impl.rs`) — `upsert_identity` does `LET $a = type::thing('agent', $aid); IF $a.kind = 'human' { THROW ... }` followed by SurrealDB upsert. Defensive guard rejects Human kind at SQL layer too.

6. **`DomainEvent::IdentityUpdated`** variant + `IdentityUpdateTrigger` enum in `domain/src/events/mod.rs`:
   ```rust
   IdentityUpdated { agent_id: AgentId, trigger: IdentityUpdateTrigger, at: DateTime<Utc>, event_id: AuditEventId },

   pub enum IdentityUpdateTrigger { SessionEnded, MemoryExtracted, SkillChanged, RatingReceived }
   ```
   Plus `kind() -> "identity_updated"` and `event_id()` arms.

**Tests** (~16 unit + 1 migration round-trip):
- 4 struct serde round-trip tests (`Identity`, `LivedExperience`, `WitnessedExperience`, `RatingPoint`).
- 4 default-value tests: `Identity::default_for_llm` produces zero state.
- 5 in-memory-repo tests: upsert + read; upsert-then-update bumps `updated_at`; defensive guard rejects Human; `delete_identity` removes; `list_identities_for_org` filters.
- 1 SurrealDB integration test (`store/tests/identity_repo.rs` NEW): full round-trip + defensive guard.
- 1 migration round-trip test (`store/tests/migration_0009.rs` NEW).
- 1 `IdentityUpdated` round-trip test in `events::mod::tests`.

**User-facing doc updates.** Land `m5_2/architecture/identity-node.md` (NEW) + update `m1/architecture/graph-model.md` Identity row + `m1/architecture/audit-events.md` (2 new event-type rows).

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if SurrealDB rejects `FLEXIBLE TYPE object` inside SCHEMAFULL with field-default pattern; or migration 0009 collides with future migration on same table; or `RatingPoint`/`SkillRef` shape needs richer fields than the v0.1 placeholder.

---

### P2 — Eager creation in `apply_agent_creation` + audit emitters + archive policy ratification (~1.0d)

**Goal.** Wire eager Identity row creation into existing `apply_agent_creation` compound tx for `AgentKind::Llm`; add audit-event emitter helpers; preventive guard skips for Human kind; ratify orphan-on-archive policy.

**Deliverables.**

1. **`AgentCreationPayload` extension** (`domain/src/repository.rs`):
   ```rust
   pub struct AgentCreationPayload {
       // ... existing fields ...
       /// Required Some(_) for Llm kind; MUST be None for Human kind.
       pub identity: Option<Identity>,
   }
   ```

2. **`apply_agent_creation` extension** (in_memory.rs + repo_impl.rs):
   - Validate: Human-kind with `identity: Some(_)` → `Err(HumanAgentHasNoIdentity)`.
   - SurrealQL: extend BEGIN TRANSACTION block with `CREATE type::thing('identity', $idid) CONTENT $identity_body` + `RELATE $a -> has_identity -> $iden` if `payload.identity.is_some()`.
   - Receipt: extend `AgentCreationReceipt` with `pub identity_id: Option<NodeId>`.

3. **Server orchestrator wiring**:
   - `server/src/platform/agents/create.rs:155-171` — build `Identity::default_for_llm(agent_id, now)` only for `AgentKind::Llm`.
   - `server/src/platform/system_agents/add.rs` — system agents are LLM-kind (Identity always built).
   - `server/src/bootstrap/claim.rs` — CEO is Human-kind (Identity stays None).

4. **Audit emitter helpers** in NEW `modules/crates/domain/src/audit/events/m5_2/identity.rs`:
   ```rust
   /// `platform.identity.created` — Alerted class; emitted post-commit.
   pub fn identity_created(actor, identity, org, provenance_ar_id, now) -> AuditEvent;

   /// `platform.identity.updated` — Routine class; CH-16 ships helper, CH-21 calls it.
   pub fn identity_updated(actor, before, after, trigger, org, now) -> AuditEvent;
   ```

5. **Production emitter** at `agents/create.rs:235` (after existing `agent_created` emit): emit `identity_created` only when `receipt.identity_id.is_some()`. Same in `system_agents/add.rs`.

6. **Archive policy ratification** — verify `server/src/platform/agents/archive.rs` does NOT call `delete_identity`. ADR-0038 §D38.6 binds: archive flips `Agent.active = false` but identity row stays. One regression test pins: archive an LLM agent → `get_identity(agent_id)` still returns `Some(_)`.

**Tests** (~10):
- 4 `apply_agent_creation` extensions in `apply_agent_creation_tx_test.rs`: Llm-with-identity (commits all); Llm-without-identity (rejected); Human-with-identity (rejected with `HumanAgentHasNoIdentity`); Human-without-identity (commits; identity table empty for that agent_id).
- 2 server-handler tests in `server/tests/agents_create_identity.rs` (NEW): LLM agent → `get_identity` returns `Some` with default fields; Human agent → returns `None`.
- 2 audit-emitter unit tests: event_type + audit_class correctness for both helpers.
- 2 archive-orphan tests: archive LLM agent → identity still queryable; explicit `delete_identity` after archive → row gone.

**User-facing doc updates.** Land `m5_2/operations/identity-operations.md` (NEW) including orphan-on-archive operator note + update `m5/operations/agent-creation-runbook.md` (if file exists).

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if a test fixture in `acceptance_common/admin.rs` builds `AgentCreationPayload` for an LLM agent without Identity (would fail validator); or `bootstrap/claim.rs` turns out to create non-Human bootstrap agent path; or `system_agents/add.rs` creates Human-kind system agent.

---

### P3 — Acceptance + ADR + drift closure + concept-doc bumps + audit + seal (~0.5d)

**Goal.** End-to-end HTTP-driven acceptance through Agent creation. Ratify ADR-0038 + ADR-0039. Close D-new-01 + D-new-23 terminally. Apply §3.C user-facing doc map. Spawn 2 audits. Seal.

**Deliverables.**

1. **Acceptance test** at `modules/crates/server/tests/identity_materialization_acceptance.rs` (NEW). 6 scenarios:
   - Create LLM agent → Identity row exists with 4 default fields populated.
   - Create Human agent → Identity row absent.
   - Direct repo `upsert_identity` for Human kind → `HumanAgentHasNoIdentity` (defensive guard).
   - List identities for org → one entry per LLM agent; no Human-kind entries.
   - Round-trip serde of `Identity` with non-trivial `lived`/`witnessed`.
   - Audit chain: `platform.agent.created` precedes `platform.identity.created` in same org_scope hash chain.

2. **ADR-0038 + ADR-0039** flipped from `Proposed` → `Accepted`.

3. **D-new-01 + D-new-23** lifecycle entries appended; Status flipped to `remediated`.

4. **`drifts/README.md`** — both row Statuses flipped.

5. **`_concept-audit-matrix.md`** — 3 rows flipped.

6. **§3.C user-facing-doc map applied** — 7 file actions:
   - `m5_2/architecture/identity-node.md` (NEW)
   - `m1/architecture/graph-model.md` (Identity row update)
   - `m1/architecture/audit-events.md` (2 new event-type rows)
   - `m5_2/operations/identity-operations.md` (NEW)
   - `m5/operations/agent-creation-runbook.md` (LLM-kind note, if exists)
   - `m5_2/user-guide/identity-overview.md` (NEW)
   - `m5/user-guide/troubleshooting.md` (`HumanAgentHasNoIdentity` entry)

7. **Concept-doc verified-headers bumped** on `concepts/agent.md`, `concepts/human-agent.md`, `concepts/ontology.md`.

8. **Spawn 2 audit agents** per §11.

**Tests.** All §8 named tests green; full workspace passes ~1067 + ~30 = ~1097.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** PAUSE if either audit reports a finding — surface to user before seal.

---

## §8 — Tests summary

- **Expected total at chunk close:** 1067 (post-CH-06 baseline) + ~30 new tests = **~1097 serialised tests**.
- **Layer breakdown:**
  - Unit (`nodes.rs::tests`): ~8 (struct round-trips + defaults)
  - Unit (`in_memory.rs::tests`): ~5 (CRUD + Human guard)
  - Unit (`audit/events/m5_2/identity.rs::tests`): 2 (emitter helpers)
  - Unit (`events::mod::tests`): 1 (`IdentityUpdated` round-trip)
  - Integration (`store/tests/identity_repo.rs` NEW): 2
  - Integration (`store/tests/migration_0009.rs` NEW): 1
  - Integration (`store/tests/apply_agent_creation_tx_test.rs` extensions): 4
  - Integration (`server/tests/agents_create_identity.rs` NEW): 2
  - Acceptance (`server/tests/identity_materialization_acceptance.rs` NEW): 6

- **New test files:**
  - `modules/crates/store/tests/identity_repo.rs`
  - `modules/crates/store/tests/migration_0009.rs`
  - `modules/crates/server/tests/agents_create_identity.rs`
  - `modules/crates/server/tests/identity_materialization_acceptance.rs`
  - `modules/crates/domain/src/audit/events/m5_2/identity.rs` (with mod tests)

- **Expected-still-green fragile tests:**
  - `apply_agent_creation_tx_test.rs::*` — `AgentCreationPayload` gains `identity: Option<Identity>` field; `#[serde(default)]` covers it.
  - `events::listeners::tests::*` — `IdentityUpdated` adds variant; non-matching arms must be unreachable per CH-22's `if let` guard pattern.
  - `acceptance_sessions_m5p4.rs` — CH-02 sessions orthogonal.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive this plan verbatim (mandatory first action)

1. Generate plan-file token: `openssl rand -hex 4`.
2. **Copy this plan file verbatim**: `cp /root/.claude/plans/sharded-discovering-stearns.md baby-phi/docs/specs/plan/build/<8hex>-ch-16-identity-node-materialization.md`. No edits during the copy.
3. Update placeholders in lines 5–6 of the archived plan only.
4. Run `bash scripts/check-doc-links.sh` to confirm relative links resolve (per CH-22 + CH-06 precedent: 6 `..` for paths into `baby-phi/`; 7 `..` for paths into `phi-core/`).
5. Verify: `head -5 baby-phi/docs/specs/plan/build/<8hex>-ch-16-*.md` shows verified-by-Claude-Code header on line 1.
6. Only AFTER successful archive does the rest of §9 run.

This step matches the chunk-lifecycle-checklist Step 1 and the precedent set by CH-01 (`2aa37c80`), CH-02 (`16fd9a3a`), CH-22 (`c5f201bb`), and CH-06 (`acd383e2`).

**Reading list (mandatory before continuing):**
1. `concepts/agent.md` § "Identity (Emergent, Event-Driven)" lines 266–345 (full).
2. `concepts/human-agent.md` § "No Identity" lines 16–17.
3. `concepts/ontology.md` § "Node Types — Core Identity" lines 24–29.
4. `concepts/README.md` (entry invariants).
5. `concepts/phi-core-mapping.md` (verify no Identity entries).
6. `concepts/token-economy.md` § "Rolling rating window" (for `RatingPoint` shape).
7. Drifts D-new-01 + D-new-23 (full content).
8. CH-01 plan (`2aa37c80`) — for `AgentKind::Human`/`Llm` discriminator + lifecycle.
9. CH-06 plan (`acd383e2`) — for ADR-0037 §D37.3 (Identity-without-tags carry-forward).
10. CH-22 plan (`c5f201bb`) — for `AgentCreationPayload` shape, `apply_agent_creation` extension precedent, UNIQUE-on-`agent_id` index pattern.
11. `forward-scope/22035b2a-...md` §1 CH-16 row + §4 dependency graph + §5 inventory.
12. `baby-phi/CLAUDE.md` phi-core Leverage section.

**Carry-forward invariants (verified green at chunk-open):**
- `cargo test --workspace -- --test-threads=1` ≈ 1067.
- 4 CI guards green.
- D-new-01 status `discovered`; D-new-23 status `scoped`.
- ADR-0034..0037 Accepted.
- `git diff --stat HEAD -- modules/` empty.
- Highest applied migration is 0008.
- Highest issued ADR is 0037.

**User-decided forks (already locked at plan-review):**
- Fork 1 EAGER, Fork 2 EMPTY embedding, Fork 3 NEW IdentityUpdated event, Fork 4 LEAVE QUERYABLE on archive, Fork 5 BOTH guards, Fork 6 EMPTY self_description.

**Cargo command convention** (per user feedback memory): all cargo invocations use `-j 4`. Tests serialise via `--test-threads=1`.

---

## §10 — Close criteria (5-aspect, post-Q9)

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — all P1–P3 deliverables shipped; `cargo test --workspace -- --test-threads=1` green at ~1097; clippy green under `RUSTFLAGS="-Dwarnings"` with `-j 4`; `cargo fmt --all -- --check` green; `identity_materialization_acceptance.rs` 6/6 pass.
- **Docs aspect** — TWO scopes (per Q9):
  - *Governance tier*: D-new-01 + D-new-23 lifecycle entries + Status flips; `_concept-audit-matrix.md` 3 rows flipped; `drifts/README.md` updated; ADR-0038 + ADR-0039 Accepted; `concepts/agent.md` + `concepts/human-agent.md` + `concepts/ontology.md` verified-headers bumped.
  - *User-facing tier*: every row of §3.C (7 actions) updated in-chunk OR carrying explicit defer-with-successor.
- **phi-core leverage aspect** — import-count delta = **0**; positive greps all ≥ expected; forbidden-duplication greps all 0; `check-phi-core-reuse.sh` exit 0; scaffold-removal grep returns 0.
- **Concept alignment aspect** — every §2 row at target-status; no `contradicted` remains; out-of-scope rows preserved as `silent-in-code` per ADR-0038 § Out-of-Scope.
- **K8s readiness aspect** — §3.B 7-axis populated; CH-16 declared K8s-neutral; ledger stays at 8.

**Two confidence % (each with named numerator/denominator):**

- **Implementation confidence** = `claims-verified-honored / claims-in-scope` = target **11/11 = 100%**. The 11 claims:
  1. `Identity` struct shipped with 4 concept-mandated fields + governance fields.
  2. `LivedExperience` field set matches concept-`agent.md` line 326 verbatim.
  3. `WitnessedExperience` field set matches concept-`agent.md` line 327 verbatim.
  4. Migration 0009 ships; UNIQUE-on-`agent_id` index present; round-trip default test passes.
  5. 4 repo methods shipped on both `InMemoryRepository` + `SurrealRepository`.
  6. `RepositoryError::HumanAgentHasNoIdentity` variant; defensive guard fires at both layers (in-memory + SurrealQL).
  7. `apply_agent_creation` writes Identity in same compound tx for `AgentKind::Llm`; preventive guard skips for `AgentKind::Human`.
  8. Server orchestrator builds `Identity::default_for_llm(agent.id, now)` for LLM kind only.
  9. `DomainEvent::IdentityUpdated` variant + `IdentityUpdateTrigger` enum + bus arms shipped (no production emitter — deferred to CH-21).
  10. Audit-emitter helpers `identity_created` (Alerted) + `identity_updated` (Routine) shipped; first emitter wired at agent-creation handler.
  11. AgentArchived orphan policy ratified: archive does not delete Identity; one regression test pins.

- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check / doc-pages-touched` = target **9/9 = 100%**.

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P3 seal phase.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 3 phases = medium chunk → **2 agents** (matches CH-22 + CH-06 precedent).

### Audit Agent A — Code correctness + phi-core leverage

> **Locked prompt** (drafted at Step 2; fired at P3 seal):
> You are auditing CH-16 (Identity node materialization + Human Agent guard) in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code. The chunk plan is at `docs/specs/plan/build/<8hex>-ch-16-identity-node-materialization.md`.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. `Identity` struct at `modules/crates/domain/src/model/nodes.rs` carries exactly 4 concept-mandated fields (`self_description`, `lived`, `witnessed`, `embedding`) plus governance fields (`id`, `agent_id`, `created_at`, `updated_at`).
> 2. `LivedExperience` field set: `sessions_completed`, `sessions_successful`, `ratings_window`, `skills`, `specializations`.
> 3. `WitnessedExperience` field set: `memories_extracted`, `subordinates_observed`, `extraction_scope_distribution`.
> 4. Old `scaffold_node!(Identity, NodeId)` line removed.
> 5. Migration 0009 exists with `DEFINE TABLE identity SCHEMAFULL` + UNIQUE-on-`agent_id` index.
> 6. `Repository` trait declares 4 new methods.
> 7. `RepositoryError::HumanAgentHasNoIdentity { agent_id }` variant defined.
> 8. Both `InMemoryRepository` and `SurrealRepository` impls cover all 4 methods.
> 9. `AgentCreationPayload` carries `identity: Option<Identity>` (#[serde(default)]).
> 10. `apply_agent_creation` validates: Llm-kind requires `identity: Some(_)`; Human-kind requires `identity: None`; mismatch returns `HumanAgentHasNoIdentity`.
> 11. Server orchestrators build `Identity::default_for_llm(...)` only for `AgentKind::Llm`.
> 12. `DomainEvent::IdentityUpdated { agent_id, trigger, at, event_id }` variant exists; `IdentityUpdateTrigger` has 4 variants; `kind() == "identity_updated"`.
> 13. Audit emitter helpers: `identity_created` (Alerted, `"platform.identity.created"`), `identity_updated` (Routine, `"platform.identity.updated"`).
> 14. `cargo test --workspace -- --test-threads=1` passes at ~1097.
> 15. `bash scripts/check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports for Identity.
> 16. `grep -rn '^pub struct Identity\b' modules/crates/ | grep -v 'domain/src/model/nodes.rs'` returns 0.
> 17. `cargo test -p store --test migration_0009` passes.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 700 words.

### Audit Agent B — Concept fidelity + docs fidelity

> **Locked prompt** (drafted at Step 2; fired at P3 seal):
> You are auditing CH-16's concept-fidelity + docs-fidelity in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code or docs.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. ADR-0038 Accepted at `m5_2/decisions/0038-identity-node-materialization.md` with sub-decisions D38.1–D38.7.
> 2. ADR-0039 Accepted at `m5_2/decisions/0039-human-agent-identity-guard.md` with sub-decisions D39.1–D39.5.
> 3. D-new-01 drift Status = `remediated`; lifecycle entry `<DATE> — remediated — CH-16 chunk-seal` present.
> 4. D-new-23 drift Status = `remediated`; lifecycle entry `<DATE> — remediated — CH-16 chunk-seal` present.
> 5. `drifts/README.md` rows for D-new-01 + D-new-23 reflect remediation.
> 6. `_concept-audit-matrix.md` 3 row flips: "Identity (Emergent)" 4-field; "No Identity" Human guard; "Identity ontology" 4-field shape.
> 7. concept-`agent.md` `Last verified` header bumped.
> 8. concept-`human-agent.md` `Last verified` bumped.
> 9. concept-`ontology.md` `Last verified` bumped.
> 10. `Identity` struct field names match concept-`agent.md` lines 323–328 verbatim.
> 11. `LivedExperience` field names match concept-`agent.md` line 326 verbatim.
> 12. `WitnessedExperience` field names match concept-`agent.md` line 327 verbatim.
> 13. Defensive guard test (read, don't run): `Repository::upsert_identity` for Human kind returns `HumanAgentHasNoIdentity`.
> 14. Preventive guard test: `apply_agent_creation` with Human-kind payload commits + zero identity rows.
> 15. New architecture page `m5_2/architecture/identity-node.md` cross-references concept-`agent.md` § "Identity Node Content" by anchor.
> 16. Operations doc `m5_2/operations/identity-operations.md` includes orphan-on-archive note + `HumanAgentHasNoIdentity` entry.
> 17. User-guide `m5_2/user-guide/identity-overview.md` explains 4 fields in operator language.
> 18. `m5/user-guide/troubleshooting.md` includes `HumanAgentHasNoIdentity` section pointing to concept-`human-agent.md`.
> 19. §3.C all 7 doc actions completed (or each non-touch explicitly justified).
> 20. §3.B K8s readiness 7-axis K8s-neutral; ledger count = 8 (unchanged).
> 21. CH-22 invariants intact: 15+ catalog-listener tests still pass.
> 22. CH-06 invariants intact: ADR-0037 §D37.3 honored — Identity has no `tags` field.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 700 words.

**Seal-blocking rule.** Both audits must report PASS on every check, OR each FAIL must be either (a) fixed in-chunk before seal, (b) reframed via user-approved ADR amendment, or (c) converted to a new drift file with explicit future-chunk assignment.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: 1067 (CH-06 baseline) + ~30 new ≈ 1097

# 3. Chunk-specific positive greps
grep -n "pub struct Identity\b" modules/crates/domain/src/model/nodes.rs                       # 1
grep -n "pub struct LivedExperience\b" modules/crates/domain/src/model/nodes.rs                # 1
grep -n "pub struct WitnessedExperience\b" modules/crates/domain/src/model/nodes.rs            # 1
grep -n "fn upsert_identity\b\|fn get_identity\b\|fn delete_identity\b\|fn list_identities_for_org\b" \
  modules/crates/domain/src/repository.rs                                                      # 4
grep -n "DomainEvent::IdentityUpdated\b\|IdentityUpdateTrigger\b" modules/crates/domain/src/events/mod.rs  # ≥ 3
grep -n "HumanAgentHasNoIdentity" modules/crates/domain/src/repository.rs                      # ≥ 1
ls modules/crates/store/migrations/0009_identity_node.surql                                    # exists
ls modules/crates/server/tests/identity_materialization_acceptance.rs                          # exists
ls modules/crates/store/tests/identity_repo.rs                                                 # exists

# 4. Chunk-specific negative greps
grep -rn "scaffold_node!.*Identity" modules/crates/domain/src/model/nodes.rs                   # 0
grep -rn "use phi_core::" modules/crates/domain/src/model/nodes.rs | grep -i identity         # 0
grep -rn "^pub struct Identity\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"        # 0
grep -A 20 "^pub struct Identity\b" modules/crates/domain/src/model/nodes.rs | grep "tags:"   # 0 (no tags field)

# 5. Targeted test runs
/root/rust-env/cargo/bin/cargo test -j 4 -p domain model::nodes
/root/rust-env/cargo/bin/cargo test -j 4 -p domain in_memory::tests::identity
/root/rust-env/cargo/bin/cargo test -j 4 -p domain audit::events::m5_2::identity
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test identity_repo
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migration_0009
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test apply_agent_creation_tx_test
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test agents_create_identity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test identity_materialization_acceptance

# 6. Drift terminal closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-01.md  # 1
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-23.md  # 1

# 7. ADR status
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0038-identity-node-materialization.md  # 1
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0039-human-agent-identity-guard.md     # 1

# 8. K8s ledger unchanged
grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md   # 8

# 9. Prior-chunk regression sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib events::listeners::tests              # 15+ catalog tests still green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test instance_tags_emission              # CH-06 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test grant_mint_conformance              # CH-06 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_sessions_m5p4            # CH-02 sessions still green
```

---

## What this plan does NOT do

- **No production emitter for `IdentityUpdated`.** ADR-0038 §D38.5: variant + bus dispatch ship at CH-16; first production emitter at CH-21 (memory-extraction listener body).
- **No embedding population.** ADR-0038 §D38.3 / Fork 2: `embedding: Vec<f32>` defaults to `vec![]` at create. Platform-level embedding model config does not exist at v0.1; populating embeddings deferred to **M6-DEFERRED-03 — Identity embedding provider integration** (NEW marker added to forward-scope at chunk seal).
- **No archive-deletes-Identity.** ADR-0038 §D38.6 / Fork 4 LEAVE QUERYABLE: archive does not call `delete_identity`. The method exists for symmetry and operator-driven cleanup.
- **No `self_description` synthesiser.** Concept-`agent.md` line 325: synthesiser is part of CH-21's body. CH-16 ships only the storage substrate.
- **No `tags` field on Identity.** ADR-0037 §D37.3 (CH-06) committed Identity to no-tags; ADR-0038 §D38.7 carries this forward.
- **No M6-deferred fields.** `lived.skills: Vec<SkillRef>` uses placeholder `SkillRef { name: String }` until M6 lands typed `SkillId`. `lived.ratings_window: Vec<RatingPoint>` uses placeholder until token-economy fields land.
- **No CLI surface for Identity inspection.** A `phi identity show <agent_id>` command is M6 / CH-19 work.
- **No HTTP API endpoint for direct Identity edits.** Identity edits flow through event-driven path; operator surface is M6 work.

---

## Notes on M5.1/P3 Q&A binding

- **Q1** (storage-backend ratification) — untouched.
- **Q4** (chunk ordering) — user-selected CH-16 next after CH-06; respects forward-scope §4 prereq edge `CH-02 → CH-16`.
- **Q5** (M5 scope) — D-new-01 is HIGH and must close before M5 tag.
- **Q6** (ADR numbering) — ADR-0038 + ADR-0039 claimed; verified next-free at draft time.
- **Q7** (uniform ExitPlanMode ritual) — this plan is being approved via ExitPlanMode.
- **Q8** (K8s readiness) — §3.B populated; CH-16 declared K8s-neutral.
- **Q9** (user-facing doc strategy, codified by CH-22) — §3.C populated; 7 file actions identified.

---

## Critical files for implementation

**New files:**
- `modules/crates/store/migrations/0009_identity_node.surql` — Identity table + UNIQUE-on-`agent_id` index
- `modules/crates/store/tests/identity_repo.rs` — SurrealDB repo round-trip + Human-guard
- `modules/crates/store/tests/migration_0009.rs` — migration round-trip
- `modules/crates/server/tests/agents_create_identity.rs` — server handler-level
- `modules/crates/server/tests/identity_materialization_acceptance.rs` — end-to-end
- `modules/crates/domain/src/audit/events/m5_2/identity.rs` — audit-emitter helpers
- `docs/specs/v0/implementation/m5_2/decisions/0038-identity-node-materialization.md`
- `docs/specs/v0/implementation/m5_2/decisions/0039-human-agent-identity-guard.md`
- `docs/specs/v0/implementation/m5_2/architecture/identity-node.md`
- `docs/specs/v0/implementation/m5_2/operations/identity-operations.md`
- `docs/specs/v0/implementation/m5_2/user-guide/identity-overview.md`

**Modified files (heavy):**
- `modules/crates/domain/src/model/nodes.rs` — replace scaffold-only `Identity` (line 852–857) with full struct + `LivedExperience` + `WitnessedExperience` + `RatingPoint` + `SkillRef` + `ExtractionScopeDistribution`
- `modules/crates/domain/src/repository.rs` — `Repository` trait gains 4 methods + `RepositoryError::HumanAgentHasNoIdentity` + `AgentCreationPayload.identity` field
- `modules/crates/domain/src/in_memory.rs` — `InMemoryRepository` impl for 4 new methods + `apply_agent_creation` extension + Human-guard
- `modules/crates/store/src/repo_impl.rs` — `SurrealRepository` impl for 4 new methods + `apply_agent_creation` SurrealQL extension
- `modules/crates/domain/src/events/mod.rs` — `DomainEvent::IdentityUpdated` variant + `IdentityUpdateTrigger` enum + `kind()`/`event_id()` arms

**Modified files (light):**
- `modules/crates/server/src/platform/agents/create.rs:155-171` (Identity built for LLM kind only) + `:235` (audit emit `identity_created`)
- `modules/crates/server/src/platform/system_agents/add.rs` (Identity always for system agents)
- `modules/crates/server/src/bootstrap/claim.rs` (CEO is Human-kind — Identity stays None)
- `modules/crates/server/tests/acceptance_common/admin.rs` (extend `AgentCreationPayload` literals with `identity: None` or `Some(default_for_llm(...))`)
- `modules/crates/store/tests/apply_agent_creation_tx_test.rs` (4 new scenarios)
- `modules/crates/domain/src/audit/events/m5_2/mod.rs` (NEW or extended — module declaration)
- Drift files + concept-doc verified-headers + m1 architecture pages + m5 troubleshooting (per §3.C)
- `docs/specs/plan/forward-scope/22035b2a-...md` (NEW M6-DEFERRED-03 marker added at P3 seal)

**Reused (no edit):**
- `modules/crates/domain/src/audit/mod.rs` — `AuditEvent` shape + `AuditEmitter` trait — Identity audit events plug in unmodified.
- `modules/crates/domain/src/events/bus.rs` — `EventBus` + `EventHandler` — `IdentityUpdated` rides existing path.

---

## Verification end-to-end (after seal)

1. `git status --short` — only the listed files in working tree.
2. `cargo test -j 4 --workspace -- --test-threads=1` — green at ~1097.
3. `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` — all green.
4. `git log --oneline -5` — chunk seal commit cites CH-16, ADR-0038 + ADR-0039, drifts D-new-01 + D-new-23.
5. Manual sanity: spawn server in dev profile → create an LLM agent → verify identity row exists (`phi agent show --json` would expose it once CH-19 ships the CLI surface; until then, `repo.get_identity(agent_id)` in a test fixture).
