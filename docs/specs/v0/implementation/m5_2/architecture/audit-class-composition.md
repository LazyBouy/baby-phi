<!-- Last verified: 2026-05-04 by Claude Code (CH-13 P3 chunk-seal — design page paired with ADR-0050 covering composer fn signature, ordering encoding, Grant denormalisation, listener wiring at TemplateA/C/D fire listeners, audit-event diff extension with `audit_class_source`, and cross-pod determinism per CH-13 plan §3.B A7.) -->

# `audit_class` composition (strictest wins) — design page

> **Status:** [EXISTS] as of CH-13 (M5.2). The pure-fn composer ships at [`modules/crates/domain/src/permissions/audit_composition.rs`](../../../../../../modules/crates/domain/src/permissions/audit_composition.rs); `Grant.audit_class` denormalisation lands at [`model/nodes.rs:693`](../../../../../../modules/crates/domain/src/model/nodes.rs); the 3 production fire listeners (Template A/C/D) at [`events/listeners.rs:303,433,552`](../../../../../../modules/crates/domain/src/events/listeners.rs) wire the composer via the `resolve_composed_audit_class` helper. For the normative concept-doc reference, read [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" lines 64–72.

---

## What this page covers

`AuditClass` (one of `Silent | Logged | Alerted`) tells the audit pipeline how a Grant — and the events fired against it — should be retained, surfaced, and routed. Concept-doc 07 specifies a **strictest-wins** composition rule whenever a template fires a Grant: the resolved class is the strictest of three candidate inputs, and operators must see which one supplied the winning value.

CH-13 lifts that rule from concept doc into typed Rust. This page describes:

- The composer fn signature + ordering encoding (Silent < Logged < Alerted).
- Grant denormalisation per ADR-0050 §D50.5.
- Listener wiring at the 3 production fire listeners.
- Fail-safe `resolve_composed_audit_class` helper semantics.
- The audit-event diff extension carrying `audit_class_source`.
- Cross-pod determinism argument from CH-13 plan §3.B A7.

ADR-0050 records the design decisions (sub-decisions D50.1–D50.7); this page is the operator-facing description.

---

## Composer fn signature + ordering encoding

```rust
// modules/crates/domain/src/permissions/audit_composition.rs

pub enum AuditClassSource {
    OrgDefault,
    TemplateAr,
    Override,
}

pub fn compose_audit_class(
    org_default: AuditClass,
    template_ar: AuditClass,
    r#override: Option<AuditClass>,
) -> AuditClass;

pub fn compose_audit_class_with_source(
    org_default: AuditClass,
    template_ar: AuditClass,
    r#override: Option<AuditClass>,
) -> (AuditClass, AuditClassSource);
```

The strictest-wins ordering is encoded via `derive(PartialOrd, Ord)` on [`AuditClass`](../../../../../../modules/crates/domain/src/audit/mod.rs) with declaration order `Silent → Logged → Alerted` (loosest → strictest). Rust auto-derives lexicographic ordering matching declaration order; no manual `cmp` impl. Concept-doc-07 line 68 phrases the ordering as `none < logged < alerted`; per ADR-0050 §D50.1 the concept-doc `none` term maps to enum `Silent` semantically (the variant doc-comment at `audit/mod.rs` describes Silent as "kept 30 days, no delivery" — the "no audit" semantics).

**Tie-breaker rule** (ADR-0050 §D50.3): when 2+ inputs tie at the strictest tier, more-specific source wins — `Override > TemplateAr > OrgDefault`. The natural `[a, b, c].max()` fold does NOT capture this (it returns the first hit at the strictest tier in iteration order); the implementation walks candidates in **least-specific to most-specific** order using `>=` so the more-specific source overwrites less-specific ties. This makes the `compose_audit_class_with_source` return value deterministic for the operator-facing diff field.

**Override-can-only-escalate** is structural: any `override < max(org_default, template_ar)` is silently rejected (the natural max wins); any `override ≥ max(org_default, template_ar)` becomes the result (concept-doc-07 line 69).

---

## Grant denormalisation per ADR-0050 §D50.5

```rust
// modules/crates/domain/src/model/nodes.rs:693
pub struct Grant {
    // ... pre-existing fields ...
    pub approval_mode: ApprovalMode,             // CH-11 / ADR-0048 D48.1 precedent
    #[serde(default = "Grant::default_audit_class")]
    pub audit_class: AuditClass,                 // CH-13 / ADR-0050 D50.5
    // ... pre-existing fields ...
}

impl Grant {
    pub fn default_audit_class() -> AuditClass {
        AuditClass::Silent
    }
}
```

The `#[serde(default = "Grant::default_audit_class")]` shield decodes pre-CH-13 grant rows as `AuditClass::Silent` (loosest). The `Silent` default preserves the concept-doc 07 line 72 invariant that adopting a template *can never silently downgrade* an org's audit posture — a `Silent` placeholder cannot mask a stricter org default the composer would otherwise apply.

This mirrors [ADR-0048](../decisions/0048-per-session-consent-gating.md) §D48.1 `approval_mode` precedent: the field rides under the existing storage `FLEXIBLE TYPE object` column in `migrations/0001_initial.surql`; **no schema migration**. Migration count stays at **13** (post-CH-12 baseline; CH-13 adds zero migrations).

---

## Listener wiring + the `resolve_composed_audit_class` helper

The 3 production grant-mint listeners in [`events/listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs) call a shared helper before constructing the `FireArgs` struct passed to the pure fire-fn:

| Listener | File:line | Trigger event | Fire pure-fn |
|---|---|---|---|
| `TemplateAFireListener` | `events/listeners.rs:303` | `HasLeadEdgeCreated` | `fire_grant_on_lead_assignment` |
| `TemplateCFireListener` | `events/listeners.rs:433` | `ManagesEdgeCreated` | `fire_grant_on_manages_edge` |
| `TemplateDFireListener` | `events/listeners.rs:552` | `HasAgentSupervisorEdgeCreated` | `fire_grant_on_has_agent_supervisor` |

Each listener body:

1. Looks up the firing org's `Organization.audit_class_default` via `Repository::get_organization`.
2. Looks up the template's adoption AR via `Repository::get_auth_request`; reads `AuthRequest.audit_class`.
3. Calls `compose_audit_class_with_source(org_default, template_ar, None)` to fold the inputs (Templates A/C/D do not currently supply a per-Grant override; reserved for forward-scope templates).
4. Stamps the resolved `(class, source)` pair onto:
   - `Grant.audit_class` (the field added at P1).
   - The `template.X.grant_fired` audit event's top-level `audit_class` field.
   - The audit-event diff's `audit_class_source` snake-case string per D50.6.

The shared helper `resolve_composed_audit_class(repo, org, adoption_ar_id)` (at `events/listeners.rs:140–194`) wraps steps 1–3 with **fail-safe semantics**:

- **Org row not found** → log at `tracing::warn!`, fall back to `(AuditClass::Silent, AuditClassSource::OrgDefault)`.
- **`get_organization` repo error** → log at `tracing::warn!`, fall back to `(AuditClass::Silent, AuditClassSource::OrgDefault)`.
- **Adoption-AR row not found** → log at `tracing::warn!`, treat the `template_ar` candidate as `AuditClass::Silent` so the org default decides alone.
- **`get_auth_request` repo error** → same fallback as adoption-AR-not-found.

The fail-safe behaviour is intentional. Concept-doc 07 line 71's no-silent-downgrade invariant only applies to the **happy path** (both rows present); a missing-row hit is **structural divergence**, not a downgrade decision. Falling back to `Silent` on a degraded read path avoids accidentally escalating audit volume during an unrelated outage. The `tracing::warn!` log always names the row and the underlying error so operators can trace the divergence; see [`audit-class-composition-operations.md`](../operations/audit-class-composition-operations.md) for the playbook.

---

## Audit-event diff extension with `audit_class_source`

Per ADR-0050 §D50.6, the 3 audit-event builders gain `audit_class: AuditClass` + `audit_class_source: AuditClassSource` parameters:

| Builder | File:line | Event id |
|---|---|---|
| `template_a_grant_fired` | `audit/events/m4/templates.rs:30` | `template.a.grant_fired` |
| `template_c_grant_fired` | `audit/events/m5/templates.rs:29` | `template.c.grant_fired` |
| `template_d_grant_fired` | `audit/events/m5/templates.rs:77` | `template.d.grant_fired` |

The `diff` JSON object gains a `"audit_class_source": <snake_case-string>` key. Wire format is `"org_default"` | `"template_ar"` | `"override"` (the snake-case serialisation of `AuditClassSource`).

This honours concept-doc-07 line 70's "operators can always see what was applied" clause — the diff lets a SurrealDB reader query "which Grants got `Alerted` because of org default vs template AR vs explicit override?" without joining back to the Grant or to the Organization row.

**String, not typed enum.** ADR-0050 §D50.6 picks a string for the diff payload — the loosest contract. Operators consuming audit events via SurrealDB `SELECT diff FROM audit_events` get a stable string they can grep/filter on. A richer typed `AuditClassProvenance` enum is a forward-scope item if operator-facing tooling needs to query it programmatically.

The pre-CH-13 `template.X.grant_fired` events hardcoded `AuditClass::Logged`. Post-CH-13, the event's top-level `audit_class` and the diff's `audit_class_source` together capture the composed class plus the operator-readable attribution.

---

## Cross-pod determinism (CH-13 plan §3.B A7)

`AuditEvent::canonical_bytes()` at [`audit/mod.rs:72`](../../../../../../modules/crates/domain/src/audit/mod.rs) DOES include `audit_class` (line 93) and `diff` (line 90) in its hash input — meaning post-CH-13 audit events have **different canonical_bytes** than pre-CH-13 ones for the same logical firing.

The hash chain is **forward-only** within an `org_scope`. Pre-existing events' bytes are unaffected; new events at any pod simply produce new chain links with the composed class. The single-writer guarantee per ADR-0033 §D33.2 (`SurrealStore::open_remote`) is preserved because:

- Every pod sees the same durable `Organization.audit_class_default` reading.
- Every pod sees the same durable `AuthRequest.audit_class` reading.
- Pure-fn `compose_audit_class_with_source(...)` is deterministic.
- Therefore every pod produces the same `(audit_class, audit_class_source)` pair for the same firing.

K8s-neutral — no new in-process state, no new IPC channel, no new pod-local resource, no migration runner / first-apply race, no new trait shape, hash-chain symmetry preserved per A7. See CH-13 plan §3.B for the full readiness check.

---

## What this page does NOT cover

- **Template E `BuildArgs::audit_class`** — Template E (the self-interested-auto-approve path used at platform-admin / page-02–05 writes) supplies its `audit_class` at construction time; composing on top would require the caller to know the org's default, which is two layers above current callers. Out of scope; would require a separate forward-scope row.
- **Platform-admin grants** (~6 production sites that mint Grants directly without going through Templates A/C/D) — CH-13 leaves them at `AuditClass::Silent` placeholder; future chunks may refine.
- **Richer typed `AuditClassProvenance` enum** — the audit-event diff carries the snake-case string per D50.6; a typed enum is a forward-scope item.

---

## Cross-references

- [ADR-0050](../decisions/0050-audit-class-composition-strictest-wins.md) — the design decisions captured as sub-decisions D50.1–D50.7.
- [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" lines 64–72 — source of truth.
- [ADR-0048 — Per-session consent gating](../decisions/0048-per-session-consent-gating.md) §D48.1 — Grant-denormalisation precedent CH-13 mirrors for `audit_class`.
- [`audit-class-composition-operations.md`](../operations/audit-class-composition-operations.md) — operator playbook.
- [`requirements/cross-cutting/nfr-observability.md`](../../../requirements/cross-cutting/nfr-observability.md) §event-class retention — operator-meaning anchor for the ordering.
- [Drift D-new-19](../../m5_1/drifts/D-new-19.md) — closed by CH-13.
