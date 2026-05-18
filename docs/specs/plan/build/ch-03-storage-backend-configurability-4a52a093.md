<!-- Last verified: 2026-04-28 by Claude Code -->

# CH-03 — Storage-backend concept refresh + configurability framing

**Plan file token:** `4a52a093` (generated 2026-04-28 via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/ch-03-storage-backend-configurability-4a52a093.md`.
**Chunk ID:** CH-03 (forward-scope §1 lines 51–59; §5 inventory row line 411).
**Severity:** ⚠HIGH.
**Expected effort:** ~1 engineer-day (**doc-only** — zero code change).
**Hard prerequisites:** none.
**Chunks unblocked at close:** none (closes a silent architectural contradiction; establishes the configurability contract for any future backend-migration chunk).

---

## Context

**The simple version.** baby-phi has used **SurrealDB** as its database since M1. The concept doc still says "SQLite". CH-03 fixes that documentation lie. No code changes — the system continues to work with SurrealDB exactly as it does today.

While we're fixing the doc, we also use the moment to write down two related facts:
1. We're not architecturally locked into SurrealDB. The way the code is structured (everything goes through one Rust trait called `Repository`), swapping databases later would mean writing a new implementation crate — not refactoring the rest of the codebase.
2. Here's a checklist of 7 things any future database candidate would need to support to be eligible (transactional writes, compound transactions, typed edge relationships, etc.).

That's the whole chunk. **Two files get the real edits:**
- `concepts/coordination.md` — replace "SQLite" with an honest description of "we ship SurrealDB; here's the swap checklist".
- A new ADR (ADR-0042) — captures the same decision in the architectural-decision-records archive.

Plus light bookkeeping: drift D-new-02 (which tracks this concept-vs-code lie) gets marked remediated, the drift catalogue gets updated, and the architecture doc that points at `coordination.md` gets a one-line header bump.

**What CH-03 explicitly does NOT do:**
- Doesn't change any Rust code.
- Doesn't add a second database (Postgres, etc.).
- Doesn't write any abstraction layer — the abstraction (`Repository` trait) already exists; we're just *documenting* that it's the swap surface.
- Doesn't run migrations.
- Doesn't add tests.

**Why expand from 0.5d to 1d?** The user's Q1 decision (2026-04-24) said: don't just do a one-line rename. While the concept-doc edit is in flight, write the swap checklist down properly in an ADR — that way, if v1 ever needs Postgres, the future chunk has an explicit contract to satisfy instead of guessing what SurrealDB-specific assumptions the codebase made. The extra half-day buys a durable record of the architectural intent.

**Outcome of this chunk:** drift D-new-02 closes; concept-`coordination.md` no longer lies; ADR-0042 is on file; future backend-swap chunks (if any) inherit a 7-item checklist.

---

## §1 — Context & principle

### Why this chunk

Two reasons:

1. **The concept doc currently lies.** It says "SQLite", code uses SurrealDB. Anyone reading the spec who then opens the code is going to be confused. Honesty matters; fix it.
2. **While fixing it, capture the architectural intent.** "We chose SurrealDB" and "we're locked into SurrealDB forever" are two different statements. The concept-doc rewrite makes the first clear; the new ADR makes the second explicitly false (it's a configured choice, not a hardcoded one). Future-us, considering Postgres or DuckDB, has a 7-item checklist to evaluate against.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them."* Applied: don't silently swap "SQLite" → "SurrealDB" — also write down the swap checklist while we're there. Five extra hours of doc work today saves a future-us from re-deriving the criteria from scratch.

### Forward-scope reference

[§1 CH-03 row](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (lines 51–59) + [§5 inventory row](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (line 411) + [Q1 decision context](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (lines ~450–456).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`coordination.md`](baby-phi/docs/specs/v0/concepts/coordination.md) | § "Design Decisions" — Storage backend (line 69 row) | v0 storage backend is SQLite (single-file, transactional, migratable). | **contradicted** (code uses SurrealDB since M1) | **honored** — new dedicated subsection states (a) v0.1 ships SurrealDB as configured backend, (b) backend is configurable, (c) 7-criterion conforming contract |
| [`coordination.md`](baby-phi/docs/specs/v0/concepts/coordination.md) | § "Design Decisions" — graph-DB framing (same row) | A graph DB is a v1 conversation once access patterns stabilise. | partial (still relevant; ratified) | preserved (CH-03 keeps the v1 framing intact; configurability covers it) |

The drift fundamentally lives at line 69 of `coordination.md`. CH-03 replaces that single table cell with a multi-paragraph subsection.

---

## §3 — phi-core leverage map

| phi-core type | Current handling | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Rationale:** Storage-backend selection + the Repository abstraction are baby-phi-native infrastructure. phi-core has no storage tier (it's a pure agent-loop library; persistence is the consumer's responsibility per `phi-core/CLAUDE.md`). No phi-core types are added or removed.

**Expected import-count delta at chunk close:** **0 phi-core imports added or removed**.

**Positive close-audit greps** (must pass at seal):
```bash
grep -n "## Storage backend\|### Storage backend\|^| \*\*Storage backend" baby-phi/docs/specs/v0/concepts/coordination.md  # ≥ 1 (new subsection)
grep -n "SurrealDB" baby-phi/docs/specs/v0/concepts/coordination.md         # ≥ 1
grep -n "configurable" baby-phi/docs/specs/v0/concepts/coordination.md      # ≥ 1
ls baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md  # exists
grep -c '^\*\*Status: Accepted\*\*' baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md  # 1
```

**Forbidden-duplication greps** (must return 0):
```bash
grep -n "^| \*\*Storage backend\*\* | \*\*SQLite\*\*" baby-phi/docs/specs/v0/concepts/coordination.md  # 0 (old line 69 row gone)
grep -rn "SQLite" baby-phi/docs/specs/v0/concepts/coordination.md            # 0 (no stray SQLite refs)
git diff --stat HEAD -- modules/                                              # empty (zero source-tree changes)
```

---

## §3.B — K8s microservice readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | None. Doc-only. | No |
| **A2** IPC channels | None. | No |
| **A3** pod-local resources | None. | No |
| **A4** migration runner / first-apply race | Documents the existing concern (covered by [CHK8S-D-05](baby-phi/docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) — leader-election lock at M7b). | No new blocker; **clarifies** existing one |
| **A5** trait-shape requirement | Documents the `Repository` trait as the conforming-backend contract — same trait-shape pattern ADR-0033 §D33.1 + §D33.2 introduced. CH-03 is the **second** ADR using this pattern. | No new blocker; reinforces ADR-0033 framing |
| **A6** cross-pod state sharing | The 7-criterion contract specifies remote/multi-pod operation as eligible (Surreal already supports `open_remote` per ADR-0033 §D33.2). | No |
| **A7** audit hash-chain symmetry | Audit chain semantics (BLAKE3 per-org) are part of the conforming criteria — any future backend must preserve. | No |

**Conclusion:** **K8s-positive.** CH-03 doesn't add blockers; it documents the configurability that ADR-0033 already started. The M7b broker carve-out + leader-election (CHK8S-D-05) remain the load-bearing items at M7b. Ledger stays at 8.

---

## §3.C — User-facing documentation impact map (post-Q9 / CH-22 binding)

| Tier | File | Touched? | Action |
|---|---|---|---|
| Concept | [`concepts/coordination.md`](baby-phi/docs/specs/v0/concepts/coordination.md) | yes — §"Design Decisions" line 69 row replaced with new dedicated §"Storage backend" subsection (3 paragraphs + 7-bullet criteria list) | (a) update in-chunk |
| Decision | `m5_2/decisions/0042-storage-backend-configurable.md` (NEW) | yes — full ADR per §5 below | (a) create in-chunk |
| Architecture | [`m1/architecture/storage-and-repository.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/storage-and-repository.md) | yes — light: bump verified header to note CH-03 ratifies the Repository trait as the conforming-backend contract; cross-reference ADR-0042 | (a) update in-chunk (light) |
| Operations | (no ops doc affected) | no — CH-03 is conceptual; no operator runbook changes | (n/a) |
| User-guide | (no user-guide doc affected) | no — CH-03 is for spec/architecture readers, not operators | (n/a) |

