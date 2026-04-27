<!-- Last verified: 2026-04-24 by Claude Code -->

# Plan: K8s-microservices readiness assessment + prep refactors

## Context

baby-phi is at M5 with **CH-02 just sealed** (`phi_core::agent_loop` runs on `tokio::spawn` inside phi-server today). The project's roadmap places production-hardening at **M7b** (per `baby-phi/CLAUDE.md` § Scope) — that's where multi-replica K8s deployment naturally lives. But the user wants two outputs ahead of that milestone:

1. **A concrete understanding** of how K8s-ready baby-phi is today + what to add/remove to support a microservices deployment.
2. **A small slice of low-risk prep refactors** that pre-position the code for that future split without breaking the current single-pod model.

The 3 parallel Explore agents that just ran confirmed the state in detail. This plan turns those findings into one assessment doc + four trait-level refactors. Total: ~3 engineer-days.

## What this plan delivers

### Deliverable 1 — Assessment document

**Path:** `baby-phi/docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md`
(creates new dir; treats this as an M7b precursor doc — aligns with the existing per-milestone `architecture/` convention used across m1/m2/m3/m4/m5.)

**Contents (≈9 sections):**

1. **Scope fence — this is a precursor doc, not the M7b plan.** Top-of-doc callout (rendered as a `> ⚠ Note:` block or similar prominent format). Verbatim language to include:

   > **This document is a research-only precursor.** A more thorough K8s microservice evaluation, the actual carve-out into separate Deployments, and the development of any new services (Redis adapter, broker-backed `EventBus`, externalized `SessionRegistry`, off-pod audit stream, OAuth + session store, Helm chart, runbook, etc.) is **explicitly deferred to milestone M7b** (production hardening per `baby-phi/CLAUDE.md` § Scope). Nothing in this document should be read as a commitment to ship multi-replica deployment before M7b. The four prep refactors that ship alongside this doc (P-1..P-4 in ADR-0033) are deliberately scoped to be single-pod-safe and do not constitute a K8s rollout.

   Same callout repeats at the end of the doc as a closing reminder.

2. **Current K8s-friendly surfaces** — 12-factor config (`config/default.toml` + `PHI_*` env vars at [`server::config`](baby-phi/modules/crates/server/src/config.rs)), env-driven secrets + AES-256-GCM master key ([`store::crypto`](baby-phi/modules/crates/store/src/crypto.rs)), stdout JSON logs ([`server::telemetry`](baby-phi/modules/crates/server/src/telemetry.rs)), `/healthz/{live,ready}` ([`server::health`](baby-phi/modules/crates/server/src/health.rs)), Prometheus `/metrics` ([`server::router`](baby-phi/modules/crates/server/src/router.rs)), multi-stage Dockerfile + non-root UID 10001, idempotent migrations ([`store::migrations`](baby-phi/modules/crates/store/src/migrations.rs)), all persistence behind traits (`Repository`, `EventBus`, `AuditEmitter`).

3. **K8s blockers for multi-replica deployment** (5 critical, 3 secondary):
   - **B1 (CRITICAL).** `SessionRegistry: Arc<DashMap<SessionId, CancellationToken>>` at [`server/src/state.rs:31`](baby-phi/modules/crates/server/src/state.rs) — pod-local cancellation tokens. Session launched on pod-A cannot be terminated from pod-B. ADR-0031 §D31.1 acknowledges + defers to M7b.
   - **B2 (CRITICAL).** `InProcessEventBus: RwLock<Vec<Arc<dyn EventHandler>>>` at [`domain/src/events/bus.rs:50`](baby-phi/modules/crates/domain/src/events/bus.rs) — pod-local listener fan-out. Compound-tx events emitted on pod-A do not trigger listeners on pod-B.
   - **B3 (CRITICAL).** SurrealDB embedded-only — `SurrealStore::open_embedded()` is the only constructor. No networked URI path.
   - **B4 (HIGH).** No SIGTERM handler at [`server::main`](baby-phi/modules/crates/server/src/main.rs) — pod kill leaves in-flight sessions stuck in `running`. ADR-0031 §D31.5 designed but **never shipped**.
   - **B5 (HIGH).** Audit hash-chain is synchronous single-writer ([`store::audit_emitter`](baby-phi/modules/crates/store/src/audit_emitter.rs)) — multi-pod writes break the chain.
   - **B6 (MEDIUM).** Cookie-JWT only (HS256) at [`server::session`](baby-phi/modules/crates/server/src/session.rs) — no server-side revocation list. Multi-replica behind ingress works for verification (shared symmetric secret) but logout is TTL-only.
   - **B7 (MEDIUM).** No request-id / OTEL trace propagation — `tracing` is wired but no `tower_http::trace::TraceLayer` middleware.
   - **B8 (LOW).** Migration runner has no leader-election lock — multi-replica startup races first-apply.

