# Plan: Split `permissions.md` + Close Corpus-Wide Gaps

> **Legend for header annotations:**
> - `[PLAN: new]` — section is part of this fresh plan
> - `[DOCS: ✅ implemented]` — executed
> - `[DOCS: ⏳ pending]` — not yet executed (plan mode)
> - `[DOCS: n/a]` — reference/meta section; nothing to implement

## Context  `[PLAN: new]` `[DOCS: n/a]`

Prior work on the Resource Ownership + Auth Request Model (Phases A–E) is complete and verified. The permissions/ownership/auth-request subsystem scored **10 of 11 concept areas coherent** in an independent review. That workstream is done.

What remains is **two pieces of orthogonal work** that the same reviewer flagged when scoring the *full corpus* rather than just the permissions subsystem:

1. **`permissions.md` has grown to 3986 lines** and is no longer scannable. It needs to be split into a folder of focused files so that future edits (and future reads) land on sensibly-sized documents.
2. **Seven ancillary gaps** in other concept files — `coordination.md`'s 8 open design questions, `token-economy.md`'s unresolved `Value` window, `ontology.md`'s missing Schema Registry extension permission, absent YAML worked examples for Templates C/D/E, a missing multi-composite tool manifest example, and one concurrency clarification on Witnessed Experience — bring the *full-corpus* confidence down to 73% even though the subsystem we built is at ~95%.

This plan closes both pieces. The split lands first (F1–F4) because the subsequent gap-closure edits are cleaner on smaller files with settled boundaries.

## History — where prior work is recorded  `[PLAN: new]` `[DOCS: n/a]`

- **Phases A–D full detail:** archived at `baby-phi/docs/specs/plan/d95fac8f-ownership-auth-request.md` (86 KB, 1101 lines, verbatim snapshot as of Phase D start).
- **Phase E execution (TransferRecord demotion, `transfer` action, embedding model scope, `audit_class` composition rule):** recoverable from `git log` / `git diff` on `permissions.md`, `ontology.md`, `agent.md` around 2026-04-15.
- **Current spec state:** `baby-phi/docs/specs/v0/concepts/` — the authoritative record of the built system. Read the files there to see what the model actually is; read the archive above to see how the design decisions got there.

This plan does **not** re-explain any of that history. It starts from the present state of the docs and describes only what to change next.

## Decisions Captured  `[PLAN: new]` `[DOCS: see Impl column]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Split target** | `baby-phi/docs/specs/v0/concepts/permissions/` folder replaces the single file. Nine files: `README.md` index + eight thematic content files (`01-`..`08-` numbered prefixes to reflect reading order). | ⏳ pending (F1) |
| **Old file disposition** | Delete `permissions.md` after cross-file references are rewritten. No stub — a stub would rot as the split evolves. | ⏳ pending (F4) |
| **Anchor preservation** | Keep existing `### Heading` anchors; intra-file anchors remain `#anchor`, cross-file become `filename.md#anchor`. | ⏳ pending (F2) |
| **coordination.md policy** | Replace the 8 bare open questions with **tentative v0 answers**, each flagged "v0 default, revisitable." Implementers get a concrete starting point, not a placeholder. | ⏳ pending (F8) |
| **Value-field window (token-economy)** | Reuse the **same rolling-window mechanism as ratings** (window=20, oldest collapses into a running average). No new mechanism. | ⏳ pending (F9) |
| **Schema Registry extension permission** | Creating a new node or edge type requires `[allocate]` on `control_plane_object` **plus** a template-adoption-style Auth Request. | ⏳ pending (F10) |
| **Witnessed Experience update cadence** | **Reactive per-extraction** (matches the existing "updated reactively" language). No batch mode. | ⏳ pending (F11) |
| **Templates C/D/E YAML examples** | Add full worked YAML for all three, parallel to the existing A and B examples. C = org-chart-edge-triggered; D = project-role-edge-triggered; E = explicit manual. | ⏳ pending (F6) |
| **Multi-composite tool example** | Add as tool #14 (`workspace_snapshot`): reads from `filesystem_object + memory_object + session_object` in one invocation. Demonstrates union of fundamentals and `#kind:` values. | ⏳ pending (F7) |
| **Co-Ownership × Multi-Scope worked example** | Add concrete bash-allowed-vs-forbidden scenario to cement the intersection-on-ceilings / union-on-grants distinction. | ⏳ pending (F5) |

