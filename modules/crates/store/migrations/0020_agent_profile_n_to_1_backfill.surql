-- ============================================================================
-- 0020_agent_profile_n_to_1_backfill.surql
--
-- Data backfill for CH-28 / ADR-0063 §D63.3 + §D63.5 (AgentProfile cardinality
-- flip 1:1 → N:1 — data-only DML, paired with 0019 schema-only DDL).
--
-- F3.b user-lock (DIVERGENT from planner-recommended F3.a): the migration is
-- split into 0019 (schema-only) + 0020 (data-only). 0019 defines the new
-- `blueprint` table + renamed `uses_profile` relation + two NEW Blueprint-
-- pointer relations. 0020 (this file) backfills the data:
--
--   (a) For each EXISTING `agent_profile` row, UPSERT a template-tier
--       `blueprint` row at deterministic id `blueprint:bp_<agent_profile_id>`
--       carrying `agent_id = NONE` (template discriminator per ADR-0063
--       §D63.1) + copies of the pre-CH-28 override fields
--       (`parallelize` / `model_config_id` / `mock_response`) read from the
--       agent_profile row's stored values. After 0019's `REMOVE FIELD`, the
--       field DEFINITIONS are gone but the stored row data persists (per the
--       SurrealDB 2.x REMOVE FIELD semantic — un-asserted properties remain
--       readable on the row), which is what the half-migrated-state
--       tolerance window relies on (per F3.b + ADR-0063 §D63.12).
--   (b) RELATE the EXISTING `agent_profile` row to its NEW template
--       `blueprint` via the `agent_profile_uses_blueprint` edge, with a
--       deterministic edge id `agent_profile_uses_blueprint:apub_<profile_id>`.
--   (c) For each EXISTING `has_profile` edge row, RELATE the corresponding
--       `agent → agent_profile` pair via the renamed `uses_profile` edge
--       (deterministic id `uses_profile:up_<has_profile_edge_id>`), then
--       DELETE the legacy `has_profile` row. The legacy `has_profile` TABLE
--       remains DEFINED (per 0019 header — table removal is a future-cycle
--       concern under M7 NFR-observability per ADR-0063 §D63.12); only the
--       row-level data is rewritten.
--
-- Override-tier `blueprint` rows (`agent_id` set to a concrete AgentId) are
-- NOT created at backfill — they are written lazily by application code
-- (P1-BLUEPRINT-STRUCT `upsert_agent_blueprint_override` repository method)
-- the first time an agent's per-agent override is mutated. Per ADR-0063
-- §D63.1, every Agent inherits its template-Blueprint's defaults until an
-- override row is written.
--
-- Idempotency: re-running 0020 on a post-apply database is a no-op:
--   - UPSERT type::thing('blueprint', $bp_id) CONTENT { ... } is by definition
--     idempotent on a deterministic id (CREATE-or-UPDATE semantic). The
--     CONTENT clause is data-stable (same source row → same target body
--     across re-runs).
--   - The `agent_profile_uses_blueprint` edge row is RELATEd with a
--     deterministic id, guarded by an `IF` block that selects the count of
--     existing rows at that id and only RELATEs when zero. SurrealDB 2.x
--     RELATION tables reject INSERT-tier writes (the relation invariant is
--     enforced at the executor); RELATE is the only ingress path.
--   - The `uses_profile` edge row is RELATEd via the same IF-guarded
--     pattern at deterministic id derived from the source `has_profile`
--     row id.
--   - The DELETE on `has_profile` is naturally idempotent (deleting a row
--     that no longer exists is a no-op in SurrealQL).
--
-- Half-migrated-state tolerance window (per ADR-0063 §D63.12): operators
-- may pause between 0019 first-apply and 0020 first-apply for inspection.
-- During the window, runtime reads via `get_agent_profile_for_agent` MUST
-- tolerate either shape — the `read_agent_profile_via_blueprint_or_fallback`
-- runtime helper (lands at P1-BLUEPRINT-STRUCT per the split-phase ordering)
-- bridges the window. This migration does NOT change the runtime read path;
-- it only rewrites the persisted data.
--
-- Forward-only per ADR-0012; no down script. The split (vs single composite)
-- is a NEW precedent codified by ADR-0063 §D63.3 (`schema-migrations.md`
-- §"Split-migration pattern (NEW at CH-28)").
--
-- Concept doc anchors:
--   - `concepts/agent.md` §"Soul (Immutable Born Structure)" (template-
--     Blueprint sharing semantic).
--   - `concepts/ontology.md` §"Edge Types" L98 (cardinality flip + verb
--     rename + 2 NEW Blueprint-pointer edges).
--   - `implementation/m1/architecture/schema-migrations.md` §"Split-
--     migration pattern (NEW)".
--   - `implementation/m6/architecture/agent-profile-cardinality.md` §5
--     (split-migration walkthrough — 0019 + 0020 bodies + idempotency).
-- ============================================================================

