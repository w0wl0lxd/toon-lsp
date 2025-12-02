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

//! Hover information generation for LSP.
//!
//! This module provides functions to generate hover information for TOON
//! document elements including keys, values, and arrays.

use super::ast_utils::{NodePathEntry, find_node_at_position};
use crate::ast::{AstNode, NumberValue, ObjectEntry, Position};

/// Hover information result.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// The hover contents as markdown text
    pub contents: String,
    /// Start line of the hovered element
    pub start_line: u32,
    /// Start column of the hovered element
    pub start_col: u32,
    /// End line of the hovered element
    pub end_line: u32,
    /// End column of the hovered element
    pub end_col: u32,
}

/// Get hover information at a position in the document.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - The line number (0-based)
/// * `column` - The column number (0-based, UTF-8)
///
/// # Returns
/// Hover information if an element is at the position, None otherwise
pub fn get_hover_at_position(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Option<HoverInfo> {
    // Calculate offset from line and column
    let offset = calculate_offset(source, line, column)?;

    // First, try to find a key at this position (check ObjectEntries)
    if let Some(hover) = find_key_hover_at_position(ast, line, column, offset) {
        return Some(hover);
    }

    // Find the node at the position
    let node_at_pos = find_node_at_position(ast, line, column, offset)?;

    // Generate hover content based on the node and path
    let contents = format_hover_content(node_at_pos.node, &node_at_pos.path);

    // Get the span for the hovered node
    let span = node_at_pos.node.span();

    Some(HoverInfo {
        contents,
        start_line: span.start.line,
        start_col: span.start.column,
        end_line: span.end.line,
        end_col: span.end.column,
    })
}

/// Find hover for a key at the given position.
fn find_key_hover_at_position(
    ast: &AstNode,
    line: u32,
    column: u32,
    offset: u32,
) -> Option<HoverInfo> {
    find_key_hover_recursive(ast, line, column, offset, &[])
}

fn find_key_hover_recursive(
    node: &AstNode,
    line: u32,
    column: u32,
    offset: u32,
    path: &[&str],
) -> Option<HoverInfo> {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                if let Some(hover) = find_key_hover_recursive(child, line, column, offset, path) {
                    return Some(hover);
                }
            }
            None
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                if let Some(hover) = check_entry_for_key_hover(entry, line, column, offset, path) {
                    return Some(hover);
                }
            }
            None
        }
        _ => None,
    }
}

fn check_entry_for_key_hover(
    entry: &ObjectEntry,
    line: u32,
    column: u32,
    offset: u32,
    path: &[&str],
) -> Option<HoverInfo> {
    let key_span = &entry.key_span;
    let pos = Position::new(line, column, offset);

    // Check if position is within key span
    if key_span.contains(pos) {
        let key_path = if path.is_empty() {
            entry.key.clone()
        } else {
            format!("{}.{}", path.join("."), entry.key)
        };

        let value_desc = describe_value(&entry.value);
        let contents = format!("**{}** : {}", key_path, value_desc);

        return Some(HoverInfo {
            contents,
            start_line: key_span.start.line,
            start_col: key_span.start.column,
            end_line: key_span.end.line,
            end_col: key_span.end.column,
        });
    }

    // Recursively check nested objects/arrays in value
    let mut new_path: Vec<&str> = path.to_vec();
    new_path.push(&entry.key);

    match &entry.value {
        AstNode::Object { entries, .. } => {
            for child_entry in entries {
                if let Some(hover) =
                    check_entry_for_key_hover(child_entry, line, column, offset, &new_path)
                {
                    return Some(hover);
                }
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                if let Some(hover) = find_key_hover_recursive(item, line, column, offset, &new_path)
                {
                    return Some(hover);
                }
            }
        }
        _ => {}
    }

    None
}

