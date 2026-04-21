// Pure wire-translator + base64 tests for lib/api/secrets.ts.
//
// No network, no Next runtime — just the pure functions under
// Node's built-in test runner.

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  decodeB64NoPad,
  encodeB64NoPad,
  parseListBody,
  parseSecretSummary,
} from "../lib/api/secrets.ts";

test("encodeB64NoPad — strips trailing `=`", () => {
  // "hello" is 5 bytes, base64 = "aGVsbG8=" (one pad). No-pad = "aGVsbG8".
  const out = encodeB64NoPad(new TextEncoder().encode("hello"));
  assert.equal(out, "aGVsbG8");
});

test("decodeB64NoPad — accepts input without padding", () => {
  const bytes = decodeB64NoPad("aGVsbG8");
  assert.equal(new TextDecoder().decode(bytes), "hello");
});

test("encodeB64NoPad/decodeB64NoPad round-trip on arbitrary bytes", () => {
  const sample = new Uint8Array([0, 1, 2, 3, 254, 255, 127, 128]);
  const encoded = encodeB64NoPad(sample);
  const decoded = decodeB64NoPad(encoded);
  assert.deepEqual(Array.from(decoded), Array.from(sample));
});

test("parseSecretSummary — wire → domain (renames snake→camel)", () => {
  const domain = parseSecretSummary({
    id: "sid-1",
    slug: "anthropic-api-key",
    custodian_id: "agent-1",
    sensitive: true,
    last_rotated_at: "2026-04-21T10:00:00Z",
    created_at: "2026-04-20T10:00:00Z",
  });
  assert.equal(domain.id, "sid-1");
  assert.equal(domain.slug, "anthropic-api-key");
  assert.equal(domain.custodianId, "agent-1");
  assert.equal(domain.sensitive, true);
  assert.equal(domain.lastRotatedAt, "2026-04-21T10:00:00Z");
  assert.equal(domain.createdAt, "2026-04-20T10:00:00Z");
});

test("parseListBody — empty array round-trips", () => {
  const list = parseListBody({ secrets: [] });
  assert.equal(list.length, 0);
});

test("parseListBody — preserves order and count", () => {
  const list = parseListBody({
    secrets: [
      {
        id: "a",
        slug: "one",
        custodian_id: "x",
        sensitive: false,
        last_rotated_at: null,
        created_at: "2026-04-20",
      },
      {
        id: "b",
        slug: "two",
        custodian_id: "x",
        sensitive: true,
        last_rotated_at: null,
        created_at: "2026-04-21",
      },
    ],
  });
  assert.equal(list.length, 2);
  assert.equal(list[0].slug, "one");
  assert.equal(list[1].slug, "two");
});
