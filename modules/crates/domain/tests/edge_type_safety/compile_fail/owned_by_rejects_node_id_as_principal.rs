//! The generic `NodeId` is a `Resource` catch-all but NOT a `Principal`.
//! Passing a `NodeId` as the principal must fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{MemoryId, NodeId};

fn main() {
    let resource = MemoryId::new();
    let principal = NodeId::new();
    let _ = Edge::new_owned_by(&resource, &principal);
}
