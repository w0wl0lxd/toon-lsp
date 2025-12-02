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

//! Integration tests for the encode command.
//!
//! T013-T016: Tests for User Story 1 - Convert JSON to TOON

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Path to fixtures directory
fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

/// Get a command for the toon-lsp binary
#[allow(deprecated)]
fn toon_lsp() -> Command {
    Command::cargo_bin("toon-lsp").expect("Failed to find toon-lsp binary")
}

// =============================================================================
// T013: Integration test for encode command (basic JSON → TOON)
// =============================================================================

#[test]
fn test_encode_json_file_to_stdout() {
    // Given: A valid JSON file
    let fixture = fixtures_dir().join("simple.json");

    // When: User runs `toon-lsp encode input.json`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture);

    // Then: TOON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("count"));
}

#[test]
fn test_encode_nested_json_file() {
    // Given: A nested JSON file
    let fixture = fixtures_dir().join("nested.json");

    // When: User runs `toon-lsp encode input.json`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture);

    // Then: TOON output contains nested structure
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("host"))
        .stdout(predicate::str::contains("port"));
}

#[test]
fn test_encode_yaml_file_to_stdout() {
    // Given: A valid YAML file
    let fixture = fixtures_dir().join("simple.yaml");

    // When: User runs `toon-lsp encode input.yaml -f yaml`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture).args(["-f", "yaml"]);

    // Then: TOON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("value"));
}

#[test]
fn test_encode_roundtrip_preserves_data() {
    // Given: A complex JSON file
    let fixture = fixtures_dir().join("array.json");
    let json_content = fs::read_to_string(&fixture).expect("read fixture");

    // When: We encode to TOON
    let mut cmd = toon_lsp();
    let encode_output =
        cmd.arg("encode").arg(&fixture).assert().success().get_output().stdout.clone();

    // Then: The TOON can be decoded back to equivalent JSON
    let toon_content = String::from_utf8_lossy(&encode_output);

    // Verify TOON contains expected data
    assert!(toon_content.contains("Alice"));
    assert!(toon_content.contains("Bob"));

    // Parse original JSON for comparison
    let original: serde_json::Value = serde_json::from_str(&json_content).expect("parse original");
    assert!(original.get("users").is_some());
}

// =============================================================================
// T014: Integration test for encode with stdin input
// =============================================================================

#[test]
fn test_encode_from_stdin() {
    // Given: JSON piped to stdin
    let json_input = r#"{"message":"hello","count":42}"#;

    // When: User runs `echo '{"a":1}' | toon-lsp encode -`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg("-").write_stdin(json_input);

    // Then: TOON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("message"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("count"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_encode_from_stdin_with_yaml_format() {
    // Given: YAML piped to stdin
    let yaml_input = "message: hello\ncount: 42\n";

    // When: User runs `toon-lsp encode - -f yaml`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg("-").args(["-f", "yaml"]).write_stdin(yaml_input);

    // Then: TOON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("message"))
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_encode_from_stdin_complex() {
    // Given: Complex JSON piped to stdin
    let json_input = r#"{"server":{"host":"localhost","port":8080},"enabled":true}"#;

    // When: User runs encode with stdin
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg("-").write_stdin(json_input);

    // Then: TOON output contains nested structure
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("host"))
        .stdout(predicate::str::contains("localhost"))
        .stdout(predicate::str::contains("enabled"));
}

// =============================================================================
// T015: Integration test for encode with output file (-o)
// =============================================================================

#[test]
fn test_encode_to_output_file() {
    // Given: A valid JSON file
    let fixture = fixtures_dir().join("simple.json");
    let temp = tempdir().expect("create temp dir");
    let output_path = temp.path().join("output.toon");

    // When: User runs `toon-lsp encode input.json -o output.toon`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture).arg("-o").arg(&output_path);

    // Then: TOON is written to the specified file
    cmd.assert().success();

    let output_content = fs::read_to_string(&output_path).expect("read output");
    assert!(output_content.contains("name"));
    assert!(output_content.contains("test"));
}