3 file touches. No M5/M5.2 ops-doc or user-guide edits.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D-new-02** | [`m5_1/drifts/D-new-02.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-02.md) | HIGH | `discovered → in-chunk-plan → remediated` | Storage-backend refresh + configurability framing per Q1 decision. Concept-doc text updated; ADR-0042 documents configurability contract; both call sites of "SQLite" wording removed from concept tree. |

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — D-new-02 row Status flipped to `remediated`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip the `coordination.md` § "Design Decisions" — Storage backend row from `contradicted` to `honored`.

**Mid-flight discovery hook:** if the concept-doc rewrite surfaces a related concern (e.g., `coordination.md` mentions another design decision that's drifted, or the Repository trait surface has a documentation gap that should accompany the ADR), surface via `AskUserQuestion` and add a row before phase close. Prior precedent: CH-21 didn't surface mid-flight drifts; CH-22 surfaced one (catalog-listener audit-mode flag) which became ADR-0035.

---

## §5 — ADRs drafted

ADR numbering check: highest currently issued = **ADR-0041** (CH-21, 2026-04-28). Next-free = **ADR-0042**.

> **Note on ADR-0042 reservation**: the M5.3 announcement plan ([`plan/core-philosophy-check/m5-3-announcement-plan-525d2085.md`](baby-phi/docs/specs/plan/core-philosophy-check/m5-3-announcement-plan-525d2085.md)) provisionally tagged ADR-0042 + 0043 for CH-25 + CH-26. Those are placeholders ("likely 0042"). Since CH-03 ships before M5 final seal (CH-24), and CH-25/CH-26 only open post-M5-seal, **CH-03 takes ADR-0042 here**; the M5.3 plan archive can be re-pointed at chunk-open time to ADR-0043 + ADR-0044. (The M5.3 announcement plan archive is itself frozen verbatim; the chunk-open of CH-25 will resolve the placeholder against the next-free-at-the-time number.)

| ADR | Title | Drafted at phase | Decision summary | Flip-to-Accepted phase |
|---|---|---|---|---|
| **ADR-0042** | Storage backend is configurable; SurrealDB is the v0.1 configured impl; 7-criterion conforming contract | P1 | **D42.1** Storage backend is **configurable** at the architecture level via the existing object-safe `domain::Repository` trait (36 async methods; consumed via `Arc<dyn Repository>` per [`baby-phi/CLAUDE.md`](baby-phi/CLAUDE.md) §"phi-core leverage" guidance). Not hardcoded to any one engine. **D42.2** v0.1 ships **SurrealDB** (RocksDB-embedded via `SurrealStore::open_embedded`; remote SurrealDB ≥ 2.0 server via `open_remote` per ADR-0033 §D33.2). Migrations 0001–0009 are SurrealQL. **D42.3** Conforming-backend criteria (any candidate impl MUST satisfy ALL): (1) transactional semantics (atomic BEGIN/COMMIT compound writes); (2) compound-transaction support (multi-entity atomic payloads matching `apply_org_creation` / `apply_project_creation` / `apply_agent_creation` shapes); (3) typed-endpoint edge semantics (RELATION FROM<src> TO<dst> in SurrealDB; equivalent typed-endpoint constraint in any other backend); (4) FLEXIBLE TYPE object support (or equivalent schema-free nested-field carrier) for phi-core wraps (`Session.inner: phi_core::Session`, `LoopRecordNode.inner`, etc.); (5) forward-only idempotent migration runner with a `_migrations` ledger or equivalent applied-version tracking; (6) SCHEMAFULL (or equivalent strict-schema) table declarations for every load-bearing M1 node; (7) UNIQUE index enforcement at schema level (e.g., `bootstrap_credentials_digest`, `secrets_vault_slug`, `identity_agent_id`). **D42.4** The audit-event hash-chain semantics (BLAKE3 per-org chain per [`m1/architecture/audit-events.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/audit-events.md)) are inherited by any conforming backend — they live in the canonical-bytes computation, not the storage layer. **D42.5** Out of scope for CH-03: actual second-backend onboarding (Postgres, etc.). The configurability *abstraction* already exists (Repository trait); the *secondary impl* is a future chunk if/when the need arises. **D42.6** This ADR pairs with [ADR-0033](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) (CH-K8S-PREP) — ADR-0033 §D33.2 already framed `SurrealStore::open_remote` as a swappable URI; ADR-0042 generalizes to *any* conforming backend. | Chunk seal (P2) |

