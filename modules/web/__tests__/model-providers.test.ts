// Pure wire-translator tests for lib/api/model-providers.ts.

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  parseListBody,
  parseProviderSummary,
} from "../lib/api/model-providers.ts";

test("parseProviderSummary — extracts display fields from phi-core config", () => {
  const domain = parseProviderSummary({
    id: "pid-1",
    config: {
      id: "claude-sonnet-4",
      name: "Claude Sonnet 4",
      api: "anthropic_messages",
      provider: "anthropic",
      base_url: "https://api.anthropic.com",
    },
    secret_ref: "anthropic-api-key",
    tenants_allowed: { mode: "all" },
    status: "ok",
    archived_at: null,
    created_at: "2026-04-21T12:00:00Z",
  });
  assert.equal(domain.id, "pid-1");
  assert.equal(domain.modelId, "claude-sonnet-4");
  assert.equal(domain.modelName, "Claude Sonnet 4");
  assert.equal(domain.providerKind, "anthropic_messages");
  assert.equal(domain.providerLabel, "anthropic");
  assert.equal(domain.baseUrl, "https://api.anthropic.com");
  assert.equal(domain.secretRef, "anthropic-api-key");
});

test("parseProviderSummary — missing fields fall back gracefully", () => {
  const domain = parseProviderSummary({
    id: "pid-2",
    config: {},
    secret_ref: "k",
    tenants_allowed: { mode: "all" },
    status: "ok",
    archived_at: null,
    created_at: "2026-04-21T12:00:00Z",
  });
  assert.equal(domain.modelId, "(unknown)");
  assert.equal(domain.providerKind, "(unknown)");
});

test("parseListBody — preserves order and count", () => {
  const list = parseListBody({
    providers: [
      {
        id: "a",
        config: { id: "m1", provider: "anthropic", api: "anthropic_messages" },
        secret_ref: "k1",
        tenants_allowed: { mode: "all" },
        status: "ok",
        archived_at: null,
        created_at: "2026-04-20",
      },
      {
        id: "b",
        config: { id: "m2", provider: "openai", api: "openai_completions" },
        secret_ref: "k2",
        tenants_allowed: { mode: "all" },
        status: "archived",
        archived_at: "2026-04-21",
        created_at: "2026-04-21",
      },
    ],
  });
  assert.equal(list.length, 2);
  assert.equal(list[0].providerLabel, "anthropic");
  assert.equal(list[1].providerLabel, "openai");
  assert.equal(list[1].archivedAt, "2026-04-21");
});

test("parseListBody — empty list round-trips", () => {
  const list = parseListBody({ providers: [] });
  assert.equal(list.length, 0);
});
