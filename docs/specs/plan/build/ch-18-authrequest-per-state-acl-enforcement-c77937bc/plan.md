<!-- Last verified: 2026-05-09 by Claude Code (chunk-planner v9 — v2 F3.B re-plan after gate-1 user-lock divergence (2026-05-09)) -->

# CH-18 — AuthRequest per-state ACL enforcement · Plan

**Cycle hex**: `c77937bc` (unchanged from v1; only the plan body is revised at gate-1 user-lock divergence).
**Plan version**: **v2 — F3.B re-plan after gate-1 user lock divergence (2026-05-09)**. v1 was structured for F3.A (4 mutation handlers); v2 expands for F3.B (Repository docstring update + dashboard + show + slot-fill read + the 4 mutation handlers + 8 submit-side create_auth_request callsites). User-locked at gate-1 to F1.B / F2.A / **F3.B (DIVERGENT from planner-recommended F3.A)** / F4.A / F5.B. The diverging fork — F3.B — explicitly broadens scope; v2 re-derives §7 (phasing) / §8 (test-count band) / §11 (audit envelope) / §3.B K8s axes accordingly.
**Forward-scope row**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 169–175.
**Severity / effort**: MED · ~2 days at v1 (F3.A) → **~3.0–3.5 days at v2 (F3.B)** · in-M5-close per forward-scope §6 + §7 Q5. Effort up-revised because F3.B adds Repository-docstring + 5 read-side handler wiring sites + 8 submit-side handler wiring sites + ≥10 new tests. Still ≤ 1.5× forward-scope target (1.5× of 2 days = 3 days; 3.0–3.5 days is on/at the boundary — orchestrator-discretion).
**Test baseline at chunk-open**: 1491 passed / 0 failed / 2 ignored (CH-17 close, cycle hex `40c4d759`, per `_cycle-index.md` line 37 + `ch-17-live-sse-tail-endpoint-40c4d759/cycle-audit.md:52,101`).
**phi-core import baseline at chunk-open**: 57 `use phi_core` statements across the workspace (`grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = 57, verified 2026-05-09).
**Migration baseline at chunk-open**: 16 (highest existing slot `0016_template_a_session_object_grant_add_observe.surql`). **CH-18 ships zero migrations** — re-verified at v2 (no schema change in F3.B; the new event-type `auth_request.access_denied` rides the existing audit_events FLEXIBLE TYPE event_type column).
**ADR number reservation**: highest existing ADR = 0055 (CH-17). **CH-18 reserves ADR-0056**.

---

## Forks for orchestrator (v2 — gate-1 user-lock outcomes)

The chunk has **5 plan-time forks**. v2 records all five as user-locked at gate-1. **F3 was DIVERGENT** — user chose F3.B over the planner's F3.A recommendation; v2 re-plans accordingly. **No new sub-forks required** at v2: the F3.B sub-questions (role-discrimination & Repository-trait-shape) have planner-auto-resolutions that hold under user's locked path, with explicit deferral notes to M6+. The new sub-fork records below are FYI for the orchestrator — none REQUIRE additional user-lock.

| Fork | Locked option | Source | Note |
|---|---|---|---|
| **F1** | **F1.B** — add `PartialEq, Eq` to `PrincipalRef` | planner-recommended (auto-resolved at gate-1) | mechanical; cascade 0; closes CH-14 retro Row 5 type-derive precedent |
| **F2** | **F2.A** — NEW `AuthRequestAccessError` thiserror enum | user-locked at gate-1 | mirrors `FrozenTagViolation` shape (CH-12 ADR-0049 §D49.4) |
| **F3** | **F3.B** — full Repository + dashboard + resolver + bootstrap wiring | **user-locked at gate-1, DIVERGENT from planner-recommended F3.A** | v2 re-plan addresses scope expansion; see §3 wiring table below + sub-forks F3.B.role and F3.B.repo-shape |
| **F4** | **F4.A** — NEW `domain/tests/auth_request_access_props.rs` test file | planner-recommended (auto-resolved at gate-1) | matches per-concern-per-file convention (CH-09/CH-10/CH-11 precedent) |
| **F5** | **F5.B** — NEW `auth_request.access_denied` Alerted-class audit-event | user-locked at gate-1 | follows CH-15 `session.launch_denied` precedent |

### F3.B sub-fork F3.B.role — Observer role-discrimination resolution

Concept doc 02 line 134 names "Observer (admin/auditor)" with read access at every state. The implementation-side classification of "is this caller an admin?" or "is this caller an auditor?" requires an explicit role discriminator:

- **F3.B.role.a** — Plumb `Agent.role: AgentRole` through every dashboard/show/resolver handler signature; enforce role-class checks in the matrix.
- **F3.B.role.b** — **Treat all non-requestor / non-slot-approver / non-bootstrap principals as "Other Agent" (DENY for read-at-every-state); flag as a known gap and file `D-CH18-FOLLOWUP-01` drift to wire admin/auditor classification at M6+.** ← **planner recommendation**
- **F3.B.role.c** — Resolve admin-class via `OrgMembership.role` lookup. **Verified at plan-draft**: `domain::model::nodes::AgentRole` enum exists at `nodes.rs:247-265` with 6 variants (Executive / Admin / Member / Intern / Contract / System) and is plumbed onto `Agent.role` (`nodes.rs:199`). However, NO repository method currently surfaces `AgentRole` against an org-scoped query in O(1) — the dashboard's existing `resolve_viewer_role` at `dashboard.rs:357-379` re-derives admin-class via "first Human in org" heuristic (line 366-372) rather than from `Agent.role`. This means F3.B.role.c would require either: (i) wiring a new `repo.get_agent_role_in_org(viewer, org)` method (Repository-shape change → A5 K8s axis flip), OR (ii) loading the full `Agent` struct in every read handler and inspecting `agent.role`. Both are heavier than (b)'s defer-via-followup-drift.

**Planner recommendation (v2 auto-applied)**: **F3.B.role.b** — defer admin/auditor role-discrimination to M6+ via `D-CH18-FOLLOWUP-01`. This honours the F3.B locked path while keeping the chunk's day-budget within forward-scope (3.0–3.5 days). The matrix's "Observer (admin/auditor) — read at every state" column is partial-honoured: requestor reads are permitted; bootstrap/system-genesis reads are permitted; slot-approvers read their own AR; **all other principals get `Err(NotAuthorisedForRead)`** — including the org's CEO when reading another agent's AR (a known gap; documented as `D-CH18-FOLLOWUP-01` MEDIUM-severity drift; M6+ chunk wires admin classification via either AgentRole or a Permission Check delegation).

**Auto-resolvable at v2**: yes (defer-via-followup-drift is the lighter ship; M6+ chunk consumes). No additional gate-1 user-lock required.

### F3.B sub-fork F3.B.repo-shape — `principal` plumbing through Repository trait

Does `Repository::get_auth_request` / `update_auth_request` / `create_auth_request` / `list_active_auth_requests_for_*` gain a `principal: PrincipalRef` parameter (signature change cascades through every Repository implementation + caller)? OR does the Repository trait remain principal-blind and access-checks live at the handler boundary above Repository?

- **F3.B.repo-shape.a** — Repository methods gain `principal: PrincipalRef` parameter. Pros: no caller can bypass access by calling Repository directly. Cons: signature change cascades through `domain::Repository` trait + 2 implementations (`domain::in_memory::InMemoryRepository` + `store::repo_impl::SurrealRepository`) + ALL existing callsites. Per the v3 chunk-planner cascade-prediction discipline: enumerated AR-method callsites = 24 production sites + ≥30 test fixture sites (verified via `git grep` below). **A5 K8s axis flips to `new blocker introduced`**: every Repository AR method gains a parameter → every implementation needs the new signature. Filed as `CHK8S-D-11` deferred-ledger entry. Plus: system-internal listeners at `events/listeners.rs:170` + 3 resolver sites at `resolvers.rs:{55,122,161}` have NO caller principal context — they would have to pass a synthetic `PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL)` which is architecturally noisy.

- **F3.B.repo-shape.b** — **Repository remains principal-blind. Access checks live at the handler boundary above Repository (handler reads AR via Repository, then calls `check_auth_request_access` separately). The Repository trait gains a docstring contract (per §D56.7 — same precedent as CH-12 tag-write).** ← **planner recommendation**

**Planner recommendation (v2 auto-applied)**: **F3.B.repo-shape.b** — Repository stays principal-blind; handler-layer enforces. Reasoning:
1. CH-12 ADR-0049 §D49.5 + §D49.7 set the precedent for paired-precondition Repository contracts via docstring (the Repository trait module-level docstring + per-method docstring document the future-callsite contract; the validator/access function lives in `domain/src/auth_requests/access.rs` and callers explicitly invoke it).
2. F3.B.repo-shape.a's K8s A5 axis flip is a real architectural cost that the chunk's day-budget shouldn't absorb (parameter cascade hits `in_memory.rs` + `store/repo_impl.rs` + every caller — order-of-magnitude expansion).
3. F3.B.repo-shape.a's "no caller can bypass" upside is partially redundant with the handler-layer enforcement: every AR-touching production handler is enumerated in §3 below; comprehensive coverage is achieved at the handler boundary without cascading the trait.
4. System-internal listeners + resolvers (the 5 sites that have no caller principal) would naturally want to skip the check anyway (these are kernel-internal paths, system-genesis-equivalent); shape-b lets them simply not invoke the check, while shape-a forces them to pass a synthetic principal — strictly more code, strictly more confusion.
5. Future tightening: when M6+ wires AR self-service surfaces, every new AR-touching path consults `check_auth_request_access` at handler-layer per the Repository trait docstring — same pattern. No further trait-shape change required.

**Auto-resolvable at v2**: yes. No additional gate-1 user-lock required.

### F3.B sub-fork F3.B.create-side — submit-side `create_auth_request` wiring

`create_auth_request` is called from **8 submit-side handlers** (verified `git grep` per §3 callsite table). Each handler builds a freshly-constructed AR and calls `repo.create_auth_request(&ar)`. Concept doc 02's matrix `Draft × Submit by requestor → Ok` cell would gate these, but in practice the freshly-constructed AR's `requestor` field is set BY the handler from the caller context — so the gate becomes `ar.requestor == PrincipalRef::Agent(input.actor)` (which is **definitionally true** at the construction site). Two options:

- **F3.B.create-side.a** — **Wire `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)` before every `create_auth_request` call. The check is definitionally redundant (the handler just constructed the AR with `requestor: input.actor`), but provides defence-in-depth: a future refactor where the AR's `requestor` field is constructed from a different source than `input.actor` would surface as a typed-error here rather than silently submitting a mis-attributed AR.** ← **planner recommendation**
- **F3.B.create-side.b** — Skip submit-side wiring; rely on handler-side callsite-authorisation (each handler already validates the caller's authority to submit before constructing the AR via standard auth middleware). The matrix `Draft × Submit by requestor` cell is **structurally honoured** by the handler's own AR-construction code (it can't construct an AR with the wrong requestor unless the caller's input is corrupt).
- **F3.B.create-side.c** — Skip submit-side wiring; document at §6 of the new `m5_2/architecture/auth-request-access-acl.md` page that submit-side gating is structurally honoured at construction.

**Planner recommendation (v2 auto-applied)**: **F3.B.create-side.a** — wire the 8 sites with the redundant check. Reasoning: the user explicitly chose F3.B's "tighten scope — wire Repository layer" path; submit-side wiring is the conservative reading of that scope. Defence-in-depth is the spirit of F3.B. The cost is +8 callsite edits + 0 new behaviour (every check returns Ok in present code) + 0 new tests beyond what's already in §8 (the integration test suite exercises the happy-path of every submit handler and would surface a regression if check_auth_request_access started returning Err for the requestor-on-own-AR path).

**Auto-resolvable at v2**: yes. The submit-side wiring is mechanical and defensive.

### F3.B sub-fork F3.B.list-filter — list-side post-filter shape

Dashboard reads at `dashboard.rs:273` (`list_active_auth_requests_for_org`) + `dashboard.rs:293` (`list_adoption_auth_requests_for_org`) return Vec<AuthRequest>. The matrix-aware shape: should the handler **post-filter** the list to exclude ARs the viewer cannot read?

- **F3.B.list-filter.a** — **Post-filter the list at the handler boundary: for each AR in the returned Vec, call `check_auth_request_access(&ar, &PrincipalRef::Agent(viewer_agent_id), IntendedOp::Read)`; retain only the ones that return Ok. No audit-event emission for filtered-out entries (silent post-filter; the viewer never knows the AR existed — same UX shape as the existing `agent.archived_at IS NOT NULL` filter).** ← **planner recommendation**
- **F3.B.list-filter.b** — Return the full list; let the dashboard frontend handle filter UX. Rejected: defeats the purpose of F3.B (per-state ACL would be advisory rather than enforcing).
- **F3.B.list-filter.c** — Return the full list AND emit `auth_request.access_denied` for every filtered entry. Rejected: noisy (a dashboard render would emit ≥10 access-denied events per non-admin viewer).

**Planner recommendation (v2 auto-applied)**: **F3.B.list-filter.a** — silent post-filter with the access function. Reasoning: dashboard rendering is a high-frequency read; emitting an audit-event per filtered AR would multiply audit-event volume by ~10×. The matrix `Pending × Read by Other Agent → Err` is silently honoured by hiding the AR from the list. The 4 production mutation callsites (templates/{approve,deny,revoke} + projects/create slot-fill) DO emit on Err (per F5.B locked) — those are explicit-action paths where the viewer is asserting an action; silent denial there would lose the audit trail. List-side reads are passive observation and warrant the silent filter.

**Auto-resolvable at v2**: yes. **Important regression-risk note**: the dashboard currently shows ALL active org ARs to the viewer; F3.B.list-filter.a's silent filter MAY reduce the visible count for non-admin viewers — the chunk's pause-discipline (§7 P2) calls this out; existing dashboard happy-path tests must continue to pass; if a happy-path test asserts "viewer X sees AR Y" and viewer X is no longer authorised under the matrix, that test gets re-evaluated as a documented behavioural change.

### Summary — gate-1 outcome (v2)

- **5 user-locked top-level forks**: F1.B / F2.A / F3.B / F4.A / F5.B.
- **4 sub-forks under F3.B**, all auto-resolvable: F3.B.role.b (defer-via-followup-drift) / F3.B.repo-shape.b (principal-blind Repository) / F3.B.create-side.a (wire 8 submit sites) / F3.B.list-filter.a (silent post-filter).
- **NO new gate-1 user-lock required at v2**. All sub-fork resolutions are conservative-defensive readings of the user's locked F3.B path; M6+ admin/auditor role plumbing is filed as `D-CH18-FOLLOWUP-01`.

---

## §1 — Context & principle

**Why this chunk**: Drift D-new-12 (MEDIUM; security-boundary). Concept doc 02 §"Per-State Access Matrix" lines 130–144 specifies that access to an AuthRequest record itself varies by state. The repository today accepts reads + writes without consulting these state-dependent rules. v2 (F3.B locked) closes D-new-12 by:
- Shipping a typed `check_auth_request_access(ar, principal, intended_op) -> Result<(), AuthRequestAccessError>` function.
- Wiring it into **every production AR-touching handler**: 8 submit (create_auth_request) + 5 read (dashboard×2 + slot-fill read + 1 cascade-internal skip + 1 audit-class-resolution skip) + 4 mutation (templates/{approve,deny,revoke} + projects/create slot-fill).
- Emitting an Alerted-class `auth_request.access_denied` audit-event at every Err return from the **4 mutation callsites only** (per F3.B.list-filter.a — silent post-filter at list-side reads).
- Updating the Repository trait docstring to document the future-callsite contract (per §D56.7).
- Filing `D-CH18-FOLLOWUP-01` to defer admin/auditor role-discrimination to M6+ (per F3.B.role.b).
- Filing `CHK8S-D-NN` IF F3.B.repo-shape.a were chosen (it is NOT — repo-shape.b stays K8s-neutral; no new ledger entry filed at v2).

**Quality-over-speed restatement**: *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* CH-18 v2 application: the per-state matrix is concept-locked at `permissions/02-auth-request.md` lines 130–144 verbatim; the typed function captures every row + column from that table; F3.B (full wiring) honours the user's locked tightening of scope; the residual deferral (admin/auditor role-discrimination) is documented as a NEW drift (`D-CH18-FOLLOWUP-01`) with explicit re-scope to M6+, NOT silently ignored.

**Forward-scope reference**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 row 18 (CH-18 entry, lines 169–175); §6 severity row line 426 (`MED, 2d, permissions/02, —, yes`); §7 Q5 binding decision (line 478) — MED chunks evaluate close-at-M5 vs defer per chunk-open. v2 effort estimate (3.0–3.5d) is at the boundary of the 1.5× scope ratio; orchestrator gate-1 user-lock to F3.B explicitly accepted the broader scope.

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close (v2 / F3.B) |
|---|---|---|---|---|
| `concepts/permissions/02-auth-request.md` | §"Per-State Access Matrix" lines 130–144 — full table of (state × principal-class × allowed ops) for the 9 AR states | silent-in-code (matrix is documented; no function captures it; no callsite consults it) | **honored** — typed function `check_auth_request_access` captures the matrix verbatim; **17 production callsites** consult it (4 mutation + 8 submit + 2 list + 1 slot-fill read + 2 NO-OP-system-internal skips); admin/auditor read-at-every-state column partially-honoured per `D-CH18-FOLLOWUP-01` deferral note |
| `concepts/permissions/02-auth-request.md` | §"State Machine" lines 70–129 — 9 request-level states + transition arrows | honored (`AuthRequestState` enum + `aggregate_request_state` + `legal_request_transition` since M1/P4) | honored (no change — CH-18 reads the state, doesn't mutate the table) |
| `concepts/permissions/02-auth-request.md` | §"Multi-Approver Dynamics" lines 175–179 — *"While a slot is `Unfilled`, only that approver can modify it. Once filled, the slot is read-only to other approvers, but the slot-holder can reconsider their own slot until the request reaches a closed terminal state. The resource owner can override any slot."* | partially-honored — slot mutation is gated by state-machine legality; principal-of-caller is not asserted | **honored** at the 4 mutation callsites + the slot-fill read at projects/create:636 (the new function gates principal-of-caller against the matrix); resource-owner-override is approximated via `ar.requestor == principal` for adoption-AR self-revoke + accepts as stub for general owner-class until resource-owner-lookup helper lands in M6+ (per `D-CH18-FOLLOWUP-01`). |
| `concepts/permissions/02-auth-request.md` | line 17 — *"Slots are independent and atomic"* | honored (per-slot transitions live; ApproverSlot::reconsider preserves slot-independence) | honored (no change) |
| `concepts/permissions/02-auth-request.md` | §"Routing Table on the Resource" lines 154–173 — *"Routing is optional; without a routing table, all requests go to the owner"* + delegated-router has `[approve]` (refinement of `allocate`) | concept-aspirational — RoutingTable struct does not exist | concept-aspirational (no change) |
| `concepts/permissions/02-auth-request.md` | §"Open Questions" line 506–508 — cross-org consent-policy edge cases | concept-aspirational (out-of-scope) | concept-aspirational (no change) |
| `concepts/permissions/03-action-vocabulary.md` | §"Standard Action Vocabulary" line 7–22 — closed 34-verb set + 1 wildcard; `Action::CANONICAL.len() == 34` invariant; line 22 is the Action × Category table; line 44 is the universal-applicability claim *"Discovery, Authority, and Observability apply universally"* | honored (`Action::CANONICAL.len() == 34` invariant test passes per CH-04) | honored (CH-18 does NOT extend the action vocabulary — see §3.D) |
| `permissions/README.md` | §"Closed action vocabulary + concept invariants" | honored (cross-doc invariants stable) | honored |

**phi-core overlap**: none — `AuthRequest` is a baby-phi domain composite with no phi-core counterpart.

## §3 — phi-core leverage map

CH-18 introduces zero new phi-core overlap (unchanged at v2). The new types (`AuthRequestAccessError`, `IntendedOp`, `auth_request.access_denied` event) sit entirely within `domain::auth_requests::access` + `domain::audit::events::m5_2::auth_request_access`.

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::types::event::AgentEvent` | imported at `server::session::events`, `server::session::live_stream`, `domain::session_recorder` | direct-reuse | none — orthogonal to AR governance |
| `phi_core::session::Session` | imported at `server::session` modules | direct-reuse | none — AR is not a session |
| `phi_core::*` (any other) | various | direct-reuse / wrap (per existing pattern) | none — CH-18 is wholly within `domain::auth_requests` + `domain::audit::events` |

