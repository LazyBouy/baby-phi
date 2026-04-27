<!-- Last verified: 2026-04-24 by Claude Code -->

# K8s-microservices readiness — research precursor for M7b

**Type:** Architecture / forward-look
**Owning milestone:** M7b (production hardening)
**Inputs:**
- Concept docs at [`v0/concepts/`](../../../concepts/)
- Forward-scope inventory: [`plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md)
- This doc's executable counterpart: [`plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md`](../../../../plan/forward-scope/ab49f22b-k8s-microservices-readiness-plan.md)
- 3 parallel Explore-agent investigation reports (process / persistence / operational surfaces)

---

## 1. Scope fence — this is a precursor, not the M7b plan

> ⚠ **This document is a research-only precursor.** A more thorough K8s microservice evaluation, the actual carve-out into separate Deployments, and the development of any new services (Redis adapter, broker-backed `EventBus`, externalized `SessionRegistry`, off-pod audit stream, OAuth + session store, Helm chart, runbook, etc.) is **explicitly deferred to milestone M7b** (production hardening per [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) § Scope). Nothing in this document should be read as a commitment to ship multi-replica deployment before M7b. The four prep refactors that ship alongside this doc (P-1..P-4 in ADR-0033) are deliberately scoped to be single-pod-safe and do not constitute a K8s rollout.

The intent is to capture the assessment now so M7b plan-open has a head start, and to land the trait-level prep refactors that prevent the M7b carve-out from becoming a multi-file refactor.

---

## 2. Current K8s-friendly surfaces

baby-phi at M5 already does a lot right for container-orchestrated deployment. The following surfaces require **no change** to be K8s-compatible:

| Surface | Status | Evidence |
|---|---|---|
| 12-factor config | ✅ | TOML defaults + profile overrides + `PHI_*` env vars at [`server::config`](../../../../../../modules/crates/server/src/config.rs); double-underscore for nested keys |
| Secrets | ✅ | `PHI_MASTER_KEY` (env var, base64) drives AES-256-GCM at-rest sealing; no file deps. [`store::crypto`](../../../../../../modules/crates/store/src/crypto.rs) |
| Logs | ✅ | stdout JSON in prod, pretty in dev; `tracing-subscriber` + `EnvFilter`. [`server::telemetry`](../../../../../../modules/crates/server/src/telemetry.rs) |
| Health probes | ✅ | `/healthz/live` (always 200) + `/healthz/ready` (pings DB; 503 on failure). [`server::health`](../../../../../../modules/crates/server/src/health.rs) |
| Metrics | ✅ | `axum-prometheus` layer + `/metrics` endpoint. [`server::router`](../../../../../../modules/crates/server/src/router.rs) |
| Container image | ✅ | Multi-stage Dockerfile, debian:bookworm-slim runtime, non-root UID 10001, tini for signal relay. [`Dockerfile`](../../../../../../Dockerfile) |
| Migrations | ✅ | Forward-only, idempotent, ledger-tracked in `_migrations` table. [`store::migrations`](../../../../../../modules/crates/store/src/migrations.rs) |
| Persistence interfaces | ✅ | `Repository`, `EventBus`, `AuditEmitter` are all traits — backend swap is impl-only |
| CLI ↔ server coupling | ✅ | CLI is HTTP-client-only via `PHI_API_URL`; multi-process safe. [`cli::main`](../../../../../../modules/crates/cli/src/main.rs) |
| Web ↔ server coupling | ✅ | Next.js Server Actions via `fetch()` to phi-server; SSR only, no CORS needed inside the cluster |

These surfaces are M7b-ready as-is.

---

## 3. K8s blockers for multi-replica deployment

Eight blockers, ordered by severity. Each names the M7b mitigation path.

### B1 (CRITICAL) · `SessionRegistry` is pod-local

- **Where:** [`server/src/state.rs:31`](../../../../../../modules/crates/server/src/state.rs)
  ```rust
  pub type SessionRegistry = Arc<DashMap<SessionId, CancellationToken>>;
  ```
- **Symptom:** Session launched on pod-A cannot be terminated from pod-B (their `DashMap`s are independent heap objects).
- **Mitigation:** Replace with a trait + Redis-backed impl (or any KV with pub/sub for cancellation broadcast). Already acknowledged in [ADR-0031 §D31.1](../../m5/decisions/0031-session-cancellation-and-concurrency.md).
- **Prep refactor (P-1, lands now):** trait-shape the registry so the Redis adapter is a new impl, not a multi-file refactor.

### B2 (CRITICAL) · `InProcessEventBus` listener fan-out is pod-local

- **Where:** [`domain/src/events/bus.rs:50`](../../../../../../modules/crates/domain/src/events/bus.rs)
  ```rust
  pub struct InProcessEventBus {
      handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
  }
  ```
- **Symptom:** A `DomainEvent` emitted on pod-A only triggers the 5 listeners (Template A/C/D fire, MemoryExtraction, AgentCatalog) registered in pod-A's `Vec`. Pod-B's listeners never fire for that event.
- **Mitigation:** Replace `InProcessEventBus` with a broker adapter (Redis Streams / NATS / Kafka — pick at M7b plan-open). The trait surface stays.
- **Prep refactor (P-4, lands now):** add `shutdown` + `drain` to the trait so a future broker-backed impl ships with consistent shutdown semantics.

### B3 (CRITICAL) · SurrealDB embedded-only

- **Where:** [`store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs) — only `SurrealStore::open_embedded(path, ns, db)` exists; no networked URI path.
- **Symptom:** Each pod has its own RocksDB on its own PVC; data is not shared. Multi-replica is impossible.
- **Mitigation:** Add `open_remote(uri)` constructor + config schema for `[storage] mode = "embedded" | "remote"`. SurrealDB SDK supports `surrealdb::engine::remote::ws::Ws` out-of-box.
- **Prep refactor (P-2, lands now):** ship the `open_remote` constructor + config knob; default stays `embedded` so dev/CI is unchanged.