ADR file path: [`m5_2/decisions/0042-storage-backend-configurable.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md).

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant relied on | Re-verification command |
|---|---|---|
| Post-CH-21 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1121; CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh`<br>`/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` (sanity) |
| ADR-0033 (CH-K8S-PREP) | `SurrealStore::open_embedded` + `open_remote` exist; trait-shape pattern established | `grep -n "open_embedded\|open_remote" modules/crates/store/src/lib.rs` ≥ 2 |
| All chunks | 4 CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` |

CH-03 is doc-only; no source-tree change should occur. The cargo workspace test count stays at 1121 (or higher if other chunks have landed in the interim). Spot-running clippy as a sanity is optional but cheap.

---

## §7 — Phases within the chunk

**Phase count: 2** → audit envelope = **1 agent** (small chunk, doc-only — single audit covers concept-fidelity + docs-fidelity in one pass).

### P1 — Concept-doc refresh + ADR-0042 draft + drift lifecycle entry (~0.6d)

**Goal.** Replace `coordination.md` line 69 row with the new §"Storage backend" subsection. Draft ADR-0042 (Status: Proposed). Append `in-chunk-plan` lifecycle entry to D-new-02.

**Deliverables.**

1. **`coordination.md` §"Storage backend" subsection** — replace the single table-row entry at line 69 with a dedicated subsection (placed immediately after the "Design Decisions" table, OR inline as the first row's expanded prose). Sketch:

   ```markdown
   ### Storage backend

   **v0.1 ships with SurrealDB** (RocksDB-embedded via `SurrealStore::open_embedded`;
   remote SurrealDB ≥ 2.0 server via `open_remote` per ADR-0033 §D33.2). Migrations
   0001–0009 are SurrealQL. The implementation lives at
   [`modules/crates/store/`](../../../../modules/crates/store/) behind the
   object-safe `domain::Repository` trait.

   **Storage backend is configurable.** SurrealDB is the v0.1 *configured* choice,
   not a hardcoded architecture. The Repository trait (36 async methods; consumed
   via `Arc<dyn Repository>`) is the conforming-backend contract. A future
   backend candidate (Postgres, etc.) plugs in by providing a parallel impl
   crate; no domain or server code change is required.

   **Conforming-backend criteria.** Any candidate impl MUST satisfy:
   1. Transactional semantics (atomic BEGIN/COMMIT compound writes).
   2. Compound-transaction support (e.g., `apply_org_creation` writes Org +
      Agent + InboxObject + OutboxObject + Identity in one atomic payload).
   3. Typed-endpoint edge semantics (Surreal's `RELATION FROM<src> TO<dst>` or
      equivalent; the 66-variant `Edge` enum in domain assumes typed endpoints).
   4. FLEXIBLE TYPE object (or equivalent schema-free nested-field carrier) for
      phi-core wraps — `Session.inner: phi_core::Session` and `LoopRecordNode.inner`.
   5. Forward-only idempotent migration runner with applied-version tracking
      (current impl: `_migrations` ledger).
   6. SCHEMAFULL (or equivalent strict-schema) table declarations for every
      load-bearing M1 node.
   7. UNIQUE index enforcement at schema level (e.g., `identity_agent_id`,
      `bootstrap_credentials_digest`, `secrets_vault_slug`).

   See [ADR-0042](../implementation/m5_2/decisions/0042-storage-backend-configurable.md).
   A graph-native DB (Neo4j, Memgraph, DuckDB-PGQ) remains a v1 conversation
   once access patterns stabilise; the configurability framing covers that
   transition without architectural rework.
   ```

   The original line-69 table cell collapses to a one-line pointer ("Storage backend → see §Storage backend below") OR is removed entirely if the new subsection sits inside the Design Decisions section.

2. **ADR-0042 file** at [`m5_2/decisions/0042-storage-backend-configurable.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) (NEW). Structure mirrors ADR-0040/0041 header convention + ADR-0033 conforming-criteria pattern. Status: **Proposed** at P1; flipped to **Accepted** at P2 seal. Decision blocks D42.1 through D42.6 per §5 above. Cross-references: ADR-0033 (paired), drift D-new-02 (closed), `m1/architecture/storage-and-repository.md`, `concepts/coordination.md` (post-refresh).

