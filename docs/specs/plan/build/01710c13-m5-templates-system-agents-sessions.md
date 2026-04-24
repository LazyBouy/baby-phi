# Plan: M5 — Templates (page 12), System Agents (page 13), First Session Launch (page 14)

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section

## Context  `[STATUS: n/a]`

**Why this milestone now.** M4 closed at **99.4% composite** (805 Rust + 68 Web tests). M4 shipped admin pages 08–11 (agents + projects) and left six explicit carryovers pinned into the base build plan's §M5 subsection: C-M5-1 (Template node persistence), C-M5-2 (UsesModel edge retype), C-M5-3 (phi-core Session/LoopRecord/Turn persistence), C-M5-4 (AgentTool per-agent binding), C-M5-5 (per-agent `ModelConfig` binding + active-session gating activation), C-M5-6 (Shape B materialisation-after-approve). All six are session-launch-blocking — M5 is the right milestone to close them.

**What M5 ships.** Three admin pages as vertical slices (Rust business logic + HTTP handler + `phi` CLI subcommand + Next.js web page + acceptance tests + ops doc):
- **Page 12 — Authority Template Adoption** (3R / 5W / 4N). Approve / deny / adopt-inline / revoke-cascade for Templates A–E; extends the M4 Template-A fire pattern to Templates C + D.
- **Page 13 — System Agents Config** (4R / 4W / 3N). Tune `memory-extraction-agent` / `agent-catalog-agent`, add org-specific system agents, disable standards with strong-warning dialog, live queue-depth + last-fired indicators.
- **Page 14 — First Session Launch** (4R / 4W / 4N). Agent + Project picker → server-side Permission Check preview (steps 0–6) → session launch → live event tail → auto session-end → post-session verification (memory extracted + catalog updated). **M5's phi-core-heaviest vertical.**

Plus the three reactive listeners that make page 14's N4 checklist pass:
- **s02 — memory-extraction-agent** subscribes to `DomainEvent::SessionEnded`.
- **s03 — agent-catalog-agent** subscribes to 8 edge-change and agent-lifecycle variants.
- **s05 — template-adoption grant fires** extends M4's Template-A listener to also cover Templates C and D.

