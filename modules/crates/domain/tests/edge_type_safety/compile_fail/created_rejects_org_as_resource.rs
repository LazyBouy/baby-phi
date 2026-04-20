//! `OrgId` is a `Principal` only — organizations are not ownable by other
//! principals in the v0 ontology (they're at the top of the social
//! hierarchy). Passing an OrgId as the resource slot of `new_created`
//! must fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{AgentId, OrgId};

fn main() {
    let creator = AgentId::new();
    let resource = OrgId::new();
    let _ = Edge::new_created(&creator, &resource);
}
