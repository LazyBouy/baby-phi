<!-- Last verified: 2026-04-28 by Claude Code -->

# CH-06 — Selector grammar (PEG tag-predicate DSL) + instance identity tags

**Plan file token:** `acd383e2` (generated via `openssl rand -hex 4`).
**Plan archive path (verbatim copy from `/root/.claude/plans/sharded-discovering-stearns.md`):** `baby-phi/docs/specs/plan/build/acd383e2-ch-06-selector-grammar-peg-and-instance-tags.md`. Archived at chunk-open Step 0 (2026-04-28).
**Chunk ID:** CH-06 (forward-scope §1 line 77; §4 dependency graph; §5 row line 338).
**Severity:** HIGH.
**Expected effort:** ~5–7 engineer-days.
**Chunks unblocked at close:** CH-07 (multi-scope cascade), CH-12 (frozen-tag enforcement), CH-15 (real permission-check gate at launch + memory tag-retrieval), `M6-DEFERRED-01` (Memory contract).

---

## Context

CH-06 is the largest single M5.2 chunk per forward-scope and "the missing piece bridging concept (`permissions/09-selector-grammar.md`: full PEG DSL) to code (`selector.rs`: 4-variant enum)". Two governance jobs land:

1. **D-new-03 (HIGH) closure** — ship the PEG-shaped selector grammar with all 6 predicates (`contains`, `intersects`, `any_match`, `subset_of`, `empty`, `non_empty`) + 3 combinators (`AND`/`OR`/`NOT`) + parens + precedence. The current `Selector` enum has only 4 flat variants (`Any`/`Exact`/`Prefix`/`KindTag`) and a hand-rolled string-slice parser — no logical composition, no tag-set intersection, no glob matching.
2. **D-new-11 (MEDIUM) closure** — wire `Composite::auto_tags(instance_id)` (already shipped at `composites.rs:119`, currently zero call sites) into every composite + relevant node creation path so each instance auto-emits its `{kind}:{instance_id}` self-identity tag. Without this, the new grammar can never match selectors like `tags contains session:s-9831` against real data.

The two halves are physically inseparable: the grammar is the *consumer* of instance tags; instance tags are the *target data* the grammar matches against. Shipping one without the other leaves a half-functional surface that the audit matrix flags and that downstream chunks (CH-12, CH-15, M6-DEFERRED-01) cannot build on.

**Per §4 dependency graph: NO hard prerequisites.** CH-06 is "isolated parser + matcher work" (forward-scope line 80). Bundles parser, matcher, instance-tag wiring, and 9-call-site verification.

**User-decided scope (locked at plan-review):**
1. **Unified chunk** (Q2 binding decision) — not split into CH-06a/b. Single seal closes both drifts.
2. **Parser library: pest** — declarative `.pest` grammar maps 1:1 to concept-09's normative PEG.
3. **Tag rollout: all 15 types in one migration 0008** — 11 composites + Memory + Session + AuthRequest + ExternalService.
4. **Existing-grant policy: semantic continuity** — keep `ResourceRef.uri: String`; new grammar's `parse_selector_or_uri()` handles legacy shapes via fast-path. Zero data migration.
5. **Backwards compatibility: keep + extend** — existing 4-variant URI shapes encoded as AST forms (or `LegacyUriShape` variant); 9 grant-mint call sites unchanged.

**Outcome:** D-new-03 + D-new-11 → `remediated`; concept-09 + concept-01 §"Instance Identity Tags" + concept-04 §"Refinement" + concept-05 §"Memory as Resource Class" all flip to `honored`; downstream chunks (CH-07, CH-12, CH-15, M6-DEFERRED-01) unblock.

---

## §1 — Context & principle

### Why this chunk

Concept doc `permissions/09-selector-grammar.md` (lines 58–102) is the **normative PEG**:
```peg
Selector     <- _ OrExpr _
OrExpr       <- AndExpr (_ "OR" _ AndExpr)*
AndExpr      <- NotExpr (_ "AND" _ NotExpr)*
NotExpr      <- "NOT" _ Predicate / Predicate
Predicate    <- "(" _ OrExpr _ ")" / TagPredicate
TagPredicate <- "tags" _ ContainsOp
ContainsOp   <- "contains" _ Tag / "intersects" _ TagSet / "any_match" _ TagGlob
              / "subset_of" _ SetRef / "empty" / "non_empty"
Tag          <- ReservedTag / NamespaceTag / LiteralTag
```

Operator precedence: NOT (right-assoc) > AND (left-assoc) > OR (left-assoc); parens override.

**Worked-parse examples (concept-09 lines 114–186):**
1. `tags contains project:acme-website-redesign AND tags contains #kind:session`
2. `tags intersects {session:s-9831, session:s-9832} AND tags contains org:acme`
3. `tags subset_of supervisors_tagging_scope(supervisor-7)`
4. `tags any_match "org:acme/eng/**" AND NOT tags contains #archived`

Concurrently, `permissions/01-resource-ontology.md §"Instance Identity Tags"` mandates `{kind}:{instance_id}` auto-emission at every composite instance creation (e.g. `session:s-9831`, `memory:m-4581`, `auth_request:req-7102`, `external_service:mcp-github-7`). The helper `Composite::auto_tags(&self, instance_id: &str) -> [String; 2]` exists at `composites.rs:119` and is unit-tested for every Composite variant — but a workspace grep shows **zero production call sites**.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* Applied: the PEG in concept-09 IS the spec. The implementation parser must be reviewable against it production-by-production, not approximated. The four worked-parse examples are golden test fixtures, not illustrative prose. Every composite type's `auto_tags()` call must land — partial closure of D-new-11 reproduces exactly the silent-drift pattern the chunk discipline forbids.

### Forward-scope reference

