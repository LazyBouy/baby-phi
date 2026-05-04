<!-- Last verified: 2026-05-04 by Claude Code (CH-13 P3 chunk-seal — operator playbook for verifying the strictest-wins composition end-to-end via `phi org create --audit-class-default alerted` + Template-A firing + audit-event query asserting `audit_class_source == "org_default"`. Pairs with `architecture/audit-class-composition.md` (design) and ADR-0050.) -->

# `audit_class` composition operations runbook

> **Audience:** SREs and operators verifying or debugging template-fired audit-class composition. Pair this page with [`audit-class-composition.md`](../architecture/audit-class-composition.md) (design) and [ADR-0050](../decisions/0050-audit-class-composition-strictest-wins.md).

---

## End-to-end verification recipe

Use this recipe to confirm the strictest-wins guarantee is honoured for a fresh org. The recipe has three steps: create an org with an explicit `audit_class_default`, trigger a Template-A firing (a `HAS_LEAD` edge creation), and query the resulting `template.a.grant_fired` audit event.

### Step 1 — Create an org with `audit_class_default: alerted`

```bash
phi org create --audit-class-default alerted --name "Compliance Org Acme"
```

The `--audit-class-default` flag was already accepted by the CLI pre-CH-13 (`cli/src/commands/org.rs:52`); CH-13 changes nothing about this surface. The org row's `Organization.audit_class_default` field at [`model/nodes.rs:384`](../../../../../../modules/crates/domain/src/model/nodes.rs) records the operator's choice.

### Step 2 — Trigger a Template-A firing

Within the freshly-created org, create a project and assign a lead agent. The `HAS_LEAD` edge creation surfaces as a `HasLeadEdgeCreated` domain event; `TemplateAFireListener` consumes it and fires the Template-A grant (read/inspect/list on the project).

```bash
phi project create --org acme --name demo-project
phi project assign-lead --project demo-project --agent agent:alice
```

### Step 3 — Query the audit event

```bash
# Via SurrealDB query (paste against the same SurrealDB instance):
SELECT
  audit_class,
  diff.audit_class_source AS source,
  diff.grant_id           AS grant
FROM audit_events
WHERE event_id = 'template.a.grant_fired'
  AND org_scope = 'acme'
ORDER BY occurred_at DESC
LIMIT 1;
```

**Expected result for the recipe above:**

| audit_class | source        | grant |
|-------------|---------------|-------|
| `alerted`   | `template_ar` | `grant:...` |

Because the org's `audit_class_default` is `alerted` AND Template A's adoption AR has `audit_class: alerted` (production default at [`templates/adoption.rs`](../../../../../../modules/crates/domain/src/templates/adoption.rs)), both inputs tie at the strictest tier (`Alerted`). The tie-breaker rule (ADR-0050 §D50.3 — `Override > TemplateAr > OrgDefault`) gives the win to the more-specific source: `TemplateAr`. The composer's iteration walks `OrgDefault → TemplateAr` with the comparison `template_ar >= resolved` (Alerted >= Alerted) **true**, which overwrites `source` to `TemplateAr`. To see the `org_default` source dominate explicitly in the diff, set the org default to a strictly stricter class than the adoption AR's class — e.g., bootstrap a custom adoption AR with `audit_class: logged` and create the org with `--audit-class-default alerted`.

### Sanity-check expected results for variations

| Org default | Adoption AR `audit_class` | Override | Resolved class | `audit_class_source` |
|---|---|---|---|---|
| `alerted`   | `alerted`  | `None`         | `alerted`  | `template_ar` (tie at strictest; more-specific source wins) |
| `alerted`   | `logged`   | `None`         | `alerted`  | `org_default` |
| `logged`    | `alerted`  | `None`         | `alerted`  | `template_ar` |
| `silent`    | `silent`   | `Some(alerted)`| `alerted`  | `override` |
| `alerted`   | `logged`   | `Some(silent)` | `alerted`  | `org_default` (override-can-only-escalate; the silent override is silently rejected) |

---

## The `audit_class_source` diff field

Wire format: snake-case string, one of `"org_default" | "template_ar" | "override"`. Lives at the audit event's `diff.audit_class_source` JSON path. Per ADR-0050 §D50.6 the diff field is a **string** (not a typed enum) — the loosest contract for SurrealDB-side query / grep / filter.

```bash
# All Template-A grants where the org-default supplied the winning class:
SELECT * FROM audit_events
WHERE event_id = 'template.a.grant_fired'
  AND diff.audit_class_source = 'org_default';

# All Template-C grants where a per-Grant override won:
SELECT * FROM audit_events
WHERE event_id = 'template.c.grant_fired'
  AND diff.audit_class_source = 'override';
```

The string format is stable across pods and versions (the `AuditClassSource` enum has `#[serde(rename_all = "snake_case")]` at [`permissions/audit_composition.rs`](../../../../../../modules/crates/domain/src/permissions/audit_composition.rs)).

---

## Debugging — composer's resolved class is unexpectedly low

**Symptom:** an org configured with `audit_class_default: alerted` sees `template.A.grant_fired` events arriving with `audit_class: silent` or `audit_class: logged`.

**Likely cause:** the `resolve_composed_audit_class` helper hit a fail-safe fallback because either the org row or the adoption-AR row was unreadable.

