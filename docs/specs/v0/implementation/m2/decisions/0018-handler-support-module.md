<!-- Last verified: 2026-04-21 by Claude Code -->

# ADR-0018 — `handler_support` as a first-class shim

**Status: Accepted** — shipped in M2/P3; consumed by every M2/P4+ handler.

## Context

M1's `server::bootstrap::claim` handler inlined three boilerplate
patterns that every authenticated M2+ handler would otherwise
re-implement:

1. **Cookie → agent_id parse** with a 401 path on missing/invalid
   cookie. M1 did it per-handler via `verify_from_cookies`.
2. **Permission Check invocation + Decision → HTTP mapping** —
   exhaustively mapping `FailedStep::*` to `(status, stable_code)`
   pairs. M1 only had bootstrap (which bypasses the engine) so the
   mapping never got centralised.
3. **Audit emission error mapping** — an audit-emit failure means
   the write's trail is missing, which should surface as a 500
   `AUDIT_EMIT_FAILED`, not a pass-through of the underlying
   `RepositoryError`.

M2 introduces ≥8 new handlers across pages 02–05. Without a shared
shim, each handler reinvents these three patterns, drifts in
unfounded ways (different 401 shapes, inconsistent stable codes,
missed audit-emit mapping), and the `Decision → HTTP` mapping table
scatters across every handler in which a manifest is constructed.

## Decision

Ship `server::handler_support` **before** any M2 page handler
begins (M2/P3), with four components:

1. **[`AuthenticatedSession`][1]** — axum `FromRequestParts`
   extractor yielding `AuthenticatedSession { agent_id: AgentId,
   claims: SessionClaims }`. Rejects with a fixed
   `ApiError::unauthenticated()` (HTTP 401, stable code
   `UNAUTHENTICATED`) on missing / malformed / expired cookies.
   Handlers never branch on a missing-cookie case; they list the
   extractor in their signature and the machinery does the rest.
2. **[`check_permission`][2] + [`denial_to_api_error`][2]** — the
   function pair every write handler calls. `check_permission`
   invokes the engine, records a duration sample on the metrics
   sink, and maps `Decision` onto `Result<Vec<ResolvedReach>,
   ApiError>` via the D10 table:

   | `Decision` | HTTP | Stable code |
   |---|---|---|
   | `Allowed { resolved_grants }` | — (returns `Ok`) | — |
   | `Pending { .. }` | 202 | `AWAITING_CONSENT` |
   | `Denied { FailedStep::Catalogue, .. }` | 403 | `CATALOGUE_MISS` |
   | `Denied { FailedStep::Expansion, .. }` | 400 | `MANIFEST_EMPTY` |
   | `Denied { FailedStep::Resolution, .. }` | 403 | `NO_GRANTS_HELD` |
   | `Denied { FailedStep::Ceiling, .. }` | 403 | `CEILING_EMPTIED` |
   | `Denied { FailedStep::Match, .. }` | 403 | `NO_MATCHING_GRANT` |
   | `Denied { FailedStep::Constraint, .. }` | 403 | `CONSTRAINT_VIOLATION` |
   | `Denied { FailedStep::Scope, .. }` | 403 | `SCOPE_UNRESOLVABLE` |
   | `Denied { FailedStep::Consent, .. }` | 202 | `AWAITING_CONSENT` |

   The mapping is an exhaustive `match` on `FailedStep`, so adding a
   new variant to the enum breaks compilation rather than producing
   a silent default. Tested by
   [`server/tests/permission_check_mapping_test.rs`][4] against
   `FailedStep::ALL`.

3. **[`emit_audit`][3]** — awaits the trait-object `AuditEmitter`
   and maps any `RepositoryError` to
   `ApiError::audit_emit_failed(..)` (HTTP 500, stable code
   `AUDIT_EMIT_FAILED`). Why not just `.await?`? Because the
   underlying repository error type differs from the handler's
   `ApiError` — and "the audit row didn't land" is a 500
   concern regardless of what the repository said went wrong.

4. **Shared [`ApiError`][5]** — `{ code: &'static str, message:
   String }` wire shape + canonical constructors
   (`unauthenticated`, `validation_failed`, `internal`,
   `audit_emit_failed`). Promoted from `handlers::bootstrap::ApiError`
   at M2/P3; every M2+ handler emits the same shape.

## Consequences

- **Uniform error shape.** Web tier's
  [`lib/api/errors.ts`][6] knows the exhaustive stable-code set;
  `ApiErrorAlert` renders a consistent hint per code.
- **Exhaustiveness is compile-time.** New `FailedStep` variants or
  new denial paths can't silently fall through to a "default 500".
- **Bootstrap migrated cleanly.** `server::bootstrap::claim` still
  emits its domain-specific codes (`BOOTSTRAP_INVALID`,
  `BOOTSTRAP_ALREADY_CONSUMED`, `PLATFORM_ADMIN_CLAIMED`) via
  `ApiError::new(...)` — one shared shape, multiple codes.
- **Testing discipline unchanged.** Handler tests exercise handlers
  via the real axum app; `handler_support` tests
  ([`handler_support_test.rs`][7]) cover all eight `FailedStep`
  variants + 401 on missing cookie + 500 on emitter failure.

## Alternatives considered

1. **Axum middleware for auth instead of an extractor.** Rejected —
   middleware runs before path parameters are resolved; handlers
   downstream would still need to re-read the session for
   agent-id-dependent business logic. The extractor pattern makes
   the dependency explicit in the handler signature.
2. **Error-type newtype around `RepositoryError`.** Rejected — the
   repository layer returns concrete failure modes (`NotFound`,
   `Conflict`, `Backend(...)`); handlers want HTTP-aware mappings
   that sometimes differ per endpoint (a `NotFound` is 404 for
   `get_secret_by_slug` but 409 for `put_secret` on a pre-existing
   slug). Per-handler `map_err` is the right granularity.
3. **Async `AuthenticatedSession::from_cookies`.** Rejected —
   cookie verification is CPU-bound crypto; async would sprinkle
   `.await` at every extraction site for zero benefit.

## Implementation pointer

- Module: [`modules/crates/server/src/handler_support/`][1].
- First consumers: [`handlers::bootstrap`][8] (migrated in P3) and
  [`handlers::platform_secrets`][9] (M2/P4, five routes).
- Exhaustiveness test: [`server/tests/permission_check_mapping_test.rs`][4].
- Integration tests: [`server/tests/handler_support_test.rs`][7].

[1]: ../../../../../../modules/crates/server/src/handler_support/session.rs
[2]: ../../../../../../modules/crates/server/src/handler_support/permission.rs
[3]: ../../../../../../modules/crates/server/src/handler_support/audit.rs
[4]: ../../../../../../modules/crates/server/tests/permission_check_mapping_test.rs
[5]: ../../../../../../modules/crates/server/src/handler_support/errors.rs
[6]: ../../../../../../modules/web/lib/api/errors.ts
[7]: ../../../../../../modules/crates/server/tests/handler_support_test.rs
[8]: ../../../../../../modules/crates/server/src/handlers/bootstrap.rs
[9]: ../../../../../../modules/crates/server/src/handlers/platform_secrets.rs
