<!-- Last verified: 2026-04-28 by Claude Code (initial draft; user-clarified scoring applied to the 5 partials and 1 drift surfaced in the audit). -->

# Core Philosophy Alignment Audit — 2026-04-28

> **What this is.** A point-in-time alignment check between [`core-philosophy.md`](core-philosophy.md) (user-supplied 18-claim brief; renamed from `Core Philosophy.md` 2026-04-28; promoted to [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) at the M5.3 announcement-plan seal) and the current state of baby-phi (concept docs + code at HEAD = post-CH-21 seal). Three independent Explore agents covered (A) Agent + ownership, (B) Resource model + hierarchy, (C) Permissions + Session/Memory ownership. The user reviewed the resulting findings and clarified intent on five open questions; this document captures the **post-clarification scoring** plus the implications for future chunks / ADRs.

> **What this is not.** A plan or chunk approval. Items flagged here become candidates for forward-scope or ADRs at the user's discretion — they are not committed work.

---

## §1 — Methodology

- **Source of truth (philosophy)**: [`core-philosophy.md`](core-philosophy.md) (post-rename) / [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) (binding source post-promotion) — 18 claims spanning agent kinds, ownership, resource ontology, hierarchy, permission tuple, and session/memory ownership.
- **Audit dimensions**:
  - **A. Agent + ownership** (7 claims): agent kinds; Agent-owns-Org; Org-has-Agents; Agent-spawns-Agents; Agent-creates-Projects; sub-orgs/sub-projects; multi-org Project.
  - **B. Resource model + hierarchy** (7 claims): Fundamental + Composite; Resource actions; Project as Resource Type; sub-orgs / sub-projects; cross-org / cross-project sharing; transfer + co-ownership; creator provenance.
  - **C. Permissions + Session/Memory** (4 claims): Grant 5-tuple shape; Permission Check semantics; Session shared ownership; Memory shared ownership.
- **Method**: each axis got a parallel Explore agent with claim-by-claim instructions to score `ALIGNED / PARTIAL / GAP / DRIFT`, quote the concept passage, quote the code surface (file:line), and flag contradictions.
- **Read-only**: no code or doc edits made by the audit agents.
- **Then** the user clarified intent on the five open questions emerging from the synthesis. **This document folds those clarifications back into the scoring.**

---

## §2 — Top-line scoreboard (post-clarification)

| Axis | Aligned | Partial | Gap | Acceptable carve-out |
|---|---|---|---|---|
| **A. Agent + ownership** | 4 | 1 | 2 | 0 |
| **B. Resource model + hierarchy** | 6 | 0 | 1 | 0 |
| **C. Permissions + Session/Memory** | 2 | 1 | 0 | 1 |
| **Total** | **12 / 18** | **2 / 18** | **3 / 18** | **1** |

The scoreboard shifts after the user's clarifications: what the raw audit recorded as one DRIFT and five PARTIALs becomes 3 GAPs (where code lacks a load-bearing concept), 2 PARTIALs (concept-doc edits would close them), and 1 acceptable carve-out (deferred per ADR-0040).

---

## §3 — Aligned claims (12 of 18, brief acknowledgement)

These are honored in code with concept-doc backing — no follow-up needed:

