<!-- Last verified: 2026-05-02 by Claude Code (chunk-planner agent, iter 2) -->

# CH-11 — Per-Session consent gating

**Plan file token:** `d5428c43` (generated 2026-05-02 at chunk-open via `openssl rand -hex 4`).
**Plan archive path:** `baby-phi/docs/specs/plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md` (folder-style, multi-agent cycle).
**Chunk ID:** CH-11 (first multi-agent-pipeline cycle in baby-phi).
**Severity:** ⚠ HIGH (closes D-new-17; closes the consent triad CH-09 + CH-10 + CH-11).
**Expected effort:** ~2.4 engineer-days (F3.A locked: full Org config — `approval_timeout` + `approval_timeout_default_response`). Original forward-scope estimate was ~2 days; the user's locked F3.A choice adds ~0.4d for the second org-field surface (typed `ApprovalTimeout` enum + migration column + deadline-computation wiring + tests).
**Hard prerequisites:** **CH-09** (sealed — `Consent` 11-field shape + `ConsentState` enum), **CH-10** (sealed — state machine + sweeper + per-transition repo methods + `deadline_at` field). Both ADRs (0045, 0047) accepted; D-new-04 + D-new-05 remediated.
**Chunks unblocked at close:** none. Closes the consent triad. (Down-stream M6+ work — Channel notification, subordinate inbox, HTTP/CLI ack/decline endpoints — remains scoped to M6+ as documented in CH-10 ADR-0047.)

---

## Forks for orchestrator

All forks **LOCKED by user 2026-05-02**. Recorded here for traceability; the body of this plan is written against these decisions.

### F1 — `ApprovalMode` representation on `Grant`

**LOCKED: F1.A** — new typed enum `ApprovalMode { Implicit, SubordinateRequired { policy: ConsentPolicy }, HumanApprovalRequired }` on `Grant` as `pub approval_mode: ApprovalMode` with `#[serde(default)]` defaulting to `Implicit`. The `policy` carried inside `SubordinateRequired` denormalises the org's ConsentPolicy so the engine doesn't have to re-look-up the org node at Step 6 — and matches concept doc 06 line 244 ("Templates issue grants with `approval_mode: subordinate_required`").

### F2 — Subordinate-approval request dispatch — how far at this chunk

