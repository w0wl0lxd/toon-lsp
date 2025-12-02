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

//! Symbol extraction for TOON documents.
//!
//! Extracts document symbols (keys) in various output formats (tree, JSON, flat).

use serde::{Deserialize, Serialize};

use super::error::{CliError, CliResult, ExitCode};
use super::io_utils::{read_input, write_output};
use super::{SymbolsArgs, SymbolsFormat};
use crate::ast::AstNode;
use crate::parser;

/// A symbol extracted from a TOON document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name (key)
    pub name: String,
    /// Symbol kind (object, array, string, etc.)
    pub kind: SymbolKind,
    /// Dot-notation path from document root
    pub path: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Child symbols for objects and arrays
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Symbol>,
}

/// Symbol kind classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    /// Object/table
    Object,
    /// Array
    Array,
    /// String value
    String,
    /// Number value
    Number,
    /// Boolean value
    Boolean,
    /// Null value
    Null,
}

impl SymbolKind {
    /// Get the string representation for display.
    fn as_str(self) -> &'static str {
        match self {
            SymbolKind::Object => "object",
            SymbolKind::Array => "array",
            SymbolKind::String => "string",
            SymbolKind::Number => "number",
            SymbolKind::Boolean => "boolean",
            SymbolKind::Null => "null",
        }
    }
}

