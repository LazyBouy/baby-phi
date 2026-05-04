<!-- Last verified: 2026-05-04 by Claude Code (chunk-planner agent, iter 2 — F5.B locked by user; revised in-place) -->

# CH-12 — Frozen session-tag immutability enforcement

**Plan file token:** `6a748175` (generated 2026-05-04 at chunk-open via `openssl rand -hex 4`).
**Plan archive path (post-approval):** `baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md` (folder-style, multi-agent cycle; orchestrator + implementer create the cycle folder via the `chunk-archive-plan` skill, **not the planner**).
**Plan-mode draft path (this file):** `/root/.claude/plans/sharded-discovering-stearns.md`.
**Chunk ID:** CH-12 (forward-scope §1 lines 123–128; §1.4 "Frozen tags + audit" block).
**Severity:** ⚠ HIGH (security-boundary; closes the exfiltration vector documented in concept doc 05 §"Frozen-at-creation tags (immutability)" + drift `D-new-08`).
**Expected effort:** ~1.6 engineer-days (revised — F5.B audit-event variant adds ~0.1d on top of the original 1.5d estimate).
**Hard prerequisites:** **CH-05** (sealed — `validate_published_manifest` + `RESERVED_NAMESPACE_LITERALS` + `reserved_namespace_prefixes()` + Repository guard; ADR-0044 Accepted; D-new-07 + D-new-31 remediated). **CH-06** (sealed — instance-identity tag emission + selector grammar; ADR-0036 + ADR-0037 Accepted; D-new-03 + D-new-11 remediated). Both prereqs verified during planning (see §6).
**Chunks unblocked at close:** none directly; closes a security-boundary exfiltration vector (per forward-scope row line 128).

**Iteration 2 banner (planner re-spawn, 2026-05-04).** All 5 forks now user-locked: F1.A / F2.A / F3.C / F4.A / **F5.B** (overrides planner-recommendation F5.A). The F5.B lock adds an audit-event builder + emission-on-failure path. **Migration check verdict: NO migration required** (`audit_events` table at `migrations/0001_initial.surql` has `event_type TYPE string` (free-form) + `diff FLEXIBLE TYPE object` + `audit_class` already accepts `"alerted"` — additive event types are schema-stable). **Auto-approval criteria still all hold** (see §"Auto-approval re-check" below). See revised §3.B A7 row + §5 sub-decision D49.7 + §7 P1+P2 deliverables + §8 test-count update + §10 close-criteria expansion + §11 Audit A claim list expansion + §12 verification recipe expansion.

---

## Forks for orchestrator

All five forks are **user-locked at iter 2** (orchestrator: 4/5 match planner recommendation; F5 diverges).

### F1 — Where does the publish-time half of CH-12 land structurally?

**Question.** CH-05 already ships `validate_published_manifest`, Rule C ("Reserved-namespace write rejection") which rejects `[Modify]` on the **bare `tag`** fundamental. But Rule C does NOT trigger on `[Modify]` against a **composite resource** (e.g., `session_object`) because `session_object`'s constituents include `Tag` and `Action::Modify.applies_to_composite(SessionObject) == true` (Mutation category × Tag fundamental — `action.rs:336`). The drift D-new-08's "publish-time half" is therefore **NOT yet covered by CH-05's Rule C** — a tool can declare `actions: [modify], resource: ["session_object"]` and ship.

CH-12 must extend the validator to detect the structural-tag-write case for composite resources. Three options:

- **F1.A — New rule (USER-LOCKED).** Add a new `ValidationError::CompositeStructuralTagWrite { composite, action, namespace }` variant + a new "Rule E" pass that triggers when `actions ∋ Modify AND resource ∪ transitive contains a composite C AND target_kinds contains a name overlapping the reserved-namespace prefix list`. Net: extend Rule C from "bare tag" to "bare tag OR composite-via-target_kinds".
  - *Pros:* Defense-in-depth on top of CH-05; surfaces a typed error variant; keeps validator structure linear.
  - *Cons:* +1 ValidationError variant + +1 cells of test surface (8 composites × Modify); requires a pattern-match decision on `target_kinds` semantics.
- **F1.B — Strict reading: reject `[modify]` on any composite whose Tag-namespace is reserved.** Rule C extended to fire whenever `actions ∋ Modify AND resource ∪ transitive contains any composite name (because every composite includes `Tag` per `composites.rs::every_composite_includes_tag_fundamental`)`. Concept doc 05 says "no grant template issues `[modify]` on the structural tags of `session_object`" — this is the strict reading.
  - *Pros:* Maximally aligned with concept doc 05 line 541. Single rule extension, no new variant.
  - *Cons:* Breaks the Memory contract — `Action::Modify.applies_to_composite(MemoryObject) == true` is asserted at `action.rs:766` (existing test). Memory tags ARE intentionally agent-mutable per concept doc 05 (Memory tags are chosen by the agent at creation). Strict reading would prevent legitimate memory_object [modify] grants, requiring a parallel exception list — which collapses into option F1.A's shape via the back door.
- **F1.C — Defer publish-half entirely; ship only runtime-half.** Land the runtime gate (the engine Step body that rejects retag) but treat the publish-time half as already covered by CH-05's Rule C, on the strict reading that "tools should declare the bare `tag` fundamental for retag operations, never a composite". The validator stays unchanged.
  - *Pros:* Smallest possible scope; halves the cascade.
  - *Cons:* Forward-scope row 127 explicitly says "**reserved-namespace validator rejects `[modify]` on reserved tags at publish**". F1.C would ship CH-12 without honouring half its forward-scope deliverable. **Incompatible with HIGH severity.**

**Lock: F1.A** (matches planner recommendation).

### F2 — Where does the runtime gate live?

**Question.** Drift D-new-08 says "would live at permission-check Step 4 or a new validator hook." The drift name is "Step 4 runtime gate" but the engine's existing Step 4 is **Constraint satisfaction** (`engine.rs:387`). There is no native "tag-write" Step in the formal algorithm.

- **F2.A — New validator function `validate_tag_write_on_session(...)` invoked at Repository boundary (USER-LOCKED).** A pure-fn `fn validate_tag_write_on_session(session_id: SessionId, current_tags: &[String], proposed_tags: &[String]) -> Result<(), FrozenTagViolation>` lives at `domain::permissions::manifest::validator` (alongside `validate_published_manifest`). Called by **future** Repository methods that propose to update session tags. *Today there is NO `update_session_tags` / `set_session_tags` / `retag_session` callsite (verified via `grep -rn "set_tags\|update_tags\|retag_session" modules/crates/`). The function is forward-defensive.*
  - *Pros:* Matches CH-05's pattern (publish-time validator + Repository guard). Pure-fn → trivially testable; no engine surgery; no new `FailedStep` variant; no new `DeniedReason` variant.
  - *Cons:* Defensive only — there's nothing in code today that calls it. The acceptance value is "if a future chunk adds `update_session_tags`, the validator is already wired into the Repository trait method's contract".
- **F2.B — New engine Step body.** (Rejected; would require concept-doc amendment + new `FailedStep`/`DeniedReason` variants for a code path that has zero callsites today.)
- **F2.C — Both.** (Rejected; duplicates without value.)

**Lock: F2.A** (matches planner recommendation).

### F3 — Scope of "structural tags" — full concept-doc tag set OR only auto-emitted prefix list?

**Question.** Concept doc 05 §"Tag Vocabulary for Sessions" (lines 220–231) lists 8 tag categories: `agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`, plus `#archived`/`#active`. Today, `Session.tags` is populated only by `composites::auto_tags_for("session", ...)` (`launch.rs:336`, `session_recorder.rs:275`) which emits **2 tags**: `#kind:session` + `session:<id>`. The other 6 namespaces are concept-aspirational.

- **F3.A — Only the reserved-namespace prefixes already shipped by CH-05's `reserved_namespace_prefixes()`.** (Rejected; doesn't surface the M5/M6 emission gap.)
- **F3.B — Full concept-doc 05 set.** (Rejected; misses the F3.A enforcement-today axis.)
- **F3.C — F3.A enforcement today + F3.B-shape `SESSION_FROZEN_TAG_PREFIXES` constant for M6+ (USER-LOCKED).** Ship F3.A's enforcement today, but place the full F3.B prefix list as a `pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` in the validator module, with a comment that says *"All session-tag prefixes that are concept-doc-frozen at creation. CH-12's runtime gate enforces every prefix in this list; the 6 M6+ categories are forward-defensive (not yet emitted on Session.tags creation; emission expansion is a separate forward-scope item)."*

**Lock: F3.C** (matches planner recommendation).

### F4 — ADR shape — single ADR-0049 or ADR-0049 + ADR-0050?

**Question.** Two distinct decisions: (a) the publish-time validator extension (Rule E / new `CompositeStructuralTagWrite` variant), and (b) the runtime-side `validate_tag_write_on_session` function. With F5.B locked, also (c) the audit-event builder. Are they one ADR or multiple?

- **F4.A — Single ADR-0049 (USER-LOCKED).** All decisions close drift D-new-08; they share concept-doc anchors. One ADR with sub-decisions D49.1–D49.7 (D49.7 is the new F5.B audit-event sub-decision) mirrors CH-05's ADR-0044 shape.
- **F4.B — Two ADRs.** (Rejected.)

**Lock: F4.A** (matches planner recommendation; ADR now carries 7 sub-decisions instead of 6 to absorb the F5.B-driven D49.7).

### F5 — Audit emission for rejection events

