//! Platform-admin project surfaces — pages 10 (creation) + 11
//! (detail) land here at M4/P6 + M4/P7.
//!
//! M4/P3 scope: the two repo-backed resolvers the Template A
//! fire-listener needs so `AppState::new` can wire the bus once the
//! listener is constructed. Page handlers themselves land later.

pub mod resolvers;

pub use resolvers::{RepoActorResolver, RepoAdoptionArResolver};
