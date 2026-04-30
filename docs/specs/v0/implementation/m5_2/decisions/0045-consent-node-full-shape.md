<!-- Last verified: 2026-04-30 by Claude Code -->

# ADR-0045 — Consent node full shape (11 fields + ConsentScope + ConsentState enum)

**Status: Accepted**

**Date:** 2026-04-30
**Chunk:** CH-09
**Closes:**
- [`D-new-04`](../../m5_1/drifts/D-new-04.md) (HIGH) — Consent node carries only 5 fields; concept mandates 11.

**Stays open (CH-10 owns):**
- [`D-new-05`](../../m5_1/drifts/D-new-05.md) (HIGH) — Consent lifecycle state machine. CH-09 ships the type + the `state` field; CH-10 wires the transition function + `update_consent_state` repo method.

---

## Context

[`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Consent Node (New Node Type)" lines 351–363 specifies an 11-field Consent record carrying everything needed to gate Authority Template grants: who consented (`agent_id`), what scope (`scope.org` + `scope.templates` + `scope.actions`), what state (a 6-variant lifecycle), three timestamps, a revocability flag, and a provenance string. §"Consent Lifecycle" lines 369–414 specifies the lifecycle: `Requested → Acknowledged / Declined / TimedOut`, optionally `Revoked` (forward-only), with `Expired` as the natural-scope-ended terminal.

Until CH-09, the shipped `Consent` struct at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) carried only 5 fields — `id`, `subordinate`, `scoped_to`, `granted_at`, `revoked_at`. Missing: the entire lifecycle state, the per-template / per-action scope, the response timestamp, the revocability flag, and provenance. Drift D-new-04 (HIGH, Bucket A) tracked the gap; downstream chunks CH-10 (state machine) and CH-11 (per-session gating) are blocked on the field set landing first.

The exploration phase confirmed two important properties of the current state:

- **Zero production callers.** Only one test fixture at [`store/tests/repository_test.rs:359`](../../../../../../modules/crates/store/tests/repository_test.rs) constructs a `Consent` today. There is no HTTP / CLI flow that creates a Consent record. Permission checks against consent go through `ConsentIndex::is_acknowledged(AgentId, OrgId)` which is populated from the existing 5-field rows.
- **Excellent reuse surfaces.** [`ConsentPolicy`](../../../../../../modules/crates/domain/src/model/composites_m3.rs) already establishes the snake-case-serialized enum pattern. [`Action`](../../../../../../modules/crates/domain/src/permissions/action.rs) (CH-04 / ADR-0043) is the typed action vocabulary `ConsentScope.actions` will reference. Migration 0009 (`identity_node`) is the most recent precedent for `DEFINE TABLE SCHEMAFULL` + `DEFINE FIELD` shape.

The user-decided forks at plan-review (2026-04-29) locked three open questions:

1. **Field naming** — Rename to match concept doc verbatim: `subordinate` → `agent_id`, flatten `scoped_to: OrgId` into nested `scope: ConsentScope`, rename `granted_at` → `responded_at` with semantic shift (now `Option<DateTime<Utc>>`, set when subordinate Acknowledges or Declines). Best concept-doc fidelity; safe to do cleanly because no production data exists.
2. **`ConsentState` enum scope** — Ships at CH-09 with all 6 variants + serde snake_case + `Default::Acknowledged`. CH-10 ships ONLY the transition function + `update_consent_state` repo method — no struct changes. Cleaner separation; the struct lands complete in one chunk.
3. **Repository surface** — Stays minimal. Only `create_consent` exists; CH-09 keeps it that way. CH-10 adds `update_consent_state`; CH-11 adds read methods (`get_consent`, `list_consents_for_subordinate`). Each chunk owns its own repo additions.

---

## Decision

### D45.1 — Consent struct rewritten with 11 concept-doc-mandated fields

The `Consent` struct at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) is rewritten with the field set from `concepts/permissions/06-multi-scope-consent.md` lines 351–363 verbatim:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub id: ConsentId,
    pub agent_id: AgentId,
    pub scope: ConsentScope,
    #[serde(default)]
    pub state: ConsentState,
    pub requested_at: DateTime<Utc>,
    #[serde(default)]
    pub responded_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocable: bool,
    pub provenance: String,
}
```

The 11 concept-doc fields land as 9 leaf Rust fields because `scope` is a nested struct holding 3 of them (`org`, `templates`, `actions`). Counting leaves: `id` + `agent_id` + (`scope.org` + `scope.templates` + `scope.actions`) + `state` + `requested_at` + `responded_at` + `revoked_at` + `revocable` + `provenance` = 11. The compile-time check is via the `consent_struct_carries_concept_doc_fields` test which constructs the full shape and touches every leaf — a missing or extra field wouldn't compile.

### D45.2 — `ConsentScope` nested struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub org: OrgId,
    #[serde(default)]
    pub templates: Vec<TemplateId>,
    #[serde(default)]
    pub actions: Vec<crate::permissions::action::Action>,
}
```

The cross-layer dep on `Action` from `permissions::action` is intentional and consistent with the precedent set by `Grant.action: Vec<Action>` ([`nodes.rs:627`](../../../../../../modules/crates/domain/src/model/nodes.rs)) and `ToolAuthorityManifest.actions: Vec<Action>` ([`nodes.rs:775`](../../../../../../modules/crates/domain/src/model/nodes.rs)). Empty `templates` / `actions` lists mean "any template / any action covered by the org-level policy"; the Permission Check engine intersects this scope with the grant's claimed reach at runtime (CH-11's job).

### D45.3 — `ConsentState` enum (6 variants)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsentState {
    Requested,
    #[default]
    Acknowledged,
    Declined,
    Revoked,
    TimedOut,
    Expired,
}
```

