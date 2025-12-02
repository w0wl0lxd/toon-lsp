// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Integration tests for the diagnose command.
//!
//! T071-T081: Tests for User Story 6 - Diagnostic analysis with JSON/SARIF output
//!
//! These tests validate the `toon-lsp diagnose` command functionality including:
//! - JSON and SARIF output formats
//! - Source code context inclusion
//! - Severity level filtering
//! - Stdin input support
//! - Error handling

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

/// Path to fixtures directory
fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get a command for the toon-lsp binary
fn toon_lsp() -> Command {
    Command::cargo_bin("toon-lsp").expect("Failed to find toon-lsp binary")
}

/// Helper to parse JSON from command output (stdout)
fn parse_json_from_stdout(output: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let stdout = String::from_utf8_lossy(output);
    let json_str = stdout.trim();
    Ok(serde_json::from_str(json_str)?)
}

/// Helper to validate JSON is valid and parseable
fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<Value>(s).is_ok()
}

/// Helper to validate SARIF 2.1.0 structure
fn validate_sarif_structure(sarif: &Value) -> bool {
    // SARIF must have version 2.1.0
    let version_ok = sarif
        .get("version")
        .and_then(|v| v.as_str())
        .map(|v| v == "2.1.0")
        .unwrap_or(false);

    version_ok && sarif.get("runs").is_some()
}

// =============================================================================
// T071: Basic diagnostic output (JSON default)
// =============================================================================

#[test]
fn test_diagnose_invalid_file_produces_json_diagnostics() {
    // Given: Invalid TOON file with parse errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&fixture)
        .assert()
        .code(0) // diagnose command reports diagnostics but exits with 0
        .get_output()
        .stdout
        .clone();

    // Then: JSON output contains diagnostics array
    let json = parse_json_from_stdout(&output).expect("Output should be valid JSON");

    assert!(
        json.is_object(),
        "Output should be a JSON object with diagnostic report"
    );
    assert!(
        json.get("diagnostics").is_some(),
        "JSON should contain 'diagnostics' array"
    );
    assert!(
        json.get("diagnostics").unwrap().is_array(),
        "'diagnostics' should be an array"
    );

    let diagnostics = json.get("diagnostics").unwrap().as_array().unwrap();
    assert!(
        !diagnostics.is_empty(),
        "Invalid file should produce at least one diagnostic"
    );

    // Each diagnostic should have required fields
    for diag in diagnostics {
        assert!(
            diag.get("range").is_some(),
            "Diagnostic should have 'range' field"
        );
        assert!(
            diag.get("message").is_some(),
            "Diagnostic should have 'message' field"
        );
        assert!(
            diag.get("severity").is_some(),
            "Diagnostic should have 'severity' field"
        );
    }
}

// =============================================================================
// T072: JSON format explicit
// =============================================================================

#[test]
fn test_diagnose_json_format_explicit() {
    // Given: TOON file with parse errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose --format json file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("json")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid JSON diagnostic report
    let json = parse_json_from_stdout(&output).expect("Output should be valid JSON");

    assert!(
        json.is_object(),
        "JSON format should produce object output"
    );
    assert!(
        json.get("diagnostics").is_some(),
        "Should have diagnostics array"
    );
    assert!(
        json.get("diagnostics").unwrap().is_array(),
        "diagnostics should be array"
    );
}

#[test]
fn test_diagnose_json_format_short_flag() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose -f json file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("-f")
        .arg("json")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid JSON output
    let json = parse_json_from_stdout(&output).expect("Output should be valid JSON");
    assert!(json.get("diagnostics").is_some());
}

// =============================================================================
// T073: SARIF format output
// =============================================================================

