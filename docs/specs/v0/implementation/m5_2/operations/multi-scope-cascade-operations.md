<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P3 — operations playbook for multi-scope cascade including IntersectionEmpty error code + contractor-bound troubleshooting tree) -->

# Multi-scope cascade operations runbook

> **Audience:** SREs and operators verifying or debugging multi-scope session-read denials, contractor-bound ceiling behaviour, and the new `DeniedReason::IntersectionEmpty` error code shipped at CH-07. Pair this page with [`multi-scope-cascade.md`](../architecture/multi-scope-cascade.md) (design) and [ADR-0051](../decisions/0051-multi-scope-cascade-contractor-model.md).

---

## Error-code dictionary entry

### `DeniedReason::IntersectionEmpty`

**Variant signature:**

```rust
DeniedReason::IntersectionEmpty {
    fundamental: Fundamental,
    action: Action,
    session_scope_count: u8,
}
```

**When fired:** A reader (the agent making the call) is in **0** of the session's tagged scopes (project tier `count == 0` AND org tier `count == 0` per the multi-scope cascade), and the intersection-fallback ceiling re-clamp at `engine.rs::cascade_intersection_fallback` returned an empty winner set. Concept-doc 06 lines 60–62 ("the outsider faces the intersection of all the session's scope ceilings; deny by default if empty") is the normative reference. ADR-0051 §D51.5 + §D51.7 pin the typed-Rust shape.

**Decision shape:**

```rust
Decision::Denied {
    failed_step: FailedStep::Scope,   // metric label "5"
    reason: DeniedReason::IntersectionEmpty { .. },
}
```

The denial routes through the existing `FailedStep::Scope` channel (no new failed-step variant; ADR-0051 §D51.7 — additive-enum discipline).

**Field semantics:**

- `fundamental`: the [`Fundamental`](../../../../../../modules/crates/domain/src/model/fundamentals.rs) the failing reach was for (e.g. `FilesystemObject`, `MemoryObject`).
- `action`: the [`Action`](../../../../../../modules/crates/domain/src/permissions/action.rs) the failing reach was for (e.g. `Read`, `Modify`).
- `session_scope_count: u8`: total count of session-tagged scopes the cascade considered = `len(session_org_tags) + len(session_project_tags)`, clamped to `u8::MAX`. This number is safe to log; the specific OrgIds / ProjectIds are deliberately NOT carried on the denial (audit-leak avoidance — concept-doc 06 line 161+).

**What to log on triage:**

- The `session_scope_count` value (number of scopes considered).
- The reader's `agent_id` (from `ctx.agent`).
- The `(fundamental, action)` reach pair (from the variant's fields).
- The session's tag set (from the calling layer's `Session.tags` — the engine itself has only the parsed `session_org_tags` / `session_project_tags` slices on `CheckContext`; the launch handler holds the source `Session` row).

**How to triage:**

1. Confirm the reader IS expected to be excluded — `IntersectionEmpty` is the **correct** denial for the contractor / outsider case. If the operator expected the reader to succeed, jump to the troubleshooting tree below.
2. Verify the session's tag set is well-formed: at least one of `[org:<uuid>, project:<uuid>]` must be present. A session with neither prefix would not enter the multi-scope path at all (the cascade falls back to the single-scope `cascade_single_scope` body).
3. If the reader IS expected to succeed but is denied, walk the troubleshooting tree below.

---

## Audit-event mapping

CH-07 introduces **no new audit events**. The new `DeniedReason::IntersectionEmpty` denial flows through the existing `Decision::Denied` audit-event path established at CH-13 (ADR-0050 §D50.1 — audit-class composition). Any audit-event builder downstream of `permission_check.denied` consumes the existing `failed_step` + `reason` payload; the new variant serializes via the existing `#[serde(tag = "kind", rename_all = "snake_case")]` shape on `DeniedReason`, producing a `kind: "intersection_empty"` JSON field.

The `audit_class` of the denial event is determined by the same composition rule as every other denial — strictest-wins via the candidate grant's `audit_class` (per ADR-0050 §D50.5 + the per-org default). For the `IntersectionEmpty` case specifically, no candidate grant won (the intersection clamped everything to empty), so the audit-class composer falls back to the org-default — consistent with every other Step-5 denial.

---

## Troubleshooting tree

Three questions cover the most common operator-confusion paths. Each branch ends with a concrete remediation step.

### Q1 — "An outsider was unexpectedly denied with `IntersectionEmpty` — but they should have access."

The cascade's outsider-denial is structural: the reader is in 0 of the session's tagged scopes AND no session-org ceiling admits their candidate grants. If you expected access to succeed, walk through these checks:

1. **Inspect the session's tag set.** Query the source session row:
   ```sql
   SELECT id, tags FROM session WHERE id = <session_id>;
   ```
   Confirm the `tags` field carries the expected `org:<uuid>` / `project:<uuid>` prefixes. Concept-doc 06 lines 14–53 specify that multi-scope sessions are tagged via `session.tags`; if a tag the operator expected is missing, the launch handler tag-emission path (`events/listeners.rs:703–704` for the governance tag-build, plus the session-creation path) is the place to look.
