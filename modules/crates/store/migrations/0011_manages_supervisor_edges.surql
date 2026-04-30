-- ============================================================================
-- 0011_manages_supervisor_edges.surql
--
-- Schema evolution for CH-23 (Template C/D production triggers).
--
-- Adds the two SurrealDB tables that back the new `MANAGES` and
-- `HAS_AGENT_SUPERVISOR` Edge variants. Both edges are inaugural at
-- CH-23 — there is no production data and no prior schema to migrate.
--
-- Concept doc: `concepts/ontology.md` §"Edges" lists MANAGES (between
-- two Agents within an Org) and HAS_AGENT_SUPERVISOR (between two
-- Agents within a Project) as first-class graph edges. CH-23 / ADR-
-- 0046 lifts both into typed Rust + adds production HTTP writers.
--
-- Idempotency is enforced via UNIQUE indexes on the (scope, from, to)
-- triple — re-POST of the same triple returns the existing edge id
-- (the compound-tx repo method handles the SELECT-then-INSERT
-- semantic; the UNIQUE index defends against a racing duplicate).
--
-- Forward-only per ADR-0012; no down script. Idempotent under
-- repeated runs of the migration runner per ADR-0042 §D42.3 #5.
-- ============================================================================

DEFINE TABLE manages SCHEMAFULL;

DEFINE FIELD edge_id     ON manages TYPE string ASSERT $value != NONE;
DEFINE FIELD org_id      ON manages TYPE string ASSERT $value != NONE;
DEFINE FIELD manager     ON manages TYPE string ASSERT $value != NONE;
DEFINE FIELD subordinate ON manages TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at  ON manages TYPE string ASSERT $value != NONE;

-- Idempotency guard: one edge per (org, manager, subordinate)
-- triple. The repo's create_manages_edge probes for the existing
-- triple before INSERT so a duplicate request returns the existing
-- edge id rather than tripping this index, but the index is the
-- last-line defense against races.
DEFINE INDEX manages_unique
    ON manages
    FIELDS org_id, manager, subordinate
    UNIQUE;

DEFINE TABLE has_agent_supervisor SCHEMAFULL;

DEFINE FIELD edge_id     ON has_agent_supervisor TYPE string ASSERT $value != NONE;
DEFINE FIELD project_id  ON has_agent_supervisor TYPE string ASSERT $value != NONE;
DEFINE FIELD supervisor  ON has_agent_supervisor TYPE string ASSERT $value != NONE;
DEFINE FIELD supervisee  ON has_agent_supervisor TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at  ON has_agent_supervisor TYPE string ASSERT $value != NONE;

DEFINE INDEX has_agent_supervisor_unique
    ON has_agent_supervisor
    FIELDS project_id, supervisor, supervisee
    UNIQUE;