## Split Design  `[PLAN: new]` `[DOCS: ⏳ pending]`

Current `permissions.md` (3986 lines) → `permissions/` folder with 9 files:

| New file | Content (current line range) | Approx lines |
|----------|------------------------------|--------------|
| `permissions/README.md` | Landing page: TOC, Core Insight, Canonical Shape, The Five Components (1–85) + map of what-lives-where | ~120 |
| `permissions/01-resource-ontology.md` | Resource Ontology — Two Tiers, Resource Ownership, Composite Identity Tags, Composite Creation Checklist (86–945) | ~860 |
| `permissions/02-auth-request.md` | Auth Request Lifecycle (entire) (258–744) | ~490 |
| `permissions/03-action-vocabulary.md` | Standard Action Vocabulary, `allocate` umbrella, Constraints, Selector vs Constraints, Per-Resource-Class Reference (947–1469) | ~525 |
| `permissions/04-manifest-and-resolution.md` | Two Shapes, Permission Check, Permission Resolution Hierarchy, Authority Chain, Grant as Graph Node (1471–1924) | ~455 |
| `permissions/05-memory-sessions.md` | Memory as Resource Class, Sessions as Tagged Resource, Authority Templates (A–E definitions), Specific Considerations for Sessions (1925–2495) | ~570 |
| `permissions/06-multi-scope-consent.md` | Multi-Scope Session Access (incl Co-Ownership interaction), Consent Policy (incl lifecycle), Open Questions for Session Permissions (2496–2906) | ~415 |
| `permissions/07-templates-and-tools.md` | Standard Permission Templates (with full A–E YAML), `audit_class` composition, Tool Authority Manifest Examples (13 → 14 tools), Authoring Guide (2908–3565) | ~660 |
| `permissions/08-worked-example.md` | Worked End-to-End Use Case, phi-core Extension Points, top-level Open Questions (3566–3986) | ~425 |

**File boundaries follow natural concept groupings:**

- 01 = the static structure of resources (ontology + ownership + identity)
- 02 = the Auth Request composite (big enough to own its own file, self-contained)
- 03 = the action language (vocabulary + constraints + per-class reference)
- 04 = how grants are expressed and checked (manifest vs grant, permission check, resolution hierarchy, authority chain)
- 05 = domain specializations (memory and sessions, sharing Authority Template machinery)
- 06 = cross-cutting session rules (multi-scope, consent)
- 07 = starter kits (standard templates A–E, tool examples, authoring guide)
- 08 = the big tabletop walkthrough + phi-core integration + remaining open questions

## Edit Plan  `[PLAN: new]` `[DOCS: ⏳ pending]`

### F0: Archive the pre-overwrite plan state  `[PLAN: new]` `[DOCS: ⏳ pending]`

Before any other action, capture the plan file state as it existed **just before this plan was written** — that state contained Phase E's execution record and Phase F's draft, which are not yet archived. Since the plan file has now been overwritten, the recovery path is:

- The Phase A–D archive (`d95fac8f-ownership-auth-request.md`) is preserved and untouched.
- Phase E execution detail is recoverable from `git log` / `git diff` on the three concept files touched (`permissions.md`, `ontology.md`, `agent.md`).
- The Phase F draft (content of this plan) is **this file itself**; once execution begins, archive a verbatim copy of this plan as `baby-phi/docs/specs/plan/<random>-split-and-gap-closure.md` with `<random>` an 8-hex-char token, following the same convention as the d95fac8f archive.

F0 is executed as the **first action after plan approval**, before F1.

### F1: Create `permissions/` folder and 9 new files  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. Create directory `baby-phi/docs/specs/v0/concepts/permissions/`.
2. Create each file from the current `permissions.md` using the line-range map above. Each file starts with:
   ```
   <!-- Status: CONCEPTUAL -->
   <!-- Last verified: 2026-04-15 by Claude Code -->
   <!-- Part of the permissions spec — see README.md for the full map -->
   ```
3. `README.md` additionally carries the full TOC and a "what lives where" map.