4. **Natural microservice boundaries** (7):
   - **A. Governance API** — auth, CRUD, policy, web. The bulk of M2-M5.
   - **B. Execution runner** — owns `spawn_agent_task` + `phi_core::agent_loop` + recorder lifecycle. Receives launch commands via internal API/queue; reports back via event bus.
   - **C. Listener workers** (5 today): TemplateA/C/D fire, MemoryExtraction (stub), AgentCatalog (stub). Each becomes a separate Deployment subscribing to the broker.
   - **D. Audit log writer** — single-writer for hash-chain integrity, plus an off-pod append-only durability stream.
   - **E. Storage tier** — externalized SurrealDB cluster (or any conforming backend per ADR-0032 D32.3).
   - **F. Web UI** — already a separate Next.js process.
   - **G. CLI** — already separate; HTTP-client-only; multi-process safe.

5. **What to add (concrete components — all targeted to land at M7b unless noted):**
   - Trait-level `SessionRegistry` interface + Redis-backed impl (M7b)
   - Broker-backed `EventBus` impl (Redis Streams / NATS / Kafka — pick at M7b plan-open)
   - `SurrealStore::open_remote(uri)` constructor + config schema for a URI pattern
   - Server-side session store (Redis-backed) for cookie revocation
   - SIGTERM handler implementing ADR-0031 §D31.5 (drain registry on shutdown)
   - OpenTelemetry tracing layer + W3C trace-id propagation through HTTP middleware
   - Per-org / per-user rate limiting (token bucket via Redis or LRU)
   - Off-pod audit append-only stream (S3/GCS object-lock or CQRS log)
   - Service-to-service auth (mTLS or SA-JWT) for API ↔ execution-plane split
   - K8s manifests / Helm chart (Deployment + Service + PVC + ConfigMap + Secret + Ingress + HPA)
   - Operator runbook (deploy / upgrade / rollback / backup / restore / incident-response)

6. **What to remove or change (M7b scope unless noted):**
   - `InProcessEventBus` — keep as test-only behind a feature flag; default impl in production becomes broker-backed
   - Migration runner → add leader-election lock for first-apply race during multi-replica startup
   - `session_max_concurrent` cap (currently per-replica) → either accept replica-local-multiplied behavior or move to a global Redis counter
   - Hardcoded 30s grace assumption → configurable, properly wired to SIGTERM handler
   - `SecretRef::new("vault://...")` placeholder pattern → replaced by real vault adapter (HashiCorp Vault / AWS KMS / GCP Secret Manager) at M7b
   - Single-writer audit hash-chain → split into "synchronous fast path" (DB) + "async durable replication" (object-lock stream)

7. **Migration order (low-risk first, 10 steps — all in M7b scope):** externalize storage → externalize session registry → externalize event bus → wire graceful shutdown → split listener workers → externalize session store → split execution runner → API gateway scale to N → audit log durability → OAuth + OTEL + rate limiting + Helm chart.

8. **Effort sketch** (rough; refined at M7b plan-open — this doc is the input, not the commitment):
   - Steps 1-4 (low-risk externalization): ~10 engineer-days
   - Steps 5-7 (microservice splits): ~15 engineer-days
   - Steps 8-10 (production polish): ~10 engineer-days
   - **Total: ~35 engineer-days** — comparable to a full M-milestone, executed inside M7b.

9. **Recommendation + scope-fence reminder** — defer the carve-out to M7b. If single-pod throughput becomes a real bottleneck pre-M7b, that triggers a renegotiation: the user opens an early-mini-milestone using this doc as input. **No K8s implementation work happens off the back of this doc alone** — every code change in Steps 1-10 needs its own per-chunk plan inside M7b's scope.

### Deliverable 3 — Place this plan verbatim in `forward-scope/`

**Path:** `baby-phi/docs/specs/plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md`
(8hex token `ab49f22b` generated via `openssl rand -hex 4`; sibling to the existing `22035b2a-remaining-scope-post-m5-p7.md`.)

**Action:** copy this `/root/.claude/plans/sharded-discovering-stearns.md` file verbatim into the path above. Same content, no edits at copy-time. Adds a `<!-- Last verified: YYYY-MM-DD by Claude Code -->` header as line 1 to satisfy `check-doc-links.sh`. Otherwise byte-identical.

**Why:** the user explicitly requested this. Rationale: forward-scope is the right home for plans that describe future work without committing to it; this plan is a sibling to M5.1's forward-scope inventory. Having the plan checked into the repo means future readers (and the M7b plan-author) can find it from `git log` + the existing `forward-scope/` index.

