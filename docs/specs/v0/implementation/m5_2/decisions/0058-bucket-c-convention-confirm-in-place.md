<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P3 chunk-seal — Status flipped Proposed → **Accepted**; all 10 sub-decisions §D58.1–§D58.10 ratified; date stamped 2026-05-10. Closes 16 Bucket-C drifts D1.1/D1.2/D1.3/D2.1/D2.2/D3.1/D3.2/D3.3/D3.4/D4.3/D4.4/D4.5/D4.6/D6.4/D7.3/D7.6 via 7 thematic sub-decisions §D58.1–§D58.7 + 3 META decisions §D58.8–§D58.10. cycle hex `240616a4`. ADR shape follows ADR-0042 (CH-03 storage-backend) + ADR-0057 (CH-19 Bucket B) precedents for "doc-only chunk → single ratifying ADR with multiple sub-decisions". F1 LOCKED at gate-1 to F1.B (5-file split per `v0/conventions/`) — DIVERGENT from planner v1 F1.A (single consolidated file) recommendation; user-locked 2026-05-10. F2 + F3 resolved at planner-recommendation level via existing precedent. Cross-cycle divergence pattern: 4 of last 6 cycles (CH-15/17/18/20).) -->

# ADR-0058 — Bucket C convention confirm-in-place (16 conventions shipped through M5/P1–P7 close + 3 META decisions for the new `v0/conventions/` peer tier)

**Status: Accepted**

**Date:** 2026-05-10
**Chunk:** CH-20
**Closes:**
- [`D1.1`](../../m5_1/drifts/D1.1.md) (LOW, C) — DEFINE FIELD on pre-existing 0001 scaffolds; never fresh DEFINE TABLE on a pre-existing table. See §D58.1.
- [`D1.2`](../../m5_1/drifts/D1.2.md) (LOW, C) — REMOVE+DEFINE retype on no-writer-scaffold (`runs_session` reverse direction). See §D58.1.
- [`D1.3`](../../m5_1/drifts/D1.3.md) (LOW, C) — Nested-`inner` wrap-pattern (NOT `#[serde(flatten)]`). See §D58.2.
- [`D2.1`](../../m5_1/drifts/D2.1.md) (LOW, C) — CREATE-not-UPDATE-as-upsert at `persist_session` / `append_loop_record` / `append_turn`. See §D58.3.
- [`D2.2`](../../m5_1/drifts/D2.2.md) (LOW, C) — LET-first RELATE statement. See §D58.3.
- [`D3.1`](../../m5_1/drifts/D3.1.md) (LOW, C) — Serde-tag-collision rename (`agent_kind` for `DomainEvent::AgentCreated`). See §D58.4.
- [`D3.2`](../../m5_1/drifts/D3.2.md) (LOW, C) — Free-function listener seam (`build_event_bus_with_m5_listeners`). See §D58.4.
- [`D3.3`](../../m5_1/drifts/D3.3.md) (LOW, C) — `Arc<Mutex<phi_core::SessionRecorder>>` interior-mutability wrap. See §D58.2.
- [`D3.4`](../../m5_1/drifts/D3.4.md) (LOW, C) — Trait-split-by-scope for resolvers (Template C org-scoped / Template D project-scoped). See §D58.4.
- [`D4.3`](../../m5_1/drifts/D4.3.md) (LOW, C per forward-scope; drift-file frontmatter says B) — `Vec<ToolSummary>` HTTP wire-shape projection over `Vec<Box<dyn AgentTool>>` trait-object. See §D58.2.
- [`D4.4`](../../m5_1/drifts/D4.4.md) (LOW, C) — Branch-on-existence for upsert-vs-create at handler-side. See §D58.3.
- [`D4.5`](../../m5_1/drifts/D4.5.md) (LOW, C) — Typed-writer-per-edge-type at Repository trait surface. See §D58.4.
- [`D4.6`](../../m5_1/drifts/D4.6.md) (LOW, C) — `Option<X>` dual-mode discriminator (`SessionLaunchContext.first_loop_id`). See §D58.2.
- [`D6.4`](../../m5_1/drifts/D6.4.md) (LOW, C) — Audit-event placement convention extends to system-agents page-13 (cross-ref CH-19 / ADR-0057 §D57.1). See §D58.5.
- [`D7.3`](../../m5_1/drifts/D7.3.md) (LOW, C) — `phi session preview` 5th CLI subcommand wraps existing HTTP route. See §D58.6.
- [`D7.6`](../../m5_1/drifts/D7.6.md) (LOW, C) — Next.js 14 hybrid server-action shape (sibling `actions.ts` + inline `<form action={run}>` closures with string-only captures). See §D58.7.

All sixteen drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../m5_1/process/drift-lifecycle.md).

**Drift-file bucket reconciliation note (CH-19 precedent for D-new-27 / D-new-30):** D4.3's drift-file frontmatter classifies it as Bucket B; forward-scope row 186 enumerates D4.3 in CH-20's Bucket-C list. **Forward-scope assignment is binding.** Drift-file `Bucket` field stays B; the CH-20 lifecycle-history entry notes the chunk-claim regardless of bucket.

---

## Context

The M5.1/P3 forward-scope inventory split the 60-drift catalogue into three buckets: **A** (load-bearing scope gap), **B** (underspecified shape choice), **C** (convention/pattern decision). Bucket C drifts are the conventions the codebase consistently follows that the original plan/concept-doc text was silent on. The shipped convention works; what was missing was the formal acceptance + the recoverable record explaining why this shape rather than another.

