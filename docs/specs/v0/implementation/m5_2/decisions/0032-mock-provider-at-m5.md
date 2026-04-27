<!-- Last verified: 2026-04-24 by Claude Code -->

# ADR-0032 — MockProvider at M5 driven by `AgentProfile.mock_response`; real providers deferred to M7

**Status: Accepted** (flipped at CH-02 chunk seal — P5, 2026-04-24).

Ratification evidence:
- `phi_core::agent_loop()` is invoked at runtime from `spawn_agent_task` in [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs); `tokio::join!(agent_fut, drain_fut)` pattern drives event flow into `BabyPhiSessionRecorder::on_phi_core_event`.
- `provider_for(runtime, profile)` shipped at [`modules/crates/server/src/platform/sessions/provider.rs`](../../../../../../modules/crates/server/src/platform/sessions/provider.rs) returning `Arc::new(MockProvider::text(profile.mock_response.unwrap_or("Acknowledged.")))`.
- `AgentProfile.mock_response: Option<String>` governance field shipped on baby-phi's wrapper at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); `#[serde(default)]` for backward-compatible reads. NOT placed on `phi_core::agents::profile::AgentProfile.blueprint` — the phi-core inner stays untouched per CLAUDE.md §phi-core Leverage rule 1.
- Migration `0006_agent_profile_mock_response.surql` adds `DEFINE FIELD mock_response ON agent_profile TYPE option<string>;`; registered in `EMBEDDED_MIGRATIONS` slice; `migrations_test.rs` asserts the 6th migration row in the `_migrations` ledger.
- Acceptance tests at [`modules/crates/server/tests/acceptance_sessions_m5p4.rs`](../../../../../../modules/crates/server/tests/acceptance_sessions_m5p4.rs) — `launch_happy_path_persists_session_and_writes_uses_model_edge` asserts `triggered_by="User"` + prompt round-trip + default `"Acknowledged."` response; `launch_with_mock_response_override_drives_agent_output` proves the per-profile override flows through the full launch → agent_loop → recorder path.
- Drift D4.2 transitioned to `remediated`; `leverage-violation` tag removed.

## Context

M5/P4 shipped a 9-step launch flow at [`sessions/launch.rs`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs) but the execution step fabricates a canonical 4-event sequence rather than calling `phi_core::agent_loop()`. The imports `use phi_core::{agent_loop, agent_loop_continue}` exist only as compile-time witnesses pinned by `_keep_agent_loop_live`. This is drift **D4.2**, tagged `leverage-violation` in the M5.1 catalogue.

Chunk [CH-02](../../../../plan/build/16fd9a3a-ch-02-real-agent-loop-wiring.md) closes D4.2 by wiring a real `phi_core::agent_loop()` call. Two sub-decisions must be locked before the wiring can land:

1. **Which provider does baby-phi use at M5?** Real providers (Anthropic / OpenAI / Bedrock / Gemini / etc.) are dispatched by `phi_core::provider::registry::ProviderRegistry` from `ModelConfig.api`. But M5 does not yet ship a credentials vault or secrets-at-rest encryption (both M7b scope). Calling real providers directly from the acceptance test suite would require per-dev API keys and break the "no network calls in tests" contract.

2. **How does baby-phi override the provider for testing + dev?** Hard-coded in the launch handler ties test fixtures to a single response. Per-profile configuration makes future chunks (CH-21 memory-extraction acceptance, CH-23 Template C/D verification) able to seed deterministic multi-turn scripts without re-opening the handler.

## Decision

### D32.1 — `provider_for(runtime, profile) -> Arc<dyn StreamProvider>` helper at M5 returns `MockProvider`

A new helper at `modules/crates/server/src/platform/sessions/provider.rs` (created by CH-02 P2):

```rust
pub fn provider_for(
    _runtime: &ModelProvider,
    profile: &AgentProfile,
) -> std::sync::Arc<dyn phi_core::provider::traits::StreamProvider> {
    use phi_core::provider::mock::MockProvider;
    std::sync::Arc::new(MockProvider::text(
        profile
            .mock_response
            .clone()
            .unwrap_or_else(|| "Acknowledged.".to_string()),
    ))
}
```

At M5, `runtime.provider_kind` is **ignored** — every session runs through `MockProvider`. The helper is wired into `AgentLoopConfig.provider_override: Some(provider_for(&runtime, &profile))` at the `phi_core::agent_loop` call site.

At M7+ the helper body fans out to `ProviderRegistry::get(model_config.api)` and dispatches real providers; the `mock_response` field stays as a test-mode override but is bypassed when real credentials are configured.

### D32.2 — `AgentProfile.mock_response: Option<String>` governance field on the baby-phi wrapper

baby-phi's `AgentProfile` wrapper at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) gains a new governance field:

```rust
pub struct AgentProfile {
    pub id: AgentProfileId,
    pub blueprint: phi_core::agents::profile::AgentProfile,
    pub parallelize: Option<u32>,
    pub model_config_id: Option<String>,

    /// Dev + test override. When Some, MockProvider returns this
    /// text instead of the default "Acknowledged." at M5. At M7+
    /// this field is bypassed when real providers are configured.
    #[serde(default)]
    pub mock_response: Option<String>,
}
```

