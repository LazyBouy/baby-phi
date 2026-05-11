<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-SEAL chunk-seal — line 211 amended: "Drifts closed: none new" → "Drifts closed: 1 (D-CH24-recent-sessions-api-flip — mid-cycle scope expansion approved at gate-2.5)" per CH-24 plan §13 retrospective routing item 2 (forward-scope row drift-count amendment at P-SEAL); mid-cycle scope expansion closed C-M5-3 API-surface flip via P-FLIP-RECENT-SESSIONS + ADR-0059 (LIMIT 10 / dedicated repo method / RecentSessionEntry 6-field shape; 7th field started_by_display_name deferred to follow-up chunk pending Agent-table-join design); first cycle to scope-expand mid-flight via P-NEW-TESTS authoring finding; gate-2.5 user-lock 2-of-3 DIVERGENT from planner-recommendations (F-D59.2.b dedicated method + F-D59.3.b richer struct); cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P3 chunk-seal — line 185 parenthetical amended `(existing 14 items)` → `(existing 16 items)` per ADR-0058 §D58.10 reconciliation; the forward-scope row's "14" was a counting error in M5.1/P3 authoring (2026-04-24); the explicit drift list at line 186 enumerates 16 IDs (D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6) — empirically verified via `ls v0/implementation/m5_1/drifts/` showing all 16 files exist with Status `discovered` at chunk-open. Categorised as housekeeping per CH-19/ADR-0057 §D57.7 edge-count reconcile precedent — no new drift file filed. cycle hex `240616a4`.) -->
<!-- Last verified: 2026-04-27 by Claude Code -->

# Forward-scope inventory — remaining work from M5/P7 close to baby-phi concept-aligned target

**Milestone**: M5.1/P3 output
**Purpose**: enumerate every unit of work needed between M5/P7's current shipped state and concept-doc-aligned delivery, broken into implementation chunks each of which is a future per-chunk-plan candidate.
**Authority**: this document **does not** replace the base build plan ([`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md)); it is a sibling forward-look produced from the M5.1 drift catalogue.

## What this document is / is not

**IS:**
- One document naming ~24 chunks + a handful of deferred-milestone scope markers
- Per-chunk one-paragraph scope, drift-id list, rough effort range, concept docs touched, prerequisites
- A dependency graph showing which chunks must land before which
- Open questions that need user decision before any chunk starts

**IS NOT:**
- A detailed implementation plan per chunk (those are written just-in-time at chunk open using the [`per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md) landing at M5.1/P4)
- A schedule or calendar commitment (effort ranges are rough)
- A replacement for [`build-plan-v01-36d0c6c5.md`](../build/build-plan-v01-36d0c6c5.md) whole-product roadmap

## Input artefacts (consumed)

- **60 drift files** at [`v0/implementation/m5_1/drifts/`](../../v0/implementation/m5_1/drifts/) — 29 existing (D1.1–D7.6) + 31 new (D-new-01–D-new-31)
- **[`_concept-audit-matrix.md`](../../v0/implementation/m5_1/drifts/_concept-audit-matrix.md)** — 20 concept docs × ~95 claims classified
- **[`_ledger-migration-log.md`](../../v0/implementation/m5_1/drifts/_ledger-migration-log.md)** — P1 migration annotations
- Base plan [§M6 / §M7 / §M7b](../build/build-plan-v01-36d0c6c5.md) sections for cross-reference

---

## §1 — Drift-remediation scope (chunks closing the 60 drifts)

24 implementation chunks grouped by concept-cohesion + dependency sequencing. Chunks that close a HIGH-severity concept-contradiction are marked ⚠HIGH.

### Foundation tier (isolated, unblock downstream)

**CH-01 — Agent durable lifecycle** · ⚠HIGH · ~2 days
- Drifts closed: **D6.5** (durable `active` + `archived_at` fields on Agent), **D-new-22** (role-immutability enforcement in update handler), **D-new-23** partial (Human Agent Identity guard — sibling to CH-16).
- Concept docs: [`agent.md`](../../v0/concepts/agent.md) §"Roles" + §"Lifecycle", [`system-agents.md`](../../v0/concepts/system-agents.md) §"disable flow", [`human-agent.md`](../../v0/concepts/human-agent.md) §"No Identity".
- Prerequisites: none (isolated schema work).
- Deliverables: migration 0006 adds `active: bool DEFAULT true` + `archived_at: Option<DateTime>` columns to `agent`; Agent struct `#[serde(default)]`-compatible extension; 2 new repo methods; disable/archive handlers flip columns; role-immutability guard in `agents/update.rs`. ADR-0034 (Agent durable state) flips Accepted at close.
- Unblocks: **CH-22** (AgentCatalogListener body reads `Agent.active`).

