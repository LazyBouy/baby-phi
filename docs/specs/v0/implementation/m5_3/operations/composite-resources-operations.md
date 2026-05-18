<!-- Last verified: 2026-05-17 by Claude Code (CH-26 / ADR-0061 P-DOCS — operations page paired with the composite-resources-model architecture page; documents how operators inspect the `tags: Vec<String>` field, verify migration `0018_org_project_tags.surql` ran cleanly, read the advisory-only handler `check_permission` results in logs, and what changes when CH-27 tightens advisory → blocking. Cycle hex `d1cb9e1f`.) -->

# Composite resources (Org/Project) — operations page

> **Status:** [EXISTS] as of CH-26 (M5.3). For the architectural design read [`composite-resources-model.md`](../architecture/composite-resources-model.md); for the originating decision read [ADR-0061](../decisions/0061-org-project-as-composite-resources.md).

---

## §1 — Inspecting the `tags` field on Organization + Project rows

CH-26 adds `tags: Vec<String>` to the Organization + Project structs. After CH-26 ships, every Org / Project row carries at least one tag — its instance-identity tag (`organization:<uuid>` or `project:<uuid>`).

### SurrealQL

```surql
-- Tags on a specific Org
SELECT id, display_name, tags FROM organization
  WHERE id = type::thing('organization', '<uuid>');

-- All Orgs missing instance-identity tag (should return 0 rows post-migration)
SELECT id, display_name, tags FROM organization
  WHERE NOT (tags ?? []) CONTAINS string::concat('organization:', string::from(id));

-- Similarly for projects
SELECT id, display_name, tags FROM project
  WHERE NOT (tags ?? []) CONTAINS string::concat('project:', string::from(id));
```

If the third / fourth query returns rows, migration `0018_org_project_tags.surql` did not run cleanly. Investigation steps in §3 below.

### In-memory backend

The `domain::in_memory` backend deserialises Organization / Project rows with `tags` via `#[serde(default)]` — pre-CH-26 fixtures load with `tags = vec![]`. Tests that exercise the instance-identity tag explicitly seed the `tags` field on the fixture (see `acceptance_common::admin::spawn_claimed_with_org_and_project` for the canonical pattern).

---

## §2 — Migration `0018_org_project_tags.surql`

### What it does

Three operations in one transaction (see ADR-0061 §D61.3 for the SurrealQL bodies):

1. Column add: `DEFINE FIELD tags ON TABLE organization TYPE array<string> DEFAULT [];` + same for project.
2. Instance-identity backfill: `UPDATE organization SET tags = ['organization:' + string::from(id)] WHERE tags IS NONE OR tags = [];` + same for project.
3. Catalogue-entry backfill: ensure a catalogue entry exists for every extant Org / Project row (INSERT-OR-IGNORE semantics).

### Verifying it ran

```bash
# 1. Migration is registered
grep -n '0018_org_project_tags' /root/projects/phi/baby-phi/modules/crates/store/src/migrations.rs

# 2. Migration ran (check the migrations table)
# (run inside a SurrealDB shell against the live database)
SELECT * FROM migrations WHERE slug = 'org_project_tags';
# Expect 1 row with the applied_at timestamp populated.

# 3. Idempotency check: re-run the migration body manually; UPDATE should affect 0 rows
UPDATE organization SET tags = ['organization:' + string::from(id)] WHERE tags IS NONE OR tags = [];
# Expect 0 rows updated on second run.
```

### Rollback notes

Per [ADR-0012 forward-only-migrations](../../m1/decisions/0012-forward-only-migrations.md): there is no rollback migration. If you need to remove the tags column post-migration, write a new forward migration `0019_*` that does `REMOVE FIELD tags ON TABLE organization;` + same for project. This will also break the production code that now reads `Organization.tags` / `Project.tags` — coordinate with a CH that removes those reads first.

For high-row-count environments where the UPDATE step is slow, consider batched UPDATE in a NEW migration that supersedes 0018 (e.g., `WHERE id IN (SELECT id FROM organization LIMIT 1000 WHERE tags = [])` in a loop). Plan §3.E Candidate 2 anticipates this. The current 0018 is sized for development + small-production deployments.

