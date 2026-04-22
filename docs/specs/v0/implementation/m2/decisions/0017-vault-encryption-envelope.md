<!-- Last verified: 2026-04-21 by Claude Code -->

# ADR-0017 — Vault encryption envelope (M2 page 04)

**Status: [EXISTS]** — accepted and implemented in M2/P4.

## Context

Page 04 persists the platform's API-key material + adjacent secrets.
M1 shipped the AES-GCM seal / unseal primitives (ADR-0014) but no
handler exercised them. M2/P4 wires them end-to-end via the page-04
handlers and the CLI + web surfaces.

Requirements driving the envelope design:

1. **Plaintext never appears on disk** in any form (no DB plaintext,
   no shadow log, no audit diff).
2. **Rotation must be cheap** — a single write flips the material;
   old nonces are discarded.
3. **Reveal must be auditable** — even if the storage layer is
   compromised and ciphertext leaked, a reviewer can enumerate every
   historical reveal.
4. **Fits the single-admin M2 model** — no need for multi-tenant
   key isolation yet.

## Decision

Per-entry seal-with-fresh-nonce + **one** platform-wide master key.

- Master key: 32 bytes, AES-256-GCM. Loaded from
  `PHI_MASTER_KEY` at server start. Deliberately `!Serialize`
  (see [`store::crypto::MasterKey`][1]) so it can't accidentally land
  in logs or JSON payloads.
- Per-entry envelope: `(ciphertext, nonce)` — both base64-encoded,
  stored side-by-side on `secrets_vault` row columns
  `value_ciphertext_b64` + `nonce_b64`. Fresh 12-byte nonce sampled
  from the OS RNG at every seal.
- Rotation: re-seal with a new nonce; overwrite the two columns; bump
  `last_rotated_at`. One database round-trip.
- Reveal: read the columns, call `open(master_key, sealed)`, return
  the plaintext on the HTTP response. Plaintext is held in a `Vec<u8>`
  that's dropped after the response body is written.

**NOT** in M2:

- **Per-entry DEK wrapping.** A full KMS envelope (per-row DEKs,
  wrapped under a KEK, rotated independently) is M7b. The one-key
  design is cryptographically sound (AES-256-GCM with a unique nonce
  per plaintext is safe for ≥ 2⁶⁴ entries); the DEK layer is about
  operational isolation, not crypto strength.
- **Shadow NDJSON log.** The hash-chain on audit events provides
  tamper evidence. A mirrored NDJSON stream is M7b (cheap
  recoverability + off-box archival).
- **Master-key rotation.** The escape hatch is the parallel-stand-up
  flow documented in [`../operations/secrets-vault-operations.md`](../operations/secrets-vault-operations.md).

## Consequences

- Losing the master key loses every sealed secret. Operationally this
  means the master key must be backed up before any seal is performed.
- Tampering with a single row corrupts exactly that row's material —
  blast radius matches the key scope. Good.
- Rotating the master key requires re-sealing every row. For M2's
  scale (~tens of secrets) this is a minutes-long operator task.
- The single-key design is a known-limited surface that M7b upgrades
  without changing the `SealedBlob` wire format.

## Alternatives considered

1. **Per-entry DEK wrapped by a KEK.** Rejected for M2 — adds a DEK
   table, KEK lookup, and a rotation protocol. Value is operational
   (independent per-row rotation) rather than cryptographic. M7b.
2. **Envelope with associated data (the slug).** Considered binding
   AES-GCM's `AAD` to the slug so a row cannot be "moved" to another
   slug position. Value is small (an attacker who can reorder rows
   can also read them). Deferred; revisit when multi-tenant isolation
   lands.
3. **No envelope — just `pgcrypto` / SurrealDB built-in field
   encryption.** Rejected. SurrealDB ships no field-level crypto in
   2.6, and relying on DB-engine features makes the crypto boundary
   unsurveyable from the audit layer.

## Implementation pointer

- Envelope: [`store::crypto::seal` + `open`][1].
- Seal-at-write path: [`server::platform::secrets::add::add_secret`][2].
- Seal-at-rotate path: [`server::platform::secrets::rotate::rotate_secret`][3].
- Unseal-at-reveal path: [`server::platform::secrets::reveal::reveal_secret`][4].
- Proptest covering seal/open identity + tampering: [`store/tests/vault_roundtrip_props.rs`][5].

[1]: ../../../../../../modules/crates/store/src/crypto.rs
[2]: ../../../../../../modules/crates/server/src/platform/secrets/add.rs
[3]: ../../../../../../modules/crates/server/src/platform/secrets/rotate.rs
[4]: ../../../../../../modules/crates/server/src/platform/secrets/reveal.rs
[5]: ../../../../../../modules/crates/store/tests/vault_roundtrip_props.rs
