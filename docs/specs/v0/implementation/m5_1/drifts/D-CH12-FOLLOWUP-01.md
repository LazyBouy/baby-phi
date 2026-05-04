<!-- Last verified: 2026-05-04 by Claude Code (filed by CH-12 retrospective; cycle hex `6a748175`) -->

# D-CH12-FOLLOWUP-01 — Session structural-tag emission gap (6 M6+ categories not yet auto-emitted on Session creation)

## Identification
- **ID**: D-CH12-FOLLOWUP-01
- **Phase of origin**: CH-12 retrospective (cycle hex `6a748175`)
- **Discovery source**: `cycle-audit-findings` (carry-forward gap surfaced at plan §3.C + cycle-audit §6 #6)
- **Date discovered**: 2026-05-04
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap with forward-defensive enforcement-half landed; emission-half deferred
- **Severity**: LOW
- **Tags**: `session`, `tags`, `composite`, `emission-vs-enforcement`
- **Blocks**: nothing at v0 (the immutability axis is honored by CH-12; the emission axis needs no other chunks before M6+ unless an HTTP retag endpoint lands sooner)
- **Blocked-by**: future M6+ "Session structural-tag emission" chunk (TBD chunk; not yet allocated in forward-scope)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Tag Vocabulary for Sessions" lines 220–231.
- **Concept claim**: Session structural tags include 8 categories — `agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`, plus `#kind:session` / `session:<id>` — all frozen-at-creation.
- **Contradiction**: today only `#kind:session` + `session:<id>` are auto-emitted on Session creation via `Composite::auto_tags_for("session", id)` (cf [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs) lines 326–337 + [`modules/crates/domain/src/session_recorder.rs`](../../../../../../modules/crates/domain/src/session_recorder.rs) lines 270–295). The other 6 prefixes (`agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:`) are NOT emitted on Session creation. CH-12's `SESSION_FROZEN_TAG_PREFIXES` const at [`modules/crates/domain/src/permissions/manifest/validator.rs`](../../../../../../modules/crates/domain/src/permissions/manifest/validator.rs) ships them as forward-defensive entries, but emission infrastructure has not been built.
- **Classification**: `partially-honored` (immutability axis honored by CH-12; emission axis remains aspirational)
- **phi-core leverage status**: `N/A — no phi-core overlap` (Session.tags emission lives wholly in baby-phi domain)

## Plan vs. reality
- **Plan said** (CH-12 plan §3.C carry-forward defer, plan §10 close-criteria carry-forward bullet): "Full session-tag emission (the 6 concept-aspirational categories at concept doc 05 lines 220–231) → deferred to M6+ 'Session structural-tag emission' follow-up. Reason: CH-12 enforces immutability on whatever is currently emitted plus all reserved namespaces; emitting `agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:` on session creation is a separate axis (the runtime needs `current_project`, `current_org`, agent-context fields wired to the session-create flow). CH-12 is the IMMUTABILITY chunk; tag-emission completeness is its own work item."
- **Reality**: matches the plan — CH-12 ships immutability enforcement + the const + the rejection rule + the audit-event builder; the runtime that produces these tags on Session creation does not yet exist. The const is forward-defensive: when emission lands, the rejection-on-mutation already covers the new prefixes byte-for-byte.

## Required follow-up
- **What needs to happen**: when a future M6+ chunk lands "Session structural-tag emission", `Composite::auto_tags_for("session", id, ctx)` (or a new helper) MUST extend the auto-emitted tag list to include `agent:<actor_id>`, `project:<current_project_id>` (when applicable), `org:<current_org_id>` (always), `task:<task_id>` (when applicable), `role_at_creation:<role>`, `agent_kind:<kind>`. The new helper takes additional `AgentContext` / `Session::shape` parameters.
- **Tests required**: acceptance scenarios verifying that a Session created with project + org + agent context emits all 8 structural tags; tests verifying that retag attempts on the now-fully-populated tag set are rejected by the existing CH-12 enforcement (`validate_tag_write_on_session`).
- **Acceptance**: every prefix in `SESSION_FROZEN_TAG_PREFIXES` is emitted on session creation in the appropriate session shape; the immutability-axis is already honored by CH-12 (no additional rejection logic needed).

## Closing chunk
- TBD — likely M6+ "Session structural-tag emission" follow-up; not yet allocated in forward-scope.

## Lifecycle
- **2026-05-04 — `discovered`** — filed by CH-12 retrospective. CH-12 enforces immutability on the full 10-prefix list via `validate_tag_write_on_session` + Rule E in `validate_published_manifest`; emission expansion deferred to M6+. Mirrors CH-11's `D-CH11-FOLLOWUP-01` pattern (chunk closes one axis of a multi-axis concept-doc claim; the other axis tracked here).

## Cross-references
- CH-12 plan: [`baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md`](../../../../plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md) §3.C row 1 + §10 carry-forward bullet.
- CH-12 retrospective: [`baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/retrospective.md`](../../../../plan/build/ch-12-frozen-session-tag-immutability-6a748175/retrospective.md) §6 Q1.
- ADR-0049: [`m5_2/decisions/0049-frozen-session-tag-immutability.md`](../../m5_2/decisions/0049-frozen-session-tag-immutability.md) §D49.6 (`SESSION_FROZEN_TAG_PREFIXES` const carries the M6+ entries forward-defensively).
- D-new-08: [`D-new-08.md`](D-new-08.md) (closed by CH-12 — immutability axis).
- Sister pattern: [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md) (CH-11's analogous follow-up for `Project.deadline_at`).
- Concept doc: [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) lines 220–231.
- Matrix row: `_concept-audit-matrix.md` row 191 ("Session tag vocabulary"; status `partially-honored`; evidence-cell cites this drift).
