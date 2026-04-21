//! Smoke tests for the `baby-phi model-provider …` clap surface.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_baby-phi")
}

fn run_help(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("run baby-phi");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "help exit was non-zero: stdout=`{stdout}` stderr=`{stderr}`"
    );
    stdout
}

#[test]
fn model_provider_group_lists_four_subcommands() {
    let help = run_help(&["model-provider", "--help"]);
    for expected in ["list", "add", "archive", "list-kinds"] {
        assert!(
            help.contains(expected),
            "`baby-phi model-provider --help` must mention `{expected}`; got:\n{help}"
        );
    }
}

#[test]
fn model_provider_add_requires_config_file_and_secret_ref() {
    let help = run_help(&["model-provider", "add", "--help"]);
    assert!(help.contains("--config-file"));
    assert!(help.contains("--secret-ref"));
    assert!(help.contains("--tenants-allowed"));
}

#[test]
fn model_provider_archive_requires_id() {
    let help = run_help(&["model-provider", "archive", "--help"]);
    assert!(help.contains("--id"));
}

#[test]
fn top_level_help_lists_model_provider() {
    let help = run_help(&["--help"]);
    assert!(help.contains("model-provider"));
}
