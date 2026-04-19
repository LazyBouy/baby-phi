<!-- Last verified: 2026-04-19 by Claude Code -->

# baby-phi operations runbook

> **Status:** stub. The full runbook is an **M7b** deliverable per the v0.1 build plan (see [`../specs/plan/build/36d0c6c5-build-plan-v01.md`](../specs/plan/build/36d0c6c5-build-plan-v01.md) §M7b). This file exists in M0 so every document that needs to point at the runbook (README, CLAUDE.md, future ADRs) has a non-404 anchor to link to.

## Sections that will land in M7b

- **Deploy** — pushing a new tagged release to staging + prod (Docker image pull, orchestrator rolling update, post-deploy smoke).
- **Upgrade** — moving from version N to N+1, including SurrealDB schema-migration preflight.
- **Rollback** — reverting to the previous Docker image tag and compensating migrations.
- **Backup** — on-demand `surreal export` + verifying the automated daily off-site dump.
- **Restore** — rebuilding a new node from the off-site dump; drill cadence and pass criteria.
- **Incident response** — 5 canonical scenarios with decision trees:
  1. DB process crash / corruption.
  2. TLS cert expired / renewal failed.
  3. Auth IdP outage (cascading to user login failures).
  4. Audit-log stream failure (hash-chain break).
  5. Unexpected p99 latency regression.
- **Known issues & workarounds** — living list, updated per release.

## Where to look in the meantime

Until the runbook is populated, operators should consult:

- [`../../CLAUDE.md`](../../CLAUDE.md) — workspace layout and local commands.
- [`../specs/v0/implementation/m0/operations/`](../specs/v0/implementation/m0/operations/) — M0 deployment, config-profile, TLS, CI, and security-scanning guidance.
- [`../specs/plan/build/36d0c6c5-build-plan-v01.md`](../specs/plan/build/36d0c6c5-build-plan-v01.md) — the v0.1 build plan (intent, not current runbook).
