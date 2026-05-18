<!-- Last verified: 2026-05-10 by Claude Code (chunk-planner v10 + v11; plan v2 — F1.B re-plan after gate-1 user lock divergence (2026-05-10); cycle hex `240616a4`) -->

# CH-20 — Bucket C convention confirm-in-place (16 drifts, doc-only)

**Cycle hex:** `240616a4`
**Slug:** `ch-20-bucket-c-confirm-in-place`
**Type:** **doc-only** — no code changes, no test count change, no migration, +0 phi-core delta.
**Forward-scope row:** [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md:185-187`](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (§5 row also at line 428).
**Per-chunk template:** [`m5_1/process/per-chunk-planning-template.md`](../../../v0/implementation/m5_1/process/per-chunk-planning-template.md).
**Drifts closed in this chunk:** D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6 — **16 drifts** (forward-scope row's "(existing 14 items)" parenthetical is empirically off-by-2; see §1 reconciliation).
**ADR drafted:** **ADR-0058** — Bucket C convention confirm-in-place (sub-decisions §D58.1–§D58.10 grouping the 16 drifts into 5 thematic conventions + 3 META decisions for the new peer tier).
**ADR home:** `m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md` (next-free slot per `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/ | tail -1` returning `0057-bucket-b-convention-ratification.md`).
**NEW directory:** [`docs/specs/v0/conventions/`](../../../v0/conventions/) (does not exist at chunk-open; verified via `ls baby-phi/docs/specs/v0/conventions/ → No such file or directory` at chunk-open 2026-05-10 + re-verified at v2 re-plan time). The chunk's deliverable creates this directory + ships **5 separate convention docs** inside it (one per thematic area).
**NEW convention docs (5 files per F1.B user-lock):**
1. `docs/specs/v0/conventions/persistence.md` (covers D1.1, D1.2, D2.1, D2.2, D4.4 — 5 drifts; schema mechanics + write verbs)
2. `docs/specs/v0/conventions/wrap-pattern.md` (covers D1.3, D3.3, D4.3, D4.6 — 4 drifts; phi-core wrap idioms)
3. `docs/specs/v0/conventions/event-bus-wiring.md` (covers D3.1, D3.2, D3.4, D4.5, D6.4 — 5 drifts; event-bus + listener seam + cross-ref to CH-19 §D57.1 for audit-event placement)
4. `docs/specs/v0/conventions/cli-patterns.md` (covers D7.3 — 1 drift; CLI scope-addition discipline)
5. `docs/specs/v0/conventions/web-patterns.md` (covers D7.6 — 1 drift; Next.js 14 server-action shape)

**No `README.md` index file** (per orchestrator gate-1 guidance: user chose pure-split F1.B, not hybrid F1.C; the directory's structure speaks for itself; ADR-0058 §D58.8 documents the peer-tier shape + lists the 5 files as the canonical entry-point).

---

## Forks for orchestrator

**F1 LOCKED at gate-1: F1.B — split across multiple files (one per convention area).** Diverged from planner v1 recommendation (F1.A single consolidated file). User explicitly chose multi-file granularity at the new `v0/conventions/` peer tier (orchestrator-mediated lock, 2026-05-10). All other forks resolved at planner-recommendation level.

- **F1 (LOCKED — F1.B)** — Convention-doc granularity. **Options (historical):**
  - **(a) Single consolidated file** at `v0/conventions/persistence-and-wiring.md` (~600–900 words; 5 thematic sections covering all 16 drifts). **Planner v1 recommendation — REJECTED at gate-1 user-lock.**
  - **(b) Split across multiple files (one per convention area)** at `v0/conventions/{persistence,wrap-pattern,event-bus-wiring,cli-patterns,web-patterns}.md`. **USER LOCK at gate-1 (2026-05-10).** Final file count: **5** (rationale below).
  - **(c) Hybrid** (single file with named sub-§ anchors). REJECTED at gate-1 user-lock (user chose pure-split, not hybrid).

  **Cross-cycle pattern note (chunk-planner v10):** planner-recommendation diverged from user-lock in **4 of last 6 cycles** (CH-15 cycle hex `c3f46f17` F5.B over F5.A; CH-17 cycle hex `40c4d759` F5.B over F5.A; CH-18 cycle hex `c77937bc` F3.B over F3.A; CH-20 cycle hex `240616a4` F1.B over F1.A). CH-19 cycle hex `2c520ba7` was the lone non-divergent cycle. The 2-cycle "doc-only chunks reliably avoid divergence" hypothesis from CH-19 is now FALSIFIED by CH-20 — even doc-only chunks see user divergence when granularity choices are present. User systematically prefers either (i) tighter scope when audit envelope ≤ medium (CH-15/17/18) OR (ii) finer-grained separability for future deep-linking (CH-20). Planner-recommendation in v2 stays surfaced (recommendation field is preserved per chunk-planner v9 surfacing-not-suppressing approach), but user is free to lock either option without anchoring on planner-recommendation. **CH-21+ planner should anticipate divergence as the modal outcome, not the exception.**

  **Final file count rationale (5 files):** Persistence is one coherent area (5 drifts: D1.1+D1.2 schema mechanics + D2.1+D2.2+D4.4 write verbs). Wrap-pattern is one coherent area (4 drifts on phi-core wrap idioms). Event-bus-wiring covers 4 drifts (D3.1+D3.2+D3.4+D4.5) + folds in D6.4 audit-event placement as a cross-ref subsection (D6.4 is a single-sentence cross-ref to CH-19 §D57.1; doesn't merit its own file). CLI patterns covers D7.3 only — kept separate from web patterns because the CLI completion-help discipline + the web Next.js shape are different sub-disciplines with different review triggers. Web patterns covers D7.6 only — same separation rationale. **Total: 5 files.** A 6-file alternative (D6.4 as standalone `audit-event-placement.md`) was considered + rejected because D6.4 is a pure cross-ref to CH-19 §D57.1 with no new convention content — folding it into `event-bus-wiring.md` as a §"Audit-event placement cross-ref" subsection is more cohesive. A 4-file alternative (merging cli + web into `surface-patterns.md`) was considered + rejected because the CLI + web review triggers are independent (CLI completion-help test stays green vs Next.js 15 migration trigger).

  **No `README.md` index** (per orchestrator gate-1 guidance: user chose pure-split, not hybrid). Discoverability via the directory listing + ADR-0058 §D58.8's documentation of the peer-tier shape. CH-21+ planner may ship a README index later if the directory grows beyond ~10 files.

- **F2 — ADR-vs-convention-doc shape (planner-recommendation lock).** **Options:**
  - **(a) Both: ADR-0058 captures the decision rationale + each sub-decision; convention-docs are the operator/reviewer-facing distillation** (ADR is governance-tier, convention-docs are how-to-tier). **Planner-recommendation; expected lock.**
  - (b) Convention-docs only (no new ADR; the convention-docs themselves record the decision rationale). Drawback: breaks Q6 + ADR-body-checklist compliance for any chunk that makes a non-trivial architectural or convention decision (per per-chunk-planning-template §5). 16 drifts is non-trivial.
  - (c) ADR only (no separate convention-docs). Drawback: forward-scope row line 187 explicitly mandates convention-doc artefacts at `v0/conventions/*.md` (note the `*.md` glob — F1.B's 5-file shape MATCHES the forward-scope's plural-glob form better than F1.A's single-file form did).

  **Planner reasoning for (a):** forward-scope binds the convention-doc(s); per-chunk-planning-template §5 binds the ADR. Both. Different audiences: ADR-0058 records decision provenance + cross-refs; convention-docs are reviewer-friendly "do this, not that" guidance. The two artefact tiers cross-reference each other (ADR §"Cross-references" cites all 5 convention-doc URLs; each convention-doc preamble cites ADR-0058 §D58.N for that file's authority).

- **F3 — Drift transition target (planner-recommendation lock).** All 16 drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../../v0/implementation/m5_1/process/drift-lifecycle.md) AND per forward-scope row line 187 verbatim ("Status of each drift flips `discovered → accepted-as-is`"). **Planner-recommendation: `accepted-as-is` for all 16 (matches CH-19/ADR-0057 §F3 precedent).**

  **Verified at chunk-open 2026-05-10 + re-verified at v2 re-plan time:** all 16 drift files carry `Status: discovered` per their frontmatter; transition target uniform.

---

## §1 — Context & principle

**Why this chunk.** The M5.1/P3 forward-scope inventory split the 60-drift catalogue into three buckets: A (load-bearing scope gap), B (underspecified shape choice), C (convention/pattern decision). **CH-20 is the dedicated convention-confirm-in-place chunk for the C-bucket drifts** that survived the M5/P5–P7 close — sibling to CH-19 (Bucket B ratification, closed 2026-05-10 / cycle hex `2c520ba7`). Where Bucket B drifts are *shape choices* the implementer picked between alternatives (= ratify via ADR), Bucket C drifts are *conventions* the codebase consistently follows that the original plan/concept-doc was silent on (= confirm-in-place via convention-docs + ADR sub-decisions). The shipped convention works; what's missing is the formal acceptance + the recoverable record explaining why this shape rather than another.

The chunk produces (a) **5 new convention-docs** under the new `v0/conventions/` directory — the FIRST files under the new peer tier — covering 5 thematic conventions across the 16 drifts, (b) one consolidated **ADR-0058** with sub-decisions §D58.1–§D58.10 referencing the 5 convention-docs as the operator/reviewer surface, (c) drift-status flips `discovered → accepted-as-is` for all 16, (d) `_concept-audit-matrix.md` row refreshes (limited — most C-bucket drifts are below matrix granularity per CH-19/ADR-0057 §"Label-coverage rollup notes" precedent), (e) `drifts/README.md` index Status flips. **No production code changes. No migrations. No test changes. phi-core import baseline preserved at 57.**

**Drift count reconciliation (forward-scope says "14"; actual is 16).** The forward-scope row at line 185 reads *"CH-20 — Bucket C convention confirm-in-place (existing 14 items)"* but the explicit drift list at line 186 contains **16 IDs**: D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6. The "14" parenthetical is a counting error in the forward-scope authoring — verified by `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/` showing all 16 files exist with Status `discovered`. **CH-20 honors the empirical 16-drift count + flags the off-by-2 in the forward-scope row as a CH-20-noticed drift in §6 below** (not a new drift file — the forward-scope row is a planning artefact, not concept-doc; the parenthetical fix lands inline as a CH-20 amendment to the forward-scope row at chunk-seal P3).

**Relationship to CH-19.** CH-19 / ADR-0057 §D57.1 (D5.2: audit-event placement convention) is the immediate cross-ref target for CH-20's D6.4 — D6.4 ratifies that **D5.2's platform-tier convention extends to system-agents** (page-13 reconfigure / add / disable / archive events live at `server::platform::system_agents::audit_events`). CH-20's `event-bus-wiring.md` cross-refs CH-19 §D57.1 in a §"Audit-event placement cross-ref" subsection rather than re-ratifying the shared convention. This matches the forward-scope row line 187 *"audit-event placement (cross-ref CH-19)"*.

**Quality-over-speed restatement.** *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-20 application: the 16 Bucket-C drifts have lived as `discovered` in the catalogue since M5/P1–P7 close (April 23–24, 2026); shipping CH-20 closes them by promoting their convention-status from "shipped but undocumented" to "shipped + ADR-0058 + 5 convention-docs + matrix-side rollup honored." The chunk's value is removing silent-convention drift and creating the FIRST `v0/conventions/` directory entries — establishing the convention-doc-as-first-class-artefact pattern for future M6+ reviewer guidance. **F1.B's multi-file shape additionally establishes the per-area-file convention** for future Bucket-C-style ratifications (one thematic convention area = one file), which CH-21+ planners can mirror.

**Forward-scope reference.** [forward-scope row at line 185-187](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md). Severity-row binding at line 428 of §5 table: `LOW | 1d | various (conventions doc) | — | yes`. F1.B raises the deliverable count from "1 doc" to "5 docs" but each doc is small (~150–250 words); total prose volume is comparable to the v1 single-file estimate (~700–900 words across 5 files vs ~600–900 words in 1 file).

---

## §2 — Concept alignment walk

Every concept doc whose claim a Bucket-C drift touches. **All 16 drifts are `concept-silent-plan-filled-gap` per their `Classification` field** — none are `contradicts-concept`. Status at chunk-open documents the silence; status at chunk-close documents the **convention-doc + ADR sub-decision** that closes the silence (the concept doc itself stays silent — concept docs are source-of-truth at the semantic layer; below-granularity conventions are documented at `v0/conventions/`).

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close (NEW: convention-doc file) |
|---|---|---|---|---|
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"table-per-node-tier" (persistence ontology) | *"Concept docs treat `session` / `loop_record` / `turn` as three distinct node tiers with governance-owned columns layered on top of phi-core's wrapped types. They do not specify migration mechanics (DEFINE TABLE vs DEFINE FIELD)."* (D1.1 paraphrase per drift body line 19) | silent-in-code (concept silent on migration mechanics; plan implied fresh DEFINE TABLE) | accepted-as-is — `persistence.md` §"Schema mechanics" subsection 1: DEFINE FIELD on pre-existing scaffold; ADR-0058 §D58.1 records the SurrealDB-DDL-mechanic convention |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"edges have a direction" (line 87+ region) | *"A session runs within a project (session → project directional edge). The M1-era reverse direction (`agent → session`) was a scaffold never matched to concept."* (D1.2 paraphrase) | silent-in-code (concept silent on retype mechanics; plan implied fresh DEFINE TABLE) | accepted-as-is — `persistence.md` §"Schema mechanics" subsection 2: REMOVE + DEFINE for direction retype on no-writer-scaffold; ADR-0058 §D58.1 secondary |
| [`concepts/phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) | §"wrap: baby-phi field holds phi-core type" | *"Baby-phi wraps phi-core types by holding them as a field (`inner: phi_core::X`). The wrap extends with phi-only governance fields alongside the phi-core value. Serde mechanics (flatten vs nested) are not specified at the concept layer."* (D1.3 paraphrase) | silent-in-code (concept silent on serde flatten-vs-nested) | accepted-as-is — `wrap-pattern.md` §"Nested-inner form" subsection: nested-`inner` form (NOT `#[serde(flatten)]`); ADR-0058 §D58.2 + cross-ref existing ADR-0029 §D29.1 |
| [`concepts/phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) | §"wrap: baby-phi field holds phi-core type" | (D3.3 — same wrap-pattern concept claim as D1.3, applied to `BabyPhiSessionRecorder`) *"Wrap pattern means the baby-phi type holds the phi-core type as a field. Interior-mutability wrapper (e.g. `Arc<Mutex<_>>`) is a concurrency-implementation detail below concept granularity."* (D3.3 paraphrase) | silent-in-code (concept silent on `Arc<Mutex<_>>` for shared interior mutability) | accepted-as-is — `wrap-pattern.md` §"Interior-mutability wrap" subsection: `Arc<Mutex<phi_core::SessionRecorder>>` for shared-mut wrap of `&mut self` phi-core APIs; ADR-0058 §D58.2 + cross-ref existing ADR-0029 §D29.2 |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | (persistence layer below concept granularity) | *"Concepts specify node creation semantics (fresh id, duplicate-rejection on UNIQUE violations) but not the SurrealDB DDL verb. Concepts treat persistence as an opaque substrate."* (D2.1 paraphrase) | silent-in-code (concept silent on CREATE-vs-UPDATE-vs-DELETE-then-UPSERT verb choice) | accepted-as-is — `persistence.md` §"Write verbs" subsection 1: CREATE-not-UPDATE-as-upsert + branch on `current_profile.is_some()` for write/upsert split; ADR-0058 §D58.3 |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"edges are first-class" — DDL mechanics below granularity | *"Edges (RELATION tables) link nodes with typed FROM/TO endpoints. Concepts are silent on SurrealDB syntax for how the endpoints are bound in RELATE statements."* (D2.2 paraphrase) | silent-in-code (concept silent on inline-`type::thing` vs LET-first binding) | accepted-as-is — `persistence.md` §"Write verbs" subsection 2: LET-first RELATE; reviewer-discipline rule rejecting inline `type::thing(...)` in RELATE FROM/TO slots |
| [`concepts/agent.md`](../../../v0/concepts/agent.md) + [`concepts/coordination.md`](../../../v0/concepts/coordination.md) | (event field naming below concept granularity) | *"Domain events carry agent-lifecycle facts. The event's field naming is below concept granularity (concept describes the kind-distinction, not the JSON key name)."* (D3.1 paraphrase) | silent-in-code (concept silent on serde-tag-discriminator collision avoidance) | accepted-as-is — `event-bus-wiring.md` §"Serde-tag-collision rename" subsection: serde tag-discriminator collision avoidance via `agent_kind` rename; ADR-0058 §D58.4 |
| [`concepts/coordination.md`](../../../v0/concepts/coordination.md) | §"event-driven reactivity" — listener seam below granularity | *"Governance reactivity flows through an event bus; listeners subscribe to domain events and fire side-effects. Concept is silent on the constructor seam (constructor method vs free function)."* (D3.2 paraphrase) | silent-in-code (concept silent on `AppState::new` vs free-function listener wiring) | accepted-as-is — `event-bus-wiring.md` §"Free-function listener seam" subsection: free-function `build_event_bus_with_m5_listeners` as the single wiring seam; ADR-0058 §D58.4 secondary |
| [`concepts/permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) | §"Template C" (org-scoped) + §"Template D" (project-scoped) | *"Template C fires on org-level events (`MANAGES` edges live at org scope); Template D fires on project-level events (`HAS_AGENT_SUPERVISOR` lives at project scope). The scoping asymmetry is concept-mandated."* (D3.4 paraphrase per drift body line 19) | silent-in-code (concept specifies the scoping; plan was silent on the trait-split that expresses it) | accepted-as-is — `event-bus-wiring.md` §"Trait-split-by-scope for resolvers" subsection: `TemplateCAdoptionArResolver(OrgId)` vs `TemplateDAdoptionArResolver(ProjectId)`; ADR-0058 §D58.4 tertiary |
| [`concepts/permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) + [`concepts/phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) | §"tools are resources" + §AgentTool | *"Tools are typed resources bound to agents via `HAS_TOOL` edges. Tools are phi-core primitives (trait objects). Wire surface is a governance projection of the tool metadata."* (D4.3 paraphrase) | silent-in-code (concept silent on HTTP wire shape `Vec<Box<dyn AgentTool>>` vs `Vec<ToolSummary>`) | accepted-as-is — `wrap-pattern.md` §"HTTP wire-shape projection" subsection: `Vec<ToolSummary>` HTTP wire shape vs trait-object runtime path; ADR-0058 §D58.2 tertiary |
| [`concepts/agent.md`](../../../v0/concepts/agent.md) | §"AgentProfile binds a blueprint" | *"Each agent has a profile capturing the phi-core blueprint + governance metadata. Profile creation vs update is CRUD mechanics below concept granularity."* (D4.4 paraphrase) | silent-in-code (concept silent on UPDATE-as-upsert-no-op vs branch-on-existence) | accepted-as-is — `persistence.md` §"Write verbs" subsection 3: branch-on-existence for upsert-vs-create; cross-ref D2.1 root-cause; ADR-0058 §D58.3 secondary |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"edges are typed by their relation" | *"Edges have distinct types with distinct semantics. The repository API surface preserves type-safety at edge writes."* (D4.5 paraphrase) | silent-in-code (concept silent on typed-writer vs generic-`create_edge` shape) | accepted-as-is — `event-bus-wiring.md` §"Typed-writer-per-edge-type" subsection: typed-writer-per-edge-type at Repository trait surface (`write_uses_model_edge`, etc.); ADR-0058 §D58.4 quaternary |
| [`concepts/permissions/05-memory-sessions.md`](../../../v0/concepts/permissions/05-memory-sessions.md) | §"session lifecycle — started, live, ended" | *"A session has one identity that persists through its lifecycle. Recording the session must be idempotent with respect to that identity."* (D4.6 paraphrase) | silent-in-code (concept silent on launch-chain-vs-recorder coordination shape) | accepted-as-is — `wrap-pattern.md` §"Dual-mode discriminator" subsection: dual-mode `SessionLaunchContext` with `first_loop_id: Option<LoopId>` discriminator; ADR-0058 §D58.2 quaternary |
| [`concepts/system-agents.md`](../../../v0/concepts/system-agents.md) | §"reconfiguration audit trail" | *"Page-13 handler events (reconfigure / add / disable / archive) produce audit entries. Concept silent on module placement."* (D6.4 paraphrase) | silent-in-code (concept silent on platform-tier vs domain-tier event-builder placement) | accepted-as-is — `event-bus-wiring.md` §"Audit-event placement cross-ref" subsection: convention is CH-19/ADR-0057 §D57.1; D6.4 is the same convention applied to system-agents page-13; ADR-0058 §D58.5 cross-refs CH-19 §D57.1 rather than re-ratifying |
| [`concepts/permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"preview mode" (Decision trace as operator inspection surface) | *"Operators may inspect permission Decision before committing an action."* (D7.3 paraphrase) | silent-in-code (concept silent on CLI subcommand surface; plan listed 4, shipped 5) | accepted-as-is — `cli-patterns.md` §"CLI scope-addition discipline" sole subsection: 5th subcommand wraps existing HTTP route; CLI completion-help test pins the addition; ADR-0058 §D58.6 |
| (concept docs silent on frontend framework idioms — below concept granularity) | (no §-anchor) | *"No direct concept claim on Next.js server-action shape."* (D7.6 paraphrase) | silent-in-code (concept-silent-plan-filled-gap; convention is below concept granularity) | accepted-as-is — `web-patterns.md` §"Next.js server-action shape" sole subsection: hybrid pattern (sibling `actions.ts` + inline `<form action={run}>` closures with `"use server"` directives); ADR-0058 §D58.7 |

**Coverage:** **8 concept docs touched** (`ontology.md`, `phi-core-mapping.md`, `agent.md`, `coordination.md`, `permissions/07-templates-and-tools.md`, `permissions/05-memory-sessions.md`, `permissions/04-manifest-and-resolution.md`, `system-agents.md`). **Permissions subtree hooked:** `permissions/04` + `permissions/05` + `permissions/07` (per per-chunk-template §2 rule), so [`permissions/README.md`](../../../v0/concepts/permissions/README.md) MUST be cited as the entry-invariants source. **CH-20 cites `permissions/README.md` here as the entry-invariants source; CH-20 does NOT modify `permissions/README.md`.**

**phi-core-mapping hook:** `concepts/phi-core-mapping.md` is touched at §"wrap: baby-phi field holds phi-core type" (D1.3 + D3.3 + D4.3 conventions all extend the wrap pattern documented there). **No phi-core-type duplication; no phi-core import-count change.**

**Important framing note (per CH-19/ADR-0057 §"Label-coverage rollup notes" precedent):** the convention-docs live at `v0/conventions/`, NOT under any concept doc. Concept docs are source-of-truth at the *semantic* layer; conventions document *shape choices below semantic granularity* (DDL mechanics, serde forms, listener seams, web-framework idioms). Per the new `v0/conventions/` directory's role, this is the FIRST chunk that establishes the doc-tree separation. CH-20 status-flips reflect this: the concept-doc claims stay `silent-in-code` AT THE CONCEPT LAYER (concept doesn't speak about CREATE-vs-UPDATE; that's correct); the matrix-side and drift-side reflections flip to `accepted-as-is` AT THE CONVENTION LAYER. The 5 convention-docs are the new, lower-granularity, durable record.

---

## §3 — phi-core leverage map

**Doc-only chunk → zero phi-core surface change.** No phi-core types are imported, wrapped, duplicated, or rejected by CH-20's deliverables. The 5 convention-docs DOCUMENT the wrap pattern (D1.3, D3.3, D4.3, D4.6 in `wrap-pattern.md`) but introduce no new wrap site, no new import, no new duplication.

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Expected import-count delta at chunk close:** **+0** (zero — doc-only). Baseline at chunk-open: 57 (verified via canonical grep below). Predicted at chunk-close: 57 (Δ +0).

**Positive close-audit greps (canonical baseline preservation):**
```bash
# Canonical phi-core baseline (CH-15 retro Row 3; chunk-planner v8+ canonical form; NO trailing ::)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (unchanged from CH-19 close)
```
Verified at chunk-open 2026-05-10 + re-verified at v2 re-plan time → returned `57`.

**Forbidden-duplication greps:** N/A — no phi-core type duplicated by CH-20 (doc-only). The standing `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` guard MUST stay green at chunk-close (re-runs as part of §12 verification recipe).

**Cascade-artifact discipline (v3+):** N/A — no struct-cascade, no enum-additive cascade, no wire-mapping cascade. The discipline applies to code chunks; CH-20 is doc-only. **Doc-file cascade (F1.B-specific):** the chunk creates 5 NEW files at the same directory level; the cascade is the file-count itself (5 files at `v0/conventions/`), not a code-cascade. No grep-prediction needed — the file count is the F1.B lock outcome, fixed by the planner.

**Cross-cycle phi-core baseline anchor:** CH-18 closed at 57; CH-19 closed at 57 (Δ +0); CH-20 expected to close at 57 (Δ +0); the baseline carries forward into CH-21 (the next active M5.2 chunk) unchanged.

---

## §3.B — K8s microservice readiness check

**Doc-only chunk → all 7 axes `no impact`.** The chunk modifies/creates only markdown files under `docs/specs/v0/conventions/` (NEW; 5 files), `docs/specs/v0/implementation/m5_1/drifts/`, `docs/specs/v0/implementation/m5_2/decisions/`, and `docs/specs/plan/build/`. No Rust code changes. No new in-process state, IPC, pod-local resources, migrations, trait shapes, cross-pod state, or audit emitters.

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`, `RwLock`, etc.) | none — doc-only | **no** | n/a |
| **A2** | New IPC channel (`mpsc`, `broadcast`, etc.) | none — doc-only | **no** | n/a |
| **A3** | New pod-local resource | none — doc-only | **no** | n/a |
| **A4** | Migration runner / first-apply race | none — no migration | **no** | n/a |
| **A5** | Trait-shape requirement | none — `Repository` trait stable; D4.5 typed-writer convention is DOCUMENTED in `event-bus-wiring.md`, not extended | **no** | n/a |
| **A6** | Cross-pod state sharing | none — doc-only | **no** | n/a |
| **A7** | Audit hash-chain symmetry — D6.4 ratifies the platform-tier audit-event placement convention from CH-19 §D57.1 (system-agents page-13 events flow through the same single-writer `AuditEmitter` chain). CH-20's `event-bus-wiring.md` §"Audit-event placement cross-ref" subsection documents the convention-extension; it does NOT introduce a new audit writer. | doc-only ratification of an existing convention | **no** | n/a — convention preserves single-writer guarantee per ADR-0033 §D33.4 + CH-19 §D57.1 |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — N/A (chunk does not touch the registry).
- D33.2 (`SurrealStore::open_remote`) — N/A (chunk does not add storage operations).
- D33.3 (SIGTERM graceful shutdown) — N/A (chunk does not add `tokio::spawn` tasks).
- D33.4 (`EventBus.shutdown` + `drain`) — N/A (chunk does not add EventBus emitters/listeners; D3.2 + D6.4 ratification documents existing conventions that already comply).

**Conclusion paragraph.** **K8s-neutral.** All 7 axes evaluate `no impact`; no new ledger entry required at `m7b/architecture/deferred-from-ch-k8s-prep.md`. ADR-0058 §D58.5's ratification of the audit-event-placement convention extension (D6.4 → CH-19 §D57.1) is structurally K8s-positive (it documents that operators reading the convention-docs understand WHY the platform-tier audit emitters all flow through the single AuditEmitter chain — preserving A7 single-writer symmetry). Same posture as CH-19. **F1.B file-count change does not affect K8s posture** — the directory's file count is invisible to the runtime.

---

## §3.C — User-facing documentation impact map

**Tier evaluation per Q9 (CH-22 codification).** CH-20 is doc-only; the affected docs are predominantly **governance-tier** (ADR-0058 + 16 drift files + concept-audit matrix + drifts/README.md + cycle-index) PLUS one new document tier: **`v0/conventions/`** — neither concept-tier nor implementation-milestone-tier. The 5 NEW convention-docs are **internal-engineering-facing** (reviewers, future planners, future implementers) rather than operator-facing. User-facing-tier impact is **MINIMAL** — Bucket C drifts are by definition silent-in-code conventions invisible to operators.

**Doc-tree placement rationale for `v0/conventions/`:** the new directory is a peer of `v0/concepts/` (semantic source-of-truth) NOT a child of `m<N>/architecture/` (implementation-milestone tier). Rationale: conventions span multiple milestones (D1.x ships at M5/P1; D3.1 ships at M5/P3; D7.6 ships at M5/P7) — they are NOT milestone-bounded. They live at the workspace level alongside `v0/concepts/` to signal "long-lived reviewer guidance, not implementation-milestone scope." The peer tier shape mirrors `v0/concepts/`'s flat-files (no per-area subdirectories at this scale) — one file per thematic convention area.

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/<milestone>/architecture/<feature>.md` | **none directly.** ADR-0058 itself functions as the architecture-tier record for the 5 thematic conventions (precedent: ADR-0042 for CH-03's storage-backend ratification; ADR-0057 for CH-19's Bucket B ratification). 5 convention-docs at `v0/conventions/` are a NEW peer tier. | (a) update in-chunk: **none** — ADR-0058 is the architecture-tier surface; convention-docs are the new peer tier |
| **Operations** | `docs/specs/v0/implementation/<milestone>/operations/<feature>-operations.md` | **none directly.** D6.4 audit-event placement convention is internal-engineering-only (operators see the events via `audit_events` table; placement is invisible to them). D7.3 `phi session preview` flag is already documented via the test pin `cli::tests::completion_help::completion_session_subcommand_includes_preview` (per drift body line 31). D7.6 web pattern is internal frontend-engineering-only. | (a) update in-chunk: **none** |
| **User-guide** | `docs/specs/v0/implementation/<milestone>/user-guide/{<feature>-walkthrough,cli-reference-mN,troubleshooting}.md` | **`m5/user-guide/cli-reference-m5.md` MAY contain a 1-line cross-ref to ADR-0058 §D58.6** (D7.3 `preview` subcommand ratification). **Amend-don't-add precedence (CH-17 retro Row 3):** verify at draft-time whether `m5/user-guide/cli-reference-m5.md` covers `phi session` subcommands; if so, add the cross-ref as a "CH-20 amendment — `phi session preview` ratification (2026-05-10)" subsection rather than fragmenting users across a new doc. **Verified at chunk-open**: `m5/user-guide/cli-reference-m5.md` is a stub (per CH-19 plan §3.C row 3 + the same M5/P9 deferral block) — no live operator content. CH-20 does not amend stubs; the cross-ref lands when the stub is filled at M5-tag-close. **Successor: M5-tag-close (matches CH-19 successor for the same stub).** | (b) defer: **deferred to M5-tag-close** with successor `M5-tag-close batch` |
| **NEW peer tier — `v0/conventions/`** | `docs/specs/v0/conventions/<topic>.md` | **CREATES the directory** AND ships **5 NEW files**: (1) `persistence.md`, (2) `wrap-pattern.md`, (3) `event-bus-wiring.md`, (4) `cli-patterns.md`, (5) `web-patterns.md`. New tier; NOT a defer target. | (a) ship in-chunk: 5 separate files per F1.B user-lock |

**Per-file content map (5 NEW files):**

| File | Drifts covered | ADR sub-decisions | Subsection count | Word-count target |
|---|---|---|---|---|
| `persistence.md` | D1.1, D1.2, D2.1, D2.2, D4.4 (5 drifts) | §D58.1 + §D58.3 | 2 §-sections × 2-3 subsections (5 subsections total: schema-mechanics-1, schema-mechanics-2, write-verb-1, write-verb-2, write-verb-3) | ≤ 250 words |
| `wrap-pattern.md` | D1.3, D3.3, D4.3, D4.6 (4 drifts) | §D58.2 (consolidated) | 4 subsections (nested-inner / interior-mut / wire-shape / dual-mode) | ≤ 250 words |
| `event-bus-wiring.md` | D3.1, D3.2, D3.4, D4.5, D6.4 (5 drifts) | §D58.4 + §D58.5 (cross-ref) | 5 subsections (rename / seam / scoped-trait / typed-writer / audit-event-cross-ref) | ≤ 250 words |
| `cli-patterns.md` | D7.3 (1 drift) | §D58.6 | 1 subsection (CLI scope-addition discipline) | ≤ 100 words |
| `web-patterns.md` | D7.6 (1 drift) | §D58.7 | 1 subsection (Next.js 14 server-action shape) | ≤ 100 words |

**Total prose volume:** ≤ ~950 words across 5 files (comparable to v1's ~600–900 words single file). Each file carries a verified-header line 1, 1-2 paragraph preamble citing ADR-0058 §D58.N, then the subsection bodies.

**Rule compliance:** every doc the chunk's deliverables make stale is listed; defer-decisions cite a bounded successor (`M5-tag-close`); no open-ended deferrals; the new `v0/conventions/` peer tier is in-scope and deliverable; **5 separate files** per F1.B user-lock at gate-1.

---

## §3.D — Forward-scope-vs-concept-doc precedence

**Pre-flight check (v9+ MANDATORY procedure).** The forward-scope row at line 185-187 enumerates 16 drift IDs but does NOT introduce any new closed-set vocabulary (action verbs, fundamental kinds, audit-event class tiers, migration order, schema field names). Bucket C is by definition convention/pattern ratification of EXISTING shipped code; the chunk does not extend any closed set.

**Mechanical procedure executed:**

1. **Action verbs:** N/A — no new action verb. ADR-0058 sub-decisions §D58.1–§D58.10 do not touch `Action::CANONICAL`. The closed-set invariant `Action::CANONICAL.len() == 34` (CH-04 / ADR-0043; verified verbatim at `domain/src/permissions/action.rs:282`) is preserved unchanged. **Verified pre-flight grep**: no action-verb mentions in any of the 16 drift files (`grep -lE 'Action::|action_vocabulary|session\.start|session\.tool_invoke|session\.read_memory' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D{1.1,1.2,1.3,2.1,2.2,3.1,3.2,3.3,3.4,4.3,4.4,4.5,4.6,6.4,7.3,7.6}.md` returned 0 hits).
2. **Fundamental kinds:** N/A — no new fundamental kind.
3. **Selector grammar predicates:** N/A — no new predicate.
4. **Audit-event class tiers:** N/A — D6.4 ratifies an EXISTING placement convention (cross-ref CH-19 §D57.1); no new event class.
5. **Migration order:** N/A — no migration. D1.1 + D1.2 reference EXISTING migrations 0001 + 0005; no new migration slot consumed.
6. **Schema field names:** N/A — no schema change. D3.1's `agent_kind` rename SHIPPED at M5/P3 close; CH-20 ratifies, does not introduce.

**Pre-flight grep precision for "definitionally redundant" claims (v10):** N/A — CH-20 makes no such claim. The convention-docs + ADR-0058 sub-decisions document conventions, not redundancies.

**Verdict:** `forward-scope row literal text` matches `concept-doc canonical phrasing` for all 16 drifts (concept docs are silent on these mechanics; forward-scope is consistent with that silence). **NO contradiction; NO CRITICAL fork required; auto-approval criteria not blocked by §3.D.** The cross-cycle pattern-watch (CH-15 + CH-17 both required iter-2 re-spawn from incorrect closed-set claims; CH-19 cleared) does NOT apply here — CH-20 makes no closed-set contradiction claim. **F1.B file-count change does not affect §3.D** — closed-set vocabulary is independent of doc-file granularity.

---

## §4 — Drifts closed

**Grouping note.** The 16 drifts cluster into **5 thematic conventions** for ADR-0058's sub-decisions; the 5 convention-doc files mirror the same 5 thematic areas (one file per area, except `event-bus-wiring.md` which folds D6.4 audit-event-placement as a cross-ref subsection). The §4 table below lists all 16 individually with their thematic group + ADR sub-decision mapping + convention-doc file mapping.

| Drift ID | File | Severity | Bucket | Theme | Convention-doc file | Transition | ADR sub-decision | Notes |
|---|---|---|---|---|---|---|---|---|
| D1.1 | [`m5_1/drifts/D1.1.md`](../../../v0/implementation/m5_1/drifts/D1.1.md) | LOW | C | Persistence — schema mechanics | `persistence.md` §"Schema mechanics" subsection 1 | discovered → accepted-as-is | §D58.1 | DEFINE FIELD on pre-existing 0001 scaffolds (not fresh DEFINE TABLE). Migration 0005 layers governance columns on `session` + `turn` scaffolds. M7b cleanup may drop the zombie `loop` table. |
| D1.2 | [`m5_1/drifts/D1.2.md`](../../../v0/implementation/m5_1/drifts/D1.2.md) | LOW | C | Persistence — schema mechanics | `persistence.md` §"Schema mechanics" subsection 2 | discovered → accepted-as-is | §D58.1 | `runs_session` REMOVE+DEFINE retype (M1-era reverse direction `agent → session` retyped to `session → project`). No-writer-scaffold safe-retype pattern. |
| D1.3 | [`m5_1/drifts/D1.3.md`](../../../v0/implementation/m5_1/drifts/D1.3.md) | LOW | C | Wrap pattern | `wrap-pattern.md` §"Nested-inner form" | discovered → accepted-as-is | §D58.2 | Session/LoopRecordNode/TurnNode wraps use plain nested `inner` field (NOT `#[serde(flatten)]`) — phi-core's `Session.session_id` would collide on flatten. Cross-ref [ADR-0029](../../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) §D29.1. |
| D2.1 | [`m5_1/drifts/D2.1.md`](../../../v0/implementation/m5_1/drifts/D2.1.md) | LOW | C | Persistence — write verbs | `persistence.md` §"Write verbs" subsection 1 | discovered → accepted-as-is | §D58.3 | `persist_session` / `append_loop_record` / `append_turn` use CREATE (UPDATE-as-upsert is silent no-op on non-existent SCHEMAFULL+FLEXIBLE rows). Mapping SurrealDB unique-violation → `RepositoryError::Conflict`. |
| D2.2 | [`m5_1/drifts/D2.2.md`](../../../v0/implementation/m5_1/drifts/D2.2.md) | LOW | C | Persistence — write verbs | `persistence.md` §"Write verbs" subsection 2 | discovered → accepted-as-is | §D58.3 | RELATE statement requires LET-first binding (`LET $f = type::thing(...); RELATE $f -> edge -> $t`); SurrealDB parser rejects inline `type::thing(...)` in FROM/TO slots. |
| D3.1 | [`m5_1/drifts/D3.1.md`](../../../v0/implementation/m5_1/drifts/D3.1.md) | LOW | C | Event-bus + listener wiring | `event-bus-wiring.md` §"Serde-tag-collision rename" | discovered → accepted-as-is | §D58.4 | `DomainEvent::AgentCreated` field renamed `kind` → `agent_kind` to avoid serde-tag-discriminator collision with `#[serde(tag = "kind")]` enum-level attr. |
| D3.2 | [`m5_1/drifts/D3.2.md`](../../../v0/implementation/m5_1/drifts/D3.2.md) | LOW | C | Event-bus + listener wiring | `event-bus-wiring.md` §"Free-function listener seam" | discovered → accepted-as-is | §D58.4 | Listener wiring at free function `state::build_event_bus_with_m5_listeners` (NOT `AppState::new` constructor). `handler_count_is_five_at_m5` test asserts the count. |
| D3.3 | [`m5_1/drifts/D3.3.md`](../../../v0/implementation/m5_1/drifts/D3.3.md) | LOW | C | Wrap pattern | `wrap-pattern.md` §"Interior-mutability wrap" | discovered → accepted-as-is | §D58.2 | `BabyPhiSessionRecorder.inner: Arc<Mutex<PhiCoreSessionRecorder>>` — `&mut self` phi-core API requires interior mutability for shared `AppState::session_registry`. Cross-ref [ADR-0029](../../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) §D29.2. |
| D3.4 | [`m5_1/drifts/D3.4.md`](../../../v0/implementation/m5_1/drifts/D3.4.md) | LOW | C | Event-bus + listener wiring | `event-bus-wiring.md` §"Trait-split-by-scope for resolvers" | discovered → accepted-as-is | §D58.4 | Template C resolver org-scoped (`OrgId → AuthRequestId`); Template D project-scoped (`ProjectId → (OrgId, AuthRequestId)`) — concept-mandated asymmetry (`MANAGES` at org / `HAS_AGENT_SUPERVISOR` at project). |
| D4.3 | [`m5_1/drifts/D4.3.md`](../../../v0/implementation/m5_1/drifts/D4.3.md) | LOW | C (drift-file says B; per forward-scope row 186 treated as C) | Wrap pattern | `wrap-pattern.md` §"HTTP wire-shape projection" | discovered → accepted-as-is | §D58.2 | `resolve_agent_tools` returns `Vec<ToolSummary>` (HTTP wire shape) NOT `Vec<Box<dyn AgentTool>>` (phi-core trait-object). `_is_phi_core_agent_tool_trait<T: AgentTool + ?Sized>` compile-time witness preserved. **Bucket reconciliation note**: drift file frontmatter classifies D4.3 as Bucket B; forward-scope row 186 enumerates D4.3 in CH-20's Bucket-C list; the chunk honors the forward-scope assignment and the drift file's Bucket field stays B (matches CH-19 precedent for D-new-27 + D-new-30). |
| D4.4 | [`m5_1/drifts/D4.4.md`](../../../v0/implementation/m5_1/drifts/D4.4.md) | LOW | C | Persistence — write verbs | `persistence.md` §"Write verbs" subsection 3 | discovered → accepted-as-is | §D58.3 | `update.rs` branches on `current_profile.is_some()` → `upsert_agent_profile` when prior row exists, `create_agent_profile` otherwise. Same root-cause as D2.1. |
| D4.5 | [`m5_1/drifts/D4.5.md`](../../../v0/implementation/m5_1/drifts/D4.5.md) | LOW | C | Event-bus + listener wiring | `event-bus-wiring.md` §"Typed-writer-per-edge-type" | discovered → accepted-as-is | §D58.4 | Repository trait method `write_uses_model_edge` (typed-per-edge-type) NOT generic `create_edge(&Edge)`. Default trait body returns `Backend(...)` so future impls fail fast. |
| D4.6 | [`m5_1/drifts/D4.6.md`](../../../v0/implementation/m5_1/drifts/D4.6.md) | LOW | C | Wrap pattern | `wrap-pattern.md` §"Dual-mode discriminator" | discovered → accepted-as-is | §D58.2 | `SessionLaunchContext.first_loop_id: Option<LoopId>` — dual-mode discriminator. `Some(...)` for launch-chain pre-persisted path; `None` for standalone-test full-persist path. |
| D6.4 | [`m5_1/drifts/D6.4.md`](../../../v0/implementation/m5_1/drifts/D6.4.md) | LOW | C | Audit-event placement (cross-ref) | `event-bus-wiring.md` §"Audit-event placement cross-ref" | discovered → accepted-as-is | §D58.5 | System-agent page-13 audit events at `server::platform::system_agents::audit_events`. **Cross-ref CH-19 / ADR-0057 §D57.1** rather than re-ratifying the convention; CH-20 records this as a "convention extends to system-agents page-13" ratification. Folded into `event-bus-wiring.md` (single-sentence cross-ref doesn't merit a standalone file). |
| D7.3 | [`m5_1/drifts/D7.3.md`](../../../v0/implementation/m5_1/drifts/D7.3.md) | LOW | C | CLI scope-addition discipline | `cli-patterns.md` §"CLI scope-addition discipline" | discovered → accepted-as-is | §D58.6 | `phi session preview` ships as a 5th subcommand alongside `launch`/`show`/`terminate`/`list`. Wraps existing `POST /sessions/preview` route. Pinned by `completion_session_subcommand_includes_preview` regression. |
| D7.6 | [`m5_1/drifts/D7.6.md`](../../../v0/implementation/m5_1/drifts/D7.6.md) | LOW | C | Web pattern | `web-patterns.md` §"Next.js server-action shape" | discovered → accepted-as-is | §D58.7 | Hybrid: top-level `actions.ts` (named exports) + inline per-row `<form action={run}>` with `"use server"` at function body. Closure captures string-only (orgId, kind) — serialization-safe per Next.js 14 dynamic-id pattern. |

**Drift-file bucket reconciliation:** D4.3's drift-file frontmatter classifies it as Bucket B; forward-scope row 186 enumerates it in CH-20's Bucket-C list. **Forward-scope assignment is binding** (CH-19 precedent for D-new-27 + D-new-30 same shape). Drift-file `Bucket` field stays B; the lifecycle-history entry notes the CH-20 chunk-claim regardless of bucket.

**Drift transition discipline (per `drift-lifecycle.md:118-133`):** `accepted-as-is` requires (a) Accepted ADR documenting the drift ID, (b) explicit risk-acceptance statement, (c) review trigger. ADR-0058 covers (a) for all 16 via §D58.1–§D58.7 (one sub-decision per theme; multiple drifts per sub-decision). Risk-acceptance statement appears once in ADR §"Risk acceptance" (consolidated; 16 drifts share Bucket-C characteristics — shipped convention works, cost of code-side remediation outweighs benefit at M5 close). Review triggers are per-theme (Persistence — schema mechanics → M7b cleanup migration if pursued; Wrap pattern → M6+ phi-core API stabilisation; Event-bus + listener wiring → none near-term; Audit-event placement → cross-ref CH-19 §D57.1's review trigger; CLI scope-addition → CLI completion-help test continues to pin; Web pattern → M7+ Next.js 15 migration if pursued).

---

## §5 — ADRs drafted

**ADR number assignment (Q6 procedure executed at draft-time):** Ran `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE "ADR-[0-9]+" | sort -u | tail -5` (functional equivalent: `ls m5_2/decisions/ | tail`) → returned `0057-bucket-b-convention-ratification.md` as highest. **Next-free: ADR-0058.** Home: `m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md`.

**ADR-0058 — Bucket C convention confirm-in-place (5 thematic conventions covering 16 drifts: Persistence (schema mechanics + write verbs) / Wrap pattern / Event-bus + listener wiring (incl. audit-event-placement cross-ref) / CLI scope-addition discipline / Web pattern). Plus 3 META decisions: the new `v0/conventions/` directory introduction (§D58.8) + the **5-file split shape** under it (§D58.9) + the forward-scope row "(14 items)" off-by-2 note (§D58.10).**

- **Status at chunk-plan draft:** Proposed.
- **Flip to Accepted:** at chunk-seal (P3 deliverable per §7 below). Single ADR; ten sub-decisions (§D58.1–§D58.10) all flip together.
- **Drafted-at-phase:** P1 (ADR body + sub-decisions written + 5 convention-docs shipped); flipped to Accepted at P3 (chunk-seal).
- **Decision-summary (one line):** Confirm-in-place 16 Bucket-C shape-choice conventions shipped through M5/P1–P7 close; introduce the `v0/conventions/` peer tier with **5 separate convention-docs (one per thematic area per F1.B user-lock)** as the first canonical convention-doc set.
- **Closes:** D1.1 + D1.2 (§D58.1), D1.3 + D3.3 + D4.3 + D4.6 (§D58.2), D2.1 + D2.2 + D4.4 (§D58.3), D3.1 + D3.2 + D3.4 + D4.5 (§D58.4), D6.4 (§D58.5), D7.3 (§D58.6), D7.6 (§D58.7) — all 16 transition `discovered → accepted-as-is`.

**Sub-decision shape mapping (16 drifts → 7 thematic sub-decisions + 3 META decisions):**

| ADR §D | Theme | Drifts grouped | Convention-doc file (F1.B) | One-line decision |
|---|---|---|---|---|
| §D58.1 | Persistence — schema mechanics | D1.1, D1.2 | `persistence.md` §"Schema mechanics" | DEFINE FIELD on pre-existing 0001 scaffolds + REMOVE+DEFINE retype on no-writer-scaffold — never fresh DEFINE TABLE on a pre-existing table |
| §D58.2 | Wrap pattern — phi-core wraps | D1.3, D3.3, D4.3, D4.6 | `wrap-pattern.md` (entire file) | Plain nested `inner: phi_core::X` field (not `#[serde(flatten)]`); `Arc<Mutex<_>>` for shared interior-mut; HTTP wire-shape projection over trait-object; dual-mode discriminator for coordination |
| §D58.3 | Persistence — write verbs | D2.1, D2.2, D4.4 | `persistence.md` §"Write verbs" | CREATE-not-UPDATE-as-upsert + LET-first RELATE + branch-on-existence at handler-side for UPSERT-shaped writes |
| §D58.4 | Event-bus + listener wiring | D3.1, D3.2, D3.4, D4.5 | `event-bus-wiring.md` §§"Serde-tag-collision rename"/"Free-function listener seam"/"Trait-split-by-scope for resolvers"/"Typed-writer-per-edge-type" | Serde-tag-collision rename (`agent_kind`); free-function listener-wiring seam; trait-split-by-scope for resolvers; typed-writer-per-edge-type at trait surface |
| §D58.5 | Audit-event placement (cross-ref CH-19 §D57.1) | D6.4 | `event-bus-wiring.md` §"Audit-event placement cross-ref" | System-agents page-13 events follow the platform-tier convention from CH-19 §D57.1; convention extends, not re-ratifies |
| §D58.6 | CLI scope-addition discipline | D7.3 | `cli-patterns.md` (entire file) | `phi session preview` ships as a 5th subcommand wrapping existing HTTP route; CLI completion-help test pins the addition |
| §D58.7 | Web pattern — Next.js server actions | D7.6 | `web-patterns.md` (entire file) | Hybrid: top-level `actions.ts` exports + inline per-row `<form action={run}>` with `"use server"` directive (Next.js 14 dynamic-id pattern) |
| §D58.8 (META) | Doc-tree introduction — `v0/conventions/` peer tier | (none) | (no convention-doc — META decision) | The new `v0/conventions/` directory is a peer of `v0/concepts/` for cross-milestone reviewer guidance below concept granularity |
| §D58.9 (META) | First convention-doc set — **5-file split** per F1.B user-lock | (none) | (lists all 5 files as the META decision body) | 5 separate convention-docs (one per thematic area: persistence / wrap-pattern / event-bus-wiring / cli-patterns / web-patterns); ADR-0058 is governance-tier; convention-docs are reviewer-tier; granularity per F1.B (gate-1 user-lock 2026-05-10, divergent from planner v1 F1.A recommendation) |
| §D58.10 (META) | Forward-scope row "(14 items)" off-by-2 amendment | (none) | (no convention-doc — META decision) | The forward-scope row's parenthetical "(existing 14 items)" is empirically off-by-2 (16 drifts listed); CH-20 amends inline at chunk-seal P3 |

**ADR-body checklist (v2026-05-04 per CH-13 retrospective; v2026-05-08 per CH-14 retrospective Row 10; v2026-05-10 per CH-19 retrospective Row 1 — formula relaxed to 3 documented variations):**

1. **§"Forks" header with explicit user-lock outcome.** Single line: *"Forks: F1 LOCKED at gate-1 to F1.B (5-file split per `v0/conventions/`) — DIVERGENT from planner v1 F1.A (single consolidated file) recommendation, user-locked 2026-05-10. F2 + F3 resolved at planner-recommendation level via existing ADR-0042 + ADR-0057 precedent. Cross-cycle divergence pattern: 4 of last 6 cycles (CH-15/17/18/20)."*

2. **§"Cross-references" with all 4 categories.**
   - **(a) Originating concept-docs:** [`ontology.md` §"table-per-node-tier"](../../../v0/concepts/ontology.md), [`ontology.md` §"edges have a direction"](../../../v0/concepts/ontology.md), [`phi-core-mapping.md` §"wrap"](../../../v0/concepts/phi-core-mapping.md), [`agent.md` §"AgentProfile binds a blueprint"](../../../v0/concepts/agent.md), [`coordination.md` §"event-driven reactivity"](../../../v0/concepts/coordination.md), [`permissions/05-memory-sessions.md` §"session lifecycle"](../../../v0/concepts/permissions/05-memory-sessions.md), [`permissions/07-templates-and-tools.md` §"Template C/D"](../../../v0/concepts/permissions/07-templates-and-tools.md), [`permissions/04-manifest-and-resolution.md` §"preview mode"](../../../v0/concepts/permissions/04-manifest-and-resolution.md), [`system-agents.md` §"reconfiguration audit trail"](../../../v0/concepts/system-agents.md).
   - **(b) Closed drifts:** D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6 (16 IDs).
   - **(c) Prior ADRs cited as precedent (MILESTONE-PREFIXED per CH-08 retro Row 1):**
     - [`m5_2/decisions/0042-storage-backend-configurable.md`](../../../v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) (CH-03; doc-only-ratification-chunk shape precedent — first instance).
     - [`m5_2/decisions/0057-bucket-b-convention-ratification.md`](../../../v0/implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md) (CH-19; doc-only-ratification-chunk shape precedent — second instance; D6.4 explicitly cross-refs §D57.1).
     - [`m5/decisions/0029-session-persistence-and-recorder-wrap.md`](../../../v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) (M5; §D29.1 nested-not-flatten + §D29.2 `Arc<Mutex<_>>` already documented these conventions per-record-type; ADR-0058 §D58.2 generalises across all wrap sites).
     - [`m5_2/decisions/0033-k8s-prep-refactors.md`](../../../v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) (CH-K8S-PREP; §D33.4 single-AuditEmitter-writer guarantee that §D58.5's convention-extension preserves).
     - [`m4/decisions/0028-domain-event-bus.md`](../../../v0/implementation/m4/decisions/0028-domain-event-bus.md) (M4; Template-A fire-listener pattern that §D58.4's listener-wiring conventions extend).
   - **(d) Forward-scope row:** [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md` lines 185-187](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (CH-20 row); §5 severity table line 428.
   - **(e) Convention-doc URLs (NEW for F1.B; ADR cross-references all 5):**
     - [`docs/specs/v0/conventions/persistence.md`](../../../v0/conventions/persistence.md) — §D58.1 + §D58.3
     - [`docs/specs/v0/conventions/wrap-pattern.md`](../../../v0/conventions/wrap-pattern.md) — §D58.2
     - [`docs/specs/v0/conventions/event-bus-wiring.md`](../../../v0/conventions/event-bus-wiring.md) — §D58.4 + §D58.5
     - [`docs/specs/v0/conventions/cli-patterns.md`](../../../v0/conventions/cli-patterns.md) — §D58.6
     - [`docs/specs/v0/conventions/web-patterns.md`](../../../v0/conventions/web-patterns.md) — §D58.7

3. **Pre-existing-behaviour preservation note (CH-14 retro Row 10; v11 formula-relaxation per CH-19 retro Row 1).** ADR-0058 does NOT change runtime behaviour — every sub-decision documents conventions SHIPPED at M5/P1–P7 close. **CH-20 expects to use BOTH the strict formula AND the v11 (b) Multi-milestone-pattern variation + (c) Never-shipped-yet variation** because the conventions emerged across multiple M-tags and the META decisions cover never-shipped surfaces. Format applied per sub-decision:
   - **§D58.1 — Persistence / schema mechanics**: strict formula. *"Pre-existing implementation preserved at `modules/crates/store/migrations/0005_sessions_templates_system_agents.surql` (M5/P1 close, 2026-04-23); CH-20 ratifies without behaviour change."*
   - **§D58.2 — Wrap pattern**: v11 (b) Multi-milestone-pattern variation. *"Pre-existing implementation preserved: nested-`inner` wrap-pattern + `Arc<Mutex<_>>` interior-mut wrap (pattern emerged across M3 `OrganizationDefaultsSnapshot` + M5/P1 `Session/LoopRecordNode/TurnNode` + M5/P3 `BabyPhiSessionRecorder` + M5/P4 `SessionLaunchContext` tags; CH-20 ratifies the convention as canonical, does not change shipped code)."*
   - **§D58.3 — Persistence / write verbs**: strict formula. *"Pre-existing implementation preserved at `modules/crates/store/src/repo_impl.rs:3147` (`persist_session` CREATE) + `modules/crates/server/src/platform/agents/update.rs:381-384` (branch-on-existence) (shipped at M5/P2 + M5/P4 close, 2026-04-23); CH-20 ratifies without behaviour change."*
   - **§D58.4 — Event-bus + listener wiring**: v11 (b) Multi-milestone-pattern variation. *"Pre-existing implementation preserved: serde-collision-rename + free-function-listener-seam + scoped-resolver-trait-split + typed-writer-per-edge (pattern emerged across M5/P3 `events/mod.rs` + `state.rs` + M5/P4 `repository.rs` + `projects/resolvers.rs`; CH-20 ratifies the convention as canonical, does not change shipped code)."*
   - **§D58.5 — Audit-event placement (cross-ref)**: strict formula. *"Pre-existing implementation preserved at `modules/crates/server/src/platform/system_agents/audit_events.rs` (M5/P6 close, 2026-04-23); CH-20 ratifies without behaviour change. Cross-ref CH-19 / ADR-0057 §D57.1 — D6.4 IS the same convention as D5.2 applied to system-agents."*
   - **§D58.6 — CLI scope-addition discipline (D7.3)**: strict formula. *"Pre-existing implementation preserved at `modules/crates/cli/src/commands/session.rs:101` (M5/P7 close, 2026-04-24); CH-20 ratifies without behaviour change."*
   - **§D58.7 — Web pattern (D7.6)**: strict formula. *"Pre-existing implementation preserved at `modules/web/app/(admin)/organizations/[id]/templates/page.tsx:142,159,182,199` (M5/P7 close, 2026-04-24); CH-20 ratifies without behaviour change."*
   - **§D58.8 (META) — Doc-tree `v0/conventions/` introduction**: v11 (c) Never-shipped-yet variation. *"Pre-existing absence preserved: no `v0/conventions/` directory existed before CH-20 (verified `ls baby-phi/docs/specs/v0/conventions/ → No such file or directory` at chunk-open 2026-05-10 + at v2 re-plan time); CH-20 ratifies the directory introduction as canonical."*
   - **§D58.9 (META) — First convention-doc set (5-file split per F1.B)**: v11 (c) Never-shipped-yet variation. *"Pre-existing absence preserved: no convention-doc files existed before CH-20; F1.B user-lock at gate-1 (2026-05-10, divergent from planner v1 F1.A single-file recommendation) settles the granularity at 5 separate files (one per thematic area: persistence / wrap-pattern / event-bus-wiring / cli-patterns / web-patterns)."*
   - **§D58.10 (META) — Forward-scope row "(14 items)" off-by-2 amendment**: strict formula. *"Pre-existing wording preserved at `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md:185` (M5.1/P3 authoring, 2026-04-24); CH-20 amends `(existing 14 items)` → `(existing 16 items)` inline at chunk-seal P3 to match the empirical drift count."*

   **Spirit-of-rule check applied:** every sub-decision identifies (i) what was the case before this chunk, (ii) whether this chunk changes it, (iii) where the historical evidence lives. The v11 variations don't loosen the spirit — they accommodate sub-decision shapes that lack a single shipped-at date (multi-milestone pattern emergence + doc-tree introductions).

**ADR cross-reference precedent map (planner self-discipline):** ADR-0058 is the **third consolidated convention-ratification ADR** in baby-phi (after ADR-0042 for CH-03 storage-backend; ADR-0057 for CH-19 Bucket B). The shape is now a stable 3-cycle precedent for any future Bucket-B / Bucket-C / convention-ratification chunks. **F1.B's 5-file convention-doc shape is a NEW precedent** that CH-21+ can mirror for similar multi-thematic ratifications.

---

## §6 — Prior-chunk regression re-verification

Doc-only chunk → minimal upstream invariants. The 16 drifts' shipped conventions all live in code that prior chunks (M3 / M4 / M5/P1–P7 era) shipped. CH-20 verifies those conventions still hold at chunk-open AND at chunk-seal (per template §6 rule).

**Cascade-grep precision (v3+ chunk-planner discipline):** the table below uses absolute paths + canonical `git -C ... grep -nE` form per CH-08 retro Row 3 + CH-15 retro Row 3.

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| M5/P1 (migrations) | `0001_initial.surql:142` ships `DEFINE TABLE session SCHEMAFULL;` (pre-existing scaffold) | `grep -nE "DEFINE TABLE session SCHEMAFULL" /root/projects/phi/baby-phi/modules/crates/store/migrations/0001_initial.surql` (expect 1 hit at line 142) |
| M5/P1 (migrations) | `0001_initial.surql:327` ships reverse `runs_session` direction; `0005_sessions_templates_system_agents.surql:141` retypes to `session → project` | `grep -nE "DEFINE TABLE runs_session" /root/projects/phi/baby-phi/modules/crates/store/migrations/{0001_initial.surql,0005_sessions_templates_system_agents.surql}` (expect 2 hits: line 327 of 0001, line 141 of 0005) |
| M5/P1 (wrap) | Session/LoopRecordNode/TurnNode use plain nested `inner` (no `#[serde(flatten)]` on these structs) | `grep -n "serde(flatten)" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` (expect 0 hits on Session/LoopRecordNode/TurnNode struct definitions; existing match at `nodes.rs:1234` is on a DIFFERENT non-Session struct per CH-19's verification) |
| M5/P2 (repo write verbs) | `persist_session` uses `CREATE type::thing('session', $id) ...` | `grep -nE "CREATE type::thing\\(.session." /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs` (expect ≥ 1 hit; current HEAD: line 3147) |
| M5/P2 (RELATE) | `runs_session` LET-first RELATE in the session-persist tx | `grep -nE "RELATE \\\$f -> runs_session -> \\\$t" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs` (expect ≥ 1 hit; current HEAD: line 3167) |
| M5/P3 (event field rename) | `DomainEvent::AgentCreated` uses `agent_kind:` field | `grep -nE "agent_kind: AgentKind" /root/projects/phi/baby-phi/modules/crates/domain/src/events/mod.rs` (expect ≥ 1 hit; current HEAD: line 168) |
| M5/P3 (listener wiring) | `state::build_event_bus_with_m5_listeners` exists; test asserts handler count == 5 | `grep -nE "fn build_event_bus_with_m5_listeners\|handler_count_is_five_at_m5" /root/projects/phi/baby-phi/modules/crates/server/src/state.rs` (expect ≥ 2 hits; current HEAD: lines 278 + 481) |
| M5/P3 (recorder wrap) | `BabyPhiSessionRecorder.inner: Arc<Mutex<PhiCoreSessionRecorder>>` | `grep -nE "inner: Arc<Mutex<PhiCoreSessionRecorder>>" /root/projects/phi/baby-phi/modules/crates/domain/src/session_recorder.rs` (expect ≥ 1 hit; current HEAD: line 107) |
| M5/P3 (Template C/D resolvers) | `RepoTemplateCAdoptionArResolver` + `RepoTemplateDAdoptionArResolver` + their `AdoptionArResolver` impl(s) | `grep -nE "RepoTemplateCAdoptionArResolver\|RepoTemplateDAdoptionArResolver" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/resolvers.rs` (expect ≥ 4 hits; current HEAD: lines 23, 107, 111, 118, 140, 144, 151) |
| M5/P4 (tools) | `pub struct ToolSummary` + `resolve_agent_tools(...) -> Result<Vec<ToolSummary>, _>` | `grep -nE "pub struct ToolSummary\|fn resolve_agent_tools.*Vec<ToolSummary>" /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/tools.rs` (expect ≥ 2 hits; current HEAD: lines 37 + 52) |
| M5/P4 (profile branch) | `update.rs` branches on `current_profile.is_some()` for create-vs-upsert | `grep -nE "current_profile.is_some\(\)" /root/projects/phi/baby-phi/modules/crates/server/src/platform/agents/update.rs` (expect ≥ 1 hit; current HEAD: line 381) |
| M5/P4 (typed-writer trait) | Repository trait method `write_uses_model_edge` | `grep -nE "fn write_uses_model_edge" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs` (expect ≥ 1 hit; current HEAD: line 1713) |
| M5/P4 (recorder dual-mode) | `SessionLaunchContext.first_loop_id: Option<LoopId>` | `grep -nE "first_loop_id: Option<LoopId>\|pub first_loop_id" /root/projects/phi/baby-phi/modules/crates/domain/src/session_recorder.rs` (expect ≥ 1 hit; current HEAD: line 91) |
| M5/P6 (system-agents audit-events) | `server::platform::system_agents::audit_events` module exists with 4 builder fns | `grep -nE "platform.system_agent." /root/projects/phi/baby-phi/modules/crates/server/src/platform/system_agents/audit_events.rs` (expect ≥ 4 hits; current HEAD: lines 29, 67, 96, 123 + 4 test assertions) |
| M5/P7 (CLI preview) | `phi session preview` 5th subcommand exists | `grep -nE "SessionCommand::Preview\|fn preview_impl" /root/projects/phi/baby-phi/modules/crates/cli/src/commands/session.rs` (expect ≥ 2 hits; current HEAD: lines 150 + 538) |
| M5/P7 (web hybrid) | `<form action={...}>` inline `"use server"` directives at templates page | `grep -rnE "\"use server\"" "/root/projects/phi/baby-phi/modules/web/app/(admin)/organizations/[id]/templates/page.tsx"` (expect ≥ 3 hits; current HEAD: lines 142, 159, 182, 199) |

**Pre-CH-20 chunk-open verification (run at gate-1 before ExitPlanMode):** all greps above MUST return their expected counts. **Verified at chunk-open 2026-05-10 + re-verified at v2 re-plan time:** all 16 greps run + returned expected counts (citations in §2 + §4 above are anchored to CURRENT-HEAD line numbers, refreshed from drift-file-author-time as some line numbers had drifted: D2.1's `repo_impl.rs:2238` is now line 3147; D2.2's `2256-2259` is now line 3167; D3.1's `events/mod.rs:162-165` is now line 165-168; D3.2's `state.rs:93,143` is now line 278+481; D3.3's `session_recorder.rs:92` is now line 107; D4.5's `repository.rs:1135` is now line 1713; D4.6's `session_recorder.rs:76` is now line 91; D7.3's `session.rs:101,453` is now line 101+538; D7.6 unchanged at 142,159,182).

**Forward-scope drift count off-by-2 also verified at chunk-open:** the row line 185 reads "(existing 14 items)" but enumerates 16 IDs at line 186. CH-20 amends inline at P3 (per §D58.10).

---

## §7 — Phases within the chunk

Three phases: P1 (5 convention-docs + ADR-0058 + concept-doc verified-header bumps if any), P2 (drift-file lifecycle entries + concept-audit matrix refreshes + drifts/README.md), P3 (chunk-seal: ADR Accepted + cycle-index row + verified-header bumps + forward-scope row "(14 items)" amendment).

### P1 — Convention-docs creation (5 files) + ADR-0058 draft

**Goal.** Create the new `v0/conventions/` directory + ship 5 convention-docs (one per thematic area per F1.B user-lock). Author ADR-0058 with all 10 sub-decisions referencing the 5 files. Single coherent doc-write phase; no code touched.

**Deliverables (8 total per F1.B re-balanced).**
1. `mkdir -p /root/projects/phi/baby-phi/docs/specs/v0/conventions/` (creates the new peer-tier directory).
2. Write `/root/projects/phi/baby-phi/docs/specs/v0/conventions/persistence.md`. Header line 1 carries `<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1; first canonical convention-doc set; cycle hex 240616a4) -->` per `Documentation Alignment` discipline. Content:
   - **Preamble** (1-2 sentences) — what `v0/conventions/` is for; cross-ref ADR-0058 §D58.8 + §D58.9.
   - **§"Schema mechanics"** — 2 numbered subsections: (1) DEFINE FIELD on pre-existing 0001 scaffolds (D1.1); (2) REMOVE+DEFINE retype on no-writer-scaffold (D1.2). Each: rule + grep-for-regression + cross-ref drift IDs + cross-ref ADR-0058 §D58.1.
   - **§"Write verbs"** — 3 numbered subsections: (1) CREATE-not-UPDATE-as-upsert (D2.1); (2) LET-first RELATE (D2.2); (3) branch-on-existence for upsert-vs-create (D4.4). Each: rule + grep + cross-ref drift IDs + cross-ref ADR-0058 §D58.3.
   - **Word-count target**: ≤ 250 words.
3. Write `/root/projects/phi/baby-phi/docs/specs/v0/conventions/wrap-pattern.md`. Same header line 1. Content:
   - **Preamble** (1-2 sentences) — cross-ref ADR-0058 §D58.2 + cross-ref ADR-0029 §D29.1/§D29.2.
   - 4 numbered subsections: (1) Nested-inner form (NOT `#[serde(flatten)]`) — D1.3; (2) Interior-mutability wrap (`Arc<Mutex<phi_core::SessionRecorder>>`) — D3.3; (3) HTTP wire-shape projection (`Vec<ToolSummary>`) — D4.3; (4) Dual-mode discriminator (`Option<X>`) — D4.6. Each: rule + cross-ref drift IDs + cross-ref ADR-0029 + ADR-0058 §D58.2.
   - **Word-count target**: ≤ 250 words.
4. Write `/root/projects/phi/baby-phi/docs/specs/v0/conventions/event-bus-wiring.md`. Same header line 1. Content:
   - **Preamble** (1-2 sentences) — cross-ref ADR-0058 §D58.4 + ADR-0058 §D58.5 + cross-ref ADR-0028 (M4 event-bus) + ADR-0057 §D57.1 (CH-19 audit-event placement).
   - 5 numbered subsections: (1) Serde-tag-collision rename (`agent_kind`) — D3.1; (2) Free-function listener seam (`build_event_bus_with_m5_listeners`) — D3.2; (3) Trait-split-by-scope for resolvers (Template C org-scoped / Template D project-scoped) — D3.4; (4) Typed-writer-per-edge-type at Repository trait — D4.5; (5) Audit-event placement cross-ref (system-agents page-13 follows CH-19 §D57.1) — D6.4. Each: rule + cross-ref drift IDs + cross-ref ADR-0058 §D58.N.
   - **Word-count target**: ≤ 250 words.
5. Write `/root/projects/phi/baby-phi/docs/specs/v0/conventions/cli-patterns.md`. Same header line 1. Content:
   - **Preamble** (1-2 sentences) — cross-ref ADR-0058 §D58.6.
   - 1 subsection: §"CLI scope-addition discipline" — `phi session preview` 5th subcommand wraps existing HTTP route; CLI completion-help test pins (`completion_session_subcommand_includes_preview`). Rule: any new CLI subcommand wrapping an existing HTTP route MUST be pinned by a completion-help regression test. Cross-ref drift D7.3 + ADR-0058 §D58.6.
   - **Word-count target**: ≤ 100 words.
6. Write `/root/projects/phi/baby-phi/docs/specs/v0/conventions/web-patterns.md`. Same header line 1. Content:
   - **Preamble** (1-2 sentences) — cross-ref ADR-0058 §D58.7.
   - 1 subsection: §"Next.js server-action shape" — hybrid (top-level `actions.ts` named exports + inline per-row `<form action={run}>` with `"use server"` at function body). Closure-capture-must-be-string-only rule (Next.js 14 dynamic-id pattern). Cross-ref drift D7.6 + ADR-0058 §D58.7.
   - **Word-count target**: ≤ 100 words.
7. Write `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md`. Status: **Proposed** at this phase. Sections: top-matter (status / date / chunk / closes) + Context + 10 sub-decisions §D58.1–§D58.10 (per the table above) + Risk acceptance + Forks (F1 LOCKED at F1.B with divergence note) + Cross-references (categories a-e per §5 above) + Pre-existing-behaviour preservation per the v11-relaxed formula. Word-count target: ≤ 4500 words (slightly larger than ADR-0057's ~4000 due to 16 drifts + 3 META decisions + F1.B divergence note + 5 convention-doc URL cross-refs).
8. **(Optional)** Concept-doc verified-header bumps: per §2 the chunk does NOT modify concept-doc bodies (concept docs stay silent — convention-docs are the new home). However, the 8 concept docs in §2 carry verified-headers; planner reviews each at P1 to determine whether a CH-20 amendment line is warranted to point to a specific convention-doc file. **Default: NO bumps** (concept-doc body unchanged → no header bump per `Documentation Alignment` rule). **Planner-recommendation: zero header bumps** (concept docs stay silent at the semantic layer; `v0/conventions/` is the discovery surface).

**Tests.** None — no test changes.

**Concept-alignment check.** §2 row target-status verification deferred to P3 (matrix-side reflections happen in P2 per matrix-update timing convention).

**phi-core leverage check.** N/A — no phi-core changes.

**User-facing doc updates.** Per §3.C: 5 NEW convention-docs ship in P1 (not deferred) — IS the user-facing-tier deliverable for CH-20 (peer-tier audience: future planners + reviewers + implementers).

**Confidence target.** ≥ 99% (5 convention-docs + ADR + (optional) header bumps are pure prose; high precision achievable). Deliverable count denominator: 8 deliverables (1 mkdir + 5 convention-docs + 1 ADR-0058 + 0 header bumps); numerator at phase-close: 8.

**Pause discipline.** Pause via `AskUserQuestion` if (a) any single convention-doc word-count exceeds 1.5× its target (`persistence.md` > 375 words; `wrap-pattern.md` > 375 words; `event-bus-wiring.md` > 375 words; `cli-patterns.md` > 150 words; `web-patterns.md` > 150 words); (b) any of the 5 thematic sections surfaces a contradiction with EXISTING ADR-0029 / ADR-0042 / ADR-0057 wording NOT anticipated in §2 / §5 (would require multi-ADR-renegotiation, not confirm-in-place — escalates beyond Bucket C); (c) ADR-0058 sub-decision word count exceeds 500 words for any single sub-decision (would suggest the sub-decision is harboring non-Bucket-C scope — re-evaluate against drift-file body before continuing); (d) the per-file split surfaces a content-ambiguity where a drift could plausibly fit either of two files (e.g., D6.4 at boundary of `event-bus-wiring.md` vs a hypothetical `audit-event-placement.md`) — current plan folds D6.4 into `event-bus-wiring.md`; if at draft-time the cross-ref subsection feels mis-fit, escalate before continuing.

### P2 — Drift-file lifecycle entries + concept-audit matrix refreshes + drifts/README.md

**Goal.** Update every drift file's `Status` field + `Lifecycle history` block + `Last verified` header; refresh `_concept-audit-matrix.md` rows for any claim flipping status at chunk-close (note: most C-bucket drifts are below matrix granularity per CH-19/ADR-0057 precedent — most §2 rows roll up under existing matrix rows or have no exact matrix row); update `drifts/README.md` index Status column.

**Deliverables.**
1. Update `Status` field in 16 drift files: `discovered` → `accepted-as-is`. Update `Last verified: 2026-05-10 by Claude Code` header on each (with CH-20 amendment description summarising the convention-doc file + ADR sub-decision). **Per-drift file mapping for the lifecycle-entry text** (specifies which of the 5 convention-docs to cite):
   - D1.1 + D1.2 → cite `persistence.md` §"Schema mechanics"
   - D2.1 + D2.2 + D4.4 → cite `persistence.md` §"Write verbs"
   - D1.3 + D3.3 + D4.3 + D4.6 → cite `wrap-pattern.md` (entire file)
   - D3.1 + D3.2 + D3.4 + D4.5 → cite `event-bus-wiring.md` (specific subsection per drift)
   - D6.4 → cite `event-bus-wiring.md` §"Audit-event placement cross-ref"
   - D7.3 → cite `cli-patterns.md`
   - D7.6 → cite `web-patterns.md`
2. Append lifecycle-history entry to each drift file body:
   ```
   - 2026-05-10 — `accepted-as-is` — ratified via CH-20 / ADR-0058 §D58.<N> + convention-doc `v0/conventions/<file>.md` §"<subsection>"; review trigger: <per-theme trigger per §4>.
   ```
   Per-theme triggers (16 entries): D1.1 + D1.2 → "M7b zombie-table cleanup if pursued"; D1.3 + D3.3 + D4.3 + D4.6 → "M6+ phi-core API stabilisation if pursued"; D2.1 + D2.2 + D4.4 → "M6+ SurrealDB UPSERT-keyword refactor if pursued"; D3.1 + D3.2 + D3.4 + D4.5 → "no near-term"; D6.4 → "cross-ref CH-19 §D57.1 review trigger (no near-term)"; D7.3 → "CLI completion-help test continues to pin"; D7.6 → "M7+ Next.js 15 migration if pursued".
3. Update `_concept-audit-matrix.md`. **Most CH-20 drifts have no exact matrix row** (CH-19 precedent for D5.2 / D7.5 / D7.4 etc.); the convention is below matrix granularity. Specific matrix-side actions:
   - **NO new matrix rows added.**
   - **D1.3 / D3.3 (wrap pattern):** the existing `phi-core-mapping.md` row "Session wrap" at line 100 — Code-evidence cell extended to cite ADR-0058 §D58.2 + cross-ref `wrap-pattern.md` + cross-ref ADR-0029 §D29.1/§D29.2; Covering-drift cell extended from `—` to `D1.3 ✓ + D3.3 ✓` (per CH-07-style label-coverage rollup precedent).
   - **D4.3 (HTTP wire-shape projection):** the existing `permissions/07` row "14 Tool Authority Manifest examples" at line 228 — D4.3 portion stays partially-honored (this matrix row is about TAM examples, not the wire-shape projection); CH-20 does NOT flip this row. Instead, ADR-0058 §D58.2 + `wrap-pattern.md` §"HTTP wire-shape projection" record the ratification; matrix-side rollup is the "Session wrap" row's Covering-drift extension.
   - **D6.4 (audit-event placement):** no exact matrix row; ratification recorded in ADR-0058 §D58.5 + the D6.4 drift file lifecycle + `event-bus-wiring.md` §"Audit-event placement cross-ref". (Mirrors CH-19's D5.2 handling.)
   - **D1.1 / D1.2 / D2.1 / D2.2 / D3.1 / D3.2 / D3.4 / D4.4 / D4.5 / D4.6 / D7.3 / D7.6:** all below matrix granularity. No matrix row exists; ratification recorded in convention-doc + ADR-0058 sub-decision + drift file lifecycle.
   - **`_concept-audit-matrix.md` verified-header bump:** prepend a new CH-20 amendment line at the head of the file describing the Code-evidence cell extension on the "Session wrap" row + the absence of new matrix rows for the other 15 drifts (per CH-12 retro Row 1 P4 paperwork addendum).
4. Update `drifts/README.md` index Status column to `accepted-as-is ✓ (CH-20 / ADR-0058 §D58.<N> / conventions/<file>.md)` for the 16 drift rows. Prepend a CH-20 amendment header line.

**Tests.** None.

**Concept-alignment check.** Every §2 row's matrix-side reflection updated per (3) above; the dominant pattern is "no row touched" (15/16 drifts) which matches the C-bucket below-granularity convention.

**phi-core leverage check.** N/A.

**User-facing doc updates.** None (per §3.C — peer-tier convention-docs shipped in P1; user-facing tiers deferred to M5-tag-close).

**Confidence target.** ≥ 99%.

**Pause discipline.** Pause if any drift file's existing lifecycle-history is ambiguous about prior state. Pause if `_concept-audit-matrix.md` row "Session wrap" can't be located (would mean the matrix has shifted shape since CH-19 close; refresh §2 + §4 if so).

### P3 — Chunk-seal: ADR Accepted + cycle-index row + verified-header bumps + forward-scope amendment

**Goal.** Flip ADR-0058 from Proposed → Accepted; insert cycle-index row at `_cycle-index.md`; bump verified-headers on touched docs; amend the forward-scope row `(14 items)` → `(16 items)` inline; final P4 paperwork checklist.

**Deliverables.**
1. Flip ADR-0058 `Status: Proposed` → `Status: Accepted` in `m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md`.
2. Insert row in [`_cycle-index.md`](../../_cycle-index.md) "Active cycles" table per CH-17 retro Row 4 paperwork rule:
   ```
   | [`240616a4`](ch-20-bucket-c-confirm-in-place-240616a4/plan.md) | CH-20 — Bucket C confirm-in-place (closes 16 drifts D1.1/D1.2/D1.3/D2.1/D2.2/D3.1/D3.2/D3.3/D3.4/D4.3/D4.4/D4.5/D4.6/D6.4/D7.3/D7.6; 5 convention-docs at v0/conventions/) | 3 | 2 (audit envelope: medium per plan §11) | <iter count from gate-3> | <status> | <retro link> |
   ```
3. Amend forward-scope row line 185 `(existing 14 items)` → `(existing 16 items)` per §D58.10 inline. Bump the forward-scope verified-header trailer with a CH-20 amendment note. **This is a 1-line content edit** — categorised as Trivial-1L per the audit-fix-loop rule (no auditor re-spawn needed if caught at gate-2).
4. Bump `<!-- Last verified: 2026-05-10 by Claude Code (CH-20 amendment — ADR-0058 + 5 convention-docs ratify 16 Bucket-C conventions) -->` header on the 5 convention-docs + ADR-0058 + the 16 drift files + `_concept-audit-matrix.md` + `drifts/README.md` + the forward-scope file.
5. Run §12 verification recipe end-to-end → confirm 4 CI guards green; confirm `cargo test --workspace` test count UNCHANGED (1529 — same as CH-19 close); confirm `clippy --all-targets` green; confirm phi-core baseline 57.
6. Append CH-20 chunk-seal lifecycle entry to ADR file (Status block reflects Accepted; date stamped 2026-05-10).

**Tests.** Run full workspace test as gate-4 sanity check (one-shot at chunk-seal; doc-only chunk does NOT need per-phase test runs). Expected: **1529** (same as CH-19 close per `_cycle-index.md` row "1529/0/2"; doc-only chunk → Δ +0). Run `cargo clean` immediately after the test per CH-18 retro Row 1 / chunk-implementer v8 immediate-post-test cleanup discipline (placement-1).

**Concept-alignment check.** Re-walk §2 table. All 16 rows confirmed at chunk-close target status `accepted-as-is`.

**phi-core leverage check.** Re-run canonical baseline grep; expect 57; verify Δ +0.

**User-facing doc updates.** None additional in P3 (5 convention-docs shipped in P1).

**Confidence target.** ≥ 99% composite.

**Pause discipline.** Pause if (a) `cargo test --workspace` test count diverges from 1529; (b) clippy/fmt fail; (c) `check-doc-links.sh` fails (most likely failure mode for a doc-heavy chunk that introduces a new directory + 5 new files + cross-refs across them — F1.B's 5-file shape increases cross-link surface vs F1.A's 1-file shape; resolve all link hits before sealing).

---

## §8 — Tests summary

**Expected total test count at chunk close:** **1529** (same as CH-19 chunk-close; doc-only chunk → Δ +0). **F1.B file-count change does not affect test count** — 5 doc files vs 1 doc file is still 0 test changes.

**Plan §8 v2 band: [1529, 1529].** Asymmetric band (CH-12 retro discipline) is degenerate here — no new tests in scope, so deliverable-listed sum × 1.0 = lower bound = 1529; deliverable-listed sum × 1.20 = upper bound = 1529 (rounding to 1529 since 0 new tests means buffer applies to 0). **Outside the band → AskUserQuestion** per template §8 rule.

**MUST-SHIP / MAY-COVER split (CH-17 retro Row 6):**
- **MUST-SHIP:** none. Doc-only chunk; no new test files in scope.
- **MAY-COVER:** none. Doc-only chunk; no band-floor surrogates in scope.

**Layer breakdown:**
- Unit: 0 added.
- Integration: 0 added.
- Acceptance: 0 added.
- e2e: 0 added.

**Named test files:** none added.

**Named expected-still-green tests** (carry-forward verification of conventions documented):
- `domain::events::mod::tests::sample_agent_created` + `event_id_accessor_matches_emitted_value_for_every_variant` (per D3.1 drift body line 31) — pin the `agent_kind:` field rename; CH-20 ratifies the convention; tests continue to enforce.
- `server::state::tests::handler_count_is_five_at_m5` (per D3.2 drift body line 31) — pins the free-function listener-wiring seam handler count; CH-20 ratifies.
- `domain::session_recorder` tests covering `Arc<Mutex<_>>` interior-mut wrap (3 lifecycle tests in `domain/tests/session_recorder_wrap_test.rs` per D3.3 drift body line 31).
- `server::platform::sessions::tools::tests::resolve_agent_tools_returns_empty_list_at_m5` (per D4.3 drift body line 31) — pins the `Vec<ToolSummary>` wire-shape; CH-20 ratifies.
- `cli::tests::completion_help::completion_session_subcommand_includes_preview` (per D7.3 drift body line 32) — pins `phi session preview` 5th subcommand existence; CH-20 ratifies.
- `acceptance_sessions_m5p4` launch + recorder-coordination scenarios (per D4.6 drift body line 31) — exercise the `first_loop_id: Option<LoopId>` dual-mode path.
- `npm run build` + `npm run typecheck` + `npm run lint` (per D7.6 drift body line 31) — green; the `"use server"` directives pinned via TypeScript + Next.js lint rules.

**Buffer ceiling:** N/A — doc-only chunk does not have an Artifact-C-cascade test-amendment scope (CH-17 retro Row 5 ×1.30 ceiling does not apply).

---

## §9 — Pre-chunk gate

**Reading list (mandatory) — verified COMPLETE at chunk-open 2026-05-10 by planner; re-confirmed at v2 re-plan time:**

1. **Concept docs cited in §2** (read in full or relevant §):
   - [`ontology.md`](../../../v0/concepts/ontology.md) §"table-per-node-tier" + §"edges have a direction" + §"edges are first-class" + §"edges are typed by their relation"
   - [`phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) §"wrap: baby-phi field holds phi-core type"
   - [`agent.md`](../../../v0/concepts/agent.md) §"AgentProfile binds a blueprint" + §"agent kind"
   - [`coordination.md`](../../../v0/concepts/coordination.md) §"event-driven reactivity"
   - [`permissions/05-memory-sessions.md`](../../../v0/concepts/permissions/05-memory-sessions.md) §"session lifecycle — started, live, ended"
   - [`permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) §"Template C" + §"Template D" + §"tools are resources"
   - [`permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) §"preview mode"
   - [`system-agents.md`](../../../v0/concepts/system-agents.md) §"reconfiguration audit trail"
   - [`permissions/README.md`](../../../v0/concepts/permissions/README.md) (entry-invariants source per §2 hook rule)
2. **Drift files cited in §4** (16 files):
   - D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6 — all under [`v0/implementation/m5_1/drifts/`](../../../v0/implementation/m5_1/drifts/)
3. **Existing `v0/concepts/` doc-tree shape as reference** (NEW for v2 per orchestrator instruction): `ls /root/projects/phi/baby-phi/docs/specs/v0/concepts/` returns:
   - `README.md` (index — but per orchestrator gate-1 guidance CH-20 does NOT ship a README.md for `v0/conventions/`; user chose pure-split)
   - Flat-file area docs: `agent.md`, `coordination.md`, `core-philosophy.md`, `human-agent.md`, `ontology.md`, `organization.md`, `permissions/` (subdirectory), `phi-core-mapping.md`, `project.md`, `system-agents.md`, `token-economy.md`.
   - **Shape reference takeaways for `v0/conventions/`:** flat-file naming (one `.md` per area, no per-area subdirectories needed at the 5-file scale); kebab-case file names; verified-header line 1; ~200-500 word area docs is the typical concept-doc shape (CH-20 convention-docs target ≤ 250 words per file, slightly tighter because conventions are guidance not narrative). **No structural deviation from concept-doc shape; F1.B's 5-file split mirrors the concept-doc-area-per-file convention.**
4. **Prior-chunk plans cited in §6** — N/A as plan files (no prior-chunk per-chunk plan needed; M5/P1–P7 plan archives are referenced via in-line line numbers in drift bodies). **Cross-cycle precedent reading**: [`ch-19-bucket-b-ratification-2c520ba7/plan.md`](../ch-19-bucket-b-ratification-2c520ba7/plan.md) (closest precedent; single-ADR doc-only ratification shape) + [ADR-0042](../../../v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) (first doc-only ratification ADR) + [ADR-0057](../../../v0/implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md) (second; immediately preceding). Read at planner-draft time + re-confirmed at v2 re-plan time.
5. [forward-scope §1 + §5 + §7](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) — the chunk row + Q4 (per-chunk ordering) + Q5 (M5-scope defer) + Q7 (uniform doc-only ritual) + Q9 (user-facing doc impact map).
6. [`baby-phi/CLAUDE.md`](../../../../../CLAUDE.md) phi-core Leverage section.
7. **Tag-write Repository conditional (v3 per CH-12 retrospective Row 5):** N/A — CH-20 does NOT introduce or reference a new tag-write Repository method.
8. **Engine.rs Step-N body conditional (v3 per CH-11 retrospective):** N/A — CH-20 does NOT touch `domain::permissions::engine` Step N body.

**Carry-forward invariants (verified green at chunk-open 2026-05-10):**
- `cargo test --workspace` test count: **1529** at HEAD (per CH-19 cycle-audit). Verified against `_cycle-index.md` row for CH-19 — `1529/0/2 within plan §8 v2 band`. Doc-only chunk → expected unchanged at chunk-close.
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh`: green (verified at CH-19 cycle close).
- `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`: green (verified at CH-19 cycle close; CH-20 will exercise this guard at gate-4 since chunk creates a new directory + 5 new files + cross-refs from drifts to convention-docs).
- `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh`: green (verified at CH-19 cycle close; convention-docs live at `v0/conventions/`, NOT `m*/operations/` so this guard's scope is unaffected).
- `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh`: green (verified at CH-19 cycle close).
- `modules/` git diff against chunk-open HEAD: empty (verified — no preload edits).

**Pending decisions carried into this chunk:**
- Q4 (chunk-ordering): user-decided per-chunk; CH-20 selected by user as next chunk to open (post-CH-19). No predecessor required (only soft cross-ref to CH-19 §D57.1 for D6.4 — `event-bus-wiring.md` references CH-19 by ADR ID).
- Q5 (M5-scope defer): CH-20 closes 16 LOW Bucket-C drifts; ratification path keeps all in M5 close (per Q5 LOW-drifts-via-CH-19/CH-20 rule).
- Q7 (uniform doc-only ritual): CH-20 follows the same ExitPlanMode + 4-aspect close ritual as CH-19; no shortcut.
- Q9 (user-facing doc impact map): per §3.C above, no user-facing-tier doc updates in CH-20 scope; M5-tag-close successor for the 1-line CLI reference cross-ref. **NEW peer tier `v0/conventions/` shipped in-chunk** — 5 first canonical instances per F1.B user-lock.

**Drift-file `discovered → classified → scoped` transitions:** All 16 drifts are at `discovered` per their `Status` field as of chunk-open. They were classified during M5.1/P2 concept-audit. They are scoped by APPEARING in this plan §4. CH-20 advances them through `in-chunk-plan` → `accepted-as-is` per §7 P2/P3 deliverables.

**Forward-scope row "(14 items)" off-by-2 amendment:** CH-20's §D58.10 records this; P3 deliverable 3 amends inline. Carries no new drift file (planning artefact, not concept-doc; categorised as housekeeping per CH-19/ADR-0057 §D57.7 edge-count reconcile precedent).

---

## §10 — Close criteria

**4 aspects:**

- **Code aspect.** **No code changes.** cargo test workspace passes at 1529. clippy green under `RUSTFLAGS="-Dwarnings"`. fmt --check green. check-phi-core-reuse.sh green.
- **Docs aspect — governance tier.** ADR-0058 Status: Accepted; 16 drift files Status: accepted-as-is + lifecycle-history entries (each citing the correct convention-doc file per §7 P2 deliverable 1 mapping); `_concept-audit-matrix.md` Code-evidence cell extension on the `phi-core-mapping.md` "Session wrap" row per §7 P2 deliverable 3 (only matrix-side touch); `drifts/README.md` index refreshed for 16 rows; verified-headers bumped on all touched docs; forward-scope row line 185 amended `(14 items)` → `(16 items)` per §D58.10.
- **Docs aspect — peer tier (`v0/conventions/`).** NEW directory created; **5 NEW files shipped per F1.B user-lock**: `persistence.md` (≤ 250 words), `wrap-pattern.md` (≤ 250 words), `event-bus-wiring.md` (≤ 250 words), `cli-patterns.md` (≤ 100 words), `web-patterns.md` (≤ 100 words). Each carries a verified-header line 1 + preamble citing ADR-0058 §D58.N + thematic subsections + cross-refs to drift IDs.
- **Docs aspect — user-facing tier (post-CH-22).** Per §3.C: 0 user-facing-tier files updated in-chunk; 1 file (`m5/user-guide/cli-reference-m5.md`) deferred to M5-tag-close batch with explicit successor-chunk-reference (matches CH-19's defer-decision; same M5/P9 stub).
- **phi-core leverage aspect.** §3 import-count delta = +0 (predicted); actual at chunk-close: 57 (verified via canonical grep). Forbidden-duplication greps: N/A. check-phi-core-reuse.sh green.
- **Concept alignment aspect.** Every §2 row at chunk-close target status `accepted-as-is`; none remains `contradicted` (no row was contradicted at chunk-open).

**2 confidence %:**
- **Implementation confidence %** = `claims-honored / claims-in-scope`. Claims-in-scope: 16 drift conventions + 8 P1 deliverables (mkdir + 5 convention-docs + ADR-0058 + 0 header bumps + 1 implicit "ADR cross-references all 5 convention-docs" claim) + 4 P2 deliverables + 6 P3 deliverables = **34 claims** (revised up from v1's 30 claims due to F1.B's 5-file split). Target: **≥ 31/34 = 91.2%** (≥ 9.1/10 — MEETS plan auto-approval threshold of 9/10). Aim: **34/34 = 100%** at chunk-close.
- **Documentation confidence %** = `doc-pages-cross-checkable-without-ambiguity / doc-pages-touched-in-chunk`. Doc-pages touched: **5 convention-docs** (5) + ADR-0058 (1) + 16 drift files (16) + `_concept-audit-matrix.md` (1) + `drifts/README.md` (1) + `_cycle-index.md` (1) + forward-scope file (1) = **26 doc pages** (revised up from v1's 22 pages due to F1.B's 5-file split). Target: **26/26 = 100%**.

**Composite =** `min(impl%, doc%, code-aspect-binary, governance-docs-binary, peer-tier-docs-binary, user-facing-docs-binary, phi-core-binary, concept-alignment-binary)`. **Target: 100%.**

**Explicit close-target discipline:** close report states ALL FIVE measures with named numerators/denominators (per template §10 rule).

**P4 paperwork checklist (CH-11 retro v2026-05-03 + CH-12 retro v2026-05-04):**
- For every modified doc with verified-header (line 1 `<!-- Last verified: ... -->`): confirm the new header description matches the body diff exactly. Mismatch → fix the header before chunk-seal. (≥ 26 docs apply.)
- For every `_concept-audit-matrix.md` row touched: new Status column value MUST be copy-pasted letter-for-letter from plan §2 target column for that row (per CH-12 retro Row 1). **Note: CH-20 makes only Code-evidence cell extension on 1 row, no Status flips on matrix rows** (the `accepted-as-is` status lives on the drift file, not the matrix; the matrix row stays `honored` for "Session wrap" per §7 P2 deliverable 3).

**Cargo-clean discipline (CH-18 retro Row 1, USER DIRECTIVE 2026-05-10, TWO placements):**
- (1) Immediate-post-test cleanup: AFTER P3's gate-4 sanity-check `cargo test --workspace` invocation, the implementer MUST run `/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` BEFORE issuing the next cargo invocation. Capture disk-reclaim metric in cycle-audit §7. **For doc-only chunks**, the gate-4 cargo-test is the only cargo invocation; one cleanup suffices.
- (2) Gate-5-close final cleanup: orchestrator runs `cargo clean` as last action of gate-5 close (post-retro, pre-commit), per existing CLAUDE.md gate-5 rule.

**Cycle-index P-seal rule (CH-17 retro Row 4):** P3 deliverable §7 includes inserting the CH-20 row in `_cycle-index.md` "Active cycles" table — verified at chunk-seal via `grep -n 240616a4 _cycle-index.md` returning ≥ 1 hit.

---

## §11 — Post-chunk independent audit plan

**Phase count: 3** (P1 + P2 + P3). Per the audit-envelope-size skill table:
- ≤ 2 phases → Small (1 auditor)
- **3–5 phases → Medium (2 auditors)** — applies here.
- 6+ phases → Large (3 auditors).

**Audit envelope: Medium (2 auditors).** Letters: A (code + phi-core + K8s) + B (concept + docs + ADR + 5 convention-docs + new directory). **F1.B's 5-file shape WIDENS Audit B's surface (5 files vs 1 file in v1) but each file is small (≤ 100-250 words); audit envelope STAYS medium** because the per-file content is ≤ 250 words; the surface widens horizontally but does not deepen vertically.

**Reasoning:** doc-only chunk with 3 phases falls in the medium tier (per CH-19 precedent). Audit A's surface is light (zero production code changes; zero migrations; phi-core delta 0; K8s axes all `no impact`); Audit B's surface is **moderately heavier than v1**: NEW directory `v0/conventions/` + **5 NEW convention-docs** (`persistence.md` + `wrap-pattern.md` + `event-bus-wiring.md` + `cli-patterns.md` + `web-patterns.md`; total ~950 words across 5 files) + ADR-0058 ≤ 4500 words covering 10 sub-decisions across 16 drifts + 16 drift files + matrix Code-evidence cell extension + README + cycle-index + forward-scope amendment. The asymmetric audit weight matches the chunk shape; matches CH-19's audit envelope tier (medium), with the additional cross-file integrity check (each drift's lifecycle-entry cites the correct convention-doc file per §7 P2 mapping).

### Audit A (code + phi-core + K8s) scaffold

```
You are auditing CH-20 in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at docs/specs/plan/build/ch-20-bucket-c-confirm-in-place-240616a4/plan.md (v2 — F1.B 5-file split per gate-1 user lock).

Verify each claim with file:line citation:
1. Zero code-side touches: run `git diff main -- modules/crates/` — expect ZERO non-comment changes (this chunk is purely doc-only). Confirm `git diff main -- modules/web/` is also zero.
2. phi-core leverage delta: predicted +0; actual at audit-time. Run `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect 57 (canonical baseline preserved; same as CH-19 close).
3. cargo test --workspace -- --test-threads=1 green at expected count 1529 (CH-20 doc-only → Δ +0 from CH-19 close).
4. CI guards green: `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` exit 0; `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` exit 0 (HEAVY for F1.B; new convention-doc directory + 5 new files + cross-refs from drifts to specific files; doc-link integrity load-bearing — F1.B widens cross-link surface vs F1.A); `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` exit 0; `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` exit 0.
5. K8s 7-axis classification per plan §3.B all `no impact`. Confirm A7 single-writer guarantee preserved: `grep -rn "AuditEmitter::emit\b" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect ≥ 1 (existing emitter present, not duplicated by D6.4 ratification).
6. Prior-chunk regression: re-run all 16 §6 grep commands; verify each returns expected count (citations refreshed at chunk-open).
7. clippy --workspace --all-targets under RUSTFLAGS="-Dwarnings" green.
8. NEW directory `docs/specs/v0/conventions/` exists (`ls /root/projects/phi/baby-phi/docs/specs/v0/conventions/` → exists + contains EXACTLY 5 files: persistence.md, wrap-pattern.md, event-bus-wiring.md, cli-patterns.md, web-patterns.md).

After cargo test run: MUST cargo clean immediately per chunk-auditor v7 / CH-18 retro Row 1 USER DIRECTIVE.

PASS/FAIL each. ≤ 600 words.
```

### Audit B (concept + docs + ADR + 5 convention-docs + directory) scaffold

```
You are auditing CH-20's concept-fidelity + docs-fidelity. Read-only. Plan at docs/specs/plan/build/ch-20-bucket-c-confirm-in-place-240616a4/plan.md (v2 — F1.B 5-file split per gate-1 user lock).

Verify each claim:
1. ADR-0058 Accepted at `docs/specs/v0/implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md` with sub-decisions §D58.1 through §D58.10 — grouped per plan §4 / §5 table (10 sub-decisions: 7 thematic + 3 META decisions for new tier + 5-file split shape + forward-scope amendment).
2. ADR-0058 cross-references include all 5 mandatory categories per CH-13 retro / CH-08 retro / CH-14 retro: (a) originating concept-docs, (b) closed drifts (16 IDs), (c) prior ADRs cited as precedent **MILESTONE-PREFIXED** (per CH-08 retro Row 1) — verify ADR-0042 + ADR-0057 + ADR-0029 (m5) + ADR-0033 + ADR-0028 (m4) cited with milestone-prefix paths, (d) forward-scope row at line 185-187, (e) **5 convention-doc URLs (NEW for F1.B)** — `persistence.md` + `wrap-pattern.md` + `event-bus-wiring.md` + `cli-patterns.md` + `web-patterns.md` ALL cited.
3. Each of §D58.1–§D58.10 includes the pre-existing-behaviour preservation note per CH-14 retro Row 10 (strict formula) OR the v11 (b) Multi-milestone-pattern variation OR the v11 (c) Never-shipped-yet variation per CH-19 retro Row 1. Verify EACH sub-decision's note matches the sub-decision shape: §D58.1 strict, §D58.2 (b), §D58.3 strict, §D58.4 (b), §D58.5 strict, §D58.6 strict, §D58.7 strict, §D58.8 (c), §D58.9 (c), §D58.10 strict. Spirit-of-rule check applied.
4. **NEW directory + 5 files (F1.B critical)**: `ls /root/projects/phi/baby-phi/docs/specs/v0/conventions/` returns the directory + EXACTLY 5 files: `persistence.md`, `wrap-pattern.md`, `event-bus-wiring.md`, `cli-patterns.md`, `web-patterns.md`. NO `README.md` (per orchestrator gate-1 guidance — pure split, not hybrid). Open EACH of the 5 files: verify line-1 verified-header carries CH-20 amendment description; verify thematic subsections per plan §3.C per-file content map (persistence: schema-mechanics-1, schema-mechanics-2, write-verb-1, write-verb-2, write-verb-3 = 5 subsections; wrap-pattern: 4 subsections; event-bus-wiring: 5 subsections incl. audit-event cross-ref; cli-patterns: 1 subsection; web-patterns: 1 subsection); verify each preamble cites ADR-0058 §D58.N for that file's authority; verify cross-refs to drift IDs in each subsection; verify cross-refs to existing ADR-0029 + ADR-0042 + ADR-0057 where appropriate; verify per-file word-count ≤ target (≤ 250 / 250 / 250 / 100 / 100).
5. 16 drift files Status flipped `discovered` → `accepted-as-is`; lifecycle-history entry appended for each (cite drift ID + line range of new entry); verified-header bumped to 2026-05-10 with CH-20 amendment description. **F1.B-specific**: each drift's lifecycle-entry cites the CORRECT convention-doc file per plan §7 P2 deliverable 1 mapping (D1.1+D1.2 → persistence.md; D2.1+D2.2+D4.4 → persistence.md; D1.3+D3.3+D4.3+D4.6 → wrap-pattern.md; D3.1+D3.2+D3.4+D4.5 → event-bus-wiring.md; D6.4 → event-bus-wiring.md; D7.3 → cli-patterns.md; D7.6 → web-patterns.md).
6. drifts/README.md index Status column refreshed for the 16 drift rows to `accepted-as-is ✓ (CH-20 / ADR-0058 §D58.<N> / conventions/<file>.md)`.
7. _concept-audit-matrix.md verified-header bumped to 2026-05-10 with CH-20 amendment description; only matrix-side touch is the `phi-core-mapping.md` "Session wrap" row Code-evidence cell extension citing ADR-0058 §D58.2 + cross-ref `wrap-pattern.md` + Covering-drift cell extension to `D1.3 ✓ + D3.3 ✓` (per plan §7 P2 deliverable 3). NO new matrix rows added.
8. Concept doc verified-headers: planner-recommendation was ZERO bumps; verify either zero changes OR (if planner chose to bump) each bump has a corresponding 1-line cross-ref to a specific convention-doc file in the body. No silent verified-header bumps.
9. Doc-sync widened sweep (CH-15 retro Row 1, widened): grep ALL `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` for stale-narrative phrase set: `FOLLOWUP-NN`, `deferred per`, `is NOT emitted`, `not emitted at CH-NN`, `advisory at M5`, `Step 0 only blocking`, `M6+ tightens the gate`, `at M5/P4`, `not blocking at M5`. Doc-only chunk should produce 0 matches needing patch.
10. `_cycle-index.md` "Active cycles" table contains the CH-20 row — `grep -n 240616a4 _cycle-index.md` returns ≥ 1 hit.
11. K8s deferred ledger entry: NO new entry added to `m7b/architecture/deferred-from-ch-k8s-prep.md`.
12. Plan archive at `ch-20-bucket-c-confirm-in-place-240616a4/plan.md` exists with cycle hex `240616a4` AND v2 verified-header reflecting F1.B re-plan.
13. Prior-chunk doc invariants intact: re-verify `_concept-audit-matrix.md` line 1 verified-header trailer is the CH-20 amendment (not silently overwriting CH-19); CH-20's verified-header should PREPEND, not REPLACE the CH-19 trailer.
14. Forward-scope-vs-concept-doc precedence (§3.D): no closed-set contradiction claim made by CH-20; verified.
15. Forward-scope row line 185 amended `(existing 14 items)` → `(existing 16 items)` per §D58.10; forward-scope verified-header trailer bumped with CH-20 amendment note.
16. **Cross-file integrity (F1.B-specific NEW)**: verify the 5 convention-docs cross-link consistently — `wrap-pattern.md` references `persistence.md` if any wrap-vs-persist seam is mentioned; `event-bus-wiring.md` references CH-19 §D57.1 in the audit-event cross-ref subsection; ADR-0058's cross-references list cites all 5 file URLs. Report any orphan cross-ref or broken intra-`v0/conventions/` link.

After cargo test run: MUST cargo clean immediately per chunk-auditor v7 / CH-18 retro Row 1 USER DIRECTIVE.

PASS/FAIL each. ≤ 600 words.
```

**Audit pass criteria (template §11 inheritance):**
- Any new drift discovered by the audit → its own drift file created BEFORE chunk seals.
- Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval, or converted to a drift file with explicit future-chunk assignment.
- Chunk seal blocked until audit returns clean on both A + B + all audit-discovered drifts explicitly scoped.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (4 — all green)
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (cargo capped at -j 4 per feedback_cargo_jobs_cap.md)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets -j 4
/root/rust-env/cargo/bin/cargo test --workspace -j 4

# 2.b Cargo-clean discipline (CH-18 retro Row 1 / USER DIRECTIVE 2026-05-10): immediate-post-test cleanup
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 3. Chunk-specific verification

# 3.a phi-core baseline (canonical form, no trailing :: per CH-15 retro Row 3)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (Δ 0 from CH-19 close)

# 3.b NEW directory + EXACTLY 5 convention-docs (F1.B critical)
ls /root/projects/phi/baby-phi/docs/specs/v0/conventions/
# Expect: directory exists, contains EXACTLY 5 files:
#   persistence.md
#   wrap-pattern.md
#   event-bus-wiring.md
#   cli-patterns.md
#   web-patterns.md
# (NO README.md per orchestrator gate-1 guidance — user chose pure-split, not hybrid)

ls /root/projects/phi/baby-phi/docs/specs/v0/conventions/ | wc -l
# Expect: 5 (exactly)

# 3.c Each of 5 convention-docs exists + has verified-header line 1
for f in persistence wrap-pattern event-bus-wiring cli-patterns web-patterns; do
  ls -la /root/projects/phi/baby-phi/docs/specs/v0/conventions/${f}.md
  head -1 /root/projects/phi/baby-phi/docs/specs/v0/conventions/${f}.md
done
# Expect: 5 files, each > 0 bytes; each line 1 = "<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 ...) -->"

# 3.d ADR-0058 Accepted
grep -nE "Status.*Accepted" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md
# Expect: 1 hit (top-matter "Status: Accepted" line)

# 3.e ADR-0058 cross-references all 5 convention-doc files (F1.B critical)
grep -nE "v0/conventions/(persistence|wrap-pattern|event-bus-wiring|cli-patterns|web-patterns)\\.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md | wc -l
# Expect: ≥ 5 hits (1 per file in cross-references section)

# 3.f 16 drift files at accepted-as-is
for d in D1.1 D1.2 D1.3 D2.1 D2.2 D3.1 D3.2 D3.3 D3.4 D4.3 D4.4 D4.5 D4.6 D6.4 D7.3 D7.6; do
  grep -lE "Status.*accepted-as-is" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/${d}.md && echo "OK $d" || echo "MISSING $d"
done
# Expect: 16 OK lines

# 3.g Each drift cites the CORRECT convention-doc file per §7 P2 mapping (F1.B critical)
grep -l "conventions/persistence.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/{D1.1,D1.2,D2.1,D2.2,D4.4}.md | wc -l
# Expect: 5 (D1.1, D1.2, D2.1, D2.2, D4.4)
grep -l "conventions/wrap-pattern.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/{D1.3,D3.3,D4.3,D4.6}.md | wc -l
# Expect: 4 (D1.3, D3.3, D4.3, D4.6)
grep -l "conventions/event-bus-wiring.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/{D3.1,D3.2,D3.4,D4.5,D6.4}.md | wc -l
# Expect: 5 (D3.1, D3.2, D3.4, D4.5, D6.4)
grep -l "conventions/cli-patterns.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D7.3.md | wc -l
# Expect: 1 (D7.3)
grep -l "conventions/web-patterns.md" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D7.6.md | wc -l
# Expect: 1 (D7.6)

# 3.h Cycle-index row present (CH-17 retro Row 4 P-seal paperwork)
grep -n 240616a4 /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# Expect: >= 1 hit

# 3.i Forward-scope row amendment (§D58.10)
grep -nE "existing 16 items" /root/projects/phi/baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md
# Expect: 1 hit at line 185 (replacing the prior "14 items")

# 4. Drift-file status totals (template §12 rule)
grep -l "Status.*accepted-as-is" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count from CH-19 close (10)> + 16 = 26

# 5. Convention-doc cross-refs to ADR-0058 (one per file)
for f in persistence wrap-pattern event-bus-wiring cli-patterns web-patterns; do
  grep -nE "ADR-0058|D58\\." /root/projects/phi/baby-phi/docs/specs/v0/conventions/${f}.md | head -3
done
# Expect: each file has ≥ 1 cross-ref to ADR-0058 §D58.N in its preamble

# 6. event-bus-wiring.md cross-refs CH-19 §D57.1 (audit-event placement convention)
grep -nE "ADR-0057|D57\\.1|CH-19" /root/projects/phi/baby-phi/docs/specs/v0/conventions/event-bus-wiring.md
# Expect: ≥ 1 hit (audit-event placement cross-ref subsection)

# 7. wrap-pattern.md cross-refs ADR-0029 (existing wrap-pattern decision)
grep -nE "ADR-0029|D29\\.1|D29\\.2" /root/projects/phi/baby-phi/docs/specs/v0/conventions/wrap-pattern.md
# Expect: ≥ 1 hit (preamble cross-ref)

# 8. Final disk-pressure check (post-cargo-clean reclaim metric)
df -h /root | head -3
du -sh /root/projects/phi/baby-phi/target 2>/dev/null
```

---

## Plan-time forks recap (F1 LOCKED at F1.B; F2 + F3 at planner-recommendation)

**F1 user-locked at gate-1 to F1.B (5-file split per `v0/conventions/`) — DIVERGENT from planner v1 F1.A (single consolidated file) recommendation.** All other forks resolved at planner-recommendation level via existing precedent (ADR-0042 / ADR-0057). **Direct-approval criteria re-evaluation (post F1.B lock):**
- ✅ F1 fork — LOCKED at gate-1; no further user-lock needed.
- ✅ F2 + F3 forks — at planner-recommendation; no divergence options.
- ✅ scope ≤ 1.5× forward-scope: forward-scope row mandates "1 consolidated convention doc covering ... + status of each drift flips discovered → accepted-as-is"; F1.B ships **5 docs** instead of 1 — a deliverable-count expansion. Total prose volume (~950 words across 5 files) is comparable to v1's ~600-900 words single file. **Scope ratio ≈ 1.05–1.1× (within 1.5×).**
- ✅ zero phi-core leverage delta (Δ +0).
- ✅ no new K8s blocker class (all 7 axes `no impact`).
- ✅ audit envelope ≤ medium (2 auditors per phase-count rule; F1.B widens horizontally but stays medium).
- ✅ confidence ≥ 9/10 (target 34/34 = 100%; threshold 31/34 = 91.2%).
- ✅ no new migration (doc-only chunk).

**Estimated effort:** ~1 day (matches forward-scope row 428 estimate). F1.B's 5-file split adds ~5-10% prose volume vs F1.A; offset by the higher per-file coherence (each file is small and focused). Net effort estimate: ~1.0–1.1 day.

**No new sub-forks requiring user-lock.** All decisions resolved.

**Orchestrator: ExitPlanMode auto-approval candidate (F1 locked at F1.B; F2 + F3 at planner-recommendation; all Direct-approval criteria hold).**
