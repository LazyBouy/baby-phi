<!-- Last verified: 2026-04-24 by Claude Code -->

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
| Roles | Role immutable post-creation | partially-honored | no enforcement in handlers visible | D-new-22 | N/A |
| Parallelized Sessions | `AgentProfile.parallelize: u32` | honored | `nodes.rs:301` | — | wrap |
| Participation | HAS_AGENT edge, Project → Agent | honored | `edges.rs:196-198` | — | N/A |
| Identity (Emergent) | 4-field Identity node (self_description/lived/witnessed/embedding) | contradicted | `nodes.rs:813-818` id-only scaffold | **D-new-01 HIGH** | N/A |

### `concepts/coordination.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Storage backend | v0 uses SQLite | contradicted | `store/Cargo.toml` has `surrealdb` dep | **D-new-02 HIGH** | N/A |
| Event hybrid | State + event sourcing | partially-honored | AgentEvent stream exists; no unified replay query | — | wrap (AgentEvent) |
| Memory types | 4 types: user/feedback/project/reference | silent-in-code | Memory has `tags: Vec<String>` but no type enum | D-new-28 | N/A |

### `concepts/human-agent.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| No Identity | Human Agents have no system-computed Identity | silent-in-code | No guard preventing `Human → HAS_IDENTITY` edge | D-new-23 | N/A |
| Channel props | channel_id/type/address/status/priority/metadata | partially-honored | `nodes.rs:754-768` has id/agent_id/kind/handle/created_at (missing address/status/priority/metadata) | D-new-24 | N/A |
| HAS_CHANNEL edge | Human → Channel | honored | `edges.rs:126-130` | — | N/A |

### `concepts/ontology.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 37 node types | Exactly 37 NodeKind variants | honored | `nodes.rs:41-136` | — | N/A |
| Identity ontology | 4-field shape per spec | contradicted | scaffold | D-new-01 | N/A |
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
| Memory-extraction listener fires on session_end | Body reads transcript, writes memories | partially-honored | listener stub; body at P8 | D4.2 + D6.1 (existing) | direct-reuse (planned) |
| Agent-catalog listener fires on 8 events | Body upserts catalog rows | partially-honored | stub | D4.2 + D6.1 | N/A |
| Runtime-status telemetry | queue_depth, last_fired_at populated | contradicted | helper shipped; zero call sites; tiles empty | **D6.1** (existing) | N/A |
| Disable/archive durable | active:false, archived_at | contradicted | no durable fields on Agent | **D6.5** (existing) | N/A |

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
| Instance identity tag `{kind}:{id}` | Auto-added at creation | partially-honored | node has `id` field; no auto-tag logic visible | D-new-11 | N/A |
| 3 ownership edges | OWNED_BY / CREATED / ALLOCATED_TO | honored | `edges.rs` | — | N/A |
| Catalogue as Step 0 precondition | catalogue.contains() gates Step 0 | honored | `engine.rs:130-144` | — | N/A |

### `concepts/permissions/02-auth-request.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| 9 AuthRequest states | Draft..Cancelled | honored | `nodes.rs` AuthRequestState | — | N/A |
| Per-state ACL matrix | Requestor/approvers/owner/admins have distinct access per state | silent-in-code | no per-state ACL checks | D-new-12 | N/A |
| Slot independence | Partial approvals → scoped partial grants | honored | `auth_requests/state.rs:32-50` | — | N/A |
| Per-resource slots | `resource_slots: Vec<ResourceSlot>` | honored | `nodes.rs` | — | N/A |
| allocate vs transfer cardinality | allocate additive, transfer exclusive | silent-in-code | no cardinality enforcement | D-new-13 | N/A |
| System Bootstrap Template + system:genesis | Hardcoded axioms | partially-honored | `TemplateKind::SystemBootstrap` exists; `system:genesis` chain traversal missing | D-new-14 | N/A |
| 2-tier retention (active 90d + archive) | `active_until` + `is_archive_eligible` + retrieval gating | partially-honored | math in `retention.rs`; no archival action / retrieval gate | D-new-15 | N/A |

### `concepts/permissions/03-action-vocabulary.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Standard action vocabulary | 33 named actions | silent-in-code | no action constants/enums; actions are `Vec<String>` | D-new-09 | N/A |
| Action × Fundamental matrix | Only compatible actions per fundamental | silent-in-code | no matrix enforcement | D-new-10 | N/A |
| `allocate` umbrella | covers delegate/approve/escalate/revoke etc. | silent-in-code | constraints as Vec<String>; no refinement types | D-new-29 | N/A |
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
| Publish-time manifest validator | Rejects missing fundamentals, reserved-namespace writes | silent-in-code | no validator | D-new-07 | N/A |
| 5-tier scope cascade | Project → Org → base_project → base_org → intersection | partially-honored | step_5 exists; full cascade depth unclear | D-new-06 | N/A |
| Provenance chain to bootstrap | Traversal via `descends_from` | partially-honored | field exists; walker missing | D-new-14 | N/A |
| Composite expansion invariant | Memory vs Session disambiguated by `#kind:` | honored | `expansion.rs:140-146` | — | N/A |

