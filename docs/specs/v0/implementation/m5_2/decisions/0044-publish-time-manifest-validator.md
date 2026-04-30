<!-- Last verified: 2026-04-29 by Claude Code -->

# ADR-0044 — Publish-time tool authority manifest validator

**Status: Accepted**

**Date:** 2026-04-29
**Chunk:** CH-05
**Closes:**
- [`D-new-07`](../../m5_1/drifts/D-new-07.md) (HIGH) — publish-time manifest validator missing.
- [`D-new-31`](../../m5_1/drifts/D-new-31.md) (LOW) — reserved-namespace write rejection at tool-publish time (sub-case of D-new-07).

---

## Context

[`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"The Transitive-Grant Match Rule" + [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"What v0 Validates vs Future Enhancements" + [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) §"Reserved Namespace Enforcement" collectively specify a publish-time validator that rejects malformed `ToolAuthorityManifest` entries. Until CH-05, no such validator existed: every field combination persisted, including manifests that declared a composite without `#kind:`, named an action that no fundamental admits, asked for a constraint that didn't apply, or tried to write to a runtime-owned reserved namespace. Drift D-new-07 (HIGH, Bucket A) and D-new-31 (LOW, Bucket C) tracked the gap together; D-new-31 is a strict sub-case of D-new-07 — the reserved-namespace rejection IS one of the four rules the validator must implement.

The exploration phase surfaced two important properties of the current state:
- **Zero production callers.** Today only one test fixture at [`store/tests/repository_test.rs:389`](../../../../../../modules/crates/store/tests/repository_test.rs) constructs a `ToolAuthorityManifest`. There is no `POST /api/v0/tools/publish` HTTP handler. The `ToolDefinition` node is still a 1-field scaffold (M2 [PLANNED]).
- **Excellent reuse surfaces.** CH-04's [`Action::applies_to(Fundamental)`](../../../../../../modules/crates/domain/src/permissions/action.rs) already encodes the matrix Rule B needs. [`Composite::ALL`](../../../../../../modules/crates/domain/src/model/composites.rs) + `kind_name()` powers the reserved-namespace prefix list. [`expand_resource_to_fundamentals`](../../../../../../modules/crates/domain/src/permissions/expansion.rs) handles composite expansion. The validator is mostly composition.

The user-decided forks at plan-review (2026-04-29) locked three open questions:

1. **Wire point** — Both: Repository guard + standalone callable function. The validator is `pub`, so handlers can run it before persistence to surface clean validation errors; both Repository impls call it as a precondition so storage can never bypass it. Defense in depth.
2. **Constraint matrix encoding** — String-based lookup function. A typed `Constraint` enum is deferred to v1 alongside the per-resource `Action` enum redesign (ADR-0043 §D43.9). The string-based `constraint_applies_to` catches the same publish-time bug class as a typed enum would, with much smaller scope.
3. **Publish handler** — Defer. No HTTP / CLI publish surface in CH-05. The validator + repo guard are the architectural gate; whichever future chunk adds the publish flow gets validation for free.

---

## Decision

### D44.1 — Validator module + entry point

New module `domain::permissions::manifest::validator` at [`modules/crates/domain/src/permissions/manifest/validator.rs`](../../../../../../modules/crates/domain/src/permissions/manifest/validator.rs). The existing `manifest.rs` was reshaped into `manifest/mod.rs` via `git mv` (history preserved) so the validator could sit beside the existing `Manifest` projection without bloating one file.

Top-level entry point:

```rust
pub fn validate_published_manifest(
    m: &ToolAuthorityManifest,
) -> Result<Vec<ValidationWarning>, ValidationError>
```

Pure function — no I/O, no repository access. Returns `Err(ValidationError)` on the first hard rejection (operator can fix and resubmit; subsequent rules are not yet visible). Returns `Ok(warnings)` (possibly empty) when all four rules pass; warnings accumulate so all are surfaced at once.

### D44.2 — `ValidationError` variants (6)

Each variant carries the offending detail so the operator can fix and resubmit without re-running validation:

- `MissingKindForComposite { composite: Composite }` — Rule A.a: manifest declares a composite (in `resource` or `transitive`) but does not list its `kind_name()` in `kinds`.
- `KindFundamentalsInconsistent { kind: Composite, declared: HashSet<Fundamental>, expected: HashSet<Fundamental> }` — Rule A.b: `kinds` declares a composite but its `constituents()` are not all present in `resource ∪ transitive`.
- `UnknownResource { name: String }` — `resource` or `transitive` entry names neither a fundamental nor a composite.
- `ActionFundamentalMismatch { action: Action, fundamental: Fundamental }` — Rule B: `(action, fundamental)` pair rejected by CH-04's matrix.
- `ConstraintFundamentalMismatch { constraint: String, fundamentals: HashSet<Fundamental> }` — Rule D: declared constraint applies to none of the manifest's expanded fundamentals (and is not a universal constraint).
- `ReservedNamespaceWrite { namespace: String, action: Action }` — Rule C: manifest declares `[Modify]` on the bare `tag` fundamental.

All variants derive `Debug + Clone + PartialEq + Eq` and impl `Display + Error` via `thiserror`. The `Display` form mentions the offending detail directly so HTTP 422 responses are operator-actionable.

### D44.3 — `ValidationWarning` variants (3)

Non-blocking advisories. Warnings accumulate alongside an `Ok(...)` return; they do not prevent persistence:

- `BlanketKindWildcard` — `kinds == ["*"]` (concept doc 04 line 75 — accepted but operator should narrow).
- `MissingCompositeShorthand { suggested: Composite }` — fundamentals match a composite shape but no composite is declared (concept doc 04 line 110 — readability suggestion).
- `TargetKindsMissingForCreate { composite: Composite }` — `[Create]` declared on a composite but `target_kinds` does not list its kind (concept doc 07 line 814 — runtime needs the hint to auto-tag instances).

### D44.4 — Rule A enforcement (composite/`#kind:` consistency)

For every entry in `manifest.resource ∪ manifest.transitive` that names a composite (`classify_resource_name` returns `Composite(c)`), the composite's `kind_name()` must appear in `manifest.kinds` — UNLESS `manifest.kinds == ["*"]` (the blanket-kind escape hatch, which fires `ValidationWarning::BlanketKindWildcard` and skips the per-composite check).

For every entry in `manifest.kinds` that names a registered composite (i.e., matches a `Composite::ALL.kind_name()`), the composite's `constituents()` set must be a subset of the union of fundamentals declared by `resource ∪ transitive`. Composite-shaped declarations expand to their constituents at classification time, so a declaration of `memory_object` automatically contributes `{DataObject, Tag}` to the expanded set. Unknown kind names are ignored at this layer — they may be org-defined kinds outside the v0 ontology, and a future chunk may tighten this once the ontology is definitive.

### D44.5 — Rule B enforcement (Action × Fundamental matrix)

For every `(action, fundamental)` pair derivable from `actions × declared_fundamentals` (after `classify_resource_name` + composite expansion), `Action::applies_to(Fundamental)` from CH-04 must return true. The validator does NOT re-encode the matrix — it queries CH-04's source-of-truth function. `Action::Wildcard` always passes (CH-04's `applies_to` returns true for it).

