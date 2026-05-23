<!-- Last verified: 2026-05-23 by Claude Code (CH-28b P4 — NEW §3 Deferred catalogue row `D-PHICORE-08-FOLLOWUP-01 — Composition I adoption` inserted between M7b-DEFERRED-02 and D-CH28-FOLLOWUP-01 rows; documents the phi-core 0.8.0 opt-in braking layer adoption deferral; allocation `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder per F3.a planner-rec lock + ADR-0064 §D64.5; cycle hex `d5b776ac`.) -->
<!-- Last verified: 2026-05-20 by Claude Code (initial authoring per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`; cross-chunk product-trajectory tracker; baby-phi v0.1 scope). -->

# baby-phi v0.1 — Feature inventory

## §1 — Purpose

This document is the **canonical cross-chunk product-trajectory tracker** for the baby-phi v0.1 build. It lets the user (or a planner / orchestrator / reviewer) answer the following questions at ANY planning gate:

- "What user-facing capabilities ship in v0.1?"
- "Which chunk(s) deliver each capability?"
- "What sub-aspects of each capability are deferred — to what target — and what does the user observe in the deferred state vs the final state?"
- "What's the cross-chunk dependency chain for capability X?"

The inventory is a **product-level view** of the build, distinct from the engineering-level views found in:
- `docs/specs/plan/build/build-plan-v01-36d0c6c5.md` — milestone-level engineering scope (M0..M8).
- `docs/specs/plan/forward-scope/*.md` — per-milestone chunk-level decomposition with effort + dependency engineering details.
- `docs/specs/plan/build/_cycle-index.md` — per-cycle paperwork audit ledger.

The product-level granularity here lives between the milestone (too coarse — entire milestones bundle many features) and the chunk (too fine — chunks bundle infrastructure with delivery). Each row in §2 is one **user-facing capability** the end user can identify.

**Update cadence**: re-author / extend when (a) a new milestone forward-scope ships (phase-planner emits new chunk rows), (b) a drift / `M*-DEFERRED-NN` marker shifts allocation, (c) a chunk's user-visible delivery diverges materially from the §2 row. See §5 for the full revisit-triggers list.

**Project-agnostic note**: this doc is baby-phi-specific. i-phi has (or will have) an equivalent at its own path; the AUTHORING DISCIPLINE (sections + columns + deferred-catalogue shape) is the reusable pattern.

---

## §2 — Feature inventory (user-facing capabilities)

Each row names one user-facing capability + the chunk(s) that deliver it + any deferred sub-aspects + v0 vs final-state delta.

