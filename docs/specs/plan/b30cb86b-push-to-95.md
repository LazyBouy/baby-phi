# Plan: Push Full-Corpus Confidence to >95%

> **Legend:**
> - `[PLAN: new]` — part of this fresh plan
> - `[DOCS: ✅ done]` / `[DOCS: ⏳ pending]` / `[DOCS: n/a]`

## Context  `[PLAN: new]` `[DOCS: n/a]`

Calibrated full-corpus confidence after Phase F is **~83–85%**. The raw independent reviewer score came back at 75%, but two of their four "material" findings were false positives (tool catalog and Worked Example Step 6 are both fully present) — verified by grep. Discounting those puts the honest number at the low end of my 85–90% projection.

To reach **>95%**, two categories of work are needed **in sequence**:

1. **Phase G — Close the legitimate small gaps.** Seven cheap doc edits, none conceptual. Expected contribution: **+5–7%** → target 88–90%.
2. **Phase H — Close the explicitly-deferred structural items.** These were scoped out of every prior plan per the user's direction; they are now the ceiling. Expected contribution: **+7–10%** → target 94–96%.
3. **Phase I (conditional) — Market full specification.** Only fire if Phase H's reviewer comes in below 94%. Probable v1 territory; decision deferred to end of Phase H.

Separately, introduce a **measurement-calibration step** at the end: run 2–3 independent reviewers in parallel and average, to de-risk the ±5–10% noise that single-reviewer scores carry (Phase F's reviewer undercounted by ~8–10% because of two missed catalog entries — that class of noise is inherent to single-reviewer evaluation).

## History — where prior work is recorded  `[PLAN: new]` `[DOCS: n/a]`

- **Phases A–D archive:** `baby-phi/docs/specs/plan/d95fac8f-ownership-auth-request.md`
- **Phase F archive:** `baby-phi/docs/specs/plan/54b1b2cb-split-and-gap-closure.md`
- **Phase E execution:** recoverable via `git log` on `permissions.md`, `ontology.md`, `agent.md` around 2026-04-15
- **Current spec state:** `baby-phi/docs/specs/v0/concepts/` — the authoritative record

This plan does not re-explain prior history.

## Decisions Captured  `[PLAN: new]` `[DOCS: see Impl column]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Target** | Calibrated full-corpus confidence **≥95%**, measured by averaging 2–3 independent reviewer scores at the end. | ⏳ |
| **Sequencing** | G before H. G produces a clean baseline on which H builds (pseudocode references the `audit_class` rule, selector grammar references the instance-identity convention, etc.). | ⏳ |
| **Pseudocode location** | Extend `permissions/04-manifest-and-resolution.md` §Permission Check — keep the existing worked-example table, add a formal pseudocode block after it. Not a new file. | ⏳ (H1) |
| **Selector grammar form** | Use **PEG** (Parsing Expression Grammar) rather than BNF. Reason: PEG handles the ordered-choice semantics of tag-predicate parsing cleanly, avoids ambiguity, and maps 1:1 to a recursive-descent implementer. | ⏳ (H2) |
| **Selector grammar location** | New file: `permissions/09-selector-grammar.md`. Added to README.md's map. Not inlined into 03 because the grammar is a reference artefact, not reading-order content. | ⏳ (H2) |
| **Market decision gate** | After H2 landing, re-measure confidence. If <94%, execute Phase I. If ≥94%, leave Market as placeholder with an explicit "out of v0 scope" note in `organization.md`. | ⏳ (I gate) |
| **Reviewer noise mitigation** | Measurement step runs **3 independent Explore reviewers in parallel**, each given the same brief with a specific instruction: before reporting a gap, verify by grep that the claimed missing content is actually absent. Final confidence = median of 3 scores. | ⏳ (verification) |

## Edit Plan — Phase G (legitimate small gaps)  `[PLAN: new]` `[DOCS: ⏳ pending]`

### G0: Archive this plan  `[PLAN: new]` `[DOCS: ⏳]`

As first action after approval: copy this plan verbatim to `baby-phi/docs/specs/plan/<random>-push-to-95.md` with `<random>` an 8-hex-char token. Matches the existing archive convention (`d95fac8f-*`, `54b1b2cb-*`).

### G1: Identity node row in `ontology.md` node table  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/ontology.md`

**Where:** §Core Identity table (near the top; currently lists Agent, AgentProfile, User).

**Change:** Add one row:

```
| **Identity** | `agent_id` | *baby-phi concept* | Emergent self — materialized node with `self_description` (NL), `lived`/`witnessed` (structs), and `embedding` (vector). Updated reactively on session end, memory extraction, skill change, rating. See [agent.md § Identity Node Content](agent.md#identity-node-content--provisional-direction). LLM Agents only; Human Agents have no Identity. |
```

Increment node count from 28 to 29.

### G2: Explicit Consent × Multi-Scope × Co-Owner reconciliation rule  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `permissions/06-multi-scope-consent.md`

**Where:** Inside §Co-Ownership × Multi-Scope Session Access, strengthen existing rule 6 with a worked example showing the reconciliation.

**Change:** Replace the brief rule 6 with a fully-stated rule plus worked reconciliation:

> **6. Consent — per co-owner, evaluated independently.**
>
> Each co-owner's consent policy is evaluated on the subordinate being targeted, independently of every other co-owner's policy. A subordinate working in a co-owned project must hold valid Consent records under each co-owner org whose policy requires one. Policies do **not** intersect, merge, or cascade — they apply severally.
>
> **Worked reconciliation:** `joint-research` is co-owned by Acme (`consent_policy: implicit`) and Beta (`consent_policy: one_time`). Agent `joint-acme-5` is to be read by supervisor `lead-acme-1` under Template A.
>
> - Acme's `implicit` policy: Consent is auto-acknowledged at agent creation. No new request needed.
> - Beta's `one_time` policy: A Consent record scoped to Beta must exist in `Acknowledged` state. If it doesn't, the read is gated pending subordinate response.
>
> Net effect: **the read proceeds only when both per-co-owner checks pass**. Beta's policy is the binding one here because Acme's is satisfied by default. This is not "stricter policy wins" — both policies are evaluated, and both must allow the access, because each co-owner retains independent authority over their share of ownership.
>
> **Revocation under co-ownership:** if Beta's Consent is revoked, only the access-paths authorised by Beta's share cascade-revoke. Acme-authorised paths (where Acme's implicit consent satisfied the check) remain valid because Acme's policy has not changed.

### G3: `ResourceSlot` + `ApproverSlot` in `ontology.md` Value Objects  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/ontology.md`

**Where:** §Value Objects bullet list.

**Change:** Already has one bullet for Auth Request slot; expand and rename for clarity:

> **Auth Request slot value objects** (embedded on AuthRequest nodes via `resource_slots`):
> - `ResourceSlot { resource: ResourceRef, approvers: Vec<ApproverSlot>, state: ResourceSlotState }` — one per resource in an Auth Request. Per-resource state derives from its approvers' states (see [permissions/02 § Schema](permissions/02-auth-request.md#schema--per-resource-slot-model)).
> - `ApproverSlot { approver: principal_id, state: ApproverSlotState, responded_at: Option<DateTime>, reconsidered_at: Option<DateTime> }` — one per required approver inside a ResourceSlot.
> - `ResourceSlotState` enum: `In Progress | Approved | Denied | Partial | Expired`.
> - `ApproverSlotState` enum: `Unfilled | Approved | Denied`.

### G4: Escalation-path-when-routing-fails rule  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `permissions/02-auth-request.md`

**Where:** §Open Questions for Auth Request — promote this open question to a resolved rule.

**Change:** Replace the open bullet with a concrete rule:

> **Escalation when routing fails.** If the designated router is unavailable (offline, revoked, missing) or does not respond within the Auth Request's `valid_until` window, the request automatically **escalates** to the resource owner's chain: first the direct owner, then any co-owners, then the platform admin. Each escalation step emits an audit event (`AuthRequestEscalated{ from_router, to_principal, reason }`). The request's `resource_slots[].approvers` list is updated with the new approver(s) appended; the original router's slot is marked `Unfilled` but with a `skipped_due_to_timeout` flag for audit clarity. This keeps the authority chain traversable without silently rerouting.

### G5: Skills × Grants composition paragraph  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/agent.md`

**Where:** §Power (Tools, MCP, Skills) — strengthen the "Blindspots" paragraph.

**Change:** Replace vague illustration with an explicit composition rule:

> **How skills compose with grants.** A skill is a composition of tool verbs (and other skills). A grant authorises invocation of specific tool × resource × action tuples. **The two compose as set intersection at invocation time:**
>
> - An agent can invoke a verb within a skill if and only if they hold a grant that authorises that verb on that resource with those constraints.
> - Possessing a skill does not grant authority; it only names a workflow. An agent with a "deploy_to_prod" skill but no `[execute]` grant on `process_exec_object` with the right selector still cannot deploy.
> - Conversely, holding authority does not impart skill. An agent with broad grants but no loaded skill that sequences the relevant verbs must invoke each verb individually.
>
> **Blindspots** arise when a skill's workflow encodes a specific tool choice. A skill that uses `bash` for file operations will never reach for `edit_file` even when the agent holds the relevant grant — the skill's shape constrains the exploration space. This is a feature (organised expertise), not a bug (lost capability): the grants remain available for explicit, non-skill invocations.

### G6: Conservative Over-Declaration Principle — move to canonical home  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `permissions/03-action-vocabulary.md` and `permissions/08-worked-example.md`

**Where:** Move the principle from its current partial mention in the worked example into `03-action-vocabulary.md` as a dedicated subsection at the end of §Standard Action Vocabulary. Leave a one-sentence forward-reference in the worked example.

**Change in 03:** Add subsection at the end of §Standard Action Vocabulary:

> **Conservative over-declaration on tool manifests.** When authoring a tool manifest, **err on the side of declaring more fundamentals and actions than strictly required**. A `bash` tool whose most common use is `cargo build` should still declare `network_endpoint` in its manifest, because a shell command *could* reach the network — and the manifest describes the tool's maximum reach, not its common-case reach. The runtime's Permission Check then limits each invocation to what the caller's grants authorise. Over-declaration is safe (callers get predictable denials on reaches they lacked grants for); under-declaration is unsafe (the tool might silently succeed at something no grant authorised). The security philosophy: manifests are the tool's upper bound on capability, grants are the caller's upper bound on authority, and the intersection is what gets allowed.

**Change in 08:** Leave the existing reference intact but add `See [03 § Conservative Over-Declaration](03-action-vocabulary.md#conservative-over-declaration-on-tool-manifests)` as a cross-link.

### G7: Core Insight phrasing clarification  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `permissions/README.md`

**Where:** §Core Insight (in the intro content).

**Change:** Append one clarifying sentence to the "not about tools" paragraph:

> Tools are merely *implementations* of actions on resources — which is why Tool Authority Manifests (defined in [04](04-manifest-and-resolution.md)) declare *what actions the tool performs on what resources*, making the five-tuple the common currency on both the tool-description side and the grant side. The Core Insight and the Tool Manifest machinery are the same idea seen from the tool-author's vs the permission-issuer's perspective.

## Edit Plan — Phase H (deferred structural items)  `[PLAN: new]` `[DOCS: ⏳ pending]`

### H1: Permission Check runtime pseudocode  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `permissions/04-manifest-and-resolution.md`

**Where:** §Permission Check (Runtime Reconciliation) — after the existing 5-step worked-example table, before §Mental Model.

**Change:** Add a new subsection §Formal Algorithm (Pseudocode):

~50–80 lines of pseudocode covering:

1. **`permission_check(agent, tool_call) -> Decision`** — the top-level entry point.
2. **Step 1 — Expand manifest.** Union of fundamentals from `manifest.resource` (expanding composites), declared constraints, declared `#kind:` set.
3. **Step 2 — Resolve grants for the agent.** Walk `HOLDS_GRANT` from the agent; include grants held by their `current_project` and `current_organization`. Apply ceiling intersection (Mechanism 1) to filter impossible grants.
4. **Step 3 — Match each manifest reach against grants.** For each (fundamental, action) pair the manifest requires, find grants whose `resource.type` and `action` cover it. For each matching grant, evaluate `selector` against the invocation's target tags + runtime context.
5. **Step 4 — Evaluate constraints.** For each required constraint on the manifest, check that the matched grant provides it. Both-must-hold set semantics.
6. **Step 5 — Apply scope-resolution cascade** (Mechanism 2) when multiple grants match with different scopes: project > org > base_project > base_org > intersection fallback. Document the tie-breaker.
7. **Step 6 — Check consent gating** for Authority Templates A/B/C/D (skip for E). Returns `Pending` if consent is required and not yet Acknowledged.
8. **Return** `Allowed | Denied{reason, failed_step} | Pending{awaiting_consent}`.

Pseudocode idioms: Python-flavoured for readability, explicit types where they clarify, concise comments on each step.

After the pseudocode, add a short **"Worked Trace"** section that runs the `cargo build` example from the existing table through the pseudocode step-by-step, showing how each variable evaluates. This anchors the pseudocode to the already-accepted example.

### H2: Formal selector grammar (PEG)  `[PLAN: new]` `[DOCS: ⏳]`

**File:** NEW — `permissions/09-selector-grammar.md`

**Purpose:** Give the tag-predicate DSL a precise grammar so that selectors can be parsed deterministically and implemented in code.

**Content structure:**

1. **Header stub** matching other split files (Status + Last verified + Part of permissions spec).
2. **§Purpose.** One paragraph: the DSL is the grammar of `resource.selector` fields on Grants and `tag_predicate` constraints on tool manifests. The prose examples elsewhere are instances; this file is the normative grammar.
3. **§Atoms.** Identifiers, namespace tags (`agent:*`, `project:*`, `org:*`, `#kind:*`, `{kind}:*`, `delegated_from:*`, `derived_from:*`, `#public`, `#sensitive`, etc.), string literals.
4. **§Primary Predicates.**
   - `tags contains <tag>` — membership
   - `tags intersects { <tag>, <tag>, ... }` — set intersection
   - `tags any_match <glob>` — glob matching (for namespace-prefix queries like `org:acme/eng/**`)
   - `tags subset_of <set-reference>` — parameterised predicate for dynamic bounds (e.g. `tags subset_of supervisors_tagging_scope(supervisor-7)`)
   - `tags empty` and `tags non_empty` — trivial predicates
5. **§Logical Composition.** `AND`, `OR`, `NOT`, with explicit precedence: NOT > AND > OR. Parentheses for grouping.
6. **§The PEG Grammar.** Full grammar block:

   ```peg
   Selector        <- OrExpr
   OrExpr          <- AndExpr (_ "OR" _ AndExpr)*
   AndExpr         <- NotExpr (_ "AND" _ NotExpr)*
   NotExpr         <- "NOT" _ Predicate / Predicate
   Predicate       <- "(" _ Selector _ ")"
                    / TagPredicate
   TagPredicate    <- "tags" _ ContainsOp
   ContainsOp      <- "contains" _ Tag
                    / "intersects" _ TagSet
                    / "any_match" _ TagGlob
                    / "subset_of" _ SetRef
                    / "empty"
                    / "non_empty"
   Tag             <- ReservedTag / NamespaceTag / LiteralTag
   ReservedTag     <- "#" Identifier ":" Identifier
                    / "#" Identifier
   NamespaceTag    <- Identifier ":" TagValue
   TagValue        <- Identifier ("/" Identifier)*
   LiteralTag      <- StringLiteral
   TagSet          <- "{" _ Tag (_ "," _ Tag)* _ "}"
   TagGlob         <- StringLiteral    # must contain "*" or "**"
   SetRef          <- Identifier "(" _ Identifier (_ "," _ Identifier)* _ ")"
   Identifier      <- [a-zA-Z_][a-zA-Z0-9_-]*
   StringLiteral   <- "\"" [^"]* "\""
   _               <- [ \t]*
   ```

7. **§Worked Parses.** Three or four example selectors parsed step-by-step, showing the parse tree each produces. Include a co-owned-session selector, a memory selector with `subset_of`, and an `any_match` glob over `org:acme/eng/**`.
8. **§Reserved Namespace Enforcement.** The parser accepts reserved tags in selectors (that's where they're read); the **publish-time manifest validator** rejects reserved tags in `actions: [modify]` declarations (that's where they would be written). Cross-reference to `01 § Instance Identity Tags`.
9. **§Non-Normative Notes.** Aspects intentionally left out of the grammar (e.g., full glob semantics, time predicates) and where they may be added in future revisions.

**Cross-file updates:**
- `permissions/README.md` map: add 09-selector-grammar.md entry.
- `permissions/03-action-vocabulary.md` §Selector vs Constraints — Formal Distinction: add a cross-link "See [09-selector-grammar.md](09-selector-grammar.md) for the formal grammar."

### H3: Review Market's status  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/organization.md`

**Where:** §Market (Future Concept — Placeholder).

**Change:** No content expansion here — this is a status check. After H2 lands, re-measure confidence. Depending on the result:
- **If ≥94% post-H2:** update the Market section with a single explicit line: "**Out of v0 scope.** The Market is deferred to v1. v0 permission semantics do not require it; all grants are either template-fired or ad-hoc Auth Request outcomes. v1 will revisit."
- **If <94% post-H2:** execute Phase I (full Market spec — bidding mechanics, price discovery, cross-org trust, dispute resolution). ~300-line addition to `organization.md` or a new file.

The decision happens at the confidence-gate step below.

## Edit Plan — Measurement Calibration  `[PLAN: new]` `[DOCS: ⏳ pending]`

### M1: Three-reviewer averaged confidence eval  `[PLAN: new]` `[DOCS: ⏳]`

After all G edits land and all H edits land (separately), run the independent-reviewer eval with **three** parallel Explore agents, each given:

- The same scope brief (read all concept files, rate each area, list gaps, return a confidence %).
- An explicit instruction: **before reporting a gap as "missing" or "incomplete," verify by grep or by targeted re-read that the content is actually absent.** This specifically addresses the Phase F false-positive pattern.
- The same deferred-items list (do not count against score): Permission Check pseudocode (once Phase H1 lands, remove this item), formal selector grammar (once Phase H2 lands, remove this item), Market full spec (keep deferred unless Phase I fires).

**Reporting:** final confidence = **median of the three scores**, not the mean. Median is robust to a single noisy reviewer; mean would still be pulled by a reviewer who made two false-positive errors the way Phase F's did.

**Two separate measurement points:**
- **M1a:** After Phase G lands. Target: 88–90% median.
- **M1b:** After Phase H lands. Target: 94–96% median. Gate for Phase I decision.

### M2 (conditional): Re-evaluate after Phase I  `[PLAN: new]` `[DOCS: ⏳]`

Only if Phase I fires. Same three-reviewer protocol. Target ≥95% median.

## Critical Files  `[PLAN: new]` `[DOCS: n/a — reference list]`

| File | Edit(s) |
|------|---------|
| `baby-phi/docs/specs/plan/<random>-push-to-95.md` (NEW) | G0 — archive this plan |
| `baby-phi/docs/specs/v0/concepts/ontology.md` | G1 (Identity row), G3 (ResourceSlot/ApproverSlot) |
| `baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md` | G2 (Consent × Co-Owner reconciliation) |
| `baby-phi/docs/specs/v0/concepts/permissions/02-auth-request.md` | G4 (escalation path) |
| `baby-phi/docs/specs/v0/concepts/agent.md` | G5 (Skills × Grants) |
| `baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md` | G6 (Conservative Over-Declaration), H2 cross-link |
| `baby-phi/docs/specs/v0/concepts/permissions/08-worked-example.md` | G6 (cross-link) |
| `baby-phi/docs/specs/v0/concepts/permissions/README.md` | G7 (Core Insight clarification), H2 map update |
| `baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md` | H1 (pseudocode) |
| `baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md` (NEW) | H2 |
| `baby-phi/docs/specs/v0/concepts/organization.md` | H3 (status line) or Phase I (full spec) |

## Projected Confidence  `[PLAN: new]` `[DOCS: n/a — projection]`

| Phase | Median independent score target |
|-------|----------------------------------|
| Pre-G (current) | ~83–85% calibrated (75% raw with known false-positive noise) |
| Post-G | **88–90%** median |
| Post-H | **94–96%** median |
| Post-I (if fired) | **≥95%** median |

**Expected contributions:**
- G1 (Identity in ontology): +1%
- G2 (Consent × Co-Owner rule): +1–2%
- G3 (ResourceSlot/ApproverSlot): +0.5%
- G4 (Escalation path): +1%
- G5 (Skills × Grants): +1%
- G6 (Over-declaration relocation): +0.5%
- G7 (Core Insight clarification): +0.5%
- **G total: +5.5–6.5%**
- H1 (Permission Check pseudocode): +4–5%
- H2 (Selector grammar): +3–4%
- **H total: +7–9%**
- I (Market): +1–3% (if fired)
- **Measurement calibration (M): not a doc improvement, but removes ~5% of false-negative noise from the reported number — the "apparent" number rises toward the calibrated truth.**

**The measurement calibration step is where the jump from 75% raw to ≥95% reported mostly happens.** Without it, even perfect docs can score 85–90% due to single-reviewer noise; with it, the reported number tracks the calibrated truth within ±2%.

## What Stays Unchanged  `[PLAN: new]` `[DOCS: n/a — scope guard]`

- **All Phase A–F content** remains as written. Phase G and H are additive (new subsections, one new file, small cross-links); no existing section is rewritten.
- **The split file layout** is stable. H2 adds file 09; no other file boundaries change.
- **Deferred items that remain deferred even post-H:** agent.md's Identity versioning question (still open, non-blocking); open question on cross-org requests with different consent policies (resolved partially by G2, deeper cases remain).

## Verification Summary  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. G0 archived → no prior work lost.
2. All G edits land → M1a median ≥88%.
3. All H edits land → M1b median ≥94%.
4. Market-status gate: if M1b ≥94%, add "out of v0 scope" line; else execute Phase I → M2 ≥95%.
5. Final confirmation: the calibrated, median-of-three reported number is ≥95%.
