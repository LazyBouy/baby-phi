<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 2 of fresh-install journey -->

# 05 — Platform Defaults

## 2. Page Purpose + Primary Actor

The platform admin configures **platform-wide defaults** that newly-created orgs inherit unless they customise:

- **Default consent policy** (`implicit` / `one_time` / `per_session`)
- **Default audit class** (`silent` / `logged` / `alerted`)
- **Auth Request retention** (active window in days, archival policy, optional `delete_after_years`)
- **Execution limit template** (default `max_turns`, `max_tokens`, `max_duration_secs`, `max_cost_usd` that orgs inherit)
- **Default authority templates enabled** (which of A–E fire by default at org adoption)

These are not immutable — orgs override at creation time. The page edits the **platform-level defaults**, not any org's specific settings.

**Primary actor:** platform admin Human Agent.

## 3. Position in the Journey

- **Phase:** 2 of 9 — page 4 of 4.
- **Depends on:** Phase 1 complete.
- **Enables:** Phase 3 (org creation wizard) uses these values as the pre-filled defaults in its forms.

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ Platform > Defaults                                [Save changes]│
├──────────────────────────────────────────────────────────────────┤
│ Default consent policy    ○ implicit  ● one_time  ○ per_session │
│ Default audit class       ○ silent  ● logged  ○ alerted         │
│                                                                   │
│ Auth Request retention                                            │
│   active_window_days      [ 90 ]                                  │
│   archived_retrieval      [ human_required ▾ ]                   │
│   delete_after_years      [ (none)   ]                            │
│                                                                   │
│ Execution limits (default, per-agent)                             │
│   max_turns               [ 50 ]     max_tokens      [ 100_000 ] │
│   max_duration_secs       [ 3600 ]   max_cost_usd    [ 5.00    ] │
│                                                                   │
│ Authority templates enabled at org adoption (default)             │
│   ☑ A  ☑ B  ☐ C  ☐ D  (E is always available)                   │
│                                                                   │
│ Last edited: 2026-04-15 09:30 by agent:alex                      │
└──────────────────────────────────────────────────────────────────┘
```

Empty state: N/A — defaults are pre-populated from the install-time factory defaults.

## 5. Read Requirements

- **R-ADMIN-05-R1:** The page SHALL display the current platform-wide default values with the most recent edit timestamp and editor agent id.
- **R-ADMIN-05-R2:** For each field, the page SHALL display the **install-time factory default** as a hint so the admin knows what they are departing from.

## 6. Write Requirements

- **R-ADMIN-05-W1:** The admin SHALL be able to update any default value. On save, a single Template E Auth Request captures all changes atomically (not one per field).
- **R-ADMIN-05-W2:** Validation: numeric fields have sensible bounds (e.g., `active_window_days >= 1`, `max_turns >= 1`, `max_cost_usd >= 0.01`); enum fields restricted to the concept-defined enums.
- **R-ADMIN-05-W3:** Changes to defaults SHALL NOT retroactively change existing orgs — only orgs created AFTER the change see the new defaults. Existing orgs' effective values remain as captured in their own configs.

## 7. Permission / Visibility Rules

- **Page access** — `[read, list]` on `control_plane_object:platform-defaults`. Platform admin.
- **Edit** — `[allocate]` on `control_plane_object:platform-defaults`. Platform admin.

## 8. Event & Notification Requirements

- **R-ADMIN-05-N1:** On save (W1), emit audit event `PlatformDefaultsUpdated { diff: {field: {old, new}}, updated_by, audit_class: alerted }`. Changes to defaults are inherently sensitive — always alerted.
- **R-ADMIN-05-N2:** The save dialog SHALL show a preview of the diff before commit.

## 9. Backend Actions Triggered

Save (W1) triggers:
- Template E Auth Request + auto-approval (platform admin on their own platform default).
- Platform `control_plane_object:platform-defaults` updated.
- Alerted audit event.
- No cascade to existing orgs (W3).

## 10. API Contract Sketch

```
GET  /api/v0/platform/defaults
     → 200: { values: {...}, factory_defaults: {...}, last_edited_at, last_edited_by }

PUT  /api/v0/platform/defaults
     Body: { consent_policy, audit_class, auth_request_retention: {...}, execution_limits: {...}, authority_templates_enabled: [...] }
     → 200: { applied_at, audit_event_id, diff: {...} }
     → 400: Validation errors
```

## 11. Acceptance Scenarios

**Scenario 1 — harden defaults for a regulated deployment.**
*Given* a fresh install whose factory defaults are `consent_policy: implicit`, `audit_class: logged`, *When* the platform admin changes defaults to `consent_policy: per_session` and `audit_class: alerted` (matching the profile of a regulated-enterprise deployment per [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md)), *Then* the next org created via [06-org-creation-wizard.md](06-org-creation-wizard.md) has its consent/audit fields pre-filled with the new stricter values (the admin can still customise down if appropriate).

**Scenario 2 — changes do not retroact.**
*Given* the platform has one existing org `minimal-startup` with `consent_policy: implicit`, *When* the admin changes the platform default to `per_session`, *Then* `minimal-startup`'s `consent_policy` remains `implicit`; only orgs created after the change inherit the new default.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/06 § Three Consent Policies](../../concepts/permissions/06-multi-scope-consent.md#three-consent-policies).
- [concepts/permissions/07 § `audit_class` Composition Through Templates](../../concepts/permissions/07-templates-and-tools.md#audit_class-composition-through-templates).
- [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy).
- [concepts/permissions/07 § Standard Organization Template](../../concepts/permissions/07-templates-and-tools.md#standard-organization-template) — the template whose defaults this page customises.

**Related admin pages:**
- [06-org-creation-wizard.md](06-org-creation-wizard.md) — consumer of these defaults.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md) — its shape implies the kind of defaults a regulated deployment chooses.
