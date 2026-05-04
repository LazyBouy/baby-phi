<!-- Last verified: 2026-05-04 by Claude Code (filed by CH-13 retrospective; cycle hex `d4fe1b7c`) -->

# D-CH13-FOLLOWUP-01 — Platform-admin Grant-mint paths do not invoke `compose_audit_class` against `Organization.audit_class_default`

## Identification
- **ID**: D-CH13-FOLLOWUP-01
- **Phase of origin**: CH-13 retrospective (cycle hex `d4fe1b7c`)
- **Discovery source**: `cycle-audit-findings` (carry-forward gap surfaced at plan §1 "What this chunk does NOT do" + cycle-audit §6 #9)
- **Date discovered**: 2026-05-04
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap with template-side enforcement landed; platform-admin-side deferred
- **Severity**: LOW
- **Tags**: `audit-integrity`, `platform-admin`, `compliance-posture`, `grant-mint`
- **Blocks**: nothing at v0 (template-side audit composition is honored by CH-13; platform-admin paths use direct grant-mint without composition, which is correct for v0 since there is no compliance-posture requirement that platform admin grants honor `Organization.audit_class_default`)
- **Blocked-by**: future M6+ chunk (TBD; not yet allocated in forward-scope) when compliance-posture extension to platform admin grants is requested

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" (lines 64–72).
- **Concept claim**: The strictest-wins composition rule is documented under "Templates" specifically. Platform-admin grants minted via direct CRUD (orgs/create, secrets/add, mcp_servers/register, model_providers/register, bootstrap/claim) are NOT template-fire paths; they don't have a `template_AR` input + may legitimately default to `Silent` until/unless the operator specifies otherwise.
- **Contradiction**: NONE today (concept doc 07's strictest-wins rule applies to templates; platform admin paths are out of scope per concept). However, an operator that opts into `Organization.audit_class_default = Alerted` for compliance reasons sees:
  - **Template-fire grants (Template A/C/D)**: composed correctly via CH-13's `compose_audit_class` — Alerted respected.
  - **Platform-admin grants (orgs/create, secrets/add, ..., 5 production sites + 1 store decoder)**: hardcoded `AuditClass::Silent`. The org's compliance preference is not honored by this category.
  - The operator may or may not consider this acceptable. If compliance posture demands ALL grants honor `Organization.audit_class_default`, this gap is load-bearing and needs closing.
- **Classification**: `partially-honored` (template axis honored by CH-13; platform-admin axis stays at default `Silent`)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (CH-13 plan §1 "What this chunk does NOT do"): "Does NOT touch Template E's `BuildArgs::audit_class` field. Template E is the self-interested-auto-approve path used at platform-admin / page-02–05 writes; its caller already supplies the audit class at construction time. Composing on top would require the caller to know the org's default, which is two layers above current callers. Out of scope; would require a separate forward-scope row."
- **Plan said** (CH-13 plan §1 "What this chunk does NOT do" — extended interpretation): The 5 platform-admin Grant production sites + 1 store decoder + ~17 test fixtures stay at `audit_class: AuditClass::Silent` placeholder per the F2.A locked path's mechanical cascade. CH-13's strictest-wins composer wires only the 3 template-fire paths (Template A/C/D); Platform admin paths are out of scope.
- **Reality**: matches the plan exactly. CH-13 ships immutability + composition for template-fire paths only. The 5 platform admin Grant paths + 1 store decoder use direct grant-mint without composition.

## Required follow-up
- **What needs to happen**: when a future M6+ chunk lands "Platform-admin Grant compliance-posture extension" (or equivalent — TBD title at the time of allocation), the 5 production platform-admin Grant-mint sites SHOULD invoke a composer (similar to CH-13's `compose_audit_class`) that uses `Organization.audit_class_default` as input. Whether `template_AR` and `override` apply is template-specific:
  - For platform-admin paths there is no `template_AR` input (the grant is not template-derived). Composer signature may simplify to `compose_admin_audit_class(org_default, override: Option<AuditClass>) -> AuditClass`.
  - The 5 sites: `bootstrap/claim.rs:230`, `mcp_servers/register.rs:117`, `model_providers/register.rs:146`, `orgs/create.rs:178`, `secrets/add.rs:133` (and the 1 store decoder at `store/src/repo_impl.rs`).
  - Audit-event diff extension to include `audit_class_source` (mirrors CH-13 / ADR-0050 §D50.6).
- **Tests required**: acceptance scenarios verifying that platform-admin Grant + audit-event from each of the 5 mint paths honors the org's `audit_class_default`.
- **Acceptance**: every platform-admin Grant-mint path honors `Organization.audit_class_default`; the immutability-axis stays honored by existing Grant denormalisation (CH-13 ADR-0050 §D50.5).

## Closing chunk
- TBD — likely M6+ "Platform-admin Grant compliance-posture extension" follow-up; not yet allocated in forward-scope.

## Lifecycle
- **2026-05-04 — `discovered`** — filed by CH-13 retrospective. CH-13 ships strictest-wins composer + Grant denormalisation for the 3 template-fire paths (Template A/C/D); platform admin paths deferred to M6+. Mirrors CH-11's `D-CH11-FOLLOWUP-01` + CH-12's `D-CH12-FOLLOWUP-01` patterns (chunk closes one axis of a multi-axis concept-doc claim; the other axis tracked here).

## Cross-references
- CH-13 plan: [`baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md`](../../../../plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md) §1 "What this chunk does NOT do".
- CH-13 retrospective: [`baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/retrospective.md`](../../../../plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/retrospective.md) §6 Q1.
- ADR-0050: [`m5_2/decisions/0050-audit-class-composition-strictest-wins.md`](../../m5_2/decisions/0050-audit-class-composition-strictest-wins.md) §D50.3 (composer is template-specific; platform-admin extension is out of scope).
- D-new-19: [`D-new-19.md`](D-new-19.md) (closed by CH-13 — template-fire axis).
- Sister patterns: [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md) (CH-11's analogous follow-up for `Project.deadline_at`); [`D-CH12-FOLLOWUP-01.md`](D-CH12-FOLLOWUP-01.md) (CH-12's analogous follow-up for session-tag emission).
- Concept doc: [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) lines 64–72.
- Affected files: `bootstrap/claim.rs:230`, `mcp_servers/register.rs:117`, `model_providers/register.rs:146`, `orgs/create.rs:178`, `secrets/add.rs:133`, `store/src/repo_impl.rs` (GrantRow translator).
