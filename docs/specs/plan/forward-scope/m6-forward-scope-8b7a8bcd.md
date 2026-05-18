<!-- Last verified: 2026-05-18 by Claude Code (phase-planner v1 review-and-update pass; inaugural real-attribution invocation post-session-reload; 5 surgical updates applied to draft authored via general-purpose proxy at prior dispatch: (1) §1 CH-28 `ontology.md` cardinality citation line 92 → line 98 corrected against L98 `Agent | HAS_PROFILE | AgentProfile | 1:1 | Blueprint identity`; (2) §1 CH-28 `agent.md` §Soul citation lines 157–169 → lines 160–169 tightened against L160 `### Soul (Immutable Born Structure)`; (3) §3 preamble softened — M6-DEFERRED-RESOLVERS-WIRING was NOT a §3 marker in prior forward-scope (it was a §2.5 post-M5.3 closure cross-ref at L281); the inheritance bullet now reads "brought in-milestone via D-CH27-FOLLOWUP-01 closure → CH-38"; (4) §1 CH-32 phi-core leverage paragraph "CH-32 follow-on or CH-36" → "CH-32 follow-on or M6-DEFERRED-04 carve-out chunk per §6 Q1" (CH-36 is a04 My Work, not memory-supervisor); (5) §5 table CH-36 + CH-28 concept-doc cells given line cites for grep-verifiability. No structural re-decomposition; 11-chunk CH-28..CH-38 graph + 9 user-locked decisions preserved.) -->
<!-- Last verified: 2026-05-18 by Claude Code (phase-planner v1 initial draft via general-purpose proxy; M6 forward-scope authored from base build-plan §M6 + prior forward-scope §3 + 2026-05-18 alignment audit + 9 user-locks per audit §4.0/§9; 11 chunks CH-28..CH-38; ~28–38 ed total). -->

# Forward-scope inventory — M6 (Agent self-service surfaces + carryovers from M5)

