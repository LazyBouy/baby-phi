<!-- Last verified: 2026-05-04 by Claude Code (CH-13 P3 chunk-seal — ADR flipped Proposed → Accepted; sub-decisions D50.1–D50.7 pinned by P1 + P2 deliverables: composer fn at `domain::permissions::audit_composition::{compose_audit_class, compose_audit_class_with_source, AuditClassSource}`, `Grant.audit_class` field at `model/nodes.rs:693`, 3 fire listeners (Template A/C/D) wire `resolve_composed_audit_class` at `events/listeners.rs:303,433,552`, audit-event builders at `audit/events/m4/templates.rs:30` + `m5/templates.rs:29,77` accept `audit_class: AuditClass` + `audit_class_source: AuditClassSource` parameters and emit `audit_class_source` snake-case string in diff per D50.6.) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-13 P1 — ADR drafted as Proposed) -->

# ADR-0050 — `audit_class` strictest-wins composition (composer fn + Grant denormalisation + listener wiring)

**Status: Accepted**

**Date:** 2026-05-04
**Chunk:** CH-13
**Closes:**
- [`D-new-19`](../../m5_1/drifts/D-new-19.md) (MEDIUM) — `audit_class` composition rule silent-in-code. CH-13 closes the composer-fn half (`compose_audit_class` + strictest-wins ordering), the Grant-denormalisation half (`Grant.audit_class` field + serde-default shielding), and the listener-wiring half ships at P2 (Template A/C/D fire listeners feed (org_default, template_ar, optional_override) into the composer and stamp the resolved class on Grant + audit-event diff).

---

## Context

