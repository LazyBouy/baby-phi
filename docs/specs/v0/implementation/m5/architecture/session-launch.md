<!-- Last verified: 2026-05-09 by Claude Code (CH-17 amendment: live SSE tail companion endpoint at `GET /api/v0/sessions/:id/events` ships per ADR-0055 — sibling permission-builder `build_session_observe_manifest` returns `[Observe]` on `session_object` (NOT the launch builder's `[Read, Inspect, List]`); broadcast tap on `BabyPhiSessionRecorder.broadcast_tx`; per-session `tokio::broadcast` registered into `AppState.session_live_stream_registry: Arc<dyn SessionLiveStreamRegistry>` immediately before `tokio::spawn` of the agent task; SSE handler subscribes via `sender.subscribe()` and serialises `phi_core::AgentEvent` JSON onto the wire; 30s keep-alive; lagged consumers receive a typed `event: lagged` SSE error then close. The launch path itself is UNCHANGED. Drift D7.1 closed.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-15 amendment: Step 3 advisory→hard-deny flipped per ADR-0054 §D54.6; synthetic launch manifest now built via `domain::permissions::build_session_launch_manifest`; new `platform.session.launch_denied` audit event emitted on every step-1-to-6 deny; drift D4.1 closed.) -->
<!-- CH-02 amendment (2026-04-24): synthetic 4-event feeder replaced with real `phi_core::agent_loop()` driven by `MockProvider` (deterministic, no network). Drift D4.2 closed at CH-02 chunk-seal. See §"CH-02 amendment" below + ADR-0032. -->

# Page 14 — First Session Launch architecture

**Status**: `[EXISTS]` as of M5/P4. Business logic lives in
[`server::platform::sessions`](../../../../../../modules/crates/server/src/platform/sessions/);
HTTP surface in
[`server::handlers::sessions`](../../../../../../modules/crates/server/src/handlers/sessions.rs).
The five M5 carryovers (C-M5-2 / C-M5-3 / C-M5-4 / C-M5-5 / C-M5-6)
all close at P4 — see the M5 plan archive §P4 for the full
deliverable list.

## HTTP surface

Seven routes registered in
[`router.rs`](../../../../../../modules/crates/server/src/router.rs):

| Method | Path | Handler | Purpose |
|---|---|---|---|
| POST | `/api/v0/orgs/:org/projects/:project/sessions` | `sessions::launch` | 9-step launch flow (C-M5-2 + C-M5-3 close) |
| POST | `/api/v0/orgs/:org/projects/:project/sessions/preview` | `sessions::preview` | D5 server-side Permission Check preview |
| GET  | `/api/v0/sessions/:id` | `sessions::show` | Full `SessionDetail` (session + loops + turns) |
| POST | `/api/v0/sessions/:id/terminate` | `sessions::terminate` | Operator-initiated abort (cancel token fire + `SessionAborted` emit) |
| GET  | `/api/v0/projects/:project/sessions` | `sessions::list_in_project` | Header strip (no phi-core `inner` leak) |
| GET  | `/api/v0/sessions/:id/tools` | `sessions::tools` | C-M5-4 tool summaries (wire shape) |
| GET  | `/api/v0/sessions/:id/events` | `sessions::events` | **CH-17** — live SSE tail (operator observability surface; gated by `Action::Observe` on `session_object` per [ADR-0055](../../m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md) §D55.5) |

### CH-17 — SSE companion endpoint

`GET /api/v0/sessions/:id/events` ships at CH-17 as the live-tail companion to the launch endpoint. It uses the **sibling** permission builder `domain::permissions::builders::build_session_observe_manifest` (returns `[Observe]` on `session_object`) rather than the launch builder's `[Read, Inspect, List]` — see [ADR-0055 §D55.5](../../m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md). Both builders feed the same `engine::check()` engine surface; the SSE path runs Steps 0–5 + skips Step 6 (consent is launch-time, not stream-time per ADR-0048 §D48.5). Pod-local `tokio::sync::broadcast::Sender<phi_core::AgentEvent>` is registered into `AppState.session_live_stream_registry` (trait `SessionLiveStreamRegistry`, default impl `InProcessSessionLiveStreamRegistry { inner: DashMap<SessionId, broadcast::Sender<AgentEvent>> }`) immediately before the agent task `tokio::spawn`. SSE consumers subscribe via `sender.subscribe()`. Cross-pod fan-out deferred via [CHK8S-D-10](../../m7b/architecture/deferred-from-ch-k8s-prep.md). For full ops detail (lagged-receiver semantics, reconnect playbook, audit-event emission), see [`m5_2/operations/session-live-stream-operations.md`](../../m5_2/operations/session-live-stream-operations.md). For the architectural design page, see [`m5_2/architecture/session-live-stream.md`](../../m5_2/architecture/session-live-stream.md).

## 9-step launch flow

Flow owner: [`platform::sessions::launch::launch_session`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs).

1. **Validate agent + project + membership**. Returns
   `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` /
   `AGENT_NOT_MEMBER_OF_PROJECT`.
2. **Resolve the agent's `ModelConfig`** via
   `profile.model_config_id` against the `ModelRuntime` catalogue.
   Returns `MODEL_RUNTIME_UNRESOLVED` / `MODEL_RUNTIME_NOT_FOUND` /
   `MODEL_RUNTIME_ARCHIVED` / `AGENT_PROFILE_MISSING`.
3. **Permission Check** via the M1 engine using the synthetic
   launch manifest from
   [`domain::permissions::build_session_launch_manifest`](../../../../../../modules/crates/domain/src/permissions/builders/session_launch.rs)
   (CH-15 / [ADR-0054](../../m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md)).
   Manifest shape: `actions = [Read, Inspect, List]` on `resource =
   ["session_object"]`. Every `Decision::Denied { failed_step,
   reason }` returns 403
   `PERMISSION_CHECK_FAILED_AT_STEP_<N>` where `<N>` is the
   FailedStep variant's numeric label (0..6). The launch handler
   emits a `platform.session.launch_denied` audit event (Alerted)
   on every step-1-to-6 deny BEFORE returning the 403. Drift
   **D4.1** closed at CH-15.
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

## CH-02 amendment — real `agent_loop()` + MockProvider wiring (2026-04-24)

The "Synthetic-feeder drift" callout above is **closed** post-CH-02. M5/P4's `spawn_replay_task` was a 4-event canned sequence; CH-02 replaces it with a runtime call to `phi_core::agent_loop(...)` driven by [`phi_core::provider::mock::MockProvider`](../../../../../../../phi-core/src/provider/mock.rs) (deterministic, no network). The outer 9-step launch flow is unchanged in shape — only the inner spawned task body changed.

### What CH-02 added

1. **New helper** [`platform/sessions/provider.rs`](../../../../../../modules/crates/server/src/platform/sessions/provider.rs) — `provider_for(profile)` returns an `Arc<dyn StreamProvider>`. At M5 always returns a `MockProvider::text(profile.mock_response.unwrap_or("Acknowledged."))`. M7+ swaps to `ProviderRegistry::resolve()` against the org's runtime catalogue, with no change to the call site.
2. **Per-profile mock-response governance field** — `domain::AgentProfile.mock_response: Option<String>` (added in [`domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs)) lets operators pin deterministic test outputs per agent. Migration `0006_agent_profile_mock_response.surql` adds the column. The field lives on the baby-phi wrapper, NOT on the phi-core inner `blueprint` (the wrap pattern from ADR-0015).
3. **Hot-path phi-core leverage** — `phi_core::agent_loop`, `MockProvider`, `StreamProvider`, `AgentContext`, `AgentLoopConfig` are all directly used at runtime. The previous compile-time witness pattern at `_keep_agent_loop_live` is retired.
4. **Real agent-loop event stream** — `BabyPhiSessionRecorder` now consumes the actual phi-core event sequence (variable in shape, not the canned 4-event run). Final `AgentEnd` still triggers `finalise_and_persist`.

### What CH-02 did NOT change

- The compound tx commit (Step 6) is unchanged.
- `SessionRegistry` cancellation semantics are unchanged.
- The `LaunchReceipt` shape is unchanged.
- Real LLM providers (Anthropic / OpenAI / etc.) defer to M7 — `MockProvider` is the only shipped variant at M5.

### Operator consequence

The recorder's persisted turn shape now reflects MockProvider's deterministic outputs. The default response is `"Acknowledged."`; operators can pin a custom string per-agent via `mock_response` on the agent profile (set via `PATCH /api/v0/agents/:id/profile` with `mock_response: "..."`). This affects acceptance-test expectations — pre-CH-02 tests that asserted exact synthetic message text need updating to the new MockProvider output.

### Drift transitions at CH-02 close

- **D4.2** (HIGH, `leverage-violation`) — `discovered → in-chunk-plan → remediated`. The phi-core agent_loop is now on a hot execution path, not a compile witness.

### ADR

- [ADR-0032](../../m5_2/decisions/0032-mock-provider-at-m5.md) — MockProvider as the at-M5 stand-in; deferral path to real providers at M7.

## Cross-references

- [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md) — session persistence + recorder wrap.
- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md) — session cancellation + concurrency.
- [ADR-0032](../../m5_2/decisions/0032-mock-provider-at-m5.md) — CH-02 MockProvider decision.
- [Event bus M5 extensions](./event-bus-m5-extensions.md) — governance events emitted by launch + terminate.
- [Session persistence](./session-persistence.md) — the 3-way wrap pattern.
- [CH-02 plan](../../../../plan/build/ch-02-real-agent-loop-wiring-16fd9a3a.md).
