// Pure translator tests for lib/api/agents.ts (M4/P4).
//
// The translator is trivial — it mostly builds a URL and passes
// through snake_case wire types. The URL builder is the interesting
// piece (query-param composition with optional filters), so that's
// where the tests sit.

import { test } from "node:test";
import assert from "node:assert/strict";

import { buildListAgentsUrl } from "../lib/api/agents.ts";

const ORG_ID = "aaaa-bbbb-cccc-dddd";

test("buildListAgentsUrl — no filters", () => {
  const url = buildListAgentsUrl(ORG_ID, undefined);
  assert.equal(url.endsWith(`/api/v0/orgs/${ORG_ID}/agents`), true);
});

test("buildListAgentsUrl — role filter only", () => {
  const url = buildListAgentsUrl(ORG_ID, { role: "intern" });
  assert.equal(
    url.endsWith(`/api/v0/orgs/${ORG_ID}/agents?role=intern`),
    true,
  );
});

test("buildListAgentsUrl — search filter only", () => {
  const url = buildListAgentsUrl(ORG_ID, { search: "alpha bot" });
  assert.equal(
    url.endsWith(`/api/v0/orgs/${ORG_ID}/agents?search=alpha+bot`),
    true,
  );
});

test("buildListAgentsUrl — both filters", () => {
  const url = buildListAgentsUrl(ORG_ID, {
    role: "contract",
    search: "bot",
  });
  assert.match(url, /role=contract/);
  assert.match(url, /search=bot/);
});

test("buildListAgentsUrl — null filters are treated as absent", () => {
  const url = buildListAgentsUrl(ORG_ID, { role: null, search: null });
  assert.equal(url.endsWith(`/api/v0/orgs/${ORG_ID}/agents`), true);
});

test("buildListAgentsUrl — special-character orgId is percent-encoded", () => {
  const url = buildListAgentsUrl("a b c", { role: "admin" });
  assert.match(url, /orgs\/a%20b%20c\/agents/);
});
