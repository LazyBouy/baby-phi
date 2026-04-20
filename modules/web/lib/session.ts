// Server-component session helper (requires `next/headers`).
//
// Reads the `baby_phi_session` cookie and delegates to
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
  process.env.BABY_PHI_SESSION_COOKIE_NAME ?? "baby_phi_session";

function getSecret(): Uint8Array {
  const raw = process.env.BABY_PHI_SESSION_SECRET ?? DEV_DEFAULT_SECRET;
  if (raw.length < 32) {
    throw new Error(
      `BABY_PHI_SESSION_SECRET must be at least 32 bytes (got ${raw.length})`,
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
