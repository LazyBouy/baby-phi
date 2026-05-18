<!-- Last verified: 2026-05-04 by Claude Code (chunk-planner agent, iter 1, plan-mode draft for CH-13) -->

# CH-13 — `audit_class` composition (strictest wins)

**Plan file token:** `d4fe1b7c` (generated 2026-05-04 at chunk-open via `openssl rand -hex 4`).
**Plan archive path (post-approval):** `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md` (folder-style, multi-agent cycle; orchestrator + implementer create the cycle folder via the `chunk-archive-plan` skill, **not the planner**).
**Plan-mode draft path (this file):** `/root/.claude/plans/sharded-discovering-stearns.md`.
**Chunk ID:** CH-13 (forward-scope §1.4 "Frozen tags + audit" lines 130–135).
**Severity:** MEDIUM (audit-integrity hardening per drift `D-new-19`; not security-boundary critical, but mis-configured templates today silently downgrade audit posture).
**Expected effort:** ~1 engineer-day (matches forward-scope estimate).
**Hard prerequisites:** none (forward-scope row line 133 confirms). Soft dependencies: ADR-0049 (CH-12 just-sealed) + CH-11/ADR-0048 ApprovalMode-on-Grant pattern (we mirror its denormalisation precedent for `audit_class` on Grant). Both verified at chunk-open (see §6).
**Chunks unblocked at close:** none directly (forward-scope row line 135). Audit-integrity hardening is its own end.

---

## Forks for orchestrator

**3 forks surface; planner recommends a path on each. None requires a user-lock unless orchestrator dissents.**

### F1 — Strictest-wins ordering — should `Silent` map to concept-doc 07's `none`?

**Question.** Concept doc 07 line 68 specifies the ordering as `none < logged < alerted` (loosest → strictest). The `AuditClass` Rust enum at `audit/mod.rs:48` has variants `{Silent, Logged, Alerted}`. There is **no `None` variant**; "no audit" semantically means `Silent` (kept 30 days, no delivery; per `audit/mod.rs` doc-comment lines 30–33). The drift D-new-19 sketch (line 35) says ordering is "Logged < Alerted < Elevated" — but **no `Elevated` variant exists**; the drift's sketch is stale and concept doc 07 is authoritative.

- **F1.A — Map `Silent ↔ none` (RECOMMENDED).** Strictest-wins ordering is `Silent < Logged < Alerted`. Encoded via `derive(PartialOrd, Ord)` with the variants declared in this loosest-to-strictest order (Rust auto-derives lexicographic ordering matching declaration order; current declaration order at `audit/mod.rs:48–52 (declaration) / line 46 (derive)` already matches). Add a doc-comment cross-referencing concept-doc 07 line 68's `none` term explicitly to `Silent`.
  - *Pros:* Aligns with shipped enum + concept-doc semantics. Zero variant additions. `derive(Ord)` is the canonical Rust pattern; no manual `cmp` impl required. Variant declaration order at lines 37–39 already encodes the right ordering.
  - *Cons:* The drift sketch's "Elevated" term is left orphaned — but the sketch is acknowledged stale; concept doc is the source of truth.
- **F1.B — Add `None` variant + deprecate `Silent`.** Aligns variant names to concept-doc 07 verbatim.
  - *Pros:* Verbatim concept-doc parity.
  - *Cons:* +1 enum variant + cascade across 12+ existing emitters that hardcode `AuditClass::Silent` (e.g., `m5/agent_catalog.rs:49`). +1 SurrealDB migration to extend the `audit_class` enum check (`migrations/0001_initial.surql` ASSERT clause). Auto-approval criteria fail (new migration). Out of scope for an audit-integrity hardening chunk.
- **F1.C — Manual `Ord` impl mapping Silent→0, Logged→1, Alerted→2 with explicit `match` body.** No derive.
  - *Pros:* Explicit control over ordering bytes.
  - *Cons:* Equivalent to F1.A but more code. The serde `rename_all = "snake_case"` already handles wire format; ordering is comparison-only and `derive` is correct.

**Planner recommendation: F1.A.** Auto-approvable via direct gate (no migration, no variant add, zero phi-core leverage delta). If orchestrator agrees, no AskUserQuestion needed.

### F2 — Should `audit_class` be denormalised onto `Grant` at issuance?

**Question.** Concept doc 07 line 70 says: *"The resolved `audit_class` is recorded on the Grant at issuance time, along with which of (a)/(b)/(c) supplied the winning value."* Today `Grant` (at `model/nodes.rs:650`) does NOT carry `audit_class` — only `AuthRequest` (line 820) and `Organization.audit_class_default` (line 384) do. Drift D-new-19 line 19 says: *"Composed result recorded on Grant at issuance."*

- **F2.A — Add `audit_class: AuditClass` field to `Grant` (RECOMMENDED).** Mirrors the CH-11 ADR-0048 D48.1 precedent of adding `approval_mode: ApprovalMode` directly to Grant. `#[serde(default)]` shields pre-CH-13 grants (decode as `AuditClass::Silent`, the loosest — preserves "no escalation by silent migration" invariant per concept-doc 07 line 72). The provenance-attribution sub-question ("along with which of (a)/(b)/(c) supplied the winning value") is recorded in the **audit event's diff** (not on Grant), since the `template.X.grant_fired` events are the operator-facing trace per concept-doc 07 line 70 second clause.
  - *Pros:* Concept-doc faithful (line 69 verbatim: "recorded on the Grant"). Mirrors ADR-0048 D48.1 Grant-denormalisation precedent. Pre-CH-13 grants decode safely as `Silent` via serde-default. No migration (Grant rows in SurrealDB use `FLEXIBLE TYPE object` for governance fields per `migrations/0001_initial.surql`).
  - *Cons:* +1 Grant field; cascade to fire-fn `Grant { ... }` literal-struct sites (3 sites confirmed via grep — see §3 cascade evidence). Audit-event diff serialises both the resolved class AND the winning-source `(org / template_ar / override)` triple.
- **F2.B — Stamp `audit_class` only on the audit event, not on Grant.** Composer fn returns `(AuditClass, WinningSource)`; listener feeds the resolved class into the audit-event builder; Grant stays unchanged.
  - *Pros:* Smaller diff (no Grant field add; no cascade).
  - *Cons:* Concept-doc 07 line 69 says "recorded on the Grant" verbatim — F2.B leaves this contradicted. Future reviewers querying "what audit class does this grant carry?" must join Grant→audit_event by grant_id. Misaligned with the ADR-0048 ApprovalMode precedent.
- **F2.C — Both (Grant + dedicated provenance struct).** Grant carries `audit_class` AND a `audit_class_provenance: AuditClassProvenance` enum naming the winning input.
  - *Pros:* Maximally explicit.
  - *Cons:* +1 Grant field +1 enum +1 cascade × 3 fire-fns. Provenance is operator-facing observability, not engine-relevant — belongs in audit-event diff, not Grant. Over-modeling.

**Planner recommendation: F2.A.** Concept-doc-faithful + ADR-0048-precedent-aligned. The provenance triple (which input won) is in the audit-event diff per F2.A's second clause, where operators read it.

### F3 — Composer fn signature — `Option<AuditClass>` for override or required input?

**Question.** Concept doc 07 lines 66–67 phrase: *"three potential sources of `audit_class` can apply: (a) the org-level default, (b) the template adoption Auth Request's `audit_class`, and (c) any per-Grant override the template specifies."* The "any per-Grant override" wording implies optional; templates A/C/D today don't supply an override at all, while Template E's `BuildArgs::audit_class` (`templates/e.rs:66`) supplies one. The drift sketch (D-new-19 line 35) signature is `compose_audit_class(org_default, template_ar, override) -> AuditClass`.

- **F3.A — `compose_audit_class(org_default: AuditClass, template_ar: AuditClass, override: Option<AuditClass>) -> AuditClass` (RECOMMENDED).** Override is optional; absent override means "use strictest of (a)+(b)". The composer body: `let candidates = [Some(org_default), Some(template_ar), override]; candidates.into_iter().flatten().max().expect("at least 2 candidates always present")`.
  - *Pros:* Mirrors concept-doc-07-line-66 "any" wording. Templates A/C/D pass `None` for override (current absence is honoured); Template E's adoption AR passes `Some(adopter_chosen_class)` for the org-creation flow. Override semantics ("can only escalate, not loosen") are enforced at fold time: any `override < max(org_default, template_ar)` is silently rejected (the strictest wins by `Ord`); any `override ≥ max(...)` becomes the result.
  - *Cons:* The "override can only escalate" property (concept-doc 07 line 69) is implicit (a `min(override, ...)` would loosen, but `max(...)` always escalates). Add a doc-test asserting the property explicitly.
- **F3.B — `compose_audit_class(org_default, template_ar, override: AuditClass) -> AuditClass`.** Override is required; callers without an override pass the loosest (`Silent`) explicitly.
  - *Pros:* Single uniform signature.
  - *Cons:* Forces every fire-fn callsite to invent an "absent override" sentinel. `Silent` would be the loosest sentinel, but that's semantically incorrect — there's a difference between "template specified Silent override" (intentional) and "template specified no override" (absent). F3.A's `Option` shape preserves this distinction.
- **F3.C — Fluent builder `AuditClassComposer::new(org).template(tpl).override_(opt).resolve() -> AuditClass`.**
  - *Pros:* Self-documenting at callsite.
  - *Cons:* Over-engineering for a 3-input pure-fn. Rust convention favours direct fn signatures for math-shaped folds.

