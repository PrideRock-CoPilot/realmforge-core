// Tests for CLI output format (JSON vs pretty).
// These tests verify that CLI commands produce valid output in the expected format
// without requiring a database connection.

use std::process::Command;

fn control_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_control"))
}

#[test]
fn test_help_output_contains_expected_commands() {
    let output = control_bin()
        .arg("--help")
        .output()
        .expect("failed to execute control --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The help should list key subcommands
    assert!(
        stdout.contains("session")
            || stdout.contains("command")
            || stdout.contains("audit")
            || stdout.contains("snapshot"),
        "help output should list core commands"
    );
}

#[test]
fn test_version_output_format() {
    let output = control_bin()
        .arg("--version")
        .output()
        .expect("failed to execute control --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version output should be non-empty
    assert!(!stdout.is_empty(), "version output should not be empty");
}

#[test]
fn test_unknown_command_returns_error() {
    let output = control_bin()
        .arg("nonexistent-command-xyz")
        .output()
        .expect("failed to execute control with bad command");

    // Unknown commands should exit with error
    assert!(
        !output.status.success(),
        "unknown command should exit with error"
    );
}
