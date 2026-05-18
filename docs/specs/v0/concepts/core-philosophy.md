<!-- Status: BINDING SPEC SOURCE (promoted 2026-04-28 from `plan/core-philosophy-check/core-philosophy.md` per post-CH-21 audit; see `plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` for the audit that ratified the promotion). -->
<!-- Last verified: 2026-05-17 by Claude Code (CH-26 P-DOCS — claim at line 16 *"Organization has Projects (A Resource Type)"* + lines 28-29 *"Organizations own Resources"* + *"Projects own Resources"* are now honored at the **composite-resource axis** (via NEW `Composite::OrganizationObject` + `Composite::ProjectObject` variants — cardinality 8 → 10) AND at the **instance-identity-tag axis** (via NEW `tags: Vec<String>` field on Organization + Project structs populated with `organization:<uuid>` / `project:<uuid>` at-creation-time + backfill migration #0018) AND at the **engine-routing axis** (via ≥ 15 `check_permission` invocations across 7 admin-page-1/3 handlers — consumed advisorily at CH-26 close; CH-27 tightens to blocking gates per drift `D-CH26-FOLLOWUP-01`). See ADR-0061 + `m5_3/architecture/composite-resources-model.md`. Drift `D-philosophy-02` transitions `discovered → remediated` at the load-bearing semantic axis at this chunk. Cycle hex `d1cb9e1f`.) -->
<!-- Last verified: 2026-05-16 by Claude Code (CH-25 P3 — claims at lines 9, 23, 24, 31, 32 (*"Agent owns Organization"*, *"Agent create Projects"*, *"Agents own Resources"*, *"Every Resource must have a creator"*, *"Every Resource ownership must be tracked to the creator - Provenance"*) are now honored for the Org/Project axis: NEW `Edge::Owns` variant + existing `Edge::Created` emission at `apply_org_creation` + `apply_project_creation`; owner-grant synth at `step_2_resolve_grants`. See ADR-0060 + `m5_3/architecture/agent-ownership-model.md`. Drift D-philosophy-01 transitions discovered → remediated at this chunk. D-philosophy-02 (unified-resource-model gap) remains discovered — CH-26 target. Cycle hex `1e01618e`.) -->
<!-- Last verified: 2026-04-28 by Claude Code (initial promotion; D-philosophy-01 + D-philosophy-02 (under `m5_3/drifts/`) file the two HIGH drifts the audit surfaced; M5.3 carve-out per `plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §2.5 closes them at CH-25 + CH-26.) -->

# Baby-Phi Core Philosophy

* Two Types of Agents
  * Human
  * LLM
* Agent owns Organization
* Organization has Resources
* Two Types of Resources
  * Fundamental
  * Composite (created by combining Fundamental Resources)
  * Resources have defined actions that can be performed on them
* Organization has Projects (A Resource Type)
* Organizations have Sub-Organization
* Projects have Sub-projects
* A Project be shared between Organizations
* A Resource be shared between Projects
* A Resource be shared between Organization through shared Project
* Organization has Agents (Ownership)
* Agent spawn other Agents (Ownership)
* Agent create Projects
* Agents own Resources
* Agents work on several Projects
* Agents work on several Organization
* Organizations own Resources
* Projects own Resources
* Resources can be Transferred
* Resources can be co-owned
* Every Resource must have a creator
* Every Resource ownership must be tracked to the creator - Provenance
* A Permission (Grant) is a record of Capability -
  * A Capability is an action to be performed on a resource by a subject under some constraints when the permission is granted through a valid provenance
  * A Permission is Tuple of
    * Subject - Who owns
    * Action - which capability
    * Resource
    * Constraints - conditions of capability
    * Provenance - Who granted this and how
* Session have shared ownership, depending on
  * Organizations under which it is generated
  * Projects under which it is generated
  * Agent who generated it
* Memory have shared ownership, depending on
  * who generated it
    * Agent on behalf Organization / Project (then inherited from Session)
    * Agent (then self - private)
