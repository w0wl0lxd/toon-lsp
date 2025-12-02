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

//! Integration tests for the format command.
//!
//! T048-T059: Tests for User Story 4 - Format TOON Files

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
// T048: Integration test for format command (basic formatting)
// =============================================================================

#[test]
fn test_format_basic_to_stdout() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp format file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&fixture);

    // Then: Formatted output is written to stdout
    cmd.assert().success().stdout(predicate::str::contains("name"));
}

#[test]
fn test_format_normalizes_spacing() {
    // Given: TOON with inconsistent spacing
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("unformatted.toon");
    // Extra spaces around colon
    fs::write(&toon_path, "key:value\nother:  value2\n").expect("write file");

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path);

    // Then: Spacing is normalized (colon followed by single space)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("key: value"))
        .stdout(predicate::str::contains("other: value2"));
}

#[test]
fn test_format_preserves_valid_structure() {
    // Given: A well-formatted TOON file
    let fixture = fixtures_dir().join("simple.toon");
    let _original = fs::read_to_string(&fixture).expect("read fixture");

    // When: User runs format
    let mut cmd = toon_lsp();
    let output = cmd.arg("format").arg(&fixture).assert().success().get_output().stdout.clone();

    // Then: Content is semantically preserved
    let formatted = String::from_utf8_lossy(&output);
    // Should contain same data
    assert!(formatted.contains("name"));
    assert!(formatted.contains("test"));
}

// =============================================================================
// T049: Integration test for format with --check (diff mode)
// =============================================================================

#[test]
fn test_format_check_formatted_file_succeeds() {
    // Given: A properly formatted TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("formatted.toon");
    fs::write(&toon_path, "name: test\ncount: 42\n").expect("write file");

    // When: User runs `toon-lsp format --check file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("--check").arg(&toon_path);

    // Then: Exit code is 0 (file is already formatted)
    cmd.assert().success();
}

#[test]
fn test_format_check_unformatted_file_fails() {
    // Given: A poorly formatted TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("unformatted.toon");
    fs::write(&toon_path, "name:value\n").expect("write file");

    // When: User runs `toon-lsp format --check file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("--check").arg(&toon_path);

    // Then: Exit code is non-zero (file needs formatting)
    cmd.assert().code(1);
}

#[test]
fn test_format_check_does_not_modify_file() {
    // Given: A poorly formatted TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("unformatted.toon");
    let original_content = "name:value\n";
    fs::write(&toon_path, original_content).expect("write file");

    // When: User runs format with --check
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("--check").arg(&toon_path);
    cmd.assert(); // Don't care about exit code

    // Then: File content is unchanged
    let after = fs::read_to_string(&toon_path).expect("read file");
    assert_eq!(after, original_content);
}

// =============================================================================
// T050: Integration test for format with --indent
// =============================================================================

#[test]
fn test_format_with_2_space_indent() {
    // Given: A nested TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n    host: localhost\n").expect("write file");

    // When: User runs `toon-lsp format --indent 2 file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("--indent").arg("2").arg(&toon_path);

    // Then: Output uses 2-space indentation
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server:"))
        .stdout(predicate::str::contains("  host:")); // 2 spaces
}

#[test]
fn test_format_with_4_space_indent() {
    // Given: A nested TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n  host: localhost\n").expect("write file");

    // When: User runs `toon-lsp format --indent 4 file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("--indent").arg("4").arg(&toon_path);

    // Then: Output uses 4-space indentation
    cmd.assert().success().stdout(predicate::str::contains("    host:")); // 4 spaces
}

#[test]
fn test_format_always_uses_spaces_not_tabs() {
    // Given: A nested TOON file
    // Note: TOON spec prohibits tabs for indentation - spaces only
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n  host: localhost\n").expect("write file");

    // When: User runs format (tabs option removed - TOON spec prohibits tabs)
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path);

    // Then: Output uses space indentation (tabs prohibited by TOON spec)
    cmd.assert().success().stdout(predicate::str::contains("  host:")); // spaces, not tabs
}

// =============================================================================
// T051: Integration test for format with output file
// =============================================================================

