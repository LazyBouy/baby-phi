<!-- Last verified: 2026-04-21 by Claude Code -->

# Operations — credentials vault

**Status: [EXISTS]**

Operational runbook for page 04 (credentials vault). Covers the master
key, rotation policy, reveal audit trail, custody reassignment, and the
"lost custodian" recovery path.

## Master key

The vault's AES-256-GCM master key lives in the
`BABY_PHI_MASTER_KEY` environment variable as a 32-byte, standard-alphabet
base64 string (no padding). The server refuses to start if the env var
is missing or malformed — see [`store::crypto::MasterKey::from_env`][1].

Generation (one-off, deploy-time):

```bash
openssl rand -base64 32 | tr -d '=' > master-key.b64
export BABY_PHI_MASTER_KEY="$(cat master-key.b64)"
```

Storage guidance:

- Development: `.env` file with mode `0600`.
- Staging/prod: a KMS-backed secrets manager (AWS Secrets Manager, GCP
  Secret Manager, HashiCorp Vault). The env-var interface is a
  deployment-friendly shim; the upgrade to full KMS lands in M7b.

Never commit the master key to the repo. Never print it in logs — the
[`MasterKey`][1] struct is deliberately `!Serialize` and its `Debug`
impl masks the bytes.

## Rotation policy

Rotation replaces the sealed material on an existing row and bumps
`last_rotated_at`. The slug + custodian are unchanged. Rotation is
triggered by:

- **Operator intent** — `baby-phi secret rotate --slug <…>
  --material-file <…>` or the web UI equivalent (plan §P4).
- **Incident response** — whenever a plaintext may have been exposed
  (shared pair-programming session, ticket with a leaked value, lost
  laptop, etc.), rotate within 1 hour.
- **Scheduled rotation** — annually at minimum; more often for
  sensitive production secrets. Automated schedule lands in M7b
  (blocked on background job infra).

Every rotation emits one `vault.secret.rotated` event (Alerted class).
The event diff captures the previous `last_rotated_at` and the new
one; plaintext is never in the diff.

## Reveal audit trail

Every reveal — success or denial — writes exactly one Alerted event:

| Scenario | Event type |
|---|---|
| Permission Check allowed + plaintext returned | `vault.secret.revealed` |
| Permission Check denied | `vault.secret.reveal_attempt_denied` |

The denial event records `failed_step` (`Constraint` / `Resolution`
/ etc.) and the triggering reason. Operators investigating a denied
reveal pull the event by `audit_event_id` and trace the engine's
decision.

The Alerted class means the delivery channel wired at org-create time
receives the event within 60 s (R-NFR-OBS-2). Recipients include the
platform admin and any configured security team distribution list —
the exact recipients depend on the org's alert-channel config (page 05).

## Custody reassignment

`reassign_custody` changes the `custodian` field on a vault row. The
sealed material is untouched — custody is a governance concern, not a
crypto one. Use when:

- The original custodian is leaving the team.
- Custody needs to move to an operations-team rotation (on-call).
- A security audit recommends narrowing the set of custodians.

Emits `vault.secret.custody_reassigned` with both old and new
custodian ids in the diff — a reviewer can reconstruct the delegation
chain without a second query.

## "I lost my custodian" recovery

**Scenario:** The sole custodian agent has left or lost access, and no
other agent holds a grant that lets them rotate the secret.

**Recovery steps:**

1. Platform admin uses Template E (via `POST
   /api/v0/platform/secrets/:slug/reassign-custody`) to re-target
   custody to a currently-reachable agent. This is a self-approved
   write — the admin does not need the former custodian's approval.
2. Verify the reassignment via the list endpoint (the
   `custodian_id` field reflects the new owner).
3. If the new custodian also needs to rotate the material, run
   `baby-phi secret rotate` next — two sequential audit events will
   land (`vault.secret.custody_reassigned` then
   `vault.secret.rotated`).

For secrets with no remaining live custodian AND no platform admin
reachable (a catastrophic state): the only recovery is the
bootstrap-init path (re-seed the platform admin via a fresh bootstrap
credential) followed by custody reassignment. This is an incident
requiring an explicit runbook escalation — the M7b full runbook
covers it.

## Master-key rotation (stub)

Full master-key rotation is M7b. For M2 the escape hatch is:

1. Generate the new key.
2. Stand up a parallel server instance against a fresh data-dir, seal
   every secret under the new key.
3. Cut over traffic.

This is acceptable for M2's scale (one platform admin, tens of
secrets). M7b adds in-place rotation via per-entry DEKs + a dual-key
transition window.

## Troubleshooting

| Symptom | Likely cause | Recovery |
|---|---|---|
| Server refuses to start with `master key missing` | `BABY_PHI_MASTER_KEY` unset | Source `.env` / export the key / check the systemd unit env |
| `VAULT_CRYPTO_FAILED` on reveal | Ciphertext tampered, wrong key, or DB corruption | Check the key matches what was used at seal time; run the integrity-sweep tool (M7b) |
| `SECRET_NOT_FOUND` | Slug typo or the row was never added | Verify via `baby-phi secret list`; slug is case-sensitive, lowercase-only |
| `SECRET_SLUG_IN_USE` on add | The slug already exists | Choose a new slug or rotate the existing entry |
| `CONSTRAINT_VIOLATION` on reveal | Engine's `purpose=reveal` constraint denied | Normal reveal-flow always asserts this — a denial here indicates a misconfigured grant; check the `descends_from` chain |

## References

- Vault encryption envelope ADR — [`../decisions/0017-vault-encryption-envelope.md`](../decisions/0017-vault-encryption-envelope.md)
- Vault-encryption architecture — [`../architecture/vault-encryption.md`](../architecture/vault-encryption.md)
- Template E pattern — [`../architecture/template-e-auto-approve.md`](../architecture/template-e-auto-approve.md)

[1]: ../../../../../../modules/crates/store/src/crypto.rs
