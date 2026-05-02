<!-- Last verified: 2026-05-02 by Claude Code -->

# ADR-0047 — Consent state machine + per-transition repo surface + auto-timeout sweeper

**Status: Accepted**

**Date:** 2026-05-02
**Chunk:** CH-10
**Closes:**
- [`D-new-05`](../../m5_1/drifts/D-new-05.md) (HIGH) — Consent lifecycle state machine missing.

**Cross-chunk dependencies:**
- Builds on [ADR-0045](0045-consent-node-full-shape.md) (CH-09 — `ConsentState` enum + `state` field on `Consent`).
- Builds on [ADR-0028](../../m4/decisions/0028-domain-event-bus.md) (event-bus fail-safe semantics — durable write before any reactive emit).
- Builds on [ADR-0042](0042-storage-backend-configurable.md) §D42.3 #5 (forward-only idempotent migration runner).
- Mirrors the [`auth_requests::state` + `auth_requests::transitions`](../../../../../../modules/crates/domain/src/auth_requests/) precedent for module structure.
- Records new K8s deferral [`CHK8S-D-09`](../../m7b/architecture/deferred-from-ch-k8s-prep.md#chk8s-d-09--consent-sweeper-leader-election-for-multi-pod-deployments) (single-pod sweeper).

---

## Context

CH-09 lifted Consent from a 5-field stub to the 11-field shape mandated by `concepts/permissions/06-multi-scope-consent.md` §"Consent Node", including a `state: ConsentState` field. But the `state` field was dead data — there was no validated way to move a Consent through the lifecycle the concept doc spells out (`Requested → Acknowledged / Declined / TimedOut → Revoked / Expired`), no enforcement of the forward-only revocation invariant, and no auto-timeout sweep. Drift D-new-05 (HIGH) tracked this gap; CH-11 (Per-Session consent gating) is hard-blocked on the state machine landing.

The exploration phase found:

- The `auth_requests` module already establishes a clean pattern: `state.rs` (pure predicates + legal-transition table), `transitions.rs` (per-operation pure-fns returning `Result<T, TransitionError>`). CH-10 mirrors this.
- The 6 `ConsentState` variants compose into a small legal-transition table (4 outbound from `Requested`, 2 from `Acknowledged`, 0 from the four terminal states) — easily expressible as a `match` and exhaustively testable across the 36-cell matrix.
- The forward-only revocation invariant from concept doc 06 §"Forward-only revocation" can be enforced at the legal-transition table level (`Revoked → _` returns `false` for everything) so every code path that consults the table inherits the property.
- The auto-timeout sweep depends on a `deadline_at: Option<DateTime<Utc>>` field that doesn't exist on `Consent` yet. Adding it is a one-field migration. Population at consent creation is policy-aware logic that belongs to CH-11's per-policy minter rewrite.

The user-decided forks at plan-review (2026-04-30) locked three open questions:

1. **Module layout** — Dedicated module (`domain::consents::{mod.rs, state.rs, transitions.rs}`) mirroring `auth_requests`. Pure-fn transitions returning `Result<Consent, ConsentTransitionError>`.
2. **Time-driven sweeper** — Ship in CH-10. The sweeper is a `Repository::sweep_consent_timeouts(now) → Vec<ConsentId>` method PLUS a tokio task at server startup that calls it on a configurable interval.
3. **Repository surface** — Per-transition methods: `acknowledge_consent`, `decline_consent`, `revoke_consent`, `mark_consent_timed_out`, `mark_consent_expired`. Five methods, each carrying its own audit-event helper.

---

## Decision

### D47.1 — Dedicated `domain::consents` module mirroring the `auth_requests` precedent

New module `domain::consents` at [`modules/crates/domain/src/consents/`](../../../../../../modules/crates/domain/src/consents/):

- `mod.rs` — module declaration + re-exports of `is_terminal`, `legal_transition`, the five transition pure-fns, and `ConsentTransitionError`.
- `state.rs` — pure predicates: `is_terminal(ConsentState) -> bool` (returns `true` for `Declined / Revoked / TimedOut / Expired`); `legal_transition(from: ConsentState, to: ConsentState) -> bool` carrying the 6-arrow legal table from concept doc 06 §"Consent Lifecycle" verbatim. Self-transitions return `false` (unlike `auth_requests::aggregate_request_state`, where re-aggregation is idempotent — Consent transitions don't have a re-aggregation step).
- `transitions.rs` — five per-operation pure-fn helpers: `acknowledge`, `decline`, `revoke`, `mark_timed_out`, `mark_expired`. Each takes `(&Consent, DateTime<Utc>)` and returns `Result<Consent, ConsentTransitionError>`. State never mutates in place — the function clones, validates, stamps the appropriate timestamp field, and returns the new row. Per locked Q1.

### D47.2 — `ConsentTransitionError` with two variants

```rust
pub enum ConsentTransitionError {
    IllegalTransition { from: ConsentState, to: ConsentState },
    NotRevocable { consent_id: ConsentId },
}
```

`IllegalTransition` covers terminal stickiness, forward-only revocation, and every other illegal pair. `NotRevocable` fires when `Acknowledged → Revoked` is attempted on a `revocable=false` consent. Per locked Q2.

The plan originally specified three variants (`IllegalTransition`, `NotRevocable`, `Terminal { state, attempted }`); implementation found that `Terminal` was redundant — the legal-transition table catches every terminal-state outbound attempt as an `IllegalTransition`, so `Terminal` would have been an unreachable third path. Two variants suffice; the simpler shape is preferred.

### D47.3 — Five per-transition Repository methods

Per locked Q3:

```rust
async fn acknowledge_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
async fn decline_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
async fn revoke_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
async fn mark_consent_timed_out(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
async fn mark_consent_expired(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
```

Each method's compound tx: SELECT-read the row → call the matching pure-fn at `domain::consents::transitions` → if `Ok`, write the new row + emit the audit event atomically (BEGIN / UPDATE / CREATE / COMMIT on SurrealDB; HashMap mutation on the in-memory backend). Returns the audit event id so callers can pivot back to the audit row. Failures inside the pure-fn surface as `RepositoryError::ConsentTransition { source: ConsentTransitionError }` — handlers map this to HTTP `409 Conflict`.

### D47.4 — Five new audit-event builders at `domain::audit::events::m5::consents`

`consent_acknowledged`, `consent_declined`, `consent_revoked`, `consent_timed_out`, `consent_expired`. All emit `AuditClass::Logged` — routine relationship-mutation traffic; `Alerted` would flood the chain. The diff carries `(consent_id, from_state, to_state, at)` plus the timestamp field that was set by the transition (`responded_at` for ack/decline, `revoked_at` for revoke, none for timed_out/expired). `org_scope` populated from `consent.scope.org`. `consent_expired` accepts the caller-supplied `from_state` since `Expired` is reachable from both `Requested` and `Acknowledged`.

### D47.5 — Sixth Repository method: `sweep_consent_timeouts`

```rust
async fn sweep_consent_timeouts(&self, now: DateTime<Utc>) -> RepositoryResult<Vec<ConsentId>>;
```

Two-phase: (1) probe eligible ids with `state = 'requested' AND deadline_at <= now`, capped at 256 per call to bound the audit-chain blast radius; (2) for each eligible id, call `mark_consent_timed_out` (its own compound tx). Returns the list of flipped ids. Rows that lose the race (already flipped concurrently) are silently skipped — the pure-fn returns `IllegalTransition` and the sweeper continues.

The 256-row cap prevents a misconfigured fixture (e.g. 100k Requested consents all past deadline) from emitting 100k audit events in one tick. Subsequent ticks pick up the remainder.

### D47.6 — New `deadline_at: Option<DateTime<Utc>>` field on `Consent`

Migration `0012_consent_deadline.surql` adds the column via `DEFINE FIELD OVERWRITE deadline_at ON consent TYPE option<string>`. The Rust struct gains the field with `#[serde(default)]` — legacy rows decode as `None`. The sweeper only flips rows where `state = Requested AND deadline_at <= now`. Population of the field at consent creation is policy-aware logic owned by CH-11.

### D47.7 — Sweeper task at `server::state::spawn_consent_sweeper`

```rust
pub fn spawn_consent_sweeper(repo: Arc<dyn Repository>, interval: Duration) -> JoinHandle<()>;
```

Wraps a tokio task that loops `interval.tick().await` then calls `repo.sweep_consent_timeouts(Utc::now())`. Configurable via `[consent] sweeper_interval_secs = 60` in `config/default.toml`. Default 60s; setting `0` disables the task (acceptance tests use this knob to drive the sweep manually).

The task is spawned at server startup ([`server/src/main.rs`](../../../../../../modules/crates/server/src/main.rs)) and the `JoinHandle` is currently dropped — the task runs until process exit. M7b will wire it into the graceful-shutdown drain alongside the rest of the K8s-readiness work.

**Single-pod-only at v0.** Multi-pod deployments would have every pod running its own sweeper; the storage UPDATE is idempotent (SurrealDB's `WHERE state = 'requested'` guards), but the audit-event creation is not. Each pod observing the same eligible row would emit a duplicate `consent.timed_out` audit. This breaks the "one audit per state transition" invariant. Multi-pod leader-election is deferred to M7b per [`CHK8S-D-09`](../../m7b/architecture/deferred-from-ch-k8s-prep.md#chk8s-d-09--consent-sweeper-leader-election-for-multi-pod-deployments).

### D47.8 — Forward-only revocation enforced at the legal-transition table

`legal_transition(Revoked, _)` returns `false` for every input. The state-machine layer enforces this so every code path that builds on `legal_transition` inherits the invariant — including the per-transition pure-fns, the per-transition Repository methods, and the sweeper. The `terminal_states_have_zero_outbound_arrows` test enumerates all 24 transitions out of the 4 terminal states and asserts every one is illegal.

### D47.9 — Default response on timeout (deny / allow) is **NOT** computed by CH-10

CH-10 stops at "the row reached `TimedOut` state on schedule". The Permission Check engine's gating semantic that maps `TimedOut → deny` (default) or `TimedOut → allow` (per the org's `approval_timeout_default_response: allow` config) is owned by CH-11. The sweeper's job is to flip the row; the read path's job is to interpret the state.

### D47.10 — Per-policy minter logic remains out of scope

Implicit-policy auto-`Acknowledged` (CH-09 default) stays the only auto-mint. OneTime / PerSession policy minters that create `Requested` consents at the right trigger points (first template fire / new session) land at CH-11. CH-10 ships the transition engine; CH-11 drives the engine.

---

## Consequences

**Positive:**
- Drift D-new-05 closed; concept-doc fidelity restored on the lifecycle front.
- CH-11 (Per-Session consent gating) unblocked.
- The `domain::consents` module establishes a clean home for future consent logic (per-policy minters, channel-notification glue, etc.).
- The 36-cell exhaustive matrix test pins the legal-transition table against future concept-doc drift.
- `#[serde(default)]` on `deadline_at` shields against future field additions following the same pattern.

**Negative:**
- One new K8s blocker (`CHK8S-D-09` — single-pod sweeper). Recorded as deferred work; M7b plan-author has the context.
- Adding `deadline_at` to `Consent` requires touching three test fixtures that construct `Consent` literally (the `#[serde(default)]` shield only covers serde callers, not struct-literal constructors). Acceptable — three single-line additions of `deadline_at: None`.

**Neutral:**
- Migration 0012 is forward-only (per ADR-0012); no down script.
- The sweeper's per-tick blast radius (256 rows) is a configurable cap that operators can revisit if real-world Consent volume exceeds the assumption.

---

## Cross-references

- Concept doc: [`permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Consent Lifecycle" lines 369–414.
- Drift closed: [`D-new-05`](../../m5_1/drifts/D-new-05.md).
- Drift dependency: D-new-17 (Per-Session consent blocks reads) — CH-11 owns; CH-10 unblocks.
- ADR-0028 — event-bus fail-safe; durable write before reactive emit.
- ADR-0042 §D42.3 #5 — migration runner conforming criteria; migration 0012 satisfies.
- ADR-0045 — CH-09 precedent for the Consent struct shape + the `ConsentState` enum.
- ADR-0046 — CH-23 precedent for compound-tx + audit-event-builder pattern.
- AuthRequest precedent: [`auth_requests/state.rs`](../../../../../../modules/crates/domain/src/auth_requests/state.rs) + [`transitions.rs`](../../../../../../modules/crates/domain/src/auth_requests/transitions.rs).
- K8s deferral: [`CHK8S-D-09`](../../m7b/architecture/deferred-from-ch-k8s-prep.md#chk8s-d-09--consent-sweeper-leader-election-for-multi-pod-deployments).

---

## Verification

- Workspace tests: `cargo test --workspace -- --test-threads=1` green at **~1261** (1223 baseline + 38 new tests).
- Clippy under `RUSTFLAGS="-Dwarnings"`: clean.
- 4 CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
- Positive greps:
  - `domain::consents::{mod.rs, state.rs, transitions.rs}` exist.
  - `pub enum ConsentTransitionError` (1) at `transitions.rs`.
  - 5 pure-fn transitions exist.
  - 6 new `Repository` methods (5 transitions + 1 sweeper) on the trait + both backends.
  - 5 audit-event builders at `audit/events/m5/consents.rs`.
  - Migration 0012 file exists; registered at version 12 / slug `consent_deadline`.
  - `spawn_consent_sweeper` exists at `server::state`.
  - `[consent]` block in `config/default.toml` with `sweeper_interval_secs = 60`.
- Carry-forward green: CH-09 (Consent shape), CH-23 (Template C/D end-to-end), CH-21 (memory extraction), CH-22 (agent catalog), CH-04 (action matrix).