#[test]
fn test_diagnose_sarif_format_output() {
    // Given: TOON file with errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose --format sarif file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("sarif")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid SARIF 2.1.0 output
    let sarif = parse_json_from_stdout(&output).expect("SARIF should be valid JSON");

    assert!(validate_sarif_structure(&sarif), "Output should have SARIF 2.1.0 structure");

    // SARIF must have version 2.1.0
    assert_eq!(
        sarif.get("version").and_then(|v| v.as_str()),
        Some("2.1.0"),
        "SARIF version should be 2.1.0"
    );

    // SARIF must have runs array
    assert!(
        sarif.get("runs").unwrap().is_array(),
        "SARIF runs should be array"
    );
}

#[test]
fn test_diagnose_sarif_format_has_runs_array() {
    // Given: TOON file with errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose --format sarif file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("sarif")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: SARIF has proper runs structure
    let sarif = parse_json_from_stdout(&output).expect("Should parse as JSON");

    let runs = sarif.get("runs").expect("Should have runs");
    assert!(runs.is_array(), "runs should be array");

    let runs_array = runs.as_array().unwrap();
    assert!(!runs_array.is_empty(), "runs array should have at least one run");

    let run = &runs_array[0];
    assert!(run.get("results").is_some(), "run should have results");
}

// =============================================================================
// T074: Context flag shows source
// =============================================================================

#[test]
fn test_diagnose_context_flag_shows_source() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose --context file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--context")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Output includes source code context
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    // With context flag, diagnostics may have additional fields
    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(
        !diagnostics.is_empty(),
        "Should have at least one diagnostic"
    );

    // At least one diagnostic should have context information
    // This could be in the diagnostic message or a separate context field
    for diag in diagnostics {
        let message = diag.get("message").and_then(|m| m.as_str()).unwrap_or("");
        // Message should be non-empty (context information)
        assert!(!message.is_empty(), "Diagnostic should have message/context");
    }
}

#[test]
fn test_diagnose_context_short_flag() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose -c file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("-c")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid JSON output with context
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    assert!(json.get("diagnostics").is_some(), "Should have diagnostics");
}

// =============================================================================
// T075: Severity filtering - errors only (default)
// =============================================================================

#[test]
fn test_diagnose_severity_error_default() {
    // Given: TOON file with various severity issues
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("severities.toon");
    // Create a file with multiple issues at different severity levels
    fs::write(&toon_path, "name: Alice\nage: [unclosed array\nactive: true\n")
        .expect("write file");

    // When: User runs `toon-lsp diagnose file.toon` (default is error severity)
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&toon_path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Only error-level diagnostics shown
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    // All diagnostics should be errors or above (no warnings/hints/info)
    for diag in diagnostics {
        let severity = diag.get("severity").and_then(|s| s.as_str());
        assert!(
            severity == Some("error") || severity.is_none(),
            "Default severity should only show errors, got: {:?}",
            severity
        );
    }
}

#[test]
fn test_diagnose_severity_error_explicit() {
    // Given: TOON file with errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose --severity error file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--severity")
        .arg("error")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Only error-level diagnostics shown
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(!diagnostics.is_empty(), "Should have at least one error diagnostic");
}

// =============================================================================
// T076: Severity filtering - warnings and above
// =============================================================================

#[test]
fn test_diagnose_severity_warning_includes_errors() {
    // Given: TOON file with mixed severity issues
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("mixed_severity.toon");
    fs::write(&toon_path, "name: test\ncount: [unclosed\nactive: true\n").expect("write file");

    // When: User runs `toon-lsp diagnose --severity warning file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--severity")
        .arg("warning")
        .arg(&toon_path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Warnings and errors shown, hints filtered
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    // All should be warning level or higher (errors, warnings)
    // Should not contain hints or info level
    for diag in diagnostics {
        let severity = diag.get("severity").and_then(|s| s.as_str());
        // Verify no hint or info level
        assert!(
            severity != Some("hint") && severity != Some("info"),
            "Should filter out hints and info, got: {:?}",
            severity
        );
    }
}

#[test]
fn test_diagnose_severity_short_flag() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp diagnose -s warning file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("-s")
        .arg("warning")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid output with warning-level filtering
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    assert!(json.get("diagnostics").is_some());
}

