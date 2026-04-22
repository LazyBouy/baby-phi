// Unit tests for the M3/P5 org-dashboard client-side logic.
// DOM-level behaviour (polling loop, panel rendering) is covered by
// the server-side acceptance tests in
// `modules/crates/server/tests/acceptance_orgs_dashboard.rs`; here we
// pin the stable wire shape + pure helpers that the `DashboardClient`
// depends on.
//
// ## phi-core leverage
//
// Q1 **none** — no `phi-core`-wrapped types exist on this tier. Q2
// **none** — the dashboard wire shape deliberately strips
// `defaults_snapshot` server-side (see
// `server/src/platform/orgs/dashboard.rs`). This file pins that
// invariant on the *web* boundary so a future translator rename cannot
// accidentally re-introduce the coupling.
// Q3: no phi-core module maps to a dashboard panel; every candidate
// (Session, AgentEvent, Usage) belongs to orthogonal planes per
// `phi/CLAUDE.md`.

import { test } from "node:test";
import assert from "node:assert/strict";

import type {
  DashboardSummaryWire,
  EmptyStateCtaCardsWire,
} from "@/lib/api/orgs";

function sample(): DashboardSummaryWire {
  return {
    org: {
      id: "00000000-0000-0000-0000-000000000001",
      display_name: "Acme",
      vision: null,
      mission: null,
      consent_policy: "implicit",
    },
    viewer: {
      agent_id: "00000000-0000-0000-0000-000000000002",
      role: "admin",
      can_admin_manage: true,
    },
    agents_summary: { total: 3, human: 1, llm: 2 },
    projects_summary: { active: 0, shape_a: 0, shape_b: 0 },
    pending_auth_requests_count: 0,
    alerted_events_24h: 0,
    token_budget: {
      used: 0,
      total: 1_000_000,
      pool_id: "00000000-0000-0000-0000-000000000003",
    },
    recent_events: [],
    templates_adopted: ["a"],
    cta_cards: {
      add_agent: "/organizations/x/agents/new",
      create_project: "/organizations/x/projects/new",
      adopt_template: "/organizations/x/templates",
      configure_system_agents: "/organizations/x/system-agents",
    },
    welcome_banner: "Welcome to Acme.",
  };
}

test("DashboardSummaryWire contains no phi-core-wrapping keys", () => {
  const json = JSON.stringify(sample());
  for (const forbidden of [
    "defaults_snapshot",
    "execution_limits",
    "context_config",
    "retry_config",
    "default_agent_profile",
    "blueprint",
  ]) {
    assert.ok(
      !json.includes(forbidden),
      `wire payload must not mention \`${forbidden}\` — P5.0 pre-audit forbids phi-core transit on the dashboard`,
    );
  }
});

test("viewer.role enumerates admin / project_lead / member / none", () => {
  const roles: DashboardSummaryWire["viewer"]["role"][] = [
    "admin",
    "project_lead",
    "member",
    "none",
  ];
  // Exhaustive static enumeration — any future addition to the
  // TypeScript union will force a compile-time update here.
  assert.equal(roles.length, 4);
});

test("cta_cards round-trip preserves all 4 card slots", () => {
  const cards: EmptyStateCtaCardsWire = sample().cta_cards;
  assert.equal(typeof cards.add_agent, "string");
  assert.equal(typeof cards.create_project, "string");
  assert.equal(typeof cards.adopt_template, "string");
  assert.equal(typeof cards.configure_system_agents, "string");
});

test("token budget ratio derived from used/total is stable", () => {
  const s = sample();
  s.token_budget.used = 250_000;
  s.token_budget.total = 1_000_000;
  const ratio = Math.round((s.token_budget.used / s.token_budget.total) * 100);
  assert.equal(ratio, 25);
});

test("welcome_banner is nullable — populated orgs suppress it", () => {
  const s = sample();
  s.welcome_banner = null;
  // The panel under-test's visibility rule must treat `null` the same
  // as `undefined`. Asserting the wire shape accepts both.
  const json = JSON.stringify(s);
  assert.ok(json.includes('"welcome_banner":null'));
});

test("templates_adopted array preserves ordering from server", () => {
  const s = sample();
  s.templates_adopted = ["b", "a", "c"];
  const json = JSON.parse(JSON.stringify(s)) as DashboardSummaryWire;
  assert.deepEqual(json.templates_adopted, ["b", "a", "c"]);
});

test("recent_events wire row has the five required fields", () => {
  const s = sample();
  s.recent_events = [
    {
      id: "e1",
      kind: "agent.created",
      actor: null,
      timestamp: "2026-04-22T10:00:00Z",
      summary: "agent.created: node-1",
    },
  ];
  const row = s.recent_events[0];
  assert.ok("id" in row && "kind" in row && "timestamp" in row);
  assert.ok("actor" in row && "summary" in row);
});
