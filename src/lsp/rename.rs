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

//! Rename symbol support for TOON keys.
//!
//! This module provides functionality to validate and rename object keys
//! throughout the document, which is used by the LSP `textDocument/rename`
//! and `textDocument/prepareRename` features.

use crate::ast::{AstNode, Span};
use crate::lsp::ast_utils::{calculate_offset, collect_all_keys, find_node_at_position};

/// Result of prepare-rename validation.
///
/// Returned by [`prepare_rename`] to indicate that a rename operation
/// is valid at the cursor position. The LSP client uses this information
/// to show a rename dialog with the current key name as the default.
///
/// # Fields
///
/// * `range` - The span of the key identifier to be renamed
/// * `placeholder` - The current key name to show in the rename dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareRenameResult {
    /// Range of the key to rename
    pub range: Span,
    /// Current key name as placeholder/default
    pub placeholder: String,
}

/// A single text replacement for rename.
///
/// Represents one edit operation to apply when renaming a key. When a key
/// appears multiple times in a document, each occurrence generates a separate
/// `RenameEdit`.
///
/// # Fields
///
/// * `span` - The location of the key occurrence to replace
/// * `new_text` - The new key name to insert
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameEdit {
    /// Range to replace
    pub span: Span,
    /// New key name
    pub new_text: String,
}

/// Validate if rename is possible at the given position.
///
/// Called by the LSP server to check if the cursor is positioned on a symbol
/// that can be renamed. Returns the range and current name if valid, or `None`
/// if the cursor is on a value or whitespace.
///
/// # Arguments
///
/// * `ast` - The parsed AST root node
/// * `text` - The original document text
/// * `line` - 0-based line number of cursor position
/// * `col` - 0-based UTF-8 column of cursor position
///
/// # Returns
///
/// `Some(PrepareRenameResult)` if cursor is on a key (renameable), `None` otherwise.
///
/// # Examples
///
/// ```ignore
/// let source = "name: Alice";
/// let (ast, _) = parse_with_errors(source);
/// let result = prepare_rename(&ast.unwrap(), source, 0, 2); // On "name"
/// assert!(result.is_some());
/// assert_eq!(result.unwrap().placeholder, "name");
/// ```
pub fn prepare_rename(
    ast: &AstNode,
    text: &str,
    line: u32,
    col: u32,
) -> Option<PrepareRenameResult> {
    // Calculate offset
    let offset = calculate_offset(text, line, col)?;

    // Use find_node_at_position to locate cursor
    let node_at_pos = find_node_at_position(ast, line, col, offset)?;

    // Check if on_key is Some
    let entry = node_at_pos.on_key?;

    // Return span and key name if on a key
    Some(PrepareRenameResult { range: entry.key_span, placeholder: entry.key.clone() })
}