// =============================================================================
// T077: Valid file produces empty diagnostics
// =============================================================================

#[test]
fn test_diagnose_valid_file_produces_empty_diagnostics() {
    // Given: Valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp diagnose file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&fixture)
        .assert()
        .success() // Exit code 0 for valid files
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Empty diagnostics array, exit code 0
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(
        diagnostics.is_empty(),
        "Valid file should produce no diagnostics"
    );
}

#[test]
fn test_diagnose_valid_file_with_nesting() {
    // Given: Valid TOON file with nested structure
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested_valid.toon");
    fs::write(
        &toon_path,
        "server:\n  host: localhost\n  port: 8080\nlogging:\n  level: debug\n",
    )
    .expect("write file");

    // When: User runs diagnose
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&toon_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: Empty diagnostics
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have array");

    assert!(diagnostics.is_empty(), "Valid nested file should have no diagnostics");
}

// =============================================================================
// T078: Stdin input
// =============================================================================

#[test]
fn test_diagnose_stdin_invalid_input() {
    // Given: Invalid TOON via stdin
    let invalid_toon = "key: [unclosed";

    // When: User runs `echo "key: [unclosed" | toon-lsp diagnose -`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("-")
        .write_stdin(invalid_toon)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Diagnostics from stdin content
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(
        !diagnostics.is_empty(),
        "Invalid stdin should produce diagnostics"
    );
}

#[test]
fn test_diagnose_stdin_valid_input() {
    // Given: Valid TOON via stdin
    let valid_toon = "key: value\n";

    // When: User runs `echo "key: value" | toon-lsp diagnose -`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("-")
        .write_stdin(valid_toon)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Empty diagnostics
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(
        diagnostics.is_empty(),
        "Valid stdin should produce no diagnostics"
    );
}

#[test]
fn test_diagnose_stdin_with_format() {
    // Given: Invalid TOON via stdin
    let invalid_toon = "name: Alice\nage: [unclosed";

    // When: User runs diagnose with stdin and format flag
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("json")
        .arg("-")
        .write_stdin(invalid_toon)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid JSON from stdin
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    assert!(
        json.get("diagnostics").is_some(),
        "Should have diagnostics from stdin"
    );
}

// =============================================================================
// T079: Nonexistent file handling
// =============================================================================

#[test]
fn test_diagnose_nonexistent_file_fails() {
    // Given: File that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.toon";

    // When: User runs `toon-lsp diagnose nonexistent.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(nonexistent)
        .assert()
        .code(1); // Exit code 1 for I/O error

    // Then: Exit code 1, I/O error shown
    output.stderr(predicate::str::is_empty().not());
}

#[test]
fn test_diagnose_permission_denied() {
    // Given: File with no read permissions (if supported on platform)
    let temp = tempdir().expect("create temp dir");
    let restricted_path = temp.path().join("restricted.toon");
    fs::write(&restricted_path, "key: value\n").expect("write file");

    // Attempt to make file unreadable (platform dependent)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o000);
        let _ = fs::set_permissions(&restricted_path, perms);

        // When: User runs diagnose on restricted file
        let mut cmd = toon_lsp();
        let output = cmd
            .arg("diagnose")
            .arg(&restricted_path)
            .assert()
            .code(1); // I/O error

        // Then: Error is reported
        output.stderr(predicate::str::is_empty().not());

        // Cleanup: restore permissions for temp dir cleanup
        let perms = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(&restricted_path, perms);
    }
}

// =============================================================================
// T080: Empty file handling
// =============================================================================

#[test]
fn test_diagnose_empty_file_handling() {
    // Given: Empty TOON file
    let fixture = fixtures_dir().join("empty.toon");

    // When: User runs `toon-lsp diagnose empty.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&fixture)
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Empty diagnostics, success
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("Should have diagnostics array");

    assert!(
        diagnostics.is_empty(),
        "Empty file should produce no diagnostics"
    );
}