2. **Inspect the reader's org-grant set.** The reader's `org_grants` slice (constructed by the launch handler from `Repository::org_grants_for(reader, session.org_scope)`) carries the candidate grants the cascade walks. Confirm the reader has a grant whose `holder == PrincipalRef::Organization(o)` for at least one `o ∈ session.tags`. If not, the reader is structurally an outsider for this session.
3. **Verify the candidate-org's ceiling permits the reader's reach.** The intersection-fallback re-clamp uses session-tagged orgs as ceilings. A reader whose candidate is `Read` on `filesystem_object`, with a session-tagged org ceiling that only grants `Connect`, will be clamped out. If you expected the cascade to succeed via the intersection fallback, audit the session-tagged orgs' grant sets for one whose actions ⊇ the reader's candidate actions.

### Q2 — "A contractor's `base_org` ceiling was unexpectedly applied (the contractor's home-org strict ceiling clamped their candidates inside another org's session)."

This would be a **regression** of the contractor-model bound. CH-07 §D51.6 enforces that a non-member org's ceiling is structurally excluded from `step_2a_ceiling` clamping. Walk these checks:

1. **Confirm the session's `org` tags.** A session tagged `[org:acme]` only must NOT have the contractor's `base_org` (e.g. Gamma) in its tag set. Re-query `Session.tags` and the launch-handler tag-parse output (the launch handler computes `session_org_tags` from `Session.tags` via the `parse_session_scope_tags` helper at `manifest/mod.rs`).
2. **Confirm the launch handler populated `CheckContext.session_org_tags` correctly.** If the slice is empty (`&[]`), the cascade reduces to single-scope back-compat and EVERY ceiling clamps uniformly — including the contractor's base_org ceiling. The bug, if any, is in the launch handler's tag-parse: did it call the parse helper, did the parse find the tags, and did the parse's output flow into `CheckContext` construction?
3. **Confirm the ceiling grant's `holder` is `PrincipalRef::Organization(o)` and not `PrincipalRef::Project(_)` / `PrincipalRef::Agent(_)`.** The membership bound applies only to Organization-tier ceilings; Project / Agent ceilings pass through unchanged (concept-doc 06 line 162 frames the bound around `base_organization`). A misclassified ceiling holder would explain a Project/Agent-tier ceiling clamping unexpectedly.

### Q3 — "A contractor's `base_org` ceiling was unexpectedly NOT applied (where it should have been)."

This is the inverse: the contractor's home-org IS a member of the session's tag set, and the operator expected the home-org ceiling to clamp.

1. **Confirm the session's `org` tags include the contractor's home-org.** A session tagged `[org:acme, org:gamma]` (Shape B with both Acme and Gamma) means Gamma's ceiling IS applicable per the membership bound — the contractor is then operating in their home-org's joint scope, so the ceiling clamps. If the session is tagged `[org:acme]` only (single Acme org), the contractor's Gamma ceiling is correctly excluded — that is the intended behaviour.
2. **Verify the launch handler emitted the home-org tag.** When a contractor is invited into a joint session, the launch handler must include the contractor's home-org in `session.tags` if the operator wants the home-org's ceiling to apply. The session-creation path (CH-15's territory) is where the operator-facing affordance to include / exclude the home-org tag will land; pre-CH-15, the test fixtures construct the tag set directly.
3. **Verify the home-org ceiling grant exists.** Ceiling grants are filtered by `revoked_at.is_none()` at `step_2a_ceiling`. A revoked ceiling will not clamp. Re-query the home-org's grant set and confirm the expected ceiling grant has not been revoked.

---

## Cross-references

- **ADR**: [`m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`](../decisions/0051-multi-scope-cascade-contractor-model.md) — sub-decisions §D51.1–§D51.7.
- **Architecture page**: [`multi-scope-cascade.md`](../architecture/multi-scope-cascade.md) — cascade design, contractor-bound diagram, intersection-fallback semantic.
- **Concept docs**:
  - [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Mechanism 2: Scope Resolution" lines 354–375.
  - [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"The Unified Resolution Rule" lines 28–63 + §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166.
  - [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §"Step 4: Multi-Scope resolution" lines 192–222 + §"Step 7: Contractor scenario" lines 287–298.
- **Drifts**:
  - [`m5_1/drifts/D-new-06.md`](../../m5_1/drifts/D-new-06.md) — multi-scope cascade body (HIGH; closed CH-07).
  - [`m5_1/drifts/D-new-20.md`](../../m5_1/drifts/D-new-20.md) — contractor-model bound (MEDIUM; closed CH-07).
- **Acceptance tests**: [`modules/crates/domain/tests/multi_scope_cascade_acceptance.rs`](../../../../../../modules/crates/domain/tests/multi_scope_cascade_acceptance.rs) — 6 end-to-end scenarios.
- **Audit-class composition (precedent for additive denial-class composition)**: [`m5_2/architecture/audit-class-composition.md`](../architecture/audit-class-composition.md) + [ADR-0050](../decisions/0050-audit-class-composition-strictest-wins.md).
