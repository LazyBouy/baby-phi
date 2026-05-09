<!-- Last verified: 2026-05-09 by Claude Code (CH-17 chunk-seal — appended `CHK8S-D-10` entry under §3 + §2 Index; status totals bumped from 9 to 10. Origin: CH-17 P-1 trait-shape `SessionLiveStreamRegistry` at `server/src/state.rs:134–208`; M7b deliverable: NEW `RedisSessionLiveStreamRegistry` impl behind the same trait, swap-only refactor.) -->

# Deferred-from-CH-K8S-PREP — M7b items registry

**Type:** Architecture / scoping input
**Owning milestone:** M7b
**Sibling:** [`k8s-microservices-readiness.md`](./k8s-microservices-readiness.md) (the assessment doc)
**Source chunk:** [CH-K8S-PREP plan](../../../../plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md)

---

## 1. Purpose & scope fence

> ⚠ **This is a living ledger of items that the CH-K8S-PREP prep refactors (P-1..P-4) explicitly scoped OUT and named M7b as the owner.** Each entry was annotated in code or doc comments at the time it was deferred. M7b plan-open uses this list as a structured input alongside [`k8s-microservices-readiness.md`](./k8s-microservices-readiness.md). Items here are **not commitments** by the M7b team to ship every one — they are the "yes, M7b is the owner" list, with provenance.

