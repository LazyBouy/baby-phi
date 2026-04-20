//! `GrantId` is a `Resource` (grants can be referenced as targets) but NOT
//! a `Principal`. Passing a GrantId as the creator in `new_created` must
//! fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{GrantId, MemoryId};

fn main() {
    let creator = GrantId::new();
    let resource = MemoryId::new();
    let _ = Edge::new_created(&creator, &resource);
}
