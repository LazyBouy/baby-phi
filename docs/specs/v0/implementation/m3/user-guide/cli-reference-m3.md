<!-- Last verified: 2026-04-22 by Claude Code -->

# User guide — CLI reference (M3 additions)

**Status: [PLANNED M3/P4-P5]** — fleshed out across P4 (create/list/show) + P5 (dashboard).

M3 adds the `baby-phi org` subcommand group:

| Subcommand | Phase | Purpose |
|---|---|---|
| `baby-phi org create` | P4 | Submit the 8-step wizard payload as a single POST. Accepts `--from-layout <ref>` to seed from a reference-layout YAML. |
| `baby-phi org list` | P4 | List orgs visible to the calling admin. |
| `baby-phi org show --id <uuid>` | P4 | Single-org detail (dashboard-preview payload). |
| `baby-phi org dashboard --id <uuid> [--json]` | P5 | Consolidated dashboard summary JSON. |

The existing `baby-phi completion <shell>` (M2/P8) auto-surfaces
every `org` subcommand via `clap_complete`'s tree walk — no M3 work
required.

See [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §P4 / §P5.
