// Server-component session helper (requires `next/headers`).
//
// Reads the `phi_kernel_session` cookie and delegates to
// [`verifySessionToken`](./session-verify.ts). Split from the pure
// verify helper so unit tests can exercise verification without
// standing up a Next.js runtime.
//
// Real OAuth 2.0 (PKCE) + local-password wiring lands in M3 per the
// build plan. M1's scope is: "the platform admin who just claimed via
// /bootstrap has a session cookie that the browser carries on every
// follow-up request."

import { cookies } from "next/headers";

import { Session, verifySessionToken } from "./session-verify";

export type { Session, SessionUser } from "./session-verify";
export { verifySessionToken } from "./session-verify";

const DEV_DEFAULT_SECRET = "dev-only-placeholder-override-via-env-var-32b";
const COOKIE_NAME =
  process.env.PHI_SESSION_COOKIE_NAME ?? "phi_kernel_session";

function getSecret(): Uint8Array {
  const raw = process.env.PHI_SESSION_SECRET ?? DEV_DEFAULT_SECRET;
  if (raw.length < 32) {
    throw new Error(
      `PHI_SESSION_SECRET must be at least 32 bytes (got ${raw.length})`,
    );
  }
  return new TextEncoder().encode(raw);
}

export async function getSession(): Promise<Session> {
  const jar = await cookies();
  const token = jar.get(COOKIE_NAME)?.value;
  if (!token) {
    return { authenticated: false };
  }
  return verifySessionToken(token, getSecret());
}

/**
 * Gate helper for Server Components inside `app/(admin)/`. On an
 * authenticated request, resolves to the `Session`. Otherwise calls
 * Next.js `redirect('/bootstrap')` which throws to short-circuit
 * rendering — callers do NOT need to branch on `{ authenticated }`.
 *
 * M3+ will extend this to enforce the `PlatformAdmin` principal
 * explicitly; M2 ships with a single principal so the check is
 * redundant but the helper centralises the gate.
 */
export async function requireAdminSession(): Promise<
  Extract<Session, { authenticated: true }>
> {
  const { redirect } = await import("next/navigation");
  const session = await getSession();
  if (!session.authenticated) {
    // `redirect` returns `never` at runtime but TS can't narrow `session`
    // through a dynamic import call — re-assert with an explicit throw.
    redirect("/bootstrap");
    throw new Error("unreachable");
  }
  return session;
}