**CH-02 — Real `agent_loop` + MockProvider execution** · ⚠HIGH · ~5 days · **M5 critical path**
- Drifts closed: **D4.2** (primary; leverage-violation tag removed).
- Concept docs: [`phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md), [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"session lifecycle", [`agent.md`](../../v0/concepts/agent.md) §"turn execution", [`system-agents.md`](../../v0/concepts/system-agents.md) §"memory extraction reads transcripts".
- Prerequisites: none.
- Deliverables: strip synthetic feeder in `sessions/launch.rs::spawn_replay_task`; build `AgentContext` from `SessionLaunchContext` + first prompt; `provider_for(runtime) -> Arc<dyn StreamProvider>` helper returns `MockProvider` at M5 (real providers defer M7); call `phi_core::agent_loop(...)` via `tokio::spawn`; event-consumer task drains mpsc channel → `recorder.on_phi_core_event(...)`; existing tests flipped from synthetic-assertion to real-loop-output assertions. ADR-0032 (MockProvider at M5) flips Accepted.
- **Unblocks** (the single biggest domino): CH-15 (real permission manifest needs real transcripts), CH-17 (SSE has real events to stream), CH-16 (Identity extraction reads real turns), CH-21 (memory-extraction extractor reads real transcripts), and the carryover re-verification at CH-24.

**CH-03 — Storage-backend concept refresh + configurability framing (SurrealDB current; others must satisfy criteria)** · ~1 day · doc-only
- Drifts closed: **D-new-02**.
- Concept docs: [`coordination.md`](../../v0/concepts/coordination.md) §"Storage backend" — **refresh required**.
- Prerequisites: none.
- **Deliverables** (expanded per Q1 decision 2026-04-24):
  1. Concept-doc refresh: `coordination.md` §Storage-backend text replaced with (a) "v0 ships with **SurrealDB** as the currently configured backend" + (b) "storage backend is **configurable** — SurrealDB is the initial configured impl, not a hardcoded choice" + (c) criteria a conforming backend must satisfy: transactional semantics, compound-tx support, RELATION edge semantics, FLEXIBLE TYPE object for phi-core wraps, migration idempotency, SCHEMAFULL table support, UNIQUE index enforcement.
  2. New ADR documenting: the decision to frame the backend as configurable; why SurrealDB is the current config; the conforming-backend criteria as an enforceable contract.
  3. No code change at M5.1 — the actual configurability abstraction (trait-based repo adapter swap) is a separate future chunk if we ever onboard a 2nd backend. Doc work here is the recognition that the **architectural decision is configurability**, not lock-in.
- Unblocks: nothing downstream; closes a silent architectural contradiction; establishes the configurability contract for any future backend-migration chunk.

### Permission-foundation tier (biggest cluster; tightly coupled)

**CH-04 — Typed action vocabulary + Action × Fundamental matrix** · ~2.5 days
- Drifts closed: **D-new-09** (action vocab as enum/constants), **D-new-10** (applicability matrix enforcement).
- Concept docs: [`permissions/03-action-vocabulary.md`](../../v0/concepts/permissions/03-action-vocabulary.md).
- Prerequisites: none.
- Deliverables: `domain::permissions::action::Action` enum with 33 variants + `TryFrom<&str>` + `as_str()`; `APPLICABILITY_MATRIX` constant; callsite migration (Grant / Manifest / ToolAuthorityManifest).
- **Unblocks**: CH-05 (validator needs typed actions), CH-08 (cardinality needs typed allocate/transfer), CH-15 (permission manifest uses typed actions).

**CH-05 — Publish-time manifest validator** · ⚠HIGH · ~2 days
- Drifts closed: **D-new-07** (validator), **D-new-31** (reserved-namespace write rejection as sub-case).
- Concept docs: [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Manifest Validation at Publish Time", [`permissions/07-templates-and-tools.md`](../../v0/concepts/permissions/07-templates-and-tools.md) §"What v0 Validates vs Future", [`permissions/09-selector-grammar.md`](../../v0/concepts/permissions/09-selector-grammar.md) §"Reserved Namespace Enforcement".
- Prerequisites: **CH-04** (typed actions needed for matrix check).
- Deliverables: `permissions::manifest::validator` module with `validate_published_manifest`; reserved-namespace denylist constant; acceptance tests covering each rejection class. Wired into ToolDefinition publish path.
- Unblocks: CH-12 (frozen-tag enforcement uses validator + reserved namespace).

**CH-06 — Selector grammar (PEG tag-predicate DSL) + instance identity tags** · ⚠HIGH · ~5–7 days · **biggest single chunk**
- Drifts closed: **D-new-03** (PEG grammar), **D-new-11** (composite instance self-identity tags — needed because selectors match against them).
- Concept docs: [`permissions/09-selector-grammar.md`](../../v0/concepts/permissions/09-selector-grammar.md) (primary), [`permissions/01-resource-ontology.md`](../../v0/concepts/permissions/01-resource-ontology.md) §"Instance Identity Tags", [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"Memory as Resource Class".
- Prerequisites: none (isolated parser + matcher work, though grammar design choices affect CH-12).
- Deliverables: Selector enum extended with `Contains`, `Intersects`, `AnyMatch`, `SubsetOf`, `Empty`, `NonEmpty` + `And`/`Or`/`Not` combinators; PEG parser (likely via `pest` crate); `matches()` body for each predicate; instance tag auto-emission helper wired into every composite creation path; all existing grants migrated to new vocabulary where applicable; selector acceptance suite covering every predicate + combinator. **May benefit from internal split** into CH-06a (grammar + parser) + CH-06b (instance tag wiring) if scope proves too large mid-chunk.
- **Unblocks**: CH-07 (multi-scope cascade uses predicates), CH-12 (frozen-tag enforcement uses grammar), CH-15 (permission manifest selectors), deferred Memory contract (M6).

**CH-07 — Multi-scope cascade + contractor model** · ⚠HIGH · ~3 days · **CLOSED 2026-05-07** (cycle hex `cc912d07`)
- Drifts closed: **D-new-06** (full 5-tier cascade), **D-new-20** (contractor-model scope-membership bounds).
- Concept docs: [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Scope Resolution", [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) §"Unified Resolution Rule", [`permissions/08-worked-example.md`](../../v0/concepts/permissions/08-worked-example.md) scenarios 4–6.
- Prerequisites: **CH-06** (cascade uses tag-predicate selectors).
- Deliverables: `step_5_scope_resolution` full 5-tier cascade (Project → Org → base_project → base_org → intersection fallback) **(informal labeling for 5 distinct outcomes — concept-doc canonical phrasing is "2 main tiers + base_X tie-breakers + intersection fallback" per concept 04 lines 354–375 + concept 06 lines 28–63; F1.A locked at plan approval to concept-doc framing per CH-07 retro §5 row 3)**; contractor-model scope-membership bound at Step 2a; acceptance tests matching worked-example scenarios 4–6.
- Unblocks: nothing critical (standalone Permission-engine quality improvement).

**CH-08 — allocate/transfer cardinality + refinements** · ⚠HIGH · ~2 days
- Drifts closed: **D-new-13** (allocate additive vs transfer exclusive cardinality enforcement), **D-new-29** (allocate refinement constraints).
- Concept docs: [`permissions/02-auth-request.md`](../../v0/concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics", [`permissions/03-action-vocabulary.md`](../../v0/concepts/permissions/03-action-vocabulary.md) §"`allocate` as Umbrella Action".
- Prerequisites: **CH-04** (typed actions).
- Deliverables: distinguish allocate vs transfer in grant-mint compound tx; transfer revokes sender's grant atomically; `AllocateRefinement { no_further_delegation, max_depth, ... }` structured constraint type.
- Unblocks: nothing structurally; closes a security-boundary drift.

### Consent model tier

**CH-09 — Consent node full shape** · ⚠HIGH · ~1 day
- Drifts closed: **D-new-04** (Consent struct field set).
- Concept docs: [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) §"Consent Node".
- Prerequisites: none.
- Deliverables: Consent struct extended with `state`, `requested_at`, `responded_at`, `revocable`, `provenance` + nested `ConsentScope { templates, actions, org }`; migration 0007 adds columns; `#[serde(default)]` for back-compat.
- Unblocks: **CH-10**, **CH-11**.

**CH-10 — Consent lifecycle state machine** · ⚠HIGH · ~1 day
- Drifts closed: **D-new-05**.
- Concept docs: [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) §"Consent Lifecycle".
- Prerequisites: **CH-09**.
- Deliverables: `ConsentState` enum (Requested / Acknowledged / Declined / TimedOut / Revoked / Expired) + transition function mirroring M1 AuthRequest state-machine pattern; forward-only revocation; timeout logic.
- Unblocks: **CH-11**.

**CH-11 — Per-Session consent gating** · ⚠HIGH · ~2 days
- Drifts closed: **D-new-17**.
- Concept docs: [`permissions/06-multi-scope-consent.md`](../../v0/concepts/permissions/06-multi-scope-consent.md) §"Per-Session Consent".
- Prerequisites: **CH-09**, **CH-10**.
- Deliverables: `ApprovalMode` enum on Grant; Step 6 real gating logic; subordinate-approval request dispatch; timeout handling; acceptance tests cover Implicit / One-Time / Per-Session paths.
- Unblocks: nothing; closes the consent triad.

### Frozen tags + audit

**CH-12 — Frozen session-tag immutability enforcement** · ⚠HIGH · ~1.5 days
- Drifts closed: **D-new-08**.
- Concept docs: [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"Frozen-at-creation tags".
- Prerequisites: **CH-05** (validator denylists), **CH-06** (grammar for "structural tag" expression).
- Deliverables: reserved-namespace validator rejects `[modify]` on reserved tags at publish; Step 4 runtime gate rejects retag on structural session tags; acceptance tests cover both paths.
- Unblocks: nothing; closes a security-boundary exfiltration vector.

**CH-13 — `audit_class` composition (strictest wins)** · ~1 day
- Drifts closed: **D-new-19**.
- Concept docs: [`permissions/07-templates-and-tools.md`](../../v0/concepts/permissions/07-templates-and-tools.md) §"audit_class Composition".
- Prerequisites: none.
- Deliverables: `compose_audit_class(org_default, template_ar, override) -> AuditClass` using AuditClass ordering; wired into grant-mint.
- Unblocks: nothing; audit-integrity hardening.

### Authority chain

**CH-14 — `system:genesis` axiom + authority-chain walker + revocation cascade** · ⚠HIGH · ~4 days
- Drifts closed: **D-new-14** (walker + genesis), **D-new-18** (full-tree revocation cascade).
- Concept docs: [`permissions/02-auth-request.md`](../../v0/concepts/permissions/02-auth-request.md) §"System Bootstrap Template", [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Authority Chain", [`permissions/08-worked-example.md`](../../v0/concepts/permissions/08-worked-example.md) §"Revocation cascade", [`permissions/README.md`](../../v0/concepts/permissions/README.md) §"Provenance".
- Prerequisites: none (but tight coupling with CH-09/10 Consent model via provenance chain).
- Deliverables: `system:genesis` constant + bootstrap root AR at system-init; `walk_provenance_chain(grant) -> Vec<AuthRequest>` repo method; full-tree revocation cascade in Template-revoke handler (audit current single-hop first; extend if incomplete); acceptance tests assert every grant traces to bootstrap + multi-hop revoke cascade correct.

### Permission-gate remediation (the M5-defining closer)

**CH-15 — Real permission-check gate at session launch** · ⚠HIGH · ~2 days · **M5 critical path**
- Drifts closed: **D4.1** (primary; advisory-only retired).
- Concept docs: [`permissions/04-manifest-and-resolution.md`](../../v0/concepts/permissions/04-manifest-and-resolution.md) (enforcement), [`permissions/07-templates-and-tools.md`](../../v0/concepts/permissions/07-templates-and-tools.md) (Template A grant extension).
- Prerequisites: **CH-02** (real transcripts), **CH-04** (typed actions for manifest), **CH-06** (grammar for manifest selectors).
- Deliverables: `domain::permissions::builders::build_session_launch_manifest(project_id, tools)` with actions `session.start` / `session.tool_invoke` / `session.read_memory` on `session:<project_id>`; extend Template A grant issuance to cover session actions (back-compat migration path for existing grants); flip launch.rs Steps 1–6 from advisory-log to hard-deny. ADR-0033 (session-launch manifest shape).

### Identity

**CH-16 — Identity node materialization** · ⚠HIGH · ~3 days
- Drifts closed: **D-new-01** (4-field Identity materialization), **D-new-23** (Human Agent Identity guard — finalises from CH-01).
- Concept docs: [`agent.md`](../../v0/concepts/agent.md) §"Identity (Emergent)", [`human-agent.md`](../../v0/concepts/human-agent.md) §"No Identity", [`ontology.md`](../../v0/concepts/ontology.md) §"Node Types — Identity".
- Prerequisites: **CH-02** (needs real transcripts for extraction-driven updates); **CH-01** (Human Agent kind field + lifecycle).
- Deliverables: Identity struct with 4 fields (`self_description: String`, `lived: LivedExperience`, `witnessed: WitnessedExperience`, `embedding: Vec<f32>`); migration 0006/0007 adds `identity` table; `upsert_identity` repo method; Human Agent kind guard in identity-creation path; reactive update path wiring (hooked into CH-21 memory-extraction listener at M5.2).

### Live SSE

**CH-17 — Live SSE tail endpoint** · ⚠HIGH · ~1 day
- Drifts closed: **D7.1**.
- Concept docs: [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"session lifecycle — live events".
- Prerequisites: **CH-02** (real events to stream).
- Deliverables: `tokio::broadcast` channel added to `BabyPhiSessionRecorder`; new handler `GET /api/v0/sessions/:id/events` streams `RecorderEvent`; axum SSE with keep-alive; CLI default-tail flips from "deferred" to real stream consumption. ADR-0035 (SSE broadcast fan-out).

### AuthRequest hardening

**CH-18 — AuthRequest per-state ACL enforcement** · ~2 days
- Drifts closed: **D-new-12**.
- Concept docs: [`permissions/02-auth-request.md`](../../v0/concepts/permissions/02-auth-request.md) §"Per-State Access Matrix".
- Prerequisites: none.
- Deliverables: `check_auth_request_access(ar, principal, intended_op) -> Result<...>` with state-dependent rules; wired into every AR read/write + state-transition handler.

### Convention ratification (doc-only)

**CH-19 — Bucket B ratification (ADR-driven shape choices)** · ~2 days · doc-only
- Drifts closed: **D5.2** (audit events at server::platform::), **D5.3** (find_adoption_ar most-recent), **D6.3** (system-agent 3-way union bucketing), **D7.2** (--model-config-id additive), **D7.4** (page-11 web-side retrofit), **D7.5** (no new web tests, deferred to Playwright), **D-new-21** (edge-count doc recount), **D-new-25** (Inbox/Outbox messages deferred to M6), **D-new-27** (token-economy fields deferred), **D-new-30** (Org/Project template-as-config refresh).
- Concept docs: various; many require targeted refresh paragraphs.
- Prerequisites: none.
- Deliverables: 1 consolidated ADR covering audit-event placement + bucketing convention + retrofit location + test-strategy; targeted concept-doc refreshes per drift; no code change.

**CH-20 — Bucket C convention confirm-in-place (existing 16 items)** · ~1 day · doc-only
- Drifts closed: **D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6** — all pattern/convention decisions, all honored in shipped code.
- Deliverables: 1 consolidated convention doc at `v0/conventions/*.md` covering persistence idioms (CREATE-not-UPDATE, LET-first RELATE, wrap-inner nested not flatten), event-bus wiring, serde-rename rationale, audit-event placement (cross-ref CH-19), Next.js inline server actions. Status of each drift flips `discovered → accepted-as-is`.

### M5.2 inherited chunks (from original M5 P8+P9 scope)

**CH-21 — Memory-extraction listener body (original M5/P8a)** · ⚠HIGH · ~1.5 days
- Drifts closed: **D6.1** (first call site — memory-extraction fires populate SystemAgentRuntimeStatus tile).
- Concept docs: [`system-agents.md`](../../v0/concepts/system-agents.md) §"Memory Extraction Agent", [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) §"Supervisor Extraction".
- Prerequisites: **CH-02** (real transcripts), **CH-06** (selector grammar for memory tag-predicate matching), **CH-16** (Identity updates hooked here).
- Deliverables: `MemoryExtractionListener::on_event` body per plan archive §P8; emits `MemoryExtracted` audit events with structured tag list; `record_system_agent_fire` called at top. Failure modes per concept: queue saturation / agent disabled / retry exhausted.

**CH-22 — Agent-catalog listener body + D6.1 full close (original M5/P8b)** · ⚠HIGH · ~1 day
- Drifts closed: **D6.1** full closure (2nd call site).
- Concept docs: [`system-agents.md`](../../v0/concepts/system-agents.md) §"Agent Catalog Agent".
- Prerequisites: **CH-01** (Agent.active needed for archive-state propagation to catalog), **CH-21**.
- Deliverables: `AgentCatalogListener::on_event` body handles 8 DomainEvent variants; `record_system_agent_fire` wired.

**CH-23 — Template C+D end-to-end + cross-listener acceptance (original M5/P8c)** · ~0.5 day
- Drifts closed: verification only — confirms existing Template C/D listeners fire correctly end-to-end.
- Concept docs: [`permissions/07-templates-and-tools.md`](../../v0/concepts/permissions/07-templates-and-tools.md) §"Template C/D".
- Prerequisites: **CH-21**, **CH-22**.
- Deliverables: `acceptance_system_flows_s05.rs` — 4 scenarios: Template C fires on MANAGES / Template D fires on HAS_AGENT_SUPERVISOR / A+C+D simultaneous / cross-listener ordering.

**CH-24 — Carryover re-verification + M5 final seal (original M5/P8d + P9)** · ~2 days (shipped ~2.4 days w/ scope expansion)
- Drifts closed: **1 (D-CH24-recent-sessions-api-flip — mid-cycle scope expansion approved at gate-2.5)** (originally planned: none new; re-verifies all 5 M4→M5 carryovers (C-M5-2/3/4/5/6) against real `agent_loop` output, confirming they're not hollow).
- Deliverables: re-run acceptance suite with real-loop output; 5 per-page acceptance files (`acceptance_m5_orgs/projects/agents/sessions/memory.rs` per F1.B user-lock — re-interprets the forward-scope `acceptance_m5.rs` literal as a 5-file split for finer-grained CI failure isolation) cross-page e2e; `e2e_first_session.rs` subprocess fixture (lives in `cli/tests/` because `CARGO_BIN_EXE_phi` is package-scoped to the `cli` package per P-NEW-TESTS deviation #5); CI extensions; ops runbook M5 section; M5.2 troubleshooting doc; phi-core reuse map; independent 3-agent re-audit targeting ≥99% composite; milestone tag `v0.1-m5` (user-managed).
- **Mid-cycle scope expansion**: C-M5-3 API-surface flip closed via P-FLIP-RECENT-SESSIONS + ADR-0059 (LIMIT 10 / dedicated repo method `list_recent_sessions_for_project` / `RecentSessionEntry` 6-field shape; 7th field `started_by_display_name` deferred to follow-up chunk pending Agent-table-join design). User-locked at gate-2 to close in-chunk rather than defer; gate-2.5 forks F-D59.2 + F-D59.3 user-locked DIVERGENT from planner-recommendations (richer + more-defensive options chosen).

---

## §2 — Original M5/P8 + P9 scope accounted for

- **P8a memory-extraction listener** → **CH-21**
- **P8b agent-catalog listener** → **CH-22**
- **P8c Template C/D end-to-end** → **CH-23**
- **P8d carryover re-verification** → **CH-24** (merged with P9 seal)
- **P9 seal + 3-agent audit + milestone tag** → **CH-24**

No M5 P8/P9 commitments orphaned.

---

## §2.5 — M5.3 carve-out — philosophy alignment foundation work

> **Purpose**: post-CH-21 philosophy alignment audit (2026-04-28) surfaced two foundational gaps the user has accepted as load-bearing. M5.3 is a dedicated carve-out closing those gaps **between M5 final seal (CH-24) and M6 start** so that M6+ chunks plan against a unified ownership-and-resource model. Gaps deferred to V1 would compound into ~8–15 engineer-days of refactor at M7+; M5.3 closes them in ~5–7 engineer-days now.
>
> **Audit + clarification record**: [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../core-philosophy-check/2026-04-28-philosophy-alignment-audit.md). User clarifications applied at §4.1 + §4.2.
>
> **Binding philosophy source**: [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) (promoted from `plan/core-philosophy-check/core-philosophy.md` at the M5.3 announcement-plan seal).
>
> **M5.3 drift catalogue**: [`v0/implementation/m5_3/drifts/`](../../v0/implementation/m5_3/drifts/) — D-philosophy-01 + D-philosophy-02 with their own README + concept-audit matrix. Parallel to (not under) `m5_1/drifts/`.
>
> **Plan archive (verbatim)**: [`plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`](../core-philosophy-check/525d2085-m5-3-announcement-plan.md).

### CH-25 — Agent-as-creator-and-owner of Org/Project (philosophy §4.1)

- **Drifts closed**: **D-philosophy-01** (HIGH) — see [`m5_3/drifts/D-philosophy-01.md`](../../v0/implementation/m5_3/drifts/D-philosophy-01.md).
- **Concept docs**: `core-philosophy.md` ("Agent owns Organization", "Agent create Projects"); `agent.md` ("Agent spawns other Agents (Ownership)").
- **Prerequisites**: **CH-24** (M5 final seal). Independent of CH-23.
- **Deliverables**:
  - New `Owns` edge variant in `EdgeKind` enum (or extended `Created` with Agent→Org/Project type pairs).
  - Bootstrap-claim flow + `apply_project_creation` compound tx emit the edge.
  - Permission Check gains an "owner-grant" rule (auto-issue `[admin, transfer]` for the owner-Agent over the child Org/Project).
  - Acceptance: owner-Agent can disable a child Agent without an explicit grant.
  - New ADR (next-free at chunk-open; likely **ADR-0042**).
- **Estimated effort**: ~3 engineer-days.
- **Carry-forward investigation from CH-24 retro R5 (deferred 2026-05-11)**: at CH-25 chunk-open or first retrospective, investigate the `permissions-audit` skill's window-filter logic. CH-24 retrospective reported `Total tool calls observed: 0` for the cycle window `2026-05-11T12:48:36Z → 2026-05-11T19:36:05Z` but the actual `.claude/tool-use.log` carried 1290 entries within that window (orchestrator spot-check). Likely a `jq` window-predicate bug, file-path resolution issue, or stdin-vs-file read mismatch in the skill. **Not a hook-daemon outage**; the telemetry pipeline is healthy. Goal: surface the skill's actual query mechanism, identify why it returned 0, fix the predicate, validate against CH-25 cycle window. CH-25 retrospector should run the skill manually + sanity-check against `grep -c "<cycle-date>" tool-use.log` before publishing §3.5 numbers.

### CH-26 — Org/Project as Composite resources (philosophy §4.2)

- **Drifts closed**: **D-philosophy-02** (HIGH) — see [`m5_3/drifts/D-philosophy-02.md`](../../v0/implementation/m5_3/drifts/D-philosophy-02.md).
- **Concept docs**: `core-philosophy.md` ("Organization has Projects (A Resource Type)"); `permissions/01-resource-ontology.md` (extends Composite list).
- **Prerequisites**: **CH-25** (the unified-resource-model rule presupposes the new owner edge).
- **Deliverables**:
  - `Composite::OrganizationObject` + `Composite::ProjectObject` variants with `constituents()`.
  - Migration to backfill instance-identity tags on existing Organization + Project rows (cf CH-06 + CH-16 migration patterns).
  - Refactor admin-page-1 (Orgs) + admin-page-3 (Projects) handlers to route through Permission Check engine instead of bespoke gates.
  - Acceptance: Permission Check matches `org:O` and `project:P` as resource types in selectors.
  - New ADR (next-free; likely **ADR-0043**).
- **Estimated effort**: ~3-4 engineer-days.

### Post-M5.3 actions

- Re-run philosophy alignment audit (same 3-Explore-agent shape as 2026-04-28). Goal: confirm 12 → 14 ALIGNED claims, surface any third-order ripple effects.
- M6 plan-mode opens against the now-unified foundation.
- Smaller philosophy follow-ups (audit §4.5 `Memory.source_session_id`, audit §4.6 concept-`agent.md` field cleanup) route to their natural homes (M6-DEFERRED-04, next docs sweep) — NOT in M5.3.

No M5.3 commitments orphaned.

---

## §3 — M6+ scope (deferred-milestone scope markers)

Scope markers for drifts explicitly deferred past M5 close. Each maps to its target milestone per base plan [§M6 / §M7 / §M7b](../build/build-plan-v01-36d0c6c5.md). Per-milestone plans will produce their own detailed chunks.

### M6-DEFERRED-01 — Memory contract + Memory operations
- Drifts: **D-new-16** (recall/store/delete actions), **D-new-28** (memory_type enum — decide tag-vs-enum at M6 plan open).
- Cross-ref: base plan C-M6-1 pinned carryover (Memory node tier + contract + ownership-by-multi-tag + permission-over-time retrieval).
- Prerequisite chain from M5.1: **CH-02** (real transcripts) + **CH-06** (grammar for tag-predicate recall) + **CH-21** (extraction wiring lands at M5.2).
- Target: M6 primary scope.

### M6-DEFERRED-02 — Inter-agent messaging (Inbox/Outbox materialization)
- Drifts: **D-new-25** (AgentMessage embedding on Inbox/Outbox composites).
- Cross-ref: base plan M6 "agent self-service surfaces".
- Target: M6.

### M6-DEFERRED-03 — Identity embedding provider integration
- Drifts: covered by ADR-0038 §D38.3 deferral block (no separate drift file — surfaced via concept-`agent.md` § "Scoping the embedding model" silent-in-code rows).
- Cross-ref: CH-16 / ADR-0038 §D38.3; concept-`agent.md` § "Scoping the embedding model" + "Model change is an admin event".
- Target: M6 — populate `Identity.embedding` via a platform-bootstrap embedding model + re-embed batch path.

### M6-DEFERRED-04 — Memory-extraction LLM supervisor body
- Drifts: covered by ADR-0040 §D40.7 Out-of-Scope block (no separate drift file — surfaced via concept-`system-agents.md` §"Memory Extraction Agent — Behaviour 2/3/4" + concept-`permissions/05-memory-sessions.md` §"Supervisor Extraction" silent-in-code rows; CH-21 ships heuristic v0).
- Cross-ref: CH-21 / ADR-0040; concept-`system-agents.md` § Memory Extraction Agent (Behaviour + Grants + Allocation Rules); concept-`permissions/05-memory-sessions.md` § Supervisor Extraction; ADR-0041 (audit class for the events the LLM body will continue to emit).
- Prerequisite chain from M5.2: **CH-21** (heuristic listener body + storage substrate + audit chain) lands at M5.2; LLM upgrade replaces the heuristic body without changing the substrate.
- Target: M6 — full LLM-driven supervisor agent loop (run `agent_loop` against the session transcript, identify candidates with judgment, route each memory to the correct pool per `Allocation Rules`, write via `store_memory` tool with grant enforcement, multi-memory-per-session emission, supervisor-as-actor pattern per ADR-0040 §D40.5).

### M6-or-M7-DEFERRED — Token economy fields + Worth formula
- Drifts: **D-new-27** (rating_window, total_tokens_earned/consumed, Worth).
- Target: whichever milestone introduces contracts/bidding.

### M7-DEFERRED-01 — Channel schema enrichment
- Drifts: **D-new-24** (Channel address/status/priority/metadata + WebUI/API/SMS/Custom kinds).
- Target: M7 operator-surface polish.

### M7-DEFERRED-02 — Task node materialization
- Drifts: **D-new-26** (Task full field set + 7-state status flow).
- Target: M7 or later (task/bidding flow).

### M7b-DEFERRED-01 — AuthRequest retention 2-tier storage
- Drifts: **D-new-15** (archive transition + `inspect_archived` retrieval gate).
- Target: M7b production-hardening milestone.

### M7b-DEFERRED-02 — K8s microservices carve-out (added 2026-04-24 by CH-K8S-PREP)
- Drifts: none from the M5.1 catalogue (all 8 items are sourced from CH-K8S-PREP prep refactors, not concept-vs-code drift).
- **Strategic input**: [`v0/implementation/m7b/architecture/k8s-microservices-readiness.md`](../../v0/implementation/m7b/architecture/k8s-microservices-readiness.md) — 8 K8s blockers, 7 microservice boundaries, 10-step migration order, ~35 engineer-day rough estimate.
- **Tactical input**: [`v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md`](../../v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) — 8 specific items the CH-K8S-PREP prep refactors named M7b as the owner of (CHK8S-D-01 through CHK8S-D-08), each with provenance + M7b sub-task assignment.
- **Trait contracts to satisfy**: per [ADR-0033](../../v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) §D33.1 / §D33.2 / §D33.4 conforming-impl criteria. M7b adapter chunks must satisfy these contracts; new ADRs may supersede each sub-decision as backend choices land.
- **Source chunk plan**: [`forward-scope/ab49f22b-k8s-microservices-readiness-plan.md`](./ab49f22b-k8s-microservices-readiness-plan.md) (verbatim copy of the approved CH-K8S-PREP plan).
- Target: M7b production-hardening milestone (per `baby-phi/CLAUDE.md` § Scope).

### M6+-OPEN-01 — AgentProfile cardinality re-evaluation (1:1 → N:1 template-sharing) (added 2026-04-27 by CH-01)

- **Status: open question — may or may not be pursued; not a committed deferred-scope item.** Distinguished from the `M*-DEFERRED-*` markers above (which are scope items that *will* happen) by the `OPEN` prefix introduced under §7 Q8 below.
- **Origin**: surfaced during CH-01 chunk-open by user during plan review (2026-04-27). User questioned why `domain::Agent` ↔ `domain::AgentProfile` is 1:1 instead of N:1 (multiple agents share one profile / template-sharing). Investigation confirmed the current 1:1 cardinality is concept-mandated by [`ontology.md`](../../v0/concepts/ontology.md) line 92 (*"Agent | HAS_PROFILE | AgentProfile | 1:1 | Blueprint identity"*) + [`agent.md`](../../v0/concepts/agent.md) §"Soul" lines 157–169 (*"The Soul is the agent's genetics — defined at creation, never mutated"*) + schema UNIQUE constraint at [`migrations/0001_initial.surql`](../../../../modules/crates/store/migrations/0001_initial.surql) line 131 (`DEFINE INDEX agent_profile_agent_id ... FIELDS agent_id UNIQUE`).
- **Question to evaluate**: should baby-phi adopt N:1 template-sharing (multiple agents share one profile row) instead of the current 1:1 model? Argument FOR: profiles are templates ("intern coder", "research assistant"), template-sharing fits standard infrastructure patterns (Kubernetes ConfigMaps, Helm values), ephemeral `AgentContext` makes profile sharing structurally feasible without disrupting runtime semantics. Argument AGAINST: concept-mandated profile-as-genetics in `agent.md` §Soul; per-agent governance fields naturally belong on the profile row (`parallelize`, `model_config_id`, `mock_response`); audit clarity (one profile change = one agent's change, not "now visible to 5 agents"); org-level isolation (each agent's profile lives in its owning org's roster).
- **Required if pursued**:
  - Concept-doc amendment to `agent.md` §"Soul" + `ontology.md` cardinality line.
  - New ADR documenting the redesign rationale + migration plan.
  - Schema migration: drop the UNIQUE constraint; introduce `uses_profile: FROM agent TO agent_profile` 1:N edge; relocate per-agent governance fields to either Agent-side columns or `agent_profile_override` table.
  - Refactor of `apply_agent_creation`, `upsert_agent_profile`, `get_agent_profile_for_agent`, `in_memory.rs` validation, `repo_impl.rs` upsert tx.
  - Decision about where per-agent overrides live in a shared-profile world (`parallelize`, `model_config_id`, `mock_response`).
  - Data migration to re-key existing per-agent profile rows.
- **Scope-defer rationale**: the redesign requires concept-doc amendment first; baby-phi's M5 scope is to align code with current concepts (M5.1 governing principle: *"concepts = source of truth, code aligns to them"*). Concept re-evaluation is a separate workflow that should not be conflated with drift remediation.
- **Target**: M6 plan-open (when "Memory contract + Memory operations" lands and the data model is being revisited anyway), or as a standalone concept re-evaluation chunk before then if the user prioritises. **Decision-maker is the user at M6 plan-open** — this entry exists to make the question recoverable, not to commit to action.
- **Provenance**: CH-01 plan [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](./../build/ch-01-agent-durable-lifecycle-2aa37c80.md) §P1 conceptual context (R1 sub-section).

---

## §4 — Chunk dependency graph

Simple `A → B` means A must close before B can open cleanly.

### Critical paths

**M5 completion critical path** (sequence-dependent):
```
CH-01 ─┐
        ├─> CH-22 ──> CH-23 ──> CH-24  (M5 seal)
CH-02 ──┼──> CH-21 ──┘
        ├──────────> CH-15 ──────────┘
        ├──────────> CH-16
        ├──────────> CH-17
CH-04 ──┼─> CH-05 ─> CH-12 ───────────┐
        ├─> CH-08                     │
        └─> CH-15                     │
CH-06 ──┼─> CH-07                     ├─> (all permission-foundation
        ├─> CH-12                     │    needed for M5 seal to pass)
        ├─> CH-15                     │
        └─> M6-DEFERRED-01             │
CH-09 ──> CH-10 ──> CH-11 ────────────┘
```

**Parallelizable foundation work** (can run concurrently):
- CH-01, CH-02, CH-03, CH-04, CH-06, CH-09, CH-14, CH-18 (all have no prereq except each other via different edges)

### Individual dependency edges

- **CH-01** (agent lifecycle) → **CH-22** (catalog listener needs Agent.active)
- **CH-02** (agent_loop) → **CH-15** / **CH-16** / **CH-17** / **CH-21** / **CH-24** (all consume real transcripts/events)
- **CH-04** (typed actions) → **CH-05** (validator) / **CH-08** (cardinality) / **CH-15** (manifest)
- **CH-05** (validator) → **CH-12** (frozen-tag)
- **CH-06** (selector grammar) → **CH-07** (cascade) / **CH-12** (frozen-tag) / **CH-15** (manifest) / **M6-DEFERRED-01** (memory tag-predicates)
- **CH-09** (Consent struct) → **CH-10** (state machine) → **CH-11** (Per-Session gating)
- **CH-14** (authority chain) → independent; provenance walker reusable downstream
- **CH-21** (memory-extraction) → **CH-22** (catalog) → **CH-23** (cross-listener) → **CH-24** (seal)
- **CH-24** (M5 seal) → **CH-25** (Agent-owns-Org/Project edge) → **CH-26** (Org/Project as Composite resources) → **M6 plan-mode open** (M5.3 carve-out per §2.5)

### M5.3 carve-out path (post-M5-seal, pre-M6)

```
                        CH-24 (M5 seal)
                              │
                              v
                        M5.3 carve-out
                        └─> CH-25 ──> CH-26 ──> M6 plan-mode
                                                  │
                                                  v
                                           M6-DEFERRED-01..04
```

Both M5.3 chunks file against new HIGH drifts in [`v0/implementation/m5_3/drifts/`](../../v0/implementation/m5_3/drifts/) (D-philosophy-01 + D-philosophy-02).

### Effort totals

- **M5-close scope** (CH-01 through CH-20 + CH-21 through CH-24): ~53 engineer-days of work
- **M5.3 carve-out** (CH-25 + CH-26): ~6-7 engineer-days
- **M6+ deferred scope** (6 markers): estimated separately at each milestone's own plan

---

## §5 — Per-chunk scope summary table

| Chunk | Title | Severity | Effort | Concept docs | Prerequisites | M5 close? |
|---|---|---|---|---|---|---|
| CH-01 | Agent durable lifecycle | HIGH | 2d | agent / system-agents / human-agent | — | yes |
| CH-02 | Real `agent_loop` + MockProvider | HIGH | 5d | phi-core-mapping / permissions/05 / agent / system-agents | — | yes |
| CH-03 | Storage-backend refresh + configurability framing | HIGH | 1d | coordination | — | yes |
| CH-04 | Typed action vocabulary + matrix | MED | 2.5d | permissions/03 | — | yes |
| CH-05 | Manifest validator at publish time | HIGH | 2d | permissions/04, 07, 09 | CH-04 | yes |
| CH-06 | Selector grammar (PEG) + instance tags | HIGH | 5–7d | permissions/09, 01, 05 | — | yes |
| CH-07 | Multi-scope cascade + contractor model | HIGH | 3d | permissions/04, 06, 08 | CH-06 | yes |
| CH-08 | allocate/transfer cardinality + refinements | HIGH | 2d | permissions/02, 03 | CH-04 | yes |
| CH-09 | Consent node full shape | HIGH | 1d | permissions/06 | — | yes |
| CH-10 | Consent lifecycle state machine | HIGH | 1d | permissions/06 | CH-09 | yes |
| CH-11 | Per-Session consent gating | HIGH | 2d | permissions/06 | CH-09, CH-10 | yes |
| CH-12 | Frozen session-tag immutability | HIGH | 1.5d | permissions/05 | CH-05, CH-06 | yes |
| CH-13 | audit_class composition | MED | 1d | permissions/07 | — | yes |
| CH-14 | system:genesis + authority chain + revocation cascade | HIGH | 4d | permissions/02, 04, 08; README | — | yes |
| CH-15 | Real permission-check gate at launch | HIGH | 2d | permissions/04, 07 | CH-02, CH-04, CH-06 | yes |
| CH-16 | Identity node materialization | HIGH | 3d | agent / human-agent / ontology | CH-01, CH-02 | yes |
| CH-17 | Live SSE tail endpoint | HIGH | 1d | permissions/05 | CH-02 | yes |
| CH-18 | AuthRequest per-state ACL | MED | 2d | permissions/02 | — | yes |
| CH-19 | Bucket B ratification (ADR/refresh) | MED | 2d | various (refresh paragraphs) | — | yes |
| CH-20 | Bucket C confirm-in-place (14 items) | LOW | 1d | various (conventions doc) | — | yes |
| CH-21 | Memory-extraction listener body | HIGH | 1.5d | system-agents / permissions/05 | CH-02, CH-06, CH-16 | yes |
| CH-22 | Agent-catalog listener + D6.1 close | HIGH | 1d | system-agents | CH-01, CH-21 | yes |
| CH-23 | Template C/D end-to-end verification | MED | 0.5d | permissions/07 | CH-21, CH-22 | yes |
| CH-24 | Carryover re-verification + M5 seal | HIGH | 2d | all M5 | all above | yes (seal) |
| CH-25 | Agent-owns-Org/Project edge + owner-grant rule | HIGH | 3d | core-philosophy / agent / permissions/04 | CH-24 | M5.3 (post-seal) |
| CH-26 | Org/Project as Composite resources (unified resource model) | HIGH | 3-4d | core-philosophy / ontology / permissions/01 | CH-25 | M5.3 (post-seal) |

**Total for M5 close: ~53 engineer-days (~10-11 weeks serial; ~5-6 weeks with parallelization of foundation tier).**

**Total for M5.3 carve-out (CH-25 + CH-26): ~6-7 engineer-days serial. See §2.5 for full deliverables; backed by D-philosophy-01 + D-philosophy-02 in [`m5_3/drifts/`](../../v0/implementation/m5_3/drifts/).**

---

## §6 — Open questions (ALL 7 ANSWERED at M5.1/P3 close, 2026-04-24)

Questions originally surfaced here during P3 drafting; answers captured via `AskUserQuestion` on 2026-04-24. See [`§7 — Planning decisions`](#§7--planning-decisions-answers-captured-2026-04-24) below for the authoritative answer log.

## §7 — Planning decisions (answers captured 2026-04-24)

Each decision below is binding for P4 template drafting + all subsequent per-chunk-plan drafting. Decisions override any conflicting text in §1–§5 above (CH-03 scope expansion already folded in).

### Q1 — Storage-backend ratification path

**Decided:** Refresh `coordination.md` §Storage-backend AND frame the storage backend as **configurable** (not hardcoded to SurrealDB) AND write a new ADR documenting the decision + the **criteria a conforming backend must satisfy** to be plug-in-eligible (transactional semantics, compound-tx support, RELATION edge semantics, FLEXIBLE-TYPE-object support for phi-core wraps, migration idempotency, SCHEMAFULL table support, UNIQUE index enforcement).

**Scope impact:** CH-03 effort 0.5 → **1 day**; deliverables expanded per §1 CH-03 block.

**Why it matters:** Honors "concept = source of truth" AND makes explicit that baby-phi's architecture doesn't lock into SurrealDB — SurrealDB is the configured initial impl; any future backend-migration chunk has a documented contract to satisfy.

### Q2 — CH-06 (Selector grammar PEG) split decision

**Decided:** Defer split decision to CH-06 chunk-open. Forward-scope lists CH-06 as "likely split candidate" — the chunk-plan drafter decides at design time whether to ship as CH-06 unified or CH-06a/b.

**Scope impact:** None to forward-scope; flagged in §1 CH-06 block already.

### Q3 — Consent model triad sequencing (CH-09 / CH-10 / CH-11)

**Decided:** Keep as three separate chunks at the planning layer. Each has its own ExitPlanMode approval + 4-aspect close. Implementation may choose to combine into one PR at execution time; planning stays chunked for auditable per-chunk close reviews.

**Scope impact:** None; already structured this way in §1 and §5.

### Q4 — M5-close chunk ordering

**Decided:** **User-decided per-chunk at chunk-open time.** The forward-scope inventory enumerates chunks + dependency edges; user picks the next chunk from the inventory at each chunk-open based on current priority + what was learned in prior chunks.

**Scope impact:** None structurally; P4 template will reflect this in the "Pre-chunk gate" section (no pre-committed sequence).

### Q5 — M5 scope — what closes before M5 tag?

**Decided:** **All 17 HIGH drifts close before M5 tag.** MEDIUM drifts evaluated case-by-case at each chunk-open for defer-to-M6 vs close-at-M5. LOW drifts all close in M5 via CH-19 / CH-20 (pure-doc).

**Scope impact:** CH-18 (AuthRequest per-state ACL, MEDIUM) and similar MEDIUM-severity chunks remain in the M5 scope list but may be deferred per-chunk at chunk-open if the user decides the trade-off at that moment. Forward-scope keeps them in-scope; operational defer decisions recorded as drift-file status transitions at per-chunk close time.

### Q6 — ADR numbering policy

**Decided:** ADR numbers assigned **at chunk-plan drafting time**. The per-chunk plan's §5 "ADRs drafted" subsection executes `ls baby-phi/docs/specs/v0/implementation/*/decisions/ | sort` to find the next free number + records it. Chunk-lifecycle-checklist step 1 enforces.

**Scope impact:** None structurally; P4 chunk-lifecycle-checklist step 1 will codify.

### Q7 — Per-chunk approval depth

**Decided:** **Same ExitPlanMode ritual for every chunk including doc-only chunks (CH-19 + CH-20).** P4 template does not fork by chunk type. Doc-only chunks approve quickly since there's no code to review, but the planning + close ritual is uniform.

**Scope impact:** P4 template remains single-format; no "lighter doc-only variant".

### Q8 — K8s microservice readiness rule (added 2026-04-27 by CH-01)

**Decided:** Every chunk plan §3.B evaluates the 7 K8s-deployability axes (A1 in-process state · A2 IPC channel · A3 pod-local resource · A4 migration runner / first-apply race · A5 trait-shape requirement · A6 cross-pod state · A7 audit hash-chain symmetry) and verifies the four ADR-0033 conforming criteria (D33.1–D33.4). Any new K8s blocker introduced by the chunk creates a new `CHK8S-D-XX` entry in [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) before chunk seal, with provenance citing the originating chunk.

**Scope impact:** [`per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md) gains a new mandatory §3.B section. CH-01 is the first chunk to apply (and ratifies the rule by codifying it across this Q8, the per-chunk template, the strategic doc, and the ledger guidance).

**Why it matters:** prevent silent K8s-deployment-debt accumulation between M5 and M7b plan-open. Each chunk pre-positions its trait surfaces incrementally instead of in a big-bang refactor at M7b. The CH-K8S-PREP precedent (4 prep refactors that pre-positioned `SessionRegistry`, `EventBus.shutdown/drain`, `SurrealStore::open_remote`, and SIGTERM handling) demonstrated this incremental approach works; Q8 generalises it.

**Pre-Q8 chunks grandfathered:** CH-02 and CH-K8S-PREP do not retroactively need §3.B (CH-K8S-PREP itself originated the 7-axis evaluation in [ADR-0033](../../v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md)).

**Open-question variant — `M6+-OPEN-*` markers in §3.** When a chunk plan-time review surfaces a *concept re-evaluation candidate* (a question whether the concept itself should be reconsidered, not a code-vs-concept drift), it is captured in §3 with the `M6+-OPEN-NN` prefix to distinguish from committed `DEFERRED-*` markers. CH-01 added the first such marker (`M6+-OPEN-01 — AgentProfile cardinality re-evaluation`).

### Q9 — User-facing documentation strategy (added 2026-04-27 by CH-22)

**Decided:** Every chunk plan §3.C evaluates a 3-tier user-facing-doc impact map (architecture / operations / user-guide under `docs/specs/v0/implementation/<milestone>/`) and either (a) updates the affected files in-chunk, or (b) explicitly defers each one with a successor-chunk reference (or `M<n>-tag-close` for milestone batches). Open-ended deferrals are not permitted. Pre-Q9 chunks (CH-01, CH-02, CH-K8S-PREP, CH-22) are grandfathered — their backfill bundled in CH-22's seal.

**Scope impact:** [`per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md) gains a new mandatory §3.C section + §10 Docs aspect extended to cover the user-facing tier. [`chunk-lifecycle-checklist.md`](../../v0/implementation/m5_1/process/chunk-lifecycle-checklist.md) Step 8 sub-action 6.B applies the §3.C map at seal. CH-22 is the codifying chunk; CH-23 onward apply the rule from chunk-open.

**Why it matters:** prevents the silent erosion that surfaced post-CH-01 / CH-02 / CH-22 — namely that chunks updated governance docs (drifts / ADRs / concept-audit matrix) but skipped the milestone-era user-facing tier (operator runbooks, walkthroughs, CLI reference, troubleshooting). Without an explicit per-chunk gate, operators reading those docs build an out-of-date mental model. The 3-tier evaluation surfaces the gap at plan time so the doc updates land as deliverables in §7 phases — not as after-the-fact appendices.

**Pre-Q9 chunks grandfathered:** CH-01, CH-02, CH-K8S-PREP, CH-22. Their backfill (bundled with CH-22's seal commit) updated `m5/architecture/{system-agents,event-bus-m5-extensions,session-launch}.md`, `m5/operations/{system-agents,session-launch,system-flows-s02-s03}-operations.md`, and `m5/user-guide/{system-agents-walkthrough,first-session-walkthrough,cli-reference-m5,troubleshooting}.md` with `<!-- CH-XX amendment -->` sections. Stub user-guide docs (P6 / P7 / P9 deferrals from the M5 plan archive) keep their stubs for prose; CH-22 backfill added only the operator-visible changes the chunks introduced — full prose closes at M5-tag-close per the existing M5 plan.

**Relationship to Q8 (K8s readiness).** Same pattern: a binding cross-cutting rule that every chunk applies, captured in a dedicated template section (Q8 → §3.B; Q9 → §3.C), enforced at the §10 close criteria. The two rules are orthogonal — Q8 governs deployment portability, Q9 governs reader fidelity.

---

---

## Post-this-document next steps

1. **M5.1/P4** — write process docs at [`v0/implementation/m5_1/process/`](../../v0/implementation/m5_1/process/): `per-chunk-planning-template.md`, `chunk-lifecycle-checklist.md`, `drift-lifecycle.md`. These lock in the ritual before any chunk opens.
2. **M5.1/P5** — seal M5.1 via independent 3-agent audit + final summary report.
3. **First implementation chunk** — user selects from §5 table; per-chunk detailed plan drafted using P4 template; approved via `ExitPlanMode`; implementation proceeds.
4. **Iterate** — repeat step 3 per chunk until all M5-close chunks seal and M5 tag ships.
