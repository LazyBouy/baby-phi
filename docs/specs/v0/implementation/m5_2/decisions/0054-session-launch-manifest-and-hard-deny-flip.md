<!-- Last verified: 2026-05-08 by Claude Code (CH-15 P4 chunk-seal — Status flipped Proposed → Accepted; sub-decisions D54.1–D54.8 pinned by P0–P3 deliverables: typed builder at `domain::permissions::builders::session_launch::build_session_launch_manifest`; hard-deny match arm at `server::platform::sessions::launch.rs:226`; Template A `Vec<Grant>` pure-fn at `templates/a.rs`; new audit-event builder at `audit/events/m5_2/session_launch.rs`; migration `0015_template_a_session_object_grant.surql`; D4.1 transitioned to remediated; concept-audit-matrix rows 180 + 219 flipped letter-for-letter per CH-12 F-AUDB-1.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-15 P0 — ADR drafted as Proposed) -->

# ADR-0054 — Session-launch manifest, hard-deny flip, and Template A `session_object` grant extension

**Status: Accepted**

**Date:** 2026-05-08
**Chunk:** CH-15
**Closes:**
- [`D4.1`](../../m5_1/drifts/D4.1.md) (HIGH, A) — Permission Check is advisory-only at M5 (launch blocks on Step 0 Catalogue only). Concept doc 04 §"Permission Check (Runtime Reconciliation)" + §"Key Invariants" line 310 ("there is no 'default allow'") is contradicted: any agent without grants on `session_object` can launch a session today. CH-15 closes the drift end-to-end via (a) typed builder for the synthetic launch manifest; (b) hard-deny flip on Steps 1–6; (c) Template A grant extension to mint a paired `session_object` grant; (d) migration `0015` backfilling legacy Template A holders; (e) new `platform.session.launch_denied` audit event on every step-1-to-6 deny.

---

## Context