**LOCKED: F2.B** — engine returns `Pending` (pure-fn, no I/O) **and** the launch handler / Step 6 caller mints the `Requested` consent record alongside via a small `Repository::request_consent(...)` helper. New row materialises with `state = Requested`, `deadline_at = computed-per-F3.A` (CH-10's sweeper picks it up). Channel notification stays out (M6+).

### F3 — `org.approval_timeout` + `org.approval_timeout_default_response` — add now or defer

**LOCKED: F3.A** — add **both** fields now via migration 0013, full concept-doc fidelity. Concretely:
- `Organization.approval_timeout: ApprovalTimeout` — typed enum `ApprovalTimeout { ProjectDuration, Fixed(Duration) }` (where `Duration` = `chrono::Duration`). Default = `ProjectDuration` per concept doc 06 line 322.
- `Organization.approval_timeout_default_response: TimeoutResponse` — typed enum `Deny | Allow`. Default = `Deny` per concept doc 06 line 349.
Migration 0013 adds **two** organization columns. The engine + launch handler consume both at the deadline-computation site for shape A/B/C/D sessions per concept doc 06 lines 336–347. **Adds ~0.4d** to the original estimate; the user accepted this trade for full concept-doc fidelity.

### F4 — Per-Session consent matching key — `(subordinate, org, session_id)` triplet

**LOCKED: F4.A** — add `session_id: Option<SessionId>` to `ConsentScope`. `None` = not session-scoped (Implicit / OneTime); `Some(id)` = applies only to reads on session `id`. The `ConsentIndex` projection extends to `(subordinate, org, Option<session_id>)`. Migration 0013 leaves the SurrealDB schema untouched here because `scope` is already `FLEXIBLE TYPE object` per ADR-0045 §D45.5 — the new field rides under the FLEXIBLE shield.

### F5 — How does Step 6 receive `session_id` at Permission Check time

**LOCKED: F5.A** — add `current_session: Option<SessionId>` to `CheckContext`. Mirrors `current_org` / `current_project` ambient-context pattern. Launch handler / preview handler / acceptance harness fill it in. `None` for class-level invocations.

---

## Context

### The simple version

CH-09 + CH-10 sealed the **shape** + **lifecycle** of the Consent node. `Consent` carries 12 fields (11 concept-doc fields + the `deadline_at` CH-10 added); the state machine validates transitions; the sweeper auto-flips abandoned `Requested` consents to `TimedOut`. But the **read path** — the Permission Check engine's Step 6 — still uses a stub: it consults `ConsentIndex::is_acknowledged(subordinate, org)` (a `(AgentId, OrgId)` tuple lookup populated from the existing `Acknowledged` rows) and ignores everything else. There is no concept of a `Per-Session` policy at runtime: a supervisor with a Per-Session-policy template grant sees the same `Pending` outcome forever (because no `Requested` consent is ever minted, and no `Acknowledged` consent is filtered by session).

CH-11 closes drift D-new-17 by wiring **real** Step 6 gating logic for all three policies (Implicit / OneTime / PerSession), and by adding the consent-creation side that mints `Requested` consents when a Per-Session grant fires for the first time on a new session. The chunk also wires the timeout-default-response semantic + the timeout-duration semantic (CH-10 deferred both — `TimedOut → deny` per default + deadline computation per `Org.approval_timeout`).

### What this chunk does NOT do

- Does NOT ship Channel notification (Slack / email / web UI delivery of consent requests). Concept doc 06 line 416 — explicitly M6+.
- Does NOT ship subordinate-facing HTTP / CLI endpoints to acknowledge / decline. M6+.
- Does NOT ship a `subordinate_inbox` queue table (rejected option F2.C).
- Does NOT extend `Grant.descends_from` to track Template provenance — that surface is already wired through `template_gated_auth_requests`.
- Does NOT introduce the `human_approval_required` `ApprovalMode` variant. The variant is named in the typed enum (per F1.A) but no engine path consumes it; it's a forward-compat placeholder.

### Forward-scope reference

[`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §1 lines 114–119.

### Concept-doc anchor

[`concepts/permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) §"Per-Session Consent" (lines 297–349) + §"Consent Policy" (lines 230–256) + §"Approval Timeout" (lines 317–347) + §"Default response on timeout" (line 349) + §"Per-policy mapping" table (lines 407–414).

---

## §1 — Why this chunk

CH-09 and CH-10 together delivered the typed Consent **shape** and **lifecycle** but left the **read path** stubbed. Drift D-new-17 (HIGH, `permissions/engine.rs:106` Step 6 stub) tracks the gap; closing it closes the consent triad and operationalises the Per-Session consent policy that orgs can already configure on Organization but cannot exercise at runtime. CH-11 ships (1) an `ApprovalMode` enum on `Grant` plus migration 0013 that adds the column, (2) `session_id` on `ConsentScope`, (3) a real Step 6 body that branches on grant.approval_mode + org consent_policy + (subordinate, org, session) consent state, mapping `Acknowledged → Allow`, `Requested → Pending`, `TimedOut → deny|allow per org default-response`, `Declined / Revoked → Denied`, (4) a launch-side mint helper `request_per_session_consent(subordinate, org, session, deadline)` that creates the `Requested` row when Step 6 surfaces `Pending` for the first time on a session, (5) **both** `Organization.approval_timeout: ApprovalTimeout` (typed enum `ProjectDuration | Fixed(Duration)`; default `ProjectDuration`) **and** `Organization.approval_timeout_default_response: TimeoutResponse` (default `Deny`) with migration 0013 adding the two columns, and (6) acceptance tests covering Implicit / OneTime / PerSession paths against the engine.

**Quality-over-speed restatement.** *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* This chunk's specific application: every assertion in §2 cites a verbatim concept-doc line, every fork is locked above with the user's choice, and the deferred items (Channel notification, subordinate HTTP/CLI endpoints) ship with explicit successor-chunk references — not with silent gaps.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | §"Per-Session Consent" line 244 | "Templates auto-issue grants with `approval_mode: subordinate_required`; every read attempt blocks until the subordinate approves it" | contradicted (no `approval_mode` field on `Grant`; engine treats every template-gated grant identically) | honored — `Grant.approval_mode: ApprovalMode` ships; engine Step 6 branches on it |
| `permissions/06-multi-scope-consent.md` | §"Per-Session Consent" lines 299–313 (state diagram) | Permission Check passes → if `subordinate_required`, notify + wait; on Approved → read proceeds; on Denied → read denied; on Timeout → default response | partially-honored (engine returns `Pending` for any template-gated grant lacking consent; doesn't differentiate per-session from one-time) | honored — engine reads `(subordinate, org, session_id)` for `PerSession`; `(subordinate, org, None)` for `OneTime`; auto-`Allow` for `Implicit` |
| `permissions/06-multi-scope-consent.md` | §"Approval Timeout" line 322 | `approval_timeout: project_duration` (default) — alternatives: a fixed `Duration` like `"24h"` | silent-in-code (no `approval_timeout` field on `Organization`) | honored — `Organization.approval_timeout: ApprovalTimeout` ships (typed enum `ProjectDuration | Fixed(Duration)`; default `ProjectDuration`); migration 0013 adds the column |
| `permissions/06-multi-scope-consent.md` | line 349 "Default response on timeout: `deny`, on the principle that absence of consent is not consent. Orgs that want the opposite … can set `approval_timeout_default_response: allow`" | silent-in-code (no `approval_timeout_default_response` field; engine doesn't read consent state, only consent presence) | honored — `Organization.approval_timeout_default_response: TimeoutResponse`; engine maps `TimedOut → Allow|Denied` per org config |
| `permissions/06-multi-scope-consent.md` | §"Per-policy mapping" lines 407–414 | implicit short-circuits → auto-Acknowledged; one_time requests on first template fire; per_session requests every new session | partially-honored (Implicit shipped at CH-09 default; OneTime/PerSession minters absent) | honored — `mint_per_session_consent_request(...)` + `mint_one_time_consent_request(...)` ship as launch-handler helpers; auto-Acknowledged for Implicit unchanged |
| `permissions/06-multi-scope-consent.md` | §"Consent Node" lines 354–356 (scope.org) | `scope` carries `org`, `templates`, `actions` | honored (CH-09 shipped) | honored (extended — `scope.session_id` adds Per-Session axis) |
| `permissions/06-multi-scope-consent.md` | §"Consent Lifecycle" lines 369–414 | Requested → {Acknowledged, Declined, TimedOut, Expired}; Acknowledged → {Revoked, Expired}; forward-only | honored (CH-10 shipped) | honored (unchanged — engine consumes; CH-10 owns the writes) |
| `permissions/06-multi-scope-consent.md` | §"Forward-only revocation" lines 416+ | revocation applies forward only; reads under since-revoked consent stand | honored (CH-10's `legal_transition` enforces) | honored (engine treats `Revoked → Denied` for new reads but doesn't reach back) |
| `permissions/06-multi-scope-consent.md` | line 416 "Request/response channel travels through Channel" | concept-aspirational (M6+) | concept-aspirational (unchanged) | concept-aspirational (unchanged — explicit M6+ defer in §3.C) |
| [`permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) | §"Provenance" + §"Resolution" | engine input contract = `CheckContext` projection of graph nodes | honored (CH-09/10 preserved the contract) | honored (CH-11 widens `CheckContext` with `current_session: Option<SessionId>` + `timeout_default_response: TimeoutResponse` — strictly additive) |
| [`concepts/phi-core-mapping.md`](baby-phi/docs/specs/v0/concepts/phi-core-mapping.md) | §"phi-core has no Consent / Permission concept" | phi-core does not model consent / governance | honored | honored (unchanged — CH-11 stays in baby-phi domain) |

**Coverage check.** Every concept doc whose claims this chunk's code touches is listed. `permissions/README.md` cited per the per-chunk-template's "Permissions subtree hook". `concepts/phi-core-mapping.md` cited per the "phi-core-mapping hook". No concept-doc claim is left as "we'll find out at implementation time".

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Rationale.** CH-11 lands wholly inside the governance layer (`domain::permissions::engine` Step 6, `domain::model::nodes::Grant` extension, `domain::consents::*`, `Organization` config). phi-core has no consent concept (`docs/specs/v0/concepts/phi-core-mapping.md` confirms). Zero `use phi_core::` imports added or removed.

**Expected import-count delta at chunk close: 0.** No new phi-core imports across `domain/`, `store/`, `server/`, `cli/`. No phi-core imports removed.

**Positive close-audit greps:**
```bash
grep -rn "use phi_core::" modules/crates/domain/src/permissions/engine.rs              # 0 (unchanged)
grep -rn "use phi_core::" modules/crates/domain/src/consents/                          # 0 (unchanged)
grep -rn "use phi_core::" modules/crates/domain/src/model/nodes.rs | wc -l             # baseline (unchanged)
grep -rn "phi_core::" modules/crates/domain/ modules/crates/server/ modules/crates/cli/ modules/crates/store/ | wc -l   # baseline (unchanged)
```

**Forbidden-duplication greps (must return 0):**
```bash
grep -rn "^struct ConsentPolicy\|^struct ApprovalMode\|^struct ConsentScope\|^struct ApprovalTimeout" modules/crates/ | grep -v "domain::model"   # 0
grep -rn "use phi_core::context::permissions\|use phi_core::permissions" modules/crates/                                                          # 0 (phi-core has no such module — sanity-check guard)
```

`scripts/check-phi-core-reuse.sh` MUST stay green at chunk close. Per `baby-phi/CLAUDE.md` §"phi-core Leverage" rules 1–5: every overlap is checked, every reuse decision is documented; in this chunk the answer is "no overlap, intentionally phi-only" — same as CH-09 and CH-10.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | None. Engine remains pure-fn (no `DashMap` / `RwLock` / etc.). The `mint_per_session_consent_request` helper is a Repository method invocation — state lives in SurrealDB. | No | — |
| **A2** | New IPC channel | None. Engine pure-fn; helper is an `async` repository call. | No | — |
| **A3** | New pod-local resource | None. | No | — |
| **A4** | Migration runner / first-apply race | Migration 0013 adds **four** columns: `Grant.approval_mode`, `ConsentScope.session_id` (FLEXIBLE-shielded — no schema column needed; `scope` is already `FLEXIBLE TYPE object`), **`Organization.approval_timeout` (FLEXIBLE TYPE object — typed enum tag-payload shape)**, and `Organization.approval_timeout_default_response` (TYPE string DEFAULT "deny"). Two organization columns instead of one (per F3.A). Single-column adds; no first-apply race beyond what migrations 0010/0012 already faced. CHK8S-D-05 (leader-election lock) is already filed and unchanged by this chunk. | No (CHK8S-D-05 unchanged) | — |
| **A5** | Trait-shape requirement | `ConsentIndex` projection grows: `is_acknowledged_for_session(subordinate, org, session_id)`. Concrete struct today (HashSet-backed); not trait-shaped. Per CH-K8S-PREP D33 conforming criteria, the projection is purely an in-process derived structure rebuilt from SurrealDB on each Permission Check; no remote-backend dependency. **No trait shaping required.** Future M7b "all-stateful-projections trait-shaped" pass would cover this — but it's not a CH-11 responsibility. | No | — |
| **A6** | Cross-pod state sharing | Consents persist in SurrealDB; queries span pods identically. The new `ConsentIndex` lookups read from the same backing rows. | No | — |
| **A7** | Audit hash-chain symmetry | One new audit-event variant: `consent.requested` (CH-11 mints `Requested` consents at session launch; CH-10 already shipped `consent.acknowledged / declined / revoked / timed_out / expired`). Single audit emitter (the launch handler), invoked atomically with the row-create — same pattern CH-10 used. No K8s blocker. | No | — |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — chunk does not touch the registry. N/A.
- D33.2 (`SurrealStore::open_remote`) — migration 0013 is forward-only `DEFINE FIELD OVERWRITE` (idempotent under remote-backend per ADR-0042 §D42.3 #5). N/A beyond confirmation.
- D33.3 (SIGTERM graceful shutdown) — chunk adds zero `tokio::spawn` tasks. N/A.
- D33.4 (`EventBus.shutdown` + `drain`) — chunk adds zero `EventBus` emitters or listeners. N/A.

**Conclusion.** **K8s-neutral.** No new K8s blockers. No `CHK8S-D-NN` ledger entry needed.

**Mid-flight discovery rule.** If a phase surfaces a K8s blocker not anticipated here, pause via `AskUserQuestion`, file a new `CHK8S-D-10` (next free) entry, then resume.

---

## §3.C — User-facing documentation impact map

| Tier | File | This chunk touches? | Action |
|---|---|---|---|
| **Concept** | [`permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) | Yes — verified-header bump only | Update in-chunk: prepend a `<!-- Last verified: 2026-MM-DD by Claude Code (CH-11 amendment: §"Per-Session Consent" lines 244 + 297–313 + 322 + 347–349 lifted into typed Rust at `domain::permissions::engine::step_6_consent_gating` (real body), `domain::model::nodes::Grant.approval_mode`, `domain::model::nodes::ConsentScope.session_id`, `domain::model::nodes::Organization.approval_timeout`, `domain::model::nodes::Organization.approval_timeout_default_response`. Per-policy minter helpers ship at `domain::consents::minters`. Channel-side notification remains M6+. Doc body unchanged.) -->`. **Doc body UNCHANGED** (per the post-CH-22 codification rule — concept docs are source-of-truth, not diff-targets). |
| **Concept** | [`permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) | No — Step 6 is documented at this anchor, but the body of the description doesn't change (the formal algorithm "Step 6 = consent gating" stays exact). | Verified-header touch only **only if** the planner's grep finds a stale claim during P1; otherwise N/A. |
| **Concept** | [`permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) | No | N/A — no body change. |
| **Architecture** | (search) `docs/specs/v0/implementation/m5/architecture/permission-check-spine.md` if exists, else `m1/architecture/permission-check-spine.md` | Yes (likely; planner confirms during P1) — the architecture doc describes Step 6 in pseudocode | Update in-chunk: refresh the Step 6 pseudocode to reflect the real body. If the file does not exist, no action. |
| **Operations** | `docs/specs/v0/implementation/m5*/operations/permission-check-operations.md` if exists | Probably no new error codes; engine can already return `Decision::Pending` and `Decision::Denied`. Verify during P1. | If new error codes land (none expected per current plan), update; else "no change". |
| **User-guide** | `docs/specs/v0/implementation/m5*/user-guide/sessions-walkthrough.md` if exists | Possibly — the per-session consent flow is operator-visible in that **a session may launch into Pending**. Whether the walkthrough already covers this depends on CH-15 (real session-launch hard-deny) status. | Defer to **CH-15** (session-launch hard-deny chunk) per forward-scope row 147–151 — the walkthrough rewrite belongs to that chunk's `§3.C` map, not this one. CH-11 records the defer in §10. |
| **Decision** | `docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md` (NEW) | Yes — the ADR for this chunk | Create + Accepted at chunk seal (§5). |
| **Drift** | `docs/specs/v0/implementation/m5_1/drifts/D-new-17.md` | Yes — close | Lifecycle entry `discovered → in-chunk-plan → remediated`. |
| **Drift index** | `docs/specs/v0/implementation/m5_1/drifts/README.md` | Yes | Flip D-new-17 row "Closes at" → CH-11 ✓; status → remediated. |
| **Concept-audit matrix** | `docs/specs/v0/implementation/m5_1/_concept-audit-matrix.md` | Yes | Flip every Per-Session row from `partially-honored` / `contradicted` / `silent-in-code` to `honored`. |

**Defer decisions.** One defer:
1. The `m5*/user-guide/sessions-walkthrough.md` walkthrough rewrite → **CH-15** (real session-launch hard-deny). Reason: CH-11 changes the engine's Step 6 internals, but operator-visible session-launch behaviour stays "may return Pending" — which CH-15 will already need to cover when it makes session-launch hard-deny. Bundling the walkthrough rewrite with CH-15 avoids two consecutive operator-doc churns.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D-new-17** | [`m5_1/drifts/D-new-17.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-17.md) | HIGH | `discovered → in-chunk-plan → remediated` | Closes the consent triad CH-09 + CH-10 + CH-11. |

**Index updates:**
- `drifts/README.md` — D-new-17 row: Status `discovered → remediated`; "Closes at" → `CH-11 ✓`.
- `_concept-audit-matrix.md` — every Per-Session-policy row updated (rows that mention Step 6 / consent gating / per-session matching).

**No new drifts mid-chunk** anticipated. If audit phase surfaces one, the lifecycle rules in `drift-lifecycle.md` apply: new drift file created BEFORE chunk seal.

---

## §5 — ADRs drafted

ADR numbering: highest accepted = ADR-0047 (CH-10). Next-free = **ADR-0048**.

| ADR | Title | Drafted at phase | Decision summary | Flip to Accepted at |
|---|---|---|---|---|
| **ADR-0048** | Per-Session consent gating (Step 6 real body + ApprovalMode + per-policy minters + timeout-default-response + approval-timeout) | P1 (Proposed) | See sub-decisions D48.1–D48.11 below. | P4 chunk seal |

**Sub-decisions (drafted as `Proposed` at P1; flipped to `Accepted` at P4 chunk seal):**

- **D48.1** — `Grant.approval_mode: ApprovalMode` field added (per Fork F1, locked F1.A). Typed enum `ApprovalMode { Implicit, SubordinateRequired { policy: ConsentPolicy }, HumanApprovalRequired }` lives at `domain::model::nodes::ApprovalMode`. `#[serde(default)]` defaulting to `Implicit`. Migration 0013 adds the column on `grant`. Pre-CH-11 grants decode as `Implicit` (engine short-circuits).
- **D48.2** — `ConsentScope.session_id: Option<SessionId>` field added (per Fork F4, locked F4.A). `None` for non-session-scoped consents (Implicit / OneTime). `Some(id)` for Per-Session consents. Migration 0013 extends the FLEXIBLE `scope` object — no schema change required because `scope` is already `FLEXIBLE TYPE object` (per ADR-0045 §D45.5).
- **D48.3** — `CheckContext.current_session: Option<SessionId>` field added (per Fork F5, locked F5.A). Mirrors `current_org` / `current_project` ambient-context pattern. `None` for class-level invocations.
- **D48.4** — `Organization.approval_timeout: ApprovalTimeout` field added (typed enum `ApprovalTimeout { ProjectDuration, Fixed(Duration) }`; default `ProjectDuration` per concept doc 06 line 322) AND `Organization.approval_timeout_default_response: TimeoutResponse` field added (typed enum `Deny | Allow`; default `Deny` per concept doc 06 line 349). Migration 0013 adds **both** columns. Per fork F3.A — full concept-doc fidelity (the alternative deferral path was considered but rejected).
- **D48.5** — Engine Step 6 real body. Branches on `winner.grant.approval_mode`:
  - `Implicit` → no consent gate; continue to Allow.
  - `SubordinateRequired { policy: Implicit }` → no consent gate (the policy at the issuing org's was Implicit at issue time; consent is auto-Acknowledged).
  - `SubordinateRequired { policy: OneTime }` → look up `(subordinate, org, None)` in `ConsentIndex`. Result mapped per §"Engine response table" below.
  - `SubordinateRequired { policy: PerSession }` → look up `(subordinate, org, Some(current_session))` in `ConsentIndex`. Same response table.
  - `HumanApprovalRequired` → reserved; engine returns `Decision::Pending { awaiting_consent: ... }` with a marker until a future chunk wires the human-approval path. (Out-of-scope at CH-11; placeholder only.)
- **D48.6** — Engine response table for `SubordinateRequired` lookups:
  | Lookup result | Decision returned |
  |---|---|
  | `Acknowledged` | proceed to Allow |
  | not present (no row) | mint `Requested` row at the launch handler + return `Pending(AwaitingConsent)` |
  | `Requested` (already minted) | return `Pending(AwaitingConsent)` |
  | `TimedOut` + org `default_response = Deny` | return `Denied(failed_step: Consent, reason: ConsentTimedOutDeny)` |
  | `TimedOut` + org `default_response = Allow` | proceed to Allow |
  | `Declined` | return `Denied(failed_step: Consent, reason: ConsentDeclined)` |
  | `Revoked` | return `Denied(failed_step: Consent, reason: ConsentRevoked)` |
  | `Expired` | return `Denied(failed_step: Consent, reason: ConsentExpired)` |
- **D48.7** — `ConsentIndex` extends with `is_acknowledged_for_session(subordinate, org, session_id) -> bool` and `lookup(subordinate, org, session_id) -> Option<ConsentState>`. The existing `is_acknowledged(subordinate, org)` method is preserved for back-compat (callers that don't carry a `session_id` — used by the OneTime-policy lookup).
- **D48.8** — Per-policy consent-request minters at `domain::consents::minters` (NEW file): `request_one_time(subordinate, org, deadline_at) -> Consent`, `request_per_session(subordinate, org, session_id, deadline_at) -> Consent`. Each returns a freshly-constructed `Consent` with `state = Requested`, the right `scope.session_id` value, `requested_at = now`, `responded_at = None`, `provenance = "engine:step_6@<event>"`. The launch handler invokes the right minter when Step 6 surfaces `Pending` for the first time.
- **D48.9** — New `Repository::request_consent(consent: &Consent) -> RepositoryResult<AuditEventId>` method. Compound tx: CREATE consent row + emit `consent.requested` audit event atomically. The fixed M5 single-emitter pattern (CH-10's audit-event-builder shape).
- **D48.10** — New audit-event builder `consent_requested` at `domain::audit::events::m5::consents` (file already created at CH-10; CH-11 adds one builder). `AuditClass::Logged`. Diff carries `(consent_id, subordinate, org, Option<session_id>, deadline_at)`.
- **D48.11** — Typed `ApprovalTimeout` enum placement. Lives at `domain::model::composites_m3::ApprovalTimeout` mirroring `ConsentPolicy` placement at `composites_m3.rs:41` (verified during planning — composites_m3 is the canonical home for org-level configuration enums consumed by the wizard payload + migration ASSERT clauses). `#[serde(tag = "kind", rename_all = "snake_case")]` so the wire format is `{"kind": "project_duration"}` or `{"kind": "fixed", "duration": "PT24H"}`. The `Duration` type is `chrono::Duration` serialized via `chrono::Duration` ISO-8601 helper (or `humantime_serde` if a string-form is preferred — implementer chooses with rationale). Default = `ProjectDuration` (matches concept doc 06 line 322).

ADR file path: [`m5_2/decisions/0048-per-session-consent-gating.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md) (NEW; drafted at P1, Accepted at P4).

**Cross-references in the ADR:**
- ADR-0045 (CH-09 — `Consent` shape).
- ADR-0047 (CH-10 — state machine + sweeper; deadline-at; transition repo methods).
- ADR-0028 (event-bus fail-safe; durable write before reactive emit).
- ADR-0042 §D42.3 #5 (forward-only idempotent migration runner).
- ADR-0033 §D33.x (K8s readiness conforming-criteria — confirmed neutral).
- Concept doc `permissions/06-multi-scope-consent.md` lines 244, 297–349, 322, 407–414.
- Drift D-new-17 (closed).

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| Post-CH-10 baseline | `cargo test --workspace -- --test-threads=1` ≈ **1265** passed / 0 failed | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` |
| Post-CH-10 baseline | 4 CI guards green | `bash scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` |
| CH-09 / ADR-0045 | `Consent` carries 11 concept-doc fields + `state` + `deadline_at`; `ConsentScope { org, templates, actions }`. CH-11 ADDS `session_id` to scope but does NOT restructure existing fields. | `cargo test -j 4 -p domain --lib model::nodes::tests` |
| CH-09 / ADR-0045 | Migration 0010 still applies cleanly | `cargo test -j 4 -p store --test migrations_test -- --test-threads=1` (assert version 12 baseline pre-CH-11; version 13 after) |
| CH-10 / ADR-0047 | `domain::consents::{state, transitions}` legal-transition table unchanged | `cargo test -j 4 -p domain --lib consents` |
| CH-10 / ADR-0047 | Sweeper task spawns at server startup; `Repository::sweep_consent_timeouts` ships on both backends | `cargo test -j 4 -p domain --test in_memory_ch10_consent_state -- --test-threads=1` |
| CH-10 / ADR-0047 | `Consent.deadline_at: Option<DateTime<Utc>>` field present | `grep -n "deadline_at:" modules/crates/domain/src/model/nodes.rs` (returns 1 hit at line 788) |
| CH-04 / ADR-0043 | Typed `Action` vocabulary stays canonical; CH-11 doesn't add new actions | `cargo test -j 4 -p domain --lib permissions::action` |
| CH-21 / ADR-0040+0041 | Audit hash chain byte-stable on existing event types | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |
| CH-22 / agent-catalog listener body | Listener wiring stays green | `cargo test -j 4 -p server --test acceptance_agents_list -- --test-threads=1` |
| CH-23 / ADR-0046 | Template C/D HTTP edges + listeners stay green | `cargo test -j 4 -p server --test acceptance_system_flows_s05 -- --test-threads=1` (if file exists; else `acceptance_system_flows_s03`) |
| CH-K8S-PREP / ADR-0033 | `SessionRegistry` trait + `SurrealStore::open_remote` + SIGTERM drain + EventBus shutdown unchanged | `cargo test -j 4 -p server --lib state` (smoke); CHK8S guards via `check-phi-core-reuse.sh` (no K8s-specific guard but the ledger entries are doc-only) |
| Step-6 existing behavior | Engine returns `Pending` for template-gated grants lacking consent (the M1 stub behavior) — CH-11 widens this without breaking the basic Pending case | `cargo test -j 4 -p domain --lib permissions::engine::tests::engine_returns_pending_when_template_gated_grant_lacks_consent` (existing test must still pass — it exercises the `Implicit`-policy default path through the new code) |

**Run discipline.** This table runs AT CHUNK OPEN before P1 starts and again at chunk seal. Any regression → new drift file + AskUserQuestion before continuing.

---

## §7 — Phases within the chunk

**Phase count: 4** → audit envelope = **2 agents** (medium chunk per audit-envelope-size skill: 4–6 phases = 2 agents).

### P1 — `ApprovalMode` enum + `Grant.approval_mode` + `ConsentScope.session_id` + Org config (`approval_timeout` + `approval_timeout_default_response`) + migration 0013 (~0.7d)

**Goal.** Land the typed extensions to `Grant`, `ConsentScope`, `Organization` (two new fields) + the migration that adds the columns. No engine changes yet.

**Deliverables.**

1. **`domain::model::nodes::ApprovalMode` enum** (NEW, in `nodes.rs` near `Grant`):
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
   `#[serde(tag = "kind")]` so the wire format is `{"kind": "subordinate_required", "policy": "per_session"}` — round-trips through SurrealDB's `object` storage.

2. **`Grant.approval_mode: ApprovalMode`** field added with `#[serde(default)]`. Pre-CH-11 grants decode as `ApprovalMode::Implicit`. Test fixtures across the workspace that construct `Grant` literally (~6 sites by grep) get a one-line `approval_mode: ApprovalMode::Implicit,` addition.

3. **`ConsentScope.session_id: Option<SessionId>`** field added with `#[serde(default)]`. Test fixtures across the workspace that construct `ConsentScope` literally get a one-line `session_id: None,` addition.

4. **`Organization.approval_timeout: ApprovalTimeout`** field added with `#[serde(default)]` defaulting to `ApprovalTimeout::ProjectDuration` (matches concept doc 06 line 322).

5. **`ApprovalTimeout` enum** (NEW) at `domain::model::composites_m3::ApprovalTimeout` (mirror `ConsentPolicy` placement per D48.11):
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
   #[serde(tag = "kind", rename_all = "snake_case")]
   pub enum ApprovalTimeout {
       #[default]
       ProjectDuration,
       Fixed { duration: chrono::Duration },   // serialize via chrono ISO-8601 helper or humantime_serde — implementer chooses
   }
   ```
   Wire shape: `{"kind": "project_duration"}` or `{"kind": "fixed", "duration": "PT24H"}` (or string-form; sub-decision recorded in ADR §D48.11).

6. **`Organization.approval_timeout_default_response: TimeoutResponse`** field added with `#[serde(default = "Organization::default_timeout_response")]` returning `TimeoutResponse::Deny`. New typed enum `TimeoutResponse { Deny, Allow }` at `domain::model::nodes` (or `composites_m3` — planner picks during implementation; mirror `ConsentPolicy` placement).

7. **Migration `0013_per_session_consent_gating.surql`** — adds columns:
   ```sql
   DEFINE FIELD OVERWRITE approval_mode ON grant FLEXIBLE TYPE object;
   DEFINE FIELD OVERWRITE approval_timeout ON organization FLEXIBLE TYPE object;
   DEFINE FIELD OVERWRITE approval_timeout_default_response ON organization TYPE string DEFAULT "deny";
   -- ConsentScope.session_id: no schema change (scope is already FLEXIBLE TYPE object per migration 0010)
   ```
   Registered at `EMBEDDED_MIGRATIONS` version 13, slug `per_session_consent_gating`. Idempotent under repeated runs (per ADR-0042 §D42.3 #5).

8. **Update CH-09 + CH-10 unit tests** that construct `Grant` / `ConsentScope` / `Organization` literally — add the new fields with their default values. The `#[serde(default)]` shield handles wire-format paths; literal-struct tests need an explicit one-line bump. **Note:** `Organization` literal-struct sites cascade more broadly than the other two — every test fixture, every CLI command test, every server handler test that builds an Organization gets `approval_timeout: ApprovalTimeout::ProjectDuration,` + `approval_timeout_default_response: TimeoutResponse::Deny,`. Planner counts ~10–15 such sites by grep at P1 open.

9. **Update `migrations_test.rs`** — assert row count = 13, version 13, slug `per_session_consent_gating`.

**Tests (P1).** ~8 unit tests:
- `approval_mode_serde_roundtrip` — round-trip every variant + `SubordinateRequired { policy }` for each `ConsentPolicy`.
- `approval_mode_default_is_implicit` — `Default::default() == ApprovalMode::Implicit`.
- `consent_scope_session_id_serde_default_is_none` — pre-CH-11 wire payload (no `session_id` key) decodes with `session_id: None`.
- `organization_approval_timeout_default_is_project_duration` — `Default::default::approval_timeout == ApprovalTimeout::ProjectDuration`.
- `approval_timeout_serde_roundtrip` — `ProjectDuration` ↔ `{"kind":"project_duration"}` and `Fixed(Duration::hours(24))` ↔ `{"kind":"fixed", "duration":"PT24H"}` (or chosen string-form).
- `organization_approval_timeout_default_response_default_is_deny`.
- `timeout_response_serde_roundtrip` — `Deny` ↔ `"deny"`; `Allow` ↔ `"allow"`.
- Migration 0013 lands cleanly + is registered at version 13.

**Concept-alignment check.** §2 row "approval_mode field per concept doc 06 line 244" transitions `contradicted → honored`. §2 row "scope.session_id" transitions `silent-in-code → honored`. §2 row "approval_timeout (line 322)" transitions `silent-in-code → honored`. §2 row "approval_timeout_default_response (line 349)" transitions `silent-in-code → honored`.

**phi-core leverage check.** Zero phi-core imports added. Greps stay at baseline.

**User-facing doc updates.** None at P1. The concept doc verified-header bump lands at P4.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- Adding `approval_mode` to `Grant` breaks any unrelated CH-09/10 test that constructs the struct (the `#[serde(default)]` should shield wire paths, but a literal struct construction in a test would break — fix in this same phase).
- The Organization literal-struct cascade is broader than ~15 sites — escalate to user; the impact may suggest a builder helper at this milestone.
- Migration 0013 conflicts with anything in CH-10's migration 0012 (the `consent` table; we're touching `grant` + `organization`, not `consent`, so no conflict expected — but the user wants verified greps).

---

### P2 — Engine Step 6 real body + `CheckContext.current_session` + `ConsentIndex` extensions + lookup tests (~0.6d)

**Goal.** Wire the real read-side gating logic into the Permission Check engine. No write-side changes yet (those are P3).

**Deliverables.**

1. **`CheckContext.current_session: Option<SessionId>`** field added to `domain::permissions::manifest::CheckContext`. All call sites (planner counts ~7 — `engine.rs::tests`, `permission_check_consent_props.rs`, `step_4_constraint_value_match_props.rs`, `tests/common/mod.rs`, `server/src/platform/sessions/preview.rs`, `server/tests/handler_support_test.rs`, plus any other) populate `None` initially; the launch handler in P3 populates it with `Some(session.id)`.

2. **`ConsentIndex` extension** at `domain::permissions::manifest::ConsentIndex`:
   ```rust
   pub fn lookup(&self, subordinate: AgentId, org: OrgId, session_id: Option<SessionId>) -> Option<ConsentState>;
   pub fn is_acknowledged_for_session(&self, subordinate: AgentId, org: OrgId, session_id: SessionId) -> bool;
   ```
   The internal store changes from `HashSet<(AgentId, OrgId)>` to `HashMap<(AgentId, OrgId, Option<SessionId>), ConsentState>`. The existing `is_acknowledged(subordinate, org)` becomes a thin wrapper: `lookup(subordinate, org, None) == Some(Acknowledged)`. New constructor `ConsentIndex::from_state_map(...)`.

3. **`Decision::Denied` reason variants** — extend `DeniedReason` with three new variants:
   ```rust
   ConsentDeclined { subordinate: AgentId, org: OrgId },
   ConsentRevoked   { subordinate: AgentId, org: OrgId },
   ConsentExpired   { subordinate: AgentId, org: OrgId },
   ConsentTimedOutDeny { subordinate: AgentId, org: OrgId },
   ```
   `FailedStep::Consent` already exists (M1; verified at `permissions::decision::FailedStep`). The 4 new reasons all map to `FailedStep::Consent`.

4. **Engine `step_6_consent_gating` real body** at `engine.rs:443+`. Reads `winner.grant.approval_mode` instead of consulting `template_gated_auth_requests`. For each winning grant in `resolved`:
   - `ApprovalMode::Implicit` → continue (no consent needed).
   - `ApprovalMode::SubordinateRequired { policy }` → branch on `policy`:
     - `Implicit` → continue (auto-Acknowledged at agent creation).
     - `OneTime` → `ctx.consents.lookup(target, org, None)` → response table per D48.6.
     - `PerSession` → `ctx.consents.lookup(target, org, ctx.current_session)` → response table per D48.6. If `current_session` is `None`, return `Denied(failed_step: Consent, reason: NoSessionContext)` — the new `NoSessionContext` reason captures the launch-handler bug case (per-session policy + class-level call). **Pause-discipline trigger:** if the planner finds an existing valid call site that legitimately passes `None` for `current_session` to a per-session-gated grant, escalate to user.
   - `ApprovalMode::HumanApprovalRequired` → return `Decision::Pending` with a marker reason (out-of-scope placeholder; no real handling at CH-11).
   - The `template_gated_auth_requests` field is preserved on `CheckContext` for back-compat with M1 callers; new code consults `approval_mode` instead. **Soft deprecation only**; removal lands at a future chunk.

5. **Engine response-table mapping** per D48.6 + `Org.approval_timeout_default_response` consulted for `TimedOut`. The engine reaches the org's timeout-response setting via... wait — the engine is pure-fn and doesn't have storage. The setting is passed in via `CheckContext.timeout_default_response: TimeoutResponse` (new field, populated by the caller from the org's config). **Defaults to `Deny`** for callers that don't populate it (back-compat). This is one extra field on `CheckContext`; the launch handler in P3 populates it.

**Tests (P2).** ~12 unit + property tests at `engine.rs::tests`:
- 8 response-table cases (1 per row in D48.6's 8-row table) — each spins up a fixture with one grant + one consent in the named state + asserts the decision.
- 1 test: `ApprovalMode::Implicit` short-circuits (no consent lookup at all).
- 1 test: `SubordinateRequired { policy: PerSession }` with `current_session = None` returns `Denied(NoSessionContext)`.
- 1 test: `ApprovalMode::HumanApprovalRequired` returns `Pending` with the marker.
- Property test: across all `(approval_mode, consent_state, default_response)` combinations, the decision is deterministic (same input → same output).

**Concept-alignment check.** §2 row "Step 6 differentiates per-session from one-time" transitions `partially-honored → honored`. §2 row "Default response on timeout (deny / allow)" transitions `silent-in-code → honored`.

**phi-core leverage check.** Zero phi-core imports added. Verify with `grep -rn "use phi_core::" modules/crates/domain/src/permissions/engine.rs` (expect 0).

**User-facing doc updates.** None at P2.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- The `template_gated_auth_requests` back-compat path can't cleanly co-exist with the new `approval_mode` path (the engine has both fields available; logic should clearly prefer `approval_mode` and fall back to the legacy field only when `approval_mode == Implicit`). If it's messy, escalate to user — soft-deprecation may need to harden faster.
- Adding `current_session` to `CheckContext` triggers a compile cascade through ≥ 10 call sites (≥ 10 sites is fine; ≥ 20 sites suggests an underlying coupling problem that needs an AskUserQuestion).
- The `Decision::Pending` shape changes (the existing `Pending { awaiting_consent: AwaitingConsent }` payload may need extension to carry `session_id`) — if so, that's a back-compat-breaking ADR addendum; pause and escalate.

---

### P3 — Per-policy minter helpers + `Repository::request_consent` + launch-handler wiring (deadline computation per `ApprovalTimeout`) + audit event + acceptance tests (~0.7d)

**Goal.** Wire the write-side of consent minting + the launch-handler integration. End-to-end: a session launch under a `PerSession` template grant → engine returns `Pending` → handler computes the `deadline_at` per `Org.approval_timeout` → handler mints `Requested` consent → audit event emitted.

**Deliverables.**

1. **`domain::consents::minters` module** (NEW file at `modules/crates/domain/src/consents/minters.rs`):
   ```rust
   pub fn request_one_time(subordinate: AgentId, org: OrgId, deadline_at: Option<DateTime<Utc>>) -> Consent;
   pub fn request_per_session(subordinate: AgentId, org: OrgId, session_id: SessionId, deadline_at: Option<DateTime<Utc>>) -> Consent;
   ```
   Each returns a freshly-constructed `Consent` (with `state = Requested`, `requested_at = Utc::now()`, etc.). Pure-fn — no I/O. The provenance string follows the format `"engine:step_6@<ISO-8601>"`.

2. **`Repository::request_consent`** trait method:
   ```rust
   async fn request_consent(&self, consent: &Consent) -> RepositoryResult<AuditEventId>;
   ```
   Compound tx: CREATE consent row + emit `consent.requested` audit event atomically. Implementations on both backends (in-memory + SurrealDB) following the CH-10 pattern (`acknowledge_consent` etc.).

3. **`consent_requested` audit-event builder** at `domain::audit::events::m5::consents` (extends the file CH-10 created). `AuditClass::Logged`. Diff carries `(consent_id, subordinate, org, session_id: Option, deadline_at: Option)`.

4. **Launch-handler wiring** at `server::platform::sessions::launch` (the existing handler) — when the engine returns `Pending(AwaitingConsent { subordinate, org })`:
   - Determine which policy applies: read the issuing org's `consent_policy` (for OneTime vs PerSession) — already on `Organization`.
   - **Compute `deadline_at` per `org.approval_timeout`** (concept doc 06 lines 322–347):
     - `ApprovalTimeout::ProjectDuration` →
       - Shape A (1 project) / Shape B (joint project) → `deadline = project.deadline_at` (single-project lookup; if `None`, fall back to `now + 24h` with a recorded log-warning).
       - Shape C (multi-project) → `deadline = max(project.deadline_at)` across all session-tagged projects.
       - Shape D (no project) → `deadline = now + 24h` (concept doc 06 line 343 — "If neither is set, an explicit fixed timeout is required" — v0 hard-codes 24h; recorded as deferred-from-CH-11 entry for "shape D needs operator-explicit timeout" in §3.C row 6).
     - `ApprovalTimeout::Fixed(d)` → `deadline = now + d`. Concept doc 06 line 347 says "the runtime treats whichever is shorter as the effective timeout" — for v0 we honour the Fixed value as-stated when set; the "min(fixed, project_duration)" rule lands as a follow-up if the user wants it (recorded as deferred-from-CH-11 entry; non-blocking for D-new-17 closure since it only reduces the timeout, never extends it).
   - Call the matching minter + persist via `Repository::request_consent`.
   - Re-call the engine after persistence? No — the response stays `Pending` for this call (the consent is `Requested`, not `Acknowledged`). Only future calls observe the row. The launch surface returns `Pending` to the operator with the new `consent_id` in the payload.

5. **Update `server::platform::sessions::preview`** to populate `current_session` and `timeout_default_response` on the `CheckContext`. Preview doesn't mint consents (it's a read-only preview), but the engine's new branches need the field populated.

6. **Repository read methods** for `list_consents_for_subordinate(agent_id) -> Vec<Consent>` and `get_consent(consent_id) -> Option<Consent>` (both backends). These were promised at CH-09 ADR-0045 §D45.7 ("CH-11 will add the read methods needed for per-session gating"). The launch handler's `ConsentIndex::lookup` path needs them: project the `(subordinate, org, session_id) -> ConsentState` map at engine call time.

7. **`ConsentIndex::project_from_repo(repo, subordinate)` helper** (likely lives at `domain::permissions::manifest`) — fetches all consents for a subordinate and builds the `HashMap<(AgentId, OrgId, Option<SessionId>), ConsentState>` projection. Called from the launch handler before constructing `CheckContext`.

**Acceptance tests (P3).** New file at `modules/crates/server/tests/acceptance_per_session_consent_gating.rs` (~8 tests):
- `implicit_policy_grant_allows_immediately` — Org with `consent_policy: implicit`; supervisor reads subordinate's session; Step 6 short-circuits to Allow.
- `one_time_policy_first_read_returns_pending_and_mints_requested_consent` — Org with `consent_policy: one_time`; first read returns Pending; `Repository::list_consents_for_subordinate` (P3 deliverable) finds 1 Requested row.
- `one_time_policy_second_read_after_acknowledge_allows` — Acknowledge the row from the previous test; next read allows.
- `per_session_policy_first_read_on_session_a_returns_pending_and_mints_session_scoped_consent` — Org with `consent_policy: per_session`; read on session A returns Pending; the minted consent has `scope.session_id = Some(A)`.
- `per_session_policy_first_read_on_session_b_after_session_a_acknowledged_still_returns_pending` — same subordinate, same supervisor, but session B; the session A consent doesn't satisfy session B (per-session matching).
- `per_session_policy_timed_out_with_deny_default_returns_denied` — let CH-10's sweeper flip the row to TimedOut; org `default_response = deny`; next read returns Denied.
- `per_session_policy_timed_out_with_allow_default_returns_allow` — same, but org configured `default_response = allow`; next read allows.
- `deadline_uses_org_approval_timeout_fixed_value` — Org with `approval_timeout: ApprovalTimeout::Fixed(Duration::hours(24))`; mint a per-session Requested consent and assert `consent.deadline_at == now + 24h` (within ±1s tolerance).

**Tests (P3).** ~8 unit/integration tests at the new acceptance file + ~4 unit tests for the new minters/repo methods at `domain::consents::minters::tests` and `store::tests::repository_test::surreal_consent`.

**Concept-alignment check.** §2 rows "Per-policy mapping (implicit / one_time / per_session)" + "Step 6 differentiates per-session" + "approval_timeout (line 322)" all flip to fully `honored`. The single remaining concept-aspirational claim ("Channel-side notification") stays out-of-scope (per §3.C defer-decision).

**phi-core leverage check.** Zero phi-core imports added across the new files.

**User-facing doc updates.** None at P3 (per §3.C; the verified-header bump is P4).

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- The launch-handler integration triggers a deeper change to `SessionLaunchOutcome` than just "now also returns the `consent_id` for Pending" (e.g., the operator-facing JSON shape needs to evolve in a way that breaks an existing CLI client) — escalate to user.
- The deadline-computation logic for shape C (multi-project max) requires graph-traversal helpers that don't exist — file a deferred-ledger entry + escalate; v0 may need to fall back to "first project" with a documented gap if the helpers are >0.2d to build.
- The `consent_requested` audit-event diff payload structure conflicts with the CH-10 audit-event shape (the diff carries an `Option<SessionId>` which CH-10 didn't have — sanity-check it round-trips via the BLAKE3 canonical-bytes path).

---

### P4 — ADR Accepted + drift closed + concept-doc bump + 2-agent audit + seal (~0.4d)

**Goal.** Ratify ADR-0048. Close D-new-17. Spawn 2 audit agents per §11. Seal the chunk.

**Deliverables.**

1. **ADR-0048 flipped from `Proposed` → `Accepted`**.
2. **D-new-17 Status flipped to `remediated`**. Lifecycle entry appended (`2026-MM-DD — in-chunk-plan → remediated — CH-11 chunk-seal`).
3. **`drifts/README.md`** row for D-new-17 flipped + "Closes at" → CH-11 ✓.
4. **`_concept-audit-matrix.md`** rows touching Per-Session policy / Step 6 gating flipped to `honored`.
5. **Concept doc `permissions/06-multi-scope-consent.md` verified-header bump** (the new amendment line — see §3.C row 1 for the exact text).
6. **Architecture-doc check** — if `m5*/architecture/permission-check-spine.md` exists with stale Step 6 pseudocode, refresh it. Otherwise N/A.
7. **Spawn 2 audit agents** per §11.
8. **Cycle index update** — add a row to `baby-phi/docs/specs/plan/build/_cycle-index.md` under "Active cycles" → flip to "Closed cycles" at seal.

**Tests (P4).** No new tests; runs the verification recipe in §12.

**Concept-alignment check.** Final pass over §2 — every row's target status is the chunk-close status.

**phi-core leverage check.** Final greps.

**User-facing doc updates.** Concept doc verified-header bump (delivery 5 above) is the §3.C row 1 deliverable.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit agent reports a finding (tactical or architectural FAIL — orchestrator's audit-fix loop kicks in). User escalation required for any architectural FAIL.

---

## §8 — Tests summary

- **Expected total at chunk close**: post-CH-10 (**1265**) + ~8 P1 + ~12 P2 + ~12 P3 = **~1297 tests**.
- **Layer breakdown**:
  - Unit (P1 + P2 + part of P3): ~24 new
  - Integration (P3 part): ~4 new
  - Acceptance (P3 acceptance file): ~8 new
- **Named new test files**:
  - `modules/crates/server/tests/acceptance_per_session_consent_gating.rs` (NEW; ~8 acceptance tests)
- **Named expected-still-green tests** (anything fragile):
  - `engine_returns_pending_when_template_gated_grant_lacks_consent` (existing M1 test) — must still pass as the back-compat path through `template_gated_auth_requests`.
  - `engine_allows_template_gated_grant_when_consent_present` (existing M1 test) — must still pass.
  - All CH-09 tests (`consent_struct_carries_concept_doc_fields`, `consent_legacy_wire_format_decodes_with_defaults`, etc.).
  - All CH-10 tests (`legal_transitions_match_concept_doc_verbatim`, `terminal_states_have_zero_outbound_arrows`, the 36-cell matrix, sweeper tests).
  - `acceptance_consent_node_shape.rs` — both `in_memory_persists_full_eleven_field_consent` + the SurrealDB equivalent.
  - `acceptance_memory_extraction.rs` (CH-21 audit hash chain byte-stable).
  - `acceptance_system_flows_s05.rs` (CH-23 Template C/D edges).

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4` → `d5428c43`.
2. Cycle folder: `mkdir -p baby-phi/docs/specs/plan/build/ch-11-per-session-consent-gating-d5428c43/`.
3. Plan file written to: `baby-phi/docs/specs/plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md` (this file).
4. `bash scripts/check-doc-links.sh` → expect green.

### Reading list (mandatory; every item end-to-end before P1 opens)

1. [`concepts/permissions/06-multi-scope-consent.md`](baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md) — full file. Anchors §"Per-Session Consent" (lines 297–349), §"Consent Policy" (lines 230–256), §"Approval Timeout" (lines 317–347), §"Per-policy mapping" (lines 407–414), line 244 (subordinate_required), line 322 (approval_timeout default), line 349 (default-response).
2. [`drifts/D-new-17.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-17.md) — full.
3. [`m5_2/decisions/0045-consent-node-full-shape.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0045-consent-node-full-shape.md) — full ADR-0045 (CH-09).
4. [`m5_2/decisions/0047-consent-state-machine-and-sweeper.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0047-consent-state-machine-and-sweeper.md) — full ADR-0047 (CH-10).
5. [`plan/build/ch-09-consent-node-full-shape-03061b67.md`](baby-phi/docs/specs/plan/build/ch-09-consent-node-full-shape-03061b67.md) — CH-09 plan (precedent for the field-add shape).
6. [`plan/build/ch-10-consent-state-machine-and-sweeper-fda01605.md`](baby-phi/docs/specs/plan/build/ch-10-consent-state-machine-and-sweeper-fda01605.md) — CH-10 plan (precedent for repo-method + audit-event-builder + migration patterns).
7. [`modules/crates/domain/src/permissions/engine.rs`](baby-phi/modules/crates/domain/src/permissions/engine.rs) — full file (Step 6 stub + the existing tests).
8. [`modules/crates/domain/src/permissions/manifest/mod.rs`](baby-phi/modules/crates/domain/src/permissions/manifest/mod.rs) — full file (`CheckContext`, `ConsentIndex`).
9. [`modules/crates/domain/src/consents/{mod.rs, state.rs, transitions.rs}`](baby-phi/modules/crates/domain/src/consents) — full files (CH-10 module).
10. [`modules/crates/domain/src/model/nodes.rs`](baby-phi/modules/crates/domain/src/model/nodes.rs) lines 360–420 (Organization), 620–640 (Grant), 760–840 (Consent + ConsentScope + ConsentState).
11. [`modules/crates/domain/src/model/composites_m3.rs`](baby-phi/modules/crates/domain/src/model/composites_m3.rs) lines 41–68 (`ConsentPolicy`) — `ApprovalTimeout` enum is added in this file mirroring `ConsentPolicy` placement (per D48.11).
12. [`modules/crates/store/migrations/0010_consent_full_shape.surql`](baby-phi/modules/crates/store/migrations/0010_consent_full_shape.surql) + [`0012_consent_deadline.surql`](baby-phi/modules/crates/store/migrations/0012_consent_deadline.surql) — migration precedents.
13. [`baby-phi/CLAUDE.md`](baby-phi/CLAUDE.md) — phi-core Leverage section.
14. [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §1 lines 114–119 + §7 binding decisions.
15. [`process/per-chunk-planning-template.md`](baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md) — full template (this plan's scaffold).
16. [`m7b/architecture/deferred-from-ch-k8s-prep.md`](baby-phi/docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) — to confirm CHK8S-D-09 + the conventions for new ledger entries.

### Carry-forward invariants (verified green at chunk open)

- `cargo test --workspace -- --test-threads=1` test count = **1265** / 0 failed (post-CH-10 baseline).
- `scripts/check-phi-core-reuse.sh` exits 0.
- `scripts/check-doc-links.sh` exits 0.
- `scripts/check-ops-doc-headers.sh` exits 0.
- `scripts/check-spec-drift.sh` exits 0.
- `git diff --stat HEAD -- modules/` empty (no preload edits).
- D-new-17 Status = `discovered`.
- ADR-0034..0047 Accepted; **next-free = ADR-0048**.
- Migrations registered = 12; **next-free = 13**.

### Pending decisions carried into this chunk

- **Forks F1–F5** (see top of plan) — **all locked by user 2026-05-02** (F1.A, F2.B, F3.A, F4.A, F5.A). No further escalation needed before P1 opens.
- Forward-scope §7 Q&A: Q1 (template uniformity) honored; Q5 (HIGH chunks must close at M5) → CH-11 is HIGH; closing it before M5 tag.
- D-new-17 lifecycle transition `discovered → in-chunk-plan` lands at chunk-open before P1.

**Chunk-ordering note.** The user selected CH-11 at chunk-open; CH-09 + CH-10 are sealed prerequisites confirmed by §6. No forward dependencies on un-sealed chunks.

---

## §10 — Close criteria

**Source of truth: concept docs.** No rounding; below-target blocks close.

### 4 aspects (each graded pass / fail)

- **Code aspect** — all phases' deliverables shipped; `cargo test --workspace -- --test-threads=1` green at ~1297; clippy green under `RUSTFLAGS="-Dwarnings"`; `cargo fmt --all -- --check` green.
- **Docs aspect** —
  - *Governance tier*: D-new-17 `remediated`; concept-audit matrix Per-Session rows flipped `honored`; ADR-0048 `Accepted`; `drifts/README.md` index updated; verified headers bumped on every modified doc; cycle-index updated.
  - *User-facing tier* (post-CH-22): every §3.C row resolved (in-chunk update OR explicit defer with successor ref).
- **phi-core leverage aspect** — import-count delta = 0 (predicted); positive greps match baseline; `check-phi-core-reuse.sh` green; no `use phi_core::` in `modules/crates/domain/src/permissions/engine.rs` or `modules/crates/domain/src/consents/`.
- **Concept alignment aspect** — every §2 row at its target chunk-close status; **no row remains `contradicted`** at close.

### 2 confidence % (each with named numerator/denominator)

- **Implementation confidence %** = `claims-honored / claims-in-scope` — **target ≥ 10/11**:
  1. `Grant.approval_mode: ApprovalMode` field + enum exist and migrate cleanly.
  2. `ConsentScope.session_id: Option<SessionId>` field exists.
  3. `Organization.approval_timeout_default_response: TimeoutResponse` field + enum exist and migrate cleanly.
  4. `CheckContext.current_session: Option<SessionId>` + `CheckContext.timeout_default_response` fields exist.
  5. Engine Step 6 real body branches on `approval_mode` per D48.5 + response table per D48.6.
  6. `ConsentIndex::lookup(...)` + `is_acknowledged_for_session(...)` ship; existing `is_acknowledged` preserved.
  7. Per-policy minters at `domain::consents::minters` (`request_one_time` + `request_per_session`) ship.
  8. `Repository::request_consent` ships on both backends + `consent.requested` audit event.
  9. `Repository::list_consents_for_subordinate` + `get_consent` ship on both backends.
  10. Acceptance tests cover Implicit / OneTime / PerSession paths + the timeout-default-response axis.
  11. `Organization.approval_timeout: ApprovalTimeout` field + `ApprovalTimeout` enum exist; migration 0013 adds the column; default `ProjectDuration`; launch handler consumes the field at deadline-computation time per concept doc 06 lines 322–347.

- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)` — **target = 100% (e.g., 6/6 for CH-11's doc surface).**

### Composite

`min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary)`. Composite below target = close blocked.

**Close-target discipline.** Close report states ALL FIVE measures with named numerators/denominators. No aspect-averaging. No rounding up.

---

## §11 — Post-chunk independent audit plan

**Agent count: 2** (medium chunk per per-chunk-template — 4–6 phases = 2 agents). Per audit-envelope-size skill: 4 phases puts CH-11 in the medium bucket; 2 agents with split aspects (a) + (d) and (b) + (c) is the standard envelope.

**Auditors are fresh sub-agents** (`Explore` or `general-purpose`), never the implementer.

### Audit A — Code correctness + phi-core leverage (≤ 600 words)

> You are auditing CH-11 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan at `docs/specs/plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md`. ADR at `docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md`.
>
> PASS/FAIL each numbered claim. Cite file:line for every claim. ≤ 600 words.
>
> 1. `domain::model::nodes::ApprovalMode` enum exists with three variants (`Implicit`, `SubordinateRequired { policy: ConsentPolicy }`, `HumanApprovalRequired`) + `Default = Implicit` + `#[serde(tag = "kind", rename_all = "snake_case")]`.
> 2. `Grant.approval_mode: ApprovalMode` field exists with `#[serde(default)]`.
> 3. `ConsentScope.session_id: Option<SessionId>` field exists with `#[serde(default)]`.
> 4. `Organization.approval_timeout_default_response: TimeoutResponse` field exists with `#[serde(default)]`. The `TimeoutResponse` enum has variants `Deny`, `Allow` with snake-case serde + `Default = Deny`.
> 4a. `Organization.approval_timeout: ApprovalTimeout` field exists with `#[serde(default)]`. The `ApprovalTimeout` enum has `ProjectDuration` + `Fixed { duration: chrono::Duration }` (or equivalent payload form) variants with snake-case + tag-payload serde (default `ProjectDuration`). Migration 0013 adds the `approval_timeout` column on `organization`.
> 5. Migration `0013_per_session_consent_gating.surql` exists, registered at version 13 / slug `per_session_consent_gating`. `migrations_test.rs` asserts row count = 13.
> 6. `CheckContext.current_session: Option<SessionId>` + `CheckContext.timeout_default_response: TimeoutResponse` fields exist.
> 7. `ConsentIndex::lookup(subordinate, org, session_id) -> Option<ConsentState>` + `ConsentIndex::is_acknowledged_for_session(...)` methods exist; existing `is_acknowledged(subordinate, org)` preserved + working.
> 8. Engine `step_6_consent_gating` body in `engine.rs` (verify the function's body changed substantively — check git diff or compare against pre-CH-11) implements the 8-row response table per D48.6: `Acknowledged → Allow`; missing → `Pending` + (mint at handler); `Requested → Pending`; `TimedOut + deny → Denied(ConsentTimedOutDeny)`; `TimedOut + allow → Allow`; `Declined → Denied(ConsentDeclined)`; `Revoked → Denied(ConsentRevoked)`; `Expired → Denied(ConsentExpired)`.
> 9. `domain::consents::minters` module exists with `request_one_time` and `request_per_session` pure-fn helpers. Both return `Consent` with `state = Requested`, the right `scope.session_id` value, and `provenance = "engine:step_6@..."`.
> 10. `Repository::request_consent(consent) -> RepositoryResult<AuditEventId>` exists on the trait + both backends (in-memory + SurrealDB). Compound tx: CREATE consent + emit `consent.requested` audit atomically.
> 11. `Repository::list_consents_for_subordinate(agent_id) -> Vec<Consent>` + `Repository::get_consent(consent_id) -> Option<Consent>` exist on both backends.
> 12. New audit-event builder `consent_requested` at `audit/events/m5/consents.rs`. `AuditClass::Logged`. Diff carries `(consent_id, subordinate, org, session_id: Option, deadline_at: Option)`.
> 13. Launch handler at `server::platform::sessions::launch` mints the `Requested` consent on first Pending response (via the right minter + `request_consent`) AND populates `current_session` + `timeout_default_response` on `CheckContext` AND computes `deadline_at` from `org.approval_timeout` per D48.5 + concept doc 06 lines 322–347.
> 14. `cargo test --workspace -j 4 -- --test-threads=1` green at ~1297 / 0 failed.
> 15. Clippy clean under `RUSTFLAGS="-Dwarnings"`.
> 16. CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
> 17. **phi-core leverage**: `grep -rn "use phi_core::" modules/crates/domain/src/permissions/engine.rs` returns 0; `grep -rn "use phi_core::" modules/crates/domain/src/consents/` returns 0; total `phi_core::` imports across `modules/crates/` is unchanged from post-CH-10 baseline.
> 18. CH-09 + CH-10 invariants intact: `cargo test -p domain --lib model::nodes::tests` green; `cargo test -p domain --lib consents` green.

### Audit B — Concept fidelity + docs fidelity (≤ 600 words)

> You are auditing CH-11's concept-fidelity + docs-fidelity. Read-only.
>
> PASS/FAIL each numbered claim. Cite file:line for every claim. ≤ 600 words.
>
> 1. ADR-0048 file exists at `m5_2/decisions/0048-per-session-consent-gating.md`. Status field reads exactly `**Status: Accepted**` (one line, bold).
> 2. ADR-0048 documents sub-decisions D48.1–D48.11 (the body of every decision present + matches the plan's §5 verbatim).
> 3. ADR-0048 documents the 5 locked forks F1–F5 (each fork mentioned with the user's locked decision F1.A / F2.B / F3.A / F4.A / F5.A).
> 4. ADR-0048 cross-references concept doc 06 (lines 244, 297–349, 322, 407–414, 349), drift D-new-17 (closed), ADR-0028 (event-bus), ADR-0042 §D42.3 #5 (migration runner), ADR-0033 §D33 (K8s readiness), ADR-0045 + ADR-0047 (CH-09 + CH-10 precedents).
> 5. Drift `D-new-17.md` Status = `remediated`; lifecycle entry for CH-11 chunk-seal present (`2026-MM-DD — in-chunk-plan → remediated — CH-11 chunk-seal`).
> 6. `drifts/README.md` row for D-new-17 flipped; "Closes at" → CH-11 ✓; status reads `remediated`.
> 7. `_concept-audit-matrix.md` Per-Session-policy rows flipped from `contradicted` / `silent-in-code` / `partially-honored` to `honored`. Includes the line-322 (approval_timeout) row + the line-349 (default_response) row. List the row indices.
> 8. Concept doc `permissions/06-multi-scope-consent.md` verified-header bumped (CH-11 amendment line added; CH-09 + CH-10 amendment lines preserved). The CH-11 amendment line mentions BOTH `approval_timeout` AND `approval_timeout_default_response` fields. Doc body UNCHANGED (cite the line count + a checksum / git-diff that confirms body bytes are identical).
> 9. Architecture-doc check — if `m5*/architecture/permission-check-spine.md` exists, verify the Step 6 pseudocode matches the engine's real body. If the file does not exist, note "N/A".
> 10. Plan archive at `plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md` exists (folder-style, multi-agent system).
> 11. CH-09 + CH-10 invariants intact: ADR-0045 + ADR-0047 still Accepted; D-new-04 + D-new-05 still remediated; concept doc 06 retains the CH-09 + CH-10 amendment lines; `domain::consents::{state, transitions}` legal-transition table unchanged (line-count match).
> 12. Forward-scope row for CH-11 still reads as before — the row is consumed by closing D-new-17, not edited.
> 13. **Cycle index** at `docs/specs/plan/build/_cycle-index.md` has a row for CH-11-d5428c43 in the appropriate section (Active → flipped to Closed at seal).

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test (cargo workers capped at -j 4 per feedback_cargo_jobs_cap)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1297 passed / 0 failed

# 3. Positive greps — chunk-specific
grep -n "pub enum ApprovalMode" modules/crates/domain/src/model/nodes.rs                      # 1
grep -n "pub approval_mode: ApprovalMode" modules/crates/domain/src/model/nodes.rs            # 1 (on Grant)
grep -n "pub session_id: Option<SessionId>" modules/crates/domain/src/model/nodes.rs          # 1 (on ConsentScope)
grep -n "pub enum ApprovalTimeout" modules/crates/domain/src/model/composites_m3.rs           # 1 (or nodes.rs if the implementer chooses that placement)
grep -n "pub approval_timeout: ApprovalTimeout" modules/crates/domain/src/model/nodes.rs      # 1 (on Organization)
grep -n "pub enum TimeoutResponse" modules/crates/domain/src/model/nodes.rs                   # 1 (or composites_m3)
grep -n "pub approval_timeout_default_response: TimeoutResponse" modules/crates/domain/src/model/nodes.rs  # 1 (on Organization)
grep -n "pub current_session: Option<SessionId>" modules/crates/domain/src/permissions/manifest/mod.rs  # 1
grep -n "pub fn lookup\|pub fn is_acknowledged_for_session" modules/crates/domain/src/permissions/manifest/mod.rs  # ≥ 2
ls modules/crates/domain/src/consents/minters.rs                                              # exists
grep -n "pub fn request_one_time\|pub fn request_per_session" modules/crates/domain/src/consents/minters.rs  # ≥ 2
grep -n "fn request_consent\b\|fn list_consents_for_subordinate\b\|fn get_consent\b" modules/crates/domain/src/repository.rs  # ≥ 3
grep -n "pub fn consent_requested" modules/crates/domain/src/audit/events/m5/consents.rs      # 1
ls modules/crates/store/migrations/0013_per_session_consent_gating.surql                       # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md  # 1

# 4. Forbidden / regression greps
grep -rn 'use phi_core::' modules/crates/domain/src/permissions/engine.rs                     # 0
grep -rn 'use phi_core::' modules/crates/domain/src/consents/                                 # 0
grep -rn 'use phi_core::' modules/crates/domain/src/audit/events/m5/consents.rs               # 0

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-17.md  # 1

# 6. Targeted suites
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::engine                  # P2 unit tests + existing M1 tests still green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib consents::minters                    # P3 minter tests
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test repository_test surreal_consent -- --test-threads=1  # CH-11 surreal cross-impl
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migrations_test -- --test-threads=1  # version 13
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_per_session_consent_gating -- --test-threads=1  # NEW acceptance suite

# 7. Carry-forward sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib model::nodes::tests                  # CH-09 invariants
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib consents                             # CH-10 invariants (state machine)
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test in_memory_ch10_consent_state -- --test-threads=1  # CH-10 sweeper invariants
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_consent_node_shape -- --test-threads=1  # CH-09 acceptance
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1   # CH-21 audit hash chain
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_authority_templates -- --test-threads=1  # template wiring

# 8. Drift / matrix counts
grep -l "Status.*remediated" docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l           # baseline + 1 (D-new-17)
```

---

## What this plan does NOT do

- HTTP / CLI endpoints for subordinate `acknowledge` / `decline` / `revoke` actions (M6+).
- Channel notification delivery (Slack / email / web UI) — concept doc 06 line 416 explicitly M6+.
- `subordinate_inbox` queue table (rejected option F2.C).
- `human_approval_required` engine path (placeholder variant only — no real handling).
- Multi-pod sweeper leader-election (CHK8S-D-09; M7b — unchanged by this chunk).
- Retroactive cleanup of artifacts touched during a since-revoked grant (concept doc 06 explicitly rules this out).
- The "min(fixed, project_duration)" effective-timeout rule (concept doc 06 line 347) — recorded as a P3 deferred-from-CH-11 entry; non-blocking for D-new-17 closure.

---

## Critical files

**New:**
- `modules/crates/domain/src/consents/minters.rs` — per-policy mint helpers.
- `modules/crates/store/migrations/0013_per_session_consent_gating.surql` — column adds (grant.approval_mode, organization.approval_timeout, organization.approval_timeout_default_response).
- `modules/crates/server/tests/acceptance_per_session_consent_gating.rs` — acceptance suite (8 tests).
- `docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md` — ADR.

**Modified:**
- `modules/crates/domain/src/model/nodes.rs` — `ApprovalMode` enum, `Grant.approval_mode`, `ConsentScope.session_id`, `Organization.approval_timeout` + `Organization.approval_timeout_default_response`, `TimeoutResponse` enum (or moved to composites_m3 — implementer chooses).
- `modules/crates/domain/src/model/composites_m3.rs` — `ApprovalTimeout` enum (mirrors `ConsentPolicy` placement per D48.11).
- `modules/crates/domain/src/permissions/engine.rs` — `step_6_consent_gating` real body.
- `modules/crates/domain/src/permissions/manifest/mod.rs` — `CheckContext.current_session` + `CheckContext.timeout_default_response`; `ConsentIndex::{lookup, is_acknowledged_for_session, project_from_repo}`.
- `modules/crates/domain/src/permissions/decision.rs` — `DeniedReason` extension (4 new variants).
- `modules/crates/domain/src/repository.rs` — `request_consent` + `list_consents_for_subordinate` + `get_consent` methods.
- `modules/crates/domain/src/in_memory.rs` — 3 impls.
- `modules/crates/store/src/repo_impl.rs` — 3 impls.
- `modules/crates/store/src/migrations.rs` — register version 13.
- `modules/crates/store/tests/migrations_test.rs` — assert version 13.
- `modules/crates/store/tests/repository_test.rs` — cross-impl tests for new methods.
- `modules/crates/domain/src/audit/events/m5/consents.rs` — `consent_requested` builder.
- `modules/crates/domain/src/consents/mod.rs` — declare `pub mod minters`.
- `modules/crates/server/src/platform/sessions/launch.rs` — populate `current_session` + `timeout_default_response`; compute `deadline_at` per `org.approval_timeout`; mint Requested on Pending.
- `modules/crates/server/src/platform/sessions/preview.rs` — populate `current_session` + `timeout_default_response`.
- Any test fixture that constructs `Grant`, `ConsentScope`, `Organization`, or `CheckContext` literally — add new fields with defaults. The Organization cascade is the broadest (~10–15 sites for the two new fields).
- Drift files: `D-new-17.md`, `drifts/README.md`, `_concept-audit-matrix.md`.
- Concept doc: `permissions/06-multi-scope-consent.md` — verified-header bump only.
- `_cycle-index.md` — add CH-11-d5428c43 row.

**Unchanged (verified by close-audit):**
- `modules/crates/domain/src/consents/{state.rs, transitions.rs}` — CH-10's legal-transition table + transition pure-fns.
- The CH-10 sweeper task + spawn site at `server::state`.
- `domain::consents::ConsentTransitionError` shape.
- phi-core (no imports added; baby-phi-only changes).

---

## Estimated effort breakdown

~2.4 engineer-days:
- 0.7d — P1 enum + field adds (Grant.approval_mode, ConsentScope.session_id, Organization.approval_timeout, Organization.approval_timeout_default_response) + ApprovalTimeout + TimeoutResponse enums + migration 0013 + ~8 unit tests + literal-struct fixture cascade (~10–15 Organization sites).
- 0.6d — P2 engine Step 6 real body + `CheckContext.current_session` + `ConsentIndex` extensions + ~12 unit/property tests.
- 0.7d — P3 minters + repo methods + audit event + launch-handler wiring (deadline computation per `ApprovalTimeout`) + ~12 acceptance/integration tests.
- 0.4d — P4 ADR Accepted + drift closure + concept-doc bump + 2-agent audit (with the additional D48.11 / approval_timeout audit claim 4a) + seal.

**Total: 2.4d.** The estimate adds ~0.4d to the original forward-scope row's "~2 days" target — the delta is the locked F3.A choice (full Org config: `approval_timeout` + `approval_timeout_default_response` instead of the single-field deferral alternative). The user accepted the trade for full concept-doc fidelity.
