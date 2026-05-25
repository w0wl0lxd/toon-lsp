// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Inlay hint generation for LSP.
//!
//! This module provides functions to generate inlay hints for TOON documents,
//! showing type information and sizes for arrays and objects.

use tower_lsp::lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintLabelPart, Position,
};

use super::utf16::utf8_to_utf16_col;
use crate::ast::{AstNode, NumberValue};

/// Collect inlay hints from the AST.
///
/// Generates type/size annotations for objects and arrays. These appear
/// as inline text in the editor to help users understand the structure
/// without needing to expand nested content.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `range` - Optional range to limit hints to
///
/// # Returns
/// Vector of inlay hints
pub fn collect_inlay_hints(
    ast: &AstNode,
    source: &str,
    range: Option<tower_lsp::lsp_types::Range>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let source_len = source.len() as u32;

    collect_hints_recursive(ast, &lines, source_len, &mut hints);

    // Filter hints to the requested range if provided
    if let Some(range) = range {
        hints.retain(|hint| {
            let pos = hint.position;
            if pos.line < range.start.line || pos.line > range.end.line {
                return false;
            }
            if pos.line == range.start.line && pos.character < range.start.character {
                return false;
            }
            if pos.line == range.end.line && pos.character > range.end.character {
                return false;
            }
            true
        });
    }

    hints
}

#[allow(clippy::only_used_in_recursion)]
fn collect_hints_recursive(
    node: &AstNode,
    lines: &[&str],
    source_len: u32,
    hints: &mut Vec<InlayHint>,
) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_hints_recursive(child, lines, source_len, hints);
            }
        }
        AstNode::Object { entries, span } => {
            for entry in entries {
                // Add type hint for values that are objects or arrays
                match &entry.value {
                    AstNode::Object { entries: child_entries, .. } => {
                        let hint = create_type_hint(
                            entry,
                            span,
                            &format!("{} entries", child_entries.len()),
                            lines,
                        );
                        hints.push(hint);
                    }
                    AstNode::Array { items, .. } => {
                        let hint =
                            create_type_hint(entry, span, &format!("{} items", items.len()), lines);
                        hints.push(hint);
                    }
                    AstNode::Number { value, .. } => {
                        if matches!(value, NumberValue::Float(_)) {
                            let hint = create_type_hint(entry, span, "float", lines);
                            hints.push(hint);
                        }
                    }
                    AstNode::Null { .. } => {
                        // Hint that this is null (may be intentional)
                        let hint = create_type_hint(entry, span, "null", lines);
                        hints.push(hint);
                    }
                    _ => {}
                }

                // Recurse into nested structures
                collect_hints_recursive(&entry.value, lines, source_len, hints);
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                collect_hints_recursive(item, lines, source_len, hints);
            }
        }
        _ => {}
    }
}

/// Create a type inlay hint positioned after a key-value pair.
fn create_type_hint(
    entry: &crate::ast::ObjectEntry,
    _object_span: &crate::ast::Span,
    type_text: &str,
    lines: &[&str],
) -> InlayHint {
    // Position the hint at the end of the value
    let value_span = entry.value.span();
    let end_line = value_span.end.line;
    let line = lines.get(end_line as usize).copied().unwrap_or("");

    let end_char = utf8_to_utf16_col(line, value_span.end.column);

    InlayHint {
        position: Position { line: end_line, character: end_char },
        label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
            value: format!("  ─ {} ─", type_text),
            tooltip: None,
            location: None,
            command: None,
        }]),
        kind: Some(InlayHintKind::TYPE),
        text_edits: None,
        tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(format!(
            "Value type: {}",
            type_text
        ))),
        padding_left: Some(true),
        padding_right: Some(false),
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_inlay_hints_for_object() {
        let source = "user:\n  name: Alice\n  age: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hints = collect_inlay_hints(&ast, source, None);
        // Should have at least one hint for the nested object
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_inlay_hints_for_array() {
        let source = "items:\n  - one\n  - two\n  - three";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hints = collect_inlay_hints(&ast, source, None);
        // Should have a hint for the array
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_inlay_hints_empty_document() {
        let source = "";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hints = collect_inlay_hints(&ast, source, None);
        assert!(hints.is_empty());
    }

    #[test]
    fn test_inlay_hints_simple_value() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hints = collect_inlay_hints(&ast, source, None);
        // Simple string value should not produce hints
        assert!(hints.is_empty());
    }
}