**Question.** When the validator rejects a manifest at publish, today no audit event is emitted (CH-05's pattern: validation errors map to HTTP 422; no audit). When the future runtime gate rejects a retag, should it emit an audit event?

- **F5.A — No audit emission for rejections.** Validator errors propagate up as `Result<_, FrozenTagViolation>`; caller maps to HTTP 422 / CLI error. No audit hash chain bytes change.
  - *Planner recommended F5.A.*
- **F5.B — Emit `tool.frozen_tag_write_rejected` audit event (USER-LOCKED — overrides planner recommendation).** New audit-event builder + `AuditClass::Alerted` (security event class). The builder is wired into the documented Repository trait contract: any future tag-write entry point that calls `validate_tag_write_on_session` and gets `Err(FrozenTagViolation)` MUST call the builder + emit the event before propagating the error.
  - *Pros:* Security operators see the attack attempt in the audit log. Aligned with CH-23 system-agents pattern (Alerted-class for security events).
  - *Cons:* +1 audit-event builder + +tests; the planner originally cited "+1 migration column for audit_event family" but **on investigation that cost is zero** (see "User lock rationale" below + revised §3.B A7).

**Lock: F5.B (USER OVERRIDE).**

#### User lock rationale (added iter 2)

User prioritises operator-visibility into retag-attempts in the audit log even before any production tag-write callsite exists. The user is willing to absorb the additional scope cost. **The investigated scope cost is materially smaller than the planner's iter-1 estimate**:

1. **No migration required.** Confirmed at `modules/crates/store/migrations/0001_initial.surql:` — `audit_events` table is schema-stable: `event_type TYPE string` (free-form, not enum-bound), `diff FLEXIBLE TYPE object` (free-form JSON), `audit_class TYPE string ASSERT $value INSIDE ["silent", "logged", "alerted"]` (so `Alerted` already valid). The migration count stays at **13** (unchanged from post-CH-11 baseline). **Auto-approval criterion "no new migration" remains intact.**
2. **`AuditClass::Alerted` already exists** at `modules/crates/domain/src/audit/mod.rs:39` (as a typed `enum AuditClass` variant with `serde(rename_all = "snake_case")`). Used by 12 existing emitters across `m4/agents.rs`, `templates/`, `system_agents/`. No new class introduced.
3. **Builder pattern is well-established.** Precedent emitters (e.g., `m5/consents.rs`, `m4/agents.rs`, `m5_2/memory.rs`) follow the same shape: `pub fn <event>(actor, target, org, ..., at) -> AuditEvent { ... event_type: "<dotted_name>".to_string(), diff: serde_json::json!({...}), audit_class: AuditClass::<...>, ... }`. CH-12's emitter mirrors this 1:1.
4. **No emission callsite today.** F5.B-locked-now ships the builder + tests + the contract docstring at the Repository trait. The actual `audit.emit(...)` call wires when a future chunk adds `update_session_tags` (Option B in investigation tasks; see §5 D49.7). This honours the user's intent (operators see retag-attempts) at the future entry point — exactly when there's something to audit. Wiring the emit-call inside `validate_tag_write_on_session` itself (Option C) was investigated and rejected: it would couple a pure-fn validator to async audit emission + an `AuditEmitter` dependency injection — violating the validator's pure-fn discipline and breaking the test ergonomics that motivated F2.A.
5. **Hash chain stays byte-stable.** `AuditEvent::canonical_bytes()` (`audit/mod.rs:72`) explicitly excludes `prev_event_hash`; existing recorded events' canonical_bytes are unaffected by the addition of a new `event_type` string. Verified by inspection: an additive variant in the dotted-string namespace does not perturb prior events' bytes. New events at any pod simply produce a new chain link with the new event_type — chain symmetry holds across pods.

---

## Context

### The simple version

Concept doc 05 §"Frozen-at-creation tags (immutability)" specifies a **security boundary**: session structural tags (`#kind:session`, `session:<id>`, plus the M6+ planned set `agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`) are immutable post-creation. A tool that declares `[modify]` on session tags can defeat the multi-scope cascade (e.g., retag session A from `org:X` to `org:Y` to gain access via Y's lead's Template A grant). The concept doc says (line 525): *"there's no mutation grant for the structural tags, so attempts to retag are denied at the permission layer, not at a separate 'validation' step."*

CH-05 closed half this story: `validate_published_manifest` Rule C rejects `[Modify]` on the **bare `tag`** fundamental. But Rule C does NOT trigger when a manifest declares `[Modify]` on the **`session_object`** composite (which internally includes Tag) — and `Action::Modify.applies_to_composite(SessionObject) == true` is asserted at `action.rs` lines 766–777 (composite-via-constituent-union). So the publish-time guard is incomplete.

CH-12 closes drift D-new-08 by:
1. **Publish-time half** — extend `validate_published_manifest` with **Rule E**: reject `[Modify]` on a composite whose `target_kinds` overlaps the reserved-namespace prefix list (per F1.A locked).
2. **Runtime half** — ship a forward-defensive validator `validate_tag_write_on_session(...)` at `domain::permissions::manifest::validator`. Today no callsite exists (verified: zero hits for `set_tags|update_tags|retag` across `modules/crates/`); the function is the precondition gate any future tag-write Repository method MUST call. Wired into the Repository trait module-level docstring as a precondition note.
3. **Audit-event half (F5.B locked iter 2)** — ship the `tool.frozen_tag_write_rejected` audit-event builder + `AuditClass::Alerted`, wired into the Repository trait contract docstring. Builder lives at `domain::audit::events::m5_2/tool_authority.rs` (NEW module). Today no production callsite emits the event; the builder is forward-defensive symmetric to F2.A's validator.
4. **Acceptance tests** for all three halves (publish-time rejection + runtime rejection + audit-event-builder shape).

The chunk closes drift D-new-08 (HIGH; security-boundary). No new K8s blockers (A7 re-reviewed for F5.B; verdict still no-blocker — see §3.B). Zero phi-core leverage delta.

### What this chunk does NOT do

- Does NOT add an `update_session_tags` / `retag_session` HTTP or CLI endpoint. There is no production retag flow today (verified via grep; zero hits). The runtime gate + audit-event emission are the precondition for any future flow.
- Does NOT extend the session-tag emission to cover the 6 concept-aspirational categories (`agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:`). That gap belongs to a separate forward-scope item (M6+ "Session structural-tag emission gap"); CH-12 documents the gap via the `SESSION_FROZEN_TAG_PREFIXES` constant per F3.C but does not close the gap.
- ~~Does NOT emit audit events on rejection (F5.A locked).~~ **REVISED iter 2: F5.B locked → CH-12 ships the audit-event builder + contract.** The actual `audit.emit(...)` callsite wires when a future chunk adds `update_session_tags`; CH-12 ships the builder, tests, and Repository trait contract docstring (mirrors the F2.A forward-defensive pattern).
- Does NOT change the engine Step ordering or add a new `FailedStep` variant (F2.A locked). The runtime gate is a Repository-boundary validator, not an engine Step.
- Does NOT touch `Memory.tags` lifecycle. Memory tags are intentionally agent-mutable per concept doc 05 lines 24–26; CH-12's enforcement is session-specific.
- Does NOT add a SurrealDB migration. The audit-events table schema is already string-typed for `event_type` and FLEXIBLE-typed for `diff` (`migrations/0001_initial.surql`), so additive event types are schema-stable. Migration count stays at **13** (post-CH-11 baseline).

### Forward-scope reference

[CH-12 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 123–128) + §1.4 "Frozen tags + audit" block (lines 121–128).

### Concept-doc anchor

- [`concepts/permissions/05-memory-sessions.md`](baby-phi/docs/specs/v0/concepts/permissions/05-memory-sessions.md) §"Tag Vocabulary for Sessions" (lines 220–231) + §"Frozen-at-creation tags (immutability)" (lines 531–541) + Example 7 (lines 516–525).
- [`concepts/permissions/01-resource-ontology.md`](baby-phi/docs/specs/v0/concepts/permissions/01-resource-ontology.md) §"Reserved tag namespaces" (lines 254–261) + §"Implication for tool authoring" rule 1 (line 267).
- [`concepts/permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) §"Reserved Namespace Enforcement" (lines 190–196).
- [`concepts/permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) §"Manifest Validation at Publish Time".

---

## §1 — Why this chunk

CH-05 closed the publish-time validator for the **bare `tag` fundamental** but a manifest declaring `actions: [Modify], resource: ["session_object"], target_kinds: ["session"]` still ships cleanly because `Action::Modify` applies to `SessionObject` via constituent union (Mutation × Tag is true; concept-doc-grounded — see `action.rs:336`). This means a malicious/buggy tool can declare modify-permission on the very namespaces that gate multi-scope read access (`org:`, `project:`, `session:`, etc. as defined in concept doc 05 §"Tag Vocabulary for Sessions"). Concept doc 05 line 541 says explicitly: *"No grant template issues `[modify]` on the structural tags of `session_object`. The only mutable tags are lifecycle (`#archived`/`#active`)..."* CH-12 closes the gap by extending the validator with **Rule E** (rejects `[modify]` on a composite when its `target_kinds` names a reserved namespace), ships a forward-defensive runtime validator function `validate_tag_write_on_session` so any future tag-write Repository method has a precondition gate already in place, and (per F5.B lock) ships a `tool.frozen_tag_write_rejected` audit-event builder so operator audit logs surface attack attempts when the future entry point wires up. The chunk's deliverables map directly to forward-scope row 127's two clauses ("publish at validator", "runtime gate at retag attempts") + acceptance tests covering all three paths.

**Quality-over-speed restatement.** *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* CH-12's specific application: every assertion below cites a verbatim concept-doc line, every fork is recommendation-locked above (orchestrator + user finalised), and the deferred items (full session-tag emission for the 6 M6+ categories; HTTP retag endpoint; audit-event-emission wiring at a future tag-write callsite) ship with explicit successor-chunk references — not silent gaps.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/05-memory-sessions.md`](baby-phi/docs/specs/v0/concepts/permissions/05-memory-sessions.md) | §"Tag Vocabulary for Sessions" lines 220–231 | Session structural tags (`agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`, `#kind:session`, `session:<id>`) are frozen-at-creation; only `#archived`/`#active` are mutable | partially-honored (only `#kind:session` + `session:<id>` actually emitted today; the 6 stated categories are aspirational; no mutation rejection exists for any of them) | partially-honored → still partially honored at the **emission** axis (M6+ closes that), but **immutability enforcement** axis flips to honored: the runtime gate + publish-time Rule E reject mutations on whatever subset is in `SESSION_FROZEN_TAG_PREFIXES` |
| `permissions/05-memory-sessions.md` | §"Frozen-at-creation tags (immutability)" lines 531–541 | "No grant template issues `[modify]` on the structural tags of `session_object`. The only mutable tags are lifecycle (`#archived`/`#active`)..." | contradicted (CH-05 closes only bare `tag` fundamental; composite-resource case is open) | honored — Rule E extends `validate_published_manifest` to reject `[Modify] × composite × reserved-namespace-target_kinds` |
| `permissions/05-memory-sessions.md` | Example 7 lines 516–525 | "A worker tries to retag... Denied. The request never reaches the storage layer" | silent-in-code (no Repository method exists today; no rejection mechanism) | honored — `validate_tag_write_on_session(...)` ships at the validator module + RepositoryError variant ready + `tool.frozen_tag_write_rejected` audit-event builder ships per F5.B; future tag-write Repository method MUST call all three as precondition |
| [`permissions/01-resource-ontology.md`](baby-phi/docs/specs/v0/concepts/permissions/01-resource-ontology.md) | §"Reserved tag namespaces" lines 254–261 + rule 1 line 267 | "Tools that create composite instances must **not** declare `[modify]` actions on the reserved tag namespaces — the runtime assigns them at creation" | partially-honored (CH-05's Rule C closes the bare-tag case; composite-via-target_kinds case open) | honored — Rule E closes the composite-via-target_kinds case |
| [`permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) | §"Reserved Namespace Enforcement" lines 190–196 | "publish-time manifest validator is what rejects reserved tags in tool manifests' `actions: [modify]` declarations" | partially-honored (CH-05 closes bare-tag) | honored — validator now closes composite-via-target_kinds |
| [`permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) | §"Manifest Validation at Publish Time" | Publish-time validator is the gate for manifest correctness | honored (CH-05 lifted into typed Rust); CH-12 extends Rule C → Rule C + Rule E | honored (extended; verified-header bump only — body unchanged) |
| [`permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) | (entry invariants) | Permissions subtree invariants | honored | honored (re-verified post-validator-extension) |
| [`concepts/phi-core-mapping.md`](baby-phi/docs/specs/v0/concepts/phi-core-mapping.md) | (phi-core surfaces) | phi-core has no manifest-validator / tag-immutability concept | honored | honored (unchanged — CH-12 stays in baby-phi domain) |

**Coverage check.** Every concept doc whose claims this chunk's code touches is listed. `permissions/README.md` cited per the per-chunk-template's "Permissions subtree hook". `concepts/phi-core-mapping.md` cited per the "phi-core-mapping hook". The §"Tag Vocabulary for Sessions" row at row 1 explicitly stays at `partially-honored` post-chunk because the **emission** half (the 6 M6+ categories that aren't auto-emitted today) is out of scope; only the **immutability enforcement** half flips. Documented as a non-blocking carry-forward in §10 close criteria.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Rationale.** CH-12 lands wholly inside the governance layer:
- `domain::permissions::manifest::validator` — extension of CH-05's pure-fn validator (no I/O, no async).
- `domain::repository` — `RepositoryError::FrozenSessionTagWrite` variant addition (additive).
- `domain::permissions::mod` — re-export of new public symbols.
- `domain::audit::events::m5_2/tool_authority.rs` — NEW module containing the `tool.frozen_tag_write_rejected` builder (per F5.B). Reuses existing `AuditEvent` + `AuditClass::Alerted` types. **No new audit-framework types.**

phi-core has no concept of tool authority manifests, reserved-namespace policy, session-tag immutability, or governance audit events (`docs/specs/v0/concepts/phi-core-mapping.md` confirms; `domain::audit::AuditEvent` is intentionally phi-only per `baby-phi/CLAUDE.md` §"Orthogonal surfaces"). Zero `use phi_core::` imports added or removed.

**Expected import-count delta at chunk close: 0.** Baseline post-CH-11: **48** `use phi_core::` imports across `modules/crates/` (verified at planning: `grep -rn "use phi_core::" modules/crates/ | wc -l == 48`). Post-CH-12: still 48.

**Positive close-audit greps:**
```bash
# Validator extension exists
grep -n "fn validate_tag_write_on_session\b" modules/crates/domain/src/permissions/manifest/validator.rs   # 1
grep -n "CompositeStructuralTagWrite\b" modules/crates/domain/src/permissions/manifest/validator.rs       # >= 2 (variant decl + match arm)
grep -n "SESSION_FROZEN_TAG_PREFIXES\b" modules/crates/domain/src/permissions/manifest/validator.rs       # >= 1 (per F3.C)
grep -n "FrozenSessionTagWrite\b" modules/crates/domain/src/repository.rs                                  # >= 1 (RepositoryError variant)
grep -rn "validate_tag_write_on_session\|CompositeStructuralTagWrite" modules/crates/server/tests/         # >= 1 (acceptance test file refs)

# F5.B audit-event builder
grep -n "fn frozen_tag_write_rejected\b" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # 1 (builder fn)
grep -n "tool.frozen_tag_write_rejected" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # >= 1 (event_type literal)
grep -n "AuditClass::Alerted" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs              # >= 1 (class)
grep -n "pub mod tool_authority" modules/crates/domain/src/audit/events/m5_2/mod.rs                       # 1 (module wired)

# phi-core baseline unchanged
grep -rn "use phi_core::" modules/crates/ | wc -l                                                          # 48 (unchanged)
grep -rn "use phi_core::" modules/crates/domain/src/permissions/                                           # 0 (unchanged)
grep -rn "use phi_core::" modules/crates/domain/src/audit/                                                 # 0 (unchanged)
```

**Forbidden-duplication greps (must return 0):**
```bash
grep -rn "^pub struct.*FrozenTag\|^pub enum.*FrozenTag" modules/crates/ | grep -v "permissions/manifest/validator.rs\|repository.rs"   # 0
grep -rn "use phi_core::permissions\|use phi_core::manifest" modules/crates/                                                            # 0 (sanity-check guard)
grep -rn "^pub struct AuditEvent\|^pub enum AuditEvent" modules/crates/ | grep -v "domain/src/audit/mod.rs"                              # 0 (no parallel audit-event type)
```

`scripts/check-phi-core-reuse.sh` MUST stay green at chunk close. Per `baby-phi/CLAUDE.md` §"phi-core Leverage" rules 1–5: zero overlap, intentionally phi-only — same as CH-05, CH-06, CH-09, CH-10, CH-11.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | None. Validator extension is pure-fn (no `DashMap` / `RwLock` / etc.). `SESSION_FROZEN_TAG_PREFIXES` is a `&'static [&'static str]` constant. The audit-event builder is a pure-fn returning `AuditEvent`; no in-process state added. | No | — |
| **A2** | New IPC channel | None. Pure-fn extensions; no `mpsc`/`broadcast`/`watch`. | No | — |
| **A3** | New pod-local resource | None. No file handles, sockets, sub-processes. | No | — |
| **A4** | Migration runner / first-apply race | **No migration.** Verified at `modules/crates/store/migrations/0001_initial.surql:` — `audit_events` table is schema-stable: `event_type TYPE string` (free-form, not enum-bound), `diff FLEXIBLE TYPE object` (free-form JSON), `audit_class TYPE string ASSERT $value INSIDE ["silent", "logged", "alerted"]`. Adding a new event-type string + JSON diff shape requires **zero schema changes**. `RepositoryError::FrozenSessionTagWrite` variant is purely Rust-typed; no SurrealDB column. Wire format unchanged (`ToolAuthorityManifest` shape unmodified). Migration count stays at **13** (post-CH-11 baseline). | No | — |
| **A5** | Trait-shape requirement | `RepositoryError` gains one variant (`FrozenSessionTagWrite { ... }`). Repository trait method **signatures** unchanged; the variant is additive on the error enum (CH-05 ADR-0044 §D44.8 precedent). No remote-backend dependency in the validator function (pure-fn). The audit-event builder (`fn frozen_tag_write_rejected(...) -> AuditEvent`) is a pure-fn at module level; not a trait method. | No | — |
| **A6** | Cross-pod state sharing | Wire format unchanged → cross-pod gossip is byte-for-byte identical. No new data persisted as part of CH-12 (the audit-event variant is buildable at any pod but emission is forward-deferred to a future tag-write callsite). | No | — |
| **A7** | Audit hash-chain symmetry | **REVISED iter 2 — F5.B locked.** F5.B adds a new `event_type` string (`"tool.frozen_tag_write_rejected"`) + `AuditClass::Alerted` (already existed at `audit/mod.rs:39`). **Hash chain stays byte-stable** because: (a) `AuditEvent::canonical_bytes()` at `audit/mod.rs:72` excludes `prev_event_hash`, so existing recorded events' canonical bytes are unaffected; (b) new events at any pod produce a chain link that includes the new event_type string in their own canonical bytes — this is a per-event property, not a global namespace; (c) the `event_type` field is a free-form `String` in `AuditEvent` (not an enum), so additive event types do not perturb prior serialised events; (d) the audit_emitter chain-link insertion logic (`store/src/audit_emitter.rs`) reads `prev_event_hash` from the last persisted event and copies it into the new event — additive event types do not change this loop. **No new K8s blocker class.** Cross-pod symmetric: pod A and pod B both emit chain links with the same canonical_bytes shape; whichever pod processes the event first sets the chain head; the other pods' chain heads converge per the existing CH-21 / ADR-0040 hash-chain semantics. | No | — |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — chunk does not touch the registry. N/A.
- D33.2 (`SurrealStore::open_remote`) — no migration, no storage operations. N/A.
- D33.3 (SIGTERM graceful shutdown) — chunk adds zero `tokio::spawn` tasks. N/A.
- D33.4 (`EventBus.shutdown` + `drain`) — chunk adds zero `EventBus` emitters or listeners. N/A.

**Conclusion.** **K8s-neutral.** No new K8s blockers. No `CHK8S-D-NN` ledger entry needed. (F5.B lock investigated; A7 cleared with cited evidence per the iter-2 axis review.)

**Mid-flight discovery rule.** If a phase surfaces a K8s blocker not anticipated here (e.g., a deeper audit-emitter byte-instability surfaces during the hash-chain byte-stability acceptance test in §11 Audit A claim 21), pause via `AskUserQuestion`, file a new `CHK8S-D-10` (next free) entry, then resume.

---

## §3.C — User-facing documentation impact map

| Tier | File | This chunk touches? | Action |
|---|---|---|---|
| **Concept** | [`permissions/05-memory-sessions.md`](baby-phi/docs/specs/v0/concepts/permissions/05-memory-sessions.md) | Yes — verified-header bump only | Update in-chunk: prepend a `<!-- Last verified: 2026-MM-DD by Claude Code (CH-12 amendment: §"Frozen-at-creation tags (immutability)" lines 531–541 + Example 7 lines 516–525 lifted into typed Rust at `domain::permissions::manifest::validator::{validate_tag_write_on_session, CompositeStructuralTagWrite, SESSION_FROZEN_TAG_PREFIXES}` + `domain::audit::events::m5_2::tool_authority::frozen_tag_write_rejected` (Alerted-class). Publish-time validator extends CH-05 Rule C to a new Rule E: reject `[Modify]` on composites whose `target_kinds` overlaps the reserved-namespace prefix list. Runtime gate is a forward-defensive validator the Repository contract requires future tag-write methods to call; rejection produces an audit event per F5.B. Doc body unchanged.) -->`. **Doc body UNCHANGED**. |
| **Concept** | [`permissions/01-resource-ontology.md`](baby-phi/docs/specs/v0/concepts/permissions/01-resource-ontology.md) | Yes — verified-header bump only | Verified-header touch noting CH-12 closes the rule-1 enforcement gap (line 267) for composites. Body unchanged. |
| **Concept** | [`permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) | Yes — verified-header bump only | Verified-header touch noting CH-12 extends CH-05's reserved-namespace enforcement to composites via target_kinds. Body unchanged. |
| **Concept** | [`permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) | Yes — verified-header bump only | Verified-header touch noting CH-12 extends CH-05's `validate_published_manifest` rule set from 4 rules + 3 warnings (Rule A/B/C/D) to 5 rules + 3 warnings (adds Rule E). Body unchanged. |
| **Concept** | [`permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) | No | N/A — no body change. |
| **Architecture** | (search) `m1/architecture/permission-check-engine.md` if exists | No (engine is unchanged; CH-12 lives at the validator module + Repository boundary, not the engine). | If grep finds a stale claim about "manifest validator covers Rule A/B/C/D", verified-header bump to "A/B/C/D/E" is the only update. Otherwise N/A. |
| **Architecture** | (search) `m1/architecture/audit-events.md` if exists — F5.B touches the audit-event family | Yes (likely) — new event type registered | Verified-header bump only. Body update if a stale "audit event types" enumeration is present (some milestone-architecture docs catalogue the dotted-name set; if CH-12's `tool.frozen_tag_write_rejected` is missing from such a list, that's a body update). Implementer verifies during P1; if list update needed, lands at P3 paperwork. |
| **Operations** | (search) `m5*/operations/permission-engine-operations.md` if exists | Possibly — new `RepositoryError::FrozenSessionTagWrite` is operator-visible if it propagates to HTTP responses; new audit event surfaces in alert channels per `AuditClass::Alerted` retention | Verified during P1: if file exists with stale error-code list, add `FrozenSessionTagWrite → HTTP 422` row + `tool.frozen_tag_write_rejected → Alerted, 60s alert delivery` row. Otherwise "no change". |
| **User-guide** | `m5*/user-guide/sessions-walkthrough.md` if exists | No — there is no operator-visible retag flow today. CH-12 is forward-defensive only. | N/A. If a future chunk wires `update_session_tags` HTTP endpoint, that chunk's §3.C covers the walkthrough. |
| **Decision** | `m5_2/decisions/0049-frozen-session-tag-immutability.md` (NEW) | Yes — the ADR for this chunk | Create + Accepted at chunk seal (§5). |
| **Drift** | `m5_1/drifts/D-new-08.md` | Yes — close | Lifecycle entry `discovered → in-chunk-plan → remediated`. |
| **Drift index** | `m5_1/drifts/README.md` | Yes | Flip D-new-08 row "Closes at" → CH-12 ✓; status → remediated. |
| **Concept-audit matrix** | `m5_1/drifts/_concept-audit-matrix.md` | Yes | Flip every "frozen-at-creation" / "session-tag immutability" / "reserved-namespace write rejection (composite case)" row from `partially-honored` / `contradicted` / `silent-in-code` to `honored`. |

**Defer decisions.** Two defers, both with successor references:

1. **Full session-tag emission (the 6 concept-aspirational categories at concept doc 05 lines 220–231)** → **deferred to M6+ "Session structural-tag emission" follow-up** (no chunk allocated yet in forward-scope; see §10 carry-forward gap). Reason: CH-12 enforces immutability on whatever is currently emitted (`#kind:session` + `session:<id>`) plus all reserved namespaces; emitting `agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:` on session creation is a separate axis. CH-12 is the IMMUTABILITY chunk; tag-emission completeness is its own work item.

2. **Operator-facing retag HTTP/CLI endpoint** → deferred to a future M6+ tool-admin chunk (no current allocation). Reason: there is no production retag flow today (verified via grep). CH-12's runtime gate + audit-event builder are forward-defensive; an HTTP endpoint adoption would carry its own §3.C (operator walkthrough; the audit-event already exists per F5.B so that future chunk only needs to wire the `audit.emit(...)` callsite).

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D-new-08** | [`m5_1/drifts/D-new-08.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-08.md) | HIGH | `discovered → in-chunk-plan → remediated` | Closes the security-boundary exfiltration vector documented in concept doc 05 §"Frozen-at-creation tags (immutability)". |

**Index updates:**
- `drifts/README.md` — D-new-08 row: Status `discovered → remediated`; "Closes at" → `CH-12 ✓`.
- `_concept-audit-matrix.md` — every row touching "Frozen-at-creation tags" / "Session-tag immutability" / "Reserved-namespace write rejection (composite case)" updated to `honored`.

**No new drifts mid-chunk** anticipated. If audit phase surfaces one, the lifecycle rules in `drift-lifecycle.md` apply: new drift file created BEFORE chunk seal.

**Carry-forward note.** The §"Tag Vocabulary for Sessions" emission gap (the 6 M6+ categories) is NOT closed by CH-12; it stays as concept-doc partially-honored. A new LOW drift `D-CH12-FOLLOWUP-01` (or accept under a future M6+ "Session structural-tag emission" chunk) is recommended at retrospective time — same pattern as CH-11's `D-CH11-FOLLOWUP-01` for the project.deadline_at gap. Logged here for retrospective consideration; not blocking CH-12 close.

---

## §5 — ADRs drafted

ADR numbering: highest accepted = ADR-0048 (CH-11). Next-free = **ADR-0049**.

(Verified at draft time: `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE "^[0-9]+" | sort -u | tail -5` → `0044, 0045, 0046, 0047, 0048` → next-free = 0049.)

| ADR | Title | Drafted at phase | Decision summary | Flip to Accepted at |
|---|---|---|---|---|
| **ADR-0049** | Frozen session-tag immutability enforcement (publish-time Rule E + runtime validator + audit-event builder) | P1 (Proposed) | See sub-decisions D49.1–D49.7 below. | P3 chunk seal |

**Sub-decisions (drafted as `Proposed` at P1; flipped to `Accepted` at P3 chunk seal):**

- **D49.1 — Publish-time Rule E** (per Fork F1.A locked). Extend `validate_published_manifest` with a new rule fired after Rule C: when `manifest.actions ∋ Action::Modify AND manifest.resource ∪ manifest.transitive contains a composite C (excluding MemoryObject — see D49.1.a) AND any entry of manifest.target_kinds matches a reserved-namespace prefix from reserved_namespace_prefixes()` (CH-05's existing helper), return `Err(ValidationError::CompositeStructuralTagWrite { composite: C, action: Modify, namespace: <matched-prefix> })`. Pure rule; no I/O. Composes with Rule C — Rule C still fires for `[Modify] × bare tag`; Rule E covers the composite-via-target_kinds case. **D49.1.a — Memory-vs-Session discrimination.** Rule E is exempt for `Composite::MemoryObject` because Memory tags ARE intentionally agent-mutable per concept doc 05 lines 24–26 (Memory tags are chosen by the agent at creation; `Action::Modify.applies_to_composite(MemoryObject) == true` is asserted at `action.rs:766`).
- **D49.2 — `ValidationError::CompositeStructuralTagWrite` variant** (new). Carries `{ composite: Composite, action: Action, namespace: String }`. Derives `Debug, Clone, PartialEq, Eq`; impls `Display + Error` via `thiserror::Error`. Enum is now 7 variants (CH-05 shipped 6 per ADR-0044 §D44.2); additive — pre-CH-12 callers matching exhaustively need a new arm (verified at P1 build time; not expected to break callers because most callers map any `ValidationError → HTTP 422` with `Display`).
- **D49.3 — Runtime validator function** (per Fork F2.A locked). New pure-fn `pub fn validate_tag_write_on_session(session_id: SessionId, current_tags: &[String], proposed_tags: &[String]) -> Result<(), FrozenTagViolation>` at `domain::permissions::manifest::validator`. Logic: for each tag in `proposed_tags` whose prefix matches any `SESSION_FROZEN_TAG_PREFIXES` entry, the tag must appear in `current_tags` (unchanged tags pass). For each tag in `current_tags` whose prefix matches `SESSION_FROZEN_TAG_PREFIXES`, the tag must appear in `proposed_tags` (cannot be removed). Lifecycle tags (`#archived`, `#active`) are unconstrained. Returns `Ok(())` on success; `Err(FrozenTagViolation { session_id, attempted_change: TagChange })` on failure where `TagChange = Added(String) | Removed(String)`.
- **D49.4 — `FrozenTagViolation` error type** (new). Lives at `domain::permissions::manifest::validator`. Variants: `FrozenTagAdded { session_id, tag }` + `FrozenTagRemoved { session_id, tag }`. Derives `Debug, Clone, PartialEq, Eq`; impls `Display + Error`. Exposed for HTTP 422 mapping by future tag-write Repository methods.
- **D49.5 — `RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }` variant** (new). Added in `modules/crates/domain/src/repository.rs`. Mirrors CH-05's `RepositoryError::ManifestValidation { source: ValidationError }` shape (ADR-0044 §D44.8). Additive on `RepositoryError`; trait method signatures unchanged. Repository contract documented in trait module-level doc-comment as a precondition note for any future tag-write method (no `update_session_tags` method exists today).
- **D49.6 — `SESSION_FROZEN_TAG_PREFIXES` constant** (per Fork F3.C locked). `pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` at the validator module. Initial contents: `["#kind:", "session:", "agent:", "project:", "org:", "task:", "delegated_from:", "role_at_creation:", "agent_kind:", "derived_from:"]` (10 prefixes). Cross-references concept doc 05 lines 220–231 in a doc-comment. Tests verify (a) every prefix in `reserved_namespace_prefixes()` for `session_object` constituents is in this list, (b) the 6 M6+ categories are present, and (c) `#archived` / `#active` are NOT in the list.
- **D49.7 — Audit-event builder for rejection (per Fork F5.B USER-LOCKED, overrides planner-recommendation F5.A).** Ship a new audit-event builder `pub fn frozen_tag_write_rejected(actor: AgentId, target_session: SessionId, org: OrgId, violation: &FrozenTagViolation, attempted_at: DateTime<Utc>) -> AuditEvent` at NEW module `modules/crates/domain/src/audit/events/m5_2/tool_authority.rs`. Wire the module into `audit/events/m5_2/mod.rs` (`pub mod tool_authority;`).
  - **Event type:** `"tool.frozen_tag_write_rejected"` (dotted name follows CH-23 pattern; `tool` namespace rather than `consent` / `platform` because the rejection is about a tool-authority assertion failing).
  - **Audit class:** `AuditClass::Alerted` (security event class — already exists at `domain/src/audit/mod.rs:39`; used by 12 existing emitters per `grep -rn "AuditClass::Alerted" modules/crates/`).
  - **`org_scope`:** populated from `org` argument; chains within the org per existing hash-chain semantics (no chain-symmetry change — see §3.B A7).
  - **`actor_agent_id`:** the `actor` argument (the agent whose tool attempted the write); `target_entity_id`: `Some(NodeId::from_uuid(*target_session.as_uuid()))`.
  - **`diff` shape:** `{ "before": null, "after": { "session_id": <id>, "violation_kind": "frozen_tag_added" | "frozen_tag_removed", "tag": <tag>, "attempted_at": <rfc3339> } }` — mirrors `consent.requested` "before:null/after:full-payload" shape for create-style rejection events.
  - **Why F5.B over planner-recommended F5.A:** user prioritises operator-visibility into retag-attempts in the audit log even when there's no production callsite today. The builder is forward-defensive symmetric to F2.A; it ships with tests but no production `audit.emit(...)` callsite — the actual emission lands in a future chunk that wires `update_session_tags` HTTP/CLI endpoint. The F5.B-driven scope cost (~0.1 engineer-days) was estimated at iter-1 as "+1 migration column" but on investigation (see "User lock rationale" in §"Forks") the migration cost is **zero** because the `audit_events` table is already schema-stable.
  - **Repository trait contract docstring update:** the trait module-level docstring's precondition note (per D49.5) MUST also state: *"Failures from `validate_tag_write_on_session` MUST be paired with an `audit.emit(frozen_tag_write_rejected(...))` call before propagating the error."* This makes the contract complete: validator + audit-event are paired at any future tag-write callsite.
  - **What CH-12 does NOT do per F5.B:** does NOT wire the `audit.emit(...)` callsite (no callsite to wire today). Does NOT add an `AuditEmitter` parameter to `validate_tag_write_on_session` (would couple a pure-fn to async emission — investigated and rejected as Option C in iter-2 analysis). Does NOT add a migration (audit_events table is schema-stable per §3.B A4).

ADR file path: [`m5_2/decisions/0049-frozen-session-tag-immutability.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md) (NEW; drafted at P1, Accepted at P3).

**Cross-references in the ADR:**
- ADR-0044 (CH-05 — `validate_published_manifest` + Rule C precedent for composite-cascade reserved-namespace).
- ADR-0036 / ADR-0037 (CH-06 — `Composite::ALL` + `kind_name()` + `reserved_namespace_prefixes()` reuse).
- ADR-0040 / ADR-0041 (CH-21 — audit hash chain semantics; D49.7 inherits chain-link symmetry).
- ADR-0033 (CH-K8S-PREP §D33 — K8s readiness; confirmed neutral here).
- Audit-event family precedent: `modules/crates/domain/src/audit/events/m5/consents.rs` (consent transition events; same builder pattern); `m5_2/memory.rs` (memory-extracted, also `m5_2`-bucketed); `m4/agents.rs` (Alerted-class for security-sensitive changes).
- Concept docs: `permissions/05-memory-sessions.md` lines 220–231 + 516–525 + 531–541; `permissions/01-resource-ontology.md` lines 254–261 + 267; `permissions/09-selector-grammar.md` lines 190–196; `permissions/04-manifest-and-resolution.md` §"Manifest Validation at Publish Time".
- Drift D-new-08 (closed).
- Forward-scope row CH-12 (lines 123–128).
- F5.B user-lock rationale (in §"Forks" of this plan).

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| Post-CH-11 baseline | `cargo test --workspace -- --test-threads=1` ≈ **1319** passed / 0 failed / 1 ignored (cycle-audit recorded value) | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` |
| Post-CH-11 baseline | 4 CI guards green | `bash scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` |
| Post-CH-11 baseline | `grep -rn "use phi_core::" modules/crates/ \| wc -l == 48` | the literal command |
| CH-05 / ADR-0044 | `validate_published_manifest` exists at `permissions/manifest/validator.rs:296`; 6 `ValidationError` variants; `RESERVED_NAMESPACE_LITERALS` constant; `reserved_namespace_prefixes()` generator wired off `Composite::ALL` | `cargo test -j 4 -p domain --lib permissions::manifest::validator::tests`<br>`cargo test -j 4 -p server --test acceptance_manifest_validator -- --test-threads=1` |
| CH-05 / ADR-0044 | `RepositoryError::ManifestValidation` variant exists; both Repository impls call validator as precondition | `grep -n "ManifestValidation\b" modules/crates/domain/src/repository.rs` returns >= 1 |
| CH-06 / ADR-0036 + 0037 | Selector grammar accepts reserved tags (read OK); `Composite::ALL` enumerates 8 composites; `auto_tags_for("session", id)` emits `#kind:session` + `session:<id>` | `cargo test -j 4 -p domain --lib permissions::selector`<br>`grep -n "auto_tags_for(\"session\"" modules/crates/server/src/platform/sessions/launch.rs` returns 1 hit at line 336 |
| CH-09 / ADR-0045 | `Consent` shape unchanged | `cargo test -j 4 -p domain --lib model::nodes::tests` |
| CH-10 / ADR-0047 | Consent state machine + sweeper unchanged | `cargo test -j 4 -p domain --lib consents` |
| CH-11 / ADR-0048 | Engine Step 6 real body unchanged; `ApprovalMode` + `ConsentScope.session_id` + `Organization.approval_timeout(_default_response)` shape unchanged | `cargo test -j 4 -p server --test acceptance_per_session_consent_gating -- --test-threads=1`<br>`cargo test -j 4 -p domain --lib permissions::engine` |
| CH-21 / ADR-0040+0041 | Audit hash chain byte-stable; `AuditEvent::canonical_bytes()` excludes `prev_event_hash`; `AuditClass::Alerted` exists at `audit/mod.rs:39`; `audit_events` table schema-stable (`event_type TYPE string` + `diff FLEXIBLE TYPE object`) | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1`<br>`cargo test -j 4 -p domain --test audit_hash_chain_props`<br>`cargo test -j 4 -p store --test audit_emitter_chain_test` |
| CH-22 / ADR-0035 | Catalog listener body unchanged | `cargo test -j 4 -p domain --lib events::listeners` |
| CH-K8S-PREP / ADR-0033 | `SessionRegistry` trait + `SurrealStore::open_remote` + SIGTERM drain + EventBus shutdown unchanged | smoke tests via existing suites |
| All chunks | Migration count = 13 (post-CH-11); CH-12 adds 0 | `cargo test -j 4 -p store --test migrations_test -- --test-threads=1` |

**Run discipline.** This table runs AT CHUNK OPEN before P1 starts and again at chunk seal. Any regression → new drift file + AskUserQuestion before continuing.

---

## §7 — Phases within the chunk

**Phase count: 3** → audit envelope = **2 agents** (medium chunk per audit-envelope-size skill: 3–5 phases = Medium / 2 agents). Per CH-05 + CH-11 precedent.

### P1 — Validator extension (Rule E + runtime validator + constants) + audit-event builder + ADR Proposed (~0.7d)

**Goal.** Land both halves of the validator at `domain::permissions::manifest::validator` (publish-time Rule E + runtime `validate_tag_write_on_session` + `SESSION_FROZEN_TAG_PREFIXES` constant) AND the F5.B audit-event builder at `domain::audit::events::m5_2::tool_authority`. ADR-0049 draft committed as Proposed. No Repository wiring yet.

**Deliverables.**

1. **Extend `ValidationError`** at `modules/crates/domain/src/permissions/manifest/validator.rs` — add 7th variant:
   ```rust
   #[error(
       "manifest declares action {:?} on composite {:?} with target_kinds entry overlapping reserved namespace {:?} — runtime owns these tag namespaces",
       action, composite, namespace
   )]
   CompositeStructuralTagWrite {
       composite: Composite,
       action: Action,
       namespace: String,
   },
   ```

2. **Implement Rule E** in `validate_published_manifest` (after Rule C, before Rule B). Rationale for ordering: Rule C remains the bare-tag fast-path; Rule E is the composite-cascade extension. Pseudocode:
   ```rust
   // Rule E - Composite reserved-namespace target_kinds rejection
   if m.actions.contains(&Action::Modify) {
       let reserved = reserved_namespace_prefixes();
       for c in &declared_composites {
           if *c == Composite::MemoryObject { continue; } // D49.1.a exemption
           for tk in &m.target_kinds {
               if let Some(matched) = reserved.iter().find(|p| {
                   tk.starts_with(p.as_str()) || tk == p.trim_end_matches(':')
               }) {
                   return Err(ValidationError::CompositeStructuralTagWrite {
                       composite: *c,
                       action: Action::Modify,
                       namespace: matched.clone(),
                   });
               }
           }
       }
   }
   ```
   *Note on matching semantics:* `target_kinds` entries can be either bare names (e.g., `"session"`) or prefixed (`"session:s-9831"`). The match handles both.

3. **`SESSION_FROZEN_TAG_PREFIXES` constant** (per F3.C):
   ```rust
   /// All session-tag prefixes that are concept-doc-frozen at creation per
   /// `permissions/05-memory-sessions.md` §"Tag Vocabulary for Sessions"
   /// lines 220-231. Lifecycle tags (`#archived`, `#active`) are NOT in
   /// this set.
   pub const SESSION_FROZEN_TAG_PREFIXES: &[&str] = &[
       "#kind:", "session:", "agent:", "project:", "org:",
       "task:", "delegated_from:", "role_at_creation:", "agent_kind:", "derived_from:",
   ];
   ```

4. **`FrozenTagViolation` error type** + `validate_tag_write_on_session` function (per D49.3, D49.4 — full body shape preserved from iter-1 plan):
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
   pub enum FrozenTagViolation {
       #[error("session {session_id} attempt to add frozen tag {tag:?}")]
       FrozenTagAdded { session_id: crate::model::ids::SessionId, tag: String },
       #[error("session {session_id} attempt to remove frozen tag {tag:?}")]
       FrozenTagRemoved { session_id: crate::model::ids::SessionId, tag: String },
   }

   pub fn validate_tag_write_on_session(
       session_id: crate::model::ids::SessionId,
       current_tags: &[String],
       proposed_tags: &[String],
   ) -> Result<(), FrozenTagViolation> { /* set-difference logic; lifecycle tags pass through */ }
   ```

5. **Re-export new symbols** at `permissions/mod.rs`:
   ```rust
   pub use manifest::validator::{
       ..., // CH-05 exports preserved
       validate_tag_write_on_session,
       FrozenTagViolation,
       SESSION_FROZEN_TAG_PREFIXES,
   };
   ```

6. **NEW audit-event builder module** `modules/crates/domain/src/audit/events/m5_2/tool_authority.rs` (per D49.7):
   ```rust
   //! Audit-event builders for tool-authority security events
   //! (CH-12 / ADR-0049 §D49.7). One event today:
   //! - `tool.frozen_tag_write_rejected` - Alerted class. Emitted when a
   //!   future tag-write Repository method's call to
   //!   `validate_tag_write_on_session` returns
   //!   `Err(FrozenTagViolation)`. Today no production callsite emits
   //!   this event; the builder is forward-defensive symmetric to
   //!   `validate_tag_write_on_session`. The Repository trait
   //!   docstring documents the pairing contract.
   //!
   //! Concept doc cross-ref: `permissions/05-memory-sessions.md`
   //! Example 7 lines 516-525 ("worker tries to retag... Denied. The
   //! request never reaches the storage layer"). The audit event makes
   //! the rejection operator-visible at Alerted retention tier.

   use chrono::{DateTime, Utc};

   use crate::audit::{AuditClass, AuditEvent};
   use crate::model::ids::{AgentId, AuditEventId, NodeId, OrgId, SessionId};
   use crate::permissions::manifest::validator::FrozenTagViolation;

   /// `tool.frozen_tag_write_rejected` - Alerted class.
   pub fn frozen_tag_write_rejected(
       actor: AgentId,
       target_session: SessionId,
       org: OrgId,
       violation: &FrozenTagViolation,
       attempted_at: DateTime<Utc>,
   ) -> AuditEvent {
       let (kind, tag) = match violation {
           FrozenTagViolation::FrozenTagAdded { tag, .. } =>
               ("frozen_tag_added", tag.clone()),
           FrozenTagViolation::FrozenTagRemoved { tag, .. } =>
               ("frozen_tag_removed", tag.clone()),
       };
       AuditEvent {
           event_id: AuditEventId::new(),
           event_type: "tool.frozen_tag_write_rejected".to_string(),
           actor_agent_id: Some(actor),
           target_entity_id: Some(NodeId::from_uuid(*target_session.as_uuid())),
           timestamp: attempted_at,
           diff: serde_json::json!({
               "before": serde_json::Value::Null,
               "after": {
                   "session_id":     target_session.to_string(),
                   "violation_kind": kind,
                   "tag":            tag,
                   "attempted_at":   attempted_at.to_rfc3339(),
               },
           }),
           audit_class: AuditClass::Alerted,
           provenance_auth_request_id: None,
           org_scope: Some(org),
           prev_event_hash: None,
       }
   }
   ```

7. **Wire the new module** at `modules/crates/domain/src/audit/events/m5_2/mod.rs`:
   ```rust
   pub mod tool_authority;
   ```

8. **ADR-0049 draft** at `docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md` (NEW). Status: `**Status: Proposed**`. Sub-decisions D49.1-D49.7 written verbatim per §5. Cross-references per §5. Body lands in P1; flipped to `Accepted` at P3 seal.

**Tests (P1).** ~26 unit tests (extending the existing CH-05 test module + new audit-event tests; revised iter 2 to absorb F5.B):

  - **Validator tests (~22 — unchanged from iter 1):** Rule E hard rejection (8 composites × 1 cell), Rule E memory-exemption (D49.1.a; 1 test verifying MemoryObject does NOT trigger Rule E), Rule E target_kinds bare-name match, Rule E does-not-fire-for-non-Modify, `SESSION_FROZEN_TAG_PREFIXES` exhaustive test, `validate_tag_write_on_session` happy paths (~6), `validate_tag_write_on_session` rejection paths (~6), `FrozenTagViolation` Display test, CH-05 regression coverage.
  - **Audit-event tests (~4 — NEW per F5.B; live in `audit::events::m5_2::tool_authority::tests`):**
    - `frozen_tag_write_rejected_added_carries_alerted_class_and_org_scope` — assert `event_type == "tool.frozen_tag_write_rejected"`, `audit_class == AuditClass::Alerted`, `org_scope == Some(org)`, `actor_agent_id == Some(actor)`, `target_entity_id == Some(NodeId::from_uuid(session))`, `diff.after.violation_kind == "frozen_tag_added"`, `diff.after.tag == <added-tag>`.
    - `frozen_tag_write_rejected_removed_branch` — same as above for `FrozenTagViolation::FrozenTagRemoved`; assert `violation_kind == "frozen_tag_removed"`.
    - `frozen_tag_write_rejected_canonical_bytes_byte_stable` — build two events with identical inputs, assert `canonical_bytes()` equal byte-for-byte (chain-symmetry sanity per §3.B A7).
    - `frozen_tag_write_rejected_diff_carries_attempted_at_rfc3339` — assert `diff.after.attempted_at` parses as RFC3339.

**Concept-alignment check.** §2 row "Frozen-at-creation tags lines 531-541" transitions `contradicted → honored` (publish-time half). §2 row "Example 7 lines 516-525" transitions `silent-in-code → honored` (runtime gate + audit-event ready).

**phi-core leverage check.** Zero phi-core imports added. Verify with `grep -rn "use phi_core::" modules/crates/domain/src/permissions/manifest/ modules/crates/domain/src/audit/events/m5_2/` (expect 0).

**User-facing doc updates.** None at P1.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- Adding the `CompositeStructuralTagWrite` variant breaks an existing exhaustive `match` on `ValidationError` somewhere in the workspace.
- The Rule E test suite reveals a contradiction with CH-05 Rule C.
- The audit-event builder's `canonical_bytes()` test fails (suggests an unintended hash-chain byte-instability — this would be a real K8s A7 blocker per §3.B; escalate to user).
- Any composite's `kind_name()` collides with one of the 6 M6+ category prefixes (would mean concept doc is internally inconsistent).

---

### P2 — Acceptance tests (publish + runtime + audit-event paths) + Repository wiring (~0.5d)

**Goal.** Wire the new variants into `RepositoryError`, document the audit-event pairing contract on the Repository trait, ship acceptance tests covering the publish-time, runtime, and audit-event-builder paths end-to-end.

**Deliverables.**

1. **`RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }`** added in `modules/crates/domain/src/repository.rs`.

2. **Repository trait module-level docstring update** noting the precondition contract (revised iter 2 to include F5.B pairing): *"Any future Repository method that proposes to update a Session's tag set MUST (a) call `domain::permissions::manifest::validator::validate_tag_write_on_session` as its first line, (b) propagate failures as `RepositoryError::FrozenSessionTagWrite`, AND (c) emit `domain::audit::events::m5_2::tool_authority::frozen_tag_write_rejected` via the injected `AuditEmitter` before propagating the error (per ADR-0049 §D49.7)."* (No actual `update_session_tags` method exists today; the docstring is the contract for future chunks.)

3. **Update CH-05's existing acceptance file** at `modules/crates/server/tests/acceptance_manifest_validator.rs` — extend with ~5 Rule E tests (per iter-1 plan; D49.1.a Memory-exemption test included).

4. **NEW acceptance test file** `modules/crates/server/tests/acceptance_frozen_session_tags.rs` covering both the runtime validator path AND the audit-event builder. ~10 tests (revised iter 2 from 7):
   - Tests 1-7 (runtime validator, unchanged from iter 1): lifecycle flip Ok; FrozenTagAdded; FrozenTagRemoved; concept-doc Example 7 fixture (worker retag attempt); lifecycle-flip happy path; round-trip happy path; empty-tags edge case.
   - **Test 8 (NEW, F5.B):** `validate_tag_write_on_session` returning `Err(FrozenTagAdded)` paired with `frozen_tag_write_rejected(...)` produces an `AuditEvent` with `event_type == "tool.frozen_tag_write_rejected"`, `audit_class == AuditClass::Alerted`, `target_entity_id` matching the session, and `diff.after.tag` matching the rejected tag. End-to-end demonstration of the trait-contract pairing.
   - **Test 9 (NEW, F5.B):** Same as Test 8 for `FrozenTagRemoved`.
   - **Test 10 (NEW, F5.B):** `frozen_tag_write_rejected` events for two different orgs produce events with different `org_scope` values (cross-org chain-isolation sanity).

5. **Confirm no existing test breaks.** P2 runs the full workspace test suite. All CH-05's `acceptance_manifest_validator` tests + CH-06's selector tests + CH-09/10/11's consent tests + CH-21 audit hash-chain tests stay green.

**Tests (P2).** ~15 acceptance tests added (5 publish-time extension + 10 runtime + audit-event path; revised iter 2 from 12).

**Concept-alignment check.** §2 rows "Frozen-at-creation tags" + "Example 7" + "Reserved tag namespaces rule 1" all stay at their P1 status (`honored` for the immutability axis).

**phi-core leverage check.** Zero phi-core imports added across the new test files.

**User-facing doc updates.** None at P2.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- Adding `RepositoryError::FrozenSessionTagWrite` breaks an exhaustive `match` somewhere.
- A SurrealDB integration test surfaces a difference between `InMemoryRepository` and `SurrealStore::open_embedded`.
- The `acceptance_memory_extraction` (CH-21) hash-chain test goes red — would indicate the additive event_type accidentally perturbed prior events' canonical_bytes (should be impossible per §3.B A7 reasoning, but the test is the proof). Escalate to user immediately.
- The Concept Doc 05 Example 7 fixture (Test 4) reveals an ambiguity in how "retag" maps to the validator's `current_tags` / `proposed_tags` argument shape.

---

### P3 — ADR Accepted + drift closed + concept-doc bumps + 2-agent audit + seal (~0.4d)

**Goal.** Ratify ADR-0049. Close D-new-08. Spawn 2 audit agents per §11. Seal the chunk.

**Deliverables.**

1. **ADR-0049 flipped from `Proposed` → `Accepted`** at `m5_2/decisions/0049-frozen-session-tag-immutability.md`.

2. **D-new-08 Status flipped to `remediated`**. Lifecycle entry appended (mentions BOTH validator extension AND audit-event builder per F5.B).

3. **`drifts/README.md`** — D-new-08 row flipped + "Closes at" → CH-12 ✓.

4. **`_concept-audit-matrix.md`** — every row touching "Frozen-at-creation tags (immutability)" / "Reserved-namespace write rejection (composite case)" flipped to `honored`. Verified-header description matches body diff exactly.

5. **Concept doc verified-header bumps (4 files):** `permissions/05-memory-sessions.md`, `permissions/01-resource-ontology.md`, `permissions/09-selector-grammar.md`, `permissions/04-manifest-and-resolution.md`. The `05-memory-sessions.md` bump explicitly mentions BOTH `validate_tag_write_on_session` AND `frozen_tag_write_rejected` per F5.B. All four: doc body UNCHANGED.

6. **Architecture/operations doc check:** `m1/architecture/audit-events.md` (if exists) — verify `tool.frozen_tag_write_rejected` is in any catalogued event-type list; if a list update is needed, lands here. `m1/architecture/permission-check-engine.md` (if exists) — header bump if "Rule A/B/C/D" claim is present. `m5*/operations/permission-engine-operations.md` (if exists) — add `FrozenSessionTagWrite → HTTP 422` + `tool.frozen_tag_write_rejected → Alerted, 60s alert delivery` rows.

7. **Cycle index update** — append a row to `baby-phi/docs/specs/plan/build/_cycle-index.md`.

8. **Spawn 2 audit agents** per §11.

9. **(Retrospective only)** — recommend filing `D-CH12-FOLLOWUP-01` LOW drift for the M5/M6 session-tag emission gap.

**Tests (P3).** No new tests; runs the verification recipe in §12.

**Concept-alignment check.** Final pass over §2.

**phi-core leverage check.** Final greps.

**User-facing doc updates.** Concept-doc verified-header bumps.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit agent reports a finding.

---

## §8 — Tests summary

**Expected total at chunk close** — apply CH-11-retro buffer factor `× 1.10–1.15` to the deliverable-listed sum:

- Deliverable-listed sum (revised iter 2 for F5.B):
  - **P1 unit tests:** 22 validator + 4 audit-event builder = **26 tests**
  - **P2 acceptance tests:** 5 publish-time extension + 10 runtime + audit-event path = **15 tests**
  - **Total deliverable-listed: 41 new tests** (was 34 at iter 1 with F5.A; +7 from F5.B = 4 P1 audit-event-builder unit + 3 P2 audit-event integration tests).
- × 1.10–1.15 buffer = **45–47 expected actual** (was 37–39 at iter 1; the implementer may parameterise builder tests across both `FrozenTagViolation` variants in a loop; orchestrator's per-phase test-count review accepts this band).
- **Expected total at chunk close**: post-CH-11 baseline (1319) + 41–47 new = **1360–1366 tests**. Orchestrator's per-phase implementation review accepts test-count delta within ±15% of this figure (so **1346–1380** is the accept band; was 1340–1372 at iter 1); outside that band → AskUserQuestion.

**Layer breakdown** (deliverable estimates):
- Unit (P1): ~26 — 22 validator-module + 4 audit-event-builder module.
- Acceptance (P2): ~15 — 5 in extended `acceptance_manifest_validator.rs`, 10 in NEW `acceptance_frozen_session_tags.rs` (7 runtime + 3 audit-event).

**Named new test files:**
- (NEW) `modules/crates/server/tests/acceptance_frozen_session_tags.rs` — runtime + audit-event acceptance (~10 tests).
- (EXTENDED) `modules/crates/server/tests/acceptance_manifest_validator.rs` — Rule E rejection cells (~5 tests added).
- (EXTENDED) `modules/crates/domain/src/permissions/manifest/validator.rs::tests` — ~22 unit tests added.
- (NEW) `modules/crates/domain/src/audit/events/m5_2/tool_authority.rs::tests` — ~4 builder unit tests.

**Cascade-fan-out estimation (per CH-11 retro v2026-05-03 standards update):**

- `ToolAuthorityManifest` literal-construction sites:
  - Invocation: `git grep -nE 'ToolAuthorityManifest\s*\{' modules/crates/`
  - Raw matched-line count: **6** (verified at planning).
  - Pause-discipline: > 9 sites for the full set; > 3 for the actual-literals subset.

- `RepositoryError` exhaustive `match` sites (additive variant `FrozenSessionTagWrite`):
  - Invocation: `git grep -nE 'RepositoryError::' modules/crates/ | grep -v test`
  - Predicted: ~10–15 sites. Pause-discipline: > 22 sites needing edits (1.5× predicted upper).

- `ValidationError` exhaustive `match` sites (additive variant `CompositeStructuralTagWrite`):
  - Invocation: `git grep -nE 'ValidationError::' modules/crates/`
  - Raw matched-line count: ~5 sites. Pause-discipline: > 8 sites needing edits.

- **NEW iter 2: AuditEvent builder callsites** (additive event_type — should require **zero** out-of-builder-module edits since callers don't `match` on `event_type` strings):
  - Invocation: `git grep -nE '"tool\.frozen_tag_write_rejected"' modules/crates/`
  - Predicted: ≤ 2 sites (the builder's own definition + tests). Pause-discipline: > 4 sites (would imply unintended emission wiring that wasn't planned).
  - Invocation: `git grep -nE 'pub fn (frozen_tag_write_rejected|consent_acknowledged|memory_extracted|agent_created)\b' modules/crates/`
  - Pattern verification: confirms `frozen_tag_write_rejected` lands as a peer of existing builders without disrupting them.

**Named expected-still-green tests** (anything fragile, expanded for F5.B):
- All CH-05 manifest-validator tests (~30 unit + ~10 acceptance) — must still pass.
- CH-06 selector grammar tests (~25 unit + property tests).
- CH-11 acceptance tests (`acceptance_per_session_consent_gating`, `acceptance_consent_node_shape`, all CH-09/10 consent tests).
- **`acceptance_memory_extraction` (CH-21 audit hash chain byte-stable)** — critical proof that adding the new event_type does NOT perturb prior events' canonical_bytes. Test goes RED → escalate to user immediately (would indicate a real K8s A7 blocker per §3.B).
- **`audit_hash_chain_props` (CH-21 property tests)** — same as above.
- **`audit_emitter_chain_test` (store-side)** — same as above.
- `acceptance_system_flows_s05` (CH-23 Template C/D edges).
- The 8-composite tests in `composites.rs::tests::auto_tags_work_for_every_composite`.
- `Action::Modify.applies_to_composite(SessionObject)` and `Action::Modify.applies_to_composite(MemoryObject)` tests at `action.rs:766–777` — both must stay TRUE; CH-12 enforces validator-side rejection only for the SessionObject case (D49.1.a memory-exemption ensures the algebra stays correct).

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Token already generated at draft time: `6a748175`.
2. Cycle folder created at chunk-open (post-approval) by orchestrator/implementer via `chunk-archive-plan` skill: `mkdir -p baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/`.
3. Plan file copied from `/root/.claude/plans/sharded-discovering-stearns.md` to `baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md`.
4. `bash scripts/check-doc-links.sh` → expect green.

### Reading list (mandatory; every item end-to-end before P1 opens)

1-22. (Iter-1 list preserved; see prior plan for items 1–22.)
23. **NEW iter 2 (F5.B):** [`modules/crates/domain/src/audit/mod.rs`](baby-phi/modules/crates/domain/src/audit/mod.rs) — full file (`AuditEvent`, `AuditClass`, `canonical_bytes`, `hash_event` — required to ground D49.7 design).
24. **NEW iter 2 (F5.B):** [`modules/crates/domain/src/audit/events/m5/consents.rs`](baby-phi/modules/crates/domain/src/audit/events/m5/consents.rs) — full file (precedent builder + tests pattern; D49.7 mirrors this shape).
25. **NEW iter 2 (F5.B):** [`modules/crates/domain/src/audit/events/m5_2/memory.rs`](baby-phi/modules/crates/domain/src/audit/events/m5_2/memory.rs) — confirms the m5_2 sub-bucket is the right home for `tool_authority.rs`.
26. **NEW iter 2 (F5.B):** [`modules/crates/store/migrations/0001_initial.surql`](baby-phi/modules/crates/store/migrations/0001_initial.surql) lines containing `DEFINE TABLE audit_events` — confirms the schema is `event_type TYPE string` + `diff FLEXIBLE TYPE object` (no migration needed; documented basis of §3.B A4 verdict).
27. **NEW iter 2 (F5.B):** [`modules/crates/store/src/audit_emitter.rs`](baby-phi/modules/crates/store/src/audit_emitter.rs) — confirms `prev_event_hash` insertion logic doesn't depend on the event_type set (additive event types are safe).
28. **NEW iter 2 (F5.B):** [`modules/crates/domain/src/audit/events/m4/agents.rs`](baby-phi/modules/crates/domain/src/audit/events/m4/agents.rs) lines 80–110 — Alerted-class precedent (`platform.agent.created` is the closest analogue: Alerted, security-sensitive event).

### Carry-forward invariants (verified green at chunk open)

- `cargo test --workspace -- --test-threads=1` test count = **1319** / 0 failed / 1 ignored.
- `scripts/check-phi-core-reuse.sh` exits 0.
- `scripts/check-doc-links.sh` exits 0.
- `scripts/check-ops-doc-headers.sh` exits 0.
- `scripts/check-spec-drift.sh` exits 0.
- `git diff --stat HEAD -- modules/` empty.
- D-new-08 Status = `discovered`.
- ADR-0034..0048 Accepted; **next-free = ADR-0049**.
- Migrations registered = 13 (post-CH-11); CH-12 adds **0** migrations (F5.B confirmed: no migration per §3.B A4).
- `grep -rn "use phi_core::" modules/crates/ | wc -l` = **48**.
- **NEW iter 2 (F5.B):** `audit_events` table schema-stable (`event_type TYPE string`); `AuditClass::Alerted` exists; `audit_hash_chain_props` + `audit_emitter_chain_test` green at baseline.

### Pending decisions carried into this chunk

- **Forks F1–F5 — ALL USER-LOCKED at iter 2:** F1.A / F2.A / F3.C / F4.A / **F5.B** (F5 overrides planner-recommendation). No further fork ambiguity.
- Forward-scope §1.4 row CH-12: HIGH severity → must close before M5 tag.
- D-new-08 lifecycle transition `discovered → in-chunk-plan` lands at chunk-open before P1.

**Chunk-ordering note.** No forward dependencies on un-sealed chunks.

---

## §10 — Close criteria

**Source of truth: concept docs.** No rounding; below-target blocks close.

### 4 aspects (each graded pass / fail)

- **Code aspect** — all phases' deliverables shipped; `cargo test --workspace -- --test-threads=1` green at ~1360–1366; clippy green under `RUSTFLAGS="-Dwarnings"`; `cargo fmt --all -- --check` green.
- **Docs aspect** —
  - *Governance tier*: D-new-08 `remediated`; concept-audit matrix immutability rows flipped `honored`; ADR-0049 `Accepted` with all 7 sub-decisions documented (D49.1–D49.7); `drifts/README.md` index updated; verified headers bumped on every modified concept doc; cycle-index updated.
  - *User-facing tier*: every §3.C row resolved.
- **phi-core leverage aspect** — import-count delta = 0 (predicted); positive greps match baseline; `check-phi-core-reuse.sh` green; no `use phi_core::` in `modules/crates/domain/src/permissions/manifest/` OR `modules/crates/domain/src/audit/events/m5_2/`.
- **Concept alignment aspect** — every §2 row at its target chunk-close status; no row remains `contradicted`.

### 2 confidence % (each with named numerator/denominator)

- **Implementation confidence %** = `claims-honored / claims-in-scope` — **target ≥ 11/12** (revised iter 2 from 9/10; F5.B adds 2 claims):
  1. `ValidationError::CompositeStructuralTagWrite` variant exists with the documented payload + Display.
  2. `validate_published_manifest` Rule E rejects `[Modify] × composite × reserved-namespace-target_kinds` per D49.1 + D49.1.a Memory-exemption.
  3. Rule E does NOT fire for non-Modify actions, non-composite resources, non-reserved target_kinds, or `Composite::MemoryObject`.
  4. `SESSION_FROZEN_TAG_PREFIXES` constant contains 10 prefixes per D49.6.
  5. `validate_tag_write_on_session(...)` exists at the validator module per D49.3.
  6. `FrozenTagViolation { FrozenTagAdded | FrozenTagRemoved }` enum exists per D49.4.
  7. `RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }` variant exists per D49.5.
  8. Acceptance tests cover publish-time path (5 tests) and runtime path (7 tests) and audit-event path (3 tests).
  9. Cross-impl consistency: `InMemoryRepository` + `SurrealStore` give identical Rule E verdicts.
  10. Wire format byte-stable; **no migration added** (F5.B verified per §3.B A4); phi-core import count unchanged.
  11. **NEW iter 2 (F5.B):** `frozen_tag_write_rejected` audit-event builder exists at `domain::audit::events::m5_2::tool_authority` per D49.7. Event type literal: `"tool.frozen_tag_write_rejected"`. AuditClass: `Alerted`. Diff shape matches D49.7. Module wired into `audit/events/m5_2/mod.rs`.
  12. **NEW iter 2 (F5.B):** Audit hash-chain byte-stability preserved — `acceptance_memory_extraction` + `audit_hash_chain_props` + `audit_emitter_chain_test` all stay green; the new event_type does NOT perturb prior events' canonical_bytes (verified by P2 pause-discipline).

- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)` — **target = 100% (e.g., 5/5 — 1 ADR + 4 concept-doc verified-header bumps; same as iter 1 since no new doc pages)**.

### Composite

`min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary)`. Composite below target = close blocked.

**Close-target discipline.** Close report states ALL FIVE measures with named numerators/denominators.

---

## §11 — Post-chunk independent audit plan

**Agent count: 2** (medium chunk per audit-envelope-size skill).

### Audit A — Code correctness + phi-core leverage + K8s readiness (≤ 600 words)

> You are auditing CH-12 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only on source. Plan at `docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md`. ADR at `docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md`.
>
> PASS/FAIL each numbered claim. Cite file:line for every claim. ≤ 600 words. NOTE: `RUSTFLAGS="..."` + `bash scripts/check-*.sh` are sandbox-blocked from sub-agents; mark `NOT-EXECUTED-IN-AUDIT`, supply grep-equivalent, defer to orchestrator.
>
> 1. `ValidationError::CompositeStructuralTagWrite { composite, action, namespace }` variant exists in `permissions/manifest/validator.rs`. Derives `Debug + Clone + PartialEq + Eq`; impls `Display + Error`. Enum has 7 hard-rejection variants.
> 2. `validate_published_manifest` body contains a Rule E pass with Memory-exemption (D49.1.a). Rule ordering: A → C → E → B → D.
> 3. `pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` exists with exactly 10 prefixes; does NOT contain `#archived` or `#active`.
> 4. `pub fn validate_tag_write_on_session(session_id, current_tags, proposed_tags) -> Result<(), FrozenTagViolation>` exists.
> 5. `FrozenTagViolation` enum has `FrozenTagAdded` + `FrozenTagRemoved` variants with full traits.
> 6. `RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }` variant exists.
> 7. Repository trait module-level docstring documents the precondition contract INCLUDING the F5.B audit-event pairing requirement (must mention `frozen_tag_write_rejected`).
> 8. `permissions/mod.rs` re-exports `validate_tag_write_on_session`, `FrozenTagViolation`, `SESSION_FROZEN_TAG_PREFIXES`.
> 9. ~22 unit tests in `permissions::manifest::validator::tests` covering Rule E + runtime gate + Memory-exemption + Display.
> 10. `acceptance_manifest_validator.rs` extended with ~5 Rule E acceptance tests including cross-impl consistency.
> 11. `acceptance_frozen_session_tags.rs` exists (NEW) with ~10 tests (7 runtime + 3 audit-event integration).
> 12. `cargo test --workspace -j 4 -- --test-threads=1` green at expected count (1360–1366; accept band 1346–1380). Cite actual count.
> 13. Clippy clean — *NOT-EXECUTED-IN-AUDIT*; defer to orchestrator.
> 14. CI guards green — *NOT-EXECUTED-IN-AUDIT*; defer to orchestrator.
> 15. **phi-core leverage**: `grep -rn "use phi_core::" modules/crates/ | wc -l == 48` (unchanged). `grep -rn "use phi_core::" modules/crates/domain/src/permissions/manifest/` returns 0. `grep -rn "use phi_core::" modules/crates/domain/src/audit/events/m5_2/` returns 0.
> 16. **K8s readiness**: A1–A6 axes `no impact`. **A7 — F5.B re-reviewed**: no new K8s blocker because (a) `audit_events` table schema-stable per `migrations/0001_initial.surql` (`event_type TYPE string`, `diff FLEXIBLE TYPE object`); (b) `AuditEvent::canonical_bytes()` excludes `prev_event_hash`; (c) additive event_type strings don't perturb prior events. Migration count = 13 (unchanged). No `CHK8S-D-NN` ledger entry.
> 17. **Cascade fan-out check**: `git grep -nE 'ToolAuthorityManifest\s*\{' modules/crates/` returns ≤ 9 sites. `git grep -nE 'ValidationError::' modules/crates/` returns ≤ 8 sites needing edits. `git grep -nE '"tool\.frozen_tag_write_rejected"' modules/crates/` returns ≤ 4 sites (F5.B builder + tests).
> 18. **Prior-chunk invariants intact**: CH-05 manifest-validator tests green. CH-11 consent gating intact. **CH-21 audit hash chain intact** (`acceptance_memory_extraction` + `audit_hash_chain_props` + `audit_emitter_chain_test` all green — proves F5.B's new event_type does NOT perturb prior events' canonical_bytes).
> 19. **NEW iter 2 (F5.B):** `pub fn frozen_tag_write_rejected(actor, target_session, org, violation, attempted_at) -> AuditEvent` exists at `modules/crates/domain/src/audit/events/m5_2/tool_authority.rs`. Module wired into `audit/events/m5_2/mod.rs` via `pub mod tool_authority;`.
> 20. **NEW iter 2 (F5.B):** `frozen_tag_write_rejected` returns an `AuditEvent` with `event_type == "tool.frozen_tag_write_rejected"`, `audit_class == AuditClass::Alerted`, `org_scope == Some(org)`, `target_entity_id == Some(NodeId::from_uuid(*target_session.as_uuid()))`. Diff shape matches D49.7 (`{before: null, after: {session_id, violation_kind, tag, attempted_at}}`). Both `FrozenTagViolation::FrozenTagAdded` and `FrozenTagRemoved` cases mapped correctly.
> 21. **NEW iter 2 (F5.B):** Audit hash-chain byte-stability — `frozen_tag_write_rejected_canonical_bytes_byte_stable` test green; two builds with identical inputs produce byte-equal `canonical_bytes()`. Proves the new event_type does not introduce non-determinism.

### Audit B — Concept fidelity + docs fidelity + ADR (≤ 600 words)

> You are auditing CH-12's concept-fidelity + docs-fidelity. Read-only.
>
> PASS/FAIL each numbered claim. Cite file:line for every claim. ≤ 600 words.
>
> 1. ADR-0049 file exists at `m5_2/decisions/0049-frozen-session-tag-immutability.md`. Status reads `**Status: Accepted**`.
> 2. ADR-0049 documents sub-decisions D49.1–**D49.7** (revised iter 2; D49.7 is the F5.B audit-event sub-decision). Body matches §5.
> 3. ADR-0049 documents the 5 forks F1–F5 with USER-LOCKED paths (F1.A / F2.A / F3.C / F4.A / **F5.B**). The F5.B lock notes that it overrides the planner-recommended F5.A; the user-lock rationale is recorded.
> 4. ADR-0049 cross-references concept docs + drift D-new-08 + ADR-0044 + ADR-0036 + ADR-0037 + **ADR-0040 + ADR-0041** (CH-21 audit-chain semantics, required for D49.7) + ADR-0033.
> 5. Drift `D-new-08.md` Status = `remediated`; lifecycle entry mentions BOTH validator extension AND audit-event builder.
> 6. `drifts/README.md` row for D-new-08 flipped.
> 7. `_concept-audit-matrix.md` rows touching frozen-tag immutability flipped to `honored`. Verified-header description matches body diff exactly.
> 8. Concept doc `permissions/05-memory-sessions.md` verified-header bumped. The CH-12 amendment line mentions `validate_tag_write_on_session` AND Rule E AND `frozen_tag_write_rejected` (per F5.B). Doc body UNCHANGED.
> 9. Concept doc `permissions/01-resource-ontology.md` verified-header bumped. Body UNCHANGED.
> 10. Concept doc `permissions/09-selector-grammar.md` verified-header bumped. Body UNCHANGED.
> 11. Concept doc `permissions/04-manifest-and-resolution.md` verified-header bumped. Body UNCHANGED.
> 12. Plan archive at `plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md` exists (folder-style).
> 13. **Cycle index** has a row for CH-12-6a748175.
> 14. CH-05 + CH-06 + CH-09 + CH-10 + CH-11 + **CH-21** invariants intact: ADR-0044, ADR-0036, ADR-0037, ADR-0045, ADR-0047, ADR-0048, **ADR-0040, ADR-0041** still Accepted; concept docs retain prior amendment lines.
> 15. Forward-scope row for CH-12 still reads as before.
> 16. Carry-forward gap documented: §"Tag Vocabulary for Sessions" emission gap explicitly retained as `partially-honored`.
> 17. **NEW iter 2 (F5.B):** If `m1/architecture/audit-events.md` (or any milestone-architecture doc) catalogues a list of audit event types, verify that `tool.frozen_tag_write_rejected` is present in the list (or that the doc has been verified-header bumped to acknowledge the new event type). If no such list exists, mark this claim N/A with citation.

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (orchestrator runs these)
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test (cap workers at -j 4)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1360-1366 passed / 0 failed / 1 ignored (within accept band 1346-1380)

# 3. Positive greps - chunk-specific
grep -n "CompositeStructuralTagWrite\b" modules/crates/domain/src/permissions/manifest/validator.rs   # >= 2
grep -n "fn validate_tag_write_on_session\b" modules/crates/domain/src/permissions/manifest/validator.rs   # 1
grep -n "pub enum FrozenTagViolation\b" modules/crates/domain/src/permissions/manifest/validator.rs   # 1
grep -n "SESSION_FROZEN_TAG_PREFIXES\b" modules/crates/domain/src/permissions/manifest/validator.rs   # >= 1
grep -n "FrozenSessionTagWrite\b" modules/crates/domain/src/repository.rs   # >= 1
grep -n "validate_tag_write_on_session\b" modules/crates/domain/src/permissions/mod.rs   # 1 (re-export)
ls modules/crates/server/tests/acceptance_frozen_session_tags.rs   # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md   # 1

# F5.B audit-event builder
ls modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # exists (NEW)
grep -n "fn frozen_tag_write_rejected\b" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # 1
grep -n '"tool.frozen_tag_write_rejected"' modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # >= 1
grep -n "AuditClass::Alerted" modules/crates/domain/src/audit/events/m5_2/tool_authority.rs   # >= 1
grep -n "pub mod tool_authority" modules/crates/domain/src/audit/events/m5_2/mod.rs   # 1

# 4. Forbidden / regression greps
grep -rn "use phi_core::" modules/crates/domain/src/permissions/manifest/   # 0
grep -rn "use phi_core::" modules/crates/domain/src/audit/   # 0
grep -rn "use phi_core::" modules/crates/ | wc -l   # 48
grep -rn '^pub struct.*FrozenTag\|^pub enum.*FrozenTag' modules/crates/ | grep -v "permissions/manifest/validator.rs"   # 0
grep -rn '^pub struct AuditEvent\|^pub enum AuditEvent' modules/crates/ | grep -v "domain/src/audit/mod.rs"   # 0

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-08.md   # 1

# 6. Targeted suites
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::manifest::validator::tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib audit::events::m5_2::tool_authority   # F5.B
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_manifest_validator -- --test-threads=1
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_frozen_session_tags -- --test-threads=1

# 7. Carry-forward sanity (CRITICAL: F5.B byte-stability proof)
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::engine
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_per_session_consent_gating -- --test-threads=1
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_consent_node_shape -- --test-threads=1
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib consents
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1   # CH-21 audit hash chain - must stay green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test audit_hash_chain_props   # F5.B byte-stability proof
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test audit_emitter_chain_test   # F5.B byte-stability proof
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::selector
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migrations_test -- --test-threads=1   # version 13 (no new migration)

# 8. Cycle index + drift / matrix counts
grep -l "Status.*remediated" docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
grep -n "6a748175" docs/specs/plan/build/_cycle-index.md   # 1
```

---

## What this plan does NOT do

- HTTP / CLI `update_session_tags` / `retag_session` endpoints (M6+ tool-admin chunk).
- ~~Audit-event emission on rejection (F5.A locked).~~ **REVISED iter 2:** F5.B locked; CH-12 ships the audit-event builder + Repository trait contract docstring. The actual `audit.emit(...)` call at a tag-write callsite is deferred to a future chunk that wires `update_session_tags`.
- Session-tag emission completeness for the 6 M6+ categories.
- Engine Step ordering changes (F2.A locked).
- Memory-tag immutability (Memory tags ARE intentionally agent-mutable; D49.1.a Memory-exemption confirmed).
- Migration 0014 — no migration needed (`audit_events` table schema-stable per `migrations/0001_initial.surql`; verified at iter 2 for F5.B).
- AuditEmitter dependency injection into `validate_tag_write_on_session` (rejected per F5.B Option C analysis — would couple a pure-fn to async emission).

---

## Critical files

**New:**
- `modules/crates/server/tests/acceptance_frozen_session_tags.rs` — runtime + audit-event acceptance suite (~10 tests).
- `modules/crates/domain/src/audit/events/m5_2/tool_authority.rs` — **NEW iter 2 (F5.B):** `frozen_tag_write_rejected` builder + ~4 unit tests.
- `docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md` — ADR-0049 with sub-decisions D49.1–**D49.7**.

**Modified:**
- `modules/crates/domain/src/permissions/manifest/validator.rs` — `ValidationError::CompositeStructuralTagWrite` variant, Rule E pass (with D49.1.a Memory-exemption), `SESSION_FROZEN_TAG_PREFIXES` constant, `validate_tag_write_on_session` function, `FrozenTagViolation` enum, ~22 unit tests added.
- `modules/crates/domain/src/permissions/mod.rs` — re-exports.
- `modules/crates/domain/src/repository.rs` — `RepositoryError::FrozenSessionTagWrite` variant + trait module-level precondition docstring (revised iter 2 to mention F5.B audit-event pairing).
- **NEW iter 2 (F5.B):** `modules/crates/domain/src/audit/events/m5_2/mod.rs` — add `pub mod tool_authority;`.
- `modules/crates/server/tests/acceptance_manifest_validator.rs` — extended.
- Drift files: `D-new-08.md`, `drifts/README.md`, `_concept-audit-matrix.md`.
- Concept docs: `permissions/05-memory-sessions.md` (CH-12 amendment line mentions validator + audit-event), `01-resource-ontology.md`, `09-selector-grammar.md`, `04-manifest-and-resolution.md` — header bumps only (body UNCHANGED).
- Architecture / operations docs (verified during P3): `m1/architecture/permission-check-engine.md` (if exists), `m1/architecture/audit-events.md` (if exists, F5.B).
- `docs/specs/plan/build/_cycle-index.md` — new row added.

**Direct-approval criteria re-check (orchestrator gate 1, post-iter-2):**

| Criterion | Status |
|---|---|
| No locked forks (planner-recommended path holds) | **WARN** — F5 diverges from planner recommendation. User explicitly locked F5.B. The orchestrator may still auto-approve if the criterion is interpreted as "no UNLOCKED forks remain blocking the plan", which is true (all 5 forks now locked). If interpreted as "no fork diverges from planner recommendation", F5 fails this clause and orchestrator must escalate to user via AskUserQuestion + ExitPlanMode. **Recommend: escalate** to be explicit about the divergence; user has already chosen F5.B but orchestrator confirms via the formal gate. |
| Scope ≤ 1.5× forward-scope | **PASS** — forward-scope row 123 estimated 1.5d; revised CH-12 estimate is 1.6d (1.07× — well below 1.5×). |
| Zero phi-core leverage delta | **PASS** — import count stays at 48; F5.B audit module reuses existing `AuditEvent` + `AuditClass`. |
| No new K8s blocker class | **PASS** — A7 re-reviewed for F5.B; no blocker (§3.B). |
| Audit envelope ≤ medium | **PASS** — 2 auditors (3-phase chunk; medium per audit-envelope-size skill). |
| Confidence ≥ 9/10 | **PASS** — target is 11/12 = 91.7% (above 9/10 = 90%). |
| No new migration | **PASS** — F5.B verified zero-migration per §3.B A4 (cited evidence: `migrations/0001_initial.surql` schema-stable). |

**Net: 6 of 7 criteria PASS. The one ambiguous criterion (forks-diverge) is a judgment call.** Recommend orchestrator interprets it strictly and escalates to user via AskUserQuestion + ExitPlanMode for transparency, even though user already locked F5.B in conversation. Iteration accounting is honored.

---

## Estimated effort

~1.6 engineer-days (revised iter 2 from 1.5d):
- 0.7d — P1 validator extension + F5.B audit-event builder + ADR Proposed + ~26 unit tests (was 0.6d at iter 1; +0.1d for the F5.B builder + 4 unit tests).
- 0.5d — P2 acceptance tests (~5 publish-time + ~10 runtime + audit-event path) + Repository wiring + workspace clippy/test green.
- 0.4d — P3 ADR Accepted + drift closure + concept-doc + architecture-doc header bumps + matrix flip + drifts/README + 2 audit agents + seal.

---

## Cross-cycle handoff (per CH-11 retrospective)

**Standards updates from CH-11 retro applied to this plan:**

- ✅ Row 1 (chunk-auditor sandbox-block disclaimer) — Audit A claims 13 + 14 marked `NOT-EXECUTED-IN-AUDIT`.
- ✅ Row 2 (orchestrator MUST-RUN list) — clippy + 4 CI guards in §12.
- ✅ Row 3 (P4 paperwork checklist) — applied at §10 + Audit B claim 7.
- ✅ Row 4 (cascade fan-out estimation) — applied at §8 with literal grep invocations + raw counts.
- ✅ Row 5 (Trivial FAIL split).
- ✅ Row 6 (§8 buffer factor × 1.10–1.15) — applied with explicit ±15% accept band; revised iter 2 for F5.B-driven test-count growth.
- ✅ Row 7 (engine-touching chunks read launch.rs + preview.rs).

**NEW iter 2 — F5.B divergence note (for retrospective):** This plan's iter-2 revision documents the case where the user's locked fork diverges from the planner's recommendation. The planner originally recommended F5.A (no audit emission) on the assumption that F5.B would require a migration. On user-driven re-investigation, that assumption was proven wrong: the `audit_events` table is schema-stable for additive event types, so F5.B costs ~0.1 engineer-days, not the iter-1 estimate of "+1 migration column". The retrospective should consider: **(retro candidate) — when planners cite a "+1 migration" cost in fork analysis, require an explicit `git grep` / file-read of the relevant migrations directory before recommending against the fork.** This is a planner-discipline gap that would have been caught at iter-1 had the planner read `migrations/0001_initial.surql:` for the audit_events schema before drafting F5.A's recommendation. Logged here for retrospector consideration.