/// Generate edits to rename a key at the given position.
///
/// Finds all occurrences of the key at the cursor position and generates
/// edit operations to rename them all to the new name. This ensures that
/// all uses of the key are updated consistently.
///
/// # Arguments
///
/// * `ast` - The parsed AST root node
/// * `text` - The original document text
/// * `line` - 0-based line number of cursor position
/// * `col` - 0-based UTF-8 column of cursor position
/// * `new_name` - The new key name to use
///
/// # Returns
///
/// Vector of [`RenameEdit`] for all occurrences of the key, sorted by position.
/// Returns empty vector if cursor is not on a key.
///
/// # Examples
///
/// ```ignore
/// let source = "id: 1\ndata:\n  id: 2";
/// let (ast, _) = parse_with_errors(source);
/// let edits = rename_key(&ast.unwrap(), source, 0, 0, "identifier");
/// assert_eq!(edits.len(), 2); // Both "id" keys renamed
/// ```
pub fn rename_key(
    ast: &AstNode,
    text: &str,
    line: u32,
    col: u32,
    new_name: &str,
) -> Vec<RenameEdit> {
    // Call prepare_rename to validate and get key name
    let prepare_result = match prepare_rename(ast, text, line, col) {
        Some(result) => result,
        None => return Vec::new(), // Not on a key
    };

    let key_name = &prepare_result.placeholder;

    // Use collect_all_keys to get all keys with spans
    let all_keys = collect_all_keys(ast);

    // Filter for exact key name matches and create RenameEdit for each
    let mut edits: Vec<RenameEdit> = all_keys
        .into_iter()
        .filter(|(k, _)| k == key_name) // Exact match only
        .map(|(_, span)| RenameEdit { span, new_text: new_name.to_string() })
        .collect();

    // Sort by position for consistent ordering
    edits.sort_by(|a, b| {
        a.span
            .start
            .line
            .cmp(&b.span.start.line)
            .then(a.span.start.column.cmp(&b.span.start.column))
    });

    edits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    // T035: Test prepare_rename validates cursor is on key
    #[test]
    fn test_prepare_rename_on_key() {
        let source = "name: Alice";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "name" (line 0, col 1 - middle of "name")
        let result = prepare_rename(&ast, source, 0, 1);
        assert!(result.is_some(), "Should validate rename on key");
        assert_eq!(result.unwrap().placeholder, "name");
    }

    // T036: Test prepare_rename returns range and placeholder
    #[test]
    fn test_prepare_rename_range_and_placeholder() {
        let source = "username: bob";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "username" (line 0, col 4)
        let result = prepare_rename(&ast, source, 0, 4);
        assert!(result.is_some(), "Should return prepare result");
        let result = result.unwrap();

        // Range should cover "username" (cols 0-8)
        assert_eq!(result.placeholder, "username");
        assert_eq!(result.range.start.column, 0);
        assert_eq!(result.range.end.column, 8);
    }

    // T037: Test prepare_rename rejects cursor on value
    #[test]
    fn test_prepare_rename_on_value_rejected() {
        let source = "name: Alice";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "Alice" (value) - line 0, col 7
        let result = prepare_rename(&ast, source, 0, 7);
        assert!(result.is_none(), "Should reject rename on value");
    }

    // T038: Test rename_key updates all occurrences
    #[test]
    fn test_rename_updates_all_occurrences() {
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on first "id" (line 0, col 0)
        let edits = rename_key(&ast, source, 0, 0, "identifier");

        // Should return 2 edits: both "id" keys
        assert_eq!(edits.len(), 2, "Should rename all occurrences");
        assert_eq!(edits[0].new_text, "identifier");
        assert_eq!(edits[1].new_text, "identifier");
    }

    // T039: Test rename preserves document validity
    #[test]
    fn test_rename_preserves_validity() {
        let source = "key: value";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Rename "key" to "new_key"
        let edits = rename_key(&ast, source, 0, 0, "new_key");
        assert_eq!(edits.len(), 1, "Should have one edit");

        // Apply edits and verify document still parses
        let new_doc = apply_edits(source, &edits);
        assert_eq!(new_doc, "new_key: value");

        // Verify it parses without errors
        let (new_ast, new_errors) = parse_with_errors(&new_doc);
        assert!(new_errors.is_empty(), "Renamed document should parse");
        assert!(new_ast.is_some(), "Should have AST after rename");
    }

    // T040: Test rename handles special characters
    #[test]
    fn test_rename_special_characters() {
        // Test with underscore (valid in TOON keys)
        let source = "my_key: value";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Rename "my_key" to "my_new_key"
        let edits = rename_key(&ast, source, 0, 0, "my_new_key");
        assert_eq!(edits.len(), 1, "Should have one edit");
        assert_eq!(edits[0].new_text, "my_new_key");

        // Verify edit span covers the original key
        assert_eq!(edits[0].span.start.column, 0);
        assert_eq!(edits[0].span.end.column, 6); // "my_key" is 6 chars
    }

    // T040a: Test rename warns when creating duplicate keys at same level
    #[test]
    fn test_rename_duplicate_key_warning() {
        let source = "name: Alice\nage: 30";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Renaming "age" (line 1, col 0) to "name" would create duplicate
        // Function should still return edits (LSP handler can add warning)
        let edits = rename_key(&ast, source, 1, 0, "name");
        assert_eq!(edits.len(), 1, "Should return edit even if creating duplicate");
        assert_eq!(edits[0].new_text, "name");

        // Apply and verify it creates duplicate (valid TOON but semantically questionable)
        let new_doc = apply_edits(source, &edits);
        assert!(new_doc.contains("name: Alice"));
        assert!(new_doc.contains("name: 30"));
    }

    /// Helper function to apply edits to source text.
    fn apply_edits(source: &str, edits: &[RenameEdit]) -> String {
        // Sort edits by position (reverse order to apply from end to start)
        let mut sorted_edits = edits.to_vec();
        sorted_edits.sort_by(|a, b| b.span.start.offset.cmp(&a.span.start.offset));

        let mut result = source.to_string();
        for edit in sorted_edits {
            let start = edit.span.start.offset as usize;
            let end = edit.span.end.offset as usize;
            result.replace_range(start..end, &edit.new_text);
        }

        result
    }
}
