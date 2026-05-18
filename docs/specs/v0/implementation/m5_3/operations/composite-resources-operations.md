<!-- Last verified: 2026-05-18 by Claude Code (CH-27 / ADR-0062 P-DOCS — operations page updated for CH-27 closure: §3 now describes blocking-gate semantics (engine deny → HTTP 403 NO_GRANTS_HELD via `denial_to_api_error`); §4 blast-radius checklist amended for the widened 4-verb synth-grant scope + new F4.b helper-fixture pattern reference; cycle hex `0edcaba9`.) -->
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

## §3 — Handler-tier `check_permission` invocations (blocking at CH-27)

CH-26 wired `handler_support::check_permission` into **7 handlers** across `server/src/platform/orgs/` + `server/src/platform/projects/`. **At CH-27** (cycle hex `0edcaba9`, per ADR-0062 §D62.1), these invocations are **blocking** — the engine verdict directly drives HTTP response status:

- **Allowed** → handler proceeds to the bespoke gate (defence-in-depth) + the success path.
- **Denied** → handler propagates `OrgError::PermissionDenied` (or `ProjectError::PermissionDenied`, or `SetSupervisorError::PermissionDenied`) via `?`, which the route layer maps to **HTTP 403 `NO_GRANTS_HELD`** per CH-25 wire convention (`denial_to_api_error` at `handler_support/permission.rs:76`).
- **Pending** (consent gate) → handler propagates as HTTP 202 `AWAITING_CONSENT`.

### Reading the blocking verdict in production

Engine denials at the 7 admin handlers now surface as user-visible HTTP 403 responses. Operators see:

```json
{
  "code": "NO_GRANTS_HELD",
  "message": "no grant covers `IdentityPrincipal` for action `inspect`"
}
```

(The message varies per `denial_to_api_error` mapping — see `permission.rs:76+` for the full code→status→message table.)

### What CH-27 changed (closes D-CH26-FOLLOWUP-01)

Per [ADR-0062](../decisions/0062-blocking-gate-and-synth-grant-widening.md):

- **§D62.1 — Wire-tier tightening**: 7 advisory `.is_ok()` sites flipped to blocking `?`-propagation via `denial_to_api_error`. Engine-deny → HTTP 403 `NO_GRANTS_HELD`.
- **§D62.2 — Synth-grant scope widening**: CH-25 synth-owner-grant rule now emits `[Allocate, Transfer, Observe, Inspect]` (4 universal-applicability verbs) for owner-Agents on owned Org/Project.
- **§D62.3 — Resolvers wiring deferred** (F3.a, M6-DEFERRED): `projects::resolvers::*` actor-passthrough architectural design moved to [`D-CH27-FOLLOWUP-01`](../drifts/D-CH27-FOLLOWUP-01.md) for M6 plan-open.
- **§D62.4 — F4.b USER-DIVERGENT fixture pattern**: NEW [`seed_owner_grants`](../../../../../../modules/crates/server/tests/acceptance_common/owner_grants.rs) helper at `server/tests/acceptance_common/owner_grants.rs`. **9 explicit call-sites** across 6 M3+M4+M5 acceptance test files updated with explicit per-test owner-grant seeding (planning band per plan §3 Artifact C was 12-18; cascade-collapse documented in ADR-0062 §D62.4 — tests using the `apply_org_creation` production path obtain `Edge::Owns` implicitly per CH-25 ADR-0060 §D60.1).
- **§D62.5 — Count amendment META**: D-CH26-FOLLOWUP-01 body cardinality corrected "15" → "7" advisory `check_permission` invocations.

### Operator deployment notes (CH-27 deploy)

When deploying CH-27, expect a transient spike in 403 `NO_GRANTS_HELD` responses if any non-owner Agents (or Agents with stale ownership) were hitting the affected endpoints under the advisory regime. Run the migration-style smoke check: list the most-recent successful Org/Project actions per Agent over a 24h window and verify they have either Edge::Owns relationships OR explicit grants on the target Org/Project URI.

**Wire-error-code migration**: the bespoke `ORG_ACCESS_DENIED` / `PROJECT_ACCESS_DENIED` codes are now **dead paths** for non-owner viewers — the engine-tier blocking fires first, surfacing `NO_GRANTS_HELD`. Clients that hard-code the old codes (UI tooltips, alerting rules, log dashboards) must be migrated to recognise `NO_GRANTS_HELD` as the canonical cross-org denial signal.

---

## §4 — Blast-radius operator guide

If an Org / Project endpoint starts returning unexpected 403s after CH-27 deploys, walk this checklist:

1. **Does the actor have an Edge::Owns edge?**
   ```surql
   SELECT * FROM owns WHERE in = type::thing('agents', '<agent_id>');
   ```
   If yes, the synth-owner-grant should resolve Allow.

2. **Is the action covered by the synth-grant scope?**
   - **Post-CH-27**: `[Allocate, Transfer, Observe, Inspect]` (all 4 universal-applicability verbs per ADR-0062 §D62.2).
   - Pre-CH-27 (CH-25 baseline): `[Allocate, Transfer]` only.

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

- [`composite-resources-model.md`](../architecture/composite-resources-model.md) — architecture page (CH-26 + CH-27 closure).
- [`ADR-0061`](../decisions/0061-org-project-as-composite-resources.md) — CH-26 design decision.
- [`ADR-0062`](../decisions/0062-blocking-gate-and-synth-grant-widening.md) — CH-27 design decision (blocking-gate + synth-grant widening + F4.b helper).
- [`agent-ownership-operations.md`](agent-ownership-operations.md) — CH-25 operations page; upstream Edge::Owns + synth-owner-grant mechanics (4-verb scope at CH-27 close).
- [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) — closed at CH-27 P-SEAL (wire-tier + synth-grant + fixture axes).
- [`D-CH27-FOLLOWUP-01`](../drifts/D-CH27-FOLLOWUP-01.md) — NEW at CH-27 (resolver actor-passthrough deferred to M6).
- Plan archive: [`plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md`](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md) (CH-26).
- Plan archive: [`plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md`](../../../../plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md) (CH-27).
