<!-- Last verified: 2026-05-18 by Claude Code (revised after user-clarification gate; 9 user-locked decisions folded into §4 + §9; ready for phase-planner Deliverable 3 dispatch). -->
<!-- Last verified: 2026-05-18 by Claude Code (initial draft post-CH-27 close; 3-axis Explore-agent audit synthesised; user-clarification fold-back pending per §4 + §9). -->

# M6 Alignment Audit — 2026-05-18

> **What this is.** A point-in-time alignment check between (i) the M6 narrative in [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) lines 292–316, (ii) the M6-DEFERRED markers in [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §3, and (iii) the current state of baby-phi (concept docs + code at HEAD = post-CH-27 seal). Three independent Explore agents covered (A) Agent self-service surfaces a01–a05, (B) Memory contract + permission-over-time retrieval (C-M6-1), (C) Inter-agent messaging + resolver wiring + identity embedding (3 M6-DEFERRED items). This document captures the **raw audit findings**; the user-clarification fold-back will land in §4 + §9 post-AskUserQuestion gate.

> **What this is not.** A plan or chunk approval. Items flagged here become candidates for the M6 forward-scope (Phase 3 deliverable of [`m6-forward-scoping-and-rename-cleanup-af8aed16.md`](../review-and-docs/m6-forward-scoping-and-rename-cleanup-af8aed16.md)) at the user's discretion — they are not committed work.

---

## §1 — Methodology

- **Source of truth (M6 surface)**:
  - Build plan §M6 narrative: [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) lines 292–316 (5 agent-self-service surfaces a01–a05 + C-M6-1 carryover with 3 sub-requirements).
  - Prior forward-scope §3 M6 markers: [`remaining-scope-post-m5-p7-22035b2a.md`](../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §3 (M6-DEFERRED-01..04 + M6-DEFERRED-RESOLVERS-WIRING + M6+-OPEN-01 AgentProfile cardinality + M6-or-M7-DEFERRED token economy).
- **Audit dimensions**:
  - **A. Agent self-service surfaces** (5 claims): a01 Inbox/Outbox + a02 Auth Requests + a03 Consent records + a04 My work + a05 My profile + grants.
  - **B. Memory contract + permission-over-time retrieval** (3 sub-claims of C-M6-1): MemoryStore trait contract + multi-tag ownership + permission-over-time retrieval gate.
  - **C. Inter-agent messaging + resolver wiring + identity embedding** (3 claims): M6-DEFERRED-02 (Inbox/Outbox AgentMessage materialization) + M6-DEFERRED-RESOLVERS-WIRING (`projects::resolvers::*` actor-passthrough; D-CH27-FOLLOWUP-01) + M6-DEFERRED-03 (Identity embedding provider integration; ADR-0038 §D38.3).
- **Method**: each axis got a parallel Explore agent with claim-by-claim instructions to score `ALIGNED / PARTIAL / GAP / DRIFT`, quote the concept passage, quote the code surface (file:line), flag contradictions, and estimate rough chunk shape (single-chunk vs split). Read-only.
- **Then** the user will clarify intent on the open questions emerging from the synthesis (§4 + §9). This document folds those clarifications back into the scoring **post-AskUserQuestion gate**.

---

## §2 — Top-line scoreboard (raw audit findings, pre-clarification)

| Axis | Aligned | Partial | Gap | Drift |
|---|---|---|---|---|
| **A. Agent self-service** | 0 | 3 (a01, a03, a05) | 2 (a02, a04) | 0 |
| **B. Memory contract + retrieval** | 1 (multi-tag) | 0 | 2 (trait, retrieval gate) | 0 |
| **C. Messaging + resolvers + embedding** | 0 | 1 (messaging substrate) | 1 (embedding provider) | 1 (resolvers — intentional deferral) |
| **Total** | **1 / 11** | **4 / 11** | **5 / 11** | **1 / 11** |

The scoreboard captures raw findings. Post-clarification, items may shift between PARTIAL ↔ GAP based on whether dependencies bind earlier or later in M6, and some "GAP — concept-defined, zero-code" items may be re-scored as carve-outs deferred further (M6+ or M7).

---

## §3 — Aligned claims (1 of 11, brief acknowledgement)

These are honored in code with concept-doc backing — no follow-up needed at M6:

1. **Memory ownership via multi-tag (§B.2)** — Memory node has `tags: Vec<String>` field; `build_memory_tags` helper at `domain/src/events/listeners.rs:709-723` derives the 4 governance tags (`agent:`, `session:`, `project:`, `org:`) on each MemoryExtracted event per ADR-0040 §D40.2. Compatible with CH-26's `tags: Vec<String>` on Organization + Project + Session (same `Vec<String>` shape, independent fields). No additional work required at M6 for this sub-requirement.

---

## §4 — Findings re-scored after user clarification

> **Status: RESOLVED.** All 9 user-clarification questions (§9) locked at 2026-05-18 AskUserQuestion gate. The 9 decisions reshape M6 chunk-sequencing as captured in §4.0 (sequencing summary) + per-axis tables below.

### 4.0 — User-locked decisions summary (2026-05-18 AskUserQuestion gate)

| # | Decision locked | Effect on M6 chunk-graph |
|---|---|---|
| Q1 | **Sequence** — M6-DEFERRED-02 first, a01 UI follows | Two chunks: foundation (messaging body) → consumer (a01 UI + tools) |
| Q2 | **Block** — a01 must close before a02/a03/a04 | Strict serial gate: a01 → {a02, a03, a04} parallel after |
| Q3 | **Display-only** — a04/a05 independent of C-M6-1 | a04/a05 chunks parallel with C-M6-1 chunk |
| Q4 | **Strict prereq** — M6-DEFERRED-03 closes before a05 | a05 chunk sequences after embedding-provider chunk |
| Q5 | **Full routing** — cross-agent delivery guarantees + ordering + duplicates | M6-DEFERRED-02 effort up: ~5-7 ed; likely splits into 2 chunks (substrate + routing) |
| Q6 | **F3.b trait re-shape** — extend resolver traits with `actor: Option<AgentId>` | M6-DEFERRED-RESOLVERS-WIRING chunk: ~3-4 ed; cascades through 4 trait defs + 4 impls + ~6 test stubs + 1 wiring site |
| Q7 | **Separate `EmbeddingProviderConfig` node** | M6-DEFERRED-03 chunk: ~2-3 ed; NEW domain node + admin lifecycle + migrations + audit |
| Q8 | **Ship trait + retrieval gate with C-M6-1** | C-M6-1 chunk: ~3-4 ed; coherent MemoryStore trait + default impl + retrieval gate; M6-DEFERRED-04 (LLM body) consumes the trait |
| Q9 | **Pursue M6+-OPEN-01 at M6** — schedule concept-redesign chunk early in M6 | NEW chunk early in M6: AgentProfile cardinality 1:1 → N:1 redesign (~3-4 ed); affects a05 + M6-DEFERRED-04 downstream |

### 4.1 — Implied M6 chunk-graph after user-locked decisions

> **Illustrative, NOT binding.** The CH-28..CH-38 numbering + grouping below is a **rough sketch** showing dependency relationships + effort distribution; it is NOT a frozen contract. Three downstream split-points remain available: (a) **phase-planner** (Phase 3) may produce a different decomposition consuming this audit + locked decisions + base build plan; (b) **chunk-planner v21 F1.b split fallback** at each CH-NN plan-mode opening can split a forward-scope row into 2+ chunks if effort exceeds the 1.5× upper-band threshold (CH-27 v2 re-plan precedent); (c) **gate-2.5 mid-cycle scope-revision** via the chunk-implementer if mid-implementation scope creeps past pause-thresholds (CH-26 → CH-27 carve-out precedent ratified in ADR-0061 §D61.5 + §D61.7). The phase-planner's authoritative output at Phase 3 supersedes this sketch.

```
                  M5.3 CLOSE (CH-27 0edcaba9)
                              │
                              v
              CH-28 — AgentProfile cardinality redesign (Q9)
                              │
        ┌─────────────────────┼─────────────────────┐
        v                     v                     v
  CH-29 — M6-DEFERRED-02   CH-30 — C-M6-1        CH-31 — M6-DEFERRED-03
   (messaging full        (Memory trait +       (EmbeddingProviderConfig
   routing; may split     retrieval gate;       + admin lifecycle;
   into 29a + 29b)        Q8 + Q3 + B.2          Q7)
        │                  already ALIGNED)            │
        v                                              v
  CH-32 — a01 UI/tools                                 (M6-DEFERRED-03 ↘)
        │                                                              │
        ├──────────────┬──────────────┬─────────────┐                  │
        v              v              v             v                  v
  CH-33 — a02    CH-34 — a03    CH-35 — a04   CH-36 — M6-DEFERRED-04  CH-37 — a05
   (AR view)     (Consent view)  (My work)    (LLM supervisor body;   (Profile + grants;
                                              consumes C-M6-1 trait)   blocked by a01 + M6-DEFERRED-03)
        │              │              │
        └──────────────┴──────────────┴─────────────┐
                                                    v
                                            CH-38 — M6-DEFERRED-RESOLVERS-WIRING
                                             (F3.b trait re-shape;
                                             may sequence parallel to
                                             agent-self-service chunks
                                             if no cross-cutting handler edits)
```

**Critical-path chain**: CH-28 → CH-29 → CH-32 → {CH-33, CH-34, CH-35} ; CH-31 → CH-37.
**Parallelizable**: CH-30, CH-31, CH-38 can run concurrently with CH-29 / CH-32 after CH-28 closes (independent surfaces).
**Effort total (rough)**: ~10-12 chunks; ~30-40 engineer-days.

### 4.2 — Per-axis status table (post-clarification)

(unchanged from pre-clarification §4.A / §4.B / §4.C below — the 11 raw findings are unaffected by the 9 locks; the locks reshape sequencing + chunk shape, not the score.)

### 4.A — Axis A: Agent self-service surfaces

| Claim | Status | Concept evidence | Code evidence | Rough chunk shape |
|---|---|---|---|---|
| **a01 Inbox/Outbox** | PARTIAL | `concepts/ontology.md:83-86` (node scaffolds + footnote deferring messages embedding to M6-DEFERRED-02) + `concepts/agent.md:21` (messaging-as-pure-information-flow) + `requirements/agent-self-service/a01-my-inbox-outbox.md` (CONCEPTUAL; 3 acceptance scenarios; § 10 API contract) | Node scaffolds at `domain/src/model/nodes.rs:1082-1102` (id + agent_id + created_at + tags; **missing `messages: Vec<AgentMessage>` field**). Zero handlers in `server/src/router.rs`. Zero web UI scaffolds. | 2-3 chunks: (a) Inbox/Outbox message-field materialization + CRUD handlers; (b) Web UI + LLM tool surface (`read_inbox`, `send_message`). |
| **a02 Auth Requests** | GAP | `concepts/permissions/02-auth-request.md:12-30` (state machine spec) + `requirements/agent-self-service/a02-my-auth-requests.md` (CONCEPTUAL; 3 acceptance scenarios; § 10 API contract). | State machine FULLY shipped at M5/CH-10 (`domain/src/permissions/{state, transitions}.rs`). Zero agent-self-service handlers; no web UI. ~100% scope is HTTP + UI on top of existing domain logic. | 2-3 chunks: (a) AR query + slot-action handlers (inbound/outbound list, approve/deny/reconsider/escalate); (b) Web UI. |
| **a03 Consent records** | PARTIAL | `concepts/permissions/06-multi-scope-consent.md:353-426` (Consent state machine spec) + `requirements/agent-self-service/a03-my-consent-records.md` (CONCEPTUAL). | State machine FULLY shipped at CH-09+CH-10 (`Consent` node + `domain/src/permissions/consents/{state, transitions}.rs` + sweeper). Per-policy minter helpers at CH-11 + `domain::consents::minters`. Zero handlers; zero UI. Notification routing incomplete (Channel-side dispatch deferred). | 1-2 chunks: (a) Consent query + action handlers (acknowledge/decline/revoke); cascading revocation already in engine; (b) Web UI (optional, bundle-able with a02). |
| **a04 My work** | GAP | `concepts/project.md` § Task + `concepts/agent.md` § Parallelized Sessions + `requirements/agent-self-service/a04-my-work.md` (CONCEPTUAL; 3 acceptance scenarios; § 10 API contract). | Task node + ASSIGNED_TO edge shipped at M4. Session node + RUNS_SESSION edge shipped at M5/P4. Task status transitions shipped at M4. Zero agent-self-service handlers; concurrency-cap query likely missing; zero web UI. | 2 chunks: (a) Task + Session query handlers scoped to viewer agent + task status update + concurrency-cap enforcement; (b) Web UI (my-tasks panel + my-sessions panel). |
| **a05 My profile + grants** | PARTIAL | `concepts/agent.md` § Identity (4-field Identity) + `concepts/permissions/04-manifest-and-resolution.md` § Authority Chain + `requirements/agent-self-service/a05-my-profile-and-grants.md` (CONCEPTUAL). | Identity node shipped at CH-16 (`nodes.rs:1149-1175`, 4 fields). Authority-chain traversal shipped at CH-14 (`domain/src/permissions/authority_chain.rs`). Zero agent-self-service handlers; zero web UI. Embedding deferred to M6-DEFERRED-03 (init as `Vec::new()` per ADR-0038 §D38.2). | 1-2 chunks: (a) Profile + grants query handlers (own profile + HOLDS_GRANT traversal) + self_description update + authority-chain expansion endpoint; (b) Web UI. |

### 4.B — Axis B: Memory contract + permission-over-time retrieval

| Claim | Status | Concept evidence | Code evidence | Rough chunk shape |
|---|---|---|---|---|
| **B.1 MemoryStore trait contract** | GAP | `build-plan:307` (trait draft shape with `extract` + `retrieve` + `tag` async methods). | Memory node exists at `domain/src/model/nodes.rs:1106-1111` (minimal v0 per CH-21: `{id, owning_agent, tags, created_at}`). Repository trait exposes `create_memory` + `list_memories_for_agent`. **No `memory_contract.rs`** at HEAD. CH-21 plan §3.C lists this file as M6 artefact; no drift body explicitly pins it to M6-DEFERRED-04. | 1-2 chunks (~2-3 ed): trait definition + in-tree default impl (SurrealDB-backed); third-party plug-ins (Chroma, Weaviate, LanceDB) follow as M6+ optional. |
| **B.2 Ownership via multi-tag** | **ALIGNED** | `build-plan:308` (multi-tag spec) + `concepts/permissions/05-memory-sessions.md:26-35` (tag vocabulary). | `Memory.tags: Vec<String>` field at `nodes.rs:1106-1111`. `build_memory_tags` helper at `listeners.rs:709-723` materialises the 4 governance tags. `MemoryExtracted` audit at `audit/events/m5_2/memory.rs:64-70` carries tags as serializable. Compatible with CH-26's `tags: Vec<String>` field on Organization + Project (independent fields, same shape). | No new work at M6 for this sub-requirement. |
| **B.3 Permission-over-time retrieval** | GAP | `build-plan:309` (live Permission Check at retrieval time; grants revoked after extraction forfeit immediate read access; interacts with M1 engine + M4 authority-chain + M7 `DESCENDS_FROM`). | `Action::Recall` defined at `action.rs:152` in `ActionCategory::Memory` but **no consumer** applies it. No `build_memory_recall_manifest` builder; builder module comment at `permissions/builders/mod.rs:16-17` flags "M6+ per-tool runtime + memory-recall builders co-locate here when they land." No `server/src/platform/memory/` directory. CH-27 / ADR-0062 §D62.2 widened synth-grant to 4 verbs `[Allocate, Transfer, Observe, Inspect]` — but Observe/Inspect address resource-discovery, NOT memory-recall semantics. | ~1.5-2 chunks (~3-5 ed): (a) `build_memory_recall_manifest` builder; (b) retrieval handler (HTTP + CLI); (c) storage tag-intersection query; (d) tie-in to M7 `DESCENDS_FROM` walk if delegation chains factor in. |

### 4.C — Axis C: Messaging + resolvers + identity embedding

| Claim | Status | Concept evidence | Code evidence | Rough chunk shape |
|---|---|---|---|---|
| **C.1 Inter-agent messaging (M6-DEFERRED-02)** | PARTIAL | `concepts/ontology.md:83-84` (composite definitions) + `:86` (deferred-state footnote; M6-DEFERRED-02 ratified at ADR-0057 §D57.8) + `concepts/permissions/05-memory-sessions.md:579-590` (full `AgentMessage` value-object 9-field shape) + `concepts/agent.md:21` (messaging-as-pure-information-flow; "no automatic reaction"). | `Composite::ALL.len() == 10` includes `InboxObject` + `OutboxObject` per CH-26. Node scaffolds shipped at `nodes.rs:1082-1102` (id + agent_id + created_at + tags). **Missing `messages: Vec<AgentMessage>` field** + **AgentMessage value object not defined in code**. Repository trait ships `create_inbox` + `create_outbox` empty bodies; no message store/retrieve methods. | 2.5-3.5 ed single chunk: AgentMessage struct + migration adding messages field + repository methods + read/write handlers + acceptance scenarios. Routing semantics open (Q1 below). |
| **C.2 Resolver actor-passthrough (D-CH27-FOLLOWUP-01)** | DRIFT (intentional) | `concepts/core-philosophy.md:16,28,29` (ownership permission-gating principle). No standalone concept doc for resolver-actor routing; rationale in `m5_3/drifts/D-CH27-FOLLOWUP-01.md` body. | 4 resolver trait defs at `domain/src/events/listeners.rs:244-507` carry NO actor parameter. 4 `Repo*Resolver` impls in `store/src/repo_impl.rs` bound to actor-blind shapes. Wiring site at `listeners.rs:276-340` calls resolvers in background event-bus dispatcher without `check_permission`. ~6 static-test stubs implement actor-blind signatures. | 1.5-4 ed depending on M6 lock: **F3.b** trait re-shape (~3-4 ed; cascades through 4 traits + 4 impls + 6 test stubs + 1 wiring site) OR **F3.c** HTTP wrapper (~1.5-2 ed; ~50 LOC helper). |
| **C.3 Identity embedding provider (M6-DEFERRED-03)** | GAP | `concepts/agent.md:332` (embedding field spec; `Vec<f32>` dim configurable default 1536) + `:342-344` (platform-level config fixed at org-bootstrap; model change is admin event; switch triggers batch re-embed via Auth Request). ADR-0038 §D38.3 deferral. | Identity node shipped at CH-16 (`nodes.rs:1149-1175`); `embedding: Vec<f32>` field exists; migration `0009_identity_node.surql:43-47` schema supports field with default empty array. Factory `Identity::default_for_llm` initialises `embedding: Vec::new()`. Acceptance test at `identity_materialization_acceptance.rs:103` confirms zero-vector default per ADR-0038 §D38.2. **Zero embedding-provider code** in `server/src/platform/` or anywhere. No `ModelRuntime` wiring for embedding-as-a-model. | 2-3 ed: (a) embedding provider mock + test doubles; (b) `EmbeddingProviderConfig` node or `ModelRuntime` entry; (c) admin endpoint to set provider + batch re-embed; (d) self_description → embedding derivation. |

---

## §5 — New gaps surfaced by the 3-axis audit

### 5.1 — Cross-page notification routing (Axis A spillover)

a02 + a03 + a04 all reference inbox-based notifications (e.g., "transitioning a task to Blocked inserts a message in the project lead's inbox"). If a01's messaging body is NOT yet operational at a02/a03/a04 chunk time, notification routing must either (a) be sequenced behind a01, or (b) fallback to Channel-based delivery for Humans + suppress for LLMs. The cross-page dependency was implicit in the build plan but the chunk-sequencing implications surface here.

### 5.2 — MemoryStore trait routing (Axis B / Axis C interaction)

The build plan §3.C (line 311) lists `memory_contract.rs` (MemoryStore trait + default impl) as M6 file-affected, but **no ADR explicitly pins it to M6-DEFERRED-04**. ADR-0040 §D40.7 (Out-of-Scope) covers LLM body + grant enforcement + multi-memory-per-session emission but does NOT list the trait contract or retrieval gate. This is a routing ambiguity for the phase-planner: does the trait ship as part of the C-M6-1 chunk (load-bearing for retrieval gate) or as a follow-on to M6-DEFERRED-04 (Memory-extraction LLM supervisor body)?

### 5.3 — Resolver-wiring design lock (Axis C / cross-cycle)

D-CH27-FOLLOWUP-01 explicitly defers the F3.b-vs-F3.c choice to M6 plan-open. The forward-scope can list it as an open question (carrying the user-decidable nature forward) but the chunk-planner cannot proceed on a specific resolver-wiring chunk until the choice is locked.

### 5.4 — Embedding provider choice + ModelRuntime dependency (Axis C)

The concept doc is silent on whether the embedding provider is (a) registered as a `ModelRuntime` entry (reusing M2 LLM provider registry), (b) a separate `EmbeddingProviderConfig` node, or (c) platform-globals configuration (TOML, not materialized as a node). The phase-planner needs this choice locked before decomposing the M6-DEFERRED-03 chunk.

---

## §6 — Acceptable carve-outs (1 of 11)

### 6.1 — Resolver actor-passthrough wiring (Axis C / Claim C.2) — DRIFT (intentional)

This is **NOT** a regression. CH-27 deferred the actor-passthrough design to M6-DEFERRED-RESOLVERS-WIRING per ADR-0062 §D62.3 (F3.a LOCKED, user-aligned). The resolver-blind background-listener pattern is correct for fire-listeners. The "drift" classification reflects the architectural-design-deferred-pending-M6-lock posture; M6 plan-open will lock either F3.b (trait re-shape) or F3.c (HTTP wrapper) and the drift transitions to remediated when the chosen chunk ships.

---

## §7 — Cross-cutting observations

### 7.1 — Code substrate generally ahead of HTTP/UI surfaces

The audit found that the **domain-tier code substrate is mostly shipped** for M6 (Memory node, Identity node + authority chain, Consent state machine, Auth Request state machine, Task + Session nodes, Inbox/Outbox composite scaffolds). What's missing is consistently the **HTTP handler tier** and **web UI tier**. This is consistent with the build plan's framing of M6 as primarily "agent self-service surfaces" — the surfaces are the HTTP + UI, with M5 having pre-built most of the domain logic. Phase-planner should expect M6 chunks to be heavier on `server/src/platform/<feature>/` + `modules/web/app/` than on `domain/`.

### 7.2 — M6-DEFERRED-02 (messaging) + a01 are tightly coupled

The audit surfaced that M6-DEFERRED-02 (Inbox/Outbox message materialization) and a01 (agent self-service inbox UI) likely belong in the same chunk (or tightly-sequenced sibling chunks). The messaging body materialization is the load-bearing pre-req for a01's tool surface (`read_inbox`, `send_message`) + UI. Phase-planner should consider whether to bundle them as a single CH-NN or split into a foundation chunk (M6-DEFERRED-02 first) followed by a UI chunk (a01 second).

### 7.3 — C-M6-1 (memory contract) decomposes into 3 distinct sub-chunks

The 3 C-M6-1 sub-requirements have meaningfully different scope:
- **B.1 trait** — ~1-2 ed (definition + default impl)
- **B.2 multi-tag** — already ALIGNED; no new chunk
- **B.3 permission-over-time retrieval gate** — ~1.5-2 ed (builder + handler + storage query + DESCENDS_FROM tie-in)

Combined effort ~3-4 ed. Phase-planner should consider whether to ship them as one CH-NN (single coherent "memory tier") or split into trait + retrieval-gate (two chunks).

### 7.4 — Identity embedding provider has dependency on M2 ModelRuntime decision

Whether embedding provider integrates with M2's `ModelRuntime` table or ships as a separate `EmbeddingProviderConfig` node is a design choice that affects the M6-DEFERRED-03 chunk shape (1 ed vs 2-3 ed delta). User-decidable.

### 7.5 — M6+-OPEN-01 (AgentProfile cardinality 1:1 → N:1) is a separate architectural question

Per forward-scope §3 line 356, the AgentProfile cardinality re-evaluation is an OPEN question, not a committed deferred-scope item. It's user-decidable at M6 plan-open (or via a standalone concept re-evaluation chunk before M6). The audit did not score this (it's a redesign question, not a drift); phase-planner should surface it in the M6 forward-scope §6+§7 open questions.

---

## §8 — Net read

**M6 surface is decomposable into ~8-12 chunks** (~25-35 engineer-days total) covering:
- 5 agent-self-service pages (a01-a05) — each likely 1-2 chunks (UI + handlers); a01 may bundle with M6-DEFERRED-02.
- 1-2 chunks for C-M6-1 (Memory trait + retrieval gate); multi-tag ownership already shipped.
- 1 chunk for resolver actor-passthrough wiring (M6-DEFERRED-RESOLVERS-WIRING; lock F3.b vs F3.c at plan-open).
- 1-2 chunks for embedding provider integration (M6-DEFERRED-03).
- 1 chunk for memory-extraction LLM supervisor body (M6-DEFERRED-04) — orthogonal to C-M6-1 trait.
- Possibly 1 chunk for AgentProfile cardinality redesign (M6+-OPEN-01) if user opts to pursue.

**No DRIFT findings (the resolver one is intentional deferral, not contradiction).** The code substrate is consistent with concept-doc claims; the gaps are honest scope-deferrals from M5.

**Key sequencing**:
- a01 + M6-DEFERRED-02 likely first (foundation for cross-page notification routing)
- C-M6-1 trait + retrieval gate before a04/a05 (if user clarification confirms data-flow dependency)
- M6-DEFERRED-RESOLVERS-WIRING + M6-DEFERRED-03 independent of agent-self-service chunks

---

## §9 — Open implications (RESOLVED — user-locked at 2026-05-18 AskUserQuestion gate)

All 9 user-clarification questions resolved. Locked answers below; the phase-planner agent consumes these as input #4 to decompose M6 into chunks.

| # | Question | **User-locked answer** | Effect on M6 forward-scope |
|---|---|---|---|
| **Q1** | a01 / M6-DEFERRED-02 sequencing | **Sequence** — M6-DEFERRED-02 (messaging body) first, a01 UI follows | M6-DEFERRED-02 chunk lands BEFORE a01 chunk |
| **Q2** | Notification routing fallback if a01 inbox not operational | **Block** — a01 must close before a02/a03/a04 | Strict serial gate: a01 → {a02, a03, a04} parallel after |
| **Q3** | a04 + a05 / C-M6-1 dependency | **Display-only** — a04/a05 are independent of C-M6-1 | C-M6-1 chunk parallel with a04/a05 chunks |
| **Q4** | a05 / M6-DEFERRED-03 (embedding) | **Strict prerequisite** — M6-DEFERRED-03 closes before a05 | a05 chunk sequences after embedding-provider chunk |
| **Q5** | M6-DEFERRED-02 messaging semantics | **Full routing** — cross-agent delivery guarantees + ordering + duplicates | M6-DEFERRED-02 effort ~5-7 ed; likely splits into 2 chunks (substrate + routing) |
| **Q6** | M6-DEFERRED-RESOLVERS-WIRING lock | **F3.b trait re-shape** — extend resolver traits with `actor: Option<AgentId>` | ~3-4 ed chunk; cascades through 4 trait defs + 4 impls + ~6 test stubs + 1 wiring site |
| **Q7** | M6-DEFERRED-03 embedding provider routing | **Separate `EmbeddingProviderConfig` node** | NEW domain node + admin lifecycle + migrations + audit; ~2-3 ed |
| **Q8** | MemoryStore trait routing | **Ship with C-M6-1** — trait + default impl + retrieval gate in one chunk | C-M6-1 chunk: ~3-4 ed; M6-DEFERRED-04 (LLM body) consumes trait |
| **Q9** | M6+-OPEN-01 AgentProfile cardinality | **Pursue at M6** — concept-redesign chunk early in M6 | NEW chunk at head of M6 chunk-graph: ~3-4 ed standalone |

### 9.1 — Implications captured

- **M6 chunk count**: ~10-12 chunks (CH-28 through CH-38ish).
- **M6 effort total**: ~30-40 engineer-days (vs CH-25..CH-27 M5.3 carve-out total ~15 ed).
- **Critical-path chain**: CH-28 (AgentProfile redesign) → CH-29 (M6-DEFERRED-02 substrate; possibly 29a + 29b) → CH-32 (a01) → {CH-33 a02, CH-34 a03, CH-35 a04} parallel.
- **Side-chain**: CH-31 (M6-DEFERRED-03 embedding provider) → CH-37 (a05).
- **Parallelizable independent chunks**: CH-30 (C-M6-1 memory trait + retrieval), CH-38 (M6-DEFERRED-RESOLVERS-WIRING F3.b trait re-shape) — can run alongside the agent-self-service track.
- **Last chunk before M7 plan-open**: CH-36 (M6-DEFERRED-04 LLM supervisor body) — consumes C-M6-1 trait + Identity/embedding work.

### 9.2 — Remaining surfaces deferred past M6 (NOT in this round's forward-scope)

- M7-DEFERRED-01: Channel schema enrichment (D-new-24).
- M7-DEFERRED-02: Task node materialization (D-new-26).
- M6-or-M7-DEFERRED: Token economy fields + Worth formula (D-new-27) — phase-planner may surface as a "decide at M7 plan-open" marker in M6 forward-scope §3.
- M7b: production-readiness hardening — already has its own forward-scope at `forward-scope/k8s-microservices-readiness-plan-ab49f22b.md`.

---

## §10 — Provenance

- **Audit date**: 2026-05-18 (post-CH-27 close, cycle hex `0edcaba9`; before M6 plan-mode open).
- **Method**: 3 parallel Explore agents (Axes A + B + C) dispatched concurrently from the orchestrator; each returned a per-axis report (under 700 words each). Synthesised into this single audit doc.
- **Code state**: HEAD post-CH-27 commit `225e263` (Ch-27 baby-phi) + parent phi `cdf015c` (CH-27 retrospective standards updates).
- **Plan archive**: this audit feeds Phase 3 of [`m6-forward-scoping-and-rename-cleanup-af8aed16.md`](../review-and-docs/m6-forward-scoping-and-rename-cleanup-af8aed16.md) (the phase-planner agent consumes this audit as input #4 to author the M6 forward-scope).
- **Next step**: AskUserQuestion gate on §9 Q1-Q9 (split into 3 questions of 3-4 sub-questions each to stay within the 4-option-per-question UI limit, or surfaced as a chained Q&A). Post-clarification, §4 + §9 are folded with answers; the final audit doc becomes the phase-planner input.
