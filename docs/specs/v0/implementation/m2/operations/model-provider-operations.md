<!-- Last verified: 2026-04-21 by Claude Code -->

# Operations — model providers

**Status: [EXISTS]**

Operational runbook for page 02 (model providers). Covers
provider registration, the phi-core-driven kind enumeration,
health-probe incidents (shape-only in M2, real probe in M7b),
and archival.

## What a "model provider" is

Each row in the `model_runtime` table is a
[`domain::model::composites_m2::ModelRuntime`][1] record that wraps
a [`phi_core::provider::model::ModelConfig`][2] struct plus
platform-governance metadata (`secret_ref`, `tenants_allowed`,
`status`, timestamps).

**Why wrap rather than redefine?** phi-core owns the canonical
shape of a model-runtime binding (API protocol, base URL, cost
config, per-provider compat flags). baby-phi adds the governance
envelope but **never** re-implements the binding — when phi-core's
`ModelConfig` gains a field (e.g. a new compat flag for a novel
OpenAI-compat provider), baby-phi picks it up at the next
`cargo update` without any handler change.

## Provider kind enumeration

`GET /api/v0/platform/provider-kinds` returns whatever phi-core's
[`ProviderRegistry::default().protocols()`][3] reports. Operators
should check this before crafting a `ModelConfig` to confirm the
`api` field value they intend to use is supported.

CLI:

```bash
baby-phi model-provider list-kinds
# provider kinds supported by this phi-core build:
#   - "anthropic_messages"
#   - "openai_completions"
#   - "openai_responses"
#   - "google_generative_ai"
#   - "google_vertex"
#   - "azure_openai_responses"
#   - "bedrock_converse_stream"
```

The list grows as phi-core adds providers; nothing in baby-phi
pins a subset.

## Registration flow

Preconditions:

1. The vault entry the runtime will reference must exist. Register
   the secret first:
   ```bash
   printf 'sk-ant-<your-key>' | baby-phi secret add \
       --slug anthropic-api-key --material-file -
   ```
2. Decide which orgs may invoke the runtime. Default `all`; narrow
   to `--tenants-allowed uuid-a,uuid-b` to pin to specific orgs.

Registration:

```bash
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
baby-phi model-provider add \
    --config-file /tmp/claude-sonnet-4.json \
    --secret-ref anthropic-api-key
# model provider registered
#   provider_id:     <uuid>
#   auth_request_id: <uuid>
#   audit_event_id:  <uuid>
```

Any field phi-core's `ModelConfig` accepts is valid in the JSON file
(including `cost`, `headers`, `compat`). Leave `api_key` unset —
the server always scrubs it before persistence.

## Health-probe incidents

M2 ships the audit event shape (`platform.model_provider.health_degraded`,
Alerted class) but **no scheduled probe** — the probe daemon is
M7b (plan §G5). For M2:

- Status rows are written as `Ok` at register time.
- A degradation event is only emitted if M3+ handler flows (not
  shipped yet) observe a runtime-level failure and update the row.
- Operators observing external-provider downtime should archive the
  affected runtime manually until the upstream is healthy, then
  re-register.

When M7b lands, the probe emits `health_degraded` automatically and
flips `status`; the wire shape is already pinned by the M2 builder
in [`domain::audit::events::m2::providers`][4].

## Archival

Archive is a **soft delete**: sets `archived_at`, emits
`platform.model_provider.archived` (Alerted). Grants descending from
the original registration AR are **not** revoked in M2 — the admin
holds the grant and the archive is admin-initiated, so there's no
multi-principal exposure. M3 introduces cascade revocation (plan
§Part 11 Q8) when delegated grants become common.

```bash
baby-phi model-provider archive --id <uuid>
# model provider archived
#   provider_id:    <uuid>
#   audit_event_id: <uuid>
```

Archived rows stay queryable via `--include-archived` for audit
replay, but the default list hides them.

## Troubleshooting

| Symptom | Likely cause | Recovery |
|---|---|---|
| `400 VALIDATION_FAILED` with "config.id must be non-empty" | JSON is missing required phi-core fields | check against `phi-core/src/provider/model.rs::ModelConfig` |
| `400 SECRET_REF_NOT_FOUND` | the vault entry the runtime references does not exist | run `baby-phi secret add --slug <slug>` first |
| `409 MODEL_PROVIDER_DUPLICATE` | an active (non-archived) runtime already binds the same `(provider, model.id)` pair | archive the old row or choose a different model id |
| `404 MODEL_PROVIDER_NOT_FOUND` on archive | id is wrong or already archived | `baby-phi model-provider list --include-archived` to confirm |
| `500 INTERNAL_ERROR` on list | SurrealDB read path | check `baby-phi-server` logs + storage health |

## References

- [Architecture: phi-core reuse map](../architecture/phi-core-reuse-map.md) — which phi-core types the runtime wraps.
- [ADR-0016 Template E](../decisions/0016-template-e-self-interested-auto-approve.md) — the self-approved write pattern every registration uses.
- [Operations: secrets vault](secrets-vault-operations.md) — the vault is the prerequisite for any `secret_ref`.
- [User guide: model providers](../user-guide/model-providers-usage.md) — day-to-day operator flow.

[1]: ../../../../../../modules/crates/domain/src/model/composites_m2.rs
[2]: ../../../../../../../phi-core/src/provider/model.rs
[3]: ../../../../../../../phi-core/src/provider/registry.rs
[4]: ../../../../../../modules/crates/domain/src/audit/events/m2/providers.rs
