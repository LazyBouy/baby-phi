-- ============================================================================
-- 0013_per_session_consent_gating.surql
--
-- Schema evolution for CH-11 / ADR-0048 — adds three columns:
--   * `approval_mode` on `grant` — typed `ApprovalMode` enum, FLEXIBLE
--     because the wire shape is a tag-payload object (`{"kind": ...}`)
--     and the `subordinate_required` variant carries a payload
--     (`policy: ConsentPolicy`).
--   * `approval_timeout` on `organization` — typed `ApprovalTimeout`
--     enum, FLEXIBLE for the same tag-payload reason (the `fixed`
--     variant carries a `duration: chrono::Duration` payload).
--   * `approval_timeout_default_response` on `organization` —
--     typed `TimeoutResponse` enum (snake-case string), TYPE string
--     DEFAULT "deny" per concept doc 06 line 349 ("absence of consent
--     is not consent").
--
-- ConsentScope.session_id — no schema change required because `scope`
-- is already FLEXIBLE TYPE object per migration 0010 + ADR-0045
-- §D45.5; the new optional field rides under the FLEXIBLE shield.
--
-- Concept doc: `permissions/06-multi-scope-consent.md`
--   * line 244 — "Templates auto-issue grants with
--     `approval_mode: subordinate_required`" (covers `grant.approval_mode`).
--   * lines 322 + 336–347 — `approval_timeout: project_duration`
--     default + per-shape mapping (covers `organization.approval_timeout`).
--   * line 349 — `approval_timeout_default_response: deny` default
--     (covers `organization.approval_timeout_default_response`).
--
-- Forward-only per ADR-0012; no down script. Idempotent under
-- repeated runs of the migration runner per ADR-0042 §D42.3 #5
-- (`DEFINE FIELD OVERWRITE` is the explicit idempotent form).
-- ============================================================================

DEFINE FIELD OVERWRITE approval_mode ON grant FLEXIBLE TYPE object;
DEFINE FIELD OVERWRITE approval_timeout ON organization FLEXIBLE TYPE object;
DEFINE FIELD OVERWRITE approval_timeout_default_response ON organization TYPE string DEFAULT "deny";
