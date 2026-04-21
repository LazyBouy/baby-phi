<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — M2 overview

M2 extends the M1 spine with the first admin-write surfaces (pages
02–05). This page is the system map; per-topic deep dives live under
`architecture/`. Phase status flags track where each piece is in its
rollout.

## What M2 adds on top of M1

| Area | Pre-M2 (M1 shipping) | M2 extension | Status |
|---|---|---|---|
| Domain model | 9 fundamentals, 8 composites, 37 nodes, 66 edges | `ModelRuntime`, `ExternalService`, `SecretCredential`, `PlatformDefaults` composite-instance structs; `TemplateKind` discriminator; `SecretId`, `ModelProviderId`, `McpServerId` ID newtypes | [EXISTS] (P1) |
| phi-core reuse | `AgentProfile` wraps phi-core's (P0) | Every M2 composite overlapping phi-core imports or wraps it — see [phi-core-reuse-map.md](phi-core-reuse-map.md) | [EXISTS] (P1) |
| Schema | Migration `0001_initial.surql` | Migration `0002_platform_setup.surql` adds `template.kind`, full `mcp_server` columns, `model_runtime` table, singleton `platform_defaults` | [EXISTS] (P1) |
| Repository | 36 methods | ~17 M2 methods (secrets ×5, providers ×3, mcp ×4, defaults ×2, cascade ×2, catalogue ×1) | [EXISTS] (P2) |
| Engine | 8-step Permission Check | Step 4 widened to support constraint **value-match** (`purpose=reveal`) | [EXISTS] (P2) |
| Templates | `SystemBootstrap` only | `TemplateKind::E` + `build_auto_approved_request` pure helper | [EXISTS] (P2) |
| Handler infra | Bootstrap bypassed the engine | `handler_support` module: `AuthenticatedSession`, `check_permission`, `emit_audit`, shared `ApiError` | [EXISTS] (P3) |
| Audit | Trait + hash helpers | `SurrealAuditEmitter` concrete impl; per-event builder functions | [EXISTS] (P3) |
| Web shell | `/bootstrap` SSR page | `app/(admin)/` Route Group + auth gate + shared primitives | [EXISTS] (P1) |
| CLI | `bootstrap`, `agent demo` | `login`, `secret`, `model-provider`, `mcp-server`, `platform-defaults`, `completion` | [EXISTS] (P4–P8) |

## Phase-to-page mapping

M2's P4–P7 are **vertical slices** — each phase ships one admin page's
Rust handler + CLI subcommand + Web page + operations doc together,
rather than batching by layer.

| Phase | Page | Artifacts |
|---|---|---|
| P4 | Page 04 — Credentials Vault | [vault-encryption.md](vault-encryption.md), CLI `secret {list,add,rotate,reveal,reassign}`, `app/(admin)/secrets/` |
| P5 | Page 02 — Model Providers | CLI `model-provider {list,add,archive}`, `app/(admin)/model-providers/` |
| P6 | Page 03 — MCP Servers | [cascading-revocation.md](cascading-revocation.md), CLI `mcp-server {list,add,patch,archive}`, `app/(admin)/mcp-servers/` |
| P7 | Page 05 — Platform Defaults | [platform-defaults.md](platform-defaults.md), CLI `platform-defaults {get,set,import,export}`, `app/(admin)/platform-defaults/` |

## Testing posture (authoritative — P1 seed)

| Layer | Count at M1 close | +M2 target | Post-M2 target |
|---|---|---|---|
| Rust (`cargo test --workspace`) | 299 | +~155 | ~454 |
| Web (`npm test`) | 14 | +~20 | ~34 |
| **Combined** | **313** | **+~175** | **~488** |

Test counts are aggregated from `cargo test` and `npm test` outputs; the
plan's P5 row spells out the per-layer budgets.

## Cross-references

- [`phi-core-reuse-map.md`](phi-core-reuse-map.md) — the durable source
  of truth for which phi-core types M2 wraps or imports.
- [`../../../concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md)
  — the mapping table M2's reuse map derives from.
- M1 overview: [`../../m1/architecture/overview.md`](../../m1/architecture/overview.md)
  — what the spine this milestone extends looks like.
