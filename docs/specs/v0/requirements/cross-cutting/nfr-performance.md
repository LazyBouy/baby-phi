<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# NFR — Performance

> Latency, throughput, and concurrency targets that apply across the v0 surface. Numeric defaults are working targets — revisitable after first production load tests.

## Latency

- **R-NFR-performance-1:** Read API calls (list / detail endpoints) SHALL return within 200ms p95 under nominal load (≤100 concurrent users at an org of ≤50 agents).
- **R-NFR-performance-2:** Write API calls that produce a single-node graph mutation (edit profile, update task status, update OKR) SHALL complete within 500ms p95.
- **R-NFR-performance-3:** Write API calls that produce a multi-node cascade (create org, archive agent, revoke template) SHALL complete within 2s p95.
- **R-NFR-performance-4:** The Permission Check (Steps 0–6 per [concepts/permissions/04 § Formal Algorithm](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode)) SHALL complete within 20ms p95 for a typical invocation and within 50ms p99. Grants the check considers are bounded by the invoking agent's direct + inherited grants — expected ≤50 grants per invocation.
- **R-NFR-performance-5:** Bootstrap claim ([admin/01-platform-bootstrap-claim.md](../admin/01-platform-bootstrap-claim.md) W1) SHALL complete within 3s p99 — an interactive one-time action; latency is not critical but it must not time out the installer.

## Throughput

- **R-NFR-performance-6:** The system SHALL sustain at least **100 Permission Checks per second** for a single org under steady-state load without latency degradation beyond NFR-performance-4.
- **R-NFR-performance-7:** The `memory-extraction-agent` ([s02-session-end-memory-extraction.md](../system/s02-session-end-memory-extraction.md)) SHALL process session-end events at a rate matching or exceeding session-end generation, for orgs at recommended `parallelize` settings. A research-lab-style org with 30 concurrent researchers generating ~30 sessions/hour should be served by `parallelize: 4` without queue-depth accumulating.
- **R-NFR-performance-8:** Auth Request state transitions ([s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md)) SHALL handle at least 50 slot-fill events per second across the platform.

## Concurrency

- **R-NFR-performance-9:** The system SHALL support the configured `compute_resources.max_concurrent_sessions` per org without resource contention across orgs. Per-org isolation of concurrency budgets is required — one org at capacity SHALL NOT starve other orgs.
- **R-NFR-performance-10:** An agent's `parallelize` value SHALL be enforced at session-start time; the `parallelize`-plus-one session attempt SHALL reject within 100ms with `PARALLELIZE_CAP_REACHED` per [admin/14-first-session-launch.md W2](../admin/14-first-session-launch.md#6-write-requirements).
- **R-NFR-performance-11:** Concurrent writes to the same graph node SHALL follow the LWW consistency rule from [concepts/coordination.md § Design Decisions](../../concepts/coordination.md#design-decisions-v0-defaults-revisitable). Collisions are resolved; no deadlock.

## Cold-start & scaling

- **R-NFR-performance-12:** A freshly-started phi instance SHALL be ready to serve bootstrap-claim within 30s of process start (including DB connection establishment).
- **R-NFR-performance-13:** An org's org-dashboard initial load ([admin/07-organization-dashboard.md](../admin/07-organization-dashboard.md)) SHALL be interactive within 1s of first request (subsequent reads may be faster via caching).

## Regression guard

- **R-NFR-performance-14:** CI SHALL run a performance smoke test against a representative workload (one `mid-product-team`-sized org from [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md)) and fail if any of R-NFR-performance-1 through -5 regresses by >50% from the previous baseline.

## Cross-references

- [concepts/permissions/04 § Formal Algorithm (Pseudocode)](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode) — the Permission Check these NFRs constrain.
- [concepts/coordination.md § Design Decisions](../../concepts/coordination.md#design-decisions-v0-defaults-revisitable) — consistency model.
