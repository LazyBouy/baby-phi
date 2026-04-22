-- ============================================================================
-- 0002_platform_setup.surql
--
-- M2 (Platform Setup) schema additions — admin pages 02–05.
--
-- What this migration ships:
--   1. `template.kind`  — discriminates the lifecycle family (SystemBootstrap
--      / A / B / C / D / E / F); M2's Template E is the first user of this.
--   2. `mcp_server`    — extends the M1 scaffolded table with the full
--      `ExternalService` composite schema (display_name, kind, endpoint,
--      secret_ref, tenants_allowed, status, archived_at).
--   3. `model_runtime` — NEW table for the `model_runtime_object` composite
--      (page 02). Holds the embedded phi-core `ModelConfig` as an opaque
--      `FLEXIBLE TYPE object`, so phi-core can evolve `ModelConfig`'s shape
--      without forcing a phi migration.
--   4. `platform_defaults` — NEW singleton table (page 05). A UNIQUE INDEX
--      on the constant `singleton = 1` column guarantees at most one row.
--
-- Forward-only. M1 pre-release invariant: secrets_vault's final column set
-- lives in 0001_initial.surql (squashed at P0/P1 handoff) — this migration
-- does NOT touch secrets_vault.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. template.kind
--
-- Pre-existing `template` rows (seeded by --bootstrap-init) satisfy the
-- ASSERT because the serde default on `TemplateKind` is `SystemBootstrap`;
-- the DB-side DEFAULT below matches.
-- ----------------------------------------------------------------------------

DEFINE FIELD kind ON template TYPE string
    DEFAULT "system_bootstrap"
    ASSERT $value INSIDE ["system_bootstrap", "a", "b", "c", "d", "e", "f"];

-- ----------------------------------------------------------------------------
-- 2. mcp_server — full M2 column set
--
-- `kind` is a discriminator (mcp / open_api / webhook / other); M2 only
-- wires `mcp`. `tenants_allowed` is stored as two columns — a `mode`
-- ("all" | "only") + a `Vec<OrgId>` list — matching the `TenantSet` enum's
-- `#[serde(tag = "mode", content = "orgs")]` representation on the domain
-- struct.
-- ----------------------------------------------------------------------------

DEFINE FIELD display_name         ON mcp_server TYPE string;
DEFINE FIELD kind                 ON mcp_server TYPE string
    ASSERT $value INSIDE ["mcp", "open_api", "webhook", "other"];
DEFINE FIELD endpoint             ON mcp_server TYPE string;
DEFINE FIELD secret_ref           ON mcp_server TYPE option<string>;
DEFINE FIELD tenants_allowed      ON mcp_server FLEXIBLE TYPE object;
DEFINE FIELD status               ON mcp_server TYPE string
    ASSERT $value INSIDE ["ok", "probing", "degraded", "error", "archived"]
    DEFAULT "ok";
DEFINE FIELD archived_at          ON mcp_server TYPE option<string>;
-- `created_at` is already declared in 0001_initial.surql.
DEFINE INDEX mcp_server_status    ON mcp_server FIELDS status;

-- ----------------------------------------------------------------------------
-- 3. model_runtime — NEW table for `model_runtime_object` composite
--
-- The embedded `config` column holds phi-core's `ModelConfig` struct as
-- a flexible object. Using `FLEXIBLE TYPE object` rather than declaring
-- each field individually means phi-core's `ModelConfig` can evolve
-- (new fields, new `compat` flags) without requiring a phi
-- migration — the struct round-trips through SurrealDB by serde.
--
-- `secret_ref` is the vault slug (e.g. "anthropic-api-key") that
-- resolves to the live API key at call time; `config.api_key` on the
-- stored record is a placeholder.
-- ----------------------------------------------------------------------------

DEFINE TABLE model_runtime SCHEMAFULL;
DEFINE FIELD config               ON model_runtime FLEXIBLE TYPE object;
DEFINE FIELD secret_ref           ON model_runtime TYPE string;
DEFINE FIELD tenants_allowed      ON model_runtime FLEXIBLE TYPE object;
DEFINE FIELD status               ON model_runtime TYPE string
    ASSERT $value INSIDE ["ok", "probing", "degraded", "error", "archived"]
    DEFAULT "ok";
DEFINE FIELD archived_at          ON model_runtime TYPE option<string>;
DEFINE FIELD created_at           ON model_runtime TYPE string;
DEFINE INDEX model_runtime_status ON model_runtime FIELDS status;

-- ----------------------------------------------------------------------------
-- 4. platform_defaults — singleton table enforcing at-most-one row
--
-- The `singleton` column is always 1 (ASSERT enforces); a UNIQUE INDEX
-- on that column means the second INSERT ever attempted fails loudly.
-- Every "nested" field is stored as a flexible object so phi-core's
-- types can evolve without forcing a phi migration.
-- ----------------------------------------------------------------------------

DEFINE TABLE platform_defaults SCHEMAFULL;
DEFINE FIELD singleton                ON platform_defaults TYPE int
    ASSERT $value = 1 DEFAULT 1;
DEFINE FIELD execution_limits         ON platform_defaults FLEXIBLE TYPE object;
DEFINE FIELD default_agent_profile    ON platform_defaults FLEXIBLE TYPE object;
DEFINE FIELD context_config           ON platform_defaults FLEXIBLE TYPE object;
DEFINE FIELD retry_config             ON platform_defaults FLEXIBLE TYPE object;
DEFINE FIELD default_retention_days   ON platform_defaults TYPE int
    ASSERT $value >= 0;
DEFINE FIELD default_alert_channels   ON platform_defaults TYPE array;
DEFINE FIELD default_alert_channels.* ON platform_defaults TYPE string;
DEFINE FIELD updated_at               ON platform_defaults TYPE string;
DEFINE FIELD version                  ON platform_defaults TYPE int
    ASSERT $value >= 0 DEFAULT 0;
DEFINE INDEX platform_defaults_singleton ON platform_defaults FIELDS singleton UNIQUE;
