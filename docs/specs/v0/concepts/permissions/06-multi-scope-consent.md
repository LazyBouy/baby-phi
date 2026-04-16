<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->


## Multi-Scope Session Access

> **Resolves Open Question 2 from earlier drafts.** This section is the canonical home for cross-project and cross-org session resolution. It applies the unified cascading rule from [Permission Resolution Hierarchy → Mechanism 2](04-manifest-and-resolution.md#mechanism-2-scope-resolution-specific-first-selection) to the specific case of session reads.

### The Hard Schema Constraint

A Session may have one of these tag shapes:

| Shape | Project tags | Org tags | When it arises |
|-------|--------------|----------|----------------|
| **A** | 1 | 1 | Standard single-org single-project work (the common case) |
| **B** | 1 | 2+ | Joint project — one project owned by multiple orgs |
| **C** | 2+ | 1 | Cross-project work within a single org |
| **D** | 0 | 1 | System session under an org but not in a project |

**Forbidden:** multi-project AND multi-org on the same session (a hypothetical "Shape E"). When work needs to span both axes, the system requires a **joint parent project** that itself is co-owned by the relevant orgs, and the session belongs to that parent. This collapses any would-be Shape E into Shape B.

Why the constraint matters: it ensures that scope resolution **never has to traverse two independent axes simultaneously**. The cascading rule always finds a unique answer because the multi-axis case is structurally impossible.

### The Unified Resolution Rule

When a reader attempts to read a session, the runtime resolves the reader's effective scope by cascading from most-specific to most-general:

```
fn resolve_scope(reader: Agent, session: Session) -> ResolvedScope {
    let session_projects = session.tags.filter(Tag::Project);
    let reader_project_matches = session_projects.intersection(reader.member_projects());

    // Project-level resolution (most specific)
    match reader_project_matches.count() {
        1 => return ResolvedScope::Project(reader_project_matches.first()),
        n if n > 1 => return ResolvedScope::Project(reader.base_project_among(reader_project_matches)),
        0 => {} // fall through to org-level
    }

    // Org-level resolution
    let session_orgs = session.tags.filter(Tag::Org);
    let reader_org_matches = session_orgs.intersection(reader.member_orgs());

    match reader_org_matches.count() {
        1 => ResolvedScope::Org(reader_org_matches.first()),
        n if n > 1 => ResolvedScope::Org(reader.base_org_among(reader_org_matches)),
        0 => ResolvedScope::Intersection(session_orgs),
    }
}
```

The resolved scope's grants apply, **bounded above** by the org caps of all orgs that own the resolved scope (this is Mechanism 1 from the Permission Resolution Hierarchy section).

**Tie-breaker:** When the reader belongs to multiple matching scopes at the same level, `base_project` (for projects) or `base_org` (for orgs) wins. This is consistent across both levels and depends only on stable agent state, not volatile context fields like `current_project`.

**Outsider rule:** A reader who belongs to none of the session's scopes at any level faces the **intersection of all the session's scope ceilings**. This is the strictest possible treatment and prevents loopholes where someone could create a permissive shadow scope to bypass restrictions.

### Worked Examples Across All Shapes

The examples below cover every allowed session shape with multiple reader roles. Each runs the resolution rule above and reports the resolved scope.

#### Shape A: 1 project, 1 org (the common case)

```yaml
session:
  tags: [agent:claude-coder-9, project:acme-internal, org:acme]
```

| Reader | Membership in session scopes | Resolved scope |
|--------|------------------------------|----------------|
| `claude-coder-9` (the owner) | project: acme-internal ✓ | **project:acme-internal** |
| `lead-acme-3` (lead of acme-internal) | project: acme-internal ✓ | **project:acme-internal** (with Template A grant) |
| `lead-other-7` (Acme employee, NOT in acme-internal) | project: ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`, so trivially that) |

#### Shape B: 1 project, 2 orgs (joint project)

```yaml
session:
  tags: [agent:<owner>, project:joint-research, org:acme, org:beta-corp]
```

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `lead-joint-3` (member of joint-research) | project ✓ | **project:joint-research** (project resolution succeeds, org axis irrelevant) |
| `lead-acme-7` (Acme but NOT in joint-research) | project ✗, org: acme ✓ (one match) | **org:acme** |
| `lead-beta-9` (Beta but NOT in joint-research) | project ✗, org: beta-corp ✓ (one match) | **org:beta-corp** |
| `dual-citizen-5` (Acme + Beta member, NOT in joint-research) | project ✗, org: both (two matches) | **base_org wins** — whichever is the agent's primary |
| `external-auditor` (neither org, not in project) | none | **intersection** of `{org:acme, org:beta-corp}` |

> **Why each lead reads under their own org's rules:** This is the contractor model. An Acme lead operates under Acme's rules even when reading joint-project data; a Beta lead operates under Beta's rules. Neither imposes their rules on the other. Only outsiders, who have no organizational citizenship, face the strict intersection.

#### Shape C: 2 projects, 1 org (cross-project within an org)

```yaml
session:
  tags: [agent:claude-coder-9, project:data-pipeline, project:ml-training, org:acme]
```

This shape arises when work legitimately spans two projects within the same org — for example, a workflow that touches both an extraction project and a training project. The system does not enforce single-project sessions, so this shape is allowed.

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `lead-pipeline-3` (member of data-pipeline only) | project: one match | **project:data-pipeline** |
| `lead-ml-7` (member of ml-training only) | project: one match | **project:ml-training** |
| `dual-project-2` (member of both projects) | project: two matches | **base_project wins** |
| `lead-other-9` (Acme, not in either project) | project ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`) |