CH-20 is the dedicated **convention-confirm-in-place chunk** for the C-bucket drifts that survived the M5/P5–P7 close — sibling to CH-19 (Bucket B ratification, closed 2026-05-10 / cycle hex `2c520ba7`). Where Bucket B drifts are *shape choices* the implementer picked between alternatives (= ratify via ADR), Bucket C drifts are *conventions* the codebase follows that the plan was silent on (= confirm-in-place via convention-docs + ADR sub-decisions).

The chunk produces (a) **5 new convention-docs** under the new `v0/conventions/` directory (FIRST files under the new peer tier) — covering 5 thematic conventions across the 16 drifts; (b) this consolidated **ADR-0058** with sub-decisions §D58.1–§D58.10 referencing the 5 convention-docs; (c) drift-status flips `discovered → accepted-as-is` for all 16; (d) `_concept-audit-matrix.md` row refreshes (limited — most C-bucket drifts are below matrix granularity per CH-19/ADR-0057 §"Label-coverage rollup notes" precedent); (e) `drifts/README.md` index Status flips. **No production code changes. No migrations. No test-count changes. phi-core import baseline preserved at 57.**

This ADR is the **third consolidated convention-ratification ADR** in baby-phi (after [ADR-0042](0042-storage-backend-configurable.md) for CH-03's storage-backend ratification + [ADR-0057](0057-bucket-b-convention-ratification.md) for CH-19's Bucket B ratification). The shape-precedent — "doc-only chunk → single ratifying ADR with multiple sub-decisions" — is now a stable 3-cycle precedent. **F1.B's 5-file convention-doc shape is a NEW precedent** that CH-21+ can mirror for similar multi-thematic ratifications.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-20 application: the 16 Bucket-C drifts have lived as `discovered` in the catalogue since M5/P1–P7 close (April 23–24, 2026); shipping CH-20 closes them by promoting their convention-status from "shipped but undocumented" to "shipped + ADR-0058 + 5 convention-docs + matrix-side rollup honored." The chunk's value is removing silent-convention drift and creating the FIRST `v0/conventions/` directory entries — establishing the convention-doc-as-first-class-artefact pattern for future M6+ reviewer guidance.

---

## Forks

**Forks: F1 LOCKED at gate-1 to F1.B (5-file split per `v0/conventions/`) — DIVERGENT from planner v1 F1.A (single consolidated file) recommendation, user-locked 2026-05-10. F2 + F3 resolved at planner-recommendation level via existing ADR-0042 + ADR-0057 precedent. Cross-cycle divergence pattern: 4 of last 6 cycles (CH-15/17/18/20).**

Three potentially-forked decisions resolved at gate-1:

- **F1 (LOCKED at F1.B — DIVERGENT from planner v1 F1.A) — Convention-doc granularity.** Options considered:
  - **(a) Single consolidated file** at `v0/conventions/persistence-and-wiring.md` (~600–900 words; 5 thematic sections covering all 16 drifts). **Planner v1 recommendation — REJECTED at gate-1 user-lock.**
  - **(b) Split across multiple files (one per convention area)** at `v0/conventions/{persistence,wrap-pattern,event-bus-wiring,cli-patterns,web-patterns}.md`. **USER LOCK at gate-1 (2026-05-10).** Final file count: **5**.
  - (c) Hybrid (single file with named sub-§ anchors). REJECTED at gate-1 user-lock.

  **Final file count rationale (5 files):** Persistence is one coherent area (5 drifts: schema mechanics + write verbs). Wrap-pattern is one coherent area (4 drifts on phi-core wrap idioms). Event-bus-wiring covers 4 drifts + folds in D6.4 audit-event placement as a cross-ref subsection. CLI patterns covers D7.3 only. Web patterns covers D7.6 only. CLI + web kept separate because their review triggers are independent (CLI completion-help test stays green vs Next.js 15 migration trigger). **No `README.md` index** (per orchestrator gate-1 guidance: user chose pure-split, not hybrid).

- **F2 — ADR-vs-convention-doc shape (planner-recommendation lock).** Both: ADR-0058 captures decision rationale + each sub-decision; convention-docs are the operator/reviewer-facing distillation. **Resolved: both.** Forward-scope binds the convention-doc(s); per-chunk-planning-template §5 binds the ADR. Different audiences: ADR-0058 records decision provenance + cross-refs; convention-docs are reviewer-friendly "do this, not that" guidance.

- **F3 — Drift transition target (planner-recommendation lock).** All 16 drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../m5_1/process/drift-lifecycle.md) AND per forward-scope row 187 verbatim. **Resolved: `accepted-as-is` for all 16** (matches CH-19/ADR-0057 §F3 precedent).

---

## Decision

### §D58.1 — D1.1 + D1.2: Persistence — schema mechanics (DEFINE FIELD on pre-existing scaffold + REMOVE+DEFINE retype on no-writer-scaffold)

**Pre-existing implementation preserved at [`modules/crates/store/migrations/0005_sessions_templates_system_agents.surql`](../../../../../../modules/crates/store/migrations/0005_sessions_templates_system_agents.surql) (M5/P1 close, 2026-04-23); CH-20 ratifies without behaviour change.**

The convention has two parts:

**Part A (D1.1) — DEFINE FIELD on pre-existing scaffolds (never fresh DEFINE TABLE).** Migration 0005 layers governance columns onto pre-existing `session` / `turn` scaffolds (defined in `0001_initial.surql`). The migration uses `DEFINE FIELD` only — never re-issues `DEFINE TABLE`. SurrealDB's `DEFINE TABLE` is destructive on existing rows; reissuing it on a populated scaffold would silently destroy session-history.

**Part B (D1.2) — REMOVE + DEFINE retype on no-writer-scaffold.** The `runs_session` edge table was originally defined in `0001_initial.surql:327` with reverse direction (`agent → session`); 0005 retypes it to forward direction (`session → project`) via `REMOVE TABLE runs_session; DEFINE TABLE runs_session ...`. The retype is safe at v0.1 because the M1-era reverse direction was a scaffold without live writers — no production session data flowed through `agent → session`. Tables WITH live writers would require a paired migration + writer change atomically.

**Risk acceptance.** Acceptable. Both parts are reviewer-enforced — the discipline is "search the migration body for forbidden DEFINE TABLE on pre-existing tables" + "search REMOVE TABLE migrations for live-writer presence." Non-compliance produces destructive migration behaviour at apply time, so reviewer discipline is the gate.

**Review trigger: M7b zombie-table cleanup if pursued** — the inactive `loop` table (M1-era scaffold; never wired) is a candidate for removal in a future cleanup migration; that migration would exercise the same REMOVE+DEFINE pattern as 0005's `runs_session` retype.

**Rejected alternative.** "Always re-issue DEFINE TABLE atomically with the field set" was rejected because SurrealDB DDL DEFINE TABLE without an explicit `IF NOT EXISTS` (which 0005 does NOT use) is destructive; reviewer-grep is more defensible than relying on a SurrealDB version-specific clause.

### §D58.2 — D1.3 + D3.3 + D4.3 + D4.6: Wrap pattern (4 phi-core wrap idioms — nested-inner, interior-mut, wire-shape projection, dual-mode discriminator)

**Pre-existing implementation preserved: nested-`inner` wrap-pattern + `Arc<Mutex<_>>` interior-mut wrap + HTTP wire-shape projection + dual-mode discriminator (pattern emerged across M3 `OrganizationDefaultsSnapshot` + M5/P1 `Session/LoopRecordNode/TurnNode` + M5/P3 `BabyPhiSessionRecorder` + M5/P4 `SessionLaunchContext` tags; CH-20 ratifies the convention as canonical, does not change shipped code).**

Four wrap idioms baby-phi uses to extend phi-core types with governance fields:

**(D1.3) Nested-inner form (NOT `#[serde(flatten)]`).** `Session` / `LoopRecordNode` / `TurnNode` wrap phi-core types via plain nested `inner: phi_core::X` field — not `#[serde(flatten)]`. Phi-core's `Session.session_id` would collide on flatten with baby-phi's governance `id` field. The per-record-type precedent is [ADR-0029](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) §D29.1.

**(D3.3) Interior-mutability wrap (`Arc<Mutex<phi_core::SessionRecorder>>`).** `BabyPhiSessionRecorder.inner: Arc<Mutex<phi_core::SessionRecorder>>` — phi-core's `&mut self` recorder API requires interior mutability when the recorder lives behind shared state (`AppState::session_registry`). The per-record-type precedent is [ADR-0029](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) §D29.2.

**(D4.3) HTTP wire-shape projection (`Vec<ToolSummary>`).** `resolve_agent_tools(...) -> Result<Vec<ToolSummary>, _>` returns the HTTP wire shape — NOT `Vec<Box<dyn AgentTool>>` (phi-core trait-object). The compile-time witness `_is_phi_core_agent_tool_trait<T: AgentTool + ?Sized>` is preserved to assert the trait remains satisfiable; the runtime wire path uses the projection.

**(D4.6) Dual-mode discriminator (`Option<X>`).** `SessionLaunchContext.first_loop_id: Option<LoopId>` — `Some(...)` for the launch-chain pre-persisted path; `None` for the standalone-test full-persist path. The Option-discriminator distinguishes two coordination modes without a separate type.

**Risk acceptance.** Acceptable. All four idioms are reviewer-enforced + grep-verifiable; non-compliance would produce compile errors (nested-inner serde) or runtime test failures (interior-mut Mutex deadlocks would fail tests). The convention is style-validated by precedent across 4 wrap sites.

**Review trigger: M6+ phi-core API stabilisation if pursued** — if phi-core's `&mut self` APIs migrate to `&self` + interior `RefCell`/`Mutex`, the interior-mut wrap (D3.3) may simplify; the other three idioms are stable.

**Rejected alternative.** "Use `#[serde(flatten)]` uniformly" was rejected because of the field-name collision with phi-core. "Use a separate type per coordination mode" (instead of D4.6 Option-discriminator) was rejected as field-duplication churn for a 1-bit discriminator.

### §D58.3 — D2.1 + D2.2 + D4.4: Persistence — write verbs (CREATE-not-UPDATE-as-upsert + LET-first RELATE + branch-on-existence)

