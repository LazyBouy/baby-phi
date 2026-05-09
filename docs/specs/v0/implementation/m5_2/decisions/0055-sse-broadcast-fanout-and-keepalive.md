<!-- Last verified: 2026-05-09 by Claude Code (CH-17 P-seal chunk-seal — Status flipped Proposed → Accepted; sub-decisions D55.1–D55.9 pinned by P1–P3 deliverables: trait `SessionLiveStreamRegistry` + `InProcessSessionLiveStreamRegistry` at `server/src/state.rs:134–208`; broadcast tap on `BabyPhiSessionRecorder.broadcast_tx: Option<broadcast::Sender<AgentEvent>>` at `domain/src/session_recorder.rs:148`; SSE handler at `server/src/handlers/sessions.rs::events` (215–266); platform body `open_live_stream` at `server/src/platform/sessions/events.rs:109`; sibling builder `build_session_observe_manifest` at `domain/src/permissions/builders/session_observe.rs:70`; new audit-event builder `session_live_stream_denied` at `domain/src/audit/events/m5_2/session_live_stream.rs:47`; migration `0016_template_a_session_object_grant_add_observe.surql`; D7.1 transitioned to remediated; concept-audit-matrix rows under `permissions/05` extended; CHK8S-D-10 ledger entry filed.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-17 P0 — ADR drafted as Proposed) -->

# ADR-0055 — SSE broadcast fan-out, keep-alive, per-session live-stream registry, and `Action::Observe` gate

**Status: Accepted**

**Date:** 2026-05-09
**Chunk:** CH-17
**Cycle hex:** `40c4d759`
**Closes:**
- [`D7.1`](../../m5_1/drifts/D7.1.md) (HIGH, A) — Live SSE tail deferred to M7 (ADR-0031 D4 path-(a) not taken at M5). Concept doc 05 §"session lifecycle — live events" + concept doc 03 §"Action × Fundamental Applicability Matrix" line 44 (Observability universal across fundamentals) are honored: a real `GET /api/v0/sessions/:id/events` endpoint streams `phi_core::AgentEvent`s; the CLI's deferred-tail print is retired; the live tail surfaces via `Action::Observe`-gated SSE.

---

## Forks

`Forks (F1.A / F2 (buffer=64) / F3.B / F4 (30s) / F5.B (Action::Observe) / F6.B (AgentEvent re-export) / F5.B.subfork.a (migration 0016) — all 6 forks + 1 sub-fork user-locked at plan approval; F5 lock diverges from planner iter-1 recommendation F5.A but corrected fact-base in iter-2 confirmed F5.B preserves the closed 34-verb invariant.)`

