-- ============================================================================
-- 0019_agent_profile_n_to_1_schema.surql
--
-- Schema evolution for CH-28 / ADR-0063 §D63.1 + §D63.3 + §D63.5 (AgentProfile
-- cardinality flip 1:1 → N:1 + hybrid Blueprint table + USES_PROFILE edge
-- rename + 2 NEW Blueprint-pointer edges).
--
-- F1.c user-lock (DIVERGENT from planner-recommended F1.a): introduces a
-- NEW hybrid `blueprint` table that holds BOTH the shared template-Blueprint
-- (rows where `agent_id = NONE`) AND per-agent override-Blueprints (rows
-- where `agent_id` is set). Per-agent override fields (`parallelize`,
-- `model_config_id`, `mock_response`) move off the `agent_profile` row
-- and onto the `blueprint` row at 0020.
--
-- F2.b user-lock (DIVERGENT from planner-recommended F2.a): renames the
-- `has_profile` relation table to `uses_profile`. Per the half-migrated-
-- state tolerance (F3.b idempotency contract), BOTH `has_profile` AND
-- `uses_profile` tables coexist after 0019 applies — 0020 rewrites the
-- data and removes the obsolete `has_profile` rows. Runtime reads
-- tolerate either edge name via the `read_agent_profile_via_blueprint_or_fallback`
-- helper at `repo_impl.rs` (ADR-0063 §D63.12).
--
-- F3.b user-lock (DIVERGENT from planner-recommended F3.a): splits the
-- cardinality flip into TWO migrations — 0019 (schema-only) + 0020
-- (data-only backfill). Operator may pause between 0019 and 0020 for
-- inspection; the half-migrated-state tolerance keeps the runtime
-- functioning during the inspection window.
--
-- Three operations in one transaction-free script (DDL is non-
-- transactional in SurrealDB 2.x embedded):
--   (a) DROP the UNIQUE index `agent_profile_agent_id` that previously
--       enforced the 1:1 cardinality lock.
--   (b) DEFINE the NEW `blueprint` SCHEMAFULL table + fields + UNIQUE
--       index on `agent_id` filtered to override rows.
--   (c) DEFINE the NEW relation tables `uses_profile` +
--       `agent_profile_uses_blueprint` + `agent_uses_blueprint_override`.
--   (d) REMOVE the per-agent override field DEFINITIONS from
--       `agent_profile` (rows retain their stored values for the
--       half-migrated tolerance window; 0020 cleans up).
--
-- Idempotency: re-running 0019 on a post-apply database is a no-op:
--   - REMOVE INDEX IF EXISTS skips if already dropped.
--   - DEFINE TABLE IF NOT EXISTS skips if already defined (SurrealDB 2.x
--     idempotent form).
--   - DEFINE FIELD IF NOT EXISTS skips if already defined.
--   - DEFINE INDEX IF NOT EXISTS skips if already defined.
--   - REMOVE FIELD IF EXISTS skips if already removed.
--
-- Forward-only per ADR-0012; no down script. Half-migrated tolerance
-- is a NEW precedent codified by ADR-0063 §D63.12 (helper removed in
-- M7 NFR-observability cycle once the migration is universally
-- applied).
--
-- Concept doc anchors:
--   - `concepts/agent.md` §"Soul (Immutable Born Structure)" (cardinality
--     reframe to template-Blueprint sharing).
--   - `concepts/ontology.md` §"Edge Types" L98 (cardinality flip 1:1 → N:1
--     + edge rename HAS_PROFILE → USES_PROFILE + 2 NEW Blueprint-pointer
--     edges).
--   - `implementation/m1/architecture/schema-migrations.md` §"Split-
--     migration pattern (NEW)" (codifies the 0019/0020 split precedent).
--   - `implementation/m6/architecture/agent-profile-cardinality.md` (NEW
--     architecture doc authored at P-DOCS).
-- ============================================================================

-- ---- (a) Drop the 1:1-enforcing UNIQUE index ------------------------------
--
-- Pre-CH-28: `agent_profile_agent_id` UNIQUE on `agent_profile.agent_id`
-- enforced the 1:1 Agent ↔ AgentProfile invariant (per 0001_initial.surql
-- L131). CH-28 flips the cardinality to N:1 — multiple AgentProfile rows
-- may now share the same `agent_id` (template-Blueprint sharing) or
-- carry `agent_id = NONE` (per-agent override-Blueprint pointer rows
-- introduced via the NEW edge variants).

REMOVE INDEX IF EXISTS agent_profile_agent_id ON agent_profile;

