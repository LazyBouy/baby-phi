<!-- Last verified: 2026-05-09 by Claude Code (CH-17 P3 — design page paired with ADR-0055; cycle hex 40c4d759.) -->

# Session live-event stream — architecture

> **Status:** [EXISTS] as of CH-17 (M5.2). The trait `SessionLiveStreamRegistry` + default `InProcessSessionLiveStreamRegistry` ship at [`server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs); the broadcast tap on `BabyPhiSessionRecorder.broadcast_tx` lives at [`domain/src/session_recorder.rs`](../../../../../../modules/crates/domain/src/session_recorder.rs); the SSE handler at [`server/src/handlers/sessions.rs`](../../../../../../modules/crates/server/src/handlers/sessions.rs) (`events`); the platform-layer `open_live_stream` at [`server/src/platform/sessions/events.rs`](../../../../../../modules/crates/server/src/platform/sessions/events.rs); the sibling permission-builder `build_session_observe_manifest` at [`domain/src/permissions/builders/session_observe.rs`](../../../../../../modules/crates/domain/src/permissions/builders/session_observe.rs); the new audit-event builder `session_live_stream_denied` at [`domain/src/audit/events/m5_2/session_live_stream.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/session_live_stream.rs); migration `0016_template_a_session_object_grant_add_observe.surql`. For the normative concept-doc reference, read [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"session lifecycle — live events" + [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) line 22 (Observability row) + line 44 (universal applicability).

## Why this exists

Concept doc 05 §"session lifecycle — live events" says operators observe a session's execution via a live event stream. Concept doc 03 line 44 says Observability is universal across all 9 fundamentals (every fundamental has `observe`/`log`/`attest`). Drift D7.1 (HIGH) recorded that no `/events` SSE endpoint existed; the CLI's `phi session launch` printed "(live tail deferred to M7)". With CH-02 (real `phi_core::agent_loop()` per ADR-0032) and CH-15 (real launch permission gate per ADR-0054) shipped, the prerequisite for a real SSE stream is in place: `BabyPhiSessionRecorder::on_phi_core_event` is the single funnel through which every `AgentEvent` flows. CH-17 attaches a `tokio::sync::broadcast::Sender<AgentEvent>` at that funnel and exposes a hard-deny-gated SSE handler at `GET /api/v0/sessions/:id/events`.

## 9-step SSE flow

```
GET /api/v0/sessions/:id/events
  ├── Step A — fetch the session row              → 404 SESSION_NOT_FOUND if absent
  ├── Step B — gather actor's grants              (same projection as launch)
  ├── Step C — build manifest                     [Observe] on session_object
  │              (sibling builder — NOT the launch builder)
  ├── Step D — engine::check() Steps 0–5          (Step 6 consent SKIPPED — launch-time only)
  │              ├── Allowed → continue
  │              └── Denied  → emit `platform.session.live_stream_denied`
  │                            return 403 PERMISSION_CHECK_FAILED_AT_STEP_<N>
  ├── Step E — fetch broadcast Sender             SessionLiveStreamRegistry.get(&id)
  │              ├── Some(tx) → continue
  │              └── None     → 410 SESSION_LIVE_STREAM_UNAVAILABLE (session finalised)
  └── Step F — subscribe via tx.subscribe()
                build BroadcastStream<AgentEvent>
                map(Ok|Err::Lagged) → SSE Event { event: "agent_event" | "lagged" }
                Sse::new(stream).keep_alive(30s ": keep-alive")
```

The SSE handler runs Steps 0–5 of the engine (same engine surface CH-15's launch handler uses). Step 6 (Per-Session consent) is intentionally skipped — observability of an already-launched session is not a fresh consent action; the consent gate is launch-time per [ADR-0048 §D48.5](../decisions/0048-per-session-consent-gating.md).

## `SessionLiveStreamRegistry` trait

```rust
pub trait SessionLiveStreamRegistry: Send + Sync {
    fn insert(&self, session_id: SessionId, tx: broadcast::Sender<AgentEvent>);
    fn get(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;
    fn remove(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}
```

Default impl `InProcessSessionLiveStreamRegistry { inner: DashMap<SessionId, broadcast::Sender<AgentEvent>> }` is the v0 production setting. Trait-shape mirrors `SessionRegistry` (CH-K8S-PREP / [ADR-0033 §D33.1](../decisions/0033-k8s-prep-refactors.md)) so the M7b cross-pod swap is a new impl, not a refactor — see [`CHK8S-D-10`](../../m7b/architecture/deferred-from-ch-k8s-prep.md#chk8s-d-10--cross-pod-live-event-fan-out-via-redis-pubsub-backed-sessionlivestreamregistry).

## Why `Action::Observe`, not `[Read, Inspect, List]`

Per [ADR-0055 §D55.5](../decisions/0055-sse-broadcast-fanout-and-keepalive.md): live-event tailing IS an observability operation per concept doc 03 line 22 (Observability vocabulary: `observe, log, attest`). Concept doc 05 line 242 names `read = "Retrieve a Session and its contents"` (full content fetch), which is what `GET /api/v0/sessions/:id` returns — a different surface from the SSE stream. `Action::Observe` is canonical pre-CH-17 (introduced at CH-04 / ADR-0043; `domain/src/permissions/action.rs:73,282,322`); CH-17 is the **first runtime exercise** of `Observe` on a session resource. `Action::CANONICAL.len() == 34` invariant preserved.

## Template A migration 0016

Pre-CH-17 Template A grants carried `[Read, Inspect, List]`. Post-CH-17 they mint `[Read, Inspect, List, Observe]` so a project lead can tail any session under their project without an explicit observe-grant. Migration `0016_template_a_session_object_grant_add_observe.surql` walks every legacy Template A grant (provenance: `descends_from(AR with kinds CONTAINS '#template:a')`; live: `revoked_at = NONE`) and appends `"observe"` to the `action` array idempotently. See [ADR-0055 §D55.9](../decisions/0055-sse-broadcast-fanout-and-keepalive.md).

## K8s readiness

Pod-local `tokio::sync::broadcast::channel(64)` per session. SSE clients connecting to pod A see only events from sessions hosted on pod A. Cross-pod fan-out deferred via [CHK8S-D-10](../../m7b/architecture/deferred-from-ch-k8s-prep.md) — M7b ships `RedisSessionLiveStreamRegistry` behind the same trait. Buffer = 64 absorbs typical multi-tool turns; lagged consumers receive a typed `lagged` SSE error and the stream closes (per [ADR-0055 §D55.3](../decisions/0055-sse-broadcast-fanout-and-keepalive.md)).

## Cross-references

- [ADR-0055 — SSE broadcast fan-out + keep-alive + per-session live-stream registry + observe-action gate](../decisions/0055-sse-broadcast-fanout-and-keepalive.md)
- [`session-launch-permission-gate.md`](./session-launch-permission-gate.md) — sibling-builder design (launch uses `[Read, Inspect, List]`; SSE uses `[Observe]`; both reuse the same `check()` engine surface)
- [`session-live-stream-operations.md`](../operations/session-live-stream-operations.md) — operator runbook
- [`m7b/architecture/deferred-from-ch-k8s-prep.md` CHK8S-D-10](../../m7b/architecture/deferred-from-ch-k8s-prep.md) — M7b cross-pod fan-out swap
- [`m5_1/drifts/D7.1.md`](../../m5_1/drifts/D7.1.md) — closed at CH-17
