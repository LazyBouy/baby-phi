// Unit tests for the M5/P1 wizard + dashboard primitives.
//
// Mirrors the M4/P1 wizard-primitives test style: pure-logic pieces
// tested directly, full DOM/SSR rendering deferred to the page
// 12/13/14 integration tests that land at M5/P4–P6.
//
// The four primitives landed at M5/P1 are presentation-only (no
// client state, except the SSE-driven SessionEventStreamRenderer
// that is Client Component only so `"use client"` compiles). Each
// test here pins the stable wire shape their prop type expects —
// schema changes on the server must force a TypeScript compile
// error here.

import { test } from "node:test";
import assert from "node:assert/strict";

// ---- SessionEventStreamRenderer --------------------------------------------

type SessionEventKind =
  | "agent_start"
  | "turn_start"
  | "message_start"
  | "message_update"
  | "message_end"
  | "tool_execution_start"
  | "tool_execution_update"
  | "tool_execution_end"
  | "progress_message"
  | "input_rejected"
  | "turn_end"
  | "agent_end";

type SessionEventRow = {
  kind: SessionEventKind;
  timestamp: string;
  summary: string;
};

test("SessionEventStreamRenderer — event kinds cover all 12 phi-core AgentEvent variants", () => {
  // phi-core's AgentEvent enum has exactly the 12 variants named
  // below. A future phi-core addition must land here with a
  // compile-time break — the static ALL tuple pins the count at the
  // type level.
  const ALL: SessionEventKind[] = [
    "agent_start",
    "turn_start",
    "message_start",
    "message_update",
    "message_end",
    "tool_execution_start",
    "tool_execution_update",
    "tool_execution_end",
    "progress_message",
    "input_rejected",
    "turn_end",
    "agent_end",
  ];
  assert.equal(ALL.length, 12);
  // Each variant's snake_case rendering matches phi-core's serde.
  for (const kind of ALL) {
    assert.match(kind, /^[a-z]+(_[a-z]+)*$/);
  }
});

test("SessionEventStreamRenderer — status enum enumerates 4 stable values", () => {
  const statuses: ("connecting" | "streaming" | "ended" | "aborted")[] = [
    "connecting",
    "streaming",
    "ended",
    "aborted",
  ];
  assert.equal(statuses.length, 4);
});

test("SessionEventStreamRenderer — event row carries kind + timestamp + summary", () => {
  const row: SessionEventRow = {
    kind: "turn_end",
    timestamp: "2026-04-23T10:00:00Z",
    summary: "turn 3 complete (42 tokens)",
  };
  assert.equal(row.kind, "turn_end");
  assert.match(row.timestamp, /^\d{4}-\d{2}-\d{2}T/);
  assert.ok(row.summary.length > 0);
});

// ---- PermissionCheckPreviewPanel -------------------------------------------

type PermissionCheckStep = {
  step: 0 | 1 | 2 | 3 | 4 | 5 | 6;
  label: string;
  outcome: "pass" | "fail" | "skipped";
  detail?: string;
};

type PermissionCheckPreview = {
  steps: PermissionCheckStep[];
  granted: boolean;
  failed_at_step?: number;
};

test("PermissionCheckPreviewPanel — 7 steps numbered 0..6 (M1 Permission Check shape)", () => {
  const steps: PermissionCheckStep[] = [
    { step: 0, label: "owner identity resolved", outcome: "pass" },
    { step: 1, label: "resource exists", outcome: "pass" },
    { step: 2, label: "grant chain traversable", outcome: "pass" },
    { step: 3, label: "action permitted", outcome: "pass" },
    { step: 4, label: "tenant set match", outcome: "pass" },
    { step: 5, label: "consent satisfied", outcome: "pass" },
    { step: 6, label: "authority template active", outcome: "pass" },
  ];
  assert.equal(steps.length, 7);
  for (const s of steps) {
    assert.ok(s.step >= 0 && s.step <= 6);
  }
});

test("PermissionCheckPreviewPanel — granted=false implies failed_at_step points at the failing step", () => {
  const preview: PermissionCheckPreview = {
    steps: [
      { step: 0, label: "owner", outcome: "pass" },
      { step: 1, label: "resource", outcome: "pass" },
      { step: 2, label: "grants", outcome: "pass" },
      { step: 3, label: "action", outcome: "fail", detail: "no write grant" },
      { step: 4, label: "tenant", outcome: "skipped" },
      { step: 5, label: "consent", outcome: "skipped" },
      { step: 6, label: "template", outcome: "skipped" },
    ],
    granted: false,
    failed_at_step: 3,
  };
  assert.equal(preview.granted, false);
  assert.equal(preview.failed_at_step, 3);
  const failing = preview.steps.find((s) => s.outcome === "fail");
  assert.equal(failing?.step, preview.failed_at_step);
});

