-- ============================================================================
-- 0016_template_a_session_object_grant_add_observe.surql
--
-- CH-17 / ADR-0055 §D55.9 — extend every legacy Template A grant
-- (project-resource + paired session-object) to carry the
-- `Action::Observe` verb at the tail of its `action` array. Pre-CH-17,
-- Template A's `fire_grant_on_lead_assignment` minted
-- `[read, inspect, list]`; CH-17 extends the production minter to
-- `[read, inspect, list, observe]` so the SSE live-event tail
-- handler's `[Observe]`-only manifest (built via
-- `build_session_observe_manifest`) covers at Step 3 against the lead's
-- existing session-object grant. This migration backfills the same
-- `"observe"` entry onto every legacy grant so pre-CH-17-era DBs
-- converge with fresh DBs after upgrade.
--
-- Concept rationale:
--   * `concepts/permissions/03-action-vocabulary.md` line 22 lists
--     Observability's three verbs (`observe, log, attest`); line 44
--     states *"Discovery, Authority, and Observability apply
--     universally (every fundamental has list/inspect,
--     delegate/allocate/transfer, and observe/log/attest)"*.
--   * `concepts/permissions/05-memory-sessions.md` §"Authority Templates"
--     line 309 specifies Template A grants on `session_object/project:<id>`;
--     the live-watch-the-project-sessions-stream affordance is a
--     natural observability surface for the lead.
--
-- Scope:
--   This migration walks every grant that:
--     1. has `descends_from` pointing at an `auth_request` whose `kinds`
--        CONTAINS `'#template:a'` (Template A adoption AR), AND
--     2. has `revoked_at = NONE` (still live), AND
--     3. does NOT already contain `"observe"` in its `action` array.
--
-- For each match, `array::push` adds `"observe"` to the action array.
-- Every other field (holder, descends_from, delegable, issued_at,
-- approval_mode, audit_class, fundamentals, resource_uri) is preserved
-- verbatim.
--
-- ## Idempotency
--
-- The `array::find_index($g.action, "observe") = NONE` guard makes the
-- migration body a no-op on grants that already carry observe (whether
-- because the migration ran before, or because the grant was minted
-- post-CH-17 by the extended `fire_grant_on_lead_assignment`). Per
-- ADR-0033 §D33.2 the migration runner reads `_migrations` ledger
-- before re-applying a version — the runner's first-class idempotency
-- arm is the ledger; this WHERE-NOT-EXISTS guard is belt-and-braces
-- against ledger drift.
--
-- The migration is forward-only per ADR-0012; no down script.
--
-- ## Authority Chain preservation (ADR-0053 / CH-14)
--
-- The migration ONLY touches the `action` array. `descends_from` is
-- untouched; `walk_provenance_chain` (CH-14) traverses the modified
-- grant transparently. Revocation cascades are unaffected.
--
-- ## Performance shape
--
-- Single SurrealDB transaction; iterates O(active Template A grants)
-- rows. Production org count + per-org lead count is bounded; the
-- loop is sub-second on every realistic dataset. Mirrors 0015's
-- shape exactly (FOR-loop over filtered grants → per-row UPDATE) so
-- operator runbooks can reuse the 0015 mental model.
-- ============================================================================

FOR $g IN (
    SELECT
        record::id(id) AS legacy_id,
        action
    FROM grant
    WHERE
        revoked_at = NONE
        AND descends_from != NONE
        AND descends_from IN (
            SELECT VALUE record::id(id)
            FROM auth_request
            WHERE kinds CONTAINS "#template:a"
        )
        -- Idempotency belt-and-braces: skip grants that already
        -- carry `"observe"` at any position in the action array.
        AND array::find_index(action, "observe") = NONE
) {
    UPDATE type::thing("grant", $g.legacy_id) SET
        action = array::push($g.action, "observe");
};