[§1 CH-06 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 77) + [§4 critical-path graph](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 284) + [§5 inventory row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 338) + [§7 Q2 split decision](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (line 378 — resolved at plan-review: unified).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"PEG Grammar" lines 58–102 | Full PEG with 13 productions (Selector / OrExpr / AndExpr / NotExpr / Predicate / TagPredicate / ContainsOp / Tag / TagSet / TagGlob / SetRef / Identifier / StringLiteral) | contradicted | honored |
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"Primary Predicates" | Six predicates: `contains` / `intersects` / `any_match` / `subset_of` / `empty` / `non_empty` | contradicted | honored |
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"Logical Composition" | NOT (right-assoc) > AND (left-assoc) > OR (left-assoc); parens override | silent-in-code | honored (precedence pinned by parser tests + 4 golden parse trees) |
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"Worked Parses" 1-4 | Four canonical parse trees | silent-in-code | honored (golden-AST tests) |
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"Reserved Namespace Enforcement" (read side) | Parser is permissive; manifest validator (publish-time) enforces reserved-tag write-restriction | partially-honored (engine reads `#kind:` but no parser surface) | honored at parser-side; write-restriction deferred to CH-07 (multi-scope cascade) per concept-09 itself |
| [`permissions/09`](../../v0/concepts/permissions/09-selector-grammar.md) | §"Non-Normative Notes" exclusions | No bracket char-classes, no time predicates, no numeric, no string-content match, no cross-instance joins | (out-of-scope) | (preserved out-of-scope; documented in ADR-0036 §Out-of-Scope) |
| [`permissions/01`](../../v0/concepts/permissions/01-resource-ontology.md) | §"Instance Identity Tags" | Every composite instance auto-emits `{kind}:{instance_id}` at creation; cannot be set/modified by agents | partially-honored (`auto_tags` helper exists; zero call sites) | **honored** (call sites wired in 11 composites + Memory + Session + AuthRequest + ExternalService) |
| [`permissions/04`](../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Formal Algorithm Step 3" | `selector_matches(g.resource.selector, call.target_tags, call.context)` | partially-honored (Step 3 calls `effective_matches` over 4-variant matcher) | honored (Step 3 calls `evaluate(ast, target_uri, target_tags, set_ref_registry)` over new AST) |
| [`permissions/04`](../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Refinement" (composite `#kind:` injection) | `resolve_grant` adds implicit `#kind:` selector for composite-class URIs | honored | honored (preserved; refinement now expressed in new grammar's `tags contains #kind:<name>`) |
| [`permissions/05`](../../v0/concepts/permissions/05-memory-sessions.md) | §"Memory as Resource Class" | Memory selectors use tag-predicate DSL (e.g. `tags intersects {session:..., project:...}`) | contradicted (no intersect predicate exists) | honored |
| [`permissions/05`](../../v0/concepts/permissions/05-memory-sessions.md) | §"Supervisor Extraction as Two Standard Grants" | Uses `tags contains project:website-redesign` (Grant A) + `tags subset_of supervisors_tagging_scope(supervisor-7)` (Grant B) | contradicted | honored at parser+matcher level; full SetRef registry wiring deferred to CH-15 — declared in ADR-0036 §"Out-of-Scope" |
| [`permissions/README.md`](../../v0/concepts/permissions/README.md) | (entry invariants) | Permissions subtree invariants | honored | honored (re-verified post-grammar) |
| [`phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md) | (phi-core surfaces) | No phi-core overlap for selector grammar | honored | honored (declared in §3 below; phi-core has no DSL parser) |

**Permissions subtree hook:** `permissions/README.md` cited as entry-invariants source.
**phi-core-mapping hook:** Cited; row asserts no phi-core overlap.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Rationale:** phi-core ships no selector / tag-predicate DSL. `phi_core::config::parser` is hand-rolled string-substitution for `${VAR}` interpolation in agent blueprints — completely unrelated to grammar parsing. Verified by Phase-1-of-exploration grep. The new `pest`-based parser ships in `domain` crate only.

**Expected import-count delta at chunk close:** **0 phi-core imports added or removed**.

**Positive close-audit greps** (must pass at seal):
```bash
grep -rn "use phi_core::" modules/crates/domain/src/permissions/ | wc -l   # expect 0
grep -nE "Contains|Intersects|AnyMatch|SubsetOf|Empty|NonEmpty" \
  modules/crates/domain/src/permissions/selector.rs   # ≥ 6
grep -nE "auto_tags\b" modules/crates/server/src/platform/ | wc -l   # ≥ 8 (one per major creation handler)
grep -n "fn evaluate\b" modules/crates/domain/src/permissions/selector.rs   # 1
```

**Forbidden-duplication greps** (must return 0):
```bash
grep -rn "^pub struct Selector\b\|^pub enum Selector\b" modules/crates/ | grep -v "domain/src/permissions/selector.rs"   # 0
bash scripts/check-phi-core-reuse.sh   # exit 0
```

`check-phi-core-reuse.sh` stays green — no new disallowed structs (`AgentProfile`, `ModelConfig`, `McpClient`, `Session`, etc.) introduced.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker? | Action |
|---|---|---|---|---|
| **A1** in-process state | `SelectorAst`, `Predicate`, `BoolExpr`, `TagPattern`, `SetRef`, `SetRefRegistry`. Registry is the only candidate for shared mutable state. Plan: registry is a `&'a dyn SetRefRegistry` field on `CheckContext`, **not** a process-global singleton | No (no `OnceCell`/`Mutex` introduced; registry caller-supplied) | — |
| **A2** IPC channels | None | No | — |
| **A3** pod-local resources | None | No | — |
| **A4** migration runner / first-apply race | New migration 0008 adds `tags ARRAY<string>` columns to up to 15 tables (11 composites + 4 nodes). Single column-add per table. Cross-ref existing CHK8S-D-05 (lock-missing). Additive column migrations are not aggravated | No (existing concern preserved) | Cross-ref CHK8S-D-05 |
| **A5** trait-shape requirement | `SetRefRegistry` is shipped as a trait (`fn resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>>`) so a future remote/Redis-backed implementation drops in. CheckContext field, not pod-global | No (trait-shaped from day one) | — |
| **A6** cross-pod state sharing | Instance tags written to durable SurrealDB columns; visible across pods via `SurrealStore::open_remote` (CH-K8S-PREP P-2). No in-process cache | No | — |
| **A7** audit hash-chain symmetry | No new audit writers. Selector parse-error cases produce `Decision::Denied` with a new `DeniedReason::SelectorParseError` variant — recorded by the existing emitter | No | — |

**Conforming criteria for ADR-0033 (CH-K8S-PREP) preserved:**
- D33.1 (`SessionRegistry`) — untouched.
- D33.2 (`SurrealStore::open_remote`) — migration 0008 is plain SurrealDB column addition; works on both `open_embedded` + `open_remote`.
- D33.3 (SIGTERM graceful shutdown) — no new `tokio::spawn` tasks added.
- D33.4 (`EventBus.shutdown` + `drain`) — no new emitters or listeners.

**Conclusion:** **K8s-neutral.** No new entries in `deferred-from-ch-k8s-prep.md`.

---

## §3.C — User-facing documentation impact map (post-Q9 / CH-22 binding)

| Tier | File | Touched? | Action |
|---|---|---|---|
| Architecture | [`m1/architecture/permission-check-engine.md`](../../v0/implementation/m1/architecture/permission-check-engine.md) | yes — Step 3 evaluator description shifts from 4-variant matcher to AST evaluator | (a) update in-chunk: rewrite Step 3 paragraph + add cross-ref to new selector-grammar architecture page |
| Architecture | `m5_2/architecture/selector-grammar.md` (NEW) | yes — design page for the grammar, AST shape, evaluator semantics, `SetRefRegistry` trait, migration impact | (a) create in-chunk |
| Architecture | [`m1/architecture/graph-model.md`](../../v0/implementation/m1/architecture/graph-model.md) | yes — adds `tags: Vec<String>` to 11 composite struct shapes + 4 node types; auto-emission mandatory at creation | (a) update in-chunk |
| Operations | `m5_2/operations/selector-grammar-operations.md` (NEW) | yes — parse-error code reference (P-001 unexpected token, P-002 unbalanced parens, P-003 unknown predicate, P-004 invalid glob, P-005 unknown set-ref); audit-event entry for `DeniedReason::SelectorParseError`; tag-emission ops note | (a) create in-chunk |
| Operations | [`m1/operations/permission-engine-operations.md`](../../v0/implementation/m1/operations/permission-engine-operations.md) (verify exists) | yes if exists — add new `DeniedReason::SelectorParseError` row + cross-ref to selector-grammar-operations | (a) update in-chunk if file present; otherwise (b) defer with successor `CH-19` (doc-only chunk slot) |
| User-guide | `m5_2/user-guide/selector-syntax-guide.md` (NEW) | yes — operator-facing reference: how to write `tags contains …`, `tags intersects {…}`, `any_match` glob syntax, AND/OR/NOT examples, the four worked parses from concept-09 | (a) create in-chunk |
| User-guide | [`m5/user-guide/cli-reference-m5.md`](../../v0/implementation/m5/user-guide/cli-reference-m5.md) | yes (if any CLI command exposes selector strings) — verify via `grep -rn "selector" modules/crates/cli/src/` | (a) update in-chunk if exposed; (b) defer with successor `CH-19` if not |
| User-guide | [`m5/user-guide/troubleshooting.md`](../../v0/implementation/m5/user-guide/troubleshooting.md) | yes — add "Selector parse error" troubleshooting section with the P-NNN codes from operations doc | (a) update in-chunk |

**Rule applied (Q9 binding):** every user-facing doc the chunk's code makes stale either updates in-chunk or carries an explicit defer with successor chunk. No open-ended deferrals.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| **D-new-03** | [`m5_1/drifts/D-new-03.md`](../../v0/implementation/m5_1/drifts/D-new-03.md) | HIGH | `discovered → in-chunk-plan → remediated` | PEG parser via pest + extended Selector enum + `matches()` for each predicate + AND/OR/NOT combinators. Lifecycle entry: `2026-04-28 — remediated — CH-06 chunk-seal; pest grammar shipped at modules/crates/domain/src/permissions/grammar.pest with 13 productions; SelectorAst evaluator + 4 worked-parse golden tests + property-based round-trip test pin the grammar.` |
| **D-new-11** | [`m5_1/drifts/D-new-11.md`](../../v0/implementation/m5_1/drifts/D-new-11.md) | MEDIUM | `discovered → in-chunk-plan → remediated` | `Composite::auto_tags()` (existing helper at composites.rs:119) wired into all 11 composite + 4 node creation paths; migration 0008 adds `tags ARRAY<string>` columns. Lifecycle entry: `2026-04-28 — remediated — CH-06 chunk-seal; auto_tags() called at 8+ creation handlers; 15-type integration test instance_tags_emission.rs pins emission per concept-01 §Instance Identity Tags.` |

**Index updates:**
- [`drifts/README.md`](../../v0/implementation/m5_1/drifts/README.md) — D-new-03 + D-new-11 row Status flips to `remediated`.
- [`drifts/_concept-audit-matrix.md`](../../v0/implementation/m5_1/drifts/_concept-audit-matrix.md) — flip 4 rows: PEG grammar (`contradicted → honored`), Predicates (`contradicted → honored`), AND/OR/NOT (`silent-in-code → honored`), Instance identity tag (`partially-honored → honored`).

**Mid-flight discovery hook:** if a phase reveals a third drift (e.g., manifest validator currently parses URI strings via legacy `Selector::parse` and would crash on new grammar; or a Memory-creation site outside `server/`), surface via `AskUserQuestion` and add a row before phase close.

---

## §5 — ADRs drafted

ADR numbering check (run at draft time): `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE '^[0-9]{4}' | sort -u | tail -5` → expected `0032 0033 0034 0035` → **next free = 0036**, then 0037.

| ADR | Title | Drafted at phase | Decision summary | Flip-to-Accepted phase |
|---|---|---|---|---|
| **ADR-0036** | Selector grammar adopts pest PEG; preserve enum-based legacy URI shapes via grammar-fast-path | P1 (grammar+parser landing) | **D36.1** — pest chosen (Fork 1-A) for spec-mirroring `.pest` file; ordered-choice handles reserved-tag ambiguity. **D36.2** — `Selector` becomes `pub struct Selector { ast: SelectorAst, source: SelectorSource }` with `enum SelectorAst { Bool(BoolExpr), Tag(Predicate), LegacyUriShape(LegacyForm) }` (Fork 3-A keep+extend). Legacy 4-variant URIs continue to parse via `parse_selector_or_uri()` fast-path. **D36.3** — `SetRefRegistry` shipped as a trait with one stub registration (kind-scoping); full registry wiring deferred to CH-15. **D36.4** — Out-of-scope: bracket char-classes, time predicates, numeric predicates, string-content match, cross-instance joins (per concept-09 §"Non-Normative Notes"). **D36.5** — Existing-grant policy: semantic continuity (Fork 5-A); `ResourceRef.uri: String` unchanged; new grammar's fast-path covers all 9 grant-mint call sites. | Chunk seal (P4) |
| **ADR-0037** | Instance-identity-tag rollout = all 15 types (11 composites + 4 nodes) in migration 0008 | P2 | **D37.1** — Fork 4-A: one migration over Fork 4-B/C trade-offs; `auto_tags()` is the single emission helper. **D37.2** — Composite types add `pub tags: Vec<String>` (#[serde(default)]) for round-trip safety on existing rows. **D37.3** — Node types: `Memory.tags` already exists at `nodes.rs:818` (just add emission); `Session`, `AuthRequest`, `ExternalService`, `InboxObject`, `OutboxObject` get `tags` field + emission. **D37.4** — Reserved-namespace enforcement (write-side) deferred to CH-07/CH-12 per concept-09's "manifest validator (publish-time)" note. **D37.5** — `auto_tags()` helper signature unchanged (`(&self, instance_id: &str) -> [String; 2]`) — call sites pass `id.to_string()`. | Chunk seal (P4) |

ADR file paths:
- [`m5_2/decisions/0036-selector-grammar-pest-peg.md`](../../v0/implementation/m5_2/decisions/0036-selector-grammar-pest-peg.md)
- [`m5_2/decisions/0037-instance-identity-tags-rollout.md`](../../v0/implementation/m5_2/decisions/0037-instance-identity-tags-rollout.md)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant relied on | Re-verification command |
|---|---|---|
| Post-CH-22 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1014 (CH-22 shipped 16 new) | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1` |
| CH-01 / ADR-0034 | `Agent.active`, `Agent.archived_at` columns + repo methods | `grep -n "set_agent_archived_at\|set_agent_active" modules/crates/domain/src/repository.rs` ≥ 2 |
| CH-02 / ADR-0032 | `BabyPhiSessionRecorder` writes Session rows; selector evaluation in Step 3 doesn't touch the recorder | `cargo test -p server --test acceptance_sessions_m5p4` |
| CH-22 / ADR-0035 | `AgentCatalogListener` body upserts `agent_catalog_entry`; CH-06 adds `tags: Vec<String>` field — listener body must populate it (or accept default via `#[serde(default)]`) without mutating other fields | `cargo test -p domain --lib events::listeners::tests` (15 catalog tests + new tag-aware tests) |
| M1 permission-check spine | Step-3 winner selection deterministic; the 9 grant-mint call sites still produce admissible grants under the new evaluator | `cargo test -p server --test '*'` (full HTTP suite passes) |
| M2/P4.5 explicit fundamentals on Grant | `resolve_grant` Case D (`grant.fundamentals` non-empty) preserves selector parsing of instance URI and binds to listed fundamentals | `grep -n "if !grant.fundamentals.is_empty" modules/crates/domain/src/permissions/expansion.rs` = 1 |
| All chunks | 4 CI guards green | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` |

---

## §7 — Phases within the chunk

**Phase count: 4** → audit envelope = **2 agents** (medium chunk per per-chunk-template guardrail).

### P1 — Grammar + parser + AST + `evaluate()` (~2.0d)

**Goal.** Land the PEG as a `.pest` file, derive the parser, build an AST mirroring concept-09 productions, implement `SelectorAst::evaluate(target_uri, target_tags, set_ref_registry) -> bool`. Preserve existing 4-variant `Selector` semantics via a grammar-fast-path so the 9 grant-mint call sites are unchanged (Fork 3-A + Fork 5-A).

**Deliverables.**

1. **Cargo deps** at workspace root + `domain/Cargo.toml`: add `pest = "2"` + `pest_derive = "2"` to `domain` only. Update `deny.toml` if cargo-deny rejects new pulls.

2. **PEG grammar file** `modules/crates/domain/src/permissions/grammar.pest` — transcribes concept-09 lines 58-102 1:1.

3. **Selector AST + parser** in `modules/crates/domain/src/permissions/selector.rs`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct Selector { pub ast: SelectorAst, pub source: SelectorSource }
   
   pub enum SelectorAst {
       Bool(Box<BoolExpr>),
       Tag(Predicate),
       LegacyUriShape(LegacyForm),  // Any / Exact / Prefix / KindTag
   }
   
   pub enum BoolExpr { Or(Vec<BoolExpr>), And(Vec<BoolExpr>), Not(Box<BoolExpr>), Pred(Predicate) }
   pub enum Predicate {
       Contains(Tag), Intersects(Vec<Tag>), AnyMatch(String), SubsetOf(SetRef),
       Empty, NonEmpty,
   }
   pub enum Tag { Reserved(String, Option<String>), Namespace(String, Vec<String>), Literal(String) }
   pub struct SetRef { pub name: String, pub args: Vec<String> }
   pub enum LegacyForm { Any, Exact(String), Prefix(String), KindTag(String) }
   ```

4. **Public functions** in `selector.rs`:
   - `pub fn parse_selector(input: &str) -> Result<Selector, SelectorParseError>` — full grammar parse via pest.
   - `pub fn parse_selector_or_uri(input: &str) -> Selector` — fast-path: `*` → `Any`, `system:root` → `Exact`, `<prefix>**` → `Prefix`, `#kind:<n>` → `KindTag`, else attempts grammar; on grammar error returns `Exact("<input>")` literal-match shim (preserves M1 semantics for the 9 call sites).
   - `pub fn evaluate(ast: &SelectorAst, target_uri: &str, target_tags: &[String], registry: &dyn SetRefRegistry) -> bool` — recursive evaluator.

5. **`SetRefRegistry` trait** + `NoopSetRefRegistry`:
   ```rust
   pub trait SetRefRegistry: Send + Sync {
       fn resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>>;
   }
   pub struct NoopSetRefRegistry;
   impl SetRefRegistry for NoopSetRefRegistry { fn resolve(&self, _: &str, _: &[&str]) -> Option<HashSet<String>> { None } }
   ```

6. **Stable parse errors** with codes P-001…P-005:
   ```rust
   pub enum SelectorParseError {
       UnexpectedToken { code: &'static str /* "P-001" */, position: usize, found: String },
       UnbalancedParens { code: &'static str /* "P-002" */, position: usize },
       UnknownPredicate { code: &'static str /* "P-003" */, name: String },
       InvalidGlob { code: &'static str /* "P-004" */, pattern: String },
       UnknownSetRef { code: &'static str /* "P-005" */, name: String },
   }
   ```

7. **Engine integration** in `modules/crates/domain/src/permissions/expansion.rs`:
   - `ResolvedGrant.selector: Selector` (was 4-variant enum); type re-exported.
   - `effective_matches(target_uri, target_tags)` signature stays; internally calls `evaluate(&self.selector.ast, target_uri, target_tags, registry)`.
   - `kind_refinement: Option<Selector>` carries `tags contains #kind:<name>` AST (built by `resolve_grant`).

8. **`CheckContext` extension** in `modules/crates/domain/src/permissions/manifest.rs`:
   - `pub set_ref_registry: &'a dyn SetRefRegistry` field.
   - Default in tests: `&NoopSetRefRegistry`.

9. **`engine.rs:286`** Step-3 call passes `ctx.set_ref_registry` through. Single-line change.

**Tests** (~25 unit + 1 proptest):
- 4 worked-parse golden ASTs (one per concept-09 example).
- 6 predicate-match unit tests (one per `contains`/`intersects`/`any_match`/`subset_of`/`empty`/`non_empty`).
- 6 combinator unit tests: AND, OR, NOT, parens, `NOT binds tighter than AND`, `AND binds tighter than OR`.
- 4 legacy-fast-path tests: `*`, `system:root`, `filesystem:/workspace/**`, `#kind:memory`.
- 5 parse-error tests (P-001…P-005).
- 1 proptest: round-trip `AST → string → re-parse → AST` equality (~256 cases).

**Concept-alignment check.** Flips concept-09 §"PEG Grammar", §"Primary Predicates", §"Logical Composition", §"Worked Parses", §"Reserved Namespace Enforcement (read side)", and `permissions/05` §"Memory as Resource Class" + §"Supervisor Extraction" (parser side).

**phi-core leverage check.** §3 row remains "(none)" — verified by grep at phase close.

**User-facing doc updates.** Land `m5_2/architecture/selector-grammar.md` (new) + update `m1/architecture/permission-check-engine.md` Step 3 paragraph.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE for `AskUserQuestion` if proptest discovers ambiguity not anticipated by concept-09 (e.g., glob-pattern semantics underspecified; nesting depth limits; whitespace handling).

---

### P2 — Instance-identity-tag wiring + migration 0008 (~1.5d)

**Goal.** Close D-new-11. Add `tags: Vec<String>` (#[serde(default)]) to all 11 composite + 4 missing-node types; wire `Composite::auto_tags(instance_id)` into every creation path. Ship migration 0008.

**Deliverables.**

1. **Migration** `modules/crates/store/migrations/0008_instance_identity_tags.surql` — adds `tags ARRAY<string>` to:
   - Composite tables: `agent_catalog_entry`, `system_agent_runtime_status`, `shape_b_pending_project`, `session_detail`, `objective`, `key_result`, `resource_boundaries`, `agent_execution_limits_override`, `organization_defaults_snapshot`, `token_budget_pool`.
   - Node tables: `auth_request`, `external_service`, `inbox_object`, `outbox_object`, `session` (Memory.tags already exists; only emission missing).

2. **Struct field additions** with `#[serde(default)]`:
   - `composites_m3.rs`: `OrganizationDefaultsSnapshot`, `TokenBudgetPool`.
   - `composites_m4.rs`: `Objective`, `KeyResult`, `ResourceBoundaries`, `AgentExecutionLimitsOverride`.
   - `composites_m5.rs`: `AgentCatalogEntry`, `SystemAgentRuntimeStatus`, `ShapeBPendingProject`, `SessionDetail`.
   - `nodes.rs`: `AuthRequest`, `ExternalService`, `InboxObject`, `OutboxObject`, `Session` (verify Memory.tags already at line 818).

3. **Wire `auto_tags()` into every creation path**:
   - `server/src/platform/orgs/create.rs:163-172` (CEO inbox + outbox + agent rows).
   - `server/src/platform/system_agents/add.rs:125-130` (system agent inbox + outbox).
   - `server/src/platform/agents/create.rs:141-146` (agent inbox + outbox).
   - `server/src/bootstrap/claim.rs:175-188` (bootstrap inbox + outbox + auth_request).
   - `server/src/platform/projects/create.rs:543-559` (project AR).
   - `server/src/platform/mcp_servers/register.rs:94` (ExternalService).
   - `server/src/platform/sessions/launch.rs:308` (Session row).
   - All `Memory` creation paths (CH-21 will add the supervisor-extractor path; for now, ensure `domain/src/in_memory.rs` Memory test fixtures emit the tag).
   - Composite constructors in `domain/src/model/composites_m3.rs`/`m4.rs`/`m5.rs` — pattern: add `pub fn new(...) -> Self` constructors that compute `tags = Composite::<Variant>.auto_tags(&id.to_string()).to_vec()`.
   - Listener-upsert paths in CH-22's `AgentCatalogListener::on_event` body — confirm `AgentCatalogEntry` is constructed with `tags` populated (cross-check post-merge).

4. **Acceptance test** `modules/crates/domain/tests/instance_tags_emission.rs` — creates one of each of the 15 types and asserts both `#kind:<name>` + `<kind>:<id>` tags are present.

**Tests** (~17):
- 15 unit tests (one per composite/node type emission).
- 1 cross-creation-path integration test confirming each creation handler emits both tags.
- 1 migration round-trip test in `store/tests/migration_0008.rs` — existing rows (`tags = []` via serde default) deserialize cleanly post-migration.

**Concept-alignment check.** Flips `permissions/01` §"Instance Identity Tags" to honored.

**phi-core leverage check.** No change.

**User-facing doc updates.** Update `m1/architecture/graph-model.md` (composite + node struct shapes); add tags-emission row to `m5_2/operations/selector-grammar-operations.md`.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if a Memory-creation site outside `server/` is discovered that can't be wired in this chunk — D-new-11 closure must cover it or carry an explicit successor.

---

### P3 — Existing-grant verification + 9-call-site test pinning (~1.0d)

**Goal.** Verify the 9 grant-mint call sites produce admissible grants under the new evaluator without code changes (Fork 5-A). Pin the conformance via tests.

**Deliverables.**

1. **No call-site changes** (Fork 5-A confirmed). Fast-path mapping in P1 covers each:
   - `secrets/add.rs` (`secret:<id>`) → `LegacyForm::Exact` shim → AST evaluates as exact-match.
   - `mcp_servers/register.rs` (`mcp_server:<id>`) → exact shim.
   - `model_providers/register.rs` (`provider:<id>`) → exact shim.
   - `orgs/create.rs` CEO bootstrap (`org:<uuid>` + `system:root`) → exact shim; `system:root` special-case in `expansion.rs:122` preserved verbatim.
   - `secrets/reveal.rs` test fixtures → exact shim.
   - `projects/create.rs` (`project:<id>`) → exact shim.
   - `mcp_servers/patch_tenants.rs` / `archive.rs` + `model_providers/archive.rs` + `defaults/put.rs` → exact shims.

2. **Conformance tests** in `domain/tests/grant_mint_conformance.rs` — one per call site (9 tests) asserting the issued grant's selector evaluates to `true` against the same target tags it did pre-CH-06.

3. **Cross-engine test** (1 test): full `check()` pipeline with a NEW-grammar selector (e.g., `tags contains org:acme AND tags contains #kind:session`) → Allowed when target has both tags, Denied otherwise.

**Tests** (~10):
- 9 grant-mint conformance tests.
- 1 cross-engine test.

**Concept-alignment check.** `permissions/04` §"Refinement" remains honored under new AST.

**phi-core leverage check.** No change.

**User-facing doc updates.** Update `m5/user-guide/troubleshooting.md` + (if applicable) `cli-reference-m5.md`.

**Confidence target.** ≥ 97%.

**Pause discipline.** PAUSE if any call site needs URI shape changes (e.g., catalogue Step-0 lookup uses a different form) — surface as new drift.

---

### P4 — Acceptance + ADR + drift closure + concept-doc bumps + audit + seal (~1.5d)

**Goal.** End-to-end HTTP-driven acceptance through the new grammar. Ratify ADR-0036 + ADR-0037. Close D-new-03 + D-new-11 terminally. Apply §3.C user-facing doc map. Spawn 2 audits. Seal.

**Deliverables.**

1. **Acceptance test** at `modules/crates/server/tests/selector_grammar_acceptance.rs` (NEW). Scenarios:
   - 4 worked-parse end-to-end tests (one per concept-09 example) — issue a grant with the new-grammar selector via test fixture, then check Permission Check `Allowed` / `Denied` per target tags.
   - 6 predicate end-to-end tests (one per primary predicate).
   - 1 supervisor-extraction scenario (uses stub `SetRefRegistry` that returns a fixed tag set).
   - 1 parse-error → `Decision::Denied { failed_step: Match, reason: SelectorParseError }`.

2. **ADR-0036 + ADR-0037** flipped from `Proposed` → `Accepted` at chunk seal.

3. **D-new-03 + D-new-11 lifecycle entries** appended; Status flipped to `remediated`.

4. **`drifts/README.md`** — both row Statuses flipped.

5. **`_concept-audit-matrix.md`** — 4 rows flipped (PEG grammar, Predicates, AND/OR/NOT, Instance identity tag).

6. **§3.C user-facing-doc map applied** — 8 file actions:
   - `m1/architecture/permission-check-engine.md` (Step 3 paragraph rewrite)
   - `m5_2/architecture/selector-grammar.md` (NEW)
   - `m1/architecture/graph-model.md` (composite + node struct shapes)
   - `m5_2/operations/selector-grammar-operations.md` (NEW)
   - `m1/operations/permission-engine-operations.md` (DeniedReason::SelectorParseError row, if file exists)
   - `m5_2/user-guide/selector-syntax-guide.md` (NEW)
   - `m5/user-guide/cli-reference-m5.md` (if any CLI selector surface, else defer)
   - `m5/user-guide/troubleshooting.md` (parse-error section)

7. **Spawn 2 audit agents** per §11 (medium chunk = 2 agents).

**Tests.** All §8 named tests green; full workspace passes ~1014 + ~52 = ~1066.

**Concept-alignment check.** All §2 rows at target-status. No `contradicted` remains. Out-of-scope rows (concept-09 §"Non-Normative Notes" exclusions) stay out-of-scope.

**phi-core leverage check.** Final green sweep — `check-phi-core-reuse.sh` exit 0; positive greps all ≥ expected counts; forbidden-duplication greps all 0.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** PAUSE if either audit reports a finding (concept-doc claim missed, drift transition wrong, ADR sub-decision incomplete) — surface to user before seal.

---

## §8 — Tests summary

- **Expected total at chunk close:** 1014 (post-CH-22) + ~52 new tests = **~1066 serialised tests**.
- **Layer breakdown:**
  - Unit (`domain/src/permissions/selector.rs::tests` + grammar tests): ~25 (4 worked-parses + 6 predicates + 6 combinators + 4 legacy fast-paths + 5 parse errors).
  - Property-based (`domain/src/permissions/selector.rs::tests`): 1 proptest (~256 cases internally).
  - Integration (`domain/tests/instance_tags_emission.rs`): ~16 (15 type emission + 1 cross-creation).
  - Conformance (`domain/tests/grant_mint_conformance.rs`): 9 (one per call site).
  - Cross-engine (`server/tests/selector_grammar_acceptance.rs`): ~12 (4 worked + 6 predicates + 1 supervisor + 1 parse-error).
  - Migration (`store/tests/migration_0008.rs`): 1 (round-trip default).
- **New test files:**
  - `modules/crates/domain/src/permissions/selector.rs` (mod tests extended)
  - `modules/crates/domain/src/permissions/grammar_tests.rs` (NEW)
  - `modules/crates/domain/tests/instance_tags_emission.rs` (NEW)
  - `modules/crates/domain/tests/grant_mint_conformance.rs` (NEW)
  - `modules/crates/server/tests/selector_grammar_acceptance.rs` (NEW)
  - `modules/crates/store/tests/migration_0008.rs` (NEW)
- **Expected-still-green fragile tests:**
  - `engine.rs::tests::*` (every test — fragile: depends on `effective_matches` signature change).
  - `expansion.rs::tests::*` (depends on `ResolvedGrant.selector` field-type change).
  - `m1/permission-check` proptest gate.
  - All 17 CH-22 catalog-listener tests (`AgentCatalogEntry` gains `tags` field).
  - `acceptance_sessions_m5p4.rs` (Session gains tag emission; assertions on `tokens_spent` etc. must be unaffected).

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive this plan verbatim (mandatory first action)

**Before any code or governance work begins:**

1. Generate the plan-file token: `openssl rand -hex 4` (record in plan archive line 5 of the placeholder `<8hex>`).
2. **Copy this plan file verbatim**: `cp /root/.claude/plans/sharded-discovering-stearns.md baby-phi/docs/specs/plan/build/<8hex>-ch-06-selector-grammar-peg-and-instance-tags.md`. No edits during the copy — the archive must match the approved plan byte-for-byte.
3. Update the archived plan's placeholders in the `**Plan file token:**` and `**Plan archive path:**` lines (the `<8hex>` placeholder) — these are the only allowed edits at archive time.
4. Run `bash scripts/check-doc-links.sh` to confirm the new archive's relative links resolve (CH-22 precedent: 6 `..` for paths into `baby-phi/`; 7 `..` for paths into `phi-core/`).
5. Verify the archive renders cleanly: `head -5 baby-phi/docs/specs/plan/build/<8hex>-ch-06-*.md` shows `Last verified: 2026-04-28 by Claude Code` on line 1, the chunk title on line 3, and the plan-file token on line 5.
6. Only AFTER successful archive does the rest of §9 (reading list + invariant checks) run.

This step matches the chunk-lifecycle-checklist Step 1 ("Create the plan file at `baby-phi/docs/specs/plan/build/<8hex>-<chunk-name>.md`") and the precedent set by CH-01 (`2aa37c80`), CH-02 (`16fd9a3a`), and CH-22 (`c5f201bb`) — all three chunks have their plans archived under this naming + path convention. The chunk-lifecycle-checklist's Cross-step invariant "concept docs are source of truth" implicitly requires the plan archive to exist for traceability — operators reading a sealed chunk's commit must be able to find the approved plan.

**Reading list (mandatory before `ExitPlanMode`):**
1. `concepts/permissions/09-selector-grammar.md` (full).
2. `concepts/permissions/01-resource-ontology.md` §"Instance Identity Tags".
3. `concepts/permissions/04-manifest-and-resolution.md` §"Refinement" + §"Formal Algorithm Step 3".
4. `concepts/permissions/05-memory-sessions.md` §"Memory as Resource Class" + §"Supervisor Extraction".
5. `concepts/permissions/README.md`.
6. `concepts/phi-core-mapping.md` (verify no parser/grammar entries).
7. Drifts D-new-03 + D-new-11 (full content).
8. CH-01 plan (`2aa37c80`) — for `Agent.active`/`archived_at` invariants.
9. CH-22 plan (`c5f201bb`) — for `AgentCatalogEntry` field expectations + listener body invariants.
10. `forward-scope/22035b2a-...md` §1 CH-06 row + §4 dependency graph + §7 Q2 + Q8 + Q9.
11. `baby-phi/CLAUDE.md` phi-core Leverage section.
12. `m5_1/process/per-chunk-planning-template.md` (this template).

**Carry-forward invariants (verified green at chunk-open):**
- `cargo test --workspace -- --test-threads=1` ≈ 1014 (CH-22 baseline).
- 4 CI guards green.
- D-new-03 + D-new-11 status currently `discovered`.
- ADR-0034 + ADR-0035 Accepted; ADR-0032 Accepted.
- `git diff --stat HEAD -- modules/` empty.
- Highest applied migration is 0007.

**Pending decisions carried into this chunk (resolved at plan-review):**
- Forward-scope Q2 (split decision): UNIFIED CH-06 (no split into CH-06a/b).
- Q4 (chunk ordering): user-selected CH-06 next after CH-22.
- Q5 (M5 scope): D-new-03 is HIGH and must close before M5 tag.
- Q6 (ADR numbering): ADR-0036 + ADR-0037 claimed.
- Q8 (K8s readiness): §3.B populated; K8s-neutral.
- Q9 (user-facing doc strategy): §3.C populated; 8 file actions identified.
- Forks 1, 2, 3, 4, 5 resolved with the recommended option (pest / unified / keep+extend / all-15-types / semantic-continuity).

**Cargo command convention** (per user feedback memory): all cargo invocations use `-j 4`. Tests serialise via `--test-threads=1`.

---

## §10 — Close criteria (5-aspect, post-Q9)

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — all P1–P4 deliverables shipped; `cargo test --workspace -- --test-threads=1` green at ~1066; clippy green under `RUSTFLAGS="-Dwarnings"` with `-j 4`; `cargo fmt --all -- --check` green; `selector_grammar_acceptance.rs` 12/12 pass.
- **Docs aspect** — TWO scopes (per Q9):
  - *Governance tier*: D-new-03 + D-new-11 lifecycle entries + Status flips; `_concept-audit-matrix.md` 4 rows flipped; `drifts/README.md` updated; ADR-0036 + ADR-0037 Accepted; concept-09 + concept-01 verified-headers bumped.
  - *User-facing tier*: every row of §3.C (8 actions) either updated in-chunk OR carrying explicit defer with successor chunk.
- **phi-core leverage aspect** — import-count delta = **0**; positive greps (§3) all ≥ expected; forbidden-duplication greps all 0; `check-phi-core-reuse.sh` exit 0.
- **Concept alignment aspect** — every §2 row at target-status; no `contradicted` remains; out-of-scope rows preserved as out-of-scope per ADR-0036 §"Out-of-Scope".
- **K8s readiness aspect** — §3.B 7-axis populated; CH-06 declared K8s-neutral; no new CHK8S-D-XX entries.

**Two confidence % (each with named numerator/denominator):**

- **Implementation confidence** = `claims-verified-honored / claims-in-scope` = target **13/13 = 100%**. The 13 claims:
  1. PEG grammar shipped (13 productions).
  2. 6 predicates implemented (`contains`/`intersects`/`any_match`/`subset_of`/`empty`/`non_empty`).
  3. 3 combinators with correct precedence (NOT > AND > OR).
  4. 4 worked-parse golden ASTs produce concept-doc parse trees.
  5. Reserved-tag PEG ordered choice handled (`#kind:session` parses as ReservedTag, not NamespaceTag).
  6. `parse_selector_or_uri()` fast-path for legacy URI shapes works for the 9 grant-mint call sites.
  7. `SetRefRegistry` trait shipped (stub registration; full at CH-15).
  8. Step-3 evaluator integrated (`engine.rs:286` calls new evaluator).
  9. `Composite::auto_tags()` wired into 8+ creation handlers.
  10. 11 composite types carry `tags: Vec<String>` field.
  11. 4 node types (Memory, Session, AuthRequest, ExternalService) carry tag emission.
  12. Migration 0008 round-trips empty tags via serde default.
  13. 9 grant-mint call sites unchanged; conformance tests pass.

- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check / doc-pages-touched` = target **10/10 = 100%**. Touched pages:
  1-2. ADR-0036 + ADR-0037
  3. D-new-03 drift
  4. D-new-11 drift
  5. `_concept-audit-matrix.md`
  6. `drifts/README.md`
  7-8. NEW: `m5_2/architecture/selector-grammar.md` + `m5_2/operations/selector-grammar-operations.md` + `m5_2/user-guide/selector-syntax-guide.md` (count as 3 in numerator if all three independently checkable; group for total-count purposes here)
  9. `m1/architecture/permission-check-engine.md` (Step 3 update)
  10. `m1/architecture/graph-model.md` (struct shapes update) + `m5/user-guide/troubleshooting.md` (parse-error section)

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P4 seal phase. No aspect-averaging; no rounding up.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 4 phases (P1/P2/P3/P4) = medium chunk → **2 agents** (per per-chunk-template guardrail).

**Audit aspects (a–e):**
- (a) Code correctness (P1+P2+P3 deliverables ship cleanly; tests pass).
- (b) Docs fidelity vs concept docs (ADR-0036 + ADR-0037 ratified; drift entries correct; matrix flipped; §3.C 8 actions completed).
- (c) Concept alignment (every concept-09 production has a mirroring AST type or parser test; every worked-parse example reproduces concept-doc tree).
- (d) phi-core leverage (`+0` imports; trait split intact; no struct duplication).
- (e) K8s readiness rule applied (§3.B populated; no new ledger entries).

**Auditor constraint.** Fresh `Explore` subagents. Not the implementer.

### Audit Agent A — Code correctness + phi-core leverage (aspects a + d)

> **Locked prompt** (drafted at Step 2; fired at P4 seal):
> You are auditing CH-06 (selector grammar PEG + instance identity tags) in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code. The chunk plan is at `docs/specs/plan/build/acd383e2-ch-06-selector-grammar-peg-and-instance-tags.md`.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. `pest` + `pest_derive` deps added to `domain/Cargo.toml` only; cargo-deny green.
> 2. `grammar.pest` exists at `modules/crates/domain/src/permissions/` with 13 productions (Selector / OrExpr / AndExpr / NotExpr / Predicate / TagPredicate / ContainsOp / Tag / TagSet / TagGlob / SetRef / Identifier / StringLiteral) matching concept-09 lines 58-102.
> 3. `SelectorAst` enum has 3 variants (Bool / Tag / LegacyUriShape); `Predicate` has 6 variants (Contains / Intersects / AnyMatch / SubsetOf / Empty / NonEmpty); `BoolExpr` has 4 (Or / And / Not / Pred).
> 4. `parse_selector` returns Result with `SelectorParseError` carrying P-001…P-005 codes.
> 5. `parse_selector_or_uri` fast-path covers `*`, `system:root`, `<prefix>**`, `#kind:<n>`; falls back to grammar; final fallback is `Exact("<input>")` literal-match shim.
> 6. `evaluate(ast, target_uri, target_tags, registry)` recursive evaluator covers all 6 predicates + 3 combinators + parens.
> 7. `SetRefRegistry` trait shipped + `NoopSetRefRegistry` default; `CheckContext.set_ref_registry: &dyn SetRefRegistry` field present.
> 8. Migration 0008 adds `tags ARRAY<string>` to all 15 tables (11 composites + auth_request + external_service + inbox_object + outbox_object + session). Memory unchanged (already had tags).
> 9. 11 composite struct types in `composites_m{3,4,5}.rs` carry `pub tags: Vec<String>` (#[serde(default)]).
> 10. 4 node struct types (AuthRequest, ExternalService, InboxObject, OutboxObject, Session) carry `tags: Vec<String>` (#[serde(default)]).
> 11. `Composite::auto_tags()` called from at least 8 creation handlers in `server/src/platform/`. Run: `grep -rn "auto_tags" modules/crates/server/src/platform/ | wc -l` ≥ 8.
> 12. `cargo test --workspace -- --test-threads=1` passes ~1066 tests.
> 13. `bash scripts/check-phi-core-reuse.sh` exit 0; `grep -rn "use phi_core::" modules/crates/domain/src/permissions/` returns 0 hits.
> 14. `grep -rn '^pub struct Selector\b\|^pub enum Selector\b' modules/crates/ | grep -v 'domain/src/permissions/selector.rs'` returns 0 hits.
> 15. Migration 0007 + 0008 round-trip test: existing rows (tags = []) deserialize cleanly post-migration.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 700 words.

### Audit Agent B — Concept fidelity + docs fidelity (aspects b + c + e)

> **Locked prompt** (drafted at Step 2; fired at P4 seal):
> You are auditing CH-06's concept-fidelity + docs-fidelity in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code or docs.
>
> Verify each claim against current HEAD. Report PASS / FAIL with 1-line evidence. Read-only.
>
> 1. ADR-0036 Accepted at `m5_2/decisions/0036-selector-grammar-pest-peg.md` with sub-decisions D36.1–D36.5.
> 2. ADR-0037 Accepted at `m5_2/decisions/0037-instance-identity-tags-rollout.md` with sub-decisions D37.1–D37.5.
> 3. D-new-03 drift Status = `remediated`; lifecycle entry `2026-04-28 — remediated — CH-06 chunk-seal` present.
> 4. D-new-11 drift Status = `remediated`; lifecycle entry `2026-04-28 — remediated — CH-06 chunk-seal` present.
> 5. `drifts/README.md` rows for D-new-03 + D-new-11 reflect remediation.
> 6. `_concept-audit-matrix.md` flips: PEG grammar (`contradicted → honored`); Predicates (`contradicted → honored`); AND/OR/NOT (`silent-in-code → honored`); Instance identity tag (`partially-honored → honored`).
> 7. concept-09 (`permissions/09-selector-grammar.md`) `Last verified` header bumped to 2026-04-28.
> 8. concept-01 (`permissions/01-resource-ontology.md`) `Last verified` bumped.
> 9. Every concept-09 production (Selector, OrExpr, AndExpr, NotExpr, Predicate, TagPredicate, ContainsOp, Tag, TagSet, TagGlob, SetRef) has a mirroring AST type or parser test.
> 10. Every concept-09 worked-parse example (4 total) reproduces the concept-doc tree as a golden test.
> 11. Reserved-tag PEG ordered choice honored: `#kind:session` parses as ReservedTag, not NamespaceTag (golden test).
> 12. Operator precedence test cases match concept-09 §"Operator Precedence" (NOT > AND > OR).
> 13. New architecture page `m5_2/architecture/selector-grammar.md` cross-references concept-09 by anchor.
> 14. Operations doc `m5_2/operations/selector-grammar-operations.md` lists P-001–P-005 parse error codes.
> 15. User-guide `m5_2/user-guide/selector-syntax-guide.md` includes the 4 worked-parse examples.
> 16. §3.C all 8 doc actions completed (or each non-touch explicitly justified with successor chunk).
> 17. §3.B K8s readiness 7-axis concludes K8s-neutral; `grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` returns 8 (unchanged from CH-22 baseline).
> 18. CH-22 invariants intact: 15 catalog-listener unit tests + 2 acceptance scenarios still pass; `record_system_agent_fire` helper unchanged.
>
> Report each as PASS / FAIL with 1-line evidence. ≤ 700 words.

**Seal-blocking rule.** Both audits must report PASS on every check, OR each FAIL must be either (a) fixed in-chunk before seal, (b) reframed via user-approved ADR amendment, or (c) converted to a new drift file with explicit future-chunk assignment.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (cap workers per user feedback)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: 1014 (CH-22 baseline) + ~52 new ≈ 1066

# 3. Chunk-specific positive greps
grep -nE "Contains|Intersects|AnyMatch|SubsetOf|Empty|NonEmpty" \
  modules/crates/domain/src/permissions/selector.rs                                              # ≥ 6
grep -n "fn evaluate\b\|fn parse_selector\b\|fn parse_selector_or_uri\b" \
  modules/crates/domain/src/permissions/selector.rs                                              # ≥ 3
ls modules/crates/domain/src/permissions/grammar.pest                                            # exists
grep -n "auto_tags" modules/crates/server/src/platform/ -r | wc -l                              # ≥ 8
grep -nE "^    pub tags: Vec<String>" modules/crates/domain/src/model/composites_m{3,4,5}.rs | wc -l   # ≥ 11
ls modules/crates/store/migrations/0008_instance_identity_tags.surql                             # exists
ls modules/crates/server/tests/selector_grammar_acceptance.rs                                    # exists
ls modules/crates/domain/tests/instance_tags_emission.rs                                         # exists

# 4. Chunk-specific negative greps
grep -rn "use phi_core::" modules/crates/domain/src/permissions/                                  # 0
grep -rn "^pub struct Selector\b\|^pub enum Selector\b" modules/crates/ | grep -v "selector.rs"  # 0

# 5. Targeted test runs
/root/rust-env/cargo/bin/cargo test -j 4 -p domain permissions::selector
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test instance_tags_emission
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test grant_mint_conformance
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test selector_grammar_acceptance
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migration_0008

# 6. Drift terminal closure
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-03.md   # 1
grep -c '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D-new-11.md   # 1
grep -c "2026-04-28 — \`remediated\`" docs/specs/v0/implementation/m5_1/drifts/D-new-03.md       # 1
grep -c "2026-04-28 — \`remediated\`" docs/specs/v0/implementation/m5_1/drifts/D-new-11.md       # 1

# 7. ADR status
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0036-selector-grammar-pest-peg.md     # 1
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0037-instance-identity-tags-rollout.md # 1

# 8. Concept-audit matrix
grep -c "PEG grammar\|Predicates\|AND/OR/NOT\|Instance identity tag" \
  docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md                              # ≥ 4 rows touched

# 9. K8s ledger unchanged
grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md   # 8

# 10. CH-22 regression sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s03            # CH-22 still 2/2
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --lib events::listeners::tests                # 15 catalog tests
```

---

## What this plan does NOT do

- **No reserved-namespace write-restriction enforcement.** Concept-09 §"Reserved Namespace Enforcement" says the parser is permissive (CH-06 honors); the manifest validator (publish-time) enforces write-restriction (deferred to CH-07's multi-scope cascade work, which already touches the validator).
- **No full SetRef registry wiring.** ADR-0036 §D36.3: ships trait + `NoopSetRefRegistry` + 1 stub registration (kind-scoping). Full registry (e.g., `supervisors_tagging_scope(supervisor-id)` runtime resolution) deferred to CH-15.
- **No data migration of existing grants.** ADR-0036 §D36.5 / Fork 5-A: `ResourceRef.uri: String` semantically continues; new grammar's fast-path covers all 9 call sites; existing Grant rows stay untouched.
- **No bracket char-classes / time predicates / numeric / string-content / cross-instance joins.** Per concept-09 §"Non-Normative Notes" — explicitly out-of-scope.
- **No new permissions/manifest API endpoint.** CH-06 lights up the engine surface; user-facing manifest editing is M6 / a05 work.
- **No CLI selector-string editing surface.** If `phi grant new --selector "..."` doesn't exist today, CH-06 doesn't add it (CH-19 doc-only chunk may add CLI surface).

---

## Notes on M5.1/P3 Q&A binding

- **Q1** (storage-backend ratification) — untouched.
- **Q2** (CH-06 split decision) — **resolved at plan-review: UNIFIED**. Single seal closes both drifts.
- **Q3** (consent triad) — untouched.
- **Q4** (chunk ordering) — user-selected CH-06 next after CH-22.
- **Q5** (M5 scope) — D-new-03 is HIGH and must close before M5 tag; D-new-11 is MEDIUM but bundled.
- **Q6** (ADR numbering) — ADR-0036 + ADR-0037 claimed; verified next-free at draft time.
- **Q7** (uniform ExitPlanMode ritual) — this plan is being approved via ExitPlanMode.
- **Q8** (K8s readiness) — §3.B populated; CH-06 declared K8s-neutral.
- **Q9** (user-facing doc strategy, codified by CH-22) — §3.C populated; 8 file actions identified.

---

## Critical files for implementation

**New files:**
- `modules/crates/domain/src/permissions/grammar.pest` — PEG transcription of concept-09
- `modules/crates/domain/src/permissions/grammar_tests.rs` — golden parse tests
- `modules/crates/domain/tests/instance_tags_emission.rs` — 15-type integration test
- `modules/crates/domain/tests/grant_mint_conformance.rs` — 9 call-site conformance
- `modules/crates/store/migrations/0008_instance_identity_tags.surql` — migration
- `modules/crates/store/tests/migration_0008.rs` — migration round-trip
- `modules/crates/server/tests/selector_grammar_acceptance.rs` — end-to-end
- `docs/specs/v0/implementation/m5_2/decisions/0036-selector-grammar-pest-peg.md` — ADR-0036
- `docs/specs/v0/implementation/m5_2/decisions/0037-instance-identity-tags-rollout.md` — ADR-0037
- `docs/specs/v0/implementation/m5_2/architecture/selector-grammar.md` — design page
- `docs/specs/v0/implementation/m5_2/operations/selector-grammar-operations.md` — runbook
- `docs/specs/v0/implementation/m5_2/user-guide/selector-syntax-guide.md` — operator reference

**Modified files (heavy):**
- `modules/crates/domain/src/permissions/selector.rs` — new `Selector` struct + `SelectorAst` + `parse_selector` + `parse_selector_or_uri` + `evaluate` + `SetRefRegistry` trait
- `modules/crates/domain/src/permissions/expansion.rs` — `ResolvedGrant.selector` type change
- `modules/crates/domain/src/permissions/manifest.rs` — `CheckContext.set_ref_registry`
- `modules/crates/domain/src/permissions/engine.rs:286` — pass registry through
- `modules/crates/domain/src/model/composites_m3.rs` / `m4.rs` / `m5.rs` — add `tags` field to 11 composites
- `modules/crates/domain/src/model/nodes.rs` — add `tags` to `AuthRequest`, `ExternalService`, `InboxObject`, `OutboxObject`, `Session`

**Modified files (light):**
- `Cargo.toml` (workspace) + `modules/crates/domain/Cargo.toml` — add `pest` + `pest_derive`
- `deny.toml` (if cargo-deny rejects the new crates)
- `modules/crates/server/src/platform/orgs/create.rs` (4 call sites for auto_tags)
- `modules/crates/server/src/platform/system_agents/add.rs` (2 call sites)
- `modules/crates/server/src/platform/agents/create.rs` (2 call sites)
- `modules/crates/server/src/platform/projects/create.rs` (1 call site for AR)
- `modules/crates/server/src/platform/sessions/launch.rs` (1 call site for Session)
- `modules/crates/server/src/platform/mcp_servers/register.rs` (1 call site for ExternalService)
- `modules/crates/server/src/bootstrap/claim.rs` (3 call sites)
- `docs/specs/v0/implementation/m5_1/drifts/D-new-03.md` (lifecycle entry)
- `docs/specs/v0/implementation/m5_1/drifts/D-new-11.md` (lifecycle entry)
- `docs/specs/v0/implementation/m5_1/drifts/README.md` (status updates)
- `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` (4 row flips)
- `docs/specs/v0/concepts/permissions/09-selector-grammar.md` (verified-header bump)
- `docs/specs/v0/concepts/permissions/01-resource-ontology.md` (verified-header bump)
- `docs/specs/v0/implementation/m1/architecture/permission-check-engine.md` (Step 3 paragraph)
- `docs/specs/v0/implementation/m1/architecture/graph-model.md` (struct shapes)
- `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` (parse-error section)

**Reused (no edit):**
- `modules/crates/domain/src/model/composites.rs::Composite::auto_tags` — already shipped at line 119 with full unit-test coverage; CH-06 just calls it.

---

## Verification end-to-end (after seal)

1. `git status --short` — only the listed files in working tree.
2. `cargo test -j 4 --workspace -- --test-threads=1` — green at ~1066.
3. `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh` — all green.
4. `git log --oneline -5` — chunk seal commit cites CH-06, ADR-0036 + ADR-0037, drifts D-new-03 + D-new-11.
5. Manual sanity: spawn server in dev profile → POST a grant with `{"selector_expr": "tags contains org:acme AND tags contains #kind:session"}` (if HTTP surface accepts) → run a session against an Org-acme target → verify Permission Check Allowed.
