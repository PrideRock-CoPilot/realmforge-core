//! # rfsource-governance
//!
//! Governance validation rules for `.rfsource` projects.
//!
//! Validates source artifacts against project standards including:
//! - PATH rules (naming conventions per language)
//! - SIZE rules (line count limits)
//! - HEADER rules (required file headers)
//! - DOC rules (documentation requirements for public items)
//! - RUST rules (safe Rust practices — no bare unwrap/unsafe)
//!
//! ## Crate Law
//!
//! - Pure validation logic — no file IO, no database
//! - Rules are data-defined (extensible without code changes)
//! - Validation is deterministic

use rfsource_core::CheckFinding;

/// Validate source content against governance rules.
///
/// Returns a list of findings. If any finding has `blocking: true`,
/// the artifact should not be committed.
pub fn run_checks(
    logical_path: &str,
    content: &str,
    object_ref: &str,
) -> Vec<CheckFinding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    // PATH-001: Logical path format (no special chars, reasonable length)
    if logical_path.contains("..") || logical_path.contains("//") {
        findings.push(CheckFinding {
            finding_id: format!("{}-PATH-001", object_ref),
            object_ref: object_ref.to_string(),
            logical_path: logical_path.to_string(),
            rule_id: "PATH-001".into(),
            severity: "error".into(),
            message: "Logical path must not contain '..' or '//'".into(),
            line: None,
            blocking: true,
        });
    }

    // PATH-002: Rust snake_case check
    if logical_path.ends_with(".rs") {
        let stem = logical_path
            .rsplit('/')
            .next()
            .unwrap_or(logical_path);
        let stem = stem.trim_end_matches(".rs");
        if stem.contains('-') || stem.contains(char::is_uppercase) {
            findings.push(CheckFinding {
                finding_id: format!("{}-PATH-002", object_ref),
                object_ref: object_ref.to_string(),
                logical_path: logical_path.to_string(),
                rule_id: "PATH-002".into(),
                severity: "warn".into(),
                message: format!(
                    "Rust file '{}' should use snake_case naming",
                    stem
                ),
                line: None,
                blocking: false,
            });
        }
    }

    // SIZE-001: Hard cap of 500 lines
    if line_count > 500 {
        findings.push(CheckFinding {
            finding_id: format!("{}-SIZE-001", object_ref),
            object_ref: object_ref.to_string(),
            logical_path: logical_path.to_string(),
            rule_id: "SIZE-001".into(),
            severity: "error".into(),
            message: format!(
                "File has {} lines, exceeds hard cap of 500",
                line_count
            ),
            line: None,
            blocking: true,
        });
    }

    // SIZE-002: Target of 300 lines
    if line_count > 300 && line_count <= 500 {
        findings.push(CheckFinding {
            finding_id: format!("{}-SIZE-002", object_ref),
            object_ref: object_ref.to_string(),
            logical_path: logical_path.to_string(),
            rule_id: "SIZE-002".into(),
            severity: "warn".into(),
            message: format!(
                "File has {} lines, exceeds target of 300",
                line_count
            ),
            line: None,
            blocking: false,
        });
    }

    // HEADER-001: Required file header (doc comment or copyright)
    let first_line = content.lines().next().unwrap_or("");
    let has_header = first_line.starts_with("//!") // Rust doc comment
        || first_line.starts_with("/*")
        || first_line.starts_with("# ") // Markdown heading
        || first_line.starts_with("---") // YAML frontmatter
        || first_line.starts_with("// Copyright")
        || first_line.starts_with("#!/"); // Shebang

    if !has_header {
        findings.push(CheckFinding {
            finding_id: format!("{}-HEADER-001", object_ref),
            object_ref: object_ref.to_string(),
            logical_path: logical_path.to_string(),
            rule_id: "HEADER-001".into(),
            severity: "warn".into(),
            message: "File should start with a descriptive header or doc comment".into(),
            line: None,
            blocking: false,
        });
    }

    // RUST-001: no bare unwrap() without invariant comment
    if logical_path.ends_with(".rs") {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(".unwrap();")
                || trimmed.ends_with(".unwrap()")
            {
                // Check previous line for invariant comment
                let prev_line = if i > 0 { lines[i - 1].trim() } else { "" };
                let has_invariant = prev_line.contains("// INVARIANT:")
                    || prev_line.contains("// invariant:");
                if !has_invariant {
                    findings.push(CheckFinding {
                        finding_id: format!("{}-RUST-001-{}", object_ref, i + 1),
                        object_ref: object_ref.to_string(),
                        logical_path: logical_path.to_string(),
                        rule_id: "RUST-001".into(),
                        severity: "warn".into(),
                        message: format!(
                            "Line {}: unwrap() without INVARIANT comment",
                            i + 1
                        ),
                        line: Some((i + 1) as u32),
                        blocking: false,
                    });
                }
            }
        }
    }

    // RUST-002: no unsafe without SAFETY comment
    if logical_path.ends_with(".rs") {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("unsafe ") || trimmed == "unsafe" {
                let prev_line = if i > 0 { lines[i - 1].trim() } else { "" };
                let has_safety = prev_line.contains("// SAFETY:");
                if !has_safety {
                    findings.push(CheckFinding {
                        finding_id: format!("{}-RUST-002-{}", object_ref, i + 1),
                        object_ref: object_ref.to_string(),
                        logical_path: logical_path.to_string(),
                        rule_id: "RUST-002".into(),
                        severity: "error".into(),
                        message: format!(
                            "Line {}: unsafe block without SAFETY comment",
                            i + 1
                        ),
                        line: Some((i + 1) as u32),
                        blocking: true,
                    });
                }
            }
        }
    }

    // DOC-001: public items need doc comments (basic check)
    if logical_path.ends_with(".rs") {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
            {
                let prev_line = if i > 0 { lines[i - 1].trim() } else { "" };
                if !prev_line.starts_with("///") && !prev_line.starts_with("//!") {
                    let item_name = trimmed
                        .split_whitespace()
                        .nth(2)
                        .unwrap_or("<unknown>");
                    findings.push(CheckFinding {
                        finding_id: format!("{}-DOC-001-{}", object_ref, i + 1),
                        object_ref: object_ref.to_string(),
                        logical_path: logical_path.to_string(),
                        rule_id: "DOC-001".into(),
                        severity: "warn".into(),
                        message: format!(
                            "Line {}: public item '{}' missing doc comment",
                            i + 1,
                            item_name
                        ),
                        line: Some((i + 1) as u32),
                        blocking: false,
                    });
                }
            }
        }
    }

    findings
}

