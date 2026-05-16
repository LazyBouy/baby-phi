-- ============================================================================
-- 0017_add_owns_relation.surql
--
-- Schema evolution for CH-25 / ADR-0060 §D60.1 (Agent-as-creator-and-owner).
--
-- Adds the `owns` relation table backing the new `Edge::Owns` variant
-- (Agent → Org/Project ownership semantics, distinct from the generic
-- `OWNED_BY` Memory-as-Resource relation).
--
-- F1.b user-lock (DIVERGENT from planner-recommended F1.a): adds a NEW
-- variant on the `Edge` enum + a dedicated `owns` relation table,
-- rather than relaxing the `Resource` trait for `OrgId` / `ProjectId`.
-- Org and Project STAY Principal-only in the v0 ontology; the typed
-- payload (`OwnedResourceId::Org(_) | OwnedResourceId::Project(_)`)
-- carries the resource-end kind without trait relaxation.
--
-- Per Decision-3 user-lock (Shape B owner): `lead_agent_id` is BOTH
-- creator AND owner at v0 for both Shape A and Shape B materialise
-- paths. The AR-submitter (Shape B) chose the lead; that's the owner.
-- Future per-org co-ownership is an M6+ refinement.
--
-- Idempotency: re-running the migration is a no-op (DEFINE TABLE is
-- idempotent in SurrealDB 2.x). Forward-only per ADR-0012; no down
-- script.
--
-- Concept doc: `concepts/core-philosophy.md:9,23,24` (*"Agent owns
-- Organization"*, *"Agent create Projects"*, *"Agents own
-- Resources"*). The owner-grant Permission Check rule (ADR-0060
-- §D60.3) synths `[Action::Allocate, Action::Transfer]` grants over
-- each owned Org/Project for the owner-Agent.
-- ============================================================================

DEFINE TABLE owns SCHEMAFULL TYPE RELATION;

DEFINE FIELD edge_id     ON owns TYPE string ASSERT $value != NONE;
-- target_kind discriminates the OwnedResourceId enum sub-variant
-- on read so list_agent_owned_orgs / list_agent_owned_projects can
-- filter without inspecting the heterogeneous `out` field.
DEFINE FIELD target_kind ON owns TYPE string ASSERT $value INSIDE ['org', 'project'];
DEFINE FIELD created_at  ON owns TYPE string ASSERT $value != NONE;

-- Idempotency guard: at most one owns edge per (agent, target) pair.
-- Re-emitting the same Owns edge from an idempotent compound-tx
-- replay (e.g., apply_org_creation retry) should not double-insert.
DEFINE INDEX owns_unique
    ON owns
    FIELDS in, out
    UNIQUE;
