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

//! Workspace symbol generation for LSP.
//!
//! This module provides functions to search for symbols across the workspace.

use tower_lsp::lsp_types::{Location, OneOf, Position, Range, SymbolKind, Url, WorkspaceSymbol};

use crate::ast::AstNode;

/// Collect all symbols from an AST for workspace-wide search.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `uri` - The document URI for symbol location
///
/// # Returns
/// A vector of workspace symbols
pub fn collect_workspace_symbols(ast: &AstNode, uri: &Url) -> Vec<WorkspaceSymbol> {
    collect_symbols_recursive(ast, uri, "")
}

fn collect_symbols_recursive(node: &AstNode, uri: &Url, prefix: &str) -> Vec<WorkspaceSymbol> {
    let mut symbols = Vec::new();

    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                symbols.extend(collect_symbols_recursive(child, uri, prefix));
            }
        }
        AstNode::Object { entries, span } => {
            for entry in entries {
                let key_path = if prefix.is_empty() {
                    entry.key.clone()
                } else {
                    format!("{}.{}", prefix, entry.key)
                };

                let full_range = Range {
                    start: Position { line: entry.key_span.start.line, character: 0 },
                    end: Position { line: span.end.line, character: 0 },
                };

                let symbol = WorkspaceSymbol {
                    name: entry.key.clone(),
                    kind: SymbolKind::KEY,
                    location: OneOf::Left(Location { uri: uri.clone(), range: full_range }),
                    container_name: if prefix.is_empty() { None } else { Some(prefix.to_string()) },
                    tags: None,
                    data: None,
                };
                symbols.push(symbol);

                // Recursively process nested objects/arrays
                match &entry.value {
                    AstNode::Object { .. } => {
                        let child_symbols = collect_symbols_recursive(&entry.value, uri, &key_path);
                        symbols.extend(child_symbols);
                    }
                    AstNode::Array { items, .. } => {
                        for (idx, item) in items.iter().enumerate() {
                            if let AstNode::Object { .. } = item {
                                let array_prefix = format!("{}[{}]", key_path, idx);
                                let child_symbols =
                                    collect_symbols_recursive(item, uri, &array_prefix);
                                symbols.extend(child_symbols);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_workspace_symbols() {
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let uri: Url = "file:///test.toon".parse().unwrap();
        let symbols = collect_workspace_symbols(&ast, &uri);

        assert_eq!(symbols.len(), 2);
    }
}
