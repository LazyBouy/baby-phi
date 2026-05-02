<!-- Last verified: 2026-05-02 by Claude Code (CH-10: terminally remediated. The 6-state Consent lifecycle from `concepts/permissions/06-multi-scope-consent.md` §"Consent Lifecycle" is now executable code at `domain::consents::{state, transitions}`. 5 pure-fn transitions (acknowledge / decline / revoke / mark_timed_out / mark_expired) + 5 per-transition Repository methods on both backends + auto-timeout sweeper task at `server::state::spawn_consent_sweeper`. Migration 0012 adds `deadline_at: option<string>` column. Single-pod-only at v0; CHK8S-D-09 records the multi-pod deferral. ADR-0047 ratifies. Status flipped to `remediated`.) -->

# D-new-05 — Consent lifecycle state machine missing (Requested → Acknowledged / Declined / TimedOut / Revoked / Expired)

## Identification
- **ID**: D-new-05
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `state-machine`, `consent-model`
- **Blocks**: D-new-17 (Per-Session consent blocks reads until Acknowledged)
- **Blocked-by**: **D-new-04** (state field must exist on Consent before state machine can operate)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Consent Lifecycle"
- **Concept claim**: Consent progresses `Requested → Acknowledged / Declined / TimedOut / Revoked / Expired` with forward-only revocation.
- **Contradiction**: No Consent state enum; no transition logic. Consent currently has only `granted_at` + `revoked_at` timestamps (implying a 2-state model).
- **Classification**: `contradicts-concept`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: 5-state machine with forward-only transitions.
- **Reality (shipped state at current HEAD)**: No state field; no transition functions; no state-machine tests.
- **Root cause**: `cascading-upstream-deferral` from D-new-04 (state field absence blocks state machine).

## Where visible in code
- **File(s)**: Consent struct in [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); grep for `ConsentState` — expect 0 hits while drift open.
- **Test evidence**: None.
- **Grep for regression**: `grep -n "enum ConsentState\|impl.*Consent.*transition" modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Define `ConsentState` enum + transition function following M1 AuthRequest state-machine pattern (see `domain/src/auth_requests/state.rs`). Wire timeout logic (default = project duration). Revoke idempotency.
- **Implementation chunk this belongs to**: CH-10
- **Dependencies on other drifts**: D-new-04 (hard prereq)
- **Estimated effort**: 1 engineer-day (after D-new-04 lands).
- **Risk to concept alignment if deferred further**: HIGH — without state machine, Per-Session consent policy is a no-op.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
- 2026-05-02 — `in-chunk-plan` — assigned to CH-10 (plan archived as [`build/ch-10-consent-state-machine-and-sweeper-fda01605.md`](../../../../plan/build/ch-10-consent-state-machine-and-sweeper-fda01605.md)).
- 2026-05-02 — `remediated` — CH-10 chunk-seal — 6-state Consent lifecycle materialised at `domain::consents::{state, transitions}` per `concepts/permissions/06-multi-scope-consent.md` §"Consent Lifecycle" verbatim. `state.rs` exposes `is_terminal` + `legal_transition` (the 6-arrow legal table). `transitions.rs` exposes 5 pure-fn helpers (`acknowledge`, `decline`, `revoke`, `mark_timed_out`, `mark_expired`) returning `Result<Consent, ConsentTransitionError>`. Forward-only revocation enforced at the table level — `Revoked → _` returns false for everything. 5 per-transition Repository methods + 5 audit-event builders at `audit::events::m5::consents` ship on both backends. Auto-timeout sweeper task at `server::state::spawn_consent_sweeper` flips past-deadline `Requested` rows to `TimedOut` on a configurable interval (default 60s). Migration 0012 adds `deadline_at: option<string>` column. Single-pod-only at v0 — `CHK8S-D-09` records the multi-pod leader-election deferral. [ADR-0047](../../m5_2/decisions/0047-consent-state-machine-and-sweeper.md) Accepted ratifies.
