// Stable-code registry for the M2 admin API error envelope.
//
// The Rust server emits `{ code: "<STABLE_CODE>", message: "<text>" }`
// for every 4xx/5xx from `server::handler_support::ApiError`. The
// table below maps each code to a human hint rendered by
// <ApiErrorAlert />. Keep this file in sync with
// `modules/crates/server/src/handler_support/errors.rs` — additions
// land with the phase that introduces them.
//
// A missing code returns `null` (no hint). That's intentional: callers
// always show `error.message`, the hint is supplementary.

export const KNOWN_CODES = {
  // Auth / session (M2/P3)
  UNAUTHENTICATED: "Your session cookie is missing or expired. Sign in again.",
  PLATFORM_ADMIN_CLAIMED:
    "A platform admin already exists; the bootstrap flow is closed.",

  // Validation (M1 bootstrap + M2 writes)
  VALIDATION_FAILED: "One or more submitted fields failed validation.",

  // Permission Check failures (M2/P3 — handler_support mapping, D10)
  CATALOGUE_MISS: "The target resource is not registered in the catalogue.",
  MANIFEST_EMPTY: "The request did not resolve to any permission manifest.",
  NO_GRANTS_HELD: "Your identity holds no grants against this resource.",
  CEILING_EMPTIED: "Narrowing by the resource ceiling left no admissible grants.",
  NO_MATCHING_GRANT: "No grant matched the required action + resource.",
  CONSTRAINT_VIOLATION: "A constraint (e.g. `purpose=reveal`) was not satisfied.",
  SCOPE_UNRESOLVABLE: "The request's scope could not be resolved.",
  AWAITING_CONSENT: "A consent record is required before this write can proceed.",

  // Emitter / internal
  AUDIT_EMIT_FAILED:
    "The server failed to persist the audit-log event; the write was rolled back.",
  INTERNAL_ERROR: "Server-side error. Check the server logs for details.",
} as const;

export type StableCode = keyof typeof KNOWN_CODES;

export function humanMessageForCode(code: string): string | null {
  if (code in KNOWN_CODES) {
    return KNOWN_CODES[code as StableCode];
  }
  return null;
}
