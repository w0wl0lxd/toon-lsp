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

//! Document symbol generation for LSP.
//!
//! This module provides functions to convert TOON AST nodes to LSP document symbols
//! for editor outline views.

use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};

use super::utf16::span_to_range;
use crate::ast::AstNode;

/// Convert an AST to document symbols for the outline view.
///
/// # Arguments
/// * `ast` - The root AST node (typically a Document)
/// * `source` - The document source text (for UTF-16 conversion)
///
/// # Returns
/// A vector of document symbols representing the document structure
#[allow(deprecated)] // DocumentSymbol::deprecated field
pub fn ast_to_document_symbols(ast: &AstNode, source: &str) -> Vec<DocumentSymbol> {
    match ast {
        AstNode::Document { children, .. } => {
            // Document root: process all children
            children
                .iter()
                .flat_map(|child| node_to_symbols(child, source))
                .collect()
        }
        AstNode::Object { entries, .. } => {
            // Object at root level: process entries
            entries
                .iter()
                .map(|entry| entry_to_symbol(entry, source))
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Convert an AST node to document symbols.
fn node_to_symbols(node: &AstNode, source: &str) -> Vec<DocumentSymbol> {
    match node {
        AstNode::Object { entries, .. } => entries
            .iter()
            .map(|entry| entry_to_symbol(entry, source))
            .collect(),
        _ => Vec::new(),
    }
}

/// Convert an object entry to a document symbol.
#[allow(deprecated)] // DocumentSymbol::deprecated field
fn entry_to_symbol(entry: &crate::ast::ObjectEntry, source: &str) -> DocumentSymbol {
    let key_range = span_to_range(&entry.key_span, source);
    let value_range = span_to_range(&entry.value.span(), source);

    // Full range includes key and value
    let range = tower_lsp::lsp_types::Range {
        start: key_range.start,
        end: value_range.end,
    };

    // Determine symbol kind and children based on value type
    let (kind, children) = match &entry.value {
        AstNode::Object { entries, .. } => {
            let child_symbols: Vec<DocumentSymbol> =
                entries.iter().map(|e| entry_to_symbol(e, source)).collect();
            let children = if child_symbols.is_empty() {
                None
            } else {
                Some(child_symbols)
            };
            (SymbolKind::OBJECT, children)
        }
        AstNode::Array { items, .. } => {
            let child_symbols: Vec<DocumentSymbol> = items
                .iter()
                .enumerate()
                .flat_map(|(idx, item)| array_item_to_symbols(item, idx, source))
                .collect();
            let children = if child_symbols.is_empty() {
                None
            } else {
                Some(child_symbols)
            };
            (SymbolKind::ARRAY, children)
        }
        AstNode::String { .. } => (SymbolKind::STRING, None),
        AstNode::Number { .. } => (SymbolKind::NUMBER, None),
        AstNode::Bool { .. } => (SymbolKind::BOOLEAN, None),
        AstNode::Null { .. } => (SymbolKind::NULL, None),
        AstNode::Document { .. } => (SymbolKind::OBJECT, None),
    };

    // For simple values, use KEY kind to indicate it's a key-value pair
    let kind = match &entry.value {
        AstNode::Object { .. } | AstNode::Array { .. } => kind,
        _ => SymbolKind::KEY,
    };

    DocumentSymbol {
        name: entry.key.clone(),
        detail: Some(value_detail(&entry.value)),
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: key_range,
        children,
    }
}

/// Convert an array item to document symbols.
#[allow(deprecated)] // DocumentSymbol::deprecated field
fn array_item_to_symbols(item: &AstNode, index: usize, source: &str) -> Vec<DocumentSymbol> {
    match item {
        AstNode::Object { entries, span } => {
            // For objects in arrays, create a container symbol
            let range = span_to_range(span, source);
            let child_symbols: Vec<DocumentSymbol> =
                entries.iter().map(|e| entry_to_symbol(e, source)).collect();

            vec![DocumentSymbol {
                name: format!("[{}]", index),
                detail: Some(format!("object with {} entries", entries.len())),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if child_symbols.is_empty() {
                    None
                } else {
                    Some(child_symbols)
                },
            }]
        }
        AstNode::Array { items, span, .. } => {
            let range = span_to_range(span, source);
            vec![DocumentSymbol {
                name: format!("[{}]", index),
                detail: Some(format!("array with {} items", items.len())),
                kind: SymbolKind::ARRAY,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            }]
        }
        _ => {
            // Simple values in arrays don't need individual symbols
            Vec::new()
        }
    }
}

/// Generate detail string for a value.
fn value_detail(value: &AstNode) -> String {
    use crate::ast::NumberValue;

    match value {
        AstNode::String { value, .. } => {
            if value.len() > 30 {
                format!("\"{}...\"", &value[..27])
            } else {
                format!("\"{}\"", value)
            }
        }
        AstNode::Number { value, .. } => match value {
            NumberValue::PosInt(n) => n.to_string(),
            NumberValue::NegInt(n) => n.to_string(),
            NumberValue::Float(n) => n.to_string(),
        },
        AstNode::Bool { value, .. } => value.to_string(),
        AstNode::Null { .. } => "null".to_string(),
        AstNode::Object { entries, .. } => format!("object ({} entries)", entries.len()),
        AstNode::Array { items, .. } => format!("array ({} items)", items.len()),
        AstNode::Document { children, .. } => format!("document ({} children)", children.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_simple_document_symbols() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "key");
    }

    #[test]
    fn test_nested_document_symbols() {
        let source = "parent:\n  child: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "parent");
        assert!(symbols[0].children.is_some());
    }
}
