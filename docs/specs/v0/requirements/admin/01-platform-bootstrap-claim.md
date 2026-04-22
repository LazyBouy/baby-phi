<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 1 of fresh-install journey -->

# 01 — Platform Bootstrap Claim

## 2. Page Purpose + Primary Actor

The first page a Human Agent encounters after a fresh install. The install process emits a **bootstrap credential** (a single-use URL + token printed by the install CLI, or presented as an environment variable). Using this credential, a Human Agent **claims the platform admin role** — the act materialises the System Bootstrap Template's adoption Auth Request, grants `[allocate]` on `system:root` to the claimant, and emits the alerted audit event that is the genesis of the organization's audit trail.

**Primary actor:** the first Human Agent of this install. No grants exist yet; authority derives from holding the bootstrap credential. After claim completes, this Human Agent is the **platform admin**.

**Secondary actors:** none. No other Agent exists in the system at this point.

## 3. Position in the Journey

- **Phase:** 1 of 9 (Claim platform admin role).
- **Depends on:** fresh install (the install process must have produced a bootstrap credential).
- **Enables:** Phase 2 (platform resource setup) and all subsequent phases.

## 4. UI Sketch

```
┌─────────────────────────────────────────────────────────────────┐
│                    phi  —  Welcome                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  No administrator has been assigned yet.                        │
│                                                                 │
│  Paste the bootstrap credential printed by your installer.      │
│                                                                 │
│  Bootstrap credential                                           │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ bphi-bootstrap-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  Your identity (you will become the platform admin)             │
│  Display name:   [ Alex Chen                                  ] │
│  Contact channel ▼ [Slack ▾]   Handle: [ @alex                ] │
│                                                                 │
│             [  Claim platform admin role  ]                     │
│                                                                 │
│  By claiming you acknowledge this action is AUDITED.            │
│  An `alerted` audit event will be emitted on success.           │
└─────────────────────────────────────────────────────────────────┘

Empty state: (the default state above — no admin exists yet)
After claim: redirect to Phase 2 landing page (02-platform-model-providers.md).
Error state: inline banner "Invalid or expired bootstrap credential — contact your installer."
```

## 5. Read Requirements

- **R-ADMIN-01-R1:** The page SHALL detect the absence of a platform admin and display the claim form. If a platform admin already exists, the page SHALL display "A platform admin has already been assigned" and refuse to accept further claims.
- **R-ADMIN-01-R2:** The page SHALL display a brief notice that claiming the role emits an `alerted` audit event.

## 6. Write Requirements

- **R-ADMIN-01-W1:** The Human Agent SHALL be able to submit the bootstrap credential + their display name + their contact channel. On success, the system materialises the System Bootstrap Template's adoption Auth Request with the submitted Human Agent as the approved slot's `system:genesis` fulfilment target, and grants `[allocate]` on `system:root` to the new Human Agent.
- **R-ADMIN-01-W2:** The submission SHALL be rejected if the bootstrap credential is invalid, expired, or has already been consumed, with error code `BOOTSTRAP_INVALID` / `BOOTSTRAP_EXPIRED` / `BOOTSTRAP_ALREADY_CONSUMED`.
- **R-ADMIN-01-W3:** The bootstrap credential SHALL be **single-use**. Once consumed, it cannot be reused even if the claim is later rolled back (recovery requires a new install or a manual admin override — both out of scope for this page).
- **R-ADMIN-01-W4:** Validation: the display name SHALL be non-empty; the contact channel SHALL be one of {Slack, email, web}; the channel handle SHALL be non-empty.

## 7. Permission / Visibility Rules

This is the one page in the system whose permission rules are **bootstrapped by the credential rather than by a grant**. There is no Agent to hold a grant yet.

- **Page access** — gated by **knowledge of the bootstrap credential**, not by a graph grant. This is the axiomatic entry point.
- **Claim action** — requires a valid, unused bootstrap credential. No grant check (there is no Agent to check).

After this page's write completes, every subsequent page's gates resolve to grants held by the newly-created platform admin Human Agent.

## 8. Event & Notification Requirements

- **R-ADMIN-01-N1:** On successful claim, the page SHALL emit the alerted audit event `PlatformAdminClaimed { human_agent_id, display_name, channel, claimed_at, bootstrap_credential_digest }` and show the event ID in a confirmation dialog before redirecting.
- **R-ADMIN-01-N2:** On failed claim, no audit event is emitted; the form redisplays with the error banner.
- **R-ADMIN-01-N3:** The installer's bootstrap credential generation event was itself audited at install time (outside the scope of this page); the `PlatformAdminClaimed` event SHALL reference the credential digest so the install event and the claim event can be correlated.

