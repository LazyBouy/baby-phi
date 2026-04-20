<!-- Last verified: 2026-04-20 by Claude Code -->

# First-bootstrap walkthrough

End-to-end guide for the first human on a fresh baby-phi install: from
running the installer to claiming the platform-admin role. P5 ships the
server-side atomic flow; **P6 lands the HTTP handlers** (`GET
/api/v0/bootstrap/status` + `POST /api/v0/bootstrap/claim` plus the
signed session cookie); P7 adds the CLI subcommands and P8 the web
page. This guide describes the intended happy path once all four are
landed; the HTTP / curl path is available today.

## 1. Install-time — emit the bootstrap credential

On a fresh data directory, run:

```bash
baby-phi-server bootstrap-init
```

Output:

```
============================================================
BOOTSTRAP CREDENTIAL (save this — shown once):

  bphi-bootstrap-<43-char-base64url>

Paste this into the /bootstrap page on first login.
============================================================
```

**Save the credential immediately.** It is shown exactly once. The
server stores only an argon2id hash (PHC-format, with a per-credential
salt), so there is no recovery path if you lose the plaintext — a
reinstall on a fresh data directory is the only option.

Under the hood:

- 32 bytes of CSPRNG entropy from `rand::OsRng` → base64url-no-pad
  encoding → `bphi-bootstrap-` prefix.
- Argon2id hash is persisted in `bootstrap_credentials.digest`.
- The plaintext is printed to stdout and **never** written to a
  file, a log, or an environment variable.

See [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md)
for why stdout-only delivery is the right default.

## 2. Start the server

```bash
baby-phi-server
```

The server binds per
[`config/default.toml`](../../../../../../config/default.toml) (or the
profile-specific override). Health endpoints come up first; the
`/api/v0/bootstrap/status` + `/api/v0/bootstrap/claim` routes are
mounted under the same process and respond immediately.

## 3. Claim the role

**Option A — web (lands in P8).** Navigate to
`https://<server>/bootstrap`. If no admin exists yet, the page displays
the claim form. Paste your saved credential, enter a display name,
pick a channel kind (Slack / email / web), enter the handle, submit.

**Option B — CLI (ships in P7).**
`baby-phi bootstrap claim --credential bphi-bootstrap-…
--display-name 'Alex Chen' --channel-kind slack --channel-handle @alex`.
See [cli-usage.md](cli-usage.md) for the full reference (including
exit-code semantics for shell-script callers).

**Option C — direct HTTP** (ships in P6, useful for automation):

```bash
curl -sS -X POST https://<server>/api/v0/bootstrap/claim \
  -H 'Content-Type: application/json' \
  -d '{
    "bootstrap_credential": "bphi-bootstrap-...",
    "display_name": "Alex Chen",
    "channel_kind": "slack",
    "channel_handle": "@alex"
  }'
```

On success (HTTP 201), the response carries a `Set-Cookie:
baby_phi_session=<jwt>; HttpOnly; SameSite=Lax; Path=/` header and the
JSON payload:

```json
{
  "human_agent_id": "...",
  "inbox_id": "...",
  "outbox_id": "...",
  "grant_id": "...",
  "bootstrap_auth_request_id": "...",
  "audit_event_id": "..."
}
```

Write these ids down somewhere; they are the anchors of your audit
trail going forward.

## 4. Verify the claim

After a successful claim:

- `GET /api/v0/bootstrap/status` returns
  `{ "claimed": true, "admin_agent_id": "..." }`.
- Every future grant traces back to your `bootstrap_auth_request_id`
  via `Grant.descends_from` → `AuthRequest.provenance_template` →
  the hardcoded `system_bootstrap` axiom.
- The alerted audit event `platform_admin.claimed` is the genesis of
  your platform-scope hash chain.

## Failure cases

| Condition | Response | Recovery |
|---|---|---|
| You pasted the wrong credential | 403 `BOOTSTRAP_INVALID` | Retry with the right credential. |
| The credential was already consumed by a prior claim | 403 `BOOTSTRAP_ALREADY_CONSUMED` | The install has an admin already; use those admin's credentials to add more admins (M3+). |
| An admin already exists (server-side detection) | 409 "A platform admin has already been claimed" | Same as above — use the existing admin's account. |
| Missing display name / channel handle | 400 | Resubmit with valid fields. |
| Transient storage error during the atomic commit | 500 | The credential stays unconsumed — wait for the admin to diagnose + retry. |

The atomicity guarantee
([architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md)
§Atomicity) means a failed claim never leaves half-created state
behind. Retry is always safe until one claim succeeds.

## What happens next (not in M1)

After the claim, the platform-admin journey continues into the M2+
phases the plan calls out: model-provider registration, the first org
setup, agent provisioning. P6 will redirect the browser to the next
phase's landing page; CLI users continue through `baby-phi` subcommands
as they land.

## Cross-references

- [architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md) — the full
  atomic sequence + entity shapes.
- [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md) —
  credential-storage + delivery decisions.
- Concept: [`permissions/02` §System Bootstrap Template](../../../concepts/permissions/02-auth-request.md#system-bootstrap-template--root-of-the-authority-tree).
- Requirements:
  [`admin/01`](../../../requirements/admin/01-platform-bootstrap-claim.md) +
  [`system/s01`](../../../requirements/system/s01-bootstrap-template-adoption.md).
