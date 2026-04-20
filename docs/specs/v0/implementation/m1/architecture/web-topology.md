<!-- Last verified: 2026-04-20 by Claude Code -->

# Web topology — M1/P8 extension

M0 shipped the Next.js 14 skeleton with a single `/` route that probes
health (see [`m0/architecture/web-topology.md`](../../m0/architecture/web-topology.md)).
P8 adds the first real UI surface: the `/bootstrap` SSR page that
wraps the M1/P6 endpoints.

## Route map (current)

| Path | Type | File | Purpose | Status |
|---|---|---|---|---|
| `/` | Server Component | [`app/page.tsx`](../../../../../../modules/web/app/page.tsx) | Home; renders API health probe | M0 |
| `/bootstrap` | Server Component + Server Action + Client form | [`app/bootstrap/page.tsx`](../../../../../../modules/web/app/bootstrap/page.tsx) | First-install flow — claim platform admin | **M1/P8** |

## `/bootstrap` composition

```
BootstrapPage (SSR, force-dynamic)
│
├─ GET /api/v0/bootstrap/status    ← lib/api.ts::getBootstrapStatus
│
├─ status.claimed == true   → Claimed view (renders admin_agent_id)
├─ status.ok    == false    → Error view (server unreachable)
└─ else                     → ClaimForm (client component)
                              │
                              └─ submitClaim (Server Action)
                                    │
                                    ├─ POST /api/v0/bootstrap/claim ← lib/api.ts::postBootstrapClaim
                                    ├─ forward Set-Cookie → next/headers cookies().set
                                    └─ rerender with success / rejection state
```

- The page is declared `export const dynamic = "force-dynamic"` so SSR
  probes the status on every request; no stale cache if the admin is
  claimed between visits.
- `ClaimForm` is a **client component** that uses
  `useFormState` / `useFormStatus` (React 18 / Next 14.2 idiom) to
  rerender the action result inline.
- `submitClaim` is a **server action** with `"use server"` — it runs
  server-side, can import `next/headers`, and returns a typed
  [`ClaimActionState`](../../../../../../modules/web/app/bootstrap/actions.ts).

## Module inventory

| File | Role |
|---|---|
| [`modules/web/app/bootstrap/page.tsx`](../../../../../../modules/web/app/bootstrap/page.tsx) | SSR entry: probes status, branches to claimed/unclaimed/error views |
| [`modules/web/app/bootstrap/actions.ts`](../../../../../../modules/web/app/bootstrap/actions.ts) | Server Action `submitClaim(prev, formData)` — validates input, calls `postBootstrapClaim`, forwards the session cookie |
| [`modules/web/app/bootstrap/ClaimForm.tsx`](../../../../../../modules/web/app/bootstrap/ClaimForm.tsx) | Client component — the form + inline error/success rendering via `useFormState` |
| [`modules/web/lib/api.ts`](../../../../../../modules/web/lib/api.ts) | API client — `getHealth`, `getBootstrapStatus`, `postBootstrapClaim` + pure wire-parsers (`parseStatusBody`, `parseClaimSuccess`, `extractSessionJwt`) |
| [`modules/web/lib/session.ts`](../../../../../../modules/web/lib/session.ts) | Server-component helper that reads `baby_phi_session` cookie + verifies via `verifySessionToken` |
| [`modules/web/lib/session-verify.ts`](../../../../../../modules/web/lib/session-verify.ts) | Pure `verifySessionToken` — split out so Node's built-in test runner can exercise JWT verification without standing up `next/headers` |

## Session cookie lifecycle (P6 ↔ P8)

```
baby-phi-server (M1/P6)                   Next.js web (M1/P8)
────────────────────────                   ─────────────────────
POST /api/v0/bootstrap/claim
      │
      ├─ 201 Created + Set-Cookie:
      │    baby_phi_session=<JWT>;
      │    HttpOnly; SameSite=Lax; Path=/
      │
      ▼
  Server Action receives the response  ──▶ extractSessionJwt(Set-Cookie)
                                       ──▶ cookies().set(COOKIE_NAME, jwt,
                                               { httpOnly, sameSite, path })
                                       ──▶ ClaimActionState::success

Next request to / or /bootstrap
      │
      ▼
  getSession() ─▶ cookies().get(COOKIE_NAME)
                ─▶ verifySessionToken(token, secret)
                ─▶ Session::Authenticated { user, expiresAt }
```

## Configuration

| Env var | Default | Purpose |
|---|---|---|
| `BABY_PHI_API_URL` | `http://127.0.0.1:8080` | Base URL the SSR code hits for `/api/v0/*` and `/healthz/*` |
| `BABY_PHI_SESSION_SECRET` | `dev-only-placeholder-override-via-env-var-32b` (32 bytes) | HS256 key; must match the Rust server's `session.secret`. MUST be overridden in every non-dev environment. |
| `BABY_PHI_SESSION_COOKIE_NAME` | `baby_phi_session` | Cookie name for the signed JWT |

The dev placeholder for `BABY_PHI_SESSION_SECRET` matches the dev
placeholder in [`config/default.toml`](../../../../../../config/default.toml)
so `npm run dev` + `baby-phi-server` play nicely out of the box.

## Test coverage

| Layer | File | Tests |
|---|---|---|
| Unit (wire-translation) | [`__tests__/api.test.ts`](../../../../../../modules/web/__tests__/api.test.ts) | 9 (status claimed/unclaimed/defensive; claim-success camelCase map; `extractSessionJwt` four shapes) |
| Unit (JWT verify) | [`__tests__/session.test.ts`](../../../../../../modules/web/__tests__/session.test.ts) | 5 (valid; wrong-secret; garbage token; expired; empty-string) |

Run via `npm test` (Node 22 built-in test runner + TypeScript
type-stripping — zero-dep). Tests load the pure helpers from
[`lib/api.ts`](../../../../../../modules/web/lib/api.ts) and
[`lib/session-verify.ts`](../../../../../../modules/web/lib/session-verify.ts)
so no Next.js runtime is required.

Playwright-less **manual smoke** ran at P8 close:

1. `baby-phi-server bootstrap-init` — mint a credential.
2. `baby-phi-server` + `npm run dev` in parallel.
3. `curl http://127.0.0.1:3000/bootstrap` on an unclaimed install →
   renders the claim form (verified against the expected HTML shape).
4. `curl -X POST /api/v0/bootstrap/claim` to flip the state → re-fetch
   `/bootstrap` → renders the "Platform admin already claimed" view
   with the correct `admin_agent_id`.

Browser-based Playwright coverage is scheduled for M7b (see the plan's
§Production-readiness commitments).

## Cross-references

- [server-topology.md](server-topology.md) — the HTTP endpoints this
  page wraps, plus the session-cookie contract.
- [http-api-reference.md](../user-guide/http-api-reference.md) — the
  request/response shapes the SSR code decodes.
- [web-usage.md](../user-guide/web-usage.md) — the end-user walkthrough
  of `/bootstrap`.
- [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md) —
  why the credential is single-use and never echoed on wire.
