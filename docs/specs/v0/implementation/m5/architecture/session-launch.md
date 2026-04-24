<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 14 — First Session Launch architecture

**Status**: `[EXISTS]` as of M5/P4. Business logic lives in
[`server::platform::sessions`](../../../../../../modules/crates/server/src/platform/sessions/);
HTTP surface in
[`server::handlers::sessions`](../../../../../../modules/crates/server/src/handlers/sessions.rs).
The five M5 carryovers (C-M5-2 / C-M5-3 / C-M5-4 / C-M5-5 / C-M5-6)
all close at P4 — see the M5 plan archive §P4 for the full
deliverable list.

## HTTP surface

Six routes registered in
[`router.rs`](../../../../../../modules/crates/server/src/router.rs):

| Method | Path | Handler | Purpose |
|---|---|---|---|
| POST | `/api/v0/orgs/:org/projects/:project/sessions` | `sessions::launch` | 9-step launch flow (C-M5-2 + C-M5-3 close) |
| POST | `/api/v0/orgs/:org/projects/:project/sessions/preview` | `sessions::preview` | D5 server-side Permission Check preview |
| GET  | `/api/v0/sessions/:id` | `sessions::show` | Full `SessionDetail` (session + loops + turns) |
| POST | `/api/v0/sessions/:id/terminate` | `sessions::terminate` | Operator-initiated abort (cancel token fire + `SessionAborted` emit) |
| GET  | `/api/v0/projects/:project/sessions` | `sessions::list_in_project` | Header strip (no phi-core `inner` leak) |
| GET  | `/api/v0/sessions/:id/tools` | `sessions::tools` | C-M5-4 tool summaries (wire shape) |

## 9-step launch flow

Flow owner: [`platform::sessions::launch::launch_session`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs).

1. **Validate agent + project + membership**. Returns
   `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` /
   `AGENT_NOT_MEMBER_OF_PROJECT`.
2. **Resolve the agent's `ModelConfig`** via
   `profile.model_config_id` against the `ModelRuntime` catalogue.
   Returns `MODEL_RUNTIME_UNRESOLVED` / `MODEL_RUNTIME_NOT_FOUND` /
   `MODEL_RUNTIME_ARCHIVED` / `AGENT_PROFILE_MISSING`.
3. **Permission Check preview** via the M1 engine. Advisory-only
   at M5 (drift **D4.1**): Step 0 (Catalogue) still gates; steps
   1–6 surface on the receipt for operator visibility but do NOT
   refuse the launch. M6+ tightens the gate once the per-action
   manifest catalogue ships.
4. **W2 — per-agent parallelize gate**:
   `count_active_sessions_for_agent < profile.parallelize`. Returns
   `PARALLELIZE_CAP_REACHED` (409).
5. **D3 — platform saturation gate**:
   `session_registry.len() < session_max_concurrent`. Returns
   `SESSION_WORKER_SATURATED` (503).
6. **Compound tx** — Session row + first LoopRecordNode +
   `runs_session` edge (P2-shipped) + `uses_model` edge
   (P4-shipped, C-M5-2 close) via
   [`Repository::persist_session`](../../../../../../modules/crates/domain/src/repository.rs)
   + `Repository::write_uses_model_edge`. Emit
   `DomainEvent::SessionStarted` on the governance bus after
   commit.
7. **Spawn the replay task** — register the `CancellationToken` in
   [`SessionRegistry`](../../../../../../modules/crates/server/src/state.rs)
   + `tokio::spawn` the feeder (see §Replay below).
8. **Return `LaunchReceipt`** — `session_id`, `first_loop_id`,
   `permission_check_decision`, `session_started_event_id`.

## Replay task

`spawn_replay_task` feeds a synthetic phi-core event sequence —
`AgentStart` → `TurnStart` → `TurnEnd` → `AgentEnd` — through
[`BabyPhiSessionRecorder`](../../../../../../modules/crates/domain/src/session_recorder.rs).
On terminal (`AgentEnd`), the recorder's `finalise_and_persist`
appends materialised turns to the first loop (reusing the launch
chain's `first_loop_id` via `SessionLaunchContext`) + flips the
Session row's `governance_state` to `Completed` +
emits `DomainEvent::SessionEnded`.

**Synthetic-feeder drift**: phi-core's `agent_loop()` is NOT called
at M5 (no concrete provider credentials + no tool impls yet). The
`use phi_core::{agent_loop, agent_loop_continue}` imports in
`launch.rs` are compile-time witnesses — M7+ swaps the feeder body
for `agent_loop(prompts, ctx, cfg, tx, cancel_token)` without
touching the outer flow shape. See drift item **D4.2** in the M5
plan archive.

## Cancellation + concurrency (ADR-0031)

- `SessionRegistry = Arc<DashMap<SessionId, CancellationToken>>`
  on `AppState`. Per-worker, not cluster-wide — Redis-backed
  shared registry deferred to M7b.
- `sessions::terminate` fires `token.cancel()` + removes the entry
  + flips the Session row's `governance_state` to `Aborted` +
  emits `DomainEvent::SessionAborted`.
- `session_max_concurrent` comes from
  `config.session.max_concurrent` (default **16**, confirmed at
  M5/P4 open per gate walk).

## Carryover closures

| Carryover | Evidence |
|---|---|
| **C-M5-2** — UsesModel edge retype writer | `launch.rs` calls `write_uses_model_edge`; SurrealDB impl uses LET-first RELATE (D2.2); acceptance `launch_happy_path_persists_session_and_writes_uses_model_edge` |
| **C-M5-3** — Session persistence end-to-end | `launch.rs` calls `persist_session(session, first_loop)`; `BabyPhiSessionRecorder::finalise_and_persist` appends turns + flips state; acceptance asserts 1 loop + 1 turn persist |
| **C-M5-4** — AgentTool resolver | `sessions::tools::resolve_agent_tools` + `ToolSummary` wire shape; `use phi_core::types::tool::AgentTool` compile-time witness; returns empty list at M5 (drift **D4.3**) |
| **C-M5-5** — ModelConfig change + 409 gate | `platform::agents::update_agent_profile` validates `model_config_id` + checks `count_active_sessions_for_agent` → `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` |
| **C-M5-6** — Shape B materialise | `approve_pending_shape_b` Approved branch reads `shape_b_pending_projects` sidecar + calls `materialise_project` + deletes sidecar |

## Cross-references

- [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md) — session persistence + recorder wrap.
- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md) — session cancellation + concurrency.
- [Event bus M5 extensions](./event-bus-m5-extensions.md) — governance events emitted by launch + terminate.
- [Session persistence](./session-persistence.md) — the 3-way wrap pattern.