**Expected import-count delta at chunk close**: **+0** phi-core imports (unchanged at v2 — F3.B expands within baby-phi only; no new phi-core consumption).

**Positive close-audit greps**:

```bash
# Baseline — must be unchanged at chunk close.
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (chunk-open baseline). Drift triggers AskUserQuestion.

# CH-18 substrate exists.
grep -rn "check_auth_request_access\|AuthRequestAccessError" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# v2 expectation (F3.B): ≥ 23 (1 def + 1 enum + 17 production callsites + ≥ 4 mod-test sites).
# (v1 / F3.A would have been ≥ 8.)

# Audit-event builder exists (F5.B locked).
grep -rn "auth_request_access_denied\|auth_request.access_denied" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# v2 expectation: ≥ 7 (1 builder + 4 mutation production callsites — per F3.B.list-filter.a — + ≥ 2 audit-event module tests).
```

**Forbidden-duplication greps** (every one MUST return 0):

```bash
# No shadow types.
grep -rn "^pub struct AuthRequest \|^struct AuthRequest " /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::\|crates/domain/src/model/nodes.rs:818"
# Expect: 0.

# No cargo-cult of phi-core types.
grep -rn "^pub struct AgentEvent\|^pub struct Session " /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::\|target/"
# Expect: 0.

# No silent re-implementation of the per-state matrix as inline code in handlers.
grep -rn "match.*ar\.state\|match.*request\.state\|match.*req\.state" /root/projects/phi/baby-phi/modules/crates/server/src/platform/ | grep -v "// \|/\*\|target/"
# v2 expectation: ≤ 12 (existing handler state-name extractors + dashboard render-class state-keys; F3.B does NOT add new inline state-matchers — the matrix lives in access.rs).
```

### §3 — Full F3.B callsite enumeration (v2)

**Mandatory cascade-prediction discipline (chunk-planner v3 — refined per CH-13 retro)**: the v2 re-plan re-runs the exhaustive `git grep` against the full workspace and pastes the per-file breakdown.

```bash
# AR Repository method callsites (production code only, excluding tests/in_memory.rs/store/repo_impl.rs/repository.rs trait def).
git -C /root/projects/phi/baby-phi grep -nE "(create|get|update|list_active_auth_requests_for_|list_adoption_auth_requests_for_)_auth_request\b" modules/crates/ | grep -v "test\|/in_memory\|/repo_impl\|/repository.rs\|src/lib.rs"
```

**Raw matched-line count**: ~28 production lines + ~12 trait-def + impl-stub lines = ~40 total `git grep` matches; the 17 production callsites (CH-18 wiring scope) listed below.

**Per-file callsite breakdown** (v2 / F3.B exhaustive):