**What M5 does NOT ship** (explicit deferrals, pinned as base-plan carryovers):
- **C-M6-1 — Memory node tier + interface contract + permission-over-time retrieval** (NEW carryover from M5/D6 decision). s02 fires at M5 and emits `MemoryExtracted` audit events + tagging metadata, but does NOT persist `Memory` nodes. M6 defines the contract any memory system (including baby-phi's default) must satisfy: (i) **well-defined memory interface contract** that any integrator (including baby-phi's default implementation) MUST implement, (ii) **ownership via multi-tag** (agent / group / project / org) with a single session/memory allowed to carry multiple tags simultaneously, (iii) **permission-over-time retrieval** — agents retrieve only memory they have current grants for at retrieval time (grants revoked after extraction forfeit read access). **Pinned at P0 via a new subsection `#### Carryovers from M5 — must-pick-up at M6 detailed planning` inside the §M6 section of [`/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md)** (same file that holds the M3→M5 and M4→M5 and M4→M8 carryovers).
- **Full Memory node CRUD** — moved to M6 under C-M6-1.
- **Redis-backed `SessionRegistry`** — multi-worker shared-state concurrency is deferred to M7b (production hardening).
- **Template F** (recurring commitment) — not in scope; M5 ships A/B/C/D/E.

**Base plan entry**: [§M5](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md) — 4 lines of scope + six M5 carryovers. This plan is the fully-resolved version.

**Archive location for this plan**: `baby-phi/docs/specs/plan/build/<8-hex>-m5-templates-system-agents-sessions.md`. First execution step (P0) archives this verbatim.

**What M4 taught us (applied preventively to M5):**

1. **phi-core leverage pre-audit BEFORE P1 opens.** M4/P0 walked Q1/Q2/Q3 up-front; every phase-close grep matched the prediction. M5 is the phi-core-heaviest milestone yet (3 new wraps at node tier) — Part 1.5 below walks Q1/Q2/Q3 per-surface with expected import counts + positive close-audit greps.
2. **Phase-boundary pause is mandatory.** Every phase close pauses for user review before opening the next.
3. **Confidence % at every phase close, no exceptions.** **Upgraded to 4-aspect at M5**: code + docs + phi-core leverage + **archive-plan compliance** (new — cross-checks the phase's shipped code against the deliverables committed in this plan's archived copy; every bullet ✅/⚠/✗). Composite reported verbatim. See Part 4 preamble for the full discipline.
4. **Carryovers are load-bearing.** M5 picks up 6 M4 carryovers + pins C-M6-1 (Memory contract). P0 updates the base plan's M6 section explicitly.
5. **Independent 3-agent re-audit at the seal phase.** Target ≥99% composite (M4/P8 hit 99.3%).
6. **Wire-contract schema snapshots at every response tier.** M5's session + template + system-agent wire shapes all ship with phi-core-strip assertions.

---

## Part 1 — Pre-implementation gap audit  `[STATUS: ⏳ pending]`

Cross-check of admin pages 12–14 requirements + system flows s02/s03/s05 + M4 carryovers C-M5-1 through C-M5-6. Findings:

| # | Finding | Source | Fix location |
|---|---|---|---|
| G1 | **Template node uniqueness blocks multi-org adoption.** `template.name` UNIQUE index (`0001_initial.surql:231`) makes two orgs adopting Template A a duplicate-key error. D1 resolves → option (b): one row per kind, UNIQUE(kind), adoption carried by AR `provenance_template`. | C-M5-1 | P1 — migration 0005 drops + redefines index; `domain/src/model/nodes.rs::Template` stays but `adopted_by_org`/`adopted_at` are NOT added (adoption lives on the AR, not the template). |
| G2 | **`uses_model` edge points at vestigial table.** `0001_initial.surql:315`: `TYPE RELATION FROM agent TO model_config` — but M2/P6 moved the authoritative catalogue to `model_runtime`. SurrealDB's TYPE RELATION constraint rejects any edge with a mismatched `TO`. | C-M5-2 | P1 — migration 0005 redefines `uses_model` as `FROM agent TO model_runtime`; the legacy `model_config` table stays as a zombie (no writers) and can be DROPed in M7b. |
| G3 | **Session/LoopRecord/Turn node scaffolds are id-only.** `nodes.rs:792-812`: `Session { id }` / `Loop { id }` / `Turn { id }` are placeholders marked `[PLANNED M5]`. | C-M5-3 | P1 — replace with the three-way wrap: `Session { inner: phi_core::Session, owning_org, owning_project, started_by, governance_state, started_at, ended_at, tokens_spent }`; `LoopRecordNode { inner: phi_core::LoopRecord, session_id, loop_index }`; `TurnNode { inner: phi_core::Turn, loop_id, turn_index }`. M3 `OrganizationDefaultsSnapshot` pattern byte-for-byte. |
| G4 | **`RunsSession` edge variant has zero production writers.** `edges.rs:95` ships the variant; no code writes it. | C-M5-3 | P4 — session-launch compound-tx writes Session + first LoopRecord + `RELATE session -> runs_session -> project`. |
| G5 | **Session repository surface is missing.** Current `count_active_sessions_for_agent` (`repository.rs:832`) is a stub returning `Ok(0)`; no other session methods exist. | C-M5-3 + C-M5-5 | P2 — 14 new methods: `persist_session`, `append_loop_record`, `append_turn`, `append_agent_event`, `fetch_session`, `list_sessions_in_project`, `list_active_sessions_for_agent`, `mark_session_ended`, `terminate_session`, `persist_shape_b_pending`, `fetch_shape_b_pending`, `delete_shape_b_pending`, `upsert_agent_catalog_entry`, `list_agent_catalog_entries_in_org`, `upsert_system_agent_runtime_status`, `fetch_system_agent_runtime_status_for_org`. Flip `count_active_sessions_for_agent` to real query. |
| G6 | **Shape B pending payload has nowhere to live between submit and both-approve.** `create.rs:634-662` shows `approve_pending_shape_b` Approved branch returning `project: None`; the `_keep_materialise_live` (line 752-764) is a dead-code marker for M5. | C-M5-6 | P1 — migration 0005 adds `shape_b_pending_projects { auth_request_id UNIQUE, payload FLEXIBLE TYPE object }`. P2 — 3 repo methods. P4 — `approve_pending_shape_b` Approved branch reads sidecar, calls `materialise_project`, deletes sidecar, returns real `project: Some(pid)`. |
| G7 | **`AgentProfile.model_config_id` governance field is missing.** phi-core's `AgentProfile` has `config_id` (stable identity for `loop_id` composition) but no provider reference. M4/P5 deliberately deferred the per-agent binding (`update.rs:167-176`). | C-M5-5 | P1 — migration 0005 adds nullable `model_config_id` column to `agent_profile` with `#[serde(default)]`. P4 — un-defer the update.rs change arm + real 409 path when `count_active_sessions_for_agent > 0`. |
| G8 | **Authority-template registry reads are missing.** Page 12 needs list+count (pending / active / revoked / available). | R-ADMIN-12-R1/R2/R3 | P5 — new `server/src/platform/templates/` dir + 4 repo methods: `list_templates_for_org`, `count_grants_fired_by_adoption`, `list_active_grants_for_adoption`, `list_revoked_adoptions_for_org`. |
| G9 | **Template C + D fire pure-fns missing.** M4/P3 shipped `domain/src/templates/a.rs::fire_grant_on_lead_assignment`. C + D need their own. | C-M5-1 (s05 extension) | P3 — `templates/c.rs::fire_grant_on_manages_edge` + `templates/d.rs::fire_grant_on_has_agent_supervisor`. Pure-fn proptests mirror Template A. |
| G10 | **8 new `DomainEvent` variants needed.** Currently only `HasLeadEdgeCreated`. M5 needs: `SessionStarted`, `SessionEnded`, `SessionAborted`, `ManagesEdgeCreated`, `HasAgentSupervisorEdgeCreated` (for Template D), `AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged` (for s03). | C-M5-3 + s03 | P3 — extend `events/mod.rs:50` enum + serde round-trip tests. |
| G11 | **`SystemAgentRuntimeStatus` counter node missing.** Page 13's R2 needs live queue-depth, last-fired-at, effective-parallelize per system agent. | R-ADMIN-13-R2 | P1 — migration 0005 adds table; P6 — listener callbacks upsert on every fire. |
| G12 | **Permission Check preview endpoint missing.** Page 14-R3 previews steps 0–6 before launch. D5 resolves → server-side: `POST /orgs/:id/projects/:pid/sessions/preview { agent_id } → { trace: PermissionCheckTrace }`. | R-ADMIN-14-R3 | P4 — new handler re-using M1's `domain::permissions::check::run`. |
| G13 | **Parallelize-cap gate missing.** W2 requires `PARALLELIZE_CAP_REACHED` when `count_active_sessions_for_agent >= profile.blueprint.parallelize`. Depends on G5's flipped stub. | R-ADMIN-14-W2 | P4 — gate in `session::launch`. |
| G14 | **`phi session` CLI subcommand missing.** `cli/src/commands/` has no `session.rs`. D4 resolves → tail-by-default + `--detach`. | C-M5-3 | P7 — new `cli/src/commands/session.rs` with `launch`, `show`, `terminate`, `list`. Binary stays `phi` (no `baby-phi`). |
| G15 | **`AgentCatalogEntry` node missing.** s03's output is a queryable catalogue per-org; R-SYS-s03-2/3/4/5 require an upsertable row per agent. | s03 | P1 — migration 0005 adds table; P8 — s03 listener body. |
| G16 | **Session `CancellationToken` registry missing in `AppState`.** W3 terminate requires cancelling a running `tokio::spawn`-ed `agent_loop` call. Needs process-global `DashMap<SessionId, CancellationToken>`. | R-ADMIN-14-W3 | P4 — add `session_registry: Arc<DashMap<SessionId, CancellationToken>>` to `AppState`. D3 pins the per-worker cap. |
| G17 | **Config table extension.** `[session] max_concurrent = 16` needs to exist in `config/default.toml`. | D3 | P1 — extend `server::config::ServerConfig` schema + default TOML. |
| G18 | **Spec-drift IDs unregistered.** 5W + 3R + 4N for page 12, 4W + 4R + 3N for page 13, 4W + 4R + 4N for page 14, plus R-SYS-s02-* + R-SYS-s03-* + R-SYS-s05-*. | Multi | P0 — `scripts/check-spec-drift.sh` registry extension. |
| G19 | **`check-phi-core-reuse.sh` CI grep doesn't yet block Session/LoopRecord/Turn/AgentTool/AgentEvent re-declarations.** | Discipline | P0 — extend deny-list before P1 opens. |

### Confidence target: **≥ 98 % at first review**, ≥ 99 % after P9 close re-audit.

Matches the M4 bar. Risk areas: (1) Session wrap serde — phi-core's `Session.loops: Vec<LoopRecord>` nests arbitrarily; the SurrealDB schema must accept `FLEXIBLE TYPE object` for the full tree vs flatten Loop + Turn into sibling tables. We flatten (one table per tier) so per-Turn queries are cheap. (2) `tokio::spawn` + cancellation + panic safety in `session::launch`. (3) Template C + D fire semantics (concept docs use "MANAGES" / "HAS_AGENT_SUPERVISOR" for the trigger edges — confirm existence in M4 edges enum at P3). Mitigated by: Part 1.5 greps + proptest property invariants + 3-agent seal re-audit.

---

## Part 1.5 — phi-core reuse map (M5)  `[STATUS: ⏳ pending]`

**Principle** (unchanged from M2/M3/M4): baby-phi is a consumer of phi-core. Every M5 surface overlapping a phi-core type uses phi-core's type directly or wraps it; re-implementations are **reject-on-review**.

**Pre-audit discipline** (Q1/Q2/Q3 per the M3 leverage checklist §2): walked at P0 BEFORE any implementation. Per-phase close assertions pinned in Part 4.

Legend: ✅ direct reuse • 🔌 wrap (baby-phi field holds phi-core type) • ♻ inherit from snapshot (no per-agent duplication per ADR-0023) • 🏗 build-from-scratch (baby-phi-native).

### Page 12 — Authority Template Adoption (M5/P5)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Template row display | (none — baby-phi governance) | 🏗 Build-native |
| AR approval / denial | (none — M1 AR state machine) | ♻ Inherit (M1) |
| Template C / D fire pure-fns | (none — phi-core has no template/grant concept) | 🏗 Build-native |

**Q1 expected: 0** in `server/src/platform/templates/`. **Q3 rejections (explicit)**: `phi_core::types::event::AgentEvent` is orthogonal (agent-loop telemetry, not governance trigger) per `phi/CLAUDE.md` §Orthogonal surfaces.

### Page 13 — System Agents Config (M5/P6)

| Surface | phi-core type / API | Mode |
|---|---|---|
| `profile_ref` swap validation | `phi_core::agents::profile::AgentProfile` | ✅ Direct (re-use M4 pattern) |
| `parallelize` tune | (none — baby-phi governance) | 🏗 Build-native |
| `trigger` enum `{session_end, edge_change, periodic, explicit, custom_event}` | (none — governance concept, NOT `AgentEvent`) | 🏗 Build-native |
| `SystemAgentRuntimeStatus` queue-depth + last-fired-at | (none) | 🏗 Build-native |

**Q1 expected: 1** (`AgentProfile` for profile_ref validation — re-use of existing M4 path). **Q3 rejections**: the trigger enum LOOKS phi-core-shaped but is pure governance plane; explicitly rejected to prevent `AgentEvent` conflation.

### Page 14 — First Session Launch (M5/P4) **← M5's phi-core-heaviest phase**

| Surface | phi-core type / API | Mode |
|---|---|---|
| `domain::Session { inner: phi_core::Session, ... }` wrap | `phi_core::session::model::Session` | 🔌 Wrap (M3 `OrganizationDefaultsSnapshot` pattern) |
| `domain::LoopRecordNode { inner: phi_core::LoopRecord, ... }` wrap | `phi_core::session::model::LoopRecord` | 🔌 Wrap |
| `domain::TurnNode { inner: phi_core::Turn, ... }` wrap | `phi_core::session::model::Turn` | 🔌 Wrap |
| Session-launch executor | `phi_core::agent_loop`, `phi_core::agent_loop_continue` | ✅ Direct |
| Event-to-record materialisation | `phi_core::session::recorder::SessionRecorder` (composed via `BabyPhiSessionRecorder` wrap per D2) | 🔌 Wrap |
| Event stream consumer | `phi_core::types::event::AgentEvent` | ✅ Direct |
| Agent tool resolver | `phi_core::types::tool::AgentTool` | ✅ Direct |
| ModelConfig catalogue lookup | `phi_core::provider::model::ModelConfig` | ✅ Direct (C-M5-5) |
| Permission Check preview | `domain::permissions::check::run` (M1) | ♻ Inherit |
| Session registry (cancellation) | `tokio_util::sync::CancellationToken` | ✅ Direct (not phi-core, but phi-core's `agent_loop` takes this as its `cancel:` param — same primitive) |

**Q1 direct imports expected at P4 close** (10 new across M5, concentrated at P1/P3/P4):
- `use phi_core::session::model::{Session as PhiCoreSession, LoopRecord, Turn};` — in `domain/src/model/nodes.rs` (P1)
- `use phi_core::types::event::AgentEvent;` — in `domain/src/session_recorder.rs` (P3) + `server/src/platform/sessions/launch.rs` (P4)
- `use phi_core::session::recorder::SessionRecorder;` — in `domain/src/session_recorder.rs` (P3)
- `use phi_core::{agent_loop, agent_loop_continue};` — in `server/src/platform/sessions/launch.rs` (P4)
- `use phi_core::types::tool::AgentTool;` — in `server/src/platform/sessions/tools.rs` (P4)
- `use phi_core::provider::model::ModelConfig;` — in `server/src/platform/agents/update.rs` (P4, C-M5-5 un-defer)

**Q2 transitive**: `Session.inner` + `LoopRecordNode.inner` + `TurnNode.inner` transit phi-core types via serde at storage + wire tier. Drill-down endpoint `GET /api/v0/sessions/:id` carries full inner; list endpoint `GET /api/v0/projects/:pid/sessions` strips to `SessionHeader { id, agent_id, started_at, ended_at, status, turn_count }` — schema-snapshot test pins this.

**Q3 rejections (explicit module walk)**:
- `phi_core::agents::{Agent, BasicAgent}` runtime traits — M5 uses `agent_loop` free functions directly; does NOT re-instantiate phi-core's `BasicAgent` at launch time (baby-phi's governance `Agent` node is the identity; `agent_loop`'s context carries the phi-core `AgentProfile` blueprint).
- `phi_core::config::{parser, schema::AgentConfig}` — external YAML blueprint parsing rejected (page 09 CRUD is the authoritative path).
- `phi_core::session::recorder::SessionRecorderConfig::after_task` callback — used, but baby-phi does NOT use phi-core's `save_session` / `load_session` helpers (baby-phi persists to SurrealDB, not to JSON files).

### s02 — memory-extraction listener (M5/P8)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Subscribes to `DomainEvent::SessionEnded` | (baby-phi event bus) | 🏗 Build-native |
| Runs supervisor extraction loop | `phi_core::agent_loop` | ✅ Direct |
| Reads persisted Turn sequence | baby-phi's `fetch_session` repo method | ♻ Inherit |

**Q1 expected: 1** (`agent_loop`). The extractor runs the SAME phi-core primitive as the Session runtime — phi-core leverage exemplar.

### s03 — agent-catalog listener (M5/P8)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Subscribes to 8 DomainEvent variants | (baby-phi event bus) | 🏗 Build-native |
| Upserts `AgentCatalogEntry` | (baby-phi) | 🏗 Build-native |

**Q1 expected: 0**. Pure governance-plane reactive flow.

### Compile-time coercion witnesses (M4 pattern)

Three new witness fns land in `domain/src/model/nodes.rs` test module at P1:

```rust
#[allow(dead_code)] fn _is_phi_core_session(_: &phi_core::session::model::Session) {}
#[allow(dead_code)] fn _is_phi_core_loop_record(_: &phi_core::session::model::LoopRecord) {}
#[allow(dead_code)] fn _is_phi_core_turn(_: &phi_core::session::model::Turn) {}
```

Applied to `Session.inner` / `LoopRecordNode.inner` / `TurnNode.inner`. A rename in phi-core breaks the baby-phi build immediately — the M3 discipline.

### Enforcement at M5 close

- `scripts/check-phi-core-reuse.sh` extended at P0: deny-list adds `struct Session `, `struct LoopRecord `, `struct Turn `, `struct AgentTool `, `struct AgentEvent `, `struct ModelConfig ` outside `modules/crates/domain/src/model/nodes.rs` (wrap layer is the single allowed exception).
- Positive close-audit greps per phase (pinned in Part 4 per-phase subsections).
- **Total `use phi_core::` imports across `modules/crates/`**: M4 close ≈ 14 lines / 7 unique types → M5 close target ≈ 24 lines / 10 unique types (+ `Session` / `LoopRecord` / `Turn` / `AgentEvent` / `SessionRecorder` / `AgentTool` new; `AgentProfile`, `ExecutionLimits`, `ModelConfig`, `ThinkingLevel` carry over from M4).

### Three phi-core surfaces M5 MIGHT miss-leverage if not pinned

1. **`phi_core::agents::BasicAgent`** — M5 launches sessions via `agent_loop()` free function, NOT `BasicAgent::run()`. If a reviewer pushes "wouldn't `BasicAgent` be cleaner?" the answer is no — we want the raw loop so baby-phi's governance `Session` node is authoritative and phi-core's runtime agent trait stays unused. Pin in ADR-0029.
2. **`phi_core::SessionRecorderConfig` with `include_streaming_events = true`** — tempting to store every `MessageUpdate` delta for a rich replay UX; don't. SurrealDB row volume balloons. Default `false` at M5; revisit at M7b if the replay UX demands it.
3. **`phi_core::session::save_session` / `load_session`** — JSON file helpers. Rejected; baby-phi persists to SurrealDB. Pin in Part 1.5 §Q3 so reviewers don't rebuild the wheel.

---

## Part 2 — Commitment ledger  `[STATUS: ⏳ pending]`

| # | Commitment | M5 deliverable | Phase | Verification |
|---|---|---|---|---|
| C1 | M4 post-flight delta log | 10-item audit confirming M4→M5 state; written to `m5/architecture/m4-postflight-delta.md` | P0 | doc-link check |
| C2 | Migration 0005 (forward-only) | 8 schema changes: UNIQUE(template.kind), `uses_model` retype, 3 session tables (`session`, `loop_record`, `turn`), `shape_b_pending_projects` sidecar, `agent_profile.model_config_id` column, `agent_catalog_entry` table, `system_agent_runtime_status` table, `runs_session` relation | P1 | `store/tests/migrations_0005_test.rs` — apply / noop / fresh-DB |
| C3 | 3-way Session/LoopRecord/Turn wrap | Full M3-pattern wrap in `nodes.rs`, replacing scaffolds; compile-time coercion witnesses; serde round-trip tests | P1 | `domain/tests/m5_wrap_roundtrip.rs` |
| C4 | `ShapeBPendingProject` composite + 3 repo methods | New composite in `composites_m5.rs`; repo `persist_shape_b_pending` / `fetch_shape_b_pending` / `delete_shape_b_pending` | P1 (struct) + P2 (repo) | `store/tests/shape_b_sidecar_test.rs` |
| C5 | `AgentProfile.model_config_id` field | Nullable column on `agent_profile` table; `#[serde(default)]` for backward compat | P1 | migration test + `acceptance_agents_profile::model_config_id_serde_round_trip` |
| C6 | 14 new session-surface repo methods | In-memory + SurrealDB impls, parity-tested | P2 | `store/tests/session_repo_parity.rs` |
| C7 | Flip `count_active_sessions_for_agent` from stub to real query | Joins `session` table on `started_by_agent` filtered to non-terminal states | P2 | `store/tests/active_sessions_count_test.rs` |
| C8 | `DomainEvent` extended with 8 new variants | `SessionStarted/Ended/Aborted`, `ManagesEdgeCreated`, `HasAgentSupervisorEdgeCreated`, `AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged` + serde round-trip | P3 | `domain/src/events/mod.rs::tests` |
| C9 | `BabyPhiSessionRecorder` wrap (D2 path a) | Composes `phi_core::SessionRecorder` + adds SurrealDB persist hooks | P3 | `domain/tests/session_recorder_wrap_test.rs` |
| C10 | Template C + D fire pure-fns | `templates/c.rs::fire_grant_on_manages_edge` + `templates/d.rs::fire_grant_on_has_agent_supervisor` + proptests (50-case each) | P3 | `domain/tests/template_{c,d}_firing_props.rs` |
| C11 | 4 new fire-listeners wired in `AppState::new` | TemplateCFireListener, TemplateDFireListener, MemoryExtractionListener (stub), AgentCatalogListener (stub) | P3 | `server/src/state.rs::tests::handler_count_at_m5` asserts 5 listeners |
| C12 | Page 14 vertical — session launch + preview + terminate + tools | Business logic + handlers + acceptance; closes C-M5-2 (UsesModel writer), C-M5-3 (Session persistence), C-M5-4 (AgentTool resolver), C-M5-5 (ModelConfig change + real 409), C-M5-6 (Shape B materialise) | P4 | `acceptance_sessions_launch.rs` + `acceptance_sessions_preview.rs` + `acceptance_sessions_terminate.rs` + extended `acceptance_projects_create.rs` + extended `acceptance_agents_profile.rs` |
| C13 | Page 12 vertical — authority template adoption | approve / deny / adopt / revoke-cascade | P5 | `acceptance_authority_templates.rs` (25+ tests) |
| C14 | Page 13 vertical — system agents config | tune / add / disable / archive; `SystemAgentRuntimeStatus` live feed | P6 | `acceptance_system_agents.rs` (25+ tests) |
| C15 | `phi session` CLI | `launch` (tail default + `--detach`), `show`, `terminate`, `list` | P7 | `cli/tests/session_snapshot.rs` + completion regression |
| C16 | Web UI pages 12/13/14 | Next.js routes with phi-core-strip on every response | P5/P6/P4 (per-phase) | `modules/web/__tests__/m5_*.test.tsx` + Playwright e2e |
| C17 | s02 memory-extraction listener | Subscribes to `SessionEnded`, runs supervisor `agent_loop`, emits `MemoryExtracted` audit per candidate with full tag metadata (agent/project/org/#public tags serialised as a structured field on the audit event so M6 can materialise Memory nodes from the audit stream) | P8 | `acceptance_system_flows_s02.rs` |
| C18 | s03 agent-catalog listener | Subscribes to 8 events, upserts `AgentCatalogEntry`; `SystemAgentRuntimeStatus.queue_depth` updates | P8 | `acceptance_system_flows_s03.rs` |
| C19 | s05 Template C + D grant-fire extension | Listeners issue grants on matching edge events; M4's TemplateAFireListener is the shape template | P8 | Template C + D acceptance coverage in `acceptance_system_flows_s05.rs` |
| C20 | Cross-page acceptance `acceptance_m5.rs` + full e2e fixture | Bootstrap → org → agent → project → `phi session launch` → session ends → memory extracted (audit) + catalog updated + UsesModel edge exists + RUNS_IN edge exists + page 11's "Recent sessions" panel shows the session | P9 | `server/tests/acceptance_m5.rs` + `server/tests/e2e_first_session.rs` |
| C21 | CI extensions | `rust.yml` acceptance job extended with 10 new `--test` binaries; `phi-core-leverage-targets` new job asserts exact import counts | P0 (grep) + P9 (CI) | PR green |
| C22 | Ops docs + troubleshooting + runbook M5 section | 3 per-page ops runbooks + session runbook + `m5/user-guide/troubleshooting.md` (full stable-code table) + `docs/ops/runbook.md §M5` appended | P4–P9 | `check-doc-links.sh` + `check-ops-doc-headers.sh` |
| C23 | phi-core reuse map (M5) populated end-to-end | `m5/architecture/phi-core-reuse-map.md` with per-page tables + positive close-audit record | P9 | doc-link check |
| C24 | ADRs 0029 + 0030 + 0031 | 0029 Session persistence + SessionRecorder wrap (D2); 0030 Template-node uniqueness (D1); 0031 Session cancellation + concurrency (D3). All Proposed at P0, Accepted at P1/P3/P4 closes. | P0 (draft) + P1/P3/P4 (flip) | doc-link check |
| C25 | Independent 3-agent re-audit at P9 | Rust correctness + docs fidelity + vertical-integrity (M4 precedent); target ≥99% composite | P9 | 3 agent reports captured; LOW findings remediated |
| C26 | **C-M6-1 pinned in base build plan** (new — from D6 user clarification) | Memory node tier + interface contract any memory system must implement + ownership-via-multi-tag (agent / group / project / org; one memory can carry multiple tags) + permission-over-time retrieval (agents retrieve only what they have current grants for) + default implementation shipping alongside the contract | P0 | Amendment landed in [`/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md) §M6 as new subsection `#### Carryovers from M5`; verified by `grep -n 'Carryovers from M5' <file>` returning 1 hit; doc-link check green. |
| C27 | Per-phase **4-aspect** confidence % reported at close (upgraded from M4's 3-aspect) | Code correctness + docs accuracy + phi-core leverage + **archive-plan compliance** (NEW — walks the archived plan deliverable-by-deliverable, marks ✅/⚠/✗, blocks close on any ✗) + composite before next phase opens | P0–P9 | Explicit numbers + compliance-walk checkmarks pinned in every phase close audit report |

Target: **27 commitments closed** at P9.

---

## Part 3 — Decisions made up-front  `[STATUS: ⏳ pending]`

All 6 decisions resolved at planning close (4 user-confirmed, 2 defaults with open-question fallback).

| # | Decision | Resolution | Rationale |
|---|---|---|---|
| **D1** | Template node uniqueness (C-M5-1) | ✅ **Option (b) — one shared row per kind with UNIQUE(kind)** | User-confirmed. Cleanest semantic: templates are platform-level patterns, adoptions are per-org acts carried on `AuthRequest.provenance_template`. Migration 0005 DROPs `template_name` index + DEFINEs `template_kind` UNIQUE. ADR-0030. |
| **D2** | Session recorder wiring (C-M5-3) | ✅ **Option (a) — wrap `phi_core::SessionRecorder`** | User-confirmed. Preserves M3 wrap pattern. `BabyPhiSessionRecorder` composes phi-core's recorder + adds `on_event_persisted` hook that writes to SurrealDB. Double-materialisation avoided by making phi-core's recorder the source, baby-phi the sink. ADR-0029. |
| **D3** | Session concurrency ceiling | ⏳ **Default: `[session] max_concurrent = 16` in `config/default.toml`** | Assumed default; flag for user confirmation at P4 opening. Ceiling exceeded → 503 `SESSION_WORKER_SATURATED` (distinct from W2's per-agent `PARALLELIZE_CAP_REACHED`). Redis-backed shared registry deferred to M7b. ADR-0031. |
| **D4** | `phi session launch` CLI UX | ✅ **Tail events live + `--detach` flag** | User-confirmed. Default streams turns + tool invocations until session ends or Ctrl-C (→ sends terminate). `--detach` returns session_id + operator polls via `phi session show <id>`. Matches admin/14's "hello world validator" intent. |
| **D5** | Permission Check preview location (R-ADMIN-14-R3) | ⏳ **Default: server-side** (`POST /orgs/:id/projects/:pid/sessions/preview`) | Assumed default; flag for user confirmation at P4 opening. Keeps M1's algorithm single-sourced; trace is reusable by CLI + web. |
| **D6** | Memory + AgentCatalogEntry scope | ✅ **Audit-only for Memory at M5; full `AgentCatalogEntry` at M5. Memory contract + node + default impl shift to M6** as new C-M6-1 with three sub-requirements: (i) well-defined **memory interface contract** any integrator (including baby-phi's default implementation) MUST implement; (ii) **ownership via multi-tag** (agent / group / project / org) with a single session/memory carrying multiple tags simultaneously; (iii) **permission-over-time retrieval** — agents retrieve only memory they have current grants for at retrieval time (grants revoked after extraction forfeit read access) | User-confirmed with additional scope clarification. M5 s02 fires on SessionEnded, emits `MemoryExtracted` audit with structured tag list + session transcript reference, but persists no Memory node. M6's C-M6-1 picks up the node + contract. |
| **D7** | Phase breakdown | 10 phases (P0–P9) | P0 planning + ADRs + CI grep. P1 migration + node wraps. P2 repo expansion. P3 event bus + recorder + Template C/D pure-fns. P4 page 14 session launch (phi-core-heaviest + 5 carryover closes). P5 page 12 authority templates. P6 page 13 system agents. P7 CLI + web polish. P8 s02 + s03 + s05 listener bodies. P9 seal + 3-agent re-audit. |

All 6 decisions user-confirmed or assumed-default-pending-P4-confirm; no pre-P0 blockers remain.

---

## Part 4 — Implementation phases  `[STATUS: ⏳ pending]`

Ten phases (P0 → P9). Each phase has **six subsections**: **Goals · Deliverables · phi-core leverage · Tests added · Archive-plan compliance check · Confidence check**. Every phase closes with `cargo fmt/clippy/test + npm test/typecheck/lint/build + check-doc-links + check-ops-doc-headers + check-phi-core-reuse` all green, commitment-ledger row(s) ticked, AND **reported confidence % before the next phase opens**.

### Per-phase close discipline — 4-aspect (not 3)

Every phase close reports **four** aspects, not three (a refinement over M4's 3-aspect model):

1. **Code correctness** — tests + clippy + fmt + type checks green.
2. **Docs accuracy** — per-page architecture + ops docs reflect the shipped code; `Last verified` headers refreshed; ADR statuses correct.
3. **phi-core leverage** — `check-phi-core-reuse.sh` green; positive greps match the phase's prediction in Part 1.5; compile-time coercion witnesses pass.
4. **Archive-plan compliance** — the *new* aspect. The phase's **Archive-plan compliance check** subsection walks the plan archive at `baby-phi/docs/specs/plan/build/<8hex>-m5-templates-system-agents-sessions.md` (the copy landed at P0) **deliverable-by-deliverable** and marks each with:
   - **✅** — shipped as specified, file + test exists.
   - **⚠ drift** — shipped but differs from the plan (shape, name, scope). Drift must be reported with a one-line explanation (why the deviation was needed) + a note whether the plan file itself should be edited to reflect the ground truth.
   - **✗ missing** — not shipped in this phase; flag whether it was pushed to a later phase, deferred entirely, or missed.

**How the compliance walk works**: at every phase close, reopen the archived plan + read only the current phase's §Deliverables list. For each bullet, cross-reference against what actually landed (check the file exists, check the test named in §Tests added exists, check the referenced function/struct exists via `grep`). Any bullet that doesn't match the archive ships a drift report + decision (either fix the code or amend the archive — never let them diverge silently).

**Composite confidence** at phase close = `min(code, docs, phi-core, archive-plan)` or a weighted average per M4 precedent. Either way, archive-plan compliance is a hard floor: a phase that achieves 99% on the first three aspects but 80% on archive-plan compliance closes at **80%**, not 95%, and the drift items must be remediated before the next phase opens.

**Why this matters**: M4 shipped clean but the plan itself silently drifted once (P5 ThinkingLevel 5-variant vs 4-variant — fixed mid-milestone). At M5 the load-bearing ADRs + 5 carryover closes + phi-core-heaviest surface mean any silent drift compounds. The compliance walk forces the plan and the code to stay in sync, OR forces an explicit decision to edit one or the other.

### P0 — M4 post-flight + ADRs 0029/0030/0031 + base-plan M6 amendment + CI grep + docs seed (~4–6 hours)

#### Goals
Archive this plan. Verify M4→M5 boundary. Draft 3 ADRs. Amend base build plan with C-M6-1 (Memory contract). Extend phi-core-reuse deny-list. Seed M5 docs tree.

#### Deliverables
1. Archive plan to `baby-phi/docs/specs/plan/build/<8hex>-m5-templates-system-agents-sessions.md` (8-hex token via `openssl rand -hex 4`).
2. `m5/architecture/m4-postflight-delta.md` — 10-item audit: (a) confirm 805 Rust / 68 web pass; (b) C-M5-1 Template UNIQUE still blocking; (c) C-M5-2 `uses_model` still mis-typed; (d) C-M5-3 Session scaffolds still id-only; (e) C-M5-4 no AgentTool resolver; (f) C-M5-5 `count_active_sessions` still `Ok(0)` + `model_config_id` field absent; (g) C-M5-6 `approve_pending_shape_b` Approved branch still `project: None`; (h) `_keep_materialise_live` dead-code still present; (i) ADRs 0024/0025/0027/0028 Accepted; (j) no unexpected drift since M4 close.
3. **ADR-0029** — Session persistence + SessionRecorder wrap (D2). Status Proposed → Accepted at P3 close.
4. **ADR-0030** — Template-node uniqueness (D1). Status Proposed → Accepted at P1 close.
5. **ADR-0031** — Session cancellation + concurrency bounds (D3). Status Proposed → Accepted at P4 close.
6. **Base plan amendment — exact file to edit: [`/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md).** Inside its §M6 subsection (header `### M6 — Agent self-service surfaces (≈2 weeks)`), add a new subsection `#### Carryovers from M5 — must-pick-up at M6 detailed planning` immediately after the §M6 header. Mirror the shape of the existing M3→M5 and M4→M5 carryover subsections in that file (lines 268–291 for reference). Content:
   - **C-M6-1** — Memory node tier + interface contract + ownership-by-multi-tag + permission-over-time retrieval. Full text per D6 resolution (Part 3). Files affected at M6: `modules/crates/domain/src/model/nodes.rs::Memory`, new `modules/crates/domain/src/memory_contract.rs` (trait + default impl), `modules/crates/store/migrations/0006_*.surql` (memory table + `HAS_MEMORY` edges + tag-index table), `modules/crates/server/src/platform/memory/` new dir, `modules/crates/cli/src/commands/memory.rs` (`phi memory {list, show, tag, retrieve}`). P0 close runs `grep -n 'Carryovers from M5' docs/specs/plan/build/36d0c6c5-build-plan-v01.md` → expects 1 hit to confirm the amendment landed.
7. **CI grep extension**: `scripts/check-phi-core-reuse.sh` deny-list adds `struct Session `, `struct LoopRecord `, `struct Turn `, `struct AgentTool `, `struct AgentEvent `, `struct ModelConfig ` (outside `domain/src/model/nodes.rs` wrap layer + phi-core itself).
8. **Spec-drift registration**: `scripts/check-spec-drift.sh` recognises R-ADMIN-12-* / R-ADMIN-13-* / R-ADMIN-14-* / R-SYS-s02-* / R-SYS-s03-* / R-SYS-s05-*.
9. M5 docs-tree seed at `docs/specs/v0/implementation/m5/{README, architecture, user-guide, operations, decisions}/` with stub files carrying `<!-- Last verified: YYYY-MM-DD -->` headers.

#### phi-core leverage
None — audit + planning phase. CI grep extension is the structural enforcement for P1+.

#### Tests added
0 (documentation + CI phase).

#### Archive-plan compliance check
P0 is the phase that **archives** this plan, so compliance is trivially self-reference. Walk: (a) the 8-hex filename exists in `baby-phi/docs/specs/plan/build/` and matches this file byte-for-byte; (b) ADRs 0029/0030/0031 exist as stub files with `Status: Proposed`; (c) C-M6-1 amendment is present in the base plan (grep confirms); (d) CI grep deny-list extended; (e) spec-drift registry extended; (f) docs tree seeded with all files named in Part 6. Mark each ✅/⚠/✗ + report any drift before P1 opens.

#### Confidence check
**Target: N/A** (audit phase) — but report a 4-aspect check: code N/A · docs 100% (self-archive) · phi-core leverage N/A · archive-plan compliance 100% or flag drift. If >1 item is `stale` in the post-flight delta OR any archive-plan compliance bullet fails, open P0.5 remediation before P1.

---

### P1 — Migration 0005, node wraps, web primitives (~3 days)

#### Goals
Schema + domain types for Session/LoopRecord/Turn + sidecars + governance fields all exist before P2 opens.

#### Deliverables
1. **Migration 0005** `store/migrations/0005_sessions_templates_system_agents.surql`:
   - DROP INDEX `template_name`; DEFINE INDEX `template_kind` ON `template` FIELDS `kind` UNIQUE (D1).
   - REMOVE TABLE `uses_model`; DEFINE TABLE `uses_model` TYPE RELATION FROM `agent` TO `model_runtime` (C-M5-2).
   - DEFINE 3 new session tables: `session`, `loop_record`, `turn` with `FLEXIBLE TYPE object` for the phi-core `inner` field + explicit governance columns (owning_org, owning_project, started_by, governance_state, started_at, ended_at, session_id link on loop_record + turn).
   - DEFINE TABLE `runs_session` TYPE RELATION FROM `session` TO `project`.
   - DEFINE TABLE `shape_b_pending_projects` with UNIQUE `auth_request_id` + `payload FLEXIBLE TYPE object` (C-M5-6).
   - ALTER `agent_profile` ADD FIELD `model_config_id` OPTIONAL TYPE option<string> (C-M5-5).
   - DEFINE TABLE `agent_catalog_entry` (s03 cache).
   - DEFINE TABLE `system_agent_runtime_status` (page 13 queue-depth + last-fired).
2. **`domain/src/model/nodes.rs`** — replace the id-only Session/Loop/Turn scaffolds with:
   ```rust
   pub struct Session {
       pub id: SessionId,
       #[serde(flatten)]
       pub inner: phi_core::session::model::Session,
       pub owning_org: OrgId,
       pub owning_project: ProjectId,
       pub started_by: AgentId,
       pub governance_state: SessionGovernanceState, // Running | Completed | Aborted | FailedLaunch
       pub started_at: DateTime<Utc>,
       pub ended_at: Option<DateTime<Utc>>,
       pub tokens_spent: u64,
   }
   pub struct LoopRecordNode {
       pub id: LoopId,
       #[serde(flatten)]
       pub inner: phi_core::session::model::LoopRecord,
       pub session_id: SessionId,
       pub loop_index: u32,
   }
   pub struct TurnNode {
       pub id: TurnNodeId,
       #[serde(flatten)]
       pub inner: phi_core::session::model::Turn,
       pub loop_id: LoopId,
       pub turn_index: u32,
   }
   ```
   Plus `pub enum SessionGovernanceState { Running, Completed, Aborted, FailedLaunch }`.
3. **`domain/src/model/composites_m5.rs`** — `ShapeBPendingProject`, `AgentCatalogEntry`, `SystemAgentRuntimeStatus`.
4. **`config/default.toml`** — add `[session] max_concurrent = 16` (D3) + `server::config::ServerConfig::session: SessionConfig { max_concurrent: u32 }`.
5. **Web wizard primitives extensions**:
   - `SessionEventStreamRenderer.tsx` — live `AgentEvent` tail renderer for page 14.
   - `PermissionCheckPreviewPanel.tsx` — renders the server-side 6-step trace for page 14.
   - `TemplateAdoptionTable.tsx` — page 12's 3-column (Pending/Active/Revoked) + Available CTA.
   - `SystemAgentStatusCard.tsx` — page 13's per-agent live status tile.
6. **CLI scaffolding**: `cli/src/commands/session.rs` with `launch`, `show`, `terminate`, `list` subcommand stubs (body returns `EXIT_NOT_IMPLEMENTED` pending P4/P7 wiring). Registered in `cli/src/main.rs`.
7. **Docs**: `m5/architecture/session-persistence.md` (the 3-wrap pattern), `m5/architecture/authority-templates.md`, `m5/architecture/session-launch.md`, `m5/architecture/shape-b-materialisation.md` (C-M5-6 pre/post).

#### phi-core leverage (Q1/Q2/Q3)
- **Q1 direct imports — 3 NEW at P1**:
  - `use phi_core::session::model::Session as PhiCoreSession;` — in `domain/src/model/nodes.rs`
  - `use phi_core::session::model::LoopRecord;` — same file
  - `use phi_core::session::model::Turn;` — same file
- **Q2 transitive**: every `Session` row on the wire carries `inner: phi_core::Session` via serde flatten. Identical pattern to M3 `OrganizationDefaultsSnapshot`.
- **Q3 rejections**: `phi_core::session::save_session` / `load_session` (JSON file helpers — baby-phi uses SurrealDB). `phi_core::SessionRecorderConfig::include_streaming_events` default stays `false`.

Positive close-audit greps:
- `grep -En '^use phi_core::session::model' domain/src/model/nodes.rs` → **3 lines** (exactly).
- `grep -En 'inner: phi_core::session::model' domain/src/model/nodes.rs` → **3 lines**.
- `grep -En 'struct Session ' modules/crates/ --include '*.rs'` → **exactly 1 line** (the wrap; phi-core's own `Session` is inside `phi-core/`).
- **Compile-time coercion tests** in `nodes.rs::tests`: 3 witness fns (`_is_phi_core_session`, `_is_phi_core_loop_record`, `_is_phi_core_turn`) applied to `Session.inner` etc.

#### Tests added (~15)
- `domain/tests/m5_wrap_roundtrip.rs` — serde round-trip for Session / LoopRecordNode / TurnNode / ShapeBPendingProject / AgentCatalogEntry / SystemAgentRuntimeStatus (6 tests).
- `store/tests/migrations_0005_test.rs` — apply / noop / broken (3 tests).
- `domain/tests/agent_profile_model_config_id_serde.rs` — round-trip + `#[serde(default)]` back-compat (2 tests).
- `modules/web/__tests__/m5_primitives.test.tsx` — each of the 4 new primitives renders (4 tests).

#### Archive-plan compliance check
Walk the P1 §Deliverables list in the archived plan deliverable-by-deliverable:
- ✅/⚠/✗ Migration 0005 exists at `modules/crates/store/migrations/0005_sessions_templates_system_agents.surql` with 8 schema changes as listed.
- ✅/⚠/✗ `Session` / `LoopRecordNode` / `TurnNode` wraps exist in `nodes.rs`; field lists match the plan's code block; `#[serde(flatten)]` present; `SessionGovernanceState` enum present.
- ✅/⚠/✗ `composites_m5.rs` carries `ShapeBPendingProject` + `AgentCatalogEntry` + `SystemAgentRuntimeStatus`.
- ✅/⚠/✗ `config/default.toml` has `[session] max_concurrent = 16`; `ServerConfig::session` field present.
- ✅/⚠/✗ 4 web primitives exist at the named filepaths.
- ✅/⚠/✗ CLI scaffolds for `phi session {launch, show, terminate, list}` present + registered in `cli/src/main.rs`.
- ✅/⚠/✗ 4 new architecture docs seeded with `Last verified` header.
- ✅/⚠/✗ ADR-0030 flipped from Proposed → Accepted.
- ✅/⚠/✗ 3 compile-time coercion witnesses pass.
Any ⚠ drift reported with a one-line "why" + decision: fix code, edit plan archive, or both. Any ✗ missing blocks P1 close.

#### Confidence check
**Target: ≥ 97%.** Close criteria (4-aspect):
- Code: `cargo test --workspace` green; clippy `-Dwarnings` green; `npm run test/lint/typecheck/build` green.
- Docs: ADRs 0029/0030/0031 status = Proposed; 0030 flips to Accepted at P1 close (schema shipped).
- phi-core leverage: exactly 3 `use phi_core::session::model::*` imports in `nodes.rs`, 0 elsewhere; `check-phi-core-reuse.sh` green.
- Archive-plan compliance: every P1 §Deliverables bullet ✅ (or ⚠ with approved drift note). No ✗ open at close.
- Report composite % = min(code, docs, phi-core, archive-plan) or weighted average; before P2 opens.

---

### P2 — Repository expansion (~3 days)

#### Goals
All 14 new repo methods + stub flip exist + in-memory + SurrealDB parity.

#### Deliverables
1. Repo method additions in `domain/src/repository.rs`:
   - **Session tier**: `persist_session`, `append_loop_record`, `append_turn`, `append_agent_event`, `fetch_session(id) -> Option<SessionDetail>` where `SessionDetail { session, loops, turns_by_loop }`, `list_sessions_in_project`, `list_active_sessions_for_agent`, `mark_session_ended(id, ended_at, state)`, `terminate_session(id, reason, terminated_by, at)` — transactional marking.
   - **Shape B sidecar**: `persist_shape_b_pending`, `fetch_shape_b_pending`, `delete_shape_b_pending`.
   - **Agent catalog**: `upsert_agent_catalog_entry`, `list_agent_catalog_entries_in_org`, `get_agent_catalog_entry`.
   - **System-agent status**: `upsert_system_agent_runtime_status`, `fetch_system_agent_runtime_status_for_org`.
2. **Flip `count_active_sessions_for_agent`** from `Ok(0)` stub to real SurrealQL: `SELECT count() FROM session WHERE started_by = $agent AND governance_state IN ['running']`.
3. **`list_authority_templates_for_org(org)` + `count_grants_fired_by_adoption(ar_id)` + `list_revoked_adoptions_for_org(org)`** — page 12 reads.

#### phi-core leverage
- **Q1 direct imports**: 0 at repo layer. Types flow through `SessionDetail` aggregate but methods don't import phi-core directly.
- **Q2 transitive**: `SessionDetail.session.inner` + every `LoopRecordNode.inner` + every `TurnNode.inner` transit phi-core types via serde.
- **Q3 rejections**: no per-turn phi-core primitive imports in the repo layer (all phi-core types stay wrapped inside the aggregate).

Positive close-audit grep:
- `grep -En '^use phi_core::' modules/crates/domain/src/repository.rs` → **0 lines** (everything flows via the wrap types already imported at `nodes.rs`).
- `grep -n 'Ok(0)' modules/crates/domain/src/repository.rs` near `count_active_sessions_for_agent` → **0**.

#### Tests added (~28)
- `store/tests/session_repo_parity.rs` — 9 methods × 2 impls (in-memory + SurrealDB) = 18 tests.
- `store/tests/shape_b_sidecar_test.rs` — persist / fetch / delete / fetch-after-delete (4 tests).
- `store/tests/agent_catalog_repo_test.rs` — upsert idempotency + list filter (3 tests).
- `store/tests/system_agent_runtime_status_test.rs` — upsert round-trip (2 tests).
- `store/tests/active_sessions_count_test.rs` — count with 0/1/N running sessions; ignores terminated (4 tests).
- Template read methods tests (3).

#### Archive-plan compliance check
Walk P2 §Deliverables:
- ✅/⚠/✗ 14 new session-surface repo methods all present in trait + both impls (in-memory + SurrealDB).
- ✅/⚠/✗ `count_active_sessions_for_agent` no longer returns `Ok(0)`; real SurrealQL query in place.
- ✅/⚠/✗ 3 Shape B sidecar methods + 3 agent-catalog methods + 2 system-agent-status methods + 3 template read methods all shipped.
- ✅/⚠/✗ Parity test file `session_repo_parity.rs` runs both impls against the same invariants.
Drift reported + decision taken before P3 opens.

#### Confidence check
**Target: ≥ 97%.** 4-aspect: code · docs · phi-core leverage (`grep ^use phi_core:: modules/crates/domain/src/repository.rs` → 0 lines) · archive-plan compliance. Report composite % before P3.

---

### P3 — Event bus extensions, recorder wrap, Template C/D pure-fns, listener scaffolds (~3 days)

#### Goals
Reactive plumbing ready. Listener bodies land at P8 but their subscriptions register now.

#### Deliverables
1. **`domain/src/events/mod.rs`** — extend `DomainEvent` enum with 8 new variants:
   - `SessionStarted { session_id, agent_id, project_id, started_at, audit_event_id }`
   - `SessionEnded { session_id, agent_id, project_id, ended_at, duration_ms, turn_count, tokens_spent, audit_event_id }`
   - `SessionAborted { session_id, reason, terminated_by, at, audit_event_id }`
   - `ManagesEdgeCreated { org_id, manager, subordinate, at, audit_event_id }` — Template C trigger.
   - `HasAgentSupervisorEdgeCreated { project_id, supervisor, supervisee, at, audit_event_id }` — Template D trigger.
   - `AgentCreated { agent_id, owning_org, kind, role, at, audit_event_id }` — s03 trigger.
   - `AgentArchived { agent_id, at, audit_event_id }` — s03 trigger.
   - `HasProfileEdgeChanged { agent_id, old_profile_id, new_profile_id, at, audit_event_id }` — s03 trigger.
2. **`domain/src/session_recorder.rs`** — `BabyPhiSessionRecorder` (D2 path a):
   ```rust
   pub struct BabyPhiSessionRecorder {
       inner: phi_core::SessionRecorder,
       repo: Arc<dyn Repository>,
       audit: Arc<dyn AuditEmitter>,
       event_bus: Arc<dyn EventBus>,
   }
   impl BabyPhiSessionRecorder {
       pub async fn on_phi_core_event(&mut self, event: phi_core::AgentEvent) { ... }
   }
   ```
   Accepts each `phi_core::AgentEvent`, pipes to phi-core's `SessionRecorder::on_event`, then reads phi-core's materialised Session/LoopRecord/Turn via accessor + writes to SurrealDB + emits governance events (`SessionStarted` on first `AgentStart`, `SessionEnded` on `AgentEnd` with non-null rejection or final turn). ADR-0029 flip to Accepted at P3 close.
3. **Template C + D pure-fns**:
   - `domain/src/templates/c.rs::fire_grant_on_manages_edge(manager, subordinate, at) -> Grant` — issues `[read, inspect]` on `agent:<subordinate>` for the manager.
   - `domain/src/templates/d.rs::fire_grant_on_has_agent_supervisor(supervisor, supervisee, at) -> Grant` — similar, project-scoped.
4. **4 new listeners in `domain/src/events/listeners.rs`**:
   - `TemplateCFireListener` subscribes to `ManagesEdgeCreated`, calls `fire_grant_on_manages_edge`, persists, emits audit.
   - `TemplateDFireListener` subscribes to `HasAgentSupervisorEdgeCreated`, calls Template D pure-fn.
   - `MemoryExtractionListener` — **stub** (no-op body at P3; body in P8). Subscribes to `SessionEnded`.
   - `AgentCatalogListener` — **stub** (no-op body at P3; body in P8). Subscribes to 8 variants.
5. **`AppState::new`** wires all 5 listeners (M4's Template A + the 4 new). `server/src/state.rs::tests::handler_count_is_five_at_m5` asserts the count.
6. **Docs**: `m5/architecture/event-bus-m5-extensions.md` (new variants + per-variant emit callsites — eventual-consistency contract).

#### phi-core leverage
- **Q1 direct imports — 2 NEW at P3**:
  - `use phi_core::SessionRecorder;` — in `domain/src/session_recorder.rs`.
  - `use phi_core::types::event::AgentEvent;` — same file.
- **Q2 transitive**: `BabyPhiSessionRecorder.on_phi_core_event` consumes `phi_core::AgentEvent` by value + stores materialised phi-core types in SurrealDB via the existing wraps.
- **Q3 rejections**: no `phi_core::save_session` / `load_session` — baby-phi persists via repo.

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/domain/src/session_recorder.rs` → **2 lines**.
- `grep -En 'impl.*SessionRecorder' modules/crates/ --include '*.rs'` → **1 line** (wrap only; no re-impl of phi-core's trait).
- Handler count test in state.rs tests passes.

#### Tests added (~22)
- `domain/src/events/mod.rs::tests` — 8 new variants × (serde round-trip + event_type string) = 16 tests.
- `domain/tests/session_recorder_wrap_test.rs` — 3-event trace (AgentStart → TurnEnd → AgentEnd) produces correct Session + 1 LoopRecord + 1 Turn; emits `SessionStarted` + `SessionEnded` governance events (3 tests).
- `domain/tests/template_c_firing_props.rs` + `template_d_firing_props.rs` — 50-case proptests each on grant shape invariants (holder, resource, actions, provenance).
- `server/src/state.rs::tests::handler_count_is_five_at_m5` (1 test).

#### Archive-plan compliance check
Walk P3 §Deliverables:
- ✅/⚠/✗ 8 new `DomainEvent` variants present with exact field lists; serde round-trip tests exist.
- ✅/⚠/✗ `BabyPhiSessionRecorder` wrap exists at `domain/src/session_recorder.rs` with the signature shown in the plan code block; composes phi-core's SessionRecorder.
- ✅/⚠/✗ `templates/c.rs` + `templates/d.rs` pure-fns with proptests.
- ✅/⚠/✗ 4 new listeners registered in `AppState::new`; state.rs handler-count test passes at 5.
- ✅/⚠/✗ `m5/architecture/event-bus-m5-extensions.md` written.
- ✅/⚠/✗ ADR-0029 flipped Proposed → Accepted.
Drift reported + decision taken before P4 opens.

#### Confidence check
**Target: ≥ 97%.** 4-aspect: code · docs (ADR-0029 Accepted) · phi-core leverage (2 new imports in session_recorder.rs; 0 re-impls of phi-core's trait) · archive-plan compliance. Report composite % before P4.

---

### P4 — Page 14 first session launch + 5 carryover closes (~5 days) **← M5's phi-core-heaviest + biggest phase**

#### Goals
Admin page 14 vertical (R1-R4, W1-W4, N1-N4) ships. Five M5 carryovers close here: **C-M5-2** (UsesModel writer) + **C-M5-3** (Session persistence via the full launch → run → persist chain) + **C-M5-4** (AgentTool resolver + `GET /sessions/:id/tools`) + **C-M5-5** (ModelConfig change + real 409) + **C-M5-6** (Shape B materialise).

#### Deliverables
1. **Business logic** — `server/src/platform/sessions/` directory with:
   - `mod.rs` — `SessionError` enum (21+ variants: PARALLELIZE_CAP_REACHED, SESSION_WORKER_SATURATED, AGENT_NOT_FOUND, PROJECT_NOT_FOUND, PERMISSION_CHECK_FAILED_AT_STEP_N, MODEL_RUNTIME_UNRESOLVED, SESSION_NOT_FOUND, SESSION_ALREADY_TERMINAL, ...).
   - `launch.rs` — `launch_session(repo, audit, event_bus, registry, input) -> Result<LaunchReceipt, SessionError>`. Flow:
     1. Validate agent + project + membership.
     2. Resolve `ModelRuntime` via `agent.profile.model_config_id` (C-M5-5 gate).
     3. Run Permission Check steps 0–6.
     4. Gate on `count_active_sessions_for_agent < profile.blueprint.parallelize` (W2).
     5. Gate on session-registry size < `max_concurrent` (D3).
     6. Compound tx: persist `Session` row + first `LoopRecordNode` + `RELATE session -> runs_session -> project` (C-M5-3) + `RELATE agent -> uses_model -> model_runtime` (C-M5-2 **close**) + emit `platform.session.started` audit + emit `DomainEvent::SessionStarted` after commit.
     7. Spawn `tokio::task` running `phi_core::agent_loop(prompts, ctx, cfg, tx, cancel_token)`. The event channel feeds `BabyPhiSessionRecorder`.
     8. Register `(session_id, cancel_token)` in `AppState::session_registry` (DashMap).
     9. Return `LaunchReceipt { session_id, first_loop_id, permission_check_trace }`.
   - `preview.rs` — `preview_session(repo, input) -> Result<PermissionCheckTrace, SessionError>` (D5 path a).
   - `terminate.rs` — `terminate_session(repo, audit, event_bus, registry, session_id, reason, actor) -> Result<TerminateReceipt, SessionError>`. Looks up cancellation token, calls `cancel_token.cancel()`, marks Session Aborted, emits audit + `SessionAborted`.
   - `show.rs` — `show_session(repo, session_id, viewer) -> Result<SessionDetail, SessionError>` with access gate (session.started_by OR member of owning_org).
   - `list.rs` — `list_sessions_in_project(repo, project_id, viewer)`.
   - `tools.rs` — `resolve_agent_tools(repo, agent_id) -> Vec<Box<dyn AgentTool>>` + `GET /sessions/:id/tools` handler (C-M5-4 **close**).
2. **Handlers** — `server/src/handlers/sessions.rs` with 6 routes:
   - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions` (launch).
   - `POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview` (preview).
   - `GET  /api/v0/sessions/:id` (show).
   - `POST /api/v0/sessions/:id/terminate` (terminate).
   - `GET  /api/v0/projects/:project_id/sessions` (list).
   - `GET  /api/v0/sessions/:id/tools` (tools).
3. **`AppState`** — add `session_registry: Arc<DashMap<SessionId, CancellationToken>>`.
4. **Router** — register the 6 routes.
5. **C-M5-5 flip** (`agents/update.rs:167-176`) — un-defer `model_config_id` change arm:
   - Validate `model_config_id` references an active `model_runtime` row in the agent's owning org catalogue.
   - Check `count_active_sessions_for_agent(agent_id) > 0` → return `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`.
   - Persist.
6. **C-M5-6 flip** (`projects/create.rs:634-662`) — `approve_pending_shape_b` Approved branch:
   - Read `shape_b_pending_projects` sidecar by `ar_id`.
   - Call `materialise_project` with reconstructed input.
   - Delete sidecar row inside same tx.
   - Return `project: Some(pid)`.
   - Remove `_keep_materialise_live` dead-code marker + its `NodeId` keep-alive.
   - Update the Shape B submit path to ALSO persist the sidecar inside its own compound tx.
7. **Ops doc**: `m5/operations/session-launch-operations.md`.
8. **Architecture doc**: `m5/architecture/session-launch.md` filled out.

#### phi-core leverage

- **Q1 direct imports — 4 NEW at P4** (all at `server/src/platform/sessions/` + 1 at `agents/update.rs`):
  - `use phi_core::{agent_loop, agent_loop_continue};` — in `launch.rs`.
  - `use phi_core::types::event::AgentEvent;` — in `launch.rs` + recorder usage.
  - `use phi_core::types::tool::AgentTool;` — in `tools.rs`.
  - `use phi_core::provider::model::ModelConfig;` — in `launch.rs` (runtime resolution) + `agents/update.rs` (C-M5-5 catalogue lookup).
- **Q2 transitive**:
  - `LaunchReceipt.permission_check_trace: PermissionCheckTrace` is phi-governance (M1 type, no phi-core).
  - `GET /sessions/:id` response carries full `SessionDetail` with wrapped phi-core types via serde flatten (acceptance test pins the depth).
  - `GET /projects/:pid/sessions` response uses `SessionHeader` (stripped) — pinned by schema-snapshot test.
  - `GET /sessions/:id/tools` returns `Vec<ToolSummary { name, label, description, parameters_schema }>` — phi-core `AgentTool` trait methods flattened to JSON. This IS phi-core transit; documented in the ops doc.
- **Q3 rejections**:
  - `phi_core::BasicAgent` explicitly NOT used at launch time (ADR-0029).
  - `phi_core::agents::Agent` trait not re-implemented — baby-phi's `domain::model::nodes::Agent` is a governance record.
  - `phi_core::session::save_session` / `load_session` — baby-phi persists via `BabyPhiSessionRecorder`.
  - `phi_core::context::ContextConfig` per-agent override — stays org-default per ADR-0023 (unchanged from M4).

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/server/src/platform/sessions/` → **≥6 lines across launch.rs + tools.rs**.
- `grep -En '^use phi_core::' modules/crates/server/src/platform/agents/update.rs` → **4 unique types (4 pre-M5 + `ModelConfig` NEW)** — check post-C-M5-5 flip.
- `grep -n '_keep_materialise_live' modules/crates/server/src/platform/projects/create.rs` → **0**.
- `grep -n 'Ok(0)' modules/crates/domain/src/repository.rs` near `count_active_sessions_for_agent` → **0** (flipped at P2, confirmed still absent).
- **Schema-snapshot test** on `GET /projects/:pid/sessions` response asserts no `blueprint` / `execution_limits` / `inner` keys at depth (header is stripped).

#### Tests added (~55)
- `server/tests/acceptance_sessions_launch.rs` — 14 scenarios:
  - Happy launch + Permission Check green + 3 expected audit events.
  - `PARALLELIZE_CAP_REACHED` when 2/2 running.
  - `SESSION_WORKER_SATURATED` when registry full.
  - `MODEL_RUNTIME_UNRESOLVED` when agent has no `model_config_id`.
  - `PERMISSION_CHECK_FAILED_AT_STEP_3` with grant-less agent.
  - `UsesModel` edge asserted exists post-launch (C-M5-2 proof).
  - `runs_session` edge exists post-launch.
  - SessionRecorder persists 1 LoopRecord + N Turns after execution.
  - `SessionEnded` governance event fires after `agent_loop` returns.
  - Concurrent launches respect session registry bounds.
  - 403 on non-member viewer.
  - 404 on unknown agent/project.
  - Dashboard's "Recent sessions" panel populates (project detail page retrofit test — the M4 placeholder flips).
  - Session response carries wrapped phi-core types end-to-end.
- `server/tests/acceptance_sessions_preview.rs` — 5 scenarios (steps 0-6 each).
- `server/tests/acceptance_sessions_terminate.rs` — 6 scenarios (happy, idempotency, already-terminal, non-existent, 403, cancel-mid-turn).
- `acceptance_projects_create.rs` Shape B extensions — 3 new scenarios: `shape_b_both_approve_materialises_project`, `shape_b_sidecar_persisted_at_submit`, `shape_b_sidecar_deleted_after_materialise` (C-M5-6 proof).
- `acceptance_agents_profile.rs` extensions — 4 new scenarios: `model_config_id_change_happy_path`, `model_config_change_on_active_session_returns_409` (C-M5-5 proof), `model_config_id_validated_against_catalogue`, `model_config_id_serde_back_compat`.
- `GET /sessions/:id/tools` acceptance: agent with 2 tools returns both with full parameters_schema (C-M5-4 proof).
- Compile-time witnesses (3).

#### Archive-plan compliance check
Walk P4 §Deliverables — the phi-core-heaviest + biggest phase (5 carryover closes). Extra care here:
- ✅/⚠/✗ `server/src/platform/sessions/{mod, launch, preview, show, terminate, list, tools}.rs` — all 7 files present.
- ✅/⚠/✗ `SessionError` enum has the ≥21 variants named in the plan.
- ✅/⚠/✗ `launch_session` follows the 9-step flow verbatim (1-9 in the plan); each step can be pointed at a specific line range.
- ✅/⚠/✗ `session_registry` DashMap added to AppState; size-cap enforcement wired.
- ✅/⚠/✗ 6 HTTP routes registered in router.
- ✅/⚠/✗ **C-M5-2 close**: `uses_model` edge written at launch; acceptance test `launch_writes_uses_model_edge` exists + passes.
- ✅/⚠/✗ **C-M5-3 close**: Session + LoopRecord + Turn persisted via `BabyPhiSessionRecorder`; `runs_session` edge written.
- ✅/⚠/✗ **C-M5-4 close**: `GET /sessions/:id/tools` returns resolved tool set.
- ✅/⚠/✗ **C-M5-5 close**: `agents/update.rs:167-176` defer comment removed; 409 path real; `model_config_id` validated against catalogue.
- ✅/⚠/✗ **C-M5-6 close**: `_keep_materialise_live` removed; `approve_pending_shape_b` Approved branch calls `materialise_project`; sidecar read + delete in same tx; shape_b_both_approve acceptance test asserts real `project: Some(pid)`.
- ✅/⚠/✗ 14 + 5 + 6 + 3 + 4 acceptance scenarios all present and green.
- ✅/⚠/✗ Schema-snapshot test on `GET /projects/:pid/sessions` response asserts strip.
- ✅/⚠/✗ ADRs 0029 + 0031 flipped Accepted.
Each ⚠ drift reported with 1-line explanation; each ✗ blocks P4 close.

#### Confidence check
**Target: ≥ 98%.** 4-aspect: code · docs (ADR-0029 + 0031 Accepted; ops + arch docs updated) · phi-core leverage (≥6 imports under `sessions/` + 1 new in `agents/update.rs`; `_keep_materialise_live` + `Ok(0)` both 0 hits) · archive-plan compliance. Report composite % before P5. **This is M5's phi-core-heaviest phase + the biggest single-phase risk surface; expect this to be the tightest composite of the milestone.**

---

### P5 — Page 12 vertical: Authority Template Adoption (~3 days)

#### Goals
Operators can approve / deny / adopt-inline / revoke-cascade authority templates via page 12 (HTTP + CLI + Web).

#### Deliverables
1. Business logic `server/src/platform/templates/{mod, list, approve, deny, adopt, revoke}.rs`:
   - `list_templates_for_org(org)` returns `{pending: [...], active: [...], revoked: [...], available: [...]}`.
   - `approve_adoption_ar(org, ar_id, actor)` — transitions AR per M1 machine; template becomes active.
   - `deny_adoption_ar(org, ar_id, actor, reason)` — terminal Denied.
   - `adopt_template_inline(org, kind, actor)` — creates new AR + auto-approves (actor is sole approver per R-ADMIN-12-W3) + persists template adoption.
   - `revoke_template(org, kind, actor)` — transitions adoption AR → Revoked + walks `DESCENDS_FROM` provenance on every grant, forward-only revokes each, emits `AuthorityTemplateRevoked { grant_count_revoked }`.
2. Handlers:
   - `GET  /api/v0/orgs/:org_id/authority-templates`
   - `POST /api/v0/orgs/:org_id/authority-templates/:kind/{approve, deny, adopt, revoke}`
3. CLI `phi template {list, approve, deny, adopt, revoke} --org-id <id> [--kind <A|B|C|D|E>]`.
4. Web page `modules/web/app/(admin)/organizations/[id]/templates/page.tsx` using `TemplateAdoptionTable.tsx` primitive + `StepShell` wizard for adopt-inline.
5. Ops doc `m5/operations/authority-templates-operations.md`.
6. Architecture doc `m5/architecture/authority-templates.md` filled out.

#### phi-core leverage
- **Q1 direct imports**: 0.
- **Q2 transitive**: 0.
- **Q3 rejections**: explicit — no phi-core equivalent of Template/Grant/AR. `phi_core::AgentEvent` orthogonal (telemetry, not governance trigger).

Positive close-audit grep: `grep -En '^use phi_core::' modules/crates/server/src/platform/templates/` → **0**.

#### Tests added (~27)
- `server/tests/acceptance_authority_templates.rs` — ~15 scenarios covering the 3 requirement scenarios + edge cases (deny, already-terminal approve, non-admin, revoke non-existent, adopt duplicate, adopt with missing prereq warning).
- CLI tests (5).
- Web component tests (7).

#### Archive-plan compliance check
Walk P5 §Deliverables:
- ✅/⚠/✗ `server/src/platform/templates/{mod, list, approve, deny, adopt, revoke}.rs` — 6 files present.
- ✅/⚠/✗ 5 HTTP routes registered (GET list + 4 POSTs).
- ✅/⚠/✗ Revoke-cascade walks `DESCENDS_FROM` provenance; grant_count_revoked surfaces on audit.
- ✅/⚠/✗ `phi template {list, approve, deny, adopt, revoke}` CLI registered.
- ✅/⚠/✗ Web page at `(admin)/organizations/[id]/templates/page.tsx` using `TemplateAdoptionTable.tsx`.
- ✅/⚠/✗ Ops doc + architecture doc filled out.
Drift + decision before P6 opens.

#### Confidence check
**Target: ≥ 98%.** 4-aspect: code · docs · phi-core leverage (0 imports under `templates/`) · archive-plan compliance. Report composite % before P6.

---

### P6 — Page 13 vertical: System Agents Config (~3 days)

#### Goals
Operators tune / add / disable / archive system agents via page 13. Live queue-depth + last-fired-at from `SystemAgentRuntimeStatus`.

#### Deliverables
1. Business logic `server/src/platform/system_agents/{mod, list, tune, add, disable, archive, events_feed}.rs`.
2. Handlers:
   - `GET   /api/v0/orgs/:org_id/system-agents`
   - `PATCH /api/v0/orgs/:org_id/system-agents/:agent_id`
   - `POST  /api/v0/orgs/:org_id/system-agents` (add)
   - `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/disable`
   - `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/archive`
3. **Listener callback extension**: P3's `MemoryExtractionListener` + `AgentCatalogListener` + `TemplateCFireListener` + `TemplateDFireListener` + M4's `TemplateAFireListener` all call `repo.upsert_system_agent_runtime_status(agent_id, queue_depth, last_fired_at, ...)` on each fire. Shared helper in `domain/src/events/listeners.rs`.
4. CLI `phi system-agent {list, tune, add, disable, archive}`.
5. Web page `modules/web/app/(admin)/organizations/[id]/system-agents/page.tsx`.
6. Ops doc `m5/operations/system-agents-operations.md`.

#### phi-core leverage
- **Q1 direct imports**: 1 — `use phi_core::agents::profile::AgentProfile;` in `system_agents/add.rs` (profile_ref validation; re-use of M4 pattern). Matches Part 1.5 prediction.
- Positive close-audit grep: `grep -En '^use phi_core::' modules/crates/server/src/platform/system_agents/` → **1 line**.

#### Tests added (~25)
- `server/tests/acceptance_system_agents.rs` — ~12 scenarios (the three from requirements + edge cases).
- Listener upsert tests (3).
- CLI tests (5).
- Web component tests (5).

#### Archive-plan compliance check
Walk P6 §Deliverables:
- ✅/⚠/✗ `server/src/platform/system_agents/{mod, list, tune, add, disable, archive, events_feed}.rs` — 7 files present.
- ✅/⚠/✗ 5 HTTP routes registered.
- ✅/⚠/✗ All 5 listeners (Template A/C/D + memory-extraction + agent-catalog) call `upsert_system_agent_runtime_status` on each fire.
- ✅/⚠/✗ `phi system-agent {list, tune, add, disable, archive}` CLI.
- ✅/⚠/✗ Web page + ops doc.
Drift + decision before P7 opens.

#### Confidence check
**Target: ≥ 98%.** 4-aspect: code · docs · phi-core leverage (1 import under `system_agents/`) · archive-plan compliance. Report composite % before P7.

---

### P7 — `phi session` CLI + `phi agent update --model-config-id` + web polish (~2 days)

#### Goals
CLI + web polish to round out all M5 surfaces.

#### Deliverables
1. **`cli/src/commands/session.rs`** — full implementation:
   - `phi session launch --agent-id <uuid> --project-id <uuid> --prompt <str> [--detach] [--json]`
     - Default: open SSE to `GET /api/v0/sessions/:id/events` + render turns + tools + audit events live until SIGINT or natural session end. SIGINT → sends `POST /sessions/:id/terminate`.
     - `--detach`: returns `{ session_id, first_loop_id }` immediately.
   - `phi session show --id <uuid> [--json]` — hit GET /sessions/:id, render or JSON.
   - `phi session terminate --id <uuid> [--reason <str>] [--json]`.
   - `phi session list --project-id <uuid> [--active-only] [--json]`.
2. **`cli/src/commands/agent.rs`** — extend `phi agent update` with `--model-config-id <str>` flag (C-M5-5 wire).
3. **`cli/src/commands/template.rs`** (if not in P5) + `system_agent.rs` (if not in P6) — round out CLI.
4. **Completion regression**: `cli/tests/completion_help.rs` asserts `phi session {launch, show, terminate, list}` surface on every shell + `phi agent update --model-config-id` flag present.
5. **Web polish**:
   - Page 14 (`/organizations/[id]/projects/[pid]/sessions/new/`) uses `PermissionCheckPreviewPanel` + `SessionEventStreamRenderer`.
   - Page 11's "Recent sessions" panel (M4 placeholder → real rows from `list_sessions_in_project`).
   - Page 07 dashboard — surface "active sessions" tile if not already present.

#### phi-core leverage
- **Q1 direct imports in CLI**: `phi session launch` may import `use phi_core::types::event::AgentEvent;` to render the live tail payload. Otherwise 0.
- Positive close-audit grep: `grep -En '^use phi_core::' modules/crates/cli/src/commands/session.rs` → **≤1 line** (the `AgentEvent` tail renderer).

#### Tests added (~12)
- `cli/tests/session_snapshot.rs` — 4 CLI UX snapshots.
- Web Playwright e2e — launch wizard happy + detach mode + terminate button (3).
- Completion regression tests (2).
- Agent update extension tests (3).

#### Archive-plan compliance check
Walk P7 §Deliverables:
- ✅/⚠/✗ `cli/src/commands/session.rs` with 4 subcommands; `launch` supports `--detach` and default-tails via SSE.
- ✅/⚠/✗ `phi agent update --model-config-id <str>` flag present and wired.
- ✅/⚠/✗ `phi template` + `phi system-agent` CLI surfaces complete.
- ✅/⚠/✗ Completion regression asserts `phi session {launch, show, terminate, list}` on all 4 shells.
- ✅/⚠/✗ Web page 14 uses `PermissionCheckPreviewPanel` + `SessionEventStreamRenderer`.
- ✅/⚠/✗ Page 11's "Recent sessions" panel flipped from placeholder to real `list_sessions_in_project` rows (verify by reading the page tsx + asserting the placeholder text removed).
- ✅/⚠/✗ **All CLI commands use `phi` prefix, NEVER `baby-phi`** — grep `baby-phi` under `modules/crates/cli/src/` returns 0 lines of user-facing strings.
Drift + decision before P8 opens.

#### Confidence check
**Target: ≥ 98%.** 4-aspect: code · docs · phi-core leverage (≤1 import under `cli/src/commands/session.rs`) · archive-plan compliance. Report composite % before P8.

---

### P8 — s02 memory-extraction + s03 agent-catalog + s05 Template C/D listener bodies (~2 days)

#### Goals
Wire reactive listeners to real work. The stubs from P3 become functional.

#### Deliverables
1. **`MemoryExtractionListener.on_event` body** — fires on `SessionEnded`:
   - Fetch Session detail (turns, loop records, project tags, org tags, agent tags).
   - Compose `phi_core::agent_loop` call with extractor profile's system prompt + transcript.
   - For each candidate memory the extractor returns, determine allocation pool via the per-tag rule table from `concepts/system-agents.md`.
   - Emit `MemoryExtracted` audit event **with structured tag list + session reference** (so M6's C-M6-1 can materialise Memory nodes from the audit stream without re-running the extractor).
   - Failure modes: queue saturation → `MemoryExtractionSkipped { reason: queue_saturated }`; extraction agent disabled → `MemoryExtractionSkipped { reason: agent_disabled }`; LLM API error → retry 3× with exponential backoff → `MemoryExtractionFailed`.
2. **`AgentCatalogListener.on_event` body** — fires on 8 `DomainEvent` variants:
   - `AgentCreated` → upsert fresh row.
   - `AgentArchived` → upsert with `active: false`.
   - `HasLeadEdgeCreated` / `ManagesEdgeCreated` / `HasAgentSupervisorEdgeCreated` → update role-index.
   - `HasProfileEdgeChanged` → refresh cached profile snapshot on catalog entry.
   - Queue-depth tracking via `SystemAgentRuntimeStatus` upsert.
   - Failure modes per s03 spec.
3. **Template C + D listener bodies** — already shipped at P3; P8 confirms they work end-to-end via acceptance (s05 vertical).
4. **Ops doc**: `m5/operations/system-flows-s02-s03-operations.md`.

#### phi-core leverage
- **Q1 direct imports — 1 at P8**:
  - `use phi_core::agent_loop;` — in `MemoryExtractionListener::on_event` (the extractor runs the same phi-core primitive as the Session runtime — phi-core leverage exemplar).
- Positive close-audit grep: `grep -En '^use phi_core::' modules/crates/domain/src/events/listeners.rs` → **1 new line** (`agent_loop`).

#### Tests added (~18)
- `server/tests/acceptance_system_flows_s02.rs` — 6 scenarios: happy extraction, queue saturation → skipped, agent disabled → skipped, LLM error → retry exhausted → MemoryExtractionFailed, multi-tag session produces memories per pool, Shape E forbidden session skipped.
- `server/tests/acceptance_system_flows_s03.rs` — 8 scenarios: each of the 8 trigger variants produces the correct catalog-entry mutation.
- `server/tests/acceptance_system_flows_s05.rs` — 4 scenarios: Template C fires on MANAGES; Template D fires on HAS_AGENT_SUPERVISOR; Templates A+C+D simultaneously active fire correct grant counts.

#### Archive-plan compliance check
Walk P8 §Deliverables:
- ✅/⚠/✗ `MemoryExtractionListener::on_event` body implemented per spec (fetch Session → `agent_loop` extractor → `MemoryExtracted` audit with **structured tag list** consumable by M6's C-M6-1).
- ✅/⚠/✗ `AgentCatalogListener::on_event` handles all 8 variants correctly.
- ✅/⚠/✗ Template C + D listener bodies verified end-to-end via s05 acceptance.
- ✅/⚠/✗ Ops doc `system-flows-s02-s03-operations.md` present.
- ✅/⚠/✗ Failure modes (queue saturation, agent disabled, LLM retry exhausted) each have an acceptance scenario.
Drift + decision before P9 opens.

#### Confidence check
**Target: ≥ 98%.** 4-aspect: code · docs · phi-core leverage (1 new `agent_loop` import in listeners.rs) · archive-plan compliance. Report composite % before P9.

---

### P9 — Seal: cross-page acceptance + e2e first-session + CI + runbook + independent 3-agent re-audit (~2 days)

#### Goals
M5 closes. Independent re-audit targets ≥99%.

#### Deliverables
1. **Cross-page acceptance** `server/tests/acceptance_m5.rs`: bootstrap → org → agent → project → adopt Template A + B on page 12 → add org-specific system agent on page 13 → launch session on page 14 → tail events → session ends → verify memory extracted (audit) + catalog updated + UsesModel edge exists + RUNS_IN edge exists + page 11's "Recent sessions" panel shows the session.
2. **End-to-end first-session fixture** `server/tests/e2e_first_session.rs`: matches admin/14-N4's post-session checklist verbatim. `phi bootstrap claim` → `phi org create` → `phi agent create` (Human CEO + LLM intern) → `phi project create` → `phi session launch` (using CLI directly, subprocess) → session ends → assertions.
3. **CI updates** `.github/workflows/rust.yml`:
   - Extend acceptance `--test` list with: `acceptance_sessions_launch`, `acceptance_sessions_preview`, `acceptance_sessions_terminate`, `acceptance_authority_templates`, `acceptance_system_agents`, `acceptance_system_flows_s02`, `acceptance_system_flows_s03`, `acceptance_system_flows_s05`, `acceptance_m5`, `e2e_first_session`.
   - New job `phi-core-leverage-targets`: asserts exact import counts per Part 1.5.
   - New job `migration-idempotency`: applies 0001 → 0005 in order on fresh DB, asserts schema hash.
4. **Ops runbook M5 section** in `docs/ops/runbook.md` — mirrors M4 structure: per-page ops runbook index + M5 error-code reference (~25 stable codes) + incident playbooks (5): session-worker-saturated, session-stuck-in-running, Template-revoke-cascade-stalled, system-agent-queue-runaway, s02-extraction-dead-letter.
5. **M5 troubleshooting doc** `m5/user-guide/troubleshooting.md` — full stable-code table + CLI exit codes + cross-org isolation invariants (matches M4 P8 follow-up).
6. **phi-core reuse map** `m5/architecture/phi-core-reuse-map.md` — M5 per-page leverage summary + P9 positive close-audit record (matches M4/P8 style end-to-end).
7. **Independent 3-agent re-audit** (mirrors M4/P8):
   - **Agent (a) Rust correctness** — audits `session::launch` + `BabyPhiSessionRecorder` + C-M5-6 Shape B flip + C-M5-5 ModelConfig gate + Template C/D pure-fns. Reports HIGH/MEDIUM/LOW/OBSERVATION.
   - **Agent (b) Docs fidelity** — audits C1–C27 against code. ADR status. Per-page doc `Last verified` currency.
   - **Agent (c) Vertical-slice integrity** — audits CLI parity (`phi session *`), web parity (3 pages + page-11 retrofit), wire-contract strip tests, s02/s03/s05 end-to-end coverage, cross-org invariant (any new session-surface isolation tests needed).
8. **Base plan M6 carryover confirmation** — re-verify the C-M6-1 entry still exists in [`/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md) §M6 `#### Carryovers from M5` subsection + structured with (i)/(ii)/(iii) per D6 resolution. If a future commit between P0 and P9 accidentally dropped or edited it, flag as HIGH regression.

#### phi-core leverage
- **Q1 direct imports**: 0 new at P9 (measurement phase).
- **Total import count verification**: `grep -rn '^use phi_core::' modules/crates/ | wc -l` → should be ~24 lines (M4 close ≈14, +10 new at M5). Actual number pinned in close report.

Positive close-audit greps: run the full Part 1.5 grep battery + confirm 0 regressions vs M4/P8 baseline.

#### Tests added (~6)
- `acceptance_m5.rs` (1 end-to-end).
- `e2e_first_session.rs` (1 fixture).
- P9 re-audit may surface LOW findings requiring 2-4 additional tests.

#### Archive-plan compliance check
Walk P9 §Deliverables + **the full M5 archive plan end-to-end** (every phase's deliverables from P0 through P8). The seal-phase compliance walk is the milestone-level audit — drift accumulated across 10 phases surfaces here:
- ✅/⚠/✗ All 27 commitments in Part 2 closed with their verification files present.
- ✅/⚠/✗ All 3 ADRs (0029/0030/0031) status = Accepted.
- ✅/⚠/✗ All 6 decisions (D1–D7) reflected in the shipped code matching the resolution in Part 3.
- ✅/⚠/✗ All 19 gap items (G1–G19) from Part 1 resolved.
- ✅/⚠/✗ Part 1.5 import-count predictions match the actual `grep` output.
- ✅/⚠/✗ Cross-page acceptance (`acceptance_m5.rs`) + e2e fixture (`e2e_first_session.rs`) both present.
- ✅/⚠/✗ CI extensions in `rust.yml` + 2 new jobs (`phi-core-leverage-targets`, `migration-idempotency`).
- ✅/⚠/✗ Ops runbook M5 section + troubleshooting doc + phi-core reuse map all present.
- ✅/⚠/✗ M6 carryover C-M6-1 still present in the base plan at `/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §M6 (re-verify — this is where drift from a P0-planted amendment would show up after 9 phases of churn).
- Any drift report captured as "M5 post-mortem" for carry-forward to M6 detailed planning.

The 3 independent re-audit agents (a/b/c) each perform their own compliance walk against their specialisation (Rust / docs / vertical-integrity); composite confidence % comes from those reports.

#### Confidence check
**Target: ≥ 99%** via independent 3-agent audit (M4/P8 precedent hit 99.3%). Close criteria (4-aspect):
- Code: `cargo test --workspace` green; target test count ≥ ~950 Rust + ≥ ~88 web.
- Docs: M5 README shows every phase ✓; ADRs 0029/0030/0031 all Accepted; runbook M5 section + troubleshooting complete; phi-core reuse map populated end-to-end.
- phi-core leverage: `check-phi-core-reuse.sh` + `phi-core-leverage-targets` CI job both green; positive grep battery passes.
- Archive-plan compliance: full-milestone walk (above) all ✅; any ⚠ documented in the M5 post-mortem.
- Report final composite % + hand off for milestone tag.

---

**Total phase estimate: ~22–25 calendar days ≈ 3.5 weeks.** Matches M4's calendar. P4 is the biggest single phase at 5 days (5 carryover closes + phi-core-heaviest surface).

---

## Part 5 — Testing strategy  `[STATUS: ⏳ pending]`

M5 aggregate test additions (revised for full scope):

| Layer | New M5 count | Purpose |
|---|---|---|
| Domain unit (wraps serde + events + recorder + template C/D + composites) | ~30 | Type shapes + pure helpers |
| Domain proptest (Template C/D firing + session-recorder persist) | ~3 proptests (50 cases each) | Behaviour invariants |
| Store unit (migration 0005 + 14 new repo methods × 2 impls + active-sessions count) | ~30 | Migration + parity |
| Server unit (permission-check preview helpers + session validators + template cascade + system-agent handlers) | ~14 | Handler helpers |
| Server integration (3 per-page acceptance suites + cross-page + session launch/preview/terminate/tools + s02/s03/s05 + e2e) | ~95 scenarios | HTTP contract + reactive flows |
| CLI integration (session + template + system-agent + agent update extension + completion regression) | ~18 | Subcommand surface |
| Web unit (4 new primitives + 3 page component suites + page-11 retrofit) | ~20 | Pure React + translators |
| Web Playwright e2e (page 14 launch + tail + terminate; page 12 approve; page 13 tune) | ~6 | UX smoke |
| **M5 added total** | **~216 tests** | — |

**Post-M5 workspace target**: 805 (M4 close) + ~150 Rust ≈ **~950 Rust**; 68 + ~20 Web ≈ **~88 Web**; **~1040 combined**, up from M4's 873.

**Key invariants shipping in M5**:
- Session wrap embeds `phi_core::Session` verbatim (compile-time witness + serde round-trip).
- Every launched session has a resolvable `ModelRuntime` via `UsesModel` edge (C-M5-2 close).
- Every launched session has a `RUNS_IN` edge to its project (C-M5-3 close).
- Shape B both-approve now materialises a real project (C-M5-6 close).
- `ModelConfig` change blocked with 409 on active sessions (C-M5-5 close).
- Template C + D fire correctly on their trigger edges (50-case proptests).
- s02 fires `MemoryExtracted` audit with full tag metadata (audit replay must reconstruct the same set M6's Memory nodes would materialise from).
- s03 catalogue updates within 5s SLO (R-SYS-s03-1).

---

## Part 6 — Documentation plan  `[STATUS: ⏳ pending]`

Root: `baby-phi/docs/specs/v0/implementation/m5/`. Mirrors M4 structure.

```
implementation/m5/
├── README.md                                10-phase index + ADR table
├── architecture/
│   ├── overview.md                          M5 system map + commitment ledger
│   ├── m4-postflight-delta.md               P0's 10-item audit
│   ├── session-persistence.md               3-wrap pattern + BabyPhiSessionRecorder
│   ├── session-launch.md                    Page 14 architecture; Permission Check preview; parallelize gate; cancellation
│   ├── authority-templates.md               Page 12 architecture + Template C/D fire rules
│   ├── system-agents.md                     Page 13 + SystemAgentRuntimeStatus
│   ├── shape-b-materialisation.md           C-M5-6 pre/post with sequence diagrams
│   ├── event-bus-m5-extensions.md           8 new DomainEvent variants + emit callsites
│   ├── template-a-firing.md                 (M4 doc, extended with C + D)
│   └── phi-core-reuse-map.md                M5 per-page leverage summary
├── user-guide/
│   ├── first-session-walkthrough.md         Fresh-install page-14 tour
│   ├── authority-templates-walkthrough.md
│   ├── system-agents-walkthrough.md
│   ├── cli-reference-m5.md                  Full `phi session` + `phi template` + `phi system-agent` surface
│   └── troubleshooting.md                   M5 stable codes + cross-page quirks
├── operations/
│   ├── session-launch-operations.md
│   ├── authority-templates-operations.md
│   ├── system-agents-operations.md
│   └── system-flows-s02-s03-operations.md
└── decisions/
    ├── 0029-session-persistence-and-recorder-wrap.md   D2
    ├── 0030-template-node-uniqueness.md                D1
    └── 0031-session-cancellation-and-concurrency.md    D3
```

Conventions unchanged from M4. ADR numbering continues (0029+).

---

## Part 7 — CI / CD extensions  `[STATUS: ⏳ pending]`

1. **`rust.yml` `acceptance` job** — extend `--test` list with `acceptance_sessions_launch`, `acceptance_sessions_preview`, `acceptance_sessions_terminate`, `acceptance_authority_templates`, `acceptance_system_agents`, `acceptance_system_flows_s02`, `acceptance_system_flows_s03`, `acceptance_system_flows_s05`, `acceptance_m5`, `e2e_first_session`. Keep `--test-threads 1`.
2. **`rust.yml` `phi-core-leverage-targets` job (new)** — runs positive-grep assertions from Part 1.5 (exact import counts per file).
3. **`rust.yml` `migration-idempotency` job (new)** — applies 0001 → 0005 in order on fresh SurrealDB, asserts schema hash matches golden.
4. **`rust.yml` `phi-core-reuse` job** — unchanged (hard-gated since M2); deny-list extended at P0.
5. **`doc-links.yml`** — new M5 docs tree; seed depth at P1.
6. **`ops-doc-headers` job** — unchanged; new M5 ops runbooks must carry header.
7. **`spec-drift.yml`** — extend grep set with `R-ADMIN-1[234]-*` + `R-SYS-s0[235]-*`.

---

## Part 8 — Verification matrix  `[STATUS: ⏳ pending]`

| # | Commitment | Test / check |
|---|---|---|
| C1 | M4 post-flight delta log | `m5/architecture/m4-postflight-delta.md` written; 0 stale items |
| C2 | Migration 0005 forward-only | `store/tests/migrations_0005_test.rs` |
| C3 | Session/LoopRecord/Turn wrap | `domain/tests/m5_wrap_roundtrip.rs` + compile-time witnesses |
| C4 | Shape B sidecar + 3 repo methods | `store/tests/shape_b_sidecar_test.rs` |
| C5 | `AgentProfile.model_config_id` field | `acceptance_agents_profile::model_config_id_serde_back_compat` |
| C6 | 14 new session-surface repo methods | `store/tests/session_repo_parity.rs` |
| C7 | `count_active_sessions_for_agent` real query | `store/tests/active_sessions_count_test.rs` |
| C8 | 8 new DomainEvent variants | `domain/src/events/mod.rs::tests` |
| C9 | `BabyPhiSessionRecorder` wrap | `domain/tests/session_recorder_wrap_test.rs` |
| C10 | Template C + D fire pure-fns | `domain/tests/template_{c,d}_firing_props.rs` |
| C11 | 4 new fire-listeners wired | `server/src/state.rs::tests::handler_count_at_m5` (5 total = M4's Template A + 4 new) |
| C12 | Page 14 vertical (5 carryover closes in one phase) | `acceptance_sessions_{launch, preview, terminate}.rs` + extended `acceptance_projects_create.rs` + extended `acceptance_agents_profile.rs` + `GET /sessions/:id/tools` acceptance |
| C13 | Page 12 vertical | `acceptance_authority_templates.rs` + CLI + web |
| C14 | Page 13 vertical | `acceptance_system_agents.rs` + CLI + web |
| C15 | `phi session` CLI | `cli/tests/session_snapshot.rs` + completion regression |
| C16 | Web UI pages 12/13/14 | Component tests + Playwright e2e |
| C17 | s02 memory-extraction listener | `acceptance_system_flows_s02.rs` |
| C18 | s03 agent-catalog listener | `acceptance_system_flows_s03.rs` |
| C19 | s05 Template C + D extension | `acceptance_system_flows_s05.rs` |
| C20 | Cross-page acceptance + e2e fixture | `acceptance_m5.rs` + `e2e_first_session.rs` |
| C21 | CI extensions | `rust.yml` green on PR |
| C22 | Ops + troubleshooting + runbook | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C23 | phi-core reuse map (M5) | doc-link check |
| C24 | ADRs 0029/0030/0031 Accepted | Flipped at P3/P1/P4 closes respectively |
| C25 | Independent 3-agent re-audit | 3 agent reports captured at P9; LOW findings remediated; composite ≥99% |
| C26 | C-M6-1 pinned in base plan | `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §M6 amendment landed at P0 |
| C27 | Per-phase **4-aspect** confidence % + archive-plan compliance walk | Each phase close reports 4-aspect (code · docs · phi-core · archive-plan) + composite. Every deliverable bullet in the archived plan walked and marked ✅/⚠/✗. |

**First-review confidence target: ≥ 98 %**. Post-P9 re-audit: ≥ 99 %.

---

## Part 9 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** → `baby-phi/docs/specs/plan/build/<8hex>-m5-templates-system-agents-sessions.md`. (~2 min)
1. **P0 — Post-flight delta + ADRs 0029/0030/0031 + base-plan M6 C-M6-1 amendment + CI grep + docs seed** (~4–6 h)
2. **P1 — Foundation** — migration 0005 + 3-way Session wrap + composites + config + web primitives + CLI scaffold + ADRs. (~3 d)
3. **P2 — Repository** — 14 new methods + flip `count_active_sessions_for_agent`. (~3 d)
4. **P3 — Event bus + `BabyPhiSessionRecorder` + Template C/D pure-fns + 4 listener scaffolds** (~3 d)
5. **P4 — Page 14 session-launch vertical** — M5's biggest phase; closes C-M5-2/3/4/5/6. (~5 d)
6. **P5 — Page 12 authority templates vertical** (~3 d)
7. **P6 — Page 13 system agents vertical** (~3 d)
8. **P7 — `phi session` CLI + `phi agent update --model-config-id` + web polish + page-11 "Recent sessions" retrofit** (~2 d)
9. **P8 — s02 + s03 + s05 listener bodies** (~2 d)
10. **P9 — Seal** — cross-page + e2e + CI + runbook + troubleshooting + 3-agent re-audit → ≥99%. (~2 d)
11. **Re-audit → remediation → ≥99%** (mirrors M4 post-P8 pass).
12. **Tag milestone** — `git tag v0.1-m5` in baby-phi submodule (user-managed per M1/M2/M3/M4 precedent).

**Total estimate: ~22–25 calendar days ≈ 3.5 weeks.** P4 can parallelise some sub-work with P5/P6 after P3 closes, but the critical path is P0 → P1 → P2 → P3 → P4 → P8 → P9.

---

## Part 10 — Critical files  `[STATUS: n/a]`

**New** (~50 production files + ~30 test files + ~15 docs):

- `modules/crates/store/migrations/0005_sessions_templates_system_agents.surql`
- `modules/crates/domain/src/model/composites_m5.rs` — `ShapeBPendingProject`, `AgentCatalogEntry`, `SystemAgentRuntimeStatus`, `SessionDetail`, `PermissionCheckTrace`.
- `modules/crates/domain/src/session_recorder.rs` — `BabyPhiSessionRecorder`.
- `modules/crates/domain/src/templates/c.rs`, `templates/d.rs`.
- `modules/crates/domain/src/audit/events/m5/{mod, sessions, templates, system_agents, memory}.rs`.
- `modules/crates/server/src/platform/sessions/{mod, launch, preview, show, terminate, list, tools}.rs`.
- `modules/crates/server/src/platform/templates/{mod, list, approve, deny, adopt, revoke}.rs`.
- `modules/crates/server/src/platform/system_agents/{mod, list, tune, add, disable, archive, events_feed}.rs`.
- `modules/crates/server/src/handlers/{sessions, templates, system_agents}.rs`.
- `modules/crates/cli/src/commands/{session, template, system_agent}.rs`.
- `modules/web/app/components/{SessionEventStreamRenderer, PermissionCheckPreviewPanel, TemplateAdoptionTable, SystemAgentStatusCard}.tsx`.
- `modules/web/app/(admin)/organizations/[id]/templates/` — page 12.
- `modules/web/app/(admin)/organizations/[id]/system-agents/` — page 13.
- `modules/web/app/(admin)/organizations/[id]/projects/[pid]/sessions/new/` — page 14.
- `docs/specs/v0/implementation/m5/**/*.md` — full tree (~15 docs).
- `modules/crates/server/tests/acceptance_sessions_{launch, preview, terminate}.rs`.
- `modules/crates/server/tests/acceptance_authority_templates.rs`.
- `modules/crates/server/tests/acceptance_system_agents.rs`.
- `modules/crates/server/tests/acceptance_system_flows_{s02, s03, s05}.rs`.
- `modules/crates/server/tests/acceptance_m5.rs`.
- `modules/crates/server/tests/e2e_first_session.rs`.

**Modified**:

- [`/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`](/root/projects/phi/baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md) — §M6 amendment at P0: new subsection `#### Carryovers from M5 — must-pick-up at M6 detailed planning` with **C-M6-1** (Memory contract + ownership-by-multi-tag + permission-over-time). Immediately after the existing `### M6 — Agent self-service surfaces (≈2 weeks)` header; mirrors the existing M3→M5 + M4→M5 + M4→M8 carryover subsection shapes in the same file.
- `modules/crates/domain/src/model/nodes.rs` — Session/LoopRecordNode/TurnNode wraps replace id-only scaffolds + `SessionGovernanceState` enum.
- `modules/crates/domain/src/model/edges.rs` — no enum change (confirm at P0 audit).
- `modules/crates/domain/src/repository.rs` — 14 new methods + flip `count_active_sessions_for_agent`.
- `modules/crates/domain/src/in_memory.rs` — matching in-memory impls.
- `modules/crates/domain/src/events/mod.rs` — 8 new DomainEvent variants.
- `modules/crates/domain/src/events/listeners.rs` — 4 new listeners (2 full bodies at P3, 2 stubs filled at P8).
- `modules/crates/server/src/state.rs` — `session_registry: Arc<DashMap<SessionId, CancellationToken>>` added.
- `modules/crates/server/src/config.rs` + `config/default.toml` — `[session] max_concurrent`.
- `modules/crates/server/src/router.rs` — 6 new session routes + 5 new template routes + 5 new system-agent routes.
- `modules/crates/server/src/platform/projects/create.rs` — C-M5-6 flip (remove `_keep_materialise_live` + add sidecar read on approve + sidecar write on submit).
- `modules/crates/server/src/platform/projects/detail.rs` — M4 "Recent sessions" placeholder flips to real rows via `list_sessions_in_project`.
- `modules/crates/server/src/platform/agents/update.rs` — C-M5-5 flip.
- `modules/crates/cli/src/main.rs` + `cli/src/commands/mod.rs` — register `session` + `template` + `system_agent` subcommands.
- `modules/crates/cli/tests/completion_help.rs` — regression extensions.
- `modules/web/app/(admin)/organizations/[id]/projects/[pid]/page.tsx` — "Recent sessions" panel retrofit.
- `.github/workflows/rust.yml` — acceptance-job `--test` list + 2 new jobs.
- `docs/ops/runbook.md` — M5 section.
- `scripts/check-phi-core-reuse.sh` — deny-list extended at P0.
- `scripts/check-spec-drift.sh` — M5 id registry.

---

## Part 11 — Open questions (non-blocking)  `[STATUS: n/a]`

Track in per-phase exec notes:

- **Q1** — `[session] max_concurrent` default value (16). Confirm with user at P4 opening; M7b revisits for Redis-backed shared registry.
- **Q2** — Permission Check preview location (D5 assumed server-side). Confirm at P4 opening.
- **Q3** — WebSocket vs SSE for `GET /sessions/:id/events` live stream. Default: **SSE** (simpler, tokio-friendly, supported by reqwest). M7b may add WebSocket for bidirectional terminate. Confirm at P7.
- **Q4** — `MemoryExtracted` audit event's tag-list shape. Must be structured (not free-form string) so M6's Memory-node materialiser can consume directly. Draft at P8 + confirm at P8 close.
- **Q5** — Template E handling on page 12 (R-ADMIN-12-R3: "not adopted — always available on demand"). Render as always-adopted row OR as "use via AR" CTA? Recommend CTA. Confirm at P5 planning.
- **Q6** — Does `UsesModel` edge get written at `session::launch` time OR at `apply_agent_creation` time? Base plan §M5 says launch-time; M5 sticks to that, but a future refactor may move it to agent-profile-update for efficiency. Document the choice in ADR-0029.
- **Q7** — Queue-depth metric emission cadence for `SystemAgentRuntimeStatus` (5s poll per R-ADMIN-13-N3). Keep simple polling at M5; M7b may add push.

---

## What stays unchanged  `[STATUS: n/a]`

- Concept docs (`docs/specs/v0/concepts/`) are the source of truth; M5 surface-count deltas land in the M5 plan if discovered at P0.
- M4 ships unchanged; M5 extends (new types + routes + audit events + docs tree + page-11 retrofit), doesn't refactor M4 surfaces.
- `phi-core` is a library dependency; M5 consumption adds **~10 new direct imports** (3 `session::model` in `nodes.rs`, 2 in `session_recorder.rs`, 4 in `sessions/` platform, 1 in `events/listeners.rs`). Every other M5 surface stays phi-core-import-free by design.
- Four-tier phi-core leverage enforcement — applied consistently per-phase.
- CLI binary name is **`phi`** (never `baby-phi`) — M5 adds `phi session {launch, show, terminate, list}` + `phi template {...}` + `phi system-agent {...}` + `phi agent update --model-config-id`.

---

## Open items — resolved at plan close

All 6 planning-decision items confirmed at plan close:
- ✅ **D1**: Template UNIQUE(kind), one shared row per kind.
- ✅ **D2**: Wrap `phi_core::SessionRecorder` as `BabyPhiSessionRecorder`.
- ⏳ **D3**: `[session] max_concurrent = 16` (assumed default; confirm at P4).
- ✅ **D4**: `phi session launch` tail-live by default + `--detach` flag.
- ⏳ **D5**: Permission Check preview server-side (assumed default; confirm at P4).
- ✅ **D6**: Memory audit-only at M5; full contract + node + default impl + ownership-by-multi-tag + permission-over-time retrieval pinned as C-M6-1 in base plan.

No pre-P0 blockers remain. P0 execution can open immediately upon plan approval.

---

## Verification (how to test end-to-end after full M5 close)

After P9 close, run the following to verify M5 shipped cleanly:

1. **Workspace tests**: `cargo test --workspace --all-targets` → expect ~950 Rust tests, 0 failures.
2. **Web tests**: `cd modules/web && npm run test && npm run typecheck && npm run lint && npm run build` → expect ~88 web tests + clean build.
3. **Acceptance gates**: `cargo test -p server --release --test acceptance_m5 --test e2e_first_session -- --test-threads 1` → cross-page + end-to-end fixtures green.
4. **phi-core grep discipline**: `bash scripts/check-phi-core-reuse.sh` + `bash scripts/check-doc-links.sh` + `bash scripts/check-ops-doc-headers.sh` + `bash scripts/check-spec-drift.sh` → all green.
5. **Positive import-count verification**: `grep -rn '^use phi_core::' modules/crates/ | wc -l` ≈ 24; `grep -En '^use phi_core::session::' modules/crates/` ≥ 5 lines.
6. **CLI smoke**: `phi session launch --help` shows the 4 subcommands + flag list; completion regression passes for all 4 shells.
7. **Fresh-install e2e** (manual): start server, run `phi bootstrap claim` → `phi org create` → `phi agent create` → `phi project create` → `phi session launch --agent-id X --project-id Y --prompt "hello"`; session starts, runs, ends; `phi project show <pid>` "Recent sessions" panel populates; audit log shows `SessionStarted`, `TurnEnd`s, `SessionEnded`, `MemoryExtracted`; `phi agent catalog show` (if shipped) reflects updated last-seen.
8. **Milestone tag**: `git tag v0.1-m5` in baby-phi submodule (user-managed).

---

# Drift addenda  `[STATUS: append-only — one section per closed phase]`

**Purpose.** The plan body above is a **P0 snapshot** — a point-in-time statement of what we intended to ship before any M5 code landed. Per the 4-aspect confidence discipline (Part 4 preamble), every phase close walks the plan archive deliverable-by-deliverable and marks ✅ / ⚠ drift / ✗ missing. Whenever a ⚠ drift surfaces, **the addendum below is the single canonical place** it lands — preserving the P0 intent visible in the plan body while surfacing the deviations so readers never re-read a stale P0 code block and trust it. Future phase closes (P2 → P9) append their own subsections here using the same shape.

**Discipline:** every phase close → walk plan archive → list ⚠ drift items → one addendum bullet per item → link to where the correction lives in downstream docs/ADRs/code. No ⚠ drift ever ships silently; no archive ever drifts out of coherence with code. This addendum pattern is the inaugural use at M5/P1 close; future phases follow the same shape.

## P1 drift addendum (closed 2026-04-23)

Three ⚠ drift items surfaced at P1 implementation. None blocked close; all three ship with in-file notes + downstream doc updates. The P1 §Deliverables list in the plan body reads the P0 intent; this addendum pins the P1 reality.

### D1.1 — `session` and `turn` tables leverage DEFINE FIELD on pre-existing 0001 scaffolds (not fresh DEFINE TABLE)

- **P0 plan said** (§P1 Deliverables item 1): *"DEFINE 3 new session tables: `session`, `loop_record`, `turn` with `FLEXIBLE TYPE object` for the phi-core `inner` field + explicit governance columns."*
- **Reality at P1 implementation**: migration 0001 (M1) already shipped `DEFINE TABLE session SCHEMAFULL;` + `DEFINE TABLE turn SCHEMAFULL;` as id-only scaffolds with a pre-existing `created_at: string` mandatory field ([`0001_initial.surql:142-149`](../../../../modules/crates/store/migrations/0001_initial.surql#L142-L149)). SurrealDB rejects a second `DEFINE TABLE` for an existing table (`"The table 'session' already exists"`).
- **What P1 shipped**: migration 0005 uses `DEFINE FIELD` on `session` + `turn` to layer the M5 governance columns (`inner`, `owning_org`, `started_by`, `governance_state`, `started_at`, `ended_at`, `tokens_spent` for session; `inner`, `loop_id`, `turn_index` for turn). Only `loop_record` is a fresh `DEFINE TABLE`. The 0001 `loop` scaffold table stays as a zombie (no writers) and may be dropped in M7b cleanup.
- **Downstream consequence P2+ must honour**: the M1 `created_at: string` field on `session` + `turn` remains mandatory. The P2 `persist_session` / `append_turn` repo writers must populate **both** `created_at` and `started_at` — typically set to the same wall-clock value.
- **Documented at**:
  - [`0005_sessions_templates_system_agents.surql` §3 header](../../../../modules/crates/store/migrations/0005_sessions_templates_system_agents.surql) — "`session` + `turn` were declared as SCHEMAFULL scaffolds with only a `created_at` column in migration 0001. M5/P1 layers the M5 governance fields on top …"
  - [`m5/architecture/session-persistence.md` §Storage layout — migration 0005](../../v0/implementation/m5/architecture/session-persistence.md) — per-tier Origin column distinguishes "0001 scaffold" vs "0005 (new)" with explicit fields-added list.
  - [`store/tests/migrations_0005_test.rs` §session_table_accepts_governance_row_with_inner_object](../../../../modules/crates/store/tests/migrations_0005_test.rs) — in-test comment reminds P2 implementers to populate both.

### D1.2 — `runs_session` RELATION retyped (REMOVE + DEFINE), not fresh DEFINE TABLE

- **P0 plan said** (§P1 Deliverables item 1 + §G4): *"DEFINE TABLE `runs_session` TYPE RELATION FROM `session` TO `project`."* Treated as a fresh table creation.
- **Reality at P1 implementation**: migration 0001 ([line 327](../../../../modules/crates/store/migrations/0001_initial.surql#L327)) already shipped `DEFINE TABLE runs_session TYPE RELATION FROM agent TO session;` — the semantically backwards direction (an "agent runs a session" vs the M5 need "session runs in project"). The 0001 scaffold had **zero production writers** (verified via grep); the RELATION was never reachable. SurrealDB rejects a second `DEFINE TABLE runs_session`.
- **What P1 shipped**: migration 0005 does `REMOVE TABLE runs_session` + `DEFINE TABLE runs_session TYPE RELATION FROM session TO project` — same forward-only retype pattern as the `uses_model` flip in the same migration (0001 had `uses_model` pointing at vestigial `model_config`; M5/P1 retyped to `model_runtime`).
- **Safe-retype proof**: `grep -rn 'RunsSession\|runs_session' modules/crates/ --include='*.rs'` returns only the edge-enum variant + migration files (no `RELATE ... runs_session ... TO ...` writer statements anywhere); the M4 `edges.rs::RunsSession` enum variant has been dormant since M3/P1 shipped it.
- **Documented at**:
  - [`0005_sessions_templates_system_agents.surql` §4 header](../../../../modules/crates/store/migrations/0005_sessions_templates_system_agents.surql) — "The 0001 scaffold typed `runs_session` as `agent → session`. It had zero production writers (the governance semantic was backwards …). Retyped here …"

### D1.3 — Session/LoopRecordNode/TurnNode wraps use nested `inner` field (not `#[serde(flatten)]`)

- **P0 plan said** (§Part 1.5 + §G3): the Session wrap code block carried `#[serde(flatten)] pub inner: phi_core::session::model::Session` on all three wraps, described as "M3 `OrganizationDefaultsSnapshot` pattern byte-for-byte".
- **Reality at P1 implementation**: two blockers:
  - `phi_core::session::model::Session` already carries `session_id: String` + `agent_id: String` ([`phi-core/src/session/model.rs:490-501`](../../../../../phi-core/src/session/model.rs#L490-L501)). With `#[serde(flatten)]` the wire JSON would carry **both** baby-phi's `id: SessionId` (UUID newtype) and phi-core's flattened `session_id: String` (UUID-as-string) for the same conceptual value — plus the same collision on `started_by: AgentId` vs `agent_id: String`. Deserialisation is ambiguous (which field wins?) and the wire payload carries duplicate data.
  - The cited M3 precedent (`composites_m3::OrganizationDefaultsSnapshot`) actually uses **nested** phi-core wraps, never `#[serde(flatten)]` ([`composites_m3.rs:93-105`](../../../../modules/crates/domain/src/model/composites_m3.rs#L93-L105)). The plan's code block inaccurately described the M3 precedent.
- **What P1 shipped**: all three wraps (`Session`, `LoopRecordNode`, `TurnNode`) use plain nested `pub inner: PhiCoreSession` / `PhiCoreLoopRecord` / `PhiCoreTurn` with no `#[serde(flatten)]`. Wire shape is `{"id": "...", "inner": {"session_id": "...", ...}, "owning_org": "...", ...}`. This matches the M3 + M4 `AgentProfile.blueprint` precedents uniformly.
- **SurrealDB storage impact**: the `inner` field maps naturally to `FLEXIBLE TYPE object` — a nested object column alongside the explicit governance columns. No change to the migration 0005 schema between the flatten vs nested options (FLEXIBLE object accepts either wire shape), but the nested form is what the repo writers serialise + what clients receive on GET.
- **Documented at**:
  - [`m5/decisions/0029-session-persistence-and-recorder-wrap.md` §D29.1](../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) — code block updated (no flatten) + new paragraph titled **"Nested `inner`, NOT `#[serde(flatten)]`"** explaining the collision + M3 precedent.
  - [`m5/decisions/0029-…md` Status line](../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) — flipped to "Accepted (in part)" with a P1 status note (recorder portion stays Proposed until P3).
  - [`m5/architecture/session-persistence.md` §Why nested, not flattened](../../v0/implementation/m5/architecture/session-persistence.md) — full walk of the rationale.
  - [`nodes.rs:820-913`](../../../../modules/crates/domain/src/model/nodes.rs) — the wrap structs carry the ADR-matching nested form + 3 compile-time coercion witnesses in the test module.

### P1 drift addendum — coherence checklist

- [x] Each drift item has a fix in downstream code OR an adapter note in the migration/ADR/architecture docs.
- [x] Each drift item's documentation link is verified present (doc-links CI green).
- [x] No drift item relies on "reader will infer from context" — every entry states what P0 said vs what shipped vs where the correction lives.
- [x] Composite P1 confidence floor held at 97% (archive-plan compliance 97%; others 99-100%). Target was ≥97%; met.

**P2 implementers**: read this addendum before opening P2. D1.1 in particular is load-bearing — the P2 repo writers must populate `created_at` on `session` + `turn` alongside `started_at` or `INSERT` will fail the 0001 ASSERT.

## P2 drift addendum (closed 2026-04-23)

Two ⚠ drift items surfaced during P2 implementation. Both are about SurrealDB's DDL/DML parser semantics, not the plan's logical structure. Both shipped with in-file notes + downstream doc updates; neither blocked close.

### D2.1 — `persist_session` / `append_loop_record` / `append_turn` use CREATE (not UPDATE-as-upsert)

- **P0 plan said** (§P2 Deliverables item 1 + §C6 + §G5): the 9 session-tier repo methods were described as plain "persist / append" without specifying CRUD verb. The implicit convention — borrowed from M3/M4 upsert_* helpers in `repo_impl.rs` (e.g. `upsert_project`, `upsert_agent`) — is `UPDATE type::thing('<table>', $id) CONTENT $body`, which existing code uses as a create-or-update idiom.
- **Reality at P2 implementation**: an initial CREATE/UPDATE probe (example binary, deleted at close) showed that `UPDATE type::thing('session', $id) CONTENT $body` on a non-existent record silently writes nothing — the session table stayed at 0 rows despite the `.check()` path returning Ok. Post-switch to `CREATE type::thing(...)`, the row is inserted correctly. SurrealDB's behaviour here appears to depend on the table's schema shape (SCHEMAFULL + FLEXIBLE object fields + NON-UNIQUE indexes on `session_by_project` / `session_by_agent`). Older M3/M4 helpers that use UPDATE-as-upsert work in their own contexts (likely because those tables lack the FLEXIBLE object column or the insert is downstream of a CREATE somewhere else in the compound tx).
- **What P2 shipped**:
  - `persist_session` uses `CREATE` (maps SurrealDB's unique-violation error to `RepositoryError::Conflict` per the method's documented "duplicate session id rejected" contract).
  - `append_loop_record` + `append_turn` use `CREATE` — each row has a fresh id so CREATE never conflicts during normal use.
  - `upsert_agent_catalog_entry` + `upsert_system_agent_runtime_status` use `BEGIN TRANSACTION; DELETE ... WHERE agent_id = $agent AND id != <this>; UPSERT type::thing(...) CONTENT $body; COMMIT` — matches the M3 `upsert_agent_profile` precedent (DELETE the prior row under the same agent_id, then UPSERT the canonical one). True upserts on `agent_id` with the UNIQUE index as the enforcement.
  - `persist_shape_b_pending` uses `CREATE` with `RepositoryError::Conflict` mapping so the UNIQUE(auth_request_id) index rejects duplicates at the SurrealDB tier.
- **Downstream consequence P3+ must honour**: repo writers that follow the "fresh row, governed by UNIQUE index" pattern use CREATE. Writers that expect idempotent replace-by-natural-key use the DELETE-then-UPSERT transaction pattern. Do NOT rely on bare `UPDATE type::thing(...)` to act as an upsert for these M5 tables. If you're tempted to, probe first via a temporary example binary (like the deleted `store/examples/p2_debug.rs`).
- **Documented at**:
  - [`modules/crates/store/src/repo_impl.rs` §persist_session](../../../../modules/crates/store/src/repo_impl.rs) — code comment: *"Use CREATE (not UPDATE) — persist_session semantics say a duplicate session id is a Conflict, not an overwrite."*
  - [`modules/crates/store/src/repo_impl.rs` §upsert_agent_catalog_entry](../../../../modules/crates/store/src/repo_impl.rs) — code comment: *"Delete-by-agent first so repeated listener fires replace the row rather than creating a second (the UNIQUE index would reject anyway…)"*.

### D2.2 — RELATE statement requires LET-first binding, not inline `type::thing(...)`

- **P0 plan said** (§P4 §launch_session compound tx): `RELATE session -> runs_session -> project`. Implementations naturally reach for the inline form: `RELATE type::thing('session', $sid)->runs_session->type::thing('project', $pid)`.
- **Reality at P2 implementation**: SurrealDB's RELATE parser rejects inline `type::thing(...)` in the FROM/TO slots with a parse error: *"Unexpected token `::`, expected :"*. The inline form never compiles. The existing M1/M2 edge-writing helpers already use a LET-first idiom (`upsert_ownership_raw` etc.) — documented in an inline comment there ("SurrealDB's RELATE parser does not accept `type::thing(...)` directly in the FROM/TO slots, so we bind the record refs via `LET` statements first") but the M5 plan did not reference this constraint.
- **What P2 shipped**: `persist_session` now ends with:
  ```rust
  "LET $f = type::thing('session', $sid); \
   LET $t = type::thing('project', $pid); \
   RELATE $f -> runs_session -> $t \
      SET id = type::thing('runs_session', $edge) \
      RETURN NONE"
  ```
  Matches the existing ownership-edge idiom byte-for-byte. `EdgeId::new()` mints the edge row's id.
- **Downstream consequence P3+ must honour**: every RELATE statement in M5 (and M6+) uses the LET-first pattern. Reviewers should reject PRs that inline `type::thing(...)` directly in RELATE slots.
- **Documented at**:
  - [`modules/crates/store/src/repo_impl.rs` §persist_session](../../../../modules/crates/store/src/repo_impl.rs) — code comment: *"SurrealDB's RELATE parser doesn't accept `type::thing(...)` in FROM/TO slots directly — bind via LET first (same idiom as the ownership edges in `upsert_ownership_raw`)."*

### P2 drift addendum — coherence checklist

- [x] Each drift item has a fix in downstream code + inline comments pointing at the correction.
- [x] Each drift item's documentation link is verified present (doc-links CI green).
- [x] No drift item relies on "reader will infer from context" — every entry states what P0 assumed vs what shipped vs where the correction lives.
- [x] Composite P2 confidence floor held at 97%+ (archive-plan compliance 97%; others 99-100).

**P3 implementers**: both D2.1 (CREATE vs UPDATE) and D2.2 (LET-RELATE) carry forward into P3's `BabyPhiSessionRecorder` persist hooks and the first post-commit RELATE-writing listener bodies. The patterns are locked in.

## P3 drift addenda  `[STATUS: captured at P3 close, 2026-04-23]`

### D3.1 — `DomainEvent::AgentCreated` field renamed `kind` → `agent_kind` (serde discriminator collision)

- **P0 plan said** (§Part 4 P3 Deliverables): `AgentCreated { agent_id, owning_org, kind, role, at, audit_event_id }` — field name `kind: AgentKind`.
- **Reality at P3**: `DomainEvent` carries `#[serde(tag = "kind", rename_all = "snake_case")]` at the enum level. Serde's `tag = "kind"` reserves the JSON key `kind` for the variant discriminator; a variant field *also* named `kind` produces the compile-time error `variant field name 'kind' conflicts with internal tag`. Renaming the field to `agent_kind` preserves the variant shape on the wire (just with a different field key) without removing the discriminator tag.
- **What shipped**: `AgentCreated { agent_id, owning_org, agent_kind: AgentKind, role: Option<AgentRole>, at, event_id }`. The variant's code-doc explicitly calls out the reason for the rename. Every test site (`sample_agent_created`, the `event_id_accessor_matches_emitted_value_for_every_variant` fixture, the stub listener test) uses `agent_kind`.
- **Downstream consequence**: any P4+ emit site (`agents/create.rs`, `agents/update.rs` for the archived-flip path, page 14 session launch) must use `agent_kind:` when constructing this variant. The serde wire key is `agent_kind` (not `kind`) — any external consumer deserialising `DomainEvent` JSON should expect that field name.
- **Audit-event field stays `kind`**: the audit event shape (`audit/events/m4/agents.rs`) is orthogonal and unchanged — the rename is scoped to `DomainEvent::AgentCreated` only.
- **Documented at**: [`modules/crates/domain/src/events/mod.rs`](../../../../modules/crates/domain/src/events/mod.rs) variant doc-comment + [event-bus-m5-extensions.md §"Field-naming note"](../../v0/implementation/m5/architecture/event-bus-m5-extensions.md).

### D3.2 — Listener wiring lives in `state::build_event_bus_with_m5_listeners` (free function), not `AppState::new`

- **P0 plan said** (§Part 4 P3 Deliverables + §Part 2 C11 + §C11 verification row): *"`AppState::new` wires all 5 listeners (M4's Template A + the 4 new). `server/src/state.rs::tests::handler_count_at_m5` asserts the count."*
- **Reality at P3**: `AppState` has no `::new()` constructor — it's been a plain `#[derive(Clone)] pub struct` with a struct-literal construction site in `main.rs` since M4. Adding a `new()` would bundle 5 `Arc<...>` parameters and three resolver trait objects into a single constructor, which is a strict subset of what `main.rs` already expresses more legibly. The test-visibility goal (letting `handler_count_is_five_at_m5` assert wiring) is served by a free helper function instead.
- **What shipped**: `pub fn build_event_bus_with_m5_listeners(repo, audit) -> Arc<InProcessEventBus>` in `server::state`. Both `main.rs` and `state::tests::handler_count_is_five_at_m5` call this helper; the test asserts `bus.handler_count() == 5` after return. Test name matches plan's `handler_count_is_five_at_m5` verbatim.
- **Downstream consequence**: any future phase that adds a listener (M5/P4's session-launch listener, M5/P8's real memory/catalog bodies, M6+) extends the helper — NOT a hypothetical `AppState::new`. The helper is the single wiring site.
- **Documented at**: [`modules/crates/server/src/state.rs`](../../../../modules/crates/server/src/state.rs) module-level doc + [event-bus-m5-extensions.md §"Listener registration"](../../v0/implementation/m5/architecture/event-bus-m5-extensions.md).

### D3.3 — `BabyPhiSessionRecorder` composes phi-core's recorder via `Arc<Mutex<_>>` (not a plain `inner: SessionRecorder`)

- **P0 plan said** (§Part 4 P3 code block): `pub struct BabyPhiSessionRecorder { inner: phi_core::SessionRecorder, ... }`.
- **Reality at P3**: `phi_core::SessionRecorder::on_event(&mut self, event: AgentEvent)` takes `&mut self`. For the recorder to be `Send + Sync` + shared across tasks (a P4+ requirement when the recorder is stored in `AppState::session_registry`), interior mutability is required. `Arc<Mutex<PhiCoreSessionRecorder>>` is the smallest available tool.
- **What shipped**: `inner: Arc<Mutex<PhiCoreSessionRecorder>>`. The wrap's `on_phi_core_event` takes `&self` (not `&mut self`), routes through `self.inner.lock()`, and releases the mutex guard BEFORE any `.await` (guarded by clippy `await_holding_lock`). `started_emitted: Arc<Mutex<bool>>` uses the same pattern to dedupe `SessionStarted` emission.
- **ADR-0029 already mentioned this** (§Consequences: *"BabyPhiSessionRecorder must be Send + Sync + 'static … Mitigated by Arc<Mutex<_>> on the phi-core recorder"*), so this is soft-drift — the ADR was correct, the P3 code block in Part 4 was the looser sketch. Recording it here so P4 launch-chain authors don't re-sketch the `&mut self` shape.
- **Downstream consequence**: P4's launch chain constructs `Arc<BabyPhiSessionRecorder>` (not `&mut recorder`) and hands clones into the `tokio::spawn`-ed task that drives `phi_core::agent_loop`. Per-session contention is bounded — one Mutex per live session, no cross-session lock.
- **Documented at**: [`modules/crates/domain/src/session_recorder.rs`](../../../../modules/crates/domain/src/session_recorder.rs) + [ADR-0029 §D29.2 + §Consequences](../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md).

### D3.4 — Template C adoption-AR resolver is org-scoped; Template D is project-scoped (asymmetry pinned)

- **P0 plan said** (§Part 4 P3 Deliverables): *"4 new listeners in `domain/src/events/listeners.rs` — `TemplateCFireListener` subscribes to `ManagesEdgeCreated`, calls `fire_grant_on_manages_edge`, persists, emits audit. `TemplateDFireListener` subscribes to `HasAgentSupervisorEdgeCreated`, calls Template D pure-fn."* The resolver trait scoping (per-org vs per-project) was not explicitly called out.
- **Reality at P3**: Template C's trigger is `ManagesEdgeCreated { org_id, manager, subordinate, ... }` — the org is in-band. Template D's trigger is `HasAgentSupervisorEdgeCreated { project_id, supervisor, supervisee, ... }` — the org isn't in the event; the listener walks `project → belongs_to → org` to find the Template-D adoption AR.
- **What shipped**: two distinct resolver traits — `TemplateCAdoptionArResolver::resolve(OrgId) -> Option<AuthRequestId>` + `TemplateDAdoptionArResolver::resolve(ProjectId) -> Option<(OrgId, AuthRequestId)>`. The D trait mirrors M4's `AdoptionArResolver` exactly, just filtering for `TemplateKind::D`. Implementations live in `server::platform::projects::resolvers` next to `RepoAdoptionArResolver`.
- **Downstream consequence**: P5's page 12 (authority-template adoption) and P6's page 13 (system-agent config) need to choose the right resolver trait when they add new per-template resolvers — don't duplicate the M4/A scoping onto the C trait (org-direct lookup is simpler and faster for org-level triggers).
- **Documented at**: [`modules/crates/server/src/platform/projects/resolvers.rs`](../../../../modules/crates/server/src/platform/projects/resolvers.rs) module-level doc block.

### P3 drift addendum — coherence checklist

- [x] Every drift item documents *what P0 planned*, *what actually shipped*, and *where the correction is now visible in code + ADR + architecture doc*.
- [x] Every drift item has a code comment or doc reference so future reviewers can find the correction without re-reading the whole plan.
- [x] `check-doc-links` / `check-phi-core-reuse` / `check-ops-doc-headers` / `check-spec-drift` all green after the P3 drift text landed.
- [x] P3 composite confidence floor held at 98%+ (reported below).

**P4 implementers**: read the Pre-P4 gate (below). All P1-P3 drift items (D1.1 through D3.4) carry forward as load-bearing invariants.

## P4 drift addenda  `[STATUS: captured at P4 close, 2026-04-23]`

### D4.1 — Permission Check is advisory-only at M5 (launch blocks on Step 0 Catalogue only)

- **P0 plan said** (§Part 4 P4 §launch_session Step 3): *"Run Permission Check steps 0–6."* The implication — based on M1 engine semantics — is that a `Decision::Denied` outcome at ANY step refuses the launch with `PERMISSION_CHECK_FAILED_AT_STEP_N` (403).
- **Reality at P4**: the synthetic launch manifest declares `actions=["launch_session"]` + `resource=["session"]`. No grant in the baby-phi governance tier at M5 covers this reach (Template A mints `[read, inspect, list]` on `project:<id>`, not `launch_session` on `session`). Strictly gating every launch on the Decision would reject every M5 launch including lead agents into their own projects. The per-action manifest catalogue that'd make step-3/4/5/6 meaningfully gate doesn't ship until M6+.
- **What shipped**: Step 0 (Catalogue) still gates (a preview catalogue-miss returns 403 `PERMISSION_CHECK_FAILED`). Steps 1–6 surface the full `Decision` on the `LaunchReceipt` + the `/sessions/preview` endpoint but do NOT refuse the launch. Logged at `tracing::info` level with `"sessions::launch: Permission Check denied (advisory at M5; not blocking)"`.
- **Downstream consequence**: the M6+ phase that ships real launch-policy manifests (agent-catalogue grant wiring is the likely trigger) removes the advisory clause + lets all 7 steps gate. The wire code `PERMISSION_CHECK_FAILED_AT_STEP_N` is stable — it just becomes reachable for steps ≥ 1.
- **Documented at**: [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) inline comment at the Step 3 block, [session-launch.md](../../v0/implementation/m5/architecture/session-launch.md) §9-step launch flow item 3, and [session-launch-operations.md](../../v0/implementation/m5/operations/session-launch-operations.md) §Error-code reference row for `PERMISSION_CHECK_FAILED`.

### D4.2 — Launch spawns a synthetic event feeder, not `phi_core::agent_loop()`

- **P0 plan said** (§Part 4 P4 Step 7): *"Spawn `tokio::task` running `phi_core::agent_loop(prompts, ctx, cfg, tx, cancel_token)`. The event channel feeds `BabyPhiSessionRecorder`."*
- **Reality at P4**: baby-phi does not yet have (a) a binding path for phi-core provider credentials into `ModelConfig.api_key` at launch time (the vault splice handler is scaffolded but untested with live providers), nor (b) concrete `Box<dyn AgentTool>` implementations to hand `agent_loop` as its `tools` arg. Calling `agent_loop` with an empty tools vec + a placeholder key would either make a live LLM call (cost + test flakiness) or fail at the first turn.
- **What shipped**: `spawn_replay_task` feeds a canned `AgentStart` → `TurnStart` → `TurnEnd` → `AgentEnd` sequence through `BabyPhiSessionRecorder`. This proves the full compound-tx + recorder + governance-event chain end-to-end without a live provider. The `use phi_core::{agent_loop, agent_loop_continue}` imports stay as compile-time witnesses (pinned by `_keep_agent_loop_live`) so a phi-core rename breaks the baby-phi build immediately. M7+ swaps the feeder body for the real `agent_loop(...)` call without changing the outer function signature or the receipt shape.
- **Downstream consequence**: acceptance tests assert Session + LoopRecord + Turn persistence end-to-end from the HTTP layer, but they do NOT prove phi-core's runtime loop is wired correctly — that lands at M7+ when the provider-credential splice + tool impls land. The plan's §P4 Deliverable §launch_session text reads "Spawn `tokio::task` running `phi_core::agent_loop`"; this is not literally true today.
- **Documented at**: [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) module doc §Replay task + [`session-launch.md`](../../v0/implementation/m5/architecture/session-launch.md) §Replay task.

### D4.3 — `resolve_agent_tools` returns `Vec<ToolSummary>`, not `Vec<Box<dyn AgentTool>>`

- **P0 plan said** (§Part 4 P4 §tools.rs): *"`resolve_agent_tools(repo, agent_id) -> Vec<Box<dyn AgentTool>>` + `GET /sessions/:id/tools` handler."* — signature implies phi-core trait objects.
- **Reality at P4**: no concrete `phi_core::AgentTool` impl exists in baby-phi yet — the `ToolDefinition` graph node is a scaffold (id-only, per `nodes.rs::scaffold_node!`). Returning `Vec<Box<dyn AgentTool>>` would always be empty AND force every caller to know phi-core's trait object interface for a collection that's always empty.
- **What shipped**: `resolve_agent_tools(repo, agent_id) -> Vec<ToolSummary>` where `ToolSummary = { name, label, description, parameters_schema }` is the HTTP projection. The return is currently empty for every agent (M5 ships the wire shape + the resolver's call site; no `HAS_TOOL` edges are written yet). `use phi_core::types::tool::AgentTool` is kept in `tools.rs` as a compile-time witness via `_is_phi_core_agent_tool_trait<T: AgentTool + ?Sized>` so phi-core renames break the build.
- **Downstream consequence**: M7+ (when real tools land) either: (a) swap `Vec<ToolSummary>` for `Vec<Box<dyn AgentTool>>` in the resolver's return + map at the HTTP boundary (same wire shape, different internal), OR (b) keep `ToolSummary` + introduce a parallel `resolve_agent_tool_impls` that returns trait objects for the launch chain's `agent_loop` call. The call-site refactor is bounded (one file).
- **Documented at**: [`modules/crates/server/src/platform/sessions/tools.rs`](../../../../modules/crates/server/src/platform/sessions/tools.rs) module doc §Scope + `ToolSummary` doc comment.

### D4.4 — `upsert_agent_profile` is no-op for fresh agent rows; update chain now uses `create_agent_profile` when prior profile is absent

- **P0 plan said** (§Part 4 P4 §agents/update.rs flip): *"Persist."* — implied that the existing `upsert_agent_profile` repo method handles both create + update.
- **Reality at P4**: the SurrealDB `upsert_agent_profile` impl uses `UPDATE type::thing('agent_profile', $id) CONTENT $body`. In SurrealDB, `UPDATE` against a non-existent thing is a silent no-op (it does NOT auto-create the row). This is the same family of issues as P2 drift D2.1 (CREATE vs UPDATE for session/loop/turn). When the M5/P4 C-M5-5 change arm creates a new `AgentProfile` row because the agent had no prior profile, `upsert` silently dropped the write.
- **What shipped**: `agents/update.rs` branches on `current_profile.is_some()` → use `upsert_agent_profile` when a prior row exists, `create_agent_profile` otherwise. Comment inline pins the D4.4 rationale. The `upsert_agent_profile` trait method is unchanged — callers with a prior row still use it; the fresh-row branch is now explicit.
- **Downstream consequence**: P5's page 12 (authority templates) + P6's page 13 (system agents) adopt the same branch when they need to write profile rows for agents that may or may not have one. A future repo-level fix could make `upsert_agent_profile` actually upsert (SurrealDB `UPSERT` keyword) — that's a separate change tracked on the P2 D2.1 follow-up list.
- **Documented at**: [`modules/crates/server/src/platform/agents/update.rs`](../../../../modules/crates/server/src/platform/agents/update.rs) inline comment at the `profile_changed` branch.

### D4.5 — Repo-trait `write_uses_model_edge` added as first-class method (not a generic `create_edge`)

- **P0 plan said** (§Part 4 P4 §launch_session Step 6): *"Compound tx: persist Session + first LoopRecord + `runs_session` edge + `uses_model` edge"* — implied a generic edge-writer path.
- **Reality at P4**: baby-phi's Repository trait has typed edge-writers for each edge family (`create_grant` mints grant edges; `apply_project_creation` bundles `HAS_LEAD`). There was no generic `create_edge(&Edge)` method. Adding one would have meant extending the trait's surface with a type-erased enum + pushing the SurrealDB impl to pattern-match on every variant — a wider trait-surface change than the P4 scope needed.
- **What shipped**: `Repository::write_uses_model_edge(agent, model_runtime) -> EdgeId` is a dedicated typed method. Both impls (in-memory + SurrealDB) override it; the default trait body returns `Backend(...)` so future impls fail fast until they implement it. SurrealDB uses the LET-first RELATE pattern (D2.2).
- **Downstream consequence**: every future edge that's first-written from a higher tier (Template C's `MANAGES`, Template D's `HAS_AGENT_SUPERVISOR`) gets its own typed writer method. The Repository trait grows by one method per new-edge-writer — slightly more verbose than a generic `create_edge` but preserves type safety at the trait boundary.
- **Documented at**: [`modules/crates/domain/src/repository.rs`](../../../../modules/crates/domain/src/repository.rs) `write_uses_model_edge` doc comment.

### D4.6 — `SessionLaunchContext` gains `first_loop_id: Option<LoopId>` to avoid double-persist

- **P0 plan said** (§Part 4 P4 §launch_session Step 6): *"persist Session + first LoopRecordNode"*. And (§P3 §BabyPhiSessionRecorder): recorder's `finalise_and_persist` reads phi-core's materialised view + writes it to SurrealDB. The plan did not clarify what happens when BOTH the launch chain and the recorder try to write the session + first loop — a potential double-persist.
- **Reality at P4**: when the launch chain calls `persist_session(session, first_loop)` synchronously BEFORE spawning the replay task, the session row exists in SurrealDB with its UNIQUE id. When the recorder's `finalise_and_persist` then calls `persist_session` again with a FRESH `Session` wrap, the CREATE path hits the UNIQUE constraint and fails. Without a fix, every launch chain call produces an "already running" session that never finalises. Observed in acceptance tests before the fix — `wait_for_session_finalised` timed out at 2000ms because the recorder silently errored.
- **What shipped**: `SessionLaunchContext.first_loop_id: Option<LoopId>` is a new field. When `Some`, the recorder's `finalise_and_persist` switches to an append-only path (add loops beyond index 0 + add all turns + call `mark_session_ended` to flip governance state) — no re-persist of the session row. When `None` (standalone P3 recorder-wrap test), the recorder uses the original full-persist path. Single `match` on the field at the top of the persist block.
- **Downstream consequence**: any P5+ caller of `BabyPhiSessionRecorder` needs to pass `first_loop_id: Some(...)` if the Session row is pre-persisted, or `None` otherwise. The P3 standalone tests continue to work unchanged.
- **Documented at**: [`modules/crates/domain/src/session_recorder.rs`](../../../../modules/crates/domain/src/session_recorder.rs) `SessionLaunchContext.first_loop_id` doc comment.

### P4 drift addendum — coherence checklist

- [x] Every drift item documents *what P0 planned*, *what actually shipped*, and *where the correction is now visible in code + ADR + architecture doc*.
- [x] Every drift item has a code comment or doc reference so future reviewers can find the correction without re-reading the whole plan.
- [x] `check-doc-links` / `check-phi-core-reuse` / `check-ops-doc-headers` / `check-spec-drift` all green after the P4 drift text landed.
- [x] P4 composite confidence floor held at 96%+ (6 drift items; 5 carryovers closed; 1 advisory-only gate D4.1 is the largest scope reduction).

**P5 implementers**: read the Pre-P5 gate (below). All P1-P4 drift items (D1.1 through D4.6) carry forward. D4.1 (Permission Check advisory) is the one with the biggest semantic impact — P5's authority-template handlers MUST treat the preview Decision as reference information, not as a hard gate; the soft-gate policy stays in effect until the per-action manifest catalogue lands (M6+).

## P5 drift addenda  `[STATUS: captured at P5 close, 2026-04-23]`

### D5.1 — CLI + Web surfaces deferred from P5 to P7

- **P0 plan said** (§Part 4 P5 §Goals + §Deliverables items 3 & 4): *"Operators can approve / deny / adopt-inline / revoke-cascade authority templates via page 12 (**HTTP + CLI + Web**)."* Item 3: `phi template {list, approve, deny, adopt, revoke}` CLI. Item 4: Next.js web page at `/organizations/[id]/templates/`.
- **Reality at P5**: only the HTTP surface shipped. The CLI + Web surfaces deferred to P7 alongside the already-planned `phi session` CLI + web polish. Rationale: (a) P7 is the milestone's dedicated CLI + web polish phase; shipping both P5's and P7's CLI/web work in one phase makes the completion-regression test + e2e wiring cleaner, (b) HTTP is the authoritative surface — the CLI + Web are thin HTTP shims that compose without restructuring the P5 business logic, (c) user explicitly asked for "one single confidence check for P4" at the previous phase and the same scoping discipline applies to P5.
- **What shipped**: 7 files under `server/src/platform/templates/` (mod + list + approve + deny + adopt + revoke + audit_events) + `server/src/handlers/templates.rs` with 5 routes + 7 acceptance tests in `acceptance_authority_templates.rs` proving HTTP-level correctness including the revoke-cascade (grant_count_revoked).
- **Downstream consequence**: P7's §Deliverables implicitly grows. P7's existing plan body already includes "CLI + web polish" as its goal; authority-template CLI / web falls under that umbrella. The P7 gate at open must explicitly confirm this extension.
- **Documented at**: this addendum + Pre-P7 gate (will be populated at P6 close with P5 CLI/web explicitly called out) + the session-launch operations doc's existing "CLI coming in P7" pattern extends unchanged.

### D5.2 — Audit events ship as `server::platform::templates::audit_events` (not `domain::audit::events::m5::templates`)

- **P0 plan said** (§Part 4 P5 §Deliverables): by omission; §Part 6 docs tree §decisions lists no ADR for audit-event placement, and earlier phases (M4's Template A fire event) live under `domain::audit::events::m4::templates::template_a_grant_fired`. Convention implied that page 12's audit events would also live in `domain::audit::events::m5::templates` (the pre-existing M5/P3 module for Template C/D fire events).
- **Reality at P5**: page 12's audit events (`template.adopted` / `adoption_denied` / `revoked`) are platform-tier events emitted by server handlers, not domain-tier events emitted by fire listeners. Keeping them in the `domain::audit::events::m5::templates` module would mix governance-plane handler events with fire-listener events that have different provenance (adoption AR vs grant node id). Splitting is cleaner; a future refactor could unify if the event shapes converge, but at M5 they don't.
- **What shipped**: `server::platform::templates::audit_events` module (4 builder fns + 4 unit tests) alongside the P5 business logic. Event types are stable + match R-ADMIN-12-N1/N2/N3 verbatim (`platform.template.adopted` / `adoption_denied` / `revoked`).
- **Downstream consequence**: M5/P8 (memory extraction) will likely ship its own audit-event module under `server::platform::` too rather than `domain::audit::events::m5::`. Convention established here: domain-tier events (fire listeners, state machine transitions) live in `domain::audit::events::mX::*`; platform-tier events (HTTP handler success / failure) live in `server::platform::<page>::audit_events`.
- **Documented at**: [`modules/crates/server/src/platform/templates/audit_events.rs`](../../../../modules/crates/server/src/platform/templates/audit_events.rs) module doc; cross-referenced from the ops doc.

### D5.3 — `find_adoption_ar` returns the **most-recent** adoption AR, not a "unique" one

- **P0 plan said** (§Part 1 G1 + ADR-0030 D1 resolution): *"Template node uniqueness via UNIQUE(kind); adoption carried by AR `provenance_template`."* The implication in §Part 4 P5's approve/deny/revoke handlers is that there's a single adoption AR per (org, kind) at any time.
- **Reality at P5**: an org's adoption history for a kind can span multiple ARs over its lifetime — `adopt` creates AR#1, operator revokes → AR#1 Revoked, operator re-adopts → AR#2 Approved. The migration 0005 UNIQUE(kind) is on the **Template** node (platform-level), not on the adoption AR (per-org, per-adoption-attempt). So `find_adoption_ar(org, kind)` must disambiguate.
- **What shipped**: `find_adoption_ar` returns the AR with the largest `submitted_at` (most recent). This is correct for the page 12 semantics — the "current" adoption state of a kind is the most recent AR's state. Older Revoked ARs stay in the `revoked` bucket for audit trail but don't block new adoptions.
- **Downstream consequence**: M7b + M8 ops playbooks that walk adoption history across an org's lifetime need to call `Repository::list_adoption_auth_requests_for_org(org)` + filter by kind tag directly (not via `find_adoption_ar`, which collapses to one row). The existing ops doc's "revoke stalled mid-cascade" playbook doesn't touch this distinction because cascade targets a specific AR id, not a kind.
- **Documented at**: [`modules/crates/server/src/platform/templates/mod.rs`](../../../../modules/crates/server/src/platform/templates/mod.rs) `find_adoption_ar` doc comment.

### P5 drift addendum — coherence checklist

- [x] Every drift item names *what P0 planned*, *what shipped*, and *where the correction is now visible in code + doc*.
- [x] `check-doc-links` / `check-phi-core-reuse` / `check-ops-doc-headers` / `check-spec-drift` all green after the P5 docs landed.
- [x] P5 composite confidence floor held at 97%+ (3 drift items: D5.1 material scope reduction with a future-phase landing path; D5.2 + D5.3 implementation-hygiene decisions with forward-compatible signatures).

**P6 implementers**: read the Pre-P6 gate (below). All P1-P5 drift items (D1.1 through D5.3) carry forward. D4.1 (Permission Check advisory) + D5.1 (CLI / web deferred to P7) are the two biggest carry-forwards for the M5 remainder.

## P6 drift addenda  `[STATUS: captured at P6 close, 2026-04-23]`

### D6.1 — Listener callback extension ships the helper, not the call sites

- **P0 plan said** (§Part 4 P6 §Deliverables item 3): *"all 5 listeners (Template A/C/D + memory-extraction + agent-catalog) call `repo.upsert_system_agent_runtime_status(agent_id, queue_depth, last_fired_at, ...)` on each fire. Shared helper in `domain/src/events/listeners.rs`."*
- **Reality at P6**: two of the five listeners (Template A/C/D fires) **target grants, not system agents** — there's no "which system agent am I" identity on those fire paths; they write governance grants issued from adoption ARs. Wiring `record_system_agent_fire` into them would require inventing a mapping ("Template A fire on this HAS_LEAD edge updates which system agent's runtime-status?"). The mapping is unclear because Template fire listeners aren't system agents in any sense. The remaining two (memory-extraction + agent-catalog) ARE system agents' fire listeners — but their bodies are **stubs at M5/P3** with the full implementation landing at **M5/P8**. At M5/P3 close those stubs only log; they have no system-agent identity resolver.
- **What shipped**: the shared helper [`domain::events::listeners::record_system_agent_fire`](../../../../modules/crates/domain/src/events/listeners.rs) is present + exported + documented. Call sites land at **M5/P8** when the memory-extraction + agent-catalog listener bodies are wired against a real extractor + upserter. At M5/P6 no listener calls the helper — the runtime-status table is empty for fresh orgs, which is semantically correct (no fires have occurred yet).
- **Downstream consequence**: M5/P8 implementers connect the helper at the top of the memory-extraction + agent-catalog listener body entries (after they resolve which system agent's row to update). The `SystemAgentRuntimeStatus.queue_depth` field stays at `0` throughout M5 (the helper seeds it) — real queue-depth tracking is M7b observability work.
- **Documented at**: [`domain::events::listeners::record_system_agent_fire`](../../../../modules/crates/domain/src/events/listeners.rs) module doc, [`system-agents.md`](../../v0/implementation/m5/architecture/system-agents.md) §SystemAgentRuntimeStatus, [`system-agents-operations.md`](../../v0/implementation/m5/operations/system-agents-operations.md) §"Listener upsert stale" playbook.

### D6.2 — CLI + Web for page 13 deferred to P7 (matching D5.1 precedent)

- **P0 plan said** (§Part 4 P6 §Goals): *"Operators tune / add / disable / archive system agents via page 13. Live queue-depth + last-fired-at from `SystemAgentRuntimeStatus`."* §Deliverables items 4 + 5: `phi system-agent {list, tune, add, disable, archive}` CLI + Next.js web page.
- **Reality at P6**: HTTP surface only, matching the D5.1 precedent (page 12's CLI + Web also deferred to P7). Rationale unchanged: P7 is the CLI + web polish phase; batching P5 + P6 CLI/web + `phi session` at P7 keeps e2e wiring + completion-regression cleanly in one place; user's "one single confidence check" phase-cadence preference applies.
- **What shipped**: 7 files under `server/src/platform/system_agents/` (mod + list + tune + add + disable + archive + audit_events); 5 HTTP routes; 8 acceptance scenarios in `acceptance_system_agents.rs`.
- **Downstream consequence**: P7's §Deliverables grows by two page-13 surfaces (CLI + Web) in addition to P5's (page 12 CLI + Web) + the originally-planned `phi session` CLI + web polish. The P7 gate at open must explicitly confirm both extensions.
- **Documented at**: this addendum + Pre-P7 gate.

### D6.3 — "Standard" vs "org-specific" bucketing needs a union filter because fixture system agents pre-date canonical slugs

- **P0 plan said** (§Part 4 P6 §Deliverables: system-agent list endpoint): bucketing by profile_ref slug. Implied: the two standard system agents minted at M3 org-creation use the canonical slugs `"system-memory-extraction"` and `"system-agent-catalog"`.
- **Reality at P6**: the acceptance-test fixture (`acceptance_common::admin::spawn_claimed_with_org`) provisions two system agents with M3-era profile slugs that don't match the canonical M5 standard-slug constants (the fixture predates the canonical slugs). A strict slug-match filter would leave both fixture "standard" agents in the `org_specific` bucket — semantically wrong. The [`list.rs`](../../../../modules/crates/server/src/platform/system_agents/list.rs) filter was widened to accept **either** the canonical slug on `AgentProfile.blueprint.config_id` **or** membership in `Organization.system_agents` (the composite-owned registry) **or** `AgentRole::System` on the agent row. Three-way union is robust across fixture + production shapes.
- **What shipped**: the union filter is live + documented + covered by the list acceptance scenario. `STANDARD_SYSTEM_AGENT_PROFILES` constant in `system_agents::mod` still holds the canonical two slugs; production system agents created via `add_system_agent` use whatever `profile_ref` the operator supplies (no enforcement that standard slugs are reserved — intentionally).
- **Downstream consequence**: M6+ authors of new system agents should pick non-colliding profile_refs. M7b-era hardening could reserve the standard slugs at the repo level if operators start reusing them by accident.
- **Documented at**: [`list.rs`](../../../../modules/crates/server/src/platform/system_agents/list.rs) inline comment on the union filter.

### D6.4 — Audit events use `server::platform::system_agents::audit_events` (matching D5.2 convention)

- **P0 plan said** (§Part 4 P6 §Deliverables): no explicit event-module placement, but M5/P3 earlier template-fire events live in `domain::audit::events::m5::templates` — implying M5/P6's system-agent events would too.
- **Reality at P6**: following the D5.2 precedent. Platform-handler audit events (page-13 reconfigure / add / disable / archive) live at `server::platform::system_agents::audit_events`. Domain-tier events (fire listener outcomes) stay at `domain::audit::events::mX::*`.
- **What shipped**: `server::platform::system_agents::audit_events` module with 4 builder fns + 4 unit tests. Event types: `platform.system_agent.reconfigured` / `.added` / `.disabled` / `.archived`.
- **Documented at**: [`audit_events.rs`](../../../../modules/crates/server/src/platform/system_agents/audit_events.rs) module doc.

### D6.5 — `disable` + `archive` at M5 don't flip a durable `active:false` field

- **P0 plan said** (§Part 4 P6 + R-ADMIN-13-W3/W4): disable pauses the trigger subscriber + "marks the agent `active: false`"; archive "graph archival".
- **Reality at P6**: the `Agent` node at M5 has no `active` field + no `archived_at` field. Flipping those would require migration 0006 to add columns + backward-compatible defaults. At M5/P6 the handlers emit the audit events + log the action but don't mutate the agent's durable state. The trigger-subscriber pause path is tied to the M5/P8 listener bodies that don't exist yet.
- **What shipped**: audit + gate (archive refuses on standards) + the error surface that a future phase can point at. The operator sees a 200 response + the audit entry — the downstream effect (actual trigger pause) lands when P8 bodies + M7b observability do.
- **Downstream consequence**: M6/M7 phases that add `Agent.active: bool` + `Agent.archived_at: Option<DateTime<Utc>>` via migration 0006 plumb the flip into `disable_system_agent` + `archive_system_agent`. The HTTP wire contract stays stable; only the body implementation grows.
- **Documented at**: [`disable.rs`](../../../../modules/crates/server/src/platform/system_agents/disable.rs) module doc note + [`archive.rs`](../../../../modules/crates/server/src/platform/system_agents/archive.rs) inline comment.

### P6 drift addendum — coherence checklist

- [x] Every drift item names *what P0 planned*, *what shipped*, and *where the correction is visible*.
- [x] `check-doc-links` / `check-phi-core-reuse` / `check-ops-doc-headers` / `check-spec-drift` all green after P6 docs landed.
- [x] P6 composite confidence floor held at 96%+ (5 drift items: D6.1 + D6.5 are the two material deferrals tied to M5/P8 + M6+ landing; D6.2 carries the D5.1 deferral forward; D6.3 + D6.4 are hygiene matching D5.2/D5.3 precedents).

**P7 implementers**: Pre-P7 gate below. All P1-P6 drift items (D1.1 through D6.5) carry forward. P7 is the CLI + web polish phase — the biggest deferred scope from D5.1 + D6.2 consolidates here.

## P7 drift addenda  `[STATUS: captured at P7 close, 2026-04-24]`

### D7.1 — Live SSE tail deferred to M7 (ADR-0031 D4 path-(a) reclassified)

- **P0 plan said** (§Part 4 P7 §Deliverables item 1 + D4 decision): *"`phi session launch` — default tails events live via SSE to `GET /api/v0/sessions/:id/events`; `--detach` returns `session_id` + first `loop_id`."* §Part 1.5 Q1 prediction: *"`phi_core::types::event::AgentEvent` may be imported in `cli/src/commands/session.rs` for SSE tail rendering (≤1 new import line)."*
- **Reality at P7**: no SSE endpoint exists. `sessions/launch.rs` ships a **synthetic replay feeder** (documented in D4.2) that writes `{AgentStart, TurnEnd, AgentEnd}` inline before returning the receipt — the real `phi_core::agent_loop` invocation is deferred to M7+ when a `StreamProvider` harness is wired. Without a live agent loop there is nothing to stream; adding an `/events` SSE endpoint would stream against the already-terminal recorder output which has no operational value. The `--detach` flag is preserved on the clap surface for wire stability, but at M5/P7 every launch is effectively detached.
- **What shipped**: 5-subcommand clap tree (`launch`, `show`, `terminate`, `list`, `preview`) all wired against existing HTTP routes; `launch` prints a human receipt + notes "live tail deferred to M7" in both the help text and the on-success output. CLI's phi-core import count stays at 1 (the pre-existing `AgentEvent` reference in `agent.rs` for the `demo` subcommand) — **zero P7 additions** vs the `≤1` prediction.
- **Downstream consequence**: M7's agent-loop integration lands the SSE endpoint + flips `launch_impl` to stream by default + adds the `AgentEvent` import to `session.rs`. At M5 the CLI still gives operators a complete launch → inspect → terminate flow via synchronous HTTP.
- **Documented at**: [`session.rs`](../../../../modules/crates/cli/src/commands/session.rs) module doc §phi-core leverage note.

### D7.2 — `--model-config-id` ships as a convenience flag alongside `--patch-json` (not replacing it)

- **P0 plan said** (§Part 4 P7 §Deliverables item 2): *"`cli/src/commands/agent.rs` — extend `phi agent update` with `--model-config-id` flag (C-M5-5 wire)."*
- **Reality at P7**: `--patch-json` at M4 was mandatory (the only way to drive the diff-producing update endpoint). Removing it in favour of only the new narrow flag would regress M4 operators who rely on the general-purpose patch. Instead the flag is additive: `--patch-json` stays, `--model-config-id` is a convenience that expands to `{"model_config_id": "<id>"}`. Exactly one of the two must be supplied.
- **What shipped**: clap accepts both as `Option<String>`. Mutual-exclusion + required-one enforced at `update_impl`'s top with a clear error message. Completion regression (`completion_help::completion_scripts_expose_m5_p7_agent_update_model_config_id_flag`) pins both flags as surface-level invariants.
- **Downstream consequence**: none. Both flags reach the same HTTP wire (`PATCH /api/v0/agents/:id/profile` with `UpdateAgentProfileBody`). C-M5-5's active-session 409 gate fires correctly from either entry point.
- **Documented at**: [`main.rs`](../../../../modules/crates/cli/src/main.rs) `Update` variant doc comment.

### D7.3 — `phi session preview` ships as a 5th subcommand

- **P0 plan said** (§Part 4 P7 §Deliverables): 4 session subcommands (`launch` / `show` / `terminate` / `list`).
- **Reality at P7**: added a 5th — `phi session preview --org-id --project-id --agent-id` — that wraps `POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview` (the D5 Permission-Check preview endpoint). The route was built at P4 but had no CLI surface; shipping a preview subcommand is a cheap operator win that avoids requiring a full launch just to inspect the decision trace.
- **What shipped**: `SessionCommand::Preview` variant + `preview_impl` fn + regression test `completion_session_subcommand_includes_preview`. The HTTP body matches `POST /sessions/preview` verbatim — no server-side change.
- **Downstream consequence**: neutral. P8/P9 acceptance scaffolding may use `phi session preview` in scripted fixtures instead of hand-crafting `curl` calls.
- **Documented at**: [`session.rs`](../../../../modules/crates/cli/src/commands/session.rs) `Preview` clap variant doc.

### D7.4 — Page 11 "Recent sessions" retrofit is web-side (no server-detail mutation)

- **P0 plan said** (§Part 4 P7 §Deliverables item 4): *"Page 11's \"Recent sessions\" panel (M4 placeholder → real rows from `list_sessions_in_project`)."*
- **Reality at P7**: two paths existed — (a) mutate the server's `ProjectDetail.recent_sessions: Vec<RecentSessionStub>` to populate real rows (requires repo + detail-handler changes + schema-snapshot churn), or (b) keep the server wire stable and fetch `/api/v0/projects/:id/sessions` in parallel from the Next.js page component. Chose (b) to keep the M4 wire contract frozen and minimize blast radius; the result on the UI is identical — the panel now shows real session rows with `id · governance_state · started_at · ended_at` or a "launch a session →" CTA when empty.
- **What shipped**: [`modules/web/app/(admin)/organizations/[id]/projects/[project_id]/page.tsx`](../../../../modules/web/app/(admin)/organizations/[id]/projects/[project_id]/page.tsx) calls `listSessionsInProjectApi` alongside the detail fetch + renders the new rows below the roster block. The server-side `ProjectDetail.recent_sessions` field stays `Vec::new()` at M5 (intentional, per `detail.rs:229`). A future phase that promotes the detail wire to carry real recent sessions can strip the web-side fetch without breaking the page.
- **Downstream consequence**: M7's project-detail hardening may elect to flip path (a) and remove the web-side extra fetch. No regression risk — both paths render the same shape.
- **Documented at**: page.tsx inline comment block on the retrofit + this addendum.

### D7.5 — Web test count unchanged (no new page-component tests at P7)

- **P0 plan said** (§Part 5 Testing strategy): "~20 Web tests added at M5." §Part 4 P5 + P7 §Tests added each mention component tests for the new pages.
- **Reality at P7**: the 3 new pages (templates, system-agents, sessions/new) are SSR Server Components with inline Server Actions. Component testing them requires mocking Next.js's server-action machinery + `cookies()` + `redirect()` flow — which exceeds the value of asserting JSX shape. Type-check + ESLint + `next build` all pass across the new routes. Web test count stays at **79** (same as P6 close; the +11 over M4's 68 landed at P1 from the primitives component tests).
- **Downstream consequence**: Playwright e2e at P9 is the right layer to cover the CLI → launch → web-render path end-to-end. If component tests are later required for the new pages, the pattern established by M4's `modules/web/__tests__/m5_primitives.test.tsx` (pure-render assertions on stateless sub-components) can extend naturally.
- **Documented at**: this addendum.

### D7.6 — Web pages use inline `async function run()` server actions (not separate `"use server"` module declarations)

- **P0 plan said** (§Part 6 Documentation plan): implicit convention from M2/M3/M4 web pages — Server Actions live in a sibling `actions.ts` file with `"use server"` at the top.
- **Reality at P7**: the three new pages hybrid the pattern. The **top-level action module** (`actions.ts`) still exists + exports `listTemplatesAction`, `approveTemplateAction`, etc. — these are the callable surface. But the **inline per-row `<form action={run}>`** handlers are nested closures inside the page component, marked `"use server"` at the function body. This is Next.js 14's officially-supported pattern for dynamic-id action dispatch (you can't pre-bind `orgId` and `kind` to a top-level action without contortions). The closure captures are strings only — serialization-safe.
- **What shipped**: every template / system-agent / session-launch page uses the hybrid. Top-level actions handle revalidation + error mapping; inline closures thread the per-row identifiers. All three pages build + lint + typecheck cleanly.
- **Downstream consequence**: future web-page authors should follow this pattern when per-row actions need identifiers captured from the SSR fetch. The top-level `actions.ts` is the canonical HTTP seam; inline closures are the UI seam.
- **Documented at**: [`templates/page.tsx`](../../../../modules/web/app/(admin)/organizations/[id]/templates/page.tsx) `ApproveForm` / `DenyForm` etc. ; [`system-agents/page.tsx`](../../../../modules/web/app/(admin)/organizations/[id]/system-agents/page.tsx) `TuneForm` / `DisableForm`; [`sessions/new/page.tsx`](../../../../modules/web/app/(admin)/organizations/[id]/projects/[project_id]/sessions/new/page.tsx) `previewSubmit` / `launchSubmit`.

### P7 drift addendum — coherence checklist

- [x] Every drift item names *what P0 planned*, *what shipped*, and *where the correction is visible in code + doc*.
- [x] `check-doc-links` / `check-phi-core-reuse` / `check-ops-doc-headers` / `check-spec-drift` all green after P7 code landed.
- [x] P7 composite confidence floor held at 96%+ (6 drift items: D7.1 is the one material deferral — live tail → M7; D7.2–D7.4 + D7.6 are implementation-shape hygiene; D7.5 is a test-strategy reclassification).
- [x] phi-core imports at P7 close = **26** (unchanged from P6; Part 1.5 prediction was `≤1` new, actual 0 due to D7.1).
- [x] ADR statuses unchanged (0029 / 0030 / 0031 all Accepted — no new ADRs at P7).

**P8 implementers**: Pre-P8 gate below. All P1-P7 drift items (D1.1 through D7.6) carry forward. P7 closes all CLI + Web scope; P8 is the listener-body phase (s02 memory-extraction + s03 agent-catalog + s05 Template C/D verification). The D6.1 helper (`record_system_agent_fire`) is the first thing P8 wires.

---

## Phase-open gates  `[STATUS: standing — grows as phases close]`

**Purpose.** The drift addenda catch one category of coherence loss — *P0 intent vs P<N> reality*. A second category is *decisions pinned at P0 pending confirmation at a specific later phase* (e.g. D3 `max_concurrent` default, D5 Permission Check preview location). Both categories must be consulted before a future phase opens or the plan body + code + decisions drift apart silently.

**Standing discipline.** At every phase close the addendum section is updated. At every phase **open** the gates below are walked, each entry confirmed before opening-phase work starts. Failure to confirm a gate item blocks the phase open — the reviewer must either accept the default, override it with a user-confirmed alternative (landing as a new drift addendum entry), or defer the gate to a later phase (landing as an updated Part 11 open-question entry).

Entries follow this shape:

- **Pre-P<N> reading list** — the addendum entries (D*.X) this phase inherits as invariants.
- **Pre-P<N> decisions to confirm** — Part 3 / Part 11 items marked "assumed default, confirm at P<N>". Each either flips to ✅ confirmed or lands as a ⚠ overridden decision in the addendum.
- **Carry-forward invariants** — any operational invariant the phase must honour (e.g. "populate `created_at` alongside `started_at`" from D1.1).

Future phase closes append their own future-phase gate blocks here as carry-forwards become apparent.

### Pre-P3 gates  `[STATUS: walked + closed at P3 open 2026-04-23; all 3 carry-forward invariants held at P3 close — see P3 drift addenda above]`

- **Reading list (5 addendum items, all P3-relevant)**:
  - **D1.1** — `session` + `turn` table augmentation. The P3 recorder writes Session/Turn rows; both tables inherit the mandatory `created_at: string` column from 0001. Recorder persist hooks MUST populate `created_at` alongside `started_at` (for turns: alongside `inner.started_at`).
  - **D1.2** — `runs_session` retype. The recorder does NOT need to write new `runs_session` edges at P3 (P2's `persist_session` already writes them). P3 just reads; no new invariant introduced.
  - **D1.3** — Nested `inner` (not `#[serde(flatten)]`). The `BabyPhiSessionRecorder` emits `phi_core::Session` / `LoopRecord` / `Turn` values that land inside baby-phi's wrap via the nested `inner` field. Serde round-trips are the invariant.
  - **D2.1** — CREATE (not UPDATE) for Session/Loop/Turn writes. The recorder's persist hooks use the P2-shipped `persist_session` / `append_loop_record` / `append_turn` repo methods (which already use CREATE). If P3 adds new direct SurrealDB writes, they MUST use CREATE for fresh-id rows, DELETE-then-UPSERT for natural-key upserts.
  - **D2.2** — LET-first RELATE pattern. If P3's listener bodies write new edges (e.g. Template C's `MANAGES`-triggered grant persist edges, Template D's `HAS_AGENT_SUPERVISOR` grants), they MUST use `LET $f = type::thing(...); LET $t = type::thing(...); RELATE $f -> <rel> -> $t SET id = type::thing('<rel>', $edge) RETURN NONE` — never inline `type::thing(...)` in RELATE slots.
- **Decisions to confirm at P3 open**: none. (P3 owns D1 (Template uniqueness — flipped Accepted at P1) and flips ADR-0029 to full Accepted at P3 close.)
- **Carry-forward invariants**:
  - phi-core imports must stay at exactly **17 lines** pre-P3; P3 adds **2 new** imports (`SessionRecorder` + `AgentEvent` in `domain/src/session_recorder.rs`). Total at P3 close: **19 lines**.
  - `count_active_sessions_for_agent` stays flipped (no regression to `Ok(0)` in production impls).
  - ADR statuses: 0029 Proposed/Accepted-in-part → flip to full Accepted at P3 close; 0030 Accepted stays; 0031 Proposed stays.

### Pre-P4 gates  `[STATUS: walked + closed at P4 open 2026-04-23 — D3/D5 both confirmed as assumed defaults; 9-item reading list honoured throughout P4]`

- **Reading list (9 addendum items, all P4-relevant)**:
  - **D1.1** — `session` + `turn` tables inherit 0001's mandatory `created_at: string` column. P4's session-launch writer + `BabyPhiSessionRecorder` persist calls already honour this via the P2 repo methods. If P4 adds any new direct Session/Turn writes, they MUST populate `created_at`.
  - **D1.2** — `runs_session` retype (session→project). P4's launch chain writes this edge from `sessions/launch.rs`; the P2 helper already does it correctly inside `persist_session`'s compound tx.
  - **D1.3** — Nested `inner` for Session/LoopRecordNode/TurnNode wraps. P4's `LaunchReceipt { session_detail: SessionDetail, ... }` response carries phi-core types via the nested form — the JSON-schema-snapshot test on `GET /projects/:pid/sessions` asserts the strip to `SessionHeader` (no `inner` leak).
  - **D2.1** — CREATE (not UPDATE) for fresh session/loop/turn rows. Applies to any direct SurrealDB writes P4's launch chain adds (the P2 repo methods already use CREATE).
  - **D2.2** — LET-first RELATE pattern. P4's `sessions/launch.rs` adds a `RELATE agent -> uses_model -> model_runtime` edge (C-M5-2 close) — this MUST follow the LET-first idiom: `LET $a = type::thing('agent', $aid); LET $m = type::thing('model_runtime', $mid); RELATE $a -> uses_model -> $m ...`. The `runs_session` edge (session→project) already uses this pattern in `persist_session`; extend the same pattern to `uses_model`.
  - **D3.1** — `DomainEvent::AgentCreated.agent_kind` (not `kind`). Any P4 emit site for this variant (agent-create flow when extended for M5) uses `agent_kind:` in struct construction. The serde wire key is `agent_kind`.
  - **D3.2** — Listener wiring lives in `state::build_event_bus_with_m5_listeners`. P4 may need to add the session-abort listener; extend the helper — do NOT create a parallel wiring path.
  - **D3.3** — `BabyPhiSessionRecorder::inner` is `Arc<Mutex<PhiCoreSessionRecorder>>`. P4's launch chain clones `Arc<BabyPhiSessionRecorder>` into the `tokio::spawn`-ed task; `on_phi_core_event(&self, …)` takes `&self`, not `&mut self`. MutexGuard must drop before any `.await` (clippy enforces).
  - **D3.4** — Template C / D adoption-AR resolver asymmetry. C is org-scoped (`resolve(OrgId)`), D is project-scoped (`resolve(ProjectId) -> Option<(OrgId, AuthRequestId)>`). If P4 needs to look up template adoption ARs from the launch chain (e.g. for session-scope permission check), use the matching resolver shape.
- **Decisions confirmed at P4 open (2026-04-23)**:
  - ✅ **D3** — `[session] max_concurrent = 16` in `config/default.toml` (confirmed as plan default; infrastructure landed at P1). Session-launch gate returns 503 `SESSION_WORKER_SATURATED` when the per-worker registry is full (distinct from per-agent W2 `PARALLELIZE_CAP_REACHED`).
  - ✅ **D5** — Permission Check preview location: **server-side** via `POST /orgs/:id/projects/:pid/sessions/preview { agent_id } → { trace: PermissionCheckTrace }`. Confirmed as plan default — single source of truth for M1's permission-check algorithm; trace reusable by CLI + web without re-implementation.
- **Carry-forward invariants**:
  - phi-core imports stay at exactly **19 lines** pre-P4; P4 adds **4 new** imports (`agent_loop`, `agent_loop_continue`, `AgentTool`, `ModelConfig`) for a P4 close total of **~23 lines**. A fresh import in `agents/update.rs` for `ModelConfig` (C-M5-5 flip) should not double-count if the same type is already imported in `sessions/launch.rs` — count **unique types**, not import lines.
  - ADR-0031 (session cancellation + concurrency) flips Proposed → Accepted at P4 close.
  - Five load-bearing carryovers close at P4: **C-M5-2** (UsesModel edge writer), **C-M5-3** (Session persistence end-to-end via `BabyPhiSessionRecorder` wired into the launch chain), **C-M5-4** (AgentTool resolver + `GET /sessions/:id/tools`), **C-M5-5** (ModelConfig change + real 409), **C-M5-6** (Shape B materialise).
  - `handler_count` moves from 5 to 5+ (if P4 adds session-lifecycle listeners) — update `handler_count_is_five_at_m5` test name + assertion accordingly, and record the new count in a P4 drift item if non-obvious.
  - P3's `SessionAborted` variant needs an emit site at P4 — `sessions/terminate.rs` writes it after the cancellation token fires + session row flips to `Aborted`.

### Pre-P5 gates  `[STATUS: ready for P5 open, populated at P4 close 2026-04-23]`

- **Reading list (15 addendum items, all P5-relevant)**:
  - **D1.1 / D1.2 / D1.3** — session/turn `created_at`, `runs_session` retype, nested `inner` wraps (unchanged invariants).
  - **D2.1 / D2.2** — CREATE-not-UPDATE + LET-first RELATE for any new SurrealDB writes (authority-template adopt/revoke flow writes multiple grant edges + revocation-cascade walks — same idioms apply).
  - **D3.1** — `DomainEvent::AgentCreated.agent_kind` (not `kind`). P5 doesn't need to construct this variant directly but any test fixture that does MUST use `agent_kind:`.
  - **D3.2** — `state::build_event_bus_with_m5_listeners` is the single wiring site. P5's authority-template page surfaces the list/approve/deny/adopt/revoke handlers; it does NOT add new listeners but MUST verify the Template A / C / D listeners are still in place (`handler_count == 5`).
  - **D3.3** — `BabyPhiSessionRecorder` is `Arc<Mutex<_>>` with `&self` API (unchanged).
  - **D3.4** — Template C (org-scoped) / Template D (project-scoped) resolver asymmetry. P5's approve/revoke handlers for Templates C + D MUST consult the right resolver shape.
  - **D4.1** — **Permission Check advisory-only at M5 (load-bearing for P5!)**. P5's authority-template handlers (approve, deny, adopt, revoke) emit reactive events that drive grant minting. Those grants are themselves permission-check inputs at launch time — but the launch's advisory-only policy means step-3/4/5/6 denials do NOT refuse the launch even if the Template C/D-minted grant would cover the reach. This is a soft invariant until M6+. P5 MUST document the handler shapes but MUST NOT rely on the launch refusing a permission-denied request.
  - **D4.2** — Synthetic replay feeder (launch chain does NOT call `phi_core::agent_loop`). P5 tests that drive a full project → session flow will see the synthetic replay, not a live LLM. Any M5/P5 acceptance test asserting "session produces 1 loop + 1 turn" is asserting the replay shape, not real agent behaviour.
  - **D4.3** — `resolve_agent_tools` returns `Vec<ToolSummary>`. P5 admin pages don't surface tools directly; no P5-specific impact.
  - **D4.4** — `upsert_agent_profile` is no-op for fresh rows; use `create_agent_profile` for first write. P5's system-agent page at P6 follows the same pattern.
  - **D4.5** — `write_uses_model_edge` is a typed Repository method. P5's authority-template handlers don't write new edges beyond grants + ARs (M4 patterns), so no new typed writers needed at P5 itself. P6 may add one for system-agent profile-swap edges.
  - **D4.6** — `SessionLaunchContext.first_loop_id: Option<LoopId>`. P5 does not construct launch contexts directly but the pattern (pre-existing row + first_loop_id + recorder append-only mode) is the template for any phase wiring `BabyPhiSessionRecorder` into a non-launch caller path (none currently planned pre-M6).
- **Decisions to confirm at P5 open**: none. (Plan §Part 3 D1 / D2 / D3 / D4 / D5 / D6 are all confirmed or have been flipped at P1 / P3 / P4.)
- **Carry-forward invariants**:
  - phi-core imports at P4 close: ~25 lines total (P3 close was 19; P4 added AgentEvent / TurnTrigger / ContinuationKind / LoopStatus / AgentMessage / LlmMessage / Message / Usage / agent_loop / agent_loop_continue / ModelConfig / AgentTool = +6 to +12 lines depending on overlap — plan expected ~23, actual 25 due to multiple phi-core types being referenced across launch.rs's synthetic replay feeder). Not a drift per se; just a delta from plan estimate.
  - ADR statuses at P4 close: 0029 **Accepted** (full) · 0030 **Accepted** · 0031 **Accepted** (flipped at P4 close, 2026-04-23).
  - Five M5 carryovers closed: **C-M5-2** (UsesModel edge writer) · **C-M5-3** (Session persistence end-to-end) · **C-M5-4** (AgentTool resolver) · **C-M5-5** (ModelConfig change + 409) · **C-M5-6** (Shape B materialise).
  - Workspace test count at P4 close: **939 passing, 0 failures** (up from 929 at P3 close; P4 added 7 acceptance tests + 3 session_recorder path tests + 2 tools tests).
  - `handler_count_is_five_at_m5` still asserts 5 listeners. P5 does not add new listeners.
- **P5 scope preview** (from plan §P5):
  - 6 files under `server/src/platform/templates/` (mod + list + approve + deny + adopt + revoke).
  - 5 HTTP routes under `/api/v0/orgs/:org/authority-templates/...`.
  - Revoke-cascade walks `DESCENDS_FROM` provenance + marks every descendant grant revoked in one compound tx.
  - `phi template` CLI subcommand (list, approve, deny, adopt, revoke).
  - Web page at `/organizations/:org/templates`.
  - Architecture doc `authority-templates.md` filled out.
  - phi-core leverage: **0 new imports** at P5 (pure governance-plane work).

### Pre-P6 gates  `[STATUS: ready for P6 open, populated at P5 close 2026-04-23]`

- **Reading list (18 addendum items, all P6-relevant)**:
  - D1.1 / D1.2 / D1.3 / D2.1 / D2.2 / D3.1 / D3.2 / D3.3 / D3.4 — unchanged load-bearing invariants (CREATE-not-UPDATE, LET-first RELATE, nested `inner`, `agent_kind` naming, `build_event_bus_with_m5_listeners` wiring, `Arc<Mutex<_>>` recorder, Template C/D resolver asymmetry).
  - **D4.1** — Permission Check advisory-only. P6's system-agent page 13 doesn't directly gate on launch but the page surfaces profile_ref + parallelize tuning that'll influence future launch gates; carries forward unchanged.
  - D4.2 / D4.3 / D4.4 / D4.5 / D4.6 — synthetic feeder, `Vec<ToolSummary>`, profile create-vs-upsert, typed `write_uses_model_edge`, `first_loop_id`.
  - **D5.1** — **CLI + Web for page 12 deferred to P7**. If P6 plans to ship CLI + Web for page 13 (system agents), the deferral precedent suggests those too shift to P7 unless the user explicitly wants them at P6. Worth raising at P6 open.
  - D5.2 — Audit events live in `server::platform::<page>::audit_events` for page handlers (not `domain::audit::events::mX::*`). P6's system-agent audit events follow the same convention.
  - D5.3 — `find_adoption_ar` returns most-recent only. No direct P6 impact (page 13 doesn't consume adoption ARs), but the idiom "call `list_*_for_org` + filter locally when multiple rows are expected" carries forward.
- **Decisions to confirm at P6 open**: **one** — whether to also defer page 13's CLI + Web to P7 alongside D5.1's page 12 surfaces. Default: defer (matches D5.1 precedent). Alternative: ship page 13 CLI inline with business logic at P6 — feasible since page 13's CLI surface is smaller than page 12's (no revoke-cascade).
- **Carry-forward invariants**:
  - phi-core imports at P5 close: **25 lines** (no change from P4; zero P5 imports).
  - ADR statuses at P5 close: 0029 Accepted · 0030 Accepted · 0031 Accepted (all three flipped in prior phases).
  - Handler count still 5 (P5 adds no new listeners; page 12 is synchronous HTTP).
  - Workspace test count at P5 close: **950** passing, 0 failures (P4 close was 939; +11 at P5: 7 acceptance scenarios in `acceptance_authority_templates.rs` + 4 audit-event unit tests).
  - 5 M5 carryovers still closed (no regression).
- **P6 scope preview** (from plan §P6):
  - 7 files under `server/src/platform/system_agents/` (mod + list + tune + add + disable + archive + events_feed).
  - 5 HTTP routes under `/api/v0/orgs/:org/system-agents/...`.
  - Listener-callback extension: all 5 existing listeners (Template A/C/D + memory-extraction + agent-catalog) call `repo.upsert_system_agent_runtime_status(...)` on each fire. Shared helper in `domain/src/events/listeners.rs`.
  - phi-core leverage: **1 new import** — `AgentProfile` re-use in `system_agents/add.rs` (Part 1.5 prediction: 1).
  - If D5.1 precedent holds: P6 ships HTTP surface only; CLI + Web defer to P7.

### Pre-P7 gates  `[STATUS: ready for P7 open, populated at P6 close 2026-04-23]`

- **Reading list (23 addendum items, all P7-relevant)**:
  - D1.1–D2.2 (5) — session/turn `created_at`, `runs_session` retype, nested `inner`, CREATE-not-UPDATE, LET-first RELATE — load-bearing across every persistence touch.
  - D3.1–D3.4 (4) — `agent_kind` naming, `build_event_bus_with_m5_listeners` wiring, `Arc<Mutex<_>>` recorder, Template C/D resolver asymmetry.
  - D4.1–D4.6 (6) — Permission Check advisory, synthetic feeder, `Vec<ToolSummary>`, profile create-vs-upsert, typed `write_uses_model_edge`, `first_loop_id`.
  - D5.1–D5.3 (3) — **CLI + Web for page 12 deferred to P7**, audit events at server tier, `find_adoption_ar` most-recent.
  - D6.1–D6.5 (5) — listener helper deferred, **CLI + Web for page 13 deferred to P7**, union-filter bucketing, audit events at server tier, disable/archive no durable flip.
- **Decisions to confirm at P7 open**: none — D5.1 + D6.2 deferrals already decided at their respective phase closes.
- **Carry-forward invariants**:
  - phi-core imports at P6 close: **26 lines** (P5 close was 25; +1 at P6 for `AgentProfile` in system_agents/add.rs, matching Part 1.5 prediction).
  - ADR statuses at P6 close: 0029 Accepted · 0030 Accepted · 0031 Accepted (no new ADRs at P5 / P6).
  - Handler count unchanged at 5 (neither P5 nor P6 added listeners).
  - Workspace test count at P6 close: **962** passing, 0 failures (P5 close was 950; +12 at P6 = 8 acceptance scenarios in `acceptance_system_agents.rs` + 4 audit-event unit tests).
  - 5 M5 carryovers still closed (C-M5-2 / -3 / -4 / -5 / -6).
- **P7 scope preview** (from plan §P7 + accumulated deferrals):
  - **Plan-original P7 deliverables**:
    - `cli/src/commands/session.rs` — full implementation (launch / show / terminate / list with SSE tail + `--detach`).
    - `cli/src/commands/agent.rs` — extend `phi agent update` with `--model-config-id`.
    - CLI completion regression.
    - Web polish: page 14 (session launch) + page 11's "Recent sessions" panel retrofit.
  - **Deferred from P5 (D5.1)**: `phi template {list, approve, deny, adopt, revoke}` CLI + Next.js `/organizations/[id]/templates/` page.
  - **Deferred from P6 (D6.2)**: `phi system-agent {list, tune, add, disable, archive}` CLI + Next.js `/organizations/[id]/system-agents/` page.
  - phi-core leverage at P7: `phi_core::types::event::AgentEvent` may be imported in `cli/src/commands/session.rs` for SSE tail rendering (≤1 new import line). Otherwise 0 new imports.
- **Recommendation for P7 open**: given the consolidated CLI + Web scope (3 pages × 2 surfaces + `phi session` = 7 surface deliverables), the phase may benefit from an explicit sub-checkpoint within its single confidence check. Up to user.

### Pre-P8 gates  `[STATUS: ready for P8 open, populated at P7 close 2026-04-24]`

- **Reading list (29 addendum items, all P8-relevant)**:
  - D1.1–D2.2 (5) — persistence invariants (session/turn `created_at`, `runs_session` retype, nested `inner`, CREATE-not-UPDATE, LET-first RELATE) — P8's listener bodies add Memory-extraction audit emits + AgentCatalogEntry upserts; both must honour these.
  - D3.1–D3.4 (4) — `agent_kind` naming, `build_event_bus_with_m5_listeners` wiring, `Arc<Mutex<_>>` recorder, Template C/D resolver asymmetry. D3.2 is load-bearing at P8: both new listener bodies extend the existing wiring helper, not a parallel path.
  - D4.1–D4.6 (6) — Permission Check advisory, **synthetic feeder**, `Vec<ToolSummary>`, profile create-vs-upsert, typed `write_uses_model_edge`, `first_loop_id`. **D4.2 is load-bearing at P8**: s02 memory-extraction reads the persisted Session transcript. The transcript is synthetic at M5 (no real agent_loop yet); s02's extractor will read whatever the synthetic feeder wrote. Acceptance tests MUST assert on the extractor's tag-bundle shape, not on extracted-memory semantics (which depend on real LLM output).
  - D5.1–D5.3 (3) — CLI + Web deferred + audit-event placement + `find_adoption_ar` most-recent. D5.2 establishes the server-tier audit-events convention; s02/s03 follow it for `MemoryExtracted` + agent-catalog change events.
  - D6.1–D6.5 (5) — **D6.1 is THE load-bearing item at P8**: the `record_system_agent_fire` helper at [`domain::events::listeners::record_system_agent_fire`](../../../../modules/crates/domain/src/events/listeners.rs) is the first thing P8 wires. Both memory-extraction and agent-catalog listener bodies call it at the top of their per-fire execution to seed `SystemAgentRuntimeStatus.queue_depth` + `last_fired_at`. D6.5 reminds P8 that disable/archive don't flip durable state at M5 — if a system-agent is disabled, P8's listener body should early-return (the flag lives in memory + audit only).
  - D7.1–D7.6 (6) — live tail deferred, `--model-config-id` additive, `preview` 5th subcommand, page-11 retrofit web-side, no new web tests, hybrid inline-closure server actions. D7.1 is the biggest P8-adjacent invariant: the synthetic-feeder Session shape is what s02 extracts from — a P8 acceptance test that assumes multi-turn transcripts will fail (synthetic feeder writes exactly 1 Turn).
- **Decisions to confirm at P8 open**: none. All P0 §Part 3 decisions (D1–D7) are confirmed or flipped. §Part 11 Q4 (`MemoryExtracted` audit-event tag-list shape) was drafted conceptually at P8 open time; P8 body + audit-event module confirm + test the exact shape.
- **Carry-forward invariants**:
  - phi-core imports at P7 close: **26** (unchanged from P6). Part 1.5 prediction for P8: **+1** (`phi_core::agent_loop` in `MemoryExtractionListener::on_event`). P8 close target: **27**.
  - ADR statuses at P7 close: 0029 Accepted · 0030 Accepted · 0031 Accepted. No new ADRs at P7; P8 unlikely to add one (the listener bodies follow the M4 Template-A fire pattern).
  - Handler count still 5 (P7 adds no listeners; CLI + web are synchronous).
  - Workspace test count at P7 close: **966** passing, 0 failures (P6 close was 962; +4 at P7 from completion regression tests: template tree, system-agent tree, `--model-config-id` flag, `session preview` subcommand).
  - Web test count at P7 close: **79** (unchanged from P6; D7.5).
  - 5 M5 carryovers still closed (C-M5-2 / -3 / -4 / -5 / -6).
  - **D4.2 synthetic feeder invariant: Session has 1 Loop + 1 Turn at M5.** P8 acceptance tests MUST honour this (no "check turn 2 has tool call X" assertions).
- **P8 scope preview** (from plan §P8):
  - **`MemoryExtractionListener::on_event` body** (fires on `SessionEnded`):
    - Resolve the session via `fetch_session(session_id)`.
    - Read the project's + org's + agent's tag metadata.
    - Run `phi_core::agent_loop` with the extractor profile's blueprint + transcript (first phi-core direct call at a listener body).
    - For each candidate memory, emit a `MemoryExtracted` audit event with **structured tag list + session reference** — M6's C-M6-1 materialises Memory nodes from this audit stream.
    - At the top: call `record_system_agent_fire(repo, owning_org, extractor_agent_id, effective_parallelize, None, now)` to seed the runtime-status tile.
    - Failure modes: queue saturation → `MemoryExtractionSkipped { queue_saturated }`; agent disabled → skipped; LLM retry exhausted → `MemoryExtractionFailed`.
  - **`AgentCatalogListener::on_event` body** (fires on 8 DomainEvent variants):
    - `AgentCreated` → upsert fresh catalog row.
    - `AgentArchived` → upsert with `active: false`.
    - `HasLeadEdgeCreated` / `ManagesEdgeCreated` / `HasAgentSupervisorEdgeCreated` → refresh role-index.
    - `HasProfileEdgeChanged` → refresh cached profile snapshot.
    - Top-of-body: call `record_system_agent_fire(...)` for the catalog system agent.
  - **Template C + D listener bodies** — shipped at P3, verified end-to-end at P8 via `acceptance_system_flows_s05.rs`.
  - **Ops doc** `m5/operations/system-flows-s02-s03-operations.md` filled out.
  - **New tests** (plan estimate ~18):
    - `acceptance_system_flows_s02.rs` — 6 scenarios (happy extraction, queue saturation, agent disabled, retry exhausted, multi-tag session, Shape E forbidden session).
    - `acceptance_system_flows_s03.rs` — 8 scenarios (one per DomainEvent variant).
    - `acceptance_system_flows_s05.rs` — 4 scenarios (Template C fires on MANAGES, Template D fires on HAS_AGENT_SUPERVISOR, A+C+D all active at once).
- **P8 operational caution**: the M5 synthetic feeder (D4.2) only emits `{AgentStart, TurnEnd, AgentEnd}` — 1 LoopRecord, 1 Turn. s02's extractor runs against this transcript. Tests asserting "memory reflects tool call on turn 2" cannot work until M7+ provides a real agent_loop harness. Stub the extractor to pass the transcript through + assert the tag-bundle survives serde round-trip; memory semantic correctness is an M7+ concern.

### Pre-P9 gates

Populated at P8 close.

---

**Closing protocol reminder.** The plan archive is the single canonical place for phase-level coherence tracking: §Part 4 phase bodies stay pristine (P0 snapshot); the Drift addenda section holds reality (both drift items and phase-open gates). Every phase close updates addendum entries; every phase open walks the gates. Neither step is optional.
