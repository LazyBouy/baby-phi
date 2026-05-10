<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 — NEW convention doc per v0/conventions/ peer tier, cycle hex 240616a4) -->

# Event-bus + listener-wiring conventions

Reviewer-tier guidance for domain-event field naming, listener-wiring seams, scoped-resolver trait splits, typed-writer-per-edge-type discipline, and audit-event placement. Governance authority: [ADR-0058](../implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md) §D58.4 (event-bus wiring) + §D58.5 (audit-event placement cross-ref). Cross-refs: [ADR-0028](../implementation/m4/decisions/0028-domain-event-bus.md) (M4 event-bus pattern) + [ADR-0057](../implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md) §D57.1 (CH-19 audit-event placement convention).

## 1. Serde-tag-collision rename

`DomainEvent::AgentCreated` field is named `agent_kind: AgentKind` (NOT `kind: AgentKind`) to avoid collision with the enum-level `#[serde(tag = "kind")]` discriminator.

- **Reviewer rule:** any new `DomainEvent` variant carrying a `kind`-named field renames to `<topic>_kind` to avoid the serde-tag collision.
- **Closes:** D3.1. **Cross-ref:** ADR-0058 §D58.4.

## 2. Free-function listener seam

Listener wiring lives at free function `state::build_event_bus_with_m5_listeners` — NOT inside `AppState::new`. The free-function seam keeps the listener set test-isolatable (`handler_count_is_five_at_m5` asserts the count without spinning a full `AppState`).

- **Reviewer rule:** new listeners attach via the free-function seam; the count test gets a row update.
- **Closes:** D3.2. **Cross-ref:** ADR-0058 §D58.4.

## 3. Trait-split-by-scope for resolvers

`TemplateCAdoptionArResolver(OrgId)` is org-scoped; `TemplateDAdoptionArResolver(ProjectId → (OrgId, AuthRequestId))` is project-scoped. The asymmetry is concept-mandated: `MANAGES` lives at org scope; `HAS_AGENT_SUPERVISOR` lives at project scope (per [`permissions/07`](../concepts/permissions/07-templates-and-tools.md)).

- **Reviewer rule:** scope-asymmetric concept primitives get split-by-scope resolver traits, not a single resolver with conditional dispatch.
- **Closes:** D3.4. **Cross-ref:** ADR-0058 §D58.4.

## 4. Typed-writer-per-edge-type at Repository trait

Repository trait surfaces typed methods like `write_uses_model_edge` per edge-type — NOT a generic `create_edge(&Edge)`. Default trait body returns `RepositoryError::Backend(...)` so future impls fail fast on missed coverage.

- **Reviewer rule:** new edge-types get a typed-writer Repository method with a `Backend(...)` default body.
- **Closes:** D4.5. **Cross-ref:** ADR-0058 §D58.4.

## 5. Audit-event placement cross-ref (system-agents page-13)

System-agent page-13 audit-events live at `server::platform::system_agents::audit_events` (4 builder fns: reconfigure / add / disable / archive). This is the same platform-tier convention CH-19 / ADR-0057 §D57.1 ratified for page-12 templates. Both feed the single-writer `AuditEmitter` chain (per ADR-0033 §D33.4), preserving the K8s A7 single-writer-symmetry guarantee.

- **Reviewer rule:** new HTTP-handler-tier audit events default to `server::platform::<page>::audit_events`; new state-machine / fire-listener events default to `domain::audit::events::mX::*`.
- **Closes:** D6.4. **Cross-ref:** ADR-0058 §D58.5 + CH-19 / ADR-0057 §D57.1.
