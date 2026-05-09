<!-- Last verified: 2026-05-09 by Claude Code (CH-17 P3 — operator runbook paired with ADR-0055 + the `m5_2/architecture/session-live-stream.md` design page; cycle hex 40c4d759.) -->

# Session live-event stream — operations

> **Status:** [EXISTS] as of CH-17. Endpoint: `GET /api/v0/sessions/:id/events`.

## Endpoint summary

| Field | Value |
|---|---|
| Method | `GET` |
| Path | `/api/v0/sessions/:id/events` |
| Content-Type (success) | `text/event-stream` |
| Permission gate | `Action::Observe` on `session_object` (sibling builder `build_session_observe_manifest`) |
| Keep-alive | 30 s (`: keep-alive`) |
| Channel buffer | 64 (config `[session_live_stream] buffer`) |
| Audit-event on deny | `platform.session.live_stream_denied` (Alerted-class) |

## Wire shape

Each emitted SSE event has shape:

```
event: agent_event
data: <serialised phi_core::types::event::AgentEvent JSON>
```

Lagged consumers receive:

```
event: lagged
data: {"missed": <count>}
```

After the lagged event the BroadcastStream returns `None`, closing the SSE connection. Operators reconnect — see "Reconnect playbook" below.

## Error codes

| HTTP | Code | Meaning | Operator action |
|---|---|---|---|
| 200 | `text/event-stream` | Live tail subscribed; events flow until session finalises or consumer disconnects | — |
| 403 | `PERMISSION_CHECK_FAILED_AT_STEP_<N>` | Actor lacks `Action::Observe` on `session_object`; `platform.session.live_stream_denied` audit event emitted before the response | Most common cause on a CH-15-era DB: legacy Template A grants lack `"observe"`. Run migration `0016_template_a_session_object_grant_add_observe.surql` (auto-applied by the migration runner per [ADR-0033 §D33.2](../decisions/0033-k8s-prep-refactors.md)). For non-Template-A actors, mint an explicit `[Observe]` grant via the appropriate authority-template flow. |
| 404 | `SESSION_NOT_FOUND` | Session id unknown | Verify the id via `GET /api/v0/sessions/:id` |
| 410 | `SESSION_LIVE_STREAM_UNAVAILABLE` | Session has finalised; broadcast Sender already removed from the registry | Use `GET /api/v0/sessions/:id` for the terminal `SessionDetail` |

## Reconnect playbook

When the SSE connection ends (lagged-then-close, network blip, LB timeout):

1. Check `GET /api/v0/sessions/:id` first — if `governance_state ∈ {Completed, Aborted, FailedLaunch}`, the session has finalised. No further events to tail.
2. If the session is still `Running`, reconnect to `/events`. The new connection sees only events emitted AFTER the resubscribe; events between the old close and the new connect are lost (no durable replay at v0 — see [CHK8S-D-10](../../m7b/architecture/deferred-from-ch-k8s-prep.md) for the M7b durable-replay path).
3. CLI consumers (`phi session launch` default tail) inherit the reconnect responsibility from the underlying SSE client — current behaviour is one connection per launch; reconnect is operator-driven.

## Lagged-receiver semantics

Buffer = 64 (per ADR-0055 §D55.2). Buffer absorbs:

- Multi-tool turns (≤ 10 events typical).
- Burst-emit windows when the agent loop produces TurnStart + content + tool_use + tool_result + TurnEnd in quick succession.

A consumer falls behind when its poll loop cannot keep up with the broadcast Sender's send rate. Common causes:

- Slow downstream (a CLI piping into a slow tail-formatter).
- HTTP/2 backpressure not propagating to the SSE stream.
- Operator's terminal scrolls slower than events arrive.

The handler catches `BroadcastStreamRecvError::Lagged(missed)` and emits a typed `lagged` SSE event with the missed count, then the stream closes. **No silent drops** — observers always see a `lagged` notification before disconnect.

## Audit-event impact

CH-17 introduces one new audit event: `platform.session.live_stream_denied` (Alerted-class). Emitted by `open_live_stream` whenever the engine returns `Decision::Denied`. Mirrors `platform.session.launch_denied` from CH-15 / [ADR-0054 §D54.5](../decisions/0054-session-launch-manifest-and-hard-deny-flip.md). Builder at `domain/src/audit/events/m5_2/session_live_stream.rs::session_live_stream_denied`.

Successful SSE connections emit ZERO audit events (read-only stream). Single-writer guarantee preserved per ADR-0054 §D54.5 precedent.

## LB / kube-proxy keep-alive

The `Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)).text(": keep-alive"))` configuration sends a comment-line ping every 30 s when no events flow. 30 s sits below typical LB idle timeouts (60 s default for kube-proxy / GCP / AWS ALB). If your deployment uses a sub-30s LB timeout, raise it in the LB config (or lower `[session_live_stream] keep_alive_secs` to a value under your LB timeout — config takes effect on next boot).

## Migration 0016 runbook

Migration `0016_template_a_session_object_grant_add_observe.surql` walks every legacy Template A grant (`descends_from(AR with kinds CONTAINS '#template:a')`; `revoked_at = NONE`) and appends `"observe"` to the `action` array. Forward-only + idempotent.

**Run order vs 0015:** 0016 mirrors 0015's body shape (UPDATE-with-array-push) and runs strictly AFTER 0015 per the migration-runner's lexicographic ordering. Both are idempotent + ledger-aware ([`_migrations` table](../../m1/operations/schema-migrations-operations.md) per ADR-0033 §D33.2). On a fresh DB the migration runner applies 0001..0016 in order; on a CH-15-era DB only 0016 needs to apply.

**Observability:** the migration emits no audit events (silent, per migration discipline). Operators verify via:

```surql
-- count Template A grants whose action array contains "observe"
SELECT count() FROM grant
  WHERE descends_from IN (SELECT id FROM auth_request WHERE kinds CONTAINS '#template:a')
    AND revoked_at = NONE
    AND action CONTAINS 'observe'
  GROUP ALL;
```

Pre-0016 the count is the number of grants minted post-CH-17 (only the production minter at `templates/a.rs:128` emits 4-action grants going forward). Post-0016 the count includes every legacy grant.

## Cross-references

- [ADR-0055 — SSE broadcast fan-out + keep-alive + per-session live-stream registry + observe-action gate](../decisions/0055-sse-broadcast-fanout-and-keepalive.md)
- [`session-live-stream.md`](../architecture/session-live-stream.md) — design page
- [`session-launch-operations.md`](../../m5/operations/session-launch-operations.md) — sibling launch endpoint runbook
- [CHK8S-D-10](../../m7b/architecture/deferred-from-ch-k8s-prep.md) — M7b cross-pod fan-out
- [Drift D7.1](../../m5_1/drifts/D7.1.md) — closed at CH-17