### B4 (HIGH) · No SIGTERM handler

- **Where:** [`server::main`](../../../../../../modules/crates/server/src/main.rs) — `axum::serve()` binds a listener directly; no `with_graceful_shutdown` registration; no `tokio::signal::unix` handler.
- **Symptom:** Pod termination sends SIGTERM → process keeps running → 30s grace expires → SIGKILL → in-flight `agent_loop` tasks die with the pod → `governance_state` stays `running` in SurrealDB → operator sees stuck sessions.
- **Mitigation:** Implement [ADR-0031 §D31.5](../../m5/decisions/0031-session-cancellation-and-concurrency.md) (already designed, never shipped).
- **Prep refactor (P-3, lands now):** wire the handler. Improves single-pod deployment behavior **immediately**.

### B5 (HIGH) · Audit hash-chain is single-writer

- **Where:** [`store/src/audit_emitter.rs`](../../../../../../modules/crates/store/src/audit_emitter.rs) — synchronous DB write per event with hash-chain linkage.
- **Symptom:** Multi-pod writes break the chain (two pods compute `prev_hash` from the same row → divergent chains). Replication safety is compromised.
- **Mitigation:** Split into a "synchronous fast path" (still hash-chained per-pod) + an "async durable replication" stream to S3/GCS object-lock or a CQRS log. M7b scope.

### B6 (MEDIUM) · Cookie-JWT only; no server-side revocation

- **Where:** [`server::session`](../../../../../../modules/crates/server/src/session.rs) — HS256 symmetric, claims `{ sub, iat, exp }`, `HttpOnly + SameSite=Lax`, 12h TTL. No server-side session table.
- **Symptom:** Logout / revocation is TTL-only. Multi-replica auth works (shared symmetric secret) but compromised tokens cannot be force-expired before TTL.
- **Mitigation:** Server-side session store (Redis-backed) + `jti` claim for revocation lookups. M7b scope; aligns with the planned OAuth 2.0 work.

### B7 (MEDIUM) · No request-id / OTEL trace propagation

- **Where:** [`server::router`](../../../../../../modules/crates/server/src/router.rs) — `tracing` is wired but no `tower_http::trace::TraceLayer` middleware, no W3C trace-context extraction, no OpenTelemetry exporter.
- **Symptom:** Cross-service request tracing (governance API → execution runner → listener worker) is impossible at multi-microservice scale.
- **Mitigation:** Add `tower-http` trace layer + `tracing-opentelemetry` + W3C `traceparent` header extraction. M7b scope.

### B8 (LOW) · Migration runner has no leader-election lock

- **Where:** [`store/src/migrations.rs`](../../../../../../modules/crates/store/src/migrations.rs) — `run_migrations()` checks the ledger and applies pending migrations.
- **Symptom:** N pods starting concurrently race the first-apply. SurrealDB's per-statement transactions partially mitigate (each migration is idempotent), but a slow apply on pod-A can fail-fast on pod-B with "field already defined" errors.
- **Mitigation:** Add an advisory lock (Redis SETNX with TTL, or DB row-lock) before `run_migrations`. M7b scope.

---

## 4. Natural microservice boundaries

The current monolith decomposes cleanly along these lines (lettered for cross-reference):

- **A. Governance API** — auth, CRUD, policy, web UI backend. The bulk of M2-M5 surfaces.
- **B. Execution runner** — owns `spawn_agent_task` + `phi_core::agent_loop` + recorder lifecycle. Receives launch commands via internal API/queue; reports back via event bus.
- **C. Listener workers** — currently 5 in-process listeners ([`server::main:90-94`](../../../../../../modules/crates/server/src/main.rs)): TemplateA/C/D fire, MemoryExtraction (stub), AgentCatalog (stub). Each becomes a separate Deployment subscribed to the broker.
- **D. Audit log writer** — single-writer service for hash-chain integrity, plus an off-pod append-only durability stream.
- **E. Storage tier** — externalized SurrealDB cluster (or any conforming backend per [ADR-0032 §D32.3](../../m5_2/decisions/0032-mock-provider-at-m5.md)).
- **F. Web UI** — already a separate Next.js process.
- **G. CLI** — already separate; HTTP-client-only; multi-process safe.