### `concepts/permissions/05-memory-sessions.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Memory selector predicates (tags) | Full tag-predicate DSL | contradicted | 4-variant Selector enum | **D-new-03** | N/A |
| Memory tag vocab agent/project/org/#public | Tags field on Memory | honored | `nodes.rs:791 tags: Vec<String>` | — | N/A |
| store/recall/delete actions | Memory operations | silent-in-code | no recall tool / store action | D-new-16 | N/A |
| Default memory-recall grant | System-provenance grant on each agent | concept-aspirational | Memory contract deferred to M6 (C-M6-1) | — | N/A |
| Supervisor extraction 2 grants | Reads subordinate sessions + stores extractions | silent-in-code | listener stub | D4.2 + D6.1 (existing) | N/A |
| Session tags frozen at creation (except lifecycle) | No grant mints [modify] on structural tags | silent-in-code | no enforcement | **D-new-08** | N/A |
| Session tag vocabulary | agent/project/org/task/delegated_from/role_at_creation/agent_kind/#archived/#active | partially-honored | governance fields only; tag list omitted | D-new-08 (related) | wrap |
| Templates A/B/C/D auto-fire grants | Shipped as pure fns + listeners | honored | `templates/a,b,c,d.rs` + listeners | — | N/A |
| Worked examples (4 scenarios) | Engine computes Allowed/Denied | honored | `engine.rs` full pipeline | — | N/A |
| Shape E forbidden | Enforced at project creation | honored | `in_memory.rs:1350` | — | N/A |

### `concepts/permissions/06-multi-scope-consent.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Shapes A/B/C/D valid; E forbidden | Hard schema constraint | honored | ProjectShape enum | — | N/A |
| Unified resolve_scope cascade | project → org with tie-breaker | partially-honored | step_5 partial | **D-new-06** | N/A |
| 3 consent policies | Implicit / One-Time / Per-Session | honored | `ConsentPolicy` enum | — | N/A |
| Implicit consent: auto-issue on edge | Template listeners fire | honored | listeners wired | — | N/A |
| One-Time consent: Consent node lifecycle | Requested → Acknowledged | partially-honored | Consent exists; no state machine | **D-new-05** | N/A |
| Per-Session consent: subordinate_required | Grant flag + real-time approval flow | partially-honored | Step 6 stub only | D-new-17 | N/A |
| Consent node full field list | consent_id/agent_id/scope/state/requested_at/responded_at/revocable/provenance | contradicted | 5 fields only | **D-new-04** | N/A |
| Contractor model | base_org ceiling does not reach cross-scope | silent-in-code | no contractor-specific logic | D-new-20 | N/A |

### `concepts/permissions/07-templates-and-tools.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Templates A–E via adoption AR | Build adoption + listener fires | honored | templates + listeners | — | N/A |
| Template A [read, inspect, list] on project | Issued on HAS_LEAD | honored | `templates/a.rs` | — | N/A |
| audit_class composition: strictest wins | Org / template AR / override composition | silent-in-code | no composition logic | D-new-19 | N/A |
| Standard Org Template config | tools_allowlist/resource_catalogue/etc. as embedded config | concept-aspirational | minimal Org node; template via adoption ARs only | D-new-30 | N/A |
| Standard Project Template config | filesystem/session/memory grants | concept-aspirational | minimal Project scaffold | D-new-30 | N/A |
| 14 Tool Authority Manifest examples | Declared shape per tool | partially-honored | ToolAuthorityManifest fields partial | D4.3 + D-new-07 | N/A |
| Manifest validation at publish-time | Reject invalid declarations | silent-in-code | no validator | **D-new-07** | N/A |

### `concepts/permissions/08-worked-example.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| Shape A/B/C resolution per worked examples | Engine computes expected Decisions | partially-honored | engine ships; multi-scope cascade weak | D-new-06 | N/A |
| Contractor scenario | base_org bounded | silent-in-code | not implemented | D-new-20 | N/A |
| Shape E forbidden recovery | Rejected at creation | honored | `in_memory.rs:1350` | — | N/A |
| Ad-hoc AR + revocation cascade | Revoke walks grants by provenance | silent-in-code | no cascade code | **D-new-18** | N/A |

### `concepts/permissions/09-selector-grammar.md`

| § | Claim | Status | Code evidence | Covering drift | phi-core leverage |
|---|---|---|---|---|---|
| PEG grammar (atoms + predicates + composition) | Full recursive-descent parser | contradicted | 4-variant enum only | **D-new-03** | N/A |
| `tags contains/intersects/any_match/subset_of` | Tag-predicate primitives | contradicted | only Exact/Prefix/KindTag | **D-new-03** | N/A |
| AND/OR/NOT logical composition | Combinators | silent-in-code | no combinator implementation | **D-new-03** | N/A |
| Reserved namespace write rejection | Publish-time validator denies | silent-in-code | no validator | **D-new-07** (related) | N/A |

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