-- ---- (b) DEFINE the NEW `blueprint` SCHEMAFULL table ----------------------
--
-- The hybrid Blueprint table holds BOTH the shared template-Blueprint
-- (`agent_id = NONE`) AND per-agent override-Blueprints (`agent_id` set
-- to a concrete AgentId). The discriminator is the `agent_id` field
-- itself — see ADR-0063 §D63.1.
--
-- Fields:
--   - `agent_id`: option<string>. NONE for template rows; set for
--     override rows. The discriminator.
--   - `parallelize`: option<int>. Pulled from the legacy
--     `agent_profile.parallelize` field; falls back to the template's
--     `parallelize` if the per-agent override is unset.
--   - `model_config_id`: option<string>. Pulled from the legacy
--     `agent_profile.model_config_id` (CH-05 / 0005).
--   - `mock_response`: option<string>. Pulled from the legacy
--     `agent_profile.mock_response` (CH-04 / 0006 / ADR-0032 D32.2).
--   - `created_at`: RFC3339 string (matches the rest of the workspace).
--
-- Index `blueprint_agent_id` is a NON-UNIQUE read-side lookup index
-- supporting efficient `SELECT * FROM blueprint WHERE agent_id = $aid`
-- queries used by the per-agent override read path. The 1:1 override-
-- row-per-agent uniqueness constraint is enforced at the REPOSITORY
-- TIER (`upsert_agent_blueprint_override` in `repo_impl.rs`) inside a
-- SurrealDB `BEGIN; ... COMMIT;` transaction — SurrealDB 2.6.5 does NOT
-- support filtered/partial UNIQUE indexes (`DEFINE INDEX ... UNIQUE WHERE ...`
-- is rejected by the parser). See ADR-0063 §D63.5 + §D63.12 for the
-- application-tier enforcement rationale + the half-migrated runtime
-- tolerance cross-reference.

DEFINE TABLE IF NOT EXISTS blueprint SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS agent_id        ON blueprint TYPE option<string> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS parallelize     ON blueprint TYPE option<int>    DEFAULT NONE
    ASSERT $value = NONE OR $value >= 1;
DEFINE FIELD IF NOT EXISTS model_config_id ON blueprint TYPE option<string> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS mock_response   ON blueprint TYPE option<string> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS created_at      ON blueprint TYPE string         ASSERT $value != NONE;

-- Non-UNIQUE read-side lookup index. Override-row uniqueness moves to
-- the application tier (see header comment above).
DEFINE INDEX IF NOT EXISTS blueprint_agent_id
    ON blueprint
    FIELDS agent_id;

-- ---- (c) NEW relation tables ---------------------------------------------
--
-- (c.1) `uses_profile`: renamed from `has_profile` per F2.b. Same semantics
-- (agent → agent_profile edge); the rename is mechanical at the table
-- and Edge enum tier. Both tables coexist during the 0019↔0020 window;
-- 0020 rewrites `has_profile` rows into `uses_profile` rows. The legacy
-- `has_profile` table definition (from 0001 L314) is preserved here for
-- the half-migrated-state window — 0020 may CLEAR rows but does not
-- REMOVE the table definition; a future cycle (M7 NFR-observability)
-- removes `has_profile` entirely.

DEFINE TABLE IF NOT EXISTS uses_profile TYPE RELATION FROM agent TO agent_profile;

-- (c.2) `agent_profile_uses_blueprint`: NEW edge pointing each
-- AgentProfile row at its template-Blueprint row. 1:1 from the
-- AgentProfile side (every AgentProfile has exactly one template
-- Blueprint), N:1 from the Blueprint side (many AgentProfile rows may
-- share one template Blueprint — the load-bearing sharing semantic).

DEFINE TABLE IF NOT EXISTS agent_profile_uses_blueprint
    TYPE RELATION FROM agent_profile TO blueprint;

-- (c.3) `agent_uses_blueprint_override`: NEW edge pointing each Agent
-- at its per-agent override-Blueprint row (if any). 1:N (zero-or-one)
-- from the Agent side; 1:1 from the Blueprint side (each override-
-- Blueprint belongs to exactly one Agent).

DEFINE TABLE IF NOT EXISTS agent_uses_blueprint_override
    TYPE RELATION FROM agent TO blueprint;

-- ---- (d) Remove per-agent override field definitions from agent_profile --
--
-- The override fields (`parallelize` / `model_config_id` / `mock_response`)
-- previously defined on `agent_profile` (per 0001 L127, 0005 L174,
-- 0006 L29) are migrated to the `blueprint` table at 0020.
--
-- Per F-half-migrated-tolerance.a: between 0019 first-apply and 0020
-- first-apply, the runtime reads of `get_agent_profile_for_agent` MUST
-- tolerate the pre-0020 shape — the existing `agent_profile` rows still
-- carry stored values for these fields after `REMOVE FIELD` (REMOVE FIELD
-- in SurrealDB 2.x drops the SCHEMAFULL enforcement; stored row values
-- remain readable as un-asserted properties). The
-- `read_agent_profile_via_blueprint_or_fallback` helper (added at
-- P1-BLUEPRINT-STRUCT) consults the new `blueprint` table first and
-- falls back to reading the legacy `agent_profile` row properties if
-- the blueprint table is empty for the queried agent. This bridges the
-- 0019↔0020 inspection window per ADR-0063 §D63.12.

REMOVE FIELD IF EXISTS parallelize     ON agent_profile;
REMOVE FIELD IF EXISTS model_config_id ON agent_profile;
REMOVE FIELD IF EXISTS mock_response   ON agent_profile;
