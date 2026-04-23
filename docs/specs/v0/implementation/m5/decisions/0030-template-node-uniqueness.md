<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0030 — Template-node uniqueness

**Status: Accepted** — flipped at M5/P1 close after migration 0005
REMOVE INDEX `template_name` + DEFINE INDEX `template_kind` shipped
and the `template_kind_unique_index_permits_multi_org_adoption`
integration test passed green.

## Context

M3/P4 closed without persisting `Template { id, name, kind,
created_at }` nodes. Root cause: migration
[`0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql)
defines `template.name` as a UNIQUE INDEX. Two orgs both adopting
Template A would both try to persist `name = "template:a"`,
violating the index + blowing up the compound transaction.

M3/P4 worked around this by **not** persisting the node, carrying
adoption state only via the AR resource URI
(`org:<id>/template:<kind>`). This dangles — at M5 the
first-session path's grant-fire flow walks `provenance_template`
back to a Template row that does not exist.

Three options considered at M5/P0 planning:

- **(a) Rename per org** — `name = "template:<kind>:org:<id>"`, keep
  UNIQUE INDEX on name. Adoption IS the node. Downside: duplicates
  the Template pattern definition per org; template-kind queries
  ("all orgs that adopted Template A") become string-parsing.
- **(b) One shared row per kind** — UNIQUE INDEX on `kind`, Template
  is a platform-level pattern, adoption is recorded on the AR's
  `provenance_template`. Template rows are written at platform
  bootstrap / migration time; adoption does NOT write new Template
  rows.
- **(c) Drop UNIQUE entirely** — multiple rows per `(kind, org)`,
  with a composite index if needed. Loses the "template is a
  reusable pattern" semantic.

## Decision

**Option (b) — one shared row per kind with UNIQUE(kind).**
User-confirmed at M5/P0 planning (plan decision D1).

### D30.1 — Migration 0005 DROPs name index, adds kind index

`modules/crates/store/migrations/0005_sessions_templates_system_agents.surql`:

```sql
REMOVE INDEX template_name ON template;
DEFINE INDEX template_kind ON template FIELDS kind UNIQUE;
```

`template.name` remains a non-unique field for display purposes
(e.g. the CLI prints `"Template A"` on adoption). The uniqueness
invariant is on `kind` (one of `A | B | C | D | E`).

### D30.2 — Template rows written at platform bootstrap, not at adoption

Platform bootstrap (M1/P0 + M5/P1 seed extension) writes the **5
Template rows** (A/B/C/D/E) once per fresh DB at migration time.
Adoption does not create Template rows; adoption creates an
`AuthRequest` whose `provenance_template: Option<TemplateKind>`
carries the link back.

This inverts the M3/P4 provisional mental model ("Template row
created on adoption") but aligns with the concept doc's
"template is a reusable pattern, adoption is the per-org act"
framing.

### D30.3 — `Template` struct stays platform-level

`domain/src/model/nodes.rs::Template` does NOT gain `adopted_by_org`
/ `adopted_at` fields. Adoption lives on the AR, not on the
Template. Walking the adoption history = walking AR history via
`provenance_template`.

```rust
pub struct Template {
    pub id: TemplateId,
    pub name: String,
    pub kind: TemplateKind, // A | B | C | D | E
    pub created_at: DateTime<Utc>,
}
```

### D30.4 — Grant provenance chain traverses via AR, not Template

When a grant fires via Template C/D listener (M5/P8) + M4's
Template A listener, the grant's `DESCENDS_FROM` edge points at the
**AR** (the adoption act), not at the Template row. The Template
row is one level further up the chain:

```
Grant --DESCENDS_FROM--> AR (adoption) --provenance_template--> Template
```

Revoke-cascade (M5/P5 page 12) walks AR → grants with
`DESCENDS_FROM edge`, forward-only-revokes each, logs
`AuthorityTemplateRevoked { grant_count_revoked, template_kind }`.

## Consequences

**Positive**
- Template semantic matches the concept doc.
- Multi-org adoption becomes trivial (no more duplicate-key error).
- Platform-level Template rows seeded once at migration time — no
  race conditions between org-create paths.
- Adoption remains auditable via the AR trail without duplicating
  data on the Template row.

**Negative**
- Data model inversion vs M3/P4's provisional assumption. Nothing
  in production depends on the old model (Template rows were never
  written), but reviewers reading M3/P4 docs first then M5/P1 might
  be briefly confused. Mitigated by the M5 architecture doc calling
  out the inversion explicitly.
- Revoking Template A for Org X must NOT affect Org Y's adoption of
  Template A. The revoke scope is the AR (per-org), not the
  Template (shared). Acceptance tests at P5 pin this
  (`revoke_template_a_in_org_x_does_not_affect_org_y`).

**Neutral**
- Migration 0005 is the FIRST migration to `REMOVE INDEX` — sets
  the pattern for future schema inversions. The roll-forward-only
  discipline holds: fresh DBs boot with 0005 applied, the
  `template_name` index never existed on such DBs; existing dev
  DBs need to apply 0005 (migration test pins idempotency).

## References

- [M5 plan archive §D1 + §G1 + §P1](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [Base plan §M5 carryovers §C-M5-1](../../../../plan/build/36d0c6c5-build-plan-v01.md).
- [M3 architecture — org-creation Template persistence deferral](../../m3/architecture/org-creation.md).
- [Authority templates architecture](../architecture/authority-templates.md) — fire rules + revoke-cascade semantics (seeded at P0, filled at P5).
