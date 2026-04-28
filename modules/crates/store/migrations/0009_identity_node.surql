-- ============================================================================
-- 0009_identity_node.surql
--
-- Schema evolution for CH-16 (D-new-01 + D-new-23 closure).
--
-- Materialises the four-field Identity node per
-- `concepts/agent.md` § "Identity (Emergent, Event-Driven)" + § "Identity
-- Node Content — Provisional Direction" (the v0 commitment). One row per
-- LLM agent; UNIQUE on `agent_id`.
--
-- Eager creation timing: `apply_agent_creation` writes the row in the
-- same compound tx as Agent / InboxObject / OutboxObject for every
-- `AgentKind::Llm` agent (ADR-0038 §D38.1). Human-kind agents do NOT
-- get a row (concept-`human-agent.md` § "No Identity"; defensive guard
-- at `Repository::upsert_identity` + preventive guard inside
-- `apply_agent_creation`). The defensive guard is also enforced in the
-- SurrealQL via an IF/THROW block at write time — see
-- `store::repo_impl::SurrealRepository::upsert_identity`.
--
-- Forward-only per ADR-0012; no down script.
-- ============================================================================

-- The `identity` table already exists from 0001_initial.surql as a
-- scaffolded carrier (SCHEMAFULL with only `created_at TYPE string`).
-- CH-16 keeps the table reference intact and just defines the new
-- field set. The `created_at` field is redefined from string to
-- datetime to match the v0 commitment shape (concept-`agent.md` line
-- 326 expects timestamps, not opaque strings).

DEFINE FIELD OVERWRITE agent_id         ON identity TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE self_description ON identity TYPE string DEFAULT "";

-- `lived` and `witnessed` are inline structs (ADR-0038 §D38.4 — they are
-- query-shape detail of the Identity row, not first-class nodes per
-- ontology.md). FLEXIBLE TYPE object lets the four nested-field shapes
-- (sessions_completed, ratings_window, skills, specializations,
-- memories_extracted, subordinates_observed, extraction_scope_distribution)
-- evolve in M6+ without forcing migration churn.
DEFINE FIELD OVERWRITE lived            ON identity FLEXIBLE TYPE object;
DEFINE FIELD OVERWRITE witnessed        ON identity FLEXIBLE TYPE object;

-- Embedding vector. Empty at create per ADR-0038 §D38.3 (Fork 2);
-- populated when the platform-level embedding provider integrates
-- (M6-DEFERRED-03). Concept-`agent.md` line 338: cosine similarity is
-- only meaningful within a single model's vector space.
DEFINE FIELD OVERWRITE embedding        ON identity TYPE array DEFAULT [];
DEFINE FIELD OVERWRITE embedding.*      ON identity TYPE float;

-- `created_at` stays `string` (RFC3339) — same shape as the M5 session
-- + composites tables. `updated_at` uses the same shape.
-- Domain-side `DateTime<Utc>` serializes to RFC3339 string by default.
DEFINE FIELD OVERWRITE created_at       ON identity TYPE string;
DEFINE FIELD OVERWRITE updated_at       ON identity TYPE string;

-- One identity per LLM agent. UNIQUE on agent_id matches the
-- `agent_catalog_entry_identity` precedent from migration 0005.
DEFINE INDEX identity_agent_id ON identity FIELDS agent_id UNIQUE;
