//! `UserId` is a `Principal` only — Users are not ownable entities in the
//! v0 ontology. Passing a UserId as the resource slot of `new_owned_by`
//! must fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{OrgId, UserId};

fn main() {
    let resource = UserId::new();
    let principal = OrgId::new();
    let _ = Edge::new_owned_by(&resource, &principal);
}
