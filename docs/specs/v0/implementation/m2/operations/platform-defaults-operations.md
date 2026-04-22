<!-- Last verified: 2026-04-21 by Claude Code -->

# Operations — Platform Defaults

**Status: [EXISTS]** — page 05 shipped with M2/P7.

This runbook is for the platform admin who edits the singleton
`platform_defaults` row. It covers:

- Reading the current defaults (§1).
- Editing safely with optimistic concurrency (§2).
- Recovering from a stale-write 409 (§3).
- Factory reset (§4).
- The non-retroactive contract — why your edit does NOT apply to
  existing orgs (§5).

## 1. Reading

### CLI

```
phi platform-defaults get [--include-factory] [--format json|yaml|toml]
```

Default format is JSON (the server wire format). YAML and TOML are
rendered client-side from the same struct — pick whichever is easier
to diff by eye.

`--include-factory` produces a wrapped object with the live row +
the factory baseline, so an operator can diff "current vs factory"
locally before deciding whether to change anything.

### Web

Navigate to `/platform-defaults`. The page runs an SSR GET and
hands the response to two components:

- `DefaultsForm` (edit surface) — pre-populated with the live row.
- `FactoryDefaultsPanel` (read-only side panel) — shows the
  phi-core baseline. Operators copy snippets from here into the
  form to reset individual sections.

On a fresh install (no row persisted), the form is pre-populated
with the factory baseline and a banner notes "using factory
defaults; submit at if_version=0 to create the first row."

### HTTP

```
GET /api/v0/platform/defaults
```

Response:

```jsonc
{
  "defaults":  { /* current row, or factory if none persisted */ },
  "persisted": true | false,
  "factory":   { /* always the phi-core factory baseline */ }
}
```

Unauthenticated → 401 `UNAUTHENTICATED`.

## 2. Editing

### CLI

```
phi platform-defaults put --file <PATH> --if-version <N> [--format yaml|json|toml]
```

`--if-version` is **required** — it carries the version the operator
saw at read time. Use `0` for the very first write.

Format auto-detects from extension when unset (`.yaml` / `.yml` /
`.toml` / `.json`; everything else → JSON). The CLI reads the file,
deserialises into `PlatformDefaults` through `serde_yaml` / `toml` /
`serde_json`, re-serialises as JSON, and PUTs.

### Web

Edit the form in-place and click "Save (if_version=N)". The form
renders each phi-native field (retention, alert channels) as a
first-class control and each phi-core section as a JSON textarea —
so new phi-core fields flow through without requiring a web-tier
change.

### HTTP

```
PUT /api/v0/platform/defaults
Content-Type: application/json

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

Every PUT emits a `platform.defaults.updated` (Alerted) audit event
whose diff carries the full before + after snapshot. Reviewers can
reconstruct the revision history by filtering `target_entity_id =
<deterministic-platform-defaults-id>`.

## 3. Stale-write recovery (409)

When two admins edit concurrently the second PUT sees a version
mismatch:

```jsonc
// HTTP 409
{
  "code":    "PLATFORM_DEFAULTS_STALE_WRITE",
  "message": "stale if_version; current server-side version is 5 — re-read and retry"
}
```

Recovery:

1. **Read the current row** (`phi platform-defaults get --format
   json`).
2. **Compare against your intended change.** The other admin's write
   may have already applied part of what you wanted; re-plan only
   the remaining delta.
3. **Resubmit** with the current version as `--if-version`.

The web UI surfaces the 409 via the shared `ApiErrorAlert` and
prompts the operator to refresh the page.

## 4. Factory reset

`PlatformDefaults::factory()` returns the phi-core-sourced baseline.
To reset:

### Option A — one section at a time

1. Run `phi platform-defaults get --include-factory --format
   yaml`.
2. Copy the section you want to reset from the `factory.…` tree
   into your local file.
3. PUT with `--if-version <current>`.

### Option B — full reset

1. Run `phi platform-defaults factory --format json > reset.json`.
2. Edit `reset.json` if you want to keep the phi-native
   `default_retention_days` or `default_alert_channels` at their
   current values.
3. `phi platform-defaults put --file reset.json --if-version
   <current>`.

The audit event carries the full diff so reviewers see exactly
which fields reverted.

## 5. The non-retroactive contract

**Editing `PlatformDefaults` does NOT affect existing orgs.** Each
org's effective config comes from its own snapshot taken at
creation time. An edit applies to orgs created *after* the write.

This is a hard guarantee, not a soft preference — see
[`../decisions/0019-platform-defaults-non-retroactive.md`](../decisions/0019-platform-defaults-non-retroactive.md).
Operators who need to push a change across existing orgs must
iterate them explicitly (M3+ surface).

### Why this matters

- **No hidden cascade.** Tightening
  `execution_limits.max_turns` from 100 to 20 cannot suddenly trip
  a live agent session mid-flight.
- **Audit clarity.** Two separate trails — platform-level
  (`platform.defaults.updated`) and per-org (creation-time
  snapshot) — never tangle.
- **Rollback safety.** An admin who needs to revert doesn't have
  to worry about cascading un-revocations.

## 6. Validation bounds

The PUT handler rejects:

- `execution_limits.max_turns == 0` → 400 `VALIDATION_FAILED`.
- `execution_limits.max_total_tokens == 0` → 400.
- `retry_config.max_retries > 100` → 400.

These are operator foot-gun guards, not phi-core contract surface;
phi-core allows the values but operational experience suggests
anything outside these bounds is almost certainly a typo.

## 7. phi-core leverage reminder

- Four of the six substantive fields wrap phi-core types directly —
  no parallel phi struct, no migration when phi-core evolves.
- The factory baseline is built from each phi-core type's
  `Default::default()` — a phi-core bump propagates automatically.
- YAML / TOML import/export in the CLI uses `serde_yaml` / `toml`
  directly on the `PlatformDefaults` struct. phi-core's
  `parse_config` is scoped to `AgentConfig` (a different envelope);
  reaching through it for this page would add friction without
  adding reuse.

## See also

- [`../architecture/platform-defaults.md`](../architecture/platform-defaults.md) — design + storage + invariant.
- [`../user-guide/platform-defaults-usage.md`](../user-guide/platform-defaults-usage.md) — operator walkthrough.
- [`../decisions/0019-platform-defaults-non-retroactive.md`](../decisions/0019-platform-defaults-non-retroactive.md) — non-retroactive ADR.
