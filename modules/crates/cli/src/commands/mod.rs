//! Subcommand implementations. Each module returns a `i32` exit code so
//! `main.rs` can pass it to `std::process::exit`.

pub mod agent;
pub mod bootstrap;
