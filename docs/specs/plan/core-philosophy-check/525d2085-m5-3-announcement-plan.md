<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 announcement — forward-scope §2.5 + drift filings + philosophy promotion

> **Plan file token:** `525d2085` (generated 2026-04-28 via `openssl rand -hex 4`).
> **Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`.

## Context

CH-21 sealed cleanly (1121 tests / 0 failed; both audits 38/38 PASS). The post-CH-21 philosophy alignment audit (`docs/specs/plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`) surfaced two foundational gaps the user has accepted as load-bearing:

- **§4.1 — Agent owns Org/Project**: no `Owns`/extended-`Created` edge from Agent to Org/Project; bootstrap claim doesn't emit it; Permission Check has no owner-grant rule.
- **§4.2 — Org/Project as Composite resources**: no `OrganizationObject` / `ProjectObject` Composite variants; admin pages 1+3 + token-economy + system flows currently encode authorization through bespoke handlers instead of routing through Permission Check.

The user picked **Option 3 — targeted M5.3 carve-out**: close M5 first (CH-23 + CH-24 in flight), then file these as drifts and run an M5.3 carve-out with two dedicated chunks before M6 starts. Re-audit after M5.3 seal; M6+ then plans on top of the unified foundation.

This plan **does not** open M5.3 chunk plans. It produces the **announcement infrastructure** so when CH-24 seals, M5.3 chunk planning has explicit governance state to operate against (drift entries + forward-scope marker + binding philosophy doc). Each M5.3 chunk (CH-25 + CH-26) gets its own plan + ADR + audit envelope at chunk-open time, following the CH-16 / CH-21 / CH-22 precedent.

**Outcome of this plan:** 10 file actions across 4 directories (1 plan-archive copy, 1 rename, 5 NEW files, 1 promotion-copy, 3 EDITs). Zero code change. After completion, CH-24 seal can proceed without M5.3 ambiguity, and M5.3 chunk-open has `Closes: D-philosophy-01` + `Closes: D-philosophy-02` lines that map to durable governance state under their own M5.3 catalogue.

---

## §1 — Scope (file actions)

| # | Action | Path | Type |
|---|---|---|---|
| 0 | **Archive** this plan verbatim | source: `/root/.claude/plans/sharded-discovering-stearns.md`<br>target: `baby-phi/docs/specs/plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md` | NEW (copy) |
| 1 | **Rename** philosophy brief in plan-tree | from: `baby-phi/docs/specs/plan/core-philosophy-check/Core Philosophy.md`<br>to: `baby-phi/docs/specs/plan/core-philosophy-check/core-philosophy.md` | RENAME |
| 2 | **Create** new M5.3 drift dir + drift file (§4.1 closure) | `baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-01.md` | NEW |
| 3 | **Create** drift file (§4.2 closure) | `baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-02.md` | NEW |
| 4 | **Create** M5.3 drift catalogue index (mirrors `m5_1/drifts/README.md` structure but scoped to M5.3 drifts) | `baby-phi/docs/specs/v0/implementation/m5_3/drifts/README.md` | NEW |
| 5 | **Create** M5.3 concept-audit matrix (mirrors `m5_1/drifts/_concept-audit-matrix.md` structure but scoped to M5.3 — primarily the new `concepts/core-philosophy.md` claims) | `baby-phi/docs/specs/v0/implementation/m5_3/drifts/_concept-audit-matrix.md` | NEW |
| 6 | **Edit** forward-scope — insert new §2.5 (M5.3 carve-out) after §2; add §4 graph edges; add §5 inventory rows | `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` | EDIT |
| 7 | **Promote** philosophy brief — copy from plan tree to concepts tree as binding source | source: `baby-phi/docs/specs/plan/core-philosophy-check/core-philosophy.md` (post-rename)<br>target: `baby-phi/docs/specs/v0/concepts/core-philosophy.md` | NEW (copy) |
| 8 | **Edit** concepts index — add `core-philosophy.md` row | `baby-phi/docs/specs/v0/concepts/README.md` | EDIT |
| 9 | **Edit** audit doc — bump cross-reference paths in `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` from `Core Philosophy.md` to `core-philosophy.md` (rename consequence) | `baby-phi/docs/specs/plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` | EDIT (light) |

10 file actions: 1 plan-archive copy, 1 rename, 5 NEW files, 1 promotion-copy, 2 EDITs (forward-scope + concepts/README), 1 light EDIT (audit-doc path fix-up). No source-tree changes (no `modules/crates/` edits). The existing `m5_1/drifts/` catalogue is **NOT touched** — M5.3 drifts live in their own dir per the user's correction.

---

## §2 — Decisions made up front (so the plan reads cleanly)

1. **Drift location: NEW `m5_3/drifts/` directory** (sibling to `m5_1/drifts/`).
   - **Rationale per user correction**: these drifts are M5.3-scoped (foundation gaps surfaced post-CH-21 audit, closed at CH-25+CH-26 in M5.3) — not M5.1 carryovers. Putting them under `m5_3/drifts/` keeps the milestone attribution honest.
   - The new dir gets its own `README.md` + `_concept-audit-matrix.md`, mirroring the `m5_1/drifts/` structure but scoped to M5.3 drifts only.
   - The existing `m5_1/drifts/` is NOT modified — its 60-drift catalogue remains an M5.1 record of M5/P2 concept-audit discoveries + M5 ledger drifts. M5.3 drifts are tracked in parallel under their own dir.
   - Cross-milestone discoverability: not addressed in this plan; if needed later, a small cross-reference at the top of `m5_1/drifts/README.md` ("See also `../m5_3/drifts/` for M5.3 drifts") can be added in a follow-up — out of scope here.

2. **Forward-scope insertion point: new §2.5 after §2, before §3**.
   - §2 covers the M5/P8 + P9 mapping (CH-21..CH-24 — already shipped or in flight).
   - §3 covers M6+ deferral markers.
   - §2.5 is the natural slot for "post-M5-close, pre-M6 carve-out work" — a foundation-only milestone that doesn't fit either existing bucket.

3. **Chunk numbering: CH-25 + CH-26** for the two M5.3 chunks (highest current = CH-24; next two free).
   - CH-25 = §4.1 (Agent-owns-Org/Project edge + Permission Check owner-rule).
   - CH-26 = §4.2 (Org/Project as Composite resources + migration to backfill instance tags + admin-handler refactor).
   - CH-26 prerequisites CH-25 (the new owner-grant rule presupposes the new edge).

4. **Philosophy doc rename + promotion**:
   - **Rename in plan-tree** (per user correction): `plan/core-philosophy-check/Core Philosophy.md` → `plan/core-philosophy-check/core-philosophy.md`. Reason: file-naming convention consistency (lowercase-hyphenated-no-spaces, matching every existing concept doc + plan archive).
   - **Promote to concept-tree**: copy renamed file to `docs/specs/v0/concepts/core-philosophy.md` as the canonical binding source.
   - Plan-tree copy stays in place as historical record (the rename is an in-place file move, not a deletion).
   - Concept-tree version gets the standard verified-by-Claude-Code header for spec-drift-guard compatibility.

5. **Plan archive (per user correction)**: a verbatim copy of this plan ships to `plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md` at execution Step 0, mirroring the chunk-plan-archive convention used in `plan/build/` (e.g., `bb95cd12-ch-21-...md`). Token generated via `openssl rand -hex 4` at execution start.

6. **No code or migration in this plan.** Code work happens at CH-25 + CH-26 chunk-open time, per separate plans following CH-16/CH-21/CH-22 discipline.

---

## §3 — Content sketches per file

### §3.1 — `m5_3/drifts/D-philosophy-01.md` (NEW)

```markdown
<!-- Last verified: 2026-04-28 by Claude Code -->

