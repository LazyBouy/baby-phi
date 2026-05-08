<!-- Last verified: 2026-05-08 by Claude Code (CH-15 P3 — new architecture page for ADR-0054: session-launch manifest builder + hard-deny flip + Template A double-grant + migration 0015 backfill + launch_denied audit event.) -->

# Session-launch permission gate — design page

> **Status:** [EXISTS] as of CH-15 (M5.2). The typed manifest builder ships at [`modules/crates/domain/src/permissions/builders/session_launch.rs`](../../../../../../modules/crates/domain/src/permissions/builders/session_launch.rs); the launch handler's hard-deny flip lives at [`server/src/platform/sessions/launch.rs`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs); the Template A double-grant pure-fn + listener live at [`domain/src/templates/a.rs`](../../../../../../modules/crates/domain/src/templates/a.rs) + [`domain/src/events/listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs); the new `platform.session.launch_denied` audit-event builder lives at [`domain/src/audit/events/m5_2/session_launch.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/session_launch.rs); migration `0015_template_a_session_object_grant.surql` backfills legacy holders. For the normative concept-doc reference, read [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Permission Check (Runtime Reconciliation)" + §"Key Invariants" + [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"Template A — Project Lead Authority".

---

## What this page covers

The permissions concept docs require every session launch to gate at the security boundary — invariant 5 of `permissions/04` line 309–314 ("there is no 'default allow'") forbids the launch handler from spawning the agent task when the engine returns `Decision::Denied`. Concept doc 07 §"Template A — Project Lead Authority" further specifies that the lead has `[Read, Inspect, List]` on every session tagged `project:P` — the on-the-wire grant must cover the engine's session-launch reach.

Pre-CH-15, the launch handler advisory-logged every step-1-to-6 deny (drift D4.1) and proceeded to spawn regardless. CH-15 closes that gap. This page describes:

- The `build_session_launch_manifest` typed builder + its forward-scope re-interpretation (D54.1, D54.2).
- The Template A `fire_grant_on_lead_assignment` `Vec<Grant>` return shape (D54.3).
- Migration `0015` legacy-holder backfill (D54.4).
- The `platform.session.launch_denied` audit event (D54.5).
- The hard-deny error mapping (D54.6).
- The preview-launch parity at the manifest layer (D54.7).
- The forward-scope literal re-interpretation note for the planning ledger (D54.8).

ADR-0054 records the design decisions (sub-decisions D54.1–D54.8); this page is the operator-facing description.

---

## `build_session_launch_manifest` typed builder (D54.1)