> **Why each lead reads under their own project's rules:** Symmetric with Shape B at the org level. A pipeline lead reads under pipeline's rules even when the session also touches ml-training. Each project lead has scope authority over their own project's work, even when that work is shared.

#### Shape D: 0 projects, 1 org (system session)

```yaml
session:
  tags: [agent:platform-monitor-1, org:acme]
```

A System Agent doing platform maintenance for Acme, not associated with any user-facing project.

| Reader | Membership | Resolved scope |
|--------|------------|----------------|
| `platform-monitor-1` (the owner) | project ✗, org: acme ✓ | **org:acme** (or own session via Default Grant 1) |
| `acme-admin-9` (Acme admin) | project ✗, org: acme ✓ | **org:acme** |
| `external-auditor` (no Acme membership) | none | **intersection** (just `org:acme`) |

System sessions are governed entirely by the org's rules, since they have no project scope to fall back to.

#### Shape E (forbidden) — what the constraint prevents

```yaml
session:
  tags: [agent:..., project:A, project:B, org:acme, org:beta-corp]
                    ^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                    multi-project + multi-org on the same session
```

**Cannot exist.** When work needs to span multiple orgs AND multiple projects, the system either:
- Refuses to create the session, OR
- Requires a joint parent project P co-owned by `{acme, beta-corp}`, and creates the session with `[project:P, org:acme, org:beta-corp]` (Shape B)

The constraint is enforced at session creation. The frozen-at-creation rule then prevents the session from drifting into Shape E later.

### Why the Cascade Stops at the First Match

A reader who is a member of Shape B's joint-research project does **not** also have org-level resolution applied — they get the project-level scope and the org axis is irrelevant for resolution purposes (org caps still apply as upper bounds via Mechanism 1). This is the cascading-resolution principle:

- Project membership is more specific than org membership
- The reader's most-specific claim wins
- The runtime never has to choose between project rules and org rules — project rules take precedence whenever the reader has project-level membership

This avoids ambiguity: there is exactly one resolved scope per (reader, session) pair, and the rule picks it deterministically.

### Subject-Side Reach Is Bounded by Scope Membership

