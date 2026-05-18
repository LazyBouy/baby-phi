<!-- Last verified: 2026-04-28 by Claude Code -->

# D-new-11 — Composite instances don't auto-emit their `{kind}:{id}` self-identity tag at creation

## Identification
- **ID**: D-new-11
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `composite-ontology`, `auto-tagging`
- **Blocks**: D-new-06 (scope resolution depends on instance tags); D-new-08 (frozen tag enforcement needs to know the structural tags)
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Instance Identity Tags"
- **Concept claim**: Every composite instance carries a self-identity tag `{kind}:{instance_id}` (e.g. `session:sess-42`) at creation, in addition to the `#kind:{name}` type tag.
- **Contradiction**: `kind_tag()` auto-adds `#kind:session`, but no code auto-adds `session:sess-42` as an instance tag. Session + Memory + other composite creation paths do not populate a tag-list field with the instance self-identity.
- **Classification**: `partially-honored` (kind tag honored; instance tag missing)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Both `#kind:{name}` AND `{kind}:{id}` tags auto-added at creation.
- **Reality (shipped state at current HEAD)**: Only kind tag auto-logic. Instance self-identity is implicit in node id but not reified as a tag.
- **Root cause**: `concept-doc-not-consulted` during M1 composite-ontology work.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/model/composites.rs:81-92`](../../../../../../modules/crates/domain/src/model/composites.rs#L81-L92) `kind_tag()` — auto-emits `#kind:*` only.
- **Test evidence**: None testing instance self-identity tag presence.
- **Grep for regression**: Check composite-creation sites emit both tags.

## Remediation scope (estimate only)
- **Approach (sketch)**: Add `instance_tag(kind, id) -> String` helper returning `format!("{kind}:{id}")`. Wire into each composite creation path (session, memory, inbox_object, outbox_object, auth_request, etc.).
- **Implementation chunk this belongs to**: CH-06
- **Dependencies on other drifts**: none
- **Estimated effort**: 1 engineer-day.
- **Risk to concept alignment if deferred further**: MEDIUM — foundational tag invariant; selectors that match `session:sess-42` against instance tags don't have the expected tag to match against.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
- 2026-04-24 — `classified` — Bucket B MEDIUM; `Composite::auto_tags` helper exists at `composites.rs:119` with full unit-test coverage but ZERO call sites in production code; concept-01 §"Instance Identity Tags" cited as binding (backfill)
- 2026-04-24 — `scoped` — assigned to CH-06 per [forward-scope §1 line 78](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (backfill; bundled with D-new-03 because instance tags are the matcher's target data — selectors like `tags contains session:s-9831` cannot match without instance-tag emission)
- 2026-04-28 — `in-chunk-plan` — CH-06 plan approved ([`build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md`](../../../../plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md)); 15-type rollout (11 composites + Memory + Session + AuthRequest + ExternalService + InboxObject + OutboxObject) in migration 0008; auto_tags() wired at all creation paths per ADR-0037
- 2026-04-28 — `remediated` — CH-06 chunk-seal; migration 0008 adds `tags ARRAY<string> DEFAULT []` to 10 SurrealDB tables (mcp_server, token_budget_pool, agent_execution_limits, agent_catalog_entry, system_agent_runtime_status, shape_b_pending_projects, auth_request, inbox_object, outbox_object, session — Memory.tags pre-existed); 14 struct types gain `pub tags: Vec<String>` (#[serde(default)]); free helper `auto_tags_for(kind, id) -> [String; 2]` ships in `composites.rs` for the M3+ struct types outside the `Composite` enum; production emission wired at 10+ creation handlers (server/src/platform/{mcp_servers/register,sessions/launch,agents/{create,update},system_agents/add,orgs/create,projects/create}, server/src/bootstrap/claim, domain/src/{templates/e,events/listeners,session_recorder}); store-side reader paths emit canonical pair on read for auth_request + mcp_server tables (other 8 tables backfill via ops runbook); 13-test acceptance fixture [`instance_tags_emission.rs`](../../../../../../modules/crates/domain/tests/instance_tags_emission.rs) pins emission per concept-01 §"Instance Identity Tags"; 5 embedded value-objects (Objective, KeyResult, ResourceBoundaries, OrganizationDefaultsSnapshot, SessionDetail) carry the field for shape consistency but emission is deferred per ADR-0037 §D37.3. ADR-0037 §D37.1–D37.5 records the design decisions.
