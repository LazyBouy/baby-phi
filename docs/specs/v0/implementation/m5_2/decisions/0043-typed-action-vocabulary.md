<!-- Last verified: 2026-04-29 by Claude Code -->

# ADR-0043 — Typed action vocabulary + Action × Fundamental applicability matrix

**Status: Accepted**

**Date:** 2026-04-29
**Chunk:** CH-04
**Closes:**
- [`D-new-09`](../../m5_1/drifts/D-new-09.md) (HIGH) — concept doc specifies a closed 33-verb action vocabulary, code stored actions as free-form `Vec<String>`.
- [`D-new-10`](../../m5_1/drifts/D-new-10.md) (MED) — concept doc specifies a 9×10 Action × Fundamental applicability matrix, code did not enforce or even encode it.

---

## Context

Drifts D-new-09 + D-new-10 record a load-bearing gap between the concept doc and the implementation:

[`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"Standard Action Vocabulary" specifies a closed set of canonical action verbs in 10 categories, plus a 9×10 applicability matrix (lines 27–37) recording which category applies to which fundamental. The concept doc is the source of truth; both are normative requirements. The implementation as of CH-04 chunk-open carried action lists as `Vec<String>` everywhere — `Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions` — with no compile-time check, no parser, and no matrix enforcement. A typo (`raed` for `read`) silently passed through the entire pipeline; a category-mismatched verb (`recall` on a `filesystem_object`) was caught only by Step 3's grant-vs-manifest match, not at publish time.

CH-04 closes both drifts together. Neither makes sense without the other: a typed enum with no matrix to query gives type-safety on construction but not semantic validity; a matrix without a typed enum has no closed domain to range over. Shipping them as a paired chunk avoids the half-measure.

The user-decided forks at plan-review (2026-04-28) locked three open questions:

1. **Wildcard handling** — the existing `"*"` escape hatch (used by `system:root` and bootstrap-claim grants) becomes a `Wildcard` variant on the enum, serialized as `"*"` for wire compat.
2. **`launch_session` synthetic action** — the one outlier callsite in [`sessions/preview.rs:86`](../../../../../../modules/crates/server/src/platform/sessions/preview.rs) does not get a new vocabulary verb. It uses canonical `Action::Invoke` (Execution category). The concept doc's 34-verb vocabulary stays unchanged.
3. **Migration scope** — atomic. The three carrier types flip from `Vec<String>` to `Vec<Action>` in one chunk; all ~60 callsites migrate together. Wire format is preserved via `#[serde(rename_all = "snake_case")]` so the audit hash chain stays byte-stable.

Note on count: the plan said "33 canonical" verbs. Counting the concept doc's 10 categories yields 3+3+4+3+3+5+3+4+3+3 = **34** canonical verbs. The implemented enum has 35 total variants (34 canonical + Wildcard). The plan's "33" was a drafting miscount; ADR-0043 records the actual count from the source-of-truth concept doc.

---

## Decision

### D43.1 — `Action` enum at `domain::permissions::action::Action`

35 variants total: 34 canonical verbs from `concepts/permissions/03-action-vocabulary.md` lines 10–21, plus one `Wildcard` variant for the `"*"` escape hatch. Each canonical variant maps 1:1 to a concept-doc verb; variant order matches concept-doc order (Discovery → Data → Mutation → Execution → Connection → Authority → Memory → Configuration → Economic → Observability).

The enum derives `Debug + Clone + Copy + PartialEq + Eq + Hash + PartialOrd + Ord + Serialize + Deserialize`. The `PartialOrd + Ord` derives are needed by the engine's deterministic-ordering paths (`step_1_expand_manifest` reach sort, `constraint_violation` key sort) — without them the engine would have to sort by string-form, which would be a needless allocation per check.

`#[serde(rename_all = "snake_case")]` ensures wire-format compatibility: `Action::Read` round-trips as `"read"`, identical to the legacy `Vec<String>` storage. `Action::Wildcard` carries a `#[serde(rename = "*")]` override so it round-trips as `"*"`.

### D43.2 — Iteration + parsing surface

- `Action::ALL: [Action; 35]` — every variant in canonical order, Wildcard last.
- `Action::CANONICAL: [Action; 34]` — all variants except Wildcard. Useful for matrix-iteration paths where the wildcard is a meta-value, not a per-cell verdict.
- `Action::as_str(&self) -> &'static str` — canonical wire form.
- `Action::category(&self) -> Option<ActionCategory>` — Some for the 34 canonical variants, None for Wildcard.
- `impl Display for Action` — uses `as_str()` so `format!("{}", Action::Read)` produces `"read"`. Needed by handlers like `denial_to_api_error` that interpolate the failed action into error messages.
- `TryFrom<&str> for Action` + `FromStr for Action` — accepts any canonical wire string + `"*"`. Returns `ParseActionError { input: String }` on unknown strings (case-sensitive parsing — `"READ"` rejected). The error type implements `Display` + `Error`.

### D43.3 — Applicability matrix at `Action::applies_to(Fundamental) -> bool`

The 9 × 10 = 90-cell matrix from `concepts/permissions/03-action-vocabulary.md` lines 27–37 is encoded in `ActionCategory::applies_to(Fundamental)`. `Action::applies_to` delegates to its category's verdict; `Action::Wildcard` returns `true` for every fundamental.

`Action::applies_to_composite(Composite) -> bool` derives the composite's applicable-actions set as the union of its constituents' applicable actions, per concept-doc line 39 ("Composite inheritance: applicable actions for a composite are the union of its constituents' actions"). The composite's `constituents()` method (already shipped in [`model::composites`](../../../../../../modules/crates/domain/src/model/composites.rs)) is the source of truth for the constituent set.

The matrix is exhaustively asserted in 306 cells (9 fundamentals × 34 canonical actions) by [`permissions::action::tests::applies_to_matrix_exhaustive_against_concept_doc`](../../../../../../modules/crates/domain/src/permissions/action.rs). Each row of the assertion table is transcribed verbatim from the concept doc; if either the doc or the code drifts, the test fails.

### D43.4 — Carrier type migration

Three carriers flip from `Vec<String>` to `Vec<Action>`:

- [`Grant.action`](../../../../../../modules/crates/domain/src/model/nodes.rs) at line 627.
- [`Manifest.actions`](../../../../../../modules/crates/domain/src/permissions/manifest.rs) at line 38 — the engine-facing projection.
- [`ToolAuthorityManifest.actions`](../../../../../../modules/crates/domain/src/model/nodes.rs) at line 775 — the persisted graph-node form.

Migration is atomic per the user-decided fork (Q3, plan-review 2026-04-28). All ~60 construction sites + ~20 test-fixture assertion sites flip together in this chunk. No SurrealDB schema migration is needed; the on-disk JSON shape is preserved via `#[serde(rename_all = "snake_case")]`.

The store crate's [`GrantRow.action`](../../../../../../modules/crates/store/src/repo_impl.rs) row-translator field is also `Vec<Action>` after migration — the SurrealDB column type is JSON array so this is a Rust-type-only change; the wire format on the column stays identical.

### D43.5 — Read-side migration (engine internals)

The engine's `ReachKey = (Fundamental, Action)` (was `(Fundamental, String)`). Three [`DeniedReason`](../../../../../../modules/crates/domain/src/permissions/decision.rs) variants flip from `String` to `Action`:

- `NoMatchingGrant.action`
- `ScopeUnresolvable.action`
- (`ResolvedReach.action` — also flips, but it's a payload type rather than a denial.)

The `Decision::resolved_grants_map() -> HashMap<(Fundamental, Action), GrantId>` follows. The engine's `expansion::ResolvedGrant::covers(Fundamental, Action) -> bool`, `engine::ceiling_admits` (Step 2a), and `step_1_expand_manifest -> Vec<(Fundamental, Action)>` all migrate to typed values. Wire-form serialization is byte-stable for every audit-event payload (verified by the existing decision-JSON round-trip test at [`permission_check_worked_trace.rs::decision_json_round_trip_preserves_content`](../../../../../../modules/crates/domain/tests/permission_check_worked_trace.rs)).

This is a deeper migration than the plan §5 first listed. The plan listed the 3 carrier types; the engine internals (`ReachKey`, `DeniedReason`, `ResolvedReach`) inherit the migration because keeping them as `String` would require a per-call `Action::as_str()` round-trip on every match — a needless allocation that would mask wire-format drift bugs. Type safety end-to-end is the cleaner answer; the implementation honors that.

### D43.6 — Wildcard semantics in the engine

`Action::Wildcard` matches any required action in two paths:

1. [`expansion::ResolvedGrant::covers`](../../../../../../modules/crates/domain/src/permissions/expansion.rs) — Step 3 reach matching.
2. [`engine::ceiling_admits`](../../../../../../modules/crates/domain/src/permissions/engine.rs) — Step 2a ceiling enforcement.

Both checks now compare against `Action::Wildcard` rather than the literal string `"*"`. Behavioral equivalent — the wire form is unchanged, so audit-event byte-stability is preserved; serialized `Wildcard` produces the same `"*"` string the prior code stored.

The wildcard is documented as a privileged escape hatch. Only system-tier grants (the bootstrap claim's `[allocate]` on `system:root`, and any future `system:genesis`-class grants) should carry it. The publish-time validator at CH-05 will reject manifests that try to declare `Wildcard` in their actions.

### D43.7 — `launch_session` → `Action::Invoke`

The synthetic `"launch_session"` action used by [`sessions/preview.rs:86`](../../../../../../modules/crates/server/src/platform/sessions/preview.rs) is replaced with canonical `Action::Invoke` (Execution category). The concept doc's 34-verb vocabulary stays exactly what it lists; the one outlier callsite conforms instead of growing the vocabulary by one (which would have required a concept-doc change + matrix-cell additions for every fundamental that the launch path could reach).

The operator-visible behavior is identical: a session-launch preview still goes through Step 3 reach matching against `(SessionObject's constituents, Invoke)`, which decomposes via composite inheritance into reaches against `data_object` and `tag`. Both fundamentals admit `Invoke` only if the manifest declares so via `actions=[Invoke]` — same gating semantics, canonical vocabulary.

### D43.8 — Out of scope for CH-04 (locked at plan-review)

The matrix is **shipped as a queryable function**, not enforced at every grant-creation site. CH-04 ships `Action::applies_to(Fundamental) -> bool`, ready to use. The actual rejection rule lands in two follow-up chunks:

- **CH-05** — publish-time validator that rejects manifests asserting `(Fundamental, Action)` pairs the matrix marks `false` (e.g., a manifest with `actions=[Recall]` and `resource=[filesystem_object]` would fail validation because `Recall` only applies to `Tag`).
- **CH-08** — allocate/transfer cardinality rules that pattern-match on `Action::Allocate` / `Action::Transfer`.

Permission Check engine semantics are unchanged at CH-04. The engine still runs the same 6+2a stages over the same data; it just operates on typed values now. CH-05's validator is a sibling enforcement path (publish-time, not invocation-time) that uses the matrix as its rule set.

### D43.9 — Deferred to v1: per-resource action enums

A more rigorous design — one `enum FilesystemObjectAction`, `enum MemoryObjectAction`, etc. per fundamental, with `Grant<R>` generic over a `ResourceClass` trait that has an associated `Action` type — would give compile-time enforcement of "this action is valid for this resource type." A construction like `Grant<FilesystemObject> { action: vec![Recall] }` would fail to compile rather than being caught by the publish-time validator at CH-05.

Rationale for deferral to v1, not v0.1:

- **Concept-doc restructuring required.** Today the concept doc has one 34-verb vocabulary table + one applicability matrix. The per-resource-enum design wants per-fundamental sections, each listing its own enum's verbs — about 9 × ~5-10 verbs each. Mechanical to write, but it's a different doc structure.
- **Per-Composite enum unions.** Composites (8 of them) would need derived enum unions per their `constituents()`. The compile-time machinery to express this in Rust (associated types, marker traits, GATs) is non-trivial.
- **Wildcard reworking.** `*` as a single variant disappears under the per-resource design — each resource enum needs its own wildcard, or the wildcard moves to a wrapper type. Either way, the bootstrap-claim grant's path needs reshaping.
- **Repository trait surface change.** Many `Repository` methods (`create_grant`, `list_grants_for_principal`, etc.) work over `Grant` regardless of resource class. Going generic means reshaping that surface — about ~10 methods touched, each needing a generic-aware caller migration.

Estimated effort: **2–3× the CH-04 scope.** The compile-time-vs-publish-time safety improvement is real but marginal: operators load manifests through the validator, which catches the same class of bugs at effectively the same stage. The flat-enum + matrix design shipped at v0.1 is sufficient for the v0.1 threat model. The concept doc's new § "Future work" subsection records the deferral and cross-references this sub-decision.

---

## Conforming criteria — none (architecture-decision ADR, not trait-shape)

ADR-0043 is a typed-vocabulary design decision, not a trait-shape conforming-criteria decision. Future work (v1 per-resource enum design) lands as a successor ADR if pursued; if not, this ADR remains the v0.1 + v1 baseline for the action vocabulary.

---

## Alternatives considered

- **Per-resource action enums (the v1 design above).** Deferred per D43.9. Fully designable, ~2-3× the engineering cost, marginal gain over flat-enum + publish-time validator at the v0.1 scale.
- **Newtype string wrapper (e.g., `pub struct Action(String)`).** Rejected — gains no compile-time safety over `String` directly; only adds a layer of allocation. The point of the enum is closed-set typing, which a newtype doesn't deliver.
- **Two-tier design — typed verbs + an `Other(String)` escape hatch.** Rejected — the concept doc's vocabulary is closed by design. An `Other` variant would re-open the typo class of bug and make the matrix uncomputable for that variant.
- **No `PartialOrd + Ord` on Action.** Rejected — the engine sorts reaches deterministically per the concept-doc spec, and the natural sort key is the canonical-form variant order. Without `Ord`, every sort would have to allocate via `as_str()` per element.
- **Migrate `AuthRequest.scope` too.** Rejected at CH-04 scope. AuthRequest's `scope: Vec<String>` carries the same semantic vocabulary, but D43.4 listed only the three carriers (Grant, Manifest, ToolAuthorityManifest). Locking that line in the user-decided forks meant AuthRequest.scope stays as `Vec<String>` at v0.1. A future chunk can extend the migration if cost-justified.
- **Add new actions to the vocabulary.** Considered but rejected — the concept doc's 34-verb list is the source of truth for v0. Growing the vocabulary requires a concept-doc revision + a matrix amendment + a renegotiation; CH-04 is doc-fidelity work, not vocabulary expansion.

---

## Out of scope

See D43.8 (CH-05/CH-08 deferrals) and D43.9 (v1 per-resource enum design). Tracked successors:

- **CH-05** — publish-time validator wires `Action::applies_to` into manifest publication as a rejection rule.
- **CH-08** — allocate/transfer cardinality + the explicit handling of the umbrella-action semantics.
- **Future:** v1 per-resource action enum redesign, only if a concrete failure mode surfaces that the publish-time validator can't catch.

---

## References

- Concept doc: [`permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) — vocabulary + matrix source of truth (header bumped at this chunk; new § "Future work — v1 revisit: per-resource action enums" appended).
- Drifts:
  - [`D-new-09.md`](../../m5_1/drifts/D-new-09.md) — closed at CH-04.
  - [`D-new-10.md`](../../m5_1/drifts/D-new-10.md) — closed at CH-04.
- Plan archive: [`build/3a65a2fc-ch-04-typed-action-vocabulary.md`](../../../../plan/build/3a65a2fc-ch-04-typed-action-vocabulary.md).
- Architecture doc: [`m1/architecture/permission-check-engine.md`](../../m1/architecture/permission-check-engine.md) — header bumped at this chunk.
- Downstream chunks: CH-05 (publish-time validator), CH-08 (allocate/transfer cardinality), CH-15 (real permission manifest at session launch).
- Code:
  - [`modules/crates/domain/src/permissions/action.rs`](../../../../../../modules/crates/domain/src/permissions/action.rs) — the new module.
  - [`modules/crates/domain/src/permissions/mod.rs`](../../../../../../modules/crates/domain/src/permissions/mod.rs) — re-exports.
  - [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) — `Grant.action` + `ToolAuthorityManifest.actions` carrier types.
  - [`modules/crates/domain/src/permissions/manifest.rs`](../../../../../../modules/crates/domain/src/permissions/manifest.rs) — `Manifest.actions` carrier type.
  - [`modules/crates/domain/src/permissions/decision.rs`](../../../../../../modules/crates/domain/src/permissions/decision.rs) — `DeniedReason` + `ResolvedReach`.
  - [`modules/crates/domain/src/permissions/engine.rs`](../../../../../../modules/crates/domain/src/permissions/engine.rs) — `ReachKey` + ceiling/match logic.
  - [`modules/crates/domain/src/permissions/expansion.rs`](../../../../../../modules/crates/domain/src/permissions/expansion.rs) — `ResolvedGrant::covers`.
- Paired ADRs:
  - [ADR-0033](0033-k8s-prep-refactors.md) — trait-shape-with-conforming-criteria pattern (precedent).
  - [ADR-0042](0042-storage-backend-configurable.md) — most recent ADR; same chunk-seal-day cross-reference style.
