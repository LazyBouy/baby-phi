-- ============================================================================
-- 0004_agents_projects.surql
--
-- Schema evolution for phi v0.1 / milestone M4.
--
-- Scope:
--   1. Extend the `agent` table with the 6-variant `role` discriminator
--      introduced by the concept-doc amendment at M4/P0 (see
--      `docs/specs/v0/concepts/agent.md §Agent Roles`). Pre-M4 rows lack
--      this column; `#[serde(default)]` on the domain type lets them
--      round-trip with `role = None` (treated as "Unclassified" by the
--      dashboard roll-up — see C17 in the M4 plan).
--
--   2. Flesh out the `project` table (scaffolded as id-only at M1) with
--      the full M4 property set: name / description / goal / status /
--      shape / token_budget / tokens_spent / objectives / key_results /
--      resource_boundaries. `objectives` + `key_results` +
--      `resource_boundaries` store as `FLEXIBLE TYPE object` so OKR
--      evolution does NOT force a migration 0005 — same absorb-mutation
--      pattern used by M3's `organization.defaults_snapshot`.
--
--   3. Create the `agent_execution_limits` table (one row per agent who
--      opts into a per-agent override per ADR-0027). `limits` is the
--      `FLEXIBLE TYPE object` wrap of `phi_core::ExecutionLimits`. The
--      UNIQUE index on `owning_agent` pins the 1:1 invariant (absence of
--      a row = "inherit from org snapshot" per ADR-0023).
--
--   4. Create the two missing edge RELATION tables named at M4/P0's
--      ontology audit:
--        * `has_lead`        (Project → Agent) — added to the domain
--          enum at M3/P1 with zero production writes; M4/P3 wires the
--          first writer via `apply_project_creation`.
--        * `has_subproject`  (Project → Project) — new at M4/P1.
--        * `has_config`      (Project → AgentConfig) — new at M4/P1.
--
-- Forward-only + idempotent via the migration ledger (the runner writes a
-- `_migrations` row after success). No rollback DDL ships with v0.1.
--
-- phi-core leverage (M4/P1 stance): the lone phi-core-wrapping column
-- added here is `agent_execution_limits.limits` (the direct wrap of
-- `phi_core::ExecutionLimits` via `AgentExecutionLimitsOverride`).
-- Stored as `FLEXIBLE TYPE object` so phi-core evolution flows through
-- without a migration 0005 — the identical strategy used for
-- `organization.defaults_snapshot` at M3 (see 0003 §1 header).
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. agent — add the `role` column for the 6-variant AgentRole enum
-- ----------------------------------------------------------------------------

-- Wire-names match `AgentRole::as_str()` / serde rename_all = "snake_case"
-- in `modules/crates/domain/src/model/nodes.rs`. `option<string>` so
-- pre-M4 rows land with `role = None`.
DEFINE FIELD role ON agent TYPE option<string>
    ASSERT $value == NONE OR $value INSIDE [
        "executive", "admin", "member",
        "intern", "contract", "system"
    ];

-- ----------------------------------------------------------------------------
-- 2. project — add the full M4 field set on top of the M1 id-only scaffold
-- ----------------------------------------------------------------------------

DEFINE FIELD name         ON project TYPE string;
DEFINE FIELD description  ON project TYPE string       DEFAULT "";
DEFINE FIELD goal         ON project TYPE option<string>;

-- Wire-names match `ProjectStatus` serde rendering (snake_case).
DEFINE FIELD status ON project TYPE string
    ASSERT $value INSIDE ["planned", "in_progress", "on_hold", "finished"]
    DEFAULT "planned";

-- Wire-names match `ProjectShape` explicit serde renames (`shape_a` /
-- `shape_b`), pinning the dashboard's `ProjectsSummary.shape_a` /
-- `.shape_b` counter field names (M3 carryover C-M4-3).
DEFINE FIELD shape ON project TYPE string
    ASSERT $value INSIDE ["shape_a", "shape_b"]
    DEFAULT "shape_a";

DEFINE FIELD token_budget ON project TYPE option<int>;
DEFINE FIELD tokens_spent ON project TYPE int
    ASSERT $value >= 0
    DEFAULT 0;

-- Embedded OKR + ResourceBoundaries value objects. FLEXIBLE so OKR
-- evolution doesn't force a migration 0005 — identical pattern to
-- `organization.defaults_snapshot`. Object shape is enforced at the
-- domain layer via serde.
DEFINE FIELD objectives          ON project FLEXIBLE TYPE array DEFAULT [];
DEFINE FIELD objectives.*        ON project FLEXIBLE TYPE object;
DEFINE FIELD key_results         ON project FLEXIBLE TYPE array DEFAULT [];
DEFINE FIELD key_results.*       ON project FLEXIBLE TYPE object;
DEFINE FIELD resource_boundaries ON project FLEXIBLE TYPE option<object>;

-- ----------------------------------------------------------------------------
-- 3. agent_execution_limits — per-agent ExecutionLimits override (ADR-0027)
-- ----------------------------------------------------------------------------
--
-- One row per agent that opts in to an override. Absence of a row means
-- "inherit from org snapshot" (ADR-0023's default path). The UNIQUE
-- index on `owning_agent` pins that 1:1 invariant — a second INSERT
-- for the same agent is rejected at storage, forcing callers to use
-- UPDATE (idempotency) or DELETE-then-INSERT (revert-and-reset).

DEFINE TABLE agent_execution_limits SCHEMAFULL;

DEFINE FIELD owning_agent ON agent_execution_limits TYPE string;
-- `limits` wraps phi_core::ExecutionLimits verbatim (max_turns +
-- max_total_tokens + max_duration + max_cost). FLEXIBLE absorbs
-- phi-core evolution without a migration 0005.
DEFINE FIELD limits       ON agent_execution_limits FLEXIBLE TYPE object;
DEFINE FIELD created_at   ON agent_execution_limits TYPE string;

DEFINE INDEX agent_execution_limits_owning_agent
    ON agent_execution_limits
    FIELDS owning_agent UNIQUE;

-- ----------------------------------------------------------------------------
-- 4. Edge RELATION tables for the three Project-scoped edges that the M4
--    compound tx will write (has_lead + has_subproject + has_config).
-- ----------------------------------------------------------------------------

-- Project → Agent: the designated project lead. Added to the domain
-- enum at M3/P1 with zero production writes; M4/P3 wires the first
-- writer (`apply_project_creation`).
DEFINE TABLE has_lead       TYPE RELATION FROM project TO agent;

-- Project → Project: subproject nesting per `concepts/project.md`
-- §Project Edges. First writer lands at M4/P6 (if operator declares a
-- parent at creation time); otherwise unused at M4.
DEFINE TABLE has_subproject TYPE RELATION FROM project TO project;

-- Project → AgentConfig: project-level root config. First production
-- write lands at M5 (session launch), when agent-config resolution
-- extends to project scope.
DEFINE TABLE has_config     TYPE RELATION FROM project TO agent_config;