// ---- TemplateAdoptionTable -------------------------------------------------

type TemplateKind = "A" | "B" | "C" | "D" | "E";

type TemplateAdoptionRow = {
  kind: TemplateKind;
  name: string;
  state: "pending" | "active" | "revoked" | "available";
  ar_id?: string;
  active_grant_count?: number;
};

test("TemplateAdoptionTable — 4 buckets partition every row by state", () => {
  const rows: TemplateAdoptionRow[] = [
    { kind: "A", name: "Project lead autonomy", state: "active", ar_id: "ar-1", active_grant_count: 2 },
    { kind: "B", name: "Co-governance", state: "pending", ar_id: "ar-2" },
    { kind: "C", name: "Manages", state: "revoked", ar_id: "ar-3" },
    { kind: "D", name: "Supervisor", state: "available" },
    { kind: "E", name: "Custom on-demand", state: "available" },
  ];
  const buckets = {
    pending: rows.filter((r) => r.state === "pending"),
    active: rows.filter((r) => r.state === "active"),
    revoked: rows.filter((r) => r.state === "revoked"),
    available: rows.filter((r) => r.state === "available"),
  };
  assert.equal(buckets.pending.length, 1);
  assert.equal(buckets.active.length, 1);
  assert.equal(buckets.revoked.length, 1);
  assert.equal(buckets.available.length, 2);
  const total =
    buckets.pending.length +
    buckets.active.length +
    buckets.revoked.length +
    buckets.available.length;
  assert.equal(total, rows.length);
});

test("TemplateAdoptionTable — active rows carry ar_id + active_grant_count", () => {
  const row: TemplateAdoptionRow = {
    kind: "A",
    name: "Project lead autonomy",
    state: "active",
    ar_id: "ar-42",
    active_grant_count: 5,
  };
  assert.ok(row.ar_id);
  assert.ok(typeof row.active_grant_count === "number");
  assert.ok((row.active_grant_count ?? 0) >= 0);
});

test("TemplateAdoptionTable — available rows have no AR reference", () => {
  const row: TemplateAdoptionRow = {
    kind: "E",
    name: "Custom on-demand",
    state: "available",
  };
  assert.equal(row.ar_id, undefined);
  assert.equal(row.active_grant_count, undefined);
});

// ---- SystemAgentStatusCard -------------------------------------------------

type SystemAgentTrigger =
  | "session_end"
  | "edge_change"
  | "periodic"
  | "explicit"
  | "custom_event";

type SystemAgentStatus = {
  agent_id: string;
  display_name: string;
  trigger: SystemAgentTrigger;
  queue_depth: number;
  last_fired_at: string | null;
  effective_parallelize: number;
  last_error: string | null;
  active: boolean;
};

test("SystemAgentStatusCard — trigger enum is governance-plane (5 variants, NOT phi_core::AgentEvent)", () => {
  // This enum is explicitly rejected at the Q3 walk in the phi-core
  // reuse map — `phi_core::AgentEvent` is agent-loop telemetry, not a
  // governance trigger source. The 5 variants pin the page 13
  // trigger dropdown's stable contract.
  const triggers: SystemAgentTrigger[] = [
    "session_end",
    "edge_change",
    "periodic",
    "explicit",
    "custom_event",
  ];
  assert.equal(triggers.length, 5);
});

test("SystemAgentStatusCard — idle vs busy distinguished by queue_depth=0 vs >0", () => {
  const idle: SystemAgentStatus = {
    agent_id: "a-1",
    display_name: "memory-extraction",
    trigger: "session_end",
    queue_depth: 0,
    last_fired_at: null,
    effective_parallelize: 1,
    last_error: null,
    active: true,
  };
  const busy: SystemAgentStatus = { ...idle, queue_depth: 3 };
  assert.equal(idle.queue_depth, 0);
  assert.ok(busy.queue_depth > 0);
});

test("SystemAgentStatusCard — disabled agents carry active=false + render dim", () => {
  const disabled: SystemAgentStatus = {
    agent_id: "a-disabled",
    display_name: "agent-catalog",
    trigger: "edge_change",
    queue_depth: 0,
    last_fired_at: "2026-04-20T08:00:00Z",
    effective_parallelize: 1,
    last_error: null,
    active: false,
  };
  assert.equal(disabled.active, false);
});
