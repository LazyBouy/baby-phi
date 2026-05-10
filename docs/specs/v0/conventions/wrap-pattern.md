<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 — NEW convention doc per v0/conventions/ peer tier, cycle hex 240616a4) -->

# Wrap-pattern conventions

Reviewer-tier guidance for the four wrap idioms baby-phi uses to extend phi-core types with governance fields. Governance authority: [ADR-0058](../implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md) §D58.2. Per-record-type precedent: [ADR-0029](../implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md) §D29.1 (nested-not-flatten) + §D29.2 (interior-mut). Concept doc [`phi-core-mapping.md`](../concepts/phi-core-mapping.md) §"wrap: baby-phi field holds phi-core type" is silent below the serde / concurrency / wire-shape / coordination granularity covered here.

## 1. Nested-inner form (NOT `#[serde(flatten)]`)

`Session` / `LoopRecordNode` / `TurnNode` wrap phi-core types via plain nested `inner: phi_core::X` field — NOT `#[serde(flatten)]`. Phi-core's `Session.session_id` would collide on flatten with baby-phi's governance `id` field.

- **Reviewer rule:** any new baby-phi wrap of a phi-core type uses nested `inner`.
- **Closes:** D1.3. **Cross-ref:** ADR-0058 §D58.2 + ADR-0029 §D29.1.

## 2. Interior-mutability wrap (`Arc<Mutex<phi_core::SessionRecorder>>`)

`BabyPhiSessionRecorder.inner: Arc<Mutex<phi_core::SessionRecorder>>` — phi-core's `&mut self` recorder API requires interior mutability when the recorder lives behind shared state (`AppState::session_registry`).

- **Reviewer rule:** any phi-core API that takes `&mut self` and lives in shared state wraps with `Arc<Mutex<_>>`.
- **Closes:** D3.3. **Cross-ref:** ADR-0058 §D58.2 + ADR-0029 §D29.2.

## 3. HTTP wire-shape projection (`Vec<ToolSummary>`)

`resolve_agent_tools(...) -> Result<Vec<ToolSummary>, _>` returns the HTTP wire shape (`ToolSummary` is a `Serialize` projection of tool metadata) — NOT `Vec<Box<dyn AgentTool>>` (phi-core trait-object). The compile-time witness `_is_phi_core_agent_tool_trait<T: AgentTool + ?Sized>` is preserved to assert the trait remains satisfiable; the runtime wire path uses the projection.

- **Reviewer rule:** HTTP handlers expose Serialize projections; trait-object runtime path stays internal.
- **Closes:** D4.3. **Cross-ref:** ADR-0058 §D58.2.

## 4. Dual-mode discriminator (`Option<X>`)

`SessionLaunchContext.first_loop_id: Option<LoopId>` — `Some(...)` for the launch-chain pre-persisted path; `None` for the standalone-test full-persist path. The `Option` discriminator distinguishes two coordination modes without a separate type.

- **Reviewer rule:** a single struct serving two coordination modes uses `Option`-discriminated semantics over a separate type when the modes share most fields.
- **Closes:** D4.6. **Cross-ref:** ADR-0058 §D58.2.
