<!-- Last verified: 2026-04-22 by Claude Code -->

# Operations — Org Dashboard

**Status: [PLANNED M3/P5]** — fleshed out when P5 ships.

Ops runbook for page 07:
- Polling cadence (30 s client-side `setInterval` via Server Actions per plan §D4).
- Data-freshness SLOs (< 30 s staleness in steady state; `p95 < 500 ms` handler latency).
- M7b upgrade path to WebSocket push (noted; not actioned in M3).
- "Dashboard shows stale data" incident playbook (flush + re-poll).

See [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §P5.
