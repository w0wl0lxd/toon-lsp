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

//! Integration tests for the decode command.
//!
//! T023-T027: Tests for User Story 2 - Convert TOON to JSON

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Path to fixtures directory
fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures")
}

/// Get a command for the toon-lsp binary
fn toon_lsp() -> Command {
    Command::cargo_bin("toon-lsp").expect("Failed to find toon-lsp binary")
}

// =============================================================================
// T023: Integration test for decode command (basic TOON → JSON)
// =============================================================================

#[test]
fn test_decode_toon_file_to_stdout() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp decode input.toon`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture);

    // Then: JSON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_decode_nested_toon_file() {
    // Given: A TOON file with nested structure
    // Create a temp file with nested TOON content
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    let toon_content = "server:\n  host: localhost\n  port: 8080\n";
    fs::write(&toon_path, toon_content).expect("write toon file");

    // When: User runs `toon-lsp decode input.toon`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&toon_path);

    // Then: JSON output contains nested structure
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("host"))
        .stdout(predicate::str::contains("localhost"))
        .stdout(predicate::str::contains("port"));
}

// =============================================================================
// T024: Integration test for decode with --pretty
// =============================================================================

#[test]
fn test_decode_with_pretty_flag() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp decode input.toon --pretty`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture).arg("--pretty");

    // Then: Pretty-printed JSON is output (contains newlines and indentation)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\n"))
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_decode_with_short_pretty_flag() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp decode input.toon -p`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture).arg("-p");

    // Then: Pretty-printed JSON is output
    cmd.assert().success().stdout(predicate::str::contains("\n"));
}

// =============================================================================
// T025: Integration test for decode to YAML (-f yaml)
// =============================================================================

#[test]
fn test_decode_to_yaml_format() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp decode input.toon -f yaml`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture).args(["-f", "yaml"]);

    // Then: YAML output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        // YAML uses `: ` not `":` like JSON
        .stdout(predicate::str::contains(": "));
}

#[test]
fn test_decode_to_yaml_with_long_option() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp decode input.toon --output-format yaml`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture).args(["--output-format", "yaml"]);

    // Then: YAML output is written to stdout
    cmd.assert().success().stdout(predicate::str::contains("name"));
}

// =============================================================================
// T026: Integration test for decode with invalid TOON (exit code 2)
// =============================================================================

#[test]
fn test_decode_invalid_toon_file_fails() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp decode bad.toon`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture);

    // Then: Parse errors are displayed and exit code is 2
    cmd.assert().code(2).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_decode_invalid_stdin_fails() {
    // Given: Invalid TOON piped to stdin
    let invalid_toon = "key: [unclosed array";

    // When: User runs decode with invalid input
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg("-").write_stdin(invalid_toon);

    // Then: Exit code is 2 (validation failure)
    cmd.assert().code(2).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_decode_nonexistent_file_fails() {
    // Given: A file that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.toon";

    // When: User runs `toon-lsp decode nonexistent.toon`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(nonexistent);

    // Then: Exit code is 1 (I/O error)
    cmd.assert().code(1).stderr(predicate::str::is_empty().not());
}

// =============================================================================
// T027: Round-trip test (encode → decode)
// =============================================================================

#[test]
fn test_roundtrip_json_to_toon_to_json() {
    // Given: Original JSON
    let original_json = r#"{"name":"test","count":42,"active":true}"#;

    // When: We encode to TOON
    let mut encode_cmd = toon_lsp();
    let encode_output = encode_cmd
        .arg("encode")
        .arg("-")
        .write_stdin(original_json)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // And then decode back to JSON
    let toon_content = String::from_utf8_lossy(&encode_output);
    let mut decode_cmd = toon_lsp();
    let decode_output = decode_cmd
        .arg("decode")
        .arg("-")
        .write_stdin(toon_content.as_bytes())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: The decoded JSON is semantically equivalent to the original
    let decoded_json = String::from_utf8_lossy(&decode_output);
    let original: serde_json::Value = serde_json::from_str(original_json).expect("parse original");
    let decoded: serde_json::Value = serde_json::from_str(&decoded_json).expect("parse decoded");

    assert_eq!(original, decoded);
}

#[test]
fn test_roundtrip_complex_structure() {
    // Given: Complex JSON with arrays and nested objects
    let original_json =
        r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}],"metadata":{"version":"1.0"}}"#;

    // When: We encode to TOON and decode back
    let mut encode_cmd = toon_lsp();
    let encode_output = encode_cmd
        .arg("encode")
        .arg("-")
        .write_stdin(original_json)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let toon_content = String::from_utf8_lossy(&encode_output);
    let mut decode_cmd = toon_lsp();
    let decode_output = decode_cmd
        .arg("decode")
        .arg("-")
        .write_stdin(toon_content.as_bytes())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: Round-trip preserves data
    let decoded_json = String::from_utf8_lossy(&decode_output);
    let original: serde_json::Value = serde_json::from_str(original_json).expect("parse original");
    let decoded: serde_json::Value = serde_json::from_str(&decoded_json).expect("parse decoded");

    assert_eq!(original, decoded);
}

// =============================================================================
// Additional decode tests
// =============================================================================

#[test]
fn test_decode_from_stdin() {
    // Given: TOON piped to stdin
    let toon_input = "name: test\ncount: 42\n";

    // When: User runs `toon-lsp decode -`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg("-").write_stdin(toon_input);

    // Then: JSON output is written to stdout
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_decode_to_output_file() {
    // Given: A valid TOON file and output path
    let fixture = fixtures_dir().join("simple.toon");
    let temp = tempdir().expect("create temp dir");
    let output_path = temp.path().join("output.json");

    // When: User runs `toon-lsp decode input.toon -o output.json`
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture).arg("-o").arg(&output_path);

    // Then: JSON is written to the specified file
    cmd.assert().success();

    let output_content = fs::read_to_string(&output_path).expect("read output");
    assert!(output_content.contains("name"));
}

#[test]
fn test_decode_empty_toon() {
    // Given: Empty TOON file
    let fixture = fixtures_dir().join("empty.toon");

    // When: User runs decode
    let mut cmd = toon_lsp();
    cmd.arg("decode").arg(&fixture);

    // Then: Should succeed (empty is valid)
    cmd.assert().success();
}
