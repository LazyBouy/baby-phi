<!-- Last verified: 2026-04-29 by Claude Code -->

# CH-05 — Publish-time tool authority manifest validator

**Plan file token:** `6bf47d46` (generated via `openssl rand -hex 4` at chunk-open 2026-04-29).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/ch-05-publish-time-manifest-validator-6bf47d46.md`.
**Chunk ID:** CH-05 (forward-scope §1 lines 70–75; §5 inventory row line 413).
**Severity:** ⚠ HIGH.
**Expected effort:** ~2 engineer-days.
**Hard prerequisites:** **CH-04** (typed actions — already sealed; ADR-0043 Accepted).
**Chunks unblocked at close:** CH-12 (frozen-tag enforcement reuses validator + reserved-namespace constant).

---

## Context

### The simple version

Today, when a tool ships its **authority manifest** (the YAML/JSON declaring "I touch the filesystem, I make network calls, I need a `purpose` constraint"), nothing checks that the declaration is internally consistent. A manifest can:

- Declare `resource: memory_object` but forget to set `kinds: [memory]` — the runtime then has no way to tell which composite the tool actually targets.
- Declare `actions: [recall]` on `resource: filesystem_object` — combination is meaningless (recall only applies to tags), but the manifest stores fine.
- Declare `constraints: [path_prefix]` on `resource: network_endpoint` — wrong constraint for that fundamental.
- Declare `actions: [modify]` with a selector matching `#kind:*` — that would let the tool overwrite system-assigned identity tags.

Each of these is a security or correctness bug. CH-05 closes them all by adding a single function — `validate_published_manifest(&ToolAuthorityManifest)` — that runs the four checks above before the manifest is persisted. Drift D-new-07 records the gap; the concept doc has specified this validator since M0.

That's the chunk. Build the validator, hard-wire it into the `Repository::create_tool_authority_manifest` boundary so future code paths can't forget to call it, ship the reserved-namespace constant + the constraint matrix it needs, and acceptance-test each rejection class. Two days of focused work.

### What this chunk does NOT do

