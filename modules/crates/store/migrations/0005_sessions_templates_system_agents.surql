-- ============================================================================
-- 0005_sessions_templates_system_agents.surql
--
-- Schema evolution for phi v0.1 / milestone M5.
--
-- Scope (per M5/P1 plan + ADRs 0029/0030):
--
--   1. Template uniqueness flip (D1 / ADR-0030). DROP the UNIQUE INDEX on
--      `template.name` (introduced in migration 0001), DEFINE a UNIQUE
--      INDEX on `template.kind`. One shared Template row per kind; adoption
--      lives on the AR's `provenance_template`.
--
--   2. UsesModel edge retype (C-M5-2). The 0001 RELATION was
--      `FROM agent TO model_config`; M2/P6 moved the authoritative runtime
--      catalogue to `model_runtime`. REMOVE the vestigial relation and
--      DEFINE it fresh as `FROM agent TO model_runtime`. The old
--      `model_config` table stays as a zombie (no writers).
--
--   3. Three session-tier tables (C-M5-3 / ADR-0029). `session`,
--      `loop_record`, `turn`. Each carries a `FLEXIBLE TYPE object` for
--      the phi-core `inner` wrap + explicit governance columns. Per-Turn
--      queries stay O(1) via the flattened storage layout; the nested
--      `phi_core::Session.loops: Vec<LoopRecord>` tree is reconstructed at
--      the repo layer only on full drill-down.
--
--   4. `runs_session` relation from `session` to `project` so page 11's
--      "Recent sessions" panel can query by project.
--
--   5. `shape_b_pending_projects` sidecar (C-M5-6). One row per Shape B
--      AR holding the full `CreateProjectInput` payload between submit
--      and both-approve. UNIQUE `auth_request_id`; `payload` is
--      FLEXIBLE so the payload shape can evolve without migration 0006.
--
--   6. `agent_profile.model_config_id` nullable column (C-M5-5). Per-agent
--      `ModelConfig` binding; absence = inherit from org snapshot path.
--      `option<string>` so pre-M5 rows deserialise with
--      `#[serde(default)] = None`.
--
--   7. `agent_catalog_entry` table (s03 cache). One row per agent per
--      org; `AgentCatalogListener` upserts on 8 trigger variants.
--
--   8. `system_agent_runtime_status` table (page 13 queue-depth / last-
--      fired indicators). Upserted by all 5 listeners (Template A/C/D +
--      memory-extraction + agent-catalog) on each fire.
--
-- Forward-only + idempotent via the `_migrations` ledger. No rollback
-- DDL ships with v0.1.
--
-- phi-core leverage (M5/P1 stance): three new wrap columns on the
-- session tier — `session.inner`, `loop_record.inner`, `turn.inner`
-- all store phi-core's `Session` / `LoopRecord` / `Turn` verbatim via
-- `FLEXIBLE TYPE object`, the same absorb-mutation strategy used for
-- `organization.defaults_snapshot` (M3 / 0003) and
-- `agent_execution_limits.limits` (M4 / 0004). phi-core evolution flows
-- through without a migration 0006 on shape changes.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. template uniqueness flip — name → kind (D1 / ADR-0030)
-- ----------------------------------------------------------------------------

REMOVE INDEX template_name ON template;
DEFINE INDEX template_kind ON template FIELDS kind UNIQUE;

-- ----------------------------------------------------------------------------
-- 2. uses_model retype — FROM agent TO model_runtime (C-M5-2)
-- ----------------------------------------------------------------------------

REMOVE TABLE uses_model;
DEFINE TABLE uses_model TYPE RELATION FROM agent TO model_runtime;

-- ----------------------------------------------------------------------------
-- 3. session / loop_record / turn — 3-way wrap tier (C-M5-3 / ADR-0029)
-- ----------------------------------------------------------------------------
--
-- `session` + `turn` were declared as SCHEMAFULL scaffolds with only a
-- `created_at` column in migration 0001. M5/P1 layers the M5 governance
-- fields on top (inner / owning_org / etc.) without re-DEFINE TABLE
-- — SurrealDB rejects a second DEFINE TABLE for an existing table.
-- `loop_record` is a fresh table (the 0001 scaffold was named `loop`
-- without governance fields; it stays as a zombie table and can be
-- dropped in M7b cleanup).

DEFINE FIELD inner            ON session FLEXIBLE TYPE object;
DEFINE FIELD owning_org       ON session TYPE string;
DEFINE FIELD owning_project   ON session TYPE string;
DEFINE FIELD started_by       ON session TYPE string;
-- Wire-names match `SessionGovernanceState` serde rendering (snake_case).
DEFINE FIELD governance_state ON session TYPE string
    ASSERT $value INSIDE ["running", "completed", "aborted", "failed_launch"]
    DEFAULT "running";
DEFINE FIELD started_at       ON session TYPE string;
DEFINE FIELD ended_at         ON session TYPE option<string>;
DEFINE FIELD tokens_spent     ON session TYPE int
    ASSERT $value >= 0
    DEFAULT 0;

DEFINE INDEX session_by_project
    ON session
    FIELDS owning_project;

DEFINE INDEX session_by_agent
    ON session
    FIELDS started_by;

DEFINE TABLE loop_record SCHEMAFULL;

DEFINE FIELD inner        ON loop_record FLEXIBLE TYPE object;
DEFINE FIELD session_id   ON loop_record TYPE string;
DEFINE FIELD loop_index   ON loop_record TYPE int
    ASSERT $value >= 0;

DEFINE INDEX loop_record_by_session
    ON loop_record
    FIELDS session_id;