### D44.6 — Rule C enforcement (reserved-namespace write rejection)

Trigger: `Action::Modify ∈ actions` AND `"tag"` ∈ `resource ∪ transitive` (the bare `Tag` fundamental as a top-level resource — NOT as a constituent of a composite).

Composites that internally include `Tag` (memory, session, auth_request, inbox, outbox, control_plane, external_service, model_runtime — all 8 composites) remain legitimate when declared as composites: a tool that says `resource: [memory_object] + actions: [Modify]` modifies memory data, NOT the runtime-assigned `#kind:memory` / `memory:<id>` / `delegated_from:*` / `derived_from:*` tags. The bare-`tag` interpretation is the cleanest publish-time discriminator; manifests don't carry actual selector strings (those live on Grants), so a more granular check would require parsing tag predicates in the validator — bigger scope without catching different bugs.

The full reserved-namespace prefix list — `#kind:`, `delegated_from:`, `derived_from:`, plus one `{kind_name}:` per `Composite::ALL` — ships via `pub fn reserved_namespace_prefixes() -> Vec<String>`. The list is generated at runtime from `Composite::ALL` so adding a new composite auto-grows it without an edit. CH-12's frozen-session-tag enforcement will consume this list directly.

### D44.7 — Rule D enforcement (Constraint × Fundamental matrix)

For every constraint name in `manifest.constraints`, the constraint must apply to at least one fundamental in the manifest's expanded set, OR be a universal constraint (`time_window`, `approval_requirement`, `non_delegability`, `purpose`).