- Does NOT add a `POST /api/v0/tools/publish` HTTP handler. There is no production publish flow today (only one test fixture creates manifests). When a future chunk wires up tool publication, the validator is already in place. Out of scope per locked Q3 (2026-04-29).
- Does NOT migrate `Manifest.constraints` to a typed enum. Constraints stay `Vec<String>`; the validator queries them via a `constraint_applies_to(&str, Fundamental)` lookup function. Mirrors CH-04's flat-enum + matrix shape; per-resource constraint enums are deferred to v1 alongside D43.9 (per-resource action enums).
- Does NOT change the Permission Check engine semantics at runtime. Step 0–6 stays unchanged; the validator is a publish-time precondition, not a runtime gate. The runtime continues to assume already-validated manifests.
- Does NOT enforce the validator on engine-internal `Manifest` projections — only on the persisted `ToolAuthorityManifest` shape. Engine-side `Manifest` instances are constructed in tests + handlers from already-trusted nodes (or hand-rolled for special paths like `secrets/reveal.rs`'s reveal-purpose contract); validating those would surface false positives without catching anything new.
- Does NOT migrate the existing test fixture at `repository_test.rs:389` to the new validator without surface scrutiny — the test is verifying trait-impl persistence, not manifest correctness; if the fixture currently constructs an internally-inconsistent manifest, P2 will adjust it to satisfy the validator.

### User-decided forks (locked at plan-review, 2026-04-29)

1. **Wire point** — **Both: Repository guard + standalone function.** `permissions::manifest::validator::validate_published_manifest(&ToolAuthorityManifest) -> Result<Vec<ValidationWarning>, ValidationError>` is publicly callable; both `Repository::create_tool_authority_manifest` impls (SurrealDB + InMemory) call it as a precondition, returning a new `RepositoryError::ManifestValidation { source: ValidationError }` variant on failure. Defense in depth — handlers can show clean validation errors before touching the DB; the repo guard is the safety net.

2. **Constraint matrix encoding** — **String-based lookup function.** `constraint_applies_to(name: &str, fundamental: Fundamental) -> bool` queries a hard-coded match table mirroring the concept doc's Constraint × Fundamental matrix at `permissions/03-action-vocabulary.md` lines 78–88. No new enum at v0; bundling Constraint and Action enum migrations into a v1 chunk later (when both lock into per-resource shapes) is cleaner than splitting them now.

3. **Publish handler** — **Defer to a later chunk.** No HTTP / CLI publish surface in this chunk. Validator + repo guard only. The validator is the architectural gate; whichever future chunk adds the publish flow gets validation for free.

### Forward-scope reference

[CH-05 row](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (lines 70–75) + [§5 inventory](baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 413).

---

## §1 — Why this chunk (one paragraph)

Tool authority manifests are the contract between tool authors and the permission system: "to use me, you need these capabilities." Today that contract is unenforced — `ToolAuthorityManifest` accepts any combination of fields. A misdeclared composite (`resource: memory_object` without `kinds: [memory]`), a nonsense action/fundamental pair (`recall` on `filesystem_object`), an unsuitable constraint (`path_prefix` on `network_endpoint`), or a write to a runtime-owned reserved namespace (`actions: [modify]` against `#kind:*`) all silently persist. The runtime then either denies in surprising ways or, worse, succeeds at something no concept-doc rule allows. CH-05 closes the gap with a single `validate_published_manifest` function plus a Repository-level precondition guard that blocks malformed manifests from ever reaching SurrealDB. Drift D-new-07 (general validator) and D-new-31 (reserved-namespace write rejection) close together; the constant + the matrix machinery the validator ships also unblocks CH-12's frozen-session-tag enforcement.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Status at chunk-close |
|---|---|---|---|---|
| [`permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) | § "The Transitive-Grant Match Rule" lines 67–75 | Publish-time validator rejects: (a) declared composite missing `#kind:`, (b) `#kind:` without matching fundamentals, (c) fundamentals inconsistent with declared composites; `#kind: *` accepted with warning | silent-in-code | honored — `validate_published_manifest` Rule A enforces a/b/c with a Vec<Warning> for the `*` case |
| [`permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) | § "What v0 Validates vs Future Enhancements" lines 803–814 | v0 declaration-only validation: composite/`#kind:` + fundamental superset + Action × Fundamental + Constraint × Fundamental + reserved-namespace-write rejection + `target_kinds` consistency warning | silent-in-code | honored — Rules A/B/C/D cover the five hard rejections; `target_kinds` warning shipped as a non-blocking `ValidationWarning::TargetKindsMissing` |
| [`permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) | § "Reserved Namespace Enforcement" lines 190–196 | Selector grammar accepts reserved tags (read OK); validator rejects `[modify]` on reserved namespaces (write rejected) | silent-in-code | honored — Rule C enforces the read-vs-write asymmetry exactly; selector parsing already accepts reserved tags (CH-06 prior work) |
| [`permissions/01-resource-ontology.md`](baby-phi/docs/specs/v0/concepts/permissions/01-resource-ontology.md) | § "Reserved tag namespaces" lines 254–260 | Reserved namespaces: `#kind:*`, `{kind}:*` (per registered composite), `delegated_from:*`, `derived_from:*` | silent-in-code | honored — `RESERVED_NAMESPACE_PREFIXES` constant generated from `Composite::ALL` for `{kind}:*`, plus the three other literals |

The four concept doc sections collectively define the validator's spec. CH-05 doesn't *modify* the concept docs — it lifts their validator rules into executable Rust.

---

## §3 — phi-core leverage map

| phi-core type | Action in this chunk |
|---|---|
| (none) | — |

The validator is baby-phi-native. phi-core has no concept of tool authority manifests, reserved namespaces, or applicability matrices (it's the agent-loop library; manifest validation lives one layer up). Zero phi-core imports added or removed.

**Positive close-audit greps**:
```bash
grep -n "fn validate_published_manifest\b" modules/crates/domain/src/permissions/manifest/validator.rs   # 1
grep -n "pub enum ValidationError\b\|pub enum ValidationWarning\b" modules/crates/domain/src/permissions/manifest/validator.rs  # ≥ 2
grep -n "fn constraint_applies_to\b" modules/crates/domain/src/permissions/manifest/validator.rs        # 1 (or in a sibling matrix.rs)
grep -n "RESERVED_NAMESPACE_PREFIXES\b" modules/crates/domain/src/permissions/manifest/validator.rs    # 1
grep -n "ManifestValidation\b" modules/crates/domain/src/repository.rs                                  # ≥ 1 (RepositoryError variant)
grep -rn "validate_published_manifest" modules/crates/store/src/ modules/crates/domain/src/in_memory.rs # ≥ 2 (precondition guard wired in both repo impls)
ls modules/crates/domain/src/permissions/manifest/validator.rs                                          # exists
```

**Forbidden-duplication / regression greps**:
```bash
grep -rn "use phi_core::" modules/crates/domain/src/permissions/manifest/   # 0
grep -rn 'fn validate_manifest\b' modules/crates/                            # 0 (no parallel validator from another author)
```

---

## §3.B — K8s readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | None. `validate_published_manifest` is a pure function over a `&ToolAuthorityManifest` reference. | No |
| **A2** IPC channels | None. Wire format unchanged — manifests still serialize as JSON. | No |
| **A3** pod-local resources | None. | No |
| **A4** migration runner | No SurrealDB schema migration. Validation runs in domain code before persistence. | No |
| **A5** trait-shape requirement | `RepositoryError` gains one variant (`ManifestValidation { source: ValidationError }`). All existing trait method signatures unchanged; the variant is additive on the error enum. | No |
| **A6** cross-pod state sharing | Wire format unchanged → cross-pod gossip is byte-for-byte identical. | No |
| **A7** audit hash-chain symmetry | No new audit-event variant. Validator failures emit no audit (callers map them to HTTP 422 or equivalent). The hash chain stays untouched. | No |

**Conclusion:** K8s-neutral. No M7b ledger entry added.

---

## §3.C — User-facing documentation impact

| Tier | File | Action |
|---|---|---|
| Concept | [`permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) | Verified-header bump noting CH-05 lifts the §"Transitive-Grant Match Rule" validator + §"Manifest Validation" rules into typed Rust at `domain::permissions::manifest::validator`. Doc body unchanged — the validator implements what the doc already specifies. |
| Concept | [`permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) | Verified-header bump noting that the §"What v0 Validates" list is now executable code. Doc body unchanged. |
| Concept | [`permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) | Verified-header bump cross-referencing CH-05's reserved-namespace constant. Doc body unchanged. |
| Decision | `m5_2/decisions/0044-publish-time-manifest-validator.md` (NEW) | Full ADR — see §5. |
| Architecture | [`m1/architecture/permission-check-engine.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/permission-check-engine.md) | Light verified-header bump cross-referencing ADR-0044 — the engine assumes validated manifests; the validator is the publish-time precondition. |

5 file touches (one new ADR, three concept-doc header bumps, one architecture-doc header bump). No ops doc, no user-guide doc.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition |
|---|---|---|---|
| **D-new-07** | [`m5_1/drifts/D-new-07.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-07.md) | HIGH | `discovered → in-chunk-plan → remediated` |
| **D-new-31** | [`m5_1/drifts/D-new-31.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-31.md) | LOW | `discovered → in-chunk-plan → remediated` |

D-new-07 (general validator) is the parent; D-new-31 (reserved-namespace write rejection) is its sub-case per the drift's own remediation note. They close together because Rule C of the validator implements both.

**Index updates:**
- [`drifts/README.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/README.md) — both row Statuses flipped to `remediated`; "Closes at" → `CH-05 ✓`.
- [`drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip rows for "Publish-time manifest validator" + "Reserved-namespace write rejection" from `silent-in-code` to `honored`.

---

## §5 — ADR drafted

ADR numbering: highest issued = ADR-0043 (CH-04). Next-free = **ADR-0044**.

| ADR | Title | Decision summary |
|---|---|---|
| **ADR-0044** | Publish-time tool authority manifest validator | **D44.1** New module `domain::permissions::manifest::validator` shipping `pub fn validate_published_manifest(m: &ToolAuthorityManifest) -> Result<Vec<ValidationWarning>, ValidationError>`. Pure function — no I/O, no repository access. Errors are returned eagerly on the first hard rejection (caller can run again with a fix; warnings accumulate so all are surfaced at once). **D44.2** `pub enum ValidationError` variants: `MissingKindForComposite { composite: Composite }` (Rule A.a), `KindFundamentalsInconsistent { kind: Composite, declared: HashSet<Fundamental>, expected: HashSet<Fundamental> }` (Rule A.b), `UnknownResource { name: String }` (resource string doesn't match any Fundamental or Composite name), `ActionFundamentalMismatch { action: Action, fundamental: Fundamental }` (Rule B), `ConstraintFundamentalMismatch { constraint: String, fundamentals: HashSet<Fundamental> }` (Rule D), `ReservedNamespaceWrite { namespace: String, action: Action }` (Rule C). All variants are `#[derive(Debug, Clone, PartialEq, Eq)]` and impl `Display + Error` for HTTP 422 mapping. **D44.3** `pub enum ValidationWarning` variants: `BlanketKindWildcard` (concept doc 04 line 75 — `#kind: *` accepted with warning), `MissingCompositeShorthand { suggested: Composite }` (concept doc 04 line 110 — fundamentals match a composite shape; suggest the composite label), `TargetKindsMissingForCreate { composite: Composite }` (concept doc 07 line 814). Warnings are non-blocking; they accumulate alongside an `Ok(...)` return. **D44.4** Rule A enforcement: for every entry in `manifest.resource ∪ manifest.transitive` that names a composite (via `expand_resource_to_fundamentals` recognising it), the composite's `kind_name()` must appear in `manifest.kinds`, OR `manifest.kinds == ["*"]` (warning). For every entry in `manifest.kinds` that names a registered composite, the composite's `constituents()` must be a subset of the union of fundamentals declared by `resource ∪ transitive`. **D44.5** Rule B enforcement: for every (Action, Fundamental) pair derivable from `actions × (resource ∪ transitive)` (after expand_resource_to_fundamentals), `Action::applies_to(Fundamental)` must return true. CH-04's matrix is the source of truth; the validator simply queries it. `Action::Wildcard` always passes. **D44.6** Rule C enforcement: when `manifest.actions` contains `Action::Modify` AND `manifest.resource ∪ manifest.transitive` contains `"tag"` (the Tag fundamental), the validator inspects the manifest's `target_kinds` field and rejects if any reserved namespace appears: `#kind:*`, `{kind}:*` for any `Composite::ALL.kind_name()`, `delegated_from:*`, `derived_from:*`. The reserved-namespace check is concept-doc-grounded; the prefix list is generated from `Composite::ALL` so adding a new composite doesn't require updating the validator. **D44.7** Rule D enforcement: for every `(constraint, fundamental)` pair from `manifest.constraints × expanded_fundamentals`, `constraint_applies_to(constraint, fundamental)` must return true. Universal constraints (`time_window`, `approval_requirement`, `non_delegability`, `purpose`) always pass. Unknown constraint names trigger `ConstraintFundamentalMismatch` with an empty fundamentals set (the operator can't prove applicability). The matrix lives at `permissions::manifest::validator::CONSTRAINT_MATRIX` as a hard-coded `match` mirroring concept doc 03 lines 78–88. **D44.8** Repository-guard wiring: `RepositoryError::ManifestValidation { source: ValidationError }` added in `domain::repository`. Both `SurrealStore::create_tool_authority_manifest` and `InMemoryRepository::create_tool_authority_manifest` call `validate_published_manifest(manifest)?.into()` at the top of the method body. Warnings are dropped at the repo boundary (callers wanting them call the function directly first); the repo only enforces hard rejections. **D44.9** Out of scope at this chunk: per locked Q3 (2026-04-29), no `POST /api/v0/tools/publish` HTTP handler. The validator + repo guard are the architectural gate; whichever future chunk wires the HTTP surface (likely a CH-2X tool-admin chunk) calls `validate_published_manifest` directly to surface warnings + structured errors before persistence. **D44.10** Out of scope at this chunk: per locked Q2 (2026-04-29), `Manifest.constraints` stays `Vec<String>`. A typed Constraint enum is deferred to a v1 chunk that bundles it with the per-resource Action enum redesign (ADR-0043 §D43.9). The string-based `constraint_applies_to` function is sufficient at v0 — it catches the same class of bug the typed enum would, at the same publish-time stage. |

ADR file: [`m5_2/decisions/0044-publish-time-manifest-validator.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0044-publish-time-manifest-validator.md) (NEW).

---

## §6 — Prior-chunk regression re-verification

| Upstream | Invariant | Verification |
|---|---|---|
| Post-CH-04 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1149; 4 CI guards green; clippy clean under `-Dwarnings` | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh`<br>`cargo test -j 4 --workspace -- --test-threads=1` |
| CH-04 / ADR-0043 | `Action::applies_to` + `Action::applies_to_composite` matrix functions reachable from validator module | New validator code calls them; tests prove the validator's Rule B uses the typed enum (not a string lookup) |
| CH-21 / ADR-0040 + 0041 | Audit hash chain byte-stable | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |
| CH-22 / ADR-0035 | Catalog listener body unchanged | `cargo test -j 4 -p domain --lib events::listeners::tests` |
| Repository trait | All existing impls compile + persist correctly with new `RepositoryError::ManifestValidation` variant | Workspace test suite covers it; the new variant is additive (no existing code matches on `RepositoryError` exhaustively in a way that would break). |

The validator must not change any audit-event canonical bytes (the BLAKE3 chain depends on byte-stable serialization). Validation failures emit no audit event, so the chain stays untouched.

---

## §7 — Phases

**Phase count: 3** → audit envelope = **2 agents** (medium chunk).

### P1 — Land the validator module + matrix + reserved-namespace constant (~0.7d)

**Goal.** Ship the validator as a standalone, fully-tested function with no consumers yet. End-state: `validate_published_manifest` is callable from anywhere; no Repository impl yet calls it. The new types are defined and unit-tested.

**Deliverables.**

1. **New module** at [`modules/crates/domain/src/permissions/manifest/validator.rs`](baby-phi/modules/crates/domain/src/permissions/manifest/validator.rs) (NEW). Note: `manifest.rs` becomes `manifest/mod.rs` (module-as-folder reshape) so the validator can sit beside the existing `Manifest` projection without bloating one file. Re-exports preserve the public API at `permissions::manifest::*`.

   Module contents:
   - `pub fn validate_published_manifest(&ToolAuthorityManifest) -> Result<Vec<ValidationWarning>, ValidationError>` — top-level entry point. Returns `Err` on first hard rejection; returns `Ok(warnings)` (possibly empty) when all rules pass.
   - `pub enum ValidationError` with the 6 variants from D44.2 (MissingKindForComposite, KindFundamentalsInconsistent, UnknownResource, ActionFundamentalMismatch, ConstraintFundamentalMismatch, ReservedNamespaceWrite). Derives `Debug + Clone + PartialEq + Eq`; impls `Display + Error` (via `thiserror::Error`).
   - `pub enum ValidationWarning` with the 3 variants from D44.3.
   - `pub const RESERVED_NAMESPACE_LITERALS: &[&str] = &["kind", "delegated_from", "derived_from"]` — the three concept-doc-named reserved namespaces.
   - `pub fn reserved_namespace_prefixes() -> Vec<String>` — generates the full list at runtime: `["#kind:", "delegated_from:", "derived_from:"]` plus `"{kind_name}:"` for every `Composite::ALL` (e.g., `"session:"`, `"memory:"`, `"auth_request:"`, etc.). Uses `Composite::kind_name()` to keep auto-growth as new composites are added.
   - `pub fn constraint_applies_to(name: &str, fundamental: Fundamental) -> bool` — encodes the matrix from concept doc 03 lines 78–88. Universal constraints (`time_window`, `approval_requirement`, `non_delegability`, `purpose`) return `true` for every fundamental.
   - Internal helpers: `enum ResourceClass { Fundamental(Fundamental), Composite(Composite), Unknown(String) }` returned by a `classify_resource_name(&str) -> ResourceClass` parser (since the existing `expand_resource_to_fundamentals` returns an empty set for unknown names without flagging the unknown — the validator needs to know which names are unknown to surface UnknownResource).

2. **Module wiring**: `permissions/mod.rs` already exports `pub mod manifest;` (line 38) — no change needed at the top level; the manifest module just gains a sub-module. Re-export `validator::{validate_published_manifest, ValidationError, ValidationWarning, RESERVED_NAMESPACE_LITERALS, reserved_namespace_prefixes, constraint_applies_to}` via `permissions::manifest::*`.

3. **`thiserror` derive** on `ValidationError` — already a workspace dep (CLAUDE.md mandates it tracks phi-core's version `"2"`). No new Cargo.toml change.

**Tests.** ~30 unit tests in `permissions::manifest::validator::tests`:

   - 6 happy-path tests (one per pass-class): valid composite manifest, valid fundamental-only manifest, valid Wildcard-action manifest, valid universal-constraint-only manifest, valid `target_kinds` per composite, valid blanket `#kind: *`.
   - 6 hard-rejection tests (one per `ValidationError` variant): MissingKindForComposite, KindFundamentalsInconsistent, UnknownResource, ActionFundamentalMismatch, ConstraintFundamentalMismatch, ReservedNamespaceWrite.
   - 3 warning tests (one per `ValidationWarning` variant): BlanketKindWildcard, MissingCompositeShorthand, TargetKindsMissingForCreate.
   - **Reserved-namespace exhaustive test**: for every `Composite::ALL`, assert the `kind_name() + ":"` prefix appears in `reserved_namespace_prefixes()`. Plus the three literals (`#kind:`, `delegated_from:`, `derived_from:`).
   - **Action × Fundamental round-trip test**: for every Action × Fundamental cell where `Action::applies_to(F) == false`, the validator must reject a manifest declaring that pair. (~30 cells; one parameterized test.)
   - **Constraint matrix exhaustive test**: 9 constraints × 9 fundamentals = 81 cells, asserted against the concept doc verbatim. Universal constraints add 4 × 9 = 36 always-pass cells. Total: 117 cells in one parameterized test.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if:
- Any concept-doc rule is internally inconsistent or ambiguous on whether something is a hard rejection or a warning.
- The `target_kinds` field on `ToolAuthorityManifest` has a different shape than expected (today it's `Vec<String>`; the validator assumes string entries are composite kind names).
- A composite's `kind_name()` collides with a non-composite namespace prefix (would mean `Composite::ALL` overlaps with one of the three literal reserved namespaces — should not happen, but worth checking).

---

### P2 — Wire validator into Repository impls + acceptance tests (~0.8d)

**Goal.** Hard-fail at the Repository boundary on invalid manifests. End-state: every code path that calls `Repository::create_tool_authority_manifest` is gated by `validate_published_manifest`; any callsite constructing an invalid manifest fails loudly.

**Deliverables.**

1. **`RepositoryError::ManifestValidation { source: ValidationError }`** added in [`modules/crates/domain/src/repository.rs`](baby-phi/modules/crates/domain/src/repository.rs). Variant carries the structured `ValidationError` so handlers can map to specific HTTP error codes.

2. **`SurrealStore::create_tool_authority_manifest`** at [`modules/crates/store/src/repo_impl.rs:643-657`](baby-phi/modules/crates/store/src/repo_impl.rs) — add `validate_published_manifest(manifest).map_err(RepositoryError::ManifestValidation)?;` at the top of the method, before the SurrealDB write. The discarded warnings are intentional at the repo boundary (callers that want them call the validator directly).

3. **`InMemoryRepository::create_tool_authority_manifest`** at [`modules/crates/domain/src/in_memory.rs:382-388`](baby-phi/modules/crates/domain/src/in_memory.rs) — same change.

4. **Doc-comment** on the trait method itself in `repository.rs` documenting that the impl validates, so future impls inherit the contract.

5. **Update existing test fixture** at [`modules/crates/store/tests/repository_test.rs:389`](baby-phi/modules/crates/store/tests/repository_test.rs) — the existing `create_tool_authority_manifest_persists_full_shape` test constructs a manifest. P2 inspects whether that manifest is internally consistent under the new validator; if not, adjusts the fixture to satisfy the validator. The test's intent is to verify trait-impl persistence, not validator behaviour.

6. **New acceptance-test file** at [`modules/crates/server/tests/acceptance_manifest_validator.rs`](baby-phi/modules/crates/server/tests/acceptance_manifest_validator.rs) (NEW). Per-rejection-class test:
   - Construct an invalid manifest (one per `ValidationError` variant — 6 tests).
   - Call `repo.create_tool_authority_manifest(&manifest)` against an in-memory repo.
   - Assert the result is `Err(RepositoryError::ManifestValidation { source })` with the expected variant.
   - Assert the manifest is NOT persisted (`repo.list_*` or similar shows no row).
   - One happy-path test: valid manifest persists cleanly via the same code path.

7. **Cross-impl consistency test** in the acceptance file: same invalid manifest fed to both `InMemoryRepository` and `SurrealStore` (via tempfile-backed `SurrealStore::open_embedded`) returns the same `ValidationError` variant. Pin the cross-impl invariant.

**Tests.** ~9 acceptance tests + happy/regression update. Existing test count baseline (1149) + 30 P1 unit tests + ~10 P2 acceptance tests = ~1189 total at chunk close.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- Any existing test constructs a manifest that the new validator would reject — that test's manifest is then either: (a) buggy and should be fixed, or (b) intentionally testing the unvalidated path (in which case it must move to a `#[allow]`-guarded test of the in-memory backend's bypass — but no such bypass exists today, so pause and discuss).
- The new `RepositoryError::ManifestValidation` variant breaks an exhaustive `match` on `RepositoryError` somewhere — the variant is additive but Rust's exhaustiveness check would catch a missed arm. Fix any uncovered match-arms before proceeding.
- A SurrealDB integration-test failure surfaces a difference between in-memory and embedded behaviour for the validator path.

---

### P3 — ADR Accepted + drifts closed + audit + seal (~0.5d)

**Goal.** Ratify ADR-0044. Close D-new-07 + D-new-31. Spawn 2 audit agents. Seal.

**Deliverables.**

1. ADR-0044 flipped from `Proposed` → `Accepted` at chunk seal.
2. D-new-07 + D-new-31 Statuses flipped to `remediated`. Lifecycle entries appended:
   - D-new-07: `2026-04-XX — remediated — CH-05 chunk-seal — validate_published_manifest function shipped + Repository guard wired in both impls; 4 hard rules + 3 warnings cover the concept-doc spec.`
   - D-new-31: `2026-04-XX — remediated — CH-05 chunk-seal — Rule C of validate_published_manifest rejects [modify] on the four reserved namespaces (#kind:, delegated_from:, derived_from:, plus auto-generated {kind}: per Composite::ALL).`
3. `drifts/README.md` rows flipped + `Closes at` columns updated.
4. `_concept-audit-matrix.md` 2 rows flipped from `silent-in-code` to `honored`.
5. Concept docs `permissions/04-manifest-and-resolution.md`, `permissions/07-templates-and-tools.md`, `permissions/09-selector-grammar.md`: verified-header bumps (CH-05 amendment line). Doc bodies UNCHANGED.
6. Architecture doc `m1/architecture/permission-check-engine.md`: light verified-header bump cross-referencing ADR-0044 (the engine assumes validated manifests; the validator is the precondition).
7. Spawn 2 audit agents per §11.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit reports a finding.

---

## §8 — Tests summary

- **Expected total at chunk close:** post-CH-04 baseline (1149) + ~30 P1 unit tests + ~10 P2 acceptance tests = **~1189 tests**.
- **New test files:** `permissions::manifest::validator::tests` (unit) + `server/tests/acceptance_manifest_validator.rs` (acceptance).
- **Matrix coverage:** Action × Fundamental cells covered by Rule B test (~30 negative cells). Constraint × Fundamental matrix covered exhaustively by P1's 117-cell test. Reserved-namespace coverage: 8 (composites) + 3 (literals) = 11 prefixes.
- **Cross-impl consistency:** 1 test ensures InMemoryRepository + SurrealStore give the same validator verdict.
- **Wire-format compat:** None needed — manifests serialize unchanged; the validator runs before persistence.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4`.
2. Copy plan verbatim to `baby-phi/docs/specs/plan/build/<8hex>-ch-05-publish-time-manifest-validator.md`.
3. Update placeholders in lines 4–5 of the archived copy.
4. `bash scripts/check-doc-links.sh`.

### Reading list (mandatory)

1. [`concepts/permissions/04-manifest-and-resolution.md`](baby-phi/docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md) — § "Tool Authority Manifest" + § "Transitive-Grant Match Rule" + § "Manifest Validation at Publish Time".
2. [`concepts/permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) — § "What v0 Validates vs Future Enhancements" (lines 803–814).
3. [`concepts/permissions/09-selector-grammar.md`](baby-phi/docs/specs/v0/concepts/permissions/09-selector-grammar.md) — § "Reserved Namespace Enforcement" (lines 190–196).
4. [`concepts/permissions/01-resource-ontology.md`](baby-phi/docs/specs/v0/concepts/permissions/01-resource-ontology.md) — § "Reserved tag namespaces" (lines 254–260).
5. [`concepts/permissions/03-action-vocabulary.md`](baby-phi/docs/specs/v0/concepts/permissions/03-action-vocabulary.md) — Constraint × Fundamental matrix (lines 78–88) + Universal Constraints table.
6. [`drifts/D-new-07.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-07.md) + [`D-new-31.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-31.md) — full.
7. [`modules/crates/domain/src/permissions/action.rs`](baby-phi/modules/crates/domain/src/permissions/action.rs) — CH-04's Action enum + matrix; precedent for validator's matrix-encoding shape.
8. [`modules/crates/domain/src/model/composites.rs`](baby-phi/modules/crates/domain/src/model/composites.rs) — `Composite::ALL` + `kind_name()` (powers the reserved-namespace prefix list).
9. [`modules/crates/domain/src/permissions/expansion.rs`](baby-phi/modules/crates/domain/src/permissions/expansion.rs) — `expand_resource_to_fundamentals` (the validator wraps it for classification).
10. [`modules/crates/domain/src/repository.rs`](baby-phi/modules/crates/domain/src/repository.rs) — RepositoryError enum + create_tool_authority_manifest signature.
11. [`modules/crates/store/src/repo_impl.rs`](baby-phi/modules/crates/store/src/repo_impl.rs) lines 643–657 — SurrealDB impl of `create_tool_authority_manifest`.
12. [`modules/crates/domain/src/in_memory.rs`](baby-phi/modules/crates/domain/src/in_memory.rs) lines 382–388 — InMemory impl.
13. [`modules/crates/store/tests/repository_test.rs`](baby-phi/modules/crates/store/tests/repository_test.rs) line 389 — the only existing test fixture that constructs a `ToolAuthorityManifest`.
14. [ADR-0043](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0043-typed-action-vocabulary.md) — most recent ADR for header/format precedent + the matrix-encoding pattern.

### Carry-forward invariants (verified at chunk-open)

- `cargo test --workspace -- --test-threads=1` ≈ 1149.
- 4 CI guards green.
- D-new-07 + D-new-31 status `discovered`.
- ADR-0034..0043 Accepted; next-free = 0044.
- `git diff --stat HEAD -- modules/` empty (or tracking only intentional pending work).
- CH-04's `Action::applies_to` + `applies_to_composite` callable from `permissions::manifest::*` (re-exported via `permissions::Action`).

---

## §10 — Close criteria (5-aspect)

- **Code aspect** — workspace builds; clippy under `RUSTFLAGS="-Dwarnings"` clean; `cargo test --workspace -- --test-threads=1` green at ~1189; no manifest-creation site that bypasses the validator.
- **Docs aspect** — D-new-07 + D-new-31 lifecycle remediated; matrix flipped honored; ADR-0044 Accepted; 3 concept-doc verified-headers bumped + 1 architecture-doc header bumped.
- **phi-core leverage** — import-count delta = 0; positive/forbidden greps all match expected.
- **Concept alignment** — every §2 row at target status.
- **K8s readiness** — neutral; ledger unchanged.

**Implementation confidence** = `claims-honored / claims-in-scope` = target **8/8**:
1. `validate_published_manifest` function exists at `permissions::manifest::validator`.
2. 6 `ValidationError` variants cover the 4 hard rules.
3. 3 `ValidationWarning` variants cover the 3 documented warnings.
4. `RESERVED_NAMESPACE_LITERALS` constant + `reserved_namespace_prefixes()` generator wired off `Composite::ALL`.
5. `constraint_applies_to(&str, Fundamental)` encodes the concept-doc matrix exactly (universal constraints + 9 specific constraints × 9 fundamentals).
6. `RepositoryError::ManifestValidation` variant added; both Repository impls call validator as precondition.
7. Acceptance tests cover each rejection class + cross-impl consistency.
8. Wire format byte-stable (manifests still serialize identically; no audit-event changes).

---

## §11 — Audit plan

**2 agents** (medium chunk per per-chunk-template; matches CH-04 / CH-21 precedent).

### Audit A — Code correctness + phi-core leverage

> You are auditing CH-05 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan: `docs/specs/plan/build/<8hex>-ch-05-publish-time-manifest-validator.md`.
>
> 1. `validate_published_manifest(&ToolAuthorityManifest) -> Result<Vec<ValidationWarning>, ValidationError>` exists at `modules/crates/domain/src/permissions/manifest/validator.rs`.
> 2. `ValidationError` enum has exactly 6 variants matching the 4 hard rules: MissingKindForComposite, KindFundamentalsInconsistent, UnknownResource, ActionFundamentalMismatch, ConstraintFundamentalMismatch, ReservedNamespaceWrite.
> 3. `ValidationWarning` enum has exactly 3 variants: BlanketKindWildcard, MissingCompositeShorthand, TargetKindsMissingForCreate.
> 4. `ValidationError` derives `Debug + Clone + PartialEq + Eq` and impls `Display + Error`.
> 5. `RESERVED_NAMESPACE_LITERALS` constant contains exactly `["kind", "delegated_from", "derived_from"]`.
> 6. `reserved_namespace_prefixes()` returns a list including `"#kind:"`, `"delegated_from:"`, `"derived_from:"` AND one entry per `Composite::ALL` (8 composites — `session:`, `memory:`, `auth_request:`, `external_service:`, `model_runtime:`, `control_plane:`, `inbox:`, `outbox:`).
> 7. `constraint_applies_to(name, fundamental)` matches concept doc 03 lines 78–88 exhaustively. Universal constraints (time_window, approval_requirement, non_delegability, purpose) return true for every fundamental.
> 8. Rule B uses `Action::applies_to(Fundamental)` from CH-04 — not a re-encoding of the matrix.
> 9. `RepositoryError::ManifestValidation { source: ValidationError }` variant exists in `modules/crates/domain/src/repository.rs`.
> 10. Both `SurrealStore::create_tool_authority_manifest` and `InMemoryRepository::create_tool_authority_manifest` call `validate_published_manifest` as the first line of the method body.
> 11. Acceptance test file `modules/crates/server/tests/acceptance_manifest_validator.rs` exists with one test per `ValidationError` variant + 1 cross-impl consistency test.
> 12. `cargo test --workspace -- --test-threads=1` green at ~1189.
> 13. CI guards green; `check-phi-core-reuse.sh` exit 0; no new `use phi_core::` imports.
> 14. `Action::applies_to` and `Action::applies_to_composite` (CH-04 carryover) remain green via existing CH-04 tests.
> 15. CH-21 acceptance test (`acceptance_memory_extraction`) still green — audit hash chain byte-stable.

PASS/FAIL each. ≤ 600 words.

### Audit B — Concept fidelity + docs fidelity

> You are auditing CH-05's concept-fidelity + docs-fidelity. Read-only.
>
> 1. ADR-0044 Accepted at `m5_2/decisions/0044-publish-time-manifest-validator.md` with sub-decisions D44.1–D44.10.
> 2. ADR-0044 documents the locked forks: Repository guard + standalone fn (Q1), string-based constraint matrix (Q2), no publish handler in this chunk (Q3).
> 3. D-new-07 Status = `remediated`; lifecycle entry present.
> 4. D-new-31 Status = `remediated`; lifecycle entry present.
> 5. `drifts/README.md` rows for D-new-07 + D-new-31 flipped; "Closes at" → CH-05 ✓.
> 6. `_concept-audit-matrix.md` 2 rows flipped silent-in-code → honored.
> 7. Concept-`permissions/04-manifest-and-resolution.md` verified-header bumped (CH-05 amendment line). Doc body UNCHANGED.
> 8. Concept-`permissions/07-templates-and-tools.md` verified-header bumped. Doc body UNCHANGED.
> 9. Concept-`permissions/09-selector-grammar.md` verified-header bumped. Doc body UNCHANGED.
> 10. Architecture-`m1/architecture/permission-check-engine.md` verified-header bumped (cross-references ADR-0044).
> 11. ADR-0044 cross-references concept docs 04/07/09, drifts D-new-07 + D-new-31, and downstream chunk CH-12.
> 12. Plan archive at `plan/build/<8hex>-ch-05-...md`.
> 13. CH-04 invariants intact; ADR-0043 still Accepted; D-new-09 + D-new-10 still remediated.
> 14. CH-21 + CH-22 + CH-16 invariants intact.

PASS/FAIL each. ≤ 600 words.

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1189 passed / 0 failed

# 3. Positive greps
ls modules/crates/domain/src/permissions/manifest/validator.rs                                          # exists
grep -n "fn validate_published_manifest\b" modules/crates/domain/src/permissions/manifest/validator.rs # 1
grep -c "^    [A-Z][a-zA-Z]* {" modules/crates/domain/src/permissions/manifest/validator.rs           # ≥ 6 ValidationError variants + 3 ValidationWarning
grep -n "RESERVED_NAMESPACE_LITERALS\b" modules/crates/domain/src/permissions/manifest/validator.rs    # 1
grep -n "fn reserved_namespace_prefixes\b" modules/crates/domain/src/permissions/manifest/validator.rs # 1
grep -n "fn constraint_applies_to\b" modules/crates/domain/src/permissions/manifest/validator.rs       # 1
grep -n "ManifestValidation\b" modules/crates/domain/src/repository.rs                                  # ≥ 1
grep -c "validate_published_manifest" modules/crates/store/src/repo_impl.rs                            # ≥ 1
grep -c "validate_published_manifest" modules/crates/domain/src/in_memory.rs                           # ≥ 1
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0044-publish-time-manifest-validator.md  # 1

# 4. Negative greps (no parallel validator from another author)
grep -rn 'fn validate_manifest\b' modules/crates/                                                       # 0
grep -rn 'use phi_core::' modules/crates/domain/src/permissions/manifest/                              # 0

# 5. Drift closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-07.md          # 1
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-31.md          # 1

# 6. Validator unit tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::manifest::validator::tests
# Expect: ~30 tests passed including matrix exhaustive + reserved-namespace exhaustive.

# 7. Acceptance suite for the validator
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_manifest_validator -- --test-threads=1
# Expect: ~10 tests (one per ValidationError variant + cross-impl + happy path).

# 8. Carry-forward sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction              # CH-21 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib permissions::action::tests                # CH-04 matrix still green
```

---

## What this plan does NOT do

- Doesn't add a `POST /api/v0/tools/publish` handler. Validator + repo guard only at this chunk.
- Doesn't migrate `Manifest.constraints` to a typed enum. String-based for now; v1 redesign per ADR-0043 §D43.9.
- Doesn't touch the runtime Permission Check engine. Validator is publish-time only.
- Doesn't validate engine-internal `Manifest` projections — only persisted `ToolAuthorityManifest` shapes.
- Doesn't re-encode CH-04's Action × Fundamental matrix. Reuses `Action::applies_to`.
- Doesn't touch wire format. Manifests serialize identically pre/post.

---

## Critical files

**New:**
- `modules/crates/domain/src/permissions/manifest/validator.rs` — the validator + matrix + reserved-namespace constant
- `modules/crates/domain/src/permissions/manifest/mod.rs` — module re-shape (was `manifest.rs`); existing `Manifest` projection moves here
- `modules/crates/server/tests/acceptance_manifest_validator.rs` — per-rejection-class acceptance suite
- `docs/specs/v0/implementation/m5_2/decisions/0044-publish-time-manifest-validator.md` — ADR

**Modified (light):**
- `modules/crates/domain/src/permissions/mod.rs` — re-exports for new validator module + types
- `modules/crates/domain/src/repository.rs` — add `RepositoryError::ManifestValidation` variant + doc-comment on trait method
- `modules/crates/store/src/repo_impl.rs` — wire validator at top of `create_tool_authority_manifest`
- `modules/crates/domain/src/in_memory.rs` — same
- `modules/crates/store/tests/repository_test.rs:389` — adjust test-fixture manifest to satisfy validator if needed
- Drift files: `D-new-07.md`, `D-new-31.md`, `drifts/README.md`, `_concept-audit-matrix.md`
- Concept docs: `permissions/04-manifest-and-resolution.md`, `permissions/07-templates-and-tools.md`, `permissions/09-selector-grammar.md` — header bumps only
- Architecture doc: `m1/architecture/permission-check-engine.md` — header bump only

---

## Estimated effort

~2 engineer-days:
- 0.7d — P1 validator module + matrix + reserved-namespace constant + ~30 unit tests.
- 0.8d — P2 repo guard wiring + ~10 acceptance tests + test-fixture update + workspace clippy/test green.
- 0.5d — P3 ADR Accepted + drift closure + concept-doc + architecture-doc header bumps + matrix flip + drifts/README + 2 audit agents + seal.