Why a separate file (not a §10 in the readiness doc): the readiness doc is the **strategic** input (what the K8s carve-out looks like). This file is the **tactical** input (what concrete bits the prep work didn't ship). Keeping them separate prevents the readiness doc from growing per prep-refactor close.

## 2. Index — items at a glance

| ID | Title | Severity | Source prep refactor | M7b sub-task owner | Cross-refs |
|---|---|---|---|---|---|
| [CHK8S-D-01](#chk8s-d-01--hard-clear-orphan-registry-entries-on-sigterm-failedlaunch-flip) | Hard-clear orphan registry entries on SIGTERM (FailedLaunch flip) | HIGH | P-3 (SIGTERM handler) | M7b "audit-durability + orphan reconciliation" | ADR-0031 §D31.4; readiness doc §B4 |
| [CHK8S-D-02](#chk8s-d-02--real-sigterm-with-real-launch-acceptance-test-via-subprocess-fixture) | Real-SIGTERM-with-real-launch acceptance test (subprocess fixture) | MEDIUM | P-3 (SIGTERM handler) | M7b "operator runbook + acceptance hardening" | ADR-0031 §D31.5; readiness doc §B4 |
| [CHK8S-D-03](#chk8s-d-03--multi-replica-registry-drain-across-pods) | Multi-replica registry drain across pods | HIGH | P-3 (SIGTERM handler) | M7b "externalize SessionRegistry" (Step 2) | ADR-0031 §D31.1; readiness doc §B1, §B2 |
| [CHK8S-D-04](#chk8s-d-04--redis-backed-sessionregistry-impl-the-cross-pod-swap) | Redis-backed `SessionRegistry` impl (the cross-pod swap) | HIGH | P-1 (trait-shape SessionRegistry) | M7b "externalize SessionRegistry" (Step 2) | ADR-0031 §D31.1; readiness doc §B1 |
| [CHK8S-D-05](#chk8s-d-05--migration-runner-leader-election-lock-for-multi-replica-startup) | Migration runner leader-election lock for multi-replica startup | LOW | P-2 (`SurrealStore::open_remote`) | M7b "externalize storage" (Step 1) | readiness doc §B8 |
| [CHK8S-D-06](#chk8s-d-06--broker-backed-eventbus-impl-the-cross-pod-pubsub) | Broker-backed `EventBus` impl (the cross-pod pub/sub) | HIGH | P-4 (EventBus drain semantics) | M7b "externalize event bus" (Step 3) | ADR-0028; readiness doc §B2 |
| [CHK8S-D-07](#chk8s-d-07--durable-replay-queue-for-events-emitted-during-shutdown-window) | Durable replay queue for events emitted during shutdown window | MEDIUM | P-4 (EventBus drain semantics) | M7b "audit-durability + orphan reconciliation" | shares scope with D-01; readiness doc §B5 |
| [CHK8S-D-08](#chk8s-d-08--audit-emitter-shutdowndrain-symmetry-with-eventbus) | `AuditEmitter` shutdown/drain symmetry with `EventBus` | MEDIUM | P-4 (EventBus drain semantics) | M7b "audit-durability + orphan reconciliation" | shares scope with D-01, D-07; readiness doc §B5 |
| [CHK8S-D-09](#chk8s-d-09--consent-sweeper-leader-election-for-multi-pod-deployments) | Consent sweeper leader-election for multi-pod deployments | MEDIUM | CH-10 (consent state machine + sweeper) | M7b "leader-election + cross-pod schedulers" | ADR-0047 §D47.7 |
| [CHK8S-D-10](#chk8s-d-10--cross-pod-live-event-fan-out-via-redis-pubsub-backed-sessionlivestreamregistry) | Cross-pod live-event fan-out via Redis pub/sub-backed `SessionLiveStreamRegistry` | HIGH | CH-17 P-1 (trait-shape `SessionLiveStreamRegistry`) | M7b "externalize SessionLiveStreamRegistry" | ADR-0055 §D55.8; ADR-0033 §D33.1 precedent; readiness doc §B1, §B2 |

**Status totals (as of last verified):** 10 captured

## 3. Items (detail)

### CHK8S-D-01 — Hard-clear orphan registry entries on SIGTERM (FailedLaunch flip)

- **Severity:** HIGH
- **Source prep refactor:** P-3 (SIGTERM graceful shutdown handler), shipped in CH-K8S-PREP/P4.
- **Where the deferral is recorded in code:**
  - [`server/src/shutdown.rs`](../../../../../../modules/crates/server/src/shutdown.rs) — module doc paragraph beginning *"Sessions that fail to drain within `timeout` are reported via the [`DrainTimeout`] error..."* explicitly punts the FailedLaunch flip to M7b.
  - [`server/src/main.rs`](../../../../../../modules/crates/server/src/main.rs) — `tracing::warn!` on drain timeout names "M7b will hard-flip these to FailedLaunch on exit".
- **Description:** When `graceful_shutdown` times out with `remaining > 0`, the spawn tasks have either panicked or hung past the deadline. Their registry entries persist; their session rows in SurrealDB stay at `governance_state = Running` (a non-terminal state). The fix per [ADR-0031 §D31.4](../../m5/decisions/0031-session-cancellation-and-concurrency.md) panic-safety is a `scopeguard`-style hard-clear: iterate the remaining registry entries on shutdown-timeout, flip each session to `governance_state = FailedLaunch`, emit `platform.session.failed_launch` audit + `DomainEvent::SessionAborted { reason: "shutdown drain timeout" }`.
- **What needs to be added at M7b:**
  - A `Repository::mark_session_failed_launch_bulk(session_ids: &[SessionId])` method (or per-id loop with idempotency).
  - A new error variant or method on the shutdown module that takes `&Repository` and performs the flip.
  - Audit + DomainEvent emission for each orphan (governance plane sees the abort reason).
  - Acceptance test: simulate a panicked spawn task, run shutdown, verify the session row's terminal state.
- **M7b sub-task owner:** "Audit-durability + orphan reconciliation". Touches the same audit-stream rework noted at readiness doc §B5.
- **Cross-refs:** [ADR-0031 §D31.4](../../m5/decisions/0031-session-cancellation-and-concurrency.md), [readiness doc §B4](./k8s-microservices-readiness.md), governance-state machine docs at `m5/architecture/session-launch.md`.

---

### CHK8S-D-02 — Real-SIGTERM-with-real-launch acceptance test (via subprocess fixture)

- **Severity:** MEDIUM
- **Source prep refactor:** P-3 (SIGTERM graceful shutdown handler), shipped in CH-K8S-PREP/P4.
- **Where the deferral is recorded in code:**
  - [`server/tests/acceptance_shutdown.rs`](../../../../../../modules/crates/server/tests/acceptance_shutdown.rs) — file-level module doc explains *"Why we don't drive a real launch + real SIGTERM here"* and names M7b as the milestone that ships the subprocess fixture.
- **Description:** The current acceptance test (`graceful_shutdown_against_live_app_state_drains_simulated_sessions`) injects synthetic stuck-task tokens because:
  1. `MockProvider::text("Acknowledged.")` finishes a turn in sub-millisecond time → no observable mid-flight window from a same-process test.
  2. SIGTERM to the test process kills the harness (and the test runner).
- **What needs to be added at M7b:** a subprocess-spawning fixture that:
  - Starts `phi-server` as a child process bound to a free port + a tempdir DB.
  - Issues a real `POST /sessions` launch via HTTP.
  - Sends SIGTERM to the child PID.
  - Waits for the child to exit within `terminationGracePeriodSeconds`.
  - Reopens the DB and asserts the session row reaches a terminal state (`completed` or `aborted` — never stuck `running`).
  - Bonus: a slower MockProvider variant (e.g., `MockProvider::text_after_delay`) that gives the test a deterministic mid-flight window without depending on real network.
- **M7b sub-task owner:** "Operator runbook + acceptance hardening". Pairs with the M7b runbook write-up that documents the actual K8s pod-termination contract.
- **Cross-refs:** [ADR-0031 §D31.5](../../m5/decisions/0031-session-cancellation-and-concurrency.md), [readiness doc §B4](./k8s-microservices-readiness.md).

---

### CHK8S-D-03 — Multi-replica registry drain across pods

- **Severity:** HIGH
- **Source prep refactor:** P-3 (SIGTERM graceful shutdown handler), shipped in CH-K8S-PREP/P4.
- **Where the deferral is recorded in code:**
  - [`server/src/shutdown.rs`](../../../../../../modules/crates/server/src/shutdown.rs) — `cancel_all` doc paragraph: *"At M7b's broker-backed impl this fans out a 'cancel' message over the shared store so cancellation is global."*
  - [`server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs) — `SessionRegistry::cancel_all` doc-comment names M7b's broker-backed impl.
- **Description:** CH-K8S-PREP P-3 implements drain-on-SIGTERM for the **single-pod** in-process registry. In a K8s deployment with N replicas, each pod has its own in-memory `InProcessSessionRegistry` (or, post-D-04, its own connection to a shared Redis registry). When pod-A receives SIGTERM, it drains its own pod-local entries; sessions that were launched on pod-B but recorded in a shared Redis registry are NOT affected by pod-A's shutdown — and that's actually correct for rolling deployments.
  - But: when *the entire cluster* is being torn down (e.g., emergency stop, or redeploy with `--cascade=foreground`), every pod independently drains its locally-tracked entries. Cross-pod coordination matters when:
    1. A pod-B-launched session has a token cached in pod-A (impossible with `InProcessSessionRegistry`; possible via Redis cache invalidation patterns).
    2. The cluster-wide "drain everything" semantic — typically delegated to K8s' rolling-update strategy + `terminationGracePeriodSeconds`, but operators may want a global "shutdown all sessions" admin endpoint.
- **What needs to be added at M7b:**
  - The Redis-backed `SessionRegistry` impl (per D-04) should expose a `pub fn drain_via_broker(&self) -> impl Future` that publishes a "cancel" message to a Redis Stream; consuming pods watch the stream + cancel locally.
  - An admin endpoint `POST /api/v0/admin/shutdown-all-sessions` that fires the global drain (gated by an admin auth class).
  - Documentation: the operator runbook explains when single-pod vs cluster-wide drain is appropriate.
- **M7b sub-task owner:** "Externalize SessionRegistry" (Step 2 of the migration order in [readiness doc §7](./k8s-microservices-readiness.md)). Tightly coupled with D-04.
- **Cross-refs:** [ADR-0031 §D31.1](../../m5/decisions/0031-session-cancellation-and-concurrency.md), [readiness doc §B1, §B2](./k8s-microservices-readiness.md).

---

### CHK8S-D-04 — Redis-backed `SessionRegistry` impl (the cross-pod swap)

- **Severity:** HIGH
- **Source prep refactor:** P-1 (trait-shape `SessionRegistry`), shipped in CH-K8S-PREP/P2.
- **Where the deferral is recorded in code:**
  - [`server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs) — trait doc: *"Trait-shaped per CH-K8S-PREP P-1 / ADR-0033 so the M7b Redis-backed shared registry (per ADR-0031 §D31.1) can be a new impl rather than a multi-file refactor."* + impl doc: *"At M7b, a sibling `RedisSessionRegistry` impl will satisfy the trait against a shared store so cancellation tokens flow across pods."*
- **Description:** The trait surface (`insert / remove / len / is_empty / cancel_all`) is in place; what's missing is the actual Redis (or NATS / etcd / Consul — pick at M7b plan-open) backend that satisfies it across pods. The constructor signature is `pub fn new(connection_pool: ...) -> Self`; the implementation persists `(SessionId → CancellationToken-or-equivalent)` in a shared store with appropriate TTLs.
- **What needs to be added at M7b:**
  - Pick a backend (Redis is the typical default; matches ADR-0031 §D31.1's mention).
  - Implement the trait against the chosen backend: `insert` writes a row with TTL ≥ `session_max_lifetime`, `remove` deletes, `len` runs a `KEYS pattern` or maintains a counter, `cancel_all` publishes a Redis-Streams "cancel-all" message.
  - Wire `cancel_all` to a per-pod subscriber that calls `.cancel()` on locally-held tokens (since `CancellationToken` itself is in-process; the trait-level cancellation broadcasts via the broker, then each pod fires its local tokens).
  - Update `provider_for` in baby-phi to construct the chosen impl from `[storage.session_registry]` config.
  - Acceptance test: 2 pods sharing a Redis instance, launch on pod-A, terminate on pod-B, verify pod-A's spawn task gets cancelled.
- **M7b sub-task owner:** "Externalize SessionRegistry" (Step 2).
- **Cross-refs:** [ADR-0031 §D31.1](../../m5/decisions/0031-session-cancellation-and-concurrency.md), [readiness doc §B1](./k8s-microservices-readiness.md).

---

### CHK8S-D-05 — Migration runner leader-election lock for multi-replica startup

- **Severity:** LOW
- **Source prep refactor:** P-2 (`SurrealStore::open_remote(uri)`), shipped in CH-K8S-PREP/P3.
- **Where the deferral is recorded in code:**
  - [`store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs) — `open_remote` doc-comment: *"Multi-replica startup migration races are an M7b concern (per the readiness doc §B8) and are NOT addressed by this constructor."*
- **Description:** When N pods start concurrently against a shared remote SurrealDB (the M7b deployment shape), `run_migrations` runs N times in parallel. Each invocation reads the `_migrations` ledger, sees no entry for migration M, applies it, and writes the ledger row. SurrealDB's per-statement transactions partially mitigate (each migration is idempotent), but a slow apply on pod-A can fail-fast on pod-B with "field already defined" errors during the race window.
- **What needs to be added at M7b:**
  - An advisory lock acquired before `run_migrations`. Options: Redis SETNX with TTL; SurrealDB-native row-lock via `LET $lock = CREATE migration_lock:singleton SET …` with conflict-detection; K8s-native lease object (`coordination.k8s.io/Lease`).
  - The lock holder runs migrations; the others poll until either the lock releases (migrations done) or the holder dies (TTL expires) and the next pod takes over.
  - Acceptance test: 3 concurrent connections to the same remote DB calling `open_remote`; assert all return `Ok(())` and no "field already defined" errors surface.
- **M7b sub-task owner:** "Externalize storage" (Step 1).
- **Cross-refs:** [readiness doc §B8](./k8s-microservices-readiness.md).

---

### CHK8S-D-06 — Broker-backed `EventBus` impl (the cross-pod pub/sub)

- **Severity:** HIGH
- **Source prep refactor:** P-4 (EventBus drain semantics), shipped in CH-K8S-PREP/P5.
- **Where the deferral is recorded in code:**
  - [`domain/src/events/bus.rs`](../../../../../../modules/crates/domain/src/events/bus.rs) — `InProcessEventBus` doc-comment: *"Cross-process fan-out is a future addition (ADR-0028 §Future evolution; the M7b broker-backed impl per ADR-0033)."*
  - The new `shutdown(&self)` + `drain(&self, timeout)` trait methods are designed precisely so the broker-backed impl can override them with broker-specific semantics (publish a "stop subscriber group" message + await ack).
- **Description:** Sibling to [D-04](#chk8s-d-04--redis-backed-sessionregistry-impl-the-cross-pod-swap). The trait surface is in place after P-4; what's missing is an actual broker-backed impl. `InProcessEventBus` runs handlers inline within each `emit()` call, so cross-pod fan-out is impossible — pod-A's emit only triggers pod-A's locally-registered listeners. The 5 M5 listeners (Template A/C/D, MemoryExtraction, AgentCatalog) all need to fire on every relevant event regardless of which pod emitted it.
- **What needs to be added at M7b:**
  - Pick a broker (Redis Streams / NATS JetStream / Kafka — Redis is the typical default, matches D-04's Redis adapter for shared infra).
  - Implement the `EventBus` trait against the chosen broker. Each `emit()` publishes to a topic per `DomainEvent` variant (or a single fan-out topic with type-discriminating consumers).
  - Listener-side: each pod runs a consumer subscribed to the topics. The 5 M5 listeners become workers as separate Deployments (per [readiness doc §4 boundary C](./k8s-microservices-readiness.md)).
  - `shutdown` publishes a "stop accepting new" sentinel; `drain` awaits the broker's pending-message-count to hit zero.
  - Acceptance test: 2 pods + a shared broker, emit on pod-A, assert listener fires on pod-B.
- **M7b sub-task owner:** "Externalize event bus" (Step 3 of the migration order in [readiness doc §7](./k8s-microservices-readiness.md)).
- **Cross-refs:** ADR-0028 (in-process bus rationale + future-evolution note); [readiness doc §B2](./k8s-microservices-readiness.md).

---

### CHK8S-D-07 — Durable replay queue for events emitted during shutdown window

- **Severity:** MEDIUM
- **Source prep refactor:** P-4 (EventBus drain semantics), shipped in CH-K8S-PREP/P5.
- **Where the deferral is recorded in code:**
  - [`domain/src/events/bus.rs`](../../../../../../modules/crates/domain/src/events/bus.rs) — `EventBus::emit` trait doc: *"return early (no-op) when [`shutdown`] has been called — emits arriving post-shutdown are dropped silently. M7b's broker-backed impl will optionally buffer to a durable replay queue; the in-process default does not."*
  - Same file, `InProcessEventBus::emit` body: *"CH-K8S-PREP P-4 — drop late events post-shutdown. Audit durability of dropped events is M7b scope (see deferred-from-ch-k8s-prep.md)."*
- **Description:** When `event_bus.shutdown()` fires (typically from the SIGTERM handler in `main.rs`), every subsequent `emit()` becomes a no-op. For events that are merely operational (e.g., `HasLeadEdgeCreated` — drives Template A grant generation, which is also derivable on next pod startup by re-scanning the graph), silent drop is acceptable. For audit-critical events (e.g., `SessionAborted` from a recorder finalising during the shutdown window), silent drop loses information that's required for the audit hash-chain (per [readiness doc §B5](./k8s-microservices-readiness.md)).
  - There's a sequencing protection in [main.rs](../../../../../../modules/crates/server/src/main.rs): `event_bus.shutdown()` runs AFTER `graceful_shutdown(registry)`, so most session-lifecycle events fire before the bus shuts down. But this protects only the spawn-task emit path; if a future M7b chunk adds emit sources that aren't drained by `graceful_shutdown(registry)`, those events would silently drop.
- **What needs to be added at M7b:**
  - Distinguish "operational" vs "audit-critical" events at the `DomainEvent` level (e.g., a marker trait or enum-variant flag).
  - For audit-critical events, the broker-backed `emit()` MUST publish to a durable topic that survives pod restarts; on recovery, replay any unconsumed events.
  - Operational events can continue to drop silently post-shutdown.
  - Tie into the M7b audit-durability + off-pod append-only-stream work (B5 from readiness doc).
- **M7b sub-task owner:** "Audit-durability + orphan reconciliation". Shares scope with [D-01](#chk8s-d-01--hard-clear-orphan-registry-entries-on-sigterm-failedlaunch-flip) (FailedLaunch flip is itself an audit-critical event that this replay queue would protect).
- **Cross-refs:** [readiness doc §B5](./k8s-microservices-readiness.md), ADR-0013 (audit class hash-chain), [D-01](#chk8s-d-01--hard-clear-orphan-registry-entries-on-sigterm-failedlaunch-flip).

---

### CHK8S-D-08 — `AuditEmitter` shutdown/drain symmetry with `EventBus`

- **Severity:** MEDIUM
- **Source prep refactor:** P-4 (EventBus drain semantics) — surfaced during P5 wiring, shipped CH-K8S-PREP/P5.
- **Where the deferral is recorded:** This document only — no per-file code comment yet (the `AuditEmitter` trait was not modified during CH-K8S-PREP since audit hash-chain semantics intersect with the M7b audit-durability rework anyway).
- **Description:** P-4 added `shutdown` + `drain` to the `EventBus` trait but did not add equivalent methods to the `AuditEmitter` trait at [`domain/src/audit/`](../../../../../../modules/crates/domain/src/audit/). Asymmetry: `event_bus.shutdown()` fires; the SurrealAuditEmitter keeps accepting writes via `audit.emit(...)` synchronously into the DB. At M5 / single-pod this is fine — `SurrealAuditEmitter` writes are synchronous and complete before the `audit.emit` future returns, so they're naturally drained by axum's request-handler shutdown.
  - At M7b's multi-pod + off-pod audit-stream architecture (per [readiness doc §B5](./k8s-microservices-readiness.md)), `AuditEmitter` impls become asynchronous (write to a queue, ack later). At that point the trait NEEDS the same `shutdown` + `drain` semantics for symmetry — without them, in-flight audit writes can be lost on pod termination.
- **What needs to be added at M7b:**
  - Mirror `shutdown(&self)` + `drain(&self, timeout) -> Result<(), DrainError>` on the `AuditEmitter` trait.
  - In-process / synchronous SurrealAuditEmitter: drain is a no-op (same as InProcessEventBus drain effectively).
  - Off-pod audit-stream emitter: drain awaits acknowledgement of all enqueued writes from the durable backend.
  - Wire `audit.shutdown()` + `audit.drain(timeout)` into the SIGTERM sequence between `event_bus.drain` and process exit.
- **M7b sub-task owner:** "Audit-durability + orphan reconciliation". Pairs with [D-07](#chk8s-d-07--durable-replay-queue-for-events-emitted-during-shutdown-window) and [D-01](#chk8s-d-01--hard-clear-orphan-registry-entries-on-sigterm-failedlaunch-flip).
- **Cross-refs:** [readiness doc §B5](./k8s-microservices-readiness.md); ADR-0013 (audit class hash-chain).
- **Note on provenance:** unlike D-01..D-07 (each cited from a code comment authored in the prep refactor), D-08 is a *gap* the prep refactor noticed but didn't address. It would not surface from a code grep — only from reading the EventBus changes and asking "what about the sibling AuditEmitter trait?". Recorded here so M7b plan-open doesn't miss it.

---

### CHK8S-D-09 — Consent sweeper leader-election for multi-pod deployments

- **Severity:** MEDIUM
- **Source prep refactor:** CH-10 / ADR-0047 §D47.7 — the consent state-machine sweeper.
- **Where the deferral is recorded:** [`server::state::spawn_consent_sweeper`](../../../../../../modules/crates/server/src/state.rs) doc-comment + the [`consent`] block in [`config/default.toml`](../../../../../../config/default.toml).
- **Description:** CH-10 ships a tokio task ([`server::state::spawn_consent_sweeper`](../../../../../../modules/crates/server/src/state.rs)) that periodically scans for past-deadline `Requested` consents and flips them to `TimedOut`, emitting one `consent.timed_out` audit per flip. The task runs on every pod that boots `phi-server`. In a single-pod deployment (the M5 / v0 production target) this is correct: one sweeper, one audit per consent flip. In a multi-pod deployment, every pod runs its own sweeper — the storage UPDATE is idempotent (SurrealDB's `WHERE state = 'requested'` guards against the second flip), but the audit-event creation is NOT idempotent. Each pod that observes the eligible row would emit a duplicate `consent.timed_out` audit before the storage UPDATE fails for the second pod. This breaks the audit hash chain's "one event per state transition" invariant.
  - The CH-10 plan §3.B A1 row records this as a single-pod-only constraint at v0; ADR-0047 §D47.7 documents it; the deferral lands here so M7b plan-open finds it.
- **What needs to be added at M7b:**
  - **Option A — leader-election lock**: a leader-election primitive (Redis SETNX with TTL, etcd lease, or SurrealDB equivalent) that elects exactly one pod as the consent-sweeper holder per tick. Non-leader pods skip the sweep. Tradeoff: ~2-3 days to wire + a new infrastructure dependency.
  - **Option B — externalised scheduler**: an off-pod cron job (Kubernetes `CronJob` or equivalent) calls a single `POST /api/v0/internal/sweep-consent-timeouts` endpoint at the configured interval. The pod that handles the HTTP request runs the sweep. Tradeoff: simpler infra, depends on the deployment topology managing the schedule.
  - **Option C — accept duplicate audits**: relax the "one audit per transition" invariant to "at least one"; the audit hash chain becomes a partial-order log. Tradeoff: invasive — affects every existing audit-driven invariant.
  - The expected M7b path is **Option A** — pairs naturally with [`CHK8S-D-04`](#chk8s-d-04--redis-backed-sessionregistry-impl-the-cross-pod-swap) (Redis-backed SessionRegistry) which would already be in scope, so the leader-election primitive is shared infra.
- **Disable knob:** the chunk ships `[consent] sweeper_interval_secs = 0` as a disable flag — multi-pod operators wanting to defer the sweeper entirely (e.g. while the sweep is run by an external cron) set this to 0 on every pod.
- **M7b sub-task owner:** "Leader-election + cross-pod schedulers". Pairs with [D-04](#chk8s-d-04--redis-backed-sessionregistry-impl-the-cross-pod-swap).
- **Cross-refs:** [ADR-0047](../../m5_2/decisions/0047-consent-state-machine-and-sweeper.md) §D47.7; [drift D-new-05](../../m5_1/drifts/D-new-05.md) (closed at CH-10).

---

### CHK8S-D-10 — Cross-pod live-event fan-out via Redis pub/sub-backed `SessionLiveStreamRegistry`

- **Severity:** HIGH
- **Source prep refactor:** CH-17 P-1 (trait-shape `SessionLiveStreamRegistry`), shipped 2026-05-09. Cycle hex `40c4d759`.
- **Originating chunk:** [CH-17 plan](../../../../plan/build/ch-17-live-sse-tail-endpoint-40c4d759/plan.md).
- **Where the deferral is recorded in code:**
  - [`server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs) — `SessionLiveStreamRegistry` trait doc-comment names *"Single-pod-only at v0. Live events produced on pod A are not visible to SSE clients connected to pod B without Redis pub/sub fan-out — see CHK8S-D-10 in `m7b/architecture/deferred-from-ch-k8s-prep.md`"*.
  - [`server/src/platform/sessions/events.rs`](../../../../../../modules/crates/server/src/platform/sessions/events.rs) — module doc names the cross-pod boundary.
- **Description:** CH-17 ships a `tokio::sync::broadcast::Sender<phi_core::AgentEvent>` per session held in a pod-local `DashMap<SessionId, broadcast::Sender>` ([`InProcessSessionLiveStreamRegistry`](../../../../../../modules/crates/server/src/state.rs)). The session-launch handler creates the channel and registers the Sender immediately before spawning the agent task. The recorder's `on_phi_core_event` funnel publishes every event onto the channel. SSE handlers `subscribe()` on a clone of the Sender and stream events out as `data:` JSON lines.
  - **Pre-CH-17 state:** no live SSE endpoint → no cross-pod concern (drift D7.1 documented the absent surface).
  - **Post-CH-17 state:** pod-local `tokio::broadcast` channel; SSE clients connected to pod A see only events that pod A's recorder produces. An SSE consumer connected to pod A who wants to tail a session hosted on pod B sees nothing — the fan-out does not cross the pod boundary.
  - **Why deferred:** trait-shaping the registry is the M5-affordable abstraction; building the cross-pod broker is M7b scope (multi-pod is the K8s-deployment target). The trait shape ensures the M7b swap is a new impl, not a refactor — every callsite (launch handler, SSE handler, recorder construction) consumes `Arc<dyn SessionLiveStreamRegistry>` and is impl-agnostic.
- **What needs to be added at M7b:**
  - A `RedisSessionLiveStreamRegistry` impl behind the same `SessionLiveStreamRegistry` trait. `insert(session_id, tx)` registers the Sender locally **and** opens a Redis pub/sub channel (`baby-phi:live-stream:<session_id>`) over which other pods' Senders publish; the local Sender's task `XADD`s incoming pub/sub messages so local SSE consumers see the merged stream.
  - Alternatively, a `RedisStreamSessionLiveStreamRegistry` using Redis Streams (`XADD` + `XREAD` BLOCKING) for durable replay if SSE clients reconnect after a brief drop.
  - Configurable channel-key prefix (`[session_live_stream] redis_channel_prefix = "baby-phi:live-stream"`) so multi-tenant K8s deployments can isolate.
  - Acceptance test: simulate two pods (process A + process B), launch a session on A, subscribe to `/events` on B, observe that B's stream contains A's events.
  - **Swap-only refactor:** no callsite changes — the boot site at `server/src/main.rs` chooses between `new_session_live_stream_registry()` (in-process default) and `new_redis_session_live_stream_registry(config)` (M7b cross-pod). Every other call to `Arc<dyn SessionLiveStreamRegistry>` is impl-agnostic.
- **M7b sub-task owner:** "Externalize SessionLiveStreamRegistry". Pairs with [CHK8S-D-04](#chk8s-d-04--redis-backed-sessionregistry-impl-the-cross-pod-swap) (Redis-backed SessionRegistry) and [CHK8S-D-06](#chk8s-d-06--broker-backed-eventbus-impl-the-cross-pod-pubsub) — sharing the Redis infra primitive.
- **Cross-refs:** [ADR-0055 §D55.8](../../m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md); [ADR-0033 §D33.1 precedent](../../m5_2/decisions/0033-k8s-prep-refactors.md); [readiness doc §B1, §B2](./k8s-microservices-readiness.md); [CH-17 plan §3.B](../../../../plan/build/ch-17-live-sse-tail-endpoint-40c4d759/plan.md).

---

## 4. Adding new entries (CH-01+)

**Codified by CH-01 / forward-scope §7 Q8 (2026-04-27).** From CH-01 onward, the per-chunk-planning-template's [§3.B "K8s microservice readiness check"](../../m5_1/process/per-chunk-planning-template.md) is the canonical source of new entries to this ledger. When a chunk's §3.B 7-axis evaluation surfaces a new K8s blocker the chunk does not address, a new `CHK8S-D-NN` entry MUST be added here before the chunk seals.

**Numbering.** Pick the next free number after the current highest in §3 (current high: `CHK8S-D-10`). Numbering is monotonic; no gaps.

**Required fields per entry** (mirror the existing CHK8S-D-01 through CHK8S-D-08 shape):
- **Item title** — one line, summarising the deferral.
- **Provenance** — exact file + line where the deferral was originally noted (code comment, test module doc, etc.) OR the chunk plan §3.B row that surfaced it.
- **Why deferred** — short paragraph: technical reason the chunk did not address it.
- **What M7b owes** — clear sub-task description: what M7b needs to deliver.
- **Cross-references** — links to relevant ADRs, sibling drifts, concept docs.
- **Originating chunk** — `CH-NN` plan token + path. New for CH-01+ entries (CH-K8S-PREP entries pre-date this convention but their provenance is implicit via the prep refactors).

**Provenance discipline.** Every CH-01+ entry's *Originating chunk* field cites the chunk plan path (e.g., `build/ch-01-agent-durable-lifecycle-2aa37c80.md`). This makes back-tracing the deferral history mechanical.

**Index update.** Add the new entry to §2 Index alongside the existing rows. Keep the index sorted by entry number (ascending).

**Example workflow.** A future chunk's §3.B 7-axis table identifies a new in-process `OnceCell` cache that becomes pod-local. The chunk author cannot trait-shape it within the chunk's scope. They:
1. Pick `CHK8S-D-11` (next free).
2. Write the entry with provenance citing the chunk plan §3.B row + the file:line of the new `OnceCell`.
3. Add a row to §2 Index.
4. Reference `CHK8S-D-11` from the chunk plan's §3.B conclusion paragraph ("Chunk introduces 1 new K8s blocker; filed as CHK8S-D-11").
5. Chunk seals as K8s-negative (one new blocker), with the ledger entry the durable record.

## 5. Closing scope fence reminder

> ⚠ **(Reminder, repeats §1)** This file lists items the prep refactors named M7b as the owner of, in the comments / docs at the time of the deferral. It is the structured input to M7b plan-open — not a commitment list. M7b's plan-author may consolidate, re-prioritise, or convert items as they design that milestone's scope. Use this file to **find** the items, not to **dictate** how M7b ships them.

The items here will eventually transition to one of:
- **scoped-into-M7b-chunk** — the M7b plan opens a chunk that includes this item.
- **renegotiated** — an ADR amends scope so the item is reframed.
- **accepted-as-is** — explicit user approval that the deferral becomes permanent (rare).

Per the [drift-lifecycle](../../m5_1/process/drift-lifecycle.md) discipline applied to drift files, these items follow the same pattern at M7b: each gets a state transition recorded as M7b chunks consume them.