[`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" lines 64–72 specifies a **strictest-wins** composition rule: when a template fires a Grant, the resolved `audit_class` is the strictest of (a) the **org's `audit_class_default`**, (b) the **template adoption Auth Request's `audit_class`**, and (c) any **per-Grant override** the template specifies. Per-Grant overrides may **only escalate, never loosen** — an org with `audit_class_default: alerted` for compliance reasons must never see a template silently downgrade its audit posture.

Today the rule is **silent in code** (drift D-new-19, MEDIUM, `discovered`). Templates A/C/D's `fire_grant_*` pure-fns mint Grants with **no `audit_class` field at all**. The companion audit events (`template.a.grant_fired` at [`audit/events/m4/templates.rs:30`](../../../../../../modules/crates/domain/src/audit/events/m4/templates.rs), `template.c.grant_fired` + `template.d.grant_fired` at [`m5/templates.rs:29,77`](../../../../../../modules/crates/domain/src/audit/events/m5/templates.rs)) hardcode `AuditClass::Logged` regardless of the org's `audit_class_default`. So an org configured as `alerted` for compliance reasons sees its template-fired grant audits routed at `Logged` — silently downgraded, contradicting concept-doc 07 line 72 ("an org that opts into `alerted` for compliance reasons is guaranteed that adopting a template can never silently downgrade its audit posture").

CH-13 closes drift D-new-19 by shipping a pure-fn composer + denormalising the resolved class onto Grant per concept-doc 07 line 70 verbatim ("recorded on the Grant"), and wires the composer into all 3 production grant-mint paths (Template A/C/D fire listeners) at P2.

### Forks (all planner-recommended at chunk-open; user-locked at plan approval to F1.A / F2.A / F3.A)

- **F1.A — Map `Silent ↔ none`.** Strictest-wins ordering is `Silent < Logged < Alerted`. Encoded via `derive(PartialOrd, Ord)` with the existing variants declared in loosest-to-strictest order. No new `AuditClass` variant; concept-doc 07 line 68's `none` term maps to enum `Silent` semantically (the doc-comment at [`audit/mod.rs:30–33`](../../../../../../modules/crates/domain/src/audit/mod.rs) describes Silent as "kept 30 days, no delivery" — the "no audit" semantics).
- **F2.A — Add `audit_class: AuditClass` field to `Grant`.** Mirrors the CH-11 ADR-0048 D48.1 precedent of adding `approval_mode: ApprovalMode` directly to Grant. `#[serde(default)]` shields pre-CH-13 grants (decode as `AuditClass::Silent`, the loosest — preserves "no escalation by silent migration" invariant per concept-doc 07 line 72).
- **F3.A — `compose_audit_class(org_default: AuditClass, template_ar: AuditClass, override: Option<AuditClass>) -> AuditClass`.** Override is `Option`-typed; absent override means "use strictest of (a)+(b)". Templates A/C/D pass `None` for override (current absence is honoured).

---

## Decision

### D50.1 — Variant-naming alignment

Enum `AuditClass::Silent` ≡ concept-doc-07 line-68 `none`. The drift D-new-19 sketch's "Elevated" term is acknowledged-stale; concept doc 07 is canonical. The Rust enum keeps its existing variants `{Silent, Logged, Alerted}` at [`audit/mod.rs:48–52`](../../../../../../modules/crates/domain/src/audit/mod.rs); CH-13 adds a doc-comment cross-referencing the concept-doc mapping rather than renaming variants (which would cascade across 12+ existing emitters and potentially require a SurrealDB enum-value migration).

### D50.2 — Ordering encoding

`derive(PartialOrd, Ord)` with declaration order `Silent → Logged → Alerted` (already at [`audit/mod.rs:48–52`](../../../../../../modules/crates/domain/src/audit/mod.rs)). Rust auto-derives lexicographic ordering matching declaration order; `Silent < Logged < Alerted` follows directly. No manual `cmp` impl. The composer fn's `max`-fold uses this ordering.

### D50.3 — Composer fn signature + tie-breaker rule

Signature: `pub fn compose_audit_class(org_default: AuditClass, template_ar: AuditClass, r#override: Option<AuditClass>) -> AuditClass` (per F3.A locked-recommendation).

Body: a strictest-wins fold over the candidate inputs. Companion fn `pub fn compose_audit_class_with_source(...) -> (AuditClass, AuditClassSource)` returns the (resolved, winning-source) tuple per concept-doc-07 line 70's "along with which of (a)/(b)/(c) supplied the winning value".

**Tie-breaker rule** when 2+ inputs tie at the strictest: more-specific source wins — `Override > TemplateAr > OrgDefault`. The natural `[a, b, c].max()` fold does not capture this (it returns the first hit at the strictest tier in iteration order); the implementation walks candidates in **least-specific to most-specific** order using `>=` so the more-specific source overwrites less-specific ties. Concept-doc-07 line 70's "the winning value" term is interpreted via this tie-breaker: the most-specific input that achieves the winning class is the source.

### D50.4 — Override-can-only-escalate property

Structural via the strictest-wins fold: any `override < max(org_default, template_ar)` is silently rejected (the natural max wins); any `override ≥ max(org_default, template_ar)` becomes the result. An explicit unit test pins concept-doc-07 line-69 invariant ("Per-Grant overrides may only go stricter").

### D50.5 — Grant denormalisation

Add `audit_class: AuditClass` field to `Grant` at [`model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) adjacent to `approval_mode` (precedent-mirroring placement). `#[serde(default = "Grant::default_audit_class")]` shielding produces `AuditClass::Silent` (loosest) for pre-CH-13 grant rows. The `Silent` default preserves the concept-doc 07 line 72 invariant that adopting a template *can never silently downgrade* an org's audit posture — a `Silent` placeholder cannot mask a stricter org default the composer would otherwise apply.

Mirrors [ADR-0048](./0048-per-session-consent-gating.md) §D48.1 `approval_mode` precedent: the field rides under the existing storage `FLEXIBLE TYPE object` column in `migrations/0001_initial.surql`; no schema migration. The store crate's `GrantRow` translator at [`repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) gains a parallel `audit_class` column with the same `#[serde(default = ...)]` shield.

### D50.6 — Audit-event diff extension (P2 deliverable; signature pinned at P1)

The 3 audit-event builders (`template_a_grant_fired`, `template_c_grant_fired`, `template_d_grant_fired`) gain `audit_class: AuditClass` + `audit_class_source: AuditClassSource` parameters at P2. Their `diff` JSON object gains a `"audit_class_source": <snake_case-string>` key (`"org_default" | "template_ar" | "override"`).

String (not typed enum) is the loosest contract for the diff payload — operators consuming audit events via SurrealDB `SELECT diff FROM audit_events` get a stable string they can grep/filter on. A richer typed `AuditClassProvenance` enum is a forward-scope item if operator-facing tooling needs to query it programmatically.

The new `AuditClassSource` enum (3 variants `OrgDefault | TemplateAr | Override`) ships at [`permissions/audit_composition.rs`](../../../../../../modules/crates/domain/src/permissions/audit_composition.rs) with `serde(rename_all = "snake_case")`; the audit-event builder serialises it to the diff via `serde_json::to_value`.

### D50.7 — Hash-chain integrity

A7 review per CH-13 plan §3.B confirms canonical_bytes change is forward-only and cross-pod deterministic; no migration needed. `AuditEvent::canonical_bytes()` at [`audit/mod.rs:84`](../../../../../../modules/crates/domain/src/audit/mod.rs) DOES include `audit_class` (line 70) and `diff` (line 69) — meaning post-CH-13 audit events have different `canonical_bytes` than pre-CH-13 ones for the same logical firing. **However**: the hash chain is **forward-only** within an `org_scope` (line 77 `prev_event_hash` doc); pre-existing events' bytes are unaffected; new events at any pod simply produce new chain links with the composed class.

Cross-pod: every pod sees the same durable `Organization.audit_class_default` + `AuthRequest.audit_class` reading the composer's inputs → every pod produces the same composed class for the same firing. Single-writer guarantee preserved.

---

## Phase placement

- **P1** — Composer fn + AuditClass Ord derive + Grant.audit_class field + ADR Proposed.
- **P2** — 3-listener wiring (TemplateAFireListener / TemplateCFireListener / TemplateDFireListener) + audit-event-builder signature changes (3 builders gain `audit_class` + `audit_class_source` params) + 7 listener-level integration tests + 3 updated audit-event-builder unit tests.
- **P3** — ADR Accept-flip + concept-audit matrix row → `honored` + drift D-new-19 lifecycle-history → `remediated` + verified-headers refreshed + new architecture doc + new operations doc + user-guide note.

---

## Consequences

**Positive:**
- Concept-doc 07 lines 64–72 honored end-to-end (composer + Grant denormalisation + listener wiring + audit-event provenance).
- Drift D-new-19 closed (silent-in-code → honored).
- Compliance-posture invariant pinned by structural property (`max`-fold) + explicit unit + integration tests.
- No phi-core leverage delta (composer is baby-phi-native; phi-core has no governance/audit-composition concept per `phi-core-mapping.md`).
- No SurrealDB migration; no K8s blockers (§3.B verdict K8s-neutral).

**Negative / accepted costs:**
- +1 Grant struct field (`audit_class`) cascades to ~30 literal-construction sites (3 fire-fn productions + 6 platform-admin Grants + ~20 test fixtures + GrantRow translator). Path-A mechanical cascade accepted at P1; orchestrator confirmed enumeration. The 6 platform-admin sites + ~20 test fixtures all default to `AuditClass::Silent` (loosest); production composition is template-specific only.
- Audit-event canonical_bytes change for `template.X.grant_fired` events — accepted as forward-only per D50.7.
- Tie-breaker rule (D50.3) is implementation-only (concept-doc 07 line 70 says "which of (a)/(b)/(c) supplied the winning value" without specifying a tie-breaker; CH-13 picks the most-natural intent-based ordering Override > TemplateAr > OrgDefault and pins it via unit test).

**Out of scope (future-chunk concerns):**
- Template E `BuildArgs::audit_class` integration. Template E is the self-interested-auto-approve path used at platform-admin / page-02–05 writes; its caller already supplies the audit class at construction time. Composing on top would require the caller to know the org's default, which is two layers above current callers.
- Richer typed `AuditClassProvenance` enum for audit-event diff (vs. the snake-case string per D50.6).
- Platform-admin grants (6 production sites) audit-class strategy. CH-13 leaves them at `Silent` placeholder; future chunks may refine.

---

## Cross-references

- [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" lines 64–72 — source of truth.
- [`concepts/permissions/README.md`](../../../concepts/permissions/README.md) — Permissions subtree invariants (re-verified honored post-CH-13).
- [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) — confirms phi-core has no governance/audit-composition concept.
- [`requirements/cross-cutting/nfr-observability.md`](../../../requirements/cross-cutting/nfr-observability.md) §event-class retention — operator-meaning anchor for the ordering.
- [ADR-0046 — Consent-node full shape](./0046-template-cd-http-edges.md) — Organization.audit_class_default field origin.
- [ADR-0048 — Per-session consent gating](./0048-per-session-consent-gating.md) §D48.1 — Grant-denormalisation precedent CH-13 mirrors for `audit_class`.
- [ADR-0049 — Frozen session-tag immutability](./0049-frozen-session-tag-immutability.md) — most recent ADR; shape reference for chunk-spanning ADRs with sub-decisions.
- [Drift D-new-19](../../m5_1/drifts/D-new-19.md) — closes here.
- [Forward-scope row CH-13](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (lines 130–135) — chunk source.