/// Execute the symbols command.
///
/// Extracts document symbols from TOON input and outputs in the specified format.
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(CliError::Parse(...))` if input cannot be parsed
/// - `Err(CliError::Io(...))` if I/O fails
pub fn execute(args: &SymbolsArgs) -> CliResult<()> {
    // Read input from file or stdin
    let content = read_input(&args.input)?;

    // Parse TOON content
    let (ast, errors) = parser::parse_with_errors(&content);

    // Report parse errors but continue with partial AST
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Parse error: {error}");
        }
    }

    // Extract symbols from AST
    let symbols = if let Some(ast) = ast {
        extract_symbols(&ast, "")
    } else {
        // No AST - return empty symbols
        Vec::new()
    };

    // Format output based on requested format
    let output = match args.format {
        SymbolsFormat::Tree => format_tree(&symbols, args, 0),
        SymbolsFormat::Json => format_json(&symbols, args),
        SymbolsFormat::Flat => format_flat(&symbols, args),
    };

    // Write output to stdout or file
    write_output(&None, &output)?;

    Ok(())
}

/// Extract symbols from an AST node recursively.
///
/// # Arguments
///
/// - `node`: AST node to extract symbols from
/// - `parent_path`: Dot-notation path of parent (empty for root)
///
/// # Returns
///
/// Vector of extracted symbols with hierarchy preserved
fn extract_symbols(node: &AstNode, parent_path: &str) -> Vec<Symbol> {
    match node {
        AstNode::Document { children, .. } => {
            // Document node - extract from all children
            children.iter().flat_map(|child| extract_symbols(child, parent_path)).collect()
        }
        AstNode::Object { entries, .. } => {
            // Object node - extract symbols from entries
            entries
                .iter()
                .map(|entry| {
                    let key = &entry.key;
                    let path = if parent_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{parent_path}.{key}")
                    };

                    // Get position from key span (convert from 0-based to 1-based)
                    let span = &entry.key_span;
                    let line = (span.start.line + 1) as usize;
                    let column = (span.start.column + 1) as usize;

                    // Determine kind and extract children from value
                    let (kind, children) = match &entry.value {
                        AstNode::Object { .. } => {
                            (SymbolKind::Object, extract_symbols(&entry.value, &path))
                        }
                        AstNode::Array { .. } => {
                            (SymbolKind::Array, extract_symbols(&entry.value, &path))
                        }
                        AstNode::String { .. } => (SymbolKind::String, Vec::new()),
                        AstNode::Number { .. } => (SymbolKind::Number, Vec::new()),
                        AstNode::Bool { .. } => (SymbolKind::Boolean, Vec::new()),
                        AstNode::Null { .. } => (SymbolKind::Null, Vec::new()),
                        AstNode::Document { .. } => {
                            (SymbolKind::Object, extract_symbols(&entry.value, &path))
                        }
                    };

                    Symbol { name: key.clone(), kind, path, line, column, children }
                })
                .collect()
        }
        AstNode::Array { items, .. } => {
            // Array items - extract from object items only (arrays don't have named keys)
            items
                .iter()
                .filter_map(|item| {
                    if matches!(item, AstNode::Object { .. }) {
                        // Array of objects - extract their symbols
                        Some(extract_symbols(item, parent_path))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect()
        }
        _ => Vec::new(), // Leaf values have no symbols
    }
}

/// Format symbols as an indented tree structure.
///
/// # Arguments
///
/// - `symbols`: Symbols to format
/// - `args`: Command arguments for display options
/// - `depth`: Current indentation depth
///
/// # Returns
///
/// Formatted tree string
fn format_tree(symbols: &[Symbol], args: &SymbolsArgs, depth: usize) -> String {
    let mut output = String::new();
    let indent = "  ".repeat(depth);

    for symbol in symbols {
        let mut line = format!("{indent}{}", symbol.name);

        // Add type annotation if requested
        if args.types {
            use std::fmt::Write;
            let _ = write!(line, " [{}]", symbol.kind.as_str());
        }

        // Add position if requested
        if args.positions {
            use std::fmt::Write;
            let _ = write!(line, "  (L{}:C{})", symbol.line, symbol.column);
        }

        output.push_str(&line);
        output.push('\n');

        // Recursively format children
        if !symbol.children.is_empty() {
            output.push_str(&format_tree(&symbol.children, args, depth + 1));
        }
    }

    output
}

/// Format symbols as JSON.
///
/// Preserves the hierarchical tree structure in the JSON output.
/// Children are nested under their parent symbols.
///
/// # Arguments
///
/// - `symbols`: Symbols to format (with hierarchy preserved)
/// - `args`: Command arguments (unused, for API consistency)
///
/// # Returns
///
/// JSON-formatted string with tree structure
fn format_json(symbols: &[Symbol], _args: &SymbolsArgs) -> String {
    // Preserve tree structure in JSON output
    serde_json::to_string_pretty(symbols).unwrap_or_else(|e| {
        eprintln!("JSON serialization error: {e}");
        "[]".to_string()
    })
}

/// Format symbols as flat dot-notation paths.
///
/// # Arguments
///
/// - `symbols`: Symbols to format
/// - `args`: Command arguments for display options
///
/// # Returns
///
/// Flat list with one symbol per line
fn format_flat(symbols: &[Symbol], args: &SymbolsArgs) -> String {
    let flat_symbols = flatten_symbols(symbols);
    let mut output = String::new();

    for symbol in flat_symbols {
        let mut line = symbol.path.clone();

        // Add type annotation if requested
        if args.types {
            use std::fmt::Write;
            let _ = write!(line, " [{}]", symbol.kind.as_str());
        }

        // Add position if requested
        if args.positions {
            use std::fmt::Write;
            let _ = write!(line, "  (L{}:C{})", symbol.line, symbol.column);
        }

        output.push_str(&line);
        output.push('\n');
    }

    output
}

/// Flatten a hierarchical symbol tree into a flat list.
///
/// # Arguments
///
/// - `symbols`: Hierarchical symbols to flatten
///
/// # Returns
///
/// Flat vector of all symbols (parents before children)
fn flatten_symbols(symbols: &[Symbol]) -> Vec<Symbol> {
    let mut result = Vec::new();

    for symbol in symbols {
        // Add the symbol itself (without children to avoid duplication in flat output)
        result.push(Symbol {
            name: symbol.name.clone(),
            kind: symbol.kind,
            path: symbol.path.clone(),
            line: symbol.line,
            column: symbol.column,
            children: Vec::new(),
        });

        // Recursively flatten children
        if !symbol.children.is_empty() {
            result.extend(flatten_symbols(&symbol.children));
        }
    }

    result
}

/// Compute exit code for errors.
///
/// # Arguments
///
/// - `error`: The CLI error
///
/// # Returns
///
/// Appropriate exit code
pub fn error_exit_code(error: &CliError) -> ExitCode {
    match error {
        CliError::Parse(_) => ExitCode::ValidationFailed,
        CliError::Io(_) | _ => ExitCode::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{NumberValue, ObjectEntry, Position, Span};

    /// Create a test span at a given line/column.
    fn test_span(line: u32, column: u32) -> Span {
        let start = Position::new(line, column, line * 1000 + column);
        let end = Position::new(line, column + 5, line * 1000 + column + 5);
        Span::new(start, end)
    }

    #[test]
    fn test_extract_symbols_from_object() {
        let ast = AstNode::Object {
            entries: vec![
                ObjectEntry {
                    key: "name".to_string(),
                    key_span: test_span(0, 0),
                    value: AstNode::String { value: "Alice".to_string(), span: test_span(0, 5) },
                },
                ObjectEntry {
                    key: "age".to_string(),
                    key_span: test_span(1, 0),
                    value: AstNode::Number {
                        value: NumberValue::PosInt(30),
                        span: test_span(1, 4),
                    },
                },
            ],
            span: test_span(0, 0),
        };

        let symbols = extract_symbols(&ast, "");
        assert_eq!(symbols.len(), 2);

        assert_eq!(symbols[0].name, "name");
        assert_eq!(symbols[0].kind, SymbolKind::String);
        assert_eq!(symbols[0].path, "name");
        assert_eq!(symbols[0].line, 1); // 1-based
        assert_eq!(symbols[0].column, 1); // 1-based

        assert_eq!(symbols[1].name, "age");
        assert_eq!(symbols[1].kind, SymbolKind::Number);
        assert_eq!(symbols[1].path, "age");
    }

    #[test]
    fn test_extract_symbols_nested_object() {
        let ast = AstNode::Object {
            entries: vec![ObjectEntry {
                key: "server".to_string(),
                key_span: test_span(0, 0),
                value: AstNode::Object {
                    entries: vec![
                        ObjectEntry {
                            key: "host".to_string(),
                            key_span: test_span(1, 2),
                            value: AstNode::String {
                                value: "localhost".to_string(),
                                span: test_span(1, 7),
                            },
                        },
                        ObjectEntry {
                            key: "port".to_string(),
                            key_span: test_span(2, 2),
                            value: AstNode::Number {
                                value: NumberValue::PosInt(8080),
                                span: test_span(2, 7),
                            },
                        },
                    ],
                    span: test_span(1, 0),
                },
            }],
            span: test_span(0, 0),
        };

        let symbols = extract_symbols(&ast, "");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "server");
        assert_eq!(symbols[0].kind, SymbolKind::Object);
        assert_eq!(symbols[0].children.len(), 2);

        assert_eq!(symbols[0].children[0].name, "host");
        assert_eq!(symbols[0].children[0].path, "server.host");
        assert_eq!(symbols[0].children[0].line, 2); // 1-based

        assert_eq!(symbols[0].children[1].name, "port");
        assert_eq!(symbols[0].children[1].path, "server.port");
    }

    #[test]
    fn test_extract_symbols_array() {
        let ast = AstNode::Object {
            entries: vec![ObjectEntry {
                key: "items".to_string(),
                key_span: test_span(0, 0),
                value: AstNode::Array {
                    items: vec![
                        AstNode::String { value: "a".to_string(), span: test_span(1, 2) },
                        AstNode::String { value: "b".to_string(), span: test_span(1, 6) },
                    ],
                    form: crate::ast::ArrayForm::Expanded,
                    span: test_span(1, 0),
                },
            }],
            span: test_span(0, 0),
        };

        let symbols = extract_symbols(&ast, "");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "items");
        assert_eq!(symbols[0].kind, SymbolKind::Array);
        assert_eq!(symbols[0].children.is_empty(), true); // String items have no symbols
    }

    #[test]
    fn test_format_tree_basic() {
        let symbols = vec![
            Symbol {
                name: "server".to_string(),
                kind: SymbolKind::Object,
                path: "server".to_string(),
                line: 1,
                column: 1,
                children: vec![Symbol {
                    name: "host".to_string(),
                    kind: SymbolKind::String,
                    path: "server.host".to_string(),
                    line: 2,
                    column: 3,
                    children: Vec::new(),
                }],
            },
            Symbol {
                name: "port".to_string(),
                kind: SymbolKind::Number,
                path: "port".to_string(),
                line: 3,
                column: 1,
                children: Vec::new(),
            },
        ];

        let args = SymbolsArgs {
            input: None,
            format: SymbolsFormat::Tree,
            types: false,
            positions: false,
        };

        let output = format_tree(&symbols, &args, 0);
        assert_eq!(output, "server\n  host\nport\n");
    }

    #[test]
    fn test_format_tree_with_types() {
        let symbols = vec![Symbol {
            name: "name".to_string(),
            kind: SymbolKind::String,
            path: "name".to_string(),
            line: 1,
            column: 1,
            children: Vec::new(),
        }];

        let args =
            SymbolsArgs { input: None, format: SymbolsFormat::Tree, types: true, positions: false };

        let output = format_tree(&symbols, &args, 0);
        assert_eq!(output, "name [string]\n");
    }

    #[test]
    fn test_format_tree_with_positions() {
        let symbols = vec![Symbol {
            name: "age".to_string(),
            kind: SymbolKind::Number,
            path: "age".to_string(),
            line: 5,
            column: 10,
            children: Vec::new(),
        }];

        let args =
            SymbolsArgs { input: None, format: SymbolsFormat::Tree, types: false, positions: true };

        let output = format_tree(&symbols, &args, 0);
        assert_eq!(output, "age  (L5:C10)\n");
    }

    #[test]
    fn test_format_tree_with_types_and_positions() {
        let symbols = vec![Symbol {
            name: "enabled".to_string(),
            kind: SymbolKind::Boolean,
            path: "enabled".to_string(),
            line: 7,
            column: 3,
            children: Vec::new(),
        }];

        let args =
            SymbolsArgs { input: None, format: SymbolsFormat::Tree, types: true, positions: true };

        let output = format_tree(&symbols, &args, 0);
        assert_eq!(output, "enabled [boolean]  (L7:C3)\n");
    }

    #[test]
    fn test_format_flat_basic() {
        let symbols = vec![
            Symbol {
                name: "server".to_string(),
                kind: SymbolKind::Object,
                path: "server".to_string(),
                line: 1,
                column: 1,
                children: vec![Symbol {
                    name: "host".to_string(),
                    kind: SymbolKind::String,
                    path: "server.host".to_string(),
                    line: 2,
                    column: 3,
                    children: Vec::new(),
                }],
            },
            Symbol {
                name: "port".to_string(),
                kind: SymbolKind::Number,
                path: "port".to_string(),
                line: 3,
                column: 1,
                children: Vec::new(),
            },
        ];

        let args = SymbolsArgs {
            input: None,
            format: SymbolsFormat::Flat,
            types: false,
            positions: false,
        };

        let output = format_flat(&symbols, &args);
        assert_eq!(output, "server\nserver.host\nport\n");
    }

    #[test]
    fn test_format_flat_with_types() {
        let symbols = vec![Symbol {
            name: "config".to_string(),
            kind: SymbolKind::Object,
            path: "config".to_string(),
            line: 1,
            column: 1,
            children: Vec::new(),
        }];

        let args =
            SymbolsArgs { input: None, format: SymbolsFormat::Flat, types: true, positions: false };

        let output = format_flat(&symbols, &args);
        assert_eq!(output, "config [object]\n");
    }

    #[test]
    fn test_format_json_basic() {
        let symbols = vec![Symbol {
            name: "key".to_string(),
            kind: SymbolKind::String,
            path: "key".to_string(),
            line: 1,
            column: 1,
            children: Vec::new(),
        }];

        let args = SymbolsArgs {
            input: None,
            format: SymbolsFormat::Json,
            types: false,
            positions: false,
        };

        let output = format_json(&symbols, &args);
        assert!(output.contains("\"name\": \"key\""));
        assert!(output.contains("\"kind\": \"string\""));
        assert!(output.contains("\"path\": \"key\""));
        assert!(output.contains("\"line\": 1"));
        assert!(output.contains("\"column\": 1"));
    }

    #[test]
    fn test_format_json_preserves_tree_structure() {
        let symbols = vec![Symbol {
            name: "server".to_string(),
            kind: SymbolKind::Object,
            path: "server".to_string(),
            line: 1,
            column: 1,
            children: vec![Symbol {
                name: "host".to_string(),
                kind: SymbolKind::String,
                path: "server.host".to_string(),
                line: 2,
                column: 3,
                children: Vec::new(),
            }],
        }];

        let args = SymbolsArgs {
            input: None,
            format: SymbolsFormat::Json,
            types: false,
            positions: false,
        };

        let output = format_json(&symbols, &args);
        // Should contain nested children in JSON
        assert!(output.contains("\"children\":"));
        assert!(output.contains("\"name\": \"server\""));
        assert!(output.contains("\"name\": \"host\""));
        // Verify it's nested (host is inside children array)
        let parsed: Vec<Symbol> = serde_json::from_str(&output).expect("valid JSON");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].children.len(), 1);
        assert_eq!(parsed[0].children[0].name, "host");
    }

    #[test]
    fn test_flatten_symbols() {
        let symbols = vec![Symbol {
            name: "root".to_string(),
            kind: SymbolKind::Object,
            path: "root".to_string(),
            line: 1,
            column: 1,
            children: vec![
                Symbol {
                    name: "child1".to_string(),
                    kind: SymbolKind::String,
                    path: "root.child1".to_string(),
                    line: 2,
                    column: 3,
                    children: Vec::new(),
                },
                Symbol {
                    name: "child2".to_string(),
                    kind: SymbolKind::Number,
                    path: "root.child2".to_string(),
                    line: 3,
                    column: 3,
                    children: Vec::new(),
                },
            ],
        }];

        let flat = flatten_symbols(&symbols);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].path, "root");
        assert_eq!(flat[1].path, "root.child1");
        assert_eq!(flat[2].path, "root.child2");
    }

    #[test]
    fn test_symbol_kind_as_str() {
        assert_eq!(SymbolKind::Object.as_str(), "object");
        assert_eq!(SymbolKind::Array.as_str(), "array");
        assert_eq!(SymbolKind::String.as_str(), "string");
        assert_eq!(SymbolKind::Number.as_str(), "number");
        assert_eq!(SymbolKind::Boolean.as_str(), "boolean");
        assert_eq!(SymbolKind::Null.as_str(), "null");
    }
}
