<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Cross-Cutting — NFRs and Traceability

> Non-Functional Requirements (NFRs) that apply across admin pages, agent-self-service surfaces, and system flows, plus the authoritative **traceability matrix** mapping every concept section to the requirements that implement it.

## Files

| File | What it covers |
|------|-----------------|
| [nfr-performance.md](nfr-performance.md) | Latency, throughput, concurrency targets. |
| [nfr-observability.md](nfr-observability.md) | Audit event schema, metrics, logs, retention of observability data. |
| [nfr-security.md](nfr-security.md) | Permission-model security properties translated to testable invariants. |
| [nfr-cost.md](nfr-cost.md) | Token-budget enforcement, cost-accounting fidelity. |
| [traceability-matrix.md](traceability-matrix.md) | Concept section → requirement-ID coverage index. |

## Scope

These NFRs apply to v0. They are:

- **Testable** — each requirement is phrased so that "is this met?" is answerable with a concrete measurement or a test case.
- **Numerical where possible** — default target values are given as concrete numbers (p95 latency caps, retention windows, throughput thresholds), revisitable in a follow-on NFR-specific plan.
- **Not implementation directives** — they do not prescribe technology choices. An implementation may meet them by any means (caching layer choice, storage tier, etc.) as long as the measurable attribute holds.

## ID convention

`R-NFR-<area>-<n>` with `<area>` ∈ { performance, observability, security, cost }.

## See also

- [../README.md](../README.md) — top-level requirements README.
- [../admin/README](../admin/00-fresh-install-journey-overview.md) and [../system/README.md](../system/README.md) — the FRs these NFRs apply to.