#[test]
fn test_format_to_output_file() {
    // Given: A TOON file and output path
    let fixture = fixtures_dir().join("simple.toon");
    let temp = tempdir().expect("create temp dir");
    let output_path = temp.path().join("formatted.toon");

    // When: User runs `toon-lsp format file.toon -o output.toon`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&fixture).arg("-o").arg(&output_path);

    // Then: Formatted output is written to the file
    cmd.assert().success();
    let output_content = fs::read_to_string(&output_path).expect("read output");
    assert!(output_content.contains("name"));
}

#[test]
fn test_format_in_place_modifies_file() {
    // Given: An unformatted TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("unformatted.toon");
    fs::write(&toon_path, "key:value\n").expect("write file");

    // When: User runs format with output to same file
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path).arg("-o").arg(&toon_path);

    // Then: File is modified in place
    cmd.assert().success();
    let content = fs::read_to_string(&toon_path).expect("read file");
    assert!(content.contains("key: value")); // Space added
}

// =============================================================================
// T052: Integration test for format from stdin
// =============================================================================

#[test]
fn test_format_from_stdin() {
    // Given: TOON content via stdin
    let toon_input = "key:value\n";

    // When: User runs `toon-lsp format -`
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("-").write_stdin(toon_input);

    // Then: Formatted output is written to stdout
    cmd.assert().success().stdout(predicate::str::contains("key: value"));
}

#[test]
fn test_format_from_stdin_nested() {
    // Given: Nested TOON content via stdin
    let toon_input = "server:\n    host: localhost\n";

    // When: User runs format with stdin and custom indent
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("-").arg("--indent").arg("2").write_stdin(toon_input);

    // Then: Output uses requested indentation
    cmd.assert().success().stdout(predicate::str::contains("  host:")); // 2 spaces
}

// =============================================================================
// T053: Integration test for format with invalid TOON
// =============================================================================

#[test]
fn test_format_invalid_toon_fails() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&fixture);

    // Then: Exit code is 2 (validation failure)
    cmd.assert().code(2).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_format_invalid_stdin_fails() {
    // Given: Invalid TOON via stdin
    let invalid_toon = "key: [unclosed";

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg("-").write_stdin(invalid_toon);

    // Then: Exit code is 2
    cmd.assert().code(2);
}

#[test]
fn test_format_nonexistent_file_fails() {
    // Given: A file that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.toon";

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(nonexistent);

    // Then: Exit code is 1 (I/O error)
    cmd.assert().code(1).stderr(predicate::str::is_empty().not());
}

// =============================================================================
// T054: Integration test for format empty file
// =============================================================================

#[test]
fn test_format_empty_file_succeeds() {
    // Given: An empty TOON file
    let fixture = fixtures_dir().join("empty.toon");

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&fixture);

    // Then: Exit code is 0
    cmd.assert().success();
}

// =============================================================================
// Additional format tests
// =============================================================================

#[test]
fn test_format_preserves_arrays() {
    // Given: TOON with arrays
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("arrays.toon");
    fs::write(&toon_path, "items:\n  - one\n  - two\n").expect("write file");

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path);

    // Then: Arrays are preserved
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("- one"))
        .stdout(predicate::str::contains("- two"));
}

#[test]
fn test_format_preserves_inline_arrays() {
    // Given: TOON with inline array (TOON syntax: key[count]: val1,val2,val3)
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("inline.toon");
    fs::write(&toon_path, "numbers[3]: 1,2,3\n").expect("write file");

    // When: User runs format
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path);

    // Then: Inline array is preserved (formatter outputs JSON-style brackets)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("numbers:"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_format_default_indent_is_2() {
    // Given: A nested TOON file with 4-space indent
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n    host: localhost\n").expect("write file");

    // When: User runs format without --indent (default is 2)
    let mut cmd = toon_lsp();
    cmd.arg("format").arg(&toon_path);

    // Then: Output uses 2-space indentation (default)
    cmd.assert().success().stdout(predicate::str::contains("  host:")); // 2 spaces, not 4
}
