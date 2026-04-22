<!-- Last verified: 2026-04-22 by Claude Code -->

# M3/P0 — M2 pre-flight delta log

**Status: [EXISTS]** — audit pass run at M3 open, 2026-04-22.

Purpose: validate that the M2-close inventory still holds before M3
opens. No code churn — this is a read-only audit. Every item below
is tagged `still-valid | stale | missing` with file+line reference.

Per the M3 plan (archived at
[`../../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md)
§P0), a `>1 stale` finding would open a P0.5 remediation phase before
P1 opens. **Result: 0 stale, 1 missing (expected — routed to P1/G1);
no P0.5 needed.**

## Audit items

| # | Claim being verified | Status | Evidence |
|---|---|---|---|
| 1 | `HAS_CEO` edge variant exists (06-W3 requires the edge from Organization → CEO Human Agent). | **still-valid** | [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) L227 (variant) + L429 (`EDGE_KIND_NAMES` entry). |
| 2 | `HAS_AGENT` edge variant exists (06-W3 step 4: CEO + system agents attached via this edge). | **still-valid** | `edges.rs` L252 + L434. |
| 3 | `MEMBER_OF` edge variant exists (baseline org-membership edge; dashboard reads use it). | **still-valid** | `edges.rs` L134 + L408. |
| 4 | `HAS_SUBORGANIZATION` edge variant exists (reference layout #05 nests orgs; M3 does not exercise this path but the variant must be present for edge-inventory completeness). | **still-valid** | `edges.rs` L242 + L432. |
| 5 | `HAS_LEAD` edge variant exists (Template A's trigger; P1/G1 added this to M3). | **missing** | `edges.rs` — no `HasLead` variant present. `HasLead` is routed to **P1** per plan §G1 / §D5. **Expected missing; no remediation needed at P0.** |
| 6 | `TemplateKind` supports A–F + SystemBootstrap (7 variants). | **still-valid** | [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) L272–L302 (`pub enum TemplateKind` + `pub const ALL: [TemplateKind; 7]` = `[SystemBootstrap, A, B, C, D, E, F]`). |
| 7 | Migration 0002 ASSERT clause on `template.kind` includes `"a"..."f"`. | **still-valid** | [`modules/crates/store/migrations/0002_platform_setup.surql`](../../../../../../modules/crates/store/migrations/0002_platform_setup.surql) L32–L34: `ASSERT $value INSIDE ["system_bootstrap", "a", "b", "c", "d", "e", "f"]`. |
| 8 | `SurrealAuditEmitter::emit` already calls `self.repo.last_event_hash_for_org(event.org_scope)` — M2 only exercised `None`; M3 will exercise `Some(org_id)` without code change. | **still-valid** | [`modules/crates/store/src/audit_emitter.rs`](../../../../../../modules/crates/store/src/audit_emitter.rs) L47: `let prev = self.repo.last_event_hash_for_org(event.org_scope).await?;`. No code change needed — P3 proptest (`two_orgs_audit_chain_props.rs`) verifies per-org chain isolation at the behavioural level. |
| 9 | `Repository::create_organization` method exists (baseline org-writer). | **still-valid** | [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) L152: `async fn create_organization(&self, org: &Organization) -> RepositoryResult<()>;`. Retained at M3 (the compound `apply_org_creation` that P3 adds composes this + agent/inbox/outbox/grant writes inside one SurrealQL tx). |

## Additional observations (beyond the 9-item minimum)

| # | Observation | Status | Evidence |
|---|---|---|---|
| 10 | `Composite::{ControlPlaneObject, InboxObject, OutboxObject}` variants exist (needed for P4: CEO inbox/outbox auto-creation + the `OrganizationDefaultsSnapshot` control-plane catalogue seed). | **still-valid** | [`modules/crates/domain/src/model/composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs) L29, L41, L44; `kind_tag` values at L85, L89–L90. |
| 11 | `EDGE_KIND_NAMES: [&str; 66]` constant + compile-time count test pin 66 today. | **still-valid** | `edges.rs` L468 + L597. **P1 bumps both to 67** when `HasLead` lands (plan §P1 deliverable 1). |
| 12 | `Organization` struct today carries only `{id: OrgId, display_name: String, created_at: DateTime<Utc>}` — no `vision`, `mission`, `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `defaults_snapshot`, `system_agents`. | **still-valid** | `nodes.rs` L238–L242. Matches plan §G3 finding. **P1 extends the struct.** |
| 13 | M2 workspace test baseline: `cargo test --workspace` = 511 passed, 0 failed. | **still-valid** | `cargo test --workspace` run at 2026-04-22 during P0; `grep -E "^test result" | awk '{sum += $4}'` = 511 across all test binaries. Matches plan §Context claim ("511 Rust + 36 Web tests green"). |

## Summary

- **8 still-valid, 1 missing (expected), 0 stale.**
- `HasLead` edge is the only missing item, and it is already routed to P1 per plan §G1 / §D5 (bumps `EDGE_KIND_NAMES` to 67 + updates compile-time count test).
- M2 infrastructure referenced by M3 (`apply_bootstrap_claim` pattern, `SurrealAuditEmitter::emit` with `org_scope`, `Composite::ControlPlaneObject`/`InboxObject`/`OutboxObject`, Template-E `build_auto_approved_request`, `handler_support::ApiError` + extractor) is all load-bearing and unchanged since M2/P8 close.
- No P0.5 remediation phase needed. **P0 closes; P1 opens.**

## Close confidence

**N/A (audit phase, no implementation target).** Close criterion met: every item has a tag with file+line reference; no `stale` tag; exactly one `missing` tag accounted for in the M3 plan.

## References

- M3 plan (this milestone): [`../../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md).
- M2 plan (precedent for vertical-slice discipline): [`../../../../plan/build/a6005e06-m2-platform-setup.md`](../../../../plan/build/a6005e06-m2-platform-setup.md).
- M2/P8 close audit (the 99% composite-confidence result this P0 audits against): `../../../../plan/build/a6005e06-m2-platform-setup.md` §P8.
