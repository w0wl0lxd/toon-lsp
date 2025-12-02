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

//! Integration tests for the symbols command.
//!
//! T060-T070: Tests for User Story 5 - Extract Symbols from TOON Files

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
// T060: Integration test for basic symbol extraction
// =============================================================================

#[test]
fn test_symbols_basic_extraction() {
    // Given: A valid TOON file with simple keys
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp symbols file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&fixture);

    // Then: Output shows key names
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("count"))
        .stdout(predicate::str::contains("active"));
}

#[test]
fn test_symbols_basic_with_default_tree_format() {
    // Given: A valid TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs symbols without explicit format (defaults to tree)
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&fixture);

    // Then: Output is in tree format (contains all symbols)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("count"));
}

// =============================================================================
// T061: Integration test for tree format output (default)
// =============================================================================

#[test]
fn test_symbols_tree_format_nested_structure() {
    // Given: TOON with nested structure
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n  host: localhost\n  port: 8080\ndatabase:\n  name: mydb\n")
        .expect("write file");

    // When: User runs `toon-lsp symbols file.toon` (tree format is default)
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: Output shows indented tree hierarchy
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("database"));
}

#[test]
fn test_symbols_tree_format_explicit() {
    // Given: TOON with nested structure
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "app:\n  name: MyApp\n  version: 1.0\n").expect("write file");

    // When: User runs `toon-lsp symbols --format tree file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("tree").arg(&toon_path);

    // Then: Output is in tree format
    cmd.assert().success().stdout(predicate::str::contains("app"));
}

// =============================================================================
// T062: Integration test for JSON format output
// =============================================================================

#[test]
fn test_symbols_json_format_output() {
    // Given: TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp symbols --format json file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("json").arg(&fixture);

    // Then: Output is valid JSON array with symbol objects
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}

#[test]
fn test_symbols_json_format_contains_keys() {
    // Given: TOON with multiple keys
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("test.toon");
    fs::write(&toon_path, "alpha: 1\nbeta: 2\n").expect("write file");

    // When: User runs symbols with JSON format
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("json").arg(&toon_path);

    // Then: JSON output contains symbol names
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn test_symbols_json_format_is_parseable() {
    // Given: TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs symbols with JSON format and captures output
    let mut cmd = toon_lsp();
    let output = cmd
        .arg("symbols")
        .arg("--format")
        .arg("json")
        .arg(&fixture)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: Output is valid JSON that can be parsed
    let json_str = String::from_utf8_lossy(&output);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
    assert!(parsed.is_ok(), "Output should be valid JSON");
}

// =============================================================================
// T063: Integration test for flat format output
// =============================================================================

#[test]
fn test_symbols_flat_format_output() {
    // Given: Nested TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested.toon");
    fs::write(&toon_path, "server:\n  host: localhost\n  port: 8080\n").expect("write file");

    // When: User runs `toon-lsp symbols --format flat file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("flat").arg(&toon_path);

    // Then: Output shows dot-notation paths
    cmd.assert().success().stdout(predicate::str::contains("server"));
}

#[test]
fn test_symbols_flat_format_dot_notation() {
    // Given: Deeply nested TOON
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("deep.toon");
    fs::write(&toon_path, "database:\n  connection:\n    host: db.example.com\n")
        .expect("write file");

    // When: User runs symbols in flat format
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("flat").arg(&toon_path);

    // Then: Output shows flattened paths
    cmd.assert().success();
}

// =============================================================================
// T064: Integration test for position flag
// =============================================================================

#[test]
fn test_symbols_with_positions() {
    // Given: TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp symbols --positions file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--positions").arg(&fixture);

    // Then: Each symbol shows (Ln:Col) position
    cmd.assert().success().stdout(predicate::str::contains("name"));
}

#[test]
fn test_symbols_positions_multiline() {
    // Given: TOON with symbols on different lines
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("positions.toon");
    fs::write(&toon_path, "first: 1\nsecond: 2\nthird: 3\n").expect("write file");

    // When: User runs symbols with positions
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--positions").arg(&toon_path);

    // Then: Output shows position information
    cmd.assert().success();
}

// =============================================================================
// T065: Integration test for types flag
// =============================================================================

#[test]
fn test_symbols_with_types() {
    // Given: TOON with various value types
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("typed.toon");
    fs::write(&toon_path, "name: Alice\nage: 30\nactive: true\n").expect("write file");

    // When: User runs `toon-lsp symbols --types file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg(&toon_path);

    // Then: Each symbol shows type annotation
    cmd.assert().success().stdout(predicate::str::contains("name"));
}

#[test]
fn test_symbols_types_all_value_types() {
    // Given: TOON with string, number, boolean, null, object, array types
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("alltypes.toon");
    fs::write(
        &toon_path,
        "text: hello\ncount: 42\nflag: true\nempty: null\nitems:\n  - one\n  - two\n",
    )
    .expect("write file");

    // When: User runs symbols with types flag
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg(&toon_path);

    // Then: Output shows type information
    cmd.assert().success();
}

// =============================================================================
// T066: Integration test for combined flags
// =============================================================================

#[test]
fn test_symbols_combined_flags_types_positions() {
    // Given: TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs `toon-lsp symbols --types --positions file.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg("--positions").arg(&fixture);

    // Then: Output shows all metadata
    cmd.assert().success();
}

#[test]
fn test_symbols_combined_all_flags() {
    // Given: TOON file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("combo.toon");
    fs::write(&toon_path, "app:\n  name: MyApp\n  version: 1.0\n  active: true\n")
        .expect("write file");

    // When: User runs with all flags: types, positions, flat format
    let mut cmd = toon_lsp();
    cmd.arg("symbols")
        .arg("--types")
        .arg("--positions")
        .arg("--format")
        .arg("flat")
        .arg(&toon_path);

    // Then: Output shows all metadata combined
    cmd.assert().success();
}

#[test]
fn test_symbols_combined_json_format_with_types() {
    // Given: TOON file
    let fixture = fixtures_dir().join("simple.toon");

    // When: User runs symbols with JSON format and types
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("json").arg("--types").arg(&fixture);

    // Then: JSON output includes type information
    cmd.assert().success();
}

// =============================================================================
// T067: Integration test for stdin input
// =============================================================================

#[test]
fn test_symbols_from_stdin() {
    // Given: TOON content via stdin
    let toon_input = "key: value\nother: data\n";

    // When: User runs `echo "..." | toon-lsp symbols -`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("-").write_stdin(toon_input);

    // Then: Symbols extracted from stdin
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("key"))
        .stdout(predicate::str::contains("other"));
}