```
Operator (browser)  →  Ingress  →  F. Web UI  ─┐
Operator (terminal) →             G. CLI  ────┤
                                                ├→  A. Governance API  (N replicas)
                                                │      │
                                                │      ├─ launch cmd ──→ B. Execution runner (N replicas)
                                                │      ├─ DomainEvent ──→ Broker  (Redis / NATS / Kafka)
                                                │      │                     │
                                                │      │                     ├→ C1. TemplateA listener
                                                │      │                     ├→ C2. TemplateC listener
                                                │      │                     ├→ C3. TemplateD listener
                                                │      │                     ├→ C4. MemoryExtraction listener
                                                │      │                     └→ C5. AgentCatalog listener
                                                │      └─ audit write ──→ D. Audit log writer  ─→  S3/GCS object-lock
                                                │
                                                └→ E. Storage tier  (SurrealDB cluster)
```

---

## 5. What to add (concrete components, all M7b scope unless noted)

- Trait-level `SessionRegistry` interface + Redis-backed impl
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

---

## 6. What to remove or change (M7b scope unless noted)

- `InProcessEventBus` — keep as test-only behind a feature flag; default impl in production becomes broker-backed
- Migration runner → add leader-election lock for first-apply race during multi-replica startup
- `session_max_concurrent` cap (currently per-replica) → either accept replica-local-multiplied behavior or move to a global Redis counter
- Hardcoded 30s grace assumption → configurable, properly wired to SIGTERM handler
- `SecretRef::new("vault://...")` placeholder pattern → replaced by real vault adapter (HashiCorp Vault / AWS KMS / GCP Secret Manager) at M7b
- Single-writer audit hash-chain → split into "synchronous fast path" (DB) + "async durable replication" (object-lock stream)

---

## 7. Migration order (low-risk first, 10 steps — all M7b scope)

1. Externalize storage (`open_remote` constructor — already prep-shipped via P-2)
2. Externalize session registry (Redis adapter — trait already prep-shipped via P-1)
3. Externalize event bus (broker adapter — drain trait already prep-shipped via P-4)
4. Wire graceful shutdown (already prep-shipped via P-3)
5. Split listener workers as separate Deployments (5 of them today)
6. Externalize session store (Redis-backed cookie revocation list)
7. Split execution runner from API gateway (gRPC/HTTP launch cmd + cancel cmd)
8. Scale API gateway to N replicas
9. Audit log durability (off-pod append-only stream)
10. OAuth 2.0 + OTEL + per-org rate limiting + Helm chart + runbook

Steps 1–4 are partially pre-shipped via the four prep refactors (P-1 to P-4). At M7b plan-open, those steps reduce to "wire the new impl to the existing trait surface" rather than a full refactor.

---

## 8. Effort sketch

Rough; refined at M7b plan-open. This doc is the **input** to that planning, not the **commitment**.

| Step group | Effort | Notes |
|---|---|---|
| Steps 1-4 (low-risk externalization) | ~10 engineer-days | Halved if P-1..P-4 prep refactors land first |
| Steps 5-7 (microservice splits) | ~15 engineer-days | Includes execution-runner gRPC/HTTP layer |
| Steps 8-10 (production polish) | ~10 engineer-days | OAuth + OTEL + rate limiting + Helm + runbook |
| **Total** | **~35 engineer-days** | Comparable to a full M-milestone, executed inside M7b |

---

## 9. Items the prep refactors explicitly deferred to M7b

A live ledger of items the CH-K8S-PREP P-1..P-4 prep refactors scoped OUT and named M7b as the owner is maintained at [`deferred-from-ch-k8s-prep.md`](./deferred-from-ch-k8s-prep.md) (sibling file in this directory). Each entry includes provenance (file + line where the deferral was originally noted), a description, the M7b sub-task owner, and cross-references.

That file is the **tactical** scoping input to M7b plan-open; this readiness doc is the **strategic** input. Read both at M7b plan-open.

## 10. Recommendation + scope-fence reminder

**Recommendation:** defer the carve-out to M7b. Single-pod deployment with the four prep refactors landed (P-1..P-4 per ADR-0033) is sufficient for v0.1's expected load.

If single-pod throughput becomes a real bottleneck before M7b opens, that triggers a renegotiation: the user opens an early-mini-milestone (e.g., M5.5) using this doc as input. **No K8s implementation work happens off the back of this doc alone** — every code change in Steps 1-10 needs its own per-chunk plan inside M7b's scope, following the M5.1/P4 chunk-lifecycle discipline.

> ⚠ **Closing reminder (repeats §1 scope fence):** this is a research-only precursor. The actual K8s microservice evaluation, carve-out, and new-service development is **explicitly deferred to milestone M7b**. The four prep refactors (P-1..P-4) that ship alongside this doc are single-pod-safe and do not constitute a K8s rollout.
