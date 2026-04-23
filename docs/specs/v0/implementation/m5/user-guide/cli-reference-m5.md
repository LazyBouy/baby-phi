<!-- Last verified: 2026-04-23 by Claude Code -->

# CLI reference — M5 surfaces

**Status**: [PLANNED M5/P7] — stub seeded at M5/P0; filled at P7
when `phi session` + `phi template` + `phi system-agent` + `phi
agent update --model-config-id` surfaces all land.

All M5 CLI additions use the **`phi` binary prefix** (never
`baby-phi` — platform-wide naming discipline pinned across M2/M3/M4/M5).

## `phi session` — first session launch + introspection

- `phi session launch --agent-id <uuid> --project-id <uuid> --prompt <str> [--detach] [--json]`
- `phi session show --id <uuid> [--json]`
- `phi session terminate --id <uuid> [--reason <str>] [--json]`
- `phi session list --project-id <uuid> [--active-only] [--json]`

Default `launch` tails events live via SSE until session end or
SIGINT (which sends `terminate`). `--detach` returns
`session_id` + first `loop_id` immediately (per D4).

## `phi template` — authority template adoption (M5/P5)

- `phi template list --org-id <uuid>`
- `phi template approve --org-id <uuid> --kind <A|B|C|D|E>`
- `phi template deny --org-id <uuid> --kind <A|B|C|D|E> --reason <str>`
- `phi template adopt --org-id <uuid> --kind <A|B|C|D|E>`
- `phi template revoke --org-id <uuid> --kind <A|B|C|D|E>`

## `phi system-agent` — system agents config (M5/P6)

- `phi system-agent list --org-id <uuid>`
- `phi system-agent tune --org-id <uuid> --agent-id <uuid> [--parallelize N] [--profile-ref <id>] [--trigger <enum>]`
- `phi system-agent add --org-id <uuid> --profile-ref <id> --trigger <enum>`
- `phi system-agent disable --org-id <uuid> --agent-id <uuid>`
- `phi system-agent archive --org-id <uuid> --agent-id <uuid>`

## `phi agent update` — `--model-config-id` extension (C-M5-5)

Adds `--model-config-id <str>` flag to the existing `phi agent
update` command. Validates against the org's ModelRuntime
catalogue; returns `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` (409) if
the agent has running sessions.

## Exit codes

Inherits the error-code → exit-code map from
[M4 CLI reference](../../m4/user-guide/cli-reference-m4.md). New
M5 codes surface as `EXIT_REJECTED` (session-level validation),
`EXIT_PRECONDITION_FAILED` (session-registry saturation, active
sessions block), and `EXIT_NOT_FOUND` (session id unknown).

## Cross-references

- [M4 CLI reference](../../m4/user-guide/cli-reference-m4.md) — M2–M4 commands + exit-code map.
- [Troubleshooting](troubleshooting.md) — full M5 stable-code table.
