<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-22 amendment (2026-04-27): six production emit sites wired (ADR-0035 §D35.5). Status flips on the variant table below: AgentCreated / AgentArchived / HasProfileEdgeChanged emitters are now `[EXISTS]`. AgentCatalogListener body shipped — see §"CH-22 amendment" below. -->

# Event bus — M5 extensions

**Status**: `[EXISTS]` as of M5/P3. Code lands; emit-site column
reads "planned" where the writer handler doesn't land until a later
phase (e.g. `sessions/launch.rs` at P4).

M4/P3 introduced the in-process event bus with a single
`DomainEvent::HasLeadEdgeCreated` variant + `TemplateAFireListener`.
M5/P3 extends the variant enum to 9 variants total and wires 4 new
listeners — 2 full-body (Template C + D fire) and 2 stubs filled at
M5/P8 (memory extraction + agent catalog).

## 8 new `DomainEvent` variants (P3)

| Variant | Emit site | Subscriber(s) at P3 | Status |
|---|---|---|---|
| `SessionStarted { session_id, agent_id, project_id, started_at, event_id }` | `BabyPhiSessionRecorder::on_phi_core_event` on first `AgentStart` (P3); `sessions/launch.rs` post-commit writer adds the redundant launch-time emit at P4 | none (M7 observability) | `[EXISTS]` emitter · `[PLANNED M5/P4]` launch-handler writer |
| `SessionEnded { session_id, agent_id, project_id, ended_at, duration_ms, turn_count, tokens_spent, event_id }` | `BabyPhiSessionRecorder::finalise_and_persist` on phi-core's final `AgentEnd` | `MemoryExtractionListener` (stub at P3; body at P8) | `[EXISTS]` emitter + subscription · body `[PLANNED M5/P8]` |
| `SessionAborted { session_id, reason, terminated_by, at, event_id }` | `sessions/terminate.rs` + task-level scopeguard panic fallback | `AgentCatalogListener` only (memory extraction skips aborted sessions) | `[PLANNED M5/P4]` emitter · `[EXISTS]` variant |
| `ManagesEdgeCreated { org_id, manager, subordinate, at, event_id }` | org / agent mutation handler that writes the `MANAGES` edge (no writer exists yet — lands when org-tree CRUD ships) | `TemplateCFireListener` (full body at P3); `AgentCatalogListener` (stub) | `[EXISTS]` listener body · `[PLANNED M6/M7]` emitter |
| `HasAgentSupervisorEdgeCreated { project_id, supervisor, supervisee, at, event_id }` | project-role assignment handler (no writer at M5; part of the project-team CRUD that follows page 14) | `TemplateDFireListener` (full body at P3); `AgentCatalogListener` (stub) | `[EXISTS]` listener body · `[PLANNED M6/M7]` emitter |
| `AgentCreated { agent_id, owning_org, agent_kind, role, at, event_id }` | `agents/create.rs` post-commit (M4 writer extended at M5/P4+) | `AgentCatalogListener` (stub at P3; body at P8) | `[EXISTS]` variant · `[PLANNED]` emitter in M4 writer |
| `AgentArchived { agent_id, at, event_id }` | agent archive / disable handler (M5/P6's system-agent page ships the first writer) | `AgentCatalogListener` (stub) | `[EXISTS]` variant · `[PLANNED M5/P6]` emitter |
| `HasProfileEdgeChanged { agent_id, old_profile_id, new_profile_id, at, event_id }` | `agents/update.rs` profile-ref swap (M4 path, extended at M5/P4) | `AgentCatalogListener` (stub) | `[EXISTS]` variant · `[PLANNED M5/P4]` emitter |

Field-naming note: the `AgentCreated` variant's kind field is named
`agent_kind` (not `kind`) because serde's `tag = "kind"` enum
discriminator claims the JSON key `kind` at the enum level.

## Fail-safe ordering

All emit sites remain **post-commit** (M4 ADR-0028's invariant).
If emit fails, the governance write is already durable; the event
is logged with `event_id` for manual replay. M7b adds persistent
event queue for exactly-once semantics.

## Listener registration

At M5/P3 close, `server::state::build_event_bus_with_m5_listeners`
wires **5 listeners** (M4's `TemplateAFireListener` + the 4 new):

1. `TemplateAFireListener` — HAS_LEAD → `[read, inspect, list]`
   grant on `project:<uuid>` (M4).
2. `TemplateCFireListener` — MANAGES → `[read, inspect]` grant on
   `agent:<subordinate-uuid>` (M5/P3).
3. `TemplateDFireListener` — HAS_AGENT_SUPERVISOR → `[read,
   inspect]` grant on `project:<puuid>/agent:<supervisee-uuid>`
   (M5/P3, project-scoped).
4. `MemoryExtractionListener` — SessionEnded → body at M5/P8
   (supervisor `agent_loop` run + `MemoryExtracted` audit).
5. `AgentCatalogListener` — 8-variant fan-in → body at M5/P8
   (upsert `AgentCatalogEntry` row per agent).

Asserted by
[`server::state::tests::handler_count_is_five_at_m5`](../../../../../../modules/crates/server/src/state.rs).

## phi-core leverage

This module is pure phi governance plumbing — **0** direct phi-core
imports in `domain/src/events/`. phi-core's `AgentEvent` is
consumed only inside `domain/src/session_recorder.rs` (2 imports —
see [phi-core-reuse-map.md](./phi-core-reuse-map.md)).

The separation is deliberate (ADR-0029): `DomainEvent` is
governance-plane post-commit notifications; `AgentEvent` is
agent-loop runtime telemetry. They share no transit surface.

## CH-22 amendment — emit-site status table (2026-04-27)

CH-22 P3 wired six production emit sites that were `[PLANNED]` at M5/P3. Updated table:

| Variant | Emit site | Status |
|---|---|---|
| `AgentCreated` | [`platform/agents/create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs); [`platform/system_agents/add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs); [`platform/orgs/create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) (×3 — CEO + 2 system agents) | `[EXISTS]` (was `[PLANNED]`) |
| `AgentArchived` | [`platform/system_agents/disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs); [`platform/system_agents/archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs) | `[EXISTS]` (was `[PLANNED]`) |
| `HasProfileEdgeChanged` | [`platform/agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) — emits only when the profile row actually changed (display-name-only / limits-only edits skip) | `[EXISTS]` (was `[PLANNED]`) |
| `SessionStarted`, `SessionEnded`, `SessionAborted` | unchanged from M5/P3 wiring | `[EXISTS]` |
| `HasLeadEdgeCreated` | unchanged from M4/P3 (`platform/projects/create.rs`) | `[EXISTS]` |
| `ManagesEdgeCreated`, `HasAgentSupervisorEdgeCreated` | still `[PLANNED M6/M7]` — emit happens when org-tree / project-team CRUD ships; CH-22 did not change these | `[PLANNED]` |

**Listener registration (post-CH-22):**

The 5-listener registration set is unchanged in count. `AgentCatalogListener` flipped from stub (silent log) to body:

5. `AgentCatalogListener` — 8-variant fan-in; body shipped at CH-22. Mutates `agent_catalog_entry` rows, calls `record_system_agent_fire` per fire, and (when `audit_mode == Debug`) emits `agent_catalog_refreshed` audit events.

Constructor signature changed: `AgentCatalogListener::new(repo, audit, audit_mode: CatalogAuditMode)` — `CatalogAuditMode` defaults to `Silent`. The harness `Acceptance` struct now exposes a public `event_bus: Arc<dyn EventBus>` field so acceptance tests can subscribe listeners post-spawn.

## Cross-references

- [M4 ADR-0028](../../m4/decisions/0028-domain-event-bus.md) — parent architectural decision.
- [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md) — session persistence + `BabyPhiSessionRecorder` design.
- [ADR-0035](../../m5_2/decisions/0035-agent-catalog-listener-audit-mode.md) — CH-22 audit-mode + emit-site wiring.
- [M5 plan §P3](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
