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

//! Code action generation for LSP.
//!
//! This module provides functions to generate code actions (quick fixes,
//! refactorings, and source actions) for TOON documents.

use tower_lsp::lsp_types::{CodeAction, CodeActionKind, Range, Url};

#[cfg(test)]
use tower_lsp::lsp_types::Position;

use crate::ast::AstNode;

/// Collect code actions for a document.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `uri` - The document URI
/// * `range` - The range for which to provide code actions
///
/// # Returns
/// A vector of code actions
pub fn collect_code_actions(
    _ast: &AstNode,
    source: &str,
    _uri: &Url,
    _range: Range,
    diagnostics: &[tower_lsp::lsp_types::Diagnostic],
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    // Generate quick fixes for diagnostics
    for diag in diagnostics {
        if let Some(fix) = generate_quick_fix(diag, source) {
            actions.push(fix);
        }
    }

    // Generate source organize imports action (if applicable)
    // For TOON, this could be a "sort keys alphabetically" action
    actions.push(CodeAction {
        title: "Sort Object Keys Alphabetically".to_string(),
        kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
        diagnostics: None,
        edit: None,
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    });

    actions
}

/// Generate a quick fix for a diagnostic.
fn generate_quick_fix(
    diagnostic: &tower_lsp::lsp_types::Diagnostic,
    _source: &str,
) -> Option<CodeAction> {
    // For TOON, common quick fixes might include:
    // - Adding missing quotes around strings
    // - Converting tabs to spaces
    // - Fixing indentation

    let message = diagnostic.message.to_lowercase();

    if message.contains("quotes") || message.contains("string") {
        return Some(CodeAction {
            title: "Add missing quotes".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: None,
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        });
    }

    None
}

/// Generate a sort keys code action.
pub fn generate_sort_keys_action(uri: &Url, source: &str) -> Option<CodeAction> {
    // Parse to get object spans and sort keys within them
    // This is a placeholder for the actual implementation
    let _ = (uri, source);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_collect_code_actions_empty() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let uri: Url = "file:///test.toon".parse().unwrap();
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 10 },
        };

        let actions = collect_code_actions(&ast, source, &uri, range, &[]);
        // Should have at least the organize action
        assert!(!actions.is_empty());
    }
}
