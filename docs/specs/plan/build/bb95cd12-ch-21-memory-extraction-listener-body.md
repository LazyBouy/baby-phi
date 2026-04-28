<!-- Last verified: 2026-04-28 by Claude Code -->

# CH-21 — Memory-extraction listener body

**Plan file token:** `bb95cd12` (generated at chunk-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/bb95cd12-ch-21-memory-extraction-listener-body.md`.
**Chunk ID:** CH-21 (forward-scope §1 lines 191–195; §4 dependency edges line 320; §5 inventory row line 353).
**Severity:** ⚠HIGH.
**Expected effort:** ~1.5 engineer-days.
**Hard prerequisites:** CH-01 (`Agent.active` discriminator — DONE), CH-02 (real `agent_loop` + SessionEnded payloads — DONE), CH-06 (selector grammar + instance tags on Session — DONE), CH-16 (Identity row + `IdentityUpdateTrigger::MemoryExtracted` + `Repository::upsert_identity` + `identity_updated` audit helper — DONE), CH-22 (catalog-listener pattern to mirror — DONE).
**Chunks unblocked at close:** CH-23 (cross-listener acceptance — verifies CH-21 + CH-22 ordering/idempotency), CH-24 (M5 final seal).

---

## Context

CH-21 closes the last HIGH listener-body drift in M5 and lights the **first production emitter** of two CH-16 surfaces (`DomainEvent::IdentityUpdated` + `identity_updated` audit helper). It also closes drift **D6.1** terminally — CH-22 shipped the second `record_system_agent_fire` call site (catalog listener); CH-21 ships the first (memory extractor).

Three governance jobs land here, physically inseparable:

1. **D6.1 first call site** — `MemoryExtractionListener::on_event` body fills in. `record_system_agent_fire` called at top after system-agent resolution closes drift D6.1's last open call site.
2. **First-emitter wiring of CH-16's `IdentityUpdated` chain** — listener directly upserts the working agent's Identity row (`witnessed.memories_extracted += 1`, scope bucket increments) AND emits `DomainEvent::IdentityUpdated { trigger: MemoryExtracted, ... }` on the bus + `platform.identity.updated` audit. Per ADR-0028 fail-safe: errors log + continue (no retry).
3. **`DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit** — new event variant, new audit helper at `audit/events/m5_2/memory.rs`.

**User-decided forks (locked at plan-review via AskUserQuestion):**

1. **Extraction strategy: HEURISTIC v0** (no LLM call). One Memory per non-aborted SessionEnded. Full LLM-driven supervisor agent loop deferred to M6 / a future chunk; documented as Out-of-Scope in ADR-0040.
2. **Memory tag scope: DERIVE FROM SESSION TAGS**. Listener reads `Session.tags` (CH-06 instance-identity set) + computes `agent:{owning_agent}` + `session:{session_id}` + `project:{project_id}` + `org:{owning_org}`. Scope bucket decision: tag set contains `#public` → public; else → private. (The {private,public} binary on `ExtractionScopeDistribution` does not have a "project" sub-bucket — project-scoped memories count as private at v0; documented in ADR-0040.)
3. **Event shape: DEFINE BOTH** — `DomainEvent::MemoryExtracted { memory_id, session_id, owning_agent, org_scope, tags, extracted_at, event_id }` + new audit helper `memory_extracted` emitting `platform.memory.extracted` (Logged class).
4. **Disabled-state behavior: SKIP BOTH**. If `memory-extractor` system agent has `active = false`, log `warn!` + return without minting a Memory and **without** calling `record_system_agent_fire`. Telemetry tile shows last-fire from when the agent was active.
5. **Identity update: ONCE PER SESSION**. Single `upsert_identity` per fire after Memory creation succeeds. Bumps `witnessed.memories_extracted += 1` + the scope bucket (`extraction_scope_distribution.{private|public} += 1`) + `updated_at = ended_at`.
6. **Failure handling: ADR-0028 FAIL-SAFE**. Listener errors log + continue. No retry. Memory write succeeds → Identity upsert fails → log error (Memory durable; Identity gap healed at next extraction).

**Outcome:** D6.1 → `remediated`; concept-`system-agents.md` § Memory Extraction Agent storage substrate clauses honored at v0 (LLM-body clauses preserved as `silent-in-code` and routed to M6); CH-23 + CH-24 unblock; M5 v0-commitment count flips from 1 partial-closure to 0.

---

## §1 — Context & principle

### Why this chunk

CH-21 is the second half of the listener-body pair (CH-22 was the first to ship; chronological ordering picked it for graph-coverage breadth). Drift D6.1's terminal-closure rule is "both call sites shipped"; CH-21 is the closer.

CH-21 also closes the gap CH-16 left open: ADR-0038 §D38.5 committed that `IdentityUpdated` + the `identity_updated` audit helper would ship variant-only at CH-16, with CH-21 lighting the first emitter. Without CH-21, the variant is dead code and the helper has zero callers.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* Applied: the LLM-driven supervisor-agent body from `system-agents.md` § Memory Extraction Agent does NOT ship at v0; the v0 listener is heuristic. We do not pretend it satisfies the LLM concept clauses. ADR-0040 explicitly carves out the LLM body as Out-of-Scope and creates **M6-DEFERRED-04 (NEW)** as the successor marker. This preserves "honored vs silent-in-code" honesty.

### Forward-scope reference

[§1 CH-21 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 191–195) + [§4 dependency edges](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 320 — `CH-02 → CH-21`, `CH-21 → CH-23 → CH-24`) + [§5 inventory row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 353).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`system-agents.md`](baby-phi/docs/specs/v0/concepts/system-agents.md) | § Memory Extraction Agent — Behaviour 1 (lines 78–79) | Runtime delivers transcript + metadata to the agent on `session_end` | silent-in-code (stub) | honored at v0 (`SessionDetail` available via `fetch_session`; listener reads it heuristically — LLM agent invocation deferred) |
| `system-agents.md` | § Memory Extraction Agent — Behaviour 2 (line 80) | Agent identifies candidate memories from transcript via LLM judgment | contradicted | preserved silent-in-code; deferred to **M6-DEFERRED-04 (NEW)** per ADR-0040 § Out-of-Scope; v0 mints 1 deterministic Memory per session |
| `system-agents.md` | § Memory Extraction Agent — Behaviour 3 (line 81) | Per-memory pool routing (private / project / org / #public) per tags + content sensitivity | contradicted | partially honored — v0 binary {private,public} bucket via session-tag inspection (`#public` → public; else → private); 4-pool routing deferred per ADR-0040 |
| `system-agents.md` | § Memory Extraction Agent — Behaviour 4 (line 82) | Memories written via `store_memory` tool | silent-in-code | preserved silent-in-code; v0 calls `Repository::create_memory` directly (tool-grant path is M6 / LLM-body work) |
| `system-agents.md` | § Memory Extraction Agent — Behaviour 5 (line 83) | Audit event `MemoryExtracted { session_id, memory_id, pool, extractor }` per memory | contradicted | honored — new `domain::audit::events::m5_2::memory::memory_extracted` helper emits `platform.memory.extracted` (Logged class) with `{session_id, memory_id, owning_agent, tags, extractor: actor_agent_id}` |
| `system-agents.md` | § Memory Extraction Agent — Grants (lines 85–114) | Two standard grants (read-session + store-memory) | silent-in-code | preserved silent-in-code; ADR-0040 §D40.4 carries forward — heuristic listener bypasses grant check; LLM-body chunk re-enables grant enforcement |
| [`permissions/05-memory-sessions.md`](baby-phi/docs/specs/v0/concepts/permissions/05-memory-sessions.md) | § Supervisor Extraction as Two Standard Grants (lines 153–171) | Extraction is a permission-checked supervisor capability | silent-in-code | preserved silent-in-code; ADR-0040 § Out-of-Scope routes to LLM-body successor chunk |
| [`agent.md`](baby-phi/docs/specs/v0/concepts/agent.md) | § Two Streams of Experience — `witnessed.memories_extracted` (line 327) | Counter incremented when this agent (as supervisor) extracts a memory from a subordinate session | contradicted (counter never moves) | honored — listener bumps `witnessed.memories_extracted += 1` per fire on the **session's `started_by`** agent (not a separate supervisor agent at v0) |
| `agent.md` | § Two Streams of Experience — `extraction_scope_distribution` (line 327) | Counts of private vs public memories extracted | contradicted | honored — listener increments correct bucket per scope decision |
| [`coordination.md`](baby-phi/docs/specs/v0/concepts/coordination.md) | § Event-driven reactivity / runtime-status telemetry | Listener fires advance system-agent runtime-status tiles | partial-closure (CH-22 closed catalog tile only) | honored — both system agents now fire telemetry; D6.1 terminally closed |
| [`README.md`](baby-phi/docs/specs/v0/concepts/README.md) | (entry invariants) | Concepts subtree invariants | honored | honored (re-verified) |
| [`phi-core-mapping.md`](baby-phi/docs/specs/v0/concepts/phi-core-mapping.md) | (phi-core surfaces) | No phi-core overlap for memory extraction | honored | honored (declared in §3) |

**Note on the v0 supervisor model:** The concept frames extraction as a **supervisor agent** (separate from the working agent) extracting memories from subordinate sessions. At v0 the heuristic listener mints memories directly attributed to the session's `started_by` agent (the working agent itself), and increments `witnessed.memories_extracted` on that same agent. This is a documented v0 simplification (ADR-0040 §D40.5) — the LLM-body chunk introduces the supervisor-as-actor distinction.

---

## §3 — phi-core leverage map

| phi-core type | Current handling | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::session::model::{Session, LoopRecord, Turn}` | Already wrapped by `domain::model::nodes::Session` (M5/P1) and read via `Repository::fetch_session` (existing) | wrap | reuse — listener calls `fetch_session(SessionId)` to access SessionDetail |
| (none other) | — | — | — |

**Rationale:** Memory extraction governance + audit + Identity update are baby-phi-native. phi-core has zero memory-extraction or audit-trail tier. The only phi-core touch is reading session traces, which already flows through the existing wrap.

**Expected import-count delta at chunk close:** **0 phi-core imports added or removed.**

**Positive close-audit greps** (must pass at seal):
```bash
grep -n "DomainEvent::MemoryExtracted\b" modules/crates/domain/src/events/mod.rs              # ≥ 1
grep -n "fn memory_extracted\b" modules/crates/domain/src/audit/events/m5_2/memory.rs         # 1
grep -n "platform.memory.extracted" modules/crates/domain/src/audit/events/m5_2/memory.rs     # 1
grep -rn "record_system_agent_fire\(" modules/crates/ | wc -l                                  # ≥ 3 (def + 2 call sites)
grep -n "MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME" modules/crates/domain/src/events/listeners.rs # ≥ 1
grep -n "fn resolve_memory_extraction_system_agent\b" modules/crates/domain/src/events/listeners.rs # 1
ls modules/crates/server/tests/acceptance_memory_extraction.rs                                 # exists
```

**Forbidden-duplication greps** (must return 0):
```bash
grep -rn "use phi_core::" modules/crates/domain/src/events/listeners.rs                       # 0
grep -rn "^pub struct .*MemoryExtraction" modules/crates/ | wc -l                              # 1 (the listener; no parallel impls)
bash scripts/check-phi-core-reuse.sh                                                            # exit 0
grep -n "MemoryExtractionListener (stub)" modules/crates/domain/src/events/listeners.rs        # 0 (stub log removed)
grep -n "_repo: Arc<dyn Repository>" modules/crates/domain/src/events/listeners.rs             # 0 (underscore-prefixed fields renamed)
```

---

## §3.B — K8s microservice readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | Listener holds `Arc<dyn Repository>` + `Arc<dyn AuditEmitter>` + `Arc<EventBus>` (all already shared); no `OnceCell`/`Mutex`/process-globals added. | No |
| **A2** IPC channels | Bus emit of `DomainEvent::IdentityUpdated` + `DomainEvent::MemoryExtracted` rides existing `EventBus`; no new IPC. | No |
| **A3** pod-local resources | None. SessionDetail read goes through Repository (durable); Memory write goes through Repository. | No |
| **A4** migration runner | No new migration. | No |
| **A5** trait-shape requirement | Listener implements existing `EventHandler` trait. Object-safe. | No |
| **A6** cross-pod state sharing | If multiple pods subscribe to the same EventBus, the same SessionEnded fires multiple extractions — this is a known M7b broker concern. CH-21 documents the assumption (single-pod at v0); adds a per-session idempotency caveat to ADR-0040 § Future Concerns (no implementation in CH-21). | Documented; no new ledger entry |
| **A7** audit hash-chain symmetry | New `platform.memory.extracted` (Logged) + first-emitter wiring of `platform.identity.updated` (Logged from CH-16). Both ride existing `AuditEmitter::emit` → per-org hash chain. | No |

**Conclusion:** **K8s-neutral.** Ledger stays at 8 (post-CH-16 baseline). Multi-pod-extraction concern documented in ADR-0040 but not promoted to new ledger entry (the M7b broker carve-out already covers it).

---

## §3.C — User-facing documentation impact map (post-Q9)

| Tier | File | Touched? | Action |
|---|---|---|---|
| Architecture | `m5_2/architecture/memory-extraction-listener.md` (NEW) | yes — design page: heuristic v0 scope, scope-derivation logic, disabled-state behavior, fail-safe semantics, first-emitter wiring of `IdentityUpdated` | (a) create in-chunk |
| Architecture | [`m1/architecture/audit-events.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/audit-events.md) | yes — add `platform.memory.extracted` row (Logged class); bump verified header to note CH-21 wires the first emitter of `platform.identity.updated` | (a) update in-chunk |
| Architecture | [`m1/architecture/graph-model.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/graph-model.md) | yes — verified header amendment noting CH-21 ships first MemoryExtracted DomainEvent + audit (no struct shape changes) | (a) update in-chunk |
| Operations | `m5_2/operations/memory-extraction-operations.md` (NEW) | yes — runbook: disabled-state behavior, fail-safe semantics (Memory durable / Identity gap), per-session idempotency caveat (multi-pod), how to re-trigger after operator-disable | (a) create in-chunk |
| Operations | [`m5/operations/system-agents-operations.md`](baby-phi/docs/specs/v0/implementation/m5/operations/system-agents-operations.md) (verify exists) | yes if exists — note that disabling memory-extractor now skips both extraction + telemetry fire (fork D); add D6.1 closure note | (a) update in-chunk if file present; otherwise (b) defer with successor `CH-23` |
| User-guide | `m5_2/user-guide/memory-extraction-overview.md` (NEW) | yes — operator-facing: what extraction does at v0 (one Memory per session, scope from session tags), what the LLM-driven upgrade will add (M6), how to inspect Memory rows + Identity counters | (a) create in-chunk |
| User-guide | [`m5/user-guide/troubleshooting.md`](baby-phi/docs/specs/v0/implementation/m5/user-guide/troubleshooting.md) | yes — add "Extraction not firing" entry mapping disabled-state behavior to operator action; add "Identity counter stale" entry pointing at fail-safe semantics | (a) update in-chunk |

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D6.1** | [`m5_1/drifts/D6.1.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D6.1.md) | HIGH | `partial-closure → remediated` | First call site (memory-extractor) ships in CH-21; second (catalog) shipped at CH-22. Lifecycle entry: `2026-04-28 — remediated — CH-21 chunk-seal — first call site landed; both system-agent runtime-status tiles now advance on listener fire.` |

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — D6.1 row Status → `remediated`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip rows for: "system-agents.md § Memory Extraction Agent — audit emission" (`contradicted → honored`); "agent.md § Two Streams of Experience — witnessed.memories_extracted counter" (`contradicted → honored`); "coordination.md § runtime-status telemetry — both call sites" (`partial-closure → honored`).

