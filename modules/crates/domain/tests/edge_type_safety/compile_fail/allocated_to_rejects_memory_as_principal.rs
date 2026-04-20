//! `MemoryId` is a `Resource`, not a `Principal`. Passing one in either
//! slot of `new_allocated_to` must fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{MemoryId, OrgId};

fn main() {
    let from = OrgId::new();
    let to = MemoryId::new();
    let _ = Edge::new_allocated_to(&from, &to);
}
