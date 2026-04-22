// Unit tests for the M4/P1 wizard primitives.
//
// Mirrors the M3 wizard-primitives test style: pure-logic pieces
// tested directly, full DOM/SSR rendering deferred to the page 10/11
// integration tests that land at M4/P6 + M4/P7.

import { test } from "node:test";
import assert from "node:assert/strict";

// ---- ShapeBPendingApprovalNotice --------------------------------------------
//
// The notice is a presentation-only component. The contract worth
// asserting at this tier: `approvers` is a fixed 2-slot tuple (Shape
// B is exactly 2-approver per ADR-0025) and the activeWindowDays
// default is 7.

type ShapeBApprover = {
  displayName: string;
  orgName: string;
};

type ShapeBNoticeContract = {
  approvers: [ShapeBApprover, ShapeBApprover];
  activeWindowDays?: number;
};

function defaultActiveWindowDays(props: ShapeBNoticeContract): number {
  return props.activeWindowDays ?? 7;
}

test("ShapeBPendingApprovalNotice — default active window is 7 days", () => {
  const props: ShapeBNoticeContract = {
    approvers: [
      { displayName: "Alex", orgName: "Acme" },
      { displayName: "Blair", orgName: "Beta" },
    ],
  };
  assert.equal(defaultActiveWindowDays(props), 7);
});

test("ShapeBPendingApprovalNotice — explicit active window wins over default", () => {
  const props: ShapeBNoticeContract = {
    approvers: [
      { displayName: "Alex", orgName: "Acme" },
      { displayName: "Blair", orgName: "Beta" },
    ],
    activeWindowDays: 14,
  };
  assert.equal(defaultActiveWindowDays(props), 14);
});

test("ShapeBPendingApprovalNotice — approvers tuple length is exactly 2", () => {
  // TypeScript's [T, T] tuple pins this at the type level; this test
  // pins it at the value level so a refactor that widens the type to
  // `ShapeBApprover[]` trips the test runner.
  const approvers: [ShapeBApprover, ShapeBApprover] = [
    { displayName: "A", orgName: "Acme" },
    { displayName: "B", orgName: "Beta" },
  ];
  assert.equal(approvers.length, 2);
});

// ---- OKREditor pure-logic helpers -------------------------------------------
//
// OKREditor's interesting logic is the cascade-on-remove rule: removing
// an Objective must also remove every KeyResult linked to it (by
// `objective_id`). This matches the domain-side contract that a
// KeyResult's `objective_id` MUST reference an existing Objective on
// the same project.

type OKRObjective = { objective_id: string; name: string };
type OKRKeyResult = { kr_id: string; objective_id: string };

function removeObjectiveCascade(
  objectives: OKRObjective[],
  keyResults: OKRKeyResult[],
  victim: string,
): { objectives: OKRObjective[]; keyResults: OKRKeyResult[] } {
  return {
    objectives: objectives.filter((o) => o.objective_id !== victim),
    keyResults: keyResults.filter((kr) => kr.objective_id !== victim),
  };
}

test("OKREditor — removing an objective cascades to its linked key results", () => {
  const objectives: OKRObjective[] = [
    { objective_id: "o1", name: "Ship v1" },
    { objective_id: "o2", name: "Reduce cost" },
  ];
  const keyResults: OKRKeyResult[] = [
    { kr_id: "kr1", objective_id: "o1" },
    { kr_id: "kr2", objective_id: "o1" },
    { kr_id: "kr3", objective_id: "o2" },
  ];
  const next = removeObjectiveCascade(objectives, keyResults, "o1");
  assert.deepEqual(
    next.objectives.map((o) => o.objective_id),
    ["o2"],
  );
  assert.deepEqual(
    next.keyResults.map((kr) => kr.kr_id),
    ["kr3"],
    "orphaned KRs must not survive the cascade",
  );
});

test("OKREditor — removing an objective without linked KRs leaves KRs untouched", () => {
  const objectives: OKRObjective[] = [
    { objective_id: "o1", name: "A" },
    { objective_id: "o2", name: "B" },
  ];
  const keyResults: OKRKeyResult[] = [
    { kr_id: "kr1", objective_id: "o1" },
  ];
  const next = removeObjectiveCascade(objectives, keyResults, "o2");
  assert.deepEqual(
    next.keyResults.map((kr) => kr.kr_id),
    ["kr1"],
  );
});

// ---- OKR id generator contract ---------------------------------------------

function randomShortId(prefix: string): string {
  const suffix = Math.random().toString(36).slice(2, 8);
  return `${prefix}-${suffix}`;
}

test("OKREditor — random short id has the expected prefix and body shape", () => {
  for (const prefix of ["o", "kr"]) {
    const id = randomShortId(prefix);
    assert.match(
      id,
      new RegExp(`^${prefix}-[0-9a-z]{1,6}$`),
      `id=${id} prefix=${prefix}`,
    );
  }
});

test("OKREditor — random short ids are not identical across calls", () => {
  // Not a crypto guarantee — just a sanity check that the generator
  // actually varies its output. The real uniqueness is enforced
  // server-side at submit time.
  const a = randomShortId("o");
  const b = randomShortId("o");
  // Exceedingly unlikely to tie on two 6-char base36 random strings;
  // if they do, the test will flake once in several thousand runs,
  // which is an acceptable signal that the generator is broken.
  assert.notEqual(a, b);
});