The 6 variants match concept doc 06 §"Consent Lifecycle" lines 397–404 verbatim. `#[serde(rename_all = "snake_case")]` matches the precedent set by `ConsentPolicy` and `AgentRole`; `TimedOut` serializes as `timed_out`, every other variant matches its lowercased name. The `ConsentState::ALL` const enumerates the closed set in declaration order for tests + future CH-10 transition logic.

### D45.4 — `Default for ConsentState` returns `Acknowledged`

Two reasons:

1. **Concept-doc semantic.** Per concept doc 06 line 410 — under the `implicit` consent policy, "Consent is auto-Acknowledged at agent creation, no request is sent". `Acknowledged` is the natural default for a freshly-created Consent record under that policy.
2. **Back-compat shield.** `#[serde(default)]` on the `state` field ensures any pre-CH-09 wire payload that omits the field decodes cleanly to `Acknowledged`. The 5-field rows that existed pre-migration-0010 would map to "consent given, never revoked" — exactly `Acknowledged` semantics. (No such rows exist today, but the back-compat shield is cheap and matches the M5 wire-format-stability discipline.)

The `consent_legacy_wire_format_decodes_with_defaults` test pins this back-compat path against regression.

### D45.5 — Migration 0010 redefines the consent table cleanly

New migration [`modules/crates/store/migrations/0010_consent_full_shape.surql`](../../../../../../modules/crates/store/migrations/0010_consent_full_shape.surql) redefines the consent table:

```sql
REMOVE TABLE IF EXISTS consent;
DEFINE TABLE consent SCHEMAFULL;
DEFINE FIELD agent_id     ON consent TYPE string ASSERT $value != NONE;
DEFINE FIELD scope        ON consent FLEXIBLE TYPE object ASSERT $value != NONE;
DEFINE FIELD state        ON consent TYPE string DEFAULT "acknowledged";
DEFINE FIELD requested_at ON consent TYPE string ASSERT $value != NONE;
DEFINE FIELD responded_at ON consent TYPE option<string>;
DEFINE FIELD revoked_at   ON consent TYPE option<string>;
DEFINE FIELD revocable    ON consent TYPE bool DEFAULT true;
DEFINE FIELD provenance   ON consent TYPE string DEFAULT "";
```

