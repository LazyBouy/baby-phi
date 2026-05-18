<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-SEAL deliverable 5 — milestone tag `v0.1-m5` staging; user-managed execution per forward-scope line 212; staging artifact for user sign-off; cycle hex `5778bb77`.) -->

# Milestone tag `v0.1-m5` — staging artifact

This file stages the M5 milestone tag for **user execution**. The chunk-implementer (and orchestrator) MUST NOT run `git tag` directly; tagging is user-managed per the forward-scope contract at `forward-scope/remaining-scope-post-m5-p7-22035b2a.md:212` ("milestone tag executed by user, not by agent").

## Proposed command

```bash
git -C /root/projects/phi/baby-phi tag v0.1-m5 <SHA>
```

Where `<SHA>` is the SHA of the commit that lands CH-24's chunk-seal (typically the squash-commit landing the cycle).

## Tag annotation

Recommended annotated tag message (paste into `git tag -a v0.1-m5 <SHA> -m "<message>"`):

```
v0.1-m5 — Milestone 5 sealed (Templates, System Agents, First Session)

Closes:
- M5/P0 (CH-K8S-PREP equivalent — post-flight delta + ADRs 0029/0030/0031)
- M5/P1–P7 (CH-15 → CH-21 chunk equivalence; 5 admin-page verticals + s02/s03/s05 listeners + CLI/web polish)
- M5/P8a (CH-21 MemoryExtractionListener body — s02)
- M5/P8b (CH-22 AgentCatalogListener body — s03)
- M5/P8c (CH-23 Template C+D listener bodies + s05 acceptance)
- M5/P8d + P9 (CH-24 milestone seal — carryover re-verification + 5-per-page acceptance + e2e first-session + recent_sessions API-surface flip + runbook + troubleshooting + reuse-map refresh)

Composite confidence: ≥99% (impl 25/25 + docs 20/20 + code aspect + phi-core Δ +0 + concept-alignment 7/7 honored).

Cycle hex: 5778bb77
ADRs Accepted across M5: 0029, 0030, 0031, 0059
Drifts remediated in CH-24: D-CH24-recent-sessions-api-flip (mid-cycle scope expansion)

Workspace state: 1537/0/2 tests passing; 0 clippy warnings; 4 CI guards green.
```

## User sign-off checklist

Before executing `git tag`, confirm:

- [ ] **Composite ≥99%**: `cycle-audit.md` (written by orchestrator gate-4) confirms composite confidence ≥99% across all 5 aspects (code / docs / phi-core leverage / concept alignment / impl%).
- [ ] **All phases shipped**: P0 (preflight) + P-VERIFY (5/5 carryovers PASS) + P-NEW-TESTS (8 new tests at v2 close → 1537/0/2 workspace) + P-FLIP-RECENT-SESSIONS (dedicated repo method + RecentSessionEntry + tripwire flip) + P-DOCS (runbook, troubleshooting, reuse-map, drift file, ADR-0059, concept-audit-matrix, drift README, detail.rs:33 doc-comment refresh) + P-SEAL (CI extensions, paperwork, milestone-tag staging, forward-scope amendment).
- [ ] **All CI guards green**:
  - `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` → OK
  - `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` → OK
  - `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` → OK
  - `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` → OK
- [ ] **Workspace health gate**:
  - `cargo fmt --all -- --check` → green
  - `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` → green, 0 warnings
  - `cargo test --workspace -j 4 -- --test-threads 1` → 1537/0/2 within plan §8 band [1537, 1539]
- [ ] **Cargo clean executed**: per chunk-implementer v8 + CH-18 retro Row 1 immediate-post-test discipline (within-cycle accumulated target/ reclaimed before commit; orchestrator runs final cargo clean at gate-5 close).
- [ ] **Audit-tmp scripts cleaned up**: `ls /root/projects/phi/baby-phi/scripts/audit-tmp-*.sh 2>/dev/null | wc -l` → 0 (orphan ephemeral scripts deleted at gate-5 close per chunk-auditor v7 canonical-script-reuse mandate).
- [ ] **ADR-0059 status flipped**: `grep "Status: " /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md | head -1` → reads `Accepted`.
- [ ] **D-CH24 drift status terminal**: `grep "Status" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-CH24-recent-sessions-api-flip.md | head -1` → reads `remediated`.
- [ ] **Forward-scope row amended**: `grep -n "D-CH24" /root/projects/phi/baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` → ≥ 1 hit (line ~211 acknowledges in-cycle scope expansion).
- [ ] **`v0.1-m5` tag does not already exist**: `git -C /root/projects/phi/baby-phi tag -l v0.1-m5` → empty.

## Verification post-tag

After tagging, verify with:

```bash
git -C /root/projects/phi/baby-phi tag -l v0.1-m5
git -C /root/projects/phi/baby-phi show v0.1-m5 --stat | head -30
```

The tag points to the commit that lands CH-24's seal; from that SHA the M5 milestone is durable history.

## Why agent does not execute `git tag`

Per forward-scope `remaining-scope-post-m5-p7-22035b2a.md:212` (paraphrased): *"milestone tag executed by user, not by agent — agent stages the tag command for user sign-off but never runs `git tag` directly."* This preserves the user as the final gatekeeper for milestone-level artifacts in the durable git history.

The chunk-implementer's role at P-SEAL is to:
1. Stage the proposed command (this file).
2. Verify all CI/test/paperwork gates pass.
3. Hand off to the orchestrator's gate-4 final cycle re-audit.

After orchestrator gate-4 + gate-5 (retrospective + standards updates), user reviews this file, runs the proposed `git tag` command, and pushes.

## Cross-references

- [CH-24 plan §7 P-SEAL deliverable 5](./plan.md) — milestone-tag staging.
- [Forward-scope row line 212](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) — user-managed milestone tag policy.
- [M5 README](../../../v0/implementation/m5/README.md) — milestone phase-status table + CH-24 close summary.
- [ADR-0059](../../../v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md) — last sub-decision ratified before M5 seal.
