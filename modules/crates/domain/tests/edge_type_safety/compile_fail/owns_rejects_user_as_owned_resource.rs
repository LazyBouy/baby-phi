//! CH-25 / ADR-0060 §D60.1 — the `Edge::Owns` variant's resource-end
//! is typed as the closed-set [`OwnedResourceId`] enum (`Org(OrgId) |
//! Project(ProjectId)`). Passing a `UserId` to the `Org` variant slot
//! (or any non-OrgId/ProjectId type to the constructor) must fail to
//! compile — `UserId` is a Principal-only ID in the v0 ontology, not
//! an ownable Org/Project.
//!
//! This fixture proves the typed-payload safety of [`Edge::new_owns`]:
//! the enum's `Org(OrgId)` arm rejects a `UserId` payload at compile
//! time so callers cannot accidentally erase the resource-end type.

use domain::model::edges::Edge;
use domain::model::ids::{AgentId, OwnedResourceId, UserId};

fn main() {
    let agent = AgentId::new();
    let user = UserId::new();
    // OwnedResourceId::Org expects OrgId, NOT UserId — type-checker
    // rejects this construction at the call site.
    let owned = OwnedResourceId::Org(user);
    let _ = Edge::new_owns(&agent, owned);
}