The matrix lives at `pub fn constraint_applies_to(name: &str, fundamental: Fundamental) -> bool` — a hard-coded `match` mirroring concept doc `permissions/03-action-vocabulary.md` lines 78–88 verbatim. The 9 specific constraints (`path_prefix`, `command_pattern`, `domain_allowlist`, `tag_predicate`, `max_size_bytes`, `max_spend`, `sandbox_requirement`, `output_channel`, `timeout_secs`) plus the 4 universal constraints are exhaustively unit-tested — 81 cells transcribed directly from the concept doc, plus 36 universal-always-pass cells.

Unknown constraint names are surfaced as `ConstraintFundamentalMismatch` with the manifest's expanded fundamentals set (since "unknown" applies to none of the matrix). This is intentionally strict — operators should declare constraints from the v0 vocabulary, and the rejection message names the offending string for fix-and-resubmit.

### D44.8 — Repository-guard wiring

`RepositoryError::ManifestValidation { source: ValidationError }` added in [`domain::repository`](../../../../../../modules/crates/domain/src/repository.rs). The variant carries the structured `ValidationError` (not a stringified message) so future HTTP handlers can map to specific error codes (typically 422 Unprocessable Entity per HTTP semantics for validation failures).

Both `Repository::create_tool_authority_manifest` impls call the validator as the first line of the method body:

- [`SurrealStore::create_tool_authority_manifest`](../../../../../../modules/crates/store/src/repo_impl.rs) — validator runs BEFORE the SurrealDB CREATE, so partial-state failures are impossible.
- [`InMemoryRepository::create_tool_authority_manifest`](../../../../../../modules/crates/domain/src/in_memory.rs) — same shape; the in-memory `HashMap.insert` runs only after validation passes.

The `Repository` trait method's doc-comment documents the contract so any future impl inherits it.

Validator warnings are dropped at the repo boundary. Callers wanting them call `validate_published_manifest` directly first — that's the "standalone function" half of the locked Q1 decision (defense in depth).

### D44.9 — Out of scope: HTTP publish handler

Per locked Q3 (2026-04-29), CH-05 does NOT add a `POST /api/v0/tools/publish` handler. There is no production publish flow today (only one test fixture creates manifests). When a future chunk wires the publish flow (likely a CH-2X tool-admin chunk or part of the M2 ToolDefinition materialization), the validator + repo guard are already in place — that chunk calls `validate_published_manifest` directly first to surface warnings + structured errors before persistence, then calls `repo.create_tool_authority_manifest` (which re-validates as the safety net). The cost of re-validating is negligible (pure function, no I/O); the safety guarantee is absolute.

### D44.10 — Out of scope: typed Constraint enum

Per locked Q2 (2026-04-29), `Manifest.constraints` and `ToolAuthorityManifest.constraints` stay `Vec<String>`. A typed `Constraint` enum mirroring CH-04's `Action` enum is deferred to a v1 chunk that bundles it with the per-resource Action enum redesign (ADR-0043 §D43.9). Rationale:

- The string-based `constraint_applies_to` function catches the same bug class a typed enum would (publish-time rejection of constraint × fundamental mismatch). Same publish-time stage, same operator experience.
- A typed enum would touch `Manifest.constraints`, `Grant.constraints`, the engine's Step 4 constraint-check loop, and the `Manifest.constraint_requirements: HashMap<String, Value>` map (which needs typed keys too). ~1 day extra.
- Bundling the v1 Action redesign + Constraint enum together lets both share the per-resource design pattern (each resource class would expose its valid Actions AND valid Constraints). Splitting them now would mean two enum migrations later — wasted effort.

The string-based path is sufficient at v0.1: operators get clear publish-time rejection messages, the audit chain is byte-stable, and the validator's matrix is exhaustively tested. Future v1 chunk can promote constraints to a typed enum without touching the validator's rule structure.

---

## Conforming criteria — none (publish-time guard, not a swappable trait)

ADR-0044 is a publish-time guard ratification, not a trait-shape conforming-criteria decision. The validator's API surface is fixed (one `validate_published_manifest` function + 6 error variants + 3 warning variants); future evolution is via additional rules (additive enum variants) or rule refinement, not pluggable backends.

---

## Alternatives considered

