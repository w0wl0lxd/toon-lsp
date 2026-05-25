// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Linked editing range generation for LSP.
//!
//! This module provides functions to generate linked editing ranges for
//! simultaneous renaming of keys that appear multiple times in a document.

use tower_lsp::lsp_types::{LinkedEditingRanges, Range};

use super::ast_utils::{calculate_offset, collect_all_keys, find_node_at_position};
use super::utf16::span_to_range;
use crate::ast::AstNode;

/// Collect linked editing ranges for the key at the given position.
///
/// When the cursor is on a key that appears multiple times in the document,
/// returns all occurrences as linked editing ranges. When the user edits
/// one occurrence, all others update simultaneously.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - 0-based line number
/// * `column` - 0-based UTF-8 column
///
/// # Returns
/// Linked editing ranges if cursor is on a key with multiple occurrences
pub fn collect_linked_editing_ranges(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Option<LinkedEditingRanges> {
    let offset = calculate_offset(source, line, column)?;

    // Find the key at cursor position
    let node_at_pos = find_node_at_position(ast, line, column, offset)?;
    let entry = node_at_pos.on_key?;

    let key_name = &entry.key;

    // Collect all keys with matching name
    let all_keys = collect_all_keys(ast);
    let matching_spans: Vec<Range> = all_keys
        .into_iter()
        .filter(|(k, _)| k == key_name)
        .map(|(_, span)| span_to_range(&span, source))
        .collect();

    if matching_spans.len() <= 1 {
        // Only one occurrence - no need for linked editing
        return None;
    }

    // Find the word pattern for the key (use word boundaries to match indented keys)
    let word_pattern = Some(format!("\\b{}\\b", regex_escape(key_name)));

    Some(LinkedEditingRanges { ranges: matching_spans, word_pattern })
}

/// Escape a string for regex use.
fn regex_escape(s: &str) -> String {
    let special_chars = r"\^$.|?*+()[]{}";
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        if special_chars.contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_linked_editing_single_occurrence() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Single occurrence should return None
        let ranges = collect_linked_editing_ranges(&ast, source, 0, 0);
        assert!(ranges.is_none());
    }

    #[test]
    fn test_linked_editing_multiple_occurrences() {
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let ranges = collect_linked_editing_ranges(&ast, source, 0, 0);
        assert!(ranges.is_some());

        let linked = ranges.unwrap();
        assert_eq!(linked.ranges.len(), 2);

        // Should have a word pattern for the key
        assert!(linked.word_pattern.is_some());
    }

    #[test]
    fn test_linked_editing_not_on_key() {
        let source = "name: Alice\nname: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on a value, not a key
        let ranges = collect_linked_editing_ranges(&ast, source, 0, 7);
        assert!(ranges.is_none());
    }

    #[test]
    fn test_linked_editing_nested() {
        let source = "data:\n  id: 1\n  nested:\n    id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "id"
        let ranges = collect_linked_editing_ranges(&ast, source, 1, 2);
        assert!(ranges.is_some());

        let linked = ranges.unwrap();
        assert_eq!(linked.ranges.len(), 2);
    }

    #[test]
    fn test_linked_editing_three_occurrences() {
        let source = "id: 1\ndata:\n  id: 2\nmore:\n  id: 3";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let ranges = collect_linked_editing_ranges(&ast, source, 0, 0);
        assert!(ranges.is_some());

        let linked = ranges.unwrap();
        assert_eq!(linked.ranges.len(), 3);
    }
}