/// Returns the complete list of rule IDs and descriptions.
pub fn rule_definitions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("PATH-001", "Logical path must not contain '..' or '//'"),
        ("PATH-002", "Rust files should use snake_case naming"),
        ("SIZE-001", "Hard line limit: 500 lines (blocking)"),
        ("SIZE-002", "Target line limit: 300 lines (warning)"),
        ("HEADER-001", "File should start with a descriptive header"),
        ("DOC-001", "Public items should have doc comments"),
        ("RUST-001", "unwrap() requires INVARIANT comment"),
        ("RUST-002", "unsafe requires SAFETY: comment"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file_passes() {
        let findings = run_checks("src/empty.rs", "", "obj_1");
        let blocking: Vec<_> = findings.iter().filter(|f| f.blocking).collect();
        assert!(blocking.is_empty(), "Empty file should have no blocking issues");
    }

    #[test]
    fn test_rust_file_without_header() {
        let findings = run_checks("src/main.rs", "fn main() {}", "obj_1");
        let header_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "HEADER-001")
            .collect();
        assert!(!header_findings.is_empty(), "Should flag missing header");
    }

    #[test]
    fn test_snake_case_check() {
        let findings = run_checks("src/MyFile.rs", "fn main() {}", "obj_1");
        let path_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "PATH-002")
            .collect();
        assert!(!path_findings.is_empty(), "Should flag non-snake-case name");
    }

    #[test]
    fn test_path_traversal_check() {
        let findings = run_checks("src/../config.rs", "fn main() {}", "obj_1");
        let traversal: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "PATH-001")
            .collect();
        assert!(!traversal.is_empty(), "Should flag path traversal");
    }

    #[test]
    fn test_size_cap() {
        let long_content = "x\n".repeat(501);
        let findings = run_checks("src/big.rs", &long_content, "obj_1");
        let size_finding: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "SIZE-001")
            .collect();
        assert!(!size_finding.is_empty(), "Should flag oversized file");
    }
}
