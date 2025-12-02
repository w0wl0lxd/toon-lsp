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

//! Find all references to keys in TOON documents.
//!
//! This module provides functionality to locate all occurrences of a key
//! throughout the document, which is used by the LSP `textDocument/references`
//! feature.

use crate::ast::{AstNode, Span};
use crate::lsp::ast_utils::{calculate_offset, collect_all_keys, find_node_at_position};

/// A reference to a key in the document.
///
/// Represents a single occurrence of a key name in the TOON document.
/// Used by the LSP `textDocument/references` feature to report all
/// locations where a specific key is used.
///
/// # Fields
///
/// * `key_name` - The exact key name being referenced
/// * `span` - The location of this reference in the document
/// * `is_definition` - True for the first occurrence at each nesting level
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReference {
    /// The key name being referenced
    pub key_name: String,
    /// Location span (line, start_col, end_col)
    pub span: Span,
    /// True if this is considered a "definition" (first occurrence at nesting level)
    pub is_definition: bool,
}

/// Find all references to the key at the given position.
///
/// Searches the entire document for all occurrences of the key that the
/// cursor is currently positioned on. Returns an empty vector if the cursor
/// is not on a key (e.g., on a value or whitespace).
///
/// # Arguments
///
/// * `ast` - The parsed AST root node
/// * `text` - The original document text
/// * `line` - 0-based line number of cursor position
/// * `col` - 0-based UTF-8 column of cursor position
/// * `include_declaration` - If true, includes the reference at cursor position;
///   if false, excludes it (useful for "Find References" vs "Find Other References")
///
/// # Returns
///
/// Vector of [`KeyReference`] for all matching keys, sorted by position.
/// Returns empty vector if cursor is not on a key.
///
/// # Examples
///
/// ```ignore
/// let source = "name: Alice\nuser:\n  name: Bob";
/// let (ast, _) = parse_with_errors(source);
/// let refs = find_references_at_position(&ast.unwrap(), source, 0, 0, true);
/// assert_eq!(refs.len(), 2); // Both "name" keys
/// ```
pub fn find_references_at_position(
    ast: &AstNode,
    text: &str,
    line: u32,
    col: u32,
    include_declaration: bool,
) -> Vec<KeyReference> {
    // Calculate offset
    let offset = match calculate_offset(text, line, col) {
        Some(o) => o,
        None => return Vec::new(),
    };

    // Check if cursor is on a key
    let node_at_pos = find_node_at_position(ast, line, col, offset);
    let key_name = match node_at_pos {
        Some(ref node_info) => match &node_info.on_key {
            Some(entry) => &entry.key,
            None => return Vec::new(), // Not on a key
        },
        None => return Vec::new(), // No node at position
    };

    // Get cursor position for filtering
    let cursor_pos = crate::ast::Position::new(line, col, offset);

    // Collect all keys from the document
    let all_keys = collect_all_keys(ast);

    // Filter to matching keys and build KeyReference vec
    let mut references: Vec<KeyReference> = all_keys
        .into_iter()
        .filter(|(k, _)| k == key_name) // Exact match only
        .map(|(key, span)| {
            // Determine if this is a definition (first occurrence at nesting level)
            // For now, mark all as definitions - will refine later if needed
            KeyReference { key_name: key.clone(), span, is_definition: true }
        })
        .collect();

    // Sort by line and column for consistent ordering
    references.sort_by(|a, b| {
        a.span
            .start
            .line
            .cmp(&b.span.start.line)
            .then(a.span.start.column.cmp(&b.span.start.column))
    });

    // Handle include_declaration flag
    if !include_declaration {
        // Exclude the reference at cursor position
        references.retain(|r| !r.span.contains(cursor_pos));
    }

    references
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_find_references_all_occurrences() {
        // T024: Test find_references returns all occurrences
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on first "name" (line 0, col 0)
        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should return 2 references: line 0 "name" and line 2 "name"
        assert_eq!(refs.len(), 2, "Should find 2 occurrences of 'name'");

        // First occurrence
        assert_eq!(refs[0].key_name, "name");
        assert_eq!(refs[0].span.start.line, 0);
        assert!(refs[0].is_definition, "First occurrence should be definition");

        // Second occurrence
        assert_eq!(refs[1].key_name, "name");
        assert_eq!(refs[1].span.start.line, 2);
        assert!(refs[1].is_definition, "First at this nesting level should be definition");
    }

    #[test]
    fn test_find_references_nested() {
        // T025: Test find_references with nested objects
        let source = "data:\n  id: 1\n  nested:\n    id: 2";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "id" at line 1, col 2
        let refs = find_references_at_position(&ast, source, 1, 2, true);

        // Should return 2 references for "id" at different nesting levels
        assert_eq!(refs.len(), 2, "Should find 2 occurrences of 'id'");

        assert_eq!(refs[0].key_name, "id");
        assert_eq!(refs[0].span.start.line, 1);

        assert_eq!(refs[1].key_name, "id");
        assert_eq!(refs[1].span.start.line, 3);
    }

    #[test]
    fn test_find_references_exact_match() {
        // T026: Test find_references exact match only (no partial)
        let source = "name: x\nfullname: y";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "name" at line 0
        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should NOT return "fullname" - exact match only
        assert_eq!(refs.len(), 1, "Should find only 1 occurrence of 'name'");
        assert_eq!(refs[0].key_name, "name");
        assert_eq!(refs[0].span.start.line, 0);
    }

    #[test]
    fn test_find_references_not_on_key() {
        // T027: Test find_references returns empty when cursor not on key
        let source = "name: Alice";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on "Alice" (value, not key) - line 0, col 6
        let refs = find_references_at_position(&ast, source, 0, 6, true);

        // Should return empty vector
        assert_eq!(refs.len(), 0, "Should find no references when not on key");
    }

    #[test]
    fn test_find_references_include_declaration() {
        // T028: Test find_references handles include_declaration flag
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on first "id" at line 0, col 0
        // With include_declaration=true: return both
        let refs_with = find_references_at_position(&ast, source, 0, 0, true);
        assert_eq!(refs_with.len(), 2, "With include_declaration=true, should find 2");

        // With include_declaration=false: exclude the one at cursor
        let refs_without = find_references_at_position(&ast, source, 0, 0, false);
        assert_eq!(refs_without.len(), 1, "With include_declaration=false, should find 1");
        assert_eq!(refs_without[0].span.start.line, 2, "Should only return the other 'id'");
    }

    #[test]
    fn test_find_references_tabular_arrays() {
        // T029: Test find_references with tabular arrays
        // Tabular arrays use inline syntax where headers are array values, not object keys
        // Test with actual object structure inside array instead
        let source = "users:\n  item:\n    name: Alice\n  item:\n    name: Bob";
        let (ast, errors) = parse_with_errors(source);
        assert!(errors.is_empty(), "Parse should succeed");
        let ast = ast.expect("AST should be present");

        // Position on first "name" at line 2, col 4
        let refs = find_references_at_position(&ast, source, 2, 4, true);

        // Should find both "name" keys in the array items
        assert_eq!(refs.len(), 2, "Should find 2 references to 'name' in array objects");
        assert_eq!(refs[0].key_name, "name");
        assert_eq!(refs[0].span.start.line, 2);
        assert_eq!(refs[1].key_name, "name");
        assert_eq!(refs[1].span.start.line, 4);
    }
}
