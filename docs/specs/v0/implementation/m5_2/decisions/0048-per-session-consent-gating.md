<!-- Last verified: 2026-05-03 by Claude Code -->

# ADR-0048 — Per-Session consent gating (Step 6 real body + ApprovalMode + per-policy minters + timeout-default-response + approval-timeout)

**Status: Accepted**

**Date:** 2026-05-03
**Chunk:** CH-11
**Closes:**
- [`D-new-17`](../../m5_1/drifts/D-new-17.md) (HIGH) — Per-Session consent gating incomplete; Step 6 stub-only.

**Cross-chunk dependencies:**
- Builds on [ADR-0045](0045-consent-node-full-shape.md) (CH-09 — `Consent` 11-field shape + `ConsentState` enum + `ConsentScope { org, templates, actions }`).
- Builds on [ADR-0047](0047-consent-state-machine-and-sweeper.md) (CH-10 — state machine + sweeper + per-transition repo methods + `deadline_at: Option<DateTime<Utc>>`).
- Builds on [ADR-0028](../../m4/decisions/0028-domain-event-bus.md) (event-bus fail-safe — durable write before reactive emit; `Repository::request_consent` follows the pattern).
- Builds on [ADR-0042](0042-storage-backend-configurable.md) §D42.3 #5 (forward-only idempotent migration runner — migration 0013 conforms).
- Confirms neutrality against [ADR-0033](0033-k8s-prep-refactors.md) §D33 conforming-criteria — CH-11 introduces no new K8s blocker (single-pod sweeper from CH-10's `CHK8S-D-09` is unchanged).
- Closes the consent triad opened by ADR-0045 (CH-09) + ADR-0047 (CH-10).

---

## Context

CH-09 + CH-10 sealed the **shape** + **lifecycle** of the Consent node — the 11-field record per concept doc 06 §"Consent Node" lines 351–363, the 6-state machine per §"Consent Lifecycle" lines 369–414, and the auto-timeout sweeper that flips abandoned `Requested` consents to `TimedOut`. But the **read path** — the Permission Check engine's Step 6 at [`engine.rs::step_6_consent_gating`](../../../../../../modules/crates/domain/src/permissions/engine.rs) — still consulted a stub: it queried `ConsentIndex::is_acknowledged(subordinate, org)` (a `(AgentId, OrgId)` tuple lookup populated from the existing `Acknowledged` rows) and ignored everything else. There was no concept of a `Per-Session` policy at runtime: a supervisor with a Per-Session-policy template grant saw the same `Pending` outcome forever (because no `Requested` consent was ever minted, and no `Acknowledged` consent was filtered by session). The default-response-on-timeout semantic from concept doc 06 line 349 (`approval_timeout_default_response: deny|allow`) was silent in code; the approval-timeout duration semantic from line 322 (`approval_timeout: project_duration | Fixed(d)`) was equally silent.

Drift D-new-17 (HIGH) tracked the gap. CH-11 closes it by:

- Adding `Grant.approval_mode: ApprovalMode` so templates can declare which policy issued them.
- Adding `ConsentScope.session_id: Option<SessionId>` so consents can be matched per session.
- Adding `Organization.approval_timeout: ApprovalTimeout` and `Organization.approval_timeout_default_response: TimeoutResponse` so the engine can compute deadlines and interpret `TimedOut` per org config.
- Wiring the real Step 6 body that branches on `approval_mode` × `(consent state)` per the response table below.
- Shipping per-policy minters (`request_one_time`, `request_per_session`) and a `Repository::request_consent` compound-tx helper that creates the `Requested` row + emits the `consent.requested` audit atomically.
- Wiring the launch handler to populate `current_session` + `timeout_default_response` on `CheckContext`, compute `deadline_at` from `org.approval_timeout`, and mint the `Requested` row when Step 6 surfaces `Pending` for the first time.

The exploration phase confirmed:

- Concept doc 06 line 244 explicitly mandates `approval_mode: subordinate_required` on template-issued grants — the field has been concept-public since the initial draft and has always been silent in code.
- The 5 forks identified at planning (ApprovalMode shape; how-far-to-go on subordinate dispatch; org-config breadth; per-session matching key; how Step 6 receives `session_id`) all had cleanly-articulated alternatives. The user locked all 5 at plan-review (2026-05-02).
- Concept doc 06 lines 322–349 (Approval Timeout + Default response) are typed Rust by this chunk. Line 416 (Channel notification) explicitly remains M6+ — no scope creep.

The user-decided forks at plan-review (2026-05-02) locked five open questions; see "Locked forks" section below.

---

## Locked forks

All five forks were **locked by the user 2026-05-02** at plan-review. Each fork's user-decision is recorded here for traceability; the body of every decision below is written against these choices.

### Fork F1 — `ApprovalMode` representation on `Grant` → **F1.A**

New typed enum `ApprovalMode { Implicit, SubordinateRequired { policy: ConsentPolicy }, HumanApprovalRequired }` on `Grant` as `pub approval_mode: ApprovalMode` with `#[serde(default)]` defaulting to `Implicit`. The `policy` field denormalises the org's `ConsentPolicy` so the engine doesn't have to re-look-up the org node at Step 6 — and matches concept doc 06 line 244 ("Templates issue grants with `approval_mode: subordinate_required`"). Alternatives F1.B (string discriminator) and F1.C (separate `consent_policy_at_issue` field) rejected as less typed / more error-prone.

### Fork F2 — Subordinate-approval request dispatch — how far at this chunk → **F2.B**

Engine returns `Pending` (pure-fn, no I/O) **and** the launch handler / Step 6 caller mints the `Requested` consent record alongside via a `Repository::request_consent(...)` helper. New row materialises with `state = Requested`, `deadline_at = computed-per-F3.A` (CH-10's sweeper picks it up). Channel notification stays out (M6+). Alternatives F2.A (engine-only; no minting) and F2.C (full `subordinate_inbox` queue table) rejected — F2.A leaves `Requested` consents unminted and the sweeper has nothing to flip; F2.C is M6+ scope creep.

### Fork F3 — `org.approval_timeout` + `org.approval_timeout_default_response` — add now or defer → **F3.A**

Add **both** fields now via migration 0013, full concept-doc fidelity. `Organization.approval_timeout: ApprovalTimeout` (typed enum `ProjectDuration | Fixed { duration: chrono::Duration }`; default `ProjectDuration` per concept doc 06 line 322) plus `Organization.approval_timeout_default_response: TimeoutResponse` (typed enum `Deny | Allow`; default `Deny` per concept doc 06 line 349). Migration 0013 adds **two** organization columns. The engine + launch handler consume both at the deadline-computation site for shape A/B/C/D sessions per concept doc 06 lines 336–347. Adds ~0.4d to the original estimate; the user accepted the trade for full concept-doc fidelity. Alternatives F3.B (defer the timeout-duration field; ship only default-response) and F3.C (defer both) rejected — F3.B leaves the deadline-computation path stranded; F3.C leaves the timeout-default-response semantic silent.

### Fork F4 — Per-Session consent matching key → **F4.A**

Add `session_id: Option<SessionId>` to `ConsentScope`. `None` = not session-scoped (Implicit / OneTime); `Some(id)` = applies only to reads on session `id`. The `ConsentIndex` projection extends to `(subordinate, org, Option<session_id>)`. Migration 0013 leaves the SurrealDB schema untouched here because `scope` is already `FLEXIBLE TYPE object` per ADR-0045 §D45.5 — the new field rides under the FLEXIBLE shield. Alternatives F4.B (keep the scope tuple unchanged; encode session in `templates` or `actions`) and F4.C (separate `ConsentSessionScope` parallel struct) rejected — both contradict concept doc 06 §"Per-Session Consent" line 332's "scoped to a single session" verbatim.

### Fork F5 — How does Step 6 receive `session_id` at Permission Check time → **F5.A**

Add `current_session: Option<SessionId>` to `CheckContext`. Mirrors `current_org` / `current_project` ambient-context pattern. Launch handler / preview handler / acceptance harness fill it in. `None` for class-level invocations. Alternatives F5.B (thread `session_id` through every engine call signature) and F5.C (special-case the launch path) rejected — F5.B is a 7-call-site signature churn; F5.C creates a non-orthogonal code path.

---

## Decision

### D48.1 — `Grant.approval_mode: ApprovalMode` field added

Per Fork F1 (locked F1.A). The typed enum lives at [`domain::model::nodes::ApprovalMode`](../../../../../../modules/crates/domain/src/model/nodes.rs):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApprovalMode {
    #[default]
    Implicit,
    SubordinateRequired { policy: ConsentPolicy },
    HumanApprovalRequired,
}
```

`#[serde(tag = "kind")]` so the wire format is `{"kind": "subordinate_required", "policy": "per_session"}` — round-trips through SurrealDB's FLEXIBLE-object storage. Pre-CH-11 grants decode as `ApprovalMode::Implicit` via `#[serde(default)]`, so the engine short-circuits them through the no-consent-needed path.

### D48.2 — `ConsentScope.session_id: Option<SessionId>` field added

Per Fork F4 (locked F4.A). `None` for non-session-scoped consents (Implicit / OneTime); `Some(id)` for Per-Session consents. Migration 0013 extends the FLEXIBLE `scope` object — no schema change required because `scope` is already `FLEXIBLE TYPE object` (per ADR-0045 §D45.5). Pre-CH-11 wire payloads decode with `session_id: None` via `#[serde(default)]`.

### D48.3 — `CheckContext.current_session: Option<SessionId>` field added

Per Fork F5 (locked F5.A). Mirrors `current_org` / `current_project` ambient-context pattern. `None` for class-level invocations. Engine returns `Decision::Denied(failed_step: Consent, reason: NoSessionContext { ... })` if a `SubordinateRequired { policy: PerSession }` lookup fires while `current_session` is `None` — captures the launch-handler-bug case (per-session policy + class-level call) cleanly.

In the same spirit, `CheckContext.timeout_default_response: TimeoutResponse` is added so the engine can interpret a `TimedOut` consent state without needing storage access. The launch handler populates this from `org.approval_timeout_default_response`; defaults to `TimeoutResponse::Deny` for back-compat callers that don't populate it.

### D48.4 — `Organization` config — both `approval_timeout` and `approval_timeout_default_response`

Per Fork F3 (locked F3.A). Both fields ship at CH-11 via migration 0013 (full concept-doc fidelity, no deferral).

`Organization.approval_timeout: ApprovalTimeout` is a typed enum `ProjectDuration | Fixed { duration: chrono::Duration }` (default `ProjectDuration` per concept doc 06 line 322). `Organization.approval_timeout_default_response: TimeoutResponse` is a typed enum `Deny | Allow` (default `Deny` per concept doc 06 line 349). Both fields use `#[serde(default)]` so legacy rows decode cleanly.

### D48.5 — Engine Step 6 real body branches on `winner.grant.approval_mode`

The new body at [`engine.rs::step_6_consent_gating`](../../../../../../modules/crates/domain/src/permissions/engine.rs) replaces the old `template_gated_auth_requests` stub:

- `ApprovalMode::Implicit` → no consent gate; continue to Allow.
- `ApprovalMode::SubordinateRequired { policy: ConsentPolicy::Implicit }` → no consent gate (the issuing org's policy was Implicit at issue time; consent is auto-Acknowledged).
- `ApprovalMode::SubordinateRequired { policy: ConsentPolicy::OneTime }` → look up `(subordinate, org, None)` in `ConsentIndex`. Result mapped per D48.6 response table.
- `ApprovalMode::SubordinateRequired { policy: ConsentPolicy::PerSession }` → look up `(subordinate, org, Some(current_session))` in `ConsentIndex`. If `current_session` is `None`, return `Decision::Denied(failed_step: Consent, reason: NoSessionContext)`. Otherwise mapped per D48.6 response table.
- `ApprovalMode::HumanApprovalRequired` → reserved; engine returns `Decision::Pending { awaiting: HumanApprovalRequired }` with a marker. No real handling at CH-11; placeholder for future human-approval chunks.

The legacy `template_gated_auth_requests` field is preserved on `CheckContext` for back-compat with M1 callers that haven't populated `approval_mode`. The new body consults `approval_mode` first and falls back to the legacy path only when `approval_mode == Implicit` (so M1 callers get the same behaviour they had pre-CH-11).

### D48.6 — Engine response table for `SubordinateRequired` lookups

| Lookup result | Decision returned |
|---|---|
| `Acknowledged` | proceed to Allow |
| not present (no row) | return `Decision::Pending(AwaitingConsent)` (mint at handler) |
| `Requested` (already minted) | return `Decision::Pending(AwaitingConsent)` |
| `TimedOut` + org `default_response = Deny` | return `Denied(failed_step: Consent, reason: ConsentTimedOutDeny)` |
| `TimedOut` + org `default_response = Allow` | proceed to Allow |
| `Declined` | return `Denied(failed_step: Consent, reason: ConsentDeclined)` |
| `Revoked` | return `Denied(failed_step: Consent, reason: ConsentRevoked)` |
| `Expired` | return `Denied(failed_step: Consent, reason: ConsentExpired)` |

Five new `DeniedReason` variants land at [`permissions::decision::DeniedReason`](../../../../../../modules/crates/domain/src/permissions/decision.rs): `ConsentDeclined`, `ConsentRevoked`, `ConsentExpired`, `ConsentTimedOutDeny`, and `NoSessionContext`. All five map to `FailedStep::Consent` (the existing M1 variant).

### D48.7 — `ConsentIndex` extends with per-session lookup methods

The internal store at [`permissions::manifest::ConsentIndex`](../../../../../../modules/crates/domain/src/permissions/manifest/mod.rs) changes from `HashSet<(AgentId, OrgId)>` to `HashMap<(AgentId, OrgId, Option<SessionId>), ConsentState>`:

```rust
pub fn lookup(&self, subordinate: AgentId, org: OrgId, session_id: Option<SessionId>) -> Option<ConsentState>;
pub fn is_acknowledged_for_session(&self, subordinate: AgentId, org: OrgId, session_id: SessionId) -> bool;
pub fn is_acknowledged(&self, subordinate: AgentId, org: OrgId) -> bool;  // preserved (back-compat)
```

The existing `is_acknowledged(subordinate, org)` remains as a thin wrapper: `lookup(subordinate, org, None) == Some(Acknowledged)`. New constructor `ConsentIndex::project_from_repo(repo, subordinate)` projects the map at engine call time by walking `Repository::list_consents_for_subordinate`.

### D48.8 — Per-policy consent-request minters at `domain::consents::minters`

A new module file [`modules/crates/domain/src/consents/minters.rs`](../../../../../../modules/crates/domain/src/consents/minters.rs) ships two pure-fn helpers:

```rust
pub fn request_one_time(subordinate: AgentId, org: OrgId, deadline_at: Option<DateTime<Utc>>) -> Consent;
pub fn request_per_session(subordinate: AgentId, org: OrgId, session_id: SessionId, deadline_at: Option<DateTime<Utc>>) -> Consent;
```

Each returns a freshly-constructed `Consent` with `state = Requested`, the right `scope.session_id` value (`None` for OneTime, `Some(session_id)` for PerSession), `requested_at = Utc::now()`, `responded_at = None`, `revocable = true`, and `provenance = "engine:step_6@<ISO-8601>"`. The launch handler invokes the right minter when Step 6 surfaces `Pending` for the first time on a session.

### D48.9 — `Repository::request_consent` compound-tx method

```rust
async fn request_consent(&self, consent: &Consent) -> RepositoryResult<AuditEventId>;
```

Compound tx: CREATE consent row + emit `consent.requested` audit event atomically. Implementations on both backends (in-memory + SurrealDB) follow the CH-10 pattern (`acknowledge_consent` etc.). Returns the audit event id so callers can pivot back to the audit row.

Two new read methods complete the read path promised at ADR-0045 §D45.7:

```rust
async fn list_consents_for_subordinate(&self, agent_id: AgentId) -> RepositoryResult<Vec<Consent>>;
async fn get_consent(&self, id: ConsentId) -> RepositoryResult<Option<Consent>>;
```

Both ship on both backends. The launch handler's `ConsentIndex::project_from_repo` path uses `list_consents_for_subordinate` to build the `(subordinate, org, session_id) -> ConsentState` map at engine call time.

### D48.10 — `consent_requested` audit-event builder

A new builder at [`domain::audit::events::m5::consents::consent_requested`](../../../../../../modules/crates/domain/src/audit/events/m5/consents.rs) (extends the file CH-10 created). `AuditClass::Logged` — routine relationship-mutation traffic. The diff carries `(consent_id, subordinate, org, session_id: Option<SessionId>, deadline_at: Option<DateTime<Utc>>)`. `org_scope` populated from `consent.scope.org`. The single-emitter pattern from CH-10 is preserved: only `Repository::request_consent` calls this builder.

### D48.11 — Typed `ApprovalTimeout` enum placement

`ApprovalTimeout` lives at [`domain::model::composites_m3::ApprovalTimeout`](../../../../../../modules/crates/domain/src/model/composites_m3.rs) mirroring `ConsentPolicy` placement at `composites_m3.rs:41`. `composites_m3` is the canonical home for org-level configuration enums consumed by the wizard payload + migration ASSERT clauses. `#[serde(tag = "kind", rename_all = "snake_case")]` so the wire format is `{"kind": "project_duration"}` or `{"kind": "fixed", "duration": "PT24H"}`. The `Duration` type is `chrono::Duration` serialised via the chrono ISO-8601 helper. Default = `ProjectDuration` (matches concept doc 06 line 322).

`TimeoutResponse` lives at `domain::model::composites_m3::TimeoutResponse` for symmetry. Two variants `Deny | Allow` with snake-case serde + `Default = Deny` (matches concept doc 06 line 349).

---

## Consequences

**Positive:**
- Drift D-new-17 closed; concept-doc fidelity restored on the read-path / per-session-matching / approval-timeout / default-response axes simultaneously.
- The consent triad CH-09 + CH-10 + CH-11 is sealed; the Per-Session consent policy is now operable end-to-end (subject to Channel notification, which remains explicit M6+ scope).
- Five new `DeniedReason` variants extend the engine's diagnostic vocabulary cleanly; every `Denied(Consent, ...)` now carries an actionable reason for operators.
- The `(subordinate, org, Option<session_id>) -> ConsentState` projection composes cleanly with the existing `is_acknowledged` back-compat surface; M1 callers that don't populate `current_session` get the same behaviour they had pre-CH-11.
- Migration 0013 follows the forward-only idempotent pattern (per ADR-0042 §D42.3 #5); no down-script needed.
- `#[serde(default)]` shields on `Grant.approval_mode`, `ConsentScope.session_id`, `Organization.approval_timeout`, and `Organization.approval_timeout_default_response` mean every legacy wire-format payload still decodes cleanly.

**Negative:**
- Adding `Organization.approval_timeout` + `Organization.approval_timeout_default_response` cascades to ~10–15 literal-struct test fixture sites; each gets a one-line addition. Acceptable cost of full concept-doc fidelity (per F3.A locked decision).
- The `template_gated_auth_requests` legacy field on `CheckContext` is now redundant for new callers; it stays for M1 back-compat. Soft-deprecation only at this chunk; removal lands at a future cleanup chunk.
- The "min(fixed, project_duration)" effective-timeout rule (concept doc 06 line 347) is recorded as a P3 deferred-from-CH-11 entry — non-blocking for D-new-17 closure since it only reduces the timeout, never extends it.

**Neutral:**
- K8s-neutral. No new K8s blockers (CHK8S-D-09 single-pod sweeper from CH-10 unchanged). Migration 0013 conforms to ADR-0033 D33.2.
- Channel notification (Slack / email / web UI delivery of consent requests) explicitly remains M6+ per concept doc 06 line 416. Subordinate-facing HTTP / CLI ack/decline endpoints also M6+. No silent gap — these are forward-scoped.
- The `human_approval_required` `ApprovalMode` variant ships as a forward-compat placeholder; engine returns `Pending` with a marker but no real handling. Future chunks own the wire-up.

---

## Cross-references

- Concept doc: [`permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md):
  - Line 244 — "Templates issue grants with `approval_mode: subordinate_required`"
  - Lines 297–349 — §"Per-Session Consent" full state diagram + flow
  - Line 322 — `approval_timeout: project_duration` (default) + Fixed alternative
  - Line 349 — Default response on timeout = `deny` (default), `allow` opt-in
  - Lines 407–414 — §"Per-policy mapping" (implicit / one_time / per_session)
- Drift closed: [`D-new-17`](../../m5_1/drifts/D-new-17.md).
- ADR-0045 — CH-09 precedent (Consent struct shape; ConsentScope; ConsentState enum).
- ADR-0047 — CH-10 precedent (state machine + per-transition repo methods + sweeper + `deadline_at`).
- ADR-0028 — event-bus fail-safe; durable write before reactive emit.
- ADR-0042 §D42.3 #5 — forward-only idempotent migration runner; migration 0013 conforms.
- ADR-0033 §D33 — K8s readiness conforming-criteria; CH-11 confirmed neutral.
- Plan archive: [`plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md`](../../../../plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md).

---

## Verification

- Workspace tests: `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` green at **1319** / 0 failed / 1 ignored (post-CH-10 baseline 1265 + ~54 new tests across P1–P3).
- Clippy under `RUSTFLAGS="-Dwarnings"`: clean.
- 4 CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
- Positive greps:
  - `pub enum ApprovalMode` (1) at `domain/src/model/nodes.rs`.
  - `pub approval_mode: ApprovalMode` (1) on `Grant`.
  - `pub session_id: Option<SessionId>` (1) on `ConsentScope`.
  - `pub enum ApprovalTimeout` (1) at `domain/src/model/composites_m3.rs`.
  - `pub enum TimeoutResponse` (1) at `composites_m3.rs`.
  - `pub approval_timeout: ApprovalTimeout` (1) on `Organization`.
  - `pub approval_timeout_default_response: TimeoutResponse` (1) on `Organization`.
  - `pub current_session: Option<SessionId>` (1) on `CheckContext`.
  - `domain::consents::minters::{request_one_time, request_per_session}` exist.
  - `Repository::{request_consent, list_consents_for_subordinate, get_consent}` exist on both backends.
  - Migration `0013_per_session_consent_gating.surql` exists; registered at version 13 / slug `per_session_consent_gating`.
  - Audit-event builder `consent_requested` at `audit/events/m5/consents.rs`.
  - New acceptance suite at `server/tests/acceptance_per_session_consent_gating.rs`.
- Forbidden greps:
  - `grep -rn 'use phi_core::' modules/crates/domain/src/permissions/engine.rs` → 0.
  - `grep -rn 'use phi_core::' modules/crates/domain/src/consents/` → 0.
  - `grep -rn 'use phi_core::' modules/crates/domain/src/audit/events/m5/consents.rs` → 0.
- Carry-forward green: CH-09 + CH-10 invariants (consent shape + state machine + sweeper); CH-21 (memory extraction); CH-22 (agent catalog); CH-23 (Template C/D edges); ADR-0033 (K8s prep).
