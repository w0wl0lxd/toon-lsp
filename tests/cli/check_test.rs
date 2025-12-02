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

//! Integration tests for the check command.
//!
//! T035-T038: Tests for User Story 3 - Validate TOON Files in CI

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
// T035: Integration test for check command with valid file
// =============================================================================

#[test]
fn test_check_valid_file_succeeds() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp check file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture);

    // Then: Exit code is 0
    cmd.assert().success();
}

#[test]
fn test_check_valid_file_with_nesting_succeeds() {
    // Given: A valid TOON file with nested structure
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    let toon_content = "server:\n  host: localhost\n  port: 8080\n";
    fs::write(&toon_path, toon_content).expect("write file");

    // When: User runs check
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&toon_path);

    // Then: Exit code is 0
    cmd.assert().success();
}

#[test]
fn test_check_empty_file_succeeds() {
    // Given: An empty TOON file
    let fixture = fixtures_dir().join("empty.toon");

    // When: User runs check
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture);

    // Then: Exit code is 0 (empty is valid)
    cmd.assert().success();
}

// =============================================================================
// T036: Integration test for check with invalid file (exit 2)
// =============================================================================

#[test]
fn test_check_invalid_file_fails() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp check bad.toon`
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture);

    // Then: Errors are displayed and exit code is 2
    cmd.assert().code(2).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_check_syntax_error_shows_location() {
    // Given: A TOON file with syntax error
    let temp = tempdir().expect("create temp dir");
    let bad_path = temp.path().join("bad.toon");
    fs::write(&bad_path, "key: [unclosed array").expect("write file");

    // When: User runs check
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&bad_path);

    // Then: Error shows line/column info
    cmd.assert().code(2).stderr(predicate::str::contains("line").or(predicate::str::contains("1")));
}

#[test]
fn test_check_nonexistent_file_fails() {
    // Given: A file that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.toon";

    // When: User runs check
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(nonexistent);

    // Then: Exit code is 1 (I/O error)
    cmd.assert().code(1).stderr(predicate::str::is_empty().not());
}

// =============================================================================
// T037: Integration test for check with --format github
// =============================================================================

#[test]
fn test_check_github_format_valid() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp check --format github file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture).args(["-f", "github"]);

    // Then: Exit code is 0 (no output for valid file)
    cmd.assert().success();
}

#[test]
fn test_check_github_format_invalid() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs check with github format
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture).args(["-f", "github"]);

    // Then: Output uses GitHub Actions annotation format (::error)
    cmd.assert().code(2).stderr(predicate::str::contains("::error"));
}

#[test]
fn test_check_json_format() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs check with json format
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture).args(["-f", "json"]);

    // Then: Output is JSON formatted
    cmd.assert().code(2).stderr(predicate::str::contains("{").and(predicate::str::contains("}")));
}

// =============================================================================
// T038: Integration test for check with multiple files (batch)
// =============================================================================

#[test]
fn test_check_multiple_valid_files() {
    // Given: Multiple valid TOON files
    let temp = tempdir().expect("create temp dir");
    let file1 = temp.path().join("file1.toon");
    let file2 = temp.path().join("file2.toon");
    fs::write(&file1, "key1: value1\n").expect("write file1");
    fs::write(&file2, "key2: value2\n").expect("write file2");

    // When: User runs check on multiple files
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&file1).arg(&file2);

    // Then: Exit code is 0
    cmd.assert().success();
}

#[test]
fn test_check_mixed_valid_invalid_files() {
    // Given: One valid and one invalid file
    let temp = tempdir().expect("create temp dir");
    let valid_file = temp.path().join("valid.toon");
    let invalid_file = temp.path().join("invalid.toon");
    fs::write(&valid_file, "key: value\n").expect("write valid");
    fs::write(&invalid_file, "key: [unclosed").expect("write invalid");

    // When: User runs check on both
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&valid_file).arg(&invalid_file);

    // Then: Exit code is 2 (at least one failed), both files are processed
    cmd.assert().code(2).stderr(predicate::str::contains("invalid.toon"));
}

#[test]
fn test_check_batch_reports_all_errors() {
    // Given: Multiple invalid files
    let temp = tempdir().expect("create temp dir");
    let bad1 = temp.path().join("bad1.toon");
    let bad2 = temp.path().join("bad2.toon");
    fs::write(&bad1, "key1: [unclosed1").expect("write bad1");
    fs::write(&bad2, "key2: [unclosed2").expect("write bad2");

    // When: User runs check on all
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&bad1).arg(&bad2);

    // Then: All errors are reported (not fail-fast)
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("bad1.toon"))
        .stderr(predicate::str::contains("bad2.toon"));
}

// =============================================================================
// Additional check tests
// =============================================================================

#[test]
fn test_check_from_stdin() {
    // Given: TOON content via stdin
    let valid_toon = "key: value\n";

    // When: User runs check with stdin
    let mut cmd = toon_lsp();
    cmd.arg("check").arg("-").write_stdin(valid_toon);

    // Then: Exit code is 0
    cmd.assert().success();
}

#[test]
fn test_check_from_stdin_invalid() {
    // Given: Invalid TOON via stdin
    let invalid_toon = "key: [unclosed";

    // When: User runs check with stdin
    let mut cmd = toon_lsp();
    cmd.arg("check").arg("-").write_stdin(invalid_toon);

    // Then: Exit code is 2
    cmd.assert().code(2);
}

#[test]
fn test_check_text_format_default() {
    // Given: An invalid TOON file
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs check without format flag (default is text)
    let mut cmd = toon_lsp();
    cmd.arg("check").arg(&fixture);

    // Then: Output is human-readable text
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}