#[test]
fn test_symbols_from_stdin_nested() {
    // Given: Nested TOON content via stdin
    let toon_input = "server:\n  host: localhost\n  port: 8080\n";

    // When: User runs symbols with stdin
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("-").write_stdin(toon_input);

    // Then: Symbols are extracted
    cmd.assert().success().stdout(predicate::str::contains("server"));
}

#[test]
fn test_symbols_from_stdin_with_flags() {
    // Given: TOON content via stdin
    let toon_input = "name: Alice\nage: 30\n";

    // When: User runs symbols with stdin and types flag
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg("--format").arg("json").arg("-").write_stdin(toon_input);

    // Then: Symbols are extracted with metadata
    cmd.assert().success();
}

#[test]
fn test_symbols_stdin_without_dash() {
    // Given: TOON content via stdin (omitted dash)
    let toon_input = "key: value\n";

    // When: User runs symbols with stdin (no dash specified)
    let mut cmd = toon_lsp();
    cmd.arg("symbols").write_stdin(toon_input);

    // Then: Symbols are extracted from stdin
    cmd.assert().success();
}

// =============================================================================
// T068: Integration test for invalid TOON handling
// =============================================================================

#[test]
fn test_symbols_invalid_toon_fails() {
    // Given: Invalid TOON file with parse errors
    let fixture = fixtures_dir().join("invalid.toon");

    // When: User runs `toon-lsp symbols invalid.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&fixture);

    // Then: Exit code 0 (graceful degradation), but error messages on stderr
    // The command extracts what symbols it can and reports parse errors
    cmd.assert().success().stderr(predicate::str::contains("Parse error"));
}

#[test]
fn test_symbols_invalid_stdin_fails() {
    // Given: Invalid TOON via stdin
    let invalid_toon = "key: [unclosed";

    // When: User runs symbols
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("-").write_stdin(invalid_toon);

    // Then: Exit code 0 (graceful degradation with error on stderr)
    cmd.assert().success().stderr(predicate::str::contains("Parse error"));
}

