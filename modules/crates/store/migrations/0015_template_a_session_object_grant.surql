-- ============================================================================
-- 0015_template_a_session_object_grant.surql
--
-- CH-15 / ADR-0054 §D54.4 — backfill the paired session-object Grant
-- for every legacy Template A grant. Pre-CH-15, `Template A` (per-lead
-- authority) minted a single Grant per HAS_LEAD edge:
--   * `resource_uri = "project:<uuid>"`
--   * `fundamentals = ["tag"]`
--   * `action = ["read", "inspect", "list"]`
--
-- CH-15 extends `fire_grant_on_lead_assignment` (per ADR-0054 §D54.3)
-- to mint TWO grants per fire — the existing project-resource grant
-- AND a paired `session_object/project:<uuid>` grant covering session
-- instances tagged `project:<uuid>`. Concept doc 07 §"Template A —
-- Project Lead Authority" specifies the lead has `[read, inspect,
-- list]` on every session tagged `project:P`; CH-15 materialises
-- that on the wire.
--
-- This migration walks every ACTIVE legacy Template A grant — i.e.,
-- a grant that:
--   1. has `descends_from` pointing at an `auth_request` whose `kinds`
--      CONTAINS `'#template:a'` (Template A adoption AR), AND
--   2. has `resource_uri STARTS_WITH 'project:'`, AND
--   3. has `revoked_at = NONE` (still live).
--
-- For each match, the migration CREATES a paired grant with the same
-- holder + action + descends_from + delegable + issued_at +
-- approval_mode + audit_class as the legacy grant; the new grant
-- differs only in `resource_uri` (`session_object/project:<uuid>`)
-- and `fundamentals` (`['data_object', 'tag']` per the SessionObject
-- composite expansion in concept doc 01).
--
-- Idempotency: re-runs of this migration are safe. Before issuing the
-- paired grant, the migration first checks whether a paired grant
-- with the SAME `descends_from` and `resource_uri` already exists —
-- this guards against re-running the migration on a tree that
-- post-CH-15 firings have already populated. Per ADR-0033 §D33.2 the
-- migration runner reads `_migrations` ledger before re-applying a
-- version, so the first-class idempotency arm is the ledger; this
-- WHERE-NOT-EXISTS guard is belt-and-braces against ledger drift.
--
-- The migration is forward-only per ADR-0012; no down script.
--
-- ## Authority Chain preservation (ADR-0053 / CH-14)
--
-- Both grants `descends_from` the same Template A adoption AR.
-- `walk_provenance_chain` (CH-14) traverses the additional row
-- transparently; revocation of the adoption AR cascades to BOTH
-- grants via the existing single-hop revoke path.
--
-- ## Performance shape
--
-- Single SurrealDB transaction; iterates O(active Template A grants)
-- rows. Production org count + per-org lead count is bounded; the
-- loop is sub-second on every realistic dataset.
-- ============================================================================

-- Loop over every legacy Template A project-resource grant that is
-- still live + does NOT already have a paired session_object grant
-- under the same provenance.
--
-- The paired grant's URI uses the selector-grammar form
--   "tags contains project:<uuid> AND tags contains #kind:session"
-- so the Permission Check engine resolves it to a `Selector::Bool`
-- predicate at Step 3 (matching the runtime
-- `fire_grant_on_lead_assignment` shape verbatim).
FOR $g IN (
    SELECT
        record::id(id) AS legacy_id,
        holder_kind,
        holder_id,
        action,
        resource_uri,
        descends_from,
        delegable,
        issued_at,
        approval_mode,
        audit_class
    FROM grant
    WHERE
        revoked_at = NONE
        AND string::starts_with(resource_uri, "project:")
        AND descends_from != NONE
        AND descends_from IN (
            SELECT VALUE record::id(id)
            FROM auth_request
            WHERE kinds CONTAINS "#template:a"
        )
        -- Skip grants that already have a paired session-tag grant
        -- (idempotency belt-and-braces). Match on the URI shape +
        -- shared descends_from to identify a paired-grant pair.
        AND descends_from NOT IN (
            SELECT VALUE descends_from
            FROM grant
            WHERE
                revoked_at = NONE
                AND string::starts_with(resource_uri, "tags contains project:")
        )
) {
    -- legacy_id = "project:<uuid>" → extract the suffix after the
    -- ":" to build the selector-grammar URI. The project-tag
    -- predicate uses the string-literal form ("project:<uuid>") so
    -- UUIDs starting with a digit parse cleanly via the grammar's
    -- literal_tag rule (concept-09 §"Tag Forms"; identifier requires
    -- `[A-Za-z_]` first char which excludes digit-leading UUIDs).
    LET $project_suffix = string::slice($g.resource_uri, 8);
    LET $session_uri = "tags contains \"project:" + $project_suffix
        + "\" AND tags contains #kind:session";
    LET $new_id = rand::uuid::v4();
    CREATE type::thing("grant", $new_id) SET
        holder_kind     = $g.holder_kind,
        holder_id       = $g.holder_id,
        action          = $g.action,
        resource_uri    = $session_uri,
        fundamentals    = ["data_object", "tag"],
        descends_from   = $g.descends_from,
        delegable       = $g.delegable,
        issued_at       = $g.issued_at,
        revoked_at      = NONE,
        approval_mode   = $g.approval_mode,
        audit_class     = $g.audit_class;
};
