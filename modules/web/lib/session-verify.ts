// Pure JWT-verification helper — no Next.js dependencies.
//
// Extracted from `session.ts` so Node's built-in test runner can
// exercise it without standing up the `next/headers` cookie jar.

import { jwtVerify, errors } from "jose";

export type SessionUser = {
  id: string;
  // Principal type — in M1 this is always `PlatformAdmin`. M3+ extends
  // to the full set from `concepts/permissions/02-auth-request.md`.
  principal: "PlatformAdmin";
};

export type Session =
  | { authenticated: true; user: SessionUser; expiresAt: string }
  | { authenticated: false };

type Claims = {
  sub?: unknown;
  iat?: unknown;
  exp?: unknown;
};

/**
 * Verify a signed HS256 session token. Returns an `authenticated`
 * session on success; `{ authenticated: false }` for every expected
 * failure (bad signature, expired, malformed, missing claims).
 *
 * Unexpected errors (e.g. crypto backend failures) are rethrown so
 * the caller can decide whether to surface 500 vs treat as signed-out.
 */
export async function verifySessionToken(
  token: string,
  secret: Uint8Array,
): Promise<Session> {
  if (!token) {
    return { authenticated: false };
  }
  try {
    const { payload } = await jwtVerify(token, secret, {
      algorithms: ["HS256"],
    });
    const claims = payload as Claims;
    const sub = typeof claims.sub === "string" ? claims.sub : null;
    const exp =
      typeof claims.exp === "number"
        ? new Date(claims.exp * 1000).toISOString()
        : null;
    if (!sub || !exp) {
      return { authenticated: false };
    }
    return {
      authenticated: true,
      user: { id: sub, principal: "PlatformAdmin" },
      expiresAt: exp,
    };
  } catch (err) {
    if (
      err instanceof errors.JWTExpired ||
      err instanceof errors.JWSSignatureVerificationFailed ||
      err instanceof errors.JWSInvalid ||
      err instanceof errors.JWTClaimValidationFailed
    ) {
      return { authenticated: false };
    }
    throw err;
  }
}
