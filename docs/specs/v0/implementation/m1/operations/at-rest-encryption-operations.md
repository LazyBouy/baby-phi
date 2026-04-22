<!-- Last verified: 2026-04-20 by Claude Code -->

# At-rest encryption — operations

M1 ships AES-256-GCM envelope encryption for the `secrets_vault`
table. The design lives in
[architecture/at-rest-encryption.md](../architecture/at-rest-encryption.md)
and its rationale in
[ADR-0014](../decisions/0014-at-rest-encryption-envelope.md). This
page covers the operator workflow.

## Scope in M1

Only the `secrets_vault.value_ciphertext_b64` column is encrypted.
Every other field of every other table is plaintext at-rest. This is
a deliberate decision — the concept contract names the vault as the
encryption-sensitive surface; broader-scope encryption without a KMS
is theatre. Full-DB encryption is scheduled for M7b.

## The master key

The server reads a single master key from the
`PHI_MASTER_KEY` environment variable. It must be a 32-byte
value (256 bits), base64-encoded without padding. Generate one with:

```bash
head -c 32 /dev/urandom | base64 -w0 | tr '+/' '-_' | tr -d '='
```

The key is loaded once at startup. The server refuses to start if:

- `PHI_MASTER_KEY` is unset **and** any code path touches the
  `secrets_vault` table, OR
- The decoded key is not exactly 32 bytes.

`[EXISTS]` in M1: **the key-missing guard only fires lazily**, when
the first `secrets_vault` operation runs. A cleaner M7b refactor
will move it to an eager startup check.

## Rotation in M1 — **not implemented**

M1 does not ship key rotation. If the current master key leaks or
needs to be rotated, the operator's options are:

1. **Fresh install.** Generate a new key, restore any non-encrypted
   data from backup, and accept that the vault contents must be
   re-entered (there's no vault-seeded data in M1 anyway — the
   first consumer lands in M3 with the OAuth client-secret flow).
2. **Manual re-encryption** (out of band). Dump the vault rows,
   decrypt with the old key, re-encrypt with the new key, upsert
   back. This is an admin-only recovery path; M1 doesn't ship
   tooling for it.

Planned for M7b: a dedicated `phi admin rotate-master-key
--old <key> --new <key>` subcommand that walks every encrypted row
in a single transaction.

## Envelope format

Each encrypted value is stored as two base64 columns:

| Column | Content |
|---|---|
| `value_ciphertext_b64` | AES-256-GCM ciphertext + 16-byte auth tag |
| `nonce_b64` | 96-bit (12-byte) nonce, randomly generated per write |

The plaintext is never logged, never written anywhere else, and never
returned by `GET` endpoints without an explicit decrypt step (which
is guarded by a grant chain that traces to the platform admin's
`[allocate]`-on-`system:root`).

## Backups

**Copy the master key separately from the data directory.** A
backup of the data dir alone is useless without the key — that's
the point. A backup of the key alone is useless without the data —
also the point.

Recommended operator flow:

- `data_dir` → standard disk snapshot, shipped to object storage
  nightly.
- `PHI_MASTER_KEY` → operator password manager + a
  break-glass hard copy in a physical safe. **Never** commit it
  to Git, never put it in a Dockerfile, never write it to an env
  file on disk.

Formal backup/restore tooling is an M7b production-hardening item.

## What M1 does NOT ship (deferred to M7b)

- KMS integration (AWS KMS, GCP Cloud KMS, HashiCorp Vault).
- Key derivation per tenant (multi-tenant installs share one
  master key in M1 because there's one org in M1).
- Rotation tooling.
- Per-row expiry + re-encryption schedule.
- Backup/restore runbook.

## Cross-references

- [architecture/at-rest-encryption.md](../architecture/at-rest-encryption.md)
  — the envelope-encryption design.
- [ADR-0014](../decisions/0014-at-rest-encryption-envelope.md) —
  rationale for AES-256-GCM + envelope.
- [modules/crates/store/src/crypto.rs](../../../../../../modules/crates/store/src/crypto.rs)
  — `MasterKey`, `seal`, `unseal`.
- [requirements/cross-cutting/nfr-security.md](../../../requirements/cross-cutting/nfr-security.md)
  — the source of the at-rest encryption commitment.
