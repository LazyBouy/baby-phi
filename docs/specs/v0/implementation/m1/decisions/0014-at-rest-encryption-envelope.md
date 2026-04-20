<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0014: At-rest encryption — AES-GCM envelope, env-var-injected master key, secrets-vault scope only

## Status

Accepted — 2026-04-20 (M1 / P1). M7b adds KMS integration + full-DB
encryption + key rotation.

## Context

The v0.1 build plan commits to at-rest encryption as a production-readiness
row owned jointly by M1 (seed) and M7b (hardening verification). The
concept contract names **credentials-vault entries** as the
encryption-sensitive surface — full-DB encryption without proper key
management is theatre because the master key still lives on the same host.

We need to decide: which columns encrypt, what algorithm, how the key is
loaded, what the debug story is.

## Decision

1. **Scope: `secrets_vault.value_ciphertext_b64` + `nonce_b64` only for
   M1.** Every other column stays in plaintext at rest. Full-DB encryption
   is M7b work, gated on KMS integration.
2. **Algorithm: AES-256-GCM.** Industry standard AEAD; built-in
   authentication tag; available in the well-audited `aes-gcm` crate.
   Key size is 32 bytes.
3. **Master key injected via env var.** `BABY_PHI_MASTER_KEY` carries the
   base64-encoded 32-byte key. The server refuses to start if the vault is
   touched and the key isn't loadable — matches the migration runner's
   fail-safe posture.
4. **Fresh nonce per seal.** 12 random bytes from the OS RNG
   (`rand::rng().fill_bytes`). Stored next to the ciphertext (base64) so
   `open()` has everything it needs; never reused across messages.
5. **Persisted as base64 strings, not native bytes.** Same rationale as
   ADR-0013: the SurrealDB Rust driver's `Vec<u8>` binding doesn't coerce
   into a `bytes` column.
6. **`MasterKey: Debug` is redacted.** The `Debug` impl prints
   `MasterKey(***)` so `tracing::info!(?key, ...)` cannot leak it. Unit
   test pins the exact string.
7. **No hand-rolled key derivation.** The master key is the AES key
   directly — no HKDF, no PBKDF2. Simpler, matches the trust model (env
   var is the root of trust in v0.1).

## Consequences

Positive:

- **Secrets never reach the disk in plaintext.** Even if the DB file is
  copied off a compromised host, the vault column is opaque without
  `BABY_PHI_MASTER_KEY`.
- **Minimal dependency footprint.** `aes-gcm`, `base64`, and `rand` are
  well-audited, small crates.
- **Debug/log leakage is hard to hit accidentally.** The redacted `Debug`
  plus the no-derive-`Serialize` on `MasterKey` cover the obvious
  mistakes.

Negative:

- **Env-var root-of-trust is crude.** A compromised host process with
  `/proc/<pid>/environ` access gets the key. M7b + KMS fixes this.
- **No key rotation tooling in M1.** Rotating the master key means
  re-sealing every row with the new key. That tooling is M7b.
- **Ciphertext and nonce live next to each other.** Standard GCM
  practice; the nonce is not secret. Named explicitly so future reviewers
  don't assume otherwise.
- **Not every sensitive column is covered.** API keys inside `ModelConfig`
  rows (in M2+) will need similar treatment at that time.

## Alternatives considered

- **Scope: full-DB encryption.** Rejected for v0.1: requires KMS to be
  genuinely useful; full-DB encryption with an env-var key adds latency
  and no real security beyond vault-column encryption.
- **Algorithm: ChaCha20-Poly1305.** Acceptable equivalent; AES-GCM picked
  for ubiquity and hardware support. If we ever want hardware-friendly
  deployments on ARM, we revisit.
- **Per-secret derived keys via HKDF.** Rejected for v0.1: adds a KDF hop
  per seal for no security benefit at our scale. M7b adds per-secret
  wrapping with KMS.
- **Nonces stored as native SurrealDB `bytes`.** Rejected due to the
  driver coercion issue (see ADR-0013). Base64 strings are portable and
  work via `.bind(String)`.
- **A random "key version" column.** Rejected for v0.1: there is only
  ever one key. M7b adds versioning when rotation lands.

## References

- Implementation: [`modules/crates/store/src/crypto.rs`](../../../../../../modules/crates/store/src/crypto.rs)
- Schema: [`modules/crates/store/migrations/0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql) — `secrets_vault` table
- Architecture page: [at-rest-encryption.md](../architecture/at-rest-encryption.md)
- Plan row: "At-rest encryption" production-readiness row — M1 (seed) +
  M7b (KMS + full-DB + rotation).
