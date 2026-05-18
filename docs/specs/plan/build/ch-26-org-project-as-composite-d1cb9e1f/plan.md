<!-- Last verified: 2026-05-17 by Claude Code (chunk-planner v14, cycle hex `d1cb9e1f` — CH-26 — Org/Project as Composite resources. Closes D-philosophy-02 HIGH-A. Drafts ADR-0061. Prereq CH-25 (ADR-0060 Edge::Owns + owner-grant rule) ✓ confirmed. phi-core dep `phi-core = "0.7.1"` (crates.io, post-migration baseline; HEAD-delta pre-flight: matches current pin, no carrier-fix). Baseline 1556 tests / 0 failed / 2 ignored. Baseline phi-core imports 57. v2 re-plan after gate-1 user-lock 2-of-3 DIVERGENT (F1.b wide handler refactor + F2.b new tags field on Org/Project structs; F3 audit envelope aligned). Cross-cycle divergence pattern at 10-of-12 cumulative (~83%) — v12 prominent gate-1 callout refreshed; v13 divergence-aware framing applied to F1 + F2. -->

# CH-26 — Org/Project as Composite resources (cycle hex `d1cb9e1f`)

## Forks for orchestrator

> ⚠️ **CROSS-CYCLE DIVERGENCE PATTERN (REFRESHED v2 — 10-of-12, ~83%)**: planner-recommendation has diverged from user-lock in **6 of last 7 closing cycles** (CH-15 `c3f46f17` F5.B / CH-17 `40c4d759` F5.B / CH-18 `c77937bc` F3.B / CH-20 `240616a4` F1.B / CH-24 `5778bb77` F1.B + F-D59.2.b + F-D59.3.b / **CH-26 `d1cb9e1f` F1.b + F2.b**; non-divergent: CH-19 `2c520ba7` only). **Cumulative divergent forks 10 of 12 (~83% — highest cumulative-divergence rate observed across all chunks)**. User systematically prefers maximal-scope / richest-wire / most-defensive options at gate-1. **The pattern has now matured to "modal-by-default"** — applying v13 §"Gate-2.5 mid-cycle scope-expansion lane + v9 re-evaluation" divergence-aware framing to F1 + F2 below.

### F1 — Handler refactor scope (CRITICAL fork; user-lock required)

D-philosophy-02 §Remediation step 3 requires: *"Refactor admin-page-1 (Orgs) + admin-page-3 (Projects) handlers to call Permission Check engine (replacing bespoke gates)"*. Reality check: handlers today (`orgs/{list,show,create,dashboard}.rs` + `projects/{create,detail,resolvers,agent_supervisor}.rs`) DO NOT call `check_permission` — they're gated only by `AuthenticatedSession` extractors + per-handler bespoke logic (`check_auth_request_access` for AR-list filtering at `show.rs:79`; CEO-only checks at `create.rs`). CH-25 ADR-0060 §D60.4 acknowledged this for `disable_system_agent` and applied the **load-bearing-form** pattern (engine-level test instead of handler-coupled test).

- **F1.a (planner-recommended at v1)** — **NARROW scope, defer full handler refactor to M7.** Add `Composite::OrganizationObject` + `Composite::ProjectObject` enum variants + `constituents()` + cardinality 8→10. Selector grammar already matches `org:O` / `project:P` resource URIs (engine.rs:212+249+257 — CH-25 wired this). Acceptance ships in CH-25 **load-bearing form**: engine-level invocation `handler_support::check_permission` against a tagged Org/Project resource URI; assert Allow under synth-owner-grant; assert Deny for stranger Agent. NEW M7-DEFERRED-NN drift filed for full Permission-Check-engine-routed handlers (the ≥ 3-hit grep assertion in D-philosophy-02:39 becomes a CH-26-deferred follow-up). Scope: ~3 ed.
- **F1.b** — **Locked at gate-1 (USER-DIVERGENT): WIDE scope, refactor handlers in-chunk.** Wire `handler_support::check_permission` into ≥ 7 handlers across `orgs/{list,show,create,dashboard}` + `projects/{create,detail,resolvers,agent_supervisor}`. Per concept-doc 01 §"Composite Creation Checklist" #3 (lifecycle rules) + #9 (default grants), each handler maps to a canonical Action verb (likely `Action::Observe` for list/detail/dashboard; `Action::Allocate` for create / state-mutation; `Action::Inspect` for resolver-tier reads). Scope: ~5 ed; blast radius spans 7+ HTTP endpoints + acceptance suite rebuild + ≥ 3 wire-format changes. **Risk**: M3 + M4 + M5 acceptance suites likely break on permission errors when the previously-pass-through gate now requires explicit grants — see §7 P2 blast-radius warning + §8 widened test band.
- **F1.c** — **MEDIUM scope: ship Composite variants + wire ONE canonical handler refactor (`list_organizations`) as the precedent**, defer remaining handlers (show / create_project / dashboard / detail / resolvers / agent_supervisor) to a NEW M7-DEFERRED drift covering "remaining org/project handler Permission-Check routing". Scope: ~4 ed. Single in-cycle handler demonstrates ≥ 1 hit on the drift's ≥ 3-hit invariant (D-philosophy-02:39); remaining ≥ 2 hits accrue at the M7 follow-up.

**Locked at gate-1 (USER-DIVERGENT): F1.b.** v1 planner-recommendation was F1.a (narrow); user diverged toward F1.b (wide). Rationale aligned with cross-cycle 83%-divergent pattern: user systematically prefers the more-defensive-against-concept-drift option. v2 re-plan honors the lock; §7 P2 enumerates ≥ 7 handler refactor sites; §8 test count band widened to absorb acceptance-suite triage; §11 Audit-A claim count widened for handler-cascade verification.

### F2 — Backfill migration scope (high-impact)

D-philosophy-02 §Remediation step 2: *"Migration to backfill instance-identity tags on existing Organization + Project rows (cf CH-06 + CH-16 migration patterns)"*. The reality: Organization + Project structs do NOT have a `tags: Vec<String>` field (verified `nodes.rs:362-489`; tags live on Session/Memory/Channel/etc.). Instance-identity tags for Org/Project must be applied either (a) via `seed_catalogue_entry_for_composite` registration (URI-keyed; the existing path used by MCP/model-providers/defaults), or (b) via a new tag-bearing field on Organization/Project structs (significant cascade).

- **F2.a (planner-recommended at v1)** — **Catalogue-entry-based instance-identity.** Migration `0018_org_project_catalogue_backfill.surql` calls the existing `seed_catalogue_entry_for_composite` path for every extant Organization row (URI `org:<uuid>`, owning_org `None`) + every extant Project row (URI `project:<uuid>`, owning_org `Some(parent_org)`). The catalogue entry is the canonical "Org/Project exists as resource type" anchor; `apply_org_creation` + `apply_project_creation` extend in-cycle to seed the entry at-creation-time. Scope: ~0.5 ed migration body + ~0.3 ed compound-tx extension.
- **F2.b** — **Locked at gate-1 (USER-DIVERGENT): Tag-field-bearing structs (RICHER WIRE-FORMAT).** Add `pub tags: Vec<String>` to `Organization` + `Project` structs (with `#[serde(default)]` for backward-compatible deserialisation of pre-CH-26 rows). Backfill migration `0018_org_project_tags.surql` populates the new column with empty array default + writes the instance-identity tag (`organization:<uuid>` / `project:<uuid>`). Cascade through 52 Organization-literal sites + 16 narrow Project-node-literal sites + InMemoryRepository materialise + SurrealStore materialise + acceptance fixture updates. ~1.5 ed core + ~0.5 ed fixture sweep = ~2 ed. **Risk-mitigated**: existing `tags` field discipline on Session/Memory/Channel/AgentCredential (`nodes.rs:836, 1062, 1074, 1082, 1269`) is the precedent — `#[serde(default)]` keeps old rows deserialisable; JSON snapshot tests are minimal (zero `serde_json::to_value(Organization|Project)` matches in source tree).

**Locked at gate-1 (USER-DIVERGENT): F2.b.** v1 planner-recommendation was F2.a (catalogue-entry path); user diverged toward F2.b (tag-field on structs). Rationale aligned with cross-cycle 83%-divergent pattern: user systematically prefers the richer-wire / more-explicit-state representation. v2 re-plan honors the lock; §3 cascade map widened; NEW phase P-FIELD-EXTEND inserted between P1 + P2; §5 ADR sub-decisions rewritten; §8 test count band widened.

### F3 — Audit envelope size (3 vs 2)

Phase count = **7** post-v2 (P0 / P1 / P-FIELD-EXTEND / P2 / P3 / P-DOCS / P-SEAL — see §7). Per chunk-planner v9 audit-envelope-size skill: 6+ phases → Large (3 auditors A+B+C). CH-25 (6 phases) used 3 auditors. CH-26 v2's surface is broader + closes a HIGH-A drift + flips ontology cardinality + introduces 2 backfill migrations (Composite + tags) + refactors ≥ 7 handlers. Recommendation: **3 auditors** (A code + cascade + phi-core; B docs + concept + ADR; C carry-forward regression — backfill migration safety + handler-refactor blast-radius across M3/M4/M5 acceptance suites).

**Locked at gate-1: F3 default 3 auditors.** No divergence from planner recommendation.

## Verified-header context

**Cycle hex**: `d1cb9e1f` (generated via `openssl rand -hex 4` at plan-draft P0).

**Chunk slug**: `ch-26-org-project-as-composite`.

**Forward-scope row**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 lines 260-271.

**Drift closed**: [`D-philosophy-02`](../../../v0/implementation/m5_3/drifts/D-philosophy-02.md) (HIGH severity, Bucket A — load-bearing scope gap).

**ADR drafted**: [`m5_3/decisions/0061-org-project-as-composite-resources.md`](../../../v0/implementation/m5_3/decisions/0061-org-project-as-composite-resources.md) (next-free; forward-scope's predicted `0043` is stale — latest shipped is 0060 from CH-25).

**Prereqs**: CH-25 ✓ (ADR-0060 / Edge::Owns / owner-grant synth-rule / `list_agent_owned_orgs/projects` / engine.rs:212+249+257 already wires `org:<uuid>`/`project:<uuid>` ResourceRef.uri).

**Baseline state** (P0 verified):
- Workspace tests: **1556 / 0 failed / 2 ignored** (post-CH-25 + post-phi-core-crates.io-migration).
- phi-core imports: **57** (canonical grep: `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l`).
- phi-core dep: `phi-core = "0.7.1"` at workspace root + `{ workspace = true }` in 4 crates. **v14 R3 phi-core HEAD delta pre-flight: `cargo search phi-core --limit 3` returns `phi-core = "0.7.1"` matching current pin — NO carrier-fix needed at CH-26 P0**.
- Migration slots: 0001-0017 taken; **next-free = 0018** (`ls modules/crates/store/migrations/`). v2 uses slot 0018 for the unified `org_project_tags` migration (combines Composite-cardinality flip + tags-column add + instance-identity backfill in one transaction).
- `Composite::ALL.len() == 8` invariant target → **10** after CH-26 (controlled invariant evolution; same pattern as CH-25's 71→72 EDGE_KIND_NAMES).
- Organization struct field count target: **+1** (`tags: Vec<String>` with `#[serde(default)]`). Project struct field count target: **+1** (same).

**v2 re-plan summary** (changes from v1):
1. F1.b user-lock — ≥ 7 handler refactor sites enumerated in §7 P2; ADR §D61.4 rewritten with handler-by-handler PASS/FAIL scenarios; §8 test band widened from [1561, 1563] → [1568, 1574].
2. F2.b user-lock — NEW phase P-FIELD-EXTEND inserted between P1 + P2; §3 cascade-grep widened for Organization + Project struct-literal sites (52 + 16 = 68 raw matches); §5 §D61.2 rewritten to add `tags: Vec<String>` field; migration 0018 reframed as `0018_org_project_tags.surql` (column add + backfill in one).
3. Cross-cycle divergence callout refreshed to 10-of-12 (~83%); pattern's maturation documented.
4. §3.E gate-2.5 candidates extended for handler-refactor + tags-field-cascade fan-out.

---

## §1 — Context & principle

**Why this chunk.** Concept-doc [`concepts/core-philosophy.md`](../../../v0/concepts/core-philosophy.md) line 16 claims *"Organization has Projects (A Resource Type)"* + line 28 claims *"Organizations own Resources"* + line 29 claims *"Projects own Resources"*. These claims are honored at the principal axis (Org/Project are Principals per `principal_resource.rs:182-186`'s Principal-only invariant) and at the ownership-edge axis (CH-25 ADR-0060 Edge::Owns). But the **resource-axis claim is contradicted**: `Composite` enum (`composites.rs:20-45`) has **8 variants** — Org and Project are NOT among them. Permission Check selectors can match `org:X` / `project:Y` as URIs (engine.rs:212+249+257 — wired by CH-25's synth-owner-grant rule), BUT the engine cannot enumerate Org/Project as `Composite` resource classes for the `applies_to_composite` action-availability check, and `seed_catalogue_entry_for_composite` cannot register Org/Project entries because no `Composite::OrganizationObject` / `Composite::ProjectObject` variant exists to pass. The unified-resource-model intent (concept-doc-01 §Resource Ontology — two tiers; the user clarification 2026-04-28 *"how else would you answer the question who grants other agents permission to a project or organization?"*) is structurally incomplete.

CH-26 closes the gap by (v2 — reflects F1.b + F2.b user-locks):

1. Adding `Composite::OrganizationObject` + `Composite::ProjectObject` to the closed `Composite` enum (+2 variants, cardinality 8 → 10), each with `constituents() = [DataObject, IdentityPrincipal, Tag]` reflecting the Org/Project dual nature (governance container + first-class resource).
2. **F2.b — Adding `tags: Vec<String>` field to `Organization` + `Project` structs** (with `#[serde(default)]` for backward-compatible deserialisation of pre-CH-26 rows; precedent: `Session.tags`, `Memory.tags`, `Channel.tags`, `AgentCredential.tags` at `nodes.rs:836, 1062, 1074, 1082, 1269`).
3. **F2.b — Backfill migration `0018_org_project_tags.surql`** (a) adds the SurrealDB `tags: array<string>` column to organization + project tables with empty-array default; (b) populates each row with its instance-identity tag (`organization:<uuid>` / `project:<uuid>`); (c) calls `seed_catalogue_entry_for_composite` for every extant Org + Project row to register the canonical catalogue anchor.
4. Extending `apply_org_creation` + `apply_project_creation` compound transactions to (a) populate `tags: vec![format!("organization:{}", id)]` / `vec![format!("project:{}", id)]` at-creation-time + (b) seed the catalogue entry at-creation-time (mirrors the MCP/model-provider/defaults pattern at `mcp_servers/register.rs:124` + `model_providers/register.rs:153` + `defaults/put.rs:114`).
5. **F1.b — Refactoring ≥ 7 admin-page-1/3 handlers** (`orgs::{list,show,create,dashboard}` + `projects::{create,detail,resolvers,agent_supervisor}`) to invoke `handler_support::check_permission` against the appropriate Action verb. Selector matches `org:<id>` / `project:<id>` via the new instance-identity tags on the row (the tags ARE the resolver-lookup keys now); the catalogue entry is the Step-0 precondition; the engine resolves Allow/Deny via the CH-25-shipped synth-owner-grant rule for the owning Agent + via cross-org isolation for strangers.
6. Acceptance test suite `server/tests/acceptance_m5_3_composite_resources.rs` exercising both engine-level invariants AND handler-level PASS/FAIL scenarios (per F1.b — handlers now ARE the surface, not the load-bearing-form workaround).
7. New ADR-0061 ratifying the unified resource model + the tag-field-on-struct pattern + the ≥ 7 handler refactor + the catalogue-entry-as-Step-0-precondition pattern.

**Quality-over-speed restatement.** Concept docs are source-of-truth. The unified-resource-model contradiction is HIGH severity. v2 re-plan absorbs the user-locked WIDER scope (F1.b + F2.b) without scope-creep beyond those locks; both deliveries are ratified in-cycle to close the D-philosophy-02:39 ≥ 3-hit invariant in full at chunk-seal (no M7 follow-up drift needed under F1.b user-lock).

**Forward-scope reference**: [§2.5 lines 260-271](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md).

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Target status at chunk-close (v2 F1.b + F2.b) |
|---|---|---|---|---|
| `concepts/core-philosophy.md` | line 16 | "Organization has Projects (A Resource Type)" | **partially-honored** | **fully-honored** at composite-axis + handler-routing-axis (F1.b in-cycle handler refactor closes the routing axis; no M7-DEFERRED) |
| `concepts/core-philosophy.md` | lines 28-29 | "Organizations own Resources" + "Projects own Resources" | **honored** post-CH-25 (Edge::Owns wires the ownership relation) | **honored** (no change; CH-26 builds on this) |
| `concepts/permissions/01-resource-ontology.md` | lines 30-43 §"Composite Classes (8)" | "Composite Classes (8)" closed table | **honored at cardinality 8** | **honored at cardinality 10** (controlled invariant evolution) |
| `concepts/permissions/01-resource-ontology.md` | lines 322-346 §"Composite Creation Checklist" | "A composite definition must include 13 fields" | **silent-in-code** for Org/Project | **honored documentarily** — ADR-0061 §body includes the 13-field walk for Org/Project; concept-audit-matrix row flips reflect the addition |
| `concepts/permissions/01-resource-ontology.md` | lines 222-263 §"Instance Identity Tags" | "Every composite instance carries `{kind}:{instance_id}`" | **silent-in-code** for Org/Project | **fully-honored** — `tags: Vec<String>` field on struct + backfill migration populates `organization:<uuid>` / `project:<uuid>` on every row; `apply_org_creation` / `apply_project_creation` extend to populate at-creation-time |
| `concepts/permissions/README.md` | "entry invariants" | permissions-subtree-hook citation per per-chunk-template §2 rule | **honored** | **honored** |
| `concepts/phi-core-mapping.md` | (no overlap) | phi-core has no Composite resource-ontology counterpart | **honored** — N/A | **honored** (no change) |

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | Resource ontology + Composite enum + Permission Check engine + Repository.tags-write paths are baby-phi-native | N/A — orthogonal | No phi-core type introduced or removed |

**Expected import-count delta at chunk close**: **+0**. CH-26 v2 surface (F1.b + F2.b) is entirely baby-phi-native (Composite enum, Permission Check engine, Repository trait, Organization/Project node-types).

**Positive close-audit greps**:
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect **57** (unchanged from baseline).

**Forbidden-duplication greps**:
- `grep -rnE 'pub struct (OrganizationObject|ProjectObject)\b' /root/projects/phi/baby-phi/modules/crates/` — expect **0** (the variants are enum members, not standalone structs).
- `grep -rnE '#kind:org\b' /root/projects/phi/baby-phi/modules/crates/` — expect **0** (kind names are `organization` / `project`, not `org` / `proj`).
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` — expect exit 0.

### §3 cascade-artifact discipline (CH-13 retro Row 2 / chunk-planner v3; v2 widened for F1.b + F2.b)

**Artifact A — `Composite::ALL` enumeration cascade (8 → 10).**

Invocation: `git -C /root/projects/phi/baby-phi grep -nE 'Composite::ALL|Composite::ExternalServiceObject|Composite::ModelRuntimeObject|Composite::ControlPlaneObject|Composite::MemoryObject|Composite::SessionObject|Composite::AuthRequestObject|Composite::InboxObject|Composite::OutboxObject' modules/crates/`

Raw count at plan-draft: **96 matches**. Per-file breakdown (production-vs-test classified per v13 R2):

| File:line | Site shape | Production / Test | Edit type |
|---|---|---|---|
| `domain/src/model/composites.rs:20-45` | enum definition | PRODUCTION | **REQUIRED** — add 2 variants |
| `domain/src/model/composites.rs:47-59` | `ALL` array literal `[Composite; 8]` | PRODUCTION | **REQUIRED** — flip cardinality `8 → 10`; add 2 entries |
| `domain/src/model/composites.rs:62-73` | `as_str()` match | PRODUCTION | **REQUIRED** — add 2 arms |
| `domain/src/model/composites.rs:81-91` | `kind_tag()` match | PRODUCTION | **REQUIRED** — add 2 arms |
| `domain/src/model/composites.rs:150-174` | `constituents()` match | PRODUCTION | **REQUIRED** — add 2 arms (Org → `[DataObject, IdentityPrincipal, Tag]`; Project → `[DataObject, IdentityPrincipal, Tag]`) |
| `domain/src/model/composites.rs:182-250` | unit-test bodies | TEST | **REQUIRED** — flip `8 → 10` literals at `:184`, `:189`, `:191`, `:206`, `:240`; tests auto-iterate `Composite::ALL` so no per-variant additions |
| `domain/src/permissions/action.rs:741, 748, 762-779, 784` | `for c in Composite::ALL` loops + action-applicability tests | mixed (loop sites auto-cover new variants; explicit-test sites add 2 `Action::*.applies_to_composite(Composite::Organization/ProjectObject)` smoke-checks) | **REQUIRED** — +2 smoke assertions (Allocate / Transfer on each); existing `applies_to_composite_via_constituent_union` test auto-covers via `Composite::ALL` iteration |
| `domain/src/permissions/manifest/validator.rs:76, 80, 219, 397, 452-460, 523, 698, 714, 848, 938-1182, 1215` | various `Composite::ALL` iterations + `Composite::MemoryObject` exemption (D49.1.a) | mixed | mostly **NO EDIT** (loops auto-cover); D49.1.a exemption stays Memory-only |
| `server/src/platform/defaults/put.rs:114` + `server/src/platform/mcp_servers/register.rs:124` + `server/src/platform/model_providers/register.rs:153` | existing `seed_catalogue_entry_for_composite` callsites | PRODUCTION | **NO EDIT** — these are precedent for the new compound-tx extensions |
| `server/src/platform/defaults/mod.rs:58, 157, 162` | doc comments + tag assertion | mixed | **NO EDIT** — Control Plane Object unaffected |
| `server/tests/acceptance_manifest_validator.rs:74, 99, 280, 318, 350, 376` | acceptance assertions on `Composite::MemoryObject` + `SessionObject` | TEST | **NO EDIT** — variant-specific |
| `domain/src/model/composites.rs:213-249` | `kind_name_strips_hashkind_prefix` / `auto_tags_*` for-loop tests | TEST | **NO EDIT** — loops auto-cover |

**Aggregate prediction band: 12-18 production edit sites + 6-10 test-literal flips = 18-28 total edits.** Pause threshold: actual sites > 42 (1.5× upper bound = 28 × 1.5).

**Canonical grep for cascade re-verification at chunk-seal**: `git -C /root/projects/phi/baby-phi grep -nE 'Composite::(OrganizationObject|ProjectObject)' modules/crates/ | wc -l` — expect ≥ 4 (variant declaration site x2 + ALL array x2; integration tests + acceptance + ADR cross-refs add 4-6 more).

**Artifact B — `Composite::ALL.len() == 8` invariant-literal cascade (8 → 10).**

Invocation: `git -C /root/projects/phi/baby-phi grep -nE 'Composite::ALL\.len\(\)|\[Composite; 8\]' modules/crates/`

Raw count at plan-draft: **5 matches** (4 in `composites.rs` self-tests + 1 in validator.rs:714 indirect via `3 + Composite::ALL.len()`). Per-file:

| File:line | Shape | Edit type |
|---|---|---|
| `domain/src/model/composites.rs:50` | `pub const ALL: [Composite; 8]` type annotation | **REQUIRED** — flip `8 → 10` |
| `domain/src/model/composites.rs:184` | `assert_eq!(Composite::ALL.len(), 8)` | **REQUIRED** — flip `8 → 10`; rename test fn `all_contains_exactly_eight → _ten` |
| `domain/src/model/composites.rs:189-191` | `let set: HashSet<_> = Composite::ALL.iter().collect(); assert_eq!(set.len(), 8)` (distinct-variants) | **REQUIRED** — flip `8 → 10` |
| `domain/src/model/composites.rs:206-207` | `kind_tags_are_distinct` — `assert_eq!(tags.len(), 8)` | **REQUIRED** — flip `8 → 10` |
| `domain/src/model/composites.rs:240-241` | `as_str_is_distinct_per_variant` — `assert_eq!(strs.len(), 8)` | **REQUIRED** — flip `8 → 10` |
| `domain/src/permissions/manifest/validator.rs:714` | `assert_eq!(prefixes.len(), 3 + Composite::ALL.len())` | **NO EDIT** — uses `.len()` symbolically; auto-flips |

**Aggregate: 5 literal-cardinality sites.**

**Artifact C — `seed_catalogue_entry_for_composite` callsite cascade.**

Invocation: `git -C /root/projects/phi/baby-phi grep -nE 'seed_catalogue_entry_for_composite' modules/crates/`

Raw count: **9 matches**. Per-file: (unchanged from v1; new CH-26 sites at `apply_org_creation` + `apply_project_creation` bodies x 2 backends = 4 new callsites).

**Artifact D — `apply_org_creation` / `apply_project_creation` compound-tx body cascade.**

Edit scope per backend body: ~6-8 lines (tags population + seed call + `?` propagation). **Aggregate: 8 lines × 2 backends = ~16 lines of body change**.

### Artifact E (NEW v2 — F2.b `tags: Vec<String>` field cascade on Organization + Project structs)

**Cascade-grep MANDATORY per chunk-planner v3 discipline; widened per v9 R7 for multi-site phi-core-type-shared types — though here the type is baby-phi-native, the multi-site shape is the same.**

**E.1 Organization struct-literal cascade.**

Invocation: `git -C /root/projects/phi/baby-phi grep -nE 'Organization\s*\{$' modules/crates/`

Raw count at plan-draft P0 (v2 re-verification): **52 matches** spread across the following files (per-file breakdown, production-vs-test classified per v13 R2):

| File:line | Site shape | P/T | Edit type |
|---|---|---|---|
| `domain/src/model/nodes.rs:362` | struct definition | PRODUCTION | **REQUIRED** — add `pub tags: Vec<String>` field with `#[serde(default)]` |
| `domain/src/model/nodes.rs:432` | `impl Organization { ... }` block | PRODUCTION | **CONDITIONAL** — add `default_tags()` helper IF needed for backward-compat (likely unneeded — `Vec::default() == vec![]`) |
| `domain/src/audit/events/m3/orgs.rs:174-175` | `sample_org()` helper | TEST | **REQUIRED** — add `tags: vec![]` field |
| `domain/src/events/listeners.rs:1699, 2182, 2597` | test-mode org constructions | TEST | **REQUIRED** — 3 fixture updates `tags: vec![]` |
| `domain/tests/in_memory_ch23_edges.rs:44` | `minimal_org()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_ch25_owns_edges.rs:38` | `minimal_org()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_m3_test.rs:23` | `minimal_org()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_m4_test.rs:63` | `minimal_org()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/platform_defaults_non_retroactive_props.rs:61` | inline org construction | TEST | **REQUIRED** — 1 fixture |
| `server/src/platform/agents/execution_limits.rs:152` | test-mode `org_with_ceiling` helper | TEST | **REQUIRED** — 1 fixture |
| `server/src/platform/agents/list.rs:110` | test-mode `org` helper | TEST | **REQUIRED** — 1 fixture |
| `server/src/platform/orgs/create.rs:230` | PRODUCTION `apply_org_creation` callsite (`let organization = Organization { ... }`) | PRODUCTION | **REQUIRED** — add `tags: vec![format!("organization:{}", id)]` (instance-identity at-creation-time) |
| `server/src/platform/orgs/dashboard.rs:770` | test-mode `org_with_name` helper | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_common/admin.rs:297` | acceptance harness `let organization = Organization { ... }` | TEST/HARNESS | **REQUIRED** — populate with instance-identity tag (mirrors production semantics) |
| `server/tests/acceptance_m4.rs:378, 534` | acceptance test inline org constructions | TEST | **REQUIRED** — 2 fixtures |
| `server/tests/acceptance_memory_extraction.rs:141` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_per_session_consent_gating.rs:79` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_projects_create.rs:89` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_projects_detail.rs:132` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/identity_materialization_acceptance.rs:23` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `server/tests/project_create_slot_fill_read_access_test.rs:68` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/project_create_submit_access_test.rs:60` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `server/tests/project_creation_access_test.rs:71` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `cli/tests/session_live_tail_test.rs:174` | CLI integration test | TEST | **REQUIRED** — 1 fixture |
| `store/tests/apply_agent_creation_tx_test.rs:32` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/apply_org_creation_tx_test.rs:41` | `sample_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/apply_project_creation_tx_test.rs:34` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/identity_repo.rs:27` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/migration_0009.rs:31` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/repo_m3_surface_test.rs:26` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/repo_m4_surface_test.rs:34` | `minimal_org()` | TEST | **REQUIRED** — 1 fixture |
| `store/tests/repository_test.rs:74, 1824` | 2 inline org constructions | TEST | **REQUIRED** — 2 fixtures |

**E.1 aggregate**: 1 PRODUCTION struct-def + 2 PRODUCTION constructions (orgs/create.rs + apply_org_creation x in-memory) + ~32 TEST fixture sites = **~35 sites**.

**E.2 Project struct-literal cascade.**

Invocation: `git -C /root/projects/phi/baby-phi grep -nE '^\s*(let \w+ = )?Project \{$|^\s*Project \{$|fn \w+\(.*\) -> Project \{$' modules/crates/`

Raw count at plan-draft P0 (narrow Project node literals, excluding `MaterialisedProject` / `PendingProject` / `ShapeBPendingProject` / `ClaimedProject` / `HasProject` / `AgentNotMemberOfProject` siblings): **16 matches** (verified by per-file breakdown):

| File:line | Site shape | P/T | Edit type |
|---|---|---|---|
| `domain/src/model/nodes.rs:460` | struct definition | PRODUCTION | **REQUIRED** — add `pub tags: Vec<String>` field with `#[serde(default)]` |
| `cli/src/main.rs:101` | CLI Project construction | PRODUCTION (CLI demo) | **REQUIRED** — add `tags: vec![]` (CLI demo populates standalone Project; instance-identity not load-bearing) |
| `cli/tests/e2e_first_session.rs:258` | CLI e2e fixture | TEST | **REQUIRED** — 1 fixture |
| `cli/tests/session_live_tail_test.rs:260` | CLI test fixture | TEST | **REQUIRED** — 1 fixture |
| `domain/src/audit/events/m4/projects.rs:235` | `sample_project()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_ch23_edges.rs:62` | `project()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_ch25_owns_edges.rs:69` | `make_project()` helper | TEST | **REQUIRED** — 1 fixture |
| `domain/tests/in_memory_m4_test.rs:46` | `project()` helper | TEST | **REQUIRED** — 1 fixture |
| `server/src/platform/projects/create.rs:366` | PRODUCTION `apply_project_creation` body (`let project = Project { ... }`) | PRODUCTION | **REQUIRED** — add `tags: vec![format!("project:{}", id)]` (instance-identity at-creation-time) |
| `server/src/platform/projects/detail.rs:627` | test-mode `sample_project()` helper | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_common/admin.rs:492` | acceptance harness | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_common/m5_bootstrap.rs:221` | acceptance harness | TEST | **REQUIRED** — 1 fixture |
| `server/tests/acceptance_m4.rs:439, 556` | acceptance fixtures (Shape A + Shape B) | TEST | **REQUIRED** — 2 fixtures |
| `server/tests/acceptance_per_session_consent_gating.rs:125` | acceptance | TEST | **REQUIRED** — 1 fixture |
| `store/tests/apply_project_creation_tx_test.rs:65` | store integration | TEST | **REQUIRED** — 1 fixture |
| `store/tests/repo_m4_surface_test.rs:65` | store integration | TEST | **REQUIRED** — 1 fixture |

**E.2 aggregate**: 1 PRODUCTION struct-def + 2 PRODUCTION constructions + ~13 TEST fixture sites = **~16 sites**.

**E.3 SurrealStore + InMemoryRepository materialise paths.**

Both `domain/src/in_memory.rs` (lines ~1620 for org materialise + ~1820 for project) and `store/src/repo_impl.rs` (apply_org_creation lines ~2306-2511 + apply_project_creation ~2512+) need:
- (a) `tags` field added to row-INSERT SurrealQL bodies for `apply_org_creation` + `apply_project_creation`.
- (b) row-fetch paths (e.g., `org_by_id` / `project_by_id` / `list_orgs` / `list_projects`) deserialise the new field via `#[serde(default)]` for pre-migration rows (no body change needed; the derive handles it).

**E.3 aggregate**: ~4 SurrealQL body modifications (2 per backend × 2 methods).

**E.4 Wire-format snapshot risk audit.**

`git grep -nE 'serde_json::to_value\(&?(Organization|Project)' modules/crates/` returns **0 hits** for the Organization/Project node structs (only `ProjectShape` enum sites). HTTP-tier JSON serialisation of Org/Project happens through axum response handlers that use serde with `#[serde(default)]` — backward-compatible. **Wire-format risk: LOW.**

**E aggregate (E.1+E.2+E.3): ~35 + ~16 + ~4 = ~55 struct/fixture-site edits** across production + tests + materialise paths.

**Combined v2 cascade prediction band: Artifact A (18-28) + Artifact B (5) + Artifact D (16 lines × ~5 spots ≈ 6 edit-blocks) + Artifact E (~55) = ~80-95 edit sites**. Pause threshold: actual sites > 142 (1.5× upper-band 95).

### Artifact F (NEW v2 — F1.b ≥ 7 handler refactor enumeration)

Handlers refactored in-cycle (locked at gate-1 per F1.b):

| Handler | Current gate | New Action verb | Selector shape |
|---|---|---|---|
| `server::platform::orgs::list::list_organizations` | `AuthenticatedSession` only | `Action::Observe` | `org:*` (scoped to actor's owning orgs via CH-25 `list_agent_owned_orgs`) |
| `server::platform::orgs::show::show_organization` | `AuthenticatedSession` + AR-filter | `Action::Inspect` | `org:<O>` |
| `server::platform::orgs::create::create_organization` | `AuthenticatedSession` + platform-admin role check | `Action::Allocate` on `platform:root` selector (CH-25 bootstrap precedent: platform-admin synth grant) | `platform:root` |
| `server::platform::orgs::dashboard::org_dashboard` | `AuthenticatedSession` | `Action::Observe` | `org:<O>` |
| `server::platform::projects::create::create_project` | `AuthenticatedSession` + CEO check | `Action::Allocate` on parent-org | `org:<O>` |
| `server::platform::projects::detail::show_project` | `AuthenticatedSession` | `Action::Inspect` | `project:<P>` |
| `server::platform::projects::resolvers::*` (project-resolver tier reads) | `AuthenticatedSession` | `Action::Observe` | `project:<P>` |
| `server::platform::projects::agent_supervisor::list_agent_supervisors` | `AuthenticatedSession` | `Action::Observe` | `project:<P>` |

**F aggregate**: **≥ 7 handler refactors** (count: 8 candidate handlers; conservative ≥ 7 per F1.b lock language). Each refactor (a) replaces or augments the bespoke gate with `handler_support::check_permission(actor, action, resource)`; (b) maps the result via `denial_to_api_error` to HTTP 403 with structured `NO_GRANTS_HELD` / `NO_APPLICABLE_GRANT` / `EXPLICIT_DENY` per CH-25 wire convention; (c) preserves the AR-filter / role-check semantics as a defence-in-depth secondary layer (per CH-09 + CH-25 precedent of layered gates).

**Blast radius**: M3 + M4 + M5 acceptance suites listed below will break on permission errors when the previously-pass-through gate now requires explicit grants. Triage strategy in §7 P2: harness updates seed `Edge::Owns` for the actor → org / project under test BEFORE invoking the handler; CH-25 owner-grant synth-rule then resolves Allow without explicit persisted grants.

**Cascade-grep MANDATORY re-verification at P-CHUNK-OPEN**: `git -C /root/projects/phi/baby-phi grep -nE 'check_permission' modules/crates/server/src/platform/{orgs,projects}/` — expect 0 at chunk-open + **≥ 7** at chunk-close. This closes the D-philosophy-02:39 ≥ 3-hit invariant in full (not just partially).

### §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface (v2 F1.b + F2.b) | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | None — Composite enum + tags field are `Copy`/`Clone`; backfill migration is one-shot | **no** | N/A |
| **A2** | New IPC channel | None | **no** | N/A |
| **A3** | New pod-local resource | None — tags persist in SurrealDB | **no** | N/A |
| **A4** | Migration runner | NEW migration `0018_org_project_tags.surql`. Idempotent (UPDATE...SET tags = [...] WHERE tags IS NONE OR tags = [] semantics); single-runner invariant covered by CHK8S-D-05 | **no** | N/A |
| **A5** | Trait-shape requirement | F1.b handler refactor uses existing `handler_support::check_permission` indirection; F2.b adds `tags` field to existing serde-friendly structs — both trait-objects-compatible | **no** | N/A |
| **A6** | Cross-pod state sharing | Tags + catalogue entries persist in SurrealDB → durable cross-pod; handler `check_permission` reads catalogue + grants from SurrealDB (no cross-pod in-memory dependency) | **no** | N/A |
| **A7** | Audit hash-chain symmetry | No new audit-event writer in the canonical case. F1.b handler refactor may emit a NEW `platform.permission.denied` audit event on denial (if we wire it — currently `denial_to_api_error` does NOT emit audit; see ADR §D61.5 note); if emitted, the audit envelope MUST include only canonical BLAKE3-stable fields per CH-09 ADR. | **no** (canonical case) | If audit emission added: verify canonical-bytes definition excludes `prev_event_hash` per CH-09 ADR-0040 |

**Conclusion**: **K8s-neutral** — no new blockers under F1.b + F2.b. Backfill migration is idempotent + adds to the existing single-runner-migration discipline already covered by CHK8S-D-05.

### §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `m5_3/architecture/agent-ownership-model.md` | YES — needs "Composite-resource axis" addition + "Tag-based instance-identity" subsection (F2.b) + "Handler-routing through check_permission" subsection (F1.b) | **(a) update in-chunk** |
| **Architecture** | NEW `m5_3/architecture/org-project-composite-model.md` | YES — full architecture brief on the unified resource model: ontology variant, tags-based instance-identity, dual-nature framing, handler-routing through check_permission | **(a) create in-chunk** |
| **Operations** | NEW `m5_3/operations/composite-resource-operations.md` | YES — operator-facing playbook: how to inspect tags, backfill verification commands, migration rollback notes, handler-refactor blast-radius operator guide | **(a) create in-chunk** |
| **User-guide** | `m5/user-guide/first-session-walkthrough.md` | NO touch — session walkthrough is not Org/Project-resource-axis-aware | mark "no change" |
| **Concept doc** | `concepts/permissions/01-resource-ontology.md` | YES — Composite Classes table cardinality 8 → 10; add 2 rows | **(a) verified-header bump + 2-row table addition** |
| **Concept doc** | `concepts/core-philosophy.md` | YES — verified-header bump (line 16 + 28 + 29 now fully honored at composite + routing axes post-CH-26 F1.b + F2.b) | **(a) verified-header bump only** |
| **Drifts subtree** | `m5_3/drifts/_concept-audit-matrix.md` + `m5_3/drifts/README.md` + `D-philosophy-02.md` | YES | **(a) status flips at P-SEAL** |

### §3.D — Forward-scope-vs-concept-doc precedence

Verification per chunk-planner v8 §3.D mechanical procedure (v1 conclusion preserved):

1. **Forward-scope row literal text claims** unchanged from v1.
2. **Concept-doc invariant checks**: NO closed-set break introduced by F1.b or F2.b user-locks. `Composite::ALL.len() == 10` is controlled invariant evolution; `tags` field addition to Organization/Project structs is additive-with-default (no JSON wire-format break); ≥ 7 handlers invoke `check_permission` (the engine indirection is precedent-shipped at CH-09 + CH-25).
3. **ADR-0043 → 0061 amendment** documented in §D61.6.

**Verdict**: NO closed-set break under v2 F1.b + F2.b locks.

### §3.E — Anticipated gate-2.5 candidates (v2 refreshed for F1.b + F2.b)

(Per chunk-planner v13 §3.E — surface candidates likely to surface mid-flight at P-NEW-TESTS or P-DOCS authoring.)

- **Candidate 1 (v2 expanded)** — **Handler bespoke-gate "dead code" surfacing.** F1.b P2 handler refactor inserts `check_permission` before the existing bespoke gate. After verification that the engine gate is load-bearing-equivalent, the bespoke gate (e.g., `check_auth_request_access` at `show.rs:79`; CEO-only role check at `create.rs`) may become defence-in-depth-only OR fully redundant. P-DOCS authoring may surface this as cleanup-candidates. **Route**: option-A close-in-chunk via a P-DEDUP-GATES phase if ≤ 0.3 ed + bespoke gates have zero added invariant beyond engine semantics; option-B file follow-up drift `D-CH26-FOLLOWUP-02` with M6-DEFERRED-NN allocation for defensive-layer cleanup retro.
- **Candidate 2 (v2 expanded)** — **Migration 0018 backfill duration for high-row-count environments.** With F2.b adding the `tags` column AND populating it for every extant org/project row, large prod fixtures (CH-23 mass-fixture-load scenarios with 100s of orgs) may show migration runtime > 1s. P-MIGRATION-BACKFILL may need to surface batched-UPDATE strategy. **Route**: option-A inline-batch in the migration body (≤ 0.2 ed); option-B M7-DEFERRED-NN for production-scale migration tuning.
- **Candidate 3 (v2 expanded)** — **Acceptance harness `acceptance_common::admin::spawn_claimed_with_org_and_project` (`admin.rs:392`) must seed instance-identity tag + Edge::Owns for the new handler refactor.** F1.b handler refactor at `list_organizations` will Deny if the test actor is not the owner of the test org. P-NEW-TESTS may surface that ALL acceptance test setup helpers need a single-line update to populate instance-identity tag + write the Edge::Owns edge during fixture setup. **Route**: option-A close-in-chunk via P-HARNESS-EXTEND phase if ≤ 0.5 ed (likely ≤ 0.3 ed); option-B reject-by-test-failure (each acceptance test individually adopts the fixture-update).
- **Candidate 4 — Concept-doc-01 line 186 amendment.** P-DOCS authoring may surface that concept-doc-01 line 186 enumeration "All composites" still lists 6 variants (pre-Inbox/Outbox); the line needs explicit Organization + Project additions to reflect post-CH-26 cardinality 10. **Route**: option-A is a 2-line amendment in-chunk at P-DOCS.
- **Candidate 5 (NEW v2)** — **Per-handler audit-event emission.** F1.b handler refactor may surface user-preference for emitting `platform.permission.denied` audit events on each Deny (rather than only at engine internal logging). This would close the audit-trail observability gap for permission decisions. **Route**: option-A close-in-chunk via P-AUDIT-EMIT phase if ≤ 0.5 ed; option-B M6-DEFERRED-NN drift filing.

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-philosophy-02` | [`m5_3/drifts/D-philosophy-02.md`](../../../v0/implementation/m5_3/drifts/D-philosophy-02.md) | HIGH (Bucket A) | `discovered → remediated` | Implementation chunk CH-26 v2 under F1.b + F2.b user-locks. The `≥ 3 hits` post-remediation invariant on `grep -rn "check_permission" modules/crates/server/src/platform/{orgs,projects}/` (D-philosophy-02:39) is **FULLY MET** by F1.b — ≥ 7 handlers wired in-cycle, well above the 3-hit threshold. No M7-DEFERRED follow-up needed for the load-bearing remediation. |

**v2 note**: under F1.a (v1 planner-recommendation), a follow-up drift `D-CH26-FOLLOWUP-01` would have been filed at M7-DEFERRED-NN. Under F1.b user-lock (in-cycle handler refactor), the follow-up drift is **NOT filed** — the load-bearing remediation completes in CH-26. Only if gate-2.5 candidates 1 or 5 surface and route to option-B will follow-up drifts be filed.

## §5 — ADRs drafted

### ADR-0061 — Org/Project as Composite resources + `tags` field + ≥ 7 handler refactor (Proposed at P0; Accepted at P-SEAL)

**ADR number assignment**: Next-free = **ADR-0061** (latest shipped = 0060 from CH-25; forward-scope's predicted `0043` documentary-stale).

**Sub-decisions** (v2 re-drafted to reflect F1.b + F2.b user-locks):

- **§D61.1** — Composite enum cardinality flip 8 → 10. Add `Composite::OrganizationObject` + `Composite::ProjectObject` variants with `constituents() = [DataObject, IdentityPrincipal, Tag]`. Controlled invariant evolution (same shape as CH-25 ADR-0060 §D60.6's 71 → 72 EDGE_KIND_NAMES evolution). Invariant test renames + literal updates at 5 cardinality sites (Artifact B).
- **§D61.2 (REWRITTEN v2 — F2.b USER-LOCK)** — `tags: Vec<String>` field on Organization + Project structs. Add `pub tags: Vec<String>` (with `#[serde(default)]`) to `Organization` (nodes.rs:362) + `Project` (nodes.rs:460). Precedent: `Session.tags`, `Memory.tags`, `Channel.tags`, `AgentCredential.tags` at `nodes.rs:836, 1062, 1074, 1082, 1269`. Instance-identity tag (`organization:<uuid>` / `project:<uuid>`) populated at-creation-time via `apply_org_creation` / `apply_project_creation` extension; backfill migration populates extant rows. **Dual-nature framing preserved**: Org/Project remain Principals (per `principal_resource.rs:182-186`) AND become tag-bearing Composite resources. No `impl Resource for OrgId/ProjectId` — same as CH-25 ADR-0060 §D60.1 precedent. **Rationale for F2.b over F2.a** (recorded per user-lock at gate-1): richer wire-format with explicit state representation on the row; aligns with the existing `tags` field discipline across 5 other node-types; closes the load-bearing "instance carries its own identity tag" invariant at the row level (not just the catalogue level). **Pre-existing-behaviour preservation note (v11 strict form)**: *"Pre-existing scaffold preserved: 5 existing node-types ship `tags: Vec<String>` with `#[serde(default)]` (Session at CH-04 / Memory at CH-08 / Channel at CH-XX / AgentCredential at CH-XX). CH-26 extends the canonical pattern to Organization + Project; does not change the field-discipline mechanism."*
- **§D61.3 (REWRITTEN v2 — F2.b USER-LOCK)** — Migration `0018_org_project_tags.surql`. Three operations in one transaction: (a) `DEFINE FIELD tags ON TABLE organization TYPE array<string> DEFAULT [];` + same for project; (b) `UPDATE organization SET tags = ['organization:' + string::from(id)] WHERE tags IS NONE OR tags = [];` + same for project; (c) `seed_catalogue_entry_for_composite` call for every extant Org + Project row (idempotent via INSERT-OR-IGNORE). Idempotent under repeated runs (UPDATE filters on `tags IS NONE OR tags = []`; catalogue seed via UPSERT semantics).
- **§D61.4 (REWRITTEN v2 — F1.b USER-LOCK)** — Acceptance scope spans ENGINE-LEVEL + HANDLER-LEVEL. NEW acceptance file `server/tests/acceptance_m5_3_composite_resources.rs` with ≥ 8 scenarios: (1) catalogue-seed succeeds; (2) `Action::Allocate.applies_to_composite(Composite::OrganizationObject) == true`; (3) engine-level `handler_support::check_permission` resolves `[Allocate]` over `org:<O1>` via CH-25 synth-owner-grant rule; (4) stranger Agent → Deny with `NO_GRANTS_HELD`; (5-8+) **per-handler PASS/FAIL** for each of the ≥ 7 refactored handlers (e.g., `list_organizations_pass_for_owner`, `list_organizations_deny_for_stranger`, `show_organization_pass_for_owner`, `create_project_pass_for_org_member`, etc.). **v14 R1 handler-gating verification PASSED**: post-F1.b, `grep -nE 'check_permission' /root/projects/phi/baby-phi/modules/crates/server/src/platform/{orgs,projects}/` returns ≥ 7 hits — the literal "Permission Check matches `org:O` and `project:P` as resource types in selectors via handler chain" scenario IS exercisable through the refactored handler chain. **D-philosophy-02:39 ≥ 3-hit invariant FULLY MET** at chunk-seal (≥ 7 hits ≥ 3 threshold).
- **§D61.5 (REWRITTEN v2 — F1.b USER-LOCK)** — Handler refactor pattern. ≥ 7 handlers refactored: `orgs::list::list_organizations` (Observe), `orgs::show::show_organization` (Inspect), `orgs::create::create_organization` (Allocate on platform:root), `orgs::dashboard::org_dashboard` (Observe), `projects::create::create_project` (Allocate on org:O), `projects::detail::show_project` (Inspect), `projects::resolvers::*` (Observe), `projects::agent_supervisor::list_agent_supervisors` (Observe). Each handler (a) invokes `handler_support::check_permission(actor, action, resource)` before any state mutation or read-side filtering; (b) maps the result via `denial_to_api_error` to HTTP 403 with structured `NO_GRANTS_HELD` / `NO_APPLICABLE_GRANT` / `EXPLICIT_DENY` per CH-25 wire convention; (c) preserves the bespoke gate as defence-in-depth (subject to gate-2.5 Candidate 1 simplification). The selector resolves via the instance-identity tag on the row (F2.b tags field) + the catalogue entry (Step 0 precondition). **Pre-existing-behaviour preservation note (v11 multi-milestone-pattern variation)**: *"Pre-existing implementation refactored: `orgs/{list,show,create,dashboard}` + `projects/{create,detail,resolvers,agent_supervisor}` handlers shipped across M3-M5 with `AuthenticatedSession`-only gating + bespoke per-handler checks. CH-26 refactors the gating to flow through `handler_support::check_permission`; preserves bespoke gates as defence-in-depth. M3 + M4 + M5 acceptance suite fixtures extended (per §7 P2 blast-radius triage) to seed Edge::Owns + instance-identity tag before invoking handlers."*
- **§D61.6** — ADR number amendment META. Forward-scope §2.5 line 270 predicted `ADR-0043`. Actual next-free is ADR-0061. META sub-decision documents the +18 amendment.
- **§D61.7 (REWRITTEN v2 — F1.b absorbs F1 deferral META)** — F1.b in-cycle remediation. Under F1.b user-lock, no M7-DEFERRED follow-up drift is filed for the handler-refactor scope. The D-philosophy-02:39 ≥ 3-hit invariant is fully met in-cycle. **Pre-existing-behaviour preservation note (v11 strict form)**: *"Shipped at CH-26 v2 P2 close (date 2026-MM-DD); CH-26 v2 does not change other handlers beyond the enumerated ≥ 7. The forward-scope §2.5 line 268 step 3 is fully honored in-cycle; the M7 chunks (admin-pages forward-scope §3) inherit the canonical pattern for future handler additions."*

**Cross-references**:
- (a) **Originating concept-doc + section + line range**: `concepts/core-philosophy.md:16` (Organization has Projects as Resource Type) + `concepts/permissions/01-resource-ontology.md:30-43` (Composite Classes table) + `concepts/permissions/01-resource-ontology.md:222-263` (Instance Identity Tags).
- (b) **Closed drift(s)**: `D-philosophy-02` (HIGH-A).
- (c) **Prior ADRs cited as precedent** (milestone-prefixed paths per chunk-planner v6):
  - [`m5_3/decisions/0060-agent-as-creator-and-owner.md`](../../../v0/implementation/m5_3/decisions/0060-agent-as-creator-and-owner.md) — CH-25 owner-grant rule + Edge::Owns wire-up.
  - [`m1/decisions/0008-permission-check-as-pipeline.md`](../../../v0/implementation/m1/decisions/0008-permission-check-as-pipeline.md) — pipeline invariants.
  - [`m1/decisions/0012-forward-only-migrations.md`](../../../v0/implementation/m1/decisions/0012-forward-only-migrations.md) — idempotent-migration discipline.
  - [`m3/decisions/0022-org-creation-compound-transaction.md`](../../../v0/implementation/m3/decisions/0022-org-creation-compound-transaction.md) — compound-tx pattern.
  - [`m2/decisions/0018-handler-support-module.md`](../../../v0/implementation/m2/decisions/0018-handler-support-module.md) — `handler_support::check_permission` entrypoint.
- (d) **Forward-scope row**: §2.5 lines 260-271.

**Expected flip-to-Accepted phase**: P-SEAL.

## §6 — Prior-chunk regression re-verification

(unchanged from v1; preserved verbatim)

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-25** | Edge::Owns variant extant + EDGE_KIND_NAMES.len() == 72 | `grep -nE 'Edge::Owns\b' .../edges.rs` ≥ 1; `grep -nE 'EDGE_KIND_NAMES.*72' .../edges.rs` ≥ 2 |
| **CH-25** | `list_agent_owned_orgs` + `list_agent_owned_projects` | `grep -nE 'list_agent_owned_(orgs\|projects)' .../repository.rs` ≥ 2 |
| **CH-25** | engine.rs synth-owner-grant rule emits `org:<id>` / `project:<id>` ResourceRef.uri | `grep -nE 'format!\("(org\|project):' .../engine.rs` ≥ 4 |
| **CH-25** | acceptance `m5_3_ceo_synth_owner_grant_resolves_allocate_over_owned_org` green | `cargo test -p server --test acceptance_m5_3_owner_grant` 1 passed |
| **CH-15** | `Action::CANONICAL.len() == 34` | `grep -nE 'Action::CANONICAL.len\(\) == 34' .../action.rs` ≥ 1 |
| **CH-12** | Composite enum count test | `grep -nE 'all_contains_exactly_eight' .../composites.rs` 1 hit at open; renamed `_ten` at close |
| **CH-06** | `seed_catalogue_entry_for_composite` callable | `grep -nE 'seed_catalogue_entry_for_composite' .../repo_impl.rs` ≥ 1 |
| **CH-05** | Principal-only invariant preserved | `grep -nE 'impl Resource for OrgId' .../principal_resource.rs` 0 hits open/close |
| **CH-04** | trybuild compile_fail fixtures green | `cargo test -p domain --test edge_type_safety` green |

## §7 — Phases within the chunk (v2 re-plan)

### P0 — Scaffolding + plan archive + ADR Proposed + pre-conditions re-verify

- **Goal**: scaffold the cycle folder, archive this plan (v2), draft ADR-0061 with sub-decisions D61.1-D61.7 Proposed, re-verify §6 regression table at chunk open.
- **Deliverables**:
  1. `docs/specs/plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md` (this v2 file).
  2. ADR-0061 Proposed.
  3. Cycle-index row.
  4. P0 baseline verified: `Composite::ALL.len() == 8` + 1556/0/2 tests + phi-core imports 57.
- **Tests**: no new tests; baseline re-verification only.
- **Confidence target**: 100%.
- **Pause discipline**: pause if baseline test count != 1556 OR phi-core imports != 57 OR `Composite::ALL.len() != 8` at P0 verify.

### P1 — Composite enum cardinality 8 → 10 + invariant cascade (Artifact A + Artifact B)

(unchanged from v1)

- **Goal**: ship the two new Composite variants + flip the 5 cardinality-literal sites + add 2 arms each to `as_str()` / `kind_tag()` / `constituents()`.
- **Deliverables**: 6 edit-blocks per v1 P1.
- **Tests**: ~10 existing tests in `composites.rs` auto-flip; 2-3 explicit literal counts flip. Test delta: 0.
- **Confidence target**: ≥ 99%.
- **Pause discipline**: pause if cascade hits > 42 sites OR if `Composite::ALL` invariant test fails after rename.

### P-FIELD-EXTEND (NEW v2 — F2.b USER-LOCK) — `tags: Vec<String>` field add + struct-literal cascade + materialise paths

- **Goal**: ship the `tags: Vec<String>` field addition to Organization + Project structs, propagate through ~51 struct-literal callsites (35 Org + 16 Project per Artifact E.1 + E.2), update InMemoryRepository + SurrealStore materialise paths.
- **Deliverables**:
  1. `domain/src/model/nodes.rs:362` — add `#[serde(default)] pub tags: Vec<String>,` to `Organization` struct.
  2. `domain/src/model/nodes.rs:460` — add `#[serde(default)] pub tags: Vec<String>,` to `Project` struct.
  3. PRODUCTION construction sites:
     - `server/src/platform/orgs/create.rs:230` — populate `tags: vec![format!("organization:{}", id)]` in `apply_org_creation` materialise.
     - `server/src/platform/projects/create.rs:366` — populate `tags: vec![format!("project:{}", id)]` in `apply_project_creation` materialise.
     - `cli/src/main.rs:101` — `tags: vec![]` (CLI demo; instance-identity not load-bearing).
  4. TEST fixture cascade (per Artifact E.1 + E.2 per-file breakdown): ~32 Organization fixtures + ~13 Project fixtures = ~45 fixture sites add `tags: vec![]` (test-mode minimal fixtures; harness sites in `acceptance_common/admin.rs` populate instance-identity tag to mirror production semantics).
  5. InMemoryRepository materialise updates: `domain/src/in_memory.rs::apply_org_creation` + `::apply_project_creation` (deserialise + re-serialise with new field).
  6. SurrealStore materialise updates: `store/src/repo_impl.rs::apply_org_creation` + `::apply_project_creation` SurrealQL INSERT bodies extended with `tags` column.
- **Tests**: ~1 NEW unit test confirming `tags` field round-trips through `#[serde(default)]` deserialisation of pre-CH-26 JSON; existing tests auto-cover (fixtures updated above).
- **Concept-alignment check**: §2 row "Instance Identity Tags" transitions `silent-in-code` → `fully-honored`.
- **phi-core leverage check**: zero delta.
- **User-facing doc updates**: none in this phase.
- **Confidence target**: ≥ 95% (large fixture cascade; mechanical but volume-heavy — chunk-planner v3 cascade prediction applies; pause threshold 142 sites for combined Artifact A+B+D+E).
- **Pause discipline**: pause via AskUserQuestion if (a) cascade sites > 142 (the 1.5× combined threshold) OR (b) any `serde_json` test surfaces a wire-format break NOT covered by `#[serde(default)]` (suggests a hidden snapshot test).

### P2 (WIDENED v2 — F1.b USER-LOCK) — Backfill migration + compound-tx extensions + ≥ 7 handler refactor

- **Goal**: ship migration `0018_org_project_tags.surql` (column add + backfill + catalogue seed) + extend compound-tx + refactor ≥ 7 handlers to invoke `handler_support::check_permission`.
- **Deliverables**:
  1. `modules/crates/store/migrations/0018_org_project_tags.surql` — three-operation transaction per §D61.3.
  2. `store/src/migrations.rs` — register `0018` slug `org_project_tags`.
  3. `store/src/repo_impl.rs::apply_org_creation` body — after the org-row INSERT, call `seed_catalogue_entry_for_composite(None, "org:<id>", Composite::OrganizationObject)?`.
  4. `store/src/repo_impl.rs::apply_project_creation` body — analogous for Project.
  5. `domain/src/in_memory.rs::apply_org_creation` + `::apply_project_creation` bodies — mirror seed calls.
  6. **F1.b handler refactor — ≥ 7 handlers refactored (per Artifact F)**:
     - `orgs::list::list_organizations` — wire `check_permission(actor, Action::Observe, ResourceRef::uri("org:*"))` (scope-to-owned via `list_agent_owned_orgs`).
     - `orgs::show::show_organization` — wire `check_permission(actor, Action::Inspect, ResourceRef::uri(format!("org:{}", id)))`.
     - `orgs::create::create_organization` — wire `check_permission(actor, Action::Allocate, ResourceRef::uri("platform:root"))` (platform-admin synth-grant path).
     - `orgs::dashboard::org_dashboard` — wire `check_permission(actor, Action::Observe, ResourceRef::uri(format!("org:{}", id)))`.
     - `projects::create::create_project` — wire `check_permission(actor, Action::Allocate, ResourceRef::uri(format!("org:{}", parent_org)))`.
     - `projects::detail::show_project` — wire `check_permission(actor, Action::Inspect, ResourceRef::uri(format!("project:{}", id)))`.
     - `projects::resolvers::*` (project-tier read resolvers) — wire `check_permission(actor, Action::Observe, ResourceRef::uri(format!("project:{}", id)))`.
     - `projects::agent_supervisor::list_agent_supervisors` — wire `check_permission(actor, Action::Observe, ResourceRef::uri(format!("project:{}", id)))`.
  7. **Acceptance-suite blast-radius triage**: each affected acceptance test in M3 + M4 + M5 suites either (a) gains a fixture-setup line writing `Edge::Owns` between the test actor + the test org (CH-25 owner-grant synth-rule then resolves Allow) OR (b) explicitly seeds a persisted grant via `apply_consent` for cases where the actor is not the owner. Helper in `acceptance_common/admin.rs::spawn_claimed_with_org_and_project` extended to write Edge::Owns by default.
- **Tests**: NEW migration-idempotency test + NEW load-bearing acceptance scenarios per §D61.4 (8 total). Existing acceptance suites: triage applied above.
- **Concept-alignment check**: §2 row "core-philosophy.md line 16" transitions `partially-honored` → `fully-honored`.
- **phi-core leverage check**: zero delta.
- **User-facing doc updates**: none in this phase.
- **Confidence target**: ≥ 92% (HIGH-blast-radius phase; ≥ 7 handler refactor + acceptance suite triage).
- **Pause discipline**: pause via AskUserQuestion if (a) more than 12 distinct acceptance test files break in unanticipated ways (e.g., expect-200 → 403 unexpectedly even after Edge::Owns fixture extension); OR (b) any handler refactor surfaces a structural mismatch between `check_permission` semantics and the existing bespoke gate (e.g., bespoke check has a wider permissive arc than `Action::Observe`); OR (c) the migration body errors on existing fixtures.

### P3 — Action-applicability smoke tests + load-bearing acceptance + per-handler PASS/FAIL scenarios

- **Goal**: ship the per-§D61.4 acceptance test suite covering ENGINE-LEVEL invariants + HANDLER-LEVEL PASS/FAIL.
- **Deliverables**:
  1. `domain/src/permissions/action.rs` — +2 unit-test smoke assertions for new variants.
  2. NEW `server/tests/acceptance_m5_3_composite_resources.rs` with ≥ 8 scenarios (4 engine-level per v1 + 4-8 per-handler per F1.b ≥ 7 handlers).
- **Tests**: ~8-12 NEW acceptance tests + 1-2 NEW unit tests = **+10 to +14 tests** in P3 alone.
- **Confidence target**: ≥ 95%.
- **Pause discipline**: pause if any handler PASS/FAIL scenario produces an unexpected verdict (suggests structural mismatch between Action verb mapping + handler semantics).

### P-DOCS — User-facing docs + concept-doc amendments + drift housekeeping

(unchanged from v1)

- **Deliverables**: 3 user-facing tier docs + 2 concept docs touched.
- **Confidence target**: ≥ 99%.

### P-SEAL — ADR flip + drift transition + cycle-index + verified-headers

(unchanged from v1 EXCEPT: no NEW follow-up drift filed under F1.b user-lock; D-philosophy-02 fully remediated.)

- **Deliverables**: ADR-0061 Accepted; D-philosophy-02 remediated; cycle-index; verified-headers.
- **Confidence target**: ≥ 99%.

## §8 — Tests summary (v2 widened bands)

**Expected total test count at chunk close**: **[1568, 1574]** (v2 widened from v1's [1561, 1563]).

**Test count delta band breakdown** per chunk-planner v8 ×1.0-×1.20 asymmetric buffer:
- Deliverable-listed sum estimate: **+12 to +18 NEW tests**:
  - P-FIELD-EXTEND: +1 (serde-default round-trip).
  - P2: +1 (migration_0018_idempotent).
  - P2 handler-refactor regression-absorbing acceptance fixture extensions: +0 (existing test counts unchanged; fixtures rewired).
  - P3 engine-level acceptance: +4 (per §D61.4 (1)-(4)).
  - P3 per-handler PASS/FAIL: +6 to +12 (≥ 3 PASS + ≥ 3 DENY pairs across ≥ 7 handlers — minimum ≥ 6 tests; with full per-handler coverage ≥ 12 tests).
  - Cross-handler regression coverage: +0 to +2.
- Lower bound: 1556 + 12 = **1568**.
- Upper bound: 1556 + 18 = **1574**.
- Outside [1568, 1574] → AskUserQuestion at gate-2.

**Layer breakdown**:
- Unit: +1 to +2 (`applies_to_composite` smoke + `Organization::tags` serde-default round-trip).
- Integration: +1 (`migration_0018_idempotent`).
- Acceptance (engine-level): +4.
- Acceptance (per-handler PASS/FAIL): +6 to +12.

**MUST-SHIP** (per chunk-planner v9 — files-on-disk by chunk-seal):
- `server/tests/acceptance_m5_3_composite_resources.rs` with ≥ 8 test functions (4 engine + ≥ 4 per-handler).
- `modules/crates/store/migrations/0018_org_project_tags.surql`.
- ADR-0061 file.
- Tags field present on `Organization` + `Project` struct definitions in `nodes.rs`.
- ≥ 7 handlers in `server/src/platform/{orgs,projects}/` invoke `check_permission` (grep ≥ 7 hits at chunk-seal).

**Named expected-still-green tests** (carry-forward + regression absorbing — F1.b blast-radius):
- `acceptance_m5_3_owner_grant::m5_3_ceo_synth_owner_grant_resolves_allocate_over_owned_org` (CH-25 — must stay green after F1.b handler refactor).
- `composites::tests::all_contains_exactly_ten` (renamed).
- `manifest_validator` test suite (~30 tests).
- All 5 trybuild compile_fail fixtures.
- M3 + M4 + M5 acceptance suites (extended with Edge::Owns fixture seeding per P2 deliverable 7).

## §9 — Pre-chunk gate

(unchanged from v1 — same reading list)

**Carry-forward invariants** (verified green at chunk open P0 — 2026-05-17):
- All v1 §9 invariants ✓.

**Pending decisions carried into v2**:
- F1.b user-lock honored ✓.
- F2.b user-lock honored ✓.
- F3 default 3 auditors ✓.
- D-philosophy-02 `discovered → remediated` transition owed at P-SEAL (no M7 follow-up under F1.b).

## §10 — Close criteria (v2 refreshed)

**Composite 4-aspect + 2 confidence % ritual.**

**4 aspects**:
- **Code aspect**: P0 + P1 + P-FIELD-EXTEND + P2 + P3 + P-DOCS + P-SEAL deliverables shipped; `cargo test --workspace` green at expected count [1568, 1574]; clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect**:
  - *Governance tier*: D-philosophy-02 fully remediated (no M7 follow-up drift); ADR-0061 Accepted; cycle-index row.
  - *User-facing tier*: §3.C map satisfied.
- **phi-core leverage aspect**: import count = 57 (Δ +0); `check-phi-core-reuse.sh` green; zero forbidden-duplication grep hits.
- **Concept alignment aspect**: every §2 row's target-status achieved.

**2 confidence %**:
- **Implementation confidence**: target ≥ **9.5/10** (= 19/20 claims honored). Numerator widened to absorb F1.b + F2.b deliverables: (1) Composite cardinality flip; (2) constituents() body; (3) Org `tags` field; (4) Project `tags` field; (5) Org tags backfill via migration; (6) Project tags backfill via migration; (7) catalogue seed at-creation-time × 2 backends; (8) ≥ 7 handler refactors hit `check_permission` grep; (9) engine-level acceptance 4 scenarios; (10) per-handler PASS acceptance ≥ 4 scenarios; (11) per-handler DENY acceptance ≥ 4 scenarios; (12) action-applicability smoke for new variants; (13) ADR-0061 7 sub-decisions; (14) cycle-index row; (15) verified-headers; (16) doc-sync sweep clean; (17) M3/M4/M5 acceptance suite green after fixture triage; (18) trybuild fixtures green; (19) Principal-only invariant preserved; (20) `tags` serde-default round-trip green. Denominator: same 20.
- **Documentation confidence**: target ≥ **9/10**.

**Composite target**: min ≥ **0.9**.

**P4 paperwork checklists**:
- Every modified doc's verified-header description matches body diff exactly.
- `_concept-audit-matrix.md` Status column copy-pasted letter-for-letter from §2 target.
- Cycle-index row inserted.
- Cargo-clean discipline placement (1) immediate-post-test cleanup after each `cargo test --workspace` invocation (per CH-18 retro Row 1).

## §11 — Post-chunk independent audit plan (v2 widened claims)

**Phase count: 7.** Per audit-envelope-size skill: **Large (3 auditors)** — locked at gate-1 (F3).

### Audit A (code + phi-core + cascade) prompt scaffold (v2 widened for F1.b + F2.b)

You are auditing CH-26 v2 in baby-phi. Read-only. Plan at `docs/specs/plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md`.

Verify each claim with file:line citation. PASS/FAIL each:

1. `Composite::OrganizationObject` + `Composite::ProjectObject` extant; `ALL` array cardinality 10; invariant test renamed `all_contains_exactly_eight → _ten`; 5 cardinality literal sites flipped.
2. `as_str()` arms emit `"organization_object"` + `"project_object"`; `kind_tag()` emits `"#kind:organization"` + `"#kind:project"`; `constituents()` returns `[DataObject, IdentityPrincipal, Tag]`.
3. **F2.b — Organization struct has `tags: Vec<String>` field with `#[serde(default)]` at `nodes.rs:362`-ish range**. Project struct has same at `nodes.rs:460`-ish range.
4. **F2.b — Production construction sites populate instance-identity tag**: `server/src/platform/orgs/create.rs:230`-ish region writes `tags: vec![format!("organization:{}", ...)]`; `server/src/platform/projects/create.rs:366`-ish region writes `tags: vec![format!("project:{}", ...)]`.
5. **F2.b — Fixture cascade complete**: cargo test passes (compile-time enforcement) → all ~45 fixture sites have `tags` field populated.
6. **F2.b — Migration `0018_org_project_tags.surql` extant** + registered + idempotent (verified by `migrations_test::migration_0018_idempotent`).
7. `apply_org_creation` + `apply_project_creation` in BOTH backends call `seed_catalogue_entry_for_composite` at-creation-time (4 callsites total).
8. **F1.b — ≥ 7 handlers in `server/src/platform/{orgs,projects}/` invoke `check_permission`**: `grep -rnE 'check_permission' modules/crates/server/src/platform/{orgs,projects}/ | wc -l` ≥ 7. Per-handler verification: each of `list_organizations`, `show_organization`, `create_organization`, `org_dashboard`, `create_project`, `show_project`, `list_agent_supervisors` (and any project resolver) cites `check_permission` invocation site.
9. **F1.b — `denial_to_api_error` mapping** invoked on `Err(PermissionDenial)` in each refactored handler.
10. phi-core imports = 57; `check-phi-core-reuse.sh` exit 0.
11. `cargo test --workspace` test count in [1568, 1574].
12. CI guards green (4 scripts).
13. Trybuild compile_fail fixtures green; Principal-only invariant preserved (no `impl Resource for OrgId/ProjectId`).
14. Cascade verification: `grep -nE 'Composite::(OrganizationObject|ProjectObject)' modules/crates/ | wc -l` ≥ 6.

PASS/FAIL each. ≤ 600 words.

### Audit B (docs + concept + ADR) prompt scaffold (v2 refreshed for F1.b + F2.b sub-decisions)

You are auditing CH-26 v2's concept-fidelity + docs-fidelity. Read-only.

Verify each claim:

1. ADR-0061 Accepted with sub-decisions D61.1-D61.7 ratified. **§D61.2 documents F2.b USER-LOCK** (tags field with rationale); **§D61.4 documents per-handler acceptance scenarios** (≥ 4 PASS + ≥ 4 DENY); **§D61.5 documents F1.b USER-LOCK** (handler refactor pattern); **§D61.7 documents F1.b absorbs F1 deferral** (no M7 follow-up filed).
2. Drift D-philosophy-02 Status = remediated; lifecycle entry for CH-26 v2 chunk-seal with cycle hex `d1cb9e1f` + 2026-MM-DD present. **No new drift `D-CH26-FOLLOWUP-01` filed under F1.b user-lock** (confirm absence).
3. `m5_3/drifts/README.md` open count 1 → 0; remediated count 1 → 2. `_concept-audit-matrix.md` rows match §2 target.
4. Concept doc `permissions/01-resource-ontology.md` Composite Classes table cardinality 8 → 10. Verified-header bumped.
5. Concept doc `core-philosophy.md` verified-header bumped (line 16 + 28 + 29 now fully honored at routing axis under F1.b).
6. NEW `m5_3/architecture/org-project-composite-model.md` extant + cross-links + **§"Tag-based instance-identity" subsection (F2.b)** + **§"Handler-routing through check_permission" subsection (F1.b)**.
7. NEW `m5_3/operations/composite-resource-operations.md` extant + operator playbook including handler-refactor blast-radius guide.
8. `m5_3/architecture/agent-ownership-model.md` amendment subsection present.
9. ADR-0061 §D61.6 META documents ADR-0043 → 0061 amendment.
10. Cycle-index row extant.

PASS/FAIL each. ≤ 600 words.

### Audit C (carry-forward regression + handler-refactor blast-radius safety) prompt scaffold (v2 widened)

You are auditing CH-26 v2's carry-forward regression posture under F1.b handler refactor blast radius. Read-only.

Verify each claim:

1. CH-25 acceptance `acceptance_m5_3_owner_grant::m5_3_ceo_synth_owner_grant_resolves_allocate_over_owned_org` still green after F1.b refactor.
2. `composites::tests::*` all green after rename + cardinality flip.
3. Migration test suite green: migration `0018_org_project_tags` registered.
4. `EDGE_KIND_NAMES.len() == 72` invariant intact.
5. `Action::CANONICAL.len() == 34` invariant intact.
6. Migration `0018` idempotent: replay against an already-backfilled DB does not duplicate tags or catalogue entries.
7. **F2.b — `tags` field serde-default round-trip**: pre-CH-26 JSON deserialises with `tags = vec![]` correctly.
8. **F1.b — M3 acceptance suite blast radius**: `acceptance_m3_*` tests green after fixture extensions (Edge::Owns seeding) — list any FAIL test.
9. **F1.b — M4 acceptance suite blast radius**: `acceptance_m4*` tests green after fixture extensions — list any FAIL test.
10. **F1.b — M5 acceptance suite blast radius**: `acceptance_m5_*` tests green after fixture extensions — list any FAIL test.
11. **F1.b — Per-handler PASS/FAIL scenarios in new `acceptance_m5_3_composite_resources.rs`**: ≥ 4 PASS scenarios (owner can Observe/Inspect/Allocate); ≥ 4 DENY scenarios (stranger denied with `NO_GRANTS_HELD`).
12. trybuild compile_fail fixtures: 5 fixtures green.
13. ADR-0060's `acceptance_m5_3_owner_grant` test's catalogue-seed precondition unbroken: the new `apply_org_creation`'s catalogue seed + tags population at `org:<id>` matches the test's lookup pattern.
14. Manifest validator suite (~30 tests) green.

PASS/FAIL each. ≤ 600 words.

## §12 — Verification recipe (v2 extended)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (4)
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1

# 3. Cargo cleanup (per CH-18 retro Row 1, placement 1 — immediate-post-test)
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 4. Chunk-specific phi-core leverage
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (baseline; Δ +0 as predicted in §3)

# 5. Composite enum cardinality
grep -nE 'pub const ALL: \[Composite; 10\]' /root/projects/phi/baby-phi/modules/crates/domain/src/model/composites.rs
# Expect: 1 hit

# 6. New variants extant
grep -nE 'Composite::(OrganizationObject|ProjectObject)' /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: ≥ 6 hits

# 7. F2.b — tags field extant on Organization + Project
grep -nE 'pub tags: Vec<String>' /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs | wc -l
# Expect: ≥ 7 hits (Session/Memory/Channel/AgentCredential precedents = 5 + Org/Project new = 2; total ≥ 7)

# 8. F2.b — production sites populate instance-identity tag
grep -nE 'tags: vec!\[format!\("organization:' /root/projects/phi/baby-phi/modules/crates/server/src/platform/orgs/create.rs
# Expect: ≥ 1 hit
grep -nE 'tags: vec!\[format!\("project:' /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/create.rs
# Expect: ≥ 1 hit

# 9. F2.b — zero leftover `tags: Vec::new()` placeholders in production paths (anti-placeholder check)
grep -rnE 'tags: Vec::new\(\)' /root/projects/phi/baby-phi/modules/crates/server/src/platform/{orgs,projects}/
# Expect: 0 hits (production code must use vec![format!(...)] not Vec::new())

# 10. F2.b — Migration extant + registered
ls /root/projects/phi/baby-phi/modules/crates/store/migrations/0018_org_project_tags.surql
/root/rust-env/cargo/bin/cargo test -p store --test migrations_test migration_0018_idempotent

# 11. F1.b — ≥ 7 handlers invoke check_permission
grep -rnE 'check_permission' /root/projects/phi/baby-phi/modules/crates/server/src/platform/orgs/ /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/ | wc -l
# Expect: ≥ 7 hits (D-philosophy-02:39 invariant fully met)

# 12. F1.b — denial_to_api_error mapping wired
grep -rnE 'denial_to_api_error' /root/projects/phi/baby-phi/modules/crates/server/src/platform/orgs/ /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/ | wc -l
# Expect: ≥ 7 hits

# 13. Catalogue seeding wired at-creation-time
grep -nE 'seed_catalogue_entry_for_composite\(.*OrganizationObject\)' /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs
# Expect: 2 hits
grep -nE 'seed_catalogue_entry_for_composite\(.*ProjectObject\)' /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs
# Expect: 2 hits

# 14. Acceptance test green
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_m5_3_composite_resources -- --test-threads=1
# Expect: ≥ 8 passed (4 engine + ≥ 4 per-handler)

# 15. CH-25 carry-forward regression
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_m5_3_owner_grant -- --test-threads=1
# Expect: 1 passed (unchanged)

# 16. M3/M4/M5 acceptance blast-radius regression (F1.b)
/root/rust-env/cargo/bin/cargo test -p server --tests -- --test-threads=1
# Expect: all green after fixture triage; if any FAIL, name them in audit

# 17. Drift-file status
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D*.md | wc -l
# Expect: 2 (D-philosophy-01 from CH-25 + D-philosophy-02 from CH-26)

# 18. No new follow-up drift filed (F1.b user-lock — confirm absence)
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-*.md 2>/dev/null
# Expect: 0 files (F1.b in-cycle remediation, no follow-up under user-lock)

# 19. Cycle-index row
grep -n d1cb9e1f /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# Expect: ≥ 1 hit
```

---

**End of plan (v2).**
