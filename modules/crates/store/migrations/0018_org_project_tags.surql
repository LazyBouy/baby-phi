-- ============================================================================
-- 0018_org_project_tags.surql
--
-- Schema evolution for CH-26 / ADR-0061 §D61.2 + §D61.3 (Org/Project as
-- Composite resources with instance-identity tags).
--
-- F2.b user-lock (DIVERGENT from planner-recommended F2.a): adds the
-- `tags: Vec<String>` field to the Organization + Project struct shapes
-- (with `#[serde(default)]` for backward compat) AND backfills the
-- instance-identity tag (`organization:<uuid>` / `project:<uuid>`) for
-- every extant row.
--
-- F1.b user-lock (≥ 7 handler refactor through `check_permission`)
-- depends on the row-level tags being present so selector matching at
-- Permission Check time resolves `org:<id>` / `project:<id>` URIs
-- against the row's tags. The catalogue entry (Step 0 precondition)
-- is also seeded for every extant row.
--
-- Three operations in one transaction:
--   (a) DEFINE FIELD tags on organization + project tables (SCHEMAFULL
--       requires explicit field declaration for the new column).
--   (b) UPDATE every extant org/project row to populate `tags` with the
--       instance-identity tag (filtered on empty-or-none tags for
--       idempotent re-run).
--   (c) Seed catalogue entries for every extant Org + Project row at
--       the canonical `org:<uuid>` / `project:<uuid>` URIs (UPSERT-on-uri
--       semantics; idempotent).
--
-- Idempotency: re-running the migration is a no-op:
--   - DEFINE FIELD is idempotent in SurrealDB 2.x.
--   - UPDATE's WHERE clause filters on `tags IS NONE OR tags = []`.
--   - Catalogue INSERT uses ON DUPLICATE KEY UPDATE semantics via
--     UPSERT (existing rows are not duplicated).
--
-- Forward-only per ADR-0012; no down script.
--
-- Concept doc anchors:
--   - `concepts/core-philosophy.md:16` (*"Organization has Projects
--     (A Resource Type)"*).
--   - `concepts/permissions/01-resource-ontology.md:30-43` (Composite
--     Classes table — flips 8 → 10 at CH-26).
--   - `concepts/permissions/01-resource-ontology.md:222-263` (Instance
--     Identity Tags — every composite instance carries
--     `{kind}:{instance_id}`).
-- ============================================================================

-- ---- (a) Column-add (idempotent) -------------------------------------------

DEFINE FIELD tags ON organization TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON project      TYPE array<string> DEFAULT [];

-- ---- (b) Instance-identity backfill (idempotent) ---------------------------
--
-- For every extant org/project row: if `tags` is missing or empty, set
-- it to `[<kind>:<id>]`. The `<id>` is derived from the SurrealDB
-- record-id, which mirrors the `OrgId` / `ProjectId` UUID payload.
--
-- The format function uses `string::concat($prefix, <-id-as-string>)`;
-- SurrealDB stores ids as `organization:⟨uuid⟩` so `<-id>` is the full
-- thing-id and we want only the trailing UUID portion. Use
-- `record::id($id)` to get the UUID alone, then prepend `organization:`
-- (matching the runtime tag form populated by `apply_org_creation`).

UPDATE organization
    SET tags = ['organization:' + record::id(id)]
    WHERE tags IS NONE OR tags = [];

UPDATE project
    SET tags = ['project:' + record::id(id)]
    WHERE tags IS NONE OR tags = [];

-- ---- (c) Catalogue-entry backfill — DEFERRED to compound-tx -----------------
--
-- Pre-CH-26 catalogue rows were already seeded with `resource_uri =
-- 'org:<id>' kind = 'control_plane'` (per `apply_org_creation`'s
-- catalogue_entries logic at `orgs/create.rs:249`). The
-- `resources_catalogue_owning_uri` UNIQUE index on `(owning_org,
-- resource_uri)` prevents adding a duplicate canonical entry from this
-- migration. The existing entry's URI ('org:<id>') is already the
-- canonical Step-0-precondition URI — only the `kind` field differs
-- from the post-CH-26 canonical name. Selector matching at Permission
-- Check time uses the URI, not the kind, so the existing seed is
-- sufficient for handler refactor F1.b.
--
-- No catalogue rows touched by this migration. The schema change in
-- step (a) and the instance-identity backfill in step (b) are the
-- load-bearing parts.
