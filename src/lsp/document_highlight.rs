// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Document highlight generation for LSP.
//!
//! This module provides functions to highlight all occurrences of a key
//! and all its active references in the document when the cursor is positioned on it.

use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind};

use super::ast_utils::{calculate_offset, find_node_at_position};
use crate::ast::AstNode;

/// Collect document highlights for the key or reference at the given position.
///
/// When the cursor is on a key or a reference, returns the definition key span
/// and all reference usage spans in the document.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - 0-based line number
/// * `column` - 0-based UTF-8 column
///
/// # Returns
/// Vector of document highlights
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

    // Find the key/reference at cursor position
    let node_at_pos = match find_node_at_position(ast, line, column, offset) {
        Some(n) => n,
        None => return vec![],
    };

    let target_path = if let Some(entry) = node_at_pos.on_key {
        let parent_path = super::ast_utils::build_key_path(&node_at_pos.path);
        if parent_path.is_empty() {
            entry.key.clone()
        } else {
            format!("{}.{}", parent_path, entry.key)
        }
    } else if let AstNode::Reference { path, .. } = node_at_pos.node {
        path.clone()
    } else {
        return vec![];
    };

    let mut highlights = Vec::new();

    // 1. Find definition site of target_path (if it's not an env var)
    if !target_path.starts_with("env:")
        && let Ok(crate::resolve::ResolvedRef::Node { key_span: Some(span), .. }) =
            crate::resolve::resolve(ast, &target_path)
    {
        let is_under_cursor = span.contains(pos);
        highlights.push(DocumentHighlight {
            range: super::utf16::span_to_range(&span, source),
            kind: if is_under_cursor {
                Some(DocumentHighlightKind::WRITE)
            } else {
                Some(DocumentHighlightKind::READ)
            },
        });
    }

    // 2. Find all references in the document matching target_path
    let mut refs = Vec::new();
    crate::resolve::collect_references(ast, &mut refs);
    for r in refs {
        if let AstNode::Reference { path, span, .. } = r
            && path == &target_path
        {
            let is_under_cursor = span.contains(pos);
            highlights.push(DocumentHighlight {
                range: super::utf16::span_to_range(span, source),
                kind: if is_under_cursor {
                    Some(DocumentHighlightKind::WRITE)
                } else {
                    Some(DocumentHighlightKind::READ)
                },
            });
        }
    }

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
    fn test_highlight_distinct_paths() {
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hovering "name" on line 0 should only highlight "name" (path: name)
        let highlights = collect_document_highlights(&ast, source, 0, 0);
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].kind, Some(DocumentHighlightKind::WRITE));
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
    fn test_highlight_reference_usages() {
        let source = "db:\n  port: 5432\nconnection: ${db.port}";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on key "port"
        let highlights = collect_document_highlights(&ast, source, 1, 2);
        assert_eq!(highlights.len(), 2); // The key definition and the reference usage
    }
}
