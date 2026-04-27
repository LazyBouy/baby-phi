-- ============================================================================
-- 0006_agent_profile_mock_response.surql
--
-- Schema evolution for phi v0.1 / chunk CH-02.
--
-- Scope (per CH-02 plan + ADR-0032):
--
--   1. `agent_profile.mock_response` (ADR-0032 D32.2). Optional governance
--      field on baby-phi's AgentProfile wrapper. When Some, the M5
--      `provider_for(runtime, profile)` helper returns
--      `MockProvider::text(mock_response)`; when None, the default
--      "Acknowledged." is used. Intended for test fixtures + dev-mode
--      overrides. Real providers dispatch in M7 bypasses this field.
--
-- Forward-only; per ADR-0012 we do not author `down` scripts. Pre-CH-02
-- rows deserialise with `#[serde(default)] = None` on
-- `domain::model::nodes::AgentProfile::mock_response` — no backfill
-- required.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. agent_profile.mock_response — ADR-0032 D32.2
-- ----------------------------------------------------------------------------
--
-- Nullable. Absence = default behaviour (MockProvider returns
-- "Acknowledged."). Validated at PATCH time only as free-form UTF-8;
-- no length cap at M5 (tests driving this field are author-controlled).

DEFINE FIELD mock_response ON agent_profile TYPE option<string>;