3. **D-new-02 lifecycle entry** appended to the drift file: `2026-04-28 — in-chunk-plan — CH-03 plan approved; concept-doc refresh + ADR-0042 in flight.` (Final `remediated` transition logged at chunk seal in P2.)

4. **`m1/architecture/storage-and-repository.md`** — light verified-header bump noting that CH-03 / ADR-0042 ratifies the Repository trait as the conforming-backend contract.

**Tests.** No code tests for a doc-only chunk. CI guards re-run as discipline:
- `bash scripts/check-doc-links.sh` (must PASS — new ADR + cross-references resolve)
- `bash scripts/check-ops-doc-headers.sh` (must PASS — m1/architecture/storage-and-repository.md header still present)
- `bash scripts/check-spec-drift.sh` (must PASS — no requirement-id changes)
- `bash scripts/check-phi-core-reuse.sh` (must PASS — no code change, no new imports)

**User-facing doc updates.** Per §3.C: `coordination.md` updated, ADR-0042 created, `m1/architecture/storage-and-repository.md` light header bump.

**Confidence target.** ≥ 95% (small chunk, narrow scope).

**Pause discipline.** PAUSE if:
- The 7 criteria don't ALL apply to current SurrealDB impl (i.e., the impl falls short of its own contract — would require either fixing the impl or relaxing the criteria).
- A second drift surfaces during concept-doc rewrite (e.g., another row in `coordination.md` Design Decisions table is also stale).
- The `m1/architecture/storage-and-repository.md` header amendment uncovers a deeper docs gap.

