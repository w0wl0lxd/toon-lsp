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

    collect_hints_recursive(ast, ast, &lines, source_len, &mut hints);

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
    root: &AstNode,
    lines: &[&str],
    source_len: u32,
    hints: &mut Vec<InlayHint>,
) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_hints_recursive(child, root, lines, source_len, hints);
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
                collect_hints_recursive(&entry.value, root, lines, source_len, hints);
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                collect_hints_recursive(item, root, lines, source_len, hints);
            }
        }
        AstNode::Reference { path, is_env, span } => {
            // Check resolved value
            let resolved_val = if *is_env {
                let var_name = path.strip_prefix("env:").unwrap_or(path);
                std::env::var(var_name).ok()
            } else {
                match crate::resolve::resolve(root, path) {
                    Ok(crate::resolve::ResolvedRef::Node { node, .. }) => match node {
                        AstNode::String { value, .. } => Some(value.clone()),
                        AstNode::Number { value, .. } => match value {
                            NumberValue::PosInt(n) => Some(n.to_string()),
                            NumberValue::NegInt(n) => Some(n.to_string()),
                            NumberValue::Float(n) => Some(n.to_string()),
                        },
                        AstNode::Bool { value, .. } => Some(value.to_string()),
                        AstNode::Null { .. } => Some("null".to_string()),
                        _ => None,
                    },
                    Ok(crate::resolve::ResolvedRef::Env(v)) => Some(v),
                    Err(_) => None,
                }
            };

            if let Some(val) = resolved_val {
                let end_line = span.end.line;
                let line = lines.get(end_line as usize).copied().unwrap_or("");
                let end_char = utf8_to_utf16_col(line, span.end.column);

                hints.push(InlayHint {
                    position: Position { line: end_line, character: end_char },
                    label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
                        value: format!(" = {}", val),
                        tooltip: None,
                        location: None,
                        command: None,
                    }]),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(format!(
                        "Resolved: {}",
                        val
                    ))),
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
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

    #[test]
    fn test_inlay_hints_for_reference() {
        let source = "db:\n  port: 5432\nconnection: ${db.port}";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hints = collect_inlay_hints(&ast, source, None);
        let has_hint = hints.iter().any(|h| {
            if let InlayHintLabel::LabelParts(parts) = &h.label {
                parts.iter().any(|p| p.value.contains("5432"))
            } else {
                false
            }
        });
        assert!(has_hint, "Should find inlay hint with evaluated reference value");
    }
}
