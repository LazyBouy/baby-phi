<!-- Last verified: 2026-05-02 by Claude Code -->

# CH-10 — Consent lifecycle state machine + per-transition repo surface + auto-timeout sweeper

**Plan file token:** `fda01605` (generated 2026-05-02 at chunk-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/ch-10-consent-state-machine-and-sweeper-fda01605.md` (slug-first naming convention).
**Chunk ID:** CH-10.
**Severity:** ⚠ HIGH (closes D-new-05; unblocks D-new-17 / Per-Session consent gating at CH-11).
**Expected effort:** ~2 engineer-days. The forward-scope row estimated 1d for the state-machine alone; per locked Q2 (2026-04-30) the user has folded the time-driven sweeper into CH-10, growing scope by ~0.5d.
**Hard prerequisites:** CH-09 (sealed — `ConsentState` enum + `state` field on `Consent` shipped). No other blockers.
**Chunks unblocked at close:** CH-11 (Per-Session consent gating — needs the state machine + read-side projection refresh).

---

## Context

### The simple version

CH-09 shipped the `ConsentState` enum and the `state: ConsentState` field on `Consent`, but no transition logic — every Consent record sits at `Acknowledged` (the back-compat default) until something writes a new state. There is no validated way to move a Consent through `Requested → Acknowledged / Declined / TimedOut → Revoked / Expired`, no enforcement of the forward-only revocation invariant from concept doc 06 §"Consent Lifecycle", and no auto-timeout sweep that turns dangling `Requested` consents into `TimedOut` after their deadline elapses.

CH-10 closes drift D-new-05 by shipping (1) the pure-fn transition module at `domain::consents::transitions`, (2) five per-transition Repository methods + audit-event builders + impls on both backends, and (3) a background sweeper task that periodically scans for past-deadline `Requested` rows and flips them to `TimedOut`.

### What this chunk does NOT do

- Does NOT change the Permission Check engine's `ConsentIndex` projection. The index still consumes `(AgentId, OrgId)` pairs at this chunk; CH-11 evolves it once the per-session gating logic lands.
- Does NOT add HTTP / CLI surface for subordinate acknowledge / decline. The repo methods are callable by handlers but those endpoints land alongside CH-11's per-session gating + the subordinate-facing inbox UI (M6+).
- Does NOT integrate with subordinate Channel notification (Slack / email / web UI delivery of consent requests). Concept doc 06 line 414 ("Request/response channel travels through Channel") is a separate concern owned by the Channel surface (M6+).
- Does NOT enforce the Consent state at any read path. Reads still pass through whatever the existing `ConsentIndex::is_acknowledged` returns; CH-11 wires the state-aware gating.
- Does NOT close drift D-new-17 (Per-Session consent blocks reads until Acknowledged) — CH-11 owns that.

### User-decided forks (locked at plan-review, 2026-04-30)

1. **Module layout (Q1)** — Dedicated module `domain::consents::{mod.rs, state.rs, transitions.rs}` mirroring the existing `auth_requests/{state.rs, transitions.rs}` precedent. Pure-fn transitions returning `Result<Consent, ConsentTransitionError>`.
2. **Time-driven sweeper (Q2)** — Ship in CH-10. Adds ~0.5d to the original 1d estimate but keeps the consent system internally consistent at chunk-close. The sweeper is exposed as a `Repository::sweep_consent_timeouts(now) -> Vec<ConsentId>` method PLUS a tokio task at server startup that calls it on a configurable interval.
3. **Repository surface (Q3)** — Per-transition methods: `acknowledge_consent`, `decline_consent`, `revoke_consent`, `mark_consent_timed_out`, `mark_consent_expired`. Each carries its own audit-event helper. Five methods total. Finer permission-gating in later chunks if needed.

### Forward-scope reference

[`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §1 lines 107–112.

---

## §1 — Why this chunk (one paragraph)

CH-09 lifted Consent to its full 11-field shape, but the `state` field is dead data without transition logic. Drift D-new-05 (HIGH) has tracked this gap since 2026-04-24; CH-11 (Per-Session consent gating) is hard-blocked on the state machine landing. CH-10 closes the gap by shipping the pure-fn transition module at `domain::consents::transitions` (mirroring the `auth_requests` precedent), five per-transition Repository methods with their audit-event builders + atomic compound-tx impls on both backends, and an auto-timeout sweeper that turns abandoned `Requested` consents into `TimedOut` per the deadline derived from the org's `approval_timeout` config (defaults: project-duration for Shape A/B/C, org-level Duration for Shape D, fallback `deny`). The chunk ships the transitions + sweep mechanic only — wiring into HTTP, CLI, Channel notification, and Permission Check gating remains downstream concerns.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Status at chunk-close |
|---|---|---|---|---|
| [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | §"Consent Lifecycle" lines 369–414 | 6-state machine: Requested → {Acknowledged, Declined, TimedOut}; Acknowledged → Revoked; any → Expired. Forward-only revocation. | partially-honored (type ships; transitions don't) | honored — `domain::consents::transitions::{acknowledge, decline, revoke, mark_timed_out, mark_expired}` enforces every legal transition + rejects illegal ones with typed `ConsentTransitionError` |
| `permissions/06-multi-scope-consent.md` | §"Default response on timeout" line 347 | Default `deny` on timeout; org-configurable `approval_timeout_default_response: allow` | silent-in-code | partially-honored — sweeper sets state to `TimedOut`; the *response* (deny vs allow) is computed by the Permission Check engine at read time (CH-11 wires it). CH-10 stops at "consent reaches TimedOut state on schedule". |
| `permissions/06-multi-scope-consent.md` | §"Forward-only revocation" lines 416+ | Acknowledged → Revoked is one-way; cannot revert | silent-in-code | honored — transition rejects `Revoked → Acknowledged` with `ConsentTransitionError::IllegalTransition` |
| `permissions/06-multi-scope-consent.md` | §"Per-policy mapping" lines 407–414 | implicit / one_time / per_session policies select when `Requested` is created | partially-honored (consent creation is policy-aware via existing minters; state machine is policy-agnostic) | partially-honored (unchanged) — the state machine doesn't *consult* policy at transition time; policy decides *whether* to mint Requested. CH-11 owns the policy-aware minter rewrite. |

---

## §3 — phi-core leverage map

| phi-core type | Action in this chunk |
|---|---|
| (none) | — |

CH-10 is baby-phi-native. phi-core has no governance-consent concept. Zero `use phi_core::` imports added or removed.

**Positive close-audit greps**:
```bash
ls modules/crates/domain/src/consents/{mod.rs,state.rs,transitions.rs}                        # all 3 exist
grep -n "pub enum ConsentTransitionError" modules/crates/domain/src/consents/transitions.rs   # 1
grep -n "pub fn acknowledge\b\|pub fn decline\b\|pub fn revoke\b\|pub fn mark_timed_out\b\|pub fn mark_expired\b" modules/crates/domain/src/consents/transitions.rs   # ≥ 5
grep -n "fn acknowledge_consent\|fn decline_consent\|fn revoke_consent\|fn mark_consent_timed_out\|fn mark_consent_expired\|fn sweep_consent_timeouts" modules/crates/domain/src/repository.rs   # ≥ 6
grep -n "fn spawn_consent_sweeper" modules/crates/server/src/state.rs   # 1
grep -n "consent.acknowledged\|consent.declined\|consent.revoked\|consent.timed_out\|consent.expired" modules/crates/domain/src/audit/events/m5/   # 5 audit-event builders
```

**Forbidden-duplication / regression greps**:
```bash
grep -rn "use phi_core::" modules/crates/domain/src/consents/                                 # 0
```

---

## §3.B — K8s readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | The sweeper task runs in-process per pod. Multiple pods running the sweeper simultaneously would race on the same `Requested` rows; SurrealDB's UPDATE with `WHERE state = 'requested'` makes the race a no-op (idempotent), but every pod emits its own audit event for the same flip. **Single-pod-only at v0**; multi-pod leader-election deferred to M7b. | **Yes — entry CHK8S-D-XX in `m7b/architecture/deferred-from-ch-k8s-prep.md`** |
| **A2** IPC channels | None. | No |
| **A3** pod-local resources | None. | No |
| **A4** migration runner | No new migration. The existing `consent` table from migration 0010 carries all needed fields. | No |
| **A5** trait-shape requirement | 6 new `Repository` methods. Both backends implement them. | No |
| **A6** cross-pod state sharing | The state mutations live in SurrealDB; queries span pods identically. | No |
| **A7** audit hash-chain symmetry | 5 new audit-event variants (`consent.acknowledged` / `consent.declined` / `consent.revoked` / `consent.timed_out` / `consent.expired`) follow the existing canonical-bytes pattern (BLAKE3). | No |

**Conclusion:** One new K8s blocker — the sweeper task is single-pod-only at v0. Add an entry to `m7b/architecture/deferred-from-ch-k8s-prep.md`.

---

## §3.C — User-facing documentation impact

| Tier | File | Action |
|---|---|---|
| Concept | [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | Verified-header bump noting CH-10 lifts §"Consent Lifecycle" + §"Forward-only revocation" + §"Default response on timeout" into typed Rust at `domain::consents::transitions`. Doc body UNCHANGED. |
| Decision | `m5_2/decisions/0047-consent-state-machine-and-sweeper.md` (NEW) | Full ADR — see §5. |
| Architecture | (none) | The state machine lives in the domain layer; no architecture-doc bump needed. |
| Operations | (none) | The consent surface has no operator-facing endpoints at v0. |
| K8s readiness | [`m7b/architecture/deferred-from-ch-k8s-prep.md`](baby-phi/docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) | Add entry CHK8S-D-XX recording the single-pod sweeper deferral. |

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition |
|---|---|---|---|
| **D-new-05** | [`m5_1/drifts/D-new-05.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-05.md) | HIGH | `discovered → in-chunk-plan → remediated` |

**Index updates:**
- `drifts/README.md` — D-new-05 row Status flipped to `remediated`; "Closes at" → `CH-10 ✓`.
- `_concept-audit-matrix.md` — flip the Consent-lifecycle row from `partially-honored` to `honored`.

---

## §5 — ADR drafted

ADR numbering: highest issued = ADR-0046 (CH-23). Next-free = **ADR-0047**.

| ADR | Title | Decision summary |
|---|---|---|
| **ADR-0047** | Consent state machine + per-transition repo surface + auto-timeout sweeper | **D47.1** New module `domain::consents::{mod.rs, state.rs, transitions.rs}` mirrors the `auth_requests/{state.rs, transitions.rs}` precedent. `state.rs` carries pure predicates (`ConsentState::is_terminal`, `legal_transition(from, to) -> bool`, the legal-transition table). `transitions.rs` carries the five per-operation pure-fn helpers `acknowledge / decline / revoke / mark_timed_out / mark_expired`, each returning `Result<Consent, ConsentTransitionError>`. Per locked Q1. **D47.2** New error type `ConsentTransitionError` at `transitions.rs` with variants `IllegalTransition { from, to }`, `NotRevocable { consent_id }` (Acknowledged → Revoked rejected when `revocable == false`), `Terminal { state, attempted }` (any transition out of `Revoked` / `Declined` / `TimedOut` / `Expired`). Derives `thiserror::Error + Serialize + Deserialize`. **D47.3** Five per-transition `Repository` methods (per locked Q3): `acknowledge_consent(consent_id, at, actor)`, `decline_consent(consent_id, at, actor)`, `revoke_consent(consent_id, at, actor)`, `mark_consent_timed_out(consent_id, at, actor)`, `mark_consent_expired(consent_id, at, actor)`. Each method's compound tx (a) reads the row, (b) calls the matching pure-fn, (c) writes the new row + audit event atomically, (d) returns the `AuditEventId`. Failure inside the pure-fn surfaces as `RepositoryError::ConsentTransition { source: ConsentTransitionError }` (new variant). **D47.4** Five new audit-event builders at `domain::audit::events::m5::consents` (NEW file): `consent_acknowledged`, `consent_declined`, `consent_revoked`, `consent_timed_out`, `consent_expired`. All `AuditClass::Logged`. Diff carries `consent_id`, `from_state`, `to_state`, plus the timestamp field that was set by the transition (`responded_at` / `revoked_at`). `org_scope` populated from `consent.scope.org`. **D47.5** Sixth Repository method `sweep_consent_timeouts(now: DateTime<Utc>) -> RepositoryResult<Vec<ConsentId>>` — performs the storage-side scan (`SELECT id FROM consent WHERE state = "requested" AND deadline_at <= $now`) + flips each row to `TimedOut` + emits the `consent.timed_out` audit per row. Returns the list of flipped consent ids. The deadline is computed at consent creation time (CH-11 will wire that; for CH-10 the helper field is added but creation-side population stays out of scope). **D47.6** New `deadline_at: Option<DateTime<Utc>>` field on `Consent` struct + migration 0012 adds the SurrealDB column. `Option` because legacy rows + `Acknowledged`-by-default (implicit-policy) consents have no deadline. The sweeper only flips rows with `state = Requested AND deadline_at <= now`. **D47.7** Sweeper task at `server::state::spawn_consent_sweeper(repo, interval)` — a tokio task spawned at server startup that loops `tokio::time::sleep(interval)` then calls `repo.sweep_consent_timeouts(Utc::now())`. Configurable interval via `[consent] sweeper_interval_secs = 60` in `config/default.toml` (default 60s). Single-pod at v0 — multi-pod leader-election deferred to M7b (entry `CHK8S-D-XX` in the K8s deferred ledger). **D47.8** Forward-only revocation enforced at `legal_transition` (line `Revoked → _` returns `false` for everything). Tested via property-test that enumerates all 36 (state, state) pairs and asserts the legal table matches concept-doc §"Consent Lifecycle" exactly. **D47.9** Default response on timeout (deny / allow) is **NOT** computed by CH-10 — that's a Permission Check gating concern owned by CH-11. CH-10 stops at "the row reached `TimedOut` state on schedule"; CH-11 wires the gating semantic that maps `TimedOut` → deny / allow per org config. **D47.10** Per-policy minter logic remains out of scope. Implicit-policy auto-`Acknowledged` (CH-09 default) stays the only auto-mint. OneTime / PerSession policy minters land at CH-11 alongside the per-session gating. |

ADR file: [`m5_2/decisions/0047-consent-state-machine-and-sweeper.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0047-consent-state-machine-and-sweeper.md) (NEW).

---

## §6 — Prior-chunk regression re-verification

| Upstream | Invariant | Verification |
|---|---|---|
| Post-CH-23 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1223; 4 CI guards green; clippy clean under `-Dwarnings` | run all four guard scripts + workspace tests |
| CH-09 / ADR-0045 | `Consent` shape stays at 11 fields; `ConsentState` enum unchanged. CH-10 ADDS the new `deadline_at` field but doesn't restructure the existing fields. | `cargo test -p domain --lib model::nodes::tests` (CH-09 unit tests stay green) |
| CH-23 / ADR-0046 | Template C/D listeners + new HTTP handlers untouched | `cargo test -p server --test acceptance_system_flows_s05` |
| CH-21 / ADR-0040+0041 | Audit hash chain byte-stable on existing event types | `cargo test -p server --test acceptance_memory_extraction` |
| Migration runner | New migration 0012 applies once + safe to re-run | extend `migrations_test.rs` to assert version 12 + slug `consent_deadline` |

---

## §7 — Phases

**Phase count: 4** → audit envelope = **2 agents** (medium chunk).

### P1 — `domain::consents` module + transition pure-fns + property tests (~0.7d)

**Goal.** Land the dedicated module + transition function bodies + the legal-transition table + property tests.

**Deliverables.**

1. **`modules/crates/domain/src/consents/mod.rs`** — module declaration + re-exports.

2. **`modules/crates/domain/src/consents/state.rs`** — pure predicates:
   ```rust
   pub fn is_terminal(state: ConsentState) -> bool;        // Revoked, Declined, TimedOut, Expired
   pub fn legal_transition(from: ConsentState, to: ConsentState) -> bool;
   ```
   The legal table:
   - `Requested → Acknowledged | Declined | TimedOut`
   - `Acknowledged → Revoked` (gated by `revocable=true` at the higher transition layer)
   - any non-terminal → `Expired`
   - everything else → false (terminal states reject all transitions)

3. **`modules/crates/domain/src/consents/transitions.rs`** — pure-fn helpers:
   ```rust
   pub fn acknowledge(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError>;
   pub fn decline(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError>;
   pub fn revoke(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError>;
   pub fn mark_timed_out(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError>;
   pub fn mark_expired(consent: &Consent, at: DateTime<Utc>) -> Result<Consent, ConsentTransitionError>;
   ```
   Each function clones the input, validates legality, sets the timestamp field that semantically belongs to the transition (`responded_at` for ack/decline, `revoked_at` for revoke; the others don't update timestamps), and returns the new row.

4. **`ConsentTransitionError` enum** at `transitions.rs` with the three variants from §5 D47.2.

5. **Add `deadline_at: Option<DateTime<Utc>>`** field on `Consent` (in `model/nodes.rs`) with `#[serde(default)]`. Update CH-09's unit tests to populate the field with `None` so they keep passing.

6. **`mod.rs` of `domain/src/lib.rs`** — declare the new `pub mod consents;`.

**Tests (P1).** ~12 unit + property tests at `consents/transitions.rs::tests`:
- Each of the 5 transitions: happy path on the right `from` state.
- Each of the 5 transitions: rejects illegal `from` states.
- `acknowledge` from `Requested` sets `responded_at` to `at`.
- `decline` from `Requested` sets `responded_at` to `at`.
- `revoke` from `Acknowledged` sets `revoked_at` to `at`; rejected if `revocable == false`.
- Property test: 36-cell legal-transition table matches concept-doc spec verbatim.
- Property test: terminal states reject all 6 transitions.
- Property test: forward-only revocation — `Revoked → Acknowledged` always errors.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- Adding `deadline_at` to `Consent` breaks any unrelated CH-09 test that constructs the struct (the `#[serde(default)]` should shield, but a literal struct construction in a test would break).
- The `legal_transition` table can't be expressed declaratively in the way the concept doc shows; the table needs to support `Expired` from any non-terminal state which isn't a 1:1 enum match.

---

### P2 — Repository methods + audit-event builders + impls on both backends (~0.7d)

**Goal.** Land the 5 per-transition repo methods + the new audit-event builders + impls.

**Deliverables.**

1. **`RepositoryError::ConsentTransition { source: ConsentTransitionError }`** — new variant with `thiserror::From` impl.

2. **5 `Repository` trait methods** at `repository.rs`:
   ```rust
   async fn acknowledge_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
   async fn decline_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
   async fn revoke_consent(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
   async fn mark_consent_timed_out(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
   async fn mark_consent_expired(&self, id: ConsentId, at: DateTime<Utc>, actor: AgentId) -> RepositoryResult<AuditEventId>;
   ```
   Each method's compound tx: read consent → call matching pure-fn → if Ok, write new row + emit audit atomically.

3. **New audit-event module** `domain::audit::events::m5::consents` (NEW) with 5 builders following the `m5::edges` precedent (CH-23). Each emits `AuditClass::Logged` with diff `{consent_id, from_state, to_state, at}`.

4. **In-memory impls** at `domain::in_memory::InMemoryRepository` — straightforward HashMap-mutation pattern.

5. **SurrealStore impls** at `store::repo_impl` — `BEGIN TRANSACTION; UPDATE consent SET state, ...; CREATE audit_events ...; COMMIT TRANSACTION;` pattern (mirror CH-23's edge-creation tx).

**Tests (P2).** ~10 unit + integration tests:
- One happy-path test per transition method (in-memory).
- Cross-impl test: same operation against `SurrealStore::open_embedded` matches.
- Illegal-transition rejection (e.g., calling `acknowledge_consent` on an already-`Revoked` row returns `RepositoryError::ConsentTransition`).
- Audit-event emission round-trip: the audit event lands with the right `event_type` + `org_scope` + diff.
- Idempotency note: re-calling `acknowledge_consent` on an already-Acknowledged row errors (no-op idempotency at this layer; the upstream caller handles retry semantics).

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if the SurrealDB UPDATE pattern fails because of a SCHEMAFULL ASSERT clause we don't anticipate (e.g., `state` field rejects the new value because the migration's DEFAULT clause doesn't allow it).

---

### P3 — Migration 0012 (`deadline_at` column) + sweeper repo method + tokio task + config (~0.5d)

**Goal.** Wire the auto-timeout path end-to-end.

**Deliverables.**

1. **Migration `0012_consent_deadline.surql`** — adds the `deadline_at: option<string>` column to the `consent` table. Registered in `EMBEDDED_MIGRATIONS` at version 12, slug `consent_deadline`. Idempotent under repeated runs.

2. **`Repository::sweep_consent_timeouts(now)`** trait method:
   ```rust
   async fn sweep_consent_timeouts(&self, now: DateTime<Utc>) -> RepositoryResult<Vec<ConsentId>>;
   ```
   Implementation:
   - SurrealDB: `SELECT id FROM consent WHERE state = 'requested' AND deadline_at <= $now LIMIT 256` (cap to bound the per-tick blast radius). For each, run `mark_consent_timed_out` in its own tx.
   - In-memory: scan the HashMap, collect matching ids, call `mark_consent_timed_out` per id.

3. **`server::state::spawn_consent_sweeper(repo, interval)`** — spawns a `tokio::task` that loops:
   ```rust
   loop {
       tokio::time::sleep(interval).await;
       if let Err(e) = repo.sweep_consent_timeouts(Utc::now()).await {
           tracing::warn!(error = %e, "consent sweeper tick failed; retrying next interval");
       }
   }
   ```
   Returns a `JoinHandle<()>` so the server can store + cancel it on graceful shutdown.

4. **Config** — add `[consent] sweeper_interval_secs = 60` to `config/default.toml`. Wire into `ServerConfig` + thread through to the spawn site.

5. **K8s deferred ledger entry** at `m7b/architecture/deferred-from-ch-k8s-prep.md` — `CHK8S-D-XX: consent sweeper is single-pod-only at v0; multi-pod leader-election deferred to M7b`.

6. **Extend `migrations_test.rs`** to expect 12 rows + version 12 / slug `consent_deadline`.

**Tests (P3).** ~6 unit + integration tests:
- `sweep_consent_timeouts` flips eligible Requested-with-elapsed-deadline rows to TimedOut.
- `sweep_consent_timeouts` ignores Acknowledged + Declined + already-TimedOut rows.
- `sweep_consent_timeouts` ignores Requested rows whose `deadline_at` is `None` or in the future.
- Returns the list of flipped ids.
- Cross-impl: same behavior on `InMemoryRepository` and `SurrealStore::open_embedded`.
- Idempotency: re-running the sweep on the same fixture returns an empty list (no rows still eligible).

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- The SurrealDB query syntax for `WHERE deadline_at <= $now` doesn't work as expected (SurrealDB's optional + datetime-as-string handling can surprise).
- Per-tick blast radius (256 rows) interacts badly with the audit hash chain — emitting 256 audit events in one tick should stay byte-stable but worth confirming.

---

### P4 — ADR Accepted + drift closed + concept-doc bump + audit + seal (~0.3d)

**Goal.** Ratify ADR-0047. Close D-new-05. Spawn 2 audit agents. Seal.

**Deliverables.**

1. ADR-0047 flipped from `Proposed` → `Accepted`.
2. D-new-05 Status flipped to `remediated`. Lifecycle entry appended.
3. `drifts/README.md` row flipped + `Closes at` column updated.
4. `_concept-audit-matrix.md` row flipped from `partially-honored` to `honored`.
5. Concept doc `permissions/06-multi-scope-consent.md` verified-header bump.
6. K8s deferred ledger entry committed.
7. Spawn 2 audit agents per §11.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit reports a finding.

---

## §8 — Tests summary

- **Expected total at chunk close:** post-CH-23 (1223) + ~12 P1 + ~10 P2 + ~6 P3 = **~1251 tests**.
- **New test files:** unit + property tests inline at `consents/transitions.rs::tests`; integration tests added to `domain/tests/in_memory_ch10_consent_state.rs` (NEW) and `store/tests/repository_test.rs` (extended).
- **Property tests:** 36-cell legal-transition table; terminal-state rejection; forward-only revocation.
- **Sweeper tests:** ignore non-Requested + ignore null-deadline + ignore future-deadline + idempotent re-sweep.
- **Audit-chain canonical-bytes check:** ADR-0040/0041 invariant — re-run `acceptance_memory_extraction`.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4`.
2. Copy plan verbatim to `baby-phi/docs/specs/plan/build/ch-10-consent-state-machine-and-sweeper-<8hex>.md`.
3. Update placeholders in lines 4–5 of the archived copy.
4. `bash scripts/check-doc-links.sh`.

### Reading list (mandatory)

1. [`concepts/permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) §"Consent Lifecycle" (lines 369–414) + §"Forward-only revocation" (lines 416+) + §"Default response on timeout" (lines 316–348).
2. [`drifts/D-new-05.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-05.md) — full.
3. [`auth_requests/state.rs`](baby-phi/modules/crates/domain/src/auth_requests/state.rs) + [`transitions.rs`](baby-phi/modules/crates/domain/src/auth_requests/transitions.rs) — full files (precedent for the module shape).
4. [`auth_request_transition_props.rs`](baby-phi/modules/crates/domain/tests/auth_request_transition_props.rs) — proptest pattern reference.
5. [`model/nodes.rs`](baby-phi/modules/crates/domain/src/model/nodes.rs) lines 759–834 — current `Consent` + `ConsentState` shapes.
6. [`composites_m3.rs`](baby-phi/modules/crates/domain/src/model/composites_m3.rs) lines 41–68 — `ConsentPolicy` enum.
7. [ADR-0045](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0045-consent-node-full-shape.md) — CH-09 precedent.
8. [ADR-0046](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0046-template-cd-http-edges.md) — CH-23 precedent for compound-tx + audit-event-builder pattern.
9. [`store/migrations/0010_consent_full_shape.surql`](baby-phi/modules/crates/store/migrations/0010_consent_full_shape.surql) — migration pattern reference.

### Carry-forward invariants (verified at chunk-open)

- `cargo test --workspace -- --test-threads=1` ≈ 1223.
- 4 CI guards green.
- D-new-05 status `discovered`.
- ADR-0034..0046 Accepted; next-free = 0047.
- `git diff --stat HEAD -- modules/` empty.

---

## §10 — Close criteria (5-aspect)

- **Code aspect** — workspace builds; clippy under `RUSTFLAGS="-Dwarnings"` clean; `cargo test --workspace -- --test-threads=1` green at ~1251.
- **Docs aspect** — D-new-05 lifecycle remediated; concept-audit matrix row flipped honored; ADR-0047 Accepted; concept-doc verified-header bumped; K8s deferred ledger entry added.
- **phi-core leverage** — import-count delta = 0; positive/forbidden greps all match expected.
- **Concept alignment** — every §2 row at target status (lifecycle row honored; default-response row partially-honored per locked Q-CH-11 carve-out).
- **K8s readiness** — single-pod sweeper deferral recorded.

**Implementation confidence** = `claims-honored / claims-in-scope` = target **10/10**:
1. `domain::consents::{mod.rs, state.rs, transitions.rs}` exist.
2. `ConsentTransitionError` with 3 variants ships.
3. 5 pure-fn transition helpers exist + reject illegal transitions.
4. `Consent.deadline_at: Option<DateTime<Utc>>` field added; migration 0012 lands.
5. 5 per-transition `Repository` methods ship on both backends.
6. 5 new audit-event builders ship at `m5::consents`.
7. `Repository::sweep_consent_timeouts` ships on both backends.
8. `server::state::spawn_consent_sweeper` spawns a tokio task at startup.
9. K8s deferred entry recorded.
10. Property tests exhaustively cover the 36-cell legal-transition table.

---

## §11 — Audit plan

**2 agents** (medium chunk).

### Audit A — Code correctness + phi-core leverage

> You are auditing CH-10 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan at `docs/specs/plan/build/ch-10-consent-state-machine-and-sweeper-<8hex>.md`.
>
> 1. `domain::consents::state` exposes `is_terminal(ConsentState) -> bool` + `legal_transition(from, to) -> bool`. The 36-cell legal table matches concept doc 06 §"Consent Lifecycle" verbatim — there's a property test that asserts this.
> 2. `domain::consents::transitions` exposes 5 pure-fn helpers (`acknowledge`, `decline`, `revoke`, `mark_timed_out`, `mark_expired`) returning `Result<Consent, ConsentTransitionError>`.
> 3. `ConsentTransitionError` has 3 variants: `IllegalTransition { from, to }`, `NotRevocable { consent_id }`, `Terminal { state, attempted }`. Derives `thiserror::Error + Serialize + Deserialize`.
> 4. `Consent` struct gains `deadline_at: Option<DateTime<Utc>>` with `#[serde(default)]`. Migration 0012 adds the column. CH-09 unit tests still pass (existing fixtures default the new field to `None`).
> 5. 5 `Repository` per-transition methods exist on the trait + both backends. Each method's compound tx does read → pure-fn → write+audit atomically.
> 6. New audit-event builders exist at `domain::audit::events::m5::consents` — 5 builders, all `AuditClass::Logged`, diff carrying `{from_state, to_state, at}` + the timestamp field set by the transition.
> 7. `Repository::sweep_consent_timeouts(now) -> Vec<ConsentId>` ships on both backends, ignores non-Requested rows, ignores null-deadline rows, ignores future-deadline rows, returns the list of flipped ids.
> 8. `server::state::spawn_consent_sweeper(repo, interval)` spawns a `tokio::task` that calls the sweep on the configured interval. Configurable via `[consent] sweeper_interval_secs` in config.
> 9. `cargo test --workspace -- --test-threads=1` green at ~1251.
> 10. CI guards green; `check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports in `domain/src/consents/`.
> 11. CH-09 invariants intact: ADR-0045 still Accepted; D-new-04 still remediated.
> 12. CH-23 + CH-21 + CH-22 + CH-04 + CH-05 invariants intact (acceptance suites still green).
> 13. The forward-only revocation invariant is enforced by `legal_transition`: any input where `from = Revoked` returns `false`.

PASS/FAIL each. ≤ 600 words.

### Audit B — Concept fidelity + docs fidelity

> You are auditing CH-10's concept-fidelity + docs-fidelity. Read-only.
>
> 1. ADR-0047 Accepted at `m5_2/decisions/0047-consent-state-machine-and-sweeper.md` with sub-decisions D47.1–D47.10.
> 2. ADR-0047 Status field reads exactly `**Status: Accepted**` (one line, bold).
> 3. ADR-0047 documents the 3 locked forks (Q1 module layout, Q2 sweeper-in-chunk, Q3 per-transition repo methods).
> 4. ADR-0047 cross-references concept doc 06 (§"Consent Lifecycle"), drift D-new-05 (closed), ADR-0028 (event-bus), ADR-0042 §D42.3 (migration runner conforming criteria), ADR-0045 + ADR-0046 (CH-09 + CH-23 precedents).
> 5. D-new-05 Status = `remediated`; lifecycle entry for CH-10 chunk-seal present.
> 6. `drifts/README.md` row for D-new-05 flipped; "Closes at" → CH-10 ✓.
> 7. `_concept-audit-matrix.md` Consent-lifecycle row flipped from `partially-honored` to `honored`.
> 8. Concept doc `permissions/06-multi-scope-consent.md` verified-header bumped (CH-10 amendment line). Doc body UNCHANGED.
> 9. K8s deferred ledger at `m7b/architecture/deferred-from-ch-k8s-prep.md` carries the `CHK8S-D-XX: consent sweeper single-pod-only at v0` entry.
> 10. Plan archive at `plan/build/ch-10-consent-state-machine-and-sweeper-<8hex>.md` exists (slug-first naming).
> 11. CH-09 invariants intact: ADR-0045 still Accepted; D-new-04 still remediated; concept doc 06 retains the CH-09 amendment line + the new CH-10 amendment line above it.
> 12. CH-23 + CH-21 + CH-22 + CH-04 + CH-05 + CH-16 + CH-01 invariants intact (file-existence smoke test).
> 13. `migrations_test.rs` asserts row count = 12 + version 12 / slug `consent_deadline`.

PASS/FAIL each. ≤ 600 words.

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1251 passed / 0 failed

# 3. Positive greps
ls modules/crates/domain/src/consents/{mod.rs,state.rs,transitions.rs}                        # all 3 exist
grep -n "pub enum ConsentTransitionError" modules/crates/domain/src/consents/transitions.rs   # 1
grep -n "pub fn acknowledge\b\|pub fn decline\b\|pub fn revoke\b\|pub fn mark_timed_out\b\|pub fn mark_expired\b" modules/crates/domain/src/consents/transitions.rs   # ≥ 5
grep -n "fn acknowledge_consent\|fn decline_consent\|fn revoke_consent\|fn mark_consent_timed_out\|fn mark_consent_expired\|fn sweep_consent_timeouts" modules/crates/domain/src/repository.rs   # ≥ 6
ls modules/crates/store/migrations/0012_consent_deadline.surql                                # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0047-consent-state-machine-and-sweeper.md   # 1

# 4. Negative greps
grep -rn 'use phi_core::' modules/crates/domain/src/consents/                                 # 0

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-05.md   # 1

# 6. Targeted suites
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib consents                              # property + unit tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test in_memory_ch10_consent_state -- --test-threads=1
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test repository_test surreal_consent -- --test-threads=1
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migrations_test -- --test-threads=1   # version 12

# 7. Carry-forward sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s05 -- --test-threads=1   # CH-23 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1  # CH-21 still green
```

---

## What this plan does NOT do

- HTTP / CLI endpoints for subordinate acknowledge / decline (M6+).
- Channel notification delivery (M6+).
- Per-policy minter rewrite (CH-11).
- Permission Check engine timeout-default-response gating (CH-11).
- Multi-pod leader-election for the sweeper (M7b).
- Retroactive cleanup of artifacts touched during a since-revoked grant (concept-doc 06 explicitly rules this out).

---

## Critical files

**New:**
- `modules/crates/domain/src/consents/{mod.rs, state.rs, transitions.rs}` — module + pure-fns.
- `modules/crates/domain/src/audit/events/m5/consents.rs` — 5 audit-event builders.
- `modules/crates/store/migrations/0012_consent_deadline.surql` — `deadline_at` column.
- `modules/crates/domain/tests/in_memory_ch10_consent_state.rs` — integration tests.
- `docs/specs/v0/implementation/m5_2/decisions/0047-consent-state-machine-and-sweeper.md` — ADR.

**Modified:**
- `modules/crates/domain/src/lib.rs` — declare `pub mod consents`.
- `modules/crates/domain/src/model/nodes.rs` — add `deadline_at` field.
- `modules/crates/domain/src/repository.rs` — 6 new methods + new `RepositoryError::ConsentTransition` variant.
- `modules/crates/domain/src/in_memory.rs` — 6 impls.
- `modules/crates/store/src/repo_impl.rs` — 6 impls.
- `modules/crates/store/src/migrations.rs` — register version 12.
- `modules/crates/store/tests/migrations_test.rs` — assert version 12.
- `modules/crates/store/tests/repository_test.rs` — cross-impl tests for the new methods.
- `modules/crates/server/src/state.rs` — `spawn_consent_sweeper` + wire into startup.
- `config/default.toml` — `[consent] sweeper_interval_secs`.
- `docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` — single-pod sweeper entry.
- Drift files: `D-new-05.md`, `drifts/README.md`, `_concept-audit-matrix.md`.
- Concept doc: `permissions/06-multi-scope-consent.md` — header bump only.

**Unchanged (verified by close-audit):**
- `modules/crates/domain/src/permissions/manifest/mod.rs` — `ConsentIndex` projection.
- The Permission Check engine — no read-side state-aware gating at this chunk.

---

## Estimated effort

~2 engineer-days:
- 0.7d — P1 module + pure-fns + property tests (~12 tests).
- 0.7d — P2 repo methods + audit builders + impls + cross-impl tests (~10 tests).
- 0.5d — P3 migration 0012 + sweeper + tokio task + config + K8s deferred entry (~6 tests).
- 0.3d — P4 ADR + drift closure + concept-doc bump + 2 audit agents + seal.
