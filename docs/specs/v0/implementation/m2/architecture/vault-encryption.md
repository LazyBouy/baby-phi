<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — credentials vault (page 04)

**Status: [EXISTS]**

How page 04 (credentials vault) is built on top of the M1 crypto
primitives, the M2 handler_support shim, and the Template E pattern.
This page is the durable architectural reference; operational details
live in [`../operations/secrets-vault-operations.md`](../operations/secrets-vault-operations.md)
and the day-to-day usage flow in [`../user-guide/secrets-vault-usage.md`](../user-guide/secrets-vault-usage.md).

## Layered view

```
                      ┌───────────────────────┐
                      │  baby-phi secret …    │
                      │  web /secrets         │  (user surfaces)
                      └──────────┬────────────┘
                                 │ HTTP + cookie
                                 ▼
                ┌───────────────────────────────┐
                │ handlers/platform_secrets.rs  │
                │  — parses body                │
                │  — decodes base64 material    │
                │  — maps SecretError → ApiError│
                └──────────┬────────────────────┘
                           │
                           ▼
           ┌────────────────────────────────────┐
           │ platform/secrets/{add,rotate,…}.rs │
           │  — Template E AuthRequest build    │
           │  — check_permission (reveal only)  │
           │  — audit event emission            │
           └──────────┬─────────────────────────┘
                      │
    ┌────────────────┬┴──────────────────┬───────────────┐
    ▼                ▼                   ▼               ▼
┌─────────┐   ┌─────────────┐   ┌──────────────┐   ┌──────────┐
│ domain  │   │ store::    │   │ domain::    │   │ audit    │
│ ::     │   │ crypto::   │   │ permissions │   │ emitter  │
│ Repo-   │   │ seal/open  │   │ ::engine    │   │          │
│ sitory  │   │            │   │             │   │          │
└─────────┘   └─────────────┘   └──────────────┘   └──────────┘
```

All five operations (add / rotate / reveal / reassign / list) are
composed from this boxset. Reveal is the only path that crosses into
the Permission Check engine; every other write is a self-approved
Template E operation.

## The `SecretCredential` composite

[`domain::model::composites_m2::SecretCredential`][1] is the
domain-layer catalogue entry. Fields:

```
SecretCredential {
    id: SecretId,            // UUID, stable across rotations
    slug: SecretRef,         // human-readable; referential integrity anchor
    custodian: AgentId,      // current governance owner
    last_rotated_at: Option<DateTime<Utc>>,
    sensitive: bool,         // mask in list/audit views
    created_at: DateTime<Utc>,
}
```

The sealed material (`value_ciphertext_b64` + `nonce_b64`) lives
side-by-side in the `secrets_vault` SurrealDB row but is NOT carried
on the domain struct — the domain layer never holds plaintext.

## Seal / unseal envelope

One master key platform-wide; per-entry fresh nonce. Detailed rationale
in [ADR-0017](../decisions/0017-vault-encryption-envelope.md).

```
seal(master_key, plaintext)
  → SealedSecret { ciphertext: Vec<u8>, nonce: [u8; 12] }
  .to_base64() → (ciphertext_b64, nonce_b64)   // persist these two

open(master_key, sealed)
  ← SealedSecret::from_base64(ct_b64, nonce_b64)
  ← `value_ciphertext_b64`, `nonce_b64` read off the row
```

The `SealedBlob { ciphertext_b64, nonce_b64 }` pair is the
domain-layer projection ([`domain::repository::SealedBlob`][2]) that
`Repository::put_secret` / `get_secret_by_slug` / `rotate_secret`
exchange.

## Template E write flow (add)

[`server::platform::secrets::add::add_secret`][3]:

1. Validate slug shape (lowercase / digits / dashes; ≤ 64 chars).
2. Uniqueness check via `get_secret_by_slug`.
3. `seal(master_key, plaintext)` → `(ct_b64, nonce_b64)`.
4. Build a Template E Auth Request via
   [`domain::templates::e::build_auto_approved_request`][4] — the
   platform admin is both requestor and approver; the AR enters
   `Approved` state at construction.
5. Persist in sequence:
   - `create_auth_request(ar)`
   - `put_secret(credential, sealed_blob)`
   - `seed_catalogue_entry(None, "secret:<slug>", "secret_credential")`
   - `create_grant(…)` — a per-instance `[read]` grant with
     `resource.uri = "secret:<slug>"` and
     `fundamentals = [Fundamental::SecretCredential]`. The
     explicit `fundamentals` field is what lets the engine's
     `resolve_grant` Case D pick up the grant at Step 2
     despite the URI being an instance reference rather than a
     class name (see M2/P4.5 — G19 / D17).
6. Emit `vault.secret.added` via the
   [`SurrealAuditEmitter`][5] (Alerted class).

Sequential writes (vs. one atomic batch) are acceptable for M2 —
there's a single platform admin, so no concurrent writers trip the
TOCTOU window. M3 introduces an atomic "AR + write + catalogue +
grant" batch API that replaces this sequence.

