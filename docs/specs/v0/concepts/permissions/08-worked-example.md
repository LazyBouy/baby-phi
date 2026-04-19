<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Worked End-to-End Use Case: Three Orgs, Twelve Projects

> **Complementary catalogue.** This single end-to-end example is complemented by the 10-layout organization catalogue at [`../../organizations/`](../../organizations/README.md) and the 5-layout project catalogue at [`../../projects/`](../../projects/README.md). The 3-orgs-12-projects walkthrough below is depth-first (one scenario, every concept exercised); the two catalogues are breadth-first (many scenarios, each exercising a subset of the concepts). Both are intended to be read together.
>
> **Purpose:** Demonstrate every concept in this doc together — org ceilings, project grants, agent defaults, Authority Templates, consent policies, Multi-Scope Session Access, resource overlap, contractor scenarios, and the forbidden Shape E recovery. This is the "tabletop test" of the model.
>
> **Setup:** A fictional but realistic organizational structure. Each org starts from the Standard Organization Template introduced above, then customizes.

### Cast of Organizations and Projects

**Organizations (3):**

- **Acme** — primary tenant; runs three internal projects and co-sponsors one joint project. Adopts the Standard Organization Template as-is.
- **Beta Corp** — secondary tenant; runs three internal projects and co-sponsors the joint project. Customizes: `bash` is entirely disabled (not merely sandbox-required), `consent_policy: one_time`.
- **Gamma Consulting** — small consultancy; runs four internal projects, no joint work. Customizes: tighter execution limits, `consent_policy: per_session`.

**Projects (12, 4 per org):**

| Org | Projects |
|-----|----------|
| Acme | `acme-internal-tools`, `acme-website-redesign`, `acme-api-platform`, `joint-research` (co-owned with Beta) |
| Beta | `beta-data-pipeline`, `beta-mobile-app`, `beta-billing`, `joint-research` (co-owned with Acme) |
| Gamma | `gamma-client-a-audit`, `gamma-client-b-migration`, `gamma-internal-ops`, `gamma-knowledge-base` |

`joint-research` is a **single project node co-owned by Acme and Beta** — Shape B from Multi-Scope Session Access. Sessions in `joint-research` carry both `org:acme` and `org:beta-corp` tags.

### Cast of Agents (~14 total)

**Acme agents (5):**
- `lead-acme-1` — Contract agent, lead of `acme-website-redesign`
- `coder-acme-2` — Intern agent, member of `acme-internal-tools`
- `coder-acme-3` — Contract agent, member of `acme-api-platform`
- `auditor-acme-4` — System agent, platform monitoring across all Acme projects
- `joint-acme-5` — Contract agent, member of both `acme-api-platform` and `joint-research`

**Beta agents (4):**
- `lead-beta-1` — Contract agent, lead of `beta-billing`
- `coder-beta-2` — Intern agent, member of `beta-data-pipeline`
- `joint-beta-3` — Contract agent, member of `joint-research`
- `monitor-beta-4` — System agent, platform monitoring for Beta

**Gamma agents (4):**
- `lead-gamma-1` — Contract agent, lead of `gamma-client-a-audit`
- `coder-gamma-2` — Contract agent, member of `gamma-client-b-migration`
- `auditor-gamma-3` — System agent, internal ops
- `compliance-gamma-5` — Contract agent, read-only auditor on `gamma-client-a-audit` with memory-only access (asymmetric scenario)

**Cross-org agents (1):**
- `contractor-x-9` — Contract agent, base_org = Gamma, currently contracted into `acme-website-redesign`

### Permission Setup Walkthrough

#### Step 0: Template adoption and resource ownership

