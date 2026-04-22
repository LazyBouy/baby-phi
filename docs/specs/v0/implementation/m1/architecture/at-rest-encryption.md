<!-- Last verified: 2026-04-20 by Claude Code -->

# Architecture — at-rest encryption

M1 ships envelope encryption for the `secrets_vault` table — the only
surface the v0.1 concept contract names as encryption-sensitive. Full-DB
encryption is deferred to M7b (where KMS integration lands). Rationale is
in [ADR-0014](../decisions/0014-at-rest-encryption-envelope.md); this page
covers mechanics.

## Algorithm

- Cipher: **AES-256-GCM** (`aes_gcm::Aes256Gcm`).
- Key size: **32 bytes**, base64-encoded in the
  `PHI_MASTER_KEY` env var.
- Nonce: **12 bytes**, freshly sampled from the OS RNG (`rand::rng()`) per
  seal. Stored base64 next to the ciphertext.
- Authentication: GCM's built-in tag — tampering with either the ciphertext
  or nonce fails `open()` with `CryptoError::Open`.

## Key loading

[`MasterKey::from_env`](../../../../../../modules/crates/store/src/crypto.rs)
reads `PHI_MASTER_KEY`. Errors:

| Error | Cause |
|---|---|
| `CryptoError::MissingMasterKey` | env var unset |
| `CryptoError::BadBase64(...)` | value isn't valid base64 (no-padding) |
| `CryptoError::BadLength(n)` | decoded length ≠ 32 bytes |

The server refuses to start if the vault is touched and the key isn't
loadable — same fail-safe posture as the migration runner.

## Wire path

```
plaintext (bytes)
       │
       ▼
  seal(key, plaintext)
       │
       ▼
  SealedSecret {
       ciphertext: Vec<u8>,
       nonce:      [u8; 12],
  }
       │ .to_base64()
       ▼
  ("<ciphertext_b64>", "<nonce_b64>")
       │
       ▼
  CREATE secrets_vault SET
     value_ciphertext_b64 = $ct,
     nonce_b64            = $nonce,
     ...
```

`open()` is the inverse — `SealedSecret::from_base64` rebuilds from two
strings, then `open(&key, &sealed)` returns the plaintext.

## Why base64 rather than SurrealDB's native `bytes`

The SurrealDB 2.x Rust driver's `.bind(Vec<u8>)` JSON-serializes as an
array of numbers — which does not coerce into a `TYPE bytes` column; the
write fails schema validation. Two escape paths exist:

1. Wrap in `surrealdb::sql::Bytes` — works but ties the persistence layer
   to a SurrealDB-specific wrapper type.
2. Store as base64 strings — portable, human-debuggable, no driver-specific
   types.

M1 picks option 2. The `SealedSecret::to_base64` / `from_base64` helpers
keep the boundary code minimal. The same pattern is used for
`audit_events.prev_event_hash_b64`.

## The `Debug` redaction

`MasterKey: Debug` is `MasterKey(***)` — the raw bytes never appear in
`format!("{:?}", key)` output, so they can't leak into logs via
`tracing::info!(?key, ...)` or similar accidents. Unit test
`debug_impl_redacts_key` pins this.

## What M7b adds on top

- KMS integration (AWS KMS / GCP KMS / HashiCorp Vault) so the master key
  is never directly held by the process.
- Per-secret wrapped keys (envelope-of-envelope) for defence in depth.
- Master-key rotation runbook + tooling.
- Full-DB encryption (SurrealDB data files at rest), not just the vault
  column.

M1's decision is intentionally narrow: the vault column is the only
plaintext-sensitive surface in v0.1, and adding broad encryption without
KMS is theatre — the master key would still live in an env var on disk.

## Tests

| Test | File | Asserts |
|---|---|---|
| `seal_open_roundtrip_recovers_plaintext` | `crypto.rs` | Seal→open gives back the original bytes |
| `nonces_are_unique_per_seal` | `crypto.rs` | Fresh nonce every call (same plaintext → different ciphertext) |
| `tampered_ciphertext_fails_to_open` | `crypto.rs` | Flipping one ciphertext byte triggers `CryptoError::Open` |
| `wrong_key_fails_to_open` | `crypto.rs` | Cross-key decryption fails cleanly |
| `base64_round_trip_matches_raw_bytes` | `crypto.rs` | Base64 form parses back to the same 32-byte key |
| `rejects_short_key` | `crypto.rs` | 16-byte input → `CryptoError::BadLength(16)` |
| `rejects_bad_base64` | `crypto.rs` | Non-base64 → `CryptoError::BadBase64` |
| `missing_env_var_reports_missing_master_key` | `crypto.rs` | Unset var → `CryptoError::MissingMasterKey` |
| `debug_impl_redacts_key` | `crypto.rs` | `format!("{:?}", key) == "MasterKey(***)"` |
| `seal_persist_read_open_roundtrip` | `tests/crypto_vault_test.rs` | End-to-end against real SurrealDB: seal → CREATE → SELECT → open |

## Concept references

- Build plan: "At-rest encryption" production-readiness row — M1 (seed) +
  M7b (full).
- ADR: [0014 At-rest encryption envelope](../decisions/0014-at-rest-encryption-envelope.md).