### F2: Rewrite intra-permissions links  `[PLAN: new]` `[DOCS: ⏳ pending]`

Every `[Text](#anchor)` link inside the old permissions.md that now points to a section in a *different* split file becomes `[Text](XX-filename.md#anchor)`. Same-file anchors stay as `#anchor`.

Strategy: grep each split file for `](#`; for each hit, determine whether the anchor lives in the same file (keep) or another file (rewrite with relative path). ~50–80 link rewrites expected.

### F3: Rewrite external links from other concept files  `[PLAN: new]` `[DOCS: ⏳ pending]`

20 references to `permissions.md` across 7 files (ontology.md: 10, agent.md: 3, organization.md: 3, project.md: 1, coordination.md: 1, human-agent.md: 1, README.md: 1). Each is rewritten to the specific split file + anchor:

- `[permissions.md → Consent Policy](permissions.md#consent-policy-organizational)` → `[permissions → Consent Policy](permissions/06-multi-scope-consent.md#consent-policy-organizational)`
- `[permissions.md → Resource Ownership](permissions.md#resource-ownership)` → `[permissions → Resource Ownership](permissions/01-resource-ontology.md#resource-ownership)`

Keep a mapping table in-commit for future traceability.

### F4: Delete `permissions.md`  `[PLAN: new]` `[DOCS: ⏳ pending]`

After F1–F3 land and F12's grep verification passes, delete the monolith. Pre-delete check: `grep -r "concepts/permissions.md" baby-phi/` returns zero hits.

### F5: Co-Ownership × Multi-Scope worked example  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `permissions/06-multi-scope-consent.md`

**Where:** Inside §Co-Ownership × Multi-Scope Session Access, after the 6 rules.

**Change:** Add a concrete worked example showing intersection on ceilings + union on grants:

> **Worked example — Acme (bash allowed) + Beta (bash forbidden) co-own `joint-research`.** Session `s-9901` is tagged `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5]`. `joint-acme-5` (base_org = Acme) tries to invoke `bash`. The ceiling check evaluates each co-owner org's ceiling independently: Acme permits `bash`; Beta forbids it. The effective ceiling is the **intersection** (rule 2): `bash` is forbidden. The invocation is denied.
>
> Contrast: a session tagged only `[project:acme-api-platform, org:acme]` has only Acme's ceiling applied; `bash` is permitted. Intersection only engages when multiple owner ceilings overlap on the same resource.
>
> **Union, not intersection, for individual grants.** If `joint-acme-5` holds two independent grants — one under Acme's authority (`read`) and one under Beta's (`list`) — both are in effect. A single grant cannot exceed its issuing co-owner's ceiling, but two grants together may touch any scope either co-owner authorises. **Intersection applies to ceilings; union applies to grants.** This keeps co-ownership additive at the grant level while conservative at the ceiling level.

### F6: Templates C/D/E full worked examples  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `permissions/07-templates-and-tools.md`

**Where:** Inside §Standard Permission Templates, after Template A and B YAML.

**Changes:** Three new subsections, each with full YAML:

- **§Template C — Org-Chart-Triggered Grants.** Fires on `MANAGES` or `REPORTS_TO` edge creation. Example: when `HAS_LEAD` fires for a new team lead, Template C grants the CEO `[read, inspect]` on the lead's sessions, gated by org consent policy.
- **§Template D — Project-Role-Triggered Grants.** Fires on `HAS_AGENT` with `role: lead`. Grants the lead `[allocate]` on project resources and `[read, inspect]` on member sessions. Parallel shape to Template A but scoped to the project role.
- **§Template E — Explicit Manual Grants.** Already named in the doc without YAML — add a canonical example: an auditor agent receiving a time-bounded `[read]` grant on a specific session set via the standard Auth Request flow.

Each subsection includes a "when to adopt" note.

### F7: Multi-composite tool manifest example (tool #14)  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `permissions/07-templates-and-tools.md`

**Where:** §Tool Authority Manifest Examples, after tool #13 (`request_grant`).

**Change:** New subsection §14 `workspace_snapshot` — a tool that reads from multiple composites in one invocation:

```yaml
tool: workspace_snapshot
manifest:
  resource: [filesystem_object, memory_object, session_object]
  actions: [read, list, inspect]
  constraints:
    tag_predicate: required
  kind: [memory, session]
  target_kinds: [memory, session]
  delegable: false
  approval: auto
```

