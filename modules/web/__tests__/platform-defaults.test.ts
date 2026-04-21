// Pure translator tests for lib/api/platform-defaults.ts.

import { test } from "node:test";
import assert from "node:assert/strict";

import {
  parseDefaults,
  toWire,
  type PlatformDefaultsWire,
} from "../lib/api/platform-defaults.ts";

const sampleWire: PlatformDefaultsWire = {
  singleton: 1,
  execution_limits: { max_turns: 50, max_total_tokens: 1_000_000 },
  default_agent_profile: { profile_id: "p-1" },
  context_config: { max_context_tokens: 100_000 },
  retry_config: { max_retries: 3 },
  default_retention_days: 30,
  default_alert_channels: ["ops@example.com"],
  updated_at: "2026-04-21T12:00:00Z",
  version: 5,
};

test("parseDefaults — maps wire fields to domain shape", () => {
  const d = parseDefaults(sampleWire);
  assert.equal(d.version, 5);
  assert.equal(d.updatedAt, "2026-04-21T12:00:00Z");
  assert.equal(d.defaultRetentionDays, 30);
  assert.deepEqual(d.defaultAlertChannels, ["ops@example.com"]);
  assert.deepEqual(d.executionLimits, { max_turns: 50, max_total_tokens: 1_000_000 });
  assert.deepEqual(d.defaultAgentProfile, { profile_id: "p-1" });
  assert.deepEqual(d.contextConfig, { max_context_tokens: 100_000 });
  assert.deepEqual(d.retryConfig, { max_retries: 3 });
});

test("toWire — round-trip via parseDefaults is identity", () => {
  const domain = parseDefaults(sampleWire);
  const back = toWire(domain);
  assert.deepEqual(back, sampleWire);
});

test("toWire — always sets singleton to 1", () => {
  // Even if a caller somehow handed us a domain struct without the
  // singleton field the server requires it; toWire must always
  // serialise with singleton = 1.
  const domain = parseDefaults(sampleWire);
  const back = toWire(domain);
  assert.equal(back.singleton, 1);
});

test("parseDefaults — phi-core sections stay opaque as Record<string, unknown>", () => {
  // Web tier must not re-specify phi-core field layouts. When
  // phi-core adds a new field, it should flow through transparently.
  const wire: PlatformDefaultsWire = {
    ...sampleWire,
    execution_limits: {
      max_turns: 100,
      new_phi_core_field: "future-value",
    },
  };
  const d = parseDefaults(wire);
  // The new field rides through as-is — translator does not drop
  // unknown keys on phi-core sections.
  assert.equal(
    (d.executionLimits as Record<string, unknown>).new_phi_core_field,
    "future-value",
  );
});
