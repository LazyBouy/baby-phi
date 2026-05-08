<!-- Last verified: 2026-05-07 by Claude Code (CH-08 plan-approval gate; user locked F1.A / F2.A / F3.A / F4.A / F5.A via AskUserQuestion; cycle hex `7cbe74a4`) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P0 plan-draft; iter 1) -->

# CH-08 — `allocate` / `transfer` cardinality + refinements (cycle `7cbe74a4`)

**Severity:** ⚠ HIGH
**Estimated effort:** ~2 engineer-days
**Forward-scope row:** [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 91–96.
**Drifts closed:** **D-new-13** (HIGH, A) + **D-new-29** (LOW, B).
**ADR slot:** ADR-0052.
**Prerequisites:** **CH-04** (typed actions — sealed at `ch-04-typed-action-vocabulary-3a65a2fc.md`; `Action::Allocate` + `Action::Transfer` already live at `action.rs:57-58`).

---

## Forks for orchestrator

This chunk has **5 forks** the planner cannot resolve from forward-scope + concept docs alone. **All require user lock at plan-approval gate.**

> **User-lock outcome (2026-05-07, via AskUserQuestion at plan-approval gate)**:
> - **F1 → F1.A** (`Repository::apply_transfer_grant` compound-tx primitive — mirrors ADR-0022/0023 precedent).
> - **F2 → F2.A** (`{ no_further_delegation: bool, max_depth: Option<u8> }` minimal v0 field set).
> - **F3 → F3.A** (`Grant.allocate_refinement: Option<AllocateRefinement>` with `#[serde(default)]` — mirrors CH-11 D48.1 + CH-13 D50.5 precedents).
> - **F4 → F4.A** (zero migrations — direct consequence of F3.A).
> - **F5 → F5.A** (forward-defensive primitive; no production caller wired — mirrors CH-12 frozen-tag-write precedent).
>
> All 5 locked at planner-recommendation. Direct-approval criteria hold under the all-A path. ADR-0052 §"Forks" header records these outcomes per CH-13 retro Row 1.

### F1 — Where does cardinality enforcement live?

The forward-scope row says *"transfer revokes sender's grant atomically"* but does not pin the enforcement boundary. Three candidate sites; concept-doc 02 line 206 favours (a), structural-invariant logic favours (b).

- **F1.A — `Repository::apply_transfer_grant` compound-tx primitive (NEW).** Add a new compound-tx Repository method modelled on the existing `apply_org_creation` precedent (`repository.rs:225`, ADR-0022/0023). Method body atomically: rewrites the `OWNED_BY` edge for the resource, revokes the sender's matching `[transfer]`-eligible grant, mints the recipient grant. `Allocate`-grant minting goes through the existing `create_grant` boundary unchanged (additive, sender's grant untouched — no enforcement step needed because `create_grant` is already additive by construction). **Planner-recommended.** Rationale: structural enforcement at the boundary the data crosses (single Repository call); concept-doc 02 line 206's *"on approval, rewrites the OWNED_BY edge and revokes any residual authority"* is a single transactional unit; mirrors ADR-0022's compound-tx pattern; no new K8s blocker (single-writer SurrealDB transaction); zero callsite cascade today (`Action::Transfer` has zero runtime mint sites — verified `grep -rn "Action::Transfer" modules/crates/`); the surface ships **forward-defensively** to make the first transfer-grant-mint chunk in M6+ a single `apply_transfer_grant` callsite.
- **F1.B — Inline cardinality branch in `AuthRequest` slot-aggregation flow (`auth_requests/transitions.rs::transition_slot`).** Each time a slot transition closes a request to `Approved` carrying a `[transfer]` action, the transition function emits a side-effect (or returns a payload) signalling the caller to revoke sender's grant. Rejected: `transitions.rs` is **pure** — every fn takes `&AuthRequest` and returns `Result<AuthRequest, _>` (verified at `transitions.rs:1-9`). Adding repository side-effects breaks the proptest invariant + concept doc 02's "AuthRequest is the slot-machine, grant-mint is downstream of approval" framing.
- **F1.C — Engine-side runtime invariant at `step_5_scope_resolution`.** Treat cardinality as a runtime check ("if a `[transfer]`-typed grant is held by 2 principals on the same resource, deny"). Rejected: this is **detection, not enforcement** — the broken state is already in storage. Concept-doc 02's Rust-analogue framing (`let y = x;`) is a write-time guarantee, not a read-time check.

**Auto-approval impact:** F1.A is the planner-recommended fork; if user locks F1.A → no migration, no K8s axis impact, no phi-core delta — plan stays inside Direct-approval criteria. **If user diverges to F1.B or F1.C, escalate.**

### F2 — `AllocateRefinement` field set

Concept-doc 02 line 197 names `no_further_delegation` as the canonical example. The forward-scope row also names `max_depth`. Concept-doc 03 line 54 talks about *"refinements expressed as constraints"* without naming a closed list.

- **F2.A — Minimal v0 field set: `{ no_further_delegation: bool, max_depth: Option<u8> }`.** **Planner-recommended.** Rationale: minimal closure of the two fields the forward-scope row + concept doc 02 line 197 explicitly name. `no_further_delegation` is a hard binary per concept doc; `max_depth: Option<u8>` is the natural generalisation (concept doc says *"one-level shareholder"* — `max_depth = Some(1)` operationalises that exactly, with `None` = unbounded). All other fields stay deferred to drift-list expansion when concrete use-cases land.
- **F2.B — Expanded set: F2.A fields + `restrict_to_resources: Vec<ResourceRef>`, `restrict_to_scopes: Vec<...>`, etc.** Rejected: concept doc names neither; speculative; v0 should ship minimal.
- **F2.C — Migration-only (define `AllocateRefinement` empty struct now, fill fields in CH-NN+).** Rejected: defeats the chunk's purpose (D-new-29 explicitly names `no_further_delegation`).

**Auto-approval impact:** F2.A is minimal + planner-recommended; if locked, no Direct-approval criterion fails.

### F3 — Constraint encoding for `AllocateRefinement`

Verified at `model/nodes.rs:650-694`: **`Grant` does NOT have a `constraints` field today**. Constraints live exclusively on `ToolAuthorityManifest.constraints: Vec<String>` at `nodes.rs:1008`. So the forward-scope row's *"`AllocateRefinement` ... structured constraint type"* must land somewhere new on `Grant`.

- **F3.A — New `Grant.allocate_refinement: Option<AllocateRefinement>` field, gated by `#[serde(default)]`.** **Planner-recommended.** Rationale: mirrors the CH-11 / ADR-0048 §D48.1 precedent of adding `approval_mode: ApprovalMode` directly to Grant (verified at `nodes.rs:674` → `Grant.approval_mode`) and the CH-13 / ADR-0050 §D50.5 precedent of adding `audit_class: AuditClass` directly to Grant (verified at `nodes.rs:692` → `Grant.audit_class`). Both prior fields ship with `#[serde(default)]` shielding; pre-CH-08 grants decode as `allocate_refinement: None`. Zero migration needed (SurrealDB schemaless persistence absorbs the new field; pre-CH-08 stored grants round-trip as `None`).
- **F3.B — Reserved-key in a new `Grant.constraint_context: HashMap<String, serde_json::Value>` map.** Adds a new field of a different shape; less typed; would require a parallel typed accessor anyway.
- **F3.C — New `Constraint` enum with variant `AllocateRefinement(AllocateRefinement)`.** Rejected: no `Constraint` enum exists today (`Vec<String>` on Manifest is the closest); inventing one as part of CH-08 inflates scope to ~3× forward-scope.

**Auto-approval impact:** F3.A reuses two existing precedents (D48.1, D50.5); locked at F3.A → zero migration, no K8s axis impact, ~2 phi-core import additions of zero (this is internal types).

### F4 — Migration count

Direct consequence of F3:
- **F4.A — Zero migrations.** F3.A adds an `Option<AllocateRefinement>` field with `#[serde(default)]`; SurrealDB schemaless persistence + serde-default shielding cover it. Pattern matches CH-11 (no migration for `approval_mode`) + CH-13 (no migration for `audit_class`). **Planner-recommended.**
- **F4.B — Migration 0014.** Only required if the user locks F3.B / F3.C. Rejected (depends on F3 outcome).

