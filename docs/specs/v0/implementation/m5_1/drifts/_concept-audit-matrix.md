<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P3 chunk-progress: in the `permissions/02` block, the "allocate vs transfer cardinality" row Status flipped letter-for-letter from `silent-in-code` to `**honored**` per CH-12 retro Row 1 P4 paperwork addendum (copy-paste of CH-08 plan §2 row 1 target); Code-evidence cell now cites `Repository::apply_transfer_grant` (repository.rs:1397) + InMemoryRepository impl (in_memory.rs:1868) + SurrealStore impl (store/src/repo_impl.rs:2691) + 6 atomicity tests + 3 SurrealStore integration tests + ADR-0052 §D52.1/§D52.5/§D52.6; Covering-drift cell flipped to `D-new-13 ✓`. In the `permissions/03` block, the "`allocate` umbrella" row Status flipped letter-for-letter from `silent-in-code` to `**honored**` (Code-evidence cell cites `domain::permissions::AllocateRefinement` (allocate_refinement.rs) + `Grant.allocate_refinement` (model/nodes.rs:701) with `#[serde(default)]` shielding + 3 unit tests + ADR-0052 §D52.2/§D52.3); Covering-drift `D-new-29 ✓`. No new rows added. Note: the §2 plan-walk also names §"`allocate` Scope Semantics" line 197 (refinement framing on permissions/02) as a third claim flipping at this chunk; the matrix does not carry a separate row for that §-anchor — the claim rolls up under the existing `allocate` umbrella row at line 165 of the matrix per CH-07-style label-coverage rollup. CH-08 implementer surfaced this label-coverage gap as a deviation in the P3 report, mirroring CH-07's pattern.) -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-07 chunk-seal: in the `permissions/04` block, the "5-tier scope cascade" row Status flipped letter-for-letter from `partially-honored` to `honored` (Code-evidence cell now cites `engine.rs::step_5_scope_resolution` (full 2-tier cascade body) + `engine.rs::cascade_intersection_fallback` + ADR-0051 §D51.1 + §D51.2; Covering-drift cell flipped to `D-new-06 ✓`). In the `permissions/06` block, the "Unified resolve_scope cascade" row Status flipped letter-for-letter from `partially-honored` to `honored` (Code-evidence cell cites the same `step_5_scope_resolution` body + ADR-0051 §D51.4 + §D51.5; Covering-drift `D-new-06 ✓`); the "Contractor model" row Status flipped letter-for-letter from `silent-in-code` to `honored` (Code-evidence cell cites `engine.rs::step_2a_ceiling` membership-bound filter + ADR-0051 §D51.6; Covering-drift `D-new-20 ✓`). In the `permissions/08` block, the "Shape A/B/C resolution per worked examples" row Status flipped letter-for-letter from `partially-honored` to `honored` (Code-evidence cell cites the cascade body + acceptance tests at `tests/multi_scope_cascade_acceptance.rs`; Covering-drift `D-new-06 ✓`); the "Contractor scenario" row Status flipped letter-for-letter from `silent-in-code` to `honored` (Code-evidence cell cites the `step_2a_ceiling` membership-bound filter + Scenario 7 acceptance test; Covering-drift `D-new-20 ✓`). No new rows added. Note: the §2 plan-walk also names §"Key Invariants line 310" (permissions/04), §"Subject-Side Reach Is Bounded" (permissions/06), and §"Summary: Who Can Read What" (permissions/08); the matrix does not carry separate rows for those §-anchors — the claims roll up under the existing rows updated above. CH-07 implementer surfaced this label-coverage gap as a deviation in the P3 report.) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-13 chunk-seal: in the `permissions/07-templates-and-tools.md` block, the "audit_class composition: strictest wins" row Status flipped letter-for-letter from `silent-in-code` to `honored` per CH-12 retro Row 1 P4 paperwork addendum (copy-paste of CH-13 plan §2 row 1's "Target status at chunk-close"); Code-evidence cell now cites `permissions/audit_composition.rs` (composer fn `compose_audit_class` + `compose_audit_class_with_source` + `AuditClassSource` enum) + `events/listeners.rs:303,433,552` (TemplateA/C/D fire listeners' `resolve_composed_audit_class` calls) + audit-event builders at `audit/events/m4/templates.rs:30` + `m5/templates.rs:29,77`; Covering-drift cell flipped to `D-new-19 ✓`. No new rows added.) -->
<!-- Last verified: 2026-05-04 by Claude Code (CH-12 chunk-seal: in the `permissions/05` block, "Session tags frozen at creation" row flipped from `silent-in-code` to `honored` (Code-evidence cell now cites CH-12 / ADR-0049 — Rule E in `validate_published_manifest` + `validate_tag_write_on_session` + `SESSION_FROZEN_TAG_PREFIXES` + `tool.frozen_tag_write_rejected` audit-event builder; Covering-drift D-new-08 ✓), and "Session tag vocabulary" row's Status stays at `partially-honored` per plan split-axis framing — immutability-axis honored by CH-12 (Code-evidence cell now cites the SESSION_FROZEN_TAG_PREFIXES const), emission-axis remains aspirational (only `#kind:session` + `session:<id>` are auto-emitted today; the 6 M6+ categories deferred to a future M6+ chunk per `D-CH12-FOLLOWUP-01`); Covering-drift D-new-08 ✓ added (immutability axis). In the `permissions/09` block, "Reserved namespace write rejection" row's Code-evidence cell extended to cite CH-12's composite-case extension via Rule E + target_kinds; Covering-drift D-new-08 ✓ added alongside D-new-31. No new rows added.) -->
<!-- Last verified: 2026-05-03 by Claude Code (CH-11 chunk-seal: existing Per-Session-consent row in the `permissions/06` block flipped from `partially-honored` to `honored`; CH-11 / ADR-0048 evidence + D-new-17 ✓ added in the Code-evidence + Covering-drift cells. No separate ADR-0048 row was added — the Per-Session claim was already enumerated.) -->

# M5.1/P2 — Concept-audit matrix

Full `{concept_doc, §anchor, claim, status, code_evidence, covering_drift}`
matrix produced by walking every one of the 20 concept docs at
[`docs/specs/v0/concepts/`](../../../concepts/) claim-by-claim against
current HEAD. **Concept docs are the source of truth.** Every row is one
testable claim extracted from a concept doc.

## Column meanings

- **Status:** `honored` | `contradicted` | `partially-honored` | `silent-in-code` | `concept-aspirational`
- **Code evidence:** where the claim is (or is not) reflected — file:line-range or "none"
- **Covering drift:** existing `DX.Y` drift id, new `D-new-NN` drift id, or `—` for honored rows
- **phi-core leverage:** `direct-reuse` | `wrap` | `inherit-from-snapshot` | `reject-build-native` | `N/A` | `leverage-violation` (new column per Guardrail #6)

## Discovery summary

- **Rows audited**: ~95 claims across 20 concept docs
- **Honored**: ~53 rows
- **Partially-honored / silent-in-code**: ~28 rows (many flagged as `concept-aspirational` for explicitly-deferred M6+ scope)
- **Contradicted**: ~14 rows — each generated a new drift or maps to an existing one
- **New drifts minted**: **31** (`D-new-01` through `D-new-31`)
- **Proposals discarded after verification** (claim was false): **2**
  - D-new-system-agent-creation-missing → [`orgs/create.rs:366+`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) actually seeds both standard system agents at org creation
  - D-new-permission-ceiling-enforcement → [`permissions/engine.rs:235`](../../../../../../modules/crates/domain/src/permissions/engine.rs) has `step_2a_ceiling` implemented
- **Proposals absorbed into existing drifts**: **2** (stub-listener proposals → covered by D4.2 + D6.1)
- **Agent-proposal pairs merged into single drift**: **4** (multi-scope cascade, manifest validator, frozen-tag enforcement, bootstrap template)

## Matrix rows by concept doc

### `concepts/README.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Core Insight | Permissions are actions on resources with constraints | honored | `permissions/engine.rs` | — | N/A |
| Canonical 5-tuple | Grant has (subject, action, resource, constraints, provenance) | honored | `nodes.rs:597-612` Grant struct | — | N/A |
| Subject in edge | Subject derived from HOLDS_GRANT edge, not stored on Grant | honored | Grant carries `holder: PrincipalRef` + `auth_request_id` | — | N/A |
| Provenance | Chain traces to bootstrap | partially-honored | `auth_request_id` field exists; traversal logic missing | D-new-14 | N/A |

### `concepts/agent.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Roles | 6 AgentRole variants | honored | `nodes.rs:229-236` | — | N/A |
| Roles | `is_valid_for(kind)` cross-kind guard | honored | `nodes.rs:266-271` | — | N/A |
| Roles | Role immutable post-creation | honored | rust-level guard at `update.rs:133-141` rejects `new_role.is_some()` returning `ImmutableFieldChanged("role")`; HTTP wire `UpdateAgentProfileRequest` does not include `role` (silent-drop); pinned by acceptance test `update_rejects_role_change_with_immutable_field_changed` (CH-01 P4) | **D-new-22 (remediated 2026-04-27 via CH-01)** | N/A |
| Parallelized Sessions | `AgentProfile.parallelize: u32` | honored | `nodes.rs:301` | — | wrap |
| Participation | HAS_AGENT edge, Project → Agent | honored | `edges.rs:196-198` | — | N/A |
| Identity (Emergent) | 4-field Identity node (self_description/lived/witnessed/embedding) | honored | CH-16 ships full struct + 4 repo methods + migration 0009 + eager creation in `apply_agent_creation` (ADR-0038) | **D-new-01 HIGH** (remediated) | N/A |

### `concepts/coordination.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Storage backend | v0 uses SurrealDB (RocksDB-embedded; remote ≥ 2.0 supported); architecture is configurable via `Arc<dyn domain::Repository>`; 7-criterion conforming-backend contract ratified | honored | concept refreshed at `coordination.md` §"Storage backend"; `store/Cargo.toml` has `surrealdb` dep; 9 SurrealQL migrations 0001–0009; `Repository` trait at `domain/src/repository.rs` (~36 async methods) is the swap surface | **D-new-02 remediated 2026-04-28 via CH-03 / ADR-0042** | N/A |
| Event hybrid | State + event sourcing | partially-honored | AgentEvent stream exists; no unified replay query | — | wrap (AgentEvent) |
| Memory types | 4 types: user/feedback/project/reference | silent-in-code | Memory has `tags: Vec<String>` but no type enum | D-new-28 | N/A |

### `concepts/human-agent.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| No Identity | Human Agents have no system-computed Identity | honored | CH-16 BOTH-guards: defensive at `Repository::upsert_identity` + preventive at `apply_agent_creation` (ADR-0039) | D-new-23 (remediated) | N/A |
| Channel props | channel_id/type/address/status/priority/metadata | partially-honored | `nodes.rs:754-768` has id/agent_id/kind/handle/created_at (missing address/status/priority/metadata) | D-new-24 | N/A |
| HAS_CHANNEL edge | Human → Channel | honored | `edges.rs:126-130` | — | N/A |

### `concepts/ontology.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 37 node types | Exactly 37 NodeKind variants | honored | `nodes.rs:41-136` | — | N/A |
| Identity ontology | 4-field shape per spec | honored | CH-16 ships full struct (ADR-0038 §D38.2) — field names match `agent.md` lines 326-327 verbatim | D-new-01 (remediated) | N/A |
| InboxObject/OutboxObject | Carry AgentMessage value objects | silent-in-code | minimal (agent_id, created_at) | D-new-25 | N/A |
| 69 edge types | Per M4/P1 claim | partially-honored | actual count needs recount; docstring claims 69 | D-new-21 | N/A |
| Grant shape | holder/action/resource/descends_from/delegable | honored | `nodes.rs:597-612` | — | N/A |
| AuthRequestState 9 variants | Draft/Pending/InProgress/Approved/Denied/Partial/Expired/Revoked/Cancelled | honored | `nodes.rs:677-690` | — | N/A |

### `concepts/organization.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Org node fields | vision/mission/consent_policy/default_audit_class/authority_templates_enabled/defaults_snapshot/system_agents | honored | `nodes.rs:335-383` | — | wrap (defaults_snapshot) |
| Permission hierarchy | Org caps project caps agent (top-down ceiling) | honored | `engine.rs:235` step_2a_ceiling | — | N/A |

### `concepts/phi-core-mapping.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Session wrap | baby-phi Session wraps phi_core::Session | honored | `nodes.rs:841+ inner: PhiCoreSession` | — | wrap |
| LoopRecord wrap | baby-phi LoopRecordNode wraps phi_core::LoopRecord | honored | `nodes.rs:909+` | — | wrap |
| Turn wrap | baby-phi TurnNode wraps phi_core::Turn | honored | `nodes.rs:922+` | — | wrap |
| AgentProfile wrap | baby-phi AgentProfile holds phi_core blueprint | honored | `nodes.rs:304` | — | wrap |
| ModelConfig/ToolDefinition reuse | phi-core types wrapped at node tier | partially-honored | scaffolds at `nodes.rs:942+`; full wrap deferred | D1.3 (related) | wrap (planned) |
| agent_loop direct-reuse | baby-phi calls phi_core::agent_loop for execution | honored | runtime call site at `launch.rs::spawn_agent_task` (CH-02 P3); `tokio::join!(agent_fut, drain_fut)` drives event flow into `BabyPhiSessionRecorder`; provider via `MockProvider` per ADR-0032 | **D4.2 (remediated 2026-04-24 via CH-02)** | direct-reuse |

### `concepts/project.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Project fields | name/description/goal/status/shape/token_budget/tokens_spent/objectives/key_results | honored | `nodes.rs:407-435` | — | N/A |
| OKRs | Objective + KeyResult as embedded value objects | honored | `Vec<Objective>`, `Vec<KeyResult>` | — | N/A |
| Status | 4 variants: Planned/InProgress/OnHold/Finished | honored | `nodes.rs:450-456` | — | N/A |
| Task | Task with full field set + 7-state flow | silent-in-code | id-only scaffold | D-new-26 | N/A |
| Shapes A/B/C/D, E forbidden | Multi-scope enforcement | honored | `in_memory.rs:1350` enforces Shape B 2-owner | — | N/A |

### `concepts/system-agents.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Two v0 system agents per org | memory-extraction + agent-catalog at org creation | honored | `orgs/create.rs:366-415` | — | wrap |
| Memory-extraction listener fires on session_end | Body reads transcript, writes memories | honored at v0 (heuristic body) | CH-21 shipped: `MemoryExtractionListener::on_event` body at [`listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs) mints 1 Memory per non-aborted SessionEnded, derives tags from session, decides `{private,public}` scope, upserts Identity (`witnessed.memories_extracted += 1`), emits `platform.memory.extracted` + `platform.identity.updated` audits, calls `record_system_agent_fire`. LLM-driven supervisor agent body deferred to **M6-DEFERRED-04** per ADR-0040 § Out-of-Scope (4-pool routing + grant enforcement land with the LLM body). | D4.2 (existing) + **D6.1 remediated 2026-04-28 via CH-21** | direct-reuse (deferred to M6) |
| Agent-catalog listener fires on 8 events | Body upserts catalog rows | honored | CH-22 shipped: `AgentCatalogListener::on_event` body at [`listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs) upserts the catalog row on every trigger variant; `SessionAborted` is a documented no-op. Six production emit sites wired (ADR-0035 D35.5): agents/create + agents/update + system_agents/add + disable + archive + orgs/create. Acceptance test `agent_create_populates_catalog_and_advances_runtime_status_tile` verifies the HTTP path lights up the row. | D4.2 (existing) | N/A |
| Runtime-status telemetry | queue_depth, last_fired_at populated | honored | CH-22 + CH-21 shipped both call sites: `AgentCatalogListener` calls `record_system_agent_fire` on every fire (catalog tile advances), and `MemoryExtractionListener` calls it on every non-aborted fire (memory-extractor tile advances). Both system-agent runtime-status tiles are populated. Tests `agent_catalog_listener_runtime_status_tile_advances_on_fire` (CH-22) + `memory_extraction_listener_advances_runtime_status_tile_for_extractor` (CH-21) + acceptance `scenario_3_disabled_extractor_skips_both_extraction_and_telemetry` (CH-21) pin the call sites. | **D6.1 remediated 2026-04-28 via CH-21** | N/A |
| Disable/archive durable | active:false, archived_at | honored | migration 0007 added `agent.active: bool DEFAULT true` + `agent.archived_at: option<string>`; repo methods `set_agent_active` + `set_agent_archived_at` flip them; system-agent `disable.rs` + `archive.rs` handlers wired (durable write BEFORE audit emit per ADR-0034 D34.4); acceptance tests verify both round-trip via `repo.get_agent` | **D6.5 (remediated 2026-04-27 via CH-01)** | N/A |

### `concepts/token-economy.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Worth formula | avg_rating × earned − consumed | silent-in-code | no token fields on Agent | D-new-27 | N/A |
| Rating window size=20 | Rolling window fields | silent-in-code | no rating fields | D-new-27 | N/A |
| Intern → Contract carry-forward | Cumulative token tracking | silent-in-code | no token fields | D-new-27 | N/A |

### `concepts/permissions/README.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 5-component grant | (subject, action, resource, constraints, provenance) | honored | `Grant` struct | — | N/A |
| Provenance traversal | Bootstrap axiom chain | partially-honored | `auth_request_id` stored; no traversal | D-new-14 | N/A |

### `concepts/permissions/01-resource-ontology.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 9 fundamental classes | `Fundamental::ALL[9]` | honored | `fundamentals.rs:44-54` | — | N/A |
| 8 composite classes | `Composite::ALL[8]` | honored | `composites.rs:50-59` | — | N/A |
| Composite `#kind:` auto-tag | `kind_tag()` canonical form | honored | `composites.rs:81-92` | — | N/A |
| Instance identity tag `{kind}:{id}` | Auto-added at creation | honored | CH-06 wired emission at 10+ creation handlers via `auto_tags_for(kind, id)` (ADR-0037) | D-new-11 (remediated) | N/A |
| 3 ownership edges | OWNED_BY / CREATED / ALLOCATED_TO | honored | `edges.rs` | — | N/A |
| Catalogue as Step 0 precondition | catalogue.contains() gates Step 0 | honored | `engine.rs:130-144` | — | N/A |

### `concepts/permissions/02-auth-request.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 9 AuthRequest states | Draft..Cancelled | honored | `nodes.rs` AuthRequestState | — | N/A |
| Per-state ACL matrix | Requestor/approvers/owner/admins have distinct access per state | silent-in-code | no per-state ACL checks | D-new-12 | N/A |
| Slot independence | Partial approvals → scoped partial grants | honored | `auth_requests/state.rs:32-50` | — | N/A |
| Per-resource slots | `resource_slots: Vec<ResourceSlot>` | honored | `nodes.rs` | — | N/A |
| allocate vs transfer cardinality | allocate additive, transfer exclusive | **honored** | CH-08 (ADR-0052 §D52.1 + §D52.5 + §D52.6) ships `Repository::apply_transfer_grant` compound-tx primitive at `repository.rs:1397` (atomic three-write tx: rewrite OWNED_BY edge + revoke sender's grant + mint recipient's grant per concept-doc 02 line 206) with adapter impls at `in_memory.rs:1868` (Mutex-guarded) and `store/src/repo_impl.rs:2691` (BEGIN/COMMIT block); Allocate path remains additive via existing `create_grant` boundary (sender's grant untouched); 6 atomicity tests + 3 SurrealStore integration tests verify the compound-tx semantics. Forward-defensive primitive — no production caller wired today (`Action::Transfer` has zero runtime mint sites; first M6+ transfer-flow chunk consumes). | **D-new-13 ✓** | N/A |
| System Bootstrap Template + system:genesis | Hardcoded axioms | partially-honored | `TemplateKind::SystemBootstrap` exists; `system:genesis` chain traversal missing | D-new-14 | N/A |
| 2-tier retention (active 90d + archive) | `active_until` + `is_archive_eligible` + retrieval gating | partially-honored | math in `retention.rs`; no archival action / retrieval gate | D-new-15 | N/A |

### `concepts/permissions/03-action-vocabulary.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Standard action vocabulary | 34 named actions (concept-doc recount; was "33" in original audit row) | **honored** | `domain::permissions::action::Action` enum with 35 variants (34 canonical + Wildcard); `ALL[35]` / `CANONICAL[34]` iteration constants; `as_str` / `TryFrom<&str>` / `Display` / `FromStr`; carriers `Grant.action` + `Manifest.actions` + `ToolAuthorityManifest.actions` are `Vec<Action>` | **D-new-09 (remediated 2026-04-29 via CH-04)** | N/A |
| Action × Fundamental matrix | Only compatible actions per fundamental | **honored** | `Action::applies_to(Fundamental) -> bool` encodes the 9×10 matrix from `concepts/permissions/03-action-vocabulary.md` lines 27–37; `Action::applies_to_composite(Composite)` derives via constituents() per concept-doc line 39; exhaustive 306-cell test transcribes the doc verbatim | **D-new-10 (remediated 2026-04-29 via CH-04)** | N/A |
| `allocate` umbrella | covers delegate/approve/escalate/revoke etc. | **honored** | CH-08 (ADR-0052 §D52.2 + §D52.3) ships typed `AllocateRefinement { no_further_delegation: bool, max_depth: Option<u8> }` at `domain::permissions::AllocateRefinement` (`permissions/allocate_refinement.rs`) per concept-doc 02 line 197 + concept-doc 03 line 54 (refinement-as-constraint framing); `Grant.allocate_refinement: Option<AllocateRefinement>` field at `model/nodes.rs:701` with `#[serde(default)]` shielding (mirrors CH-11 D48.1 + CH-13 D50.5 field-add precedents); 3 unit tests cover round-trip + legacy-grant decode-as-None. Engine-side enforcement is forward-defensive — typed-field structural-boundary closes the typing-half of D-new-29; first M6+ chunk wiring engine-enforcement consumes the typed field. | **D-new-29 ✓** | N/A |
| Over-declaration principle | Manifest = max reach; intersection with grants | honored | `manifest.rs` + `engine.rs` Step 1 | — | N/A |

### `concepts/permissions/04-manifest-and-resolution.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 8-step algorithm (0+1+2+2a+3+4+5+6) | Matches pseudocode | honored | `engine.rs` step_N functions | — | N/A |
| Step 0 hard-gates | Catalogue miss → Denied | honored | `engine.rs:130-144` | — | N/A |
| Step 3 hard-match | Missing reach match → Denied | honored | `engine.rs:88-92` | — | N/A |
| Step 6 consent gating | Missing consent → Pending | honored | `engine.rs:106` | — | N/A |
| Decision outcomes | Allowed/Denied/Pending | honored | `decision.rs:99-130` | — | N/A |
| **All steps hard-gate** (not advisory) | Any Denied → refuse | contradicted | launch.rs:221 advisory-only | **D4.1** (existing HIGH) | N/A |
| Publish-time manifest validator | Rejects missing fundamentals, reserved-namespace writes | **honored** | `validate_published_manifest` at `domain::permissions::manifest::validator` shipped with 4 hard rules + 3 warnings; Repository-level guard wired at both `SurrealStore::create_tool_authority_manifest` + `InMemoryRepository::create_tool_authority_manifest` via `RepositoryError::ManifestValidation { source }` variant; 38 tests cover all rejection classes including 81-cell Constraint × Fundamental matrix transcribed verbatim | **D-new-07 (remediated 2026-04-29 via CH-05)** | N/A |
| 5-tier scope cascade | Project → Org → base_project → base_org → intersection | **honored** | CH-07 (ADR-0051) ships full 2-tier cascade + intersection fallback at `engine.rs::step_5_scope_resolution` (`cascade_multi_scope` + `cascade_single_scope` + `cascade_intersection_fallback`); base_project / base_org tie-breakers use lexicographic-min placeholder per ADR-0051 §D51.2/§D51.3 (M6+ deferral via `D-CH07-FOLLOWUP-01`); 10 unit tests + 6 acceptance tests cover the 5 distinct outcomes (`tests/multi_scope_cascade_acceptance.rs`) | **D-new-06 ✓** | N/A |
| Provenance chain to bootstrap | Traversal via `descends_from` | partially-honored | field exists; walker missing | D-new-14 | N/A |
| Composite expansion invariant | Memory vs Session disambiguated by `#kind:` | honored | `expansion.rs:140-146` | — | N/A |

### `concepts/permissions/05-memory-sessions.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Memory selector predicates (tags) | Full tag-predicate DSL | honored | CH-06 ships 6 predicates + 3 combinators via pest-PEG (ADR-0036) | **D-new-03** (remediated) | N/A |
| Memory tag vocab agent/project/org/#public | Tags field on Memory | honored | `nodes.rs:791 tags: Vec<String>` | — | N/A |
| store/recall/delete actions | Memory operations | silent-in-code | no recall tool / store action | D-new-16 | N/A |
| Default memory-recall grant | System-provenance grant on each agent | concept-aspirational | Memory contract deferred to M6 (C-M6-1) | — | N/A |
| Supervisor extraction 2 grants | Reads subordinate sessions + stores extractions | silent-in-code | listener stub | D4.2 + D6.1 (existing) | N/A |
| Session tags frozen at creation (except lifecycle) | No grant mints [modify] on structural tags | **honored** | CH-12 (ADR-0049) ships: publish-time Rule E in `validate_published_manifest` rejects `[Modify]` on composites whose `target_kinds` overlaps `reserved_namespace_prefixes()`; runtime `validate_tag_write_on_session` + `SESSION_FROZEN_TAG_PREFIXES` + `FrozenTagViolation` at `domain::permissions::manifest::validator`; `tool.frozen_tag_write_rejected` (Alerted) audit-event builder at `domain::audit::events::m5_2::tool_authority` per F5.B; Repository trait docstring documents the validator + audit-event pairing as a precondition for any future tag-write method | **D-new-08 ✓** | N/A |
| Session tag vocabulary | agent/project/org/task/delegated_from/role_at_creation/agent_kind/#archived/#active | **partially-honored** | governance fields plus the full session-tag prefix list at `domain::permissions::manifest::validator::SESSION_FROZEN_TAG_PREFIXES` (10 prefixes: `#kind:`, `session:`, `agent:`, `project:`, `org:`, `task:`, `delegated_from:`, `role_at_creation:`, `agent_kind:`, `derived_from:`); CH-12 closes the **immutability-axis** by enforcing rejection of `[Modify]` on every prefix; the **emission-axis** remains aspirational — only `#kind:session` + `session:<id>` are auto-emitted today via `Composite::auto_tags_for("session", id)`; the 6 M6+ categories ship as forward-defensive entries in the const, with full emission deferred to a future M6+ "Session structural-tag emission" chunk (tracked as recommended LOW drift `D-CH12-FOLLOWUP-01` per plan §10) | **D-new-08 ✓** (immutability axis) | wrap |
| Templates A/B/C/D auto-fire grants | Shipped as pure fns + listeners | honored | `templates/a,b,c,d.rs` + listeners | — | N/A |
| Worked examples (4 scenarios) | Engine computes Allowed/Denied | honored | `engine.rs` full pipeline | — | N/A |
| Shape E forbidden | Enforced at project creation | honored | `in_memory.rs:1350` | — | N/A |

### `concepts/permissions/06-multi-scope-consent.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Shapes A/B/C/D valid; E forbidden | Hard schema constraint | honored | ProjectShape enum | — | N/A |
| Unified resolve_scope cascade | project → org with tie-breaker | **honored** | CH-07 (ADR-0051) ships the `resolve_scope` 2-tier branching at `engine.rs::cascade_multi_scope` (project-tier match-count branch → org-tier match-count branch → intersection fallback) reading session scopes from `ctx.session_org_tags` / `ctx.session_project_tags` per ADR-0051 §D51.4; intersection-fallback ceiling re-clamp + `DeniedReason::IntersectionEmpty` per ADR-0051 §D51.5 + §D51.7 | **D-new-06 ✓** | N/A |
| 3 consent policies | Implicit / One-Time / Per-Session | honored | `ConsentPolicy` enum | — | N/A |
| Implicit consent: auto-issue on edge | Template listeners fire | honored | listeners wired | — | N/A |
| One-Time consent: Consent node lifecycle | Requested → Acknowledged | honored | 6-state machine + per-transition repo + sweeper shipped at CH-10 (ADR-0047) | **D-new-05** ✓ | N/A |
| Per-Session consent: subordinate_required | Grant flag + real-time approval flow | honored | CH-11 shipped: `Grant.approval_mode: ApprovalMode` + engine `step_6_consent_gating` real body branches on `approval_mode × ConsentPolicy × ConsentState` per ADR-0048 D48.5/D48.6; per-policy minters + `Repository::request_consent` ship at `domain::consents::minters` + `repository.rs`; `consent.requested` audit event + `(subordinate, org, Option<session_id>)` ConsentIndex; Channel notification remains M6+ per concept-doc line 416 | **D-new-17** ✓ | N/A |
| Consent node full field list | consent_id/agent_id/scope/state/requested_at/responded_at/revocable/provenance | honored | 11-field shape shipped at CH-09 (ADR-0045) | **D-new-04** ✓ | N/A |
| Contractor model | base_org ceiling does not reach cross-scope | **honored** | CH-07 (ADR-0051 §D51.6) ships the membership-bounded ceiling clamp at `engine.rs::step_2a_ceiling` — non-empty `session_org_tags` filters Organization-tier ceilings to those whose `holder == PrincipalRef::Organization(o) ∧ session_org_tags.contains(&o)`; concept-doc 06 line 162 verbatim invariant enforced; Project / Agent ceilings pass through unchanged; 4 unit tests + Scenario 7 acceptance test (`tests/multi_scope_cascade_acceptance.rs`) | **D-new-20 ✓** | N/A |

### `concepts/permissions/07-templates-and-tools.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Templates A–E via adoption AR | Build adoption + listener fires | honored | templates + listeners | — | N/A |
| Template A [read, inspect, list] on project | Issued on HAS_LEAD | honored | `templates/a.rs` | — | N/A |
| audit_class composition: strictest wins | Org / template AR / override composition | honored | composer at `permissions/audit_composition.rs` (`compose_audit_class` + `compose_audit_class_with_source` + `AuditClassSource` enum); `Grant.audit_class` denormalisation at `model/nodes.rs:693`; 3 fire listeners wire `resolve_composed_audit_class` at `events/listeners.rs:303,433,552`; audit-event builders at `audit/events/m4/templates.rs:30` (template_a) + `m5/templates.rs:29,77` (template_c/d) accept composed class + emit `audit_class_source` snake-case string in diff per ADR-0050 §D50.6 | **D-new-19 ✓** | N/A |
| Standard Org Template config | tools_allowlist/resource_catalogue/etc. as embedded config | concept-aspirational | minimal Org node; template via adoption ARs only | D-new-30 | N/A |
| Standard Project Template config | filesystem/session/memory grants | concept-aspirational | minimal Project scaffold | D-new-30 | N/A |
| 14 Tool Authority Manifest examples | Declared shape per tool | partially-honored | ToolAuthorityManifest fields partial | D4.3 + D-new-07 | N/A |
| Manifest validation at publish-time | Reject invalid declarations | **honored** | Same as the §04 row above — validator + repo guard land at CH-05 | **D-new-07 (remediated 2026-04-29 via CH-05)** | N/A |

### `concepts/permissions/08-worked-example.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Shape A/B/C resolution per worked examples | Engine computes expected Decisions | **honored** | CH-07 (ADR-0051) — engine cascade body covers Shape A (`cascade_single_scope` back-compat) + Shape B/C (`cascade_multi_scope` 2-tier + intersection fallback); worked-example Scenarios 4 / 5 / 6 covered by `tests/multi_scope_cascade_acceptance.rs::scenario_{4,5,6}_*`; Shape D system-session covered by `shape_d_system_session_org_tier_wins`; Shape A regression covered by `shape_a_baseline_empty_session_tags_project_tier_wins` | **D-new-06 ✓** | N/A |
| Contractor scenario | base_org bounded | **honored** | CH-07 (ADR-0051 §D51.6) — concept-08 §"Step 7" Scenario 7 (`contractor-x-9` with base_org=Gamma reading `acme-website-redesign`, session tagged `[org:acme]` only) covered by `tests/multi_scope_cascade_acceptance.rs::scenario_7_contractor_reads_acme_session_project_tier_wins`; Gamma ceiling structurally excluded by the membership filter in `step_2a_ceiling`; project-tier resolution wins as concept-doc specifies | **D-new-20 ✓** | N/A |
| Shape E forbidden recovery | Rejected at creation | honored | `in_memory.rs:1350` | — | N/A |
| Ad-hoc AR + revocation cascade | Revoke walks grants by provenance | silent-in-code | no cascade code | **D-new-18** | N/A |

### `concepts/permissions/09-selector-grammar.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| PEG grammar (atoms + predicates + composition) | Full recursive-descent parser | honored | CH-06 ships pest-PEG with 13 productions matching concept-09 lines 58-102 | **D-new-03** (remediated) | N/A |
| `tags contains/intersects/any_match/subset_of` | Tag-predicate primitives | contradicted | only Exact/Prefix/KindTag | **D-new-03** | N/A |
| AND/OR/NOT logical composition | Combinators | honored | CH-06 ships `BoolExpr::Or/And/Not` with NOT > AND > OR precedence + parens; 6 unit tests pin precedence | **D-new-03** (remediated) | N/A |
| Reserved namespace write rejection | Publish-time validator denies | **honored** | Rule C of `validate_published_manifest` rejects `[Modify]` on bare `tag` resource (CH-05 — cleanest publish-time discriminator); full reserved-namespace prefix list (`#kind:`, `delegated_from:`, `derived_from:`, plus auto-generated `{kind}:` per `Composite::ALL`) ships via `reserved_namespace_prefixes()` for downstream consumers; **CH-12 (ADR-0049) extends the rule set to a fifth Rule E** that closes the composite-case gap by rejecting `[Modify]` on a composite whose `target_kinds` overlaps the reserved-namespace prefix list — Rule C still fires for `[Modify] × bare tag`; Rule E covers `[Modify] × composite × target_kinds`. CH-05 + CH-12 together close both publish-time discriminators. | **D-new-31 (remediated 2026-04-29 via CH-05)**, **D-new-08 ✓ (composite case via CH-12)** | N/A |

---

## Coverage statement

All 20 concept docs walked. All major § headings produced at least
1 matrix row (most produced 3–6). Denominator for
documentation-vs-concept-docs confidence: **20/20 docs audited = 100%**.

Every row with `contradicted` / `partially-honored` / `silent-in-code`
status is either:
- Already covered by an existing drift (D1.1–D7.6), OR
- Generating a new drift (D-new-01 through D-new-31), OR
- Flagged `concept-aspirational` with explicit deferral note (not a drift).

No unclassified rows. Discovery phase satisfied the close invariant that
every concept claim has an explicit classification + covering pointer.