**Planner recommendation: F3.A.** `Option`-typed override aligns with concept-doc 07's "any" wording and matches Rust idiom for "absent input" semantics.

---

## Context

### The simple version

Concept doc 07 (`permissions/07-templates-and-tools.md` §"audit_class Composition Through Templates" lines 64–72) specifies a **strictest-wins** composition rule: when a template fires a Grant, the resolved `audit_class` is the strictest of (a) the **org's `audit_class_default`**, (b) the **template adoption Auth Request's `audit_class`**, and (c) any **per-Grant override** the template specifies. Per-Grant overrides may **only escalate, never loosen** — an org with `audit_class_default: alerted` for compliance reasons must never see a template silently downgrade its audit posture.

Today the rule is **silent in code** (drift D-new-19, MEDIUM, `discovered`). Templates A/C/D's `fire_grant_*` pure-fns mint Grants with **no `audit_class` field at all**. The companion audit events (`template.a.grant_fired` at `audit/events/m4/templates.rs:30` (post-CH-13: 23 pre-chunk), `template.c.grant_fired` at `m5/templates.rs:29` (post-CH-13: 22 pre-chunk), `template.d.grant_fired` at `m5/templates.rs:77` (post-CH-13: 61 pre-chunk)) hardcoded `AuditClass::Logged` pre-CH-13; post-CH-13 they accept the composed class as a parameter. So an org configured as `alerted` for compliance reasons used to see its template-fired grant audits routed at `Logged` — silently downgraded — pre-CH-13.

CH-13 closes drift D-new-19 by:
1. **Pure-fn composer.** Ship `domain::permissions::audit_composition::compose_audit_class(org_default, template_ar, override) -> AuditClass` (per F3.A locked-recommendation). Add `derive(PartialOrd, Ord)` to `AuditClass` with declaration order `Silent < Logged < Alerted` (per F1.A).
2. **Grant denormalisation.** Add `audit_class: AuditClass` field to `Grant` per F2.A (mirrors ADR-0048 D48.1 ApprovalMode precedent). `#[serde(default)]` shields pre-CH-13 grants → decode as `Silent`.
3. **Wire into grant-mint pipeline.** The 3 fire pure-fns (`fire_grant_on_lead_assignment` at `templates/a.rs:93`, `fire_grant_on_manages_edge` at `templates/c.rs:74`, `fire_grant_on_has_agent_supervisor` at `templates/d.rs:69`) gain the inputs needed to call the composer; the 3 fire listeners (`TemplateAFireListener`, `TemplateCFireListener`, `TemplateDFireListener` at `events/listeners.rs:143/291/402`) supply `org_default` (via repo lookup) and `template_ar` (already in scope as the adoption AR). The 3 audit-event builders' hardcoded `Logged` becomes the composed-and-stamped class from the Grant.
4. **Acceptance tests** — pure-fn truth-table (`Silent × Logged × Alerted` × `None | Silent | Logged | Alerted`) covering the documented escalation properties + 3 listener-level integration tests confirming the composed class flows from Grant → audit event.

The chunk's deliverables map directly to forward-scope row 134's two clauses ("`compose_audit_class(...)` using AuditClass ordering"; "wired into grant-mint"). The audit-integrity hardening is the chunk's only end (forward-scope row 135 confirms unblocks: nothing).

### What this chunk does NOT do