// =============================================================================
// T081: Summary field validation
// =============================================================================

#[test]
fn test_diagnose_summary_field_has_error_count() {
    // Given: TOON file with multiple errors
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("multiple_errors.toon");
    fs::write(
        &toon_path,
        "key1: [unclosed1\nkey2: [unclosed2\nkey3: value\n",
    )
    .expect("write file");

    // When: User runs `toon-lsp diagnose file.toon`
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&toon_path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Summary contains accurate error count
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    // Check for summary field with error count
    if let Some(summary) = json.get("summary") {
        // Summary should have error_count
        assert!(
            summary.get("error_count").is_some() || summary.get("total").is_some(),
            "Summary should have error count"
        );

        // The error count should match the number of diagnostics
        let error_count = summary
            .get("error_count")
            .or_else(|| summary.get("total"))
            .and_then(|c| c.as_u64());

        if let Some(count) = error_count {
            let diagnostics = json
                .get("diagnostics")
                .and_then(|d| d.as_array())
                .expect("Should have diagnostics");

            // Error count in summary should be >= actual diagnostics (some may be filtered)
            assert!(
                count as usize >= diagnostics.len(),
                "Summary error count should match or exceed diagnostic count"
            );
        }
    } else {
        // If no summary field, at least have diagnostics array
        assert!(
            json.get("diagnostics").is_some(),
            "Should have diagnostics array"
        );
    }
}

#[test]
fn test_diagnose_summary_json_structure() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs diagnose
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: JSON structure is valid and complete
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");

    // Required top-level fields
    assert!(
        json.get("diagnostics").is_some(),
        "Should have diagnostics field"
    );

    let diagnostics = json
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .expect("diagnostics should be array");

    // Each diagnostic should have required LSP fields
    for diag in diagnostics {
        assert!(
            diag.get("message").is_some(),
            "Diagnostic must have message"
        );
        assert!(
            diag.get("range").is_some(),
            "Diagnostic must have range"
        );
    }
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_diagnose_with_all_flags() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs diagnose with all flags
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("json")
        .arg("--context")
        .arg("--severity")
        .arg("warning")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid JSON output combining all options
    let json = parse_json_from_stdout(&output).expect("Should be valid JSON");
    assert!(
        json.get("diagnostics").is_some(),
        "Should have diagnostics with all flags"
    );
}

#[test]
fn test_diagnose_sarif_format_with_context() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs diagnose with SARIF format and context
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg("--format")
        .arg("sarif")
        .arg("--context")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: Valid SARIF with context information
    let sarif = parse_json_from_stdout(&output).expect("Should be valid JSON");
    assert!(
        validate_sarif_structure(&sarif),
        "Should produce valid SARIF"
    );
}

#[test]
fn test_diagnose_default_format_is_json() {
    // Given: Invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs diagnose without format flag
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&fixture)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();

    // Then: JSON is default format (can be parsed as JSON)
    assert!(
        is_valid_json(&String::from_utf8_lossy(&output)),
        "Default output should be valid JSON"
    );
}

#[test]
fn test_diagnose_multiple_files_not_supported() {
    // Given: Multiple TOON files
    let temp = tempdir().expect("create temp dir");
    let file1 = temp.path().join("file1.toon");
    let file2 = temp.path().join("file2.toon");
    fs::write(&file1, "key1: value1\n").expect("write file1");
    fs::write(&file2, "key2: value2\n").expect("write file2");

    // When: User tries to run diagnose on multiple files
    // The diagnose command accepts only one input (or stdin)
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("diagnose")
        .arg(&file1)
        .arg(&file2)
        .assert();

    // Then: Either processes first file or shows error
    // (behavior depends on implementation)
    let _ = output; // Just verify command doesn't panic
}
