<!-- Last verified: 2026-04-21 by Claude Code -->

# User guide — model providers

**Status: [EXISTS]**

Day-to-day operator flows for page 02 (model providers). CLI +
web parity: the web UI is the default interactive surface, CLI
is authoritative for scripted provisioning.

## Concepts

- **ModelConfig** — phi-core's single provider binding shape
  (see [`phi-core/src/provider/model.rs`](../../../../../../../phi-core/src/provider/model.rs)).
  Baby-phi persists this struct **verbatim** inside `ModelRuntime.config`;
  the wire payload for `POST /model-providers` is exactly what
  phi-core's serde accepts.
- **Provider kind** — the `api` field on `ModelConfig`. Stable
  snake_case string (e.g. `anthropic_messages`, `openai_completions`).
  Enumerated by phi-core's `ProviderRegistry`.
- **secret_ref** — a vault slug pointing at the API key. API keys
  live exclusively in the Credentials Vault (page 04); registrations
  just reference them by slug.
- **tenants_allowed** — which orgs may invoke this runtime. `all`
  by default; scope to specific org UUIDs for M3+ multi-tenant
  deployments.

## Registering a provider

### Prereq: seed the API key

```bash
printf 'sk-ant-<your-key>' | phi secret add \
    --slug anthropic-api-key --material-file -
```

### CLI

```bash
# 1. Discover the kinds this build of phi-core supports.
phi model-provider list-kinds

# 2. Draft a ModelConfig as a JSON file. Any phi-core-accepted field
#    is valid; leave api_key unset.
cat > /tmp/claude-sonnet-4.json <<'JSON'
{
  "id": "claude-sonnet-4-20250514",
  "name": "Claude Sonnet 4",
  "api": "anthropic_messages",
  "provider": "anthropic",
  "base_url": "https://api.anthropic.com",
  "reasoning": false,
  "context_window": 200000,
  "max_tokens": 8192
}
JSON

# 3. Register.
phi model-provider add \
    --config-file /tmp/claude-sonnet-4.json \
    --secret-ref anthropic-api-key

# Defaults tenants_allowed to `all`. Narrow explicitly if you
# need per-org isolation:
phi model-provider add \
    --config-file /tmp/claude-sonnet-4.json \
    --secret-ref anthropic-api-key \
    --tenants-allowed 6f3a1c2d-...,8e12bb34-...
```

### Web

Navigate to **Model Providers** in the admin sidebar:

1. The sidebar panel shows the phi-core-supported kinds as pills.
2. Paste the `ModelConfig` JSON into the form.
3. Type the vault slug into `Vault secret_ref` (must already exist).
4. Click **Register provider**. The result banner shows the new
   provider UUID + audit event id.

## Listing

```bash
phi model-provider list
# provider_id                            provider            model_id                         status      archived
# <uuid>                                 anthropic           claude-sonnet-4-20250514         "ok"        -

phi model-provider list --include-archived
phi model-provider list --json  # pipe into jq
```

The web table shows the same rows — archived entries render at
reduced opacity with the archive button hidden.

## Archiving

```bash
phi model-provider archive --id <uuid>
```

Web: click **Archive** in the provider's row. The archive is
Alerted-audited; there is no undo — re-register under a new UUID
if you changed your mind.

## Exit codes (CLI)

Same contract as `secret`:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Transport / IO failure |
| 2 | Server rejected with a stable error code |
| 3 | Server 5xx or unexpected shape |
| 4 | Precondition failed (no saved session, missing file) |
| 5 | Reserved for M2/P6 cascade-aborted |

## Error codes

| Code | HTTP | Meaning |
|---|---|---|
| `UNAUTHENTICATED` | 401 | No valid session cookie. |
| `VALIDATION_FAILED` | 400 | `ModelConfig` missing required field / bad `secret_ref` slug shape. |
| `SECRET_REF_NOT_FOUND` | 400 | The named vault entry doesn't exist. |
| `MODEL_PROVIDER_DUPLICATE` | 409 | Same `(provider, config.id)` already registered and active. |
| `MODEL_PROVIDER_NOT_FOUND` | 404 | Archive/lookup on an unknown UUID. |

## References

- [Operations runbook](../operations/model-provider-operations.md)
- [phi-core reuse map](../architecture/phi-core-reuse-map.md)
- [User guide — secrets vault](secrets-vault-usage.md)