- Does NOT introduce a new `AuditClass` variant. F1.A locks to `Silent < Logged < Alerted` — concept-doc 07 line 68's `none` term maps to `Silent` semantically.
- Does NOT change SurrealDB schema. The `audit_events` table's `audit_class` column already accepts `"silent" | "logged" | "alerted"` per `migrations/0001_initial.surql`. Grant's serialisation uses `FLEXIBLE TYPE object` for governance fields; adding a struct field is schema-stable. Migration count stays at **13** (post-CH-12 baseline).
- Does NOT touch Template E's `BuildArgs::audit_class` field. Template E is the self-interested-auto-approve path used at platform-admin / page-02–05 writes; its caller already supplies the audit class at construction time. Composing on top would require the caller to know the org's default, which is two layers above current callers. Out of scope; would require a separate forward-scope row. (The drift D-new-19 §Where-visible names "template builders" and "Grant mint path" — the M4/M5 fire pure-fns ARE the grant-mint path; Template E's adoption AR builder is NOT a grant minter.)
- Does NOT change ApprovalMode behaviour. ApprovalMode (CH-11/ADR-0048) and `audit_class` (CH-13) are **orthogonal Grant fields** — denormalisation precedent is shared, but engine-Step semantics are independent.
- Does NOT extend the audit-event diff to encode the **winning-source attribution** in a typed enum. Concept-doc 07 line 69 says "along with which of (a)/(b)/(c) supplied the winning value" — CH-13 honours this minimally by extending the `template.X.grant_fired` audit-event diff with a string field `audit_class_source: "org_default" | "template_ar" | "override"` (the loosest contract that satisfies the concept doc); a richer typed `AuditClassProvenance` enum is a forward-scope item if operator-facing tooling needs to query it programmatically.

### Forward-scope reference

[CH-13 row](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (lines 130–135) + §1.4 "Frozen tags + audit" block (lines 121–135).

### Concept-doc anchor

- [`concepts/permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) §"audit_class Composition Through Templates" (lines 64–72).
- [`concepts/permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) (entry invariants for permissions subtree).
- [`docs/specs/v0/requirements/cross-cutting/nfr-observability.md`](baby-phi/docs/specs/v0/requirements/cross-cutting/nfr-observability.md) (event-class retention rules — the ordering's operator-meaning anchor).

---

## §1 — Why this chunk

Concept doc 07 lines 64–72 specifies a strictest-wins composition rule for `audit_class` whenever a template fires a Grant: the resolved class is the strictest of (org-default, template-AR, per-grant-override), and per-grant overrides may only escalate. Today, the rule is silent in code — every `template.X.grant_fired` audit event hardcodes `AuditClass::Logged` (verified via grep at `audit/events/m4/templates.rs:53`, `m5/templates.rs:48,89`), regardless of the issuing org's `Organization.audit_class_default` (the field at `model/nodes.rs:384` exists but is read nowhere in the firing path). An operator-facing pathology results: an org configured `audit_class_default: alerted` for regulatory compliance silently sees template-fired grants audited at `Logged`, defeating the operator's compliance posture.

CH-13 closes drift D-new-19 (MEDIUM, audit-integrity hardening) by shipping a pure-fn composer (`compose_audit_class(org_default, template_ar, override) -> AuditClass`), denormalising the resolved class onto Grant per concept-doc 07 line 70 verbatim ("recorded on the Grant"), and wiring the composer into all 3 production grant-mint paths (Template A/C/D fire listeners). The deliverables map 1:1 to forward-scope row 134's two clauses. No K8s blockers (audit hash chain stays byte-stable per A7 review — see §3.B). Zero phi-core leverage delta (no overlap with phi-core types). No SurrealDB migration.

**Quality-over-speed restatement.** *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* CH-13's specific application: every assertion below cites a verbatim concept-doc line, every fork is recommendation-locked above for orchestrator gating, and the deferred items (Template E composer integration; richer typed `AuditClassProvenance` enum) ship with explicit out-of-scope notes — not silent gaps.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) | §"audit_class Composition Through Templates" line 68 | "Strictest wins. The effective `audit_class` is the strictest of (a), (b), and (c). The ordering from loosest to strictest is `none < logged < alerted`." | silent-in-code (no composer fn; templates hardcode `Logged`) | honored — `compose_audit_class(...)` ships with `Silent < Logged < Alerted` `Ord` derive (concept-doc `none` maps to enum `Silent` per F1.A); 3 fire listeners call it |
| `permissions/07-templates-and-tools.md` | §"audit_class Composition Through Templates" line 69 | "Per-Grant overrides may only go stricter. A template's per-Grant override can escalate to a stricter class but cannot loosen." | silent-in-code (no override mechanism today) | honored — `compose_audit_class(...)`'s `max`-fold property ensures any override below the (a)+(b) max is silently rejected (the max wins); explicit doc-test pins the property |
| `permissions/07-templates-and-tools.md` | §"audit_class Composition Through Templates" line 70 | "Operators can always see what was applied. The resolved `audit_class` is recorded on the Grant at issuance time, along with which of (a)/(b)/(c) supplied the winning value." | silent-in-code (Grant has no `audit_class` field; `model/nodes.rs:650`) | honored — Grant gains `audit_class: AuditClass` field per F2.A; audit-event diff gains `audit_class_source` string field naming the winning input |
| `permissions/07-templates-and-tools.md` | §"audit_class Composition Through Templates" line 72 | "An org that opts into `alerted` for compliance reasons is guaranteed that adopting a template can never silently downgrade its audit posture." | contradicted (current implementation hardcodes `Logged`; Alerted-default org sees Logged-class template grants — silent downgrade) | honored — composer's `max(org_default, template_ar, override)` makes downgrade a structural impossibility; pure-fn truth-table test pins the invariant |
| [`permissions/README.md`](baby-phi/docs/specs/v0/concepts/permissions/README.md) | (entry invariants) | Permissions subtree invariants | honored | honored (re-verified post-composer integration) |
| [`concepts/phi-core-mapping.md`](baby-phi/docs/specs/v0/concepts/phi-core-mapping.md) | (phi-core surfaces) | phi-core has no governance / audit-composition concept | honored | honored (unchanged — CH-13 stays in baby-phi `domain::audit` + `domain::permissions` + `domain::events`) |
| [`docs/specs/v0/requirements/cross-cutting/nfr-observability.md`](baby-phi/docs/specs/v0/requirements/cross-cutting/nfr-observability.md) | §event-class retention | "`Silent` — kept 30 days, no delivery; `Logged` — kept 365 days, logged to structured sink; `Alerted` — kept 365+ days, delivered to alert channel within 60 s" | honored (the `AuditClass` enum doc-comment at `audit/mod.rs:30–33` mirrors the requirement verbatim) | honored — composer respects the ordering implied by retention (longer retention = stricter) |

**Coverage check.** Every concept doc whose claims this chunk's code touches is listed. `permissions/README.md` cited per the per-chunk-template's "Permissions subtree hook". `concepts/phi-core-mapping.md` cited per the "phi-core-mapping hook". `nfr-observability.md` cited because the ordering's operator-semantics anchor on retention durations is its source-of-truth.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | (no overlap) | N/A | — CH-13 surface is wholly inside baby-phi `domain::audit` + `domain::permissions` + `domain::events`. `AuditClass` is a baby-phi-native enum (no phi-core counterpart per `phi-core-mapping.md`'s "Orthogonal surfaces" §). The composer fn lives at `domain::permissions::audit_composition` (new module). |

**Expected import-count delta at chunk close: 0 phi-core imports added.** CH-13 changes are local to `domain::audit` (Ord derive on `AuditClass`), `domain::permissions::audit_composition` (new module — pure-fn composer), `domain::model::nodes` (Grant.audit_class field add), and `domain::events::listeners` (composer-wiring at 3 listener bodies). None overlap with phi-core types.

**Positive close-audit greps** (the post-chunk audit will run these):

```bash
# Composer fn ships at the expected path and is exported.
grep -rn "pub fn compose_audit_class" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l
# Expect: 1 (the fn definition).

# Grant struct gains audit_class field (mirrors approval_mode precedent).
grep -nE "pub audit_class: AuditClass" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs
# Expect: ≥ 2 hits — line 384 (Organization.audit_class_default), line 790 (AuthRequest.audit_class), and a NEW line in Grant.

# 3 fire listeners now look up org_default + invoke composer.
grep -nE "compose_audit_class\(" /root/projects/phi/baby-phi/modules/crates/domain/src/events/listeners.rs | wc -l
# Expect: 3 (TemplateAFireListener, TemplateCFireListener, TemplateDFireListener).

# Audit-event builders are parametrised by AuditClass (no longer hardcoded).
grep -nE "audit_class:\s*AuditClass::Logged\b" /root/projects/phi/baby-phi/modules/crates/domain/src/audit/events/m4/templates.rs /root/projects/phi/baby-phi/modules/crates/domain/src/audit/events/m5/templates.rs | wc -l
# Expect: 0 in production paths (test fixtures may keep specific values; the count above is for the production builder bodies — the seal-time auditor inspects the diff to confirm Logged-as-default is replaced by parameter-passthrough).
```

**Forbidden-duplication greps** (must return 0):

```bash
# No second AuditClass enum sneaking into a sub-module.
grep -rn "^pub enum AuditClass\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "audit/mod.rs"
# Expect: 0.

# No `enum AuditClass` defined inside CLI / server (the CLI's AuditClassArg is the only legitimate parallel; not a duplicate).
grep -rn "^enum AuditClass\b" /root/projects/phi/baby-phi/modules/crates/cli/ /root/projects/phi/baby-phi/modules/crates/server/
# Expect: 0.

# scripts/check-phi-core-reuse.sh stays green.
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
# Expect: 0 violations.
```

**Cascade evidence (per CH-11/CH-12 retro discipline — literal-struct fan-out from F2.A's Grant.audit_class field add).** The Grant literal-construction sites that need the new field:

```bash
git grep -nE 'Grant\s*\{$' /root/projects/phi/baby-phi/modules/crates/
```

Predicted matched-line count: **3 production sites** (`templates/a.rs:100`, `templates/c.rs:81`, `templates/d.rs:77` — each `Grant { id, holder, action, ... }` literal in a `fire_grant_*` pure-fn). Plus test-fixture sites that may construct synthetic Grants directly. The `#[serde(default)]` attribute on the new field shields any deserialisation path that doesn't construct via literal (legacy stored-row decoding). Pause discipline: if actual cascade > **5 sites** (1.67× predicted upper bound — asymmetric ×1.0–×1.20 buffer per CH-11/CH-12 cycle data extended for struct-cascade discipline), AskUserQuestion before continuing P2 — could indicate test-fixture Grant constructions we missed.

**Additive-enum cascade discipline note (per CH-12 retro Row 4).** CH-13 does NOT add an `AuditClass` variant (F1.A locks to existing `{Silent, Logged, Alerted}`). The additive-enum cascade rule (catch-all check + 0 callsite-edit prediction when ≥ 80% catch-all dominance) is **not applicable** to this chunk. Confirmed.

Per [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core Leverage" rules 1–5. `scripts/check-phi-core-reuse.sh` MUST stay green at chunk close.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`, `RwLock`, `AtomicBool`, etc.) | Composer is a pure fn — no state. Grant's new `audit_class` field is per-row data on durable storage, not in-process. Listener bodies remain stateless. | no | — |
| **A2** | New IPC channel (`mpsc`, `broadcast`, `oneshot`, `watch`, `Notify`) | None added. Listener bodies still consume from the existing `EventBus` — no new channels. | no | — |
| **A3** | New pod-local resource (file handle, listener socket, sub-process, lock file, on-disk cache) | None. Composer is pure; Grant field is durable; audit-event-builder change is shape-only. | no | — |
| **A4** | Migration runner / first-apply race | **No migration added.** Grant's new field is `#[serde(default)]`-shielded; SurrealDB stores Grant rows in a `FLEXIBLE TYPE object` shape (per `migrations/0001_initial.surql`); audit-events table accepts string `audit_class` values already (Silent / Logged / Alerted); no enum-extension migration needed. Migration count stays at **13**. | no | — |
| **A5** | Trait-shape requirement (trait-objects-friendly for future broker / Redis swap) | Composer is `pub fn`, not a trait. Listener bodies still use the `Repository: Send + Sync` trait + `AuditEmitter` trait — no new shape requirements. | no | — |
| **A6** | Cross-pod state sharing (data visible across pods) | The org's `audit_class_default` lookup the listener now performs is a **read** against the durable `Repository::get_organization`-style call (already trait-object-dispatched). Composed class is stamped onto Grant + audit-event — both durably written to SurrealDB and visible across pods via the `open_remote` path (ADR-0033 D33.2). | no | — |
| **A7** | Audit hash-chain symmetry (single-writer guarantee; new audit writer breaking chain) | **The chunk does NOT add a new audit-event writer.** It changes the `audit_class` field value on existing `template.X.grant_fired` events from a hardcoded `Logged` to a composed value, and adds an `audit_class_source` string field to the diff. `AuditEvent::canonical_bytes()` (`audit/mod.rs:84`) DOES include `audit_class` (line 70) and `diff` (line 69) — meaning post-CH-13 audit events have different `canonical_bytes` than pre-CH-13 ones for the same logical firing. **However**: the hash chain is **forward-only** within an `org_scope` (line 77 `prev_event_hash` doc); pre-existing events' bytes are unaffected; new events at any pod simply produce new chain links with the composed class. Cross-pod: every pod sees the same durable `Organization.audit_class_default` + `AuthRequest.audit_class` reading the composer's inputs (A6) → every pod produces the same composed class for the same firing. **Single-writer guarantee preserved.** | no | — |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — chunk does NOT touch the registry.
- D33.2 (`SurrealStore::open_remote`) — chunk's storage operations (Grant create with new field, audit-event insert with new diff key) work identically on `open_embedded` and `open_remote` (both go through `Repository::create_grant` + `AuditEmitter::emit`; no new storage surface).
- D33.3 (SIGTERM graceful shutdown) — chunk does NOT add new `tokio::spawn` tasks.
- D33.4 (`EventBus.shutdown` + `drain`) — chunk does NOT add new `EventBus` emitters or listeners (extends 3 existing listener bodies in-place).

**Conclusion paragraph.** *"K8s-neutral"*. No new blockers introduced. The audit-class composition runs inside existing pod-local listener bodies that read durable cross-pod state (Organization.audit_class_default + AuthRequest.audit_class) and write durable cross-pod outputs (Grant + AuditEvent). Hash-chain symmetry preserved per A7 (no new writer; canonical_bytes change is forward-only and deterministic across pods).

**Mid-flight discovery.** If a phase surfaces a K8s blocker not anticipated above, pause via `AskUserQuestion` and add a new ledger entry before the phase closes — identical pattern to §2 concept-contradiction discovery.

---

## §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/audit-class-composition.md` (new file expected) | yes — design + composer fn signature + winning-source attribution rule + 3 listener wiring points | (a) update in-chunk — ship the new architecture doc as part of P3 (seal phase) deliverables. Cross-references ADR-0050. |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/audit-class-composition-operations.md` (new file expected) | yes — operator playbook explaining how to verify the org's audit posture is honoured: the new `audit_class_source` diff field surfaces in audit-event queries; the strictest-wins property is testable end-to-end via `phi org create --audit-class-default alerted` followed by Template-A firing and inspecting the resulting `template.a.grant_fired` event | (a) update in-chunk — ship as part of P3 seal. |
| **User-guide** | nearest existing `cli-reference-mN.md` walkthrough for `phi org create` (P0 fallback decision required — `m5_2/user-guide/` may not exist; planner verified `m5_2/architecture/` + `m5_2/operations/` exist but did NOT verify `user-guide/` tier — implementer P0 task) | partial — the existing `phi org create` CLI already accepts `--audit-class-default` (`cli/src/commands/org.rs:52`). No new CLI flags are added by CH-13 — the composer is internal. The user-guide may want a one-paragraph note in the org-create walkthrough explaining the cascading guarantee. | (a) update in-chunk — append a one-paragraph note. If `m5_2/user-guide/` does not exist, fall back to `m5/user-guide/` or `m4/user-guide/`. If neither, defer to the M5-tag-close batch with explicit successor-marker `M5-tag-close`. |

**Files to verify exist at P0 (planner could not pre-confirm without writing to disk; implementer Step 0 task):**

```bash
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/architecture/ \
   /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/operations/ \
   /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/user-guide/ 2>/dev/null
```

**Mid-flight discovery.** If a phase makes a doc stale that wasn't anticipated in §3.C, pause via `AskUserQuestion` and add a row to the table before the phase closes — same pattern as §2 + §3.B.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-19` | [`baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-19.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-19.md) | MEDIUM | `discovered` → `remediated` | All three remediation aspects ship: composer fn (§7 P1), Grant denormalisation (§7 P1), 3-listener wiring (§7 P2). The drift sketch's stale "Elevated" term is acknowledged-stale via §1 + ADR-0050 §"Variant-naming alignment"; concept doc 07 is the source of truth. |

**Concept-audit matrix update at seal (P3):** [`m5_1/drifts/_concept-audit-matrix.md`](baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md) line 215 — `audit_class composition: strictest wins | Org / template AR / override composition | silent-in-code | no composition logic | D-new-19 | N/A` — the Status column flips from `silent-in-code` to **`honored`** per the per-chunk-template P4 paperwork addendum (v2026-05-04, CH-12 retro Row 1: copy-pasted letter-for-letter from §2 row 1's chunk-close target). The "Code evidence" column updates to cite `permissions/audit_composition.rs:NN` (composer fn) + `events/listeners.rs:NNN` (3 wiring points). The "Covering drift" column updates to `D-new-19 (remediated 2026-05-XX via CH-13)`.

---

## §5 — ADRs drafted

**ADR number assignment.** The current highest ADR is **ADR-0049** (CH-12 — at `docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md`, `Accepted` 2026-05-04). Next-free is **ADR-0050**. Verified at chunk-open via `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | grep -oE '00[0-9]+' | sort -un | tail -5` returning `0045 0046 0047 0048 0049`.

**ADR-0050 — `audit_class` strictest-wins composition (composer fn + Grant denormalisation + listener wiring).**
- **Drafted-at-phase:** P1 (along with the composer fn body + Grant field add).
- **Status at draft:** `Proposed`.
- **Flip to `Accepted`:** P3 (chunk seal).
- **Path:** `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0050-audit-class-composition-strictest-wins.md` (new file, m5_2 tier consistent with ADR-0049).
- **Decision summary (one-line):** Compose `audit_class` strictest-wins of (org-default, template-AR, optional per-grant override) via pure fn `compose_audit_class(...)` returning `AuditClass` with `Silent < Logged < Alerted` `Ord` derive; denormalise resolved value onto `Grant.audit_class` per concept-doc 07 line 70; wire into 3 production grant-mint listener bodies; concept-doc `none` term maps to enum `Silent`.
- **Sub-decisions** (mirrors ADR-0049's D49.1–D49.7 numbering precedent for chunk-spanning ADRs):
  - **D50.1** — Variant-naming alignment: enum `AuditClass::Silent` ≡ concept-doc-07 line-68 `none`. The drift D-new-19 sketch's "Elevated" term is acknowledged-stale; concept doc is canonical.
  - **D50.2** — Ordering encoding: `derive(PartialOrd, Ord)` with declaration order `Silent → Logged → Alerted` (already at `audit/mod.rs:48–52 (declaration) / line 46 (derive)`); no manual `cmp` impl.
  - **D50.3** — Composer fn signature: `compose_audit_class(org_default: AuditClass, template_ar: AuditClass, override: Option<AuditClass>) -> AuditClass` (per F3.A locked-recommendation). Body is `[Some(a), Some(b), c].into_iter().flatten().max().expect(...)`. Tie-breaker rule when 2 inputs tie at strictest: `Override > TemplateAr > OrgDefault` (per-grant override is most specific intent; encoded by ordering the source enum + folding via `max_by`).
  - **D50.4** — Override-can-only-escalate property: structural via `max`-fold; explicit doc-test pins concept-doc-07 line-69 invariant.
  - **D50.5** — Grant denormalisation: `audit_class: AuditClass` field on Grant; `#[serde(default = "Grant::default_audit_class")]` shielding produces `AuditClass::Silent` (loosest) for pre-CH-13 grant rows. Mirrors ADR-0048 D48.1 `approval_mode` precedent.
  - **D50.6** — Audit-event diff extension: `audit_class_source: String` ("org_default" | "template_ar" | "override") added to `template.X.grant_fired` events' diff per concept-doc-07 line 70 second clause. String (not typed enum) is the loosest contract; richer typed `AuditClassProvenance` enum is a forward-scope item.
  - **D50.7** — Hash-chain integrity: A7 review confirms canonical_bytes change is forward-only and cross-pod deterministic; no migration; chain symmetry preserved.

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-04** (typed actions per ADR-0027) | `Action::*` enum stable; `Action::Read / Inspect / List` exported from `permissions/action.rs` | `grep -nE "Action::(Read\|Inspect\|List)" modules/crates/domain/src/permissions/action.rs` — expect ≥ 3 hits |
| **CH-09** (consent-policy frozen on Org per ADR-0046) | `Organization.consent_policy` field stable + `Organization.audit_class_default` field stable | `grep -n "pub audit_class_default" modules/crates/domain/src/model/nodes.rs` — expect 1 hit at line 384 |
| **CH-11** (ApprovalMode-on-Grant per ADR-0048 D48.1) | `Grant.approval_mode: ApprovalMode` field on Grant; precedent for CH-13's audit_class denormalisation | `grep -nE "pub approval_mode: ApprovalMode" modules/crates/domain/src/model/nodes.rs` — expect 1 hit (currently line 675); confirms ADR-0048 D48.1 `#[serde(default)]` pattern still in place |
| **CH-12** (frozen-session-tag immutability per ADR-0049) | The 3 fire listeners' bodies (TemplateAFireListener, TemplateCFireListener, TemplateDFireListener) unchanged in shape; pure-fn `validate_tag_write_on_session` unrelated to CH-13 surface | `grep -nE "TemplateAFireListener\|TemplateCFireListener\|TemplateDFireListener" modules/crates/domain/src/events/listeners.rs` — expect ≥ 3 hits each (struct + impl + EventHandler impl) |
| **M5/P3** (Template C/D fire pure-fns per ADR-0028) | `fire_grant_on_lead_assignment`, `fire_grant_on_manages_edge`, `fire_grant_on_has_agent_supervisor` exist and return `Grant` directly (CH-13 extends their `FireArgs` to carry composed audit_class — does NOT change return type) | `grep -nE "^pub fn fire_grant_on" modules/crates/domain/src/templates/*.rs` — expect 3 hits at `a.rs:93`, `c.rs:74`, `d.rs:69` |
| **M3/P2** (adoption AR's `audit_class: AuditClass::Alerted`) | Adoption ARs hardcode `AuditClass::Alerted` at `templates/adoption.rs:109`; CH-13 reads `AuthRequest.audit_class` from this AR and uses it as the `template_ar` composer input | `grep -n "audit_class: AuditClass::Alerted" modules/crates/domain/src/templates/adoption.rs` — expect 1 hit at line 109 |

**Regression posture statement.** All 6 upstream invariants verified green at chunk-open via the commands above (planner ran each before publishing this plan; cf. citation-freshness P4 paperwork rule from CH-12 retro Row 7). Re-run at chunk-seal in §12.

**Conditional re-spawn re-verification (per CH-12 retro Row 2).** This is iter 1 (initial planner spawn). If orchestrator escalates a fork to user-lock and the user diverges from the planner's recommendation, the iter-2 re-spawn MUST re-run the auto-approval criteria checklist on the user-locked path (migration delta, K8s axes, scope ratio, phi-core leverage delta, audit envelope, confidence) and state the re-verdict in an iter-2 banner before publishing the revised plan.

---

## §7 — Phases within the chunk

### P0 — Chunk-open ritual (≤ 30 min)

- **Goal.** Confirm reading-list complete; verify §6 prior-chunk invariants green; verify §3.C target doc files exist (or fall back to nearest tier); generate cycle folder via `chunk-archive-plan` skill (orchestrator-driven, not planner).
- **Deliverables.**
  1. Cycle folder created at `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/` (orchestrator + implementer; planner only writes the plan-mode draft).
  2. P0 verification log appended to plan as a §"Cross-cycle handoff" pre-block: §6 commands run + outputs pasted, §3.C `ls` run + tier-fallback decision recorded, baseline workspace test count confirmed (1345 from chunk-open `cargo test --workspace`).
  3. ADR-0050 file scaffold created (header + Status: Proposed; body filled at P1).
- **Tests.** None (scaffold-only).
- **Concept-alignment check.** None (scaffold-only).
- **phi-core leverage check.** Run baseline `bash scripts/check-phi-core-reuse.sh` — confirm green pre-touch.
- **User-facing doc updates.** §3.C tier-fallback decision recorded.
- **Confidence target.** 100% (scaffold).
- **Pause discipline.** AskUserQuestion if §3.C reveals neither `m5_2/user-guide/` nor `m5/user-guide/` exists (would force a 3-way decision: create-new vs defer-to-tag-close vs different-tier-name).

### P1 — Composer fn + Grant denormalisation + ADR-0050 body

- **Goal.** Ship the pure-fn `compose_audit_class(org_default, template_ar, override) -> AuditClass`, add `derive(PartialOrd, Ord)` to `AuditClass`, denormalise `audit_class: AuditClass` onto `Grant`, draft ADR-0050 body. **No listener wiring yet** (P2 owns that). At end of P1, `cargo test --workspace` must remain green (the new field is `#[serde(default)]`-shielded so existing Grant constructions still compile).
- **Deliverables.**
  1. `modules/crates/domain/src/audit/mod.rs` — extend `derive(...)` on `AuditClass` (line 34) to add `PartialOrd, Ord`. Add a doc-comment on the variant ordering noting concept-doc-07 line-68 `none ↔ Silent` mapping (per F1.A / D50.1).
  2. `modules/crates/domain/src/permissions/audit_composition.rs` (NEW file) — pure-fn `pub fn compose_audit_class(org_default: AuditClass, template_ar: AuditClass, r#override: Option<AuditClass>) -> AuditClass` per D50.3 + helper enum `pub enum AuditClassSource { OrgDefault, TemplateAr, Override }` for the second-clause attribution + companion `pub fn compose_audit_class_with_source(...) -> (AuditClass, AuditClassSource)` returning the (resolved, winning-source) tuple.
  3. `modules/crates/domain/src/permissions/mod.rs` — register the new `audit_composition` sub-module.
  4. `modules/crates/domain/src/model/nodes.rs` — add `pub audit_class: AuditClass` field to `Grant` struct (after line 675's `approval_mode` per the F2.A precedent-mirroring placement); `#[serde(default = "Grant::default_audit_class")]` shielding (default to `AuditClass::Silent` — loosest = "no escalation by silent migration" property).
  5. **Cascade fix-ups (predicted 3 sites; pause if > 5).** `templates/a.rs:100`, `templates/c.rs:81`, `templates/d.rs:77` Grant literal-struct sites add `audit_class: AuditClass::Silent` placeholder (P2 replaces with composer call). The per-pure-fn `FireArgs` struct gains a temporary `pub audit_class: AuditClass` field at P1 (P2 wires it from the listener; for P1 the existing test-only `fire_args()` helpers pass `AuditClass::Silent`).
  6. ADR-0050 body filled at `m5_2/decisions/0050-audit-class-composition-strictest-wins.md` (Status: Proposed at P1; flips to Accepted at P3).
- **Tests.** New unit + property tests in `audit_composition.rs`:
  - `unit_silent_loosest()` — `AuditClass::Silent < AuditClass::Logged && AuditClass::Logged < AuditClass::Alerted` (D50.2).
  - `unit_compose_returns_strictest_of_three()` — table-driven over 27 input combos × `Option<override>` (concept-doc-07 line 68 truth-table).
  - `unit_override_can_only_escalate()` — for every `(org, tpl)` pair, asserting that `override < max(org, tpl)` ⇒ result = `max(org, tpl)`; `override ≥ max(org, tpl)` ⇒ result = `override` (D50.4 concept-doc-07 line-69 invariant).
  - `unit_compose_with_source_attributes_winning_input()` — for each of `(OrgDefault, TemplateAr, Override)` winning, asserts the returned source matches.
  - `unit_compose_with_source_tie_breaker()` — when 2 inputs tie at the strictest, Override wins over TemplateAr wins over OrgDefault (D50.3 tie-breaker).
  - `unit_grant_serde_default_audit_class_is_silent()` — round-trip a serialised pre-CH-13 Grant JSON (no `audit_class` key) and assert `Grant::default_audit_class() == AuditClass::Silent`.
  - `unit_audit_class_ord_derive_property()` — proptest 50 cases over arbitrary `Vec<AuditClass>`, asserting `vec.iter().max() == Some(&AuditClass::Alerted)` whenever the vec contains Alerted.
  - **Predicted P1 test count: 7 new tests.**
- **Concept-alignment check.** §2 row 1 (line 68 ordering) → `silent-in-code` → `honored` for the **fn-level** axis (the **listener-level** axis flips at P2). §2 row 2 (line 69 escalate-only) → `silent-in-code` → `honored` (`max`-fold structural property + doc-test). §2 row 3 (line 70 recorded-on-Grant) → `silent-in-code` → `honored` for the **field-level** axis (the **listener-level** axis flips at P2). §2 row 4 (line 72 no-silent-downgrade) → `contradicted` → still `contradicted` at P1 (listener wiring missing); flips to `honored` at P2.
- **phi-core leverage check.** Run `bash scripts/check-phi-core-reuse.sh` — expect green (no phi-core types touched). Predicted phi-core import-count delta: 0.
- **User-facing doc updates.** None at P1 (architecture + operations docs land at P3; user-guide note at P3).
- **Confidence target.** ≥ 97% composite.
- **Pause discipline.** Pause if Grant literal-struct cascade exceeds **5 sites** (1.67× predicted upper bound — CH-11/CH-12 retrospective Row 1 + asymmetric ×1.0–×1.20 buffer per CH-12 retro Row 3). Pause if `cargo test --workspace` regresses by even one test (the `#[serde(default)]` shield should preserve all existing test behaviours).

### P2 — Wire composer into 3 fire listeners

- **Goal.** The 3 production grant-mint listeners (`TemplateAFireListener`, `TemplateCFireListener`, `TemplateDFireListener`) read `Organization.audit_class_default` + the adoption-AR's `audit_class`, call `compose_audit_class_with_source(...)`, and stamp the resolved class on (a) the new Grant field, (b) the `template.X.grant_fired` audit event's `audit_class` field, and (c) the audit event's `diff.audit_class_source` string. `cargo test --workspace` remains green at P2 close.
- **Deliverables.**
  1. **`modules/crates/domain/src/repository.rs`** — extend the `Repository` trait with a getter the listener can call to fetch `Organization.audit_class_default` for the org owning the firing project. Likely already exists as `Repository::get_organization` — verify at P2 open. If absent, ship a thin `Repository::get_org_audit_class_default(org_id) -> RepositoryResult<AuditClass>` helper. (Planner did NOT verify this at plan-draft; flagged as a P2-open subtask.)
  2. **`modules/crates/domain/src/events/listeners.rs:223–256`** (TemplateAFireListener body) — between `let now = Utc::now();` (line 222) and `let grant = fire_grant_on_lead_assignment(FireArgs { ... });` (line 223), look up org_default + read adoption_ar's audit_class; call `compose_audit_class_with_source(org_default, adoption_ar.audit_class, None)`; pass the resolved class through new `FireArgs::audit_class` field. Then pass the resolved `(class, source)` to the audit-event builder.
  3. **`modules/crates/domain/src/events/listeners.rs:345–382`** (TemplateCFireListener body) — same pattern.
  4. **`modules/crates/domain/src/events/listeners.rs:457–498`** (TemplateDFireListener body) — same pattern.
  5. **Audit-event builder signature changes:**
     - `audit/events/m4/templates.rs:30–60` `template_a_grant_fired(...)` (post-CH-13 line range) — add `audit_class: AuditClass` + `audit_class_source: AuditClassSource` parameters; replaces hardcoded `AuditClass::Logged` with the parameter; extends `diff` with `"audit_class_source": <source-as-string>`.
     - `audit/events/m5/templates.rs:22–53` `template_c_grant_fired(...)` — same shape.
     - `audit/events/m5/templates.rs:61–94` `template_d_grant_fired(...)` — same shape.
  6. **Per-fire-pure-fn signature finalisation** — `fire_grant_on_lead_assignment(args)` / `fire_grant_on_manages_edge(args)` / `fire_grant_on_has_agent_supervisor(args)` — `FireArgs` now carries `pub audit_class: AuditClass` (added at P1). The pure-fn body sets `Grant.audit_class = args.audit_class`. Pure-fn discipline preserved (no I/O, no Repository).
- **Tests.** Implementer-listed:
  - **Per-listener integration test** — using the existing in-memory Repository fixtures, set the org's `audit_class_default` to `Alerted`, fire a `HasLeadEdgeCreated`, assert the resulting Grant carries `audit_class: AuditClass::Alerted` + the audit event carries `audit_class: AuditClass::Alerted` + `diff["audit_class_source"] == "org_default"`. **3 tests** (one per listener).
  - **Strictest-wins integration test** — set org's default to `Logged` and adoption-AR's `audit_class` to `Alerted` (already the production default per `templates/adoption.rs:109`), assert the resulting Grant + audit-event carry `Alerted` + `diff["audit_class_source"] == "template_ar"`. **3 tests** (one per listener).
  - **No-silent-downgrade integration test** — set org's default to `Alerted`, adoption-AR `audit_class: Alerted` (production default), confirm Grant + audit-event carry `Alerted`. **1 test** (pinning concept-doc-07 line-72 for the canonical compliance posture).
  - **Updated audit-event-builder unit tests** — the existing `template_a_grant_fired_is_logged_and_org_scoped` (`audit/events/m4/templates.rs:67`), `template_c_grant_fired_is_logged_and_org_scoped` (`m5/templates.rs:101`), `template_d_grant_fired_carries_project_scope` (`m5/templates.rs:135`) tests change shape from "asserts Logged" to "passes through composed class". **3 tests updated** (not new, but signature+body changes — counted in Pre-existing-still-green table).
  - **Predicted P2 test count: 7 new tests + 3 updated.**
- **Concept-alignment check.** §2 row 1 (line 68) — listener-level axis flips to `honored`. §2 row 3 (line 70) — listener-level axis flips to `honored` (audit-event diff carries source attribution). §2 row 4 (line 72) — flips from `contradicted` to `honored`.
- **phi-core leverage check.** `scripts/check-phi-core-reuse.sh` green; predicted import-count delta: 0.
- **User-facing doc updates.** None at P2 (docs land at P3).
- **Confidence target.** ≥ 97% composite.
- **Pause discipline.** Pause if `Repository::get_organization`-style getter doesn't exist and would require a NEW Repository trait method. Trait-method addition is a small architectural decision (touches the trait shape per A5; potentially K8s-relevant); AskUserQuestion before extending the trait surface. Pause if any audit-event-builder signature change cascades to > 8 callsites (audit-event builders are called from listeners + tests; >8 indicates a test fixture we missed).

### P3 — Seal: docs, ADR Accept-flip, paperwork, post-chunk audit prep

- **Goal.** All seal-time paperwork shipped: ADR-0050 flipped to `Accepted`; concept-audit matrix line 215 status flipped to `honored` (letter-for-letter from §2 row 1's chunk-close target — per CH-12 retro Row 1 P4 paperwork addendum); drift D-new-19 lifecycle-history entry appended; verified-headers refreshed on every touched doc; user-facing docs (architecture + operations + optional user-guide note) shipped.
- **Deliverables.**
  1. `docs/specs/v0/implementation/m5_2/decisions/0050-audit-class-composition-strictest-wins.md` — **Status: Accepted**, verified-header bump.
  2. `docs/specs/v0/implementation/m5_1/drifts/D-new-19.md` — append `## Lifecycle history` entry: *"2026-05-XX — `discovered` → `remediated` — CH-13/P3 chunk-seal (cycle hex `d4fe1b7c`); ADR-0050 Accepted."*
  3. `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` line 215 — Status column copy-pasted letter-for-letter from §2 row 1's chunk-close target (per CH-12 retro Row 1): from `silent-in-code` to `honored`. "Code evidence" cell updated to `permissions/audit_composition.rs:NN` + `events/listeners.rs:NNN-NNN`. "Covering drift" cell updated to `D-new-19 (remediated 2026-05-XX via CH-13)`.
  4. `docs/specs/v0/implementation/m5_2/architecture/audit-class-composition.md` (NEW) — design doc covering composer fn signature, ordering encoding, Grant denormalisation, listener wiring, audit-event diff extension, and the cross-pod determinism argument from §3.B A7. Header `<!-- Last verified: 2026-05-XX by Claude Code (CH-13 P3) -->`.
  5. `docs/specs/v0/implementation/m5_2/operations/audit-class-composition-operations.md` (NEW) — operator playbook covering: (a) how to verify the org's audit posture is honoured end-to-end via `phi org create` + Template-A firing + audit-event query; (b) the `audit_class_source` diff field semantics; (c) error-conditions / debugging when the composer's resolved class is unexpectedly low.
  6. `docs/specs/v0/implementation/<m5_X>/user-guide/cli-reference-mN.md` (existing — appendage) — one-paragraph note in the `phi org create --audit-class-default` walkthrough explaining the strictest-wins guarantee. Tier-fallback per §3.C.
  7. **Verified-header refresh** on every touched doc: ADR-0050, drift D-new-19, concept-audit matrix, the 3 user-facing docs above. Headers say verbatim what changed in this chunk (per CH-11 retro P4 verified-header rule).
  8. **Citation freshness re-grep** (per CH-12 retro Row 7) — re-run the §3 + §6 + §11 line-number greps and refresh any drifted citations in the plan's archived copy at `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md` before the orchestrator's final `cycle-audit.md` write.
- **Tests.** Re-run full workspace test suite + 4 CI guards. No new tests at P3.
- **Concept-alignment check.** Re-verify all §2 rows at chunk-close-target status. Run the §3 positive + forbidden greps; confirm scoreboard.
- **phi-core leverage check.** Re-run `scripts/check-phi-core-reuse.sh`; confirm green; expected import-count delta = 0 verified.
- **User-facing doc updates.** All §3.C rows shipped.
- **Confidence target.** ≥ 99% composite.
- **Pause discipline.** Pause for AskUserQuestion if any §2 row remains `contradicted` at P3 close. Pause if any §3 forbidden grep returns > 0. Pause if `scripts/check-doc-links.sh` fails on the 3 new doc files. Pause if any 3-aspect (Code/Docs/phi-core/Concept) marking is not unambiguously `pass`.

---

## §8 — Tests summary

**Predicted deliverable-listed sum (sum of P1 + P2 listed tests):** 7 (P1) + 7 (P2) = **14 new tests**.

**Plan §8 chunk-close prediction band (× 1.10–1.15 buffer per per-chunk-template line 157):** 14 × 1.10 = 15.4 → **15** lower; 14 × 1.15 = 16.1 → **16** upper.

**Orchestrator-accept band (asymmetric ×1.0–×1.20 per CH-12 retro Row 3):** 14 × 1.0 = 14 lower; 14 × 1.20 = 16.8 → **17** upper.

**Predicted final workspace test count:** baseline **1345** (verified at chunk-open via `cargo test --workspace 2>&1 | grep "test result: ok" | grep -v "0 passed" | awk '{print $4}' | paste -sd+ | bc`) + 14 new tests = **1359** target. Accept band: **1359–1362**. Outside band → AskUserQuestion before chunk-seal per template P4 paperwork rule.

**Layer breakdown:**
- Unit (in `audit_composition.rs`): 7 tests (P1).
- Integration (in `events/listeners.rs` `#[cfg(test)] mod tests`): 7 tests (P2).
- Acceptance / e2e: 0 (no HTTP/CLI surface change; the strictest-wins guarantee is verifiable via the integration tests' Repository-fixture path).
- Property (proptest): 1 of the P1 unit tests is a 50-case proptest (`unit_audit_class_ord_derive_property`).

**Named test files:**
- NEW: `modules/crates/domain/src/permissions/audit_composition.rs` (7 tests in `#[cfg(test)] mod tests`).
- EXTENDED: `modules/crates/domain/src/events/listeners.rs` (existing `#[cfg(test)] mod tests` gains 7 tests).
- EXTENDED (test-fixture updates only — counted as updated, not new): `modules/crates/domain/src/audit/events/m4/templates.rs:67-121`, `m5/templates.rs:101-169`. **3 existing tests updated to consume the new audit-event-builder signatures.**

**Named expected-still-green tests** (re-verified at chunk close):
- `audit_class_serde_roundtrip` (`audit/mod.rs:189`) — must still pass after `derive(PartialOrd, Ord)` added.
- `template_a_adoption_carries_template_a_tag` (`templates/a.rs:135`) — adoption AR shape unchanged.
- `adoption_ar_audit_class_is_alerted` (`templates/adoption.rs:162`) — unchanged.
- `fire_grant_holder_is_the_lead_agent` (`templates/a.rs:161`) — Grant's other fields unchanged; only `audit_class` is new.
- `fire_grant_action_is_read_inspect_list_in_stable_order` (`templates/a.rs:172`) — Action vec unchanged.
- 3 fire-fn proptests (`template_a_fire_grant_shape_props` style at 50 cases) — unchanged in property.
- `apply_org_creation_tx_test::*` (`store/tests/apply_org_creation_tx_test.rs:47`) — Org's `audit_class_default` field flow unchanged.

---

## §9 — Pre-chunk gate

**Reading list (mandatory, all read by planner before publishing this draft):**

1. `baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` lines 121–135 (CH-13 row + §1.4 "Frozen tags + audit" block) ✓
2. `baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-19.md` ✓
3. `baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md` (full file, with focus on §"audit_class Composition Through Templates" lines 64–72) ✓
4. `baby-phi/docs/specs/v0/concepts/permissions/README.md` (entry invariants — per the per-chunk-template Permissions subtree hook) — implementer to re-read at P0.
5. `baby-phi/docs/specs/v0/concepts/phi-core-mapping.md` (no overlap confirmed — per the per-chunk-template phi-core hook) — implementer reviews at P0.
6. `baby-phi/docs/specs/v0/requirements/cross-cutting/nfr-observability.md` (event-class retention rules — operator-meaning anchor for the ordering) — implementer reviews at P0.
7. `baby-phi/modules/crates/domain/src/audit/mod.rs` (full file — current `AuditClass` enum + `AuditEvent` + canonical_bytes) ✓
8. `baby-phi/modules/crates/domain/src/templates/a.rs` ✓ + `c.rs` ✓ + `d.rs` ✓ + `e.rs` ✓ + `adoption.rs` ✓
9. `baby-phi/modules/crates/domain/src/events/listeners.rs` lines 1–500 (TemplateAFireListener + TemplateCFireListener + TemplateDFireListener bodies) ✓
10. `baby-phi/modules/crates/domain/src/audit/events/m4/templates.rs` ✓ + `m5/templates.rs` ✓
11. `baby-phi/modules/crates/domain/src/model/nodes.rs` lines 370–800 (Organization + Grant + AuthRequest field declarations) ✓
12. CH-12 cycle plan: `baby-phi/docs/specs/plan/build/ch-12-frozen-session-tag-immutability-6a748175/plan.md` (just-sealed; shape reference + 7 honor-rules from CH-12 retro Standards) ✓
13. ADR-0048 (`docs/specs/v0/implementation/m5_2/decisions/0048-per-session-consent-gating.md`) — for the D48.1 Grant-denormalisation precedent CH-13 mirrors.
14. ADR-0049 (`docs/specs/v0/implementation/m5_2/decisions/0049-frozen-session-tag-immutability.md`) — for shape reference (chunk-spanning ADR with sub-decisions D49.1–D49.7).
15. **Per-chunk-template** `baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md` ✓ — applied 7 standards-updates from CH-12 retro Row 1–7.
16. `baby-phi/CLAUDE.md` §"phi-core Leverage" rules 1–5 ✓.

**Conditional reading-list (per CH-12 retro Row 5 — tag-write Repository contract).** This chunk does NOT introduce a new tag-write Repository method. Pattern signatures `update_*_tags`, `set_*_tags`, `retag_*`, `apply_tag_*` not added by CH-13. The conditional reading-list bullet for `repository.rs` module-level docstring (CH-12 ADR-0049 §D49.5 + §D49.7 contract) is therefore **not applicable** to this chunk.

**Carry-forward invariants** (verified green at chunk-open by planner):
- `cargo test --workspace` test count = **1345** (baseline; verified 2026-05-04 via the §8 command).
- `bash scripts/check-phi-core-reuse.sh` green (verified at chunk-open).
- `bash scripts/check-doc-links.sh` green (planner not running due to plan-mode draft; implementer P0 to verify).
- `bash scripts/check-ops-doc-headers.sh` green (planner not running due to plan-mode draft; implementer P0 to verify).
- `modules/` diff against the chunk-open git HEAD is empty (no preload edits — planner did read-only operations).

**Pending decisions carried into this chunk:**
- forward-scope Q4 (chunk-ordering) — user selects which chunk opens next; planner does not assume any pre-committed sequence.
- forward-scope Q5 (M5-scope defer rules) — D-new-19 is MEDIUM. Per the rule, MEDIUM drifts evaluate case-by-case at chunk-open. Forward-scope row 130 lists CH-13 as ~1 day with no prerequisites — user has indicated by spawning chunk-planner for CH-13 that it ships in M5; defer-to-M6 not invoked.

**Chunk-ordering note (Q4 decision):** No predecessor chunk strictly required (forward-scope row 133 — *"Prerequisites: none"*); CH-12 just sealed and CH-11 / CH-09 / CH-05 are ancestral. CH-13 is independently runnable.

---

## §10 — Close criteria

**Source of truth: concept docs.** No rounding; below-target blocks close.

**4 aspects (each graded pass / fail):**

- **Code aspect** — all P1+P2 deliverables shipped (cf. §7); `cargo test --workspace` passes (target count 1359; band 1359–1362); clippy green under `RUSTFLAGS="-Dwarnings"`; `cargo fmt --all -- --check` green.
- **Docs aspect** —
  - *Governance tier*: ADR-0050 status flipped Proposed → Accepted at P3; verified-headers updated on ADR + drift D-new-19 + concept-audit matrix + 3 new user-facing docs (per CH-11 retro Row P4 verified-header rule); concept-audit matrix row 215 Status copy-pasted letter-for-letter from §2 row 1's chunk-close target (per CH-12 retro Row 1); D-new-19 lifecycle-history entry appended; ADR-0050 cross-references ADR-0048 (D48.1 precedent) + concept-doc 07.
  - *User-facing tier* (post-CH-22): all 3 §3.C rows shipped (architecture-doc new file + operations-doc new file + user-guide one-paragraph note OR explicit defer-decision per fallback at P0). Header conventions per `check-doc-links.sh` + `check-ops-doc-headers.sh`.
- **phi-core leverage aspect** — predicted import-count delta of 0 verified. All §3 forbidden greps return 0. `scripts/check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's chunk-close target status achieved; none remain `contradicted`.

**2 confidence % (each with named numerator/denominator):**

- **Implementation confidence %** — target `claims-honored / claims-in-scope` ≥ **18/19 = 94.7%** at P3 close. The 19 in-scope claims: 7 §2-row chunk-close target statuses (incl. partial / split-axis rows counted once each) + 7 sub-decisions D50.1–D50.7 + 5 §10 governance-tier paperwork items (ADR Accept-flip, concept-audit-matrix row, drift lifecycle-history, ADR cross-refs, verified-headers).
- **Documentation confidence %** — target = **6/6 = 100%** (the 6 doc pages touched: ADR-0050, drift D-new-19, concept-audit matrix, architecture doc, operations doc, user-guide appendage).

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** Target **9/10** (orchestrator auto-approval criterion per CH-13's MEDIUM severity + clean composer-shape design). Composite below 9/10 blocks close.

**Tag-write Repository contract conditional close-criterion bullet (per CH-12 retro Row 5).** Not applicable to CH-13 (no new tag-write Repository method introduced).

**P4 chunk-seal paperwork checklist (CH-11 retro Row P4):** for every modified doc with a verified-header (line 1), the header description matches the body diff exactly. The 6 P3 doc edits are gated on this rule.

**P4 paperwork addendum (CH-12 retro Row 1):** concept-audit matrix line 215's new Status value copy-pasted letter-for-letter from §2 row 1's chunk-close target — exactly `honored`. NOT an interpretive flip; the matrix row's "Status" column says `honored` literally.

---

## §11 — Post-chunk independent audit plan

**Audit envelope size (per `audit-envelope-size` skill output):** chunk has 4 phases (P0+P1+P2+P3); `audit-envelope-size` skill rule "1 agent for ≤ 3 phases; 2 agents for 4–6 phases" → **2 auditors**. The chunk touches 3 listener bodies + 1 new module + Grant struct + 6 doc files; matches CH-12's audit envelope shape (also 2 auditors).

**Audit aspects (a–d) coverage map:**
- **Audit A (Code+phi-core)** — covers (a) Code correctness + (d) phi-core leverage.
- **Audit B (Concept+Docs)** — covers (b) Docs fidelity + (c) Concept alignment.

**Audit A prompt (≤ 600 words):**

```
You are chunk-auditor for CH-13 (audit_class composition strictest-wins). Audit aspects (a) Code correctness + (d) phi-core leverage. The cycle plan is at `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md`. Read §1, §3, §7 (P1 + P2 deliverables), §8, ADR-0050 at `m5_2/decisions/0050-audit-class-composition-strictest-wins.md`.

Verify these 11 claims (each cite file:line):

1. `AuditClass` enum at `modules/crates/domain/src/audit/mod.rs:48–52` derives `PartialOrd, Ord` per ADR-0050 D50.2; declaration order is `Silent, Logged, Alerted` (loosest → strictest); concept-doc 07 line 68 mapping `none ↔ Silent` documented in a doc-comment per D50.1.
2. `compose_audit_class(org_default, template_ar, override) -> AuditClass` exists at `modules/crates/domain/src/permissions/audit_composition.rs` per D50.3.
3. `compose_audit_class_with_source(...) -> (AuditClass, AuditClassSource)` exists at the same path; the `AuditClassSource` enum has 3 variants `OrgDefault | TemplateAr | Override` per the audit-event diff schema (D50.6).
4. The composer body uses `[Some(a), Some(b), c].into_iter().flatten().max()` (or equivalent fold) — confirm via reading the body.
5. The composer's tie-breaker rule is `Override > TemplateAr > OrgDefault` per D50.3 — confirm via doc-test or unit test.
6. `Grant.audit_class: AuditClass` field exists at `modules/crates/domain/src/model/nodes.rs` (around line 676 — adjacent to `approval_mode` at line 675 from the F2.A precedent); `#[serde(default = "Grant::default_audit_class")]` shielding produces `AuditClass::Silent` per D50.5.
7. The 3 fire pure-fns (`fire_grant_on_lead_assignment` `templates/a.rs:93`, `fire_grant_on_manages_edge` `templates/c.rs:74`, `fire_grant_on_has_agent_supervisor` `templates/d.rs:69`) construct Grant with `audit_class: args.audit_class` — pure-fn discipline preserved.
8. The 3 fire listeners at `events/listeners.rs:303,433,552` (post-CH-13 line numbers; pre-CH-13 estimate was 223,345,457) call `compose_audit_class_with_source` (via the `resolve_composed_audit_class` helper at `events/listeners.rs:140`) BEFORE constructing FireArgs.
9. The 3 audit-event builders at `audit/events/m4/templates.rs:30` (template_a_grant_fired), `m5/templates.rs:29` (template_c), `m5/templates.rs:77` (template_d) accept `audit_class: AuditClass` + `audit_class_source: AuditClassSource` parameters; their bodies do NOT hardcode `AuditClass::Logged`.
10. phi-core import-count delta = 0 (run `grep -rn "use phi_core::" modules/crates/domain/src/permissions/audit_composition.rs` — expect 0 hits; same for the listener edits).
11. Forbidden greps return 0: (a) `grep -rn "^pub enum AuditClass\b" modules/crates/ | grep -v "audit/mod.rs"` — expect 0; (b) `grep -rn "audit_class:\s*AuditClass::Logged\b" modules/crates/domain/src/audit/events/m4/templates.rs modules/crates/domain/src/audit/events/m5/templates.rs` (production builder bodies, not test fixtures) — expect 0 since the builder no longer hardcodes Logged.

Run `cargo test --workspace --lib -p domain` and confirm all `audit_composition::tests::*` pass (7 tests) + `events::listeners::tests::*` pass with 7 added.

DO NOT execute `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` or the 4 `bash scripts/check-*.sh` CI guards (sandbox-blocked; mark as NOT-EXECUTED-IN-AUDIT — orchestrator's final cycle-audit closes them per `baby-phi/CLAUDE.md` §"final cycle re-audit" MUST-RUN list).

Write the audit log to `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/audit-A-iter1.md`. Mark each claim PASS/FAIL/NOT-EXECUTED with cited file:line. Verdict at end.
```

**Audit B prompt (≤ 600 words):**

```
You are chunk-auditor for CH-13 (audit_class composition strictest-wins). Audit aspects (b) Docs fidelity vs concept docs + (c) Concept alignment. The cycle plan is at `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/plan.md`. Read §2, §3.C, §4, §5, §10, ADR-0050.

Verify these 9 claims (each cite file:line):

1. ADR-0050 at `docs/specs/v0/implementation/m5_2/decisions/0050-audit-class-composition-strictest-wins.md` — Status `Accepted`; verified-header line 1 carries the chunk-seal date + "(CH-13 P3 chunk-seal — ADR flipped Proposed → Accepted)"; sub-decisions D50.1–D50.7 present.
2. Drift D-new-19 at `docs/specs/v0/implementation/m5_1/drifts/D-new-19.md` — `Status: remediated`; lifecycle-history entry appended with the chunk seal date + cycle hex `d4fe1b7c` + ADR-0050 reference.
3. Concept-audit matrix at `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` line 215 — Status column says `honored` (copy-pasted letter-for-letter from §2 row 1's chunk-close target per CH-12 retro Row 1 P4 paperwork addendum). "Code evidence" cites `permissions/audit_composition.rs` + `events/listeners.rs`. "Covering drift" updated to `D-new-19 (remediated 2026-05-XX via CH-13)`.
4. Architecture doc at `docs/specs/v0/implementation/m5_2/architecture/audit-class-composition.md` — exists; verified-header line 1 dated chunk-seal; covers composer fn signature + ordering encoding + Grant denormalisation + listener wiring + audit-event diff extension + cross-pod determinism (§3.B A7 argument).
5. Operations doc at `docs/specs/v0/implementation/m5_2/operations/audit-class-composition-operations.md` — exists; verified-header dated; covers `phi org create` + Template-A firing + audit-event query verification recipe; `audit_class_source` diff field semantics; debugging when composer's resolved class is unexpectedly low.
6. User-guide note — confirm appended to nearest existing user-guide tier (per §3.C P0 fallback decision recorded in P0 verification log); one-paragraph strictest-wins-guarantee note next to the `phi org create --audit-class-default` walkthrough.
7. Concept-doc 07 lines 64–72 verbatim claims (read the source) — every claim referenced in §2 still matches the concept doc body. The concept doc itself was NOT modified by CH-13 (read-only consumption per `baby-phi/CLAUDE.md` §"Documentation Alignment").
8. Permissions subtree-invariant doc `concepts/permissions/README.md` — unchanged; no edits needed.
9. phi-core-mapping doc `concepts/phi-core-mapping.md` — unchanged; CH-13 confirms the existing "Orthogonal surfaces" entry that audit governance is baby-phi-only.

Verify §2 walk: read each row's "Target status at chunk-close" and confirm the post-chunk code state matches. The post-chunk concept-audit matrix line 215's Status of `honored` MUST be a copy-paste of §2 row 1's target — flag any drift.

DO NOT execute `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` or the 4 `bash scripts/check-*.sh` CI guards (sandbox-blocked; mark as NOT-EXECUTED-IN-AUDIT — orchestrator's final cycle-audit closes them).

Write the audit log to `baby-phi/docs/specs/plan/build/ch-13-audit-class-composition-strictest-wins-d4fe1b7c/audit-B-iter1.md`. Mark each claim PASS/FAIL/NOT-EXECUTED with cited file:line. Verdict at end.
```

**Audit pass criteria.** Both audit logs return clean (no FAIL on Code/Docs/Concept/phi-core aspects). Any new drift surfaced is filed with explicit M5/M6 future-chunk assignment. Chunk seal blocked until both audits return clean + all audit-discovered drifts scoped.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (cargo capped at -j 4 per memory feedback_cargo_jobs_cap.md)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace

# 3. Chunk-specific tests (named in §8)
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib audit_composition::
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib events::listeners::tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib audit::events::m4::templates::tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib audit::events::m5::templates::tests

# 4. Positive close-audit greps from §3
grep -rn "pub fn compose_audit_class" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l
# Expect: 1
grep -nE "pub audit_class: AuditClass" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs
# Expect: ≥ 2 hits (Organization line ~384, AuthRequest line ~790, NEW Grant line ~676)
grep -nE "compose_audit_class\(" /root/projects/phi/baby-phi/modules/crates/domain/src/events/listeners.rs | wc -l
# Expect: 3 (TemplateAFireListener, TemplateCFireListener, TemplateDFireListener)

# 5. Forbidden-duplication greps from §3
grep -rn "^pub enum AuditClass\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "audit/mod.rs"
# Expect: 0
grep -rn "^enum AuditClass\b" /root/projects/phi/baby-phi/modules/crates/cli/ /root/projects/phi/baby-phi/modules/crates/server/
# Expect: 0
grep -nE "audit_class:\s*AuditClass::Logged\b" /root/projects/phi/baby-phi/modules/crates/domain/src/audit/events/m4/templates.rs /root/projects/phi/baby-phi/modules/crates/domain/src/audit/events/m5/templates.rs | grep -v "test"
# Expect: 0 in production builder bodies (test fixtures may legitimately keep specific values)

# 6. Drift status check
grep -E "^- \*\*Status" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-new-19.md
# Expect: "**Status**: `remediated`"

# 7. ADR status check
grep -E "^\*\*Status:" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0050-audit-class-composition-strictest-wins.md
# Expect: "**Status: Accepted**"

# 8. Concept-audit-matrix row check (CH-12 retro Row 1 P4 paperwork addendum)
grep -nE "audit_class composition.*honored" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md
# Expect: 1 hit at line ~215 with Status column = "honored" letter-for-letter from §2 row 1's target

# 9. Test count delta confirmation
/root/rust-env/cargo/bin/cargo test -j 4 --workspace 2>&1 | grep -E "^test result: ok" | grep -v "0 passed" | awk '{print $4}' | paste -sd+ | bc
# Expect: 1359 (1345 baseline + 14 new); accept band 1359–1362

# 10. Citation-freshness re-grep (CH-12 retro Row 7) — re-run §3 + §6 + §11 line-number greps
grep -n "pub enum AuditClass\|pub fn template_a_grant_fired\|pub fn fire_grant_on_lead_assignment\|pub struct Grant\b\|pub audit_class_default\|pub audit_class:" /root/projects/phi/baby-phi/modules/crates/domain/src/audit/mod.rs /root/projects/phi/baby-phi/modules/crates/domain/src/audit/events/m4/templates.rs /root/projects/phi/baby-phi/modules/crates/domain/src/templates/a.rs /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs
# Refresh any plan citations that drifted post-implementation
```

---

## Cross-cycle handoff

**5-line cycle summary:**
- Chunk goal: ship `compose_audit_class(org_default, template_ar, override) -> AuditClass` strictest-wins composer; denormalise `audit_class` onto Grant; wire into 3 production fire listeners (Template A/C/D); extend audit-event diff with `audit_class_source` attribution.
- Drifts closed: D-new-19 (MEDIUM, audit-integrity hardening; `discovered → remediated`).
- ADR(s) drafted: ADR-0050 at `m5_2/decisions/0050-audit-class-composition-strictest-wins.md` with sub-decisions D50.1–D50.7.
- New K8s deferral: none (K8s-neutral per §3.B; hash-chain symmetry preserved per A7).
- Key risks / pause-discipline triggers: (1) Grant literal-struct cascade > 5 sites at P1 (asymmetric ×1.0–×1.20 buffer per CH-12 retro Row 3); (2) Repository trait extension if `get_organization`-style getter doesn't exist at P2; (3) any §2 row remaining `contradicted` at P3 close.

**Auto-approval criteria evaluation (orchestrator gate):**
- No locked forks: ✓ (3 forks, all planner-recommendations; no user-lock unless orchestrator escalates).
- Scope ≤ 1.5× forward-scope: ✓ (forward-scope row 134 says ~1 day; planner predicts 1 day).
- Zero phi-core leverage delta: ✓ (predicted 0 imports added).
- No new K8s blocker class: ✓ (§3.B verdict K8s-neutral).
- Audit envelope ≤ medium: ✓ (2 auditors, medium envelope).
- Confidence ≥ 9/10: ✓ (target 18/19 = 94.7% impl × 6/6 = 100% docs → composite 9.4/10).
- No new migration: ✓ (§3.B A4 — migration count stays at 13).

**Verdict:** Direct-approvable via ExitPlanMode (orchestrator's call). All criteria hold; no AskUserQuestion needed unless orchestrator dissents on F1/F2/F3 recommendations.

**Estimated effort:** ~1 engineer-day.
- 0.1d — P0 chunk-open ritual + ADR scaffold.
- 0.4d — P1: composer fn + AuditClass Ord derive + Grant field + 7 unit tests + ADR-0050 body draft.
- 0.4d — P2: 3-listener wiring + audit-event-builder signature changes + 7 integration tests + 3 test-fixture updates.
- 0.1d — P3: doc tier + ADR Accept-flip + matrix-row + drift lifecycle-history + verified-headers + citation-freshness re-grep.