- **Standalone function only (no Repository guard).** Rejected per locked Q1 — easy to forget; if a future handler skips the call, malformed manifests reach the DB. The Repository guard is the safety net.
- **Repository guard only (no public function).** Rejected per locked Q1 — handlers can't show validation errors before touching the DB; UX suffers. Public function gives clean error surface.
- **Validate engine-internal `Manifest` projections too.** Rejected — engine `Manifest` instances are constructed in tests + handlers from already-trusted nodes (or hand-rolled for special paths like `secrets/reveal.rs`'s reveal-purpose contract). Validating those would surface false positives without catching anything new. The `Manifest` shape is intentionally permissive at v0; the persisted `ToolAuthorityManifest` is the security-relevant boundary.
- **Migrate constraints to typed enum at CH-05.** Rejected per locked Q2 — bigger scope; deferred to v1.
- **Add HTTP publish handler at CH-05.** Rejected per locked Q3 — no production publish flow exists; would be premature wiring.
- **Strict `target_kinds`-based reserved-namespace check.** Rejected — the plan's first sketch had Rule C inspect `target_kinds` for any reserved-namespace prefix, but every `target_kinds` entry IS a composite kind name (which IS a reserved namespace), making the check too aggressive. The bare-`tag` interpretation is the cleanest discriminator that matches the concept-doc spirit ("runtime-assigned tag namespaces cannot be written by tools") without false positives on legitimate composite-modify tools.

---

## Out of scope

See D44.9 (HTTP handler) and D44.10 (typed Constraint enum). Tracked successors (none required at v0.1 — speculative if a real driver appears):

- **Future:** `POST /api/v0/tools/publish` handler — opens as a new chunk when ToolDefinition node materialization is wired (likely paired with M2's full ToolDefinition shape).
- **Future:** Typed `Constraint` enum — bundled with v1 per-resource Action enum redesign (ADR-0043 §D43.9).
- **CH-12 (frozen session-tag immutability):** consumes `reserved_namespace_prefixes()` for runtime-side gating of structural-tag mutations; the publish-time half is now in place.

---

## References

- Concept docs (header bumped at this chunk; bodies UNCHANGED):
  - [`permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) — Rule A source.
  - [`permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) — full v0 rule list.
  - [`permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) — Rule C read-vs-write asymmetry.
  - [`permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) — Reserved tag namespace catalog.
  - [`permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) — Constraint × Fundamental matrix (Rule D source) + universal constraint list.
- Drifts (closed at this chunk):
  - [`D-new-07.md`](../../m5_1/drifts/D-new-07.md) — HIGH, parent.
  - [`D-new-31.md`](../../m5_1/drifts/D-new-31.md) — LOW, sub-case of Rule C.
- Plan archive: [`build/ch-05-publish-time-manifest-validator-6bf47d46.md`](../../../../plan/build/ch-05-publish-time-manifest-validator-6bf47d46.md).
- Architecture doc: [`m1/architecture/permission-check-engine.md`](../../m1/architecture/permission-check-engine.md) — header bumped; the engine assumes validated manifests.
- Paired ADR: [ADR-0043](0043-typed-action-vocabulary.md) — CH-04's Action enum + matrix; this chunk's Rule B queries `Action::applies_to` directly.
- Downstream chunk: **CH-12** (frozen session-tag immutability) — consumes `reserved_namespace_prefixes()` for runtime-side enforcement.
- Code:
  - [`modules/crates/domain/src/permissions/manifest/validator.rs`](../../../../../../modules/crates/domain/src/permissions/manifest/validator.rs) — the new module.
  - [`modules/crates/domain/src/permissions/manifest/mod.rs`](../../../../../../modules/crates/domain/src/permissions/manifest/mod.rs) — reshaped from `manifest.rs` (history preserved via `git mv`).
  - [`modules/crates/domain/src/permissions/mod.rs`](../../../../../../modules/crates/domain/src/permissions/mod.rs) — re-exports.
  - [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) — `RepositoryError::ManifestValidation` variant + trait method doc.
  - [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) — SurrealDB impl wires validator at top.
  - [`modules/crates/domain/src/in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs) — InMemoryRepository impl wires validator at top.
  - [`modules/crates/server/tests/acceptance_manifest_validator.rs`](../../../../../../modules/crates/server/tests/acceptance_manifest_validator.rs) — per-rejection-class acceptance suite (9 tests).