Each org starts from the Standard Organization Template (see above) and customizes. At org/project creation, ownership of the associated resources is established per the [Resource Ownership](01-resource-ontology.md#resource-ownership) rules:

- `/workspace/acme/**` → owned by `org:acme`
- `/workspace/acme/acme-website-redesign/**` → owned by `project:acme-website-redesign` (most-specific scope wins)
- `/workspace/joint-research/**` → **co-owned** by `org:acme` and `org:beta-corp` (Shape B joint project)
- Each agent's `/home/{agent_id}/**` → owned by that agent
- Session and memory resources tagged with a project are owned by that project; else by the creating agent

Each project lead holds `scope: [allocate]` on their project's workspace — issued automatically by Authority Template A (pre-authorized at template adoption time). Project co-owners each hold `scope: [allocate]` independently; either can issue sub-grants within their authority.

The deltas are:

**Acme** — adopts as-is:
```yaml
organization: acme
inherits_from: standard
customizations: none
```

**Beta** — disables `bash` and tightens consent:
```yaml
organization: beta
inherits_from: standard
customizations:
  tools_allowlist_remove: [bash]
  consent_policy: one_time
```

**Gamma** — tightest cost caps, per-session consent:
```yaml
organization: gamma
inherits_from: standard
customizations:
  execution_limits:
    max_cost_usd: 2.00          # tighter than standard 5.00
    max_turns: 30
  consent_policy: per_session
  approval_timeout: project_duration
  approval_timeout_default_response: deny
```

This single step replaces dozens of individual grants — the templates handle the defaults.

#### Step 1: Project-level grants for `joint-research`

The joint project inherits from both Acme AND Beta org templates. Both orgs' ceilings apply as upper bounds. The project lead authors project-specific grants:

```yaml
project: joint-research
owned_by: [acme, beta]
permissions:
  # Inherits Standard Project Template
  # PLUS project-specific additions:

  - action: [read, store]
    resource: memory_object
    selector: "tags contains project:joint-research"
    kind: [memory]
    provenance: config:joint-research.toml
    delegable: false

  - action: [read, list, inspect]
    resource: session_object
    selector: "tags contains project:joint-research"
    kind: [session]
    provenance: config:joint-research.toml
    delegable: false

# Effective ceiling: intersection of Acme and Beta org ceilings.
# Since Beta disabled bash entirely, joint-research inherits that — no agent in
# joint-research can use bash, even an Acme agent. Subject-side ceilings from the
# reader's base_org do NOT reach into this project (contractor model), but the
# project's own effective ceiling is the intersection of its owning orgs.
```

#### Step 2: Authority Template auto-issuance

**When `lead-acme-1` is appointed lead of `acme-website-redesign`**, Template A fires:

```yaml
grant:
  # subject = agent:lead-acme-1
  action: [read, inspect, list]
  resource: session_object
  selector: "tags contains project:acme-website-redesign"
  kind: [session]
  constraints: {}
  provenance: system:has_lead@project:acme-website-redesign
  delegable: false
  approval_mode: auto   # Acme uses implicit consent
  audit_class: logged
  revocation_scope: revoke_when_edge_removed
```

**When `contractor-x-9` is brought into `acme-website-redesign`**, project membership grants apply but NO Template A grant fires (they're not a lead). The contractor inherits `acme-website-redesign`'s project template grants for the duration of the contract.

**When `lead-acme-1` delegates work to `contractor-x-9` at loop `L42`**, Template B fires:

```yaml
grant:
  # subject = agent:lead-acme-1
  action: [read, inspect]
  resource: session_object
  selector: "tags contains delegated_from:L42"
  kind: [session]
  constraints: {}
  provenance: system:delegation@L42
  delegable: false
  approval_mode: auto
  audit_class: logged
  revocation_scope: end_of_delegation_chain
```

#### Step 3: Consent policies in action

**Acme's `implicit` policy:** `lead-acme-1` reads a session owned by `coder-acme-3`. The read proceeds immediately without any consent prompt. Audit log records the read.

**Beta's `one_time` policy:** When `lead-beta-1` was first appointed lead of `beta-billing`, each member signed a one-time consent record:
```yaml
consent:
  agent_id: coder-beta-2
  scope:
    org: beta
    templates: [A, B]
    actions: [read, inspect]
  granted_at: 2026-04-05T10:00:00Z
  revocable: true
```
Subsequent reads by `lead-beta-1` proceed because the consent record exists.

**Gamma's `per_session` policy:** `lead-gamma-1` tries to read a session owned by `coder-gamma-2`. The runtime notifies `coder-gamma-2` via their Channel. The coder approves (or denies, or times out) for this specific read. If approved, the read proceeds; if denied or timed out (after project_duration), the read is denied with an audit entry.

#### Step 4: Multi-Scope resolution for joint-research

Session tagged `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5]`.

**Reader: `joint-acme-5`** (member of joint-research):
- Project resolution: reader is in joint-research ✓
- Resolved scope: `project:joint-research`
- Joint project's rules apply (capped by intersection of Acme + Beta org ceilings — so `bash` denied)
- **Allowed**

**Reader: `lead-acme-1`** (not in joint-research, base_org = Acme):
- Project resolution: reader not in joint-research ✗
- Org resolution: reader's base_org is Acme, session has `org:acme` ✓
- Resolved scope: `org:acme`
- Acme's rules apply
- **Allowed** (subject to grant match)

**Reader: `lead-beta-1`** (not in joint-research, base_org = Beta):
- Project resolution: reader not in joint-research ✗
- Org resolution: reader's base_org is Beta, session has `org:beta-corp` ✓
- Resolved scope: `org:beta-corp`
- Beta's rules apply
- **Allowed** (subject to grant match AND Beta's one_time consent)

**Reader: `lead-gamma-1`** (base_org = Gamma, no membership in Acme or Beta):
- Project resolution: ✗
- Org resolution: Gamma is neither Acme nor Beta ✗
- Resolved scope: `Intersection(org:acme, org:beta-corp)`
- Both ceilings apply
- **Denied** — the intersection is highly restrictive, and Gamma has no grant to read joint-research sessions anyway

#### Step 5: Resource overlap example

`coder-acme-3` runs `cat /workspace/acme-api-platform/.env` via bash.

Entity classification: `/workspace/acme-api-platform/.env` → `{filesystem_object, secret/credential}`

Plus manifest-level requirements from bash: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Union: `{process_exec_object, filesystem_object, network_endpoint, secret/credential}`

Permission check per fundamental:
- `process_exec_object`: agent has bash grant ✓
- `filesystem_object`: agent has workspace grant ✓
- `network_endpoint`: check for network grant... agent has no network grant in its default set, but this specific invocation doesn't actually touch the network. ✗ **Denied**.

Wait — this illustrates a key point. The `bash` manifest declares transitive `network_endpoint`, so even a `cat` invocation that doesn't actually hit the network requires a `network_endpoint` grant. This is the **[conservative over-declaration](03-action-vocabulary.md#conservative-over-declaration-on-tool-manifests)** principle at work: the manifest declares the tool's *maximum* reach, not its per-invocation reach. To fix, either:
- Use a more narrow tool (e.g., `read_file` which has no transitive network) for reading files
- Grant `coder-acme-3` a narrow `network_endpoint` grant scoped to a domain allowlist

**If the agent had only `read_file`:** the permission check would be just `{filesystem_object, secret/credential}`. The agent has filesystem read on the project workspace. Does it have `secret/credential`? Under the Standard Project Template, no — secrets require explicit grants. So **denied** because of the `.env` file's entity-level overlap with `secret/credential`, not because of a missing filesystem grant.

#### Step 6: Asymmetric tagged-composite access (the keystone example)

`compliance-gamma-5` is an auditor agent granted **memory_object read only** on `project:gamma-client-a-audit`. Sessions are off-limits. Grant:

```yaml
grant:
  # subject = agent:compliance-gamma-5
  action: [recall, read]
  resource: memory_object   # composite
  selector: "tags contains project:gamma-client-a-audit"
  kind: [memory]
  provenance: agent:human-sarah@2026-04-09
  delegable: false
```

**Scenario A — auditor reads a project memory:**
- Target entity: Memory node with tags `{project:gamma-client-a-audit, agent:coder-gamma-2, #kind:memory}`
- Required fundamentals: `{data_object, tag}`
- Agent's grant resolves to: fundamentals `{data_object, tag}`, effective selector `tags contains project:gamma-client-a-audit AND tags contains #kind:memory`
- Selector match: `project:gamma-client-a-audit` ✓, `#kind:memory` ✓
- **Allowed** ✓

**Scenario B — auditor tries to read a project session:**
- Target entity: Session node with tags `{project:gamma-client-a-audit, agent:coder-gamma-2, #kind:session, org:gamma}`
- Required fundamentals: `{data_object, tag}`
- Agent's grant (same as above) has effective selector requiring `#kind:memory`
- Selector match: `project:gamma-client-a-audit` ✓, but `#kind:memory` ✗ (session has `#kind:session`)
- No grant matches → **Denied** ✗

**Scenario C — what if the sponsor had granted bare fundamentals?**

```yaml
# WRONG grant — too broad
grant:
  action: [read]
  resource: data_object   # fundamental, not composite
  # plus an implicit tag grant
  selector: "tags contains project:gamma-client-a-audit"
  # no kind filter
```

Both memory AND session reads would be allowed because there's no `#kind:` filter. This is why **composite vs bare-fundamental distinction matters** even though the runtime check runs on fundamentals.

#### Step 7: Contractor scenario

`contractor-x-9` (base_org = Gamma) reads sessions in `acme-website-redesign`:

- Session tags: `[project:acme-website-redesign, org:acme, agent:coder-acme-3, #kind:session]`
- Multi-Scope resolution: reader is a member of `acme-website-redesign` (via the contract) → project-level resolution succeeds → `project:acme-website-redesign` applies
- Gamma's subject-side ceilings do **not** reach (the session has no `org:gamma` tag)
- Acme's project rules apply
- The contractor operates entirely under Acme's rules for the duration of the contract
- **Allowed** if the contractor has the project grants

This is the contractor model from Multi-Scope Session Access — the reader's base_org is irrelevant when a project-level resolution succeeds.

#### Step 8: Forbidden Shape E recovery

`joint-acme-5` starts work that spans `acme-api-platform` AND `joint-research`. Attempting to create a single session with tags `[project:acme-api-platform, project:joint-research, org:acme, org:beta-corp]` is **rejected** at session creation time — this is Shape E (multi-project AND multi-org), which is forbidden.

**Valid alternatives:**

1. **Two separate sessions**, one per project:
   - Session 1: `[project:acme-api-platform, org:acme, agent:joint-acme-5]`
   - Session 2: `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5, delegated_from:<session_1_loop>]`
   - Linked via the `delegated_from:` tag

2. **A new joint parent project** (`acme-beta-platform-research`) co-owned by all three scopes, with a single session: `[project:acme-beta-platform-research, org:acme, org:beta-corp, agent:joint-acme-5]` — this is Shape B, which is allowed.

The session creation layer enforces the Shape E constraint; the frozen-at-creation rule then prevents drift into Shape E afterward.

#### Step 9: Ad-hoc Auth Request and revocation cascade

An outside auditor agent `auditor-x-1` (base_org = Gamma) needs temporary read access to two specific Acme sessions for a retrospective review. No Authority Template covers this — it's an ad-hoc cross-org request, so an explicit Auth Request is required.

**9.1 — Submission:**

```yaml
auth_request:
  request_id: req-9001
  requestor: agent:auditor-x-1
  kinds: [session]
  scope: [read, list]
  state: Pending
  valid_until: 2026-05-01T00:00:00Z
  submitted_at: 2026-04-15T10:00:00Z
  resource_slots:
    - resource: session:s-9831
      approvers:
        - approver: agent:lead-acme-1   # project lead (owner) of acme-website-redesign
          state: Unfilled
    - resource: session:s-9832
      approvers:
        - approver: agent:lead-acme-1
          state: Unfilled
  justification: "Q1 retrospective review across Acme web projects"
  audit_class: logged
```

**9.2 — Approval:** `lead-acme-1` approves slot for `session:s-9831` (state: Approved) and denies the slot for `session:s-9832` (sensitive — contains credential-reset flow). The request reaches terminal state `Partial`. A Grant is issued covering only the approved resource:

```yaml
grant:
  grant_id: grant-9001
  # HOLDS_GRANT edge → agent:auditor-x-1
  action: [read, list]
  resource:
    type: session_object
    selector: "tags contains session:s-9831"
  constraints: {}
  provenance: auth_request:req-9001
  expires_at: 2026-05-01T00:00:00Z
  revocation_scope: tied_to_auth_request
  audit_class: logged
```

**9.3 — Revocation cascade:** On 2026-04-22, the project's scope changes and `lead-acme-1` revokes `auth_request:req-9001`. Because `grant-9001` has `revocation_scope: tied_to_auth_request`, it auto-revokes. Past reads remain in audit logs; future reads are denied. No downstream sub-allocations existed (the auditor is a leaf), so the cascade terminates immediately.

If `auditor-x-1` had been issued sub-allocations based on `grant-9001` (impossible here since the scope did not include `allocate`, but illustrative), those would also be revoked forward — the cascade is tree-wide forward-only.

**9.4 — What the walkthrough demonstrates:**

- Atomic per-resource slots produce partial Grants (only `session:s-9831` in the final Grant).
- `Partial` is a useful outcome, not a failure — the requestor received the access they needed for one session and can submit a narrower request if they want to escalate the `s-9832` denial.
- The Grant's `provenance` is a structural pointer (`auth_request:req-9001`), not a string.
- Revocation of the Auth Request cascades to the Grant via `revocation_scope: tied_to_auth_request`.
- The authority chain for this Grant: `grant-9001 → auth_request:req-9001 → agent:lead-acme-1 (as project lead/owner) → Template A adoption Auth Request → Standard Organization Template adoption → System Bootstrap Template`.

### Summary: Who Can Read What

A condensed access matrix showing which agents can read which session types after all the setup above. This is the "tabletop test" result.

| Agent | Own sessions | Sessions in base project | Sessions in other projects (same org) | joint-research sessions | Cross-org sessions |
|-------|-------------|-------------------------|----------------------------------------|-------------------------|---------------------|
| `lead-acme-1` | ✓ (default) | ✓ (Template A on website-redesign) | list/inspect only | ✓ (via org:acme resolution) | — |
| `coder-acme-2` | ✓ (default) | list/inspect only (intern, no Template A) | list/inspect only | — | — |
| `joint-acme-5` | ✓ (default) | list/inspect only | list/inspect only | ✓ (project member) | — |
| `lead-beta-1` | ✓ (default) | ✓ (Template A on billing) | list/inspect only | ✓ (via org:beta resolution, with one_time consent) | — |
| `coder-beta-2` | ✓ (default) | list/inspect only | — | — | — |
| `lead-gamma-1` | ✓ (default) | ✓ (Template A on client-a-audit, with per_session consent) | — | ✗ (intersection fallback) | — |
| `compliance-gamma-5` | ✓ (default) | memory only on client-a-audit (no session access) | — | — | — |
| `contractor-x-9` | ✓ (default) | ✓ on acme-website-redesign (as contracted member) | — | — | — |
| `auditor-acme-4` (System) | N/A (system) | list/inspect across all Acme projects | list/inspect | list/inspect on joint-research | — |

**What the matrix demonstrates:**

- Default grants cover own-session reads universally
- Templates A and B are what enable reading other agents' sessions within the project
- Interns (like `coder-acme-2`) can't supervise, only work
- Joint project membership wins over org-level resolution
- Consent policies vary per org without affecting the structural rules
- Cross-org read is restricted to project-level participation or explicit Template E grants
- Asymmetric composite access (`compliance-gamma-5`) is expressible via `#kind:memory` scoping

This completes the end-to-end demonstration. Every concept in the doc — from fundamentals and composites through Authority Templates to consent policies — is exercised in a single coherent setup.

---

## phi-core Extension Points

Permissions are a baby-phi concern. phi-core provides the hooks:

| phi-core hook | Permission enforcement |
|---|---|
| `InputFilter` | Check `read` permission on `data_object` before message reaches agent |
| `BeforeToolExecutionFn` | Check tool's authority manifest against agent's permissions |
| `ExecutionLimits` | Enforce `time_compute_resource` and `economic_resource` constraints |
| `BeforeLoopFn` | Check `delegate` permission before sub-agent loop starts |

---

## Open Questions

### Resolved

- [x] **How are permissions bootstrapped?** — Resolved by the [System Bootstrap Template](02-auth-request.md#system-bootstrap-template-root-of-the-authority-tree). Initialization adopts a hardcoded template, which emits an Auth Request approved by the axiomatic `system:genesis` principal, allocating `system:root` authority to the platform admin.
- [x] **Should there be a "root" permission that is non-revocable?** — Resolved. The bootstrap adoption Auth Request is naturally non-revocable: no principal exists upstream of `system:genesis` to authorize revocation. No special `non_revocable` flag is needed.
- [x] **How do permissions interact with the Market?** — A market posting is an Auth Request whose approver is the poster; on contract acceptance, the approved Grant is issued to the winning bidder. Routing is the poster's responsibility; `valid_until` limits exposure.
- [x] **Should audit_class be per-permission or per-action?** — Per-Grant, inherited from the originating Auth Request. Per-Grant override is allowed when a specific use is more or less sensitive than the umbrella approval.

### Still open

- [ ] How do MCP server capabilities interact with the resource ontology?
- [ ] Auth Request TTL defaults beyond the 90-day active-window baseline — do audit-heavy regulated orgs need a standard "long window" template?
- [ ] Bulk Auth Requests that span hundreds of resources — is the per-resource slot model ergonomic, or do we need a "template match" selector at request time?
