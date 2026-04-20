<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0008: Permission Check as an eight-stage typed pipeline

## Status

Accepted — 2026-04-20 (M1 / P3).

## Context

`concepts/permissions/04-manifest-and-resolution.md` specifies a 6-step
Permission Check (+ an explicit Step 2a for ceiling enforcement, and the
concept doc's source-order also defers Step 4 until after Step 5). The
runtime needs an implementation that:

- Returns a structured `Decision` with a discrete `failed_step` label, so
  the M1 Prometheus histogram `baby_phi_permission_check_duration_seconds
  {result, failed_step}` (commitment C10) has something to populate.
- Is a **pure** function — callable from the bootstrap flow (P5), HTTP
  handlers (P6), and acceptance tests (P9) without requiring any of them
  to build a DB-backed fixture.
- Handles every denial/pending path the concept doc names (catalogue
  miss, manifest empty, no grants held, ceiling emptied, no matching
  grant, constraint violation, scope unresolvable, missing consent) and
  leaves M2+ room for richer constraint lattices and scope cascades.

The build plan's M1 entry calls this out as a "6-step engine" and caps
P3 at ~3 days. Two shape decisions were open:

1. Engine-as-method on `Repository` (engine calls out to `Repository`
   internally for catalogue + grant lookups) vs engine-as-free-function
   (caller assembles context).
2. Pipeline decomposition — one giant `check()` with inline steps vs
   named `step_N` helpers returning typed intermediates.

## Decision

### Shape 1 — Engine is a **pure free function**

`check(ctx: &CheckContext<'_>, manifest: &Manifest, metrics: &dyn PermissionCheckMetrics) -> Decision`.

- `CheckContext` carries borrowed references to every grant vector, the
  catalogue, the consent index, and the call. The caller assembles it
  from `Repository` queries (or from test fixtures).
- `check()` performs **no I/O**, holds no state, is re-entrant.
- `PermissionCheckMetrics` is a domain-owned trait so the engine can
  record timing/result labels without depending on `metrics` or
  `prometheus`. P6 will impl the trait against
  `prometheus::HistogramVec` in the server crate.

### Shape 2 — Pipeline is **eight named step functions**

Public `step_0_catalogue`, `step_1_expand_manifest`,
`step_2_resolve_grants`, `step_2a_ceiling`, `step_3_match_reaches`,
`step_4_constraints`, `step_5_scope_resolution`,
`step_6_consent_gating` in
[`engine.rs`](../../../../../modules/crates/domain/src/permissions/engine.rs).
Each step returns a typed intermediate that the next step consumes:

```
Step 0  :  Option<DeniedReason>
Step 1  :  Vec<(Fundamental, String)>           -- reaches
Step 2  :  Vec<Candidate>                        -- tier-tagged
Step 2a :  Vec<Candidate>                        -- clamped
Step 3  :  Result<HashMap<Reach, Vec<Candidate>>, Decision>
Step 5  :  Result<HashMap<Reach, ResolvedGrant>, Decision>
Step 4  :  Option<Decision>                      -- constraint violation
Step 6  :  Option<Decision>                      -- pending consent
```

The `Decision` early-return from any step flows straight back to the
caller. No panics. No `todo!()`. No implicit state.

The concept-doc order runs Step 4 after Step 5 — the implementation
preserves that ordering even though the step numbering suggests the
opposite, because scope resolution must pick a winner before
"does this winning grant satisfy the manifest's constraints?" is
meaningful.

### Encoding of `FailedStep` for metric labels

- Step 0 → `"0"`, Step 1 → `"1"`, Step 2 → `"2"`, **Step 2a → `"2a"`**,
  Step 3 → `"3"`, Step 4 → `"4"`, Step 5 → `"5"`, Step 6 → `"6"`.
- Eight distinct labels surfaced via `FailedStep::as_metric_label`.
- The `Pending` outcome does not carry a `failed_step` — the histogram
  labels it as `result="pending"` with `failed_step=""`.

## Consequences

Positive:

- **Unit-testable without storage.** 14 proptest invariants × 100 cases
  = ≈ 1,400 branches per CI run, all under 100 ms combined. Adding
  invariants costs nothing (no DB fixture, no async).
- **Step isolation.** `step_0_catalogue` is its own `pub fn`, so tests
  can assert "catalogue miss → Step 0 denial" without running any other
  steps. Every proptest file exploits this.
- **Callers stay honest.** Because the engine takes `&CheckContext`,
  P6 handlers cannot accidentally reuse a stale context, and tests
  cannot accidentally grant themselves permissions by mocking the
  repository — they have to assemble a real grant/catalogue pair.
- **Metric labels are enum-gated.** `FailedStep` is an eight-variant
  enum; cardinality cannot grow accidentally. If we later add a new
  step the enum change surfaces every caller automatically.
- **Trait-gated observability** means the domain crate continues to
  have zero dependencies on `prometheus` / `metrics`. P6 will wrap a
  `HistogramVec` in its own `PermissionCheckMetrics` impl.

Negative:

- **Context assembly is caller-owned.** P6 handlers have to do the work
  of fetching the grants from `Repository` before calling the engine.
  This is intentional (keeps the engine pure) but means the engine can't
  short-circuit "you only need org grants for this check" — the caller
  always materialises the full set. For M1 that's a handful of rows;
  M2+ will add an overload if profiling shows a problem.
- **No grant-side constraint shapes yet.** M1 models constraint
  satisfaction as "the call provides a value under the constraint
  name" — the concept doc pseudocode's "matched grant supplies a
  value" semantics is a superset. M2+ will add concrete constraint
  lattices; P3 acknowledges this as a known gap with a test that
  locks in the M1 semantics.
- **Scope cascade is lightweight.** For M1 there's at most one org +
  one project, so the cascade reduces to tier ordering
  (`Agent → Project → Organization`) plus a timestamp tie-break. The
  multi-org/multi-project selection logic lands in M4 when Projects and
  Orgs grow their M4 fields.

## Alternatives considered

- **Engine on `Repository`.** Rejected: makes the engine untestable
  without a storage backend, forces every caller to use the same
  repository mock, and breaks the crate DAG (domain depends on store
  adapter details).
- **One monolithic `check()` without named steps.** Rejected: would
  make the "no grant denies at step 3" invariant untestable without
  also threading a full fixture for steps 0–2. Named steps let us
  pin each stage independently.
- **Metric recording via a callback `Fn(...)`.** Rejected vs trait:
  trait lets us use `&dyn PermissionCheckMetrics`, making it trivial
  to swap `NoopMetrics` in tests and a real `HistogramVec` impl in
  production. Closures would've required generic propagation through
  every step helper.
- **Return `Result<Allowed, DeniedOrPending>`.** Rejected: collapses
  two distinct outcomes (Denied vs Pending) into one failure branch.
  The three-valued enum keeps the control flow explicit, and matches
  the concept doc's pseudocode.

## References

- Implementation:
  [`modules/crates/domain/src/permissions/engine.rs`](../../../../../modules/crates/domain/src/permissions/engine.rs)
  (pipeline + step helpers);
  [`decision.rs`](../../../../../modules/crates/domain/src/permissions/decision.rs)
  (enums);
  [`metrics.rs`](../../../../../modules/crates/domain/src/permissions/metrics.rs)
  (trait + noop).
- Architecture page: [permission-check-engine.md](../architecture/permission-check-engine.md).
- Proptest coverage: 6 files under
  [`modules/crates/domain/tests/`](../../../../../modules/crates/domain/tests/)
  with prefix `permission_check_`.
- Source-of-truth pseudocode:
  [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md)
  §Formal Algorithm.
- Plan: [015a217a-m1-permission-check-spine.md §P3](../../../plan/build/015a217a-m1-permission-check-spine.md).