**Side note:** because the plan file describes its own deliverables in first-person (`### Deliverable 1 — Assessment document`, etc.), the published copy reads naturally as a plan artefact. The "Deliverable 3" entry — this entry — describes its own publication, which is consistent with the rest of the document's self-referential plan-structure.

### Deliverable 2 — Four prep refactors (~2 engineer-days)

All four are **single-pod-safe** (no infrastructure additions, no behavior change in dev/CI, no broken acceptance tests). They pre-position trait surfaces so the M7b carve-out becomes additive (new impls) rather than a multi-file refactor.

#### P-1 · Trait-shape `SessionRegistry` (~0.5d)

- Replace the `pub type SessionRegistry = Arc<DashMap<SessionId, CancellationToken>>` alias at [`server/src/state.rs:31`](baby-phi/modules/crates/server/src/state.rs) with `pub trait SessionRegistry: Send + Sync` (4 methods: `insert`, `remove`, `get`, `len`) + `pub struct InProcessSessionRegistry { inner: DashMap<...> }` impl.
- Update 3-4 call sites ([`platform/sessions/launch.rs`](baby-phi/modules/crates/server/src/platform/sessions/launch.rs), [`terminate.rs`](baby-phi/modules/crates/server/src/platform/sessions/terminate.rs), state-builder in [`main.rs`](baby-phi/modules/crates/server/src/main.rs)).
- Adds 1 unit test confirming trait-object dispatch + DashMap impl semantics intact.
- **Why now:** ADR-0031 §D31.1 explicitly defers a Redis-backed shared registry to M7b. Trait-shaping makes that swap a new impl, not a refactor.

#### P-2 · Add `SurrealStore::open_remote(uri)` constructor (~0.5d)

- New constructor in [`store::lib`](baby-phi/modules/crates/store/src/lib.rs) sibling to `open_embedded`. Real impl using `surrealdb::engine::remote::ws::Ws` (the SDK supports both backends out-of-box; this is genuinely shippable).
- Config schema: add `[storage] mode = "embedded" | "remote"` and `[storage.remote] uri = "..."` to [`server::config`](baby-phi/modules/crates/server/src/config.rs); default stays `embedded` so dev/CI is unchanged.
- 1 integration test: spin up a transient SurrealDB server in-memory mode + connect via `open_remote` + run smoke migrations. (May skip if test-infra cost is too high; alternative is a smaller compile-only test.)
- **Why now:** the API surface gets defined; M7b just selects `mode = "remote"` in prod config. Avoids a future pod-restart-wide refactor.

#### P-3 · Wire SIGTERM graceful shutdown handler (~0.5d)

