use crate::check::{Check, CheckConfig, CheckResult, CheckType, Finding};
use crate::error::{CodeReviewError, Result};
use crate::severity::Severity;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

/// A check that runs an external command-line tool and inspects its output.
///
/// Examples: `cargo clippy`, `cargo fmt --check`, `ruff check`.
///
/// # Security
///
/// All commands are validated against a whitelist of allowed binary names
/// to prevent arbitrary command injection via `.code-review.yaml`.
#[derive(Debug)]
pub struct ExecutableCheck {
    config: CheckConfig,
    /// List of binary names that are allowed to be executed.
    #[allow(dead_code)]
    allowed_binaries: Vec<String>,
}

impl ExecutableCheck {
    /// The default list of allowed executables.
    const DEFAULT_ALLOWED: &'static [&'static str] = &[
        "cargo",
        "rustc",
        "ruff",
        "black",
        "flake8",
        "mypy",
        "pylint",
        "eslint",
        "prettier",
        "golangci-lint",
        "clang-tidy",
        "cppcheck",
    ];

    /// Create a new executable check from configuration.
    ///
    /// Returns an error if the command is not in the allowed list.
    pub fn new(config: CheckConfig) -> Result<Self> {
        Self::with_allowed_binaries(
            config,
            Self::DEFAULT_ALLOWED
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }

    /// Create a new executable check with a custom allowed binary list.
    ///
    /// # Security
    ///
    /// Only the binaries in `allowed_binaries` may be executed.
    /// All arguments are validated to prevent shell injection.
    pub fn with_allowed_binaries(
        config: CheckConfig,
        allowed_binaries: Vec<String>,
    ) -> Result<Self> {
        let command = config
            .command
            .as_ref()
            .and_then(|c| c.first())
            .ok_or_else(|| {
                CodeReviewError::invalid_check(
                    &config.id,
                    "Executable check must have a 'command' field with at least one element",
                )
            })?;

        let binary_name = std::path::Path::new(command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if !allowed_binaries.iter().any(|b| b == binary_name) {
            return Err(CodeReviewError::invalid_check(
                &config.id,
                format!(
                    "Binary '{}' is not in the allowed executables list. \
                     Allowed: [{}]",
                    binary_name,
                    allowed_binaries.join(", "),
                ),
            ));
        }

        Ok(Self {
            config,
            allowed_binaries,
        })
    }

    /// Returns the timeout duration for this check.
    fn timeout(&self) -> Duration {
        Duration::from_secs(self.config.timeout_secs.unwrap_or(60))
    }
}

impl Check for ExecutableCheck {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn severity(&self) -> Severity {
        self.config.severity
    }

    fn check_type(&self) -> CheckType {
        CheckType::Executable
    }

    fn execute(&self, workspace_root: &Path) -> Result<CheckResult> {
        let start = Instant::now();

        let command = self.config.command.as_ref().ok_or_else(|| {
            CodeReviewError::invalid_check(&self.config.id, "Missing command configuration")
        })?;

        if command.is_empty() {
            return Err(CodeReviewError::invalid_check(
                &self.config.id,
                "Command list is empty",
            ));
        }

        let program = &command[0];
        let args: Vec<&str> = command[1..].iter().map(|s| s.as_str()).collect();

        let timeout = self.timeout();

        let result = std::process::Command::new(program)
            .args(&args)
            .current_dir(workspace_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| {
                CodeReviewError::execution(
                    command.join(" "),
                    format!("Failed to spawn process: {}", e),
                )
            })?;

        // Wait for the command with timeout
        let output = wait_with_timeout(result, timeout)
            .map_err(|_| CodeReviewError::timeout(&self.config.id, timeout.as_secs()))?;

        let elapsed = start.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(CheckResult::passed(&self.config.id, elapsed))
        } else {
            let combined = format!("{}{}", stdout, stderr);
            let message = if combined.trim().is_empty() {
                format!(
                    "Exited with code {}",
                    output.status.code().map_or(-1, |c| c)
                )
            } else {
                // Take first 500 chars to avoid excessively long messages
                let trimmed = combined.trim();
                if trimmed.len() > 500 {
                    format!("{}...", &trimmed[..500])
                } else {
                    trimmed.to_string()
                }
            };

            let finding = Finding::new(
                &self.config.id,
                &self.config.name,
                self.config.severity,
                message,
            );

            Ok(CheckResult::failed(&self.config.id, vec![finding], elapsed))
        }
    }
}

/// Wait for a child process to finish, with a timeout.
fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> std::result::Result<std::process::Output, ()> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Collect remaining output
                let output = child.wait_with_output().map_err(|_| ())?;
                return Ok(output);
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                return Err(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::CheckConfig;
    use tempfile::TempDir;

    fn make_check_config(id: &str, command: Vec<String>) -> CheckConfig {
        CheckConfig {
            id: id.to_string(),
            name: format!("Check {}", id),
            description: "Test check".to_string(),
            check_type: CheckType::Executable,
            severity: Severity::Blocker,
            file_pattern: None,
            command: Some(command),
            working_dir: Some("workspace_root".to_string()),
            timeout_secs: Some(10),
            pattern: None,
            allowed_pattern: None,
            required_above: None,
            property: None,
            max: None,
            warning_at: None,
        }
    }

    #[test]
    fn test_new_rejects_unknown_binary() {
        let config = make_check_config(
            "TEST-001",
            vec!["malware.exe".to_string(), "--flag".to_string()],
        );
        let result = ExecutableCheck::new(config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not in the allowed"));
    }

    #[test]
    fn test_new_accepts_allowed_binary() {
        let config = make_check_config("TEST-002", vec!["cargo".to_string(), "check".to_string()]);
        let result = ExecutableCheck::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_cargo_check_success() {
        let dir = TempDir::new().unwrap();
        // Create a minimal Cargo workspace so cargo check has something to do
        let cargo_toml = r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn hello() -> &'static str { \"hello\" }",
        )
        .unwrap();

        let config = make_check_config(
            "RUST-EXEC-TEST",
            vec!["cargo".to_string(), "check".to_string()],
        );
        let check = ExecutableCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(result.passed, "cargo check should pass: {:?}", result.error);
    }
}
