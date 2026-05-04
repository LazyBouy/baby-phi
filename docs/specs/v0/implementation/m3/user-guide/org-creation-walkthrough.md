<!-- Last verified: 2026-05-04 by Claude Code (CH-13 amendment: appended a one-paragraph "audit_class strictest-wins guarantee" note covering `phi org create --audit-class-default <silent|logged|alerted>`. Doc body otherwise UNCHANGED — still a [PLANNED M3/P4] placeholder.) -->
<!-- Last verified: 2026-04-22 by Claude Code -->

# User guide — Org Creation Walkthrough

**Status: [PLANNED M3/P4]** — fleshed out when P4 ships.

End-to-end 8-step walkthrough of the org creation wizard (CLI + Web).
Fleshed out at P4 close with the shipped handler + YAML reference-
layout seeding (`phi org create --from-layout minimal-startup`).

See [`../../../plan/build/m3-organization-creation-563945fe.md`](../../../../plan/build/m3-organization-creation-563945fe.md) §P4 for the plan.

## `audit_class` strictest-wins guarantee (CH-13)

The `phi org create --audit-class-default <silent|logged|alerted>` flag
sets the org-level audit posture. As of CH-13 (M5.2), every Grant fired
by the standard templates (A/C/D — lead assignment, manager edge,
agent-supervisor edge) has its `audit_class` composed strictest-wins
of (a) the org default set here, (b) the template adoption Auth
Request's `audit_class`, and (c) any per-Grant override (currently
unused by Templates A/C/D). An org configured `--audit-class-default
alerted` for compliance reasons is guaranteed that adopting a template
can never silently downgrade its audit posture; the strictest-wins
fold is structural per ADR-0050. Operators can verify the resolved
class + the winning source via `SELECT audit_class, diff.audit_class_source
FROM audit_events WHERE event_id = 'template.a.grant_fired'` —
see [`../../m5_2/operations/audit-class-composition-operations.md`](../../m5_2/operations/audit-class-composition-operations.md).