1. **Two agent types** — `AgentKind::{Human, Llm}` with role-validity rules. ([nodes.rs:225](../../../../modules/crates/domain/src/model/nodes.rs#L225))
2. **Org has Agents** — `MEMBER_OF` edge + `Agent.owning_org`. ([edges.rs:136](../../../../modules/crates/domain/src/model/edges.rs#L136))
3. **Sub-orgs + sub-projects** — `HasSuborganization` + `HasSubproject` edges, no parent-id fields. ([edges.rs:244,275](../../../../modules/crates/domain/src/model/edges.rs#L244))
4. **Multi-org Project (Shape B)** — `BelongsTo` N:N + `ProjectShape::B` + two-approver Auth Request flow.
5. **Resources have actions** — open string vocabulary per [`permissions/03-action-vocabulary.md`](../../v0/concepts/permissions/03-action-vocabulary.md), inheritance via `Composite::constituents()`.
6. **Two resource types** — 9 Fundamentals + 8 Composites with `constituents()` codified, including the `Tag` fundamental in every composite.
7. **Resource transfer + co-ownership** — `transfer` action + `AllocatedTo` edge supports multi-allocator co-ownership.
8. **Provenance is structural** — `Grant.descends_from: Option<AuthRequestId>`, `AuthRequest.provenance_template`, BLAKE3 per-org hash chain.
9. **Permission Check 6+2a-stage algorithm** — `domain::permissions::engine::check` has Step 0–6 + 2a present and ordered.
10. **Session shared ownership** — type-safe fields (`owning_org`/`owning_project`/`started_by`) + frozen tags + Templates A–D issuing grants over those tags at runtime.
11. **Agent-private memory** — emerges from set disjointness (no `agent:Y` overlap), not special-case logic.
12. **Resource creation provenance via `CREATED` edge** — `Edge::new_created(creator, resource)` constructor; every node-creation handler can emit it.

---

## §4 — Findings re-scored after user clarification

### 4.1 — "Agent owns Organization" — re-scored as **GAP** (not DRIFT)

**Original audit reading.** The audit flagged DRIFT because `Agent.owning_org: Option<OrgId>` and `MEMBER_OF` express the inverse direction (Org-owns-Agent), with no field/edge for Agent-owns-Org. The audit's interpretation was *"philosophy doc typo"*.

**User clarification (2026-04-28, verbatim paraphrase).**

> *"This is not a typo. An Organization as well as a Project needs to be created by an agent to maintain provenance chain. Organization also can create agents and projects who would be owned by the organization (in turn will be owned by the agent owning the organization indirectly). But agents owned by an organization may work for other orgs."*

**What this means in concrete model terms.**

- **Agent → CREATES → Org** is a real, intended edge — the creator-Agent **owns** the Org for governance and audit purposes.
- **Org → OWNS → child Agents / Projects / Resources** stays as is (current model).
- **Transitive ownership**: creator-Agent owns Org → Org owns child Agents → ∴ creator-Agent **indirectly** owns the child Agents. This is a chain, not a duplicated field.
- **Multi-org work**: an Agent owned-by-Org-A may have `MEMBER_OF` edges into Org-B and work on Org-B's projects. Ownership ≠ work assignment.

**Where the gap lives in code today.**

- No `OWNS` edge variant in the EdgeKind enum (66 variants per [edges.rs](../../../../modules/crates/domain/src/model/edges.rs)).
- No `Agent.owns_orgs: Vec<OrgId>` or analogous field.
- No `Organization.created_by_agent: AgentId` field.
- `bootstrap/claim.rs` creates `User → HAS_CEO → Agent` and the Org separately; the "first Agent created the Org" relationship is implicit, not modeled.
- `HAS_CEO` ≠ `OWNS` — a future CEO change should not transfer ownership; the original-creator relationship should persist.

**Re-scoring.** **GAP** — load-bearing concept, code does not express it.

**Forward-scope candidate (not committed).**
- New ADR formalizing **Agent-as-creator-of-Org-and-Project** semantics.
- New `OWNS` edge variant (or extend `CREATED` to load-bear ownership): `Agent → OWNS → Organization` and `Agent → OWNS → Project`.
- Bootstrap claim flow gains the `OWNS` edge on first-org-creation.
- Permission Check gains "owner of containing Org/Project bypasses normal grant resolution" rule (or an explicit owner-grant template).
- Drift entry filed for the gap.

### 4.2 — "Project is a Resource Type" — re-scored as **GAP** (not PARTIAL)

**Original audit reading.** Projects are first-class governance nodes (with `BelongsTo` edges + grants targetable at resource selectors over their tags), but Project does NOT appear in the `Fundamental` (9-variant) or `Composite` (8-variant) ontology. The audit scored PARTIAL.

**User clarification.**

> *"In a unified view should not Governance containers be part of Resources. That way it can be maintained and scaled using same Permission rules. For example, how else would you answer the question who grants other agents permission to a project or organization?"*

**What this means.**

The user picks the **unified-resource-model** path: governance containers (Project, Organization) should be Composite resources subject to the same Permission Check selectors as `MemoryObject`, `SessionObject`, etc. This makes the question *"who can `[invite, modify, archive]` Org X?"* answerable through the same grant/manifest/permission-check chain.

**Where the gap lives in code today.**

- `Composite` enum at [composites.rs](../../../../modules/crates/domain/src/model/composites.rs) has 8 variants; **no `OrganizationObject` or `ProjectObject` entries**.
- Permission Check is invoked with `target_kind: Composite` (or a fundamental); a request to *"check whether agent A can `[invite]` org O"* has no canonical resource type.
- Operational consequence today: Org/Project membership changes go through bespoke handlers (e.g., `set_agent_active`, `apply_project_creation`) rather than a generic *"check `[modify]` on `org:O`"* path. Provenance audit works (audit events fire) but the same control-plane reasoning that governs `MemoryObject` writes does not govern Org/Project writes.

**Re-scoring.** **GAP** — concept (now ratified by user) intends a unified resource model; code has 8 composites and would need (at least) 2 more.

**Forward-scope candidate (not committed).**
- ADR adding `Composite::OrganizationObject` + `Composite::ProjectObject` with their `constituents()` (likely include `IdentityPrincipal` + `DataObject` + `Tag` for orgs; `DataObject` + `Tag` for projects).
- Migration to add resource-tagging on existing Org/Project rows so selectors can match.
- Auth Request templates for Org/Project lifecycle actions (`invite`, `archive`, `transfer`).
- Net effect: Page-1 (Orgs admin) and Page-3 (Projects admin) collapse into the existing Permission Check spine instead of carrying bespoke handlers.

### 4.3 — `Grant.constraints` field missing — re-scored as **PARTIAL → ACCEPTABLE** (not load-bearing)

**Original audit reading.** Permission tuple is 5-component per philosophy (Subject/Action/Resource/Constraints/Provenance). The Grant struct has 4 of them as fields plus Subject via the `HOLDS_GRANT` edge; **no `constraints` field**. Constraints are checked at Permission Check Step 4 from the **Manifest**, not from the Grant. The audit scored PARTIAL with a note that "Grant cannot express its own constraints".

**User clarification.**

> *"As long as there is provision for constraints this should be fine. For example file read permission can be constrained to specific folder. Memory read can be constrained to specific tags or groups etc."*

**Re-scoring.** **ACCEPTABLE** — the manifest-level constraint provision satisfies the philosophy. The implementation expresses constraints; they live on the Manifest (call-time), not on the Grant (issue-time). The two examples the user named (folder-scoped file read, tag-scoped memory read) both work today via Manifest constraints + Grant's `resource.uri` selector.

**Optional follow-up (not required).**
- Light ADR formalizing the manifest-only design as a deliberate v0 choice (vs. a HIGH drift). Documents *why* Grants don't carry constraints — keeps the audit-vs-runtime split clean.
- Concept-doc clarification at [`permissions/04-manifest-and-resolution.md:550-566`](../../v0/concepts/permissions/04-manifest-and-resolution.md#L550) that the "5-tuple" framing is logical, with one component (constraints) sourced from the manifest at call-time rather than from the Grant directly.

### 4.4 — Creator-foreign-key on nodes — **DROPPED** (not a finding)

**Original audit reading.** No `created_by_agent` field on Agent / Project / Memory; provenance only via `CREATED` edges and audit events.

**User clarification.**

> *"It is not important who created it, rather who can access the resources. For example a memory created must be either private or for project (more than one agent in an org can access it) or public (any agent in org can access it)."*

**Re-scoring.** **NOT A FINDING.** The philosophy is access-control-first, not creator-first. The `CREATED` edge handles the audit-trail / provenance need; daily operations rely on tag-based access control (which is honored). The audit's framing of "asymmetry with `Grant.descends_from`" was over-stated.

> Note: this clarification subtly **strengthens** §4.5's scoring — Memory tag-based access control IS the model; the absence of `source_session_id` is by design, not a gap.

### 4.5 — Memory ownership lineage (`source_session_id`) — re-scored as **PARTIAL** (small enhancement, optional)

**Original audit reading.** Memory struct ([nodes.rs:827-832](../../../../modules/crates/domain/src/model/nodes.rs#L827)) is `{id, owning_agent, tags, created_at}` — no `source_session_id`. CH-21 / ADR-0040 §D40.2 derives the `session:{id}` tag at extraction time, so the relationship is encoded in tags. The audit noted this is opaque at the schema level (queries route through tag matches).

**User clarification.**

> *"One major way to extract memory would be from sessions. There could be other ways as well. Therefore it would make sense to refer to the session from which it is extracted, though it may not be a mandatory field."*

**Re-scoring.** **PARTIAL — small optional enhancement.** An optional `source_session_id: Option<SessionId>` would make the most-common extraction path navigable as a foreign key without forcing every Memory to have a Session lineage (e.g., user-typed memories, future bulk imports). The current tag-based encoding stays valid; the optional field is additive.

**Forward-scope candidate (low priority, not committed).**
- Add `Memory.source_session_id: Option<SessionId>` (with `#[serde(default)]` for back-compat).
- CH-21 listener populates it alongside the existing `session:{id}` tag.
- Schema migration adds the column (optional, nullable).
- No change to permission semantics — tag-based access control still rules.

### 4.6 — Multi-project / multi-org Agent context — **PARTIAL** (concept-doc edit closes it)

**Status unchanged from raw audit (user did not specifically address this one).**

The concept doc [`agent.md:107-116`](../../v0/concepts/agent.md#L107) references `current_project`, `base_project`, `current_organization`, `base_organization` as Agent-level fields. The Agent struct has none of these — they are session-scoped context per the [`baby-phi/CLAUDE.md`](../../../../CLAUDE.md) "ID-only at session-launch" rule.

**Recommended close-out.** Concept-doc edit on `agent.md` to clarify these are session-context values, not persisted Agent fields. Goes into the next docs sweep — no code change needed; it's pure documentation drift.

---

## §5 — New gaps surfaced by §4.1 clarification

The "Agent owns Org" clarification implies two ownership-chain mechanics not previously enumerated:

### 5.1 — Agent → CREATES / OWNS → Organization (and Project) edges

Today's bootstrap flow ([`server/src/bootstrap/claim.rs`](../../../../modules/crates/server/src/bootstrap/claim.rs)) creates a User and a CEO Agent, then creates an Organization, **without an explicit "Agent created Org" edge**. The implicit creator information is recoverable from the audit log (the actor on `platform.org.created`) but is not part of the graph.

For a unified ownership-and-provenance model:
- New edge variant: `Created { from: AgentId, to: OrgId }` (or expand the existing `Created` to include this pair).
- Or a dedicated `Owns` edge variant: `Owns { from: AgentId, to: { Organization | Project } }`.
- Bootstrap-claim flow + `apply_project_creation` emit the edge inside the same compound tx.
- Permission Check's owner-grant rule consults this edge.

### 5.2 — Indirect ownership via Org → child Agents

The user's clarification implies that an Agent that owns an Org indirectly owns the Org's child Agents (the Agents the Org spawns). This is a **transitive** relationship, not a duplicated field.

Concrete operational consequence: a question like *"can Agent X disable Agent Y?"* should answer YES if X owns the Org that owns Y, and Y has no other owner-relationship blocking. Today this is a bespoke check (CEO/admin handlers); under the unified model it's a Permission Check on `[disable]` against `agent:Y`.

This sub-gap closes naturally when §5.1 lands, plus when §4.2 (Org/Project as resources) lands.

---

## §6 — Acceptable carve-out (1 of 18)

**4-pool memory routing (`agent:` / `project:` / `org:` / `#public`)** — explicitly deferred to **M6-DEFERRED-04** in [ADR-0040 §D40.7](../../v0/implementation/m5_2/decisions/0040-memory-extraction-listener-heuristic-v0.md). CH-21 collapses to binary `{private, public}` at v0 by design. Marked `silent-in-code` in the [concept-audit matrix](../../v0/implementation/m5_1/drifts/_concept-audit-matrix.md). Not a drift — a documented carry-forward with a successor marker.

---

## §7 — Cross-cutting observations

1. **Graph-first design is consistent.** Sub-orgs/sub-projects are edges, not parent_id fields. Creator is a `CREATED` edge, not a `created_by` column. This is a coherent design philosophy. The two new gaps (§4.1 and §4.2) extend this same philosophy to cover Org/Project ownership semantics — they are additions, not contradictions.

2. **Type-safety-vs-tag-based ownership is split.** Sessions are dual (struct fields + tags); Memory is tag-only (with §4.5's optional enhancement); Grants have structural Provenance but tag-based scope. After the user's §4.4 clarification (access-control over creator-tracking), this split is **deliberate** and acceptable.

3. **Auth-Request-and-Provenance is the strongest part of the implementation against the philosophy.** The chain `Grant.descends_from → AuthRequest.provenance_template → Template` is unbroken. Audits A and C both flag this.

4. **Two of the three `GAP` items would naturally bundle into one chunk.** §4.1 (Agent-owns-Org/Project) and §4.2 (Org/Project as Composite resources) share the same root: a unified ownership-and-resource model where governance containers are first-class resources with explicit creator/owner edges. Closing one without the other leaves the chain half-built.

5. **The `_concept-audit-matrix.md` does not currently surface §4.1, §4.2, or §4.5.** If the philosophy doc graduates to a binding source-of-truth (alongside the existing concept docs), each of these would become drift entries with explicit successor chunks. Today they are findings in this document; tomorrow they could be drift files.

---

## §8 — Net read

baby-phi is **substantially aligned** with the core philosophy — 12 of 18 claims fully honored, the graph-first design is internally consistent, and the Permission Check engine + Auth Request provenance chain are the strongest single piece of the implementation.

After applying the user's clarifications, the remaining gaps reduce to **two foundational additions** (Agent-owns-Org/Project edge + Org/Project-as-Composite resources — §4.1 + §4.2) and **two small enhancements** (concept-doc clarification on session-scoped agent fields — §4.6; optional `Memory.source_session_id` — §4.5). The Grant.constraints concern (§4.3) is closed as acceptable; the creator-foreign-key concern (§4.4) is dropped.

The two foundational additions belong together in a single design effort because they share the same root — *governance containers as first-class resources with explicit ownership edges*. This is the natural shape of a future ADR / chunk pair that would close the remaining philosophy alignment.

---

## §9 — Open implications (for user discretion, not committed)

1. **File the foundational gaps as drifts?** — §4.1 + §4.2 could become a paired drift file (`D-philosophy-01: Agent-as-creator-and-owner of Organization and Project under unified resource model`) or live as findings here until a future planning cycle picks them up.
2. **Promote `Core Philosophy.md` to a binding source?** — currently it's a user brief; promoting it would put it on the same footing as concept docs and trigger drift-tracking against it.
3. **Concept-doc sweep?** — §4.6 is pure doc drift on `agent.md`; could fold into the next planned docs review.
4. **CH-21 successor `Memory.source_session_id`?** — §4.5 is a low-priority enhancement; could land in M6-DEFERRED-04 alongside the LLM-driven supervisor body, or sooner as a small standalone chunk.

---

## §10 — Provenance

- Source philosophy doc: [`core-philosophy.md`](core-philosophy.md) (user-supplied 2026-04-28; renamed from `Core Philosophy.md` 2026-04-28; promoted to binding source at [`v0/concepts/core-philosophy.md`](../../v0/concepts/core-philosophy.md) at the M5.3 announcement-plan seal).
- Audit run: 2026-04-28, three parallel Explore agents (A/B/C).
- User clarifications: 2026-04-28 (folded into §4 scoring).
- HEAD at audit time: post-CH-21 seal (1121 tests / 0 failed across 102 groups; ADR-0040 + ADR-0041 Accepted; drift D6.1 remediated).
- This document: written by Claude Code at user request to capture the audit + clarifications in a durable form.
- M5.3 announcement plan archive (verbatim): [`m5-3-announcement-plan-525d2085.md`](m5-3-announcement-plan-525d2085.md) — files D-philosophy-01 + D-philosophy-02 in [`v0/implementation/m5_3/drifts/`](../../v0/implementation/m5_3/drifts/), promotes this audit's source-of-truth philosophy to the concept tree, and inserts §2.5 in the [forward-scope file](../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) with CH-25 + CH-26 closing the two HIGH drifts.
