<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0011: Bootstrap credential — argon2id-hashed, stdout-delivered, single-use

## Status

Accepted — 2026-04-20 (M1 / P5).

## Context

The System Bootstrap Template adoption is the one flow in the platform
that cannot be gated by a prior grant — there's no agent yet to hold
one. The concept doc
([`permissions/02` §System Bootstrap Template](../../../concepts/permissions/02-auth-request.md))
treats the template as a hardcoded axiom; the requirements
([`requirements/admin/01`](../../../requirements/admin/01-platform-bootstrap-claim.md))
specify that the install emits a **bootstrap credential** the first
human presents in exchange for `[allocate]` on `system:root`.

Three shape decisions for the credential are open in the plan:

1. **Storage form.** Plaintext, symmetrically encrypted, or hashed? If
   hashed, which algorithm?
2. **Delivery.** Where does the plaintext live between install-time and
   claim-time? A file on disk? An env var? Stdout-only?
3. **Lifecycle.** Can it be revoked? Reissued? Multi-use?

## Decision

### Storage — argon2id hash, PHC-encoded, in the row's `digest` column.

The `bootstrap_credentials.digest` column holds a PHC-format argon2id
hash. Each row has its own salt (argon2id embeds the salt in the PHC
string); verification runs `argon2::PasswordVerifier::verify_password`
which is constant-time.

We chose argon2id over:

- **SHA-256 / SHA-512.** Rejected. Not designed for password hashing —
  a stolen database lets an attacker brute-force the credential offline
  with commodity hardware.
- **bcrypt.** Acceptable but older; argon2id is the OWASP 2024
  recommendation for new code and is already workspace-standard.
- **scrypt.** Also acceptable; chose argon2id for ecosystem momentum.

### Delivery — stdout, **once**, on `phi-server bootstrap-init`.

The install process runs `phi-server bootstrap-init` once. That
command generates 32 CSPRNG bytes, base64url-encodes them, prefixes
`bphi-bootstrap-`, stores the argon2id hash, and prints the plaintext
to stdout surrounded by a one-time banner.

We chose stdout-only over:

- **A file on disk.** Rejected. The plaintext would sit on the install
  host until somebody remembered to delete it — a window of exposure
  with no operational value. 12-factor strongly discourages
  file-resident secrets.
- **An env var the server itself reads.** Rejected. The credential is
  meant to be typed by a human into the `/bootstrap` page, not
  consumed by the server. Putting it in `PHI_BOOTSTRAP_CRED` would
  mean the running server has access to a secret it never uses.
- **Log output.** Rejected. Structured logs get shipped to log
  aggregators (the whole point of structured logging); sending the
  credential there is a leak.

Stdout has the right property: it's ephemeral, it goes to the admin's
terminal, and it's the admin's job to capture it (copy into a password
manager, paste into `/bootstrap`). If the admin loses the stdout
output, the credential is gone. The only recovery path is reinstall
or a manual admin override — both out of scope for M1 per the
requirement doc.

### Lifecycle — single-use, no expiry, no reissue.