/// Generate a description of a value for hover.
fn describe_value(value: &AstNode) -> String {
    match value {
        AstNode::Object { entries, .. } => format!("Object ({} entries)", entries.len()),
        AstNode::Array { items, .. } => format!("Array ({} items)", items.len()),
        AstNode::String { value, .. } => {
            if value.len() > 30 {
                format!("String \"{}...\"", &value[..27])
            } else {
                format!("String \"{}\"", value)
            }
        }
        AstNode::Number { value, .. } => {
            let num_str = match value {
                NumberValue::PosInt(n) => format!("{}", n),
                NumberValue::NegInt(n) => format!("{}", n),
                NumberValue::Float(n) => format!("{}", n),
            };
            format!("Number {}", num_str)
        }
        AstNode::Bool { value, .. } => format!("Boolean {}", value),
        AstNode::Null { .. } => "Null".to_string(),
        AstNode::Document { .. } => "Document".to_string(),
    }
}

/// Calculate byte offset from line and column.
fn calculate_offset(source: &str, line: u32, column: u32) -> Option<u32> {
    let mut current_line = 0u32;
    let mut offset = 0u32;

    for (idx, ch) in source.char_indices() {
        if current_line == line {
            let col_offset = idx as u32 - offset;
            if col_offset >= column {
                return Some(idx as u32);
            }
        }

        if ch == '\n' {
            if current_line == line {
                // Column is past end of line
                return None;
            }
            current_line += 1;
            offset = idx as u32 + 1;
        }
    }

    // Handle position at end of file or last line
    if current_line == line {
        Some(source.len() as u32)
    } else {
        None
    }
}

/// Format hover content for an AST node.
fn format_hover_content(node: &AstNode, path: &[NodePathEntry<'_>]) -> String {
    // Build the key path
    let key_path = build_key_path(path);

    match node {
        AstNode::Object { entries, .. } => {
            let entry_count = entries.len();
            if key_path.is_empty() {
                format!("**Object** ({} entries)", entry_count)
            } else {
                format!("**{}** : Object ({} entries)", key_path, entry_count)
            }
        }
        AstNode::Array { items, .. } => {
            let item_count = items.len();
            if key_path.is_empty() {
                format!("**Array** ({} items)", item_count)
            } else {
                format!("**{}** : Array ({} items)", key_path, item_count)
            }
        }
        AstNode::String { value, .. } => {
            let preview = if value.len() > 50 {
                format!("\"{}...\"", &value[..47])
            } else {
                format!("\"{}\"", value)
            };
            if key_path.is_empty() {
                format!("**String**: {}", preview)
            } else {
                format!("**{}** : String\n\n{}", key_path, preview)
            }
        }
        AstNode::Number { value, .. } => {
            let num_str = match value {
                NumberValue::PosInt(n) => format!("{}", n),
                NumberValue::NegInt(n) => format!("{}", n),
                NumberValue::Float(n) => format!("{}", n),
            };
            let num_type = match value {
                NumberValue::Float(_) => "Float",
                _ => "Integer",
            };
            if key_path.is_empty() {
                format!("**Number** ({}): {}", num_type, num_str)
            } else {
                format!("**{}** : Number ({})\n\n{}", key_path, num_type, num_str)
            }
        }
        AstNode::Bool { value, .. } => {
            if key_path.is_empty() {
                format!("**Boolean**: {}", value)
            } else {
                format!("**{}** : Boolean\n\n{}", key_path, value)
            }
        }
        AstNode::Null { .. } => {
            if key_path.is_empty() {
                "**Null**".to_string()
            } else {
                format!("**{}** : Null", key_path)
            }
        }
        AstNode::Document { children, .. } => {
            format!("**Document** ({} top-level entries)", children.len())
        }
    }
}

/// Build a dot-separated key path from the node path.
fn build_key_path(path: &[NodePathEntry<'_>]) -> String {
    let keys: Vec<&str> = path.iter().filter_map(|entry| entry.key).collect();
    keys.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_hover_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 0);
        assert!(hover.is_some());
    }

    #[test]
    fn test_hover_on_value() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 6);
        assert!(hover.is_some());
    }

    #[test]
    fn test_calculate_offset() {
        let source = "line1\nline2\nline3";
        assert_eq!(calculate_offset(source, 0, 0), Some(0));
        assert_eq!(calculate_offset(source, 1, 0), Some(6));
        assert_eq!(calculate_offset(source, 2, 0), Some(12));
    }
}