## Reveal flow — engine-gated

[`server::platform::secrets::reveal::reveal_secret`][6] is the
one M2/P4 path that calls the Permission Check engine directly.
Why not the shared [`handler_support::check_permission`][7] helper?
Because the denial path needs the full `Decision` in hand so it can
emit `vault.secret.reveal_attempt_denied` with the failed step
before converting to `ApiError`.

The manifest the reveal handler synthesises:

```
Manifest {
    actions: ["read"],
    resource: ["secret_credential"],           // fundamental class name
    constraints: ["purpose"],
    constraint_requirements: { "purpose" → "reveal" },
    kinds: ["#kind:secret_credential"],
    ...
}
```

Paired with the `ToolCall`:

```
ToolCall {
    target_uri: "secret:<slug>",               // catalogue-checked
    target_tags: ["#kind:secret_credential", "secret:<slug>"],
    constraint_context: { "purpose" → "reveal" },
    target_agent: None,
}
```

This means:

- Step 0 Catalogue — `secret:<slug>` must be in the platform
  catalogue (seeded at add time).
- Step 1 Expansion — `"secret_credential"` is a
  [`Fundamental`][8] name, expands to
  `{Fundamental::SecretCredential}`.
- Step 2 Resolution — the admin's per-instance `[read]` grant
  on `secret:<slug>` is picked up. Case D in `resolve_grant`
  uses the grant's explicit `fundamentals` field so the
  instance URI doesn't block matching.
- Step 3 Match — the grant's parsed selector matches the
  exact `secret:<slug>` target; the explicit
  `SecretCredential` fundamental matches the manifest reach.
- Step 4 Constraints — the manifest requires
  `purpose=reveal`; the `ToolCall` provides exactly that, so the
  constraint passes. A hypothetical invocation with a different
  purpose value (only possible today by bypassing the reveal handler)
  would fail here.
- Step 5 Scope — only one grant; trivially the winner.
- Step 6 Consent — no template gating on platform-admin grants.
- **Allowed** → handler emits `vault.secret.revealed` (Alerted),
  then unseals and returns the plaintext.

## Audit events

Seven builders in [`domain::audit::events::m2::secrets`][9]:

| Event | Class | Trigger |
|---|---|---|
| `vault.secret.added` | Alerted | add handler |
| `vault.secret.rotated` | Alerted | rotate handler |
| `vault.secret.archived` | Alerted | (reserved — M3+ archive flow) |
| `vault.secret.custody_reassigned` | Alerted | reassign handler |
| `vault.secret.revealed` | Alerted | reveal handler (allowed path) |
| `vault.secret.reveal_attempt_denied` | Alerted | reveal handler (engine denial) |
| `vault.secret.list_read` | Logged | list handler |

Emitting **before** plaintext is streamed back (reveal path) is the
contract that guarantees a crash between emit + send still leaves an
audit trail.

## Audit chain scope

Vault events chain under `org_scope = None` (the platform-root chain).
When M3+ introduces per-org vault partitions, the emitter will pass
the owning org on each event — no downstream change on the chain
protocol.

## Data-model shape (persistence)

`secrets_vault` table, populated at migration 0001:

```sql
DEFINE FIELD slug                ON secrets_vault TYPE string;
DEFINE FIELD custodian_id        ON secrets_vault TYPE string;
DEFINE FIELD value_ciphertext_b64 ON secrets_vault TYPE string;
DEFINE FIELD nonce_b64           ON secrets_vault TYPE string;
DEFINE FIELD sensitive           ON secrets_vault TYPE bool  DEFAULT false;
DEFINE FIELD last_rotated_at     ON secrets_vault TYPE option<datetime>;
DEFINE FIELD created_at          ON secrets_vault TYPE datetime;
DEFINE INDEX secrets_vault_slug_unique ON secrets_vault FIELDS slug UNIQUE;
```

## References

- [ADR-0017 — vault encryption envelope](../decisions/0017-vault-encryption-envelope.md)
- [Template E — self-interested auto-approve](template-e-auto-approve.md)
- [handler_support — auth + permission + audit shim](handler-support.md)
- [platform-catalogue — how vault entries resolve at Step 0](platform-catalogue.md)
- [Operations runbook](../operations/secrets-vault-operations.md)
- [User guide](../user-guide/secrets-vault-usage.md)

[1]: ../../../../../../modules/crates/domain/src/model/composites_m2.rs
[2]: ../../../../../../modules/crates/domain/src/repository.rs
[3]: ../../../../../../modules/crates/server/src/platform/secrets/add.rs
[4]: ../../../../../../modules/crates/domain/src/templates/e.rs
[5]: ../../../../../../modules/crates/store/src/audit_emitter.rs
[6]: ../../../../../../modules/crates/server/src/platform/secrets/reveal.rs
[7]: ../../../../../../modules/crates/server/src/handler_support/permission.rs
[8]: ../../../../../../modules/crates/domain/src/model/fundamentals.rs
[9]: ../../../../../../modules/crates/domain/src/audit/events/m2/secrets.rs