# D-philosophy-01 — Agent-as-creator-and-owner of Organization and Project not modeled

## Identification
- **ID**: D-philosophy-01
- **Phase of origin**: post-CH-21 philosophy alignment audit (2026-04-28); filed under M5.3 drift catalogue.
- **Discovery source**: `core-philosophy-audit`
- **Date discovered**: 2026-04-28
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `philosophy-gap`, `ownership-model`, `provenance-chain`, `permission-engine`
- **Blocks**: D-philosophy-02 (the unified-resource-model gap presupposes the OWNS edge); future M7 admin-page-1 (Orgs CRUD) + admin-page-3 (Projects CRUD) refactor; token-economy bid-revenue routing.
- **Blocked-by**: CH-24 M5 final seal (the M5.3 carve-out opens after M5 close).

## Concept alignment
- **Concept doc(s)**: `concepts/core-philosophy.md` (promoted from `plan/core-philosophy-check/core-philosophy.md` at this plan's seal) — claims "Agent owns Organization" and "Agent create Projects".
- **Concept claim (verbatim)**: *"An Organization as well as a Project needs to be created by an agent to maintain provenance chain. Organization also can create agents and projects who would be owned by the organization (in turn will be owned by the agent owning the organization indirectly). But agents owned by an organization may work for other orgs."* (User clarification 2026-04-28, captured in `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` §4.1.)
- **Contradiction**: Concept asserts Agent → CREATES/OWNS → Org/Project. Code has `Agent.owning_org: Option<OrgId>` (Org-owns-Agent direction) but no edge or field expressing the inverse direction. `MEMBER_OF` and `HAS_CEO` edges exist but don't carry ownership semantics. Bootstrap-claim flow at `server/src/bootstrap/claim.rs:158` creates first User+CEO+Org without an explicit ownership edge.
- **Classification**: `contradicts-concept` (foundational ownership-model gap).
- **phi-core leverage status**: `N/A` — ownership semantics are baby-phi-native governance.

## Plan vs. reality
- **Plan said**: Core Philosophy claim list, line 4: "Agent owns Organization".
- **Reality (shipped state at HEAD post-CH-21 seal)**: 
  - Edge enum at `modules/crates/domain/src/model/edges.rs` (66 variants) has no `Owns` or analogous Agent→Org/Project edge.
  - `Organization` struct at `modules/crates/domain/src/model/nodes.rs:262-334` has no `created_by_agent` / `owner_agent` field.
  - `Project` struct at `modules/crates/domain/src/model/nodes.rs:434-463` has no creator/owner field.
  - Permission Check engine at `modules/crates/domain/src/permissions/engine.rs` has no owner-grant rule (no automatic admin authority for the owner-Agent over child Org/Project).
- **Root cause**: `concept-implementation-asymmetry`. Bootstrap was modeled on User-as-platform-identity + Agent-as-org-identity without explicit Agent-creates-Org provenance modeling. Each subsequent admin handler (CH-01 disable/archive, CH-22 catalog refresh) extended the model without revisiting the gap.

## Where visible in code
- **Files**: 
  - `modules/crates/domain/src/model/edges.rs` (no `Owns` variant; 66 total).
  - `modules/crates/domain/src/model/nodes.rs:262-334` (`Organization` lacks owner field).
  - `modules/crates/domain/src/model/nodes.rs:434-463` (`Project` lacks owner field).
  - `modules/crates/server/src/bootstrap/claim.rs:158` (no ownership edge emitted at first-org creation).
- **Test evidence**: No acceptance test exercises "Agent X is the owner of Org Y → therefore X has admin authority over child Agent Z". The capability simply doesn't exist.
- **Grep for regression**: 
  - `grep -n "Owns\b\|owner_agent\|created_by_agent" modules/crates/domain/src/model/{nodes,edges}.rs` — expect 0 hits while drift open; ≥ 3 hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: 
  1. Add `Owns` edge variant (or extend `Created` with new from/to type pair) to `EdgeKind` enum.
  2. Bootstrap-claim flow + `apply_project_creation` emit the edge inside the same compound tx.
  3. Permission Check gains an "owner-grant" rule (analogous to Template A/C/D auto-issue): the owner-Agent of an Org/Project gets `[admin, transfer]` on the container.
  4. Acceptance test: owner-Agent can disable a child Agent without an explicit grant.
- **Implementation chunk**: **CH-25** (M5.3 first chunk). New ADR (likely 0042).
- **Dependencies on other drifts**: D-philosophy-02 builds on this (the unified-resource-model gap presupposes the OWNS edge so Permission Check selectors can match).
- **Estimated effort**: ~3 engineer-days inside CH-25.
- **Risk to concept alignment if deferred further**: HIGH — every M7 admin page that lands without this gap closed encodes the bespoke-handler pattern, increasing V1 refactor cost.

## Prior documentation locations
- `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` §4.1
- `plan/core-philosophy-check/core-philosophy.md` (philosophy brief, post-rename)
- `plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md` (this plan archived verbatim)
- `plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §2.5 M5.3 (added by this plan)

## Lifecycle history
- 2026-04-28 — `discovered` — surfaced by post-CH-21 philosophy alignment audit; user-confirmed as load-bearing intent; filed as drift in same session.
```

### §3.2 — `m5_3/drifts/D-philosophy-02.md` (NEW)

Same template as `D-philosophy-01.md` (with `## Identification` block, `## Concept alignment`, etc.), with these specifics:

- **ID**: D-philosophy-02
- **Title**: "Org/Project not represented in resource ontology (no Composite::OrganizationObject / Composite::ProjectObject)"
- **Severity**: HIGH
- **Bucket**: A
- **Tags**: `philosophy-gap`, `unified-resource-model`, `composite-ontology`, `permission-engine-coverage`
- **Blocks**: future M7 admin-page-1 (Orgs CRUD) + admin-page-3 (Projects CRUD) Permission-Check-routed handlers; "transfer Org ownership" capability; system-flow consistency.
- **Blocked-by**: D-philosophy-01 (the new owner-grant rule depends on the OWNS edge from CH-25).
- **Concept claim**: *"In a unified view should not Governance containers be part of Resources. That way it can be maintained and scaled using same Permission rules."* (User clarification 2026-04-28.) Plus the Core Philosophy line: *"Organization has Projects (A Resource Type)"*.
- **Contradiction**: `Composite` enum at `modules/crates/domain/src/model/composites.rs` has 8 variants (`ExternalServiceObject`, `ModelRuntimeObject`, `ControlPlaneObject`, `MemoryObject`, `SessionObject`, `AuthRequestObject`, `InboxObject`, `OutboxObject`). Org and Project are NOT among them. Permission Check selectors cannot target `org:O` or `project:P` as resource types — only as tags.
- **Where visible**: `modules/crates/domain/src/model/composites.rs` (8 Composite variants); `modules/crates/server/src/platform/orgs/*.rs` + `platform/projects/*.rs` (handlers don't route through Permission Check engine).
- **Approach (sketch)**: 
  1. Add `Composite::OrganizationObject` + `Composite::ProjectObject` with `constituents()` (Organization → IdentityPrincipal + DataObject + Tag; Project → DataObject + Tag).
  2. Migration to backfill instance-identity tags on existing Organization + Project rows (cf CH-06 pattern).
  3. Refactor Page-1 + Page-3 admin handlers to call Permission Check (replacing bespoke gates).
  4. New ADR (likely 0043) ratifying the unified resource model.
- **Implementation chunk**: **CH-26** (M5.3 second chunk).
- **Estimated effort**: ~3-4 engineer-days inside CH-26.

### §3.3 — `m5_3/drifts/README.md` (NEW)

Mirrors the structure of [`m5_1/drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) but scoped to M5.3 drifts only. Sketch:

```markdown
<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 Drift catalogue — index

This directory tracks drifts surfaced **post-M5-close** that close at M5.3 (carve-out between CH-24 final M5 seal and M6 plan-open). It is the **single source of truth for M5.3 drifts**, parallel to (and discoverable alongside) [`../../m5_1/drifts/README.md`](../../m5_1/drifts/README.md) which tracks M5.1+M5.2 drifts.

**Status (as of 2026-04-28 / M5.3 catalogue open):**
- Drifts open: **2** (both HIGH, Bucket A, source = post-CH-21 philosophy alignment audit).
- Drifts remediated: **0** (CH-25 + CH-26 will close them when M5.3 chunks seal).

## Severity + Bucket summary

| Severity | Bucket | Count | IDs |
|---|---|---|---|
| HIGH | A (load-bearing scope gap) | 2 | D-philosophy-01, D-philosophy-02 |

## Lifecycle states

Same lifecycle as `m5_1/drifts/` per [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md): `discovered → classified → scoped → in-chunk-plan → remediated`.

## Drift table

| ID | Title | Severity | Bucket | Concept doc(s) | phi-core leverage | Status | Closes at | File |
|---|---|---|---|---|---|---|---|---|
| D-philosophy-01 | Agent-as-creator-and-owner of Org/Project not modeled (no OWNS edge, no creator field, no owner-grant rule) | **HIGH** | **A** | core-philosophy; agent; organization; project | N/A | discovered | CH-25 | [D-philosophy-01.md](D-philosophy-01.md) |
| D-philosophy-02 | Org/Project not in resource ontology (no Composite::OrganizationObject / ProjectObject) | **HIGH** | **A** | core-philosophy; ontology; permissions/01 | N/A | discovered | CH-26 | [D-philosophy-02.md](D-philosophy-02.md) |

## Companion files

- [`_concept-audit-matrix.md`](_concept-audit-matrix.md) — claim-by-claim audit of `concepts/core-philosophy.md` against current code (the binding source for M5.3 drifts).
- Drift bodies follow `../../m5_1/drifts/_schema.md` structure.

## Cross-reference

- Philosophy alignment audit + clarifications: [`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md).
- Forward-scope M5.3 carve-out: [`../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5.
- M5.3 announcement plan archive: [`../../../../plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md`](../../../../plan/core-philosophy-check/) (this plan, archived verbatim).
```

### §3.4 — `m5_3/drifts/_concept-audit-matrix.md` (NEW)

Mirrors [`m5_1/drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) but scoped to M5.3-relevant claims. Primarily covers `concepts/core-philosophy.md` (the newly-promoted binding source). Sketch:

```markdown
<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 Concept-audit matrix

Claim-by-claim audit of M5.3-scoped concept docs against current code. Built post-CH-21 from the philosophy alignment audit ([`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md)).

The M5.1 + M5.2 concept-audit matrix at [`../../m5_1/drifts/_concept-audit-matrix.md`](../../m5_1/drifts/_concept-audit-matrix.md) tracks claims under `agent.md`, `organization.md`, `project.md`, `permissions/*`, etc. — those are NOT duplicated here. This matrix tracks ONLY the new `concepts/core-philosophy.md` (binding source promoted at this plan's seal).

## Status legend

- `honored` — concept claim implemented in code, with test evidence.
- `partially-honored` — claim partially expressed; specific carve-out documented.
- `silent-in-code` — claim not yet implemented; deferred via successor marker.
- `contradicted` — claim contradicted by code; covering drift filed.

## `concepts/core-philosophy.md`

| § | Claim | Status | Code evidence | Covering drift |
|---|---|---|---|---|
| Two Types of Agents | `AgentKind::{Human, Llm}` enum + role-validity rules | honored | `nodes.rs:225-228` + `nodes.rs:284-289` | — |
| Agent owns Organization | Agent creates Org/Project; transitive ownership through Org-owned children | contradicted | no `Owns` edge; no creator field on Organization / Project; bootstrap-claim doesn't emit ownership edge | **D-philosophy-01** (HIGH, CH-25) |
| Organization has Resources | `OWNED_BY` / `BELONGS_TO` edges + Org-scoped resource catalogue | honored | `edges.rs` + `composites_m4::ResourceBoundaries` | — |
| Two Types of Resources (Fundamental + Composite) | 9 Fundamental + 8 Composite variants with `constituents()` codified | honored | `fundamentals.rs` (9 variants) + `composites.rs:150-174` (`constituents()`) | — |
| Resources have actions | Open string vocabulary per `permissions/03-action-vocabulary.md`; inheritance via composite expansion | honored | `permissions/manifest.rs:38` (`actions: Vec<String>`) + `composites.rs:150` | — |
| Org has Projects (Project = Resource Type) | Project is a Composite resource subject to Permission Check selectors | contradicted | `Composite` enum has 8 variants; no `ProjectObject` / `OrganizationObject` entries; Page-1+Page-3 handlers bypass Permission Check engine | **D-philosophy-02** (HIGH, CH-26) |
| Sub-orgs + Sub-projects | `HasSuborganization` + `HasSubproject` edges | honored | `edges.rs:244` + `edges.rs:275` | — |
| Project shared between Orgs | Shape-B project + `BelongsTo` N:N edge | honored | `edges.rs:289` + `nodes.rs:510-516` (`ProjectShape::B`) | — |
| Org has Agents (Ownership) | `MEMBER_OF` edge + `Agent.owning_org` | honored | `edges.rs:136` + `nodes.rs:194` | — |
| Agent spawn other Agents | `DELEGATES_TO` edge + audit-event capture of creator-actor | honored (creator-foreign-key on node deliberately omitted per philosophy §4.4 clarification — access-control over creator-tracking) | `edges.rs:100` + `agents/create.rs` audit emit | — |
| Agent create Projects | `apply_project_creation` actor recorded in audit; CREATED edge emitted | honored (with §4.1 closure adding Agent→Project Owns edge) | `apply_project_creation` audit + post-CH-25 `Owns` edge | — (post-D-philosophy-01) |
| Agents own Resources | `OWNED_BY` edge + `Grant.holder` | honored | `edges.rs` + `nodes.rs:624-639` | — |
| Agents work on several Projects/Orgs | N:N `MEMBER_OF` + `HAS_AGENT` edges | honored | `edges.rs:136,254` | — |
| Resources can be Transferred + co-owned | `transfer` action + `AllocatedTo` edge | honored | `permissions/03-action-vocabulary.md` + `edges.rs:334` | — |
| Every Resource has a creator (Provenance) | `CREATED` edge variant + `Edge::new_created` constructor | honored | `edges.rs:329-333` + `edges.rs:597-601` | — |
| Permission Tuple (Subject, Action, Resource, Constraints, Provenance) | Grant carries 4-of-5 directly; Constraints sourced from Manifest at call-time (acceptable per philosophy §4.3 clarification) | honored-with-design-note | `nodes.rs:624-639` + `permissions/engine.rs` Step 4 | — |
| Capability semantics (Permission Check) | 6+2a-stage algorithm: catalogue → expand → resolve grants → ceiling → match → scope → constraints → consent | honored | `permissions/engine.rs:38-121` | — |
| Session shared ownership (Org + Project + Agent) | `Session.owning_org` + `owning_project` + `started_by` + frozen tags + Templates A-D auto-issue grants | honored | `nodes.rs:998-1018` + `permissions/05-memory-sessions.md:218-290` | — |
| Memory shared ownership (private vs Org/Project, inherited from Session) | Tag-based inheritance via `session:{id}` / `org:{id}` / `project:{id}` derived in CH-21 listener; binary `{private, public}` bucket on `extraction_scope_distribution`; 4-pool routing deferred to M6-DEFERRED-04 | partially-honored | CH-21 / ADR-0040 §D40.2 + `events/listeners.rs` `build_memory_tags` | — (M6-DEFERRED-04 owns the 4-pool upgrade) |

## Cross-reference

- M5.1+M5.2 matrix: [`../../m5_1/drifts/_concept-audit-matrix.md`](../../m5_1/drifts/_concept-audit-matrix.md) — claims under `agent.md`, `organization.md`, `project.md`, `permissions/*`.
- Drift schema: [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md).
- Philosophy alignment audit: [`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md).
```

### §3.5 — `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` (EDIT — three sub-edits)

**Sub-edit A: Insert new §2.5 after line 223** (current `No M5 P8/P9 commitments orphaned.` line) and before line 225 (the `---` separator that starts §3):

```markdown
---

## §2.5 — M5.3 carve-out — philosophy alignment foundation work

> **Purpose**: post-CH-21 philosophy alignment audit (2026-04-28) surfaced two foundational gaps the user has accepted as load-bearing. M5.3 is a dedicated carve-out closing those gaps **between M5 final seal (CH-24) and M6 start** so that M6+ chunks plan against a unified ownership-and-resource model. Gaps deferred to V1 would compound into ~8–15 engineer-days of refactor at M7+; M5.3 closes them in ~5–7 engineer-days now.
>
> **Audit + clarification record**: [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../core-philosophy-check/2026-04-28-philosophy-alignment-audit.md). User clarifications applied at §4.1 + §4.2.
> **Binding philosophy source**: [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) (promoted from `plan/core-philosophy-check/core-philosophy.md` at this plan's seal).
> **M5.3 drift catalogue**: [`v0/implementation/m5_3/drifts/`](../../v0/implementation/m5_3/drifts/) — D-philosophy-01 + D-philosophy-02 with own README + concept-audit matrix. Parallel to (not under) `m5_1/drifts/`.
> **Plan archive**: [`plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md`](../core-philosophy-check/) — verbatim copy of this announcement plan.

### CH-25 — Agent-as-creator-and-owner of Org/Project (philosophy §4.1)

- **Drifts closed**: **D-philosophy-01** (HIGH).
- **Concept docs**: `core-philosophy.md` ("Agent owns Organization", "Agent create Projects"); `agent.md` ("Agent spawns other Agents (Ownership)").
- **Prerequisites**: **CH-24** (M5 final seal). Independent of CH-23.
- **Deliverables**: 
  - New `Owns` edge variant in `EdgeKind` enum (or extended `Created` with Agent→Org/Project type pairs).
  - Bootstrap-claim flow + `apply_project_creation` compound tx emit the edge.
  - Permission Check gains an "owner-grant" rule (auto-issue `[admin, transfer]` for the owner-Agent over the child Org/Project).
  - Acceptance: owner-Agent can disable a child Agent without an explicit grant.
  - New ADR (next-free at chunk-open; likely **ADR-0042**).
- **Estimated effort**: ~3 engineer-days.

### CH-26 — Org/Project as Composite resources (philosophy §4.2)

- **Drifts closed**: **D-philosophy-02** (HIGH).
- **Concept docs**: `core-philosophy.md` ("Organization has Projects (A Resource Type)"); `permissions/01-resource-ontology.md` (extends Composite list).
- **Prerequisites**: **CH-25** (the unified-resource-model rule presupposes the new owner edge).
- **Deliverables**: 
  - `Composite::OrganizationObject` + `Composite::ProjectObject` variants with `constituents()`.
  - Migration to backfill instance-identity tags on existing Organization + Project rows (cf CH-06 + CH-16 migration patterns).
  - Refactor admin-page-1 (Orgs) + admin-page-3 (Projects) handlers to route through Permission Check engine instead of bespoke gates.
  - Acceptance: Permission Check matches `org:O` and `project:P` as resource types in selectors.
  - New ADR (next-free; likely **ADR-0043**).
- **Estimated effort**: ~3-4 engineer-days.

### Post-M5.3 actions

- Re-run philosophy alignment audit (same 3-Explore-agent shape as 2026-04-28). Goal: confirm 12 → 14 ALIGNED claims, surface any third-order ripple effects.
- M6 plan-mode opens against the now-unified foundation.
- Smaller philosophy follow-ups (§4.5 `Memory.source_session_id`, §4.6 concept-`agent.md` field cleanup) route to their natural homes (M6-DEFERRED-04, next docs sweep) — NOT in M5.3.

No M5.3 commitments orphaned.
```

**Sub-edit B: §4 Chunk dependency graph** (currently around lines 295–337 per the Explore report). Add the new edges:

```
                    └─> CH-24  (M5 final seal)
                          │
                          v
                    M5.3 carve-out
                    └─> CH-25 ──> CH-26 ──> M6 plan
                                              │
                                              v
                                       M6-DEFERRED-01..04
```

Specifically: add 2 new lines to the existing graph showing `CH-24 → CH-25 → CH-26 → M6 plan-open`. Preserves existing structure.

**Sub-edit C: §5 per-chunk scope summary table** (currently around lines 340–370). Add 2 rows after CH-24:

```markdown
| CH-25 | Agent-owns-Org/Project edge + owner-grant rule | HIGH | 3d | core-philosophy / agent / permissions/04 | — | yes |
| CH-26 | Org/Project as Composite resources (unified resource model) | HIGH | 3-4d | core-philosophy / ontology / permissions/01 | — | yes |
```

### §3.6 — `concepts/core-philosophy.md` (NEW — copy + header)

Copy the verbatim content from `plan/core-philosophy-check/core-philosophy.md` (post-rename, 1876 bytes — the 18-claim brief). Prepend a verified-by-Claude-Code header conforming to the convention:

```markdown
<!-- Status: BINDING SPEC SOURCE (promoted 2026-04-28 from `plan/core-philosophy-check/core-philosophy.md` per post-CH-21 audit; see `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` for the audit that ratified the promotion). -->
<!-- Last verified: 2026-04-28 by Claude Code (initial promotion; D-philosophy-01 + D-philosophy-02 (under `m5_3/drifts/`) file the two HIGH drifts the audit surfaced; M5.3 carve-out per `plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §2.5 closes them at CH-25 + CH-26.) -->

# Baby-Phi Core Philosophy

[... 18-claim brief, verbatim from the source file ...]
```

The post-rename file at `plan/core-philosophy-check/core-philosophy.md` stays in place as a historical record of how the brief originally landed (do NOT delete). The concept-tree copy becomes the canonical binding source.

### §3.7 — `concepts/README.md` (EDIT)

Add a new row to the index table at line 8–19 (alphabetical by name; goes near the top under "Agent" or as the FIRST conceptual row since philosophy is foundational):

```markdown
| **Core Philosophy** | [core-philosophy.md](core-philosophy.md) | The 18-claim source-of-truth philosophy brief that all concept docs and code align against. Promoted 2026-04-28 from the audit at `plan/core-philosophy-check/`. |
```

Position: **first row** of the table (lines 8 onwards), before "Agent". Rationale: philosophy is the meta-concept that the others derive from.

### §3.8 — `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` (EDIT — light)

The audit doc currently references the source brief by its old name. Two cross-reference touches:

1. Update the §1 (Methodology) line that reads:
   > **Source of truth (philosophy)**: `Core Philosophy.md` — 18 claims spanning ...

   to:
   > **Source of truth (philosophy)**: `core-philosophy.md` (post-rename) / [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) (binding source post-promotion) — 18 claims spanning ...

2. Update the opening blockquote that says:
   > A point-in-time alignment check between [`Core Philosophy.md`](Core%20Philosophy.md) (user-supplied 18-claim brief) ...

   to:
   > A point-in-time alignment check between [`core-philosophy.md`](core-philosophy.md) (user-supplied 18-claim brief; renamed from `Core Philosophy.md` 2026-04-28; promoted to [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) at the M5.3 announcement-plan seal) ...

3. Add a new §10 — Provenance bullet pointing at the M5.3 announcement plan archive (`<8hex>-m5-3-announcement-plan.md`) so the audit doc → plan archive link is bidirectional.

No other edits to the audit doc. Its findings + clarifications + scoring stay frozen as the historical record of what was decided 2026-04-28.

---

## §4 — Verification recipe (after edits land)

This plan is documentation-only, so verification is doc-link integrity + spec-drift guard + grep coherence.

```bash
cd /root/projects/phi/baby-phi

# 1. Doc-link integrity (every relative link in the new + edited files resolves)
bash scripts/check-doc-links.sh
# Expect: PASS (links resolve from §2.5 → core-philosophy.md, drift files → matrix, m5_3 README → audit doc).

# 2. Operator-doc header convention
bash scripts/check-ops-doc-headers.sh
# Expect: PASS — no ops doc edited by this plan, but run as discipline.

# 3. Spec-drift guard
bash scripts/check-spec-drift.sh
# Expect: PASS — no requirement-id changes; new concept doc adds, doesn't remove.

# 4. phi-core reuse guard
bash scripts/check-phi-core-reuse.sh
# Expect: PASS — no code change.

# 5. New M5.3 drift files exist + scan as valid markdown
ls baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-0{1,2}.md
head -25 baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-01.md
# Expect: file exists; "## Identification" + Status: discovered + Severity: HIGH visible.

# 6. M5.3 drift catalogue index + matrix
ls baby-phi/docs/specs/v0/implementation/m5_3/drifts/{README,_concept-audit-matrix}.md
head -30 baby-phi/docs/specs/v0/implementation/m5_3/drifts/README.md
# Expect: both files exist; README shows the 2-row drift table.

# 7. Forward-scope §2.5 visible + chunk references
grep -n "## §2.5 — M5.3 carve-out" baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md
grep -n "CH-25\|CH-26" baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md
# Expect: §2.5 header ≥ 1 hit; CH-25/CH-26 each ≥ 2 hits (in §2.5 + §4 graph + §5 inventory).

# 8. Concept-tree promotion + index
ls baby-phi/docs/specs/v0/concepts/core-philosophy.md
grep -n "core-philosophy.md" baby-phi/docs/specs/v0/concepts/README.md
# Expect: file exists; index row references it.

# 9. Plan-tree rename (post-rename) — no spaces in filename
ls "baby-phi/docs/specs/plan/core-philosophy-check/core-philosophy.md"
ls "baby-phi/docs/specs/plan/core-philosophy-check/Core Philosophy.md" 2>&1 | grep -c "No such file"
# Expect: post-rename file exists; old-name lookup returns "No such file" (= 1 from the grep -c).

# 10. Plan archive (this plan, copied verbatim)
ls baby-phi/docs/specs/plan/core-philosophy-check/*-m5-3-announcement-plan.md
head -3 baby-phi/docs/specs/plan/core-philosophy-check/*-m5-3-announcement-plan.md
# Expect: 1 file matching `<8hex>-m5-3-announcement-plan.md`; header line 1 matches the verified-by-Claude-Code convention.

# 11. Cross-references intact (drift IDs appear in: drift bodies + matrix + forward-scope §2.5 + audit doc)
grep -rn "D-philosophy-01\|D-philosophy-02" baby-phi/docs/specs/v0/implementation/m5_3/drifts/
grep -n "D-philosophy-0[12]" baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md
# Expect: each grep returns multiple hits.

# 12. m5_1/drifts/ untouched (sanity)
git diff --stat HEAD -- baby-phi/docs/specs/v0/implementation/m5_1/
# Expect: empty (no changes to m5_1 tree).

# 13. (Optional) Re-read the audit doc to confirm the implications routed through this plan
head -50 baby-phi/docs/specs/plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md
```

No cargo invocations needed. No tests to run.

---

## §5 — Out of scope (intentionally deferred)

This plan does NOT:

- **Open M5.3 chunk plans** (CH-25 + CH-26). Each gets its own plan via the standard chunk-template at chunk-open time.
- **Write code or migrations**. CH-25 + CH-26 land the code work.
- **File §4.5 / §4.6 follow-ups** as drifts. Per the audit doc §4.5 routes to M6-DEFERRED-04 (alongside the LLM supervisor body); §4.6 routes to the next docs sweep. Neither needs a drift file at this point — they are tracked in the audit doc and the post-M5.3 re-audit will re-evaluate.
- **Promote any other concept doc** to a binding-spec status. Only `core-philosophy.md` graduates here.
- **Update the M3/M5 ledger** beyond what the drift README requires. Ledger touches are minimal and contained.
- **Re-run the philosophy alignment audit**. The post-M5.3 re-audit happens after CH-25 + CH-26 seal, NOT in this plan.
- **Touch existing M5/M5.2 drift files** (D-new-XX, DX.Y). Their statuses are independent and remain untouched.

---

## §6 — Critical files for implementation

**Renamed file:**
- `baby-phi/docs/specs/plan/core-philosophy-check/Core Philosophy.md` → `baby-phi/docs/specs/plan/core-philosophy-check/core-philosophy.md` (in-place rename; preserves audit-doc cross-references after the §3.8 light edit).

**New files (5 plus 1 plan-archive copy):**
- `baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-01.md`
- `baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-philosophy-02.md`
- `baby-phi/docs/specs/v0/implementation/m5_3/drifts/README.md` (M5.3 drift catalogue index)
- `baby-phi/docs/specs/v0/implementation/m5_3/drifts/_concept-audit-matrix.md` (M5.3 concept-audit matrix)
- `baby-phi/docs/specs/v0/concepts/core-philosophy.md` (copy of `plan/core-philosophy-check/core-philosophy.md` + header)
- `baby-phi/docs/specs/plan/core-philosophy-check/<8hex>-m5-3-announcement-plan.md` (verbatim copy of this plan, archived per user correction)

**Edited files:**
- `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` (new §2.5; §4 graph edges; §5 inventory rows)
- `baby-phi/docs/specs/v0/concepts/README.md` (1 new row — `core-philosophy.md` as the first index entry)
- `baby-phi/docs/specs/plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` (light: 2-3 cross-reference path fix-ups + 1 §10 provenance bullet)

**Untouched (per user correction — m5_1 stays clean):**
- `baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md` (M5.1+M5.2 catalogue, 60 entries; M5.3 drifts live in their own dir).
- `baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` (M5.1+M5.2 matrix; M5.3 has its own).

**Reused (no edit) — for context only:**
- `baby-phi/docs/specs/v0/implementation/m5_1/drifts/_schema.md` (drift-file structure spec — the new D-philosophy-NN files conform to it).

---

## §7 — Estimated effort

~2 hours of focused doc work:
- 5 min — Step 0: generate token + archive plan verbatim to `plan/core-philosophy-check/`.
- 5 min — rename `Core Philosophy.md` → `core-philosophy.md`.
- 30 min — drafting `D-philosophy-01.md` + `D-philosophy-02.md` content + cross-checking line refs.
- 20 min — drafting `m5_3/drifts/README.md` + `_concept-audit-matrix.md` (mirror m5_1 structure, scoped to M5.3).
- 25 min — editing forward-scope §2.5 + §4 graph + §5 inventory.
- 10 min — concept-tree promotion (copy + header) + concepts/README index row.
- 10 min — audit-doc light edits (§3.8): fix `Core Philosophy.md` → `core-philosophy.md` cross-refs + add §10 provenance bullet.
- 15 min — verification recipe pass (13 checks) + spot-fixes.
