// Pure translator tests for lib/api/mcp-servers.ts.

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  cascadeSummary,
  isNarrowing,
  parseListBody,
  parseServerSummary,
} from "../lib/api/mcp-servers.ts";

test("parseServerSummary — maps wire fields into domain shape", () => {
  const domain = parseServerSummary({
    id: "s1",
    display_name: "memory-mcp",
    kind: "mcp",
    endpoint: "stdio:///usr/local/bin/memory-mcp",
    secret_ref: "mcp-memory-key",
    tenants_allowed: { mode: "all" },
    status: "ok",
    archived_at: null,
    created_at: "2026-04-21T12:00:00Z",
  });
  assert.equal(domain.id, "s1");
  assert.equal(domain.displayName, "memory-mcp");
  assert.equal(domain.kind, "mcp");
  assert.equal(domain.endpoint, "stdio:///usr/local/bin/memory-mcp");
  assert.equal(domain.secretRef, "mcp-memory-key");
  assert.deepEqual(domain.tenantsAllowed, { mode: "all" });
});

test("parseListBody — preserves order and count", () => {
  const list = parseListBody({
    servers: [
      {
        id: "a",
        display_name: "one",
        kind: "mcp",
        endpoint: "stdio:///one",
        secret_ref: null,
        tenants_allowed: { mode: "all" },
        status: "ok",
        archived_at: null,
        created_at: "2026-04-20",
      },
      {
        id: "b",
        display_name: "two",
        kind: "mcp",
        endpoint: "http://two.local/",
        secret_ref: "two-key",
        tenants_allowed: { mode: "only", orgs: ["org-1", "org-2"] },
        status: "archived",
        archived_at: "2026-04-21",
        created_at: "2026-04-21",
      },
    ],
  });
  assert.equal(list.length, 2);
  assert.equal(list[0].displayName, "one");
  assert.equal(list[1].endpoint, "http://two.local/");
});

test("isNarrowing — strict subset detection", () => {
  // Only → Only (shrink)
  assert.equal(
    isNarrowing(
      { mode: "only", orgs: ["a", "b"] },
      { mode: "only", orgs: ["a"] },
    ),
    true,
    "dropping an org is narrowing",
  );
  // Only → Only (no change)
  assert.equal(
    isNarrowing(
      { mode: "only", orgs: ["a"] },
      { mode: "only", orgs: ["a"] },
    ),
    false,
    "same set is not narrowing",
  );
  // Only → Only (widen)
  assert.equal(
    isNarrowing(
      { mode: "only", orgs: ["a"] },
      { mode: "only", orgs: ["a", "b"] },
    ),
    false,
    "widening is not narrowing",
  );
  // All → Only — must count as narrowing (the client can't enumerate
  // the full platform org index, but the server will still audit)
  assert.equal(
    isNarrowing({ mode: "all" }, { mode: "only", orgs: ["a"] }),
    true,
    "All → Only is narrowing",
  );
  // Only → All — widens
  assert.equal(
    isNarrowing({ mode: "only", orgs: ["a"] }, { mode: "all" }),
    false,
    "Only → All is widening",
  );
});

test("cascadeSummary — aggregates counts across revocations", () => {
  const summary = cascadeSummary({
    mcp_server_id: "s1",
    cascade: [
      { org: "org-a", auth_request: "ar-1", revoked_grants: ["g1", "g2"] },
      { org: "org-b", auth_request: "ar-2", revoked_grants: ["g3"] },
    ],
    audit_event_id: "ev-1",
  });
  assert.equal(summary.orgCount, 2);
  assert.equal(summary.arCount, 2);
  assert.equal(summary.grantCount, 3);
  assert.equal(summary.isNarrowing, true);
});

test("cascadeSummary — empty cascade reports isNarrowing=false", () => {
  const summary = cascadeSummary({
    mcp_server_id: "s1",
    cascade: [],
    audit_event_id: null,
  });
  assert.equal(summary.orgCount, 0);
  assert.equal(summary.arCount, 0);
  assert.equal(summary.grantCount, 0);
  assert.equal(summary.isNarrowing, false);
});
