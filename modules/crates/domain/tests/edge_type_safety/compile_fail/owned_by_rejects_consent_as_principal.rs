//! `ConsentId` is not a `Principal` — passing it as the principal slot of
//! `Edge::new_owned_by` must fail to compile.

use domain::model::edges::Edge;
use domain::model::ids::{ConsentId, MemoryId};

fn main() {
    let resource = MemoryId::new();
    let principal = ConsentId::new();
    let _ = Edge::new_owned_by(&resource, &principal);
}
