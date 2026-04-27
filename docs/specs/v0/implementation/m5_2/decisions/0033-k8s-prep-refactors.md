<!-- Last verified: 2026-04-24 by Claude Code -->

# ADR-0033 — CH-K8S-PREP: four single-pod-safe refactors that pre-position the codebase for M7b microservices

**Status: Accepted** (flipped at CH-K8S-PREP chunk seal — P6, 2026-04-24).

Ratification evidence (per sub-decision):

| Sub-decision | Evidence |
|---|---|
| D33.1 (`SessionRegistry` trait) | `pub trait SessionRegistry` at [`server/src/state.rs`](../../../../../../modules/crates/server/src/state.rs) (5 methods: `insert`/`remove`/`len`/`is_empty`/`cancel_all`); `InProcessSessionRegistry` impl shipped; `Arc<dyn SessionRegistry>` plumbed across launch.rs, terminate.rs, main.rs (via `SharedSessionRegistry` local alias). Unit test `in_process_session_registry_round_trips_through_trait_object`. |
| D33.2 (`SurrealStore::open_remote`) | `pub async fn open_remote(uri, ns, db)` at [`store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs); shared `open_with_uri` private helper used by both constructors; `SurrealStore.db` switched to `Surreal<Any>` for runtime backend dispatch; config schema gains `[storage] mode = "embedded" \| "remote"` + `[storage.remote] uri = ...`. Unit tests `open_remote_dispatches_via_any_engine_and_runs_migrations` + `open_remote_returns_connect_error_for_unreachable_uri`. |
| D33.3 (SIGTERM graceful shutdown) | `pub async fn graceful_shutdown` at [`server/src/shutdown.rs`](../../../../../../modules/crates/server/src/shutdown.rs); `wait_for_shutdown_signal()` + axum-graceful-shutdown wiring at [`server/src/main.rs`](../../../../../../modules/crates/server/src/main.rs) (both plaintext + TLS paths); `SessionRegistry::cancel_all` trait method. Unit tests `empty_registry_drains_immediately`, `cancel_all_fires_every_token_and_drain_succeeds_when_tasks_remove_themselves`, `drain_timeout_reports_remaining_count_for_stuck_tasks`. Integration test `graceful_shutdown_against_live_app_state_drains_simulated_sessions` at [`server/tests/acceptance_shutdown.rs`](../../../../../../modules/crates/server/tests/acceptance_shutdown.rs). |
| D33.4 (`EventBus` shutdown + drain) | `async fn shutdown` + `async fn drain(timeout) -> Result<(), DrainError>` on the `EventBus` trait at [`domain/src/events/bus.rs`](../../../../../../modules/crates/domain/src/events/bus.rs); `InProcessEventBus` impl gains `is_shutdown: AtomicBool` + `in_flight: AtomicUsize` + `EmitGuard` RAII pattern; `main.rs` SIGTERM sequence calls `event_bus.shutdown().await` + `event_bus.drain(timeout).await` after `graceful_shutdown(registry)`. Unit tests `shutdown_drops_subsequent_emits_without_invoking_handlers`, `drain_returns_immediately_when_no_emits_are_in_flight`, `drain_waits_for_in_flight_emits_to_complete`. |

## Context

CH-02 just sealed (`phi_core::agent_loop` runs on `tokio::spawn` inside phi-server). Before opening the next M5 implementation chunk, the user asked for a research investment into the K8s-microservices deployment story: "what's compatible, what isn't, what to add/remove?" The 3-agent investigation produced [`m7b/architecture/k8s-microservices-readiness.md`](../../m7b/architecture/k8s-microservices-readiness.md) (the strategic input to M7b plan-open).

The user then chose option B from the planning question: **assessment doc + immediate prep refactors that pre-position code without breaking single-binary mode**. Four prep refactors fit that constraint:

1. **P-1 — Trait-shape `SessionRegistry`.** Pre-positions for the M7b Redis-backed shared registry (ADR-0031 §D31.1).
2. **P-2 — `SurrealStore::open_remote(uri)` constructor.** Pre-positions for externalized SurrealDB (readiness doc B3).
3. **P-3 — SIGTERM graceful shutdown handler.** Implements ADR-0031 §D31.5 (designed M5/P4, never shipped before now).
4. **P-4 — `EventBus` shutdown + drain semantics.** Pre-positions for the M7b broker-backed `EventBus` impl (readiness doc B2).

The chunk also produced two living documents in [`m7b/architecture/`](../../m7b/architecture/):
- [`k8s-microservices-readiness.md`](../../m7b/architecture/k8s-microservices-readiness.md) — strategic input (8 K8s blockers, 7 microservice boundaries, 10-step migration order, ~35 engineer-day rough estimate)
- [`deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md) — tactical input (8 specific items the prep refactors named M7b as the owner of)