```rust
// domain::permissions::builders::session_launch::
pub fn build_session_launch_manifest(_project_id: ProjectId) -> Manifest {
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

Pure — no I/O, no Repository. Both `preview_session` and `gate_session_launch_consent` call this builder so the preview-launch parity invariant holds — divergence at the manifest layer would re-open D4.1's "advisory layer" pattern at the consent boundary.

The `project_id` parameter is preserved on the signature even though today's body does not reference it. Future M6+ extensions will use it to vary the manifest per-project (e.g., per-project allowed-tool sets); preserving the signature now avoids a callsite cascade later.

### Forward-scope re-interpretation (D54.2 / D54.8)

The forward-scope row literal text *"actions `session.start` / `session.tool_invoke` / `session.read_memory`"* is a scoping-gloss, NOT a literal `Action` variant set. Concept doc 03's closed 34-verb vocabulary takes precedence; the launch boundary gates only the `session.start` semantics → `[Read, Inspect, List]` on `session_object`. `tool_invoke` + `read_memory` are runtime per-tool manifest reaches that ship at M6+ (forward-scope CH-21 memory-extraction-listener-body + CH-22 agent-catalog-listener-body).

`Action::CANONICAL.len() == 34` invariant test is unbroken at chunk close. The action × fundamental matrix stays at 9×10 cells.

---

## Template A `Vec<Grant>` pure-fn + paired session-object grant (D54.3)

```rust
// domain::templates::a::fire_grant_on_lead_assignment
pub fn fire_grant_on_lead_assignment(args: FireArgs) -> Vec<Grant> {
    // ...
    let project_grant = Grant {
        action: vec![Read, Inspect, List],
        resource: ResourceRef { uri: format!("project:{project}") },
        fundamentals: vec![Tag],
        // ...
    };
    let session_grant = Grant {
        action: vec![Read, Inspect, List],
        resource: ResourceRef {
            uri: format!(
                r#"tags contains "project:{project}" AND tags contains #kind:session"#
            ),
        },
        fundamentals: vec![DataObject, Tag],
        // ...
    };
    vec![project_grant, session_grant]
}
```

Each `HasLeadEdgeCreated` event mints **two** grants:

1. `grants[0]` — the **project-resource grant** (preserved verbatim from pre-CH-15 shape; the existing CEO-lead-on-project authority).
2. `grants[1]` — the NEW **session-object grant** covering session instances tagged `project:<uuid>`. The URI uses the selector-grammar string-literal form (`tags contains "project:<uuid>"`) because the grammar's `namespace_tag` rule requires `[A-Za-z_]` first char which excludes digit-leading UUIDs (concept-09 §"Tag Forms").

Both grants `descends_from` the same Template A adoption AR. The Authority Chain ([ADR-0053](../decisions/0053-system-genesis-authority-chain-revocation-cascade.md) §D53.3 walker) traverses the additional row transparently — `walk_provenance_chain` works without modification.

The TemplateAFireListener body (`domain/src/events/listeners.rs:264+`) iterates the Vec, persists each grant via `repo.create_grant(&g).await?`, and emits one `template.a.grant_fired` audit event per grant (so operators see the full pair in the audit log).

---

## Migration `0015` legacy backfill (D54.4)

```surql
-- modules/crates/store/migrations/0015_template_a_session_object_grant.surql
FOR $g IN (
    SELECT ... FROM grant
    WHERE
        revoked_at = NONE
        AND string::starts_with(resource_uri, "project:")
        AND descends_from IN (SELECT VALUE record::id(id)
            FROM auth_request WHERE kinds CONTAINS "#template:a")
        AND descends_from NOT IN (SELECT VALUE descends_from
            FROM grant WHERE revoked_at = NONE
                AND string::starts_with(resource_uri, "tags contains project:"))
) {
    -- mint paired session-object grant ...
};
```

The migration walks every active legacy Template A grant (pre-CH-15 shipped with `resource_uri = "project:<uuid>"` only) and inserts the paired session-object grant under the same `descends_from` provenance. Idempotent on re-run via the migration-runner ledger (ADR-0033 §D33.2) + the inline NOT-EXISTS guard against ledger drift.

Standard SurrealDB embedded-mode behaviour: the migration runs on `SurrealStore::open_*` BEFORE the server accepts requests (per ADR-0033 §D33.2); the same-deploy code change activates hard-deny only after the migration runs. The "gap" between deploy + backfill is sub-millisecond.

Skipped rows: revoked legacy grants + grants whose `descends_from` points at a non-Template-A AR (e.g., Template B/C/D adoption ARs). The acceptance test at [`acceptance_template_a_session_grant_backfill.rs`](../../../../../../modules/crates/server/tests/acceptance_template_a_session_grant_backfill.rs) pins both invariants.

---

## `platform.session.launch_denied` audit event (D54.5)

```rust
// domain::audit::events::m5_2::session_launch::session_launch_denied
pub fn session_launch_denied(
    actor: AgentId,
    session_id: SessionId,
    agent_id: AgentId,
    project_id: ProjectId,
    org_id: OrgId,
    failed_step: u8,
    reason_kind: &str,
    reason_detail: Option<serde_json::Value>,
    emitted_at: DateTime<Utc>,
) -> AuditEvent { /* ... */ }
```

Diff shape (canonical_bytes contributors marked `*`):

```json
{
  "before": null,
  "after": {
    "session_id":   "<uuid>",          // *
    "agent_id":     "<uuid>",          // *
    "project_id":   "<uuid>",          // *
    "org_id":       "<uuid>",          // *
    "failed_step":  0,                  // * (u8, 0..6)
    "reason_kind":  "no_grants_held",  // * (DeniedReason variant tag, snake_case)
    "reason_detail": { ... },          // optional, non-canonical
    "emitted_at":   "<rfc3339>"        // *
  }
}
```

`audit_class = Alerted` per concept doc 04 invariant 5 ("audit trail on every outcome") + concept doc 07 §"audit_class composition" (failed permission checks default to alerted). 60s delivery to org alert channel per `nfr-observability.md`.

`session_id` is allocated at Step 3.5 by the launch handler even on the deny path so the audit row carries a stable cross-ref to the (would-be) session row + matches `platform.session.started`'s convention.

`provenance_auth_request_id = None` — the deny event isn't tied to an AR; it's a runtime-side decision from the engine, not a Template fire.

---

## Hard-deny error mapping (D54.6)

The launch handler's Step 3 advisory-arm (pre-CH-15: `if let Decision::Denied { ... } = preview.decision { ... advisory log ... }`) is replaced with:

```rust
if let Decision::Denied { ref failed_step, ref reason } = preview.decision {
    let step: u8 = match failed_step {
        FailedStep::Catalogue => 0,
        FailedStep::Expansion => 1,
        FailedStep::Resolution => 2,
        FailedStep::Ceiling => 2,
        FailedStep::Match => 3,
        FailedStep::Constraint => 4,
        FailedStep::Scope => 5,
        FailedStep::Consent => 6,
    };
    let reason_kind = denied_reason_kind(reason);
    let denied_event = session_launch_denied(...);
    let _ = audit.emit(denied_event).await;
    return Err(SessionError::PermissionCheckFailed {
        step,
        reason: format!("{reason:?}"),
    });
}
```

`SessionError::PermissionCheckFailed` already maps to 403 via `http_status_for` and to wire-code `PERMISSION_CHECK_FAILED` via `wire_code_for` — no new error variant needed. `Ceiling` is a sub-step of `Resolution` and surfaces as `2` on the wire (the metric label `"2a"` is preserved on the receipt's reason text).

Step 6 (Consent) deny remains routed through `gate_session_launch_consent` per [ADR-0048](../decisions/0048-per-session-consent-gating.md) — only Steps 1–5 widen from advisory to hard-deny because Step 6 was already enforced (CH-11).

Audit-emit failures do NOT block the deny path; the launch is rejected regardless. Failed audit emits are logged at `tracing::warn` so operators see the gap in the audit chain.

---

## Preview-launch parity (D54.7)

Both `preview.rs::preview_session` and `launch.rs::gate_session_launch_consent` call `build_session_launch_manifest(input.project_id)`. The synthetic `ToolCall` in both paths carries:

- `target_uri = ""` — class-level reach (skips Step 0 Catalogue).
- `target_tags = ["#kind:session", "project:<uuid>", "org:<uuid>"]` — provides the kind_refinement match for Step 3 + the project predicate for Step 5's scope cascade.
- `target_agent` — `Some(input.agent_id)` in launch (so Step 6 consent gate routes correctly); `None` in preview (preview is stateless).

Preview-launch parity is exercised by the `preview_decision_matches_launch_decision_on_grants_stable` acceptance test.

---

## Acceptance tests + invariants

| Test | Pinned invariant |
|---|---|
| `launch_denies_with_403_when_agent_holds_no_session_grants` | Agent without Template A session grant fails Step 2 (Resolution) — 403 `PERMISSION_CHECK_FAILED_AT_STEP_2`. |
| `launch_succeeds_with_template_a_session_grant_after_p1_extension` | Template A holder with paired grants launches successfully. |
| `launch_emits_launch_denied_audit_event_on_403` | Every step-1-to-6 deny emits exactly one `platform.session.launch_denied` audit event with `audit_class = Alerted`. |
| `launch_does_not_register_session_on_403` | `SessionRegistry` size unchanged after deny ([ADR-0033](../decisions/0033-k8s-prep-refactors.md) §D33.1 invariant — `registry.insert` never reached). |
| `preview_decision_matches_launch_decision_on_grants_stable` | Preview's Decision matches launch's Decision when grants are stable. |
| `preview_and_launch_both_deny_when_grants_absent` | Mirror invariant — both deny when no grants. |
| `launch_denied_audit_event_failed_step_matches_wire_step` | Audit-event `failed_step` numeric == wire `STEP_<N>` body — dashboard correlation. |
| `launch_denied_audit_event_carries_correlation_ids` | Audit event carries agent_id + project_id + org_id + session_id for cross-ref. |
| `migration_0015_seeds_paired_session_object_grant_for_legacy_template_a_grant` | Backfill writes the paired grant under shared provenance. |
| `migration_0015_skips_revoked_legacy_grants` | Idempotent skip: revoked grants are not backfilled. |
| `migration_0015_skips_non_template_a_grants` | Template B/C/D ARs are ignored by the backfill. |
| `template_a_listener_persists_paired_session_object_grant` | Production listener mints both grants per `HasLeadEdgeCreated`. |
| `template_a_listener_emits_two_audit_events_per_lead_assignment` | Listener emits one audit event per persisted grant. |

---

## K8s readiness — axes A1–A7 conformance

| Axis | Verdict | Notes |
|---|---|---|
| **A1** New in-process state | none | Reuses `gate_session_launch_consent`'s in-process `HashSet<AuthRequestId>` (`template_gated`). |
| **A2** New IPC channel | none | CH-15 stays synchronous (engine call + repo call). |
| **A3** New pod-local resource | none | |
| **A4** Migration runner / first-apply race | covered by ADR-0033 §D33.2 | Migration `0015` adds an additive UPDATE; runs on `open_*` before request-serving begins. Standard CHK8S-D-05 leader-election applies; not aggravated by additive UPDATEs. |
| **A5** Trait-shape requirement | none | `build_session_launch_manifest` is a pure free-fn. |
| **A6** Cross-pod state sharing | none | Permission Check is read-only against persisted Grant rows; cross-pod sharing happens via SurrealDB. |
| **A7** Audit hash-chain symmetry | preserved | New `platform.session.launch_denied` variant flows through the existing single-writer `AuditEmitter` trait. Additive event types do not perturb prior events' canonical bytes. |

CH-15 is K8s-neutral. No new blocker class.

---

## Cross-references

- [ADR-0054](../decisions/0054-session-launch-manifest-and-hard-deny-flip.md) — design decisions D54.1–D54.8.
- [ADR-0033](../decisions/0033-k8s-prep-refactors.md) §D33.1 / §D33.2 — `SessionRegistry` trait + migration runner.
- [ADR-0044](../decisions/0044-publish-time-manifest-validator.md) — manifest validator precedent.
- [ADR-0048](../decisions/0048-per-session-consent-gating.md) §D48.3 / §D48.5 / §D48.7 — per-session consent gating reused by hard-deny path.
- [ADR-0050](../decisions/0050-audit-class-composition-strictest-wins.md) §D50.5 — audit-class composition precedent.
- [ADR-0053](../decisions/0053-system-genesis-authority-chain-revocation-cascade.md) §D53.3 / §D53.5 — provenance chain reused (both new grants `descends_from` the Template A adoption AR).
- [m5/architecture/session-launch.md](../../m5/architecture/session-launch.md) — pre-CH-15 launch flow + Step 3 amendment.
- [m5_2/operations/session-launch-permission-gate-operations.md](../operations/session-launch-permission-gate-operations.md) — runbook for the audit event + per-step deny playbook.
- [drifts/D4.1.md](../../m5_1/drifts/D4.1.md) — closed at CH-15 chunk-seal.
