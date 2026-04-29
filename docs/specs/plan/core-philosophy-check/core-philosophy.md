<!-- Source-of-truth philosophy brief (user-supplied 2026-04-28). The audit at `2026-04-28-philosophy-alignment-audit.md` checks the implementation against this document. -->

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
