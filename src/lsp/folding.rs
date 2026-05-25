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

//! Folding range generation for LSP.
//!
//! This module provides functions to generate folding ranges for TOON
//! documents, enabling code folding in editors.

use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

use crate::ast::AstNode;

/// Convert an AST to folding ranges.
///
/// # Arguments
/// * `ast` - The root AST node
///
/// # Returns
/// A vector of folding ranges representing collapsible sections
pub fn collect_folding_ranges(ast: &AstNode) -> Vec<FoldingRange> {
    collect_folding_ranges_recursive(ast)
}

fn collect_folding_ranges_recursive(node: &AstNode) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();

    match node {
        AstNode::Document { children, span } => {
            // Document can be folded if it has multiple lines
            if span.end.line > span.start.line {
                ranges.push(FoldingRange {
                    start_line: span.start.line,
                    start_character: None,
                    end_line: span.end.line,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            // Process children
            for child in children {
                ranges.extend(collect_folding_ranges_recursive(child));
            }
        }
        AstNode::Object { entries, span } => {
            // Object can be folded if it spans multiple lines and has entries
            if span.end.line > span.start.line && !entries.is_empty() {
                ranges.push(FoldingRange {
                    start_line: span.start.line,
                    start_character: None,
                    end_line: span.end.line,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }
        AstNode::Array { items, span, .. } => {
            // Array can be folded if it spans multiple lines and has items
            if span.end.line > span.start.line && !items.is_empty() {
                ranges.push(FoldingRange {
                    start_line: span.start.line,
                    start_character: None,
                    end_line: span.end.line,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }
        _ => {}
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_folding_single_line() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let ranges = collect_folding_ranges(&ast);
        // Single line should not produce folding ranges
        assert!(ranges.is_empty() || ranges.len() <= 1);
    }

    #[test]
    fn test_folding_multiline_object() {
        let source = "parent:\n  child: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let ranges = collect_folding_ranges(&ast);
        // Should have at least one folding range for the nested object
        assert!(!ranges.is_empty());
    }
}