This ADR ratifies the four prep refactors and binds the trait surfaces as the contract M7b's broker/Redis adapters will implement against.

## Decision

### D33.1 — `SessionRegistry` is a trait; `InProcessSessionRegistry` is the M5 default

`pub type SessionRegistry = Arc<DashMap<...>>` alias retired. Replaced with:

```rust
pub trait SessionRegistry: Send + Sync {
    fn insert(&self, session_id: SessionId, token: CancellationToken);
    fn remove(&self, session_id: &SessionId) -> Option<CancellationToken>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn cancel_all(&self);
}

pub struct InProcessSessionRegistry { inner: DashMap<SessionId, CancellationToken> }
```

`AppState.session_registry: Arc<dyn SessionRegistry>` typed against the trait. Every call site (launch, terminate, shutdown handler, acceptance harness) consumes the trait.

**Why this contract:** the 5 trait methods are the minimum surface needed by both single-pod (`InProcessSessionRegistry`) and multi-pod (the future `RedisSessionRegistry` per ADR-0031 §D31.1) backends. `cancel_all` is added now (not just at the M7b chunk) because the SIGTERM handler in D33.3 needs it.

**Conforming-impl criteria for M7b** (the "what does Redis-backed need to satisfy"):
1. `insert`/`remove`/`len`/`is_empty` semantics identical to `DashMap` for single-pod observers; `len` may be approximate for cross-pod observers (eventual consistency on the count is acceptable since it's a saturation gate, not a hard constraint).
2. `cancel_all` MUST broadcast cancellation to every pod (not just the local pod's locally-cached tokens) — this is [CHK8S-D-03](../../m7b/architecture/deferred-from-ch-k8s-prep.md) + [CHK8S-D-04](../../m7b/architecture/deferred-from-ch-k8s-prep.md).
3. Implementations MUST be `Send + Sync` and cheap to `Arc::clone` (the trait is consumed via `Arc<dyn SessionRegistry>`).

### D33.2 — `SurrealStore::open_remote(uri, ns, db)` ships now; `Surreal<Any>` becomes the canonical engine type

The store crate switches from `Surreal<Db>` (local-only) to `Surreal<Any>` (runtime-erased) — the SurrealDB SDK's official multi-backend dispatcher. Both constructors funnel through a private `open_with_uri` helper so the migration-runner code path is identical between modes.

Config schema:
```toml
[storage]
mode = "embedded"  # or "remote" — defaults to embedded so legacy configs round-trip
[storage.remote]
uri = ""           # required when mode = "remote"; e.g., "ws://surreal.svc:8000"
```

`main.rs` boot path switches via a new `open_store_for_config(cfg)` helper.

**Cargo features added** at workspace root: `surrealdb` features extended from `["kv-rocksdb"]` to `["kv-rocksdb", "kv-mem", "protocol-ws"]`. The `kv-mem` feature enables direct `Surreal::new::<Mem>()` for backend-specific tests; `protocol-ws` is the M7b production transport for `ws://` URIs.

**Conforming criteria for M7b** (what a remote URI must satisfy when wired in production):
1. SurrealDB ≥ 2.0 server-mode protocol on `ws://`, `wss://`, `http://`, or `https://`.
2. Migration runner runs once per pod startup; multi-pod startup races are NOT mitigated by this constructor — see [CHK8S-D-05](../../m7b/architecture/deferred-from-ch-k8s-prep.md) for the leader-election lock M7b adds.
3. The same `_migrations` ledger schema applies — operators upgrading from embedded to remote can dump/restore without re-running migrations.

### D33.3 — SIGTERM/SIGINT graceful shutdown ships as ADR-0031 §D31.5 implementation

`tokio::signal::unix::{SignalKind::terminate, SignalKind::interrupt}` are wired in `main.rs::wait_for_shutdown_signal()`. Both axum's `with_graceful_shutdown` (plaintext) and `axum_server::Handle::graceful_shutdown` (TLS) paths are wired. After axum returns (i.e., new HTTP requests stop + in-flight HTTP handlers drain), the SIGTERM handler:

1. Calls `server::shutdown::graceful_shutdown(registry, timeout)` — fires `cancel_all()` on the registry; polls `is_empty()` every 50ms until drained or `cfg.shutdown.timeout_secs` (default 30) expires.
2. Logs `Ok` or `Err(DrainTimeout { remaining })` outcome.
3. Per D33.4 sequencing, then runs `event_bus.shutdown()` + `event_bus.drain(timeout)`.
4. Process exits.

**What this DOES NOT do** (deferred to M7b per [CHK8S-D-01](../../m7b/architecture/deferred-from-ch-k8s-prep.md), [CHK8S-D-02](../../m7b/architecture/deferred-from-ch-k8s-prep.md), [CHK8S-D-03](../../m7b/architecture/deferred-from-ch-k8s-prep.md)):
- Hard-clear of orphan registry entries → `governance_state = FailedLaunch` (DB-write logic; M7b panic-safety per ADR-0031 §D31.4).
- Real-SIGTERM-with-real-launch acceptance test via subprocess fixture (current acceptance suite cannot send signals to itself; integration test uses synthetic stuck-task tokens instead).
- Multi-pod cluster-wide drain coordination (single-pod drain only at M5).

**Acceptance test** at [`acceptance_shutdown.rs`](../../../../../../modules/crates/server/tests/acceptance_shutdown.rs) exercises the live `AppState.session_registry` path with synthetic stuck-task tokens. Module-doc explains why real SIGTERM + real MockProvider mid-flight is M7b-fixture territory.

### D33.4 — `EventBus` gains `shutdown` + `drain` trait methods; `InProcessEventBus` adopts atomic-counter drain

`EventBus` trait extended:

```rust
async fn shutdown(&self);                         // idempotent flag-flip
async fn drain(&self, timeout: Duration)
    -> Result<(), DrainError>;                    // poll in-flights to 0 or timeout
```

`InProcessEventBus` gains:
- `is_shutdown: AtomicBool` — checked at the top of `emit()`; post-shutdown emits are dropped silently
- `in_flight: AtomicUsize` — incremented at emit-start (RAII via `EmitGuard`), decremented at emit-end
- `drain` polls `in_flight` every 50ms

**SIGTERM sequencing** in `main.rs` is critical: `event_bus.shutdown()` runs **after** `graceful_shutdown(registry)`, NOT before. This protects late session-finalisation events (recorder emits SessionEnded as agent_loop tasks unwind post-cancellation) from being silently dropped. Code comment in `main.rs` documents this + cross-references [CHK8S-D-07](../../m7b/architecture/deferred-from-ch-k8s-prep.md).

**Conforming criteria for M7b** (what a broker-backed `EventBus` must satisfy):
1. `shutdown()` MUST be idempotent and globally visible (every pod's `emit()` returns early after one pod calls `shutdown()` on the shared broker).
2. `drain(timeout)` MUST honor the timeout and return a `DrainError { remaining }` reflecting cluster-wide pending message count, not just the local pod's.
3. Audit-critical `DomainEvent` variants (TBD at M7b plan-open per [CHK8S-D-07](../../m7b/architecture/deferred-from-ch-k8s-prep.md)) MUST publish to a durable replay queue, not drop silently post-shutdown.
4. The trait stays `Send + Sync` and `Arc`-friendly.

**Tokio runtime dependency**: domain crate's `tokio` moved from `[dev-dependencies]` to `[dependencies]` for the drain poll-loop (`tokio::time::sleep`). Workspace tokio feature set unchanged.

## Consequences

**Positive**
- M7b's microservice carve-out becomes additive (new trait impls + a new ADR per backend choice) rather than a multi-file refactor across the storage / registry / event-bus layers.
- SIGTERM handling improves single-pod deployment behavior **immediately**: pre-CH-K8S-PREP, SIGTERM left in-flight sessions stuck in `running`; post-CH-K8S-PREP, they cancel cleanly within the configured grace period.
- The deferred-items ledger ([`deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md)) gives M7b plan-open a structured input — 8 specific items with provenance, owner sub-task, and cross-refs, rather than a re-discovery exercise.
- Trait-level decoupling makes acceptance tests cleaner (fakes can satisfy the trait without a real DB / Redis).

**Negative**
- One additional layer of indirection (trait dispatch) on hot paths: `SessionRegistry`, `EventBus`, `Surreal<Any>`. Negligible at single-pod scale; M7b can revisit if profiling surfaces an issue.
- SurrealDB feature footprint grows: `kv-mem` + `protocol-ws` add to the build closure even at single-pod deployments. Not removable without bifurcating the workspace.
- Domain crate now depends on `tokio` at runtime (was dev-only). Acceptable since downstream crates already pulled tokio in.
- `event_bus.shutdown()` silently drops post-shutdown emits — potentially loses audit-critical events. [CHK8S-D-07](../../m7b/architecture/deferred-from-ch-k8s-prep.md) names the M7b mitigation.

**Neutral**
- Storage `mode = "embedded"` stays the default — operators upgrading to M7b's K8s deployment opt in via `PHI_STORAGE__MODE=remote` + `PHI_STORAGE__REMOTE__URI=ws://…`.
- The `_keep_agent_loop_live` compile-time witness from CH-02 stays; complementary to the new `cancel_all` runtime exercise.

## Alternatives considered

- **Defer everything to M7b** (research-only assessment doc, no code refactors). Smallest scope. Rejected: the user explicitly chose option B (assessment + immediate refactors); SIGTERM specifically delivers immediate-value at single-pod (no longer "stuck running" sessions on pod restart).
- **Full M5.5 milestone proposal** (assessment + per-chunk decomposition of the entire microservice migration). Largest scope. Rejected at planning time: M7b is the right home for the actual carve-out; this chunk only pre-positions traits.
- **Different `SurrealStore` shape options**: (a) generic `SurrealStore<C: Connection>` — invasive, every `repo_impl` becomes generic; (b) two struct types `EmbeddedSurrealStore` + `RemoteSurrealStore` with parallel Repository impls — high duplication; (c) `Surreal<Any>` — chosen, official multi-backend type, near-zero call-site impact.
- **Hard-set the SIGTERM grace period to 30s**. Rejected: K8s' `terminationGracePeriodSeconds` is per-deployment; making the value config-driven (`PHI_SHUTDOWN__TIMEOUT_SECS`) lets operators tune.
- **`EventBus::drain` with no timeout parameter** (caller wraps in `tokio::time::timeout`). Rejected: different broker-backends will want different drain timeout strategies (e.g., a Kafka backend might want per-topic timeouts); putting `timeout` on the trait keeps the signature uniform.

## Review trigger

**M7b plan-open.** When M7b opens, the plan-author reads this ADR alongside [`k8s-microservices-readiness.md`](../../m7b/architecture/k8s-microservices-readiness.md) (strategic) + [`deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md) (tactical). Each conforming-criteria block (D33.1 §criteria, D33.2 §criteria, D33.4 §criteria) becomes the contract the M7b adapter chunks must satisfy.

This ADR may flip to `Superseded by ADR-NNNN` per sub-decision as M7b ships its specific backend choices (e.g., D33.1's `RedisSessionRegistry` ADR may supersede D33.1; D33.2's `SurrealCluster` config ADR may supersede D33.2).

## References

- [CH-K8S-PREP plan](../../../../plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md) — the chunk plan this ADR ratifies.
- [`m7b/architecture/k8s-microservices-readiness.md`](../../m7b/architecture/k8s-microservices-readiness.md) — strategic K8s-readiness assessment.
- [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md) — tactical deferred-items ledger (8 entries).
- [ADR-0031 — Session cancellation + concurrency bounds](../../m5/decisions/0031-session-cancellation-and-concurrency.md) — §D31.1 + §D31.4 + §D31.5 are the load-bearing references.
- [ADR-0032 — MockProvider at M5; real providers deferred to M7](./0032-mock-provider-at-m5.md) — sibling decision from CH-02 (the chunk that just preceded CH-K8S-PREP).
- [ADR-0028 — Domain event bus](../../m4/decisions/0028-domain-event-bus.md) — original `InProcessEventBus` rationale; D33.4 extends without superseding.
- [ADR-0029 — Session persistence and recorder wrap](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) — same wrap-with-governance pattern that D33.1 + D33.4 follow at the trait layer.
- [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core Leverage" — the wrap/reuse principle the trait designs honor.