**Teaching value:** illustrates that composite resource types list together, `kind` unions across touched composites, `target_kinds` names only the kinds resolved per-instance, and bare fundamentals (filesystem) carry no `#kind:` entry. Walk through the caller's required grant set.

### F8: coordination.md tentative v0 answers  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `baby-phi/docs/specs/v0/concepts/coordination.md`

**Where:** §Open Design Questions.

**Change:** Replace the bare bullet list with a tentative-v0-answer table:

| Question | Tentative v0 answer |
|----------|----------------------|
| Storage backend | **SQLite** (single-file, transactional, migratable; step up from phi-core's JSON). Graph DB is a v1 conversation once access patterns stabilise. |
| Query language | **Custom tag-predicate DSL** for Grant selectors; **Cypher-inspired subset** (MATCH/WHERE/RETURN over named edges) for graph traversal. Agents see one `query(...)` API surface. |
| Schema versioning | **Additive-only in v0.** Adding a property is non-breaking. Removing/renaming requires a **migration Auth Request** (Template E shape). |
| Event sourcing | **Hybrid.** State-based current-node view for reads; append-only `AgentEvent` stream for audit and replay. Matches phi-core's existing event pattern. |
| Consistency model | **Last-writer-wins per node** with timestamped optimistic concurrency on writes. Provenance-carrying edges (`DESCENDS_FROM`, `EMITTED_BY`) are append-only. |
| Memory types | The four from Claude Code: **user, feedback, project, reference**. Adopt as the v0 `memory_type` enum. |
| MCP lifecycle | **Lazy connection on first use**, persistent for the Session. Disconnect on session end. |
| Provider testing | **System-session shape** with no `project:` tag. Tests run under a system agent; the session carries `agent:system-tester, org:{test_org}` only. |

Each row flagged "v0 default, revisitable."

### F9: Lock in Value field rolling window  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `baby-phi/docs/specs/v0/concepts/token-economy.md`

**Where:** §Value section's open question.

**Change:** Replace the "Probably the same window mechanism — let's reuse it" hedge with:

> **Decision (v0):** Value uses the **same rolling-window mechanism as ratings** (window = 20, oldest entries collapse into a running average on overflow). No new mechanism to design. Revisitable if usage shows the two signals want different windows.

### F10: Schema Registry extension permission  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `baby-phi/docs/specs/v0/concepts/ontology.md`

**Where:** §Schema Registry (Meta-Graph), after the "Enables" bullet list.

**Change:** Add a subsection:

> **Who may extend the schema.** Creating a new `SchemaNode` or `SchemaEdge` at runtime requires:
> 1. The agent holds `[allocate]` on `control_plane_object` (typically platform admins and specifically-designated System Agents).
> 2. A **schema-extension Auth Request** (Template E shape) has been approved for the specific extension. The approved request serves as the extension's provenance.
>
> This mirrors template adoption: one gate event authorises the extension; subsequent nodes/edges of the new type follow the standard ownership model. Schema removal follows the same pattern in reverse.

### F11: Witnessed Experience concurrent update cadence  `[PLAN: new]` `[DOCS: ⏳ pending]`

**File:** `baby-phi/docs/specs/v0/concepts/agent.md`

**Where:** §Witnessed Experience Is Mediated by Extraction.

**Change:** Add a paragraph:

> **Concurrent sub-agent supervision.** A supervisor with multiple concurrently-running sub-agents accumulates Witnessed Experience **reactively per extraction**, not batch-at-session-end. Each Memory extraction emits a discrete update to the supervisor's `witnessed` struct; concurrent extractions from different sub-agents produce concurrent updates, ordered by the standard last-writer-wins consistency rule. No batching mode — this keeps the identity model's reactive semantics uniform across all triggers.

### F12: Verification  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. **No stale monolith references.** `grep -r "concepts/permissions.md"` returns zero hits across the repo.
2. **Intra-split links resolve.** For each `](XX-filename.md#anchor)` in the split files, target file exists and anchor is present.
3. **Cross-file refs.** Every `permissions/` occurrence in other concept files targets an existing split file + valid anchor.
4. **No stray omitted/duplicated content.** Grep split files for `[Omitted long matching line]` and duplicate section headers.
5. **Line count sanity.** Sum of split file lines within ~15% of 3986 + header overhead.

### F13: Independent Confidence Re-Evaluation  `[PLAN: new]` `[DOCS: ⏳ pending]`

After F1–F12 land, launch an Explore subagent with no anchoring to prior scores (72 / 88 / 96 / 73). Scope: all files under `baby-phi/docs/specs/v0/concepts/` including the split `permissions/` folder. Exclusions unchanged (Permission Check pseudocode, selector grammar, Market spec). Deliverables: fresh confidence %, per-area ratings, any new gaps introduced by the split. Reconcile against the ≥85% projection below.

## Critical Files  `[PLAN: new]` `[DOCS: n/a — reference list]`

| File | Edit(s) |
|------|---------|
| `baby-phi/docs/specs/plan/<random>-split-and-gap-closure.md` (NEW) | F0 (archive this plan) |
| `baby-phi/docs/specs/v0/concepts/permissions/` (NEW folder, 9 files) | F1, F2 |
| `baby-phi/docs/specs/v0/concepts/permissions/06-multi-scope-consent.md` | F5 |
| `baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md` | F6, F7 |
| `baby-phi/docs/specs/v0/concepts/permissions.md` | F4 (delete after F1–F3 verify) |
| `baby-phi/docs/specs/v0/concepts/ontology.md` | F3 (links), F10 (Schema Registry extension) |
| `baby-phi/docs/specs/v0/concepts/agent.md` | F3 (links), F11 (Witnessed Experience concurrency) |
| `baby-phi/docs/specs/v0/concepts/organization.md` | F3 (links) |
| `baby-phi/docs/specs/v0/concepts/project.md` | F3 (links) |
| `baby-phi/docs/specs/v0/concepts/coordination.md` | F3 (links), F8 (tentative answers) |
| `baby-phi/docs/specs/v0/concepts/human-agent.md` | F3 (links) |
| `baby-phi/docs/specs/v0/concepts/token-economy.md` | F9 (Value window) |
| `baby-phi/docs/specs/v0/concepts/README.md` | F3 (links) |

## Projected Confidence  `[PLAN: new]` `[DOCS: n/a — projection]`

**Pre-Phase-F full-corpus confidence:** 73%.
**Projected post-Phase-F full-corpus confidence:** **≥ 85%**, target band 85–90%.

| Edit | Gap closed | Expected contribution |
|------|-----------|------------------------|
| F1–F4 (split) | Readability / navigability | +2% |
| F5 (co-ownership worked example) | Phase-E review's only "has-gaps" rating | +2% |
| F6 (Templates C/D/E) | Missing YAML for 3 of 5 templates | +3% |
| F7 (tool #14) | Multi-composite example gap | +1% |
| F8 (coordination.md answers) | 8 open questions → tentative answers | +3–4% |
| F9 (Value window) | 1 unresolved design question | +1% |
| F10 (Schema Registry extension) | 1 permission-model gap | +1–2% |
| F11 (Witnessed Experience concurrency) | 1 unspecified semantic | +1% |

**Upper bound capped at ~90%** because Permission Check runtime pseudocode, formal selector grammar, and Market full spec remain deferred — they're real gaps that only a follow-on plan would close.

## What Stays Unchanged  `[PLAN: new]` `[DOCS: n/a — scope guard]`

- **Conceptual content** of the permissions spec. The split is structural; no section is rewritten beyond the specific additions in F5–F7.
- **Phases A–E work.** All previously-verified content stays bit-identical in its new file location.
- **Deferred items** (Permission Check pseudocode, selector grammar, Market spec) remain out of scope.
- **No split of other concept files.** Only `permissions.md` is large enough to justify the cost.

## Verification Summary  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. Plan archived (F0) and no prior work is lost.
2. `permissions/` folder is populated with 9 files totalling ~4,500 lines; old `permissions.md` deleted.
3. Zero stale `permissions.md` references anywhere in the repo.
4. All split internal and cross-file links resolve.
5. Seven content gaps (F5–F11) are addressed in-place.
6. Independent confidence re-evaluation returns ≥ 85%.