---

### P2 — Drift remediation + ADR Accepted + audit + seal (~0.4d)

**Goal.** Flip ADR-0042 Proposed → Accepted. Close D-new-02 terminally. Update drifts/README.md + concept-audit-matrix.md. Spawn 1 audit agent. Seal.

**Deliverables.**

1. **ADR-0042** flipped from `Proposed` → `Accepted`.
2. **D-new-02** Status flipped from `discovered` (or `in-chunk-plan` if P1 entry was added) to `remediated`. Final lifecycle entry: `2026-04-28 — remediated — CH-03 chunk-seal — concept-doc refreshed; ADR-0042 ratifies the configurable backend + 7-criterion contract.`
3. **`drifts/README.md`** — D-new-02 row Status column flipped to `remediated`; `Closes at` column shows `CH-03 ✓`.
4. **`_concept-audit-matrix.md`** — flip the `coordination.md` § Design Decisions / Storage-backend row from `contradicted` to `honored`; cite ADR-0042 in the evidence column.
5. **Concept-doc verified-headers bumped** on `concepts/coordination.md` (note CH-03 amendment).
6. **Spawn 1 audit agent** per §11.

**Tests.** All §8 named checks green (CI guards + post-CH-21 baseline preserved); audit returns PASS on every claim.

**Confidence target.** ≥ 99% (chunk seal target — doc-only chunks have lower-variance audit outcomes than code chunks).

