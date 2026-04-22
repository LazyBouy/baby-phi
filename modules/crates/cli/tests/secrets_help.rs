//! Smoke tests for the `phi secret …` clap surface.
//!
//! Drives the real binary via `std::process::Command` — rejects any
//! typo that breaks `--help` parsing, without exercising the HTTP path
//! (covered by `server/tests/acceptance_secrets.rs`).
//!
//! The binary path is discovered via `CARGO_BIN_EXE_phi`, which
//! Cargo sets automatically for integration tests against the `cli`
//! package's `[[bin]]`.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_phi")
}

fn run_help(args: &[&str]) -> String {
    let out = Command::new(bin()).args(args).output().expect("run phi");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "help exit was non-zero: stdout=`{stdout}` stderr=`{stderr}`"
    );
    stdout
}

#[test]
fn secret_group_lists_five_subcommands() {
    let help = run_help(&["secret", "--help"]);
    for expected in ["list", "add", "rotate", "reveal", "reassign"] {
        assert!(
            help.contains(expected),
            "`phi secret --help` must mention `{expected}` subcommand; got:\n{help}"
        );
    }
}

#[test]
fn secret_add_requires_slug_and_material_file() {
    let help = run_help(&["secret", "add", "--help"]);
    assert!(help.contains("--slug"));
    assert!(help.contains("--material-file"));
}

#[test]
fn secret_reveal_requires_slug_and_purpose_and_accept_audit() {
    let help = run_help(&["secret", "reveal", "--help"]);
    assert!(help.contains("--slug"));
    assert!(help.contains("--purpose"));
    assert!(help.contains("--accept-audit"));
}

#[test]
fn top_level_help_lists_secret_subcommand() {
    let help = run_help(&["--help"]);
    assert!(
        help.contains("secret"),
        "top-level `--help` must list the secret subcommand; got:\n{help}"
    );
}
