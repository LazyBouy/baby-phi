<!-- Last verified: 2026-05-04 by Claude Code (chunk-auditor agent A iter 1) -->

# Audit A — CH-12 frozen session-tag immutability — iter 1

**Auditor:** Audit-A (code correctness + phi-core leverage + K8s readiness)
**Plan:** [./plan.md](./plan.md)
**Cycle hex:** 6a748175
**Date:** 2026-05-04
**Word count:** ~520

## Per-claim PASS/FAIL

| # | Claim | Verdict | File:line evidence | Notes |
|---|-------|---------|---------------------|-------|
| 1 | `ValidationError::CompositeStructuralTagWrite` variant + 7 hard-rejection variants total + correct derives | PASS | `validator.rs:309-313` (variant); `validator.rs:231` `#[derive(Debug, Clone, PartialEq, Eq, Error)]`; 7 variants enumerated at lines 240/251/263/272/284/299/309 | thiserror::Error via `use thiserror::Error;` line 48 |
| 2 | Rule E pass after Rule C, Memory exemption, target_kinds matching | PASS | `validator.rs:440-476`; comment block at 440-456 documents D49.1.a; `Composite::MemoryObject` exemption at line 460-462; reserved-prefix match at 464-466 | Rule E body verified; ordering after Rule C at line 421-438 |
| 3 | `pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` with exactly 10 prefixes; no `#archived`/`#active` | PASS | `validator.rs:103-114`; tests at `validator.rs:1340` assert `len() == 10`; `validator.rs:1376-1377` assert `#archived`/`#active` NOT in list | All 10 prefixes verified verbatim |
| 4 | `validate_tag_write_on_session(...)` rejects added/removed frozen tags; lifecycle pass | PASS | `validator.rs:605-643`; helper `is_frozen` at 610-614; FrozenTagAdded at 622-625; FrozenTagRemoved at 635-638 | Lifecycle tags pass through (not in prefix list) |
| 5 | `FrozenTagViolation` enum has FrozenTagAdded/FrozenTagRemoved with proper derives | PASS | `validator.rs:559` `#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]`; FrozenTagAdded at 565; FrozenTagRemoved at 574 | Display via `#[error(...)]` per variant |
| 6 | `RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }` variant | PASS | `repository.rs:127-130`; mirrors ManifestValidation pattern at lines 97-101 | Additive variant |
| 7 | Repository trait module-level docstring documents precondition contract | PASS | `repository.rs:19-48` (lines 19-48 document the 3-step contract: validator call + propagate + emit audit event) | Both validate_tag_write_on_session AND frozen_tag_write_rejected called out; pairing contract explicit |
| 8 | `permissions/mod.rs` re-exports new symbols | PASS | `permissions/mod.rs:48-53` re-exports `validate_tag_write_on_session, FrozenTagViolation, SESSION_FROZEN_TAG_PREFIXES` | All three exports present |
| 9 | ~22 unit tests in validator (accept range to 27) | PASS | `validator.rs` mod tests has 56 total `#[test]` functions; CH-12-specific tests visible at lines 1151-1573 (Rule E + frozen-tag-prefixes + validate_tag_write + Display) | Net test addition matches "~27 actual" buffer band |
| 10 | `acceptance_manifest_validator.rs` extended with ~5 Rule-E tests + cross-impl consistency | PASS | 5 Rule-E tests at lines 297, 335, 361, 387 (cross-impl), 435 (memory_object exemption); file has 14 `#[tokio::test]` total | Cross-impl consistency InMemory + SurrealStore at line 387 |
| 11 | NEW `acceptance_frozen_session_tags.rs` with ~10 tests (7 runtime + 3 audit-event integration) | PASS | 10 tests total: tests 1-7 (runtime, lines 73-219), tests 8-10 (audit-event integration, lines 242-426) | Counts confirmed via grep |
| 12 | Workspace test count green at expected | PASS | `cargo test -j 4 --workspace -- --test-threads=1` → **1365 passed / 0 failed / 1 ignored**; matches plan §8 chunk-close target exactly | Within accept band 1346-1380 |
| 13 | Clippy clean under `RUSTFLAGS="-Dwarnings"` | NOT-EXECUTED-IN-AUDIT | Sandbox-blocked. Grep equivalent: `grep -rE "#\[allow\(clippy::" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs modules/crates/server/tests/acceptance_frozen_session_tags.rs` returns 0 matches; `git diff modules/crates/` does not introduce any new `#[allow(clippy::...)]` annotations | Defer to orchestrator MUST-RUN |
| 14 | 4 CI guards green | NOT-EXECUTED-IN-AUDIT | Sandbox-blocked. Defer to orchestrator MUST-RUN list per CLAUDE.md gate 4 | — |
| 15 | phi-core leverage: 48 imports unchanged; 0 in permissions/manifest/ and audit/events/m5_2/ | PASS | `grep -rn "use phi_core::" modules/crates/ \| wc -l == 48`; `grep ... modules/crates/domain/src/permissions/manifest/` returns 0; `grep ... modules/crates/domain/src/audit/events/m5_2/` returns 0 | Baseline preserved |
| 16 | K8s readiness all axes no-impact; canonical_bytes test still passes; audit_events schema unchanged | PASS | `audit_events table` schema at `migrations/0001_initial.surql:84-99` unchanged (`event_type TYPE string` line 85; `audit_class ASSERT INSIDE [silent,logged,alerted]` lines 91-92); `canonical_bytes_excludes_prev_event_hash` test at `audit/mod.rs:161` still in source; CH-21 chain test passes (`acceptance_memory_extraction` 7 passed, `audit_emitter_chain_test` 4 passed) | No new K8s blocker class |
| 17 | Cascade fan-out within caps | PASS | `git grep -nE 'ToolAuthorityManifest\\s*\\{' modules/crates/` returns **8 sites** (≤ 9 cap); `ValidationError::` references in **2 files** (≤ 8 cap); FrozenSessionTagWrite touches 2 files; CompositeStructuralTagWrite touches 2 files | Within all 1.5× upper bounds |
| 18 | Prior-chunk invariants intact | PASS | `domain --lib permissions::manifest::validator::tests` 56/56 passed; `acceptance_per_session_consent_gating` 8/8 passed; `acceptance_memory_extraction` 7/7 passed; `audit_emitter_chain_test` 4/4 passed | All upstream chunks green |
| 19 | F5.B audit-event builder + correct event_type literal + AuditClass::Alerted + module wiring | PASS | `tool_authority.rs:56-87` (`fn frozen_tag_write_rejected`); event_type literal `"tool.frozen_tag_write_rejected"` at line 69; `AuditClass::Alerted` at line 82; `audit_class` enum at `audit/mod.rs:36` (note: prompt cited line 39, actual is 36); `audit/events/m5_2/mod.rs:13` `pub mod tool_authority;` | event_type + class + wiring all confirmed |
| 20 | F5.B audit-event tests (≥ 4 unit + ≥ 3 integration) | PASS | 4 unit tests at `tool_authority.rs:99,132,151,170` (added/removed/rfc3339/canonical_bytes); 3 integration tests at `acceptance_frozen_session_tags.rs:242,316,365` (test_8/9/10); test_8 + test_9 assert `stored.audit_class == AuditClass::Alerted` after SurrealDB roundtrip exercising "alerted" snake_case serde | All assertions present |
| 21 | F5.B no migration; audit_events table schema unchanged | PASS | `ls modules/crates/store/migrations/ \| wc -l == 13` (last is `0013_per_session_consent_gating.surql`); `audit_events` table at `migrations/0001_initial.surql:83-99` unchanged | Schema-stable confirmed |

