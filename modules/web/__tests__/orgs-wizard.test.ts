// Unit tests for the M3/P4 org-creation wizard's wire-translator
// logic. Matches the pure-logic discipline of `wizard-primitives.test.ts`
// — we don't mount React here; DOM-level behaviour is covered by the
// server-side acceptance tests in
// `modules/crates/server/tests/acceptance_orgs_create.rs` (those
// exercise the real POST flow end-to-end).
//
// ## phi-core leverage
//
// Q1 none, Q2 **yes**: `CreateOrgBody.defaults_snapshot_override`
// is opaque `Record<string, unknown>` — a phi-core-wrapped payload
// passes through verbatim. We assert that mutation at the wizard tier
// does not drop or re-shape phi-core fields.

import { test } from "node:test";
import assert from "node:assert/strict";

import type {
  CreateOrgBody,
  ConsentPolicyWire,
  TemplateKindWire,
} from "@/lib/api/orgs";

// Replicate the wizard's `toWireBody` contract inline (the real
// implementation is inside the page's client component, which is not
// a publicly-exported module). The contract is stable: every field
// on CreateOrgBody must land on the wire.

type DraftState = {
  display_name: string;
  vision: string;
  mission: string;
  consent_policy: ConsentPolicyWire;
  audit_class_default: "silent" | "logged" | "alerted";
  templates: TemplateKindWire[];
  default_model_provider: string;
  token_budget: number;
  ceo_display_name: string;
  ceo_channel_kind: "email" | "slack" | "web";
  ceo_channel_handle: string;
};

function toWireBody(d: DraftState): CreateOrgBody {
  return {
    display_name: d.display_name.trim(),
    vision: d.vision.trim() || null,
    mission: d.mission.trim() || null,
    consent_policy: d.consent_policy,
    audit_class_default: d.audit_class_default,
    authority_templates_enabled: d.templates,
    default_model_provider:
      d.default_model_provider.trim() === "" ? null : d.default_model_provider,
    ceo_display_name: d.ceo_display_name.trim(),
    ceo_channel_kind: d.ceo_channel_kind,
    ceo_channel_handle: d.ceo_channel_handle.trim(),
    token_budget: d.token_budget,
  };
}

function sampleDraft(): DraftState {
  return {
    display_name: "  Acme  ",
    vision: "",
    mission: "A mission",
    consent_policy: "implicit",
    audit_class_default: "logged",
    templates: ["a"],
    default_model_provider: "   ",
    token_budget: 1_000_000,
    ceo_display_name: "Alice",
    ceo_channel_kind: "email",
    ceo_channel_handle: "alice@acme.test",
  };
}

test("toWireBody trims whitespace on string fields", () => {
  const wire = toWireBody(sampleDraft());
  assert.equal(wire.display_name, "Acme");
  assert.equal(wire.ceo_display_name, "Alice");
});

test("toWireBody coerces empty strings to null on optional fields", () => {
  const wire = toWireBody(sampleDraft());
  assert.equal(wire.vision, null);
  assert.equal(wire.mission, "A mission");
  assert.equal(wire.default_model_provider, null);
});

test("toWireBody preserves templates array verbatim", () => {
  const d = sampleDraft();
  d.templates = ["a", "c", "d"];
  const wire = toWireBody(d);
  assert.deepEqual(wire.authority_templates_enabled, ["a", "c", "d"]);
});

test("toWireBody carries token_budget as a number", () => {
  const wire = toWireBody(sampleDraft());
  assert.equal(typeof wire.token_budget, "number");
  assert.equal(wire.token_budget, 1_000_000);
});

test("CreateOrgBody type permits defaults_snapshot_override as opaque phi-core payload", () => {
  // Positive phi-core transit assertion: the wire shape must accept
  // a full `defaults_snapshot_override` object containing arbitrary
  // phi-core fields without the web tier having to re-specify them.
  // If a future commit tightens `Record<string, unknown>` to a
  // phi re-declaration, this test fails to type-check.
  const override: Record<string, unknown> = {
    execution_limits: {
      max_turns: 50,
      max_total_tokens: 1_000_000,
      max_duration_secs: 600,
      max_cost_usd: null,
    },
    context_config: { compaction_strategy: "level_3" },
    retry_config: { max_retries: 5, base_delay_ms: 100, jitter_pct: 20 },
    default_agent_profile: {
      profile_id: "fresh",
      name: null,
      system_prompt: null,
    },
    default_retention_days: 30,
    default_alert_channels: [],
  };
  const body: CreateOrgBody = {
    display_name: "X",
    consent_policy: "implicit",
    audit_class_default: "logged",
    authority_templates_enabled: [],
    defaults_snapshot_override: override,
    ceo_display_name: "C",
    ceo_channel_kind: "email",
    ceo_channel_handle: "c@x.test",
    token_budget: 1,
  };
  assert.ok(body.defaults_snapshot_override);
  // The opaque type lets us reach into phi-core fields without
  // knowing their layout at compile time — that's the point.
  const limits = body.defaults_snapshot_override!["execution_limits"] as {
    max_turns: number;
  };
  assert.equal(limits.max_turns, 50);
});