| Feature ID | Feature (user-facing name) | v0 scope | Closing chunk(s) | Deferred sub-aspects | v0 vs final state |
|---|---|---|---|---|---|
| **F-bootstrap** | Fresh-install bootstrap claim | Operator runs `phi bootstrap claim` once per install; single-use credential exchanges for platform-admin authority on `system:root`. CLI + HTTP API + Web UI Phase 1 page. | M1 (CH-01..CH-02 era) | — | Final at v0; OAuth wiring at M7b means the bootstrapped admin can later authenticate via IdP. |
| **F-permcheck** | Permission Check engine | 6-step formal algorithm runs on every governance-plane write; every admin / agent action passes through it. Property-based tested. | M1 spine | — | Final at v0. |
| **F-authreq** | Auth Request state machine | Approval workflow for sensitive operations (Shape-B project creation, model-config change, template adoption). Per-state ACL enforcement; forward-only revocation. | M1 + CH-09..CH-18 hardening | M7b retention 2-tier storage (M7b-DEFERRED-01) | v0 ships full state machine in single-tier hot storage; M7b adds cold-storage tier for retention compliance. User sees no behavioral change at v0; storage architecture evolves. |
| **F-model-providers** | Configure model providers | Admin page 02; CLI + HTTP API + Web UI. Set provider credentials, rate limits, default models. | M2 | — | Final at v0. |
| **F-mcp-servers** | Configure MCP servers | Admin page 03; register + manage MCP tool surfaces shared across orgs. | M2 | — | Final at v0. |
| **F-credentials-vault** | Credentials vault | Admin page 04; store + rotate API keys, OAuth secrets. At-rest encryption activates at M7b. | M2 (base) + M7b (encryption) | At-rest encryption deferred to M7b | v0 stores in plaintext SurrealDB rows (dev acceptable); M7b enables AES-256 at-rest. User sees no change; ops profile hardens. |
| **F-platform-defaults** | Platform defaults | Admin page 05; default model, context window, retention period at platform tier. | M2 | — | Final at v0. |
| **F-org-creation** | Create organization | Admin page 06; multi-step wizard with autosaving draft. Provisions two system agents (memory-extraction-agent + agent-catalog-agent) + adoption Auth Requests. | M3 | — | Final at v0. |
| **F-org-dashboard** | Organization dashboard | Admin page 07; agents/projects/Auth-Requests/alerts/budget tiles. Viewer-role-filtered. | M3 + M4 carryovers (C-M4-1..6) | ProjectLead filtering wired at M4 | M3 ships Admin + Member roles; ProjectLead filtering arrives at M4. User sees only Admin/Member views in M3 closure. |
| **F-agents-page** | Manage agents | Admin page 08; create + edit + archive agents. Agent role (Human/Intern/Contract/System) discriminator. | M4 | — | Final at v0. |
| **F-agent-profile-editor** | Agent profile editor | Admin page 09; edit blueprint fields (system prompt, thinking level, parallelize, model_config_id, mock_response). | M4 + M5 (per-agent model binding C-M5-5) + CH-28 (cardinality redesign) | — | v0 at M5 ships per-agent model binding; CH-28 reshapes to N:1 cardinality (templates sharable). User now sees "this profile may be shared across agents" semantics in editor. |
| **F-projects-page** | Manage projects | Admin page 10; create Shape-A (single-owner) + Shape-B (co-owned) projects with OKRs. | M4 (P5–P6) | — | Final at v0. |
| **F-project-detail** | Project detail page | Admin page 11; project metadata + leads + members + recent sessions panel + alerted-events. | M4 + M5 (session persistence C-M5-3) | — | v0 at M5 ships full panel with persisted sessions. |
| **F-template-adoption** | Template adoption | Admin page 12; org adopts authority templates (A=memory, B=catalog, C=...). Triggers s05 template-adoption grant fires. | M5 (admin page 12) + M7 (s05 broaden to all 5 templates) | s05 broaden to all 5 templates → M7 | v0 at M5 wires Template A (memory) + first-session-launch path; M7 broadens to all 5 templates. User sees Template A available at M5; remaining templates surface at M7. |
| **F-system-agents-config** | System agents configuration | Admin page 13; configure memory-extraction-agent + agent-catalog-agent tunables. | M5 | — | Final at v0. |
| **F-first-session** | Launch first session | Admin page 14; preview Permission Check (Steps 0–6) for a proposed session, launch, persist `phi_core::Session`. | M5 (P4 + P7–P9) + CH-25 carve-out | — | v0 at M5 ships full preview + launch + persistence. |
| **F-memory-extraction** | Memory extraction (heuristic v0) | `memory-extraction-agent` (s02) subscribes to `SessionEnded` events, emits `MemoryExtracted` audit events with structured tag list. | M5 (CH-21 heuristic v0) | LLM supervisor body → M6-DEFERRED-04 / CH-36 | v0 at M5 ships heuristic extractor (deterministic rules); M6-DEFERRED-04 lifts to LLM-driven extractor at CH-36. User sees extraction firing at SessionEnd; quality improves at CH-36. |
| **F-agent-catalog** | Agent catalog | `agent-catalog-agent` (s03) subscribes to edge changes, materializes profile snapshots. | M5 | — | Final at v0. |
| **F-memory-recall** | Memory recall (retrieval gate) | `MemoryStore` trait + default SurrealDB impl + `Action::Recall` consumer + permission-over-time retrieval gate. CLI + HTTP API. | M6 (CH-32) | Third-party storage backends (Chroma/Weaviate/LanceDB) → optional follow-on | v0 ships trait + default impl + retrieval gate; third-party backends are user-pluggable via the trait. User sees `phi memory recall` CLI + `GET /api/v0/memory/recall` HTTP at M6 close. |
| **F-agent-profile-cardinality** | Template-shared profiles across agents | N:1 cardinality flip: one `AgentProfile` template can be referenced by multiple agents (e.g., 5 customer-service agents share one config) with per-agent overrides for governance fields. | M6 (CH-28; closed 2026-05-20 cycle hex `0412eb06`) | Listener template-tier fan-out → M6-DEFERRED-04 / CH-36 (D-CH28-FOLLOWUP-01) | v0 at CH-28 ships hybrid Blueprint table + override-tier listener; template-tier fan-out (one template upsert refreshes N agents' profile snapshots) ships at CH-36 a04 supervisor body. **User observation in deferral state**: when an admin upserts a shared template, sibling agents see refreshed snapshots only on next per-agent event (no immediate cross-agent broadcast). After CH-36 closes: template upserts trigger immediate fan-out. |
| **F-a01-inbox-outbox** | Agent inbox + outbox | Page a01; agents see incoming + outgoing AgentMessages with priority badges + read/unread state + tag filters. Web-UI for Humans; `read_inbox` + `send_message` LLM tools for LLM agents. | M6 (CH-33; requires CH-29 substrate + CH-30 routing) | — | Final at v0. Q1 user-lock locked a01 close behind messaging substrate. |
| **F-a02-auth-requests** | Agent's own Auth Requests | Page a02; inbound + outbound ARs with state badges + slot-progress + approve/deny/reconsider/escalate actions. | M6 (CH-34; requires CH-33 a01 close per Q2 user-lock) | — | Final at v0. |
| **F-a03-consent-records** | Agent's own consent records | Page a03; consent state machine with acknowledge/decline/revoke actions. Revoke cascades via engine. | M6 (CH-35; requires CH-33 a01 close per Q2 user-lock) | — | Final at v0. |
| **F-a04-my-work** | Agent's own tasks + sessions | Page a04; my-tasks panel (with status transitions) + my-sessions panel + concurrency-cap status. Display-only re: memory (no recall surface at a04). | M6 (CH-36; requires CH-33 a01 close per Q2 user-lock) | — | Final at v0. Q3 user-lock: a04 does NOT integrate memory-recall surface. |
| **F-a05-profile-grants** | Agent's own profile + grants | Page a05; profile editor (with re-embed on self_description PATCH) + grants list + authority-chain visualizer per NFR-observability R6. | M6 (CH-37; requires CH-31 embedding + CH-33 a01 + CH-28 cardinality redesign per Q4+Q2 user-lock) | — | Final at v0. |
| **F-embedding-provider** | Embedding provider configuration | Admin sets platform-level embedding provider config at org-bootstrap; provider change triggers batch re-embed via Auth Request. Mock provider at v0. | M6 (CH-31) | Real OpenAI / Voyage provider integration → optional follow-on (user-opt) | v0 ships mock provider (deterministic); real provider integrations are user-wireable. User can configure embedding model + dimension at provision time. |
| **F-inter-agent-messaging-routing** | Cross-agent message delivery | Routing-tier dispatcher with ordering (priority-then-FIFO) + duplicate suppression + audit-event emission. | M6 (CH-30; substrate at CH-29) | — | Final at v0. |
| **F-resolver-actor-routing** | Permission-checked resolvers | Resolver traits at `domain/src/events/listeners.rs` thread `actor: Option<AgentId>` parameter + emit `check_permission` calls (advisory at M6 first; blocking at follow-on). | M6 (CH-38; closes D-CH27-FOLLOWUP-01) | Blocking wire-tier promotion → M6 follow-on per ADR-0062 §D62.1 | v0 at CH-38 ships advisory check_permission emissions; blocking promotion happens at follow-on. User sees advisory audit events without enforcement at v0; enforcement firms up later. |
| **F-s04-state-machine-observability** | s04 full state-machine observability | Audit + Prometheus metrics for AR state machine + Permission Check + Consent state machine. | M7 | — | Final at v0. |
| **F-s05-template-fires-broadened** | Template adoption fires for all 5 templates | s05 grants fire for Template A (memory), B (catalog), C (...), D (...), E (...). | M7 (broaden from M5 narrow A-only) | — | v0 at M5 fires Template A only; M7 broadens. User sees grants firing for all 5 templates at M7. |
| **F-s06-periodic-triggers** | Retention + secret rotation + heartbeat | Scheduled periodic triggers: retention archival, secret rotation reminders, heartbeat, token-budget snapshot. | M7 | — | Final at v0. |
| **F-nfr-observability** | NFR observability | Audit event schema finalized; Prometheus metrics; OpenTelemetry traces. Audit hash-chain (predecessor hash per org-scope). | M7 + M7b | — | Final at v0. |
| **F-nfr-performance** | NFR performance | p95/p99 measured against targets; hotspots optimized. | M7 + M7b load-test | — | Final at v0. |
| **F-nfr-security** | NFR security invariants | 17 property tests for security invariants. | M7 + M7b scans | — | Final at v0. |
| **F-backup-restore** | Backup + restore | Scheduled `surreal export` with off-site upload + restore script. Off-site stream to append-only S3/GCS at M7b. | M7 + M7b | — | Final at v0. |
| **F-oauth-auth** | OAuth 2.0 auth wired | OAuth 2.0 PKCE against IdP; local-password for dev. | M7b | — | Final at v0. |
| **F-tls** | TLS configured | Native axum-rustls path + reverse-proxy-with-TLS-termination pattern. | M7b | — | Final at v0. |
| **F-at-rest-encryption** | At-rest encryption | SurrealDB data files AES-256; key from env; rotation procedure. | M7b | — | Final at v0. |
| **F-rate-limiting** | Rate limiting | Per-endpoint + per-principal rate limits. | M7b | — | Final at v0. |
| **F-gdpr-erasure** | GDPR right-to-erasure | Data-subject deletion API + audit-record preservation. | M7b | — | Final at v0. |
| **F-k8s-manifests** | Kubernetes deployment manifests | Reference Deployment + Service + PVC + ConfigMap + Secret + Ingress + HPA. Single-pod deployment supported. | M7b | Multi-pod microservices carve-out → M7b-DEFERRED-02 | v0 ships single-pod K8s deployment; multi-pod / microservice split is M7b-DEFERRED-02 (10 captured items at `m7b/architecture/deferred-from-ch-k8s-prep.md`). User can deploy to K8s at v0; horizontal scale is post-v0. |
| **F-runbook** | Operator runbook | `docs/ops/runbook.md` covers deploy / upgrade / rollback / backup / restore / 5 incident scenarios + known issues. | M7b | — | Final at v0. |
| **F-channel-schema** | Channel schema enrichment | Channel node enriched with subscribers + retention + tag schema. | M7 (DEFERRED carried forward via M7-DEFERRED-01) | — | v0 at M7 ships enriched channel schema. |
| **F-task-node-mat** | Task node materialization | Task node persisted with full schema (vs derived/computed). | M7 (DEFERRED carried forward via M7-DEFERRED-02) | — | v0 at M7 ships persisted Task node. |
| **F-token-economy** | Token economy + Worth formula | Token budget fields on Project + Worth scoring formula. | M6 or M7 (DEFERRED carried forward — decided at M6/M7 plan-open) | — | Decision pending; routes via M6-or-M7-DEFERRED marker. |

---

## §3 — Deferred catalogue

Every `M*-DEFERRED-NN` marker + every `D-CH<NN>-FOLLOWUP-*` drift filed across the build, indexed by user-facing feature. For each: the user-visible state during deferral + the allocation chunk + cross-chunk dependency chain.

### M6-DEFERRED-04 — Memory-extraction LLM supervisor body
- **Feature impact**: F-memory-extraction (lifts heuristic extractor at M5/CH-21 to LLM-driven extractor).
- **User-visible state in v0 (at M5/CH-21 close)**: memory extraction fires deterministically via rule-based heuristic; some session-end events surface as memory candidates that an LLM-driven extractor would otherwise reject (lower precision); some session content an LLM extractor would catch is missed (lower recall).
- **User-visible state at final (CH-36 close)**: LLM supervisor body extracts memory with higher precision/recall; same audit-event shape; same trait surface.
- **Allocation chunk**: M6-DEFERRED-04 → CH-36 (a04 My Work supervisor body inherits the same fan-out shape).
- **Cross-chunk dependency**: CH-32 (MemoryStore trait + default impl) must close first; CH-36 consumes the trait.

### M6-or-M7-DEFERRED — Token economy + Worth formula
- **Feature impact**: F-token-economy.
- **User-visible state in v0**: token budgets visible as a numeric field on Project but no Worth scoring; budget tracking is manual.
- **User-visible state at final**: Worth formula computes per-session value; budget snapshots auto-record at session-end.
- **Allocation chunk**: M6-or-M7-DEFERRED — decision routes at M6/M7 plan-open.
- **Cross-chunk dependency**: depends on `D-new-27` resolution + session-persistence (C-M5-3 closed at M5).

### M7-DEFERRED-01 — Channel schema enrichment
- **Feature impact**: F-channel-schema.
- **User-visible state in v0 (pre-M7)**: channels exist but lack subscribers/retention/tag-schema fields.
- **User-visible state at final (M7 close)**: full channel surface usable for messaging governance.
- **Allocation chunk**: M7-DEFERRED-01 → M7 plan-open.

### M7-DEFERRED-02 — Task node materialization
- **Feature impact**: F-task-node-mat (full materialization of Task node).
- **User-visible state in v0 (pre-M7)**: tasks computed/derived from edges + audit events (no first-class Task node persistence).
- **User-visible state at final (M7 close)**: persistent Task node with full schema; querying tasks is a first-class API.
- **Allocation chunk**: M7-DEFERRED-02 → M7 plan-open.

### M7b-DEFERRED-01 — AuthRequest retention 2-tier storage
- **Feature impact**: F-authreq.
- **User-visible state in v0 (pre-M7b)**: ARs live in single-tier hot storage; retention policy enforced by app-tier policies; older ARs may bloat hot storage.
- **User-visible state at final (M7b close)**: cold-tier (S3/GCS append-only) holds aged ARs; hot tier remains tight.
- **Allocation chunk**: M7b-DEFERRED-01 → M7b plan-open.

### M7b-DEFERRED-02 — K8s microservices carve-out
- **Feature impact**: F-k8s-manifests.
- **User-visible state in v0 (M7b single-pod close)**: deployable to K8s as single pod; horizontal-pod-autoscaler scales replicas but not microservices.
- **User-visible state at final (post-v0)**: microservice carve-out for hot-path services (session-launch, memory-recall) with independent scaling.
- **Allocation chunk**: M7b-DEFERRED-02 → post-v0 milestone.
- **10 captured items**: see `docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` (CHK8S-D-01..CHK8S-D-10).

### D-PHICORE-08-FOLLOWUP-01 — Composition I adoption (phi-core 0.8.0 opt-in braking layer)
- **Feature impact**: F-agent-context-management (post-0.8.0 opt-in tree-structured braking layer; complements existing compaction surface).
- **User-visible state in v0 (at CH-28b close, 2026-05-23)**: agents continue to operate on the pre-0.8.0 monotonically-growing-context posture; compaction (existing `BlockCompactionStrategy` + `compact_messages`) is the only relief. Composition I is opt-in upstream + remains opt-out in baby-phi via `AgentLoopConfig.revert_pending: None` at `launch.rs:577`.
- **User-visible state at final (at future M6+ Composition I adoption chunk close)**: agents call `revert_to_state` between turns to abandon failed/finished/completed/step-summary branches (the 4 `RevertCategory` cases); active context stays lean; compaction fires less often; no user-perceived behavior delta beyond efficiency (faster turns + lower token cost).
- **Allocation chunk**: M6+-FUTURE-COMPOSITION-I-ADOPTION (placeholder; no specific CH-NN slot reserved at CH-28b close; future planning session decides whether adoption lands as a dedicated FUNCTIONAL chunk OR bundles into an existing M6+ FUNCTIONAL chunk that touches BasicAgent construction).
- **Cross-chunk dependency**: requires 4 prerequisites (per drift body §"Remediation scope"): (1) `BasicAgent::with_revert_tool()` builder call at the construction site (`launch.rs` or `cli/agent.rs` or NEW factory); (2) optional `RevertApplied` event surfacing in `BabyPhiSessionRecorder`; (3) skill / prompt teaching the agent the revert discipline (load-bearing axis — without it, enabling the tool is a no-op); (4) `RevertRenderPolicy` tuning for baby-phi's compaction posture.

### D-CH28-FOLLOWUP-01 — Listener template-tier fan-out for BlueprintUpserted
- **Feature impact**: F-agent-profile-cardinality (the template-tier broadcast path).
- **User-visible state in v0 (at CH-28 close, 2026-05-20)**: when an admin upserts a shared template Blueprint (e.g., changes the system prompt of a template referenced by 5 customer-service agents), the 5 agents' profile snapshots refresh only on the next per-agent event (no immediate broadcast). Override-tier upserts (per-agent overrides) refresh that single agent immediately.
- **User-visible state at final (at CH-36 close)**: template upserts trigger immediate fan-out — all agents referencing the upserted template see refreshed snapshots in the same audit-event window.
- **Allocation chunk**: M6-DEFERRED-04 / CH-36 (a04 supervisor body inherits the fan-out requirement).
- **Cross-chunk dependency**: requires NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` (per drift body); rolls into CH-36 supervisor design.

### D-CH27-FOLLOWUP-01 — Resolvers actor passthrough wiring (closed at CH-38)
- **Feature impact**: F-resolver-actor-routing.
- **User-visible state at v0 close (CH-38)**: 4 resolvers thread actor + emit advisory `check_permission` audit events. NOT yet blocking (advisory-only at M6).
- **User-visible state at final (M6 follow-on, per ADR-0062 §D62.1 wire-tier pattern)**: advisory promotes to blocking; resolvers reject calls where actor lacks `check_permission` clearance.
- **Allocation chunk**: M6-DEFERRED-RESOLVERS-WIRING → CH-38 (advisory-form closure); blocking-promotion is a follow-on.

---

## §4 — Cross-chunk dependency graph (feature-tier)

Feature-tier dependency chains for v0. Mirrors forward-scope §4 but at product-capability granularity. Critical paths first; parallelizable branches collapsed into rows.

```
M1 spine:
  F-bootstrap → F-permcheck → F-authreq

M2 platform:
  F-permcheck → { F-model-providers, F-mcp-servers, F-credentials-vault, F-platform-defaults }   (parallel)

M3 orgs:
  F-permcheck + F-authreq → F-org-creation → F-org-dashboard

M4 agents+projects:
  F-org-creation → { F-agents-page, F-projects-page } (parallel)
  F-agents-page → F-agent-profile-editor
  F-projects-page → F-project-detail
  C-M4-1..6 carryovers backfill F-org-dashboard

M5 first session:
  F-projects-page + F-agent-profile-editor → F-template-adoption (Template A) → F-first-session
  F-first-session → { F-memory-extraction (heuristic v0), F-agent-catalog }   (parallel)
  C-M5-3 + C-M5-4 + C-M5-5 + C-M5-6 carryovers backfill F-first-session + F-agent-profile-editor

M6 agent self-service (CH-28..CH-38):
  CH-28 F-agent-profile-cardinality (foundation; closed 2026-05-20)
    ↓
  { CH-29 substrate → CH-30 routing → F-inter-agent-messaging-routing }   (foundation parallel band)
  { CH-31 F-embedding-provider }                                          (parallel)
  { CH-32 F-memory-recall }                                               (parallel)
  { CH-38 F-resolver-actor-routing }                                      (parallel; closes D-CH27-FOLLOWUP-01)
    ↓
  CH-33 F-a01-inbox-outbox  (strict serial gate per Q2)
    ↓
  { CH-34 F-a02-auth-requests, CH-35 F-a03-consent-records, CH-36 F-a04-my-work }   (parallel band; all gated on CH-33)
    +
  CH-37 F-a05-profile-grants  (requires CH-31 + CH-33 + CH-28 per Q4+Q2 user-lock)
    +
  CH-36 absorbs M6-DEFERRED-04 LLM supervisor body + D-CH28-FOLLOWUP-01 template-tier fan-out

M7 system flows + NFRs:
  M6 close → { F-s04-state-machine-observability, F-s05-template-fires-broadened, F-s06-periodic-triggers,
              F-nfr-observability, F-nfr-performance, F-nfr-security, F-backup-restore, F-channel-schema,
              F-task-node-mat, F-token-economy (if not M6) }   (parallel band; M7 milestone scope)

M7b hardening:
  M7 close → { F-oauth-auth, F-tls, F-at-rest-encryption, F-rate-limiting, F-gdpr-erasure, F-k8s-manifests,
              F-runbook, F-credentials-vault encryption-arm }   (parallel band; M7b production-hardening)

M8 v0.1 release:
  M7b close → all-features GREEN → v0.1 release
```

**Foundation-band callout (CH-28..CH-32 + CH-38)**: parallelizable post-CH-28 close. Each lane delivers an independent capability + is consumed by the consumer-tier (a01..a05) downstream. The Q2 user-lock makes CH-33 a01 the strict serial gate between foundation + consumer tiers.

---

## §5 — Revisit triggers

This document is re-authored / extended when ANY of the following happen. Each trigger has a designated author + cadence:

| Trigger | Author | Cadence |
|---|---|---|
| **NEW milestone forward-scope ships** (phase-planner emits CH-NN+M+1 .. CH-NN+M+K rows) | orchestrator (or phase-planner if scope is wholly within an existing milestone) | At forward-scope archive |
| **Drift / `M*-DEFERRED-NN` allocation shifts** (allocation chunk changes; deferral routing flips e.g. M6 → M7) | orchestrator at retro or at chunk-plan-open | At allocation decision |
| **Chunk's user-visible delivery diverges materially from §2 row** (e.g., CH-28 lands extra capability the row didn't predict; OR ships a narrower surface) | orchestrator at gate-4 cycle-audit OR at retro | At chunk close |
| **NEW deferred sub-aspect surfaces** (drift filed mid-cycle with allocation different from §2 row's "Deferred sub-aspects" cell) | orchestrator at chunk close | At drift filing |
| **Feature renamed / merged / split** (rare; e.g., F-a04-my-work absorbs F-memory-recall preview at some plan-mode decision) | orchestrator at plan-mode session | At plan-mode close |
| **Cross-chunk dependency graph (§4) changes** (chunk re-ordering at gate-2.5 swap; parallelization band shifts) | orchestrator at retro | At cycle close |

**Stale-detection cadence**: this doc is considered "stale" if > 5 cycles closed without a re-author pass. Orchestrator's gate-5 close should grep recent cycle indexes and surface a stale-detection observation if applicable.

---

## §6 — Cross-references

- **Engineering-tier**: `docs/specs/plan/build/build-plan-v01-36d0c6c5.md` (milestone-level), per-milestone forward-scope files at `docs/specs/plan/forward-scope/*.md`, per-cycle plan archives at `docs/specs/plan/build/<slug>-<8hex>/plan.md`.
- **Drifts**: `docs/specs/v0/implementation/m*/drifts/*.md` (per-milestone drift files) + `docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` (K8s deferral ledger).
- **Concept docs**: `docs/specs/v0/concepts/*.md` (the source-of-truth for feature semantics; this inventory translates concept-doc semantics to user-facing capability names).
- **Cycle ledger**: `docs/specs/plan/build/_cycle-index.md` (per-cycle audit ledger; this inventory is the product-trajectory peer).

**Project-agnostic note**: i-phi (and future projects) maintain their own equivalent inventories at their own paths. The PATTERN (§1..§6 shape + cross-chunk dependency graph + revisit triggers) is the reusable design.
