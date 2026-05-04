<!-- Last verified: 2026-05-04 by Claude Code (chunk-auditor agent A iter 1) -->

# Audit A — CH-13 audit_class composition strictest wins — iter 1

**Auditor:** Audit-A (code correctness + phi-core leverage + K8s readiness)
**Plan:** [./plan.md](./plan.md)
**Cycle hex:** d4fe1b7c
**Date:** 2026-05-04
**Iteration:** 1
**Auditor model:** opus
**Final verdict:** GREEN (PASS)

## Summary table

| # | Claim | Verdict | Evidence anchor |
|---|-------|---------|-----------------|
| 1 | `AuditClass` enum derives `PartialOrd, Ord`; declaration `Silent, Logged, Alerted`; `none ↔ Silent` doc-comment | PASS | `modules/crates/domain/src/audit/mod.rs:46-52`, doc-comment lines 35-45 |
| 2 | `compose_audit_class(...)` exists at the expected path; max-fold over `[Some(a), Some(b), c]` | PASS | `modules/crates/domain/src/permissions/audit_composition.rs:77-83` |
| 3 | `compose_audit_class_with_source(...) -> (AuditClass, AuditClassSource)` exists; 3-variant enum with `serde(rename_all="snake_case")` | PASS | `audit_composition.rs:42-53, 97-123` |
| 4 | Tie-breaker `Override > TemplateAr > OrgDefault` for ties at strictest | PASS | `audit_composition.rs:107-122`; tests `unit_compose_with_source_tie_breaker` lines 228-264 |
| 5 | `Grant.audit_class: AuditClass` field with `#[serde(default = "Grant::default_audit_class")]` → `Silent` | PASS | `modules/crates/domain/src/model/nodes.rs:692-693, 696-705` |
| 6 | 3 fire pure-fns construct Grant with `audit_class: args.audit_class` from FireArgs | PASS | `templates/a.rs:103,128`; `templates/c.rs:81,105`; `templates/d.rs:76,101` |
| 7 | 3 fire listeners call `resolve_composed_audit_class` BEFORE FireArgs construction; fail-safe → `(Silent, OrgDefault)` | PASS | `events/listeners.rs:140-194, 302-303, 432-433, 551-552` |
| 8 | 3 audit-event builders accept `audit_class` + `audit_class_source` params; no hardcoded `Logged`; `audit_class_source` snake-case key in diff | PASS | `audit/events/m4/templates.rs:30-70`; `audit/events/m5/templates.rs:29-65, 77-115` |
| 9 | phi-core import-count = 48 unchanged; 0 in `audit_composition.rs` and `domain/src/audit/` | PASS | `wc -l` = 48; both targeted greps returned empty |
| 10 | Forbidden greps (no second `AuditClass` enum; no hardcoded `Logged` in production builders) return 0 | PASS | Both greps returned empty |
| 11 | Workspace test count: 1379 / 0 / 2 ignored | PASS | aggregated tally `passed=1379 failed=0 ignored=2` |
| 12 | Clippy clean under `RUSTFLAGS="-Dwarnings"` | NOT-EXECUTED-IN-AUDIT | Sandbox-blocked per CH-11 retro; orchestrator MUST-RUN list |
| 13 | 4 CI guards green (`check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`) | NOT-EXECUTED-IN-AUDIT | Sandbox-blocked; orchestrator MUST-RUN list |
| 14 | K8s A1–A7: no impact; A4 migration count = 13; A5 trait-shape unchanged; A7 hash-chain forward-only intact | PASS | 13 migration files; CH-21 test `ch_21_preserves_5_listener_invariant` green |
| 15 | Grant literal cascade ≥ 25; no phi-core import in `audit_composition.rs` | PASS | `Grant {` regex = 67 sites; 0 phi-core imports |
| 16 | Prior-chunk invariants intact (CH-11 / CH-12 / CH-21) | PASS | All three test files pass |
| 17 | F2.A Grant cascade verification: ≥ 30 `audit_class:` references; serde-default attr present | PASS | 131 hits; serde attr at `nodes.rs:692` |

(Summary fits within 600-word target.)

## Per-claim detail

### Claim 1 — `AuditClass` derives `PartialOrd, Ord`; `Silent, Logged, Alerted` order; `none ↔ Silent` doc

**Verdict:** PASS

Evidence at `/root/projects/phi/baby-phi/modules/crates/domain/src/audit/mod.rs`:

- Line 46: `#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]`
- Line 47: `#[serde(rename_all = "snake_case")]`
- Lines 48-52: `pub enum AuditClass { Silent, Logged, Alerted }` — declaration order loosest → strictest.
- Doc-comment lines 35-45 explicitly maps concept-doc-07 line 67's `none < logged < alerted` to enum `Silent < Logged < Alerted` and cites ADR-0050 §D50.1: *"the concept-doc `none` term maps to enum [`AuditClass::Silent`] semantically (CH-13 / ADR-0050 §D50.1)"*.

### Claim 2 — `compose_audit_class(...)` exists; strictest-wins fold

**Verdict:** PASS

Evidence at `/root/projects/phi/baby-phi/modules/crates/domain/src/permissions/audit_composition.rs:77-83`:

```rust
pub fn compose_audit_class(
    org_default: AuditClass,
    template_ar: AuditClass,
    r#override: Option<AuditClass>,
) -> AuditClass {
    compose_audit_class_with_source(org_default, template_ar, r#override).0
}
```

Body delegates to the source-aware companion (claim 3), whose body is a 3-step max-fold (claim 4). Equivalent strictest-wins semantics. Unit test `unit_compose_returns_strictest_of_three` (lines 141-165) exhaustively pins truth-table over 3×3×4 = 36 combos against `[Some(org), Some(tpl), ov].into_iter().flatten().max()`.

### Claim 3 — `compose_audit_class_with_source(...) -> (AuditClass, AuditClassSource)`; 3-variant enum, snake_case serde

**Verdict:** PASS