**Milestone**: M6
**Purpose**: enumerate every chunk-level unit of work required to close the M6 surface (agent-self-service pages a01–a05 + C-M6-1 Memory tier carryover + 4 M6-DEFERRED markers brought in-milestone + D-CH27-FOLLOWUP-01 resolvers-wiring follow-on + M6+-OPEN-01 cardinality redesign brought in-milestone per user Q9 lock). Each chunk is a future per-chunk-plan candidate for the `chunk-planner` agent.
**Authority**: this document **does not** replace the base build plan ([`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) §M6 lines 292–316); it is a sibling forward-look produced from the [2026-05-18 M6 alignment audit](../core-philosophy-check/2026-05-18-m6-alignment-audit.md) + the 9 user-locked decisions captured at that audit's §4.0 + §9.

## What this document is / is not

**IS:**
- One document naming 11 chunks (CH-28 through CH-38) + a short tail of deferred-milestone scope markers preserved from the prior forward-scope §3 (those that did not come in-milestone for M6).
- Per-chunk one-paragraph scope, drift-id list, rough effort range (lower–upper engineer-days), concept docs touched, prerequisites, closes-milestone? flag.
- A dependency graph showing critical path + parallelizable surfaces + cross-milestone edges.
- Open questions surfacing chunk-zero choice + sub-fork decisions the user must lock before each chunk's plan-mode opens.

**IS NOT:**
- A detailed implementation plan per chunk (those are written just-in-time at chunk-open using [`per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md)).
- A schedule or calendar commitment (effort ranges are rough; per-chunk upper-band is the pause-trip threshold).
- A replacement for [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) whole-product roadmap.
- An ADR-level architectural-design lock (sub-fork choices identified in §6 must be locked at each chunk's plan-open via AskUserQuestion).

## Input artefacts (consumed)

- **Base build plan §M6**: [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) lines 292–316 (M6 narrative: 5 agent-self-service pages a01–a05 + C-M6-1 carryover with 3 sub-requirements).
- **Prior-milestone forward-scope**: [`remaining-scope-post-m5-p7-22035b2a.md`](./remaining-scope-post-m5-p7-22035b2a.md) — §3 (M6-DEFERRED-01..04 + M6+-OPEN-01 + M6-or-M7-DEFERRED token economy + M7-DEFERRED + M7b-DEFERRED markers). Note: D-CH27-FOLLOWUP-01 "M6-DEFERRED-RESOLVERS-WIRING" is referenced inline in prior §2.5 (post-M5.3 closure block, L281) rather than as a standalone §3 marker; this M6 forward-scope routes the drift to CH-38.
- **Pre-scoping alignment audit** (primary input): [`2026-05-18-m6-alignment-audit.md`](../core-philosophy-check/2026-05-18-m6-alignment-audit.md) — 3-axis Explore-agent findings (axis A self-service / axis B memory / axis C messaging+resolvers+embedding); 9 user-locked decisions folded into §4.0 + §9; chunk-graph sketch at §4.1 (illustrative, not binding).
- **Concept docs walked**: [`agent.md`](../../v0/concepts/agent.md) (§Identity + §Soul + §Parallelized Sessions); [`system-agents.md`](../../v0/concepts/system-agents.md) (§Memory Extraction Agent §Behaviour + §Allocation Rules); [`ontology.md`](../../v0/concepts/ontology.md) (Inbox/Outbox composite definitions + Memory edges); [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) (§AgentMessage value object lines 590–618 + §Supervisor Extraction lines 155+); [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) (Authority Chain); [`permissions/02-auth-request.md`](../../v0/concepts/permissions/02-auth-request.md) (AR state machine); [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) (Consent state machine lines 353–426); [`project.md`](../../v0/concepts/project.md) (§Task lines 107–149); [`requirements/agent-self-service/`](../../v0/requirements/agent-self-service/) (a01–a05 page files, each carrying §11 acceptance scenarios + §10 API contract).
- **Drift cross-references**: [`v0/implementation/m5_3/drifts/D-CH27-FOLLOWUP-01.md`](../../v0/implementation/m5_3/drifts/D-CH27-FOLLOWUP-01.md) (resolvers-wiring closure); [`v0/implementation/m5_2/decisions/0038-identity-node-materialization.md`](../../v0/implementation/m5_2/decisions/0038-identity-node-materialization.md) §D38.3 (embedding deferral); [`v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md`](../../v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md) §D40.7 (M6-DEFERRED-04 LLM body); [`v0/implementation/m5_1/drifts/D-new-25.md`](../../v0/implementation/m5_1/drifts/D-new-25.md) (messaging materialization); [`v0/implementation/m5_1/drifts/D-new-16.md`](../../v0/implementation/m5_1/drifts/D-new-16.md), [`D-new-28.md`](../../v0/implementation/m5_1/drifts/D-new-28.md) (memory tier).
- **Last shipped chunk**: CH-27 cycle hex `0edcaba9` (2026-05-18) per [`_cycle-index.md`](../build/_cycle-index.md) line 64. Therefore M6 chunks begin at **CH-28**.

---

## §1 — M6 in-milestone scope decomposition (chunks closing the M6 surface)

11 implementation chunks (**CH-28 through CH-38**). Chunks that close a HIGH-severity concept-contradiction OR ship load-bearing M6 substrate are marked ⚠HIGH. The decomposition honors the 9 user-locked decisions at the alignment-audit §4.0 + §9 — see §4 dep graph for the locks' structural effects.

### Foundation tier (architectural redesign + load-bearing primitives)

**CH-28 — AgentProfile cardinality 1:1 → N:1 redesign** · ⚠HIGH · 3–4d
- **Drifts closed**: **M6+-OPEN-01** brought in-milestone per Q9 user-lock 2026-05-18 (NOT a `D-*` drift; this is a concept-redesign chunk surfaced as an OPEN question at CH-01 and routed for M6 plan-open per `forward-scope/remaining-scope-post-m5-p7-22035b2a.md:356-370`).
- **Concept docs**: [`agent.md`](../../v0/concepts/agent.md) §"Soul" lines 160–169 (profile-as-genetics framing must be amended to template-sharing); [`ontology.md`](../../v0/concepts/ontology.md) line 98 (`Agent | HAS_PROFILE | AgentProfile | 1:1 | Blueprint identity` cardinality row — flip 1:1 → N:1).
- **Prerequisites**: none (M5.3 closed at CH-27 0edcaba9).
- **Deliverables**: concept-doc amendment (agent.md §Soul + ontology.md cardinality row); NEW ADR documenting the redesign rationale + migration plan; schema migration dropping the UNIQUE constraint on `agent_profile.agent_id`; NEW `uses_profile` edge (1:N from agent to agent_profile); decision-with-rationale about where per-agent overrides live in a shared-profile world (`parallelize`, `model_config_id`, `mock_response` — either Agent-side columns or new `agent_profile_override` table); data migration to re-key existing per-agent rows; refactor of `apply_agent_creation`, `upsert_agent_profile`, `get_agent_profile_for_agent`, `in_memory.rs` validation, `repo_impl.rs` upsert tx.
- **K8s posture**: A1 (in-process state) re-evaluated; A4 (migration runner) gains 1 new migration with first-apply data backfill — must verify idempotency.
- **phi-core leverage**: zero new phi-core types consumed; the `AgentProfile` wrap may need to expose a `blueprint_id` view distinct from the wrap's `agent_id` to support sharing (see ADR-0034 §D34.6 for wrap-vs-runtime separation).
- **Unblocks**: CH-37 (a05 profile editor consumes the shared-profile shape); CH-36 (M6-DEFERRED-04 supervisor body references AgentProfile by id).

**CH-29 — M6-DEFERRED-02 messaging substrate (Inbox/Outbox AgentMessage materialization, no routing)** · ⚠HIGH · 2.5–3.5d
- **Drifts closed**: **D-new-25** (AgentMessage embedding on Inbox/Outbox composites; cited in [`ontology.md:86`](../../v0/concepts/ontology.md) deferred-state footnote pointing at M6-DEFERRED-02; ratified at ADR-0057 §D57.8). Partial closure for CH-30's routing-tier dependency.
- **Concept docs**: [`ontology.md`](../../v0/concepts/ontology.md) §"Node Types — Social Structure" lines 83–87 (Inbox/Outbox composites) + §"Agent messaging value object" line 229 (`AgentMessage` 10-field shape: `{message_id, sender, recipient, subject, body, sent_at, thread_id, priority, delivered_to_inbox_at, read_at}`); [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"AgentMessage Value Object" lines 590–618 (priority enum + tag vocabulary); [`agent.md`](../../v0/concepts/agent.md) line 21 (messaging-as-pure-information-flow: "no automatic reaction; recipient decides").
- **Prerequisites**: **CH-28** (AgentProfile cardinality may affect the `sender`/`recipient` foreign-key shape if per-agent vs per-profile addressability is in play).
- **Deliverables**: `AgentMessage` value-object struct at `domain/src/model/composites/agent_message.rs` (new); migration `0019_inbox_outbox_messages.surql` adding `messages: Vec<AgentMessage>` field with `#[serde(default)]` to both `inbox_object` + `outbox_object` tables; Repository methods `append_message_to_inbox`, `append_message_to_outbox`, `list_inbox_messages_for_agent`, `list_outbox_messages_for_agent`, `mark_message_read`; basic write-and-read handlers without cross-agent routing; 6+ acceptance scenarios at `acceptance_m6_messaging_substrate.rs` (create + read + mark-read + priority enum + tag vocabulary + idempotent re-deliver). **Per Q5 user-lock**: cross-agent routing semantics (delivery guarantees + ordering + duplicate suppression) split into CH-30. **Per Q1 user-lock**: this chunk lands BEFORE a01 chunk (CH-33).
- **K8s posture**: A2 (IPC) — message-delivery between agents is a candidate IPC channel post-M7b microservices split; intra-pod direct write at M6, evaluate event-bus form at chunk-plan time.
- **phi-core leverage**: zero new phi-core types consumed; `AgentMessage` is pure baby-phi governance (no phi-core counterpart per phi-core-mapping concept doc).
- **Unblocks**: CH-30 (routing tier consumes the substrate), CH-33 (a01 read_inbox/send_message tool surface).

**CH-30 — M6-DEFERRED-02 messaging routing (cross-agent delivery + ordering + duplicate suppression)** · ⚠HIGH · 2.5–3.5d
- **Drifts closed**: **D-new-25** (final closure; the routing-tier half of M6-DEFERRED-02 per Q5 user-lock effort-split into 2 chunks).
- **Concept docs**: [`agent.md`](../../v0/concepts/agent.md) line 21 (messaging-as-pure-information-flow); [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"AgentMessage Value Object" lines 590–618 (priority enum drives delivery-ordering policy).
- **Prerequisites**: **CH-29** (substrate must exist; routing wires atop it).
- **Deliverables**: routing-tier dispatcher (cross-agent send wires sender's outbox-append + recipient's inbox-append atomically in compound tx); ordering semantics (priority-then-FIFO per priority bucket); duplicate-suppression idempotency key (`message_id` UNIQUE on inbox materialization); 6+ acceptance scenarios at `acceptance_m6_messaging_routing.rs` (cross-agent send + delivery ordering + duplicate suppression + priority enum + tag-vocabulary routing + audit-event emission). **Per Q5 user-lock**: full routing (the cheaper "outbox-only without delivery guarantee" was rejected).
- **K8s posture**: A2 (IPC channel) — routing path becomes the candidate canonical IPC surface for inter-agent communication; A6 (cross-pod state) — duplicate suppression key must be globally-unique across pods (use ULID for `message_id`); A7 (audit hash-chain) — every routing event emits one audit row.
- **phi-core leverage**: zero new phi-core types consumed.
- **Unblocks**: CH-33 (a01 tool surface can rely on routing guarantees).

### Independent infrastructure tier (parallelizable with foundation tier)

**CH-31 — M6-DEFERRED-03 EmbeddingProviderConfig domain node + admin lifecycle** · ⚠HIGH · 2–3d
- **Drifts closed**: **M6-DEFERRED-03** (no separate `D-*` drift file; covered by [`v0/implementation/m5_2/decisions/0038-identity-node-materialization.md`](../../v0/implementation/m5_2/decisions/0038-identity-node-materialization.md) §D38.3 deferral block).
- **Concept docs**: [`agent.md`](../../v0/concepts/agent.md) §"Scoping the embedding model" lines 342–344 (platform-level config fixed at org-bootstrap) + line 344 ("Model change is an admin event") + §"Identity Node Content" line 332 (`embedding: Vec<f32>` field shape; dim configurable, default 1536).
- **Prerequisites**: none (independent of CH-28/CH-29/CH-30 messaging track).
- **Deliverables** **per Q7 user-lock**: NEW `EmbeddingProviderConfig` domain node (separate from M2 `ModelRuntime` — the audit §7.4 + Q7 lock chose this routing); migration `0020_embedding_provider_config.surql` adding the node table + RELATION edge from Org to chosen config; admin endpoint to set provider (creates Auth Request per concept doc line 344 "switch triggers batch re-embed via Auth Request"); batch re-embed handler invoked on Approved Auth Request; `derive_embedding_from_self_description` adapter (with mock provider at v0; real OpenAI/Voyage integration is a follow-on if user opts to wire); 5+ acceptance scenarios at `acceptance_m6_embedding_provider.rs` (set provider + Auth Request fires + batch re-embed updates Identity.embedding + denial cascades + mock provider deterministic output).
- **K8s posture**: A3 (pod-local resource) — embedding model invocation is a network call out of pod; A4 (migration runner) adds 1 new migration; A7 (audit hash-chain) — provider-change is auditable per concept-doc spec.
- **phi-core leverage**: zero direct phi-core types consumed; the mock embedding provider mirrors phi-core's `MockProvider` pattern for testability (no phi-core import — it's a separate concern).
- **Unblocks**: CH-37 (a05 profile editor + grants needs Identity.embedding populated to render the similarity surface).

**CH-32 — C-M6-1 Memory tier: MemoryStore trait + default impl + retrieval gate** · ⚠HIGH · 3–4d
- **Drifts closed**: **M6-DEFERRED-01** (carryover C-M6-1 from base build plan lines 306–315; Q8 user-locks ship trait + default impl + retrieval gate together); **D-new-16** (recall/store/delete actions); **D-new-28** (memory_type enum vs tag — decided at chunk plan-open).
- **Concept docs**: [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) §C-M6-1 lines 306–315 (3 sub-requirements: (i) trait contract + (ii) multi-tag ownership ALREADY ALIGNED per audit §3.1 / §4.B B.2 / `listeners.rs:709-723` + (iii) permission-over-time retrieval); [`system-agents.md`](../../v0/concepts/system-agents.md) §"Memory Extraction Agent" §"Behaviour" lines 80–117 + §"Allocation Rules" lines 119–135; [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"Supervisor Extraction as Two Standard Grants" line 155+ + §"Memory as Resource Class" + §"tag vocabulary"; [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Authority Chain" (retrieval composes with authority traversal).
- **Prerequisites**: none structural (parallelizable with CH-29/CH-30 + CH-31 + CH-38 per Q3 user-lock; M6+ Memory body upgrade CH-36 consumes this trait).
- **Deliverables** **per Q8 user-lock** (ship trait + default impl + retrieval gate in ONE chunk): NEW `domain/src/memory_contract.rs` with `trait MemoryStore { async fn extract(...) -> Vec<Memory>; async fn retrieve(query, viewer) -> Vec<Memory>; async fn tag(memory, tags) -> Result<()>; }`; in-tree default impl (SurrealDB-backed; reuses CH-21 `MemoryExtractionListener` for `extract` path with heuristic v0); NEW `Action::Recall` consumer at `permissions::builders::build_memory_recall_manifest(viewer, query)`; retrieval gate composes `step_2_resolve_grants` (CH-25 owner rule + CH-14 authority-chain walker) — grants revoked AFTER extraction forfeit IMMEDIATE read access per concept-doc line 309; NEW `server/src/platform/memory/` directory with retrieval HTTP endpoints (`GET /api/v0/memory/recall?tag_intersection=...&viewer=...`); NEW `phi memory recall` CLI subcommand (`modules/crates/cli/src/commands/memory.rs`); 8+ acceptance scenarios (`acceptance_m6_memory_recall.rs`: tag-intersection query + grant-revoked-after-extraction blocks recall + authority-chain delegation walk + multi-tag ownership round-trip + Recall verb gating + tag mutation + viewer-as-actor pattern + storage-backend swap-out via trait); **storage-backend swap-out criterion**: third-party impls (Chroma, Weaviate, LanceDB) satisfy the trait but are NOT shipped at M6 (deferred to M6+ optional follow-on per audit §8 line 198).
- **K8s posture**: A1 (in-process state via SurrealDB default impl); A3 (pod-local resource — vector store is in-DB at v0; cloud backends deferred); A4 (migration `0021_memory_recall_indexes.surql` for tag-intersection query); A5 (trait-shape requirement — the `MemoryStore` trait IS the K8s-prep trait surface for backend swap-out, conforming to ADR-0033 D33.4 criteria).
- **phi-core leverage**: zero new phi-core types consumed; the retrieval surface may compose `phi_core::types::tool::AgentTool` if a `retrieve_memory` LLM tool ships at CH-32 (otherwise deferred to a CH-32 follow-on or to the M6-DEFERRED-04 LLM-supervisor-body carve-out chunk per §6 Q1); consult leverage map at plan-open.
- **Unblocks**: M6-DEFERRED-04 LLM supervisor body (consumes the trait — carve-out routing per §6 Q1); CH-32 retrieval gate must close BEFORE CH-37 a05 grants surface (if a05 surfaces memory-recall capability — Q3 routing pending CH-37 plan-open verification).

### a01 + agent-self-service consumer tier (Q2 strict serial gate)

**CH-33 — a01 Inbox/Outbox UI + LLM tool surface (`read_inbox`, `send_message`)** · ⚠HIGH · 3–4d
- **Drifts closed**: a01 PARTIAL → ALIGNED (per audit §4.A row 1).
- **Concept docs**: [`requirements/agent-self-service/a01-my-inbox-outbox.md`](../../v0/requirements/agent-self-service/a01-my-inbox-outbox.md) (CONCEPTUAL; 12-section page spec — §10 API Contract + §11 Acceptance Scenarios at lines 76+92); [`ontology.md`](../../v0/concepts/ontology.md) lines 83–87 (Inbox/Outbox composites); [`agent.md`](../../v0/concepts/agent.md) line 21 (messaging-as-information-flow framing).
- **Prerequisites**: **CH-29** (substrate), **CH-30** (routing).
- **Deliverables**: HTTP handlers `GET /api/v0/agents/:id/inbox` + `GET /api/v0/agents/:id/outbox` + `POST /api/v0/agents/:id/messages` + `PATCH /api/v0/messages/:id/read`; LLM tool surface (`read_inbox` + `send_message` as phi-core `AgentTool` impls — first agent-self-service-tier consumers of phi-core's AgentTool trait); Next.js page `app/(agent)/inbox/page.tsx` rendering messages list with priority badges + read/unread state + tag filters; web-UI write modal for `send_message`; 5+ acceptance scenarios at `acceptance_m6_a01_inbox_outbox.rs` (matches a01 §11 scenarios).
- **K8s posture**: A1 (read-only HTTP); A5 (trait-shape: AgentTool impls are stateless per phi-core convention).
- **phi-core leverage**: **direct consumption** of `phi_core::types::tool::AgentTool` for `read_inbox` + `send_message` tools (first a-tier consumer; reuses M5 session-launch tool-resolver from C-M5-4). Plan §3.A leverage map must enumerate the wraps.
- **Unblocks** **per Q2 user-lock**: blocks {CH-34 a02, CH-35 a03, CH-36 a04} — all three depend on a01 inbox-routing being operational for cross-page notification routing.

**CH-34 — a02 My Auth Requests (inbound/outbound query + slot-action handlers + Next.js page)** · ⚠HIGH · 2.5–3.5d
- **Drifts closed**: a02 GAP → ALIGNED (per audit §4.A row 2). Domain logic (AR state machine + 2-of-2 Shape B slots) shipped at CH-10 + CH-18; this chunk is 100% HTTP + UI on top.
- **Concept docs**: [`permissions/02-auth-request.md`](../../v0/concepts/permissions/02-auth-request.md) §"Per-State Access Matrix" lines 12–30 (state machine); [`requirements/agent-self-service/a02-my-auth-requests.md`](../../v0/requirements/agent-self-service/a02-my-auth-requests.md) (CONCEPTUAL; §10 API Contract + §11 Acceptance Scenarios).
- **Prerequisites** **per Q2 user-lock**: **CH-33** (a01 must close first for inbox-based AR-event notification routing).
- **Deliverables**: HTTP handlers `GET /api/v0/agents/:id/auth-requests/inbound` + `outbound` + `POST /api/v0/auth-requests/:id/{approve,deny,reconsider,escalate}` (slot-actions reuse the M5/CH-10 state-machine transitions); Next.js page `app/(agent)/auth-requests/page.tsx` rendering AR list with state badges + slot-progress + action buttons; cross-page notification: state transitions insert AgentMessages in interested-party inboxes via CH-30 routing; 5+ acceptance scenarios per a02 §11.
- **K8s posture**: A1 (read-only + state-transition write); A7 (audit hash-chain — every AR transition is logged per M1 audit conventions).
- **phi-core leverage**: zero direct phi-core types consumed (AR state machine is pure baby-phi governance).
- **Unblocks**: nothing (sibling to CH-35 + CH-36 in the post-a01 parallel band).

**CH-35 — a03 My Consent Records (query + action handlers + Next.js page)** · ⚠HIGH · 2–3d
- **Drifts closed**: a03 PARTIAL → ALIGNED (per audit §4.A row 3). Consent state machine shipped at CH-09+CH-10 + sweeper at CH-11; per-policy minter helpers at `domain::consents::minters`. This chunk is 100% HTTP + UI.
- **Concept docs**: [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) §"Consent state machine" lines 353–426; [`requirements/agent-self-service/a03-my-consent-records.md`](../../v0/requirements/agent-self-service/a03-my-consent-records.md) (CONCEPTUAL).
- **Prerequisites** **per Q2 user-lock**: **CH-33** (a01 must close first for consent-event notification routing into requestor inboxes).
- **Deliverables**: HTTP handlers `GET /api/v0/agents/:id/consents` + `POST /api/v0/consents/:id/{acknowledge,decline,revoke}` (revoke cascades via engine logic already in place); Next.js page `app/(agent)/consents/page.tsx`; cross-page notification (state transitions insert AgentMessages in inboxes per CH-30 routing); 5+ acceptance scenarios per a03 §11.
- **K8s posture**: A1 (read-only + state-transition write); A7 (audit hash-chain).
- **phi-core leverage**: zero direct phi-core types consumed.
- **Unblocks**: nothing.

**CH-36 — a04 My Work (Task + Session query handlers + concurrency-cap enforcement + Next.js page)** · ⚠HIGH · 3–4d
- **Drifts closed**: a04 GAP → ALIGNED (per audit §4.A row 4). Task node + ASSIGNED_TO edge shipped at M4; Session + RUNS_SESSION edge shipped at M5/P4; Task status transitions shipped at M4. Concurrency-cap query is the key NEW addition.
- **Concept docs**: [`project.md`](../../v0/concepts/project.md) §"Task (Node Type — Optional Decomposition)" lines 107–149 (4 subsections: Properties + Status Flow + Edges + Who Creates Tasks); [`agent.md`](../../v0/concepts/agent.md) §"Parallelized Sessions" lines 209–221 (concurrency cap; cap-check is the new query); [`requirements/agent-self-service/a04-my-work.md`](../../v0/requirements/agent-self-service/a04-my-work.md) (CONCEPTUAL; 3 acceptance scenarios per audit §4.A; §10 API Contract). **Per Q3 user-lock**: display-only — a04 does NOT depend on C-M6-1 memory contract; renders existing Task + Session graph state without memory-recall integration.
- **Prerequisites** **per Q2 user-lock**: **CH-33** (a01 must close first for task-blocked + session-error notification routing to project lead's inbox).
- **Deliverables**: HTTP handlers `GET /api/v0/agents/:id/tasks` + `GET /api/v0/agents/:id/sessions` (scoped to viewer-agent via authority-chain traversal); `PATCH /api/v0/tasks/:id/status` (state-flow validation per project.md §"Task Status Flow"); concurrency-cap query (joins active Session + AgentProfile.parallelize at session-launch — new repo method `count_active_sessions_for_agent` was a stub at M4 and flipped to live query at M5 via C-M5-5; a04 now CONSUMES the live query, surfacing cap-remaining metric); Next.js page `app/(agent)/work/page.tsx` with my-tasks panel + my-sessions panel + concurrency-cap status; 5+ acceptance scenarios per a04 §11.
- **K8s posture**: A1 (read-only + state-transition write); A4 (no new migration; reuses M4/M5 schemas); A7 (audit hash-chain).
- **phi-core leverage**: indirect — Task is pure baby-phi governance; Session shape consumes `phi_core::session::model::{Session, LoopRecord, Turn}` already wrapped at C-M5-3 (no NEW phi-core import).
- **Unblocks**: nothing (terminal post-a01 page).

### Embedding-dependent agent-self-service tier (Q4 strict prereq)

**CH-37 — a05 My Profile + Grants (profile editor + grants traversal + authority-chain expansion endpoint + Next.js page)** · ⚠HIGH · 3–4d
- **Drifts closed**: a05 PARTIAL → ALIGNED (per audit §4.A row 5). Identity node shipped at CH-16; authority-chain traversal shipped at CH-14; embedding populated at CH-31.
- **Concept docs**: [`agent.md`](../../v0/concepts/agent.md) §"Identity (Emergent, Event-Driven)" lines 270–344 (4-field Identity + embedding model) + §"Soul" lines 160–169 (immutable nature; sharing semantics per CH-28 cardinality redesign); [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Authority Chain" (HOLDS_GRANT + DESCENDS_FROM walk per NFR-observability R6); [`requirements/agent-self-service/a05-my-profile-and-grants.md`](../../v0/requirements/agent-self-service/a05-my-profile-and-grants.md) (CONCEPTUAL).
- **Prerequisites** **per Q4 + Q2 user-lock**: **CH-31** (M6-DEFERRED-03 embedding provider closed — embedding-derivation must populate Identity.embedding before a05 surfaces it); **CH-33** (a01 must close first per Q2 strict serial gate); **CH-28** (AgentProfile cardinality redesign — a05 profile editor surfaces shared-profile awareness).
- **Deliverables**: HTTP handlers `GET /api/v0/agents/:id/profile` + `PATCH /api/v0/agents/:id/profile/self_description` (mutation triggers re-embed via CH-31 helper) + `GET /api/v0/agents/:id/grants` (HOLDS_GRANT traversal + authority-chain expansion endpoint per concept-doc R6 observability requirement); Next.js page `app/(agent)/profile/page.tsx` with profile editor + grants list + authority-chain visualizer; 5+ acceptance scenarios per a05 §11.
- **K8s posture**: A1 (read-only + profile-edit write); A4 (no new migration); A7 (audit hash-chain — profile edits + grants traversal are auditable).
- **phi-core leverage**: indirect — `phi_core::agents::profile::AgentProfile` is wrapped at the baby-phi domain layer (per CH-28 redesign); embedding integration via CH-31 doesn't touch phi-core directly.
- **Unblocks**: nothing (terminal a-tier page).

### M6-deferred follow-on tier (parallel with consumer tier)

**CH-38 — M6-DEFERRED-RESOLVERS-WIRING (F3.b trait re-shape per Q6 user-lock)** · MEDIUM · 3–4d
- **Drifts closed**: [`D-CH27-FOLLOWUP-01`](../../v0/implementation/m5_3/drifts/D-CH27-FOLLOWUP-01.md) (LOW; deferred at CH-27 P-SEAL with explicit `M6-DEFERRED-RESOLVERS-WIRING` allocation per ADR-0062 §D62.3 F3.a LOCKED).
- **Concept docs**: [`core-philosophy.md`](../../v0/concepts/core-philosophy.md) lines 16, 28, 29 (ownership permission-gating principle); no standalone concept doc for resolver-actor routing (rationale in the drift body).
- **Prerequisites**: none structural (independent of agent-self-service chunks; **per Q6 user-lock + audit §4.1**, can run parallel to CH-29..CH-37 after CH-28 closes).
- **Deliverables** **per Q6 user-lock (F3.b trait re-shape, NOT F3.c HTTP wrapper)**: extend 4 resolver trait defs at `domain/src/events/listeners.rs:244-507` with `actor: Option<AgentId>` parameter; refactor 4 `Repo*Resolver` impls in `store/src/repo_impl.rs` to thread actor; refactor ~6 static-test stubs to match the new signatures; refactor 1 wiring site at `listeners.rs:276-340` to pass `check_permission` actor; emit `check_permission` calls at each resolver when actor is `Some` (advisory at M6 first, then blocking per ADR-0062 §D62.1 wire-tier pattern at follow-on); 4+ acceptance scenarios at `acceptance_m6_resolvers_actor_passthrough.rs`.
- **K8s posture**: A1 (in-process state); A5 (trait-shape requirement — broad cascade through 4 traits + 4 impls + 6 stubs; ADR-0033 §D33.4 criteria to re-verify).
- **phi-core leverage**: zero new phi-core types consumed.
- **Unblocks**: nothing critical (standalone resolver-tier hardening; brings the M5.3 D-CH27-FOLLOWUP-01 drift to closure).

---

## §2 — Carryovers from M5 accounted for

The base build plan §M6 lines 302–315 names **C-M6-1** as the singular M4→M5→M6 carryover for M6, with 3 sub-requirements:

- **C-M6-1 (i) — Well-defined memory interface contract (MemoryStore trait)** → **CH-32** (per Q8 user-lock ship trait + default impl + retrieval gate in one chunk).
- **C-M6-1 (ii) — Ownership via multi-tag** → **ALREADY ALIGNED** per audit §3.1 + §4.B B.2: Memory node has `tags: Vec<String>` at `domain/src/model/nodes.rs:1106-1111`; `build_memory_tags` helper at `domain/src/events/listeners.rs:709-723` derives the 4 governance tags (`agent:`, `session:`, `project:`, `org:`) per ADR-0040 §D40.2. **No new chunk needed for this sub-requirement.**
- **C-M6-1 (iii) — Permission-over-time retrieval** → **CH-32** (live Permission Check at retrieval time composes M1 engine + CH-14 authority-chain + CH-25 owner-grant + Q7 future M7 `DESCENDS_FROM` walk).

No M6 carryover commitments orphaned.

**Note on M6-DEFERRED-04 routing**: the LLM supervisor body upgrade (deferred from CH-21 per ADR-0040 §D40.7) is brought in-milestone **at CH-32's retrieval surface** only if user opts to bundle; otherwise it routes to its own chunk post-CH-32. The alignment audit §5.2 identified this as a routing ambiguity. **Open question Q1 in §6 below surfaces the decision.**

---

## §3 — Remaining post-M6 scope (deferred-milestone scope markers, preserved verbatim from prior forward-scope §3 minus markers brought in-milestone)

Scope markers for drifts explicitly deferred past M6 close. Each maps to its target milestone per base plan [§M7 / §M7b](../build/build-plan-v01-36d0c6c5.md). Per-milestone plans will produce their own detailed chunks. **Brought in-milestone by this M6 forward-scope (removed from prior §3 markers)**: M6-DEFERRED-01 (carryover C-M6-1 → CH-32) + M6-DEFERRED-02 (→ CH-29 + CH-30) + M6-DEFERRED-03 (→ CH-31) + M6+-OPEN-01 (→ CH-28 per Q9 user-lock). **Brought in-milestone via D-CH27-FOLLOWUP-01 closure** (cross-referenced in prior §2.5 post-M5.3 closure block at L281, NOT prior §3): M6-DEFERRED-RESOLVERS-WIRING (→ CH-38).

### M6-DEFERRED-04 — Memory-extraction LLM supervisor body (carried forward; chunk-zero open question — see §6 Q1)

- **Drifts**: covered by ADR-0040 §D40.7 Out-of-Scope block (no separate drift file — surfaced via concept-`system-agents.md` §"Memory Extraction Agent — Behaviour 2/3/4" + concept-`permissions/05-memory-sessions.md` §"Supervisor Extraction" silent-in-code rows; CH-21 ships heuristic v0).
- **Cross-ref**: CH-21 / ADR-0040; concept-`system-agents.md` §"Memory Extraction Agent" (Behaviour + Grants + Allocation Rules); concept-`permissions/05-memory-sessions.md` §"Supervisor Extraction"; ADR-0041 (audit class for the events the LLM body will continue to emit).
- **Prerequisite chain**: **CH-32** (MemoryStore trait closure must precede; LLM body consumes the trait's `store_memory` path).
- **Target**: M6 if user opts to bundle (see §6 Q1); otherwise carries forward to M7 or a dedicated M6+ follow-on chunk. **Decision-maker: user at CH-32 plan-open or sub-chunk-zero gate.**

### M6-or-M7-DEFERRED — Token economy fields + Worth formula

- **Drifts**: **D-new-27** (rating_window, total_tokens_earned/consumed, Worth).
- **Target**: whichever milestone introduces contracts/bidding. Per audit §9.2 phase-planner may "surface as a 'decide at M7 plan-open' marker in M6 forward-scope §3" — this row IS that surface. **Decision-maker: user at M7 plan-open.**

### M7-DEFERRED-01 — Channel schema enrichment

- **Drifts**: **D-new-24** (Channel address/status/priority/metadata + WebUI/API/SMS/Custom kinds).
- **Target**: M7 operator-surface polish.

### M7-DEFERRED-02 — Task node materialization

- **Drifts**: **D-new-26** (Task full field set + 7-state status flow).
- **Target**: M7 or later (task/bidding flow). **Note**: a04 (CH-36) renders the existing CH-19 Task node minimal shape; the full 7-state status flow remains M7-tier work.

### M7b-DEFERRED-01 — AuthRequest retention 2-tier storage

- **Drifts**: **D-new-15** (archive transition + `inspect_archived` retrieval gate).
- **Target**: M7b production-hardening milestone.

### M7b-DEFERRED-02 — K8s microservices carve-out (added 2026-04-24 by CH-K8S-PREP)

- **Drifts**: none from the M5.1 catalogue (all 8 items are sourced from CH-K8S-PREP prep refactors, not concept-vs-code drift).
- **Strategic input**: [`v0/implementation/m7b/architecture/k8s-microservices-readiness.md`](../../v0/implementation/m7b/architecture/k8s-microservices-readiness.md) — 8 K8s blockers, 7 microservice boundaries, 10-step migration order, ~35 engineer-day rough estimate.
- **Tactical input**: [`v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md`](../../v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) — 8 specific items (CHK8S-D-01 through CHK8S-D-08).
- **Source chunk plan**: [`forward-scope/k8s-microservices-readiness-plan-ab49f22b.md`](./k8s-microservices-readiness-plan-ab49f22b.md).
- **Target**: M7b production-hardening milestone.

---

## §4 — Chunk dependency graph

Simple `A → B` means A must close before B can open cleanly. The graph honors all 9 user-locked decisions from the alignment-audit §4.0 + §9.

### Critical path (Q2 strict serial gate enforced)

```
                 M5.3 CLOSE (CH-27 0edcaba9, 2026-05-18)
                              │
                              v
                CH-28 — AgentProfile cardinality 1:1 → N:1 (Q9)
                              │
        ┌─────────────────────┼───────────────────┬───────────────────┐
        v                     v                   v                   v
   CH-29 — messaging       CH-31 — embedding   CH-32 — Memory tier   CH-38 — resolvers
   substrate (Q5 split)    provider (Q7)       trait+default+gate    actor-passthrough
        │                     │                 (Q8 + Q3 parallel)   (Q6 trait re-shape)
        v                     │                                       (independent;
   CH-30 — messaging          │                                        parallel-band)
   routing (Q5)               │
        │                     │
        v                     │
   CH-33 — a01 UI/tools       │
   (Q1 + Q2 gate)             │
        │                     │
        ├───────┬───────┬─────┤
        v       v       v     v
   CH-34   CH-35   CH-36   CH-37 — a05 (Q4 strict prereq: CH-31)
   a02 AR  a03     a04
           Consent My Work
```

### Critical-path chain (sequential)

`CH-28 → CH-29 → CH-30 → CH-33 → {CH-34, CH-35, CH-36}` (a04 + a05 + a03 + a02 fan-out post-a01).

Side-chain: `CH-31 → CH-37` (embedding provider gates a05 per Q4 user-lock).

### Parallelizable bands

- **Foundation parallel band (post-CH-28)**: CH-29 + CH-31 + CH-32 + CH-38 can run concurrently (no inter-dependencies; CH-28 unlocks all four).
- **Post-CH-33 fan-out**: CH-34, CH-35, CH-36 run concurrently (no inter-dependencies; CH-33 is the strict gate per Q2 user-lock).
- **CH-37 join**: requires CH-31 (Q4 strict prereq) + CH-33 (Q2 strict prereq) + CH-28 (cardinality awareness in profile editor) — joins the parallel bands.

### Cross-milestone edges (M6 outputs that feed M7 / M7b)

- **CH-30** routing-tier audit hash-chain symmetry → M7 audit-event schema finalization (NFR observability).
- **CH-32** MemoryStore trait → M7+-DEFERRED third-party impls (Chroma/Weaviate/LanceDB) + M6-DEFERRED-04 LLM body if not bundled in CH-32.
- **CH-38** resolvers actor-passthrough → M7+ advisory→blocking tightening (mirrors CH-26→CH-27 wire-tier pattern per ADR-0062 §D62.1).
- **CH-36** a04 concurrency-cap query → M7-DEFERRED-02 (Task 7-state full status flow).

### Effort totals (M6 round)

- **M6 in-milestone scope** (CH-28 through CH-38): **28–38 engineer-days** sum of upper-bands.
- **Critical-path serial walk**: CH-28 (4d) + CH-29 (3.5d) + CH-30 (3.5d) + CH-33 (4d) + max(CH-34/35/36) (4d) + CH-37 (4d) = **~23d serial**. With parallelization of foundation tier + post-a01 fan-out, achievable in **~12–15 calendar working days assuming continuous engineer attention** (effort total still ~28–38 ed; parallelization compresses calendar, not effort).

---

## §5 — Per-chunk scope summary table

| Chunk | Title | Severity | Effort | Concept docs | Prerequisites | Closes-M6? |
|---|---|---|---|---|---|---|
| CH-28 | AgentProfile cardinality 1:1 → N:1 redesign | HIGH | 3–4d | agent.md §Soul L160–169, ontology.md L98 cardinality row | — | yes |
| CH-29 | M6-DEFERRED-02 messaging substrate | HIGH | 2.5–3.5d | ontology.md L83–87 + L229, permissions/05 L590–618, agent.md L21 | CH-28 | yes |
| CH-30 | M6-DEFERRED-02 messaging routing | HIGH | 2.5–3.5d | agent.md L21, permissions/05 L590–618 | CH-29 | yes |
| CH-31 | M6-DEFERRED-03 EmbeddingProviderConfig | HIGH | 2–3d | agent.md L332 + L342–344 | — | yes |
| CH-32 | C-M6-1 Memory tier (trait + default + retrieval gate) | HIGH | 3–4d | build-plan §306–315, system-agents L80–135, permissions/05 L155+, permissions/04 §Authority Chain | — | yes |
| CH-33 | a01 Inbox/Outbox UI + LLM tool surface | HIGH | 3–4d | requirements/a01, ontology.md L83–87, agent.md L21 | CH-29, CH-30 | yes |
| CH-34 | a02 My Auth Requests UI + handlers | HIGH | 2.5–3.5d | permissions/02 L12–30, requirements/a02 | CH-33 | yes |
| CH-35 | a03 My Consent Records UI + handlers | HIGH | 2–3d | permissions/06 L353–426, requirements/a03 | CH-33 | yes |
| CH-36 | a04 My Work (Task + Session + cap) | HIGH | 3–4d | project.md L107–149, agent.md L209–221 §Parallelized Sessions, requirements/a04 | CH-33 | yes |
| CH-37 | a05 My Profile + Grants | HIGH | 3–4d | agent.md L270–344 + §Soul L160–169, permissions/04 §Authority Chain, requirements/a05 | CH-28, CH-31, CH-33 | yes |
| CH-38 | M6-DEFERRED-RESOLVERS-WIRING (F3.b trait re-shape) | MEDIUM | 3–4d | core-philosophy.md L16, L28, L29 | — | yes |

**Total upper-band: 38 engineer-days. Total lower-band: 28 engineer-days. Range: 28–38 ed for M6 close (11 chunks).**

---

## §6 — Open questions (decisions the user must lock before chunk-planning opens)

The 9 alignment-audit user-locks (§9 of the audit) covered the macro-sequencing + chunk-shape decisions. The questions below are **per-chunk plan-open decisions** that emerge from the decomposition. The phase-planner does NOT recommend a default — each must be locked at the cited chunk's plan-open by `chunk-planner` via AskUserQuestion.

### Q1 — M6-DEFERRED-04 LLM supervisor body routing (decision-point: CH-32 plan-open)

- **Question**: at CH-32 (C-M6-1 Memory tier), does the LLM supervisor body upgrade (M6-DEFERRED-04) bundle in-chunk OR carve out as a separate post-CH-32 chunk (CH-39)?
- **Sub-options**:
  - **(a) Bundle in CH-32**: ship the trait + default impl + retrieval gate + LLM body upgrade all together. **Pro**: single coherent memory delivery; **Con**: pushes CH-32 effort to 6–8 ed (above 1.5× upper-band threshold per chunk-planner v21 split-fallback criterion).
  - **(b) Carve out as CH-39**: ship CH-32 with heuristic v0 retained + LLM body as standalone follow-on. **Pro**: keeps CH-32 within 3–4 ed band; **Con**: extends M6 chunk count to 12.
- **Phase-planner recommendation**: **(b) carve-out** — keeps CH-32 within bounds; M6-DEFERRED-04 has its own substrate (CH-21 heuristic) that won't regress while the LLM body lands as a follow-on.
- **Target-decision-point**: user-decidable at CH-32 plan-open via AskUserQuestion (chunk-planner v21 F1.b split-fallback applies).

### Q2 — CH-29 + CH-30 split-vs-combine final ratification (decision-point: CH-29 plan-open)

- **Question**: Q5 audit-lock effected the split into 2 chunks (substrate + routing); does that split hold at CH-29 plan-open OR does the chunk-planner combine if it determines effort fits a single chunk?
- **Sub-options**:
  - **(a) Hold split**: CH-29 substrate + CH-30 routing, per Q5 audit-lock.
  - **(b) Combine**: CH-29 ships both substrate + routing (5–7 ed; above 1.5× threshold but with explicit user-lock to combine).
- **Phase-planner recommendation**: **(a) hold split** — Q5 user-lock already chose this routing; revisit only if mid-plan-mode evidence emerges that the routing tier is trivial atop the substrate.
- **Target-decision-point**: phase-planner-resolvable now (Q5 already locked); restated here only to flag that chunk-planner v21 split-fallback may surface this at CH-29 plan-mode.

### Q3 — Concurrency-cap surface placement (decision-point: CH-36 plan-open)

- **Question**: does the concurrency-cap query (joins active Session + AgentProfile.parallelize) surface ONLY at a04 (CH-36) or ALSO at the session-launch handler (block launch when cap-exceeded)?
- **Sub-options**:
  - **(a) a04 surface only (display-only)**: query exists for a04 reporting; cap is NOT enforced at session-launch (existing soft-cap stays soft).
  - **(b) Both surfaces (display + enforcement)**: a04 displays cap-remaining; session-launch enforces hard-block at cap-exceeded (409 error).
- **Phase-planner recommendation**: **(a) display-only at CH-36** — enforcement is a separate concern, route to M7+ if user prioritises; CH-36 stays display-only consistent with Q3 audit-lock spirit.
- **Target-decision-point**: user-decidable at CH-36 plan-open via AskUserQuestion.

### Q4 — Embedding provider mock-vs-real lock (decision-point: CH-31 plan-open)

- **Question**: at CH-31 (EmbeddingProviderConfig + admin lifecycle), does the chunk ship with mock-only provider OR also wire one real provider (OpenAI `text-embedding-3-small` is the audit-cited default)?
- **Sub-options**:
  - **(a) Mock-only**: ship the config + mock provider for testability + deterministic Identity.embedding output. Real provider wires as a follow-on chunk.
  - **(b) Mock + 1 real provider**: ship the config + mock + OpenAI integration (with API-key handling + retry policy + cost-budget guard).
- **Phase-planner recommendation**: **(a) mock-only** — keeps CH-31 within 2–3 ed; real-provider integration carries cost-budget + secret-handling concerns better routed to M7b production-hardening.
- **Target-decision-point**: user-decidable at CH-31 plan-open via AskUserQuestion.

### Q5 — a02/a03/a04 page-bundling (decision-point: post-CH-33 close)

- **Question**: do a02 + a03 + a04 land as 3 separate chunks (CH-34 + CH-35 + CH-36) per the decomposition above, OR combine into 1–2 chunks if the user wants a single "agent self-service consumer surface" delivery?
- **Sub-options**:
  - **(a) 3 separate chunks**: per the decomposition above; allows per-page audit-fix loop independence.
  - **(b) Combine a02 + a03 into one chunk** (audit §4.A flagged a02 + a03 bundling as "optional"); keeps a04 separate.
  - **(c) Combine all three (a02 + a03 + a04)** into one consumer-surface chunk: 7–10 ed (above 1.5× threshold but explicit user-lock).
- **Phase-planner recommendation**: **(a) 3 separate chunks** — preserves per-page acceptance-scenario fidelity + independent audit-fix loops.
- **Target-decision-point**: user-decidable post-CH-33 close (when a01 substrate is shipped + the user can re-evaluate consumer-tier bundling).

### Q6 — CH-38 schedule placement (decision-point: chunk-zero gate)

- **Question**: when does CH-38 (M6-DEFERRED-RESOLVERS-WIRING F3.b trait re-shape) ship in the M6 calendar?
- **Sub-options**:
  - **(a) Parallel with foundation tier (alongside CH-29..CH-32)**: ship CH-38 as a parallel-band chunk; no dependencies on agent-self-service track.
  - **(b) Defer to post-CH-37 close (M6 terminal)**: ship CH-38 as the last M6 chunk; permits any cross-cutting handler edits in CH-33..CH-37 to land first.
  - **(c) Defer entirely to M7**: re-route CH-38 to M7+ if user prioritises agent-self-service surface over resolver hardening.
- **Phase-planner recommendation**: **(a) parallel with foundation tier** — keeps M5.3 carve-out closure clean (D-CH27-FOLLOWUP-01 closes early in M6); cascade through 4 traits + 4 impls + 6 stubs has no cross-cutting handler dependencies per audit §4.C C.2.
- **Target-decision-point**: user-decidable at chunk-zero gate (the M6 plan-mode opening; pre-CH-28).

---

## §7 — Open implications (drifts the M6 decomposition surfaces that map to M7+ work)

### 7.1 — Resolvers advisory→blocking tightening (mirrors CH-26→CH-27 wire-tier pattern)

CH-38 ships resolver actor-passthrough at the **advisory layer** (emit `check_permission` calls when actor is `Some`; do not deny). The tightening from advisory to blocking — mirror of CH-26→CH-27's wire-tier evolution per ADR-0062 §D62.1 — is a candidate M7 follow-on. Surfaces in §3 as a new marker post-M6 close: `M7-DEFERRED-RESOLVERS-BLOCKING`.

### 7.2 — MemoryStore trait third-party impls (Chroma / Weaviate / LanceDB)

CH-32 ships the trait + in-tree SurrealDB default impl. Third-party impls satisfy the trait contract without further phi-core change but are NOT shipped at M6 per audit §8. If user opts to onboard a third-party impl, it routes as an `M6+-OPTIONAL` chunk or M7+ depending on prioritisation.

### 7.3 — Real embedding provider wiring + cost-budget guard

CH-31 ships mock-only per Q4 phase-planner recommendation. Wiring a real provider (OpenAI / Voyage / Cohere) introduces cost-budget tracking + API-key vault + retry-on-rate-limit semantics — all of which belong in M7b production-hardening. Surfaces as `M7b-DEFERRED-EMBEDDING-REAL`.

### 7.4 — Concurrency-cap session-launch enforcement

CH-36 ships display-only per Q3 phase-planner recommendation. Hard-blocking session-launch when cap-exceeded is a separate enforcement concern. Routes to M7 NFR-performance hardening or its own follow-on.

### 7.5 — Token economy fields + Worth formula

Carried forward in §3 (M6-or-M7-DEFERRED) per audit §9.2. Decision-point: M7 plan-open. Surfaces here only as the chunks that touch agent-self-service surface (CH-37 a05 grants traversal) may interact with Worth-display in a future iteration.

### 7.6 — AgentProfile shared-profile audit-clarity follow-on

CH-28 ships the cardinality flip + override-routing decision. Post-CH-28, the audit-event narrative for profile mutations changes: "one profile change visible to N agents." This may require audit-event schema enrichment (per-event affected-agent-list). Routes to M7 NFR-observability if surfaced.

---

## Post-this-document next steps

1. **User reviews this M6 forward-scope** + locks any open §6 questions that are pre-decidable (chunk-zero choice; CH-38 schedule placement).
2. **First M6 implementation chunk** — user selects CH-28 (the chunk-graph head) at chunk-zero gate; per-chunk detailed plan drafted using [`per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md); approved via ExitPlanMode.
3. **Iterate** — repeat step 2 per chunk until all M6 chunks (CH-28 through CH-38) seal and M6 tag ships.
4. **Post-M6**: re-run alignment audit to confirm a01–a05 + C-M6-1 closure; surface any third-order ripple effects to M7 plan-open.
