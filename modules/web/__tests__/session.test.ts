// Unit tests for the pure JWT verification helper in lib/session.ts.

import { test } from "node:test";
import assert from "node:assert/strict";
import { SignJWT } from "jose";

import { verifySessionToken } from "../lib/session-verify.ts";

const SECRET = new TextEncoder().encode(
  "test-secret-test-secret-test-secret-test-secret",
);

async function mint(opts: {
  sub?: string;
  expSecondsFromNow?: number;
  secret?: Uint8Array;
}): Promise<string> {
  const now = Math.floor(Date.now() / 1000);
  return await new SignJWT({ sub: opts.sub ?? "agent-123" })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt(now)
    .setExpirationTime(now + (opts.expSecondsFromNow ?? 3600))
    .sign(opts.secret ?? SECRET);
}

test("verifySessionToken — valid token returns authenticated session", async () => {
  const token = await mint({ sub: "agent-42" });
  const session = await verifySessionToken(token, SECRET);
  assert.equal(session.authenticated, true);
  if (session.authenticated) {
    assert.equal(session.user.id, "agent-42");
    assert.equal(session.user.principal, "PlatformAdmin");
    assert.match(session.expiresAt, /^\d{4}-\d{2}-\d{2}T/);
  }
});

test("verifySessionToken — wrong secret returns unauthenticated", async () => {
  const token = await mint({ sub: "agent-42" });
  const wrongSecret = new TextEncoder().encode(
    "WRONG-secret-WRONG-secret-WRONG-secret-WRONG-secret",
  );
  const session = await verifySessionToken(token, wrongSecret);
  assert.equal(session.authenticated, false);
});

test("verifySessionToken — garbage token returns unauthenticated", async () => {
  const session = await verifySessionToken("not-a-jwt", SECRET);
  assert.equal(session.authenticated, false);
});

test("verifySessionToken — expired token returns unauthenticated", async () => {
  const token = await mint({ sub: "agent-42", expSecondsFromNow: -60 });
  const session = await verifySessionToken(token, SECRET);
  assert.equal(session.authenticated, false);
});

test("verifySessionToken — empty token string returns unauthenticated", async () => {
  const session = await verifySessionToken("", SECRET);
  assert.equal(session.authenticated, false);
});
