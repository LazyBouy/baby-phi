<!-- Last verified: 2026-04-21 by Claude Code -->

# User guide — credentials vault

**Status: [EXISTS]**

Day-to-day operator flows for page 04 (the credentials vault). Every
action has CLI + web parity; the CLI is authoritative for
shell-scripted rotations, the web UI is the default interactive
surface.

## Concepts

- **Slug** — the stable human identifier for a vault entry (e.g.
  `anthropic-api-key`). Lowercase letters, digits, dashes; no leading
  or trailing dash; ≤ 64 chars.
- **Custodian** — the agent currently authorized to rotate + reveal
  this secret. Defaults to the platform admin at add time.
- **Sensitive** — when `true`, the value is masked in list views and
  audit diffs. Default `true`; set to `false` only for clear-text
  metadata (never for API keys).
- **Material** — the plaintext bytes. Sealed with AES-256-GCM at add
  time; never logged; never returned by the list endpoint.

## Adding a secret (CLI)

```bash
# 1. Authenticate (one-time; reuses the cookie saved by `bootstrap claim`).
phi login status     # confirms a saved session exists

# 2. Pipe the plaintext on stdin (preferred — no on-disk copy).
printf 'sk-ant-…' | phi secret add --slug anthropic-api-key --material-file -

# Or, if the material is already on disk with mode 0600:
phi secret add --slug anthropic-api-key --material-file ./api-key.txt
```

The command prints the new `secret_id`, the auto-approved AR id, and
the Alerted audit event id. Save the audit event id in the ops ticket —
the reviewer will reference it during the weekly audit sweep.

## Adding a secret (web)

Navigate to **Credentials Vault** in the admin sidebar:

1. Fill in the slug (validation: lowercase / digits / dashes).
2. Paste the material into the text area.
3. Leave "Mask value in list views + audit diffs" checked unless the
   material is deliberately non-sensitive.
4. Click **Add secret**. The result banner shows the audit event id.

## Listing

```bash
phi secret list
# slug                                      sensitive       rotated     custodian
# anthropic-api-key                         sensitive       (never)     <uuid>
# …

phi secret list --json   # JSON shape for piping into jq
```

The web table shows the same columns. Plaintext never appears.

## Rotating

```bash
printf 'new-plaintext' | phi secret rotate \
    --slug anthropic-api-key --material-file -
```

`last_rotated_at` is bumped; slug + custodian are unchanged. One
`vault.secret.rotated` event is emitted (Alerted class).

## Revealing plaintext

Reveal is always audited. The CLI refuses to print plaintext unless
the operator explicitly opts in with `--accept-audit`:

```bash
phi secret reveal --slug anthropic-api-key \
    --purpose "rotate-downstream" --accept-audit
# secret revealed (audit_event_id = <uuid>); plaintext on stdout:
# sk-ant-…
```

- `--purpose` is free-form text captured into the audit diff. Use the
  concrete operator intent (e.g. `"bring-up-staging"`,
  `"rotate-downstream"`, `"incident-2026-04-21"`). The Permission
  Check engine separately asserts the structured constraint
  `purpose=reveal` — the human purpose shows up in the diff for the
  reviewer.
- Plaintext is streamed to stdout; the audit annotation goes to
  stderr so pipes stay clean: `phi secret reveal … | downstream-tool`.

The web UI runs a 3-state flow — **idle** → **confirming** →
**revealed** — with a 30-second countdown after which the plaintext
is auto-discarded from the browser state.

## Reassigning custody

```bash
phi secret reassign --slug anthropic-api-key \
    --new-custodian 00000000-0000-0000-0000-000000000042
```

Only changes the governance pointer — no re-seal. Emits
`vault.secret.custody_reassigned` with both custodians in the diff.

## Exit codes (CLI)

Stable contract for shell scripts:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Transport / IO failure — retry with backoff |
| 2 | Server rejected with a stable error `code` — do not retry |
| 3 | Server 5xx or unexpected shape — escalate |
| 4 | Precondition failed (no saved session, missing file, reveal without `--accept-audit`) |
| 5 | Cascade aborted (reserved for M2/P6) |

## References

- Operations runbook — [`../operations/secrets-vault-operations.md`](../operations/secrets-vault-operations.md)
- CLI reference — [`cli-reference-m2.md`](cli-reference-m2.md)
- Vault encryption architecture — [`../architecture/vault-encryption.md`](../architecture/vault-encryption.md)