-- `turn` table already exists from 0001; only add the M5 governance
-- fields + index.

DEFINE FIELD inner      ON turn FLEXIBLE TYPE object;
DEFINE FIELD loop_id    ON turn TYPE string;
DEFINE FIELD turn_index ON turn TYPE int
    ASSERT $value >= 0;

DEFINE INDEX turn_by_loop
    ON turn
    FIELDS loop_id;

-- ----------------------------------------------------------------------------
-- 4. runs_session edge retype — session → project (C-M5-3)
-- ----------------------------------------------------------------------------
--
-- The 0001 scaffold typed `runs_session` as `agent → session`. It had
-- zero production writers (the governance semantic was backwards —
-- an *agent* doesn't "run" a session, a *session* "runs in" a
-- project). Retyped here to `session → project` so page 11's
-- "Recent sessions" panel query walks the correct direction. Same
-- forward-only retype pattern as `uses_model`.

REMOVE TABLE runs_session;
DEFINE TABLE runs_session TYPE RELATION FROM session TO project;

-- ----------------------------------------------------------------------------
-- 5. shape_b_pending_projects sidecar (C-M5-6)
-- ----------------------------------------------------------------------------
--
-- One row per Shape B project-creation AR; the full `CreateProjectInput`
-- lives in `payload` between submit and both-approve. The Approved
-- branch of `approve_pending_shape_b` reads the row + deletes it inside
-- the same compound tx that materialises the project.

DEFINE TABLE shape_b_pending_projects SCHEMAFULL;

DEFINE FIELD auth_request_id ON shape_b_pending_projects TYPE string;
-- `payload` = serialised `CreateProjectInput` (name, goal, leads,
-- members, OKRs, token_budget, resource_boundaries). FLEXIBLE so the
-- input shape can evolve across milestones without migration 0006.
DEFINE FIELD payload         ON shape_b_pending_projects FLEXIBLE TYPE object;
DEFINE FIELD created_at      ON shape_b_pending_projects TYPE string;

DEFINE INDEX shape_b_pending_projects_ar
    ON shape_b_pending_projects
    FIELDS auth_request_id UNIQUE;

-- ----------------------------------------------------------------------------
-- 6. agent_profile.model_config_id — per-agent ModelConfig binding (C-M5-5)
-- ----------------------------------------------------------------------------
--
-- Nullable. Absence = inherit from org snapshot (ADR-0023 default path).
-- Pre-M5 rows deserialise with `#[serde(default)] = None` on the domain
-- type. Validated at PATCH time against the owning org's ModelRuntime
-- catalogue (M2/P6 surface).

DEFINE FIELD model_config_id ON agent_profile TYPE option<string>;

-- ----------------------------------------------------------------------------
-- 7. agent_catalog_entry — s03 catalogue cache
-- ----------------------------------------------------------------------------
--
-- One row per agent per org. `AgentCatalogListener` upserts on:
--   AgentCreated / AgentArchived / HasLeadEdgeCreated / ManagesEdgeCreated
--   / HasAgentSupervisorEdgeCreated / HasProfileEdgeChanged
-- Queryable by org for page-07 dashboard roll-ups + M6/a05 grants view.

DEFINE TABLE agent_catalog_entry SCHEMAFULL;

DEFINE FIELD agent_id       ON agent_catalog_entry TYPE string;
DEFINE FIELD owning_org     ON agent_catalog_entry TYPE string;
DEFINE FIELD display_name   ON agent_catalog_entry TYPE string;
DEFINE FIELD kind           ON agent_catalog_entry TYPE string
    ASSERT $value INSIDE ["human", "llm"];
DEFINE FIELD role           ON agent_catalog_entry TYPE option<string>;
DEFINE FIELD active         ON agent_catalog_entry TYPE bool
    DEFAULT true;
DEFINE FIELD profile_snapshot ON agent_catalog_entry FLEXIBLE TYPE option<object>;
DEFINE FIELD last_seen_at   ON agent_catalog_entry TYPE string;
DEFINE FIELD updated_at     ON agent_catalog_entry TYPE string;

DEFINE INDEX agent_catalog_entry_identity
    ON agent_catalog_entry
    FIELDS agent_id UNIQUE;

DEFINE INDEX agent_catalog_entry_by_org
    ON agent_catalog_entry
    FIELDS owning_org;

-- ----------------------------------------------------------------------------
-- 8. system_agent_runtime_status — page 13 live indicators
-- ----------------------------------------------------------------------------
--
-- Upserted by all 5 listeners (Template A / C / D + memory-extraction +
-- agent-catalog) on every fire. Powers page 13's queue-depth +
-- last-fired-at tiles.

DEFINE TABLE system_agent_runtime_status SCHEMAFULL;

DEFINE FIELD agent_id              ON system_agent_runtime_status TYPE string;
DEFINE FIELD owning_org            ON system_agent_runtime_status TYPE string;
DEFINE FIELD queue_depth           ON system_agent_runtime_status TYPE int
    ASSERT $value >= 0
    DEFAULT 0;
DEFINE FIELD last_fired_at         ON system_agent_runtime_status TYPE option<string>;
DEFINE FIELD effective_parallelize ON system_agent_runtime_status TYPE int
    ASSERT $value >= 1
    DEFAULT 1;
DEFINE FIELD last_error            ON system_agent_runtime_status TYPE option<string>;
DEFINE FIELD updated_at            ON system_agent_runtime_status TYPE string;

DEFINE INDEX system_agent_runtime_status_identity
    ON system_agent_runtime_status
    FIELDS agent_id UNIQUE;
