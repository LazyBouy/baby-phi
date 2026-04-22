<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — web topology

The Next.js 14 web UI is a separate process that talks to `phi-server` over HTTP. It shares no compile-time artefacts with the Rust workspace — the integration contract is the REST API (M1+) plus the `/healthz/*` endpoints already live in M0.

## File map

[`modules/web/`](../../../../../../modules/web/):

| File | Role |
|---|---|
| [`package.json`](../../../../../../modules/web/package.json) | Dependencies; scripts (`dev`, `build`, `start`, `lint`, `typecheck`, `test`) |
| [`package-lock.json`](../../../../../../modules/web/package-lock.json) | Committed lockfile for reproducible `npm ci` |
| [`tsconfig.json`](../../../../../../modules/web/tsconfig.json) | TypeScript strict mode, App Router paths, `@/*` alias |
| [`next.config.mjs`](../../../../../../modules/web/next.config.mjs) | App config: `reactStrictMode`, `output: "standalone"`, `/api/v0/*` rewrite |
| [`tailwind.config.ts`](../../../../../../modules/web/tailwind.config.ts) | Tailwind content scan paths |
| [`postcss.config.mjs`](../../../../../../modules/web/postcss.config.mjs) | PostCSS pipeline (Tailwind + autoprefixer) |
| [`app/layout.tsx`](../../../../../../modules/web/app/layout.tsx) | Root layout, metadata, body className |
| [`app/page.tsx`](../../../../../../modules/web/app/page.tsx) | Home page (renders API health) |
| [`app/globals.css`](../../../../../../modules/web/app/globals.css) | Tailwind directives, CSS vars |
| [`lib/api.ts`](../../../../../../modules/web/lib/api.ts) | API client: `getHealth()` |
| [`lib/session.ts`](../../../../../../modules/web/lib/session.ts) | Auth placeholder contract |
| [`.eslintrc.json`](../../../../../../modules/web/.eslintrc.json) | `next/core-web-vitals` |

## Dependencies

| Dep | Version | Why |
|---|---|---|
| `next` | 14.2.18 | App Router, SSR, route handlers |
| `react` + `react-dom` | 18.3.1 | — |
| `typescript` | 5.6.3 | Strict mode |
| `tailwindcss` | 3.4.14 | Utility CSS |
| `autoprefixer`, `postcss` | — | Tailwind pipeline |
| `eslint`, `eslint-config-next` | 8.57.1 / 14.2.18 | Linting |

## App Router

M0 renders a single route (`/`) that server-renders the API health state:

```
modules/web/app/
├── layout.tsx       (RootLayout — always SSR)
├── page.tsx         (/   — SSR, dynamic = "force-dynamic")
└── globals.css
```

`page.tsx` calls `getHealth()` at request time (not build time), because `export const dynamic = "force-dynamic"` disables caching and ensures each request re-probes the server. This is intentional for M0 where the page's job is to prove SSR + API connectivity work.

## API proxy — `next.config.mjs`

At [`next.config.mjs`](../../../../../../modules/web/next.config.mjs):

```js
async rewrites() {
    const api = process.env.PHI_API_URL || "http://127.0.0.1:8080";
    return [{ source: "/api/v0/:path*", destination: `${api}/api/v0/:path*` }];
}
```

This is a **Next.js proxy rewrite**: requests the browser makes to `/api/v0/...` are forwarded server-side (or at the edge) to `PHI_API_URL`. Benefits:

- **Browser-side** requests avoid CORS — the browser talks to the same origin as the web app.
- **Server-side** (SSR) code can also use relative `/api/v0/...` paths; they resolve through Next.js.
- In dev, `PHI_API_URL=http://127.0.0.1:8080` (default); in prod, it's injected via env (typically `http://phi-server:8080` inside a pod / compose network).

`getHealth()` in [`lib/api.ts`](../../../../../../modules/web/lib/api.ts) currently hits `/healthz/live` and `/healthz/ready` directly (not through `/api/v0/*`) because those endpoints aren't versioned; the proxy rewrite is set up for the *future* REST API that lands in M1+.

## Output mode: `standalone`

`next.config.mjs` sets `output: "standalone"`. At build time, Next.js produces a minimal `.next/standalone` directory containing exactly the files needed to run the server, plus a `server.js` entrypoint. This makes the web Dockerfile (M3+) trivial: copy `.next/standalone` and `public/`, run `node server.js`.

M0 does not yet ship a web Dockerfile — that lands with M3 when the web UI gets its first real page. The server Dockerfile at the repo root is for `phi-server` only.

## Auth contract — placeholder

[`lib/session.ts`](../../../../../../modules/web/lib/session.ts) declares the types the rest of the web app will use once auth is wired:

```ts
export type SessionUser = {
  id: string;
  displayName: string;
  principal: string;   // PlatformAdmin | OrgAdmin | AgentLead | Human | LlmAgent
};

export type Session =
  | { authenticated: true; user: SessionUser; expiresAt: string }
  | { authenticated: false };

export async function getSession(): Promise<Session> {
  // TODO (M1): read the server-signed session cookie and validate.
  return { authenticated: false };
}
```

M0 returns `{ authenticated: false }` unconditionally. In M3, this will:

1. Read the server-signed, HttpOnly session cookie.
2. Validate the cookie against `phi-server`'s `/api/v0/auth/session` endpoint (SSR-side so the client never sees the signing key).
3. Cache the result for the request duration via React's `cache()`.

Every downstream page that needs auth imports this module — when M3 lands, the contract does not change, only the implementation.

## Styling

Tailwind 3 with the default content scan:

```ts
content: [
  "./app/**/*.{js,ts,jsx,tsx,mdx}",
  "./lib/**/*.{js,ts,jsx,tsx,mdx}",
]
```

CSS custom properties for dark-mode colours live in [`app/globals.css`](../../../../../../modules/web/app/globals.css); M1+ pages will introduce design tokens and component primitives.

## Dev vs build

| Command | Behaviour |
|---|---|
| `npm run dev` | Next.js dev server on port 3000 with hot reload. Proxy rewrites active. |
| `npm run build` | Production build (standalone output). |
| `npm run start` | Serve the standalone build on port 3000. |
| `npm run lint` | `next lint` (ESLint + next/core-web-vitals). |
| `npm run typecheck` | `tsc --noEmit`. |
| `npm run test` | Placeholder; real tests arrive with M3. |

CI runs every one of these on every web-touching PR via [`.github/workflows/web.yml`](../../../../../../.github/workflows/web.yml). See [`../operations/ci-pipelines.md`](../operations/ci-pipelines.md).

## What ships in M0 vs later

| Area | M0 | Later |
|---|---|---|
| App Router + SSR | ✓ | — |
| Tailwind | ✓ | Design-token system (M2+) |
| API proxy | ✓ | — |
| Auth types | ✓ (contract only) | OAuth 2.0 wiring (M3) |
| Admin pages | — | 14 pages across M2–M5 |
| Agent self-service pages | — | 5 pages in M6 |
| Component library | — | M2 first pages |
| E2E tests | — | M3+ (Playwright) |
| Web Dockerfile | — | M3+ |
