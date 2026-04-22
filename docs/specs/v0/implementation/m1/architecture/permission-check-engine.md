<!-- Last verified: 2026-04-20 by Claude Code -->

# Permission Check engine

The Permission Check engine is the runtime's authorization spine. Every
tool invocation, every grant issue, every Auth Request transition runs
through it. P3 ships the full 6-step (+Step 2a) pipeline in the `domain`
crate as a **pure** function with a structured [`Decision`] return type.

- Module: [`modules/crates/domain/src/permissions/`](../../../../../../modules/crates/domain/src/permissions/mod.rs)
- Concept doc: [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §Formal Algorithm.
- ADR: [0008 — Permission Check as an eight-stage typed pipeline](../decisions/0008-permission-check-as-pipeline.md)

## Pipeline

```
                ┌────────────────────────────────────────┐
                │         CheckContext + Manifest        │
                └──────────────────┬─────────────────────┘
                                   │
                                   ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 0 — Resource Catalogue precondition            │
      │   target ∈ owning-org catalogue? if not → Denied    │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 1 — Manifest expansion                         │
      │   resource ∪ transitive  →  Σ (fundamental, action) │
      │   composite expansion adds implicit #kind: filter   │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 2 — Candidate grant resolution                 │
      │   agent + project + org HOLDS_GRANT, non-revoked    │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 2a — Ceiling enforcement (top-down clamp)      │
      │   keep candidate iff some ceiling admits it         │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 3 — Match each required reach                  │
      │   for each (f, a): find candidates s.t.             │
      │     covers(f, a) ∧ selector matches call target     │
      │   empty match → Denied                              │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 5 — Scope-resolution cascade (most-specific)   │
      │   Agent → Project → Organization                    │
      │   tie-break: most recently issued                   │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 4 — Constraint satisfaction                    │
      │   every manifest-required constraint must have a    │
      │   value in call.constraint_context                  │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
      ┌─────────────────────────────────────────────────────┐
      │ Step 6 — Consent gating (Templates A–D)             │
      │   template-sourced winner + missing consent         │
      │     → Pending (not Denied)                          │
      └──────────────────┬──────────────────────────────────┘
                         │
                         ▼
              Decision { Allowed | Denied | Pending }
```

Steps 4 and 5 appear swapped vs their numbering because the concept doc's
pseudocode defers constraint checks until **after** scope resolution has
chosen a winner per reach — there's no point asking "does the winning
grant satisfy the manifest's constraints?" before the winner exists.

## Module layout

| File | Purpose |
|---|---|
| [`mod.rs`](../../../../../../modules/crates/domain/src/permissions/mod.rs) | Re-exports + crate-level module docs |
| [`decision.rs`](../../../../../../modules/crates/domain/src/permissions/decision.rs) | `Decision`, `DeniedReason`, `FailedStep`, `ResolvedReach`, `AwaitingConsent` |
| [`manifest.rs`](../../../../../../modules/crates/domain/src/permissions/manifest.rs) | `CheckContext`, `Manifest`, `ToolCall`, `ConsentIndex` |
| [`selector.rs`](../../../../../../modules/crates/domain/src/permissions/selector.rs) | `Selector` grammar + parser + matcher |
| [`catalogue.rs`](../../../../../../modules/crates/domain/src/permissions/catalogue.rs) | `CatalogueLookup` trait + `StaticCatalogue` in-memory fake |
| [`metrics.rs`](../../../../../../modules/crates/domain/src/permissions/metrics.rs) | `PermissionCheckMetrics` trait + `NoopMetrics` |
| [`expansion.rs`](../../../../../../modules/crates/domain/src/permissions/expansion.rs) | `expand_resource_to_fundamentals`, `resolve_grant`, `ResolvedGrant` |
| [`engine.rs`](../../../../../../modules/crates/domain/src/permissions/engine.rs) | `check()` + `step_0..step_6` helpers + `ScopeTier` + `Candidate` |

Everything compiles as one crate module (`pub mod permissions;`); each
sub-file is `pub mod <name>;` inside `permissions/mod.rs`.

## Why the engine is pure

`check()` is a free function, not a method. It takes:

- `ctx: &CheckContext<'_>` — borrows the grant vectors, catalogue, consent
  index, and call details from the caller.
- `manifest: &Manifest` — borrowed.
- `metrics: &dyn PermissionCheckMetrics` — caller-provided sink.

It returns a `Decision` and does no I/O. The caller (P6 HTTP handlers,
P5 bootstrap flow, P9 acceptance tests) is responsible for assembling the
context from the `Repository`. This shape gives us three properties:

1. **Unit-testable without a DB.** 14 proptest invariants in P3 run under
   100 ms total because no storage is touched.
2. **Decoupled from observability.** The engine records one sample via
   the `PermissionCheckMetrics` trait; the server crate plugs in a real
   Prometheus histogram in P6. The domain crate stays free of
   `metrics`/`prometheus` dependencies.
3. **Re-entrant.** No shared mutable state; callers can run many checks
   in parallel with distinct contexts.

## Decision shape

```rust
pub enum Decision {
    Allowed { resolved_grants: Vec<ResolvedReach> },
    Denied  { failed_step: FailedStep, reason: DeniedReason },
    Pending { awaiting_consent: AwaitingConsent },
}
```

- `resolved_grants` lets callers record *which* grant authorised *which*
  reach — load-bearing for audit events (C6).
- `failed_step` doubles as the Prometheus label in
  `phi_permission_check_duration_seconds{result, failed_step}` (C10).
  Step 2a is encoded as `"2a"` for label readability.
- `awaiting_consent` carries the `(subordinate, org)` pair the caller must
  present to the subordinate before the check can progress. `Pending` is
  distinct from `Denied` — the caller can wait / request / retry rather
  than reject outright.

See [`decision.rs`](../../../../../../modules/crates/domain/src/permissions/decision.rs)
for the full enum definitions + helper methods
(`metric_result_label`, `failed_step`, `resolved_grants_map`).

## Composite handling via `resolve_grant`

Memory and Session both expand to `{data_object, tag}` — indistinguishable
at the fundamental level. The concept doc's `resolve_grant` refinement
([04 §Refinement](../../../concepts/permissions/04-manifest-and-resolution.md))
adds an implicit `#kind:{name}` predicate to the grant's effective selector
when the grant targets a composite. The Rust implementation matches:

| Grant URI | Fundamentals | Selector | `#kind:` refinement |
|---|---|---|---|
| `memory_object` | `{data_object, tag}` | `Any` | `#kind:memory` |
| `session_object` | `{data_object, tag}` | `Any` | `#kind:session` |
| `external_service_object` | `{network_endpoint, secret_credential, tag}` | `Any` | `#kind:external_service` |
| `system:root` *(special case)* | all 9 fundamentals | `Any` | — |
| `filesystem_object` | `{filesystem_object}` | `Any` | — |
| `filesystem:/workspace/**` | *(caller-injected)* | `Prefix` | — |

The `#kind:` filter is what keeps a memory-specific grant from matching a
session entity even though both share `data_object + tag` — the target's
self-identity tag is part of the `ToolCall.target_tags`, and
`Selector::KindTag` checks for its presence.

## Selector grammar

For M1 we ship a small, total selector grammar — four forms cover every
M1 use case:

- `Any` — written as `"*"`; matches anything.
- `Exact(s)` — plain string; target URI must equal `s`.
- `Prefix(p)` — written as `"<p>**"`; target URI must start with `p`.
- `KindTag(k)` — written as `"#kind:<k>"`; target tags must contain the
  corresponding tag.

`AndSelector` combines the explicit selector with the `#kind:` refinement.
Richer predicates (regex, tag-intersection, numeric ranges) are M2+.

## Metric recording

The engine records one sample per call into `PermissionCheckMetrics`:

```rust
metrics.record(elapsed, result_label, failed_step_label);
```

- `result_label` ∈ `{"allowed", "denied", "pending"}`.
- `failed_step_label` ∈ `{Some("0"..="6"), Some("2a"), None}`.
- P6 will implement the trait against `prometheus::HistogramVec` labelled
  `{result, failed_step}`. The histogram name lands as
  `phi_permission_check_duration_seconds` (C10) and the P9
  acceptance tests scrape `/metrics` to assert non-zero count.

## Tracing

`check()` is `#[tracing::instrument(level = "debug", skip_all, fields(agent))]`.
Every call yields a debug-level span with the agent id, so operators can
correlate audit events (C6) with their originating check.

## Canonical invariants (proptest coverage)

Six integration files under
[`modules/crates/domain/tests/`](../../../../../../modules/crates/domain/tests/)
cover the engine — at `PROPTEST_CASES=100` per invariant that's ≈ 1,700
random-input branches per CI run, scaling to ≈ 17,000 at
`PROPTEST_CASES=1000`.

| File | Invariants |
|---|---|
| [`permission_check_catalogue_props.rs`](../../../../../../modules/crates/domain/tests/permission_check_catalogue_props.rs) | Step 0 denies exactly when target ∉ catalogue; Step 0 never fires when target ∈ catalogue |
| [`permission_check_match_props.rs`](../../../../../../modules/crates/domain/tests/permission_check_match_props.rs) | Empty grants → Step 2 denial; disjoint grant/manifest → Step 3 denial; covering grant → Allowed; every fundamental has a working single-grant path |
| [`permission_check_constraint_props.rs`](../../../../../../modules/crates/domain/tests/permission_check_constraint_props.rs) | Missing constraint → Step 4 denial; full constraint context → never Step 4 denial |
| [`permission_check_consent_props.rs`](../../../../../../modules/crates/domain/tests/permission_check_consent_props.rs) | Template-gated grant + no consent → Pending; + consent → Allowed; non-template grant → never Pending |
| [`permission_check_monotonicity_props.rs`](../../../../../../modules/crates/domain/tests/permission_check_monotonicity_props.rs) | Adding an unrelated grant preserves Allowed; ceiling never widens a denial; revoked grants are invisible |
| [`permission_check_worked_trace.rs`](../../../../../../modules/crates/domain/tests/permission_check_worked_trace.rs) | `bash cargo build` canonical trace → Allowed; `bash rm -rf /` without filesystem grant → Step 3 denial; Decision JSON round-trip |

## What's deferred to later phases

- **Full template-provenance lookup.** P3 takes a
  `template_gated_auth_requests: &HashSet<AuthRequestId>` out-of-band;
  P4's Auth Request state machine will populate it via the
  `provenance_template` field on `AuthRequest`.
- **Richer constraint shapes.** M1 models constraint satisfaction as
  "the call provides a value for every required constraint name." M2+
  will encode concrete lattices (numeric ranges, regex matches, etc.) on
  both the grant and the manifest.
- **Scope resolution for multi-org/multi-project sessions.** The
  cascade is implemented but the tie-break cases for "session spans
  multiple orgs/projects" only come into play once `Project` and
  `Organization` node shapes grow their M4 fields. For M1 there is at
  most one of each.
