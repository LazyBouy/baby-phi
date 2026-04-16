<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Two Shapes: Tool Authority Manifest vs Grant

The doc has been describing two related but distinct shapes. Here is how they differ:

| Shape | What It Describes | Carries Subject? | Carries Provenance? | Where It Lives |
|-------|-------------------|------------------|---------------------|----------------|
| **Tool Authority Manifest** | A tool's REQUIREMENTS — "to use me, you need these capabilities" | No (a tool isn't owned by anyone) | No (it's a static tool spec) | Shipped with the tool definition |
| **Grant** | A capability HELD by a specific principal | Yes (implicit in the edge source) | Yes (audit trail) | Stored as a graph node, attached via `HOLDS_GRANT` |

The runtime reconciles these via a **Permission Check** (see further below).

### Tool Authority Manifest (Tool Requirements)

**Design rule:** Every tool must ship a machine-readable authority manifest declaring what it does, so the system can check whether a caller has the matching grants.

A manifest declares:
- Resource classes touched
- Actions performed
- Transitive resources consumed (e.g., `bash` can reach `network endpoint` transitively)
- Delegation behavior
- Approval defaults
- Constraints that callers must satisfy

> A manifest is **descriptive of the tool**, not prescriptive of any user. It says "I do X" — it does NOT say "Bob is allowed to call me."

Example for `write_file`:
```yaml
tool: write_file
manifest:
  actions: [create, modify]
  resource: filesystem_object
  constraints:
    path_prefix: required     # caller must scope the path
    max_size_bytes: 1048576   # 1MB default limit
  transitive: []              # no transitive access
  delegable: true
  approval: auto              # no human approval needed
```

Example for `bash`:
```yaml
tool: bash
manifest:
  actions: [execute]
  resource: process_exec_object
  constraints:
    command_pattern: required
    sandbox: recommended
    timeout_secs: 120
  transitive:
    - filesystem_object       # can read/write files
    - network_endpoint        # can make HTTP calls
    - secret/credential       # can access env vars
  delegable: false            # too powerful to delegate
  approval: human_recommended
```

#### The Transitive-Grant Match Rule

The `transitive` field is **load-bearing**, not merely documentary. When an agent invokes a tool, the Permission Check must pass for **every fundamental** the manifest implies — derived from its primary class, its transitive list, and the target entity's classification. Missing a grant for any fundamental is a denial.

**The rule, stated plainly:**

> A tool manifest must declare every fundamental the tool touches and every composite `#kind:` value the tool operates on. These declarations are validated at **tool publish time** by the tool registry's manifest validator; inconsistent manifests are rejected and the tool cannot be registered. Once a tool is published, the runtime trusts the manifest and performs Permission Checks against its declarations.
>
> **At runtime** (when an agent invokes an already-published tool), the runtime derives the full set of required **fundamentals** by (a) expanding the manifest's primary class to fundamentals (including the implicit `tag` on any composite), (b) expanding each class in the transitive list to fundamentals, and (c) classifying the target entity (if any) to fundamentals. The Permission Check must pass for **every fundamental in this union**. A missing grant for any fundamental is a **runtime denial** (the agent lacks authorization — not a manifest problem).
>
> **At publish time**, the validator rejects a manifest that (a) declares a composite but omits its `#kind:` value, (b) declares a `#kind:` without the matching fundamentals, or (c) declares fundamentals inconsistent with the composites it names. The error message names the specific missing declaration so the tool creator can fix it and resubmit.
>
> `#kind: *` (blanket) is legal but throws a publish-time warning. The composite label (using `external_service_object` instead of its fundamentals) is optional — a warning at publish time may suggest adding it for readability, but the manifest is accepted either way.

**Worked example:**

An agent invokes `bash` to run `curl https://api.example.com | tee /tmp/response.json`.

The `bash` manifest declares:
- `resource: process_exec_object` (fundamental)
- `transitive: [filesystem_object, network_endpoint, secret/credential]` (all fundamentals)

Runtime derives the required fundamental set: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Runtime runs four parallel Permission Checks:

| Fundamental | Agent grant? | Check result |
|---|---|---|
| `process_exec_object` | ✓ (sandbox execute grant) | ✓ |
| `filesystem_object` | ✓ (workspace read grant) | ✓ |
| `network_endpoint` | ✗ (no network grant) | ✗ |
| `secret/credential` | — (not required for this specific call) | — |

**Result: DENIED** — missing `network_endpoint` is a hard fail, even though the agent has the primary class grant and two of the three transitive classes.

**Worked example with composite shorthand:**

Agent invokes `mcp_github` to read a pull request. `mcp_github` manifest declares `resource: external_service_object` (composite). Runtime expands: `{network_endpoint, secret/credential, tag}` + `#kind:external_service` filter.

If the agent holds a composite grant for `external_service_object`, that grant auto-expands to both fundamentals and satisfies both checks. Alternatively, the agent could hold two separate fundamental grants for `network_endpoint` and `secret/credential` plus a `tag_predicate` selector matching `#kind:external_service`, and both would satisfy the checks. **Both forms are semantically identical** because the runtime normalizes everything to fundamentals.

**The security implication:**

A manifest that under-declares its fundamental reach is a **security bug**, caught at publish time by the manifest validator. If `bash` forgot to declare `secret/credential` in its transitive set, the validator would reject the publish because the declared behavior of `bash` is inconsistent with the declared fundamental set. The validator is the last-line safety net; under-declaration cannot reach production.

**The documentation implication (warning-only rule):**

A manifest that declares the right fundamentals but forgets to use a composite label (e.g., declares `network_endpoint + secret/credential` directly instead of `external_service_object`) is semantically correct. The validator accepts it. The linter may warn that the pattern matches a known composite and suggest the composite form for readability — but this is cosmetic only, never load-bearing.

**Relationship to the entity overlap rule:**

Both the entity-overlap rule (Edit 1c) and the transitive rule converge on the same runtime semantic: **a set of fundamentals must all be satisfied**. They differ only in where the fundamentals come from:

- **Entity overlap**: an entity is classified to multiple fundamentals via `classify_to_fundamentals(entity)`
- **Manifest transitive**: a tool manifest declares multiple fundamentals via `expand_composites(manifest.resource) ∪ expand_composites(manifest.transitive)`
- **Both compose**: a `bash` call operating on `.env` has required fundamentals from both sources. Stage 1 of `check_tool_invocation` unions them.

See the [`check_tool_invocation` pseudocode](#how-the-two-mechanisms-combine) for the full implementation.

### Grant (5-Tuple, Held by a Subject)

A Grant fills in all five components of the canonical shape. It is attached to a subject via a `HOLDS_GRANT` edge.

Example: `claude-coder-7` is allowed to run a narrow set of cargo commands.

```yaml
grant:
  # subject is the source of the HOLDS_GRANT edge → agent:claude-coder-7
  action: [execute]
  resource:
    type: process_exec_object
    selector: "command matches /^cargo (build|test|fmt)$/"
  constraints:
    timeout_secs: 60
    sandbox: true
  provenance: agent:project-lead-1@2026-04-01
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: end_of_session
```

Example: a project-scoped grant that propagates to all project members.

```yaml
grant:
  # subject is the source of the HOLDS_GRANT edge → project:website-redesign
  action: [read, modify]
  resource:
    type: filesystem_object
    selector: "/workspace/website-redesign/**"
  constraints:
    max_size_bytes: 10485760  # 10MB
  provenance: config:org-default.toml#project-workspace-policy
  delegable: true
  approval_mode: auto
  audit_class: silent
  revocation_scope: manual
```

### Permission Check (Runtime Reconciliation)

When an agent invokes a tool, the runtime executes a **Permission Check** that combines:
- The tool's **Authority Manifest** (what the tool requires)
- The agent's **Grants** (what the agent holds)
- The **resolution hierarchy** (org → project → agent — see further below)

#### Worked Example

`claude-coder-7` calls `bash` with the command `cargo build`.

The runtime asks five questions:

| Step | Check | Manifest Side | Grant Side | Result |
|------|-------|---------------|------------|--------|
| 1 | Does the agent hold a grant for the manifest's action? | `actions: [execute]` | grant has `action: [execute]` | ✓ |
| 2 | Does the grant's resource type match the manifest's resource? | `resource: process_exec_object` | `resource.type: process_exec_object` | ✓ |
| 3 | Does the actual call satisfy the grant's resource selector? | call: `cargo build` | selector: `^cargo (build|test|fmt)$` | ✓ |
| 4 | Are the manifest's required constraints satisfied by the grant? | `command_pattern: required`, `sandbox: recommended`, `timeout_secs: 120` | grant provides selector (satisfies pattern), `sandbox: true`, `timeout_secs: 60` (≤ 120) | ✓ |
| 5 | Do org and project ceilings allow this? | — | org allows `process_exec_object`, project allows `bash` | ✓ |
| → | **Decision** | | | **Allowed** |

If the same agent tries to run `rm -rf /`:

| Step | Check | Result |
|------|-------|--------|
| 3 | Does `rm -rf /` match the grant's selector `^cargo (build|test|fmt)$`? | ✗ |
| → | **Decision** | **Denied** (with audit log entry citing the failed selector match) |

#### Mental Model

A manifest and a grant are **two halves of a key**:
- The manifest says *"this is what I need."*
- The grant says *"this is what you can do."*
- A permission check is **set containment**: every requirement on the manifest side must be covered by some grant on the grant side, AND every constraint must be jointly satisfied.

If you remember nothing else: **Manifests describe tools. Grants describe subjects. Permission Checks combine them at runtime.**

#### Formal Algorithm (Pseudocode)

The runtime's Permission Check is shown below as an executable-looking pseudocode in Python style. This is the normative description of what an implementation must do; the worked-example table above is a trace of this algorithm for one specific invocation. Types in angle brackets (e.g., `<Fundamental>`) are defined in [03-action-vocabulary.md](03-action-vocabulary.md) and [01-resource-ontology.md](01-resource-ontology.md).

```python
# Returns Allowed | Denied{reason, failed_step} | Pending{awaiting_consent}
def permission_check(agent: Agent, call: ToolCall) -> Decision:
    manifest = call.tool.manifest

    # --- Step 1: Expand manifest into required (fundamental, action) pairs ---
    # Composite resource types expand into their constituent fundamentals per the
    # composite-class definitions in 01-resource-ontology.md. The manifest's declared
    # #kind: values are carried through as required tag-predicate constraints.
    required_reaches: set[(Fundamental, Action)] = set()
    required_kinds: set[Kind] = set(manifest.kind or [])
    for resource_type in flatten(manifest.resource):
        for fundamental in expand_to_fundamentals(resource_type):
            for action in manifest.actions:
                required_reaches.add((fundamental, action))
    required_constraints: set[Constraint] = set(manifest.constraints)

    # --- Step 2: Resolve the agent's applicable grants ---
    # Walk HOLDS_GRANT from the agent, plus from the agent's current_project and
    # current_organization. Each returns the grants that subject holds.
    candidate_grants: list[Grant] = (
          grants_held_by(agent)
        + grants_held_by(agent.current_project)    # project-level grants the agent inherits
        + grants_held_by(agent.current_organization) # org-level grants
    )

    # --- Step 2a: Mechanism 1 — Ceiling Enforcement (top-down) ---
    # Org ceilings bound project grants; project ceilings bound agent grants.
    # Any grant whose scope exceeds its ancestor's ceiling is clamped down.
    ceilings: list[Grant] = ceiling_grants_above(agent)
    effective_grants: list[Grant] = [
        clamp_to_ceilings(g, ceilings) for g in candidate_grants
    ]
    effective_grants = [g for g in effective_grants if not g.is_empty()]

    # --- Step 3: Match each required reach against some grant ---
    matches: dict[(Fundamental, Action), Grant] = {}
    for (fundamental, action) in required_reaches:
        matching = [
            g for g in effective_grants
            if covers(g.resource.type, fundamental)
            and action in g.action
            and selector_matches(g.resource.selector, call.target_tags, call.context)
        ]
        if not matching:
            return Denied(reason="no grant covers this reach",
                          failed_step=3,
                          detail=(fundamental, action))
        # Multiple grants may match — defer resolution to Step 5
        matches[(fundamental, action)] = matching

    # --- Step 4: Evaluate required constraints against matched grants ---
    # A manifest constraint is satisfied if at least one of the matched grants
    # supplies a value that meets the constraint's requirement. Constraints are
    # checked AFTER scope resolution picks the winning grant in Step 5, per-reach.

    # --- Step 5: Mechanism 2 — Scope Resolution (most-specific-first) ---
    # When multiple grants match a reach, cascade from most-specific to least:
    #   1. Match on current_project (most specific)
    #   2. Match on current_organization
    #   3. Match on base_project (tie-breaker 1)
    #   4. Match on base_organization (tie-breaker 2)
    #   5. Intersection of all matches (fallback for cross-scope outsiders)
    # The first tier that yields a match wins; later tiers are ignored.
    resolved: dict[(Fundamental, Action), Grant] = {}
    for reach, candidates in matches.items():
        winner = cascade_resolve(candidates, agent)
        if not constraints_satisfied(required_constraints, winner, call):
            return Denied(reason="constraint violation",
                          failed_step=4,
                          detail=(reach, winner.id))
        resolved[reach] = winner

    # --- Step 6: Consent gating (for Templates A/B/C/D; Template E is bypass) ---
    # If any winning grant was issued by Authority Template A/B/C/D, the target
    # subordinate's Consent under the issuing org's policy must be Acknowledged.
    for reach, winner in resolved.items():
        if winner.provenance.kind == "authority_template" and winner.provenance.template_id in {"A","B","C","D"}:
            consent = consent_record(
                subordinate=call.target_agent,
                org=winner.provenance.org,
            )
            if consent is None or consent.state != "Acknowledged":
                return Pending(awaiting_consent=(call.target_agent, winner.provenance.org))

    # --- All reaches covered, all constraints satisfied, consent present where required ---
    return Allowed(resolved_grants=resolved)
```

**Key invariants:**

1. **Every required reach must match some grant.** There is no "default allow" — an action on a fundamental the manifest declares must find a covering grant, or the check fails at Step 3.
2. **Ceilings clamp, they don't add.** Step 2a can only remove scope from a grant, never widen it. A grant that already fits within its ceilings is unchanged.
3. **Scope resolution is deterministic.** The cascade in Step 5 is an ordered search; the first tier with a match wins. Ties within a tier are broken by the base-org rule (see [06 § Multi-Scope Session Access](06-multi-scope-consent.md#multi-scope-session-access)).
4. **Consent is evaluated last.** Consent gating only runs *after* Steps 1–5 have identified a winning grant. A missing consent produces `Pending`, not `Denied` — the caller can wait, request, or retry.
5. **Audit trail on every outcome.** Every return value (Allowed/Denied/Pending) emits an audit event recording the winning grants, the reason, and the failed step if any.

**Worked Trace — `cargo build` example.**

Running the `claude-coder-7` / `bash cargo build` example from the worked-example table above through this pseudocode:

- **Step 1** expands `manifest.resource: process_exec_object` into fundamentals `{process_exec_object, filesystem_object, network_endpoint, secret/credential, time/compute_resource}` with action `execute`. Required kinds: none (bash is a bare-fundamental tool).
- **Step 2** gathers `claude-coder-7`'s grants plus project + org grants. Suppose the bash grant has selector `^cargo (build|test|fmt)$`.
- **Step 2a** clamps with org and project ceilings — bash is org-permitted, project-permitted.
- **Step 3** matches the `(process_exec_object, execute)` reach against the bash grant. Selector matches `cargo build`. ✓
- **Step 4** checks manifest constraints: `command_pattern: required` (grant's selector provides it), `sandbox: recommended` (grant has `sandbox: true`), `timeout_secs: 120` (grant has `60 ≤ 120`). ✓
- **Step 5** no cascade needed — only one matching grant.
- **Step 6** bash grant's provenance is a standard Default Grant, not an Authority Template A/B/C/D — consent gating is skipped.
- **Return `Allowed`.**

If `claude-coder-7` tried `rm -rf /`: Step 3 finds no grant whose selector matches `rm -rf /` (the selector `^cargo (build|test|fmt)$` rejects it), returning `Denied{reason: "no grant covers this reach", failed_step: 3, detail: (process_exec_object, execute)}`.

---

## Permission Resolution Hierarchy

There are **two distinct mechanisms** at play when resolving a Permission Check, and the doc has historically conflated them. Separating them makes the rules clearer.

### Mechanism 1: Ceiling Enforcement (top-down upper bound)

Organizations and Projects can attach Permission grants to themselves (via `HOLDS_GRANT` edges). These act as **upper bounds** on what any subject within their scope can do. This mechanism enforces "no project within Acme can exceed Acme's policies" and similar containment rules.

```
Organization (highest ceiling)
    │ caps ↓
Project (capped by its owning orgs)
    │ caps ↓
Agent (capped by its project + org)
```

**Rules:**
- **Org caps project:** If an org restricts `network_endpoint` access, no project within it can grant it back to its members.
- **Project caps agent:** If a project restricts `bash` tool, no agent within it can use it via that project's grants.
- **Agent grants are most specific:** Within the bounds set by org and project, the agent's own grants determine fine-grained behavior.

**Delegation:** When Agent A delegates to Agent B, B inherits A's permission *ceiling* (never more than A has), further narrowed by B's own grants.

### Mechanism 2: Scope Resolution (specific-first selection)

When a session has multiple `org:` tags (a joint project) — or, in principle, multiple `project:` tags — the runtime needs to pick **which scope's grants to apply** for a given reader. Resolution cascades from most-specific to most-general, with `base_*` as the tie-breaker:

```
1. Project-level resolution
     - Reader is a member of one of the session's projects?  → use that project's scope
     - Reader is a member of multiple of the session's projects?  → base_project wins
     - Reader is a member of none?  → fall through

2. Org-level resolution
     - Reader's base_org matches one of the session's orgs?  → use that org's scope
     - Reader is a member of multiple of the session's orgs?  → base_org wins
     - Reader is a member of none?  → fall through

3. Intersection fallback (outsider)
     - Apply the intersection of all the session's scope ceilings
```

**Cascade rationale:** Resolution always tries the *narrowest* scope the reader has a legitimate claim to. Outsiders who hold no claim at any level face the strictest treatment (intersection-of-everything), eliminating loopholes where someone could "shop" for a permissive scope.

**Where this matters:** Scope resolution only kicks in when a session has multiple scope tags at some level. The common single-org single-project case bypasses Mechanism 2 entirely. See **[Multi-Scope Session Access](06-multi-scope-consent.md#multi-scope-session-access)** under Sessions for the full rule, worked examples, and the schema constraint that prevents simultaneously-multi-project AND multi-org sessions.

### How the Two Mechanisms Combine

Both mechanisms apply on every Permission Check:

```
allowed = scope_resolution_picks_a_grant_that_matches(reader, session)
        AND ceiling_enforcement_does_not_block(reader, session, picked_scope)
        AND has_matching_subject_grant(reader, session)
```

Scope resolution picks *which* scope's grants apply. Ceiling enforcement bounds those grants from above. The agent's own grants must match within those bounds. All three must hold for the read to succeed.

#### A Possible Refinement

The high-level pseudocode above is intentionally abstract — it states the three required conditions without committing to where each grant lookup happens. A more concrete formulation, useful when reasoning about implementation, makes the two-tier resource ontology and the `#kind:` tag filtering explicit. The concrete version has two stages: derive the required **fundamentals** (from the manifest and entity classification), then run a per-fundamental permission check.

```
// A resolved grant carries its expanded fundamentals AND its effective selector
// (which may include an implicit #kind: refinement if the grant targeted a composite).
struct ResolvedGrant {
    fundamentals: HashSet<Fundamental>,
    effective_selector: Selector,  // explicit selector AND implicit #kind: filter
    constraints: Constraints,
    approval_mode: ApprovalMode,
    // ... other standard fields
}

fn resolve_grant(g: &Grant) -> ResolvedGrant {
    // If the grant targets a composite, expand it to fundamentals AND
    // add the implicit #kind: tag predicate to the effective selector.
    let (fundamentals, extra_selector) = match &g.resource.target {
        ResourceTarget::Fundamental(f) => (HashSet::from([*f]), None),
        ResourceTarget::Composite(c) => {
            let fs = expand_composites(c);   // always includes `tag` implicitly
            let kind_filter = Selector::TagPredicate(
                format!("tags contains #kind:{}", c.name())
            );
            (fs, Some(kind_filter))
        }
    };

    let effective_selector = match extra_selector {
        Some(k) => Selector::And(vec![g.resource.selector.clone(), k]),
        None => g.resource.selector.clone(),
    };

    ResolvedGrant { fundamentals, effective_selector, /* ... */ }
}

fn check_tool_invocation(reader: Agent, tool: Tool, target: Option<Entity>) -> bool {
    // STAGE 1: Derive the required fundamental set.
    //
    // Sources:
    //   (a) The tool manifest's primary class and transitive list.
    //       Composites in the manifest are expanded to their constituents
    //       (including the implicit `tag` on any composite).
    //   (b) The target entity's classification (if the action touches a
    //       specific entity). An entity like `.env` maps to multiple
    //       fundamentals (filesystem_object + secret/credential).
    let mut required: HashSet<Fundamental> = HashSet::new();
    required.extend(expand_composites(tool.manifest.resource));
    required.extend(expand_composites(tool.manifest.transitive));
    if let Some(e) = target {
        required.extend(classify_to_fundamentals(e));
    }

    // STAGE 2: For every required fundamental, run the per-fundamental
    // Permission Check. ALL must pass for the invocation to be allowed.
    required.iter().all(|fundamental| {
        check_action_for_fundamental(reader, tool.manifest.actions, target, fundamental)
    })
}

fn check_action_for_fundamental(
    reader: Agent,
    actions: Vec<Action>,
    target: Option<Entity>,
    fundamental: Fundamental,
) -> bool {
    // Single-fundamental check: scope resolution, candidate grant
    // collection (including composite expansion + #kind: filter), selector
    // matching, ceiling filtering, approval gates, set non-emptiness.

    // 1. Scope resolution — pick which scope's grants apply.
    let scope = resolve_scope(reader, target);

    // 2. Gather candidate grants the reader holds that cover THIS fundamental.
    //    Grants that declare a composite are auto-expanded via resolve_grant().
    //    A composite grant is a candidate for any fundamental in its expansion,
    //    but its effective_selector includes the #kind: filter — so memory-specific
    //    grants will not match session entities even though both grants target
    //    the same fundamentals (data_object + tag).
    let candidates: Vec<ResolvedGrant> = reader.grants()
        .iter()
        .map(resolve_grant)
        .filter(|g| g.fundamentals.contains(&fundamental))
        .filter(|g| g.actions_cover(&actions))
        .collect();

    // 3. Filter by effective selector (includes any implicit #kind: refinement).
    let matching = candidates.iter().filter(|g| g.effective_selector.matches(target));

    // 4. Ceiling enforcement — drop any grant that exceeds an upper bound.
    let allowed_by_ceiling = matching.filter(|g| {
        ceiling_for_scope(scope).bounds(g)
            && all_org_caps_for_target(target).bounds(g)
    });

    // 5. Approval gates — handle subordinate_required, human_required, etc.
    let final_grants = allowed_by_ceiling
        .filter(|g| approval_satisfied(g, reader, target));

    // 6. Decision: non-empty grant set.
    !final_grants.is_empty()
}
```

This concrete version makes six things explicit that the abstract version leaves implicit:

1. **Two-stage structure.** Stage 1 derives what's required; Stage 2 checks each requirement. This separates "what does the operation touch" from "does the agent have permissions for it."
2. **Composite expansion happens in Stage 1.** Both on the manifest side (`expand_composites(tool.manifest.resource)`) and on the grant side inside `resolve_grant` (a composite grant auto-expands to its constituent fundamentals plus an implicit `#kind:` selector refinement).
3. **Entity classification happens in Stage 1 too.** `classify_to_fundamentals(entity)` returns every fundamental the entity belongs to — the overlap rule falls out naturally.
4. **Grants come from multiple sources** — defaults, Authority Templates, Template E, and inherited project/org grants — and they all participate in the same candidate pool.
5. **Ceiling enforcement is a filter on the candidate set** — not a separate step that runs after grant selection.
6. **Approval gates run last** — after scope resolution, after grant matching, after ceiling filtering. This is where Consent Policy fits (`subordinate_required` lives in `approval_satisfied`).

**The key role of `resolve_grant()`:** it expands composite grants by adding an implicit `#kind:` tag predicate to the effective selector. This is how memory-specific grants are prevented from matching session entities even though both share `data_object + tag` fundamentals. The entity's `#kind:` tag is what makes the match possible; the grant's `#kind:` filter is what narrows grant applicability.

**Composites that are missing from the manifest (but whose fundamentals are all present) do NOT cause a hard deny at runtime** — the manifest was already validated at publish time (see "Enforcement Asymmetry"). A missing composite label is a warning at publish time; missing fundamentals or missing `#kind:` is a publish-time rejection. The runtime operates on already-validated manifests and focuses on the agent's authorization.

The abstract version remains the canonical statement of the rule because it's easier to reason about and harder to get wrong. The refinement is a useful pseudocode reference for implementation discussions and edge-case validation.

### The Authority Chain

Every Grant in the system points to an Auth Request (as its `provenance`). Every Auth Request was approved by one or more approvers, acting under ownership or allocation authority over the affected resources. Every owner's ownership itself traces back through an Auth Request (or through the hardcoded System Bootstrap Template). This forms a **tree of authority** rooted at the bootstrap.

```
Grant (e.g., grant-7101)
  │ provenance
  ▼
Auth Request (e.g., auth_request:req-4581)
  │ approved by
  ▼
Owner / Approver (e.g., agent:lead-acme-1)
  │ ownership derived from
  ▼
Auth Request that allocated ownership to this agent
  │ ... continues up the tree ...
  ▼
Auth Request issued by Template A adoption
  │ provenance
  ▼
Template A adoption Auth Request
  │ part of
  ▼
System Bootstrap Template adoption Auth Request
  │ approved by
  ▼
system:genesis (axiomatic — root of the tree)
```

**What this tree enables:**

- **Auditability** — "Who approved Agent X's access to Resource R on date D?" is a standard graph traversal: walk from the Grant up to the bootstrap. Every node on the path is an auditable Auth Request with a timestamp, approver, and justification.
- **Revocation cascades** — If any Auth Request on the path is revoked, the descendants in its subtree are revoked (forward-only, per the Consent Revocation rule). A compromised approver's authority can be fully undone without losing audit history.
- **Accountability** — Every Grant has a named human (or named System Agent) somewhere on its path. There are no "system-issued" grants without a traceable human decision at some level.
- **Trust assessment** — A Grant whose path touches only trusted approvers is more trusted than one whose path includes an ad-hoc Template E grant by a recently-onboarded admin. The path length and the trust of each hop are both queryable.

The Grant node's `provenance` field (documented below in "Grant as a Graph Node") is the entry point to this traversal. In earlier drafts, `provenance` was a string annotation; with the Auth Request model, it is a **structural reference** that enables all of the above without special tooling.

---

## Grant as a Graph Node

A Grant node stores **four of the five components** of the canonical 5-tuple. The fifth — `subject` — is **not** stored as a property; it is expressed structurally as the source of the `HOLDS_GRANT` edge that points to the node.

```
Permission                       -- the node stores 4 of 5 components
  resource_type: String          -- e.g. "filesystem_object"     [resource]
  resource_selector: String      -- e.g. "/workspace/project-a/**"  [resource]
  action: Vec<String>            -- e.g. ["read", "modify"]      [action]
  constraints: Json              -- condition slots              [constraints]
  delegable: bool                -- can this be passed to sub-agents
  approval_mode: String          -- "auto", "human_required", "human_recommended"
  audit_class: String            -- "silent", "logged", "alerted"
  provenance: String             -- e.g. "system", "agent:sarah", "config:..."  [provenance]
  revocation_scope: String       -- "immediate", "end_of_session", "manual"

  -- subject is NOT a field — it is the source of the HOLDS_GRANT edge
```

**Edges (subject is the source — this is where subject lives):**

| Edge | Subject Type | Meaning |
|------|--------------|---------|
| `Agent ──HOLDS_GRANT──▶ Permission` | An agent | Agent-specific grant |
| `Project ──HOLDS_GRANT──▶ Permission` | A project | Project-scoped grant; propagates to all member agents |
| `Organization ──HOLDS_GRANT──▶ Permission` | An organization | Org-level ceiling; cannot be exceeded by anything inside the org |
| `Role ──HOLDS_GRANT──▶ Permission` | A role | Role-based grant; held by anyone occupying the role |
| `Agent ──ISSUED_GRANT──▶ Grant` | (provenance edge, not a holding edge) | Records which agent created this grant — feeds the `provenance` audit trail |

> **Why two edge types involving Agent?** `HOLDS_GRANT` is "the agent HOLDS this capability." `ISSUED_GRANT` is "the agent CREATED this capability." The same agent can do both — and a separate agent can be the grantor of a grant held by yet another agent. Provenance is the audit trail; subject is the holder.

### Why subject is structural, not a field

A single Grant node can be referenced by multiple `HOLDS_GRANT` edges from different subjects. Storing subject as a property would force one Grant node per subject, even when the capability shape is identical. Modeling subject as the edge source enables capability *templates* — define the Permission once, attach it to N subjects.

For example, an org-default "read project workspace" grant can be defined once and attached to every project in the org via N `HOLDS_GRANT` edges from the projects to the same Grant node. Provenance still tracks who CREATED the template (`config:org-default.toml`), and the resolution hierarchy still applies.

---