**Pre-existing implementation preserved at [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) (`persist_session` CREATE at line 3147; `runs_session` LET-first RELATE at line 3167) + [`modules/crates/server/src/platform/agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) (branch-on-existence at line 381) (shipped at M5/P2 + M5/P4 close, 2026-04-23); CH-20 ratifies without behaviour change.**

Three write-verb conventions:

**(D2.1) CREATE-not-UPDATE-as-upsert.** `persist_session` / `append_loop_record` / `append_turn` use `CREATE type::thing(...) ...` rather than `UPDATE type::thing(...) MERGE { ... }` as upsert. UPDATE on a non-existent SCHEMAFULL+FLEXIBLE row is a silent no-op in SurrealDB (returns empty result, no error). CREATE surfaces a duplicate-violation that maps to `RepositoryError::Conflict` — exactly the surface the rest of the codebase expects for double-write attempts.

**(D2.2) LET-first RELATE.** RELATE statements bind FROM/TO endpoints via LET first: `LET $f = type::thing('agent', $agent_id); LET $t = type::thing('session', $session_id); RELATE $f -> runs_session -> $t SET ...`. SurrealDB's parser rejects inline `type::thing(...)` in FROM/TO slots (parser-level constraint, not lookup-time). Reviewer rule: any RELATE with inline `type::thing(...)` in FROM/TO is rejected.

**(D4.4) Branch-on-existence for upsert-vs-create.** The agent-profile rebind handler at `update.rs` branches on `current_profile.is_some()` → `upsert_agent_profile` when prior row exists, `create_agent_profile` otherwise. Same root-cause as D2.1 (UPDATE-as-upsert is silent no-op); the branch lifts the upsert-vs-create decision to handler-side where the operator-action context is known.

**Risk acceptance.** Acceptable. All three are reviewer-grep-enforced. Non-compliance would produce silent data-loss (D2.1 + D4.4) or runtime parser errors (D2.2) — failing fast at test time.

**Review trigger: M6+ SurrealDB UPSERT-keyword refactor if pursued** — SurrealDB v2.x ships an explicit `UPSERT` keyword that subsumes CREATE-or-UPDATE-by-id semantics; if a future chunk migrates to `UPSERT`, all three sub-conventions could simplify. Until then, the shipped pattern is stable.

**Rejected alternative.** "Use UPDATE MERGE everywhere as upsert" was rejected because of silent-no-op-on-missing-row. "Inline `type::thing(...)` in RELATE" was rejected because the SurrealDB parser rejects it (not a style choice — a hard constraint).

### §D58.4 — D3.1 + D3.2 + D3.4 + D4.5: Event-bus + listener wiring (4 idioms)

**Pre-existing implementation preserved: serde-collision-rename + free-function-listener-seam + scoped-resolver-trait-split + typed-writer-per-edge (pattern emerged across M5/P3 [`events/mod.rs`](../../../../../../modules/crates/domain/src/events/mod.rs) + [`state.rs`](../../../../../../modules/crates/server/src/state.rs) + M5/P4 [`repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) + [`projects/resolvers.rs`](../../../../../../modules/crates/server/src/platform/projects/resolvers.rs); CH-20 ratifies the convention as canonical, does not change shipped code).**

Four event-bus + listener-wiring conventions:

**(D3.1) Serde-tag-collision rename.** `DomainEvent::AgentCreated` field is named `agent_kind: AgentKind` (NOT `kind: AgentKind`) because the enum-level `#[serde(tag = "kind")]` discriminator already uses the JSON key `"kind"` for variant disambiguation. A `kind:` field on the variant body would collide on serialization. Reviewer rule: any new `DomainEvent` variant carrying a `kind`-named field renames to `<topic>_kind`.

**(D3.2) Free-function listener seam.** Listener wiring lives at free function `state::build_event_bus_with_m5_listeners` — NOT inside `AppState::new` constructor. The free-function seam keeps the listener set test-isolatable: the `handler_count_is_five_at_m5` test asserts the listener count without spinning a full `AppState`. New listeners attach via the seam; the count test gets a row update.

**(D3.4) Trait-split-by-scope for resolvers.** `TemplateCAdoptionArResolver(OrgId)` is org-scoped (`OrgId → AuthRequestId`); `TemplateDAdoptionArResolver(ProjectId)` is project-scoped (`ProjectId → (OrgId, AuthRequestId)`). The asymmetry is concept-mandated by [`permissions/07`](../../../concepts/permissions/07-templates-and-tools.md): `MANAGES` lives at org scope; `HAS_AGENT_SUPERVISOR` lives at project scope. Reviewer rule: scope-asymmetric concept primitives get split-by-scope resolver traits, not a single resolver with conditional dispatch.

**(D4.5) Typed-writer-per-edge-type at Repository trait.** Repository trait surfaces typed methods like `write_uses_model_edge` per edge-type — NOT a generic `create_edge(&Edge)`. Default trait body returns `RepositoryError::Backend(...)` so future impls fail fast on missed coverage. Reviewer rule: new edge-types get a typed-writer Repository method with a `Backend(...)` default body.

