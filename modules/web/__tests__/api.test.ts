// Unit tests for the pure wire-translation helpers in lib/api.ts.
// Run via `npm test` (Node's built-in test runner + TypeScript
// type-stripping).

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  extractSessionJwt,
  parseClaimSuccess,
  parseStatusBody,
} from "../lib/api.ts";

test("parseStatusBody — claimed with admin_agent_id", () => {
  const parsed = parseStatusBody({
    claimed: true,
    admin_agent_id: "agent-123",
  });
  assert.equal(parsed.claimed, true);
  if (parsed.claimed) {
    assert.equal(parsed.adminAgentId, "agent-123");
  }
});

test("parseStatusBody — unclaimed defaults awaiting=true", () => {
  const parsed = parseStatusBody({ claimed: false });
  assert.equal(parsed.claimed, false);
  if (!parsed.claimed) {
    assert.equal(parsed.awaitingCredential, true);
  }
});

test("parseStatusBody — unclaimed respects explicit awaiting=false", () => {
  const parsed = parseStatusBody({
    claimed: false,
    awaiting_credential: false,
  });
  if (!parsed.claimed) {
    assert.equal(parsed.awaitingCredential, false);
  }
});

test("parseStatusBody — claimed=true but no admin_agent_id falls back to unclaimed", () => {
  // Defensive parse: the server should never send this shape, but if
  // the envelope drifts we prefer to mark unclaimed rather than throw.
  const parsed = parseStatusBody({ claimed: true });
  assert.equal(parsed.claimed, false);
});

test("parseClaimSuccess — maps snake_case wire to camelCase domain", () => {
  const parsed = parseClaimSuccess({
    human_agent_id: "h1",
    inbox_id: "i1",
    outbox_id: "o1",
    grant_id: "g1",
    bootstrap_auth_request_id: "ar1",
    audit_event_id: "ae1",
  });
  assert.deepEqual(parsed, {
    humanAgentId: "h1",
    inboxId: "i1",
    outboxId: "o1",
    grantId: "g1",
    bootstrapAuthRequestId: "ar1",
    auditEventId: "ae1",
  });
});

test("extractSessionJwt — pulls JWT from a realistic Set-Cookie header", () => {
  const header =
    "baby_phi_session=eyJhbGciOiJIUzI1NiJ9.payload.sig; HttpOnly; SameSite=Lax; Path=/; Expires=Wed, 01 Jan 2030 00:00:00 GMT";
  assert.equal(
    extractSessionJwt(header),
    "eyJhbGciOiJIUzI1NiJ9.payload.sig",
  );
});

test("extractSessionJwt — returns null when cookie absent", () => {
  assert.equal(
    extractSessionJwt("some_other_cookie=value; Path=/"),
    null,
  );
});

test("extractSessionJwt — returns null when header is null", () => {
  assert.equal(extractSessionJwt(null), null);
});

test("extractSessionJwt — honours custom cookie name", () => {
  assert.equal(
    extractSessionJwt("my_custom_sess=abc.def.ghi; Path=/", "my_custom_sess"),
    "abc.def.ghi",
  );
});