The credential is consumed exactly once. Per
[R-ADMIN-01-W3](../../../requirements/admin/01-platform-bootstrap-claim.md#6-write-requirements):

> The bootstrap credential SHALL be **single-use**. Once consumed, it
> cannot be reused even if the claim is later rolled back (recovery
> requires a new install or a manual admin override — both out of
> scope for this page).

M1 does **not** ship:

- **Credential expiry.** The schema has no `expires_at` column. The
  concept doc notes "expired" as a failure mode, but in v0.1 we treat
  expiry as "the credential predates a new install" — a reinstall
  generates a fresh credential, which simultaneously retires any prior
  credentials (they stay in the table but no claim handler will ever
  match them on a fresh data dir).
- **Reissue.** Generating a second credential and invalidating the
  first is out of scope for M1. The plan defers it to "manual admin
  override" and marks it for M2+ if the need arises in practice.
- **Multi-credential installs.** Every install produces one
  credential. Future installs overwrite the data dir; reusing a data
  dir across installs is undefined.

### Lookup by verify-per-row (not by exact hash).

Each argon2id hash embeds its own salt, so the digest column doesn't
support exact-equality lookup. The claim handler
(`execute_claim` in
[`server/src/bootstrap/claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs))
scans unconsumed credentials and runs `verify_credential` against each.
For M1 (1 credential per install) that's O(1) wall time. If future
M-levels introduce multi-tenant installs with dozens of credentials we
can either (a) add a short non-sensitive HMAC prefix alongside the PHC
hash to narrow the scan, or (b) move to bcrypt with a static salt if
exact-match lookup becomes load-bearing. Neither is needed for M1.

### Atomic write via a dedicated `apply_bootstrap_claim` repo method.

R-SYS-s01-6 requires the seven writes of the adoption flow be atomic.
Rather than thread a transaction handle through `Repository` (which
would force every impl to model transactions), we added a single
batch-oriented method
[`Repository::apply_bootstrap_claim`](../../../../../../modules/crates/domain/src/repository.rs)
that takes a full `BootstrapClaim` payload and commits every side-effect
in one go. SurrealStore wraps the batch in `BEGIN TRANSACTION … COMMIT
TRANSACTION`; the in-memory fake wraps it in its single write-lock.
Both back-ends share the rollback invariant: on any inner-query error,
no partial state survives. See
[architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md)
§Atomicity.

## Consequences

Positive:

- **Plaintext is never stored.** Even a full data-dir snapshot can't
  replay the claim — an attacker would need to brute-force a 256-bit
  random through argon2id, which is infeasible.
- **One-shot delivery matches operator expectation.** An SSH host key
  is the cultural reference: you see it once, you save it, you move
  on. The error path ("I lost it, now what?") is acceptably "reinstall
  on a fresh data dir," since the platform hasn't done anything yet.
- **Atomicity is localised.** The seven writes live in one repo method
  with one transaction. No `Drop`-based rollback, no compensating
  delete paths, no "partial claim" state to clean up afterward.
- **Single-use is schema-enforced.** `consumed_at IS NONE` on the
  lookup + stamping `consumed_at` inside the transaction gives a
  hard-to-misuse contract. Per D4 and the store integration
  rollback test, a failed claim leaves `consumed_at` untouched.

Negative:

- **No credential rotation in M1.** If the admin suspects the
  plaintext leaked (e.g. they accidentally committed their terminal
  scroll-back), there's no in-band rotation path. The admin has to
  reinstall. M2+ can add a "rotate bootstrap credential" admin flow
  behind the existing `[allocate]` grant if the need surfaces.
- **The verify-per-row scan is O(n).** With 1 credential per install
  this is O(1) in practice, but it's a latent cost multiplier if a
  future flow creates many credentials. We document this rather than
  optimise; the data point we'd need is "how many credentials do
  real installs have in practice?"
- **Stdout-only delivery is awkward in some orchestration contexts.**
  A fully-automated install pipeline can't easily capture stdout from
  a non-interactive `phi-server bootstrap-init` run. The M1
  answer is "run the command as part of install scripting and capture
  the output line matching `bphi-bootstrap-*`." We accept this
  operational wart rather than add an "emit to file" flag that would
  encourage leaving plaintext on disk.

## Alternatives considered

- **HMAC-SHA256 keyed by a long-lived server secret, stored alongside
  a non-secret per-credential salt.** Cleaner lookup (exact match on
  `HMAC(salt ‖ credential)`); trades the argon2id memory-hardness
  for faster brute-force if the server secret leaks. Rejected for M1
  — memory-hard is the right default when you have single-digit
  credentials.
- **Long-lived credential (no single-use).** Rejected. Violates
  R-ADMIN-01-W3 and expands blast radius to "anyone who ever saw the
  credential." Single-use is the whole point.
- **Separate "admin kit" install command that bundles the credential
  with a generated TLS cert + config file.** Over-engineered for M1
  — the plan defers the kit idea to M7b production hardening; M1's
  admin is expected to copy/paste.

## References

- Implementation:
  [`server/src/bootstrap/init.rs`](../../../../../../modules/crates/server/src/bootstrap/init.rs)
  (generation);
  [`server/src/bootstrap/credential.rs`](../../../../../../modules/crates/server/src/bootstrap/credential.rs)
  (hash + verify);
  [`server/src/bootstrap/claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs)
  (claim business logic);
  [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs)
  (`BootstrapClaim` struct + `apply_bootstrap_claim` method);
  [`store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs)
  (`BEGIN/COMMIT TRANSACTION` envelope).
- Architecture page:
  [bootstrap-flow.md](../architecture/bootstrap-flow.md).
- Tests: 13 server unit tests + 3 store integration tests (16 total).
- Requirements:
  [`admin/01`](../../../requirements/admin/01-platform-bootstrap-claim.md) +
  [`system/s01`](../../../requirements/system/s01-bootstrap-template-adoption.md).
- Plan: [015a217a-m1-permission-check-spine.md §P5](../../../../plan/build/015a217a-m1-permission-check-spine.md).
