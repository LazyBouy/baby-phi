<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Page 12 authority template adoption

**Status**: `[EXISTS]` as of M5/P5.

Scope:

- List / approve / deny / adopt-inline / revoke-cascade handlers.
- Template C + D fire pure-fns + listeners (M5/P3).
- `platform.template.adopted` / `adoption_denied` / `revoked`
  audit events (all Alerted).

## Error-code reference

| HTTP | Code | Meaning | Fix |
|---|---|---|---|
| 400 | `TEMPLATE_INPUT_INVALID` | Empty reason on deny/revoke, or unknown kind slug | Re-submit with non-empty reason / valid kind |
| 400 | `TEMPLATE_E_ALWAYS_AVAILABLE` | Caller tried to adopt/approve/deny/revoke E | Template E has no adoption lifecycle — use it directly per AR |
| 403 | `TEMPLATE_ADOPT_FORBIDDEN` | Org has no Human-kind agent to act as CEO | Create a Human agent in the org first |
| 404 | `ORG_NOT_FOUND` | Unknown `org_id` | Verify id |
| 404 | `TEMPLATE_ADOPTION_NOT_FOUND` | No adoption AR exists for (org, kind) | Call `adopt` first |
| 409 | `TEMPLATE_KIND_NOT_ADOPTABLE` | `:kind` is `system_bootstrap` / `f` | Use A / B / C / D only |
| 409 | `TEMPLATE_ADOPTION_ALREADY_PENDING` | Live pending adoption AR | Approve / deny the existing AR |
| 409 | `TEMPLATE_ADOPTION_ALREADY_ACTIVE` | Adoption already Approved | Revoke first if re-adopting |
| 409 | `TEMPLATE_ADOPTION_TERMINAL` | AR is in a terminal state that can't transition further | No action — already terminal |
| 409 | `AR_STATE_TRANSITION_FAILED` | Race — another admin flipped the state between read + write | Retry (idempotent) |
| 500 | `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` | Internal failure | Check server logs |

## Audit event dictionary

| Event | Class | Triggered by | Fields |
|---|---|---|---|
| `platform.template.adopted` (mode=`approve_existing`) | Alerted | approve | `template_kind`, `adoption_auth_request_id` |
| `platform.template.adopted` (mode=`adopt_inline`) | Alerted | adopt | `template_kind`, `adoption_auth_request_id` |
| `platform.template.adoption_denied` | Alerted | deny | `template_kind`, `adoption_auth_request_id`, `reason` |
| `platform.template.revoked` | Alerted | revoke | `template_kind`, `adoption_auth_request_id`, `grant_count_revoked`, `reason` |

All four carry `org_scope = Some(org_id)` + `provenance_auth_request_id = Some(adoption_ar_id)` for hash-chain cross-reference.

## Incident playbooks

- **Revoke stalled mid-cascade** — the `revoke_grants_by_descends_from`
  repo call walks every live grant under the adoption AR and
  flips them in one compound tx. If the audit log shows partial
  revokes (the AR is Revoked but some grants remain live), the
  tx failed mid-commit. Fix: call `POST /:kind/revoke` again — the
  handler is idempotent (second call returns 409
  `TEMPLATE_ADOPTION_TERMINAL`, but the grant cascade re-runs on
  the first successful call; if the AR is already Revoked the
  cascade has already drained live grants to completion).
- **Template adoption wouldn't activate** — the fixture adopts
  all enabled templates at org-creation time as Template-E-shape
  (immediately Approved). If `GET /authority-templates` shows a
  template in `pending` instead, investigate org-creation flow
  for a rare race OR a template created via a non-wizard path.
  Workaround: `POST /:kind/approve` to force-transition.
- **Revoke affects grants you didn't expect** — cascade is
  forward-only: `DESCENDS_FROM` walked from THIS adoption AR.
  Grants minted under a different adoption AR are unaffected.
  Cross-reference the audit log's
  `platform.template.grant_fired` events to see which grants
  descended from which adoption.
- **Multi-org shared template confusion** — Migration 0005 made
  Template rows platform-level (UNIQUE by kind, not by name).
  Per-org adoption lives on the AR. Walking adoption history
  requires AR traversal via
  `Repository::list_adoption_auth_requests_for_org`, NOT Template
  traversal.

## Cross-references

- [Authority templates architecture](../architecture/authority-templates.md).
- [ADR-0030](../decisions/0030-template-node-uniqueness.md).
- [Event bus M5 extensions](../architecture/event-bus-m5-extensions.md).
