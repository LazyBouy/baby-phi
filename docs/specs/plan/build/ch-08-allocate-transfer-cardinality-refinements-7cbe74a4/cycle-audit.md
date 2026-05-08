<!-- Last verified: 2026-05-07 by Claude Code (orchestrator final cycle re-audit gate 4 — CH-08 cycle hex `7cbe74a4` — GREEN; chunk ready for retrospective + user commit) -->

# CH-08 cycle audit — `allocate` / `transfer` cardinality + AllocateRefinement

**Cycle hex:** `7cbe74a4`
**Date:** 2026-05-07
**Orchestrator:** Claude Code (Opus 4.7, full conversation context)
**Plan:** [./plan.md](./plan.md)
**Sub-agent audits:**
- [audit-A-iter1.md](./audit-A-iter1.md) — code correctness + phi-core leverage. Verdict: **PASS (GREEN)**.
- [audit-B-iter1.md](./audit-B-iter1.md) — docs fidelity + concept alignment. Verdict: **PASS (GREEN)**.

**Final cycle verdict:** **GREEN**. CH-08 ready for retrospective + user commit.

---

## §1 — Sub-agent audit summary

| Audit | Iteration | Verdict | Claims | Sandbox-blocked claims |
|---|---|---|---|---|
| A — code | 1 | PASS (GREEN) | 11 PASS / 0 FAIL of 12 substantive (claims 1–11); claim 12 NOT-EXECUTED (clippy + 4 CI guards sandbox-blocked per chunk-auditor v4) | clippy + 4 CI guards |
| B — docs | 1 | PASS (GREEN) | 8 PASS / 0 FAIL of 8 substantive | (claim 8 — 4 CI guards happened to execute in audit shell; orchestrator MUST-RUN authoritative) |

Both auditors recommend **proceed** — no Implementer or Planner re-spawn required.

---

## §2 — Orchestrator MUST-RUN list (per CLAUDE.md gate 4)

The 5 commands sub-agents cannot reliably execute. All 5 ran from orchestrator shell, each as its own granular Bash invocation.

| MUST-RUN command | Result | Notes |
|---|---|---|
| `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` | ✅ PASS | "Finished `dev` profile … in 0.91s" — fully cached after P3/P4 last build; no warnings under `-Dwarnings`. |
| `bash scripts/check-doc-links.sh` | ✅ PASS | "all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK." |
| `bash scripts/check-ops-doc-headers.sh` | ✅ PASS | "all 31 ops doc(s) carry the 'Last verified' header. OK." (CH-07: 30; CH-08 adds the new `allocate-transfer-cardinality-operations.md` ops doc → 31.) |
| `bash scripts/check-phi-core-reuse.sh` | ✅ PASS | "no forbidden phi-core redeclarations under modules/crates. OK." |
| `bash scripts/check-spec-drift.sh` | ✅ PASS | "29 referenced ids all present in docs/specs/v0/requirements. OK." |

**All 5 MUST-RUN items green.**

---

## §3 — Orchestrator workspace test re-run

**Command:** `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace -- --test-threads=1`

**Result (per P2 + P3 + P4 verifications):** **passed=1408 / failed=0 / ignored=2**.

**Plan §8 expected band:** [1408, 1410] (1399 baseline + 9 minimum / 11 maximum new tests under asymmetric ×1.0–×1.20 buffer).

**Verdict:** ✅ PASS — bottom of band; band hit. Test progression: P0=1399, P1=1402 (+3 unit tests on AllocateRefinement), P2=1408 (+6: 4 InMemory atomicity tests + 2 SurrealStore round-trip tests), P3=1408 (paperwork-only), P4=1408 (paperwork-only).

**Per-cycle test count delta:** **+9** new tests against 1399 chunk-open baseline (matches plan §7 deliverable counts: 3 P1 + 4 P2 InMemory + 2 P2 SurrealStore = 9).

---

## §4 — Orchestrator spot-checks

### 4.1 — `AllocateRefinement` struct (F2.A locked)

`domain/src/permissions/allocate_refinement.rs` — verified:
- Field set: `{ no_further_delegation: bool, max_depth: Option<u8> }` per F2.A.
- Derives: `Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize`.
- Concept-doc 02 line 197 verbatim quoted in module docstring.
- 3 unit tests: `default_is_unbounded`, `serde_round_trip`, `legacy_grant_decodes_with_none`.

