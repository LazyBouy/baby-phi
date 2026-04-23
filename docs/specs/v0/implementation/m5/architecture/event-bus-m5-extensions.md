<!-- Last verified: 2026-04-23 by Claude Code -->

# Event bus — M5 extensions

**Status**: [PLANNED M5/P3] — stub seeded at M5/P0; full emit-site
table + consumer matrix lands at P3 when the new `DomainEvent`
variants + 4 listener scaffolds ship.

M4/P3 introduced the in-process event bus with a single
`DomainEvent::HasLeadEdgeCreated` variant + `TemplateAFireListener`.
M5 extends the variant enum + wires 4 new listeners (2 full-body at
P3, 2 stubs filled at P8).

## 8 new `DomainEvent` variants (P3)

| Variant | Emit site | Subscriber(s) |
|---|---|---|
| `SessionStarted { session_id, agent_id, project_id, started_at, audit_event_id }` | `sessions/launch.rs` post-commit | (none at M5; M7 observability) |
| `SessionEnded { session_id, agent_id, project_id, ended_at, duration_ms, turn_count, tokens_spent, audit_event_id }` | `BabyPhiSessionRecorder` on final `AgentEnd` | `MemoryExtractionListener` (s02) |
| `SessionAborted { session_id, reason, terminated_by, at, audit_event_id }` | `sessions/terminate.rs` + panic scopeguard | (observability only) |
| `ManagesEdgeCreated { org_id, manager, subordinate, at, audit_event_id }` | org / agent mutation handlers | `TemplateCFireListener`, `AgentCatalogListener` |
| `HasAgentSupervisorEdgeCreated { project_id, supervisor, supervisee, at, audit_event_id }` | project-lead assignment flow | `TemplateDFireListener`, `AgentCatalogListener` |
| `AgentCreated { agent_id, owning_org, kind, role, at, audit_event_id }` | `agents/create.rs` post-commit | `AgentCatalogListener` |
| `AgentArchived { agent_id, at, audit_event_id }` | `agents/archive.rs` (if shipped) OR via PATCH | `AgentCatalogListener` |
| `HasProfileEdgeChanged { agent_id, old_profile_id, new_profile_id, at, audit_event_id }` | `agents/update.rs` post-commit (profile_ref swap) | `AgentCatalogListener` |

## Fail-safe ordering

All emit sites remain **post-commit** (M4 ADR-0028's invariant).
If emit fails, the governance write is already durable; the event
is logged with `event_id` for manual replay. M7b adds persistent
event queue for exactly-once semantics.

## Listener registration

`AppState::new` at M5/P3 close wires **5 listeners** (M4's
`TemplateAFireListener` + the 4 new): asserted by
`server/src/state.rs::tests::handler_count_is_five_at_m5`.

## Cross-references

- [M4 ADR-0028](../../m4/decisions/0028-domain-event-bus.md) — parent architectural decision.
- [M5 plan §P3](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
