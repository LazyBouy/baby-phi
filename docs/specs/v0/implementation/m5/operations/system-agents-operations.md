<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Page 13 system agents config

**Status**: [PLANNED M5/P6] — stub seeded at M5/P0; filled at P6
with queue-depth playbook + disable-standard risk assessment +
listener-upsert debugging.

Scope at M5/P6:

- List / tune / add / disable / archive handlers.
- `SystemAgentRuntimeStatus` queue-depth + last-fired-at upsert
  via shared helper from 5 listeners.
- Strong-warning dialog on disable-standard.

## Incident playbooks (land at P6)

- **Queue runaway** — `SystemAgentRuntimeStatus.queue_depth`
  climbs without bound; listener body is leaking OR the upstream
  event bus is emitting faster than the listener drains. Debug
  via listener log + event bus metrics.
- **Standard system agent disabled** — surfaces strong-warning
  dialog pre-action; post-disable, M6 inbox + M7 observability
  surfaces are affected. Re-enable via re-add-with-same-role.

## Cross-references

- [System agents architecture](../architecture/system-agents.md).
- [System flows s02 + s03 operations](system-flows-s02-s03-operations.md).
