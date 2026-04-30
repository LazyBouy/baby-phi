-- ============================================================================
-- 0010_consent_full_shape.surql
--
-- Schema evolution for CH-09 (D-new-04 closure).
--
-- Lifts the Consent record from its 5-field M2 stub to the 11-field shape
-- mandated by `concepts/permissions/06-multi-scope-consent.md`
-- §"Consent Node (New Node Type)" lines 351-363. CH-09 ships the data
-- shape; CH-10 will add the state-transition function (drift D-new-05).
--
-- Concept-doc field set:
--   consent_id, agent_id, scope.org, scope.templates, scope.actions,
--   state, requested_at, responded_at, revoked_at, revocable, provenance
--
-- The `consent` table was first scaffolded by 0001_initial.surql with
-- the M2 fields (subordinate, scoped_to, granted_at, revoked_at). CH-09
-- redefines the table cleanly via REMOVE TABLE IF EXISTS — safe because
-- there is no production data: the only Consent constructor in the
-- workspace is the test fixture at `store/tests/repository_test.rs`.
-- This is documented in ADR-0045 §D45.5.
--
-- Forward-only per ADR-0012; no down script.
-- Idempotency note: REMOVE TABLE IF EXISTS + DEFINE TABLE makes this
-- migration safe under repeated runs (per the existing _migrations
-- ledger semantics — ADR-0042 §D42.3 conforming-criteria #5).
-- ============================================================================

REMOVE TABLE IF EXISTS consent;

DEFINE TABLE consent SCHEMAFULL;

-- The subordinate giving consent. Concept doc 06 line 354.
DEFINE FIELD agent_id     ON consent TYPE string ASSERT $value != NONE;

-- Nested scope: { org, templates, actions }. FLEXIBLE TYPE object
-- because the inner shape is owned by the typed Rust `ConsentScope`
-- struct + serde, not the SurrealDB schema. Concept doc 06 lines 355-357.
DEFINE FIELD scope        ON consent FLEXIBLE TYPE object ASSERT $value != NONE;

-- Lifecycle state. The default mirrors the `Default for ConsentState`
-- impl (Acknowledged) so legacy rows that omit the field decode cleanly.
-- Concept doc 06 §"Consent Lifecycle" lines 397-404.
DEFINE FIELD state        ON consent TYPE string DEFAULT "acknowledged";

-- When the consent was first requested. Concept doc 06 line 359.
-- Stored as RFC3339 string (matches the `created_at` shape across the
-- M5 session + composites tables; serde converts DateTime<Utc> by
-- default).
DEFINE FIELD requested_at ON consent TYPE string ASSERT $value != NONE;

-- When the subordinate responded (Acknowledged or Declined). NULL until
-- the subordinate acts. Concept doc 06 line 360.
DEFINE FIELD responded_at ON consent TYPE option<string>;

-- When the subordinate later withdrew consent. NULL unless state is
-- Revoked. Concept doc 06 line 361.
DEFINE FIELD revoked_at   ON consent TYPE option<string>;

-- Whether the subordinate can later withdraw consent. Concept doc 06
-- line 362.
DEFINE FIELD revocable    ON consent TYPE bool DEFAULT true;

-- Audit trail. Conventional format: `agent:{subordinate_id}@{event}`
-- per concept doc 06 line 363. Stays untyped at v0 (a typed Provenance
-- enum is deferred to v1 — ADR-0045 §D45.10).
DEFINE FIELD provenance   ON consent TYPE string DEFAULT "";