-- ---- (a) Template-Blueprint backfill --------------------------------------
--
-- For each pre-existing `agent_profile` row, UPSERT a template-tier blueprint
-- row at a deterministic id derived from the agent_profile id. Reading
-- `parallelize` / `model_config_id` / `mock_response` from the agent_profile
-- row is safe in the post-0019 state — 0019 REMOVEd the SCHEMAFULL field
-- DEFINITIONS but the stored row data persists (SurrealDB 2.x semantic —
-- un-asserted properties remain readable). The reads return NONE for any
-- field that was never set on the legacy row.
--
-- `bp_<id>` prefix yields a record-id that is collision-safe with the
-- override-tier blueprint rows (which use `ob_<agent_id>` prefix — written
-- lazily by `upsert_agent_blueprint_override` at P1-BLUEPRINT-STRUCT).

FOR $profile IN (SELECT record::id(id) AS pid, parallelize, model_config_id, mock_response FROM agent_profile) {
    LET $bp_id = "bp_" + $profile.pid;
    UPSERT type::thing('blueprint', $bp_id) CONTENT {
        agent_id: NONE,
        parallelize: $profile.parallelize,
        model_config_id: $profile.model_config_id,
        mock_response: $profile.mock_response,
        created_at: <string>time::now()
    } RETURN NONE;

    -- (b) Edge: agent_profile → blueprint (template pointer). Deterministic
    -- id `apub_<profile_id>` makes the edge presence query-stable across
    -- re-runs. The IF-block guards against duplicate RELATE — RELATION
    -- tables in SurrealDB 2.x reject INSERT-tier writes, so RELATE is the
    -- only ingress path; we hand-roll the existence guard.
    LET $apub_id = "apub_" + $profile.pid;
    LET $existing_apub = SELECT count() FROM agent_profile_uses_blueprint WHERE id = type::thing('agent_profile_uses_blueprint', $apub_id) GROUP ALL;
    LET $apub_count = IF array::len($existing_apub) = 0 THEN 0 ELSE $existing_apub[0].count END;
    LET $apub_in = type::thing('agent_profile', $profile.pid);
    LET $apub_out = type::thing('blueprint', $bp_id);
    IF $apub_count = 0 {
        RELATE $apub_in -> agent_profile_uses_blueprint -> $apub_out
            SET id = type::thing('agent_profile_uses_blueprint', $apub_id) RETURN NONE;
    };
};

-- ---- (c) has_profile → uses_profile edge rewrite + legacy delete --------
--
-- For each pre-existing `has_profile` edge row, write a corresponding
-- `uses_profile` edge (deterministic id derived from the legacy edge id) and
-- then DELETE the legacy row. The two operations are not transaction-
-- atomic at the SurrealDB embedded backend (DDL/DML mixing precludes
-- BEGIN/COMMIT here), but the deterministic-id UPSERT pattern ensures that
-- a partial application (e.g., uses_profile written, has_profile delete
-- not yet) is recoverable on re-run — the IF NOT EXISTS guards skip the
-- already-written `uses_profile` row and the DELETE WHERE id = ... is
-- naturally idempotent.

FOR $hp IN (SELECT record::id(id) AS eid, in AS hp_in, out AS hp_out FROM has_profile) {
    LET $up_id = "up_" + $hp.eid;
    -- IF-guarded RELATE — RELATION tables reject INSERT-tier writes; the
    -- existence guard makes the RELATE idempotent across re-runs. The
    -- deterministic id shape `uses_profile:up_<has_profile_edge_id>`
    -- makes the rewrite 1:1 traceable from legacy to new. Each Thing
    -- value used at RELATE's from/to positions must be a bound variable
    -- (not a field access expression) per SurrealQL grammar.
    LET $existing_up = SELECT count() FROM uses_profile WHERE id = type::thing('uses_profile', $up_id) GROUP ALL;
    LET $up_count = IF array::len($existing_up) = 0 THEN 0 ELSE $existing_up[0].count END;
    LET $up_in = $hp.hp_in;
    LET $up_out = $hp.hp_out;
    IF $up_count = 0 {
        RELATE $up_in -> uses_profile -> $up_out
            SET id = type::thing('uses_profile', $up_id) RETURN NONE;
    };
    DELETE has_profile WHERE id = type::thing('has_profile', $hp.eid) RETURN NONE;
};