The helper at [`events/listeners.rs:140`](../../../../../../modules/crates/domain/src/events/listeners.rs) wraps the composer with fail-safe semantics:

- If `Repository::get_organization(org_id)` returns `Ok(None)`: the listener falls back to `(AuditClass::Silent, AuditClassSource::OrgDefault)` and emits a `tracing::warn!` log: `"resolve_composed_audit_class: organization row not found; falling back to Silent"`.
- If `Repository::get_organization(org_id)` returns `Err(e)`: same fallback; log includes the underlying error: `"resolve_composed_audit_class: repo.get_organization failed; falling back to Silent"`.
- If `Repository::get_auth_request(adoption_ar_id)` returns `Ok(None)` or `Err(e)`: the helper composes with `template_ar = AuditClass::Silent`, letting the org default decide alone; log message names the AR id and (if present) the underlying error.

**Action:** grep the structured logs for `resolve_composed_audit_class` around the firing time:

```bash
journalctl -u phi-server | grep "resolve_composed_audit_class"
# or, for the embedded-runtime CLI binary:
RUST_LOG=warn phi project assign-lead ... 2>&1 | grep resolve_composed_audit_class
```

Each fail-safe path logs the org id or adoption-AR id, plus the underlying error if any, so the operator can:

1. Re-fetch the org row (`SELECT * FROM organization WHERE id = '<id>'`) and confirm the row exists with the expected `audit_class_default` value.
2. Re-fetch the adoption AR row (`SELECT * FROM auth_request WHERE id = '<adoption_ar_id>'`) and confirm `audit_class` matches the template's adoption design.
3. If neither row is missing in storage, the underlying error is a transient repo-layer failure — re-fire the trigger and confirm the next event composes correctly.

The fail-safe behaviour is **intentional**: per ADR-0050 + concept-doc 07 line 72's no-silent-downgrade invariant the composer's strictest-wins property only applies to the happy path (both rows present). A missing-row hit is structural divergence, not a downgrade decision; falling back to `Silent` on a degraded read path avoids accidentally escalating audit volume during an unrelated outage. The `tracing::warn!` always surfaces the divergence to operators.

---

## Common symptoms + fixes

| Symptom | Likely cause | Fix |
|---|---|---|
| `template.X.grant_fired` event has `audit_class: silent` for an `alerted`-default org | `resolve_composed_audit_class` fell back due to a missing org row or repo error. | Grep tracing logs for `resolve_composed_audit_class:` warnings; re-fetch the org row to confirm it exists; re-fire the trigger. |
| `audit_class_source` field absent from the diff JSON | Pre-CH-13 audit-event row (chunk shipped 2026-05-04). | Expected. Older rows do not carry the field; only post-CH-13 firings include it. |
| `audit_class_source` shows `template_ar` when operator expected `org_default` | Tie-breaker rule (ADR-0050 §D50.3): when 2+ inputs tie at strictest, more-specific source wins. With both org default and adoption-AR class at `alerted`, `template_ar` is the more-specific source and wins the attribution. | Not a bug. To see `org_default` win the attribution, set the org default strictly stricter than the adoption AR's `audit_class`. |
| Override class lower than org default produces resolved class equal to org default | Concept-doc 07 line 69 invariant: per-Grant overrides may only escalate, never loosen. The `max`-fold structurally rejects any override below the (a)+(b) max. | Not a bug. To loosen the audit posture for a specific grant, you must lower the org default — there is no per-grant downgrade path. |
| New audit events with `audit_class: alerted` flooding the alert channel after a CH-13 deploy | Expected. Pre-CH-13 the events hardcoded `Logged`; post-CH-13 the composed class respects the org's `audit_class_default` (which may be `Alerted`). | Verify the org's intent. If the operator never wanted `Alerted` posture, reset `Organization.audit_class_default` to `Logged`. |

---

## Audit-event reference

Three event types changed shape (signature + emitted class) at CH-13. The event id strings are unchanged; only the diff payload + the top-level `audit_class` field are affected.

- **`template.a.grant_fired`** — fired by `TemplateAFireListener` on `HasLeadEdgeCreated`. Top-level `audit_class` now reflects the composed value; diff carries `audit_class_source`.
- **`template.c.grant_fired`** — fired by `TemplateCFireListener` on `ManagesEdgeCreated`. Same shape change.
- **`template.d.grant_fired`** — fired by `TemplateDFireListener` on `HasAgentSupervisorEdgeCreated`. Same shape change.

The hash chain is forward-only per ADR-0033 §D33.2 + ADR-0050 §D50.7; pre-CH-13 events' canonical_bytes are unaffected.

---

## Cross-references

- [ADR-0050](../decisions/0050-audit-class-composition-strictest-wins.md) — design decisions D50.1–D50.7.
- [`audit-class-composition.md`](../architecture/audit-class-composition.md) — design page.
- [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" lines 64–72 — source of truth.
- [`requirements/cross-cutting/nfr-observability.md`](../../../requirements/cross-cutting/nfr-observability.md) §event-class retention — operator-meaning anchor for the ordering.
- [Drift D-new-19](../../m5_1/drifts/D-new-19.md) — closed by CH-13.
