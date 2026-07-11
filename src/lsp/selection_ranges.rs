// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Selection range generation for LSP.
//!
//! This module provides functions to generate selection ranges for TOON
//! documents, enabling incremental selection in editors.

use tower_lsp::lsp_types::SelectionRange;

use super::ast_utils::{calculate_offset, find_node_at_position};
use crate::ast::AstNode;

/// Convert an AST to selection ranges for a position.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - The line number
/// * `column` - The column number (UTF-8)
///
/// # Returns
/// A selection range if a node is at the position, None otherwise
pub fn get_selection_range(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Option<SelectionRange> {
    let offset = calculate_offset(source, line, column)?;
    let node_at_pos = find_node_at_position(ast, line, column, offset)?;

    // We build a list of spans from the narrowest to the widest (root)
    let mut spans = Vec::new();

    // 1. Check if the cursor is on the key
    if let Some(entry) = node_at_pos.on_key {
        spans.push(entry.key_span);
        let entry_span = crate::ast::Span::new(entry.key_span.start, entry.value.span().end);
        spans.push(entry_span);
    } else {
        // Otherwise, start with the leaf node
        spans.push(node_at_pos.node.span());
    }

    // 2. Walk up the parent path entries to build the outer scopes
    for entry in node_at_pos.path.iter().rev() {
        spans.push(entry.node.span());
    }

    // Append the root document range if not already included
    let root_span = ast.span();
    if spans.last() != Some(&root_span) {
        spans.push(root_span);
    }

    // Deduplicate adjacent identical spans
    spans.dedup();

    // 3. Construct the linked chain of SelectionRange from narrowest to widest (rev)
    let mut current_range: Option<SelectionRange> = None;
    for span in spans.into_iter().rev() {
        let lines: Vec<&str> = source.lines().collect();
        let start_line = span.start.line as usize;
        let end_line = span.end.line as usize;

        let start_char = super::utf16::utf8_to_utf16_col(
            lines.get(start_line).copied().unwrap_or(""),
            span.start.column,
        );
        let end_char = super::utf16::utf8_to_utf16_col(
            lines.get(end_line).copied().unwrap_or(""),
            span.end.column,
        );

        current_range = Some(SelectionRange {
            range: tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: span.start.line,
                    character: start_char,
                },
                end: tower_lsp::lsp_types::Position { line: span.end.line, character: end_char },
            },
            parent: current_range.map(Box::new),
        });
    }

    current_range
}

/// Collect selection ranges for multiple positions.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `positions` - List of positions to get selection ranges for
///
/// # Returns
/// A vector of optional selection ranges
pub fn get_selection_ranges(
    ast: &AstNode,
    source: &str,
    positions: &[(u32, u32)],
) -> Vec<Option<SelectionRange>> {
    positions
        .iter()
        .map(|(line, column)| get_selection_range(ast, source, *line, *column))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_get_selection_range_hierarchy() {
        let source = "user:\n  name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Cursor on "name" key (line 1, col 2)
        let range = get_selection_range(&ast, source, 1, 2).expect("should find range");

        // Narrowest range: key "name"
        assert_eq!(range.range.start.line, 1);
        assert_eq!(range.range.start.character, 2);
        assert_eq!(range.range.end.line, 1);
        assert_eq!(range.range.end.character, 6);

        // Parent range: the full "name: Alice" entry
        let parent = range.parent.expect("should have parent");
        assert_eq!(parent.range.start.line, 1);
        assert_eq!(parent.range.start.character, 2);
        assert_eq!(parent.range.end.line, 1);
        assert_eq!(parent.range.end.character, 13);
    }
}
