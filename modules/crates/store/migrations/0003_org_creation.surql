-- ============================================================================
-- 0003_org_creation.surql
--
-- Schema evolution for phi v0.1 / milestone M3.
--
-- Scope:
--   1. Extend the `organization` table with the M3 governance fields
--      (vision/mission/consent_policy/audit_class_default/
--      authority_templates_enabled/defaults_snapshot/default_model_provider/
--      system_agents). Pre-M3 rows lack these columns; the M3 domain types
--      deserialise them via `#[serde(default)]` so round-trips from old
--      data stay valid — this migration just tells SurrealDB the columns
--      exist on SCHEMAFULL tables.
--
--   2. Create the `token_budget_pool` table (one row per org at creation
--      time). `initial_allocation` is the ceiling; `used` is monotonically
--      non-decreasing; the `used <= initial_allocation` invariant is pinned
--      by a domain-level `TokenBudgetPool::debit` guard + mirrored here as
--      a storage-layer ASSERT so a rogue INSERT bypassing the domain layer
--      still fails.
--
-- Forward-only + idempotent via the migration ledger (the runner writes a
-- `_migrations` row after success; on subsequent startups this file is
-- skipped). No rollback DDL ships with v0.1 — the migration runner's
-- failure path is "abort start-up + surface the error to the operator"
-- rather than "auto-undo."
--
-- phi-core leverage: the phi-core-wrapping sub-fields of
-- `organization.defaults_snapshot` (execution_limits, default_agent_profile,
-- context_config, retry_config) are stored as `FLEXIBLE TYPE option<object>`
-- so phi-core's type evolution does NOT force a migration 0004 — the same
-- absorb-mutation pattern `platform_defaults` uses (see 0002 §4).
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. organization — extend with M3 governance fields
-- ----------------------------------------------------------------------------

DEFINE FIELD vision       ON organization TYPE option<string>;
DEFINE FIELD mission      ON organization TYPE option<string>;

-- Wire-name matches `ConsentPolicy::as_wire()` in
-- `modules/crates/domain/src/model/composites_m3.rs` (serde snake_case).
DEFINE FIELD consent_policy ON organization TYPE string
    ASSERT $value INSIDE ["implicit", "one_time", "per_session"]
    DEFAULT "implicit";

-- Wire-names match `AuditClass` serde rendering in
-- `modules/crates/domain/src/audit/mod.rs`.
DEFINE FIELD audit_class_default ON organization TYPE string
    ASSERT $value INSIDE ["silent", "logged", "alerted"]
    DEFAULT "logged";

-- Authority templates enabled at creation time. `a`..`f` matches
-- `TemplateKind::as_wire()` in `model/nodes.rs`. `system_bootstrap` is
-- deliberately excluded — orgs never adopt the bootstrap template.
DEFINE FIELD authority_templates_enabled   ON organization TYPE array DEFAULT [];
DEFINE FIELD authority_templates_enabled.* ON organization TYPE string
    ASSERT $value INSIDE ["a", "b", "c", "d", "e", "f"];

-- Platform-defaults snapshot at creation time. `FLEXIBLE` absorbs
-- phi-core evolution (matching 0002's `platform_defaults` columns).
-- `option<object>` lets pre-M3 rows round-trip with a `None` snapshot.
DEFINE FIELD defaults_snapshot ON organization FLEXIBLE TYPE option<object>;

-- Optional org-level default provider binding (points at a
-- `model_providers` row id).
DEFINE FIELD default_model_provider ON organization TYPE option<string>;

-- System agents provisioned at org-creation time. Elements are
-- AgentId-shaped UUID strings.
DEFINE FIELD system_agents   ON organization TYPE array DEFAULT [];
DEFINE FIELD system_agents.* ON organization TYPE string;

-- ----------------------------------------------------------------------------
-- 2. token_budget_pool — per-org token-allocation ledger (1:1 with Org)
-- ----------------------------------------------------------------------------

DEFINE TABLE token_budget_pool SCHEMAFULL;

DEFINE FIELD owning_org          ON token_budget_pool TYPE string;
DEFINE FIELD initial_allocation  ON token_budget_pool TYPE int
    ASSERT $value >= 0;
DEFINE FIELD used                ON token_budget_pool TYPE int
    ASSERT $value >= 0
    DEFAULT 0;
DEFINE FIELD created_at          ON token_budget_pool TYPE string;

-- One pool per org in v0 — query-critical for the dashboard's
-- `TokenBudget` panel. M7+ may relax this to permit multi-pool orgs
-- (e.g. per-project sub-budgets); the UNIQUE index here makes that a
-- deliberate migration change rather than a silent evolution.
DEFINE INDEX token_budget_pool_owning_org ON token_budget_pool FIELDS owning_org UNIQUE;