**Mid-flight discovery hook:** if a phase reveals a third drift (e.g., a fixture asserting Memory rows are absent post-SessionEnded; or `system-agents-operations.md` runbook missing), surface via `AskUserQuestion` and add a row before phase close.

---

## §5 — ADRs drafted

ADR numbering check: next-free **0040**, then **0041** (verified at draft time — current max is 0039).

| ADR | Title | Drafted at phase | Decision summary | Flip-to-Accepted phase |
|---|---|---|---|---|
| **ADR-0040** | Memory-extraction listener — heuristic v0 scope; LLM-body deferred | P1 | **D40.1** Fork 1 HEURISTIC v0; one Memory per non-aborted SessionEnded. **D40.2** Fork 2 DERIVE-FROM-SESSION-TAGS; tags = union of `Session.tags` + `agent:{owning_agent}` + `session:{session_id}` + `project:{project_id}` + `org:{owning_org}`; bucket decision: `#public` in tag set → public, else private. **D40.3** Fork 4 SKIP-BOTH on disabled state (no telemetry fire either). **D40.4** Heuristic listener bypasses Supervisor Extraction grants (concept-`permissions/05-memory-sessions.md`); deferred to **M6-DEFERRED-04** "Memory-extraction LLM supervisor body" (NEW marker added at P3 seal). **D40.5** v0 simplification — `witnessed.memories_extracted` increments on the session's `started_by` agent (the working agent acts as its own extractor at v0); supervisor-as-actor distinction lands with the LLM body. **D40.6** Per-session idempotency: multi-pod fan-out concern documented; v0 assumes single-pod listener; M7b broker carve-out covers cross-pod dedup. **D40.7** Out-of-Scope at v0 (carve-out): per-memory pool routing (private/project/org/#public), grant enforcement, LLM agent_loop invocation, multi-memory-per-session extraction. | Chunk seal (P3) |
| **ADR-0041** | `DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit class | P1 | **D41.1** New `DomainEvent::MemoryExtracted { memory_id, session_id, owning_agent, org_scope: Option<OrgId>, tags: Vec<String>, extracted_at: DateTime<Utc>, event_id: AuditEventId }` variant; `kind() == "memory_extracted"`. **D41.2** New audit helper `domain::audit::events::m5_2::memory::memory_extracted(actor, memory, session_id, scope_bucket, org, ts) -> AuditEvent` emitting `event_type = "platform.memory.extracted"`, `audit_class = AuditClass::Logged`, `target_entity_id = Some(memory.id.into())`, diff carries `{session_id, owning_agent, tags, scope_bucket: "private"|"public"}`. **D41.3** Audit class is **Logged** (not Alerted) — extraction is routine bookkeeping, not a security-relevant change. **D41.4** First emitter wires at `MemoryExtractionListener::on_event`; ADR-0028 fail-safe semantics — emit is post-commit, errors log + continue. | Chunk seal (P3) |

ADR file paths:
- [`m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md)
- [`m5_2/decisions/0041-memory-extracted-event-and-audit.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0041-memory-extracted-event-and-audit.md)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant relied on | Re-verification command |
|---|---|---|
| Post-CH-16 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1102 | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` |
| CH-01 / ADR-0034 | `Agent.active`/`archived_at` discriminator; listener honors disabled state via `repo.get_agent` | `grep -n "agent.active" modules/crates/domain/src/events/listeners.rs` ≥ 2 (catalog + memory) |
| CH-02 / ADR-0032 | `BabyPhiSessionRecorder` emits real `DomainEvent::SessionEnded` | `cargo test -p server --test acceptance_sessions_m5p4 -j 4` (still green) |
| CH-06 / ADR-0036 + 0037 | `Session.tags` carries `#kind:session` + `session:{id}` | `grep -n "Session\.tags\b\|session\.tags" modules/crates/domain/src/events/listeners.rs` ≥ 1 |
| CH-16 / ADR-0038 + 0039 | `Identity` row, `IdentityUpdateTrigger::MemoryExtracted`, `Repository::upsert_identity`, `identity_updated` audit helper, `apply_agent_creation` materializes Identity for LLM kind | `cargo test -p server --test identity_materialization_acceptance -j 4` (still green) |
| CH-22 / ADR-0035 | `AgentCatalogListener` body + `record_system_agent_fire` second call site; 5-listener invariant | `cargo test -p domain --lib events::listeners::tests -j 4` (15+ tests still green); `grep -c "bus.subscribe" modules/crates/server/src/state.rs` = 5 |
| All chunks | 4 CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` |

---

## §7 — Phases within the chunk

**Phase count: 3** → audit envelope = **2 agents** (medium chunk; matches CH-16 + CH-22 + CH-06 precedent).

### P1 — Listener body + new event variant + new audit helper (~0.7d)

**Goal.** Replace the M5/P3 stub with a functioning heuristic body. Add `DomainEvent::MemoryExtracted` variant + `platform.memory.extracted` audit helper. Wire `record_system_agent_fire` for the memory-extractor system agent.

**Deliverables.**

1. **`MemoryExtractionListener` body** at [`modules/crates/domain/src/events/listeners.rs:505-536`](baby-phi/modules/crates/domain/src/events/listeners.rs#L505-L536) (replace stub):
   - Rename `_repo` → `repo`, `_audit` → `audit`. Constructor signature unchanged (no bus field at v0; see "Bus re-emission deferred" note below).
   - Constant `MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME: &str = "memory-extraction-agent"` near the catalog constant.
   - Helper `resolve_memory_extraction_system_agent(org_id) -> Option<AgentId>` mirroring CH-22's catalog resolver (lines 590–600).
   - Body on `DomainEvent::SessionEnded`:
     1. Read `Agent` row for `session.started_by`; bail if missing or `kind != Llm` (Human-extraction not part of v0).
     2. Read `SessionDetail` via `repo.fetch_session(session_id)`; bail if missing or `governance_state == Aborted`.
     3. Resolve memory-extractor system agent for the working agent's org via `org.system_agents`; bail if not found.
     4. Read system agent's `Agent` row; if `!agent.active || agent.archived_at.is_some()` → log `warn!` + return (Fork D: skip both).
     5. Mint Memory: derive tag set (see §1 fork 2); generate `MemoryId`; call `repo.create_memory(&memory)`.
     6. Decide scope bucket: tag set contains `#public` → `public`; else → `private`.
     7. Read working agent's Identity via `repo.get_identity(working_agent.id)`; if `None`, log warn + skip Identity update (CH-16 invariant says LLM agents have identity rows; missing → diagnostic).
     8. Compute `after = before.clone(); after.witnessed.memories_extracted += 1; after.witnessed.extraction_scope_distribution.{private|public} += 1; after.updated_at = ended_at;`
     9. Call `repo.upsert_identity(&after)`.
     10. Emit `platform.memory.extracted` audit via `memory_extracted(...)` helper.
     11. Emit `platform.identity.updated` audit via `identity_updated(actor=working_agent.id, before, after, MemoryExtracted, org, ended_at)` helper.
     12. Call `record_system_agent_fire(repo, org, memory_extractor_agent_id, parallelize, last_error=None, ended_at)`.
   - Each step's failure is fail-safe: log + continue to next step where safe; bail when continuing would be incoherent (e.g., scope decision needs the Memory minted).
   - **Bus re-emission deferred at v0** — `DomainEvent::MemoryExtracted` + `DomainEvent::IdentityUpdated` variants are defined for forward-compat, but the listener does NOT call `bus.emit()` (would create Arc cycle: bus → listener → bus). No current subscriber consumes either variant; the audit log captures all reactive state. Future chunk that wires a real consumer will introduce a `Weak<dyn EventBus>` field with a clean break-cycle design.

2. **New `DomainEvent::MemoryExtracted` variant** in [`events/mod.rs`](baby-phi/modules/crates/domain/src/events/mod.rs):
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
   Plus `kind() => "memory_extracted"` and `event_id() => *event_id` arms.

3. **New audit helper** at [`modules/crates/domain/src/audit/events/m5_2/memory.rs`](baby-phi/modules/crates/domain/src/audit/events/m5_2/memory.rs) (NEW):
   ```rust
   pub enum ExtractionScope { Private, Public }

   pub fn memory_extracted(
       actor: AgentId,
       memory: &Memory,
       session_id: SessionId,
       scope_bucket: ExtractionScope,
       org: OrgId,
       timestamp: DateTime<Utc>,
   ) -> AuditEvent
   ```
   `event_type = "platform.memory.extracted"`, `audit_class = AuditClass::Logged`, `target_entity_id = Some(NodeId::from_uuid(memory.id.as_uuid()))`, diff = `{session_id, owning_agent, tags, scope_bucket}`.

4. **Module declaration** in [`audit/events/m5_2/mod.rs`](baby-phi/modules/crates/domain/src/audit/events/m5_2/mod.rs) — add `pub mod memory;`.

5. **`AgentCatalogListener` non-impact verification**: confirm the existing `IdentityUpdated` short-circuit at line 630 also short-circuits `MemoryExtracted` (add the new variant to the `None` arm in `agent_id_and_timestamp_for`).

**Tests** (~12 unit + 1 round-trip):
- 1 listener unit test: heuristic body fires on SessionEnded → Memory created + Identity updated + audit emitted (in-memory repo + audit emitter).
- 1 listener unit test: disabled memory-extractor system agent → no Memory, no Identity update, no telemetry fire.
- 1 listener unit test: SessionAborted → no Memory (aborted sessions skip extraction).
- 1 listener unit test: no system agents resolvable → graceful skip + log warn.
- 1 listener unit test: Human-kind working agent → graceful skip (Human has no Identity per CH-16).
- 1 listener unit test: scope decision — `#public` in session.tags → public bucket increments.
- 1 listener unit test: scope decision — only project/org tags → private bucket increments.
- 1 audit-helper unit test: `memory_extracted` produces correct event_type + audit_class + diff fields.
- 1 audit-helper unit test: `memory_extracted` with empty tags → diff has empty tag list.
- 1 DomainEvent round-trip: `MemoryExtracted` variant serde-stable; `kind()`/`event_id()` arms green.
- 1 EventBus dispatch test: emit `MemoryExtracted` on bus → registered listeners observe it.
- 1 record_system_agent_fire integration test: tile advances after listener fire (mirror CH-22's pattern).

**User-facing doc updates.** Land `m5_2/architecture/memory-extraction-listener.md` (NEW) + update `m1/architecture/audit-events.md` (add `platform.memory.extracted` row).

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if `EventBus::emit` from inside a handler creates a re-entrant deadlock (the `IdentityUpdated`/`MemoryExtracted` emit happens from within a SessionEnded handler — verify single-task semantics); or if `Repository::create_memory` requires an additional argument we haven't seen; or if `ExtractionScope` enum collides with an existing type.

---

### P2 — Acceptance test + system-agent-disabled path + prior-chunk regression sanity (~0.5d)

**Goal.** End-to-end acceptance through HTTP-driven session-launch → session-end → extraction fires → Memory + Identity + audit chain. Plus CH-22 regression sanity (15+ catalog tests).

**Deliverables.**

1. **Acceptance test** at [`modules/crates/server/tests/acceptance_memory_extraction.rs`](baby-phi/modules/crates/server/tests/acceptance_memory_extraction.rs) (NEW). 6 scenarios:
   1. End-to-end: launch + complete a real session → `repo.list_memories_for_agent(agent_id)` returns 1 Memory; `repo.get_identity(agent_id)` returns `Some(_)` with `witnessed.memories_extracted == 1`; audit chain has `platform.memory.extracted` followed by `platform.identity.updated` in same org_scope.
   2. SessionAborted variant: abort a session → no Memory minted; `witnessed.memories_extracted == 0`.
   3. Disabled extractor: operator-disable `memory-extraction-agent` → run a session → no Memory, no telemetry tile advance, warn-level log present.
   4. Human-kind agent (no Identity): launch a session under a Human agent (CH-16 says no Identity row) → Memory still minted; Identity update gracefully skipped.
   5. Scope public bucket: session has `#public` in tags → `extraction_scope_distribution.public` increments.
   6. Audit chain ordering: `platform.session.ended` → `platform.memory.extracted` → `platform.identity.updated` in correct hash-chain order within the same org.

2. **`list_memories_for_agent` repo method** (if not already present — quick grep at P2 start):
   ```rust
   async fn list_memories_for_agent(&self, agent_id: AgentId) -> RepositoryResult<Vec<Memory>>;
   ```
   Add to both `InMemoryRepository` + `SurrealRepository` if missing. If already present (used by other tests), reuse.

3. **CH-22 regression sanity:** confirm 15+ catalog-listener tests still green; confirm `record_system_agent_fire` is now called from 2 sites (memory + catalog), tile-advance test pinned for both.

4. **`MemoryExtractionListener` constructor** at [`server/src/state.rs:200-204`](baby-phi/modules/crates/server/src/state.rs#L200-L204) — unchanged; constructor signature stays `(repo, audit)` per v0 bus-re-emission deferral. Verify 5-listener invariant test still green (CH-22's `state.rs:262`).

5. **Drift D6.1 lifecycle entry** appended (text only, status flips at P3 seal).

**Tests** (~10):
- 6 acceptance scenarios (above).
- 1 list_memories_for_agent unit test (if method newly added).
- 1 5-listener-invariant regression test (state.rs:262 — confirms unchanged).
- 1 second-call-site test for `record_system_agent_fire` — tile advances on extraction fire (mirror CH-22 line 559's catalog test).
- 1 cross-listener regression: `AgentCatalogListener` ignores `IdentityUpdated` + `MemoryExtracted` (verifies the existing short-circuit at `listeners.rs:630` extended for new variant).

**User-facing doc updates.** Land `m5_2/operations/memory-extraction-operations.md` (NEW) + update `m5/operations/system-agents-operations.md` if exists.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if a multi-pod EventBus design surface emerges (would change ADR-0040 §D40.6 framing); or if `BabyPhiSessionRecorder` emits SessionEnded BEFORE compound-tx commit (would invalidate the post-commit fail-safe assumption).

---

### P3 — ADRs Accepted + drift remediated + concept-doc bumps + audit + seal (~0.3d)

**Goal.** Ratify ADR-0040 + ADR-0041. Close D6.1 terminally. Apply §3.C user-facing doc map. Spawn 2 audits. Seal.

**Deliverables.**

1. **ADR-0040 + ADR-0041** flipped from `Proposed` → `Accepted`.
2. **D6.1** Status → `remediated`; lifecycle entry: `2026-04-28 — remediated — CH-21 chunk-seal — first call site landed; both system-agent runtime-status tiles now advance on listener fire.`
3. **`drifts/README.md`** — D6.1 row Status flipped.
4. **`_concept-audit-matrix.md`** — 3 row flips (audit emission honored; witnessed.memories_extracted honored; coordination runtime-status fully honored).
5. **§3.C user-facing-doc map applied** — 7 file actions (in-chunk where applicable; explicit defer-with-successor where not).
6. **Concept-doc verified-headers bumped** on `concepts/system-agents.md`, `concepts/permissions/05-memory-sessions.md`, `concepts/agent.md`, `concepts/coordination.md`.
7. **Forward-scope marker added**: `M6-DEFERRED-04 — Memory-extraction LLM supervisor body` appended to forward-scope under the M6 deferrals block.
8. **Spawn 2 audit agents** per §11.

**Tests.** All §8 named tests green; full workspace passes ~1102 + ~22 = ~1124.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** PAUSE if either audit reports a finding — surface to user before seal.

---

## §8 — Tests summary

- **Expected total at chunk close:** 1102 (post-CH-16 baseline) + ~22 new tests = **~1124 serialised tests**.
- **Layer breakdown:**
  - Unit (`listeners.rs::tests` mod-internal): ~7 (listener body scenarios)
  - Unit (`audit/events/m5_2/memory.rs::tests`): 2 (helper correctness)
  - Unit (`events::mod::tests`): 1 (`MemoryExtracted` round-trip)
  - Unit (EventBus dispatch): 1 (cross-listener observation)
  - Integration (`server/tests/acceptance_memory_extraction.rs` NEW): 6
  - Regression sanity: ~5 (5-listener invariant + catalog tests still green)

- **New test files:**
  - `modules/crates/server/tests/acceptance_memory_extraction.rs`
  - `modules/crates/domain/src/audit/events/m5_2/memory.rs` (with mod tests)

- **Expected-still-green fragile tests:**
  - `acceptance_system_flows_s03.rs::agent_create_populates_catalog_and_advances_runtime_status_tile` — CH-22's tile test; the catalog system agent's tile advances independently of the memory tile.
  - `events::listeners::tests::*` — 15+ catalog tests; `MemoryExtracted` adds a variant; non-matching arms must be unreachable per CH-22's `if let` guard pattern.
  - `acceptance_sessions_m5p4.rs::*` — CH-02 sessions still pass with new listener body running on every SessionEnded (must not change session governance state).

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive this plan verbatim (mandatory first action)

1. Generate plan-file token: `openssl rand -hex 4`.
2. **Copy this plan file verbatim**: `cp /root/.claude/plans/sharded-discovering-stearns.md baby-phi/docs/specs/plan/build/<8hex>-ch-21-memory-extraction-listener-body.md`. No edits during the copy.
3. Update placeholder in lines 4–5 of the archived plan only.
4. Run `bash scripts/check-doc-links.sh` to confirm relative links resolve.
5. Verify: `head -5 baby-phi/docs/specs/plan/build/<8hex>-ch-21-*.md` shows verified-by-Claude-Code header on line 1.
6. Only AFTER successful archive does the rest of §9 run.

This step matches the chunk-lifecycle-checklist Step 1 and the precedent set by CH-01 (`2aa37c80`), CH-02 (`16fd9a3a`), CH-22 (`c5f201bb`), CH-06 (`acd383e2`), and CH-16 (`2ae4fabe`).

**Reading list (mandatory before continuing):**
1. `concepts/system-agents.md` § Memory Extraction Agent (lines 34–135) — full.
2. `concepts/permissions/05-memory-sessions.md` § Supervisor Extraction (lines 153–171).
3. `concepts/agent.md` § Two Streams of Experience (lines 276–304) — for `witnessed` semantics.
4. `concepts/coordination.md` § Event-driven reactivity / runtime-status telemetry.
5. Drift D6.1 (full content; partial-closure lifecycle).
6. CH-16 plan archive (`2ae4fabe`) — for `IdentityUpdated` + `identity_updated` carrier surface.
7. CH-22 plan archive (`c5f201bb`) — for `record_system_agent_fire` second call site + ADR-0035 listener-pattern precedent.
8. ADR-0028 (fail-safe listener semantics) — for error-handling discipline.
9. ADR-0034 (Agent.active discriminator) — for disabled-state behavior.
10. ADR-0038 §D38.5 (CH-16 first-emitter commitment).
11. `forward-scope/22035b2a-...md` §1 CH-21 row + §4 dependency edges + §5 inventory.

**Carry-forward invariants (verified green at chunk-open):**
- `cargo test --workspace -- --test-threads=1` ≈ 1102.
- 4 CI guards green.
- D6.1 status `partial-closure`.
- ADR-0034..0039 Accepted.
- `git diff --stat HEAD -- modules/` empty.
- Highest applied migration is 0009 (no new migration in CH-21).
- Highest issued ADR is 0039.

**User-decided forks (already locked at plan-review):**
- Fork 1 HEURISTIC v0, Fork 2 DERIVE FROM SESSION TAGS, Fork 3 DEFINE BOTH (DomainEvent + audit), Fork 4 SKIP BOTH on disabled, Fork 5 ONCE-PER-SESSION Identity update, Fork 6 FAIL-SAFE no retry.

**Cargo command convention** (per memory): all cargo invocations use `-j 4`. Tests serialise via `--test-threads=1`.

---

## §10 — Close criteria (5-aspect, post-Q9)

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — all P1–P3 deliverables shipped; `cargo test --workspace -- --test-threads=1` green at ~1124; clippy green under `RUSTFLAGS="-Dwarnings"` with `-j 4`; `cargo fmt --all -- --check` green; `acceptance_memory_extraction.rs` 6/6 pass.
- **Docs aspect** — TWO scopes:
  - *Governance tier*: D6.1 lifecycle entry + Status flip; `_concept-audit-matrix.md` 3 rows flipped; `drifts/README.md` updated; ADR-0040 + ADR-0041 Accepted; `concepts/system-agents.md` + `concepts/permissions/05-memory-sessions.md` + `concepts/agent.md` + `concepts/coordination.md` verified-headers bumped.
  - *User-facing tier*: every row of §3.C (7 actions) updated in-chunk OR carrying explicit defer-with-successor.
- **phi-core leverage aspect** — import-count delta = **0**; positive greps all ≥ expected; forbidden-duplication greps all 0; `check-phi-core-reuse.sh` exit 0; stub-removal grep returns 0.
- **Concept alignment aspect** — every §2 row at target-status; `silent-in-code` rows preserved with explicit ADR-0040 § Out-of-Scope routing.
- **K8s readiness aspect** — §3.B 7-axis populated; CH-21 declared K8s-neutral; ledger stays at 8.

**Two confidence % (each with named numerator/denominator):**

- **Implementation confidence** = `claims-verified-honored / claims-in-scope` = target **9/9 = 100%**. The 9 claims:
  1. `MemoryExtractionListener` body fills in (no stub log; `_repo`/`_audit` renamed).
  2. `record_system_agent_fire` called for memory-extractor system agent (D6.1 first call site).
  3. `DomainEvent::MemoryExtracted` variant + `kind()`/`event_id()` arms shipped.
  4. `memory_extracted` audit helper shipped at `audit/events/m5_2/memory.rs`; `platform.memory.extracted` Logged class.
  5. First production emitter of `DomainEvent::IdentityUpdated` + `identity_updated` audit (CH-16 carry-forward closed).
  6. Heuristic body mints 1 Memory per non-aborted SessionEnded; tags derived from session.
  7. Scope bucket decision: `#public` → public; else → private; correct counter increments on Identity.
  8. Disabled-state behavior: skip both extraction + telemetry fire; warn-level log.
  9. Fail-safe semantics: errors log + continue; no retry; no partial state corruption.

- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check / doc-pages-touched` = target **8/8 = 100%**.

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P3 seal phase.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 3 phases = medium chunk → **2 agents** (matches CH-16 + CH-22 + CH-06 precedent).

### Audit Agent A — Code correctness + phi-core leverage

> **Locked prompt** (drafted at Step 2; fired at P3 seal):
> You are auditing CH-21 (memory-extraction listener body) in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code. The chunk plan is at `docs/specs/plan/build/<8hex>-ch-21-memory-extraction-listener-body.md`.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. `MemoryExtractionListener` body at `modules/crates/domain/src/events/listeners.rs` is no longer a stub — fields renamed `_repo`/`_audit` → `repo`/`audit`; debug log removed.
> 2. `record_system_agent_fire` is called from at least 2 sites under `modules/crates/` (catalog from CH-22, memory from CH-21). Grep `record_system_agent_fire\(` ≥ 2 hits beyond the def.
> 3. `MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME` constant defined; `resolve_memory_extraction_system_agent` helper present.
> 4. `DomainEvent::MemoryExtracted` variant exists in `events/mod.rs` with payload `{memory_id, session_id, owning_agent, org_scope, tags, extracted_at, event_id}`.
> 5. `kind() == "memory_extracted"` and `event_id()` arms present.
> 6. `memory_extracted` helper at `audit/events/m5_2/memory.rs` returns `event_type = "platform.memory.extracted"`, `audit_class = AuditClass::Logged`.
> 7. `audit/events/m5_2/mod.rs` declares `pub mod memory;`.
> 8. Listener emits `DomainEvent::IdentityUpdated` + `identity_updated` audit (CH-16 first-emitter wiring).
> 9. Disabled-state behavior: when memory-extractor agent has `active = false`, no Memory written + no telemetry fire (mock-repo unit test).
> 10. SessionAborted variant: listener early-returns; no Memory minted.
> 11. `cargo test --workspace -- --test-threads=1` green at ~1124.
> 12. `bash scripts/check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports in listeners.rs.
> 13. 5-listener invariant test at `server/src/state.rs:262` still green.
> 14. CH-22 catalog tile-advance acceptance test still green; new memory tile-advance test green.
> 15. CH-16 acceptance test (`identity_materialization_acceptance.rs`) still green; the new path doesn't break Identity creation.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 700 words.

### Audit Agent B — Concept fidelity + docs fidelity

> **Locked prompt** (drafted at Step 2; fired at P3 seal):
> You are auditing CH-21's concept-fidelity + docs-fidelity in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code or docs.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. ADR-0040 Accepted at `m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md` with sub-decisions D40.1–D40.7 + § Out-of-Scope listing the LLM-body deferral.
> 2. ADR-0041 Accepted at `m5_2/decisions/0041-memory-extracted-event-and-audit.md` with sub-decisions D41.1–D41.4.
> 3. Drift D6.1 Status = `remediated`; lifecycle entry `2026-04-28 — remediated — CH-21 chunk-seal` present.
> 4. `drifts/README.md` row for D6.1 reflects remediation.
> 5. `_concept-audit-matrix.md` rows flipped: audit emission `contradicted → honored`; `witnessed.memories_extracted` `contradicted → honored`; runtime-status telemetry `partial-closure → honored`.
> 6. concept-`system-agents.md` `Last verified` header bumped with CH-21 amendment.
> 7. concept-`permissions/05-memory-sessions.md` `Last verified` header bumped (Supervisor Extraction LLM-body deferral noted).
> 8. concept-`agent.md` `Last verified` bumped (witnessed counter now moves).
> 9. concept-`coordination.md` `Last verified` bumped (D6.1 closed).
> 10. New architecture page `m5_2/architecture/memory-extraction-listener.md` cross-references concept-`system-agents.md` § Memory Extraction Agent by anchor.
> 11. New operations doc `m5_2/operations/memory-extraction-operations.md` documents disabled-state + fail-safe semantics + per-session idempotency caveat.
> 12. New user-guide `m5_2/user-guide/memory-extraction-overview.md` explains v0 heuristic in operator language; flags M6 LLM-body upgrade.
> 13. `m5/user-guide/troubleshooting.md` includes "Extraction not firing" + "Identity counter stale" sections.
> 14. `m1/architecture/audit-events.md` includes `platform.memory.extracted` row (Logged class) + verified header amendment.
> 15. `m1/architecture/graph-model.md` verified header notes CH-21 amendment.
> 16. §3.C all 7 doc actions completed (or each non-touch explicitly justified).
> 17. §3.B K8s readiness 7-axis K8s-neutral; ledger count = 8 (unchanged).
> 18. CH-22 invariants intact: 15+ catalog-listener tests still pass.
> 19. CH-16 invariants intact: ADR-0038 §D38.5 promise honored — first emitter of `IdentityUpdated` is now CH-21's listener.
> 20. CH-06 invariants intact: Session.tags still carries `#kind:session` + `session:{id}`.
> 21. M6-DEFERRED-04 marker added to `forward-scope/22035b2a-...md` under M6 deferrals block.
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
# Expect: 1102 (CH-16 baseline) + ~22 new ≈ 1124

# 3. Chunk-specific positive greps
grep -n "DomainEvent::MemoryExtracted\b" modules/crates/domain/src/events/mod.rs              # ≥ 1
grep -n "fn memory_extracted\b" modules/crates/domain/src/audit/events/m5_2/memory.rs         # 1
grep -n "platform.memory.extracted" modules/crates/domain/src/audit/events/m5_2/memory.rs     # 1
grep -rn "record_system_agent_fire\(" modules/crates/ | wc -l                                  # ≥ 3
grep -n "MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME" modules/crates/domain/src/events/listeners.rs # ≥ 1
grep -n "fn resolve_memory_extraction_system_agent\b" modules/crates/domain/src/events/listeners.rs # 1
ls modules/crates/server/tests/acceptance_memory_extraction.rs                                 # exists

# 4. Chunk-specific negative greps
grep -n "MemoryExtractionListener (stub)" modules/crates/domain/src/events/listeners.rs        # 0
grep -n "_repo: Arc<dyn Repository>" modules/crates/domain/src/events/listeners.rs             # 0
grep -rn "use phi_core::" modules/crates/domain/src/events/listeners.rs                       # 0
grep -n "memory_extraction_listener_is_a_noop_at_p3" modules/crates/domain/src/events/listeners.rs # 0

# 5. Targeted test runs
/root/rust-env/cargo/bin/cargo test -j 4 -p domain events::listeners::tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain audit::events::m5_2::memory
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s03   # CH-22 catalog tile still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test identity_materialization_acceptance  # CH-16 still green

# 6. Drift terminal closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D6.1.md    # 1

# 7. ADR status
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md  # 1
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0041-memory-extracted-event-and-audit.md         # 1

# 8. K8s ledger unchanged
grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md   # 8

# 9. Forward-scope marker
grep -n "M6-DEFERRED-04" docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md  # ≥ 1

# 10. Prior-chunk regression sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib events::listeners::tests              # 15+ catalog tests still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_sessions_m5p4             # CH-02 sessions still green
```

---

## What this plan does NOT do

- **No LLM-driven supervisor agent body.** ADR-0040 §D40.1: heuristic v0 only. Full `agent_loop` invocation per concept-`system-agents.md` deferred to **M6-DEFERRED-04**.
- **No 4-pool memory routing.** ADR-0040 §D40.2: binary {private, public} bucket only at v0; project / org / #public sub-pools deferred to LLM body.
- **No Supervisor Extraction grant enforcement.** ADR-0040 §D40.4: heuristic listener bypasses grants; permission-checked path lands with LLM body.
- **No multi-memory-per-session extraction.** ADR-0040 §D40.7: 1 Memory per session at v0; LLM body may emit N.
- **No multi-pod idempotency.** ADR-0040 §D40.6: single-pod assumption documented; M7b broker carve-out covers cross-pod dedup.
- **No new migration.** Memory + Identity tables already exist (M1 + CH-16); CH-21 only writes through them.
- **No CLI surface for Memory inspection.** `phi memory list <agent_id>` is M6 / future-CH work.
- **No per-memory text/body field.** v0 Memory struct is `{id, owning_agent, tags, created_at}`; content encoded in tags. Future schema-evolution chunk (M6) extends this.

---

## Critical files for implementation

**New files:**
- `modules/crates/domain/src/audit/events/m5_2/memory.rs` — `memory_extracted` audit helper + `ExtractionScope` enum + mod tests
- `modules/crates/server/tests/acceptance_memory_extraction.rs` — 6-scenario end-to-end
- `docs/specs/v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md`
- `docs/specs/v0/implementation/m5_2/decisions/0041-memory-extracted-event-and-audit.md`
- `docs/specs/v0/implementation/m5_2/architecture/memory-extraction-listener.md`
- `docs/specs/v0/implementation/m5_2/operations/memory-extraction-operations.md`
- `docs/specs/v0/implementation/m5_2/user-guide/memory-extraction-overview.md`

**Modified files (heavy):**
- `modules/crates/domain/src/events/listeners.rs:505-536` — replace `MemoryExtractionListener` stub with full body + helpers; add `MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME` constant + `resolve_memory_extraction_system_agent` helper
- `modules/crates/domain/src/events/mod.rs` — add `DomainEvent::MemoryExtracted` variant + `kind()`/`event_id()` arms
- `modules/crates/domain/src/audit/events/m5_2/mod.rs` — add `pub mod memory;`

**Modified files (light):**
- `modules/crates/server/src/state.rs:200-204` — `MemoryExtractionListener::new(...)` constructor extended to pass `event_bus.clone()`
- `modules/crates/domain/src/repository.rs` — add `list_memories_for_agent(AgentId) -> Vec<Memory>` if not already present
- `modules/crates/domain/src/in_memory.rs` — impl `list_memories_for_agent` if newly added
- `modules/crates/store/src/repo_impl.rs` — impl `list_memories_for_agent` if newly added
- `modules/crates/domain/src/events/listeners.rs::tests` — extend with new listener-body unit tests; remove `memory_extraction_listener_is_a_noop_at_p3` test (line 1128)
- Drift files + concept-doc verified-headers + m1 architecture pages + m5 troubleshooting (per §3.C)
- `docs/specs/plan/forward-scope/22035b2a-...md` — append `M6-DEFERRED-04` marker

**Reused (no edit):**
- `modules/crates/domain/src/audit/mod.rs` — `AuditEvent` + `AuditEmitter` trait — Memory audit events plug in unmodified.
- `modules/crates/domain/src/audit/events/m5_2/identity.rs` — CH-16's `identity_updated` helper — listener calls it for the IdentityUpdated audit emit.
- `modules/crates/domain/src/events/bus.rs` — `EventBus` + `EventHandler` — `MemoryExtracted` + `IdentityUpdated` ride existing path.
- `modules/crates/domain/src/events/listeners.rs:52` — `record_system_agent_fire` — second call site (memory) calls existing helper unmodified.

---

## Verification end-to-end (after seal)

1. `git status --short` — only the listed files in working tree.
2. `cargo test -j 4 --workspace -- --test-threads=1` — green at ~1124.
3. `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` — all green.
4. `git log --oneline -5` — chunk seal commit cites CH-21, ADR-0040 + ADR-0041, drift D6.1.
5. Manual sanity: spawn server in dev profile → launch a session under an LLM agent → end the session → verify `repo.list_memories_for_agent(agent_id)` returns 1 row + `repo.get_identity(agent_id).witnessed.memories_extracted == 1` + audit log shows `platform.memory.extracted` followed by `platform.identity.updated`.
