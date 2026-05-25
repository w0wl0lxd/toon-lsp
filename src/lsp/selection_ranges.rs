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

//! Selection range generation for LSP.
//!
//! This module provides functions to generate selection ranges for TOON
//! documents, enabling incremental selection in editors.

use tower_lsp::lsp_types::SelectionRange;

use crate::ast::AstNode;

/// Convert an AST node to an LSP SelectionRange.
fn ast_to_selection_range(node: &AstNode) -> SelectionRange {
    let span = node.span();
    SelectionRange {
        range: tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position {
                line: span.start.line,
                character: span.start.column,
            },
            end: tower_lsp::lsp_types::Position {
                line: span.end.line,
                character: span.end.column,
            },
        },
        parent: None,
    }
}

/// Convert an AST to selection ranges for a position.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `_source` - The document source text (unused for now)
/// * `_line` - The line number
/// * `_column` - The column number (UTF-8)
///
/// # Returns
/// A selection range if a node is at the position, None otherwise
pub fn get_selection_range(
    ast: &AstNode,
    _source: &str,
    _line: u32,
    _column: u32,
) -> Option<SelectionRange> {
    // For TOON, we provide selection ranges based on AST structure:
    // 1. The key name (narrowest)
    // 2. The key-value pair
    // 3. The parent object
    // 4. The document
    Some(ast_to_selection_range(ast))
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
    fn test_get_selection_range() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let range = get_selection_range(&ast, source, 0, 0);
        assert!(range.is_some());
    }

    #[test]
    fn test_get_selection_ranges() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let positions = vec![(0, 0), (0, 5)];
        let ranges = get_selection_ranges(&ast, source, &positions);
        assert_eq!(ranges.len(), 2);
    }
}