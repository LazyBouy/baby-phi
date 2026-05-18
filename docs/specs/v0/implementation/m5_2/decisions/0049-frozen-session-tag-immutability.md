<!-- Last verified: 2026-05-04 by Claude Code (CH-12 P3 chunk-seal — ADR flipped Proposed → Accepted) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-12 P1 — ADR drafted as Proposed) -->

# ADR-0049 — Frozen session-tag immutability enforcement (publish-time Rule E + runtime validator + audit-event builder)

**Status: Accepted**

**Date:** 2026-05-04
**Chunk:** CH-12
**Closes:**
- [`D-new-08`](../../m5_1/drifts/D-new-08.md) (HIGH) — frozen-at-creation session tag immutability not enforced. CH-12 closes the publish-time half (Rule E composite-via-target_kinds rejection), the runtime half (`validate_tag_write_on_session` forward-defensive Repository precondition), and the audit-event half (`tool.frozen_tag_write_rejected` Alerted-class builder per F5.B user-lock).

---

## Context

[`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Frozen-at-creation tags (immutability)" lines 531–541 specifies a security boundary: session structural tags (`#kind:session`, `session:<id>`, plus the M6+ planned set `agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`) are immutable post-creation. A tool that declares `[modify]` on session tags can defeat the multi-scope cascade (e.g., retag session A from `org:X` to `org:Y` to gain access via Y's lead's Template A grant). The concept doc says (line 525): *"there's no mutation grant for the structural tags, so attempts to retag are denied at the permission layer, not at a separate 'validation' step."*

CH-05 (ADR-0044) closed half this story: `validate_published_manifest` Rule C rejects `[Modify]` on the **bare `tag`** fundamental. But Rule C does NOT trigger when a manifest declares `[Modify]` on the **`session_object`** composite (which internally includes Tag) — and `Action::Modify.applies_to_composite(SessionObject) == true` is asserted at [`action.rs:766`](../../../../../../modules/crates/domain/src/permissions/action.rs) (composite-via-constituent-union; CH-04 algebra). So the publish-time guard is incomplete.

CH-12 closes drift D-new-08 by:
1. **Publish-time half** — extend `validate_published_manifest` with **Rule E**: reject `[Modify]` on a composite whose `target_kinds` overlaps the reserved-namespace prefix list (per locked Fork F1.A).
2. **Runtime half** — ship a forward-defensive validator `validate_tag_write_on_session(...)` at `domain::permissions::manifest::validator`. Today no callsite exists (verified: zero hits for `set_tags|update_tags|retag` across `modules/crates/`); the function is the precondition gate any future tag-write Repository method MUST call. Wired into the Repository trait module-level docstring as a precondition note (per locked Fork F2.A).
3. **Audit-event half (F5.B locked)** — ship the `tool.frozen_tag_write_rejected` audit-event builder + `AuditClass::Alerted`, wired into the Repository trait contract docstring. Builder lives at `domain::audit::events::m5_2::tool_authority` (NEW module). Today no production callsite emits the event; the builder is forward-defensive symmetric to F2.A's validator.

### Forks (all user-locked)

All five forks were locked at plan iter 2 (2026-05-04). 4 of 5 match the planner recommendation; F5 diverges (planner recommended F5.A; user locked F5.B).

- **F1.A — New ValidationError variant + Rule E.** Add `CompositeStructuralTagWrite { composite, action, namespace }` and a new validator pass. Defense-in-depth on top of CH-05's Rule C.
- **F2.A — New validator function `validate_tag_write_on_session` at the validator module.** Pure-fn at the Repository boundary; trivially testable; no engine surgery.
- **F3.C — F3.A enforcement today + F3.B-shape `SESSION_FROZEN_TAG_PREFIXES` constant for M6+.** Ship F3.A's enforcement today, but place the full F3.B prefix list as a `pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` in the validator module so the runtime gate is forward-defensive against the M6+ tag-emission expansion.
- **F4.A — Single ADR-0049.** All decisions close drift D-new-08; they share concept-doc anchors. Mirrors CH-05's ADR-0044 shape.
- **F5.B — Emit `tool.frozen_tag_write_rejected` audit event on rejection (USER OVERRIDE).** Planner recommended F5.A (no audit emission, citing "+1 migration column" cost). On user-driven re-investigation, the migration cost was proven zero: the `audit_events` table at [`migrations/0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql) is schema-stable for additive event types (`event_type TYPE string`, `diff FLEXIBLE TYPE object`, `audit_class` already accepts `"alerted"`). `AuditClass::Alerted` already exists at [`audit/mod.rs:39`](../../../../../../modules/crates/domain/src/audit/mod.rs). Hash chain is byte-stable (canonical_bytes excludes prev_event_hash). Net cost: ~0.1 engineer-days, not "+1 migration column".

---

## Decision

### D49.1 — Publish-time Rule E

Extend `validate_published_manifest` with a new rule fired after Rule C (rule ordering: A → C → E → B → D): when

```text
manifest.actions ∋ Action::Modify
  AND manifest.resource ∪ manifest.transitive contains a composite C (excluding MemoryObject — see D49.1.a)
  AND any entry of manifest.target_kinds matches a reserved-namespace prefix from reserved_namespace_prefixes()
```

return `Err(ValidationError::CompositeStructuralTagWrite { composite: C, action: Modify, namespace: <matched-prefix> })`.

Pure rule; no I/O. Composes with Rule C — Rule C still fires for `[Modify] × bare tag`; Rule E covers the composite-via-target_kinds case.

#### D49.1.a — Memory-vs-Session discrimination

Rule E is exempt for `Composite::MemoryObject` because Memory tags ARE intentionally agent-mutable per concept doc 05 lines 24–26 (Memory tags are chosen by the agent at creation; `Action::Modify.applies_to_composite(MemoryObject) == true` is asserted at [`action.rs:766`](../../../../../../modules/crates/domain/src/permissions/action.rs)). Without the exemption, Rule E would conflict with the existing CH-04 algebra; with it, both invariants stay green.

### D49.2 — `ValidationError::CompositeStructuralTagWrite` variant (new)

Carries `{ composite: Composite, action: Action, namespace: String }`. Derives `Debug, Clone, PartialEq, Eq`; impls `Display + Error` via `thiserror::Error`. The enum is now 7 variants (CH-05 shipped 6 per ADR-0044 §D44.2); additive — pre-CH-12 callers matching exhaustively need a new arm. Verified at P1 build time that no existing exhaustive match breaks.

### D49.3 — Runtime validator function

New pure-fn at `domain::permissions::manifest::validator`:

```rust
pub fn validate_tag_write_on_session(
    session_id: SessionId,
    current_tags: &[String],
    proposed_tags: &[String],
) -> Result<(), FrozenTagViolation>
```

Logic: for each tag in `proposed_tags` whose prefix matches any [`SESSION_FROZEN_TAG_PREFIXES`] entry, the tag must appear in `current_tags` (unchanged frozen tags pass). For each tag in `current_tags` whose prefix matches `SESSION_FROZEN_TAG_PREFIXES`, the tag must appear in `proposed_tags` (cannot be removed). Lifecycle tags (`#archived`, `#active`) are unconstrained. Returns `Ok(())` on success; `Err(FrozenTagViolation { session_id, attempted_change: TagChange })` on failure where `TagChange = Added(String) | Removed(String)`.

### D49.4 — `FrozenTagViolation` error type (new)

Lives at `domain::permissions::manifest::validator`. Variants:

- `FrozenTagAdded { session_id, tag }`
- `FrozenTagRemoved { session_id, tag }`

Derives `Debug, Clone, PartialEq, Eq`; impls `Display + Error` via `thiserror::Error`. Exposed for HTTP 422 mapping by future tag-write Repository methods.

### D49.5 — `RepositoryError::FrozenSessionTagWrite { source: FrozenTagViolation }` variant (new)

Added in [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs). Mirrors CH-05's `RepositoryError::ManifestValidation { source: ValidationError }` shape (ADR-0044 §D44.8). Additive on `RepositoryError`; trait method signatures unchanged. Repository contract documented in trait module-level doc-comment as a precondition note for any future tag-write method (no `update_session_tags` method exists today). **(Lands at P2 per the chunk's phase split; this ADR §D49.5 documents the decision; the diff lands in P2.)**

### D49.6 — `SESSION_FROZEN_TAG_PREFIXES` constant (per Fork F3.C)

`pub const SESSION_FROZEN_TAG_PREFIXES: &[&str]` at the validator module. Initial contents (10 prefixes):

```rust
&[
    "#kind:", "session:", "agent:", "project:", "org:",
    "task:", "delegated_from:", "role_at_creation:", "agent_kind:", "derived_from:",
]
```

Cross-references concept doc 05 lines 220–231 in a doc-comment. Tests verify (a) every prefix in `reserved_namespace_prefixes()` for `session_object` constituents is in this list, (b) the 6 M6+ categories are present, and (c) `#archived` / `#active` are NOT in the list.

The 6 M6+ categories (`agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:`) are not yet emitted on Session.tags creation today (`auto_tags_for("session", id)` emits only `#kind:session` + `session:<id>`); their inclusion in `SESSION_FROZEN_TAG_PREFIXES` is forward-defensive. Emission expansion is a separate forward-scope item ("Session structural-tag emission" — M6+).

### D49.7 — Audit-event builder for rejection (per Fork F5.B USER-LOCKED, overrides planner-recommendation F5.A)

New audit-event builder at NEW module [`modules/crates/domain/src/audit/events/m5_2/tool_authority.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/tool_authority.rs):

```rust
pub fn frozen_tag_write_rejected(
    actor: AgentId,
    target_session: SessionId,
    org: OrgId,
    violation: &FrozenTagViolation,
    attempted_at: DateTime<Utc>,
) -> AuditEvent
```

Wires into `audit/events/m5_2/mod.rs` via `pub mod tool_authority;`.

- **Event type:** `"tool.frozen_tag_write_rejected"` (dotted name follows CH-23 pattern; `tool` namespace rather than `consent` / `platform` because the rejection is about a tool-authority assertion failing).
- **Audit class:** `AuditClass::Alerted` (security event class — already exists at `domain/src/audit/mod.rs:39`; used by 12 existing emitters).
- **`org_scope`:** populated from `org` argument; chains within the org per existing hash-chain semantics.
- **`actor_agent_id`:** the `actor` argument (the agent whose tool attempted the write); `target_entity_id`: `Some(NodeId::from_uuid(*target_session.as_uuid()))`.
- **`diff` shape:** `{ "before": null, "after": { "session_id": <id>, "violation_kind": "frozen_tag_added" | "frozen_tag_removed", "tag": <tag>, "attempted_at": <rfc3339> } }` — mirrors `consent.requested` "before:null/after:full-payload" shape for create-style rejection events.

**Repository trait contract docstring update.** The trait module-level docstring's precondition note (per D49.5) MUST also state: *"Failures from `validate_tag_write_on_session` MUST be paired with an `audit.emit(frozen_tag_write_rejected(...))` call before propagating the error."* This makes the contract complete: validator + audit-event are paired at any future tag-write callsite. **(Lands at P2 alongside the D49.5 trait change.)**

**Why F5.B over planner-recommended F5.A.** User prioritises operator-visibility into retag-attempts in the audit log even when there's no production callsite today. The builder is forward-defensive symmetric to F2.A; it ships with tests but no production `audit.emit(...)` callsite — the actual emission lands in a future chunk that wires `update_session_tags` HTTP/CLI endpoint. The F5.B-driven scope cost (~0.1 engineer-days) was estimated at iter-1 as "+1 migration column" but on investigation the migration cost is **zero** because the `audit_events` table is already schema-stable.

**What CH-12 does NOT do per F5.B.**
- Does NOT wire the `audit.emit(...)` callsite (no callsite to wire today).
- Does NOT add an `AuditEmitter` parameter to `validate_tag_write_on_session` (would couple a pure-fn to async emission — investigated and rejected as Option C in iter-2 analysis).
- Does NOT add a migration (audit_events table is schema-stable).

---

## Consequences

### Positive

- Closes drift D-new-08 (HIGH; security-boundary).
- Both publish-time and runtime halves of the immutability invariant are enforced in typed Rust.
- Forward-defensive: any future `update_session_tags` Repository method is forced to call the validator + emit the audit event by the trait docstring contract.
- Audit operators see retag-attempt events in their alert channel within 60s of the future emission point (Alerted-class retention + delivery per `nfr-observability.md`).
- Zero phi-core leverage delta (validator + audit-event builder are wholly governance-layer).
- Zero migration impact (audit_events schema-stable).
- Hash chain stays byte-stable (canonical_bytes excludes prev_event_hash; additive event types do not perturb prior events).

### Negative / cost

- +1 `ValidationError` variant (7 → from 6); existing exhaustive matches need a new arm. Cascade fan-out estimated ≤ 8 sites at planning; verified at implementation.
- +1 `RepositoryError` variant (additive; lands at P2).
- +10 `SESSION_FROZEN_TAG_PREFIXES` entries — 4 are forward-defensive (no auto-emission today). Documented in §D49.6.
- +1 audit event type (additive; no migration; no hash-chain perturbation).

### Carry-forward (open at chunk close)

- Concept doc 05 §"Tag Vocabulary for Sessions" emission gap: 6 M6+ categories (`agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:`) are still NOT auto-emitted on Session.tags creation. CH-12 enforces immutability on whatever subset is currently emitted; emission expansion is a separate forward-scope item ("Session structural-tag emission" — M6+). The §2 row in the plan stays at `partially-honored` for the emission axis at chunk close. Recommended retro candidate: file `D-CH12-FOLLOWUP-01` LOW drift.
- HTTP / CLI `update_session_tags` / `retag_session` endpoint: deferred to a future M6+ tool-admin chunk. CH-12's runtime gate + audit-event builder are forward-defensive; the HTTP endpoint adoption would carry its own §3.C (operator walkthrough).

---

## Cross-references

- **ADR-0044** (CH-05) — `validate_published_manifest` + Rule C precedent for composite-cascade reserved-namespace; the `RESERVED_NAMESPACE_LITERALS` constant + `reserved_namespace_prefixes()` generator that Rule E reuses.
- **ADR-0036** (CH-06) — `Composite::ALL` + `kind_name()` reuse; `auto_tags_for` emits the 2 tags currently auto-applied to sessions.
- **ADR-0037** (CH-06) — `reserved_namespace_prefixes()` generator wired off `Composite::ALL`.
- **ADR-0040** (CH-21) — audit hash chain semantics; D49.7 inherits chain-link symmetry. Verified iter 2 that additive event types do NOT perturb prior events' canonical_bytes.
- **ADR-0041** (CH-21) — memory-extracted event-and-audit precedent; D49.7's builder shape mirrors `memory_extracted` 1:1.
- **ADR-0033** (CH-K8S-PREP) — K8s readiness; confirmed neutral here per plan §3.B (A1–A7 axes all clear for F5.B).
- Audit-event family precedent: [`m5/consents.rs`](../../../../../../modules/crates/domain/src/audit/events/m5/consents.rs) (consent transition events; same builder pattern); [`m5_2/memory.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/memory.rs) (memory-extracted, also `m5_2`-bucketed); [`m4/agents.rs`](../../../../../../modules/crates/domain/src/audit/events/m4/agents.rs) (Alerted-class for security-sensitive changes).
- Concept docs: [`permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) lines 220–231 + 516–525 + 531–541; [`permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) lines 254–261 + 267; [`permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) lines 190–196; [`permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Manifest Validation at Publish Time".
- Drift [D-new-08](../../m5_1/drifts/D-new-08.md) (closed by this ADR).
- Forward-scope row CH-12 (lines 123–128) at [remaining-scope-post-m5-p7-22035b2a.md](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md).
- Plan archive: [`plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md`](../../../../plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md).

---

## Lifecycle

- 2026-05-04 — Drafted as Proposed at CH-12 P1.
- _pending_ — Flipped to Accepted at CH-12 P3 chunk seal (after all phases shipped + 2-agent audit + drift D-new-08 remediated).