A subtle but important consequence of this rule: an agent's home org (`base_organization`) does **not** reach into sessions belonging to scopes the agent is not a member of. When `claude-coder-9` (base_org = Acme) works in Beta's single-org project, the session is tagged `org:beta-corp` only — no `org:acme` tag. Acme has no scope claim on this session, and Acme's policies cannot constrain `claude-coder-9`'s read of it.

This is the **contractor model** made explicit: when an agent operates in another org's scope, they follow that scope's rules, full stop. Acme governs `claude-coder-9` only when `claude-coder-9` is operating within Acme's scope (i.e., reading sessions tagged `org:acme`).

If Acme wants to impose policies that follow its agents everywhere, those policies are not part of the permission system — they belong to the contract Acme signed when sending its agents to work elsewhere.

### Co-Ownership × Multi-Scope Session Access

[Co-Ownership](01-resource-ontology.md#co-ownership-shared-resources) and Multi-Scope Session Access interact whenever a session carries more than one scope tag **and** each of those scopes corresponds to an independent owner. The canonical case is **Shape B** — a joint project co-owned by two organizations (e.g., `joint-research` co-owned by Acme and Beta). Sessions in this project carry both `org:acme` and `org:beta-corp` tags; both orgs hold `[allocate]` on the project resources.

The rules that resolve this interaction:

1. **Approval — per-resource slot fan-out.** When an Auth Request targets a co-owned resource, the [Per-Resource Slot Model](02-auth-request.md#schema--per-resource-slot-model) creates **one approver slot per co-owner** for that resource. All required slots must reach `Approved` for the resource to be included in the issued Grant. Either co-owner can Deny the slot independently; co-owner disagreement lands the resource in `Partial` state, where the owner-set can exercise `override-approve` or `close-as-denied` (see the [per-state access matrix](02-auth-request.md#per-state-access-matrix)).

2. **Ceilings — intersection, not union.** For [Mechanism 1 Ceiling Enforcement](04-manifest-and-resolution.md#mechanism-1-ceiling-enforcement-top-down-upper-bound), a co-owned session is subject to the **intersection** of all co-owner orgs' ceilings — the stricter bound wins on every axis. If Acme permits `bash` but Beta forbids it, sessions in `joint-research` forbid `bash`. The reasoning: a co-owner whose ceiling forbids an action has not authorized it on the shared resource, and co-ownership does not create authority neither party has.

3. **Scope resolution — standard cascade, base-org as tie-breaker.** For [Mechanism 2 Scope Resolution](04-manifest-and-resolution.md#mechanism-2-scope-resolution-specific-first-selection), the normal most-specific-first cascade applies. When two co-owner scopes are equally specific (e.g., both `org:acme` and `org:beta-corp` have a matching grant), the agent's `base_organization` field breaks the tie — the grant issued by the agent's home org wins. This is the same base-org rule used throughout Multi-Scope.

4. **Derived resources — base-org tagging.** When a co-owned session produces derived resources (extracted memories, logs, spawned sub-sessions), those derivatives are tagged with the creating agent's `base_organization`. A Memory extracted by `joint-acme-5` (base_org = Acme) from a `joint-research` session carries tags `[project:joint-research, org:acme, agent:joint-acme-5, derived_from:session:...]` — not `org:beta-corp`. This prevents derived artefacts from inheriting ambiguous multi-org ownership.

5. **Revocation — per-owner sub-trees.** When a co-owner revokes an allocation they issued, only that co-owner's sub-tree cascades. Grants issued under the other co-owner's authority are unaffected. The session itself remains accessible to holders whose grants descend from the other co-owner.

6. **Consent — per co-owner, evaluated independently.**

   Each co-owner's consent policy is evaluated on the subordinate being targeted, **independently** of every other co-owner's policy. A subordinate working in a co-owned project must hold valid Consent records under each co-owner org whose policy requires one. Policies do **not** intersect, merge, or cascade — they apply severally.

   **Worked reconciliation:** `joint-research` is co-owned by Acme (`consent_policy: implicit`) and Beta (`consent_policy: one_time`). Agent `joint-acme-5` is to be read by supervisor `lead-acme-1` under Template A.

   - Acme's `implicit` policy: Consent is auto-acknowledged at agent creation. No new request needed.
   - Beta's `one_time` policy: A Consent record scoped to Beta must exist in `Acknowledged` state. If it doesn't, the read is gated pending subordinate response.

   Net effect: **the read proceeds only when both per-co-owner checks pass**. Beta's policy is the binding one here because Acme's is satisfied by default. This is not "stricter policy wins" — both policies are evaluated, and both must allow the access, because each co-owner retains independent authority over their share of ownership.

   **Revocation under co-ownership:** if Beta's Consent is revoked, only the access paths authorised by Beta's share cascade-revoke. Acme-authorised paths (where Acme's implicit consent satisfied the check) remain valid because Acme's policy has not changed. Revocation is always per-co-owner — it walks only the revoker's subtree.

These rules preserve the single-scope behaviour in all non-co-owned cases and only activate when a resource genuinely has multiple owners.

#### Worked Example — Intersection on Ceilings, Union on Grants

Concrete scenario to cement the intersection-vs-union distinction.

**Setup:** Acme permits `bash` at the org-ceiling level; Beta forbids `bash` entirely. They co-own the joint project `joint-research`. Session `s-9901` in this project is tagged `[project:joint-research, org:acme, org:beta-corp, agent:joint-acme-5]`. Agent `joint-acme-5` has `base_organization = acme`.

**Case 1 — `joint-acme-5` tries to invoke `bash` inside `s-9901`.**

The ceiling check evaluates each co-owner org's ceiling independently:

- Acme's ceiling permits `bash`.
- Beta's ceiling forbids `bash`.

Per rule 2 (ceilings intersect), the effective ceiling on the co-owned session is the **intersection**: `bash` is forbidden. The tool invocation is denied. No grant — no matter how specifically it authorises `bash` — can override a forbidding ceiling.

**Case 2 — contrast with a non-co-owned session.**

Consider a different session `s-9500` in `acme-api-platform`, tagged only `[project:acme-api-platform, org:acme]`. Only Acme's ceiling applies; `bash` is permitted. Intersection only engages when multiple owner ceilings overlap on the same resource.

**Case 3 — union applies to individual grants.**

Back in `joint-research`. Suppose `joint-acme-5` holds two independent grants:

- Grant G1, issued under Acme's allocation authority: `[read]` on `session:s-9901`.
- Grant G2, issued under Beta's allocation authority: `[list]` on `session:s-9901`.

Both grants are simultaneously in effect. The agent can `read` (via G1) **and** `list` (via G2) on the session. A single grant cannot exceed its issuing co-owner's ceiling, but **two grants together can touch any scope either co-owner authorises**. That is union-at-the-grant-level.

**The rule, restated plainly.** Intersection applies to **ceilings** (the upper bounds a co-owner places on everything they own); union applies to **grants** (the specific authorities individual principals hold). This split is what keeps co-ownership additive at the grant level — letting both co-owners independently delegate authority within their own share — while remaining conservative at the ceiling level, where a forbidden action from any co-owner forbids it globally.

---

## Consent Policy (Organizational)

> **Resolves Open Question 1 from earlier drafts.** Subordinate consent is now an organizational policy expressed through the standard `approval_mode` machinery, rather than a separate concept.

### Three Consent Policies

The consent policy is **configurable per organization**, with the default set to **`implicit`**. This keeps the common case frictionless — orgs that need stronger consent guarantees can opt into `one_time` or `per_session` without affecting orgs that don't.

Each Organization picks one consent policy that applies to its Authority Templates (A, B, C, D — Template E is always explicit and doesn't need consent gating):

| Policy | When Authority Template Grants Fire | Storage |
|--------|--------------------------------------|---------|
| **Implicit (default)** | Auto-issue grants immediately when relationship edge is created | Consent is implied by joining the project/org |
| **One-time consent** | Auto-issue grants only after the subordinate signs a one-time consent record | A `Consent` node attached to the Agent, scoped to the org |
| **Per-session consent** | Templates issue grants with `approval_mode: subordinate_required` — every read attempt blocks until the subordinate approves it | Real-time approval, recorded per Permission Check |

The consent policy lives on the Organization node:

```yaml
organization:
  org_id: acme
  consent_policy: one_time   # implicit | one_time | per_session
  consent_scope:
    templates: [A, B, C, D]    # which templates the policy applies to
    excluded_actions: []       # actions exempt from consent (e.g. emergency reads)
```

### Implicit Consent (Default)

```
Edge created (e.g. HAS_LEAD) → grant auto-issued immediately
```

This is the current behavior. No consent step. The act of joining a project/org is taken as implicit consent to standard supervisory access.

### One-Time Consent

```
Edge created (e.g. HAS_LEAD)
    │
    ▼
Does subordinate have a Consent node for this template + org?
    │
    ├── Yes → grant auto-issued
    │
    └── No → grant queued; subordinate prompted to consent
              On consent → grant issued; Consent node created
              On refusal → grant never issued; supervisor sees a "no consent" status
```

The Consent node looks like:

```yaml
consent:
  consent_id: c-4581
  agent_id: claude-coder-9        # the subordinate
  scope:
    org: acme
    templates: [A, B]
    actions: [read, inspect]
  granted_at: 2026-04-09T12:00:00Z
  revocable: true
  provenance: agent:claude-coder-9@onboarding
```

A typical place to collect one-time consent is at agent creation (for a base-org consent record) or at project join (for a project-specific consent record). The org policy decides which.

### Per-Session Consent

The most invasive option. Templates auto-issue grants with `approval_mode: subordinate_required`. When the supervisor invokes `read_session`:

```
Permission check passes (grant exists, selector matches)
    │
    ▼
approval_mode is subordinate_required
    │
    ▼
Notify subordinate via their Channel (or in-band if subordinate is an LLM agent)
    │
    ├── Approved → read proceeds
    ├── Denied → read denied with audit log entry
    └── Timeout reached → default response (deny) applied with audit log entry
```

The pattern reuses `human_approval_required`, just naming a new approval principal: `subordinate_required`. Both go through the same `approval_mode` machinery already documented in the Grant section.

#### Approval Timeout

Each `subordinate_required` approval request carries a **timeout**, configurable per organization. The default timeout is the **remaining duration of the project** that owns the session being read:

```yaml
organization:
  org_id: acme
  consent_policy: per_session
  approval_timeout: project_duration   # default; alternatives: a fixed Duration like "24h"
  approval_timeout_default_response: deny   # default; alternative: allow
```

**Why project duration as the default:**

- Approval requests are meaningful only while the project is active. Once the project closes, supervisor reads on its sessions are typically retrospective audits, which are governed by the archived-project rules (see Open Question on time-bounded authority for archived projects), not by per-session consent.
- It avoids requiring orgs to invent arbitrary timeout numbers ("is 24h right? 7 days?") — the natural lifecycle of the work itself is the answer.
- It scales with project complexity: a one-week sprint gets a one-week timeout; a six-month engagement gets a six-month timeout.
- A subordinate who is genuinely unreachable for the duration of an entire project has effectively withdrawn from the work — the timeout-deny default surfaces that fact rather than blocking the supervisor indefinitely.

**Computing "project duration" across session shapes:** The session's tags determine which project deadline(s) apply.

| Session shape | Project tags | How `project_duration` resolves |
|---------------|--------------|----------------------------------|
| **A** (1 project, 1 org) | exactly one `project:P` | The remaining duration of project `P` |
| **B** (1 project, joint orgs) | exactly one `project:P` (the joint project) | The remaining duration of the joint project `P` |
| **C** (multi-project within an org) | multiple `project:P1, project:P2, ...` | The **maximum** remaining duration among all the session's projects — i.e., the timeout lasts as long as *any* of the projects is still active |
| **D** (no project — system session) | zero `project:` tags | No project to anchor the timeout. Falls back to the org-level default (or a fixed Duration if the org configured one). If neither is set, an explicit fixed timeout is required. |

The "max across projects" rule for Shape C reflects the same intuition as the rest of the Multi-Scope rule: **the timeout is bounded by the most permissive project context that still claims the session**. If any of the session's projects is still active, the work that produced the session is still meaningful, so the supervisor's approval request remains relevant.

**Alternative timeouts:** Orgs that want tighter loops can set a fixed Duration (e.g. `"24h"`). The runtime treats whichever is shorter as the effective timeout — if the org sets `24h` but the resolved project duration is `2h`, the request times out in `2h`.

**Default response on timeout:** `deny`, on the principle that absence of consent is not consent. Orgs that want the opposite (e.g., for low-risk read operations where unblocking the supervisor matters more than strict consent) can set `approval_timeout_default_response: allow`.

### Consent Node (New Node Type)

| Property | Type | Description |
|----------|------|-------------|
| `consent_id` | String | Unique identifier |
| `agent_id` | agent_id | The subordinate giving consent |
| `scope.org` | org_id | The org under whose policy this consent operates |
| `scope.templates` | Vec<TemplateId> | Which Authority Templates this consent covers |
| `scope.actions` | Vec<Action> | Which actions are consented to |
| `state` | `ConsentState` enum | Lifecycle state (see below) |
| `requested_at` | DateTime | When the consent was first requested of the subordinate |
| `responded_at` | Option<DateTime> | When the subordinate acknowledged or declined (null until the subordinate acts) |
| `revoked_at` | Option<DateTime> | When the subordinate later withdrew consent (null unless state is Revoked) |
| `revocable` | bool | Whether the subordinate can later withdraw consent |
| `provenance` | String | Audit trail (typically `agent:{subordinate_id}@{event}`) |

**Edges:**
- `Agent ──HAS_CONSENT──▶ Consent` — the subordinate holds the consent record
- `Consent ──SCOPED_TO──▶ Organization` — the org under whose policy it applies

### Consent Lifecycle

A Consent node progresses through the following states:

```
  (policy triggers a request)
             │
             ▼
        Requested ──subordinate acknowledges──▶ Acknowledged
             │                                       │
             │                                       │ (revocable &
             │                                       │  subordinate chooses)
             │                                       ▼
             │                                    Revoked
             │
             ├─subordinate declines──▶ Declined
             │
             └─timeout (see policy)──▶ TimedOut
                                           │
                                           ▼
                                  (policy maps to
                                   deny or allow —
                                   see per-session
                                   timeout rules)
```

**States:**

| State | Meaning |
|-------|---------|
| `Requested` | The runtime has asked the subordinate for consent; awaiting response |
| `Acknowledged` | The subordinate agreed; the covered actions are now permitted under this consent |
| `Declined` | The subordinate refused; covered actions are blocked (and the attempting supervisor is notified) |
| `Revoked` | The subordinate previously Acknowledged, then withdrew — forward-only: past actions stand; future actions blocked until a fresh Consent is Acknowledged |
| `TimedOut` | `Requested` reached its timeout without a response. The policy decides the default (`deny` by default; orgs may configure `allow` for low-risk reads) |
| `Expired` | The Consent's natural scope ended — e.g. a `per_session` consent outlives its session; a `one_time` consent outlives the `current_organization` relationship |

**Per-policy mapping:**

| Policy | What triggers `Requested` | What reaches `Acknowledged` |
|--------|----------------------------|------------------------------|
| `implicit` | Never — policy short-circuits; Consent is auto-`Acknowledged` at agent creation, no request is sent | Automatic |
| `one_time` | The first Authority-Template fire that targets this subordinate within the org | Explicit subordinate response (or a pre-registered standing acknowledgement) |
| `per_session` | Every new Session in which the subordinate is targeted | Explicit subordinate response per session |

**Request/response channel:** Consent requests travel through the subordinate's Channel (Slack, email, web UI — see [ontology.md → Channel](ontology.md)). Acknowledgement, Declining, and Revocation are each separate explicit actions; the runtime does not infer consent from silence (silence maps to `TimedOut`).

**Revocation semantics** are covered in the next subsection; in short: forward-only.

### Interaction with Template E

Template E (manual explicit grants) **does not go through consent**, because it represents a deliberate one-off authorization (e.g., a sponsor granting an auditor read access for a week). The provenance of Template E is always a specific human or admin agent, and the audit class is typically `alerted`. Consent is implicit in the act of issuing the grant.

### Consent Revocation Semantics

> Resolved: revocation applies forward only.

When a subordinate revokes a previously granted `one_time` consent (or denies a `per_session` approval request), the revocation applies to **future actions only**. Reads that have already happened under the consent are not retroactively undone — there is no "unread" operation, and audit logs of past reads remain intact.

This puts the responsibility for both granting and revoking consent squarely on the agent, and avoids two thorny problems:

1. **No retroactive cleanup** — the system doesn't have to track which extracted memories or downstream artifacts originated from a now-revoked consent and try to undo them. That would be an unbounded cascade with no clean stopping point.
2. **No "consent rollback" race conditions** — concurrent reads in flight at the moment of revocation are not interrupted; they complete, and any read attempted *after* the revocation point is denied.

In practical terms:

- For `one_time` consent: revocation deletes the Consent node (or marks it `revoked: true` for audit). The next time the runtime checks for the supervisor's Authority Template grant, the consent precondition fails and the grant is not issued (or is revoked if already issued).
- For `per_session` consent: each approval request is independent. A subordinate who has approved one read can deny the next one with no retroactive effect on the first.

### Open Questions for Consent

- [x] ~~**Default policy**~~ — Resolved: configurable per org, default `implicit`. See [Three Consent Policies](#three-consent-policies) above.
- [x] ~~**Consent revocation cascade**~~ — Resolved: revocation applies forward only. See [Consent Revocation Semantics](#consent-revocation-semantics) above.
- [x] ~~**Per-session consent timeout**~~ — Resolved: configurable per org, default is the remaining project duration; default response on timeout is `deny`. See [Approval Timeout](#approval-timeout) under Per-Session Consent above.

> All consent open questions resolved for v0. Future iterations may revisit these defaults based on real usage patterns.

---

## Open Questions for Session Permissions

- [x] ~~**Subordinate consent**~~ — Resolved by the [Consent Policy](#consent-policy-organizational) section above.
- [x] ~~**Cross-org session ceilings**~~ — Resolved by the [Multi-Scope Session Access](#multi-scope-session-access) section above.
- [ ] **Time-bounded authority for archived projects:** When a project ends and its sessions become `#archived`, do Template A grants survive (so the lead can still review historical work) or are they revoked (forcing re-authorization for any post-mortem)?
- [ ] **Conflict resolution during disputes:** During a rating dispute, can a sponsor override a project lead's authority to read disputed sessions? Probably yes via Template E, but should there be a structured "dispute grant" template?
- [ ] **Granularity escalation:** Should a `list` grant be auto-upgraded to `inspect` after some threshold? (E.g., "if you've listed it 10 times, you probably need to see it.") Probably no — feels like over-engineering.
- [ ] **Sessions with no project:** What about ad-hoc sessions created outside any project (e.g., a system agent doing platform maintenance)? They get an `org:` tag but no `project:` tag. Templates A/B/D don't fire; Templates C/E may.
- [ ] **Re-parenting:** If a session was created under one project and the project is later merged into another, do the tags update? Currently no (frozen rule). Is there a controlled "re-parent" operation that the system performs and audits?
- [ ] **Tag schema extensibility:** Should the tag vocabulary be open (agents can invent new tag namespaces) or closed (only system-defined tags)? Currently closed, which keeps reasoning tractable.

---