**Risk acceptance.** Acceptable. All four conventions are grep-enforced + the listener-count test pins (D3.2). Non-compliance fails fast (D3.1: serde error at runtime; D3.2: listener-count test fails; D3.4: per-scope helpers don't compile against the wrong concept primitive; D4.5: Repository default-body `Backend(...)` panics at runtime).

**Review trigger: none near-term** — all four conventions are stable across the M5 listener set. M6+ may add new listeners, which would extend §D58.4 rather than revisit it.

**Rejected alternative.** "Use generic `kind:` field with a separate manual disambiguator" (instead of serde-tag rename) was rejected as more verbose. "Wire listeners in `AppState::new`" was rejected because the test-isolation cost outweighed constructor concision. "Single resolver with `match scope { Org(_) | Project(_) }`" (instead of trait-split) was rejected because the concept asymmetry is real — a unified type would require representing the union internally. "Generic `create_edge(&Edge)` Repository method" was rejected because typed-per-edge methods catch missing-impl errors at compile time.

### §D58.5 — D6.4: Audit-event placement convention extends to system-agents page-13 (cross-ref CH-19 / ADR-0057 §D57.1)

**Pre-existing implementation preserved at [`modules/crates/server/src/platform/system_agents/audit_events.rs`](../../../../../../modules/crates/server/src/platform/system_agents/audit_events.rs) (M5/P6 close, 2026-04-23); CH-20 ratifies without behaviour change. Cross-ref CH-19 / [ADR-0057](0057-bucket-b-convention-ratification.md) §D57.1 — D6.4 IS the same convention as D5.2 applied to system-agents.**

The convention: system-agent page-13 audit-event builders live at `server::platform::system_agents::audit_events` (4 builder fns: reconfigure / add / disable / archive). This is the **same platform-tier convention** CH-19 / ADR-0057 §D57.1 ratified for page-12 templates. Both feed the single-writer `AuditEmitter` chain (per [ADR-0033](0033-k8s-prep-refactors.md) §D33.4), so the K8s A7 single-writer-symmetry guarantee is preserved.

D6.4 is documented here as a **convention-extension** (not a re-ratification). The 1-paragraph cross-ref subsection in [`event-bus-wiring.md` §"Audit-event placement cross-ref"](../../../conventions/event-bus-wiring.md) names the file location + cross-refs CH-19. Future page-N HTTP handlers default to platform-tier when the event corresponds to an operator action; default to domain-tier when the event corresponds to a state-machine transition or fire listener.

**Risk acceptance.** Acceptable. The convention IS the CH-19 §D57.1 convention; D6.4 confirms the convention extends to a second admin page. Non-compliance produces no runtime symptom (audit hash-chain still computes correctly), so the convention is style-not-correctness.

**Review trigger: cross-ref CH-19 / ADR-0057 §D57.1 review trigger (none near-term)** — convention stable across 2 admin pages (page-12 + page-13).

**Rejected alternative.** "Re-ratify the convention in §D58.5 verbatim" was rejected because it would duplicate the ADR-0057 §D57.1 risk-acceptance + rejected-alternative narrative without adding new substance. "Promote D6.4 to a standalone convention-doc file (`audit-event-placement.md`)" was rejected because D6.4 is a single-sentence cross-ref to CH-19 — folding it into `event-bus-wiring.md` as a §"Audit-event placement cross-ref" subsection is more cohesive.

### §D58.6 — D7.3: CLI scope-addition discipline (`phi session preview` 5th subcommand)

**Pre-existing implementation preserved at [`modules/crates/cli/src/commands/session.rs`](../../../../../../modules/crates/cli/src/commands/session.rs) line 101 (subcommand definition) + line 538 (handler) (M5/P7 close, 2026-04-24); CH-20 ratifies without behaviour change.**

The convention: `phi session preview` ships as a 5th subcommand alongside `launch` / `show` / `terminate` / `list`. It wraps the existing `POST /api/v0/sessions/preview` HTTP route — no new HTTP surface is introduced by the CLI addition. The completion-help regression test `cli::tests::completion_help::completion_session_subcommand_includes_preview` pins the CLI surface stable.

Reviewer rule: any new CLI subcommand that wraps an existing HTTP route MUST be pinned by a completion-help regression test; pure CLI scope-additions never bypass an existing HTTP-route surface (i.e., the CLI is always a thin wrapper, never a parallel implementation).

**Risk acceptance.** Acceptable. The completion-help test gates the CLI surface; any future `phi session` subcommand reorder (or accidental removal) trips the test. Non-compliance is detected at every `cargo test` run.

**Review trigger: CLI completion-help test continues to pin** — no near-term action; the test is the binding gate.

**Rejected alternative.** "Add `--preview` flag to `phi session launch` instead of a 5th subcommand" was rejected because the preview action is read-only (computes the session manifest without persisting) — it doesn't share the launch action's persistence semantics, so a separate subcommand is the cleaner surface. "Skip the CLI surface; preview only via HTTP" was rejected because the preview action is operator-attended (operators inspect manifests before approving an action); CLI parity with HTTP is the operator-ergonomic surface.

### §D58.7 — D7.6: Web pattern (Next.js 14 hybrid server-action shape)

**Pre-existing implementation preserved at `modules/web/app/(admin)/organizations/[id]/templates/page.tsx` lines 142, 159, 182, 199 (M5/P7 close, 2026-04-24); CH-20 ratifies without behaviour change.** (Path uses Next.js parenthesized route-group convention; backtick-only ref per ADR-0057 §D57.5 precedent.)

The convention: the templates page uses a **hybrid server-action shape**:

- **Top-level sibling `actions.ts`** (named exports) — for actions reused across the page (e.g. `revokeAdoption`, `submitAdoption`).
- **Inline per-row `<form action={run}>` closures** with `"use server"` directive at the function body — for actions where the closure captures a row-specific dynamic id (e.g. `orgId`, `templateKind`).

Inline closures MUST capture string-only data — Next.js 14's dynamic-id pattern is serialization-safe only for string captures; capturing a struct or object breaks the server-action boundary at the runtime serialization step.

Reviewer rule: server-action closures capture string-only; row-keyed actions inline; cross-row actions go to sibling `actions.ts`.

**Risk acceptance.** Acceptable. The pattern is reviewer-grep-enforced (`grep "use server"` returns 4 inline directives + the file references in `actions.ts`); non-compliance would produce a runtime serialization error caught at the first interaction with the form.

**Review trigger: M7+ Next.js 15 migration if pursued** — Next.js 15 may stabilize the dynamic-id pattern (or remove it); a migration would re-evaluate the hybrid shape against the new framework guarantees. Until then, the M5/P7 pattern is stable.

**Rejected alternative.** "All actions in `actions.ts`" was rejected because dynamic-id closures can't be top-level exports (Next.js 14 server-action boundary requires the captured closure live at the call site for dynamic-id pattern to serialize). "All actions inline" was rejected because cross-row reused actions would duplicate inline closure bodies — the sibling-`actions.ts` reuse is cleaner.

### §D58.8 (META) — Doc-tree introduction: the `v0/conventions/` peer tier

**Pre-existing absence preserved: no `v0/conventions/` directory existed before CH-20 (verified `ls baby-phi/docs/specs/v0/conventions/ → No such file or directory` at chunk-open 2026-05-10 + at v2 re-plan time); CH-20 ratifies the directory introduction as canonical.**

The convention: a new peer-tier directory `docs/specs/v0/conventions/` exists alongside `docs/specs/v0/concepts/`. The peer tier holds **reviewer-tier convention documents** — guidance below concept-doc semantic granularity but above per-feature implementation-tier docs. Conventions span multiple milestones (D1.x ships at M5/P1; D3.1 ships at M5/P3; D7.6 ships at M5/P7) — they are NOT milestone-bounded, so they live at the workspace level.

Future Bucket-C-style ratification chunks (or any chunk that documents a multi-milestone convention) ships convention-docs at this peer tier. The doc-tree role is "long-lived reviewer guidance, not implementation-milestone scope."

**Risk acceptance.** Acceptable. The peer-tier introduction is structural — no behavioural risk. The directory shape (flat files, one per thematic area) follows `v0/concepts/`'s flat-file precedent.

**Review trigger: none near-term** — peer tier stable; CH-21+ can add new files without revisiting the tier shape.

**Rejected alternative.** "Place convention-docs under `m5/architecture/conventions.md`" was rejected because conventions span multiple milestones. "Place convention-docs as new sub-§ inside concept docs" was rejected because concept docs are source-of-truth at the semantic layer; conventions document shape choices below semantic granularity.

### §D58.9 (META) — First convention-doc set: 5-file split per F1.B user-lock

**Pre-existing absence preserved: no convention-doc files existed before CH-20; F1.B user-lock at gate-1 (2026-05-10, divergent from planner v1 F1.A single-file recommendation) settles the granularity at 5 separate files (one per thematic area: persistence / wrap-pattern / event-bus-wiring / cli-patterns / web-patterns).**

The convention: the first canonical convention-doc set ships as **5 separate files**:

1. [`v0/conventions/persistence.md`](../../../conventions/persistence.md) — covers D1.1 + D1.2 (schema mechanics) + D2.1 + D2.2 + D4.4 (write verbs) — 5 drifts.
2. [`v0/conventions/wrap-pattern.md`](../../../conventions/wrap-pattern.md) — covers D1.3 + D3.3 + D4.3 + D4.6 — 4 drifts.
3. [`v0/conventions/event-bus-wiring.md`](../../../conventions/event-bus-wiring.md) — covers D3.1 + D3.2 + D3.4 + D4.5 + D6.4 (audit-event placement cross-ref) — 5 drifts.
4. [`v0/conventions/cli-patterns.md`](../../../conventions/cli-patterns.md) — covers D7.3 — 1 drift.
5. [`v0/conventions/web-patterns.md`](../../../conventions/web-patterns.md) — covers D7.6 — 1 drift.

ADR-0058 is governance-tier; convention-docs are reviewer-tier. The two artefact tiers cross-reference each other (ADR §"Cross-references" cites all 5 convention-doc URLs; each convention-doc preamble cites ADR-0058 §D58.N for that file's authority).

**No `README.md` index** (per F1.B user-lock pure-split, not F1.C hybrid). Discoverability via the directory listing + this §D58.9's documentation of the file set.

**Risk acceptance.** Acceptable. The 5-file shape is the user-lock outcome at gate-1. Future Bucket-C-style ratifications may mirror the 5-file shape OR consolidate further if their drift count is small (< 3 drifts in a single thematic area might fold into an existing convention-doc rather than a new file).

**Review trigger: none near-term.** CH-21+ planners may ship a README index later if the directory grows beyond ~10 files.

**Rejected alternative.** "(a) Single consolidated `persistence-and-wiring.md`" — rejected at gate-1 user-lock. "(c) Hybrid (single file with named sub-§ anchors)" — rejected at gate-1 user-lock. "(b) 6-file with D6.4 standalone `audit-event-placement.md`" — rejected because D6.4 is a single-sentence cross-ref. "(b) 4-file merging cli + web into `surface-patterns.md`" — rejected because the CLI + web review triggers are independent.

### §D58.10 (META) — Forward-scope row "(14 items)" off-by-2 amendment

**Pre-existing wording preserved at [`docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` line 185](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (M5.1/P3 authoring, 2026-04-24); CH-20 amends `(existing 14 items)` → `(existing 16 items)` inline at chunk-seal P3 to match the empirical drift count.**

The convention: when a forward-scope row's parenthetical drift-count is empirically off, the chunk that closes the drifts amends the wording inline at chunk-seal (rather than filing a separate planning-artefact correction drift). The forward-scope row is a planning artefact, not a concept-doc; the parenthetical fix is a 1-line content edit (categorised as Trivial-1L per the audit-fix-loop rule).

**Risk acceptance.** Acceptable. The forward-scope row's enumerated drift list at line 186 carries 16 IDs; the parenthetical "(14)" was a counting error at authoring time. The amendment makes the wording match the empirical reality.

**Review trigger: none near-term** — wording aligned with empirical count.

**Rejected alternative.** "File a CH-20-noticed planning-artefact drift" was rejected because the forward-scope row is a planning artefact, not a concept-doc — drift-tracking applies to concept-doc deltas. "Leave the wording at (14) and footnote in the ADR" was rejected because the row is referenced by future planners; an inline fix at chunk-seal is cleaner than a permanent footnote.

---

## Risk acceptance (consolidated)

All sixteen sub-decisions across §D58.1–§D58.7 share Bucket-C characteristics: **shipped convention works; cost of code-side remediation outweighs benefit at M5 close.** None of the sixteen introduces a runtime-correctness risk. Five of seven thematic sub-decisions (§D58.1, §D58.4, §D58.5, §D58.6, §D58.7) carry no near-term review trigger; two (§D58.2 → M6+ phi-core API stabilisation; §D58.3 → M6+ SurrealDB UPSERT-keyword refactor) carry milestone-bounded review triggers. The three META decisions (§D58.8 / §D58.9 / §D58.10) are structural — no runtime risk.

Per [`drift-lifecycle.md:122-128`](../../m5_1/process/drift-lifecycle.md), `accepted-as-is` requires (a) Accepted ADR documenting the drift ID — **this ADR**, with §D58.1–§D58.7; (b) explicit risk-acceptance statement — **above, this section**; (c) review trigger — **per-theme, in each §D58.N body above**. All three preconditions met; transitions enabled at chunk-seal P3.

---

## Pre-existing-behaviour preservation (consolidated)

Per CH-14 retro Row 10 (v8 strict formula) + CH-19 retro Row 1 (v11 formula relaxation — 3 documented variations permitted), each sub-decision opens with a preservation note. ADR-0058 uses the appropriate variation per sub-decision shape:

- **Strict formula** (single shipped-at date + file:line): §D58.1, §D58.3, §D58.5, §D58.6, §D58.7, §D58.10.
- **(b) Multi-milestone-pattern variation** (pattern emerged across multiple M-tags): §D58.2, §D58.4.
- **(c) Never-shipped-yet variation** (META decisions covering doc-tree introductions): §D58.8, §D58.9.

The spirit of the rule — "make explicit (i) what was the case before this chunk, (ii) whether this chunk changes it, (iii) where the historical evidence lives" — is satisfied for every sub-decision. ADR-0058 does NOT change runtime behaviour — every sub-decision §D58.1–§D58.7 documents conventions SHIPPED at M5/P1–P7 close; the META decisions §D58.8–§D58.10 introduce documentation structure, not behaviour.

---

## Out of scope

Tracked successors (each binds to a specific future chunk; none is open-ended):

- **§D58.2 → M6+ phi-core API stabilisation if pursued** — interior-mut wrap may simplify if phi-core's `&mut self` APIs migrate to `&self` + interior-mut.
- **§D58.3 → M6+ SurrealDB UPSERT-keyword refactor if pursued** — explicit `UPSERT` keyword may subsume the three sub-conventions.
- **§D58.7 → M7+ Next.js 15 migration if pursued** — framework-version migration may re-evaluate the hybrid shape.
- **§D58.1 → M7b zombie-table cleanup if pursued** — inactive `loop` table candidate for REMOVE+DEFINE retype.

Out-of-scope explicitly:

- **No new code-side abstraction** — CH-20 is doc-only. No new traits, no new enum variants, no new Repository methods.
- **No migration** — the `_migrations` table count stays unchanged.
- **No phi-core import change** — baseline preserved at 57 (Δ +0).
- **No README.md** in `v0/conventions/` (per F1.B pure-split user-lock at gate-1).

---

## Cross-references

### (a) Originating concept docs

- [`concepts/ontology.md`](../../../concepts/ontology.md) §"table-per-node-tier" + §"edges have a direction" — D1.1 + D1.2 + D2.1 + D2.2 + D4.5 source-of-truth.
- [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §"wrap: baby-phi field holds phi-core type" — D1.3 + D3.3 + D4.3 + D4.6 source-of-truth.
- [`concepts/agent.md`](../../../concepts/agent.md) §"AgentProfile binds a blueprint" — D4.4 source-of-truth.
- [`concepts/coordination.md`](../../../concepts/coordination.md) §"event-driven reactivity" — D3.1 + D3.2 source-of-truth.
- [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"preview mode" — D7.3 source-of-truth.
- [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"session lifecycle" — D4.6 source-of-truth.
- [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"Template C/D" — D3.4 + D4.3 source-of-truth.
- [`concepts/system-agents.md`](../../../concepts/system-agents.md) §"reconfiguration audit trail" — D6.4 source-of-truth.
- [`concepts/permissions/README.md`](../../../concepts/permissions/README.md) — entry-invariants source for the permissions subtree (cited per per-chunk-template §2 rule; CH-20 does NOT modify).

### (b) Closed drifts (16)

- [D1.1](../../m5_1/drifts/D1.1.md), [D1.2](../../m5_1/drifts/D1.2.md), [D1.3](../../m5_1/drifts/D1.3.md), [D2.1](../../m5_1/drifts/D2.1.md), [D2.2](../../m5_1/drifts/D2.2.md), [D3.1](../../m5_1/drifts/D3.1.md), [D3.2](../../m5_1/drifts/D3.2.md), [D3.3](../../m5_1/drifts/D3.3.md), [D3.4](../../m5_1/drifts/D3.4.md), [D4.3](../../m5_1/drifts/D4.3.md), [D4.4](../../m5_1/drifts/D4.4.md), [D4.5](../../m5_1/drifts/D4.5.md), [D4.6](../../m5_1/drifts/D4.6.md), [D6.4](../../m5_1/drifts/D6.4.md), [D7.3](../../m5_1/drifts/D7.3.md), [D7.6](../../m5_1/drifts/D7.6.md).

### (c) Prior ADRs cited as precedent (milestone-prefixed per CH-08 retro Row 1)

- [`m5_2/decisions/0042-storage-backend-configurable.md`](0042-storage-backend-configurable.md) (CH-03) — doc-only-ratification-chunk shape precedent (first instance).
- [`m5_2/decisions/0057-bucket-b-convention-ratification.md`](0057-bucket-b-convention-ratification.md) (CH-19) — doc-only-ratification-chunk shape precedent (second instance); §D57.1 cross-referenced from §D58.5 (D6.4 IS the same convention extended to system-agents).
- [`m5/decisions/0029-session-persistence-and-recorder-wrap.md`](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md) (M5; §D29.1 nested-not-flatten + §D29.2 `Arc<Mutex<_>>`) — per-record-type precedent for §D58.2 wrap-pattern generalisation.
- [`m5_2/decisions/0033-k8s-prep-refactors.md`](0033-k8s-prep-refactors.md) (CH-K8S-PREP; §D33.4 single-AuditEmitter-writer guarantee) — preserved by §D58.5's convention-extension.
- [`m4/decisions/0028-domain-event-bus.md`](../../m4/decisions/0028-domain-event-bus.md) (M4; Template-A fire-listener pattern) — extended by §D58.4's listener-wiring conventions.

### (d) Forward-scope row

- [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md` lines 185–187](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (CH-20 row); §5 severity table line 428.

### (e) Convention-doc URLs (NEW for F1.B; ADR cross-references all 5)

- [`docs/specs/v0/conventions/persistence.md`](../../../conventions/persistence.md) — §D58.1 (schema mechanics) + §D58.3 (write verbs).
- [`docs/specs/v0/conventions/wrap-pattern.md`](../../../conventions/wrap-pattern.md) — §D58.2.
- [`docs/specs/v0/conventions/event-bus-wiring.md`](../../../conventions/event-bus-wiring.md) — §D58.4 + §D58.5.
- [`docs/specs/v0/conventions/cli-patterns.md`](../../../conventions/cli-patterns.md) — §D58.6.
- [`docs/specs/v0/conventions/web-patterns.md`](../../../conventions/web-patterns.md) — §D58.7.

### Plan archive

- [`build/ch-20-bucket-c-confirm-in-place-240616a4/plan.md`](../../../../plan/build/ch-20-bucket-c-confirm-in-place-240616a4/plan.md) — cycle hex `240616a4`; v2 plan post-gate-1 user-lock (F1.B divergent from v1 F1.A).

---

## Lifecycle history

- **2026-05-10 — CH-20 P1 (chunk-open):** ADR drafted as `Status: Proposed`; 10 sub-decisions §D58.1–§D58.10 + 5 convention-docs at NEW `v0/conventions/` peer tier shipped; F1.B 5-file split user-locked at gate-1 (DIVERGENT from planner v1 F1.A single-consolidated-file recommendation).
- **2026-05-10 — CH-20 P2 (drift-status flips):** all 16 drifts (D1.1, D1.2, D1.3, D2.1, D2.2, D3.1, D3.2, D3.3, D3.4, D4.3, D4.4, D4.5, D4.6, D6.4, D7.3, D7.6) flipped `discovered → accepted-as-is` with CH-20 lifecycle entries citing the relevant convention-doc + ADR sub-decision; `_concept-audit-matrix.md` `phi-core-mapping.md` "Session wrap" row Code-evidence cell extended; `drifts/README.md` index refreshed.
- **2026-05-10 — CH-20 P3 (chunk-seal):** ADR-0058 flipped **Proposed → Accepted**; `_cycle-index.md` row appended with status `ready-for-audit`; forward-scope row line 185 amended `(existing 14 items) → (existing 16 items)` per §D58.10; verified-headers bumped on all 24 touched docs; gate-4 sanity check confirmed `cargo test --workspace` 1529/0/2 unchanged + clippy green + 4 CI guards green + phi-core import baseline 57 preserved.