## 9. Backend Actions Triggered

Claim (W1) triggers:
- **Materialise** the hardcoded System Bootstrap Template adoption. Per [system/s01-bootstrap-template-adoption.md](../system/s01-bootstrap-template-adoption.md), this creates a regular Auth Request in `Approved` state with `requestor: system:genesis`, `approver: system:genesis`, `scope: [allocate]`, `resource: system:root`.
- **Create** the first Human Agent node for the claimant with the supplied display name and channel.
- **Create** the Human Agent's `inbox_object` and `outbox_object` composite instances and add them to the bootstrap adoption's downstream catalogue.
- **Issue** a Grant with `[allocate]` on `system:root` to the new Human Agent, with provenance = the bootstrap adoption Auth Request.
- **Emit** audit event `PlatformAdminClaimed` (alerted).
- **Invalidate** the bootstrap credential.

## 10. API Contract Sketch

```
GET  /api/v0/bootstrap/status
     → 200: { claimed: false, awaiting_credential: true }  OR
             { claimed: true, admin_agent_id: "..." }

POST /api/v0/bootstrap/claim
     Body: {
       bootstrap_credential: String,
       display_name: String,
       channel: { kind: "slack"|"email"|"web", handle: String }
     }
     → 201: {
       human_agent_id,
       inbox_id,
       outbox_id,
       grant_id,           # the [allocate] on system:root grant
       bootstrap_auth_request_id,
       audit_event_id
     }
     → 400: Validation errors (empty fields, invalid channel)
     → 403: BOOTSTRAP_INVALID | BOOTSTRAP_EXPIRED | BOOTSTRAP_ALREADY_CONSUMED
     → 409: A platform admin has already been claimed
```

## 11. Acceptance Scenarios

**Scenario 1 — first successful claim.**
*Given* a fresh install with a newly-generated bootstrap credential printed by the installer and no platform admin yet, *When* the Human Agent "Alex Chen" submits the credential with their Slack handle, *Then* the claim succeeds, a Human Agent node is created with `role: platform-admin`, a Grant with `[allocate]` on `system:root` is issued to them, the bootstrap credential is marked consumed, an alerted audit event `PlatformAdminClaimed` is emitted, and the page redirects to [02-platform-model-providers.md](02-platform-model-providers.md).

**Scenario 2 — reused credential.**
*Given* a bootstrap credential that was already consumed by a prior successful claim, *When* a different Human Agent attempts to submit the same credential, *Then* the claim is rejected with `BOOTSTRAP_ALREADY_CONSUMED`, no audit event is emitted, and the existing platform admin remains in place.

**Scenario 3 — second admin attempt after successful claim.**
*Given* a platform admin already exists, *When* any Human Agent navigates to this page, *Then* the page displays "A platform admin has already been assigned" and does not present the claim form. Creating a second platform admin is not a first-install operation — it requires a separate Auth Request flow provisioned by the existing admin (out of scope for this page).

## 12. Cross-References

**Concept files:**
- [concepts/system-agents.md § How System Agents Fit the Standard Organization Template](../../concepts/system-agents.md#how-system-agents-fit-the-standard-organization-template) — but note that System Agents are provisioned at org-creation time, not at platform bootstrap.
- [concepts/permissions/02 § System Bootstrap Template — Root of the Authority Tree](../../concepts/permissions/02-auth-request.md#system-bootstrap-template--root-of-the-authority-tree) — the axiomatic template this page adopts.
- [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain) — why every grant must trace back to the bootstrap event.
- [concepts/agent.md § Human Agent](../../concepts/human-agent.md) — the Agent kind the claimant becomes.

**phi-core types used:**
- None directly on this page. The Human Agent has no `AgentProfile` / `ModelConfig` / `ExecutionLimits` — those are LLM-Agent-only.

**Related admin pages:**
- [02-platform-model-providers.md](02-platform-model-providers.md) — the next page in the journey.

**Related system flows:**
- [system/s01-bootstrap-template-adoption.md](../system/s01-bootstrap-template-adoption.md) — what happens server-side when claim succeeds.

**Org layouts exercised in Acceptance Scenarios:**
- None — Phase 1 is pre-org. The acceptance scenarios are universal.