| # | File:line | Method | Current behaviour | Required v2 wiring | IntendedOp | Principal source |
|---|---|---|---|---|---|---|
| **1** | `server/src/platform/templates/approve.rs:80` | `update_auth_request(&next)` | upstream `transition_slot` already validates state-machine legality; no principal check | upstream of line 78 `transition_slot` insert: `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Approve)?;` On `Err(e)` → emit `auth_request_access_denied(...)` audit-event then return `Err(TemplateError::AccessDenied(e))` | `Approve` | `PrincipalRef::Agent(input.actor)` (already in `ApproveInput`) |
| **2** | `server/src/platform/templates/deny.rs:85` | `update_auth_request(&next)` | analogous | analogous; on `Err` → audit-event + typed error | `Deny` | `PrincipalRef::Agent(input.actor)` |
| **3** | `server/src/platform/templates/revoke.rs:84` | `update_auth_request(&next)` | analogous | analogous; on `Err` → audit-event + typed error | `Revoke` | `PrincipalRef::Agent(input.actor)` |
| **4** | `server/src/platform/projects/create.rs:658` | `update_auth_request(&next)` (slot-fill mutation in `approve_pending_shape_b`) | slot-locate already validates `input.approver_id` is in a slot; no principal check against the matrix | upstream of line 656 `transition_slot`: `check_auth_request_access(&ar, &PrincipalRef::Agent(input.approver_id), op)?;` where `op` = `Approve` if `input.approve` else `Deny`; on `Err` → audit-event + typed error | `Approve` or `Deny` | `PrincipalRef::Agent(input.approver_id)` |
| **5** | `server/src/platform/projects/create.rs:636` | `get_auth_request(input.ar_id)` (read-before-mutate in `approve_pending_shape_b`) | unauthenticated read | upstream of line 636: build the AR via the existing `repo.get_auth_request`; THEN call `check_auth_request_access(&ar, &PrincipalRef::Agent(input.approver_id), IntendedOp::Read)?;` (the slot-fill flow needs to read the AR before mutating it; the matrix's `Pending × Read by slot-approver → Ok` cell allows this) | `Read` | `PrincipalRef::Agent(input.approver_id)` |
| **6** | `server/src/platform/orgs/dashboard.rs:273` | `list_active_auth_requests_for_org(org_id)` (returns `Vec<AuthRequest>`) | unauthenticated org-scoped list | post-filter on the returned Vec: retain entry IF `check_auth_request_access(&ar, &PrincipalRef::Agent(viewer_agent_id), IntendedOp::Read).is_ok()` per F3.B.list-filter.a (silent post-filter; no audit-event per filtered entry) | `Read` | `PrincipalRef::Agent(viewer_agent_id)` |
| **7** | `server/src/platform/orgs/dashboard.rs:293` | `list_adoption_auth_requests_for_org(org_id)` (returns `Vec<AuthRequest>`) | unauthenticated | post-filter analogous to row 6 | `Read` | `PrincipalRef::Agent(viewer_agent_id)` |
| **8** | `server/src/platform/orgs/show.rs:63` | `list_adoption_auth_requests_for_org(id)` (only `.len()` is consumed — count-aggregate) | unauthenticated count | F3.B.list-filter.a applied: post-filter the Vec on the access predicate, then count. Requires plumbing `viewer: AgentId` through `show_organization` signature. | `Read` | `PrincipalRef::Agent(viewer_agent_id)` (NEW arg on `show_organization`) |
| **9** | `server/src/platform/templates/mod.rs:178` | `list_adoption_auth_requests_for_org(org)` inside `find_adoption_ar` helper | unauthenticated; called by approve/deny/revoke handlers BEFORE the principal context is meaningful (the handler's caller IS the AR-mutation caller — but `find_adoption_ar` is a helper that doesn't know the caller's principal yet) | **NO wiring at this site** — `find_adoption_ar` is a kernel-internal helper consumed by approve/deny/revoke handlers. The mutation callsite (rows 1-3) DOES gate via `check_auth_request_access` after the helper returns. The helper itself stays principal-blind (CH-12 precedent — domain helpers don't take principals; handlers do). Documented as a KNOWN FAST-PATH skip in §6 of `m5_2/architecture/auth-request-access-acl.md`. | n/a | n/a (helper) |
| **10** | `server/src/platform/templates/revoke.rs:115` | `get_auth_request(cascaded_ar_id)` inside the cascade-revoke loop | unauthenticated; the cascade is system-internal (kernel-driven by the parent revoke) | **NO wiring at this site** — cascade revoke is system-genesis-equivalent (the parent revoke at row 3 gates the user-facing entry; the cascade fires from the kernel's revoke-graph traversal, not a fresh user action). Documented as a KNOWN FAST-PATH skip. The cascade's `update_auth_request(&next_cascaded)` at line 131 also stays unwired — kernel-internal. | n/a | n/a (kernel cascade) |
| **11** | `server/src/bootstrap/claim.rs:307` | `apply_bootstrap_claim(&claim)` (transaction wrapping `create_auth_request` + 4 other inserts) | unauthenticated; bootstrap is the system-genesis principal | **NO wiring at this site** — `apply_bootstrap_claim` is the system-genesis-AR creation path; `is_bootstrap_ar(&ar)` returns true on the freshly-built AR (per CH-14 ADR-0053 §D53.2); `check_auth_request_access` would trivially pass via the `classify_principal` system-genesis fast-path. Documented as a KNOWN FAST-PATH skip. | n/a | n/a (bootstrap) |
| **12** | `server/src/platform/templates/adopt.rs:96` | `create_auth_request(&ar)` (Template adoption submit) | unauthenticated submit; AR is freshly constructed with `requestor: input.actor` | F3.B.create-side.a: insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)?;` BEFORE the `create_auth_request` call. Definitionally redundant (`ar.requestor == input.actor` by construction); defence-in-depth. | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **13** | `server/src/platform/projects/create.rs:470` | `create_auth_request(&ar)` (Project Shape B submit — 2-slot AR creation) | unauthenticated submit | F3.B.create-side.a: insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)?;` | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **14** | `server/src/platform/defaults/put.rs:88` | `create_auth_request(&ar)` (org defaults change submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **15** | `server/src/platform/secrets/add.rs:107` | `create_auth_request(&ar)` (secret submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **16** | `server/src/platform/mcp_servers/register.rs:92` | `create_auth_request(&ar)` (MCP register submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **17** | `server/src/platform/mcp_servers/patch_tenants.rs:98` | `create_auth_request(&ar)` (MCP tenants patch submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **18** | `server/src/platform/mcp_servers/archive.rs:54` | `create_auth_request(&ar)` (MCP archive submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **19** | `server/src/platform/model_providers/register.rs:124` | `create_auth_request(&ar)` (model provider register submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **20** | `server/src/platform/model_providers/archive.rs:60` | `create_auth_request(&ar)` (model provider archive submit) | unauthenticated submit | F3.B.create-side.a | `Submit` | `PrincipalRef::Agent(input.actor)` |
| **21** | `domain/src/events/listeners.rs:170` | `get_auth_request(adoption_ar_id)` (kernel listener — composes `audit_class` from AR's tier) | unauthenticated; system-internal (event-bus listener after `events.publish_*`) | **NO wiring at this site** — kernel listeners are system-genesis-equivalent (run after publish; no user-facing principal in scope). Documented as a KNOWN FAST-PATH skip. | n/a | n/a (kernel listener) |
| **22** | `server/src/platform/projects/resolvers.rs:55` | `list_adoption_auth_requests_for_org(org.id)` (Template A AR resolver) | unauthenticated; system-internal | **NO wiring** — same fast-path as row 21 (kernel resolver). | n/a | n/a |
| **23** | `server/src/platform/projects/resolvers.rs:122` | `list_adoption_auth_requests_for_org(org)` (Template C AR resolver) | unauthenticated; system-internal | **NO wiring** — same. | n/a | n/a |
| **24** | `server/src/platform/projects/resolvers.rs:161` | `list_adoption_auth_requests_for_org(org.id)` (Template D AR resolver) | unauthenticated; system-internal | **NO wiring** — same. | n/a | n/a |

**Summary of v2 wiring scope** (vs v1 / F3.A scope):
- v1 / F3.A: rows 1, 2, 3, 4 = **4 callsites wired** + 0 kernel-skip docs.
- v2 / F3.B: rows 1–8 + 12–20 = **17 callsites wired** (8 submit + 5 read + 4 mutation) + rows 9, 10, 11, 21, 22, 23, 24 = **7 callsites explicitly documented as kernel-internal fast-path skips**.

**Test count delta v2 vs v1**: +13 callsites under wiring scope ⇒ approximately +5 to +8 new integration tests beyond v1's 23-test MUST-SHIP. v2 MUST-SHIP target: **30 tests** (see §8).

**§3 cascade-artifact discipline (CH-13 / CH-14 / CH-15 retro precedent — chunk-planner v9 §3 cascade-artifact rules)**:

CH-18 v2 introduces NO struct-field cascade (no AR field added) and NO additive enum-variant cascade through callsite `match` arms. Per chunk-planner v9 §3 enumeration discipline applied to v2's broader scope:

**Artifact A (struct-cascade)** — N/A. The chunk adds no fields to `AuthRequest`/`ApproverSlot`/`ResourceSlot`. F1.B's `PartialEq` derive is a single-line trait-derive change; cascade is purely additive.

**Artifact B (concept-doc multi-step placement)** — N/A.

**Artifact C (test-amendment cascade)** — **partial** at v2: F3.B.list-filter.a's silent post-filter at dashboard.rs:273,293 may change the dashboard test fixture results IF an existing test asserts "viewer X sees AR Y" where viewer X is now denied by the matrix. v2 mitigation: §7 P2 pause-discipline lists "regress dashboard happy-path test count" as a pause trigger. Predicted impact: ≤ 2 existing dashboard tests need updates (the dashboard test suite at `server/tests/dashboard_*` exercises happy-path admin views; admin viewers are documented in `D-CH18-FOLLOWUP-01` as currently classified "Other Agent" and DENIED — this WILL break the existing tests UNLESS the test fixtures use the ar.requestor or slot-approver as viewer). **v2 contingency**: if pause-discipline triggers (≥ 3 existing test breakages), the implementer escalates to user via AskUserQuestion at gate-2 P2 review; possible follow-up is to change the test fixtures to use slot-approver-as-viewer, or relax `D-CH18-FOLLOWUP-01` to ship admin-class read-bypass in this chunk. Pre-flight `git grep` on dashboard test fixtures:

```bash
git -C /root/projects/phi/baby-phi grep -nE "viewer_agent_id|dashboard_summary" modules/crates/server/tests/ | head -20
```

Result (verified 2026-05-09):
- `server/tests/dashboard_persistence_test.rs` — 1 test using ceo viewer
- `server/tests/dashboard_view_test.rs` — multiple tests using ceo viewer (Admin role) + non-admin viewers
- `server/tests/dashboard_alerted_24h_test.rs` — uses ceo viewer
- `server/tests/dashboard_recent_events_test.rs` — uses ceo viewer

**Estimated test impact**: 0–4 existing dashboard tests will need fixture adjustments to use slot-approver/requestor as viewer for AR-bearing assertions; the bulk of dashboard tests assert agent count / project count / pending_auth_requests_count integers — those still pass because the ceo IS the AR-creator/slot-approver in the typical fixture. **Test-count delta**: any breaking test gets re-stated to assert post-filter behaviour; +0 net tests but +N test fixture amendments (counts as Artifact-C cascade).

**Artifact D (additive enum-variant cascade)** — N/A.

### §3.B — K8s microservice readiness check (v2 re-evaluated)

The chunk's surface at v2: new module `auth_requests::access`, new module `audit::events::m5_2::auth_request_access`, 17 server handler emission/wiring sites, 1 trait-derive on `PrincipalRef`, 7 kernel-skip documentation sites, Repository trait docstring update on 2 methods (`get_auth_request` + `update_auth_request` per §D56.7).

| Axis | What to check | This chunk's surface (v2) | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | `AuthRequestAccessError` is a thiserror enum (zero-state); `IntendedOp` is a copy-Cell-style enum; no shared state at v2 | no | none |
| **A2** | New IPC channel | none — the function is a pure synchronous predicate on `(&AuthRequest, &PrincipalRef, IntendedOp)` | no | none |
| **A3** | New pod-local resource | none | no | none |
| **A4** | Migration runner / first-apply race | **CH-18 v2 ships zero migrations.** No schema change. The `audit_events` table absorbs the new `auth_request.access_denied` event-type without migration (event_type is FLEXIBLE TYPE per migration 0001) | no | none |
| **A5** | Trait-shape requirement | **F3.B.repo-shape.b chosen (planner-recommended; auto-resolved at v2)**: Repository trait stays principal-blind. Methods get docstring updates (no method-signature change) per §D56.7. ADR-0033 §D33.1 (`SessionRegistry` trait) untouched. **If F3.B.repo-shape.a had been chosen, A5 would have flipped to "new blocker introduced"; v2 explicitly does NOT take that path.** | **no** (under repo-shape.b) | none — no `CHK8S-D-NN` entry filed at v2 |
| **A6** | Cross-pod state sharing | none — the predicate consumes a (possibly already-loaded) `&AuthRequest` snapshot + a `&PrincipalRef` from the request context; both flow through standard handler-side data plumbing | no | none |
| **A7** | Audit hash-chain symmetry | The new `auth_request.access_denied` event-type's `canonical_bytes()` excludes `prev_event_hash`. Single-writer guarantee preserved. CH-12 plan §3.B A7 + CH-15 plan §3.B A7 precedent. Emission frequency at v2: 4 mutation-callsite sites × ~1 event-per-rejected-request — not a high-volume axis. List-side reads do NOT emit (per F3.B.list-filter.a) — so no high-frequency hash-chain hot path. | no | none |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP)** (re-checked at v2):
- D33.1 (`SessionRegistry` trait): unchanged.
- D33.2 (`SurrealStore::open_remote`): unchanged.
- D33.3 (SIGTERM graceful shutdown): unchanged.
- D33.4 (`EventBus.shutdown` + `drain`): unchanged.

**Conclusion (v2)**: **K8s-neutral** under F3.B.repo-shape.b. No new blockers; existing CHK8S-D-01 through CHK8S-D-10 ledger entries unchanged. **No new `CHK8S-D-NN` entry filed at v2.** The `D-CH18-FOLLOWUP-01` drift (admin/auditor role-discrimination) is a domain-layer concern, NOT a K8s axis concern.

### §3.C — User-facing documentation impact map (v2 expanded)

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/auth-request-access-acl.md` (NEW) | yes — NEW page documenting the per-state matrix + 17-callsite wiring map + 7-callsite kernel-internal skip-list + the F2.A error-shape rationale + the F5.B audit-event shape + F3.B sub-fork resolutions + `D-CH18-FOLLOWUP-01` deferral | (a) update in-chunk — ship as new architecture page in P3 |
| **Architecture** | `docs/specs/v0/implementation/m1/architecture/audit-events.md` | yes — register the new `auth_request.access_denied` event-type | (a) update in-chunk |
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/repository-trait-contract.md` (or equivalent) | yes — Repository trait docstring update on `get_auth_request` + `update_auth_request` per §D56.7 — needs to be captured in any consolidated Repository contract page if one exists; pre-flight grep showed NO such page exists today, so the docstring-only update is sufficient + the new `auth-request-access-acl.md` §6 documents the kernel-internal skip-list. | (a) docstring-only update — no separate doc page needed |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/auth-request-access-acl-operations.md` (NEW) | yes — NEW page documenting `auth_request.access_denied` event-type's audit-class (Alerted), retention (365+ days), delivery channel (org alert), expected emission rate (low — only on principal-mismatch at 4 mutation sites), troubleshooting playbook, error-code reference, list-side silent-filter operator-narrative | (a) update in-chunk |
| **User-guide** | `m5/user-guide/troubleshooting.md` (existing, AMEND) | yes — amend with "CH-18 amendment — auth_request.access_denied troubleshooting (2026-05-09)" subsection per amend-don't-add discipline | (a) update in-chunk |

All §3.C rows are **(a) update in-chunk** — no deferrals. The full map runs as P3 deliverables.

### §3.D — Forward-scope-vs-concept-doc precedence (mandatory pre-flight grep)

**Mandatory pre-flight check (chunk-planner v9 — strengthened per CH-15+CH-17 retros)**: BEFORE writing any §3.D contradiction claim, the planner ran the closed-set verification grep. This is **re-verified at v2** since F3.B's broader scope might have implied new action verbs:

```bash
git -C /root/projects/phi/baby-phi grep -nE '^\s*(Discover|List|Inspect|Read|Modify|Approve|Escalate|Allocate|Delegate|Observe),$' modules/crates/domain/src/permissions/action.rs
```

Output (verified 2026-05-09):
```
33:    Discover,
34:    List,
35:    Inspect,
37:    Read,
42:    Modify,
54:    Delegate,
55:    Approve,
56:    Escalate,
57:    Allocate,
73:    Observe,
```

All AR-touching action verbs the matrix names (`read`, `modify`, `submit`/Approve, `cancel`, `approve`, `deny`/Approve-denied-variant, `revoke`/Approve-reconsidered, `read_only-audit`/Observe) are **already in `Action::CANONICAL`**. The `IntendedOp` enum in CH-18 v2 is a SEPARATE closed set scoped to AR access semantics — it is NOT a member of `Action::CANONICAL`.

**Closed-set invariant preserved**: `Action::CANONICAL.len() == 34` after CH-18 v2 chunk close. Re-verified.

**No CRITICAL forks for orchestrator escalation on §3.D grounds** (re-verified at v2).

## §4 — Drifts closed (v2 expanded)

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-12` | [`m5_1/drifts/D-new-12.md`](../../v0/implementation/m5_1/drifts/D-new-12.md) | MEDIUM | `discovered` → `remediated` | Per-state ACL captured as typed predicate; matrix table verbatim from concept doc 02 lines 130–144; **17 production callsites + 7 documented kernel-skip callsites** consult/skip the predicate per F3.B locked path; Repository trait docstring update per §D56.7 documents future-callsite contract; admin/auditor role-discrimination deferred to `D-CH18-FOLLOWUP-01`. |
| `D-CH18-FOLLOWUP-01` (NEW at v2) | `m5_1/drifts/D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md` (NEW; created at P0 scaffold) | MEDIUM | discovered (filed at chunk-open) | Concept doc 02 line 134 names "Observer (admin/auditor) — read at every state"; CH-18 v2 implements requestor + slot-approver + bootstrap/system-genesis read-allow but NOT admin/auditor read-allow (per F3.B.role.b deferral). Owners: M6+ — wire admin classification via either `Agent.role` lookup or Permission Check delegation. Cross-ref: ADR-0056 §D56.5 + §D56.8 (NEW sub-decision at v2). |

## §5 — ADRs drafted (v2 expanded sub-decisions)

**ADR number reserved**: **ADR-0056** — *AuthRequest per-state ACL enforcement* (status: `Proposed` at plan draft → `Accepted` at chunk seal).

**Decision summary** (one line, v2): *"Capture concept-doc 02's per-state access matrix as a typed pure-function `check_auth_request_access(&AuthRequest, &PrincipalRef, IntendedOp) -> Result<(), AuthRequestAccessError>`; wire into 17 production AR-touching handlers (8 submit + 5 read + 4 mutation) per F3.B locked path; emit `auth_request.access_denied` Alerted-class audit-event at the 4 mutation callsites only (per F3.B.list-filter.a silent post-filter); Repository trait stays principal-blind with docstring contract per §D56.7; admin/auditor role-discrimination deferred to M6+ via `D-CH18-FOLLOWUP-01` (per F3.B.role.b)."*

**Sub-decisions** (each with `D56.N` numbering — v2 adds D56.8, D56.9, D56.10):

- **§D56.1** — typed function shape: pure-function `fn check_auth_request_access(ar: &AuthRequest, principal: &PrincipalRef, intended_op: IntendedOp) -> Result<(), AuthRequestAccessError>`. No `&self`, no Repository, no async — domain-layer purity per the existing `auth_requests::transitions` precedent.
- **§D56.2** — `IntendedOp` enum captures the operation-kind dimension of the matrix. Variants: `Read`, `Modify`, `Submit`, `Cancel`, `Approve`, `Deny`, `Reconsider`, `Revoke`, `OverrideApprove`, `CloseAsDenied`, `Expire` — exactly the 11 columns of the concept-doc matrix. NOT a member of `Action::CANONICAL`.
- **§D56.3** — `AuthRequestAccessError` typed-error enum at `domain/src/auth_requests/access.rs`. Variants: `NotAuthorisedForRead { state, principal_kind }`, `NotAuthorisedForModify { state, principal_kind }`, `OperationForbiddenInState { state, intended_op }`, `RequestorOnlyOperation { ... }`, `UnfilledApproverSlotOnly { ... }`. **F2.A user-locked.**
- **§D56.4** — `PrincipalRef` derives `PartialEq, Eq` (additive trait-derive). **F1.B planner-recommended; auto-approved at gate-1.**
- **§D56.5** — wiring depth: **F3.B (full wiring) — user-locked at gate-1, DIVERGENT from planner's F3.A recommendation.** Wiring spans 17 production callsites: (a) 4 mutation handlers at `templates/{approve,deny,revoke}.rs` + `projects/create.rs:658` slot-fill mutation; (b) 5 read sites at `dashboard.rs:273,293`, `show.rs:63`, `projects/create.rs:636` slot-fill read, plus the slot-fill flow's principal-of-caller assertion; (c) 8 submit sites at `templates/adopt.rs:96`, `projects/create.rs:470`, `defaults/put.rs:88`, `secrets/add.rs:107`, `mcp_servers/{register,patch_tenants,archive}.rs`, `model_providers/{register,archive}.rs`. Per §D56.8: 7 kernel-internal sites are explicitly documented as fast-path skips. The "resource-owner-lookup helper" is DESCOPED from CH-18 — the matrix cell `Approved × Revoke by resource-owner` is approximated via `ar.requestor == principal` for adoption-AR self-revoke and accepts as stub for general owner-class until resource-owner-lookup helper lands in M6+ (per `D-CH18-FOLLOWUP-01` cross-reference + the deferred user-fork at gate-1).
- **§D56.6** — audit-event emission: **F5.B user-locked.** Emit `auth_request.access_denied` (Alerted) at every Err return from `check_auth_request_access` at the **4 mutation callsites only** per F3.B.list-filter.a silent post-filter rule (list-side reads filter silently). Pre-existing-behaviour preservation note: existing 4 handlers' typed-error → HTTP 4xx behaviour is **unchanged**; the new audit-event is **additive**; no existing test assertion is broken.
- **§D56.7** — Repository trait docstring update: `get_auth_request` + `update_auth_request` gain a contract sentence *"Callers in handler-layer surfaces MUST pair their consumption with `domain::auth_requests::access::check_auth_request_access` per ADR-0056 §D56.5 future-callsite contract; system-internal kernel paths (event-bus listeners, AR resolvers, cascade-revoke loops) are explicit fast-path skips per ADR-0056 §D56.8"*. Mirrors CH-12's Repository docstring tag-write contract precedent.
- **§D56.8** (**NEW at v2**) — kernel-internal fast-path skip-list. The following 7 callsites do NOT invoke `check_auth_request_access` because they execute as system-internal paths with no caller principal context:
  - `events/listeners.rs:170` (kernel listener — composes `audit_class`)
  - `templates/revoke.rs:115,131` (cascade-revoke loop, system-internal kernel cascade)
  - `bootstrap/claim.rs:307` (bootstrap AR creation — system-genesis equivalent)
  - `templates/mod.rs:178` (`find_adoption_ar` helper — handler gates after helper returns)
  - `projects/resolvers.rs:55` (Template A AR resolver — kernel event-bus)
  - `projects/resolvers.rs:122` (Template C AR resolver — kernel event-bus)
  - `projects/resolvers.rs:161` (Template D AR resolver — kernel event-bus)

  Rationale: CH-12 ADR-0049 §D49.5 + §D49.7 set the precedent that kernel-internal helpers stay principal-blind; access-checks live above the kernel boundary. The §D56.7 Repository docstring + the NEW `m5_2/architecture/auth-request-access-acl.md` §6 explicitly enumerate this skip-list to prevent future drift (a successor chunk that adds a new kernel listener consulting AR's `audit_class` is told here that it does NOT need to gate; a successor chunk that adds a new user-facing HTTP handler IS told that it DOES need to gate).
- **§D56.9** (**NEW at v2**) — sub-fork resolutions under F3.B:
  - **F3.B.role.b** chosen: defer admin/auditor role-discrimination to M6+ via `D-CH18-FOLLOWUP-01`. Concept doc 02's "Observer (admin/auditor) — read at every state" column is partial-honoured at v2; M6+ chunk wires admin classification.
  - **F3.B.repo-shape.b** chosen: Repository trait stays principal-blind; docstring contract enforced. K8s A5 axis stays neutral.
  - **F3.B.create-side.a** chosen: 8 submit sites get the redundant defence-in-depth check. Per-callsite cost: +1 line per submit handler.
  - **F3.B.list-filter.a** chosen: silent post-filter at the 5 list-side reads; no audit-event per filtered entry.
- **§D56.10** (**NEW at v2**) — divergent-fork audit-trail note. Planner v1 recommended F3.A; user gate-1 locked F3.B. v2 re-plan honours F3.B's tightened scope. The auto-approval criteria re-evaluated in v2 are documented in this plan's gate-1 banner; the implementer at P0 records the gate-1 outcome in this ADR's Forks header.

**Cross-references**:
- (a) Originating concept-doc + section + line range: `docs/specs/v0/concepts/permissions/02-auth-request.md` §"Per-State Access Matrix" lines 130–144 + §"Multi-Approver Dynamics" lines 175–179.
- (b) Closed drift(s) by ID: D-new-12; NEW drift `D-CH18-FOLLOWUP-01` filed at chunk-open and remains `discovered`-state at chunk-close.
- (c) Prior ADRs cited as precedent (with milestone-prefixed paths per chunk-planner v6 / CH-08 retro Row 1):
  - [`m1/decisions/0008-auth-request-state-machine.md`](../../../v0/implementation/m1/decisions/0008-auth-request-state-machine.md) — original AR state machine ADR.
  - [`m5_2/decisions/0049-frozen-session-tag-immutability.md`](../../../v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md) — typed-violation-enum + Repository docstring contract precedent.
  - [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](../../../v0/implementation/m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md) — `is_bootstrap_ar` two-witness + system-internal cascade precedent.
  - [`m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md`](../../../v0/implementation/m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md) — Alerted-class deny audit-event pattern.
- (d) Forward-scope row: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §1 lines 169–175 + §5 row 18 line 426 + §6 line 478.

**Forks header (v2)**: at v2 plan-draft time the ADR header reads `Forks user-locked at gate-1 to F1.B / F2.A / F3.B / F4.A / F5.B (F3 DIVERGENT from planner's F3.A recommendation; v2 re-plan applied)`. Sub-forks under F3.B all auto-resolved to defer-via-followup-drift / principal-blind-Repository / wire-submit-sites / silent-post-filter.

## §6 — Prior-chunk regression re-verification (v2 unchanged from v1)

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-04** (Action vocabulary) | `Action::CANONICAL.len() == 34` invariant | `cargo test -p domain action::tests::canonical_action_vocabulary_has_exactly_34_verbs` |
| **CH-08** (allocate/transfer cardinality) | `AllocateRefinement` typed variant exists; `Grant.allocate_refinement` field exists | `grep -n "allocate_refinement" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` |
| **CH-09 / CH-10 / CH-11** (Consent triad) | `Consent` struct + `ConsentState` enum + per-session gating exist | `cargo test -p domain consents::tests` |
| **CH-12** (Frozen tag immutability) | `validate_tag_write_on_session` + `FrozenTagViolation` typed-violation pattern; Repository docstring contract | `grep -n "FrozenTagViolation\|frozen_tag_write_rejected" /root/projects/phi/baby-phi/modules/crates/domain/src/` |
| **CH-14** (system:genesis axiom) | `is_bootstrap_ar` two-witness predicate exists; `system_genesis_principal` constant | `grep -n "is_bootstrap_ar\|system_genesis_principal" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/axioms.rs` |
| **CH-15** (Permission-check gate at session launch) | `session_launch_denied` Alerted-class precedent; `LaunchReceipt.permission_check_decision` field; `Decision::Allowed/Denied/Pending` enum unchanged | `cargo test -p domain audit::events::m5_2::session_launch::tests` |
| **CH-17** (Live SSE tail endpoint) | broadcast channel orthogonal to CH-18 — regression-check is "tests still green" | `cargo test -p server sse_live_stream_test` |
| **All M1+ baseline** | `cargo test --workspace -j 4` returns 1491 / 0 / 2 ignored at chunk-open | `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace 2>&1 \| tail -3` |

This table runs at chunk-open (before P0) and again at chunk seal (P4).

## §7 — Phases within the chunk (v2 re-phased per F3.B scope expansion)

**5 content phases at v2** (vs v1's 4) — F3.B's scope expansion justifies splitting v1's monolithic P2 into v2's P2a (4 mutation handlers — F3.A baseline scope) + P2b (Repository docstring + 5 read-side handlers + 8 submit-side handlers — F3.B addition). Audit envelope shifts from "small (1 auditor)" at v1 to **"medium (2 auditors)"** at v2 per audit-envelope-size skill heuristic (5 content phases = 2 auditors).

### P0 — Scaffold + ADR draft + drift files + verified-header bumps

- **Goal**: scaffold; `cargo test -j 4 --no-run` green; ADR-0056 scaffold drafted as `Proposed`; `D-CH18-FOLLOWUP-01` filed at `discovered`.
- **Deliverables**:
  1. New file `domain/src/auth_requests/access.rs` (empty module + `//!` docstring).
  2. New file `domain/src/audit/events/m5_2/auth_request_access.rs` (empty module + `//!` docstring).
  3. Existing file `domain/src/auth_requests/mod.rs` — add `pub mod access;` (placeholder; populate exports in P1).
  4. Existing file `domain/src/audit/events/m5_2/mod.rs` — add `pub mod auth_request_access;`.
  5. NEW file `docs/specs/v0/implementation/m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md` — full ADR scaffold per `m5_2/decisions/0054-...` template; status `Proposed`; populated with §D56.1–§D56.10 sub-decisions; Forks header capturing gate-1 user-lock outcome (F1.B / F2.A / F3.B / F4.A / F5.B + sub-fork resolutions under F3.B).
  6. NEW file `docs/specs/v0/implementation/m5_1/drifts/D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md` — drift filed at `discovered` per §4.
  7. Existing file `docs/specs/plan/build/_cycle-index.md` — append CH-18 row.
  8. Verified-header bumps on every file modified.
- **Tests**: `cargo test -j 4 --workspace --no-run` (compile only).
- **Concept-alignment check**: §2 row 1 status `silent-in-code` → `silent-in-code` (no body content yet).
- **phi-core leverage check**: §3 row 1 status unchanged.
- **User-facing doc updates**: none — P3 ships user-facing tier.
- **Confidence target**: 100% — pure scaffold.
- **Pause discipline**: pause if either NEW module fails to compile.

### P1 — Implement `check_auth_request_access` + per-state matrix table + `AuthRequestAccessError` typed enum + F1.B PartialEq derive

(Same as v1 P1 — substrate is unchanged; F3.B expansion happens at the WIRING phase, not the substrate.)

- **Goal**: P1 ships the substrate. Function captures concept doc 02 lines 130–144 verbatim. Tests cover every (state × principal-class × intended_op) cell-class.
- **Deliverables**:
  1. `domain/src/auth_requests/access.rs` body:
     - `pub enum IntendedOp { Read, Modify, Submit, Cancel, Approve, Deny, Reconsider, Revoke, OverrideApprove, CloseAsDenied, Expire }` (11 variants).
     - `pub enum AuthRequestAccessError { ... }` (5 variants per §D56.3).
     - `pub fn check_auth_request_access(ar: &AuthRequest, principal: &PrincipalRef, intended_op: IntendedOp) -> Result<(), AuthRequestAccessError>` — pure function.
     - `fn classify_principal(p: &PrincipalRef, ar: &AuthRequest) -> PrincipalClass` (matches `ar.requestor` via `==` once F1.B lands; checks slot ownership via `==` against `ar.resource_slots[*].approver_slots[*].approver`; checks `is_bootstrap_ar(ar)` for system-genesis fast path).
  2. `domain/src/auth_requests/mod.rs` — replace placeholder with `pub mod access; pub use access::{check_auth_request_access, AuthRequestAccessError, IntendedOp};`.
  3. F1.B trait-derive: `domain/src/model/nodes.rs:797` — `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` on `pub enum PrincipalRef`. Add `#[test] fn principal_ref_partial_eq_round_trips` at end of `nodes.rs::tests`.
  4. New `#[cfg(test)] mod tests` block at end of `auth_requests/access.rs`.

- **Tests**:
  - **MUST-SHIP** (P1): `auth_requests::access::tests::*` — ~15 unit tests covering distinct matrix cell-classes (per §8 enumeration).
  - **MUST-SHIP** (P1): `model::nodes::tests::principal_ref_partial_eq_round_trips` — 1 test, 5 sub-asserts.
- **Concept-alignment check**: §2 row 1 `silent-in-code` → `honored` (matrix captured + tested).
- **phi-core leverage check**: unchanged.
- **User-facing doc updates**: none — deferred to P3.
- **Confidence target**: ≥ 97%.
- **Pause discipline**: pause if any matrix-cell test fails; pause if `Action::CANONICAL.len() == 34` regresses.

### P2a — Wire 4 mutation handlers + emit Alerted audit-event on Err (v1 / F3.A baseline scope)

- **Goal**: F3.A's 4 mutation callsites + F5.B audit emission live.
- **Deliverables**:
  1. `domain/src/audit/events/m5_2/auth_request_access.rs` body — `pub fn auth_request_access_denied(actor: AgentId, target_ar: AuthRequestId, org: OrgId, error: &AuthRequestAccessError, attempted_op: IntendedOp, attempted_at: DateTime<Utc>) -> AuditEvent` builder. Mirrors `frozen_tag_write_rejected` shape verbatim. 3 unit tests in same file.
  2. `server/src/platform/templates/approve.rs:78` area — insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Approve)` upstream; on Err → audit-event emit + return `Err(TemplateError::AccessDenied(e))`.
  3. Same pattern at `templates/deny.rs:83` (IntendedOp::Deny).
  4. Same pattern at `templates/revoke.rs:82` (IntendedOp::Revoke). The cascade revoke loop at line 115/131 is documented as kernel-skip per §D56.8.
  5. Same pattern at `projects/create.rs:656` (IntendedOp::Approve or Deny based on input.approve).
  6. NEW variant `AccessDenied(AuthRequestAccessError)` on `TemplateError` + `ProjectError` (additive enum; cascade 0 callsite edits via the catch-all `_.to_string()` pattern verified at `grep -nE 'match.*TemplateError\b' modules/crates/server/src/`).

- **Tests**:
  - **MUST-SHIP**: 3 unit tests for `auth_request_access_denied` builder.
  - **MUST-SHIP**: 4 new integration tests: `template_approve_access_test.rs` / `template_deny_access_test.rs` / `template_revoke_access_test.rs` / `project_creation_access_test.rs`.
- **Concept-alignment check**: §2 row 3 `partially-honored` → `partially-honored (4 mutation callsites improved)`.
- **phi-core leverage check**: unchanged.
- **User-facing doc updates**: none.
- **Confidence target**: ≥ 97%.
- **Pause discipline**: pause if any of the 4 handler integration tests regress; pause if test count drops below baseline 1491.

### P2b — Wire Repository docstring + 5 read-side handlers + 8 submit-side handlers (F3.B addition)

- **Goal**: F3.B locked path's expansion — 13 additional production callsites wired; Repository trait docstring updated; F3.B sub-fork resolutions implemented.
- **Deliverables**:
  1. **Repository trait docstring update**: `domain/src/repository.rs:792-796` — append the §D56.7 contract sentence to docstrings on `get_auth_request` (line 793) + `update_auth_request` (line 796) per F3.B.repo-shape.b.
  2. **Read-side wiring (5 sites)**:
     - `server/src/platform/projects/create.rs:636` — upstream of `repo.get_auth_request(input.ar_id)`, after the AR is fetched, insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.approver_id), IntendedOp::Read)?;` returning `Err(ProjectError::AccessDenied(e))`. Note: read happens BEFORE the existing line-643 `is_terminal(ar.state)` check; if access fails the handler returns access-denied without divulging the AR's state to the caller.
     - `server/src/platform/orgs/dashboard.rs:273` — replace `let active_ars = repo.list_active_auth_requests_for_org(org_id).await?;` with the same call followed by a silent post-filter retaining entries where `check_auth_request_access(&ar, &PrincipalRef::Agent(viewer_agent_id), IntendedOp::Read).is_ok()` per F3.B.list-filter.a.
     - `server/src/platform/orgs/dashboard.rs:293` — same pattern for `list_adoption_auth_requests_for_org`.
     - `server/src/platform/orgs/show.rs:42-72` — modify `show_organization` signature: ADD `viewer: AgentId` parameter (NEW arg). Update the line-63 `list_adoption_auth_requests_for_org` call to apply the same silent post-filter. Cascade: every caller of `show_organization` (handler at HTTP route level) needs to pass `viewer`. Cascade scope: verify via `git grep -n 'show_organization' modules/crates/server/`.
  3. **Submit-side wiring (8 sites)** per F3.B.create-side.a:
     - `server/src/platform/templates/adopt.rs:96` — insert `check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Submit)?;` BEFORE the `repo.create_auth_request(&ar)` call.
     - `server/src/platform/projects/create.rs:470` — same.
     - `server/src/platform/defaults/put.rs:88` — same.
     - `server/src/platform/secrets/add.rs:107` — same.
     - `server/src/platform/mcp_servers/register.rs:92` — same.
     - `server/src/platform/mcp_servers/patch_tenants.rs:98` — same.
     - `server/src/platform/mcp_servers/archive.rs:54` — same.
     - `server/src/platform/model_providers/register.rs:124` — same.
     - `server/src/platform/model_providers/archive.rs:60` — same.
  4. **Cascade discipline (chunk-planner v3)** — `show_organization` signature change cascades to every HTTP-route caller. Pre-flight grep:
     ```bash
     git -C /root/projects/phi/baby-phi grep -n "show_organization" modules/crates/server/
     ```
     Expected raw count: ~2-4 sites (HTTP route handler + integration tests). Per-file breakdown captured at P2b implementation time. **Pause if actual sites > 6** (1.5× predicted).
- **Tests**:
  - **MUST-SHIP**: 1 new integration test per submit handler asserting denied-path (`Submit` op fails when `ar.requestor != input.actor` — definitionally a unit-level test; the integration test asserts the structurally-redundant happy-path doesn't regress). 8 tests.
  - **MUST-SHIP**: 1 new integration test asserting dashboard's silent post-filter behaviour: build a fixture with viewer = non-requestor / non-slot-approver → assert AR is filtered out. 1 test.
  - **MUST-SHIP**: 1 new integration test asserting `show_organization` count uses post-filter (build fixture as above; assert `adopted_template_count` reflects the filter). 1 test.
  - **MUST-SHIP**: 1 new integration test asserting `projects/create.rs:636` slot-fill read denies non-slot-approver-non-requestor with `ProjectError::AccessDenied`. 1 test.
  - **MAY-COVER**: 1 new property test `domain/tests/auth_request_access_props.rs` (NEW file per F4.A) covering matrix invariants — 1 test (proptest generator + ≤5 assertions).
- **Concept-alignment check**: §2 row 1 `silent-in-code` → `honored` for the 17 production callsites; the 7 kernel-skip sites are documented in §D56.8.
- **phi-core leverage check**: unchanged.
- **User-facing doc updates**: none — P3 ships.
- **Confidence target**: ≥ 95% (broader scope; one or two test fixtures may need adjustment).
- **Pause discipline**:
  - Pause if `show_organization` cascade > 6 sites (per chunk-planner v3 1.5× predicted).
  - **Pause if any of the existing dashboard happy-path integration tests regress** (per §3 Artifact-C cascade discipline; predicted impact ≤ 4 tests; if > 4, escalate to user via AskUserQuestion).
  - Pause if F3.B.list-filter.a's silent post-filter unexpectedly hides ARs from the org's CEO viewer in a way that breaks dashboard rendering tests (this is the chunk's highest regression risk per F3.B's own warning at gate-1).
  - Pause if test count drops below baseline 1491.
  - Pause if any submit-side wiring fails compilation (the 8 submit handlers each have slightly different `input` shapes; verify each passes the right `actor` field to the access check).

### P3 — User-facing doc tier + audit-event dictionary registration + drift-row flips

- **Goal**: every §3.C row honoured; Repository trait docstring documented in the new architecture page; audit-event dictionary registers the new event-type; drift rows flipped.
- **Deliverables**:
  1. NEW `docs/specs/v0/implementation/m5_2/architecture/auth-request-access-acl.md` — sections: §1 Why this exists, §2 Function shape, §3 Per-state matrix table, §4 17-callsite wiring map (full table per §3 above), §5 audit-event shape, §6 7-callsite kernel-internal skip-list (per §D56.8), §7 forward-defensive descope of admin/auditor role-discrimination (per `D-CH18-FOLLOWUP-01`), §8 K8s posture (neutral; F3.B.repo-shape.b). Verified-header line 1.
  2. NEW `docs/specs/v0/implementation/m5_2/operations/auth-request-access-acl-operations.md` — sections: §1 audit-event dictionary entry, §2 expected emission rate, §3 troubleshooting playbook, §4 typed-error reference, §5 list-side silent-filter operator-narrative (operator sees fewer ARs than DB rows for non-admin viewers; this is intentional). Verified-header line 1.
  3. AMEND `docs/specs/v0/implementation/m1/architecture/audit-events.md` — append the `auth_request.access_denied` event-type to dictionary table.
  4. AMEND `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` — add "CH-18 amendment — auth_request.access_denied troubleshooting (2026-05-09)" subsection.
  5. AMEND `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` — add row for `permissions/02 §"Per-State Access Matrix"` flipping `silent-in-code` → `honored` (letter-for-letter from §2 target column).
  6. AMEND `docs/specs/v0/implementation/m5_1/drifts/D-new-12.md` — Status: `discovered` → `remediated`; lifecycle history append.
  7. `D-CH18-FOLLOWUP-01.md` stays at `discovered` — chunk-close review + lifecycle history confirms it's M6+-deferred per F3.B.role.b.

- **Tests**: `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` green; `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` green; `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` green.
- **Concept-alignment check**: §2 row 1 `silent-in-code` → `honored`.
- **phi-core leverage check**: unchanged.
- **User-facing doc updates**: every §3.C row delivered.
- **Confidence target**: ≥ 99%.
- **Pause discipline**: pause if `check-doc-links.sh` fails.

### P4 — Chunk seal

- **Goal**: P4 paperwork green; cycle-index row Status `in-flight` → `ready-for-audit`; ADR-0056 flips `Proposed → Accepted`; `D-new-12` lifecycle entry → `remediated`; iteration count bumped 0 → 1.
- **Deliverables**:
  1. ADR-0056 status flip `Proposed → Accepted`.
  2. `D-new-12.md` Status flip + lifecycle.
  3. `D-CH18-FOLLOWUP-01.md` lifecycle confirmation entry (status stays `discovered`; M6+ deferral confirmed).
  4. `_concept-audit-matrix.md` row letter-for-letter flip.
  5. `_cycle-index.md` Status flip + Auditors set to "**2 (audit envelope: medium per v2 re-plan)**" — per chunk-planner v9 §11 re-evaluation.
  6. Final `cargo test -j 4 --workspace` pass; `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` pass; `cargo fmt --all -- --check` pass; 4 CI guards green.
  7. P4 chunk-seal paperwork checklist (CH-11 + CH-12 + v9): every modified verified-header description matches body diff exactly; `_concept-audit-matrix.md` row Status copy-pasted letter-for-letter from §2 target column.

- **Tests**: full workspace at chunk close — predicted ~1491 + ~30 new = **~1521** (v2 §8 band).
- **Concept-alignment check**: every §2 row's target-status achieved.
- **phi-core leverage check**: import baseline 57 unchanged.
- **User-facing doc updates**: full §3.C map satisfied (4 rows: 2 NEW + 2 AMEND).
- **Confidence target**: ≥ 99% composite.
- **Pause discipline**: pause if MUST-RUN clippy reports any warning; pause if test count drops below baseline 1491.

## §8 — Tests summary (v2 re-derived for F3.B)

**Test-count prediction band (v2 — asymmetric ×1.0–×1.20 per chunk-template §8 chunk-planner v3 / CH-12 retro)**:

### MUST-SHIP (binding contract)

P1 — `domain/src/auth_requests/access.rs::tests`, ~15 tests covering distinct matrix cell-classes (unchanged from v1):
  - Draft × Read by requestor → Ok
  - Draft × Read by non-requestor → Err
  - Draft × Modify by requestor → Ok
  - Draft × Modify by non-requestor → Err
  - Draft × Submit by requestor → Ok
  - Pending × Approve by unfilled-slot approver → Ok
  - Pending × Approve by filled-slot approver → Err
  - Pending × Approve by non-slot agent → Err
  - InProgress × Reconsider by filled-slot approver (own slot) → Ok
  - InProgress × Reconsider by filled-slot approver (other slot) → Err
  - Approved × Modify by anyone → Err
  - Approved × Revoke by `ar.requestor == principal` adoption-AR self-revoke → Ok (descoped owner-class stub per `D-CH18-FOLLOWUP-01` cross-ref)
  - Closed-terminals (Denied / Cancelled / Revoked / Expired) × Read by any → Ok
  - Closed-terminals × Modify by any → Err
  - System-genesis bootstrap AR × Read by any → Ok (system-genesis fast-path)

P1 — `domain/src/model/nodes.rs::tests::principal_ref_partial_eq_round_trips` — 1 test, 5 sub-asserts.

P2a — `domain/src/audit/events/m5_2/auth_request_access.rs::tests` — 3 tests (mirror frozen_tag_write_rejected_*).

P2a — `server/tests/template_*` — 4 new integration tests (1 per mutation handler).

P2b (NEW at v2 / F3.B addition):
  - `server/tests/template_adopt_submit_access_test.rs` — 1 test (Submit by requestor → 200 + create_auth_request invoked).
  - `server/tests/project_create_submit_access_test.rs` — 1 test (Shape B project Submit → 200).
  - `server/tests/defaults_put_submit_access_test.rs` — 1 test.
  - `server/tests/secrets_add_submit_access_test.rs` — 1 test.
  - `server/tests/mcp_register_submit_access_test.rs` — 1 test.
  - `server/tests/mcp_patch_tenants_submit_access_test.rs` — 1 test.
  - `server/tests/mcp_archive_submit_access_test.rs` — 1 test.
  - `server/tests/model_provider_register_submit_access_test.rs` — 1 test.
  - `server/tests/model_provider_archive_submit_access_test.rs` — 1 test.

  **Submit-side total**: 9 tests (one per submit handler — defence-in-depth happy-path coverage).

P2b — `server/tests/dashboard_silent_filter_test.rs` (NEW) — 1 test asserting silent post-filter at the dashboard.
P2b — `server/tests/show_organization_post_filter_test.rs` (NEW) — 1 test asserting `show_organization` count reflects the filter.
P2b — `server/tests/project_create_slot_fill_read_access_test.rs` (NEW) — 1 test asserting `projects/create.rs:636` slot-fill read denies non-slot-approver-non-requestor.

**MUST-SHIP total at v2**: 15 (matrix) + 1 (PartialEq) + 3 (audit-event builder) + 4 (mutation integration) + 9 (submit-side) + 1 (dashboard filter) + 1 (show_organization filter) + 1 (slot-fill read) = **35 tests**.

### MAY-COVER (band-floor surrogate)

- 1 NEW property test in `domain/tests/auth_request_access_props.rs` (NEW file per F4.A) — proptest generator covering matrix invariants. 1 test.
- Edge-case enumeration tests at `auth_requests::access::tests` covering specific principal × state pairings beyond the 15 distinct cell-classes — 0–3 tests.

**MAY-COVER total**: 1–4 tests.

**Combined band v2**: 35–39 new tests (vs v1's 23–29).

### Layer breakdown (v2)

- Unit tests in `domain/src/auth_requests/access.rs::tests`: 15
- Unit tests in `domain/src/audit/events/m5_2/auth_request_access.rs::tests`: 3
- Unit tests in `domain/src/model/nodes.rs::tests`: 1
- Property tests in `domain/tests/auth_request_access_props.rs` (NEW per F4.A): 1
- Integration tests in `server/tests/`: 15 (4 mutation + 9 submit + 2 list-filter + 1 slot-fill read)
- Acceptance tests: 0
- E2E tests: 0

**Predicted total at chunk close (v2)**: 1491 baseline + 35 (MUST-SHIP) + 0 to 4 (MAY-COVER) = **[1526, 1530]**.

**Asymmetric accept band** (×1.0 lower / ×1.20 upper per chunk-template §8): 1491 + ⌊35 × 1.0⌋ to 1491 + ⌈39 × 1.20⌉ = **[1526, 1538]**.

**Outside accept band**: AskUserQuestion. CH-17 retro Row 5 widens to ×1.30 on Artifact-C-cascade chunks; v2 has Artifact-C cascade impact (dashboard test fixtures may need adjustment per §3 Artifact-C note) — apply **×1.30 ceiling**: upper bound = 1491 + ⌈39 × 1.30⌉ = **[1526, 1542]**.

### Named test files

**NEW files** (MUST-SHIP at v2):
- `modules/crates/server/tests/template_approve_access_test.rs`
- `modules/crates/server/tests/template_deny_access_test.rs`
- `modules/crates/server/tests/template_revoke_access_test.rs`
- `modules/crates/server/tests/project_creation_access_test.rs` (mutation slot-fill)
- `modules/crates/server/tests/project_create_slot_fill_read_access_test.rs` (read-side)
- `modules/crates/server/tests/template_adopt_submit_access_test.rs`
- `modules/crates/server/tests/project_create_submit_access_test.rs`
- `modules/crates/server/tests/defaults_put_submit_access_test.rs`
- `modules/crates/server/tests/secrets_add_submit_access_test.rs`
- `modules/crates/server/tests/mcp_register_submit_access_test.rs`
- `modules/crates/server/tests/mcp_patch_tenants_submit_access_test.rs`
- `modules/crates/server/tests/mcp_archive_submit_access_test.rs`
- `modules/crates/server/tests/model_provider_register_submit_access_test.rs`
- `modules/crates/server/tests/model_provider_archive_submit_access_test.rs`
- `modules/crates/server/tests/dashboard_silent_filter_test.rs`
- `modules/crates/server/tests/show_organization_post_filter_test.rs`
- `modules/crates/domain/tests/auth_request_access_props.rs` (MAY-COVER, F4.A)

**Existing files extended** (NEW tests appended):
- `modules/crates/domain/src/auth_requests/access.rs` (NEW file with mod tests block)
- `modules/crates/domain/src/audit/events/m5_2/auth_request_access.rs` (NEW file with mod tests block)
- `modules/crates/domain/src/model/nodes.rs` (1 test added at end of existing tests mod)

### Named expected-still-green tests

Upstream tests CH-18 v2 must NOT break:
- `domain::permissions::action::tests::*` — Action::CANONICAL invariant (CH-04).
- `domain::auth_requests::transitions::tests::*` — 13 existing tests (CH-08 baseline).
- `domain::auth_requests::revocation::tests::*` — 4 existing tests (CH-14 baseline).
- `domain::audit::events::m5_2::tool_authority::tests::*` — 4 existing tests on `frozen_tag_write_rejected` (CH-12 baseline).
- `domain::audit::events::m5_2::session_launch::tests::*` — existing tests (CH-15 baseline).
- All workspace integration tests under `server/tests/dashboard_*` — happy-path tests must continue to pass (Artifact-C risk per §3; potential 0–4 fixture amendments).
- All workspace integration tests under `server/tests/template_*` (existing happy-path approve/deny/revoke + adopt).

## §9 — Pre-chunk gate

**Reading list (v2 expanded — adds Repository trait + dashboard handler)**:

1. ✅ `docs/specs/v0/concepts/permissions/02-auth-request.md` — §"Per-State Access Matrix" lines 130–144 + §"State Machine" lines 70–129 + §"Multi-Approver Dynamics" lines 175–179.
2. ✅ `docs/specs/v0/concepts/permissions/03-action-vocabulary.md` — line 7–22 (closed 34-verb table) + line 44.
3. ✅ `docs/specs/v0/implementation/m5_1/drifts/D-new-12.md` — full file.
4. ✅ `docs/specs/plan/build/ch-15-real-permission-check-gate-at-session-launch-c3f46f17/plan.md` — for permission-check style precedent + Alerted-class deny event shape.
5. ✅ `docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md` — for forward-defensive ship + typed-violation enum + Repository docstring contract precedent.
6. ✅ `docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md` — for `is_bootstrap_ar` 2-witness predicate + pre-existing-behaviour preservation note discipline.
7. ✅ `docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §5 row 18 + §6 line 426 + §7 Q5 line 478.
8. ✅ `baby-phi/CLAUDE.md` §"phi-core Leverage" + §"Multi-agent chunk pipeline" + §"Granular Bash discipline".
9. ✅ Source code (v2 expanded):
   - `domain/src/auth_requests/{mod,state,transitions,revocation,retention}.rs` (all 5 files)
   - `domain/src/audit/events/m5_2/{mod,session_launch,tool_authority,session_live_stream}.rs`
   - `domain/src/permissions/{action,axioms}.rs`
   - `domain/src/model/nodes.rs:780-906` (PrincipalRef + AR + state enum + slot)
   - **`domain/src/repository.rs:760-830,1290-1325`** (NEW at v2 reading list — AR Repository methods for §D56.7 docstring update)
   - `server/src/platform/templates/{approve,deny,revoke,mod,adopt}.rs` (v2 expanded — adds adopt.rs)
   - `server/src/platform/projects/{create,resolvers}.rs` (v2 expanded — adds resolvers.rs for kernel-skip documentation)
   - **`server/src/platform/orgs/{dashboard,show}.rs`** (NEW at v2 reading list — for F3.B read-side wiring)
   - **`server/src/platform/{defaults,secrets}/`** (NEW at v2 reading list — for F3.B.create-side.a wiring)
   - **`server/src/platform/{mcp_servers,model_providers}/`** (NEW at v2 reading list — for F3.B.create-side.a wiring)
   - **`server/src/bootstrap/claim.rs`** (NEW at v2 — to confirm system-genesis fast-path applies)
   - **`domain/src/events/listeners.rs:160-200`** (NEW at v2 — to confirm kernel-internal skip applies)
10. ✅ Conditional reading list (chunk-template §9, v2026-05-03 per CH-11 retrospective): N/A — CH-18 does NOT touch `domain::permissions::engine` Step N body.
11. ✅ Conditional reading list (chunk-template / CH-12 retro Row 5): N/A — CH-18 introduces no new tag-write Repository method.

**Carry-forward invariants** (verified green at chunk-open):

- ✅ `cargo test -j 4 --workspace` baseline = 1491 / 0 / 2 ignored (CH-17 close).
- ✅ `bash scripts/check-phi-core-reuse.sh` green.
- ✅ `bash scripts/check-doc-links.sh` green.
- ✅ `bash scripts/check-ops-doc-headers.sh` green.
- ✅ `bash scripts/check-spec-drift.sh` green.
- ✅ `modules/` diff against chunk-open git HEAD is empty.

**Pending decisions carried into v2 (post-gate-1)**:
- Q5 (line 478) MEDIUM-severity: close-at-M5 (planner recommended; gate-1 user-locked F3.B explicitly accepted broader scope).
- F1.B / F2.A / F3.B / F4.A / F5.B all locked at gate-1.
- F3.B sub-forks all auto-resolved at v2 (role.b / repo-shape.b / create-side.a / list-filter.a).

**Chunk-ordering note (Q4 decision)**: CH-18 is one of the 8 parallelizable foundation-tier chunks per forward-scope §4 line 367. All prereqs closed (CH-01 through CH-17 retro-complete).

## §10 — Close criteria (v2 expanded confidence math)

Composite 4-aspect + 2 confidence % ritual.

**4 aspects (each pass / fail)**:

- **Code aspect** — all P0/P1/P2a/P2b/P3/P4 deliverables shipped; `cargo test -j 4 --workspace` passes (predicted [1526, 1542] per v2 §8 ×1.30 ceiling); `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` passes 0 warnings; `cargo fmt --all -- --check` passes.
- **Docs aspect** — *Governance tier*: ADR-0056 status `Accepted`; D-new-12 Status `remediated`; `D-CH18-FOLLOWUP-01` Status `discovered` with M6+ deferral confirmed; `_concept-audit-matrix.md` row letter-for-letter flipped; verified-headers updated. *User-facing tier (§3.C)*: every row delivered (2 NEW + 2 AMEND).
- **phi-core leverage aspect** — `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l == 57`; `bash scripts/check-phi-core-reuse.sh` green; forbidden-duplication greps return 0.
- **Concept alignment aspect** — every §2 row's target-status achieved. §2 row 1 flips `silent-in-code` → `honored` with the documented `D-CH18-FOLLOWUP-01` deferral note for admin/auditor-class read-bypass.

**2 confidence %**:

- **Implementation confidence %** = `(claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk)`. Target ≥ **9 / 10** (10 sub-decisions §D56.1–§D56.10 + 17 wiring claims + 7 kernel-skip-documentation claims + 7 §2 rows + 1 K8s posture claim + 1 D-CH18-FOLLOWUP-01 filed-and-deferred claim = ~43 distinct claims; at 9/10 floor 39 honored is acceptable). The known-deferred admin/auditor read-bypass claim is filed as `D-CH18-FOLLOWUP-01` per chunk-planner v9 forward-scope-vs-concept-doc precedence discipline (this counts as 1 honored claim for "deferral documented" rather than an unhonored claim).
- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check) / (doc-pages-touched)`. Target ≥ **8/8 = 100%**. CH-18 v2 touches 4 doc files (§3.C) + ADR-0056 + drifts D-new-12 and D-CH18-FOLLOWUP-01 + concept-audit-matrix row + audit-events.md amendment + Repository trait docstring update = 9 doc-page-equivalents.

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** Target ≥ **9.5 / 10**.

**P4 chunk-seal paperwork checklist**:
- ✅ Every modified verified-header description matches body diff exactly.
- ✅ `_concept-audit-matrix.md` row Status copy-pasted letter-for-letter from §2 target column.
- ✅ `_cycle-index.md` row appended at chunk-open + Status flipped + iteration count bumped 0 → 1.
- ✅ ADR-0056 carries explicit Forks header capturing user-lock outcome (F3.B DIVERGENT note + sub-fork auto-resolutions).
- ✅ All 4 cross-references categories present in ADR-0056.

## §11 — Post-chunk independent audit plan (v2 — medium envelope)

**Audit envelope (v2)**: **medium** (2 auditors) — per audit-envelope-size skill heuristic the v1 plan was at the 1-auditor / 2-auditor boundary (4 content phases); v2 adds P2b for a total of 5 content phases, which lands squarely in 2-auditor territory. This is the F3.B-induced envelope shift.

**Audit aspects covered (a–d per chunk-template §11)**:
- (a) Code correctness — 17 wiring sites + 7 kernel-skip sites + Repository trait docstring + audit-event builder + matrix function
- (b) Docs fidelity vs concept docs — ADR-0056 + matrix table cross-check
- (c) Concept alignment — every §2 row at audit time
- (d) phi-core leverage — forbidden-duplication greps + canonical baseline (57)

### Audit Agent A — substrate + mutation-side (≤ 600 words)

> **Scope**: CH-18 v2 (cycle hex c77937bc) — substrate (P0+P1) + mutation-side wiring (P2a) + governance docs (ADR-0056 + drifts + audit-events.md amendment).
>
> **Files to audit** (Agent A's slice):
> - `domain/src/auth_requests/access.rs` (NEW — function body + matrix table + tests)
> - `domain/src/audit/events/m5_2/auth_request_access.rs` (NEW — Alerted-class builder + tests)
> - `domain/src/auth_requests/mod.rs` (modified — pub mod + pub use)
> - `domain/src/audit/events/m5_2/mod.rs` (modified — pub mod)
> - `domain/src/model/nodes.rs:797` (modified — F1.B PartialEq derive on PrincipalRef + 1 test)
> - `server/src/platform/templates/{approve,deny,revoke}.rs` (modified — `check_auth_request_access` upstream + audit-event emit on Err)
> - `server/src/platform/projects/create.rs:656` (modified — slot-fill mutation wiring)
> - 4 NEW integration test files in `server/tests/template_*` + 1 in `project_creation_access_test.rs`
> - ADR-0056 + drift D-new-12 + drift D-CH18-FOLLOWUP-01 (filed) + `_concept-audit-matrix.md` row.
> - AMEND `m1/architecture/audit-events.md` event-type registry row.
>
> **Specific claims to verify** (numbered 1–14):
> 1. `Action::CANONICAL.len() == 34` invariant test passes (concept-doc 03 closed-set unchanged).
> 2. `IntendedOp` enum has 11 variants matching matrix columns.
> 3. Matrix-cell map in `access.rs` matches concept doc 02 lines 130–144 verbatim — pick 5 random cells.
> 4. `AuthRequestAccessError` typed-error has 5 variants per §D56.3.
> 5. `PrincipalRef` derives `PartialEq, Eq` per F1.B; `principal_ref_partial_eq_round_trips` passes.
> 6. The 4 mutation production callsites (`templates/{approve,deny,revoke}.rs` + `projects/create.rs:656`) each gain a `check_auth_request_access` call upstream.
> 7. The 4 mutation production callsites emit `auth_request_access_denied` on Err per F5.B.
> 8. `auth_request_access_denied` builder mirrors `frozen_tag_write_rejected` shape; canonical_bytes excludes prev_event_hash.
> 9. ADR-0056 carries Forks header capturing F3.B DIVERGENT outcome + sub-fork resolutions; 4-category Cross-references present; Pre-existing-behaviour preservation note for §D56.6.
> 10. `D-CH18-FOLLOWUP-01` filed + cross-referenced from ADR-0056 §D56.5 + §D56.9.
> 11. `_concept-audit-matrix.md` row Status flipped letter-for-letter.
> 12. `bash scripts/check-phi-core-reuse.sh` green; phi-core baseline 57 unchanged.
> 13. `auth_request.access_denied` registered in `m1/architecture/audit-events.md` dictionary.
> 14. K8s posture claim (§3.B) verified: A1–A7 all `no impact` under F3.B.repo-shape.b.
>
> **Pass criteria**: every claim PASS or NOT-EXECUTED-IN-AUDIT.
>
> **Format**: `audit-A-iter<N>.md` in cycle folder.
>
> **Audit MUST NOT be performed by the same agent as the implementer.**

### Audit Agent B — F3.B expansion-side (read + submit + list-filter + show + Repository docstring) (≤ 600 words)

> **Scope**: CH-18 v2 (cycle hex c77937bc) — F3.B expansion (P2b) + user-facing docs (P3 docs that span F3.B-specific surfaces).
>
> **Files to audit** (Agent B's slice):
> - `domain/src/repository.rs:792-797` (modified — docstring update on `get_auth_request` + `update_auth_request` per §D56.7)
> - `server/src/platform/orgs/dashboard.rs:273,293` (modified — silent post-filter per F3.B.list-filter.a)
> - `server/src/platform/orgs/show.rs:42-72` (modified — `viewer: AgentId` parameter + post-filter)
> - `server/src/platform/projects/create.rs:636` (modified — slot-fill read-side wiring per F3.B)
> - 8 submit-side wiring sites at `templates/adopt.rs:96` + `projects/create.rs:470` + `defaults/put.rs:88` + `secrets/add.rs:107` + `mcp_servers/{register,patch_tenants,archive}.rs` + `model_providers/{register,archive}.rs`
> - 9 NEW submit-side integration test files + 2 NEW list-filter test files + 1 slot-fill read test file + 1 NEW property test file
> - 2 NEW user-facing doc files (`m5_2/architecture/auth-request-access-acl.md` + `m5_2/operations/auth-request-access-acl-operations.md`)
> - 1 AMEND'd file (`m5/user-guide/troubleshooting.md`)
>
> **Specific claims to verify** (numbered 1–14):
> 1. Repository trait docstring on `get_auth_request` + `update_auth_request` carries §D56.7 future-callsite-contract sentence.
> 2. The 5 read-side wiring sites (`projects/create.rs:636`, `dashboard.rs:273`, `dashboard.rs:293`, `show.rs:63`, `show.rs:42` signature change) each call `check_auth_request_access` per F3.B.list-filter.a.
> 3. The 8 submit-side wiring sites each call `check_auth_request_access(..., IntendedOp::Submit)` BEFORE `repo.create_auth_request(...)`.
> 4. `show_organization` cascade — every caller passes the new `viewer: AgentId` parameter; `git grep -n show_organization modules/crates/server/` returns ≤ 6 sites; every call updated.
> 5. Dashboard silent post-filter behaviour — `dashboard_silent_filter_test.rs` exercises the filter with non-requestor viewer.
> 6. `show_organization` post-filter behaviour — `show_organization_post_filter_test.rs` asserts count reflects filter.
> 7. Slot-fill read denies non-slot-approver-non-requestor — `project_create_slot_fill_read_access_test.rs`.
> 8. None of the existing dashboard happy-path integration tests regress (or, if regressions, fixture amendments are documented and the test count stays in band [1526, 1542]).
> 9. The 7 kernel-internal skip-list sites in §D56.8 are documented (verify by reading `m5_2/architecture/auth-request-access-acl.md` §6).
> 10. NEW architecture page `auth-request-access-acl.md` covers all 8 sections per §3.C row 1.
> 11. NEW operations page covers all 5 sections per §3.C row 4.
> 12. AMEND `m5/user-guide/troubleshooting.md` carries CH-18 amendment subsection.
> 13. F3.B.list-filter.a's silent post-filter is documented in operator narrative (operator sees fewer ARs than DB rows for non-admin viewers).
> 14. Property test in `domain/tests/auth_request_access_props.rs` (NEW per F4.A) — ≥ 1 proptest covering matrix invariants.
>
> **Pass criteria**: every claim PASS or NOT-EXECUTED-IN-AUDIT.
>
> **Format**: `audit-B-iter<N>.md` in cycle folder.
>
> **Audit MUST NOT be performed by the same agent as the implementer or Agent A.**

**Audit pass criteria**: zero FAILs across the 28 combined claims (14+14); any new drift discovered by audit gets its own drift file BEFORE chunk seal.

## §12 — Verification recipe (end-to-end, v2 expanded)

```bash
# Working directory: any (use absolute paths per CLAUDE.md granular-bash discipline).

# 1. CI guards (4 of them).
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (cap -j 4 per feedback_cargo_jobs_cap).
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace

# 3. CH-18 v2 specific — substrate exists.
grep -rn "check_auth_request_access\|AuthRequestAccessError" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# v2 / F3.B expectation: ≥ 23 (1 def + 1 enum + 17 production callsites + ≥ 4 mod-test sites).

grep -rn "auth_request_access_denied\|auth_request.access_denied" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# v2 expectation: ≥ 7 (1 builder + 4 mutation production callsites + ≥ 2 audit-event module tests).

# 4. CH-18 v2 specific — every Repository AR method now has the §D56.7 contract docstring.
grep -nE "check_auth_request_access\|ADR-0056" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs
# v2 expectation: ≥ 2 hits (1 on get_auth_request docstring + 1 on update_auth_request docstring).

# 5. CH-18 v2 specific — F3.B.create-side.a — 8 submit handlers wire the access check.
for f in templates/adopt.rs projects/create.rs defaults/put.rs secrets/add.rs mcp_servers/register.rs mcp_servers/patch_tenants.rs mcp_servers/archive.rs model_providers/register.rs model_providers/archive.rs; do
  grep -l "check_auth_request_access" /root/projects/phi/baby-phi/modules/crates/server/src/platform/$f
done
# v2 expectation: 9 file paths printed (one for each of the 9 submit sites; one is projects/create.rs already-printed via slot-fill).

# 6. CH-18 v2 specific — closed-set invariant preserved.
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 -p domain action::tests::canonical_action_vocabulary_has_exactly_34_verbs
# Expect: 1 test passing.

# 7. phi-core leverage baseline preserved.
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (chunk-open baseline; +0 delta).

# 8. F1.B trait-derive landed.
grep -nE "^#\[derive.*PartialEq.*\]\s*$|^#\[derive.*PartialEq.*Serialize" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs | grep -B1 "pub enum PrincipalRef"
# Expect: 1 hit on the line above PrincipalRef.

# 9. Drift-file status.
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count> + 1 (D-new-12 transitions).

grep -l "D-CH18-FOLLOWUP-01" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/
# Expect: 1 file (the NEW drift file at v2).

# 10. ADR-0056 status + Forks header.
grep -nE "^- Status:|^Status:|Forks user-locked" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0056-auth-request-per-state-acl-enforcement.md | head -5
# Expect: "Status: Accepted" at chunk close + Forks header capturing F3.B DIVERGENT outcome.

# 11. Cycle-index row exists with retro status flipped + audit envelope = medium.
grep -n "c77937bc" /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# Expect: ≥ 1 hit at chunk close; status `audited-pending-retro` after audit + `retro-complete` after retrospective; Auditors column = "2 (medium)".

# 12. NEW architecture page exists.
test -f /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/architecture/auth-request-access-acl.md && echo "OK"
test -f /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/operations/auth-request-access-acl-operations.md && echo "OK"
# Expect: 2 OK lines.

# 13. show_organization signature cascade closed (every caller passes viewer parameter).
grep -nE "show_organization\(" /root/projects/phi/baby-phi/modules/crates/server/ -r | wc -l
# Expect: ≥ 2 (HTTP route handler + tests); every call passes viewer: AgentId per F3.B P2b.
```

---

## v2 re-plan summary (gate-1 user-lock outcome banner)

**Re-plan trigger**: gate-1 user-lock divergence — user chose F3.B (full Repository + dashboard + resolver + bootstrap wiring) over planner's F3.A recommendation (mutation-handlers-only forward-defensive ship).

**Auto-approval criteria re-evaluated under F3.B**:

| Criterion | v1 / F3.A status | v2 / F3.B status | Note |
|---|---|---|---|
| **No locked forks (or all auto-resolvable)** | clean (planner-recommended F1.B + F3.A + F4.A; F2 + F5 user-locked) | **flagged** — F3 user-locked DIVERGENT from planner-recommended F3.A; sub-forks under F3.B all auto-resolvable | gate-1 explicit user-lock; v2 sub-forks (F3.B.role.b / F3.B.repo-shape.b / F3.B.create-side.a / F3.B.list-filter.a) all planner-auto-resolvable |
| **Scope ≤ 1.5× forward-scope target** | 6 files / ~2 days at v1 | **at boundary** — 17 wiring sites / 7 kernel-skip docs / ~3.0–3.5 days; 1.5× of 2 days = 3 days | orchestrator-discretion; user-lock at gate-1 explicitly accepted broader scope |
| **Zero phi-core leverage delta** | clean (+0) | **clean** (+0) — F3.B is wholly within baby-phi | unchanged |
| **No new K8s blocker class** | clean | **clean** — F3.B.repo-shape.b chosen (Repository stays principal-blind); A5 axis stays neutral; no `CHK8S-D-NN` filed | F3.B.repo-shape.a was the alternative that would have flipped A5; v2 explicitly does NOT take that path |
| **Audit envelope ≤ medium** | small (1 auditor, 4 phases) | **medium** (2 auditors, 5 phases) — boundary case | unchanged |
| **Confidence ≥ 9/10** | ~14/15 | **~39/43** ≈ 9.07/10 | meets the floor |
| **No new migration** | clean | **clean** | unchanged |

**Result**: under F3.B, **direct-approval criteria mostly hold** with one boundary case (scope ratio at 1.5× boundary). The orchestrator's gate-1 user-lock to F3.B explicitly accepted this. **No additional user-lock required at v2** — the F3.B sub-forks are all planner-auto-resolvable per the locked path's spirit.

**Re-plan banner**: this plan file (v2) is the implementer's binding spec. v1 (the F3.A draft) is preserved in this file's earlier history for audit-trail clarity (the v2 banner at the top makes the versioning explicit; gate-3 audit reviews the v2 plan, not v1).