- Implements ADR-0031 §D31.5 (already specified, never shipped).
- Add a `tokio::signal::unix` listener in [`main.rs`](baby-phi/modules/crates/server/src/main.rs) that on SIGTERM:
  1. Stops accepting new requests (axum's `with_graceful_shutdown`).
  2. Iterates `session_registry` + calls `cancel()` on every token.
  3. Waits up to a configurable timeout (default 30s, env `PHI_SHUTDOWN__TIMEOUT_SECS`) for spawned tasks to drain.
  4. Marks any still-running session as `governance_state = FailedLaunch` before exit.
- Adds 1 acceptance test that launches a session, sends SIGTERM mid-flight, asserts the session ends in a terminal state (not stuck `running`).
- **Why now:** Improves single-pod deployment behavior **immediately** (currently SIGKILL leaves stuck sessions). M7b doesn't have to re-design this — just verify it scales.

#### P-4 · `EventBus` drain semantics + listener-stop trait (~0.5d)

- The `EventBus` trait at [`domain/src/events/bus.rs`](baby-phi/modules/crates/domain/src/events/bus.rs) already abstracts well. Add two new methods: `async fn shutdown(&self)` (signals all listeners to stop accepting new events) and `async fn drain(&self) -> Result<(), DrainError>` (awaits in-flight handler completion up to a timeout).
- Default impls: in-process bus's `shutdown` flips an `AtomicBool` checked by `emit`; `drain` joins any pending handler tasks.
- Wire `shutdown` + `drain` into the SIGTERM handler from P-3.
- 1 unit test for each new method.
- **Why now:** A future Redis Streams / NATS / Kafka adapter naturally has these semantics. Defining them in the trait now means the broker-backed impl ships with consistent shutdown behavior.

## Critical files

**Created (Deliverable 1):**
- `baby-phi/docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md`

**Created (Deliverable 3):**
- `baby-phi/docs/specs/plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md` (verbatim copy of this plan + `Last verified` header)

**Modified (Deliverable 2):**
- `baby-phi/modules/crates/server/src/state.rs` (P-1 trait + impl)
- `baby-phi/modules/crates/server/src/platform/sessions/launch.rs` (P-1 trait method calls + P-3 token cancel during shutdown)
- `baby-phi/modules/crates/server/src/platform/sessions/terminate.rs` (P-1 trait method calls)
- `baby-phi/modules/crates/server/src/main.rs` (P-1 instantiation + P-3 SIGTERM handler wire)
- `baby-phi/modules/crates/server/src/config.rs` (P-2 storage mode + P-3 timeout config)
- `baby-phi/modules/crates/store/src/lib.rs` (P-2 `open_remote` constructor)
- `baby-phi/modules/crates/domain/src/events/bus.rs` (P-4 trait additions + InProcessEventBus impl)
- `baby-phi/modules/crates/server/tests/acceptance_sessions_m5p4.rs` (P-3 SIGTERM acceptance test addition)

**Touched docs:**
- `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md` — new ADR-0033 documenting P-1..P-4 scope + rationale + future M7b connection. Status: Accepted at chunk seal.
- `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` — append a §3 entry pointing to the new M7b precursor doc.

## Existing functions/utilities to reuse

- `Repository`, `EventBus`, `AuditEmitter` traits — already in place; just extend.
- `tower_http`, `tokio_util::sync::CancellationToken`, `tokio::signal::unix`, `tracing` — all already in `Cargo.toml`.
- ADR-0031 §D31.5 design — copy-ready spec for P-3.
- `wait_for_session_finalised` test helper in [`acceptance_sessions_m5p4.rs`](baby-phi/modules/crates/server/tests/acceptance_sessions_m5p4.rs) — reusable for P-3 acceptance test.

## Phases

| Phase | Goal | Effort |
|---|---|---|
| P1 | Write the assessment doc (Deliverable 1) + place this plan verbatim at `forward-scope/ab49f22b-...md` (Deliverable 3) | ~1d |
| P2 | P-1: Trait-shape `SessionRegistry` | ~0.5d |
| P3 | P-2: `SurrealStore::open_remote(uri)` + config schema | ~0.5d |
| P4 | P-3: SIGTERM graceful shutdown handler | ~0.5d |
| P5 | P-4: EventBus shutdown + drain semantics | ~0.5d |
| P6 | Draft ADR-0033, append forward-scope §3 entry, run 1 audit agent, seal | ~0.5d |

**Total: ~3.5 engineer-days.** Each phase closes per the M5.1/P4 checklist (4-aspect close + 2 confidence %).

## Verification

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1
# Expect: 975 (CH-02 baseline) + ~5 new tests (P-1, P-3, P-4) ≈ 980

# 3. Prep-refactor positive greps
grep -rn "trait SessionRegistry" modules/crates/server/src/state.rs       # ≥ 1 (P-1)
grep -rn "open_remote" modules/crates/store/src/lib.rs                    # ≥ 1 (P-2)
grep -rn "tokio::signal::unix\|signal::ctrl_c" modules/crates/server/     # ≥ 1 (P-3)
grep -rn "fn shutdown\|fn drain" modules/crates/domain/src/events/bus.rs  # ≥ 2 (P-4)

# 4. Assessment doc landed (Deliverable 1)
ls docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md   # 1
grep -c "K8s-BLOCKER\|K8s blocker" docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md   # ≥ 5
# Scope-fence note must appear at top AND bottom of the doc:
grep -c "deferred to milestone M7b\|deferred to M7b" docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md   # ≥ 2

# 4b. Forward-scope verbatim copy landed (Deliverable 3)
ls docs/specs/plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md   # 1
# Confirm verbatim (modulo the prepended Last-verified header line):
diff <(tail -n +3 docs/specs/plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md) /root/.claude/plans/sharded-discovering-stearns.md  # expect empty diff

# 5. ADR-0033 Accepted
grep -c "^\- \*\*Status\*\*: Accepted\|^\*\*Status: Accepted\*\*" docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md   # 1
```

## What this plan does NOT do

- No microservice carve-out (deferred to M7b — this plan only pre-positions traits + ships immediate-value SIGTERM handler).
- No actual broker / Redis / Vault wiring.
- No K8s manifests or Helm chart.
- No M7b plan re-scoping (the assessment doc informs M7b plan-open; doesn't replace it).
- No code that breaks single-pod deployment.

## Recommendation

After this plan ships, baby-phi will have:
- A clear K8s-readiness scorecard for future M7b plan-open
- Trait surfaces for `SessionRegistry`, `EventBus.shutdown/drain`, `SurrealStore::open_remote` already in place
- Graceful shutdown actually working today (not just designed in ADR-0031)
- ADR-0033 documenting which prep refactors landed when

When the user opens M7b, the precursor doc + ADR-0033 give the M7b plan a head start; the trait surfaces mean Redis/broker/remote-DB adapters are net-new files rather than multi-file refactors.