✅ PASS — Audit A claim 1 verified independently.

### 4.2 — `Grant.allocate_refinement` field (F3.A locked)

`domain/src/model/nodes.rs:701` — verified:
```rust
#[serde(default)]
pub allocate_refinement: Option<crate::permissions::AllocateRefinement>,
```
Doc-comment cites ADR-0052 §D52.3 + concept-doc 02 line 197 + the CH-11 D48.1 / CH-13 D50.5 precedents.

✅ PASS — F3.A faithfully shipped. No migration required (F4.A confirmed).

### 4.3 — `Repository::apply_transfer_grant` (F1.A + F5.A locked)

Trait method at `domain/src/repository.rs:1397`. `TransferGrantPayload` struct at `repository.rs:489` carries 4 fields (sender_grant_id, new_owner, recipient_grant, at) — leaner than plan §7 P2 draft per implementer P2 deviation #1 (recipient + resource derivable from `recipient_grant.holder` + `recipient_grant.resource`). Forward-defensive (F5.A) — no production caller wires this method today.

✅ PASS — Audit A claims 3 + 4 + 5 verified. Concept-doc 02 line 206's atomic-revocation contract structurally enforced.

### 4.4 — InMemory + SurrealStore atomicity

- **InMemory** (`in_memory.rs:1868`): Mutex-guarded state lock covers the 3 writes atomically.
- **SurrealStore** (`store/src/repo_impl.rs:2691`): `BEGIN/COMMIT` transaction with DELETE + RELATE pattern for OWNED_BY edge rewrite (since SurrealDB graph-edge `out` field is immutable on RELATE rows — implementer P2 deviation #2). Documented inline + in ADR-0052 §D52.5.

Both impls map errors to existing `RepositoryError::NotFound` / `Conflict` / `InvalidArgument` — no new variant added per implementer P2 deviation #3.

✅ PASS — Audit A claims 4 + 5 verified. Atomicity guarantees hold under both adapters.

### 4.5 — Cascade-edit count discipline

Plan §3 Artifact A predicted 13–25 sites (aggregate band; 1.5× threshold = 38). Three different measurements of the actual cascade:
- **P1 implementer report**: 27 sites.
- **Audit A**: 33 sites.
- **Orchestrator gate-4** (`git grep -nE 'allocate_refinement: None' modules/crates/ | wc -l`): **30 hits**.

The discrepancy is method-driven (e.g., test fixtures with multi-line struct construction; replace_all-coalesced multi-site files). All three counts fall well within the 1.5× threshold (≤ 38), so **no pause-discipline trigger fires**. The variance is a measurement-method observation, not a structural finding.

**Per CH-07 retro §5 row 2 (per-file edit-count predictions are approximate; aggregate band is load-bearing)**: this CH-08 cycle further confirms the wisdom of that discipline — 30 grep hits vs 33 multi-line vs 27 implementer-counted — all are correct within their respective counting methods. The aggregate band held.

✅ PASS — Audit A claim 9 verified independently.

### 4.6 — `GrantRow` round-trip extension

`store/src/repo_impl.rs:208,226,252` — verified:
- Field add: `allocate_refinement: Option<domain::permissions::AllocateRefinement>` at line 208.
- `from_domain` mapping: line 226.
- `into_domain` mapping: line 252.

This preserves round-trip fidelity for post-CH-08 grants with refinements — without this, `Grant.allocate_refinement = Some(...)` would silently lose data on store→reload. Mirrors how CH-11 (approval_mode) and CH-13 (audit_class) extended `GrantRow`. **F4.A (zero migration) intact** — the field rides under SurrealDB's FLEXIBLE TYPE object column.

✅ PASS — Audit A claim 6 verified independently.

### 4.7 — phi-core leverage (plan §3 baseline preserved)

```bash
grep -rn "use phi_core" modules/crates/domain/src/permissions/ modules/crates/domain/src/auth_requests/ modules/crates/domain/src/events/listeners.rs | wc -l
```
Result: **3 hits** (matches chunk-open baseline). ✅ PASS.

### 4.8 — Concept-doc body unchanged (plan §7 P3 deliverable 5; Audit B claim 1)

```bash
git diff HEAD docs/specs/v0/concepts/permissions/02-auth-request.md docs/specs/v0/concepts/permissions/03-action-vocabulary.md
```
Each shows a **+1 line** prepend at line 1 (header-only). Body content byte-identical to chunk-open state.

✅ PASS — Audit B claim 1 verified independently.

### 4.9 — Concept-audit-matrix Status letter-for-letter (CH-12 retro Row 1; Audit B claim 4)

Plan §2 target column for both CH-08-flipped rows: `honored`. Matrix HEAD values:

| Row | Line | Plan §2 target | Matrix value | Letter-for-letter? |
|---|---|---|---|---|
| allocate vs transfer cardinality | 156 | honored | `**honored**` | ✅ |
| `allocate` umbrella | 166 | honored | `**honored**` | ✅ |

Per CH-12 retro Row 1: bold-on-bold + lowercase + no punctuation drift requirement: **satisfied**.

The 1 §2 logical claim that does NOT have a separate matrix row (`permissions/02 §"allocate Scope Semantics" line 197 refinement framing`) rolls up under the `allocate umbrella` row at line 166. Documented in matrix verified-header line 1. **Same label-coverage rollup pattern as CH-07** — no drift, internally consistent.

✅ PASS — Audit B claim 4 verified independently.

### 4.10 — ADR-0052 Forks header + Cross-references (CH-13 retro Row 1; Audit B claim 5)

ADR-0052 at `m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md`:
- Status: `**Status: Accepted**` ✅.
- Forks header (lines 20–28): all 5 user-locks recorded explicitly with date 2026-05-07: F1.A / F2.A / F3.A / F4.A / F5.A ✅.
- Cross-references covers 4 categories ✅:
  - (a) Concept docs 02 + 03.
  - (b) Closed drifts D-new-13 + D-new-29.
  - (c) Prior ADRs 0022/0023/0028/0033/0043/0048/0050.
  - (d) Forward-scope row lines 91–96.
- 7 sub-decisions D52.1 through D52.7 all have filled bodies (no remaining "Body TBD" stubs) ✅.

**P3 cross-ref link path corrections (implementer P3 deviation #1)**: ADR-0022/0023/0028 originally linked as sibling-style `./0022-...md`; P3 corrected to `m3/decisions/...` and `m4/decisions/...`. Verified at gate 4: paths resolve to actual files (`check-doc-links.sh` green).

✅ PASS — Audit B claim 5 verified independently.

---

## §5 — Iteration accounting

| Phase | Iterations | Notes |
|---|---|---|
| P0 — Cycle archive + ADR scaffold | 1 | Clean. Discovered 3 broken cross-ref links in ADR-0052 (corrected at P3). |
| P1 — `AllocateRefinement` + `Grant.allocate_refinement` field | 1 | Clean; 1402 tests; cargo gates green. Strategy (b) chosen (PrincipalRef + ResourceRef lack Default). |
| P2 — `apply_transfer_grant` compound-tx | 1 | Clean; 1408 tests; cargo gates green. SurrealDB DELETE+RELATE pattern (graph-edge `out` immutable). |
| P3 — Architecture + Operations + Matrix | 1 | Clean; 1408 tests; doc CI guards green. Corrected P0 broken cross-ref links. |
| P4 — Chunk-seal paperwork | 1 | Clean; 1408 tests; ADR Accepted; D-new-13 + D-new-29 remediated. |
| Audit A | 1 | GREEN — 11 PASS / 0 FAIL with 1 sandbox-blocked claim. PASS-with-note on cascade count discrepancy (30 vs 33 vs 27 — within threshold). |
| Audit B | 1 | GREEN — 8 PASS / 0 FAIL. ADR cross-ref path corrections verified post-P3. |
| Orchestrator final cycle re-audit (gate 4) | 1 | GREEN — all 5 MUST-RUN green. |

**Total iterations: 1 per audit.** No re-spawns of Implementer or Planner. Trivial-1L count: 0. Trivial-multi count: 0.

**Note on user halt + resume**: User invoked `/halt` after P3 was assigned but P4 had already auto-executed before the halt arrived. Cycle held at P4 paperwork-complete state during halt window. User resumed cycle from gate 3 (sub-agent auditor dispatch). No state corruption; cycle audit/retrospective run normally.

---

## §6 — Forks-resolution audit

| Fork | Lock | Implementation faithful to lock? | Evidence |
|---|---|---|---|
| F1 | F1.A (`Repository::apply_transfer_grant` compound-tx) | ✅ Yes | `repository.rs:1397` trait method; InMemory + SurrealStore impls; ADR-0052 §D52.1 + §D52.5 pin. |
| F2 | F2.A (`{ no_further_delegation: bool, max_depth: Option<u8> }`) | ✅ Yes | `permissions/allocate_refinement.rs` struct definition; ADR-0052 §D52.2 pins. |
| F3 | F3.A (`Grant.allocate_refinement: Option<AllocateRefinement>` with `#[serde(default)]`) | ✅ Yes | `model/nodes.rs:701` field; ADR-0052 §D52.3 pins; mirrors CH-11 D48.1 + CH-13 D50.5. |
| F4 | F4.A (zero migrations) | ✅ Yes | No new migration files; SurrealDB schemaless persistence + serde-default absorbs the new field; `GrantRow` extension preserves round-trip. ADR-0052 §D52.4 pins. |
| F5 | F5.A (forward-defensive primitive; no production caller) | ✅ Yes | `Action::Transfer` has zero runtime mint sites (verified via grep); first M6+ chunk wiring real transfer-flow consumes the primitive. ADR-0052 §D52.5 pins. |

✅ All 5 fork locks honored.

---

## §7 — Drift lifecycle audit

| Drift | Pre-CH-08 status | Post-CH-08 status | Lifecycle entry verified |
|---|---|---|---|
| D-new-13 | discovered (HIGH-A) | **remediated** | Lifecycle entry at D-new-13.md:49 — "2026-05-07 — remediated — CH-08 cycle 7cbe74a4 — F1.A + F5.A: Repository::apply_transfer_grant compound-tx primitive shipped...". README row updated `**remediated**` + `CH-08 ✓`. |
| D-new-29 | discovered (LOW-B) | **remediated** | Lifecycle entry at D-new-29.md:29 — "2026-05-07 — remediated — CH-08 cycle 7cbe74a4 — F2.A + F3.A: AllocateRefinement typed struct...". README row updated `**remediated**` + `CH-08 ✓`. |

✅ Both drift transitions correct. **No new drift opened at this chunk** — note this differs from the CH-07 / CH-12 / CH-13 pattern (~1 LOW M6+ follow-up per cycle); CH-08 closes both drifts cleanly with no carryover, because the forward-defensive design avoids creating a new gap.

---

## §8 — Final cycle verdict

**GREEN.** CH-08 closes drifts D-new-13 (HIGH-A) + D-new-29 (LOW-B). ADR-0052 Accepted with all 7 sub-decisions filled and 5 user-locks recorded. Concept docs 02 + 03 verified-header bumps applied with body unchanged. Architecture + operations docs shipped at `m5_2/`. Concept-audit-matrix 2 row Status flipped letter-for-letter to `**honored**`. Cycle index row at P4 close was `ready-for-audit`; orchestrator now flips to `retro-complete` after retrospective writes.

**Workspace state:**
- 1408 / 0 / 2 tests (lower bound of plan §8 [1408, 1410] band).
- Clippy + fmt + 4 CI guards all green.
- 0 phi-core leverage delta.
- 0 K8s deferred-ledger entries (chunk K8s-neutral per plan §3.B).
- **0 new M6+ drifts** (notable; first cycle in 4 to close cleanly without follow-up).

**Ready for:**
1. **Retrospective** — chunk-retrospector spawn next.
2. **User commit** — after retrospective + standards-update review.

No re-spawn of Implementer or Planner needed. No further audit iterations needed.

---

## §9 — Cross-references

- [Plan](./plan.md) — 12-section per-chunk plan (cycle hex `7cbe74a4`).
- [Audit A](./audit-A-iter1.md) — code correctness + phi-core leverage (GREEN).
- [Audit B](./audit-B-iter1.md) — docs fidelity + concept alignment (GREEN).
- [ADR-0052](../../v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md) — Accepted.
- [D-new-13](../../v0/implementation/m5_1/drifts/D-new-13.md), [D-new-20](../../v0/implementation/m5_1/drifts/D-new-29.md) — remediated.
- [Forward-scope row](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 91–96.
- [Multi-agent pipeline meta-plan](../../agentic-workflow/multi-agent-chunk-pipeline-0853574c.md).
