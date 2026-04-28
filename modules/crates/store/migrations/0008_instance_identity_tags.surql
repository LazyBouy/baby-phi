-- ============================================================================
-- 0008_instance_identity_tags.surql
--
-- Schema evolution for CH-06 (D-new-11 closure).
--
-- Adds `tags ARRAY<string>` to every composite + node table whose struct
-- now carries a `tags: Vec<String>` field per
-- `concepts/permissions/01-resource-ontology.md` §"Instance Identity Tags".
-- The two reserved entries (`#kind:{name}` + `{name}:{id}`) are emitted at
-- creation by the runtime; pre-migration rows fall through to the
-- DEFAULT [] backfill and #[serde(default)] on the struct field, so
-- existing data deserialises cleanly.
--
-- Memory.tags already ships in 0001_initial.surql — only the production
-- emission was missing (closed in CH-06 by struct-side wiring). This
-- migration covers the other 10 tables.
--
-- Forward-only per ADR-0012; no down script.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- M2 / governance core composites
-- ----------------------------------------------------------------------------

DEFINE FIELD tags ON mcp_server TYPE array<string> DEFAULT [];

-- ----------------------------------------------------------------------------
-- M3 / org-scope composites
-- ----------------------------------------------------------------------------

DEFINE FIELD tags ON token_budget_pool TYPE array<string> DEFAULT [];

-- ----------------------------------------------------------------------------
-- M4 / per-agent overrides
-- ----------------------------------------------------------------------------

DEFINE FIELD tags ON agent_execution_limits TYPE array<string> DEFAULT [];

-- ----------------------------------------------------------------------------
-- M5 / catalog + runtime + sidecar tables
-- ----------------------------------------------------------------------------

DEFINE FIELD tags ON agent_catalog_entry TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON system_agent_runtime_status TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON shape_b_pending_projects TYPE array<string> DEFAULT [];

-- ----------------------------------------------------------------------------
-- Node-tier tables
-- ----------------------------------------------------------------------------

DEFINE FIELD tags ON auth_request TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON inbox_object TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON outbox_object TYPE array<string> DEFAULT [];
DEFINE FIELD tags ON session TYPE array<string> DEFAULT [];