(Body fills at P1–P3 — context paragraph will pin concept doc `permissions/04-manifest-and-resolution.md` §"Permission Check (Runtime Reconciliation)" lines 166–202 + §"Formal Algorithm (Pseudocode)" lines 209–305 + §"Key Invariants" lines 309–314 as canonical for the hard-deny semantic; concept doc `permissions/07-templates-and-tools.md` §"Template A — Project Lead Authority" lines 36–40 + §"Templates Are Pre-Authorized Allocations" lines 14–72 as canonical for the Template A double-grant shape; concept doc `permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" lines 31–82 as canonical for the closed-set-action re-interpretation rationale.)

## Forks

- F1 → F1.A (extend Template A pure-fn return type from `Grant` to `Vec<Grant>` so each fire mints both the existing `project:<id>` grant and a new `session_object` grant; ship migration `0015` to backfill legacy Template A holders) — user-locked at plan approval 2026-05-08.
- F2 → F2.A (NEW `domain::permissions::builders` module hosting `build_session_launch_manifest(project_id) -> Manifest`; both preview + launch + consent-gate call it) — user-locked at plan approval 2026-05-08.
- F3 → F3.A (reuse existing `[Read, Inspect, List]` action set on `session_object`; closed 34-verb action vocabulary preserved verbatim per concept doc 03; the forward-scope row's `session.start / session.tool_invoke / session.read_memory` wording is re-interpreted as scoping-gloss — only `session.start` semantics is launch-time-relevant; `tool_invoke` + `read_memory` ship at M6+ via the per-tool runtime manifest path) — user-locked at plan approval 2026-05-08.
- F4 → F4.C (migration `0015` runs BEFORE launch.rs hard-deny flip via standard SurrealDB embedded-mode `SurrealStore::open_*` migration-runner discipline per ADR-0033 §D33.2; no grace window; no implicit-allow back-door) — user-locked at plan approval 2026-05-08.
- F5 → F5.B (new `platform.session.launch_denied` audit event lives in `domain/src/audit/events/m5_2/session_launch.rs` per the m5_2 audit-events module organisation; CH-13 / CH-14 added per-feature audit-event modules and CH-15 follows the same pattern) — user-locked at plan approval 2026-05-08.

All five forks at planner-recommendation (F1–F5 user-locked at plan approval to F1.A / F2.A / F3.A / F4.C / F5.B — all align with planner recommendation).

---

## Decision

### D54.1 — `domain::permissions::builders::build_session_launch_manifest` pure-fn (F2.A)

A new module `domain::permissions::builders` ships at `modules/crates/domain/src/permissions/builders/`:

```rust
// builders/session_launch.rs
pub fn build_session_launch_manifest(project_id: ProjectId) -> Manifest {
    Manifest {
        actions: vec![Action::Read, Action::Inspect, Action::List],
        resource: vec!["session_object".to_string()],
        transitive: vec![],
        constraints: vec![],
        constraint_requirements: HashMap::new(),
        kinds: vec![],
    }
}
```

The builder is pure (no I/O, no Repository). Both preview.rs (line 88-95) and launch.rs (gate_session_launch_consent line 594-601) replace their inline `Manifest { ... }` literals with `build_session_launch_manifest(input.project_id)` calls. Preview-launch parity preserved at the manifest layer.

Future builders (per-action invocation manifests for tools, memory-recall manifests, etc.) co-locate under `permissions/builders/`.

**Rejected alternatives:**
- F2.B (sibling fn in `permissions::manifest::session_launch`) — `manifest/mod.rs` should stay focused on the engine input contract type rather than its construction sites; F2.A gives a natural home for the M6+ builder fan-out.

### D54.2 — Action vocabulary preserved verbatim (F3.A re-interpretation)

CH-15 adds **zero new `Action` variants**. The closed 34-verb vocabulary defined in `concepts/permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" lines 31–82 is preserved; `Action::CANONICAL.len() == 34` invariant unbroken.

The forward-scope row's literal text *"actions `session.start` / `session.tool_invoke` / `session.read_memory`"* is re-interpreted as a scoping-gloss describing the three logical reaches the launch handler eventually wants to gate, NOT as literal Action variants. The mapping per concept docs 03 + 04 + 07:

- `session.start` → `Action::{Read, Inspect, List}` on `session_object` (the launching agent reads/inspects/lists the session lifecycle).
- `session.tool_invoke` → `Action::Invoke` on the tool's manifest at runtime — NOT a launch-time reach. Per concept doc 04 §"Permission Check (Runtime Reconciliation)" runtime tool invocations gate at the per-tool manifest, not the launch manifest. **Out of scope for CH-15; ships at M6+.**
- `session.read_memory` → `Action::Recall` on `memory_object` — also NOT a launch-time reach. Per concept doc 07 §"recall_memory" gated at the per-tool manifest at runtime. **Out of scope for CH-15; ships at M6+.**

CH-15's launch manifest gates **only** the `session.start` semantics → `[Read, Inspect, List]` on `session_object`. This matches Template A's existing `[Read, Inspect, List]` issuance shape exactly (per `templates/a.rs:114-118`), giving F1.A a clean grant-shape match.

**Rejected alternatives:**
- F3.B (add 3 new `Action` variants `session.start` / `session.tool_invoke` / `session.read_memory`) — would expand the closed 34-verb vocabulary to 37 verbs; concept doc 03 update required; widens the action × fundamental matrix from 9×10 to 9×11 (+27 cells); breaks `Action::CANONICAL.len() == 34` invariant. Concept-contradiction.

### D54.3 — Template A `fire_grant_on_lead_assignment` returns `Vec<Grant>` (F1.A)

The pure-fn signature changes from `(args: FireArgs) -> Grant` to `(args: FireArgs) -> Vec<Grant>`. CascadeResult-style typed-multi-value precedent per CH-14 retro v7.

Each `HAS_LEAD` edge fire mints **two** grants:
1. The existing `project:<id>` grant (preserved verbatim — same `holder`, `action`, `resource.uri`, `fundamentals`, `descends_from`, `delegable`, `issued_at`, `revoked_at`, `approval_mode`, `audit_class`, `allocate_refinement` shape).
2. A NEW `session_object` grant covering session instances tagged `project:<id>`. Same holder + action set + descends_from + delegable + issued_at + revoked_at + approval_mode + audit_class as the project-resource grant; differs only in `resource.uri` (`session_object/project:<id>` selector form) and `fundamentals` (`vec![DataObject, Tag]` per concept doc 01 §"Composite Classes" — `SessionObject` expands to `[DataObject, Tag]`).

Both grants `descends_from` the same `adoption_auth_request_id`. The Authority Chain (concept doc 04 §"The Authority Chain" lines 510–547) is preserved — `walk_provenance_chain` traversal handles the additional row transparently.

The TemplateAFireListener body iterates the Vec, persists each grant via `repo.create_grant(&g).await?`, and emits one `template.a.grant_fired` audit event per grant. The 7 existing single-grant unit tests in `templates/a.rs` are amended to assert on `grants[0]` (the project-resource grant) plus 6 new tests covering the second (session-resource) grant.

**Rejected alternatives:**
- F1.B (implicit-allow-via-Template-A-presence at launch.rs Step 3.5) — circumvents concept doc 04 §"Permission Check" worked example ("the agent must hold a grant covering the manifest's (action, fundamental) reaches"). New HIGH drift exactly as we close D4.1.
- F1.C (extend pure-fn for new grants + grace window for legacy holders) — ships drift in the form of a defined grace window with no automatic close.

### D54.4 — Migration `0015_template_a_session_object_grant.surql` backfills legacy Template A holders (F4.C)

Migration `0015` walks every active `Grant` whose `descends_from` matches a Template A adoption AR (`auth_request` with `kinds CONTAINS '#template:a'`) and `resource_uri STARTS_WITH 'project:'` and inserts the paired `session_object/project:<id>` grant. Idempotent on re-run via existence check (skip if a paired grant on the same `descends_from` already exists with `resource_uri` matching the session-object selector form).

Standard SurrealDB embedded-mode behaviour per ADR-0033 §D33.2 — migrations run on `SurrealStore::open_*` before the server accepts requests. Same-deploy ordering: migration `0015` populates session-scoped grants atomically; same-deploy code change activates hard-deny only after migration runs. The "gap" between deploy + backfill is sub-millisecond.

**Rejected alternatives:**
- F4.A (hard-deny flips immediately, no backfill) — pre-CH-15 sessions launching mid-deploy fail. The standard migration-runner discipline already prevents this, but F4.A is silent about it; F4.C documents it explicitly.
- F4.B (implicit-allow grace window) — concept-aspirational drift; rejected for the same reasons as F1.B / F1.C.

### D54.5 — `platform.session.launch_denied` audit event (F5.B)

A new audit-event builder lives at `domain/src/audit/events/m5_2/session_launch.rs`:

```
platform.session.launch_denied {
  session_id*: SessionId,        // pre-allocated at Step 3.5 even on deny path
  agent_id*: AgentId,
  project_id*: ProjectId,
  org_id*: OrgId,
  failed_step*: u8,              // 0..6 per FailedStep::as_metric_label
  reason_kind*: String,           // DeniedReason variant tag (snake_case)
  reason_detail: serde_json::Value,  // OPTIONAL — non-canonical (operator data)
  emitted_at*: DateTime<Utc>,
}
```

Asterisked fields contribute to `canonical_bytes`. Audit class: `Alerted` (concept doc 04 invariant 5: "audit trail on every outcome"; deny is alert-worthy per concept doc 07 §"audit_class composition" — failed permission checks default to `alerted`).

Emitted by the launch handler on every step-1-to-6 deny BEFORE returning `Err(SessionError::PermissionCheckFailed { step, reason })`. Threaded through the existing `Arc<dyn AuditEmitter>`.

**Rejected alternatives:**
- F5.A (reuse `permission_check_decision` field on `LaunchReceipt` only; no new audit event) — receipt isn't preserved across the deny path (the function returns `Err` before constructing a receipt); without a discrete audit event the deny is operator-invisible.

### D54.6 — Hard-deny error mapping (Step N → `PERMISSION_CHECK_FAILED_AT_STEP_<N>` 403)

The `if let Decision::Denied { ... }` advisory-arm at `launch.rs:226-246` is converted to a `match preview.decision` with a hard-deny arm:

```rust
match preview.decision {
    Decision::Allowed { .. } => { /* fall through to Step 3.5 */ }
    Decision::Pending { .. } => { /* fall through to Step 3.5 (consent gating) */ }
    Decision::Denied { failed_step, ref reason } => {
        let step = failed_step.as_metric_label().parse::<u8>().unwrap_or(0);
        // emit launch_denied audit event
        // ...
        return Err(SessionError::PermissionCheckFailed {
            step,
            reason: format!("{reason:?}"),
        });
    }
}
```

`SessionError::PermissionCheckFailed` already maps to 403 via `http_status_for` (line 162) and to wire-code `PERMISSION_CHECK_FAILED` via `wire_code_for` (line 200) — no new error variant needed. The `tracing::info!(... "advisory at M5; not blocking")` line is removed; the corresponding string disappears from the codebase.

Step 6 (Consent) deny remains routed through `gate_session_launch_consent` per CH-11 + ADR-0048 §D48.5 — only Steps 1–5 widen from advisory to hard-deny because Step 6 was already enforced (CH-11).

### D54.7 — Preview-launch parity at the manifest layer (F2.A consequence)

Both `preview.rs::preview_session` (line 88-95) and `launch.rs::gate_session_launch_consent` (line 594-601) replace their inline `Manifest { ... }` literals with calls to `domain::permissions::builders::build_session_launch_manifest(input.project_id)`. This preserves the M5 invariant that preview's Decision matches launch's Decision when grants are stable — divergence at the manifest layer would re-open D4.1's "advisory layer" pattern at the consent boundary.

The catalogue seed at preview.rs:101 + launch.rs:603 also flips from `"session"` / `"identity_principal"` to `"session_object"` so Step 0 doesn't mis-miss on the synthetic resource URI. Catalogue is per-org so the change is local to the launch-time call sites.

### D54.8 — Forward-scope row literal re-interpretation note for the planning ledger

The forward-scope row at `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 147–151 names *"actions `session.start` / `session.tool_invoke` / `session.read_memory`"* as the launch-time reaches. CH-15 re-interprets this wording as scoping-gloss per D54.2 above. Future planners reading the row should consult this ADR (D54.2) for the resolved canonical Action mapping rather than introducing new Action variants.

---

## Pre-existing behaviour preservation

**Pre-CH-15 launch.rs advisory-log behaviour** (preserved at the field level for receipts; behaviour flips at Step 3 hard-deny):
- At M5/P4, `launch.rs:198-246` advisory-logs every step-1-to-6 Permission-Check denial via `tracing::info!(..., 'sessions::launch: Permission Check denied (advisory at M5; not blocking)')` and proceeds to `spawn_agent_task`.
- Only Step 0 (Catalogue) gates, returning 403 `PERMISSION_CHECK_FAILED_AT_STEP_0`.

**Post-CH-15 behaviour:**
- Every `Decision::Denied { failed_step, reason }` from the launch-time engine call returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>` where `<N>` is `failed_step.as_metric_label()` (0..6).
- The advisory `tracing::info!` block at line 244 is removed; the corresponding 'advisory at M5; not blocking' string disappears from the codebase.
- Step 6 (Consent) deny remains routed through `gate_session_launch_consent` per CH-11 + ADR-0048 — only Steps 1–5 widen from advisory to hard-deny because Step 6 was already enforced (CH-11).

**Template A pre-existing behaviour preserved:**
- Pre-CH-15 Template A grants minted `[Read, Inspect, List]` on `project:<id>` only (`templates/a.rs:114-122`).
- CH-15 extends `fire_grant_on_lead_assignment` to mint a SECOND grant on `session_object` filtered by `tags contains project:<id>`. The first grant is preserved verbatim — CH-15 does NOT remove the project-resource grant, only ADD a session-resource grant alongside.
- Migration `0015` walks every `Grant` whose `descends_from` matches a Template A adoption AR and inserts the paired session-resource grant; idempotent on re-run via existence check.

---

## Cross-references

### Originating concept doc + section + line range

- `concepts/permissions/04-manifest-and-resolution.md` §"Permission Check (Runtime Reconciliation)" lines 166–202 + §"Formal Algorithm (Pseudocode)" lines 209–305 + §"Key Invariants" lines 309–314.
- `concepts/permissions/07-templates-and-tools.md` §"Template A — Project Lead Authority" lines 36–40 + §"Templates Are Pre-Authorized Allocations" lines 14–72.
- `concepts/permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" lines 31–82 (re-interpretation rationale for D54.2).

### Closed drifts

- [`m5_1/drifts/D4.1.md`](../../m5_1/drifts/D4.1.md) (HIGH, A) — primary; transitions `discovered → remediated` at chunk seal.

### Prior ADRs cited as precedent (milestone-prefixed)

- [`m5_2/decisions/0033-k8s-prep-refactors.md`](0033-k8s-prep-refactors.md) §D33.1/§D33.2 — `SessionRegistry` trait + `SurrealStore::open_*` migration-runner discipline (D54.4 reuse).
- [`m5_2/decisions/0044-publish-time-manifest-validator.md`](0044-publish-time-manifest-validator.md) §D44.A–§D44.D — manifest validator precedent (D54.1 builder reuse pattern).
- [`m5_2/decisions/0048-per-session-consent-gating.md`](0048-per-session-consent-gating.md) §D48.3 / §D48.5 / §D48.7 — per-session ambient-context plumbing reused by hard-deny path (D54.6).
- [`m5_2/decisions/0050-audit-class-composition-strictest-wins.md`](0050-audit-class-composition-strictest-wins.md) §D50.5 / §D50.6 — audit-class composition precedent (D54.5 `Alerted` justification).
- [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](0053-system-genesis-authority-chain-revocation-cascade.md) §D53.3 / §D53.5 — provenance chain reused (D54.3 — both new grants `descends_from` the Template A adoption AR).
- [`m4/decisions/0028-domain-event-bus.md`](../../m4/decisions/0028-domain-event-bus.md) — Template A listener wiring precedent (D54.3 dual-grant emission).
- [`m5/decisions/0029-session-persistence-and-recorder-wrap.md`](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) — launch handler architecture (D54.6 hard-deny placement).
- [`m5/decisions/0031-session-cancellation-and-concurrency.md`](../../m5/decisions/0031-session-cancellation-and-concurrency.md) — launch handler concurrency model (D54.6 hard-deny BEFORE registry.insert).

### Forward-scope row

- [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 row 13 (CH-15 row, lines 147–151) — including the *forward-scope-literal-re-interpretation* note (§D54.8).

---

## Consequences

(Body fills at P1–P3.)

---

## Audit / verification

(Body fills at P3 with the canonical `cargo test` + greps + CI guard list per plan §12.)
