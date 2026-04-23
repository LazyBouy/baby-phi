<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Page 12 authority template adoption

**Status**: [PLANNED M5/P5] — stub seeded at M5/P0; filled at P5
with revoke-cascade incident runbook + audit replay procedure.

Scope at M5/P5:

- List / approve / deny / adopt-inline / revoke-cascade handlers.
- Template C + D fire pure-fns + listeners.
- `AuthorityTemplateRevoked { grant_count_revoked, template_kind }` audit event.

## Incident playbooks (land at P5)

- **Revoke stalled mid-cascade** — `DESCENDS_FROM` walk blocked;
  audit log shows partial revokes. Fix: retry the revoke handler
  (idempotent per design).
- **Template adoption rejected with "prereq missing"** — Template
  C requires MANAGES edge; Template D requires
  HAS_AGENT_SUPERVISOR edge. Adopt after the edge lands.
- **Multi-org shared template confusion** — migration 0005 made
  Template rows platform-level; per-org adoption lives on the AR.
  Walking adoption history requires AR traversal, NOT Template
  traversal.

## Cross-references

- [Authority templates architecture](../architecture/authority-templates.md).
- [ADR-0030](../decisions/0030-template-node-uniqueness.md).
