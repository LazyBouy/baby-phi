//! `phi login` subcommand — M2/P1 scaffold.
//!
//! The real login flow lands in M2/P4 when the credentials-vault
//! vertical is stood up. For M2/P1 this subcommand only validates that
//! a saved session exists (or doesn't) and points operators at the
//! still-working `phi bootstrap claim` path for first-time login.
//!
//! Why ship it now?
//!   1. Reserves the `login` subcommand name so tests and docs can
//!      reference it starting at P4.
//!   2. Wires [`session_store`](crate::session_store) into the main
//!      binary so its test coverage runs on every `cargo test`.
//!   3. Gives shell-scripted operators a deterministic pre-check
//!      (`phi login status`) to run before other M2 subcommands.

use crate::exit::{EXIT_OK, EXIT_PRECONDITION_FAILED};
use crate::session_store;

/// Subcommand parameter surface. Mirrors the eventual P4 shape so no
/// argument renames are needed when the real flow lands.
#[derive(Debug, clap::Subcommand)]
pub enum LoginCommand {
    /// Report whether a saved session exists (without contacting the server).
    Status,
    /// Clear the saved session (local only — does not revoke the
    /// server-side token; that lands in M3 with OAuth).
    Logout,
}

pub async fn run(cmd: LoginCommand) -> i32 {
    match cmd {
        LoginCommand::Status => status(),
        LoginCommand::Logout => logout(),
    }
}

fn status() -> i32 {
    let path = match session_store::default_session_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("phi login status: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    match session_store::load(&path) {
        Ok(s) => {
            println!(
                "logged in as {} (cookie issued {}), session file: {}",
                s.agent_id,
                s.issued_at,
                path.display()
            );
            EXIT_OK
        }
        Err(session_store::SessionStoreError::NotFound { .. }) => {
            println!(
                "no saved session at {}. First-time login: run \
                 `phi bootstrap claim --credential <…>`. \
                 (M2/P4 will add credential-based re-login.)",
                path.display()
            );
            EXIT_PRECONDITION_FAILED
        }
        Err(e) => {
            eprintln!("phi login status: {e}");
            EXIT_PRECONDITION_FAILED
        }
    }
}

fn logout() -> i32 {
    let path = match session_store::default_session_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("phi login logout: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    match session_store::clear(&path) {
        Ok(()) => {
            println!("cleared saved session at {}", path.display());
            EXIT_OK
        }
        Err(e) => {
            eprintln!("phi login logout: {e}");
            EXIT_PRECONDITION_FAILED
        }
    }
}
