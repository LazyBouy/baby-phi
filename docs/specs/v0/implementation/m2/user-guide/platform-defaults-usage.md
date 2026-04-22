<!-- Last verified: 2026-04-21 by Claude Code -->

# User guide — Platform Defaults

**Status: [EXISTS]** — page 05 shipped with M2/P7.

Platform Defaults is the platform-wide baseline that new orgs adopt
at creation time. It composes four phi-core types directly —
`ExecutionLimits`, `AgentProfile`, `ContextConfig`, `RetryConfig` —
plus two phi-native fields (`default_retention_days`,
`default_alert_channels`).

Edits are **non-retroactive**: they apply only to orgs created after
the write. See
[`../decisions/0019-platform-defaults-non-retroactive.md`](../decisions/0019-platform-defaults-non-retroactive.md)
for the rationale.

## Quick reference

| Surface | Command / URL | Effect |
|---|---|---|
| Web | `/platform-defaults` | SSR form + factory side panel |
| CLI | `phi platform-defaults get [--include-factory] [--format json\|yaml\|toml]` | Read current |
| CLI | `phi platform-defaults put --file <PATH> --if-version <N>` | Update (OCC-checked) |
| CLI | `phi platform-defaults factory [--format json\|yaml\|toml]` | Print phi-core baseline (no server call) |
| HTTP | `GET /api/v0/platform/defaults` | Wire read |
| HTTP | `PUT /api/v0/platform/defaults` | Wire update |

## CLI

All subcommands share the M2 auth flow (reads the session cookie
saved by `phi bootstrap claim`). The `factory` subcommand is the
exception — it's offline, no HTTP call.

### `get`

```
phi platform-defaults get --format yaml
```

Prints the persisted row (or the factory baseline if no row exists
yet). On a fresh install the CLI prints a `# note:` stderr line
warning that what you see is the factory baseline, not a stored row.

```
phi platform-defaults get --include-factory --format json
```

Wraps the output with `{ defaults, persisted, factory }` so you can
diff your live row against the phi-core baseline.

### `put`

```
phi platform-defaults put --file defaults.yaml --if-version 3
```

`--file` can be `-` to read from stdin. Format auto-detects from
extension when `--format` is unset. The CLI deserialises into
`PlatformDefaults`, re-serialises as JSON, and PUTs — the server is
JSON-only; multi-format support is client-side.

`--if-version` is **required**. The server rejects with 409 if the
value is stale:

```
phi: rejected (PLATFORM_DEFAULTS_STALE_WRITE): stale if_version; current server-side version is 5 — re-read and retry
```

Exit code in that case is `2` (`EXIT_REJECTED`).

### `factory`

```
phi platform-defaults factory --format yaml > reset.yaml
```

Prints `PlatformDefaults::factory(now)` — the phi-core-sourced
baseline. Offline; useful for:

- Seeding the very first write (`put --file reset.yaml --if-version
  0`).
- Reference when editing an existing row.

## Web

Navigate to `/platform-defaults`. The page:

1. Fetches `GET /api/v0/platform/defaults` at SSR time.
2. Renders a two-column layout:
   - Left (2/3 width): `DefaultsForm` — the edit surface.
   - Right (1/3 width): `FactoryDefaultsPanel` — the phi-core
     baseline as read-only reference.
3. On a fresh install (no row), an amber banner notes the form is
   pre-populated with the factory baseline and that submitting at
   `if_version=0` creates the first row.

### DefaultsForm

Each phi-native field gets a dedicated control:

- `Default retention (days)` — number input.
- `Default alert channels` — comma-separated text input.

Each phi-core section is a JSON textarea:

- `ExecutionLimits (phi-core)`.
- `AgentProfile (phi-core)`.
- `ContextConfig (phi-core)`.
- `RetryConfig (phi-core)`.

The textareas are opaque on purpose — when phi-core adds or renames
fields, the web tier carries the new shape through without
needing a change.

Submit sends `PUT /api/v0/platform/defaults` with the current
`if_version`. On 200 the form renders a success panel with the new
version + audit event id. On 409 the shared `ApiErrorAlert` surfaces
the stale-write message — refresh to pull the latest.

### FactoryDefaultsPanel

Read-only. Shows the four phi-core sections as JSON blocks + the
phi baselines as plain text. Operators copy sections into the
form to reset individual slices.

## HTTP

Two routes, gated by the session cookie:

| Method | Path | Op |
|---|---|---|
| `GET` | `/api/v0/platform/defaults` | Read current + factory |
| `PUT` | `/api/v0/platform/defaults` | Update (OCC-checked) |

### `GET`

```jsonc
{
  "defaults":  { /* full PlatformDefaults — phi-core fields embedded */ },
  "persisted": true,
  "factory":   { /* phi-core factory baseline, always present */ }
}
```

### `PUT`

```jsonc
{
  "if_version": 3,
  "defaults":   { /* full PlatformDefaults struct */ }
}
```

On success (200):

```jsonc
{
  "new_version":     4,
  "auth_request_id": "<uuid>",
  "audit_event_id":  "<uuid>"
}
```

### Error codes

| Code | Status | Meaning |
|---|---|---|
| `UNAUTHENTICATED` | 401 | Session cookie missing or expired. |
| `VALIDATION_FAILED` | 400 | `max_turns == 0`, `max_total_tokens == 0`, `max_retries > 100`, or malformed request body. |
| `PLATFORM_DEFAULTS_STALE_WRITE` | 409 | `if_version` doesn't match current row version; message surfaces the current version. |
| `AUDIT_EMIT_FAILED` | 500 | Audit emitter returned an error — the underlying write MAY have succeeded. |
| `INTERNAL_ERROR` | 500 | Repository error — see server logs. |

## Audit trail

Every successful PUT emits a `platform.defaults.updated` (Alerted)
event. The diff carries:

- `before` — full snapshot of the prior row, or `null` on first write.
- `after` — full snapshot of the new row.

Both snapshots embed the phi-core serde shapes verbatim, so a
reviewer walking the chain can diff individual phi-core fields (e.g.
`execution_limits.max_turns`) across revisions.

Filter `target_entity_id = <deterministic-platform-defaults-id>` to
recover the full revision history; the target id is a Blake3-derived
constant (see
[`modules/crates/domain/src/audit/events/m2/defaults.rs`](../../../../../../modules/crates/domain/src/audit/events/m2/defaults.rs)),
so every defaults event chains under the same id.

## phi-core leverage

See [`../architecture/phi-core-reuse-map.md`](../architecture/phi-core-reuse-map.md)
§Page 05 for the full map. Highlights:

- Four phi-core types wrapped directly: `ExecutionLimits`,
  `AgentProfile`, `ContextConfig`, `RetryConfig`.
- Factory baseline uses each phi-core `Default::default()` — phi-core
  bumps propagate automatically.
- YAML / TOML import/export uses `serde_yaml` / `toml` directly on
  the `PlatformDefaults` struct; phi-core's `parse_config` pipeline
  is scoped to `AgentConfig` (a different envelope) and deliberately
  not reached through here.
