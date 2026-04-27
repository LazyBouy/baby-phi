-- ============================================================================
-- 0007_agent_active_archived.surql
--
-- Schema evolution for phi v0.1 / chunk CH-01.
--
-- Scope (per CH-01 plan + ADR-0034):
--
--   1. `agent.active` (ADR-0034 D34.1). Durable disable flag. Default true
--      so pre-CH-01 rows materialise active. Read by AgentCatalogListener
--      (CH-22 body, future) and flipped by the system-agent disable
--      handler (CH-01 / P3).
--
--   2. `agent.archived_at` (ADR-0034 D34.2). Optional archive timestamp
--      stored as RFC3339 string per existing project convention
--      (see grant.revoked_at + consent.revoked_at in 0001_initial.surql).
--      None = not archived. Set by the system-agent archive handler
--      (CH-01 / P3).
--
-- Forward-only; per ADR-0012 we do not author `down` scripts. Pre-CH-01
-- rows deserialise with `#[serde(default = "default_agent_active")]` →
-- `active = true` and `#[serde(default)]` → `archived_at = None` on
-- `domain::model::nodes::Agent` — no backfill required.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. agent.active — ADR-0034 D34.1
-- ----------------------------------------------------------------------------
--
-- Durable disable flag. DEFAULT true: rows inserted before CH-01 migration
-- (or rows that omit the column on insert) materialise as active so the
-- existing fleet keeps running unchanged. Disable handler flips this to
-- false; AgentCatalogListener (CH-22) reads it for `is_paused` state.

DEFINE FIELD active ON agent TYPE bool DEFAULT true;

-- ----------------------------------------------------------------------------
-- 2. agent.archived_at — ADR-0034 D34.2
-- ----------------------------------------------------------------------------
--
-- Optional archive timestamp. None = not archived. RFC3339 string per
-- the project convention for optional datetime columns
-- (grant.revoked_at, consent.revoked_at use the same shape).
-- chrono's serde feature serialises DateTime<Utc> as RFC3339 by default.

DEFINE FIELD archived_at ON agent TYPE option<string>;
