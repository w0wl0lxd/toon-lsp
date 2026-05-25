// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Document highlight generation for LSP.
//!
//! This module provides functions to highlight all occurrences of a key
//! when the cursor is positioned on it, enabling editor highlighting.

use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind};

use super::ast_utils::{calculate_offset, collect_all_keys, find_node_at_position};
use crate::ast::AstNode;

/// Collect document highlights for the key at the given position.
///
/// When the cursor is on a key, returns all occurrences of that key
/// in the document with READ or WRITE highlight kind. The occurrence
/// under the cursor is marked as WRITE, all others as READ.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - 0-based line number
/// * `column` - 0-based UTF-8 column
///
/// # Returns
/// Vector of document highlights, or empty if cursor is not on a key
pub fn collect_document_highlights(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Vec<DocumentHighlight> {
    let offset = match calculate_offset(source, line, column) {
        Some(o) => o,
        None => return vec![],
    };
    let pos = crate::ast::Position::new(line, column, offset);

    // Find the key at cursor position
    let node_at_pos = match find_node_at_position(ast, line, column, offset) {
        Some(n) => n,
        None => return vec![],
    };
    let entry = match node_at_pos.on_key {
        Some(e) => e,
        None => return vec![],
    };

    let key_name = &entry.key;
    let cursor_pos = pos;

    // Collect all keys with matching name
    let all_keys = collect_all_keys(ast);

    let highlights: Vec<DocumentHighlight> = all_keys
        .into_iter()
        .filter(|(k, _)| k == key_name)
        .map(|(_, span)| {
            let is_under_cursor = span.contains(cursor_pos);
            DocumentHighlight {
                range: super::utf16::span_to_range(&span, source),
                kind: if is_under_cursor {
                    Some(DocumentHighlightKind::WRITE)
                } else {
                    Some(DocumentHighlightKind::READ)
                },
            }
        })
        .collect();

    highlights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_highlight_single_occurrence() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let highlights = collect_document_highlights(&ast, source, 0, 0);
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].kind, Some(DocumentHighlightKind::WRITE));
    }

    #[test]
    fn test_highlight_multiple_occurrences() {
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let highlights = collect_document_highlights(&ast, source, 0, 0);
        assert_eq!(highlights.len(), 2);

        // First occurrence (under cursor) should be WRITE
        assert_eq!(highlights[0].kind, Some(DocumentHighlightKind::WRITE));
        // Second occurrence should be READ
        assert_eq!(highlights[1].kind, Some(DocumentHighlightKind::READ));
    }

    #[test]
    fn test_highlight_not_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on value, not key
        let highlights = collect_document_highlights(&ast, source, 0, 7);
        assert!(highlights.is_empty());
    }

    #[test]
    fn test_highlight_nested() {
        let source = "data:\n  id: 1\n  nested:\n    id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "id"
        let highlights = collect_document_highlights(&ast, source, 1, 2);
        assert_eq!(highlights.len(), 2);
    }
}
