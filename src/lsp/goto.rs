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

//! Go-to-definition functionality for LSP.
//!
//! This module provides functions to find all definitions of a key,
//! supporting navigation between duplicate keys.

use super::ast_utils::calculate_offset;
use crate::ast::{AstNode, ObjectEntry, Position, Span};

/// A location result for go-to-definition.
#[derive(Debug, Clone)]
pub struct DefinitionLocation {
    /// Line number (0-based)
    pub line: u32,
    /// Start column (0-based, UTF-8)
    pub start_col: u32,
    /// End column (0-based, UTF-8)
    pub end_col: u32,
}

impl DefinitionLocation {
    /// Create a new definition location from a span.
    pub fn from_span(span: &Span) -> Self {
        Self { line: span.start.line, start_col: span.start.column, end_col: span.end.column }
    }
}

/// Get all definition locations for a key at a position.
///
/// If the position is on a key, returns all locations where that key
/// is defined within the same object scope.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - The line number (0-based)
/// * `column` - The column number (0-based, UTF-8)
///
/// # Returns
/// Vector of definition locations, empty if position is not on a key
pub fn get_definition_at_position(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Vec<DefinitionLocation> {
    // Calculate offset
    let offset = match calculate_offset(source, line, column) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let pos = Position::new(line, column, offset);

    // Find the key at this position and its containing object
    find_key_and_definitions(ast, pos)
}

/// Find a key at position and return all definitions in its scope.
fn find_key_and_definitions(ast: &AstNode, pos: Position) -> Vec<DefinitionLocation> {
    match ast {
        AstNode::Document { children, .. } => {
            for child in children {
                let results = find_key_and_definitions(child, pos);
                if !results.is_empty() {
                    return results;
                }
            }
            Vec::new()
        }
        AstNode::Object { entries, .. } => {
            // First, check if position is on any key in this object
            if let Some(key_name) = find_key_at_position(entries, pos) {
                // Return all definitions of this key within this object
                return find_all_key_definitions(entries, &key_name);
            }

            // Otherwise, recurse into entry values
            for entry in entries {
                let results = find_key_and_definitions(&entry.value, pos);
                if !results.is_empty() {
                    return results;
                }
            }
            Vec::new()
        }
        AstNode::Array { items, .. } => {
            for item in items {
                let results = find_key_and_definitions(item, pos);
                if !results.is_empty() {
                    return results;
                }
            }
            Vec::new()
        }
        // Leaf nodes don't have key definitions
        _ => Vec::new(),
    }
}

/// Find which key (if any) the position is on.
fn find_key_at_position(entries: &[ObjectEntry], pos: Position) -> Option<String> {
    for entry in entries {
        if entry.key_span.contains(pos) {
            return Some(entry.key.clone());
        }
    }
    None
}

/// Find all definitions of a key within an object.
fn find_all_key_definitions(entries: &[ObjectEntry], key_name: &str) -> Vec<DefinitionLocation> {
    entries
        .iter()
        .filter(|e| e.key == key_name)
        .map(|e| DefinitionLocation::from_span(&e.key_span))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_definition_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let locations = get_definition_at_position(&ast, source, 0, 2);
        assert_eq!(locations.len(), 1);
    }

    #[test]
    fn test_definition_on_value_empty() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let locations = get_definition_at_position(&ast, source, 0, 8);
        assert!(locations.is_empty());
    }
}