---

## §3 — Handler-tier `check_permission` invocations (advisory-only at CH-26)

CH-26 wires `handler_support::check_permission` into ≥ 7 handlers across `server/src/platform/orgs/` + `server/src/platform/projects/`. At CH-26 close these invocations are **advisory-only** — the engine computes a verdict, the result is consumed by the handler (typically captured into a `tracing::debug!` or local variable), but the bespoke `AuthenticatedSession` / role-check / AR-filter gates remain the HTTP-tier rejection surface.

### Reading the advisory verdict in logs

Each refactored handler emits a tracing span event when `check_permission` is invoked. To filter on the engine verdict:

```bash
# In production logs, filter on the handler's tracing target
RUST_LOG=server::platform::orgs::show=debug,server::platform::projects::create=debug \
  cargo run -p server -- --config dev
```

Look for `permission_check_advisory_verdict={Allowed|Denied(...)}` annotations in the spans. A `Denied` verdict at this layer is informational — the bespoke gate decides response status. CH-27 will tighten this so the engine `Denied` directly causes HTTP 403 / 404.

### What CH-27 will change

Per drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md):

- Advisory → blocking: engine `Denied` will map via `denial_to_api_error` to a structured HTTP 403 with `NO_GRANTS_HELD` / `NO_APPLICABLE_GRANT` / `EXPLICIT_DENY` codes (per CH-25 wire convention from `acceptance_m5_3_owner_grant`).
- CH-25 synth-owner-grant rule widens to cover `Action::Observe` + `Action::Inspect` for owner-Agents (current scope: `[Allocate, Transfer]`).
- M3 + M4 + M5 acceptance fixtures gain Edge::Owns seeding where the new blocking gate would otherwise break tests.

When deploying CH-27, expect a transient spike in 403 responses if any non-owner Agents (or Agents with stale ownership) were hitting the affected endpoints under the advisory regime. Run the migration-style smoke check: list the most-recent successful Org/Project actions per Agent over a 24h window and verify they have the expected Edge::Owns relationships.

---

## §4 — Blast-radius operator guide

If an Org / Project endpoint starts returning unexpected 403s after CH-27 deploys, walk this checklist:

1. **Does the actor have an Edge::Owns edge?**
   ```surql
   SELECT * FROM owns WHERE in = type::thing('agents', '<agent_id>');
   ```
   If yes, the synth-owner-grant should resolve Allow.

2. **Is the action covered by the synth-grant scope?**
   - At CH-26: `[Allocate, Transfer]` only.
   - At CH-27: `[Allocate, Transfer, Observe, Inspect]`.

3. **Does the target Org / Project have the expected instance-identity tag?**
   ```surql
   SELECT id, tags FROM organization WHERE id = type::thing('organization', '<uuid>');
   ```
   The `tags` array should contain `organization:<uuid>`.

4. **Is the catalogue entry present?**
   ```surql
   SELECT * FROM catalogue_entry WHERE uri = 'org:<uuid>';
   ```
   At least one row with the expected `composite_class = "organization_object"`.

5. **For non-owner Agents who SHOULD have access** — they need an explicit Grant. Check:
   ```surql
   SELECT * FROM grant WHERE holder = type::thing('agents', '<agent_id>')
     AND target_uri = 'org:<uuid>';
   ```
   If empty, the actor truly has no authority over the target.

If steps 1-4 are all clean but the endpoint still 403s, capture the request + response + Permission Check decision-log entry (the engine emits one per check) and file a follow-up drift.

---

## §5 — Cross-references

- [`composite-resources-model.md`](../architecture/composite-resources-model.md) — architecture page.
- [`ADR-0061`](../decisions/0061-org-project-as-composite-resources.md) — formal design decision.
- [`agent-ownership-operations.md`](agent-ownership-operations.md) — CH-25 operations page; describes the upstream Edge::Owns + synth-owner-grant mechanics.
- [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) — CH-27 carve-out tracking.
- Plan archive: [`plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md`](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md).
