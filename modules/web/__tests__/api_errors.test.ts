import { test } from "node:test";
import assert from "node:assert/strict";

import {
  humanMessageForCode,
  KNOWN_CODES,
  type StableCode,
} from "../lib/api/errors.ts";

test("humanMessageForCode returns a hint for every registered code", () => {
  for (const code of Object.keys(KNOWN_CODES) as StableCode[]) {
    const hint = humanMessageForCode(code);
    assert.ok(
      hint && hint.length > 0,
      `expected a non-empty hint for ${code}`,
    );
  }
});

test("humanMessageForCode returns null for unknown codes", () => {
  assert.strictEqual(humanMessageForCode("DEFINITELY_NOT_A_REAL_CODE"), null);
  assert.strictEqual(humanMessageForCode(""), null);
});

test("every Permission-Check failed-step code has a hint", () => {
  // D10 in the M2 plan maps each FailedStep variant to a distinct
  // ApiError.code — the web client must ship a hint for each so
  // operators see something useful in the UI.
  const stepCodes: StableCode[] = [
    "CATALOGUE_MISS",
    "MANIFEST_EMPTY",
    "NO_GRANTS_HELD",
    "CEILING_EMPTIED",
    "NO_MATCHING_GRANT",
    "CONSTRAINT_VIOLATION",
    "SCOPE_UNRESOLVABLE",
    "AWAITING_CONSENT",
  ];
  for (const code of stepCodes) {
    const hint = humanMessageForCode(code);
    assert.ok(hint, `missing hint for Permission-Check code ${code}`);
  }
});