`REMOVE TABLE IF EXISTS` + `DEFINE TABLE` is safe because the consent table has no production data — the only constructor in the workspace is the test fixture. The `_migrations` ledger applies it once at startup per the existing forward-only idempotent runner pattern (per ADR-0042 §D42.3 conforming-criteria #5). The migration is registered at [`store::migrations::EMBEDDED_MIGRATIONS`](../../../../../../modules/crates/store/src/migrations.rs) at version 10, slug `consent_full_shape`.

`scope` lands as `FLEXIBLE TYPE object` because the inner shape is owned by the typed Rust `ConsentScope` struct + serde, not the SurrealDB schema — same pattern as `Identity.lived` / `Identity.witnessed` from migration 0009.

### D45.6 — Field-rename rationale

| Old (M2) | New (CH-09) | Reason |
|---|---|---|
| `subordinate: AgentId` | `agent_id: AgentId` | Concept-doc verbatim (line 354). The "subordinate" naming made sense as an M2 shorthand; "agent_id" matches the concept doc and the rest of the node-id naming convention (`agent_id` appears across Identity, Grant, AuthRequest). |
| `scoped_to: OrgId` | `scope.org: OrgId` (nested) | The concept doc uses `scope.{org, templates, actions}` as a single grouping; collapsing it to a flat `scoped_to` lost the per-template / per-action narrowing that the consent flow needs to express. |
| `granted_at: DateTime<Utc>` (always set) | `responded_at: Option<DateTime<Utc>>` (None until response) | Semantic shift: the M2 stub set `granted_at` at record creation (which is wrong — at `Requested`, the subordinate hasn't responded yet). The new field is `None` until the subordinate Acknowledges or Declines (concept doc line 360). |

The test fixture at `store/tests/repository_test.rs:353` is renamed `create_consent_persists_row_with_agent_id_and_scope` and constructs the new shape; per locked Q1 (2026-04-29).

### D45.7 — Repository trait surface unchanged

`Repository::create_consent(&Consent) -> RepositoryResult<()>` is the only consent-related method at this chunk. No `get_consent`, no `update_consent_state`, no `list_consents_for_subordinate`. Per locked Q3 (2026-04-29):

- CH-10 will add `update_consent_state(consent_id, ConsentState) -> RepositoryResult<()>` paired with the transition function (drift D-new-05).
- CH-11 will add the read methods needed for per-session gating (`get_consent`, `list_consents_for_subordinate`).

Each chunk owns its own repo additions; CH-09 stays focused on the field shape.

### D45.8 — `ConsentIndex` projection unchanged at this chunk

[`domain::permissions::manifest::ConsentIndex::is_acknowledged(subordinate, org)`](../../../../../../modules/crates/domain/src/permissions/manifest/mod.rs) still consumes `(AgentId, OrgId)` pairs. CH-11 evolves the projection to include scope (`templates`, `actions`) + state filtering once the per-session gating logic lands. CH-09 leaves the index alone — the new fields are present on the underlying rows but not yet read by the index.

### D45.9 — Out of scope at this chunk: the state-transition function

Per locked Q2 (2026-04-29), `ConsentState` ships as a TYPE only at CH-09. The transition function — `Consent::transition(current_state, ConsentEvent) -> Result<ConsentState, TransitionError>` — is CH-10's job. CH-10 also owns:

- Forward-only revocation invariant (`Revoked` cannot transition back to `Acknowledged`).
- Timeout handling (`Requested` after policy-driven timeout becomes `TimedOut`; default response is `deny` per concept doc line 348).
- Per-policy mapping (`implicit` short-circuits to `Acknowledged`; `one_time` requests on first fire; `per_session` requests every new session).
- Audit-event emission for transitions (`consent.acknowledged`, `consent.declined`, `consent.revoked`, `consent.timed_out`, `consent.expired`).

CH-09 emits zero new audit events; the BLAKE3 hash chain is byte-stable across the chunk.

### D45.10 — `provenance: String` stays untyped at v0

The concept doc shows examples like `"agent:claude-coder-9@onboarding"` (concept doc line 363) which are conventional audit labels, not a structured type. A typed `Provenance` enum is deferred to v1, where it would bundle with a broader audit-event schema overhaul. The `String` field is sufficient to record the audit trail in v0 and matches the existing untyped audit-payload precedent on `AuditEvent.actor` / similar fields.

---

## Consequences

**Positive:**
- Drift D-new-04 closed; concept-doc fidelity restored.
- CH-10 (state machine) and CH-11 (per-session gating) unblocked.
- Wire format is now the inaugural production shape — no migration churn for downstream consumers.
- `#[serde(default)]` on `state` + `responded_at` + `revoked_at` shields against future field additions following the same pattern.

**Negative:**
- Three field renames (`subordinate` → `agent_id`, `scoped_to` → `scope.org`, `granted_at` → `responded_at`) require touching the one existing test fixture. Acceptable: the rename is one-time, the cost is zero in production, and the new names match the concept doc verbatim.

**Neutral:**
- Migration 0010 is forward-only (per ADR-0012); no down script.
- `provenance: String` accepts arbitrary content at v0; CH-10 will likely add validation when audit-event emission lands.

---

## Cross-references

- Concept doc: [`permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) lines 349–414.
- Drift closed: [`D-new-04`](../../m5_1/drifts/D-new-04.md).
- Drift stays open: [`D-new-05`](../../m5_1/drifts/D-new-05.md) (CH-10 owns).
- Action enum dependency: [ADR-0043](0043-typed-action-vocabulary.md) — `ConsentScope.actions: Vec<Action>` consumes the typed vocabulary.
- Migration runner conforming criteria: [ADR-0042](0042-storage-backend-configurable.md) §D42.3 — migration 0010 satisfies #5 (forward-only, idempotent under repeated runs).
- Forward-only migration policy: ADR-0012.
- Downstream chunks: CH-10 (consent state machine), CH-11 (per-session consent gating).

---

## Verification

- Workspace tests: `cargo test --workspace -- --test-threads=1` green at ~1197 (1187 baseline + 8 new unit tests + 3 new acceptance tests).
- Clippy under `RUSTFLAGS="-Dwarnings"`: clean.
- 4 CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
- Positive greps: `pub struct Consent` (1), `pub struct ConsentScope` (1), `pub enum ConsentState` (1), migration 0010 file exists with 9 `DEFINE FIELD` lines.
- Negative greps: `pub subordinate: AgentId`, `pub scoped_to: OrgId`, `pub granted_at: DateTime` in consent context — all 0.
- Carry-forward green: CH-04 matrix tests, CH-05 manifest validator acceptance, CH-21 memory extraction acceptance (audit hash chain byte-stable).
