<!-- Last verified: 2026-04-20 by Claude Code -->

# Web UI — `/bootstrap` walkthrough

The M1/P8 web surface exposes a single human-facing route:
`/bootstrap`. It's the SSR twin of the `baby-phi bootstrap
{status,claim}` CLI subcommands and the `POST /api/v0/bootstrap/claim`
HTTP endpoint — but aimed at an admin who prefers a browser to a
terminal.

## Prerequisites

1. The Rust server is running and reachable at
   `BABY_PHI_API_URL` (default `http://127.0.0.1:8080`).
2. You've run `baby-phi-server bootstrap-init` and copied the
   `bphi-bootstrap-…` credential printed to stdout.
3. `BABY_PHI_SESSION_SECRET` is set to the same value on both the
   Rust server and the Next.js web process. Both default to the dev
   placeholder so `npm run dev` + `baby-phi-server` works out of the
   box — but change it before shipping.

## Happy path

1. Navigate to `http://localhost:3000/bootstrap` (dev) or
   `https://<your-host>/bootstrap` (prod).
2. The page greets you with:

   > **Claim platform admin.** First-install flow — exchange your
   > single-use bootstrap credential for the `[allocate]`-on-
   > `system:root` grant that roots every subsequent authority chain
   > in the platform.

3. Fill the form:
   - **Bootstrap credential** — paste the `bphi-bootstrap-…` string
     from step 2 above. Shown in monospace; `autocomplete="off"` so
     the browser never suggests it for unrelated fields.
   - **Display name** — your human-readable name (e.g. "Alex Chen").
   - **Channel** — `Slack`, `Email`, or `Web`.
   - **Channel handle** — the address on that channel (e.g.
     `@alex` / `alex@example.com` / `https://example.com/profile`).
4. Click **Claim platform admin**.
5. On success the form is replaced by a green confirmation block:

   > **Claim succeeded**
   > You are now the platform admin. Save these identifiers — they
   > anchor the audit trail.
   >
   >     human_agent_id:   <uuid>
   >     grant_id:         <uuid>
   >     audit_event_id:   <uuid>

   A signed `baby_phi_session` cookie (HS256 JWT, HttpOnly,
   SameSite=Lax) is set on the same response. Your browser will send
   it on every follow-up request; M2+ routes will read it.

## Already-claimed path

If a platform admin has already been claimed on this install,
`/bootstrap` renders a terminal view:

> **Platform admin already claimed.** A platform admin was assigned
> for this install.
>
>     admin_agent_id: <uuid>
>
> Creating additional admins is out of scope for the first-install
> flow — it requires an Auth Request from the existing admin (lands
> in M3+).

No form is shown. The ← Back to home link returns to `/`.

## Error cases

| Inline alert | When | Recovery |
|---|---|---|
| *Invalid input: credential must not be empty* | You submitted the form with an empty field. | Fill it and resubmit. |
| *BOOTSTRAP_INVALID (403): bootstrap credential is not recognised* | Typo in the credential, or this install has no stored credential. | Re-check the plaintext; or run `baby-phi-server bootstrap-init` on a fresh data dir if you've truly lost it. |
| *BOOTSTRAP_ALREADY_CONSUMED (403): bootstrap credential has already been consumed* | A previous claim succeeded, was rolled back at the app layer, but the credential was consumed. Shouldn't happen in the normal flow. | Contact an admin (manual override is out of scope for M1). |
| *PLATFORM_ADMIN_CLAIMED (409): a platform admin has already been claimed* | An admin already exists — the form shouldn't have rendered; if you saw this, you beat another admin to the claim. | Navigate away; use the existing admin's credentials. |
| *Request failed: …* | Transport error (server down, DNS, TLS mismatch). | Check `baby-phi-server` is up and `BABY_PHI_API_URL` is correct. |

The page also shows a full-page **Cannot reach the server** view if
the SSR status probe fails before render — no form to submit through.

## What happens next

After a successful claim:

- `GET /api/v0/bootstrap/status` now reports
  `{ claimed: true, admin_agent_id: "…" }`. Revisiting `/bootstrap`
  lands on the "already claimed" view.
- The `baby_phi_session` cookie lets M2+ pages identify you. Until M2
  lands, visiting `/` shows the same home page as before the claim;
  no redirect is wired yet.
- Every future grant traces back to your `bootstrap_auth_request_id`
  via [`Grant.descends_from`](../architecture/bootstrap-flow.md).

## Cross-references

- [http-api-reference.md](http-api-reference.md) — the HTTP endpoints
  the form wraps, including stable error codes.
- [cli-usage.md](cli-usage.md) — the same flow via `baby-phi bootstrap
  claim`, for terminal users.
- [first-bootstrap.md](first-bootstrap.md) — end-to-end install-to-claim
  walkthrough.
- [architecture/web-topology.md](../architecture/web-topology.md) —
  how `/bootstrap` is composed, what modules do what.
- [architecture/server-topology.md](../architecture/server-topology.md)
  §Session cookie — the JWT the server issues.