## Workspace test result

- **Command:** `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1`
- **Output:** **1365 passed / 0 failed / 1 ignored**
- **Plan §8 expected target at chunk-close:** 1360–1366 (orchestrator-accept band 1346–1380)
- **Match:** ✅ — exact match against the plan's chunk-close prediction

## CI guards

- `check-doc-links.sh` — NOT-EXECUTED-IN-AUDIT (sandbox)
- `check-ops-doc-headers.sh` — NOT-EXECUTED-IN-AUDIT (sandbox)
- `check-phi-core-reuse.sh` — NOT-EXECUTED-IN-AUDIT (sandbox); grep-equivalent passes (zero `use phi_core::` in `domain/src/permissions/manifest/` + `domain/src/audit/events/m5_2/`; total still 48)
- `check-spec-drift.sh` — NOT-EXECUTED-IN-AUDIT (sandbox)

All 4 deferred to orchestrator MUST-RUN list per `CLAUDE.md` gate 4 / CH-11 retro.

## Final verdict

**OVERALL: GREEN.**

All 21 claims verified: 19 PASS, 2 NOT-EXECUTED-IN-AUDIT (claims 13 + 14, sandbox-blocked, properly deferred). Test count is an exact match to plan §8's chunk-close prediction (1365 / 0 / 1). All seven `ValidationError` variants confirmed, the 10-entry `SESSION_FROZEN_TAG_PREFIXES` constant has the correct shape (with `#archived`/`#active` properly excluded), Rule E correctly exempts `Composite::MemoryObject` per D49.1.a, the F5.B audit-event builder produces the right `event_type` + `AuditClass::Alerted` shape, three integration tests pin the validator-→-builder pairing through SurrealDB chain-link insertion, and zero phi-core imports were added (baseline 48 preserved). Cascade fan-out is well within plan caps (8/9 for `ToolAuthorityManifest`, 2/8 file-count for `ValidationError`). Hash-chain symmetry held at the CH-21 acceptance level. Migration count unchanged at 13. Repository trait module-level docstring documents the full validator + audit-event pairing contract per F5.B. No findings; no recommended fixes. Recommendation: proceed to Audit-B then orchestrator final cycle re-audit (which closes the 4 CI guards + clippy claims).

Minor observation (non-finding): plan claim 19 cited `AuditClass` at `audit/mod.rs:39` while actual location is line 36 — line drift only, no semantic gap.
