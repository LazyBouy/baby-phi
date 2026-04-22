//! `phi completion <shell>` — generate shell-completion scripts.
//!
//! Delegates entirely to `clap_complete::generate`; phi adds no
//! logic of its own. The generated scripts are written to stdout so
//! operators pipe them into `.bashrc` / `.zshrc` / the appropriate
//! completion dir.
//!
//! Usage:
//!   phi completion bash   > /usr/share/bash-completion/completions/phi
//!   phi completion zsh    > "${fpath[1]}/_phi"
//!   phi completion fish   > ~/.config/fish/completions/phi.fish
//!   phi completion powershell > phi.ps1
//!
//! This subcommand is intentionally offline — it never opens a
//! session cookie or calls the server. That keeps `EXIT_OK` achievable
//! in fresh shells that haven't run `bootstrap claim` yet, so shell
//! init scripts (which typically run completion generation at install
//! time) stay idempotent across reinstalls.

use std::io::{self, Write};

use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell};

use crate::exit::{EXIT_INTERNAL, EXIT_OK};

/// Render completions for the supplied shell. The top-level `Cli`
/// struct is the authoritative clap tree — `CommandFactory` walks it
/// and `clap_complete::generate` emits shell-specific output.
pub fn run(shell: Shell) -> i32 {
    let mut cmd: Command = crate::Cli::command();
    let bin_name = cmd.get_name().to_string();
    let mut out = io::stdout().lock();
    generate(shell, &mut cmd, bin_name, &mut out);
    if let Err(e) = out.flush() {
        eprintln!("phi: failed to flush completion output: {e}");
        return EXIT_INTERNAL;
    }
    EXIT_OK
}