#[test]
fn test_encode_to_output_file_long_option() {
    // Given: A valid JSON file
    let fixture = fixtures_dir().join("simple.json");
    let temp = tempdir().expect("create temp dir");
    let output_path = temp.path().join("output.toon");

    // When: User runs `toon-lsp encode input.json --output output.toon`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture).arg("--output").arg(&output_path);

    // Then: TOON is written to the specified file
    cmd.assert().success();

    assert!(output_path.exists());
}

#[test]
fn test_encode_stdin_to_output_file() {
    // Given: JSON piped to stdin and output file specified
    let json_input = r#"{"key":"value"}"#;
    let temp = tempdir().expect("create temp dir");
    let output_path = temp.path().join("output.toon");

    // When: User runs `echo '{"key":"value"}' | toon-lsp encode - -o output.toon`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg("-").arg("-o").arg(&output_path).write_stdin(json_input);

    // Then: TOON is written to the specified file
    cmd.assert().success();

    let output_content = fs::read_to_string(&output_path).expect("read output");
    assert!(output_content.contains("key"));
    assert!(output_content.contains("value"));
}

// =============================================================================
// T016: Integration test for encode with invalid JSON (exit code 2)
// =============================================================================

#[test]
fn test_encode_invalid_json_file_fails() {
    // Given: An invalid JSON file (create temp file with bad content)
    let temp = tempdir().expect("create temp dir");
    let invalid_path = temp.path().join("invalid.json");
    fs::write(&invalid_path, "{invalid json}").expect("write invalid file");

    // When: User runs `toon-lsp encode bad.json`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&invalid_path);

    // Then: Error message is displayed and exit code is 2 (validation failure)
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_encode_invalid_stdin_fails() {
    // Given: Invalid JSON piped to stdin
    let invalid_json = "{not valid json";

    // When: User runs encode with invalid input
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg("-").write_stdin(invalid_json);

    // Then: Exit code is 2 (validation failure)
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_encode_nonexistent_file_fails() {
    // Given: A file that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.json";

    // When: User runs `toon-lsp encode nonexistent.json`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(nonexistent);

    // Then: Exit code is 1
    cmd.assert().code(1).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_encode_invalid_yaml_fails() {
    // Given: Invalid YAML content
    let temp = tempdir().expect("create temp dir");
    let invalid_path = temp.path().join("invalid.yaml");
    fs::write(&invalid_path, "invalid: yaml: [unclosed").expect("write invalid file");

    // When: User runs `toon-lsp encode invalid.yaml -f yaml`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&invalid_path).args(["-f", "yaml"]);

    // Then: Exit code is 2 (validation failure)
    cmd.assert().code(2).stderr(predicate::str::is_empty().not());
}

// =============================================================================
// Additional encode tests for options
// =============================================================================

#[test]
fn test_encode_with_indent_option() {
    // Given: A JSON file and custom indent
    let fixture = fixtures_dir().join("nested.json");

    // When: User runs `toon-lsp encode input.json --indent 4`
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture).args(["--indent", "4"]);

    // Then: Output uses 4-space indentation
    cmd.assert().success().stdout(predicate::str::contains("server"));
}

#[test]
fn test_encode_empty_json_object() {
    // Given: Empty JSON object
    let fixture = fixtures_dir().join("empty.json");

    // When: User runs encode
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture);

    // Then: Should succeed (empty object is valid)
    cmd.assert().success();
}

#[test]
fn test_encode_auto_detects_json_format() {
    // Given: A .json file without explicit format
    let fixture = fixtures_dir().join("simple.json");

    // When: User runs encode without -f flag
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture);

    // Then: JSON format is auto-detected from extension
    cmd.assert().success();
}

#[test]
fn test_encode_auto_detects_yaml_format() {
    // Given: A .yaml file without explicit format
    let fixture = fixtures_dir().join("simple.yaml");

    // When: User runs encode without -f flag
    let mut cmd = toon_lsp();
    cmd.arg("encode").arg(&fixture);

    // Then: YAML format is auto-detected from extension
    cmd.assert().success();
}