- **F1 → F1.A** (trait-shaped per-session `SessionLiveStreamRegistry` mirroring `SessionRegistry`; `Arc<dyn SessionLiveStreamRegistry>` dispatch through `AppState`) — user-locked at iter-1 plan approval 2026-05-09.
- **F2 → F2 buffer=64** (`tokio::sync::broadcast::channel::<AgentEvent>(64)` per session) — user-locked at iter-1 plan approval 2026-05-09.
- **F3 → F3.B** (lagging-receiver policy: emit a typed SSE error event with kind `lagged` then close; `BroadcastStream` returns `None` after the lag report) — user-locked at iter-1 plan approval 2026-05-09.
- **F4 → F4 keep-alive=30s** (`Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)).text(": keep-alive"))`) — user-locked at iter-1 plan approval 2026-05-09.
- **F5 → F5.B** (use `Action::Observe` on `session_object` for the SSE gate — sibling builder `build_session_observe_manifest(project_id) -> Manifest` returns `[Observe]` on `session_object`) — user-locked at iter-2 plan approval 2026-05-09 (iter-1 recommended F5.A `[Read, Inspect, List]` but the iter-1 §3.D claim that F5.B would extend the closed-set was retracted in iter-2 with the corrected fact-base: `Action::Observe` IS already in `Action::CANONICAL`).
- **F6 → F6.B** (re-export `phi_core::types::event::AgentEvent` directly as the SSE wire format; no new domain type wrapping it) — user-locked at iter-1 plan approval 2026-05-09.
- **F5.B.subfork → F5.B.subfork.a** (NEW migration `0016_template_a_session_object_grant_add_observe.surql` parallel to CH-15's 0015 — appends `"observe"` to the action array of every legacy Template A grant; production minter at `domain/src/templates/a.rs::fire_grant_on_lead_assignment` extends both grants from `[Read, Inspect, List]` to `[Read, Inspect, List, Observe]`) — user-locked at iter-2 plan approval 2026-05-09.

---

## Cross-references

**(a) Originating concept docs**

- [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Standard Actions Applied to Sessions" line 242 (canonical `read = "Retrieve a Session and its contents"` — out-of-scope for SSE since SSE uses `observe`); §"Default Grants Issued to Every Agent" lines 253–290 (Default Grants 1+2 — out-of-scope-in-v0; future M6+ chunk SHOULD include `observe` in the action set per D55.5); §"Authority Templates" lines 294–433 (A/B/C/D/E grant shape — Template A extended to mint `[Read, Inspect, List, Observe]` in this chunk per D55.9).
- [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"Standard Action Vocabulary" line 22 (Observability row — `observe, log, attest`); §"Action × Fundamental Applicability Matrix" line 44 (Observability universal across fundamentals — the cited rationale for F5.B). **Source-citations for `Action::Observe`'s pre-existing canonical status (per D55.5 fact-base correction): `domain/src/permissions/action.rs:73` (variant declaration), `:282` (CANONICAL array entry), `:322` (`as_str() => "observe"` mapping).**
- [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"All steps hard-gate" — invariant flipped to `honored` at CH-15 / ADR-0054 §D54.6, reused verbatim at SSE: every `Decision::Denied` from `check()` → 403.
- [`concepts/permissions/README.md`](../../../concepts/permissions/README.md) — entry invariants (closed action vocabulary + closed fundamental kinds + closed selector grammar).
- [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §"Sessions / events" row — `phi_core::types::event::AgentEvent` is the canonical agent-loop event type; baby-phi consumes via the recorder.

**(b) Closed drifts by ID**

- `D7.1` (HIGH, A) — Live SSE tail deferred to M7 (ADR-0031 D4 path-(a) not taken at M5). Closed by this chunk via the SSE handler + CLI default-tail flip + recorder broadcast tap.

**(c) Prior ADRs cited as precedent (milestone-prefixed paths per CH-08 retro Row 1)**

- [`m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md`](./0054-session-launch-manifest-and-hard-deny-flip.md) — CH-15 manifest-builder + hard-deny precedent. F5.B introduces a SIBLING manifest-builder `build_session_observe_manifest` for SSE; preview-launch parity preserved (preview/launch keep `[Read, Inspect, List]`; SSE alone uses `[Observe]`).
- [`m5_2/decisions/0048-per-session-consent-gating.md`](./0048-per-session-consent-gating.md) — CH-11 per-session-consent launch flow. SSE stays consent-free per D55.5 — consent is launch-time, not stream-time.
- [`m5_2/decisions/0033-k8s-prep-refactors.md`](./0033-k8s-prep-refactors.md) §D33.1 (`SessionRegistry` trait-shape precedent for `SessionLiveStreamRegistry`); §D33.2 (migration-runner ledger pattern — basis for migration 0016 idempotency).
- [`m4/decisions/0028-domain-event-bus.md`](../../m4/decisions/0028-domain-event-bus.md) — CH-K8S-PREP / CH-04 governance event-bus design (clarifies CH-17's broadcast is orthogonal to `EventBus`; the broadcast registry is per-session telemetry, not cross-cutting governance).
- [`m5/decisions/0029-session-persistence-and-recorder-wrap.md`](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) — recorder-wrap design where the broadcast tap attaches.
- [`m5/decisions/0031-session-cancellation-and-concurrency.md`](../../m5/decisions/0031-session-cancellation-and-concurrency.md) — `SessionRegistry` precedent + 503 saturation gate (`SessionLiveStreamRegistry` mirrors this shape).
- [`m5_2/decisions/0032-mock-provider-at-m5.md`](./0032-mock-provider-at-m5.md) — CH-02 the real `agent_loop` runtime + MockProvider that produces the events being streamed.

**(d) Forward-scope row** (mandatory per CH-13 retro Row 1): [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §"Live SSE", lines 161–167.

---

## Context

Concept doc 05 §"Standard Actions Applied to Sessions" line 242 names `read = "Retrieve a Session and its contents (Loops, Turns, Messages)"`. Concept doc 03 §"Action × Fundamental Applicability Matrix" lines 28–38 lists `Observability` as universal across all 9 fundamentals (line 44: *"Discovery, Authority, and Observability apply universally (every fundamental has list/inspect, delegate/allocate/transfer, and observe/log/attest)"*). Drift D7.1 (`m5_1/drifts/D7.1.md` lines 17–22) records that no `/events` SSE endpoint exists; the CLI prints "(live tail deferred to M7)" at `cli/src/commands/session.rs:228`. Operators have no live-transcript surface.

With CH-02 (real `phi_core::agent_loop()` runtime per ADR-0032) and CH-15 (real permission gate at session launch per ADR-0054) shipped, the prerequisite for a real SSE stream is in place: `BabyPhiSessionRecorder::on_phi_core_event` (`domain/src/session_recorder.rs:126`) is the single funnel through which every `AgentEvent` flows. CH-17 attaches a `tokio::broadcast::Sender<AgentEvent>` at that funnel, exposes a hard-deny-gated SSE handler at `GET /api/v0/sessions/:id/events` using `Action::Observe` on `session_object` (semantically precise observability gate per F5.B user-lock), and flips the CLI's deferred-tail print into a real stream consumer.

---

## Decision

### D55.1 — Trait-shaped per-session `SessionLiveStreamRegistry` (F1.A)

A new trait `SessionLiveStreamRegistry: Send + Sync` ships at `server/src/state.rs:134` mirroring the `SessionRegistry` shape from CH-K8S-PREP / ADR-0033 §D33.1:

```rust
pub trait SessionLiveStreamRegistry: Send + Sync {
    fn insert(&self, session_id: SessionId, tx: broadcast::Sender<AgentEvent>);
    fn get(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;
    fn remove(&self, session_id: &SessionId) -> Option<broadcast::Sender<AgentEvent>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}
```

The default impl `InProcessSessionLiveStreamRegistry` wraps a `DashMap<SessionId, broadcast::Sender<AgentEvent>>` for lock-free per-key access. `AppState.session_live_stream_registry: Arc<dyn SessionLiveStreamRegistry>` is the injection point. Trait-shape ensures the M7+ Redis-pub/sub-backed swap (CHK8S-D-10) is a new impl, not a refactor.

**Rejected alternatives:**
- F1.B (recorder-side-table) — `BabyPhiSessionRecorder` already has multiple responsibilities (persisting + auditing + emitting); a side-table would couple lifecycle. Trait-shape on `AppState` is symmetric with the existing `SessionRegistry`.
- F1.C (global `OnceCell`) — pod-singleton state outside `AppState` violates the existing dependency-injection discipline. Rejected.

### D55.2 — Channel buffer = 64 (F2)

`tokio::sync::broadcast::channel::<AgentEvent>(64)` is created at session-launch time and registered before the agent task spawns. Buffer 64 is large enough to absorb burst-emissions (typical `agent_loop` produces ≤ 10 events per turn) yet small enough that a slow consumer is detected within ~6 turns. Configurable via the `[session_live_stream] buffer = 64` block in `config/default.toml`; the default 64 is the v0 production setting.

**Rejected alternatives:**
- 16 — too tight; routine bursts (multi-tool turns) would trip Lagged.
- 256 — masks slow consumers; lag detection would be deferred until ~25 turns of accumulation.
- unbounded — memory unbounded; defeats the purpose of bounded broadcast.

### D55.3 — Lagging-receiver policy: typed SSE error event then close (F3.B)

When `BroadcastStream::poll_next()` returns `Err(BroadcastStreamRecvError::Lagged(missed))`, the handler emits a typed SSE event with `event: lagged` and JSON body `{"missed": <count>}`. After the lag report, `BroadcastStream` returns `None` on the next poll, closing the SSE stream. Operators reconnect via the documented reconnect playbook (m5_2 ops doc).

**Rejected alternatives:**
- F3.A (swallow + warn) — silent data loss; observers cannot distinguish missing events from quiet sessions.
- F3.C (reconnect-signal) — bespoke; adds protocol surface area. The lagged-then-close pattern is idiomatic for `BroadcastStream`.

### D55.4 — Keep-alive interval = 30 s (F4)

`Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)).text(": keep-alive"))`. 30 s sits comfortably below typical load-balancer idle timeouts (60 s default for kube-proxy / GCP / AWS ALB) while keeping ping volume modest. Configurable via `[session_live_stream] keep_alive_secs = 30`; default 30 s is v0 production.

**Rejected alternatives:**
- 15 s — chattier than necessary; doubles ping bandwidth on idle connections.
- 60 s — risks LB-mediated connection close before the next ping; some LBs use 60 s exactly.

### D55.5 — `Action::Observe` on `session_object` for the SSE gate (F5.B, iter-2 user-lock)

The SSE handler builds a sibling manifest via `domain::permissions::builders::session_observe::build_session_observe_manifest(project_id) -> Manifest`. The manifest body:

```rust
Manifest {
    actions: vec![Action::Observe],
    resource: vec!["session_object".to_string()],
    transitive: vec![],
    constraints: vec![],
    constraint_requirements: HashMap::new(),
    kinds: vec![],
}
```

**Pre-existing canonical status of `Action::Observe` (fact-base correction over iter-1):**

- `domain/src/permissions/action.rs:73` — variant declaration `Observe,` (canonical, pre-CH-17 introduced at CH-04 / ADR-0043 when the 34-verb enum landed).
- `domain/src/permissions/action.rs:282` — `Action::CANONICAL` array contains `Action::Observe`.
- `domain/src/permissions/action.rs:322` — `as_str()` mapping returns `"observe"`.
- `concepts/permissions/03-action-vocabulary.md` line 22 — Observability row: `Observability | observe, log, attest`.
- `concepts/permissions/03-action-vocabulary.md` line 44 — *"Discovery, Authority, and Observability apply universally (every fundamental has list/inspect, delegate/allocate/transfer, and observe/log/attest)"*.

`Action::CANONICAL.len() == 34` invariant unbroken — F5.B exercises an already-canonical verb. **F5.B does NOT break the closed-set invariant.** Iter-1's claim to the contrary is retracted in this ADR. The SSE gate using `Action::Observe` is concept-aligned, not concept-divergent, and is semantically more precise than `[Read, Inspect, List]` because live-event tailing IS an observability operation per concept-doc nomenclature (line 22: `observe = "Subscribe to a stream of state-change events on a fundamental"`).

**Forward-defensive note for Default Grant 1 (concept doc 05 lines 253–268).** Default Grant 1 is currently NOT-IMPLEMENTED-IN-V0 (every `create_agent` callsite passes `default_grants: vec![]`); CH-17 does not change that. The future M6+ chunk that ships Default Grant 1 issuance at agent creation SHOULD include `Action::Observe` in the action set so every agent has an observe-grant on its self-tag without needing a Template A fire. This ADR captures the forward-defensive reservation; no v0 code change.

**SSE stays consent-free.** Per concept doc 06 §"Per-Session consent" + ADR-0048 §D48.5, consent is a launch-time gate (Step 6 of the engine's 7-step pipeline). The SSE handler runs Steps 0–5 of the engine but does NOT run Step 6 — observability of an already-launched session does not require a fresh consent ack. This preserves the consent-launch-only invariant.

**Rejected alternatives:**
- F5.A (reuse `[Read, Inspect, List]` — CH-15 launch parity) — semantically conflates `read` (full content fetch) with `observe` (event subscription); concept-doc nomenclature distinguishes them.

### D55.6 — Re-export `phi_core::types::event::AgentEvent` as the wire format (F6.B)

The SSE handler serialises `phi_core::types::event::AgentEvent` JSON directly onto the wire. No new domain type wrapping it. The forward-scope row's literal text *"streams `RecorderEvent`"* is a scoping-gloss naming "the events the recorder funnels" — `RecorderEvent` does not exist anywhere in the codebase or concept docs. The CH-17 implementation surface is `phi_core::types::event::AgentEvent` per phi-core leverage Rules 1 + 4 (direct-reuse).

**Forbidden-duplication greps (all return 0 hits at chunk seal):**
- `^pub (struct|enum|type) RecorderEvent` in `modules/crates/`.
- `^pub enum SsEvent\b|^pub enum SseEvent\b|^pub struct SseRecorderEvent\b` in `modules/crates/`.

**Rejected alternatives:**
- F6.A (new domain type wrapping `phi_core::AgentEvent`) — two-source-of-truth violation (per `baby-phi/CLAUDE.md` §"phi-core Leverage" Rule 1); compounds at every event-shape change in phi-core.

### D55.7 — Pre-existing-behaviour preservation note

Per CH-14 retro Row 10 — pre-CH-17 absent surface and changed surfaces are documented explicitly:

> *"Pre-CH-17 behaviour preserved: the CLI's `phi session launch` print at `cli/src/commands/session.rs:228` ('(live tail deferred to M7 — `phi session show --id <id>` inspects terminal state)') is retired. The `--detach` flag's wire shape is preserved verbatim — `--detach` continues to skip the live tail and return the JSON receipt only. Pre-CH-17, every launch was effectively `--detach`. CH-17 makes `--detach` semantic for the first time. The `permission_check.decision` field on the launch receipt is preserved; CH-17 adds zero fields to the receipt schema. Pre-CH-17 absent surface: `GET /api/v0/sessions/:id/events` returned 404 ROUTE_NOT_FOUND. CH-17 is the first writer.*
>
> *Iter-2-specific preservation note: `Action::Observe` was already canonical at `domain/src/permissions/action.rs:73,282,322` pre-CH-17 (introduced at CH-04 / ADR-0043 when the 34-verb enum landed). CH-17 does NOT introduce the `Observe` variant; it merely exercises the pre-existing canonical verb on `session_object` for the first time. Migration 0016 walks legacy Template A grants and appends `\"observe\"` to their action arrays — preserving every other field (holder, descends_from, delegable, issued_at, approval_mode, audit_class) verbatim. The pre-CH-15 single-grant-per-fire shape is unchanged; the pre-CH-15 paired-grant migration 0015 is unchanged; 0016 is purely additive."*

### D55.8 — K8s readiness: single-pod fan-out today; CHK8S-D-10 ledgers the multi-pod swap

The `tokio::sync::broadcast::channel<AgentEvent>` is pod-local by definition. SSE clients connecting to pod A see events from sessions hosted on pod A. Pod-B session events do NOT reach pod-A SSE clients without Redis pub/sub fan-out.

**K8s axes lit by CH-17 (per chunk plan §3.B):**

- **A1** (new in-process state): YES — `InProcessSessionLiveStreamRegistry { inner: DashMap<SessionId, broadcast::Sender<AgentEvent>> }`.
- **A2** (new IPC channel): YES — `tokio::sync::broadcast::channel(64)` per session.
- **A4** (migration runner / first-apply race): YES — migration 0016 follows ADR-0033 §D33.2 ledger pattern + CH-15's 0015 precedent (no NEW blocker class; A4 is a pre-existing axis already mitigated by the migration-runner pattern).
- **A5** (trait-shape requirement): YES — shipped as a trait so the M7b Redis-pub/sub impl swaps in.
- **A6** (cross-pod state sharing): YES — single-pod-only at v0; cross-pod fan-out deferred via CHK8S-D-10.

**CHK8S-D-10 ledger entry filed at `m7b/architecture/deferred-from-ch-k8s-prep.md`:**

- ID: `CHK8S-D-10`
- Title: Cross-pod live-event fan-out via Redis pub/sub-backed `SessionLiveStreamRegistry`
- Severity: HIGH
- Origin: CH-17 P-1 (trait-shape `SessionLiveStreamRegistry`)
- Successor target: M7b "externalize SessionLiveStreamRegistry"
- M7b deliverable: NEW `RedisSessionLiveStreamRegistry` impl behind the same trait; swap-only refactor (no callsite changes).

**Audit-event A7 axis preserved.** SSE handler writes ZERO audit events on the success path (read-only stream). On 403 it emits a NEW `platform.session.live_stream_denied` audit event (Alerted-class, parallel to CH-15's `platform.session.launch_denied`) — single-writer guarantee preserved per ADR-0054 §D54.5 precedent.

### D55.9 — Migration 0016 + extended Template A mint (F5.B.subfork.a)

**Migration `0016_template_a_session_object_grant_add_observe.surql`** (forward-only, idempotent) walks every legacy Template A grant whose `action` array does not yet contain `"observe"` and appends it. Provenance filter: `descends_from(AR with kinds CONTAINS '#template:a')`; live filter: `revoked_at = NONE`. Mirrors the 0015 ledger-aware shape (uses the `_migrations` table per ADR-0033 §D33.2 + a body-level `array::find_index(action, "observe") = NONE` belt-and-braces guard against ledger drift).

**Production minter extension.** `domain/src/templates/a.rs::fire_grant_on_lead_assignment` mints both grants (`project_grant` + `session_grant`) with the action set `[Read, Inspect, List, Observe]` going forward (extended from `[Read, Inspect, List]`). The function signature is unchanged (`-> Vec<Grant>` per ADR-0054 §D54.3); only the action vec body changes.

**Cascade discipline (per chunk plan §3 Artifact C):** the 24 callsites of `fire_grant_on_lead_assignment` are unchanged; only the 6 assertion sites in `templates::a::tests` + `template_a_firing_props.rs` that verify the action set verbatim are updated (the rest assert on length / holder / resource / descends_from / delegable shape, which is unchanged).

**Authority Chain preservation (ADR-0053 / CH-14).** Migration 0016 leaves `descends_from` untouched; provenance walks (`walk_provenance_chain`) traverse the modified action array transparently. Test `acceptance_authority_chain` remains green.

**Rejected alternatives:**
- F5.B.subfork.b (manifest-shape change at the SSE handler — build a 4-action manifest covering legacy 3-action grants by relaxing Step 3) — requires engine semantic change at Step 3 + breaks the explicit-action covering-grant model. REJECTED.
- F5.B.subfork.c (hard-deny on legacy; require operator to re-issue grants) — operationally cheap today (zero prod data) but fragile: every persisted-fixture acceptance test breaks unless test fixtures are updated; no migration discipline. REJECTED.

---

## Consequences

**Positive:**
- Operator UX delivered: `phi session launch` (live tail by default) + `curl /events` direct-tail.
- Concept doc 05 §"session lifecycle — live events" + concept doc 03 §"Observability universal" honored at the runtime gate for the first time.
- Drift D7.1 closed (HIGH severity).
- Single phi-core import added (`use phi_core::types::event::AgentEvent;` in `server/src/platform/sessions/events.rs`); leverage Rule 1 honored.
- Trait-shape on `SessionLiveStreamRegistry` keeps M7+ Redis-pubsub swap as a new-impl path, not a refactor.

**Negative / known limitations:**
- Single-pod-only at v0 (CHK8S-D-10 deferred).
- Migration 0016 must run before any post-CH-17 SSE call against a CH-15-era DB (the migration runner's idempotent ledger guards this; `cargo test -p store --test migration_0016_test` is the deterministic-seed proof).

**Neutral:**
- `--detach` flag wire shape preserved verbatim (continues to mean "no live tail").

---

## Audit-event impact

CH-17 introduces one new audit event:
- `platform.session.live_stream_denied` (Alerted class) — emitted by `open_live_stream` whenever the engine returns `Decision::Denied`; mirrors `platform.session.launch_denied` from CH-15 / ADR-0054 §D54.5.

Builder at `domain/src/audit/events/m5_2/session_live_stream.rs::session_live_stream_denied`. Fields: `actor`, `session_id`, `agent_id`, `project_id`, `org_id`, `step`, `reason_kind`, `error_summary`, `now`. Emitted BEFORE returning Err so the audit-write never lags the wire response (ADR-0054 §D54.5 precedent).

---

## Verification at chunk seal

- `cargo test -p domain permissions::action::tests::canonical_contains_thirty_four_variants` → 1 PASS (closed 34-verb invariant preserved).
- `cargo test -p domain permissions::builders::session_observe::tests` → 4 PASS.
- `cargo test -p store --test migration_0016_test` → 2 PASS.
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` → 50 (was 49; +1 for `events.rs` AgentEvent import; observed 51 with one transitive AgentEvent re-export per gate-2 import-count delta).
- `git -C /root/projects/phi/baby-phi grep -nE '^pub (struct|enum|type) RecorderEvent' modules/crates/` → 0 hits.
- `grep -nE '^\s*Observe,$' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs` → 1 hit at line 73.
- `grep -nE 'Action::Read.*Action::Inspect.*Action::List.*Action::Observe' /root/projects/phi/baby-phi/modules/crates/domain/src/templates/a.rs | wc -l` → ≥ 2 (project_grant + session_grant).
- `bash scripts/check-doc-links.sh` + `check-ops-doc-headers.sh` + `check-phi-core-reuse.sh` + `check-spec-drift.sh` → all GREEN.
