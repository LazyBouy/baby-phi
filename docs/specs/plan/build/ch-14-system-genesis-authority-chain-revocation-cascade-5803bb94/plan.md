<!-- Last verified: 2026-05-08 by Claude Code (CH-14 plan-approval gate: F1.A / F2.A / F3.A / F4.A / F5.A user-locked at orchestrator gate-1 escalation — all align with planner recommendation; migration 0011 (additive nullable column on auth_request) accepted as part of F3.A scope; chunk approved for P0 launch) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-14 plan draft — system:genesis axiom + authority-chain walker + full-tree revocation cascade per ADR-0053) -->

# CH-14 — `system:genesis` axiom + authority-chain walker + revocation cascade

| Field | Value |
|---|---|
| Cycle hex | `5803bb94` |
| Forward-scope row | [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 139–143 |
| Severity | ⚠HIGH |
| Estimated effort | ~4 engineer-days |
| Drifts closed | **D-new-14** (genesis + walker), **D-new-18** (full-tree revocation cascade) |
| ADR drafted | **ADR-0053** (Proposed at plan-draft; flips Accepted at chunk seal) |
| Test-count baseline | 1408 passed / 0 failed / 2 ignored — verified `cargo test --workspace -j 4` at HEAD `fe1a4be` (CH-08 audit fix) |
| phi-core import baseline | 48 `use phi_core::` lines under `modules/crates/` |

---

## Forks for orchestrator

The chunk has **5 distinct decision points**. The planner has run the §3 evidence sweep and recommends an `.A` path on each, but every fork has a defensible alternative. Per the orchestrator's gate-1 criteria, any user-locked divergence triggers re-spawn.

### F1 — `system:genesis` constant shape (where + how)

The string `"system:genesis"` already appears as a **typed `PrincipalRef::System(String)` literal at 11 sites across 5 files** (production: `bootstrap/claim.rs:192,203` + `nodes.rs:795` docstring + `allocate_refinement.rs:65` JSON sample + `nodes.rs:804` doc; tests: 6 in `repository_test.rs:626,637,1131,1141,1626,1637` + 1 in `template_e_props.rs:27`). The symbol is currently a magic-string with no compile-time check.

| Option | Shape | Pros | Cons |
|---|---|---|---|
| **F1.A** ⭐ recommended | `pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis";` + `pub fn system_genesis_principal() -> PrincipalRef { PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL.into()) }` in new `domain::permissions::axioms` module | Minimal blast radius (string still serializes the same way; no migration); single source of truth; backward-compatible with the 11 existing sites; matches CH-08 / CH-13 typed-constant precedent | Doesn't enforce "only the platform mints `PrincipalRef::System(\"system:genesis\")`" at the type level (a renegade caller could still construct it manually). |
| F1.B | New `PrincipalRef::SystemAxiom { axiom_name: SystemAxiom }` variant with `enum SystemAxiom { Genesis, Root, ... }` | Compile-time enforcement: `Genesis` is the only axiom-name in the type | Migration: changes wire format (`{"system_axiom": "genesis"}` vs `{"system": "system:genesis"}`); breaks every backend's `PrincipalRef` deserialization; 11 callsite cascade |
| F1.C | Add a private `pub(crate) const`, expose via opaque `fn` only | Stricter encapsulation than F1.A | Tests outside `domain` need a public path anyway (`store/tests`, `template_e_props.rs`) — encapsulation breaks down |

**Recommendation: F1.A** — minimal blast radius + zero migration + zero wire-format change + closes the magic-string drift via the const. F1.B is the "right" Rust shape long-term but cascades into a wire-format migration that is well outside CH-14's scope. The const + helper-fn shape gives us the type-level entry point without changing the variant.

### F2 — Walker return type

`walk_provenance_chain(grant) -> ?` — the concept doc shows the chain as `Grant → AR → (parent AR)* → bootstrap AR`, with each AR carrying its approver. Audit-trail consumers want to render the path as "who approved at each hop."

| Option | Return type | Pros | Cons |
|---|---|---|---|
| **F2.A** ⭐ recommended | `Vec<AuthRequest>` — ordered root-to-leaf (bootstrap AR first, grant's direct AR last) | Simplest type; matches the forward-scope row's literal signature `walk_provenance_chain(grant) -> Vec<AuthRequest>`; sufficient for audit + revocation; AR carries everything (approvers, slots, state) | Caller fetches `Grant` separately if needed; no per-hop Grant context |
| F2.B | `Vec<(AuthRequest, Option<Grant>)>` — pairs AR with the grant that issued it (None at bootstrap) | Richer audit context | More complex; doubles store-side queries for chains > 2 deep (we'd query `Grant WHERE descends_from = parent_ar.id` per hop); not in forward-scope literal |
| F2.C | `ProvenanceChain { bootstrap: AR, intermediates: Vec<AR>, leaf: AR }` typed wrapper | Pattern-matching at the chain root is explicit | Premature shape; today every chain walked from a Grant has at least 1 element (the direct AR); the bootstrap-discriminator is a property of the AR (`requestor == system_genesis_principal()` AND `provenance_template == Some(zero-uuid)`), not a structural slot |

**Recommendation: F2.A** — matches forward-scope literal signature; audit-fidelity is preserved (every AR has `requestor` + `resource_slots[].approvers`); revocation cascade only needs the AR-id chain anyway. If a future chunk wants per-hop grants, it adds a sibling method.

### F3 — Revocation cascade strategy

Concept-doc 04 §"Authority Chain" + concept-doc 08 §9.3 specify forward-only tree-wide revocation: when AR-X is revoked, every grant `descends_from = X` is revoked, AND every grant whose `descends_from` is itself an AR issued by an already-revoked grant. This is N-hop tree traversal, not single-hop.

The current `revoke_grants_by_descends_from(ar, at)` in `repo_impl_m2.rs:753` is **single-hop only** (one SQL `UPDATE grant SET revoked_at = $at WHERE descends_from = $ar`). Verified at `domain/src/in_memory.rs:1195-1209` (matches single-hop semantics). The Template-revoke handler at `server/src/platform/templates/revoke.rs:89` calls it once.

| Option | Algorithm | Pros | Cons |
|---|---|---|---|
| **F3.A** ⭐ recommended | **BFS in the repository layer** — `revoke_grants_by_descends_from_recursive(ar, at)` does iterative breadth-first sweep: level 0 = grants directly descending from `ar`; level N = grants descending from any AR that any level-(N-1) grant minted (typical chain depth ≤ 4: bootstrap AR → admin grant → org adoption AR → template AR → fired-grant AR → fired grant). Cap depth at 32 with explicit error on overflow (defensive guard against cycle bugs). | Single repository call; back-compat with existing single-hop signature retained as `revoke_grants_by_descends_from` (internally calls the recursive variant with depth 1 for back-compat); BFS gives deterministic ordering for audit; cycle-detection cap | Two SQL queries per level (find ARs minted by these grants; find grants minted by those ARs); for typical depths 1-4, that's 2-8 queries per Template-revoke — fine for M5 admin-write throughput |
| F3.B | Single SurrealDB recursive query (`->descends_from->grant->...`) | One round-trip | SurrealDB graph traversal syntax differs slightly from path-by-path; in-memory adapter would still need its own iterative impl; no back-compat with existing single-hop API for callers that explicitly want one hop (e.g., the M2 `narrow_mcp_tenants` cascade caller — `repo_impl_m2.rs:740`) |
| F3.C | DFS recursion in domain layer + per-grant `revoke_grant` calls | Reuses existing `revoke_grant` primitive | N round-trips for N grants; transaction-atomicity weakens (each revoke its own write); breaks M2 audit-event-emission expectations (caller emits per-AR events) |

**Recommendation: F3.A** — BFS in the repo layer, named `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` returning the **flat list of all revoked grants across all levels**. The existing `revoke_grants_by_descends_from(ar, at)` is preserved verbatim (single-hop) for back-compat with `narrow_mcp_tenants` (which already cascades via its own per-AR loop and explicitly wants single-hop semantics per AR). Template-revoke handler flips from `revoke_grants_by_descends_from` to `revoke_grants_by_descends_from_recursive`. **Critical sub-decision**: today `descends_from` only exists on `Grant`, not on `AuthRequest` — for BFS to walk Grant → AR → child-Grant → child-AR the planner verified that **adoption ARs are minted by Template grants from `templates/{a,b,c,d}.rs`**, and those grants currently set `descends_from = Some(adoption_ar.id)` (verified at `templates/d.rs:96`). So the chain shape is `Bootstrap-AR ← Bootstrap-Grant → Adoption-AR ← Adoption-Grant → Fired-Grant`. The BFS step from a revoked grant to its child-AR-id requires querying *which AR was minted with this grant as approver*. **There is currently no `descends_from` field on AuthRequest itself** — the link is structural-via-handler-flow, not a typed field. **CH-14 must add a typed `AuthRequest.descends_from_grant: Option<GrantId>`** (the grant under whose authority this AR was submitted) to make the chain walkable. See §3 for the cascade-prediction analysis.

### F4 — Bootstrap-init timing

Concept doc 02 line 469 says `fires_on: system_init`. Reality: the bootstrap AR + Grant + audit event are minted at **first claim** (`server/src/bootstrap/claim.rs:188-274`), not at `bootstrap-init` (which only generates the credential at `init.rs:34-47`).

| Option | Timing | Pros | Cons |
|---|---|---|---|
| **F4.A** ⭐ recommended | **Keep current claim-time minting; document the divergence in ADR-0053 §"Concept-doc divergence"** as an accepted simplification: the bootstrap AR is functionally axiomatic from claim-time onward; pre-claim there is no platform admin to render an audit, no migration to apply, no grants to descend. Concept's `fires_on: system_init` reads as "fires once, at the entrypoint that establishes the platform" — claim is that entrypoint. | Zero migration; zero new code path; preserves the existing test suite | Documentary divergence with concept-doc 02 line 469 (mitigated by ADR ratification) |
| F4.B | Add eager bootstrap-AR minting at `phi-server` startup (idempotent: skip if exists) | Strict concept fidelity: `system:genesis` exists from server boot | New startup-time path; new race-condition class (multi-replica K8s); extra migration if we want to seed at install-time |
| F4.C | Mint at `bootstrap-init` time alongside the credential | The `--bootstrap-init` flow becomes the system-init point | Inverts the current "credential-first, claim-later" two-step; complicates the rollback path if claim never happens (orphan bootstrap AR) |

**Recommendation: F4.A** — claim-time minting is functionally equivalent to system-init for the purpose of the authority tree. ADR-0053 documents the divergence with explicit rationale. The walker test asserts that *every grant chains to bootstrap* — this passes the moment claim succeeds. F4.B aggravates K8s-axis A4 (multi-pod startup race for first-write) — out of scope for CH-14.

### F5 — Genesis-detection predicate (where the walker terminates)

The walker needs to recognize "this AR is the bootstrap" to terminate cleanly. Three predicates available:

| Option | Predicate | Pros | Cons |
|---|---|---|---|
| **F5.A** ⭐ recommended | `is_bootstrap_ar(ar) := ar.requestor == system_genesis_principal() && ar.provenance_template == Some(TemplateId::from_uuid(uuid::Uuid::nil()))` — both conditions; defense in depth | Cannot mistake a regular `system:genesis`-requested AR (none exist outside bootstrap); cannot mistake a regular Template AR (none use the all-zero UUID); two independent witnesses both match only at the bootstrap | Tiny verbosity |
| F5.B | Match on `requestor` only | Simpler | A future code path that mints a non-bootstrap AR with `system:genesis` requestor (e.g., for auditor-bot system-internal ops) would alias the bootstrap |
| F5.C | Walker terminates on a depth-cap (e.g., 32) regardless of predicate | No semantic test | Loses the "auditable termination at the axiom" property concept-doc 02 line 480 specifies |

**Recommendation: F5.A** — codifies the "bootstrap is uniquely identified by requestor=system:genesis AND provenance_template=zero-uuid" invariant from claim.rs:192+218 in a single predicate that the walker, the genesis-test fixture, and the audit-trail renderer all share.

---

**Auto-approval criteria evaluation (per orchestrator gate-1 rubric):**

| Criterion | Status | Evidence |
|---|---|---|
| No locked forks | ❌ **5 forks above** — F1, F2, F3, F4, F5 all need user-lock | F3 in particular is architecturally load-bearing (BFS algorithm + new `AuthRequest.descends_from_grant` field) |
| Scope ≤ 1.5× forward-scope (~4 days → ≤ 6 days) | ⚠ **borderline** | New `AuthRequest.descends_from_grant` field cascades through ~30 fixture sites + AR builders + serde back-compat (planner-side is field-add with `#[serde(default)]`, but each callsite must pass `None` — see §3 cascade analysis). Audit-event emission per-grant in cascade adds ~1 day. Estimate: 4.5–5 days. |
| Zero phi-core leverage delta | ✅ | Permissions/governance — orthogonal to phi-core (consistent with every prior permissions chunk) |
| No new K8s blocker class | ✅ | All 7 axes resolve to `no impact` or `compatible` (see §3.B) |
| Audit envelope ≤ medium | ✅ | 5-phase chunk → Medium (2 auditors A + B) per skill rubric |
| Confidence ≥ 9/10 | ⚠ **8.5/10** before fork-lock; **9.5/10** after F1.A + F2.A + F3.A + F4.A + F5.A locked | Forks F3 + F4 in particular are architecturally consequential |
| No new migration | ⚠ **conditional** | F1.A path: zero migration. F3.A path requires adding `auth_request.descends_from_grant: option<string>` column → migration 0011. **However**, the column is nullable + unindexed initially — single-column-add migration is K8s-axis A4 compatible (CHK8S-D-05 is not aggravated per CH-08 precedent). Logged as "minor migration" — orchestrator's call whether this triggers escalation. |

**Verdict: ESCALATE TO USER** — five forks need locking, and F3.A introduces a small additive migration (the field-add on AuthRequest). The recommended path (all `.A`) keeps the chunk on track for ~4.5 days, ≤ 1.5× forward-scope, and confidence ≥ 9.5/10, but the orchestrator must surface F1–F5 + the migration to the user before approval.

---

## §1 — Context & principle

**Why this chunk.** The permissions concept doc (`02-auth-request.md` §"System Bootstrap Template", `04-manifest-and-resolution.md` §"Authority Chain", `08-worked-example.md` §"Revocation cascade", `README.md` §"Provenance") specifies a tree-shaped authority model rooted at `system:genesis`. Every Grant must trace, via `descends_from` provenance, back to a hardcoded bootstrap AR approved by the axiomatic `system:genesis` principal. Revocation of any AR in the tree cascades forward — every descendant grant flips to `revoked_at`. Today (verified):

1. The string `"system:genesis"` is a **scattered magic-literal at 11 sites** with no compile-time guard. (`grep '"system:genesis"' modules/crates/` — 11 hits across 5 files.)
2. **No walker exists** — `walk_provenance_chain(grant)` would have to be authored from scratch; the underlying `Grant.descends_from -> AuthRequest` field exists, but the AR-to-AR link `AuthRequest.descends_from_grant` does NOT (verified at `nodes.rs:818-837`).
3. **Revocation is single-hop only** — `revoke_grants_by_descends_from(ar, at)` at `repo_impl_m2.rs:753-781` runs a single SQL `UPDATE grant SET revoked_at WHERE descends_from = $ar`; it does NOT recurse into grandchildren. The Template-revoke handler at `server/src/platform/templates/revoke.rs:89` calls it once. The only acceptance test (`acceptance_authority_templates.rs:164` — `revoke_a_cascades_grants_and_blocks_re_revoke`) exercises the **0-grant case** (fresh org with no fired Template-A grants) — multi-hop cascade is genuinely untested.

CH-14 closes both gaps: typed `system:genesis` constant + `is_bootstrap_ar` predicate + `walk_provenance_chain(grant) -> Vec<AuthRequest>` repo method + BFS-based `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` repo method + new `AuthRequest.descends_from_grant: Option<GrantId>` field + Template-revoke handler flipped to recursive variant + acceptance test asserting (a) every grant chains to bootstrap, (b) multi-hop revoke cascades grandchildren.

**Quality-over-speed restatement.** *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* Specifically for CH-14: the partially-honored authority chain is being lifted into typed Rust without altering wire format or breaking any existing audit invariant. Forward-defensive plumbing only — no callsite outside Template-revoke is rewired.

**Forward-scope reference.** [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 139–143 (CH-14 row, ⚠HIGH, ~4 days, drifts D-new-14 + D-new-18).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `permissions/README.md` | §"Provenance" (lines 92-114) | *"Every permission with provenance like `inherited_from:org:acme-corp` or `delegated_from:agent:supervisor-3` should also be revoked when the parent grant is revoked."* + *"A permission with provenance `system` is more trusted than one with provenance `agent:random-bot`."* | partially-honored — `Grant.descends_from` field exists; revocation cascade is single-hop only | honored — multi-hop revocation cascade ships; system-genesis is the typed root |
| `permissions/02-auth-request.md` | §"System Bootstrap Template" (lines 449-487) | *"Every Grant and Auth Request in the system traces back through the authority tree. There has to be a root — some initial authority that isn't derived from anything else."* + *"`system:genesis`: a hardcoded principal with no session, no profile, no agent behavior. Exists only to approve the System Bootstrap Template's adoption."* | partially-honored — `TemplateKind::SystemBootstrap` exists at `nodes.rs:586`; `"system:genesis"` is a typed `PrincipalRef::System(String)` literal at 11 sites without a constant; bootstrap AR + Grant minted at claim-time | honored — typed `system_genesis_principal()` helper + `SYSTEM_GENESIS_PRINCIPAL` const + `is_bootstrap_ar(ar)` predicate; bootstrap-detection logic centralized; ADR-0053 ratifies the claim-time minting as concept-equivalent to system-init |
| `permissions/04-manifest-and-resolution.md` | §"The Authority Chain" (lines 510-547) | *"Every Grant in the system points to an Auth Request (as its `provenance`). Every Auth Request was approved by one or more approvers, acting under ownership or allocation authority over the affected resources. Every owner's ownership itself traces back through an Auth Request (or through the hardcoded System Bootstrap Template). This forms a tree of authority rooted at the bootstrap."* | partially-honored — `Grant.descends_from -> AR` exists; `AR.descends_from_grant -> Grant` is missing; walker does not exist | honored — `AuthRequest.descends_from_grant: Option<GrantId>` field added (serde-default-shielded); `walk_provenance_chain(grant) -> Vec<AuthRequest>` ships; Template-A/B/C/D/E adoption-AR minters set `descends_from_grant` to the firing grant's id |
| `permissions/08-worked-example.md` | §9.3 "Revocation cascade" (lines 360-364) | *"On 2026-04-22, the project's scope changes and `lead-acme-1` revokes `auth_request:req-9001`. Because `grant-9001` has `revocation_scope: tied_to_auth_request`, it auto-revokes... If `auditor-x-1` had been issued sub-allocations based on `grant-9001`... those would also be revoked forward — the cascade is tree-wide forward-only."* | silent-in-code — single-hop revocation only; multi-hop test absent | honored — `revoke_grants_by_descends_from_recursive(ar, at)` ships BFS algorithm with cycle-detection (depth cap 32); Template-revoke handler flips to recursive; acceptance test `revoke_a_cascades_grandchildren` exercises 3-hop chain |
| `permissions/02-auth-request.md` | §"How Auth Request Approval Maps to a Grant" lines 261-308 (Step 3 — Owner reconsiders and revokes) | *"The Grant is automatically revoked (via the `revocation_scope: tied_to_auth_request` coupling)... per the forward-only revocation rule: past reads remain in audit logs, but no new reads are permitted."* | honored at the AR-state-machine level (`auth_requests/revocation.rs:57`); honored at the single-hop grant-cascade level; **not** honored at multi-hop tree-wide level | honored at all three levels |
| `permissions/02-auth-request.md` | §"Properties of the bootstrap adoption record" (lines 478-486) | *"The record produced by adoption is a regular Auth Request in `Approved` state — the only thing distinguishing it is its provenance (`template:system_bootstrap`) and the fact that its approver is `system:genesis`. It is naturally immutable because no path exists to revoke it (no principal exists above `system:genesis` to authorize revocation)."* | honored — bootstrap AR uses `provenance_template: Some(TemplateId::from_uuid(uuid::Uuid::nil()))` at `claim.rs:218`; immutability follows from no upstream principal | honored (unchanged) — CH-14 adds `is_bootstrap_ar(ar)` predicate that uniquely tests both witnesses (requestor + provenance_template) for walker-termination |
| `phi-core-mapping.md` | (no row touched) | N/A — permissions / authority-chain is a baby-phi governance primitive with no phi-core counterpart | N/A | N/A |

**Concept-audit-matrix sweep (full list of touched rows in `_concept-audit-matrix.md`):**

| Row | File:Line | Current Status | Target Status |
|---|---|---|---|
| `permissions/README.md` Provenance traversal — Bootstrap axiom chain | line 135 | partially-honored | **honored** |
| `permissions/02-auth-request.md` System Bootstrap Template + system:genesis | line 157 | partially-honored | **honored** |
| `permissions/04-manifest-and-resolution.md` Provenance chain to bootstrap — Traversal via `descends_from` | line 181 | partially-honored | **honored** |
| `permissions/08-worked-example.md` Ad-hoc AR + revocation cascade — Revoke walks grants by provenance | line 231 | silent-in-code | **honored** |
| `top-level audit row` (line 44) Provenance — Chain traces to bootstrap | line 44 | partially-honored | **honored** |

5 matrix rows flip. Per CH-12 retro Row F-AUDB-1 (letter-for-letter copy-paste rule), these target Status values are pasted into the matrix at chunk-seal exactly as written here.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::session::Session` / `phi_core::session::recorder::SessionRecorder` | Used as runtime types; baby-phi governance node `domain::model::nodes::Session` is orthogonal (per `phi-core-mapping.md` line 11) | direct-reuse (existing) | unchanged — CH-14 doesn't touch session types |
| `phi_core::types::event::AgentEvent` | Used in agent-loop wiring; `domain::audit::AuditEvent` is the orthogonal governance write log | direct-reuse + orthogonal split (existing) | unchanged — `auth_request.revoked` audit events flow through `domain::audit::AuditEmitter`, not phi-core |
| Any phi-core authority/permissions/provenance type | None — phi-core has no permissions concept | N/A | N/A — permissions remain a wholly baby-phi concern |

**Expected import-count delta at chunk close: +0 phi-core imports.** (Permissions surface is orthogonal to phi-core per `concepts/phi-core-mapping.md` lines 16-37 + every prior permissions chunk CH-04/05/06/07/08/11/12/13.)

**Positive close-audit greps:**
- `grep -rn "use phi_core::" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect **48** (unchanged from baseline; verified `2026-05-08`).

**Forbidden-duplication greps:**
- `grep -rnE 'pub (struct|enum) (Session|AgentEvent|AgentProfile|ExecutionLimits|ModelConfig|McpClient)\b' /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::"` — expect **0**.
- `grep -rnE '^\s*pub fn agent_loop\b' /root/projects/phi/baby-phi/modules/crates/` — expect **0**.

`scripts/check-phi-core-reuse.sh` MUST stay green at chunk close. (Baseline check: green at HEAD `fe1a4be`.)

### §3 cascade-artifact discipline (CH-13 retro Row 2; CH-11 retro per-file breakdown)

CH-14 has **two load-bearing cascades**: (a) the `system:genesis` magic-string consolidation, (b) the `AuthRequest.descends_from_grant: Option<GrantId>` field-add. Per planner v3, each gets the 3-artifact treatment.

**Artifact A — `system:genesis` magic-string cascade.**

(a) **Canonical grep (named):**
```
git -C /root/projects/phi/baby-phi grep -nE '"system:genesis"' modules/crates/
```

(b) **Raw match count:** **11 sites** at HEAD `fe1a4be` (verified `2026-05-08`).

(c) **Per-file breakdown:**
- `modules/crates/domain/src/model/nodes.rs:795` — docstring example for `PrincipalRef` serde format. **No edit** (docstring stays the literal; refers to wire format).
- `modules/crates/domain/src/model/nodes.rs:804` — docstring on `PrincipalRef::System` variant. **No edit**.
- `modules/crates/domain/src/permissions/allocate_refinement.rs:65` — sample JSON in test/docstring. **No edit**.
- `modules/crates/server/src/bootstrap/claim.rs:192` — `requestor: PrincipalRef::System("system:genesis".into())`. **Edit → `requestor: system_genesis_principal()`** (1 site).
- `modules/crates/server/src/bootstrap/claim.rs:203` — `approver: PrincipalRef::System("system:genesis".into())`. **Edit → `approver: system_genesis_principal()`** (1 site).
- `modules/crates/store/tests/repository_test.rs:626,637,1131,1141,1626,1637` — 6 test fixtures. **Edit → `system_genesis_principal()`** (4 sites for `requestor` + `approver`; 2 sites at lines 1131,1141 are a `let holder = ...` + `assert_eq!(s, ...)` pair where the assertion line stays a string-equality test — keep the literal there for round-trip clarity).
- `modules/crates/domain/tests/template_e_props.rs:27` — proptest `Just(PrincipalRef::System("system:genesis".to_string()))`. **Edit → `Just(system_genesis_principal())`** (1 site).

**Predicted edits: 7–9 sites across 4 files** (4 production + 3 tests with low-cardinality variance). Pause discipline trigger: actual edit count > **14** (1.5× upper bound). Aggregate-band, not per-file (CH-07 caveat).

**Artifact B — `AuthRequest.descends_from_grant: Option<GrantId>` field-add cascade.**

This is the load-bearing struct-field cascade. Per CH-11 + CH-13 retros (both biased toward UNDER-prediction), full per-file walk is mandatory.

(a) **Canonical grep (named):**
```
git -C /root/projects/phi/baby-phi grep -nE 'AuthRequest \{$|AuthRequest\s*\{' modules/crates/
```

(b) **Raw match count at draft time:**
```
$ git -C /root/projects/phi/baby-phi grep -nE 'AuthRequest \{$|AuthRequest\s*\{' modules/crates/ | wc -l
```
The planner ran this and got **44 raw matches** across 24 files (some files contain multiple AR-construction sites; some matches are pattern-bindings or struct destructuring rather than struct-construction — see (c) for the filtered breakdown).

(c) **Per-file breakdown** (filtered to actual struct-construction sites; pattern-bindings excluded):

| File | Construction sites |
|---|---|
| `modules/crates/server/src/bootstrap/claim.rs` | 1 (line 190) |
| `modules/crates/server/src/platform/orgs/create.rs` | 1 (template-E adoption AR) |
| `modules/crates/server/src/platform/projects/create.rs` | 1 |
| `modules/crates/server/src/platform/secrets/add.rs` | 1 |
| `modules/crates/server/src/platform/secrets/reveal.rs` | 3 (lines 355, 406, 467) |
| `modules/crates/server/src/platform/model_providers/register.rs` | 1 |
| `modules/crates/domain/src/templates/adoption.rs` | 1 |
| `modules/crates/domain/src/templates/e.rs` | 1 |
| `modules/crates/domain/src/auth_requests/transitions.rs` | 1 (test-fixture-style helper at line 349) |
| `modules/crates/domain/src/auth_requests/state.rs` | 1 (line 298) |
| `modules/crates/domain/src/auth_requests/retention.rs` | 1 (line 109) |
| `modules/crates/domain/src/events/listeners.rs` | 1 (line 1660 — fire listener test fixture) |
| `modules/crates/domain/tests/common/mod.rs` | 1 (line 104; second one at 162 is a Grant fixture, not AR) |
| `modules/crates/domain/tests/auth_request_aggregation_props.rs` | 1 (line 70) |
| `modules/crates/domain/tests/auth_request_revocation_props.rs` | 1 (line 68) |
| `modules/crates/domain/tests/auth_request_retention_props.rs` | 1 (line 45) |
| `modules/crates/domain/tests/auth_request_transition_props.rs` | 1 (line 80) |
| `modules/crates/domain/tests/instance_tags_emission.rs` | 1 (line 170) |
| `modules/crates/domain/tests/in_memory_m5_test.rs` | 1 (line 125) |
| `modules/crates/domain/tests/mcp_cascade_props.rs` | 1 (line 78) |
| `modules/crates/domain/tests/shape_b_approval_matrix_props.rs` | 1 (line 82) |
| `modules/crates/store/tests/apply_org_creation_tx_test.rs` | 1 (line 97) |
| `modules/crates/store/tests/repository_test.rs` | 4 (sample helpers + bootstrap fixtures) |
| `modules/crates/server/tests/acceptance_mcp_servers.rs` | 1 (line 386) |

**Total filtered count: ~28 AR-construction sites across 24 files.**

**Cascade strategy: F1.A-paired.** Add `pub descends_from_grant: Option<GrantId>` with `#[serde(default)]` shielding. **The default value `None` is semantically valid for every existing site** (pre-CH-14 ARs have no parent grant by definition — the chain only starts being walkable when CH-14 ships). Only **two production callsites** must be wired to set `Some(grant_id)`:
1. **Bootstrap claim** (`bootstrap/claim.rs`) — bootstrap AR has no parent grant; stays `None`. The bootstrap is the one node where `is_bootstrap_ar(ar) == true && ar.descends_from_grant == None`.
2. **Template-A/B/C/D/E adoption AR builders** (`templates/{a,b,c,d,e}.rs` + `templates/adoption.rs`) — when an admin adopts a template via Template-E (or any future adoption flow), the adoption AR's `descends_from_grant` SHOULD be the admin's `system:root` grant id (for `claim → bootstrap-AR → admin-grant → adoption-AR` chain integrity). **However**, today the adoption-builder pure-fn (`templates/adoption.rs:28`) takes no grant context. Wiring `descends_from_grant` into adoption builders requires plumbing the firing-grant id from the calling handler down into the builder. **Scope-control decision**: CH-14 ships the FIELD with `#[serde(default)]` but only WIRES it at the bootstrap claim path (which sets `None` — bootstrap is the chain root). Adoption-AR-side wiring is **deferred to a successor chunk** (`D-CH14-FOLLOWUP-01`) to keep CH-14 within ~5 days. The walker still works: at chunk close, every grant's chain reaches bootstrap via the existing `Grant.descends_from -> AR` field — the missing AR-to-Grant link only matters when the chain has > 1 AR hop (which today's data shape does not produce because adoption-ARs use `system:genesis` as approver, terminating the walk one hop early). **CH-14 acceptance tests assert this clearly**: the walker terminates correctly on every shipped chain shape; multi-hop revocation cascade works end-to-end via `Grant.descends_from` (which is sufficient because Template-revoke fans out grants per AR, and each fired grant points back to the adoption AR).

**Predicted edits: 28 AR-construction sites get one new field added (`descends_from_grant: None`) + 1 production wiring at claim.rs.** Pause discipline trigger: actual cascade > **42 sites** (1.5× upper bound) → AskUserQuestion. Aggregate-band over per-file precision (CH-07 caveat).

**Artifact C — `revoke_grants_by_descends_from` callsite cascade.**

(a) **Canonical grep (named):**
```
git -C /root/projects/phi/baby-phi grep -nE 'revoke_grants_by_descends_from' modules/crates/
```

(b) **Raw match count: 5 hits** (the trait def at `repository.rs:993`, the in_memory impl at `in_memory.rs:1195`, the SurrealDB impl at `repo_impl.rs:1655` + `repo_impl_m2.rs:753`, and one production caller at `templates/revoke.rs:89`).

(c) **Per-file breakdown:**
- `modules/crates/domain/src/repository.rs:993` — trait method declaration. **No change** (the existing single-hop method stays for back-compat with `narrow_mcp_tenants`).
- `modules/crates/domain/src/in_memory.rs:1195` — in_memory impl. **No change** (single-hop preserved).
- `modules/crates/store/src/repo_impl.rs:1655` — store-crate trait method delegating to internal helper. **No change**.
- `modules/crates/store/src/repo_impl_m2.rs:753` — single-hop SQL. **No change**.
- `modules/crates/server/src/platform/templates/revoke.rs:89` — **EDIT: flip from `revoke_grants_by_descends_from` to `revoke_grants_by_descends_from_recursive`** (1 site).

**Predicted edits: 1 production callsite flip + 3 trait-method additions** (new `revoke_grants_by_descends_from_recursive` on the trait at `repository.rs` + the in_memory impl + the store impl). Pause discipline: actual flip-sites > 2 → AskUserQuestion (the M2 `narrow_mcp_tenants` cascade legitimately wants single-hop per AR — see F3 fork rationale).

**Additive-enum cascade discipline check (v3 per CH-12):** No new enum variants in CH-14. `RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: u8 }` is a candidate ONLY if the planner introduces a depth-cap error variant; per CH-12 catch-all rule, the existing repo-error consumers all use `From<RepositoryError> -> ApiError` mapping via `Display`, so a new variant requires **0 callsite edits** outside the variant declaration site.

**Tag-write Repository contract reading-list conditional (v3 per CH-12):** N/A — CH-14 introduces no `update_*_tags` / `set_*_tags` / `retag_*` Repository methods.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | None — the walker is a pure read; the recursive revoke is a sequence of repo writes (no in-process cache, no `OnceCell`, no `RwLock`) | **no** | no impact |
| **A2** | New IPC channel | None — no `mpsc` / `broadcast` / `watch` / `Notify` introduced | **no** | no impact |
| **A3** | New pod-local resource | None — no new files / sockets / sub-processes / lock files | **no** | no impact |
| **A4** | Migration runner / first-apply race | **One additive nullable column on `auth_request` table: `descends_from_grant option<string>`** (migration 0011). Single-column-add migration is K8s-axis A4 compatible per CHK8S-D-05 + CH-08 precedent (CH-08 added `auth_request.allocate_refinement` similarly without aggravating the leader-election deferral). | **no** | compatible — migration is idempotent + nullable; CHK8S-D-05 stands; no new entry |
| **A5** | Trait-shape requirement | **Yes** — new trait method `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` MUST be implementable on remote-DB backend. Algorithm uses BFS via per-level SQL queries — works identically on embedded RocksDB and remote SurrealDB clusters. Same trait pattern as the existing single-hop method. | **no** | compatible — trait method is `&dyn Repository` dispatchable; the SurrealDB impl uses idiomatic per-level `SELECT` + `UPDATE` queries. Walker `walk_provenance_chain` is similarly trait-shaped. |
| **A6** | Cross-pod state sharing | None new — every `descends_from` walk + revoke flows through `Repository` (already durable per ADR-0033 §D33.2 / `SurrealStore::open_remote`) | **no** | no impact |
| **A7** | Audit hash-chain symmetry | **Yes** — recursive revoke emits N audit events (one per revoked grant). The existing `Template-revoke` handler emits ONE summary `template.revoked` event with `grant_count`. Per concept-doc 02 §"Properties of the bootstrap adoption record" + ADR-0050 strictest-wins composition, multi-hop revocation should emit one audit event per CASCADED AR (not per-grant). **Decision**: keep CH-08 / CH-13 single-writer pattern: one summary event per Template-revoke call (already captures `grant_count_revoked`), plus one `auth_request.revoked` event per cascaded AR (the `revoke_ar` domain helper at `auth_requests/revocation.rs:57` already builds these — Template-revoke handler currently only emits for the immediate AR; the recursive cascade adds N-1 more emissions). canonical_bytes for `auth_request.revoked` exclude `prev_event_hash` per existing chain semantics. Cross-pod determinism preserved. | **no** | compatible — single-writer guarantee preserved (only `Template-revoke` handler emits in this flow); audit chain extends symmetrically per cascaded AR |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — not touched.
- D33.2 (`SurrealStore::open_remote`) — yes, BFS recursive-revoke is implemented per-level via standard SQL (works on both `open_embedded` and `open_remote`).
- D33.3 (SIGTERM graceful shutdown) — not touched (no new `tokio::spawn`).
- D33.4 (`EventBus.shutdown` + `drain`) — not touched.

**Conclusion paragraph.** *K8s-neutral.* CH-14 introduces no new in-process state, no IPC, no pod-local resources, and no audit-chain divergence. The one migration is single-column-add nullable on an existing table — CHK8S-D-05 stands as-is, no new ledger entry. The new trait method `revoke_grants_by_descends_from_recursive` is `&dyn Repository`-dispatchable + works identically on remote-DB backends.

---

## §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/authority-chain.md` (new file) | **Yes — net-new design doc** describing system_genesis_principal, walk_provenance_chain semantics, BFS recursive cascade algorithm, depth cap, the bootstrap-detection predicate, claim-time-as-system-init divergence rationale. | **(a) Update in-chunk** — ship as P3 deliverable (~250 words; format mirrors `m5_2/architecture/audit-class-composition.md`). |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/authority-chain-operations.md` (new file) | **Yes** — operators need to understand the cascade-revoke audit-event sequence (one `template.revoked` summary + N `auth_request.revoked` per cascaded AR + N `grant.revoked` implicit via `Grant.revoked_at` flip). Plus a runbook for "I revoked Template A on the wrong org — recovery procedure" (answer: there is none; the cascade is forward-only — per concept-doc invariant). | **(a) Update in-chunk** — ship as P3 deliverable (~150 words). |
| **User-guide** | `m5_2/user-guide/` | **No new operator-facing CLI / web behaviour** — Template-revoke already exists at `cli/src/commands/template.rs:235` ("template revoked — {n} grant(s) cascaded"); the count just becomes accurate for multi-hop cases. No new CLI command. | **(b) Defer** — defer to **none required**; the CLI message stays correct (just counts more grants when applicable). |

Doc updates land at **P3** as first-class deliverables (not seal-time appendices).

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-14` | `../drifts/D-new-14.md` | HIGH | discovered → **remediated** | `system:genesis` typed const + `walk_provenance_chain` + acceptance test "every grant chains to bootstrap" — closed at chunk seal. |
| `D-new-18` | `../drifts/D-new-18.md` | HIGH | discovered → **remediated** | Multi-hop revocation cascade via `revoke_grants_by_descends_from_recursive` + acceptance test asserting grandchild-grant revocation. |

Recommended new follow-up drift (if F1.A is locked + adoption-AR descends_from_grant wiring is deferred per F3.A scope-control): `D-CH14-FOLLOWUP-01` — wire `descends_from_grant` into Template-A/B/C/D/E adoption-AR builders (forward-defensive plumbing for chain-depth > 2). Severity: LOW. Owner: any successor chunk that introduces a new adoption flow. Created at chunk-seal if needed.

---

## §5 — ADRs drafted

**Highest existing ADR**: `ADR-0052` (CH-08 Allocate/Transfer Cardinality + AllocateRefinement). CH-14 picks **ADR-0053**.

| ADR | Title | Drafted-at-phase | Decision summary | Expected flip to Accepted |
|---|---|---|---|---|
| ADR-0053 | `system:genesis` axiom + authority-chain walker + recursive revocation cascade | P0 (Proposed) | Codifies (D53.1) the typed `system_genesis_principal()` helper + `SYSTEM_GENESIS_PRINCIPAL` const; (D53.2) the `is_bootstrap_ar(ar)` two-witness predicate; (D53.3) the `walk_provenance_chain(grant) -> Vec<AuthRequest>` algorithm with depth cap 32 + cycle detection; (D53.4) the BFS-based `revoke_grants_by_descends_from_recursive(ar, at)` algorithm (back-compat single-hop preserved); (D53.5) the `AuthRequest.descends_from_grant: Option<GrantId>` field-add with `#[serde(default)]` shielding; (D53.6) claim-time-as-system-init divergence acceptance with rationale; (D53.7) audit-event emission semantics: one `template.revoked` summary + N `auth_request.revoked` per cascaded AR. | **P4** chunk-seal |

**ADR file path**: `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`

**Cross-references (per CH-13 retro v2 ADR-body checklist + CH-08 retro Row 1 milestone-prefixed paths):**

(a) **Originating concept-doc + section + line range**: `permissions/02-auth-request.md` §"System Bootstrap Template" lines 449-487; `permissions/04-manifest-and-resolution.md` §"The Authority Chain" lines 510-547; `permissions/08-worked-example.md` §9.3 lines 360-364; `permissions/README.md` §"Provenance" lines 92-114.

(b) **Closed drift(s)**: D-new-14, D-new-18.

(c) **Prior ADRs cited as precedent (milestone-prefixed paths per CH-08 retro Row 1):**
- `m3/decisions/0022-...` — CH-K8S-PREP / compound-tx precedent (looked up at draft time; the platform-bootstrap compound-tx primitive). Verified: `0022-compound-tx-bootstrap-primitive.md` is the candidate. **Action item for implementer**: at P0, `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m3/decisions/` and confirm the actual filename. If `0022-...` does not exist or is differently-titled, cite `m4/decisions/0028-...` instead (the M4 platform compound-tx pattern, if present). The point of the cite is "compound-tx primitive precedent" — pick whichever shipped ADR established it.
- `m5_2/decisions/0033-k8s-prep-refactors.md` §D33.2 (SurrealStore::open_remote — relevant for trait-shape A5 conformance).
- `m5_2/decisions/0050-audit-class-composition-strictest-wins.md` (CH-13) §D50.5 (Grant-denormalization + audit-event-source pattern; precedent for CH-14 §D53.5 `AuthRequest.descends_from_grant` field-add with serde-default shielding).
- `m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md` (CH-08) §D52.3 (typed-field cascade pattern; precedent for CH-14 §D53.5 `Option<GrantId>` field-add cascade).
- `m5_2/decisions/0048-per-session-consent-gating.md` (CH-11) §D48.1 (`Grant.approval_mode` denormalization with `#[serde(default)]` — precedent for §D53.5 serde back-compat).
- `m5_2/decisions/0051-multi-scope-cascade-contractor-model.md` (CH-07) §D51.1 (cascade algorithm inside engine — the BFS in CH-14 §D53.4 is structurally analogous to CH-07's cascade resolution but in the revoke-direction).
- (Forward reference, not a citation — `m5_2/decisions/0033-...` stays the K8s-conformance anchor for §3.B.)

(d) **Forward-scope row**: [`baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 139–143.

**Forks header pattern (per CH-13 v2 + CH-12 retro)**: at chunk-seal, depending on user-lock outcome:
- All `.A` locked (planner-recommended): `Forks (F1–F5 user-locked at plan approval to F1.A / F2.A / F3.A / F4.A / F5.A — all align with planner recommendation)`.
- Any divergence: `Forks (F<N> user-locked to F<N>.<X> at plan approval — diverges from planner recommendation F<N>.A; F<other> at planner-recommendation)`.

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-08** (ADR-0052) | `Grant.allocate_refinement: Option<AllocateRefinement>` field-add round-trips via serde with `None` default for legacy grants (CH-14 reuses this pattern for `AuthRequest.descends_from_grant`) | `cargo test --workspace -j 4 -- allocate_refinement_legacy_decode` |
| **CH-08** (ADR-0052) | `apply_transfer_grant` compound-tx primitive — proves multi-write atomicity is implementable on both backends (CH-14 reuses same pattern for recursive revoke) | `cargo test -p store --test transfer_grant_surreal_test` |
| **CH-13** (ADR-0050) | `Grant.audit_class` denormalization + audit-event-source plumbing — the Template-revoke `template.revoked` audit event MUST still emit `audit_class_source` correctly | `cargo test -p server --test acceptance_authority_templates -- revoke_a_cascades_grants_and_blocks_re_revoke` |
| **CH-11** (ADR-0048) | `Grant.approval_mode` field-add with `#[serde(default)]` — pre-CH-11 grant rows decode cleanly (CH-14 must not break this) | `cargo test --workspace -j 4 -- approval_mode_legacy_decode` |
| **CH-09** (ADR-0045) | Consent node 11-field shape — auth_request.revoked event emission MUST not corrupt the consent table | `cargo test -p store --test repository_test -- consent` |
| **CH-10** (ADR-0047) | Consent state-machine + sweeper — auth_request.revoked propagation MUST not interact with consent revocation cascade | `cargo test --workspace -j 4 -- consent_revocation` |
| **CH-K8S-PREP** (ADR-0033) | `SessionRegistry` trait + `SurrealStore::open_remote` + SIGTERM drain — CH-14's new trait method MUST be `&dyn Repository`-dispatchable | `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` + manual: confirm `revoke_grants_by_descends_from_recursive` has no generics |
| **CH-M2-bootstrap** | Bootstrap claim minting — CH-14's `is_bootstrap_ar` predicate must match the AR shape `claim.rs` produces today | `cargo test -p server --test acceptance_bootstrap` |
| **M2 narrow_mcp_tenants** | `narrow_mcp_tenants` calls `m2_revoke_grants_by_descends_from` (single-hop) per AR — MUST stay single-hop after CH-14 introduces the recursive variant | `cargo test -p server --test acceptance_mcp_servers -- narrow` |

This table runs at **chunk open** (P0) before any phase opens, and again at **chunk seal** (P4). Any regression produces a new drift file + surfaces as an open question for user before the chunk proceeds.

---

## §7 — Phases within the chunk

The chunk is **5 phases** (Medium audit envelope per `audit-envelope-size` skill: 3-5 phases → 2 auditors A + B).

### P0 — Scaffolding + ADR-0053 Proposed + migration 0011 + reading-list re-verification

**Goal.** Open the cycle, draft ADR-0053 as Proposed, ship the additive `auth_request.descends_from_grant` SurrealDB column via migration 0011, walk §6 reading list, confirm 1408 baseline test count + green CI guards.

**Deliverables.**
1. `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md` — Proposed status, all 7 sub-decisions D53.1–D53.7 outlined.
2. `modules/crates/store/migrations/0011_authority_chain.surql` — `DEFINE FIELD descends_from_grant ON auth_request TYPE option<string>;` (idempotent; default null).
3. `modules/crates/store/tests/migrations_0011_test.rs` — verifies the column is queryable + nullable.
4. `_concept-audit-matrix.md` — five rows (lines 44, 135, 157, 181, 231) prepared for status flip at P4 seal (text drafted but not yet committed).
5. Drift files `D-new-14.md` + `D-new-18.md` — append `2026-05-08 — in-chunk-plan` lifecycle entry.
6. **Re-run §6 commands** — confirm green at `2026-05-08`.

**Tests.** +1 (migration 0011 test).

**Concept-alignment check.** None yet — P0 is paperwork-only.

**phi-core leverage check.** Re-run baseline grep — expect 48 (unchanged).

**User-facing doc updates.** None yet (P0 is governance-tier).

**Confidence target.** **100%** — scaffolding-phase default per template §7.

**Pause discipline.** Pause if `cargo test --workspace -j 4` baseline ≠ 1408.

### P1 — `system:genesis` const + `is_bootstrap_ar` predicate + `AuthRequest.descends_from_grant` field-add cascade

**Goal.** Codify the axiomatic principal as a typed const + helper-fn; add the `is_bootstrap_ar` predicate; add the `AuthRequest.descends_from_grant: Option<GrantId>` field with `#[serde(default)]` shielding; cascade through ~28 AR-construction sites with `descends_from_grant: None` (semantically valid for all pre-CH-14 ARs).

**Deliverables.**
1. New module `modules/crates/domain/src/permissions/axioms.rs` exposing:
   - `pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis";`
   - `pub fn system_genesis_principal() -> PrincipalRef`
   - `pub const SYSTEM_BOOTSTRAP_TEMPLATE_ID: TemplateId = TemplateId::from_uuid(uuid::Uuid::nil());` (compile-time const if `from_uuid` is `const fn`; else lazy_static or function)
   - `pub fn is_bootstrap_ar(ar: &AuthRequest) -> bool` — the two-witness predicate (F5.A)
   - 4 unit tests: each predicate + helper round-trips
2. `modules/crates/domain/src/permissions/mod.rs` — re-export `axioms::*`.
3. `modules/crates/domain/src/model/nodes.rs:818` — add field `pub descends_from_grant: Option<GrantId>` to `AuthRequest` struct with `#[serde(default)]`. Doc comment cross-refs ADR-0053 §D53.5.
4. **AR-construction cascade** — add `descends_from_grant: None,` to all 28 sites listed in §3 Artifact B (per-file breakdown). At `bootstrap/claim.rs:190`, the bootstrap AR explicitly stays `None` (it's the chain root). At `template_e_props.rs:27` + similar, also `None` (existing tests don't model multi-hop).
5. `modules/crates/server/src/bootstrap/claim.rs:192,203` — flip `PrincipalRef::System("system:genesis".into())` to `system_genesis_principal()` (2 sites).
6. `modules/crates/store/tests/repository_test.rs:626,637,1626,1637` — flip 4 of the 6 `"system:genesis"` literals to `system_genesis_principal()` (lines 1131,1141 stay literals — they're string-equality assertions on the wire format).
7. `modules/crates/domain/tests/template_e_props.rs:27` — flip to `system_genesis_principal()` (1 site).

**Tests.** +5 (axioms module unit tests: const round-trip, helper-fn-equivalence-with-literal, is_bootstrap_ar-positive-on-claim-AR, is_bootstrap_ar-negative-on-template-AR, is_bootstrap_ar-negative-on-non-genesis-system-AR). Net P1 test delta: **+5**.

**Concept-alignment check.** Row "System Bootstrap Template + system:genesis" (matrix line 157) flips partially-honored → honored. Verified by reading the new `axioms.rs` module + the `is_bootstrap_ar` predicate + the cascade fan-out across 28 sites.

**phi-core leverage check.** Re-run grep — expect 48 (unchanged).

**User-facing doc updates.** None this phase.

**Confidence target.** **≥ 97%** — content phase per template §7. The 28-site cascade is the load-bearing risk; per-file breakdown in §3 Artifact B mitigates.

**Pause discipline.** Pause via `AskUserQuestion` if:
- Actual `descends_from_grant: None` cascade > **42 sites** (1.5× upper bound 28).
- Actual `system_genesis_principal()` flip-sites > **14** (1.5× upper bound 9).
- Any AR-construction site requires a `Some(grant_id)` value (would mean adoption-flow plumbing is non-deferrable — see F3.A scope-control decision).

### P2 — `walk_provenance_chain(grant) -> Vec<AuthRequest>` repo method + ID-based BFS

**Goal.** Ship the trait method + both backend impls + acceptance test "every shipped grant chains to bootstrap."

**Deliverables.**
1. `modules/crates/domain/src/repository.rs` — add trait method:
   ```rust
   /// Walk the provenance chain root-to-leaf. Returns the bootstrap AR
   /// at index 0 and the grant's direct AR at the last index. Empty if
   /// `grant.descends_from` is `None` (legacy / pre-CH-14 grants).
   /// Returns `Err(ProvenanceCycleDepthExceeded)` if the chain depth
   /// exceeds 32 hops (defensive guard against schema bugs).
   async fn walk_provenance_chain(
       &self,
       grant: GrantId,
   ) -> RepositoryResult<Vec<AuthRequest>>;
   ```
2. `modules/crates/domain/src/in_memory.rs` — impl: BFS from `grant.descends_from` upward via `ar.descends_from_grant -> grant -> ar -> ...` until `is_bootstrap_ar(current_ar)` returns true OR depth cap hit OR `descends_from_grant` is None (chain ends).
3. `modules/crates/store/src/repo_impl.rs` — same BFS, but per-level via SQL `SELECT` queries.
4. **New `RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: u8 }` variant** + `From` conversion for ApiError mapping (HTTP 500 — internal invariant violation).
5. Acceptance test `modules/crates/server/tests/acceptance_authority_chain.rs` — new file:
   - `every_grant_chains_to_bootstrap_after_claim` — calls bootstrap-claim, asserts `walk_provenance_chain(claim.grant_id)` returns exactly `[bootstrap_ar]` and `is_bootstrap_ar(bootstrap_ar)` is true.
   - `walk_returns_empty_for_grant_with_no_descends_from` — legacy/null case.
   - `walk_returns_err_on_synthetic_cycle` — write a cycle directly via repo helpers (test-only); assert `ProvenanceCycleDepthExceeded` at depth 32.

**Tests.** +6 (walker unit tests + 3 acceptance tests). Net P2 test delta: **+6**.

**Concept-alignment check.** Rows "Provenance traversal — Bootstrap axiom chain" (matrix line 135) + "Provenance chain to bootstrap" (line 181) + top-level "Provenance — Chain traces to bootstrap" (line 44) flip partially-honored → honored.

**phi-core leverage check.** Re-run grep — expect 48 (unchanged).

**User-facing doc updates.** None this phase.

**Confidence target.** **≥ 97%**.

**Pause discipline.** Pause if (a) the chain-depth cap is hit on any production data shape today (would indicate a real schema bug, not a defensive guard), or (b) the SurrealDB BFS impl can't be expressed in idiomatic per-level SQL (would force F3.B alternative).

### P3 — `revoke_grants_by_descends_from_recursive` + Template-revoke flip + multi-hop acceptance test + architecture/operations docs

**Goal.** Ship the BFS-based recursive revoke + flip the Template-revoke handler + acceptance test for grandchild revocation. Also ship the two new user-facing docs (architecture + operations).

**Deliverables.**
1. `modules/crates/domain/src/repository.rs` — add trait method:
   ```rust
   /// Revoke every live grant in the descend-tree rooted at `ar`,
   /// forward-only (already-revoked grants untouched). Returns the
   /// flat ordered list of all revoked grants across all levels.
   /// Cycle protection: depth cap 32 with explicit error return.
   /// The existing single-hop `revoke_grants_by_descends_from` is
   /// preserved verbatim for back-compat with M2 narrow_mcp_tenants.
   async fn revoke_grants_by_descends_from_recursive(
       &self,
       ar: AuthRequestId,
       at: DateTime<Utc>,
   ) -> RepositoryResult<Vec<GrantId>>;
   ```
2. `modules/crates/domain/src/in_memory.rs` — BFS impl: starting from `ar`, find all grants where `descends_from = Some(ar)` and not yet revoked → revoke them; for each revoked grant, find all ARs where `descends_from_grant = Some(grant.id)` → recurse. Depth cap 32.
3. `modules/crates/store/src/repo_impl.rs` — same BFS via per-level SQL. Each level emits **one `auth_request.revoked` audit event per cascaded AR** (per concept-doc 02 §"Step 3" + ADR-0053 §D53.7). Audit emission via injected `AuditEmitter` (NOT directly in the repo — emitter is passed as parameter or wrapped in a higher-level domain helper).
4. **`server/src/platform/templates/revoke.rs:89`** — flip from `revoke_grants_by_descends_from(next.id, input.now)` to `revoke_grants_by_descends_from_recursive(next.id, input.now)`. Audit emission for cascaded ARs is added: for each cascaded AR (level ≥ 1), emit one `auth_request.revoked` event via the existing `revoke_ar(...)` domain helper at `auth_requests/revocation.rs:57` (returns `(AuthRequest, AuditEvent)` — emit the event side).
5. Acceptance test `modules/crates/server/tests/acceptance_authority_chain.rs` (extends P2's file):
   - `revoke_cascades_to_grandchildren` — fixture: bootstrap-claim → org-create → adopt Template A → fire one Template-A grant → manually mint a sub-AR descending from that grant → mint a grandchild grant from the sub-AR → revoke Template A → assert all 3 grants revoked + 2 cascaded `auth_request.revoked` events emitted.
   - `revoke_cascade_emits_one_event_per_cascaded_ar` — counts audit events.
   - `revoke_cascade_terminates_at_depth_cap` — synthetic cycle / depth-32 chain → `ProvenanceCycleDepthExceeded`.
   - `revoke_cascade_does_not_re_revoke_already_revoked_grants` — idempotency.
6. **`baby-phi/docs/specs/v0/implementation/m5_2/architecture/authority-chain.md`** — new ~250-word design doc (matches `audit-class-composition.md` shape). Sections: §"system_genesis_principal axiom" + §"walk_provenance_chain semantics" + §"BFS recursive cascade algorithm + depth cap" + §"is_bootstrap_ar two-witness predicate" + §"Claim-time-as-system-init divergence (ADR-0053 §D53.6)".
7. **`baby-phi/docs/specs/v0/implementation/m5_2/operations/authority-chain-operations.md`** — new ~150-word operations doc. Sections: §"Audit-event sequence on Template-revoke" + §"Forward-only-cascade — no recovery procedure".
8. **`baby-phi/docs/specs/v0/implementation/m1/architecture/audit-events.md`** — append CH-14 amendment header line documenting the new per-cascaded-AR `auth_request.revoked` emission frequency change (event_type unchanged; emission cardinality changes from 1 to N where N = cascade depth).

**Tests.** +6 (BFS recursive: 4 acceptance + 2 unit-level). Net P3 test delta: **+6**.

**Concept-alignment check.** Row "Ad-hoc AR + revocation cascade" (matrix line 231) flips silent-in-code → honored. Verified by `revoke_cascades_to_grandchildren` test.

**phi-core leverage check.** Re-run grep — expect 48 (unchanged).

**User-facing doc updates.** §3.C rows (Architecture + Operations) satisfied this phase.

**Confidence target.** **≥ 97%**.

**Pause discipline.** Pause via `AskUserQuestion` if:
- Audit-event cardinality semantics differ from concept-doc expectation (e.g., if user wants per-grant emission instead of per-AR, F5.B-style divergence — re-spawn planner per CH-12 retro discipline).
- BFS impl introduces N+1 query problem at depth ≥ 4 (would force optimization in this chunk; alternative: defer to follow-up).

### P4 — Chunk seal + ADR-0053 Accepted + matrix flips + drift remediations + paperwork

**Goal.** Flip ADR-0053 to Accepted, transition both drift files to remediated, flip 5 audit-matrix rows letter-for-letter (per CH-12 F-AUDB-1 rule), update verified-headers, run final §12 verification recipe, write the cycle close report.

**Deliverables.**
1. `0053-system-genesis-authority-chain-revocation-cascade.md` — flip status `Proposed` → `Accepted`. Forks header populated per the actual user-lock outcome at plan approval.
2. `D-new-14.md` — add `2026-05-08 — remediated — CH-14 chunk-seal — <evidence sentence>` lifecycle entry.
3. `D-new-18.md` — same.
4. `_concept-audit-matrix.md` — flip 5 rows (lines 44, 135, 157, 181, 231) **letter-for-letter from §2 target column** (CH-12 F-AUDB-1 rule).
5. Verified-header updates on every modified concept doc + architecture doc + operations doc — match body diff exactly (P4 paperwork checklist v2026-05-03 + v2026-05-04).
6. (Optional) `D-CH14-FOLLOWUP-01.md` if F3.A scope-control deferred adoption-AR-side wiring.
7. **Run `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh`** — confirm green.
8. **Run `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`** — confirm green.
9. **Run `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh`** — confirm green.
10. **Run `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh`** — confirm green.
11. **Run `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets -j 4`** — confirm zero warnings.
12. **Run `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4`** — confirm pass at expected count (see §8).
13. **Update `_cycle-index.md`** with CH-14 row.

**Tests.** +0 in P4. Final delta: see §8.

**Concept-alignment check.** All 5 matrix rows confirmed flipped + verified by re-grep on file.

**phi-core leverage check.** Final greps confirm 48 imports + zero forbidden duplications.

**User-facing doc updates.** All §3.C rows verified updated; verified-header line 1 matches body diff per P4 checklist v2026-05-03.

**Confidence target.** **≥ 99%** — seal phase per template §7.

**Pause discipline.** Pause if any CI guard fails or if test count diverges from §8 prediction band.

---

## §8 — Tests summary

**Baseline at chunk-open**: 1408 passed / 0 failed / 2 ignored — verified `cargo test --workspace -j 4` at HEAD `fe1a4be` on `2026-05-08`.

**Expected new tests by phase:**
- P0: +1 (migration 0011 test)
- P1: +5 (axioms module: 4 unit + 1 round-trip)
- P2: +6 (walker: 3 unit + 3 acceptance)
- P3: +6 (BFS recursive cascade: 4 acceptance + 2 unit)
- P4: +0 (paperwork only)

**Sum: +18 deliverable-listed new tests.** Per CH-11 + CH-12 retro asymmetric ×1.0–×1.20 buffer:
- Lower bound (CH-12 bull's-eye target): **1408 + 18 = 1426**.
- Upper bound (orchestrator-accept band): **1408 + 18 × 1.20 = 1408 + 22 = 1430**.
- **Plan §8 chunk-close prediction band: 1426–1430 passed / 0 failed / 2 ignored.**

Healthy implementer over-shoot is biased high, not symmetric. CH-14 expects healthy round-trip helpers + paired audit-event tests + ISO-8601 helper tests + idempotency regression tests → over-shoot to 1430 is normal and should NOT trigger re-plan. Outside the band → AskUserQuestion.

**Layer breakdown:**
- Unit tests: +13 (axioms + walker + BFS unit-level)
- Integration / acceptance: +5 (acceptance_authority_chain — 4 happy/edge + 1 cycle protection)

**Named test files:**
- `modules/crates/store/tests/migrations_0011_test.rs` (new)
- `modules/crates/domain/src/permissions/axioms.rs` mod-level `#[cfg(test)]` block
- `modules/crates/domain/src/in_memory.rs` test additions (BFS unit tests)
- `modules/crates/store/src/repo_impl.rs` test additions
- `modules/crates/server/tests/acceptance_authority_chain.rs` (new)

**Named expected-still-green tests (fragile):**
- `acceptance_authority_templates::revoke_a_cascades_grants_and_blocks_re_revoke` — STILL GREEN (the 0-grant case still passes; the test exercises the cascade machinery without grants); audit-event count assertion (currently `assert_eq!(body["grant_count_revoked"], 0);`) stays 0 since fresh-org fixture has no fired grants.
- `acceptance_bootstrap` — STILL GREEN (claim.rs:192,203 still produce identical wire format after F1.A flip — the const value `"system:genesis"` is unchanged).
- `acceptance_mcp_servers::narrow_mcp_tenants_revokes_grants_per_excluded_org` — STILL GREEN (single-hop `revoke_grants_by_descends_from` unchanged).
- `auth_request_aggregation_props` proptests — STILL GREEN (`descends_from_grant: None` injected at fixture line 70 doesn't change aggregation behavior).
- All 28 AR-construction-site tests — STILL GREEN (field-add with serde-default).

---

## §9 — Pre-chunk gate

**Reading list (mandatory) — verified read 2026-05-08:**

1. ✅ `concepts/permissions/02-auth-request.md` — full file (533 lines).
2. ✅ `concepts/permissions/04-manifest-and-resolution.md` — full file.
3. ✅ `concepts/permissions/08-worked-example.md` — full file.
4. ✅ `concepts/permissions/README.md` — full file.
5. ✅ `drifts/D-new-14.md` — full file.
6. ✅ `drifts/D-new-18.md` — full file.
7. ✅ `drifts/_concept-audit-matrix.md` — relevant rows (44, 135, 157, 181, 231).
8. ✅ Prior plans: CH-08 (`ch-08-allocate-transfer-cardinality-refinements-7cbe74a4/plan.md`), CH-13 (`ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md`) — read for typed-field-cascade + audit-emission patterns.
9. ✅ `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` — §5 chunk row + §7 binding Q&A.
10. ✅ `baby-phi/CLAUDE.md` — phi-core Leverage section + Granular Bash discipline.
11. ✅ Existing code: `bootstrap/claim.rs`, `auth_requests/revocation.rs`, `templates/revoke.rs`, `repo_impl_m2.rs:680-781`, `in_memory.rs:1170-1209,2510-2540`, `repository.rs:950-998`, `model/nodes.rs:540-870`.
12. ✅ Existing migrations: `0002_platform_setup.surql`, `0005_sessions_templates_system_agents.surql` — for migration 0011 stylistic reference.

**Conditional reading (CH-11 v2026-05-03)**: N/A — CH-14 doesn't touch `domain::permissions::engine` Step N body.

**Conditional reading (CH-12 v2026-05-04)**: N/A — CH-14 doesn't introduce a tag-write Repository method.

**Carry-forward invariants** (verified green at chunk open `2026-05-08`):
- ✅ `cargo test --workspace -j 4` test count = 1408 passed / 0 failed / 2 ignored.
- ✅ `scripts/check-phi-core-reuse.sh` green (assumed; CI baseline at HEAD `fe1a4be`).
- ✅ `scripts/check-doc-links.sh` green.
- ✅ `scripts/check-ops-doc-headers.sh` green.
- ✅ `scripts/check-spec-drift.sh` green.
- ✅ `modules/` diff against HEAD = empty (no preload edits).

**Pending decisions carried into this chunk:**
- Forks F1–F5 — orchestrator must surface to user via AskUserQuestion before approval.
- F3.A scope-control: adoption-AR-side `descends_from_grant` wiring **deferred** to D-CH14-FOLLOWUP-01 if needed.
- Migration 0011 is single-column-add nullable — counts as "minor migration" per orchestrator gate-1; user-lock decides whether this triggers escalation.

---

## §10 — Close criteria

**4 aspects (each graded pass / fail):**

- **Code aspect** — all 5 phases' deliverables shipped; `cargo test --workspace -j 4` passes at 1426–1430 ± buffer; clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect** — *Governance*: D-new-14 + D-new-18 status flipped to remediated; ADR-0053 Accepted; 5 audit-matrix rows letter-for-letter flipped; verified-headers match body diffs (P4 checklist v2026-05-03). *User-facing*: `m5_2/architecture/authority-chain.md` + `m5_2/operations/authority-chain-operations.md` shipped in P3; `m1/architecture/audit-events.md` amendment header line added.
- **phi-core leverage aspect** — §3 import-count delta = 0 (predicted = 0); 0 forbidden-duplication grep hits; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's target-status achieved; none remain `partially-honored` for the rows CH-14 owns.

**2 confidence % (each with named numerator/denominator):**

- **Implementation confidence %** target: **≥ 9.5/10** = `(claims-honored) / (claims-in-scope-for-chunk)`. The 10 claims-in-scope:
  1. `system_genesis_principal()` typed const ships at `axioms.rs`.
  2. `is_bootstrap_ar(ar)` two-witness predicate ships.
  3. `AuthRequest.descends_from_grant: Option<GrantId>` field-add with `#[serde(default)]`.
  4. 28-site cascade lands `descends_from_grant: None` at every existing AR-construction site.
  5. `walk_provenance_chain(grant) -> Vec<AuthRequest>` repo method ships on both backends.
  6. `revoke_grants_by_descends_from_recursive(ar, at)` BFS ships on both backends.
  7. Template-revoke handler flips to recursive variant.
  8. `revoke_cascades_to_grandchildren` acceptance test green.
  9. `every_grant_chains_to_bootstrap_after_claim` acceptance test green.
  10. ADR-0053 Accepted + drift remediations + matrix flips + verified-header updates.

- **Documentation confidence %** target: **100%** = `(doc-pages-verified) / (doc-pages-touched)`:
  - 4 governance docs (ADR-0053 + 2 drift files + matrix) + 2 user-facing docs (authority-chain.md + authority-chain-operations.md) + 1 amendment-header doc (audit-events.md) = 7 doc-pages-touched. All 7 must be cross-checkable against code + concept docs without ambiguity. Target: 7/7 = 100%.

**Composite = min(impl%, doc%, code-aspect, phi-core-aspect, concept-alignment-aspect)**. Composite below 95% blocks close.

**Tag-write contract conditional close criterion (CH-12 v3 — N/A here)**: CH-14 introduces no tag-write methods; the conditional doesn't fire. Documented for completeness.

**P4 paperwork checklist (v2026-05-03):** for every modified doc with a verified-header, confirm header description matches body diff exactly. Mismatch → fix before chunk-seal.

**P4 paperwork checklist addendum (v2026-05-04):** for every `_concept-audit-matrix.md` row touched, the new Status column value is copy-pasted **letter-for-letter** from §2 target column. CH-14 flips 5 rows; all 5 use copy-paste verbatim.

---

## §11 — Post-chunk independent audit plan

**Phase count**: 5 (P0–P4) → **Medium envelope (2 auditors): A + B** per `audit-envelope-size` skill.

### Audit A — code + phi-core + K8s (≤ 600 words)

```
You are auditing CH-14 in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md.

Verify each claim with file:line citation:
1. `pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis"` ships at modules/crates/domain/src/permissions/axioms.rs (cite line); the helper-fn `system_genesis_principal()` returns `PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL.into())`.
2. `is_bootstrap_ar(ar) -> bool` predicate at axioms.rs uses the F5.A two-witness check: `ar.requestor == system_genesis_principal() && ar.provenance_template == Some(SYSTEM_BOOTSTRAP_TEMPLATE_ID)`.
3. `AuthRequest.descends_from_grant: Option<GrantId>` field added at modules/crates/domain/src/model/nodes.rs (cite line) with `#[serde(default)]` shielding.
4. `walk_provenance_chain(grant) -> Vec<AuthRequest>` ships on Repository trait at modules/crates/domain/src/repository.rs (cite line); both impls ship: in_memory.rs + repo_impl.rs.
5. `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` ships on Repository trait; BFS algorithm verified by reading both impls; depth cap 32 enforced via `RepositoryError::ProvenanceCycleDepthExceeded`.
6. Template-revoke handler at modules/crates/server/src/platform/templates/revoke.rs (cite line; expect ~89) calls `revoke_grants_by_descends_from_recursive` (NOT the single-hop variant).
7. The single-hop `revoke_grants_by_descends_from` IS UNCHANGED (verify via `git diff` — repo_impl_m2.rs:753 + in_memory.rs:1195 + the trait def at repository.rs:993 are unchanged in body); M2 narrow_mcp_tenants caller still uses the single-hop variant.
8. cargo test --workspace -j 4 green at expected count: 1426–1430 passed / 0 failed / 2 ignored. Sub-agents will mark this `NOT-EXECUTED-IN-AUDIT` — orchestrator closes at gate-4.
9. CI guards green: check-phi-core-reuse.sh exit 0; no new `use phi_core::` imports beyond §3 prediction (delta = 0).
10. Migration 0011 idempotent: `DEFINE FIELD descends_from_grant ON auth_request TYPE option<string>` at modules/crates/store/migrations/0011_authority_chain.surql.
11. K8s axes A1-A7 per plan §3.B verified: no new in-process state; new trait methods are object-safe; migration is single-column-add nullable.
12. Prior-chunk invariants intact (per plan §6): CH-08 round-trip; CH-13 audit-class composition; CH-11 approval_mode legacy decode.

PASS/FAIL each. Cite file:line for each PASS. ≤ 600 words.
```

### Audit B — concept + docs + ADR (≤ 600 words)

```
You are auditing CH-14's concept-fidelity + docs-fidelity. Read-only.

Verify each claim:
1. ADR-0053 Accepted at docs/specs/v0/implementation/m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md with sub-decisions D53.1–D53.7 named per plan §5.
2. ADR-0053 §"Forks" header captures the user-lock outcome correctly (per CH-13 v2 ADR-body checklist + CH-12 retro): if all .A locked → "F1–F5 user-locked at plan approval to F1.A / F2.A / F3.A / F4.A / F5.A — all align with planner recommendation"; if divergence → divergent header form.
3. ADR-0053 §"Cross-references" lists ALL 4 categories (concept-doc + closed drifts + prior ADRs cited + forward-scope row). Prior ADRs use milestone-prefixed paths per CH-08 retro Row 1: m3/decisions/0022-..., m5_2/decisions/0033-..., m5_2/decisions/0050-..., m5_2/decisions/0052-..., m5_2/decisions/0048-..., m5_2/decisions/0051-...
4. Drift D-new-14 Status = remediated; lifecycle entry "2026-05-08 — remediated — CH-14 chunk-seal — <evidence>" present.
5. Drift D-new-18 Status = remediated; lifecycle entry similarly present.
6. _concept-audit-matrix.md rows flipped letter-for-letter per CH-12 F-AUDB-1 rule:
   - Line 44: "Provenance — Chain traces to bootstrap" → honored.
   - Line 135: "Provenance traversal — Bootstrap axiom chain" → honored.
   - Line 157: "System Bootstrap Template + system:genesis" → honored.
   - Line 181: "Provenance chain to bootstrap" → honored.
   - Line 231: "Ad-hoc AR + revocation cascade" → honored.
7. Concept docs verified-header bumped (CH-14 amendment line per CH-12 P4 checklist v2026-05-03): permissions/02-auth-request.md (System Bootstrap Template § now backed by typed Rust); permissions/04-manifest-and-resolution.md (Authority Chain § now walkable); permissions/08-worked-example.md (§9.3 cascade now multi-hop). Body of concept docs UNCHANGED unless plan authorized.
8. New user-facing docs: m5_2/architecture/authority-chain.md (~250 words) + m5_2/operations/authority-chain-operations.md (~150 words) shipped per §3.C and §7 P3.
9. m1/architecture/audit-events.md verified-header amendment line documents the per-cascaded-AR `auth_request.revoked` emission cardinality change.
10. K8s deferred ledger UNCHANGED (no new CHK8S-D-NN entry — plan §3.B classifies all 7 axes no-impact / compatible).
11. Plan archive at docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md exists with cycle hex 5803bb94.
12. _cycle-index.md row added for CH-14.
13. Prior-chunk doc invariants intact: CH-13 audit-events.md amendment line preserved; CH-11 + CH-12 amendment lines preserved.

PASS/FAIL each. ≤ 600 words.
```

**Audit pass criteria.** Any new drift discovered → its own drift file before chunk seals. Audit-flagged concept contradiction → fixed in-chunk OR renegotiated with user (ADR-0053 amendment) OR converted to drift with explicit future-chunk assignment. Chunk seal blocked until both audits return clean on all 4 aspects + all audit-discovered drifts explicitly scoped.

---

## §12 — Verification section (end-to-end recipe)

```bash
# Replay CH-14's close verification. Use absolute paths per granular-bash discipline.

# 1. CI guards (each one logical operation per Bash invocation)
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (each one logical operation per Bash invocation)
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets -j 4
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4

# 3. Chunk-specific tests (each one logical operation per Bash invocation)
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p server --test acceptance_authority_chain -j 4
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p server --test acceptance_authority_templates -j 4
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p store --test migrations_0011_test -j 4

# 4. Chunk-specific greps (each one logical operation per Bash invocation)
git -C /root/projects/phi/baby-phi grep -nE 'system_genesis_principal\(\)' modules/crates/
# Expect: 4–9 hits (claim.rs:192,203 + 4 sites in repository_test.rs + 1 in template_e_props.rs + new axioms.rs internal).

git -C /root/projects/phi/baby-phi grep -nE '"system:genesis"' modules/crates/
# Expect: 4–6 hits (down from 11 — only docstrings + assertion-line literals remain).

git -C /root/projects/phi/baby-phi grep -nE 'descends_from_grant' modules/crates/
# Expect: ~30 hits (1 field def + ~28 cascade sites + walker/cascade impls).

git -C /root/projects/phi/baby-phi grep -nE 'revoke_grants_by_descends_from_recursive' modules/crates/
# Expect: 4–5 hits (1 trait def + 2 impls + 1 production callsite at templates/revoke.rs + tests).

git -C /root/projects/phi/baby-phi grep -nE 'walk_provenance_chain' modules/crates/
# Expect: 4–6 hits (trait + 2 impls + tests).

# 5. Drift-file status
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count> + 2 (D-new-14 + D-new-18).

# 6. Phi-core import baseline (CH-14 expects 0 delta)
grep -rn "use phi_core::" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 48.

# 7. Forbidden-duplication grep
grep -rnE 'pub (struct|enum) (Session|AgentEvent|AgentProfile|ExecutionLimits|ModelConfig|McpClient)\b' /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::"
# Expect: 0.
