// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Code lens generation for LSP.
//!
//! This module provides functions to generate code lenses that show
//! reference counts above keys in TOON documents.

use tower_lsp::lsp_types::{CodeLens, Command, Position, Range, Url};

use super::utf16::span_to_range;
use crate::ast::AstNode;

/// Collect code lenses from the AST.
///
/// Shows reference counts above keys that appear multiple times
/// in the document, helping users quickly understand usage patterns.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `uri` - The document URI for command arguments
///
/// # Returns
/// Vector of code lenses
pub fn collect_code_lenses(ast: &AstNode, source: &str, uri: &Url) -> Vec<CodeLens> {
    let mut lenses = Vec::new();
    let all_keys = super::ast_utils::collect_all_keys(ast);

    // Group keys by name and count occurrences
    let mut key_counts: std::collections::HashMap<String, Vec<crate::ast::Span>> =
        std::collections::HashMap::new();

    for (key, span) in &all_keys {
        key_counts.entry(key.clone()).or_default().push(*span);
    }

    // Generate lenses for keys that appear multiple times
    for (key_name, spans) in &key_counts {
        if spans.len() > 1 {
            // Add a lens above the first occurrence
            let first_span = spans[0];
        let range = span_to_range(&first_span, source);

        lenses.push(CodeLens {
            range: Range {
                start: Position { line: range.start.line, character: 0 },
                end: Position { line: range.start.line, character: 0 },
            },
            command: Some(Command {
                title: format!("{} references", spans.len()),
                command: "toon-lsp.peekReferences".to_string(),
                arguments: Some(vec![serde_json::json!({
                    "uri": uri.to_string(),
                    "position": {
                        "line": range.start.line,
                        "character": range.start.character,
                    }
                })]),
                }),
                data: Some(serde_json::json!({ "key": key_name })),
            });
        }
    }

    lenses
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_code_lens_multiple_references() {
        let uri: Url = "file:///test.toon".parse().unwrap();
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let lenses = collect_code_lenses(&ast, source, &uri);
        // Should have a lens for "name" (2 references)
        assert!(!lenses.is_empty());

        let name_lens = lenses.iter().find(|l| {
            l.data.as_ref().and_then(|d| d.get("key")).and_then(|v| v.as_str()) == Some("name")
        });
        assert!(name_lens.is_some());
        assert!(name_lens
            .unwrap()
            .command
            .as_ref()
            .unwrap()
            .title
            .contains("2 references"));
    }

    #[test]
    fn test_code_lens_single_key() {
        let uri: Url = "file:///test.toon".parse().unwrap();
        let source = "unique: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let lenses = collect_code_lenses(&ast, source, &uri);
        // Single-use keys should not have lenses
        assert!(lenses.is_empty());
    }

    #[test]
    fn test_code_lens_empty_document() {
        let uri: Url = "file:///test.toon".parse().unwrap();
        let source = "";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let lenses = collect_code_lenses(&ast, source, &uri);
        assert!(lenses.is_empty());
    }

    #[test]
    fn test_code_lens_three_references() {
        let uri: Url = "file:///test.toon".parse().unwrap();
        let source = "id: 1\ndata:\n  id: 2\nmore:\n  id: 3";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let lenses = collect_code_lenses(&ast, source, &uri);
        assert!(!lenses.is_empty());

        let id_lens = lenses.iter().find(|l| {
            l.data.as_ref().and_then(|d| d.get("key")).and_then(|v| v.as_str()) == Some("id")
        });
        assert!(id_lens.is_some());
        assert!(id_lens
            .unwrap()
            .command
            .as_ref()
            .unwrap()
            .title
            .contains("3 references"));
    }
}