**Auto-approval impact:** F4.A means migration count = 0 → preserves the Direct-approval "no new migration" criterion.

### F5 — Transfer compound-tx scope at this chunk

The compound-tx for `apply_transfer_grant` (F1.A) atomically performs three writes: (1) rewrite `OWNED_BY` edge for the resource; (2) revoke the sender's matching grant; (3) mint the recipient's grant. **But:** `Action::Transfer` has **zero runtime mint sites today** — verified by `grep -rn "Action::Transfer" modules/crates/`, only the action-defs file matches. So the chunk ships the **enforcement primitive** but has no production caller wiring it.

- **F5.A — Ship `apply_transfer_grant` Repository method on both `InMemoryRepository` + `SurrealStore` adapters, with full unit-test coverage. No production caller wired (forward-defensive).** **Planner-recommended.** Rationale: the structural-enforcement boundary is what closes D-new-13 (HIGH); the first M6+ chunk that introduces a real transfer-grant-mint AuthRequest flow (e.g., resource hand-off UX) consumes the primitive. Mirrors the CH-12 frozen-tag-write validator pattern (CH-12 shipped `validate_tag_write_on_session` + `frozen_tag_write_rejected` audit-event builder forward-defensively; first wiring caller in a future chunk). Test coverage: unit tests directly construct sender + recipient grants and call `apply_transfer_grant` to exercise atomicity.
- **F5.B — Ship F5.A primitive **and** wire the AuthRequest approve flow at slot-close: when an AR with `Action::Transfer` reaches `Approved`, the listener invokes `apply_transfer_grant` (parallels how Templates A/C/D mint grants on edge-creation).** Rejected: AuthRequest does not currently carry `action: Vec<Action>` or `actions: ...` field that scopes the request to `[transfer]` (verified by reading `nodes.rs:650-694`'s Grant — actions are on Grant, not AuthRequest). Adding AR-level action-scope routing inflates the chunk to ~3 engineer-days + introduces a new listener type. Defer to M6+ when a real transfer-flow surface is shipped.
- **F5.C — Defer the entire transfer-cardinality enforcement primitive to M6+; close D-new-13 by accepting-as-is for now.** Rejected: D-new-13 is HIGH-A (security-boundary drift); concept-doc 02 line 206 mandates atomic revocation; postponing structural-enforcement keeps the gap that the `Vec<Action>` field encoding makes silently exploitable.

**Auto-approval impact:** F5.A is planner-recommended + matches the CH-12 forward-defensive precedent; preserves Direct-approval criteria.

---

### Summary of recommended fork-resolution

| Fork | Recommended | Rationale |
|---|---|---|
| F1 | **F1.A** (`apply_transfer_grant` compound-tx Repository method) | Mirrors ADR-0022 compound-tx precedent; concept-doc 02 line 206 atomic-revocation framing |
| F2 | **F2.A** (`{ no_further_delegation: bool, max_depth: Option<u8> }`) | Minimal closure of fields the forward-scope row + concept doc 02 line 197 name |
| F3 | **F3.A** (`Grant.allocate_refinement: Option<AllocateRefinement>` with `#[serde(default)]`) | Mirrors CH-11 D48.1 + CH-13 D50.5 precedents; zero migration |
| F4 | **F4.A** (zero migrations) | F3.A consequence |
| F5 | **F5.A** (forward-defensive primitive; no production caller wired) | Mirrors CH-12 forward-defensive precedent; closes D-new-13 structurally |

If the user locks all 5 to the planner recommendation, **all Direct-approval criteria hold** (analysis below in §0).

### §0 — Auto-approval criteria self-analysis (planner-recommended path)

| Criterion | Status under all-A path |
|---|---|
| No locked forks diverging from planner recommendation | ✓ (5 recs, all .A) |
| Scope ≤ 1.5× forward-scope row | ✓ (forward-scope says ~2 days; planner estimate ~2 days; phase count 4) |
| Zero phi-core leverage delta | ✓ (zero new phi-core imports; permissions+repository surface) |
| No new K8s blocker class | ✓ (verified §3.B — `apply_transfer_grant` is single-writer SurrealDB tx, no new in-process state, no new IPC) |
| Audit envelope ≤ medium | ✓ (medium — Audit A code + Audit B docs/paperwork) |
| Confidence ≥ 9/10 | ✓ (target 9/10 — see §10) |
| No new migration | ✓ (F4.A: zero migrations) |

**All Direct-approval criteria hold under the planner-recommended path. Awaiting user-lock at plan-approval gate.**

---

### §1 — Context & principle

**Why this chunk.** Two drifts surface a security-boundary gap in the v0 permission model. **D-new-13 (HIGH-A)**: the concept doc 02 (line 206) says `transfer` is exclusive — *"on approval, rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership"* — but at current HEAD nothing distinguishes allocate from transfer at grant-mint time. Both Action variants exist (`Allocate` + `Transfer` at `action.rs:57-58`), but `Action::Transfer` has zero runtime callsites that mint a transfer-grant atomically with sender-revocation. **D-new-29 (LOW-B)**: concept doc 03 line 54 says `allocate` is umbrella with refinements expressed as constraints (canonical example `allocate: no_further_delegation` per concept 02 line 197); today `Grant` has no typed refinement field. CH-08 closes both by (1) shipping `Repository::apply_transfer_grant` as a forward-defensive compound-tx primitive that atomically revokes sender + mints recipient on transfer-flows + rewrites `OWNED_BY`, and (2) adding `Grant.allocate_refinement: Option<AllocateRefinement>` with `{ no_further_delegation: bool, max_depth: Option<u8> }` typed field set.

**Quality-over-speed restatement.** *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* For CH-08 specifically: the cardinality distinction is **structural** (a transfer that fails to revoke sender = silent over-accounting of authority across multiple holders, contradicting concept-doc 02 line 199–204 verbatim) and **the chunk-plan refuses the shortcut of leaving D-new-13 closed-as-aspirational**. The forward-defensive primitive is the minimum viable structural enforcement; it ships now even though no production transfer-flow exists yet.

**Forward-scope reference.** [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 91–96 (CH-08 row).

---

### §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `permissions/02-auth-request.md` | §"`allocate` Scope Semantics" lines 199–207 (cardinality table) | `allocate` is **additive** — sender retains full share; `transfer` is **exclusive** — sender loses authority. Arc::clone semantics for allocate; move semantics for transfer. *"An Auth Request with `scope: [transfer]`, on approval, rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership"* (line 206). | silent-in-code (per `_concept-audit-matrix.md:155`, drift D-new-13) | **honored** (`apply_transfer_grant` compound-tx primitive ships forward-defensive atomic revocation + edge rewrite + recipient-mint per F5.A; Allocate path unchanged + still additive) |
| `permissions/02-auth-request.md` | §"`allocate` Scope Semantics" line 197 (refinement framing) | `allocate: no_further_delegation` constraint **removes the allocate-with-delegation sub-capability** | silent-in-code (drift D-new-29 — same line referenced from concept-03) | **honored** (typed `AllocateRefinement { no_further_delegation, max_depth }` available on `Grant.allocate_refinement: Option<AllocateRefinement>`) |
| `permissions/03-action-vocabulary.md` | §"`allocate` as the Umbrella Action" line 54 (umbrella framing) | *"actions describe what authority; constraints describe how it's narrowed"* — allocate refinements live as constraints | silent-in-code (`_concept-audit-matrix.md:165`, drift D-new-29) | **honored** (typed refinement; preserves vocabulary line: `Action::Allocate` covers the umbrella, `AllocateRefinement` narrows it) |
| `permissions/README.md` | (entry-invariants section) | Standard reference for permissions subtree invariants | (entry source) | (cross-referenced in §6 invariant matrix) |

**Permissions subtree hook**: ✓ `permissions/02` + `permissions/03` are touched → `permissions/README.md` cited as entry-invariants source.

**phi-core mapping hook**: not applicable. CH-08 surface is permission-engine + auth-request-flow internal to baby-phi. No phi-core type overlap. Verified by `concepts/phi-core-mapping.md` (no `Grant` / `AuthRequest` / `Constraint` row exists — these are baby-phi-only governance shapes).

**Cross-step concept-doc-semantic placement (CH-07 retro §5 row 4 discipline).** Concept-doc 02 line 206's atomic-revocation semantic could plausibly land at three pipeline steps (per F1 above): (a) Repository compound-tx, (b) AuthRequest slot-aggregation, (c) Engine read-time invariant. **The plan locks the placement at F1.A (Repository compound-tx) + cites the rationale**: structural enforcement at the boundary the data crosses (single transactional unit), not detection at the boundary it leaves (engine-side) and not pure-fn pollution at slot-aggregation (which would break the proptest invariant verified at `transitions.rs:1-9`).

---

### §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | (no phi-core type overlaps with CH-08's surface) | N/A | keep orthogonal |

CH-08 touches **`Grant`, `AuthRequest`, `Action`, `Constraint`, `Repository`**. None of these have phi-core counterparts (`phi_core::types::*` carries agent-loop/session/provider types, none govern grants/auth-requests/permissions).

**Expected import-count delta at chunk close**: **0 new phi-core imports.**

**Positive close-audit greps:**
- `grep -rn "phi_core::" modules/crates/domain/src/permissions/ modules/crates/domain/src/auth_requests/ modules/crates/domain/src/events/listeners.rs | wc -l` — **expect: 3** (verified baseline at plan-draft).
- `grep -rn "AllocateRefinement" modules/crates/domain/src/ | wc -l` — **expect: ≥ 5** (struct def + Grant field + helper methods + tests).
- `grep -rn "apply_transfer_grant" modules/crates/domain/src/repository.rs modules/crates/domain/src/in_memory.rs modules/crates/store/src/ | wc -l` — **expect: ≥ 3** (trait method + InMemory impl + SurrealStore impl).

**Forbidden-duplication greps:**
- `grep -rn "^struct Grant\|^pub struct Grant" modules/crates/ | grep -v "model/nodes.rs"` — **expect: 0** (no parallel Grant definition anywhere; `Grant` is defined exclusively at `nodes.rs:650`).
- `grep -rn "^struct Action\|^pub enum Action" modules/crates/ | grep -v "permissions/action.rs"` — **expect: 0** (Action defined exclusively at `action.rs:31`).
- `bash scripts/check-phi-core-reuse.sh` — **expect: green** (verified baseline).

**Per `baby-phi/CLAUDE.md` §"phi-core Leverage" rules 1–5.** This chunk is squarely in baby-phi-only governance surface. Zero phi-core delta is the correct outcome.

#### §3 cascade-artifact discipline (chunk-planner v3→v4)

CH-08 touches the **`Grant` struct** (F3.A adds field) — that is the canonical struct-cascade trigger the discipline addresses.

**Artifact A — `Grant { ... }` literal-construction sites** (new `allocate_refinement` field needs explicit construction OR will rely on `..Default::default()` / serde-default-shielding for tests):

(a) Invocation: `git -C /root/projects/phi/baby-phi grep -lnE '\bGrant\s*\{' -- 'modules/crates/' | grep -v 'AgentHolds\|IssuedGrant\|ProjectHolds\|OrgHolds\|NoMatchingGrant\|ResolvedGrant\|impl Grant\|pub struct Grant'`

(b) Raw count: **20 files** carrying `Grant { ... }` literal-construction sites. Total individual sites (counted via `grep -nE '^\s*Grant\s*\{$'` strict): **13 sites** + ~12 secondary mentions (continuation lines, `let x = Grant {` shapes), aggregate band **13–25 sites**.

(c) Per-file breakdown (the 20 distinct files):

| File | Sites |
|---|---|
| `modules/crates/domain/src/model/nodes.rs` | (definition + impl, no construction) |
| `modules/crates/domain/src/permissions/engine.rs:1027` | 1 (test helper) |
| `modules/crates/domain/src/permissions/expansion.rs:218` | 1 (test helper) |
| `modules/crates/domain/src/templates/a.rs:111` | 1 (Template A fire) |
| `modules/crates/domain/src/templates/c.rs:89` | 1 (Template C fire) |
| `modules/crates/domain/src/templates/d.rs:85` | 1 (Template D fire) |
| `modules/crates/domain/tests/common/mod.rs:96, 153` | 2 (test helpers; one is `prop_map`) |
| `modules/crates/domain/tests/in_memory_m5_test.rs:131` | 1 |
| `modules/crates/domain/tests/instance_uri_grant_match_props.rs:100, 148, 209` | 3 |
| `modules/crates/domain/tests/mcp_cascade_props.rs:84` | 1 |
| `modules/crates/domain/tests/multi_scope_cascade_acceptance.rs:52` | 1 |
| `modules/crates/domain/tests/step_4_constraint_value_match_props.rs:27` | 1 |
| `modules/crates/server/src/bootstrap/claim.rs:233` (verified line) | 1 (production grant-mint) |
| `modules/crates/server/src/platform/mcp_servers/register.rs` | 1 (production grant-mint) |
| `modules/crates/server/src/platform/model_providers/register.rs` | 1 (production grant-mint) |
| `modules/crates/server/src/platform/orgs/create.rs:181` (verified line) | 1 (production grant-mint, CEO `[allocate]`) |
| `modules/crates/server/src/platform/secrets/add.rs` | 1 (production grant-mint) |
| `modules/crates/server/tests/acceptance_common/admin.rs:233` | 1 |
| `modules/crates/server/tests/acceptance_mcp_servers.rs` | 1 |
| `modules/crates/server/tests/acceptance_per_session_consent_gating.rs` | 1 |
| `modules/crates/server/tests/handler_support_test.rs:38` | 1 |
| `modules/crates/store/tests/repo_m2_surface_test.rs:290` | 1 |
| `modules/crates/store/tests/repository_test.rs:490, 589, 1133, 1654` | 4 |

**Per-file edit-count predictions are approximate; the aggregate band is load-bearing.** Aggregate predicted edit band: **0–25 sites** depending on whether the field uses `..Default::default()` shorthand or explicit `allocate_refinement: None`. **Mitigation**: F3.A field carries `#[serde(default)]` AND can be omitted via `..Default::default()` IF `Grant: Default` exists; today `Grant` does **not** derive `Default` (verified `nodes.rs:649`). The implementer MUST decide at P1 whether to add `Default` to Grant (zero-callsite cost) OR explicitly add `allocate_refinement: None` to every site. Plan **recommends adding `Default` to Grant** to keep cascade at 0 callsites — but the implementer may justify the explicit-field approach in P1 review.

**Pause discipline trigger**: PAUSE if actual cascade edits exceed 1.5× the upper band (`> 38 sites`) — that signals the field-add did not close cleanly under either strategy, and the planner should re-evaluate F3.

**Artifact B — Repository trait surface (`Repository::apply_transfer_grant` ADD)** is a **new method addition** (not a signature change) so cascade is **0 implementation-sites today** (no callers) + **2 trait-impl sites** (`InMemoryRepository`, `SurrealStore`):

(a) `git -C /root/projects/phi/baby-phi grep -nE '^impl(\s|<).*\bRepository\b\s+for\b' -- 'modules/crates/' 2>&1 | head`

(b) Raw count: **2 trait-impl blocks**.

(c) Per-file breakdown:

| File | Action |
|---|---|
| `modules/crates/domain/src/in_memory.rs` (around line 665 region) | Add `apply_transfer_grant` method body |
| `modules/crates/store/src/repo_impl.rs` (existing M5 module) | Add `apply_transfer_grant` method body |

**Pause discipline trigger**: PAUSE if a 3rd Repository impl is discovered (zero extant per repo audit).

**Artifact C — `AllocateRefinement` cascade**: zero today (`grep -rn "AllocateRefinement\|allocate_refinement\|no_further_delegation" modules/crates/ | head` returns empty per planner verification). **0 sites at chunk-open → ~5 sites at chunk-close** (struct def + Grant field + 1 audit-event builder if needed + 2–3 unit tests).

---

### §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`, `RwLock`, `AtomicBool`, `Mutex`, `OnceCell`, `RefCell`) | `apply_transfer_grant` operates on Repository (durable SurrealDB); `AllocateRefinement` is a serializable struct on Grant | **no** | — |
| **A2** | New IPC channel (`mpsc`, `broadcast`, `oneshot`, `watch`, `Notify`) | Pure data + Repository writes; no new tasks/channels | **no** | — |
| **A3** | New pod-local resource (file handle, listener socket, sub-process, lock file, on-disk cache) | None | **no** | — |
| **A4** | Migration runner / first-apply race | F4.A: zero migrations | **no** | — |
| **A5** | Trait-shape requirement | `Repository::apply_transfer_grant` is added to existing trait; trait-object-friendly per existing pattern (`async fn` returning `RepositoryResult<()>`) | **no** | Already trait-shaped (Repository is a `#[async_trait]` trait — verified at `repository.rs`) |
| **A6** | Cross-pod state sharing | All mutated state is in SurrealDB (durable, cross-pod-visible by design) | **no** | — |
| **A7** | Audit hash-chain symmetry | If `apply_transfer_grant` emits an audit event, it MUST go through the existing `AuditEmitter` (single writer per CHK8S-D-08) | **no** (audit-event emission, if any, routes through existing `AuditEmitter` per ADR-0028 + CH-13's pattern at `events/listeners.rs:342`) | If a new `transfer_grant.minted` audit event is added, route through `self.audit.emit(...)` — same pattern as CH-13 Templates A/C/D |

**Conforming-criteria check against ADR-0033:**
- **D33.1 (`SessionRegistry` trait)** — CH-08 does not touch the registry. ✓
- **D33.2 (`SurrealStore::open_remote`)** — `apply_transfer_grant` is a Repository method; ships an impl on `SurrealStore` that works under both `open_embedded` and `open_remote` (uses `surrealdb` query API symmetrically). ✓
- **D33.3 (SIGTERM graceful shutdown)** — no new `tokio::spawn` tasks. ✓
- **D33.4 (`EventBus.shutdown` + `drain`)** — no new `EventBus` emitters/listeners (audit-event routes through existing `AuditEmitter`, same as CH-13 templates). ✓

**Conclusion**: **K8s-neutral.** No new ledger entry needed.

---

### §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/<feature>.md` | **YES** — new file: `m5_2/architecture/allocate-transfer-cardinality.md` covering F1.A compound-tx primitive + F3.A `Grant.allocate_refinement` field + cardinality table mirroring concept-doc 02 lines 199–207 | **(a) update in-chunk** at P3 |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/<feature>-operations.md` | **YES** — new file: `m5_2/operations/allocate-transfer-cardinality-operations.md` covering: error codes for transfer-cardinality violations (if any new variants surface); `RepositoryError::TransferGrantConflict` (or similar) error-code reference; SRE playbook for "what if transfer compound-tx fails halfway" (atomic-rollback guarantee); audit-event reference for `transfer_grant_minted` if shipped | **(a) update in-chunk** at P3 |
| **User-guide** | `docs/specs/v0/implementation/m5_2/user-guide/{<feature>-walkthrough,cli-reference-mN,troubleshooting}.md` | **NO** — no operator-visible behaviour shifts at this chunk. `Action::Transfer` has zero runtime mint sites (verified) and the F5.A primitive is forward-defensive. The first M6+ chunk wiring a real transfer-flow surface owns the user-guide tier. | **(b) defer with reason + successor: M6+ first-transfer-flow chunk** |

---

### §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-13` | [`../drifts/D-new-13.md`](../../../v0/implementation/m5_1/drifts/D-new-13.md) | **HIGH** (A) | discovered → **remediated** | F1.A + F5.A: `Repository::apply_transfer_grant` compound-tx primitive ships atomic revocation + edge rewrite + recipient-mint per concept-doc 02 line 206; allocate-path unchanged + still additive (concept-doc 02 lines 199–204 honoured). |
| `D-new-29` | [`../drifts/D-new-29.md`](../../../v0/implementation/m5_1/drifts/D-new-29.md) | LOW (B) | discovered → **remediated** | F2.A + F3.A: `Grant.allocate_refinement: Option<AllocateRefinement>` with typed `{ no_further_delegation: bool, max_depth: Option<u8> }` field set per concept-doc 02 line 197 + concept-doc 03 line 54. |

Both transitions land at chunk seal (P4) per [`drift-lifecycle.md`](../../../v0/implementation/m5_1/process/drift-lifecycle.md).

**Drifts table count**: at chunk close, drifts with status `remediated` increases by 2. Pre-CH-08 baseline: 18 remediated; post-CH-08: 20 remediated.

---

### §5 — ADRs drafted

**ADR-0052 — `allocate` / `transfer` cardinality + `AllocateRefinement` typed constraint**

- **Number**: 0052 (verified next-free at plan-draft via `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE "[0-9]{4}" | sort -n | tail`; latest is 0051 at `m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`).
- **Path**: `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md`.
- **Drafted at**: **P0** (Proposed status).
- **Decision summary**: ratifies F1.A compound-tx primitive (`Repository::apply_transfer_grant`) + F2.A minimal field set (`{ no_further_delegation: bool, max_depth: Option<u8> }`) + F3.A `Grant.allocate_refinement: Option<AllocateRefinement>` field with `#[serde(default)]` shielding + F4.A zero-migration confirmation + F5.A forward-defensive primitive (no production caller).
- **Sub-decisions to document**: D52.1 — cardinality enforcement boundary (F1.A); D52.2 — `AllocateRefinement` field set (F2.A); D52.3 — `Grant.allocate_refinement` denormalisation (F3.A precedent: D48.1 + D50.5); D52.4 — zero-migration property (F4.A); D52.5 — `apply_transfer_grant` compound-tx atomicity guarantee (F5.A); D52.6 — `Allocate`-path unchanged invariant (sender's grant untouched on Allocate-mint, additive cardinality preserved); D52.7 — `Default` impl on Grant for cascade neutralisation (or alternative explicit-field approach if Default-derive proves problematic).
- **Flips Proposed → Accepted at**: **P4** (chunk seal).

**ADR-body checklist (chunk-planner v5 + CH-13 retro Row 1 discipline):**

1. **§Forks header** — drafted at P0 with explicit user-lock outcome, format mirroring ADR-0050:
   - Direct-approval cycle: `Forks (all planner-recommended at chunk-open; user-locked at plan approval to F1.A / F2.A / F3.A / F4.A / F5.A)`.
   - Divergent cycle: `Forks (F<N> user-locked to F<N>.<X> at plan approval — diverges from planner recommendation F<N>.<rec>; F<rest> at planner-recommendation)`.
2. **§Cross-references — all 4 categories.**
   - **(a) Originating concept-doc**: `permissions/02-auth-request.md` §"`allocate` Scope Semantics" lines 179–207 (cardinality table at lines 201–204; atomic-revocation language at line 206; refinement framing at line 197); `permissions/03-action-vocabulary.md` §"`allocate` as the Umbrella Action" lines 48–54 (umbrella + refinement-as-constraint framing).
   - **(b) Closed drifts**: `D-new-13` (HIGH-A); `D-new-29` (LOW-B).
   - **(c) Prior ADRs cited as precedent**: ADR-0022 (compound-tx pattern for org creation — F1.A precedent); ADR-0023 (inherit-from-snapshot — F1.A relevance); ADR-0028 (audit-event emission via `AuditEmitter` — A7 pattern); ADR-0033 (CH-K8S-PREP conforming criteria); ADR-0048 §D48.1 (Grant.approval_mode field-add precedent for F3.A); ADR-0050 §D50.5 (Grant.audit_class field-add precedent for F3.A); ADR-0043 (typed Action enum — CH-04 prerequisite).
   - **(d) Forward-scope row**: [`baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 91–96 (CH-08 row).

---

### §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-04** | Typed `Action` enum with `Allocate` + `Transfer` variants (drifts D-new-09 + D-new-10 remediated) — `_concept-audit-matrix.md:163-164` Status: **honored** | `grep -nE "Action::Allocate\|Action::Transfer" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs \| head -10` — expect lines 57-58, 149-150, 229-230, 270-271, 310-311, 344-345 still match (all verified at plan-draft) |
| **CH-04** | `Vec<Action>` carriers on `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` (D-new-09 close evidence) | `grep -nE "action: Vec<.*Action>\|actions: Vec<.*Action>" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs \| head -5` — expect ≥ 3 hits |
| **CH-11 / ADR-0048 §D48.1** | `Grant.approval_mode: ApprovalMode` field with `#[serde(default)]` shielding (the precedent F3.A mirrors) | `grep -n "pub approval_mode: ApprovalMode" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` — expect line 675 |
| **CH-13 / ADR-0050 §D50.5** | `Grant.audit_class: AuditClass` field with `#[serde(default = "Grant::default_audit_class")]` shielding (precedent F3.A mirrors) | `grep -n "pub audit_class: crate::audit::AuditClass" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` — expect line 693 |
| **CH-12 forward-defensive precedent** | `validate_tag_write_on_session` validator + `frozen_tag_write_rejected` audit-event builder shipped without runtime caller (parallels F5.A) | `grep -rn "frozen_tag_write_rejected\|validate_tag_write_on_session" /root/projects/phi/baby-phi/modules/crates/domain/src/ \| wc -l` — expect ≥ 4 |
| **ADR-0022 / ADR-0023** | Compound-tx pattern (`apply_org_creation` etc.) ships atomic multi-row writes through `Repository` trait | `grep -n "apply_org_creation\|apply_bootstrap_claim\|fn apply_" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs \| head -10` |
| **CH-K8S-PREP / ADR-0033** | `SurrealStore::open_remote` works for new compound-tx (D33.2) | (no command — verified by §3.B A5/A6 conformance) |
| **`scripts/check-phi-core-reuse.sh`** green at chunk-open | (CI guard — verified `OK` at plan-draft) | `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` |
| **Test count baseline** | 1399 passed / 0 failed / 2 ignored at CH-07 close (commit 892e702) | `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4 2>&1 \| tail -3` |

This table runs at **chunk open** (P0 reading-list verification) AND at **chunk seal** (P4 close-criteria gate). Any regression → new drift file + open question for user before chunk proceeds.

**Matrix Status discipline (CH-12 retro Row 1)**: every Status entry above is **copy-pasted letter-for-letter** from the corresponding `_concept-audit-matrix.md` row's Status column. No paraphrase ("**honored**" stays bolded, "**partially-honored**" stays bolded, etc.).

---

### §7 — Phases within the chunk

#### **P0 — Plan archive + ADR-0052 scaffold**

- **Goal.** Ship the cycle plan + ADR-0052 (Proposed) + cycle-index row update + drift-file `discovered → planned` flip if applicable. No code changes.
- **Deliverables.**
  1. Plan file: `baby-phi/docs/specs/plan/build/ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/plan.md` (this file).
  2. ADR-0052 file: `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md` (Proposed status).
  3. Cycle-index row added: `baby-phi/docs/specs/plan/build/_cycle-index.md`.
- **Tests.** N/A (no code).
- **Concept-alignment check.** §2 rows enter `silent-in-code` baseline (unchanged from chunk-open).
- **phi-core leverage check.** §3 baseline: 0 phi-core delta predicted (re-verified at P0 plan-draft).
- **User-facing doc updates.** None at P0.
- **Confidence target.** 100% (scaffold).
- **Pause discipline.** None at P0; user lock on F1–F5 happens at plan-approval gate before P1 opens.

#### **P1 — `AllocateRefinement` type + `Grant.allocate_refinement` field**

- **Goal.** Land the smaller deliverable first (D-new-29). Add `AllocateRefinement` struct + Grant field with serde-default shielding. Decide between `Default`-derive on Grant vs explicit-field cascade.
- **Deliverables.**
  1. `domain/src/permissions/allocate_refinement.rs` (NEW) — `pub struct AllocateRefinement { pub no_further_delegation: bool, pub max_depth: Option<u8> }` with `Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default` derives + module docstring citing concept-doc 02 line 197 + concept-doc 03 line 54.
  2. `domain/src/permissions/mod.rs` — `pub mod allocate_refinement; pub use allocate_refinement::AllocateRefinement;`.
  3. `domain/src/model/nodes.rs:650-694` — extend `Grant` with `#[serde(default)] pub allocate_refinement: Option<crate::permissions::AllocateRefinement>` + doc-comment citing ADR-0052 §D52.3 + concept-doc 02 line 197.
  4. **Grant `Default` derive decision** — implementer attempts `derive(Default)` on Grant first (cleanest cascade neutralisation). If a field type lacks `Default` (e.g., chrono `DateTime`/`Utc` does have `Default` for some shapes; `GrantId` requires verification), fall back to per-callsite explicit field addition. Decision documented in ADR-0052 §D52.7.
  5. Unit tests for `AllocateRefinement` round-trip + `Grant` field decode-as-`None` for pre-CH-08 JSON.
- **Tests.** New unit tests in `allocate_refinement.rs::tests`: (a) `default_is_unbounded` — `AllocateRefinement::default()` produces `{ no_further_delegation: false, max_depth: None }`; (b) `serde_round_trip` — JSON round-trip; (c) `legacy_grant_decodes_with_none` — pre-CH-08 grant JSON (without the field) decodes into `Grant.allocate_refinement = None`. Estimated **3 new unit tests**.
- **Concept-alignment check.** §2 row 2 (D-new-29 / `permissions/02:197`) transitions `silent-in-code → honored` (the type exists, even before any wiring consumes it — closes the structural gap per concept-doc framing).
- **phi-core leverage check.** §3 imports unchanged. `grep -rn "phi_core::" modules/crates/domain/src/permissions/` count stays at baseline.
- **User-facing doc updates.** None at P1; landing at P3 with the architecture page.
- **Confidence target.** ≥ 97%.
- **Pause discipline.** PAUSE if `Grant`-cascade actual edits exceed 1.5× upper band (`> 38 sites`). Per §3 Artifact A, both strategies (Default-derive + explicit-field) should keep cascade ≤ 25 sites. Trigger threshold = 38.

#### **P2 — `Repository::apply_transfer_grant` compound-tx primitive**

- **Goal.** Land the load-bearing deliverable (D-new-13). Add a new compound-tx Repository method that atomically (1) rewrites the resource's `OWNED_BY` edge, (2) revokes the sender's matching grant, (3) mints the recipient's grant.
- **Deliverables.**
  1. `domain/src/repository.rs` — `pub struct TransferGrantPayload { sender_grant_id: GrantId, recipient: PrincipalRef, resource: ResourceRef, recipient_grant: Grant, new_owner: NodeId, at: DateTime<Utc>, audit_event: Option<AuditEvent> }` + new trait method `async fn apply_transfer_grant(&self, payload: &TransferGrantPayload) -> RepositoryResult<()>` with full docstring citing ADR-0052 §D52.5 + concept-doc 02 line 206.
  2. `domain/src/in_memory.rs::InMemoryRepository::apply_transfer_grant` — body executes the three writes within the existing `Mutex`-guarded state lock (`lock()?` covers atomicity in-memory). Sender-grant-not-found → `RepositoryError::NotFound`; sender-grant-already-revoked → `RepositoryError::Conflict { ... }` (or new variant `TransferGrantConflict { reason: ... }`).
  3. `store/src/repo_impl.rs::SurrealStore::apply_transfer_grant` — body uses SurrealDB's `BEGIN`...`COMMIT` transaction wrapping the three writes (mirroring existing compound-tx patterns at `repo_impl.rs:2738, 2871`).
  4. New error variant if needed: `RepositoryError::TransferGrantConflict { reason: String }` — only added if existing `Conflict` / `InvalidInput` variants don't cleanly carry the cardinality-violation framing.
- **Tests.**
  - `domain/tests/transfer_grant_atomicity_test.rs` (NEW) — unit tests on `InMemoryRepository`: (a) `transfer_revokes_sender_and_mints_recipient` — happy path; (b) `transfer_rolls_back_on_sender_grant_missing` — atomicity; (c) `transfer_rolls_back_on_sender_grant_already_revoked` — re-entry safety; (d) `allocate_path_remains_additive` — sender's grant survives `create_grant` of an Allocate-grant on the same resource.
  - `store/tests/transfer_grant_surreal_test.rs` (NEW) — happy-path round-trip on SurrealStore (in-memory mode) confirming all three writes commit atomically.
  - Estimated **5–7 new tests**.
- **Concept-alignment check.** §2 row 1 (D-new-13 / `permissions/02:199-207`) transitions `silent-in-code → honored` (the structural enforcement primitive exists; concept-doc 02 line 206 atomic-revocation semantic is realisable through the Repository surface).
- **phi-core leverage check.** §3 forbidden-greps still return 0. New trait method does not introduce phi-core import.
- **User-facing doc updates.** None at P2; landing at P3.
- **Confidence target.** ≥ 97%.
- **Pause discipline.** PAUSE if SurrealDB transaction support proves insufficient for the 3-write atomic guarantee (verified feasible at plan-draft via existing precedents at `repo_impl.rs:2738, 2871` — but the `apply_transfer_grant` write-set is more complex). Trigger: any test in P2 that asserts atomicity fails non-deterministically.

#### **P3 — Architecture + operations docs + concept-audit-matrix updates + verified-headers**

- **Goal.** User-facing doc tier (per CH-22 §3.C mandate) + governance-doc tier updates.
- **Deliverables.**
  1. `docs/specs/v0/implementation/m5_2/architecture/allocate-transfer-cardinality.md` (NEW) — design page covering: cardinality table mirroring concept-doc 02 lines 199–207; the three-write atomic compound-tx; `AllocateRefinement` field semantics; **forward-defensive note** explaining why `Action::Transfer` has no production caller today; cross-refs to ADR-0052 + drifts D-new-13 + D-new-29.
  2. `docs/specs/v0/implementation/m5_2/operations/allocate-transfer-cardinality-operations.md` (NEW) — error-code reference for `RepositoryError::TransferGrantConflict` (if added); SRE playbook for half-completed transfer (atomic-rollback guarantee — operator action: re-emit the AR, no manual cleanup); audit-event reference for `transfer_grant_minted` (if shipped).
  3. `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` — update rows at lines 155 and 165 to flip Status from `silent-in-code` → **honored** with chunk-close evidence cells. **Letter-for-letter copy-paste from §2 target column** per CH-12 retro Row 1.
  4. Concept-doc verified-header bumps: `permissions/02-auth-request.md` + `permissions/03-action-vocabulary.md` (Last-verified line bumped to 2026-05-07 with CH-08 amendment notes).
  5. Cycle-index row updated to reflect P3 progress.
- **Tests.** Doc-link guard (`scripts/check-doc-links.sh`) + ops-doc-headers guard (`scripts/check-ops-doc-headers.sh`) re-run.
- **Concept-alignment check.** §2 rows 1 + 2 + 3 fully `honored` (matrix updated).
- **phi-core leverage check.** §3 baseline preserved.
- **User-facing doc updates.** §3.C row 1 (architecture) + row 2 (operations) — both close in-chunk at P3. Row 3 (user-guide) deferred per §3.C with successor M6+ first-transfer-flow chunk.
- **Confidence target.** ≥ 99%.
- **Pause discipline.** None — pure docs phase.

#### **P4 — Chunk seal: drift remediation + ADR flip + final CI guards**

- **Goal.** Flip drifts to `remediated`, ADR to `Accepted`, run all CI guards + workspace tests + final auditor preparation.
- **Deliverables.**
  1. `docs/specs/v0/implementation/m5_1/drifts/D-new-13.md` — Status flipped `discovered → remediated`; lifecycle entry added: `2026-05-07 — remediated — CH-08 P4 — F1.A + F5.A: Repository::apply_transfer_grant compound-tx primitive ships; Allocate-path remains additive; ADR-0052 §D52.1, §D52.5, §D52.6 ratify`.
  2. `docs/specs/v0/implementation/m5_1/drifts/D-new-29.md` — Status flipped `discovered → remediated`; lifecycle entry added: `2026-05-07 — remediated — CH-08 P4 — F2.A + F3.A: AllocateRefinement { no_further_delegation, max_depth } typed field on Grant.allocate_refinement; ADR-0052 §D52.2, §D52.3 ratify`.
  3. `docs/specs/v0/implementation/m5_1/drifts/README.md:112, 128` — Status cells flipped + chunk-close marker (`CH-08 ✓`).
  4. `docs/specs/v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md` — Status flipped `Proposed → Accepted`; verified-header refreshed.
  5. **Verified-header diff-vs-body audit** (CH-11 retro P4 paperwork checklist): for every modified doc, compare verified-header description against actual diff; fix mismatches.
  6. **Matrix-letter-letter audit** (CH-12 retro Row 1 P4 addendum): re-verify §2-target → `_concept-audit-matrix.md` Status copy-paste fidelity.
  7. CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
- **Tests.** Full workspace test sweep.
- **Concept-alignment check.** §2 all rows reach target status; matrix row count of `silent-in-code` decreases by 2.
- **phi-core leverage check.** §3 forbidden-greps return 0; positive-greps return predicted counts; `check-phi-core-reuse.sh` green.
- **User-facing doc updates.** §3.C row 1 + row 2 finalised at P3; row 3 defer-decision recorded in §3.C table.
- **Confidence target.** ≥ 99%.
- **Pause discipline.** PAUSE on any CI-guard failure or test-count delta outside the §8 asymmetric accept band.

---

### §8 — Tests summary

**Baseline at chunk open**: 1399 passed / 0 failed / 2 ignored (CH-07 close, commit `892e702`).

**Deliverable-listed unit/integration test count predictions:**
- P1 — AllocateRefinement: **3 new unit tests** (`default_is_unbounded`, `serde_round_trip`, `legacy_grant_decodes_with_none`).
- P2 — apply_transfer_grant InMemory: **4 new unit tests** (`transfer_revokes_sender_and_mints_recipient`, `transfer_rolls_back_on_sender_grant_missing`, `transfer_rolls_back_on_sender_grant_already_revoked`, `allocate_path_remains_additive`).
- P2 — apply_transfer_grant SurrealStore: **2 new tests** (happy-path round-trip + atomicity).
- P3 — pure docs phase: **0 new tests**.
- **Sum of deliverable-listed tests: 9 new tests.**

**Plan §8 chunk-close prediction band** (per asymmetric ×1.0–×1.20 buffer per CH-12 retro):
- **Lower bound**: 1399 + 9 = **1408**.
- **Upper bound**: 1399 + (9 × 1.20) = **1399 + 11 = 1410**.
- **Prediction**: **1408–1410 passed**.

Healthy implementer over-shoot (one-property-per-row determinism tests; round-trip helpers; paired audit-event tests; ISO-8601 / serde-format helper unit tests; precedence regression tests) is normal and should not trigger a re-plan unless the count exceeds 1410.

**Layer breakdown:**
- Unit (in-source `#[cfg(test)]`): 3 (P1) + 4 (P2 InMemory) = **7**.
- Integration (`tests/*.rs`): 2 (P2 SurrealStore) = **2**.
- Acceptance / e2e: 0.

**Named test files:**
- New: `domain/tests/transfer_grant_atomicity_test.rs`.
- New: `store/tests/transfer_grant_surreal_test.rs`.
- Existing inlined `#[cfg(test)] mod tests`: `domain/src/permissions/allocate_refinement.rs`, `domain/src/repository.rs` (potentially).

**Named expected-still-green tests** (regression risk):
- `domain/tests/repository_test.rs` (Grant struct cascade — risk if F3.A field-add cascade is mis-handled).
- `domain/tests/multi_scope_cascade_acceptance.rs` (CH-07 baseline — risk if `Grant` cascade affects struct construction).
- `domain/tests/in_memory_m5_test.rs` (Grant struct cascade — same as above).
- `domain/src/permissions/engine.rs::tests` (Grant test helpers — verified Grant construction at line 1027).
- `server/tests/acceptance_per_session_consent_gating.rs` (CH-11 baseline — Grant.approval_mode precedent same shape).

---

### §9 — Pre-chunk gate

**Reading list (mandatory):**
1. **Concept docs cited in §2:**
   - `baby-phi/docs/specs/v0/concepts/permissions/02-auth-request.md` §"`allocate` Scope Semantics" lines 179–207.
   - `baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md` §"`allocate` as the Umbrella Action" lines 48–54.
   - `baby-phi/docs/specs/v0/concepts/permissions/README.md` (entry-invariants).
2. **Drift files cited in §4:**
   - `baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-13.md`.
   - `baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-29.md`.
3. **Prior-chunk plans cited in §6:**
   - `baby-phi/docs/specs/plan/build/ch-04-typed-action-vocabulary-3a65a2fc.md` (CH-04 sealed plan).
   - `baby-phi/docs/specs/plan/build/ch-11-per-session-consent-gating-d5428c43/plan.md` (CH-11 D48.1 precedent).
   - `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-d4fe1b7c/plan.md` (CH-13 D50.5 precedent — folder name confirmed via `ls`).
   - `baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md` (CH-12 forward-defensive precedent — folder name confirmed via `ls`).
4. `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §5 (CH-08 row at lines 91–96) + §7 (binding Q&A decisions).
5. `baby-phi/CLAUDE.md` §"phi-core Leverage" rules 1–5.
6. **CH-11 conditional reading-list (per `engine` Step N body discipline)**: not triggered. CH-08 does not modify any `domain::permissions::engine::step_N_*` body.

**Carry-forward invariants** (verified green at chunk open via §6 commands):
- `cargo test --workspace` baseline: 1399 / 0 / 2.
- `scripts/check-phi-core-reuse.sh` green (verified `OK` at plan-draft).
- `scripts/check-doc-links.sh` green (verified at CH-07 close).
- `scripts/check-ops-doc-headers.sh` green (verified at CH-07 close).
- `scripts/check-spec-drift.sh` green (verified at CH-07 close).
- `git diff modules/` against chunk-open HEAD: empty (verified clean tree at `892e702 ch07`).

**Pending decisions carried into this chunk:**
- 5 forks (F1–F5) — all require user-lock at plan-approval gate before P1 opens.
- Drift-file `discovered → planned` flip not applicable (drifts stay `discovered` until P4 final remediation per drift-lifecycle.md).

**Chunk-ordering note (Q4 decision).** User selected CH-08 next from the forward-scope dependency graph after CH-07 close. CH-04 is the only declared prerequisite (verified sealed). No other chunk-order dependency assumed.

---

### §10 — Close criteria

**Source of truth: concept docs.** No rounding; below-target blocks close.

**4 aspects (each graded pass / fail):**

- **Code aspect** — all phases' deliverables shipped; `cargo test --workspace` passes (≥ 1408 passing); clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect** — both scopes:
  - *Governance tier*: D-new-13 + D-new-29 status flipped to `remediated` (drift README + drift files + lifecycle); `_concept-audit-matrix.md:155, 165` Status flipped `silent-in-code → honored` with letter-for-letter §2 target copy; ADR-0052 flipped Proposed → Accepted; verified-header bumps on `permissions/02-auth-request.md` + `permissions/03-action-vocabulary.md`.
  - *User-facing tier* (post-CH-22): §3.C row 1 (architecture) + row 2 (operations) docs created in-chunk at P3; row 3 (user-guide) carries explicit defer-decision with successor reference (M6+ first-transfer-flow chunk).
- **phi-core leverage aspect** — §3 import-count delta = 0 (predicted) ± 0 documented variance; all forbidden-duplication greps return 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's target-status at chunk-close achieved (no `contradicted`, no remaining `silent-in-code`).

**2 confidence % (each with named numerator/denominator):**

- **Implementation confidence %** = `(claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk)`. **Target: 9/10 = 90% minimum**, planner predicts **10/10 = 100%** at close. Claims-in-scope for chunk:
  1. Allocate cardinality is **additive** (sender's grant survives Allocate-mint).
  2. Transfer cardinality is **exclusive** (sender's grant revoked atomically with recipient mint).
  3. `OWNED_BY` edge rewritten in transfer compound-tx.
  4. Atomicity of the three-write transfer (rollback on partial failure).
  5. `AllocateRefinement { no_further_delegation, max_depth }` typed field shipped.
  6. `Grant.allocate_refinement: Option<AllocateRefinement>` field with `#[serde(default)]` shielding.
  7. Pre-CH-08 grants decode with `allocate_refinement: None` (back-compat).
  8. Zero migrations (F4.A confirmation).
  9. Zero phi-core leverage delta.
  10. Both drifts (D-new-13 + D-new-29) flip to `remediated`.

- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)`. Target: **8/8 = 100%**. Doc pages touched: ADR-0052; architecture page; operations page; D-new-13.md; D-new-29.md; drifts/README.md; concept-audit-matrix.md; cycle-index.md.

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** Below 90% blocks close. Target composite: **≥ 90%**.

**Locked forks** (recorded for ADR-0052 §Forks header):
- F1 → F1.A — *(planner-recommended; awaiting user-lock at plan-approval gate)*
- F2 → F2.A — *(planner-recommended; awaiting user-lock)*
- F3 → F3.A — *(planner-recommended; awaiting user-lock)*
- F4 → F4.A — *(planner-recommended; awaiting user-lock)*
- F5 → F5.A — *(planner-recommended; awaiting user-lock)*

**P4 chunk-seal paperwork checklist (CH-11 v + CH-12 v addenda):**
- For every modified doc with verified-header (line 1 `<!-- Last verified: ... -->`), confirm header description matches body diff exactly. Mismatch → fix before chunk-seal.
- For every `_concept-audit-matrix.md` row touched, new Status column value MUST be **copy-pasted letter-for-letter** from §2 target column (no paraphrase, no binary-flip on partially-honored split-axis cases).

---

### §11 — Post-chunk independent audit plan

**Phase count: 4 → audit envelope: medium → 2 audit agents** (per audit-envelope-size skill).

**Agent assignments:**

- **Audit A (code correctness + phi-core leverage)** — code-correctness lane covering P1 + P2 deliverables.
- **Audit B (docs fidelity + concept alignment + drift paperwork)** — doc-correctness lane covering P3 + P4 paperwork + concept-doc 02 + 03 verified-header diffs + matrix-letter-letter check.

**Audit A prompt (≤ 600 words):**

> *You are an independent auditor for baby-phi chunk CH-08 cycle `7cbe74a4`. Read the cycle plan at `baby-phi/docs/specs/plan/build/ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/plan.md` and the implementation diff. Audit the **code-correctness + phi-core-leverage** lanes. Do NOT audit docs/paperwork (Audit B owns that).*
>
> *Specifically verify:*
> *(1) `AllocateRefinement` struct exists at `domain/src/permissions/allocate_refinement.rs` with the exact field set `{ no_further_delegation: bool, max_depth: Option<u8> }` from ADR-0052 §D52.2 / F2.A. Run `grep -n "pub struct AllocateRefinement" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/allocate_refinement.rs`.*
> *(2) `Grant.allocate_refinement: Option<AllocateRefinement>` field exists with `#[serde(default)]` shielding at `domain/src/model/nodes.rs`. Run `grep -nA1 "allocate_refinement" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs`. Confirm the serde-default attribute is on the line above the field.*
> *(3) `Repository::apply_transfer_grant` trait method exists at `domain/src/repository.rs` with full docstring citing ADR-0052 §D52.5 + concept-doc 02 line 206 atomic-revocation language verbatim.*
> *(4) Both `InMemoryRepository::apply_transfer_grant` (`in_memory.rs`) and `SurrealStore::apply_transfer_grant` (`store/src/repo_impl.rs`) implementations exist; SurrealStore body uses `BEGIN`/`COMMIT` (or equivalent compound-tx mechanism) per ADR-0022 precedent.*
> *(5) Atomicity tests exist: `transfer_grant_atomicity_test.rs::transfer_rolls_back_on_sender_grant_missing` AND `..._already_revoked` both pass.*
> *(6) `Allocate`-path additive invariant test exists: `allocate_path_remains_additive` — confirms calling `create_grant` with an `Allocate`-action grant on the same resource leaves any prior sender-grant intact.*
> *(7) phi-core leverage delta is zero: `grep -rn "phi_core::" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ /root/projects/phi/baby-phi/modules/crates/domain/src/auth_requests/ /root/projects/phi/baby-phi/modules/crates/domain/src/events/listeners.rs | wc -l` returns 3 (baseline preserved).*
> *(8) `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` returns 0 (green). Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked.*
> *(9) `Grant`-cascade was handled cleanly: either `Grant: Default` is derived (verify `derive(Default)` at `nodes.rs:649`) OR every `Grant { ... }` literal-construction site (per plan §3 Artifact A 20-file table) carries an explicit `allocate_refinement: None`. Mismatch on any production callsite (templates a/c/d, claim.rs, orgs/create.rs, etc.) is a FAIL.*
> *(10) `cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4` passes 1408–1410 tests. Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked.*
>
> *Report format: §A claims (1)–(10) PASS/FAIL/NOT-EXECUTED with cited file:line evidence; §B summary; §C any new drift surfaced. Do not propose fixes — only assess.*

**Audit B prompt (≤ 600 words):**

> *You are an independent auditor for baby-phi chunk CH-08 cycle `7cbe74a4`. Read the cycle plan at `baby-phi/docs/specs/plan/build/ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/plan.md` and the implementation diff. Audit the **docs-fidelity + concept-alignment + drift-paperwork** lanes. Do NOT audit code (Audit A owns that).*
>
> *Specifically verify:*
> *(1) ADR-0052 exists at `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md` with Status: Accepted at chunk-seal. Body contains §Forks header capturing the F1.A/F2.A/F3.A/F4.A/F5.A user-locks per chunk-planner v5 ADR-body checklist.*
> *(2) ADR-0052 §Cross-references contains all 4 categories: (a) concept-doc + sections + line ranges (`permissions/02:179-207` + `permissions/03:48-54`); (b) closed drifts (`D-new-13` + `D-new-29`); (c) prior ADRs (ADR-0022, ADR-0023, ADR-0028, ADR-0033, ADR-0043, ADR-0048 §D48.1, ADR-0050 §D50.5); (d) forward-scope row (lines 91–96).*
> *(3) `D-new-13.md` Status flipped `discovered → remediated`; lifecycle entry references CH-08 P4 + ADR-0052 §D52.1, §D52.5, §D52.6.*
> *(4) `D-new-29.md` Status flipped `discovered → remediated`; lifecycle entry references CH-08 P4 + ADR-0052 §D52.2, §D52.3.*
> *(5) `drifts/README.md:112, 128` Status cells flipped to **remediated** with `CH-08 ✓` close-marker. (Confirm via `grep -n "D-new-13\|D-new-29" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md`.)*
> *(6) `drifts/_concept-audit-matrix.md:155, 165` Status flipped `silent-in-code → honored`. **Verify letter-for-letter copy from plan §2 target column** (CH-12 retro Row 1).*
> *(7) `permissions/02-auth-request.md` verified-header bumped to 2026-05-07 with CH-08 amendment note. Header description matches body diff (CH-11 retro P4 paperwork checklist).*
> *(8) `permissions/03-action-vocabulary.md` verified-header bumped similarly.*
> *(9) Architecture page exists at `m5_2/architecture/allocate-transfer-cardinality.md` with cardinality table mirroring concept-doc 02 lines 199–207 + cross-refs to ADR-0052 + drifts.*
> *(10) Operations page exists at `m5_2/operations/allocate-transfer-cardinality-operations.md` with error-code reference + SRE playbook.*
> *(11) §3.C row 3 (user-guide tier) defer-decision recorded with explicit successor reference (M6+ first-transfer-flow chunk).*
> *(12) Cycle-index updated at `baby-phi/docs/specs/plan/build/_cycle-index.md`.*
>
> *Report format: §A claims (1)–(12) PASS/FAIL/NOT-EXECUTED with cited file:line evidence; §B summary; §C any new drift surfaced. Do not propose fixes — only assess.*

**Audit pass criteria.** Any new drift discovered → its own drift file created BEFORE chunk seals. Any audit-flagged concept contradiction → fixed in-chunk OR renegotiated with user approval OR converted to drift file with explicit future-chunk assignment. Chunk seal blocked until both audits return clean.

---

### §12 — Verification section (end-to-end recipe)

Granular Bash discipline (per `granular-bash-discipline-ab19399b.md`): one logical operation per Bash invocation. All cargo invocations capped at `-j 4`.

```bash
# 1. CI guards (4 separate invocations)
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (3 separate invocations)
/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace 2>&1 | tail -5

# 3. Chunk-specific positive greps
grep -rn "phi_core::" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ /root/projects/phi/baby-phi/modules/crates/domain/src/auth_requests/ | wc -l
grep -rn "AllocateRefinement" /root/projects/phi/baby-phi/modules/crates/domain/src/ | wc -l
grep -rn "apply_transfer_grant" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs /root/projects/phi/baby-phi/modules/crates/store/src/ | wc -l

# 4. Chunk-specific forbidden greps (expect 0)
grep -rn "^pub struct Grant\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "model/nodes.rs" | wc -l
grep -rn "^pub enum Action\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "permissions/action.rs" | wc -l

# 5. Targeted CH-08 tests
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 -p domain --test transfer_grant_atomicity_test 2>&1 | tail -10
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 -p store --test transfer_grant_surreal_test 2>&1 | tail -10
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 -p domain allocate_refinement 2>&1 | tail -10

# 6. Drift-file status verification
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-*.md | wc -l
# Expect: previous-count + 2 (D-new-13 + D-new-29).

# 7. ADR status verification
grep -E "^\*\*Status: Accepted\*\*" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md
```

**Expected outcome at chunk seal:**
- All 4 CI guards green.
- `cargo fmt --check` green; `cargo clippy` green under `RUSTFLAGS="-Dwarnings"`; `cargo test` 1408–1410 passing / 0 failed / 2 ignored.
- Positive greps return ≥ predicted counts (3 phi-core baseline preserved; AllocateRefinement ≥ 5 hits; apply_transfer_grant ≥ 3 hits).
- Forbidden greps return 0.
- Targeted CH-08 tests all pass.
- Drift-remediated count: pre-CH-08 + 2.
- ADR-0052 Status: Accepted.

---

## Plan-archive metadata

**Cycle hex**: `7cbe74a4` (generated via `openssl rand -hex 4` at plan-draft).
**Cycle folder**: `baby-phi/docs/specs/plan/build/ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/`.
**Plan file**: `plan.md` (this file).
**Audit log files (created post-implementation)**: `audit-a-iter1.md`, `audit-b-iter1.md`.
**Cycle-audit file**: `cycle-audit.md` (orchestrator-written at chunk seal).
**Retrospective file**: `retrospective.md`.

**Plan version**: chunk-planner v5 (per CH-13 retro standards-update; CH-07 retro pre-archive line-number re-verification active; CH-12 retro asymmetric ×1.0–×1.20 buffer + Matrix Status discipline + ADR Forks/Cross-references discipline active).

**Pre-archive line-number re-verification (chunk-planner v5)**: All §3 grep claims re-verified against current git HEAD (`892e702 ch07`) at plan-write time:
- `action.rs:57-58` (Allocate/Transfer variants) — verified live.
- `action.rs:149-150, 229-230, 270-271, 310-311, 344-345` (Allocate/Transfer constants) — verified live.
- `nodes.rs:650` (Grant struct) — verified live.
- `nodes.rs:675` (Grant.approval_mode CH-11 precedent) — verified live.
- `nodes.rs:693` (Grant.audit_class CH-13 precedent) — verified live.
- `nodes.rs:1008` (ToolAuthorityManifest.constraints) — verified live.
- `repository.rs:686` (Repository::create_grant) — verified live.
- `in_memory.rs:665` (InMemoryRepository::create_grant) — verified live.
- `events/listeners.rs:321, 445, 566` (Template A/C/D create_grant calls) — verified live.
- `transitions.rs:221` (override_approve fn) — verified live.
- `_concept-audit-matrix.md:155, 165` (drift D-new-13 + D-new-29 rows) — verified live.
- `drifts/README.md:112, 128` (drift status cells) — verified live.
- Latest ADR `0051` at `m5_2/decisions/` — verified live; next-free = `0052`.
- Latest migration `0013_per_session_consent_gating.surql` — verified; next-free = `0014` (only consumed if F4.B locked, which is rejected).

No drift detected. Plan is fresh as written.