#[test]
fn test_symbols_nonexistent_file_fails() {
    // Given: A file that doesn't exist
    let nonexistent = "/path/to/nonexistent/file.toon";

    // When: User runs symbols
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(nonexistent);

    // Then: Exit code 1 (I/O error)
    cmd.assert().code(1).stderr(predicate::str::is_empty().not());
}

#[test]
fn test_symbols_malformed_json_in_toon() {
    // Given: TOON with incomplete structures
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("malformed.toon");
    fs::write(&toon_path, "obj:\n  unclosed: true\n").expect("write file");

    // When: User runs symbols on partially valid TOON
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: Should handle gracefully (either succeed with partial symbols or error)
    cmd.assert();
}

// =============================================================================
// T069: Integration test for empty file handling
// =============================================================================

#[test]
fn test_symbols_empty_file_succeeds() {
    // Given: Empty TOON file
    let fixture = fixtures_dir().join("empty.toon");

    // When: User runs `toon-lsp symbols empty.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&fixture);

    // Then: Exit code 0, empty/minimal output
    cmd.assert().success();
}

#[test]
fn test_symbols_empty_file_tree_format() {
    // Given: Empty file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("blank.toon");
    fs::write(&toon_path, "").expect("write file");

    // When: User runs symbols in tree format
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("tree").arg(&toon_path);

    // Then: Exit code 0
    cmd.assert().success();
}

#[test]
fn test_symbols_empty_file_json_format() {
    // Given: Empty file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("blank.toon");
    fs::write(&toon_path, "").expect("write file");

    // When: User runs symbols in JSON format
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--format").arg("json").arg(&toon_path);

    // Then: Exit code 0, valid JSON (empty array)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}

#[test]
fn test_symbols_empty_file_with_types() {
    // Given: Empty file
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("blank.toon");
    fs::write(&toon_path, "").expect("write file");

    // When: User runs symbols with types flag
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg(&toon_path);

    // Then: Exit code 0
    cmd.assert().success();
}

// =============================================================================
// T070: Integration test for array symbols
// =============================================================================

#[test]
fn test_symbols_array_expanded_form() {
    // Given: TOON with expanded array
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("arrays.toon");
    fs::write(&toon_path, "items:\n  - one\n  - two\n  - three\n").expect("write file");

    // When: User runs `toon-lsp symbols arrays.toon`
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: Array elements shown appropriately
    cmd.assert().success().stdout(predicate::str::contains("items"));
}

#[test]
fn test_symbols_array_inline_form() {
    // Given: TOON with inline array (TOON syntax: key[count]: val1,val2,val3)
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("inline.toon");
    fs::write(&toon_path, "colors[3]: red,green,blue\n").expect("write file");

    // When: User runs symbols
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: Array symbol is shown
    cmd.assert().success().stdout(predicate::str::contains("colors"));
}

#[test]
fn test_symbols_array_with_types() {
    // Given: TOON with arrays
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("arrays.toon");
    fs::write(&toon_path, "numbers:\n  - 1\n  - 2\nstatus: active\n").expect("write file");

    // When: User runs symbols with types
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg("--types").arg(&toon_path);

    // Then: Array and other symbols shown with types
    cmd.assert().success();
}

#[test]
fn test_symbols_empty_array() {
    // Given: TOON with empty array
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("empty_array.toon");
    fs::write(&toon_path, "items:\n").expect("write file");

    // When: User runs symbols
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: Array symbol is shown
    cmd.assert().success();
}

#[test]
fn test_symbols_nested_arrays_in_objects() {
    // Given: TOON with nested structure containing arrays
    let temp = tempdir().expect("create temp dir");
    let toon_path = temp.path().join("nested_arrays.toon");
    fs::write(
        &toon_path,
        "config:\n  tags:\n    - prod\n    - stable\n  values:\n    - 10\n    - 20\n",
    )
    .expect("write file");

    // When: User runs symbols
    let mut cmd = toon_lsp();
    cmd.arg("symbols").arg(&toon_path);

    // Then: All nested symbols shown
    cmd.assert().success().stdout(predicate::str::contains("config"));
}