**Critical placement rule** (per [`CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core Leverage" rule 1): the field lives on baby-phi's wrapper struct, **not** on the inner `phi_core::agents::profile::AgentProfile` blueprint. The blueprint is phi-core's canonical shape and must not be forked. Governance / test-mode extensions belong on the wrapper.

Schema: a new migration [`modules/crates/store/migrations/0006_agent_profile_mock_response.surql`](../../../../../../modules/crates/store/migrations/) adds `DEFINE FIELD mock_response ON TABLE agent_profile TYPE option<string>;`.

Default: `None`. No admin UI surfaces this field at M5 (internal/test-only); M6+ may expose it in the profile detail view if operators need it for debugging.

### D32.3 — Real providers deferred to M7

`phi_core::provider::registry::ProviderRegistry` dispatch defers to M7's credentials-vault milestone. Until then, baby-phi does not call any network-facing LLM provider from its own launch path.

**Conforming-real-provider criteria** (binding for M7 plan-open):

1. **Credentials** — honors `ModelConfig.api_key` lookup via baby-phi's secrets vault (M7 scope); never reads keys from process env directly in a test or acceptance context.
2. **Event-stream fidelity** — emits the same `phi_core::types::event::AgentEvent` variants in the same order as `MockProvider` for identical input messages; any divergence is logged as a phi-core upstream bug, not papered over.
3. **Cancellation** — respects `cancel_token.is_cancelled()` at every turn boundary + at every streaming-delta boundary; cancellation latency ≤ 1 turn.
4. **Integration test fixture** — each real provider ships with a recorded VCR-style fixture under `modules/crates/server/tests/fixtures/providers/<provider>/` and an integration test that re-runs the fixture; CI uses the fixture by default and only hits the real network under a `RUN_PROVIDER_LIVE=1` env flag.

### D32.4 — `_keep_agent_loop_live` compile-time witness preserved

The dead-code witness at [`launch.rs:89-98`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs#L89-L98) stays in place; its comment is rewritten from "would need a live provider; out of M5 scope" to "runtime-exercised via MockProvider at M5; pins against phi-core rename." The witness doubles as regression insurance: if phi-core ever renames `agent_loop`, the build fails immediately rather than silently through the runtime call site.

## Consequences

**Positive**
- Zero network dependency in the acceptance test suite — every test determinisitc + offline.
- Downstream chunks (CH-21 memory-extraction acceptance, CH-23 Template C/D verification) get a clean per-profile knob to seed fixture transcripts without touching `launch.rs` again.
- Real `phi_core::agent_loop` runs in production code paths → `leverage-violation` tag removed from D4.2; phi-core integration is genuine, not nominal.
- The conforming-criteria list (D32.3) is an explicit pre-agreed contract for M7 — future provider integrations have a rubric to satisfy.

**Negative**
- `mock_response` is a dev/test-shaped field leaking into the production `agent_profile` schema. Minor concern: the column adds ~16 bytes per row + serialises as `Option<string>` on the wire.
- Per-profile config means test setup code must remember to seed `mock_response` — tests that forget will get the default `"Acknowledged."` response and may fail opaquely. Mitigation: a `seed_mock_response` test helper lands alongside CH-21 when richer scripts first become needed.

**Neutral**
- No new phi-core imports beyond what CH-02 already requires (`MockProvider`, `StreamProvider`, `AgentContext`, `AgentLoopConfig`). Leverage lint stays green.
- M7 can trivially backfill the `runtime.provider_kind` dispatch: `provider_for`'s signature already takes a `&ModelProvider`, so the body swap doesn't cascade through call sites.

## Alternatives considered

- **Hardcoded single-turn ack (`MockProvider::text("Acknowledged.")`), no per-profile field.** Smallest M5 surface but defers the problem — CH-21 / CH-23 would need to add the knob later, which risks a second round of migration + repo churn. Rejected at CH-02 plan-approval time (2026-04-24) in favour of doing it once, now.
- **`LaunchInput.mock_script: Option<String>`** — per-request test override. Simpler than a schema column but forces every test call site to thread the field through; loses the "agent has a known deterministic behaviour" framing.
- **Read mock script from a top-level `config/dev.toml` override.** Would work for dev but not for per-agent tests in the acceptance suite. Rejected.

## Review trigger

**M7 plan-open.** When M7 opens, the author of the real-provider chunk re-reads this ADR and either flips sections D32.1 + D32.3 to `Superseded by ADR-NNNN` (where NNNN is the M7 decision doc for the credentials vault + ProviderRegistry dispatch) or amends here if the criteria list needs tightening.

## References

- [CH-02 chunk plan `16fd9a3a-ch-02-real-agent-loop-wiring.md`](../../../../plan/build/16fd9a3a-ch-02-real-agent-loop-wiring.md) — the implementation this ADR ratifies.
- [Drift D4.2](../../m5_1/drifts/D4.2.md) — the gap this ADR closes.
- [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §"agent_loop free function" — the source-of-truth claim this ADR re-aligns code with.
- [`CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core Leverage" rules 1–5 — the wrap-with-governance-fields pattern ADR-0032 D32.2 follows.
- [ADR-0031 — Session cancellation + concurrency bounds](../../m5/decisions/0031-session-cancellation-and-concurrency.md) — sibling M5 cancellation contract the `cancel_token` pass-through in CH-02 P3 honors.
- [ADR-0027 — Per-agent execution limits override](../../m4/decisions/0027-per-agent-execution-limits-override.md) — sibling governance-field-on-wrapper precedent.
