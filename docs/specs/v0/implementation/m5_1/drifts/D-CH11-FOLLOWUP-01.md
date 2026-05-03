<!-- Last verified: 2026-05-03 by Claude Code (filed by CH-11 retrospective; cycle hex `d5428c43`) -->

# D-CH11-FOLLOWUP-01 — `Project.deadline_at` missing; CH-11 falls back to `now+24h` for `ApprovalTimeout::ProjectDuration`

## Identification
- **ID**: D-CH11-FOLLOWUP-01
- **Phase of origin**: CH-11 retrospective (cycle hex `d5428c43`)
- **Discovery source**: `cycle-audit-findings`
- **Date discovered**: 2026-05-03
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap with implementer-applied fallback
- **Severity**: LOW
- **Tags**: `consent-policy`, `project-shape`, `deadline`
- **Blocks**: nothing at v0 (the fallback works; full fidelity needed when M6+ adds `Project.deadline_at`)
- **Blocked-by**: M6+ Project enrichment work (TBD chunk)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Per-Session Consent" lines 322–349 — "approval_timeout: project_duration | "24h" (default project_duration); for shape-C multi-project sessions, deadline = max(project.deadline_at across the session's projects)".
- **Concept claim**: `ApprovalTimeout::ProjectDuration` resolves to `project.deadline_at` for shapes A/B/C; for shape-C (multi-project), the deadline is the maximum of all the session's projects' `deadline_at` fields; for shape D (no project), falls back to `now + 24h` (the `"24h"` default in the concept doc's syntax).
- **Contradiction**: `Project` struct in [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) has no `deadline_at` field at this milestone. CH-11's `compute_consent_deadline` helper at [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs) falls back to `now + 24h` for `ApprovalTimeout::ProjectDuration` regardless of session shape — for shape A/B/C this is too permissive (concept-doc says project-bounded), and the shape-C multi-project max() semantic is unimplemented.
- **Classification**: `partially-honored`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: `compute_consent_deadline` for `ApprovalTimeout::ProjectDuration` reads `project.deadline_at` for shapes A/B/C; falls back to `now + 24h` for shape D only.
- **Reality**: Falls back to `now + 24h` always, because `Project` has no `deadline_at` field. CH-11's plan §3.B + the launch-handler implementation comment + ADR-0048 §D48.4 all explicitly acknowledged this as a deferred element.

## Severity rationale (LOW)

- **Functional impact**: minimal at v0. Shape A/B/C sessions get `now + 24h` (same as shape D); operators can still configure `ApprovalTimeout::Fixed(d)` to override. The default-response `Deny` semantic + the sweeper from CH-10 still flip TimedOut consents on schedule. Engine response-table behaviour is untouched.
- **Concept-doc fidelity impact**: partial — the **shape** of the timeout works, but the per-project derivation + multi-project max() are absent.
- **Operator impact**: operators that want shape-A/B/C consent deadlines tied to project-end have no v0 path; they must use `Fixed(d)` workarounds or accept 24h.

## Resolution path

Closure requires:
1. Adding `deadline_at: Option<DateTime<Utc>>` (or similar) to the `Project` struct.
2. Migration that adds the column (idempotent per ADR-0042 §D42.3 #5).
3. Updating `compute_consent_deadline` to read `project.deadline_at` for shapes A/B/C.
4. Implementing the `max(project.deadline_at)` aggregation for shape-C multi-project sessions.
5. Acceptance test additions: deadline derivation per shape × per `ApprovalTimeout` variant.

This is naturally bundled with **M6+ Project enrichment work** (project status, archival, hierarchy, deadline tracking). Not a CH-12, CH-13, or other M5-close blocker.

## Owning chunk
- **Closes at**: M6+ Project enrichment chunk (TBD).
- **Tracking**: this drift file. CH-11 retrospective (`plan/build/ch-11-per-session-consent-gating-d5428c43/retrospective.md` §6 Q1) flagged the decision to file vs accept; user approved filing.

## Lifecycle
- 2026-05-03 — `discovered` (filed by CH-11 retrospective; user-approved at retro review).
