// Integration tests for operator-cli commands.
// Tests gracefully skip when no database is available.

use std::process::Command;

/// Helper: check if PostgreSQL is reachable via a quick TCP probe.
#[allow(dead_code)]
fn db_available() -> bool {
    let uri = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/realmforge".to_string());

    let host_port = uri
        .trim_start_matches("postgres://")
        .trim_start_matches("postgresql://")
        .split('@')
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or("localhost:5432");

    let port = host_port
        .split(':')
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5432);

    let addr = format!("127.0.0.1:{}", port);
    std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_secs(2))
        .is_ok()
}

fn control_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_control"))
}

#[test]
fn test_cli_help() {
    let output = control_bin()
        .arg("--help")
        .output()
        .expect("failed to execute control --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("Commands"));
}

#[test]
fn test_cli_version() {
    let output = control_bin()
        .arg("--version")
        .output()
        .expect("failed to execute control --version");

    assert!(output.status.success());
}

#[test]
fn test_cli_health_or_help() {
    let output = control_bin()
        .arg("--help")
        .output()
        .expect("failed to execute control --help");

    assert!(output.status.success());
}

#[test]
fn test_cli_session_help() {
    let output = control_bin()
        .args(["session", "--help"])
        .output()
        .expect("failed to execute control session --help");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check either stdout or stderr (clap may use either)
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        output.status.success() || combined.contains("Usage"),
        "session command should show help or error: {}",
        combined
    );
}
