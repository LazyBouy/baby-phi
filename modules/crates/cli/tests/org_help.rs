//! Smoke tests for the `baby-phi org` subcommand family.
//!
//! Shape assertions only — help renders + `--id` is required on
//! `dashboard`/`show`; `--from-layout` surfaces on `create`. These
//! catch accidental arg-surface regressions (a renamed flag or a
//! dropped subcommand) during CLI refactors.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_baby-phi")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("run baby-phi");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn org_help_lists_all_four_subcommands() {
    let (ok, stdout, _) = run(&["org", "--help"]);
    assert!(ok);
    for sub in ["create", "list", "show", "dashboard"] {
        assert!(
            stdout.contains(sub),
            "`baby-phi org --help` must mention `{sub}`; got:\n{stdout}"
        );
    }
}

#[test]
fn org_dashboard_help_requires_id_flag() {
    let (ok, stdout, _) = run(&["org", "dashboard", "--help"]);
    assert!(ok);
    assert!(
        stdout.contains("--id"),
        "`org dashboard --help` must surface `--id`; got:\n{stdout}"
    );
    assert!(
        stdout.contains("--json"),
        "`org dashboard --help` must surface `--json`; got:\n{stdout}"
    );
}

#[test]
fn org_create_help_surfaces_from_layout_flag() {
    let (ok, stdout, _) = run(&["org", "create", "--help"]);
    assert!(ok);
    assert!(
        stdout.contains("--from-layout"),
        "`org create --help` must surface `--from-layout`; got:\n{stdout}"
    );
}