**Pause discipline.** PAUSE if the audit reports a finding (e.g., the 7 criteria don't fully match what the SurrealDB impl actually delivers; or a cross-reference link in ADR-0042 is broken; or the matrix row flip creates a contradiction with another claim).

---

## §8 — Tests summary

- **Expected total at chunk close:** post-CH-21 baseline (~1121) **unchanged**. Doc-only chunk; zero new tests; zero test removals.
- **Layer breakdown:** none (doc-only).
- **CI guard checks:** 4 (doc-links, ops-doc-headers, phi-core-reuse, spec-drift) — all expected PASS.
- **Spot sanity (optional):** `cargo build -j 4 --workspace` to confirm zero source-tree change has zero build impact.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive this plan verbatim (mandatory first action)

1. Generate plan-file token: `openssl rand -hex 4`.
2. **Copy this plan file verbatim** to `baby-phi/docs/specs/plan/build/<8hex>-ch-03-storage-backend-configurability.md`. No edits during the copy.
3. Update placeholders in lines 4–5 of the archived plan only.
4. Run `bash scripts/check-doc-links.sh` to confirm relative links resolve.
5. Verify: `head -5 baby-phi/docs/specs/plan/build/<8hex>-ch-03-*.md` shows the verified-by-Claude-Code header on line 1.

This step matches the chunk-lifecycle-checklist Step 1 and the precedent set by CH-01 (`2aa37c80`), CH-02 (`16fd9a3a`), CH-22 (`c5f201bb`), CH-06 (`acd383e2`), CH-16 (`2ae4fabe`), CH-21 (`bb95cd12`), and the M5.3 announcement (`525d2085`).

**Reading list (mandatory before continuing):**
1. [`concepts/coordination.md`](baby-phi/docs/specs/v0/concepts/coordination.md) — full (~80 lines), focus on lines 63–78 (Design Decisions table) + the Storage backend row at line 69.
2. [`m5_1/drifts/D-new-02.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-02.md) — full content.
3. [`m1/architecture/storage-and-repository.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/storage-and-repository.md) — for the cross-reference bump.
4. [ADR-0033](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) — for the trait-shape-with-conforming-criteria precedent.
5. [ADR-0040](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md) + [ADR-0041](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0041-memory-extracted-event-and-audit.md) — for the most-recent ADR header convention.
6. [`modules/crates/store/src/lib.rs`](baby-phi/modules/crates/store/src/lib.rs) — module-level doc-comment (lines 1–15) + constructor signatures.
7. `modules/crates/store/migrations/` listing — to confirm 9 migrations + naming convention for the criteria evidence.
8. `forward-scope/22035b2a-...md` §1 CH-03 row + Q1 decision context (lines 51–59 + ~450–456).

**Carry-forward invariants (verified green at chunk-open):**
- `cargo test --workspace -- --test-threads=1` ≈ 1121 (post-CH-21 baseline; or higher if intervening chunks land).
- 4 CI guards green.
- D-new-02 status `discovered` (or `in-chunk-plan` after P1 entry).
- ADR-0034..0041 Accepted.
- `git diff --stat HEAD -- modules/` empty (CH-03 should remain at empty for source-tree changes throughout).
- Highest applied migration is 0009 (CH-16's; no migration in CH-03).
- Highest issued ADR is 0041 → next-free = 0042.

**ADR numbering caveat (per §5 above):** the M5.3 announcement plan ([`m5-3-announcement-plan-525d2085.md`](baby-phi/docs/specs/plan/core-philosophy-check/m5-3-announcement-plan-525d2085.md)) provisionally tagged ADR-0042 for CH-25. If CH-03 ships first (this is the assumption — CH-03 has no prereqs and is doc-only), **CH-03 takes ADR-0042**; CH-25 becomes ADR-0043 at its chunk-open. The M5.3 plan archive is frozen verbatim; the actual numbering resolves at each chunk-open against next-free-at-the-time.

**Cargo command convention** (per memory): all cargo invocations use `-j 4`. Tests serialise via `--test-threads=1`. (Not relevant for CH-03 — doc-only — but discipline preserved.)

---

## §10 — Close criteria (5-aspect, post-Q9)

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — zero source-tree change (verified via `git diff --stat HEAD -- modules/` empty); workspace builds + tests still pass at post-CH-21 baseline.
- **Docs aspect** — TWO scopes:
  - *Governance tier*: D-new-02 lifecycle entry + Status flip to `remediated`; `_concept-audit-matrix.md` Storage-backend row flipped `contradicted → honored`; `drifts/README.md` updated; ADR-0042 Accepted; `concepts/coordination.md` verified-header bumped.
  - *User-facing tier*: §3.C 3 actions completed in-chunk (concept-doc refresh + new ADR + light architecture-doc bump).
- **phi-core leverage aspect** — import-count delta = **0**; positive close-audit greps all ≥ expected; forbidden-duplication greps all 0; `check-phi-core-reuse.sh` exit 0.
- **Concept alignment aspect** — `coordination.md` §"Storage backend" row at target-status `honored`. The deferred "graph-native DB at v1" framing is preserved (carry-forward).
- **K8s readiness aspect** — §3.B 7-axis populated; CH-03 declared K8s-positive (clarifies existing ADR-0033 framing); ledger stays at 8.

**Two confidence % (each with named numerator/denominator):**

- **Implementation confidence** = `claims-verified-honored / claims-in-scope` = target **5/5 = 100%**. The 5 claims:
  1. `coordination.md` §"Storage backend" subsection states v0.1 ships SurrealDB.
  2. Configurability framing present (backend is not hardcoded; Repository trait is the contract).
  3. 7-criterion conforming-backend list present (transactional / compound-tx / RELATION / FLEXIBLE TYPE / migration idempotency / SCHEMAFULL / UNIQUE).
  4. ADR-0042 Accepted with sub-decisions D42.1–D42.6 covering configurability, current impl, criteria, audit-chain inheritance, out-of-scope, ADR-0033 pairing.
  5. Drift D-new-02 lifecycle entry transitioned to `remediated`; matrix row flipped.

- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check / doc-pages-touched` = target **3/3 = 100%**. 3 doc pages: `coordination.md`, ADR-0042, `m1/architecture/storage-and-repository.md`.

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-pass).** Target ≥ 99% (chunk seal — doc-only).

---

## §11 — Post-chunk independent audit plan

**Agent count.** 2 phases, doc-only, narrow scope → **1 agent** (combined concept + docs fidelity audit). Smaller envelope than CH-21's 2-agent setup, matching the chunk's scope.

### Audit Agent A — Concept + docs fidelity

> **Locked prompt** (drafted at Step 2; fired at P2 seal):
> You are auditing CH-03 (storage-backend concept refresh + configurability framing) in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code or these docs. The chunk plan is at `docs/specs/plan/build/<8hex>-ch-03-storage-backend-configurability.md`.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence (file:line + grep result). Read-only.
>
> 1. `concepts/coordination.md` no longer references "SQLite" anywhere. Run `grep -n "SQLite" docs/specs/v0/concepts/coordination.md` — expect 0 hits.
> 2. `concepts/coordination.md` has a new §"Storage backend" subsection (or expanded entry) that names SurrealDB as the v0.1 backend.
> 3. The new section states the backend is **configurable** (not hardcoded).
> 4. The new section enumerates the 7-criterion conforming-backend contract: (1) transactional semantics, (2) compound-tx support, (3) typed-endpoint edge semantics / RELATION, (4) FLEXIBLE TYPE object, (5) idempotent migration runner, (6) SCHEMAFULL, (7) UNIQUE index enforcement.
> 5. ADR-0042 exists at `docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md` with `**Status: Accepted**`.
> 6. ADR-0042 contains sub-decisions D42.1 through D42.6 covering configurability framing, current impl, criteria, audit-chain inheritance, out-of-scope, ADR-0033 pairing.
> 7. ADR-0042 cross-references ADR-0033 (paired) + drift D-new-02 (closed) + `m1/architecture/storage-and-repository.md`.
> 8. Drift `D-new-02` Status = `remediated`; lifecycle entry `2026-04-28 — remediated — CH-03 chunk-seal` present.
> 9. `m5_1/drifts/README.md` row for D-new-02 reflects remediation (Status column = remediated).
> 10. `m5_1/drifts/_concept-audit-matrix.md` row for `coordination.md` § Design Decisions / Storage-backend flipped from `contradicted` to `honored`.
> 11. concept-`coordination.md` `Last verified` header bumped with CH-03 amendment.
> 12. `m1/architecture/storage-and-repository.md` `Last verified` header bumped with CH-03 amendment + ADR-0042 cross-reference.
> 13. Zero source-tree changes: `git diff --stat HEAD -- modules/` returns empty.
> 14. CI guards green: run `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` — all 4 PASS.
> 15. Workspace test count unchanged: `cargo test --workspace -- --test-threads=1` shows ~1121 passed (or higher if intervening chunks landed) / 0 failed.
> 16. CH-21 invariants intact: ADR-0040/0041 still Accepted; D6.1 still remediated; CH-21 acceptance test still passes.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 500 words.

**Seal-blocking rule.** The audit must report PASS on every check, OR each FAIL must be either (a) fixed in-chunk before seal, (b) reframed via user-approved ADR amendment, or (c) converted to a new drift file with explicit future-chunk assignment.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (all 4 must PASS)
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (sanity — should be unchanged)
/root/rust-env/cargo/bin/cargo build -j 4 --workspace
# (Optional) cargo test --workspace -- --test-threads=1 — expect ~1121 / 0 failed

# 3. Chunk-specific positive greps
grep -n "SurrealDB" docs/specs/v0/concepts/coordination.md                                       # ≥ 1
grep -n "configurable" docs/specs/v0/concepts/coordination.md                                    # ≥ 1
grep -c "transactional semantics\|compound-transaction\|RELATION\|FLEXIBLE TYPE\|migration\|SCHEMAFULL\|UNIQUE" \
  docs/specs/v0/concepts/coordination.md                                                           # ≥ 7
ls docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md              # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md  # 1
grep -c '^### D42\.[1-6]\b' docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md         # 6

# 4. Chunk-specific negative greps
grep -n "SQLite" docs/specs/v0/concepts/coordination.md                                          # 0
grep -rn "SQLite" docs/specs/v0/concepts/                                                         # 0 (or low; check none reference storage-backend)
git diff --stat HEAD -- modules/                                                                  # empty (zero source-tree changes)

# 5. Drift terminal closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-02.md   # 1
grep "D-new-02" docs/specs/v0/implementation/m5_1/drifts/README.md | grep -c "remediated"        # ≥ 1

# 6. Concept-audit matrix flip
grep -A2 "Storage backend" docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md | grep -c "honored"  # ≥ 1

# 7. ADR location + numbering sanity
ls docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE '^[0-9]{4}' | sort -u | tail -3
# Expect: 0040, 0041, 0042 (CH-03 takes 0042)

# 8. Cross-references intact (ADR-0042 cites ADR-0033, drift D-new-02, m1/architecture)
grep -n "ADR-0033\|D-new-02\|storage-and-repository" docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md

# 9. Plan archive exists at build/<8hex>-...
ls docs/specs/plan/build/*-ch-03-storage-backend-configurability.md
```

No cargo invocations needed beyond the optional sanity build.

---

## What this plan does NOT do

- **No code change.** Doc-only chunk per Q1 decision. The Repository trait stays unchanged; no second-backend impl ships.
- **No migration.** Migration count stays at 9 (latest = `0009_identity_node.surql` from CH-16).
- **No second-backend onboarding.** Postgres / DuckDB-PGQ / etc. are explicitly Out-of-Scope per ADR-0042 §D42.5.
- **No `Repository` trait surface change.** The 36-method trait already IS the configurability abstraction; CH-03 documents that, doesn't extend it.
- **No M7b deferred-block changes.** CHK8S-D-01 through CHK8S-D-08 are independent K8s items; CH-03 references them as cross-context but doesn't move them.

---

## Critical files for implementation

**New files:**
- `docs/specs/v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md`

**Modified files:**
- `docs/specs/v0/concepts/coordination.md` — replace line 69 row with new §"Storage backend" subsection
- `docs/specs/v0/implementation/m1/architecture/storage-and-repository.md` — light verified-header bump
- `docs/specs/v0/implementation/m5_1/drifts/D-new-02.md` — Status flip + lifecycle entries
- `docs/specs/v0/implementation/m5_1/drifts/README.md` — D-new-02 row Status column
- `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` — Storage-backend row flip

**Reused (no edit) — for context only:**
- `modules/crates/store/src/lib.rs` (read for ADR criteria evidence)
- `modules/crates/store/migrations/` listing (read for migration count)
- `modules/crates/domain/src/repository.rs` (read for trait-shape signatures)
- `docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md` (paired ADR — cross-reference target)
- `docs/specs/v0/implementation/m5_2/decisions/0040-...md` + `0041-...md` (style precedent)

---

## Estimated effort

~1 engineer-day total:
- 30 min — chunk-open ritual (token, archive, header bump on archive copy, doc-link check).
- 90 min — drafting `coordination.md` §"Storage backend" subsection (re-reading concept-doc context; iterating on framing; preserving v1-graph-DB carry-forward).
- 90 min — drafting ADR-0042 (header + Context + 6 sub-decisions + Conforming criteria + Out-of-Scope + References + Cross-refs).
- 30 min — `m1/architecture/storage-and-repository.md` light header bump + cross-reference.
- 30 min — drift remediation: D-new-02 Status flip + README + matrix row flip.
- 20 min — concept-doc verified-header bump (`coordination.md`).
- 30 min — verification recipe pass + spot-fixes.
- 30 min — Audit Agent A spawn + review + (if PASS) seal.

**Total: ~5.7 hours of focused doc work**, comfortably within the 1-day budget.