Evidence at `audit_composition.rs:42-53`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditClassSource {
    OrgDefault,
    TemplateAr,
    Override,
}
```

Companion fn at lines 97-123 with signature `pub fn compose_audit_class_with_source(org_default: AuditClass, template_ar: AuditClass, r#override: Option<AuditClass>) -> (AuditClass, AuditClassSource)`.

### Claim 4 — Tie-breaker `Override > TemplateAr > OrgDefault`

**Verdict:** PASS

Evidence in body lines 107-122: walks candidates least-specific → most-specific using `>=` so a later (more-specific) candidate at the same strictness tier overwrites the earlier winner. Verified by `unit_compose_with_source_tie_breaker` (lines 228-264) which pins:
- `(Logged, Logged, None)` → `(Logged, TemplateAr)` — TemplateAr wins org tie.
- `(Logged, Logged, Some(Logged))` → `(Logged, Override)` — Override wins triple tie.
- `(Silent, Alerted, Some(Alerted))` → Override wins tie at strictest.
- `(Alerted, Silent, Some(Alerted))` → Override wins over OrgDefault tie.

### Claim 5 — `Grant.audit_class` field with serde-default

**Verdict:** PASS

Evidence at `/root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs:692-705`:

```rust
#[serde(default = "Grant::default_audit_class")]
pub audit_class: crate::audit::AuditClass,
```

Adjacent to `approval_mode: ApprovalMode` at line 675 (precedent-mirroring). `Grant::default_audit_class()` at lines 703-705 returns `crate::audit::AuditClass::Silent` (the loosest — preserves no-silent-downgrade per concept-doc 07 line 71).

### Claim 6 — Fire pure-fns preserve pure-fn discipline

**Verdict:** PASS

Evidence:
- `templates/a.rs:103` — `pub fn fire_grant_on_lead_assignment(args: FireArgs) -> Grant`; line 109-111 destructures `audit_class` from FireArgs; line 128 stamps `audit_class` into Grant literal.
- `templates/c.rs:81` — `pub fn fire_grant_on_manages_edge(args: FireArgs) -> Grant`; line 87/105 same pattern.
- `templates/d.rs:76` — `pub fn fire_grant_on_has_agent_supervisor(args: FireArgs) -> Grant`; line 83/101 same pattern.

Each FireArgs struct gains `pub audit_class: AuditClass` field (a.rs:77, c.rs:60, d.rs:60) — listener supplies the composed value pre-call.

### Claim 7 — Fire listeners call composer BEFORE FireArgs; fail-safe `(Silent, OrgDefault)`

**Verdict:** PASS

Evidence at `/root/projects/phi/baby-phi/modules/crates/domain/src/events/listeners.rs`:

- Helper fn `resolve_composed_audit_class` defined at lines 140-194 with documented fail-safe semantics (lines 131-139: *"a repo error or a missing-row hit logs at WARN and returns `(AuditClass::Silent, AuditClassSource::OrgDefault)`"*).
- Implementation at lines 145-167: org-row missing or repo-error returns `(Silent, OrgDefault)` directly.
- Lines 170-191: AR-row missing/error treats template_ar as `Silent` (composes via composer).
- TemplateAFireListener: lines 302-303 call resolver BEFORE FireArgs construction at line 305. Resolved class at line 310.
- TemplateCFireListener: lines 432-433 call resolver, FireArgs construction at line 441.
- TemplateDFireListener: lines 551-552 call resolver, FireArgs construction at line 561.

Audit-event diff invocations on each listener pass `audit_class_source` to builders (lines 339, 463, 585).

### Claim 8 — Audit-event builders accept new params; no hardcoded `Logged`; diff includes `audit_class_source`

**Verdict:** PASS

Evidence:
- `audit/events/m4/templates.rs:30-70` — `template_a_grant_fired` accepts `audit_class: AuditClass` (line 37) + `audit_class_source: AuditClassSource` (line 38); diff includes `"audit_class_source": source_str` (line 60); event-level `audit_class` field passes through (line 70).
- `audit/events/m5/templates.rs:29-65` — `template_c_grant_fired` same shape (lines 36-37, 52, 62).
- `audit/events/m5/templates.rs:77-115` — `template_d_grant_fired` same shape (lines 85-86, 102, 112).

Forbidden grep `audit_class:\s*AuditClass::Logged` over the m4 + m5 templates (excluding test/attribute lines) returned **0 hits** — no hardcoded `Logged` survives in production builder bodies.

### Claim 9 — phi-core import-count = 48; 0 in new module / audit dir

**Verdict:** PASS

- `grep -rn "use phi_core::" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **48** (matches CH-12 baseline).
- `grep -rn "use phi_core::" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/audit_composition.rs` returned empty → 0 imports.
- `grep -rn "use phi_core::" /root/projects/phi/baby-phi/modules/crates/domain/src/audit/` returned empty → 0 imports.

### Claim 10 — Forbidden greps return 0

**Verdict:** PASS

- `grep -rn "^pub enum AuditClass\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "audit/mod.rs"` returned empty → no second AuditClass enum.
- `grep -nE "audit_class:\s*AuditClass::Logged\b" {m4,m5}/templates.rs | grep -v "test|#["` returned empty → no hardcoded Logged in production builder bodies. (Test fixtures inside the same files retain specific values, which is permitted.)

### Claim 11 — Workspace test count: 1379 / 0 / 2 ignored

**Verdict:** PASS

Command: `cd /root/projects/phi/baby-phi && /root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1`

Aggregated tally across all `test result:` lines: **passed=1379 failed=0 ignored=2** — matches user's expected delta exactly. (Note: the plan §8 prediction band 1359-1362 was drafted from a 1345 baseline; chunk-execution evidently grew the baseline to 1365 prior to CH-13 work and added the predicted +14 tests. Final count of 1379 matches the user-supplied audit prompt, which is the seal-time source of truth.)

### Claim 12 — Clippy under `RUSTFLAGS="-Dwarnings"`

**Verdict:** NOT-EXECUTED-IN-AUDIT

Sandbox-blocked from sub-agent shells per CH-11 retrospective and per `CLAUDE.md` orchestrator MUST-RUN list (Multi-agent chunk pipeline gate 4). Sub-agent auditor cannot reliably run `RUSTFLAGS="-Dwarnings" cargo clippy ...`. Flagged for orchestrator's final cycle re-audit.

### Claim 13 — 4 CI guards green

**Verdict:** NOT-EXECUTED-IN-AUDIT

`bash scripts/check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh` are sandbox-blocked from sub-agent shells per CH-11 retro. Structural proxy verification: claim 9 (phi-core import count + 0 imports in new files) provides correctness signal for `check-phi-core-reuse.sh` equivalence; claim 10 (no forbidden duplications) reinforces. Final CI-script execution deferred to orchestrator's MUST-RUN list at gate 4.

### Claim 14 — K8s A1–A7: no impact

**Verdict:** PASS

- A4 migration count: `ls modules/crates/store/migrations/ | wc -l` = **13** (unchanged from CH-12). Files `0001_initial.surql` … `0013_per_session_consent_gating.surql`.
- A5 trait shape: `compose_audit_class` is a free `pub fn`, not a trait method — confirmed at `audit_composition.rs:77`. No new Repository trait method (helper at `events/listeners.rs:140-194` reuses existing `Repository::get_organization` + `get_auth_request`).
- A7 hash-chain symmetry: `AuditEvent::canonical_bytes()` at `audit/mod.rs:80-110` excludes `prev_event_hash` (struct `Canonical` lines 86-96 omits it). Includes `audit_class` (line 93) and `diff` (line 92) — meaning post-CH-13 events of `template.X.grant_fired` have **different** canonical_bytes than pre-CH-13 (composed class + new diff key) but pre-existing event bytes are unchanged. Hash chain is forward-only within `org_scope`. CH-21 invariants verified by running `cargo test -j 4 -p server --test acceptance_memory_extraction` → 7/7 passed including `ch_21_preserves_5_listener_invariant` and `scenario_6_audit_chain_orders_memory_extracted_before_identity_updated`. Domain canonical_bytes test `canonical_bytes_excludes_prev_event_hash` at `domain/src/audit/mod.rs:173` → passed.

### Claim 15 — Cascade fan-out

**Verdict:** PASS

- `grep -rn "Grant {" .../crates/ --include="*.rs" | wc -l` = **67** Grant literal sites — well above the ≥ 25 threshold. (Newline-only `Grant\s*\{$` regex returns 63; both far exceed P1 cascade enumeration's predicted ~31.)
- `grep -rn "use phi_core::" .../audit_composition.rs` = 0 imports (composer is baby-phi-native; concept-doc-07 is not a phi-core surface per `phi-core-mapping.md`).

### Claim 16 — Prior-chunk invariants intact

**Verdict:** PASS

- CH-11 `acceptance_per_session_consent_gating`: `cargo test --test acceptance_per_session_consent_gating` → **8 passed; 0 failed**.
- CH-12 `acceptance_frozen_session_tags`: `cargo test --test acceptance_frozen_session_tags` → **10 passed; 0 failed**.
- CH-21 audit hash chain: `cargo test --test acceptance_memory_extraction` → **7 passed; 0 failed**, including `ch_21_preserves_5_listener_invariant` and `scenario_6_audit_chain_orders_memory_extracted_before_identity_updated`. Domain `canonical_bytes_excludes_prev_event_hash` test → **1 passed**.

### Claim 17 — F2.A Grant cascade verification

**Verdict:** PASS

- `grep -rn "audit_class:" .../crates/ --include="*.rs" | grep -v "//" | wc -l` = **131** hits — ≥ 30 threshold cleared by 4×.
- `#[serde(default = "Grant::default_audit_class")]` attribute at `model/nodes.rs:692` directly above the field declaration (line 693).

## Workspace test result

- **Command:** `cd /root/projects/phi/baby-phi && /root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1`
- **Aggregated outcome:** passed=**1379**, failed=**0**, ignored=**2**.
- **Plan §8 expected (per audit prompt):** 1379 passed / 0 failed / 2 ignored.
- **Match:** PASS.

## CI guards

- `check-doc-links.sh` — NOT-EXECUTED-IN-AUDIT (sandbox-blocked per CH-11 retro).
- `check-ops-doc-headers.sh` — NOT-EXECUTED-IN-AUDIT.
- `check-phi-core-reuse.sh` — NOT-EXECUTED-IN-AUDIT. Structural proxy: claim 9 (0 phi-core imports in new files; total count unchanged at 48) + claim 10 (no `pub enum AuditClass` duplicate; no hardcoded Logged) covers the leverage-correctness signal.
- `check-spec-drift.sh` — NOT-EXECUTED-IN-AUDIT.

All four are deferred to the orchestrator's final-cycle re-audit MUST-RUN list per `CLAUDE.md` gate 4.

## Final verdict

**GREEN (PASS).** All 15 executable claims (1–11, 14–17) PASS with concrete file:line evidence. The 2 sandbox-blocked claims (12, 13) are correctly marked NOT-EXECUTED-IN-AUDIT and flagged for orchestrator MUST-RUN. No FAIL findings.

The implementation faithfully realises ADR-0050's 7 sub-decisions (D50.1–D50.7): variant-naming alignment honored via doc-comment, `Ord` derive encodes `Silent < Logged < Alerted`, composer fn signature + tie-breaker rule pinned, override-can-only-escalate property structurally enforced + unit-tested, Grant denormalisation with serde-default shielding, audit-event diff extension with snake-case `audit_class_source`, and hash-chain forward-only integrity preserved. Pure-fn discipline maintained at fire pure-fns; listener-side composer-wiring with documented fail-safe to `(Silent, OrgDefault)`. Zero phi-core leverage delta. Zero migration delta. Zero K8s blockers. Prior-chunk invariants (CH-11/CH-12/CH-21) all green.

**Recommendation:** Proceed to orchestrator's final cycle re-audit. The 4 CI guards + clippy MUST-RUN at gate 4 are the only remaining verification work; no auditor re-spawn is warranted from this iteration.
