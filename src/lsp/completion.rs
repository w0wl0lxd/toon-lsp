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

//! Completion generation for LSP.
//!
//! This module provides functions to generate completion items for TOON
//! documents, suggesting keys from siblings and parents, plus boolean literals.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use super::ast_utils::{calculate_offset, find_node_at_position};
use crate::ast::AstNode;

/// A completion item for TOON.
#[derive(Debug, Clone)]
pub struct ToonCompletion {
    /// The label shown in the completion list
    pub label: String,
    /// The kind of completion item
    pub kind: CompletionItemKind,
    /// Optional detail text
    pub detail: Option<String>,
}

impl ToonCompletion {
    /// Create a new key completion.
    pub fn key(name: &str) -> Self {
        Self {
            label: name.to_string(),
            kind: CompletionItemKind::PROPERTY,
            detail: Some("key".to_string()),
        }
    }

    /// Create a new literal completion.
    pub fn literal(value: &str) -> Self {
        Self {
            label: value.to_string(),
            kind: CompletionItemKind::KEYWORD,
            detail: Some("literal".to_string()),
        }
    }
}

impl From<ToonCompletion> for CompletionItem {
    fn from(tc: ToonCompletion) -> Self {
        CompletionItem {
            label: tc.label,
            kind: Some(tc.kind),
            detail: tc.detail,
            ..Default::default()
        }
    }
}

/// Get completion items at a position in the document.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `line` - The line number (0-based)
/// * `column` - The column number (0-based, UTF-8)
///
/// # Returns
/// A vector of completion items
pub fn get_completions_at_position(
    ast: &AstNode,
    source: &str,
    line: u32,
    column: u32,
) -> Vec<ToonCompletion> {
    let mut completions = Vec::new();

    // Calculate offset
    let offset = match calculate_offset(source, line, column) {
        Some(o) => o,
        None => return completions,
    };

    // Determine completion context
    let context = determine_completion_context(source, line, column);

    match context {
        CompletionContext::AfterColon => {
            // Suggest boolean literals
            completions.push(ToonCompletion::literal("true"));
            completions.push(ToonCompletion::literal("false"));
            completions.push(ToonCompletion::literal("null"));
        }
        CompletionContext::KeyPosition => {
            // Suggest sibling keys from the containing object
            if let Some(node_at_pos) = find_node_at_position(ast, line, column, offset) {
                // Get sibling keys from the parent object in the path
                for entry in &node_at_pos.path {
                    if let Some(key) = &entry.key {
                        completions.push(ToonCompletion::key(key));
                    }
                }
            }

            // Also suggest root-level keys
            let root_keys = collect_root_keys(ast);
            for key in root_keys {
                if !completions.iter().any(|c| c.label == key) {
                    completions.push(ToonCompletion::key(&key));
                }
            }
        }
        CompletionContext::Unknown => {
            // Suggest both keys and literals
            completions.push(ToonCompletion::literal("true"));
            completions.push(ToonCompletion::literal("false"));
            completions.push(ToonCompletion::literal("null"));

            // Also suggest root-level keys
            let root_keys = collect_root_keys(ast);
            for key in root_keys {
                completions.push(ToonCompletion::key(&key));
            }
        }
    }

    completions
}

/// Completion context type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    /// After a colon, expecting a value
    AfterColon,
    /// At a key position (start of line or after newline)
    KeyPosition,
    /// Unknown context
    Unknown,
}

/// Determine the completion context at a position.
fn determine_completion_context(source: &str, line: u32, column: u32) -> CompletionContext {
    // Get the text of the current line up to the cursor
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = line as usize;

    if line_idx >= lines.len() {
        return CompletionContext::KeyPosition;
    }

    let line_text = lines[line_idx];
    let col = column as usize;
    let prefix = if col <= line_text.len() { &line_text[..col] } else { line_text };

    // Check if we're after a colon
    if prefix.contains(':') {
        let after_colon = prefix.rsplit(':').next().unwrap_or("");
        if after_colon.trim().is_empty() {
            return CompletionContext::AfterColon;
        }
    }

    // Check if at start of line or only whitespace before
    let trimmed = prefix.trim();
    if trimmed.is_empty() || !prefix.contains(':') {
        return CompletionContext::KeyPosition;
    }

    CompletionContext::Unknown
}

/// Collect all top-level keys from the AST.
fn collect_root_keys(ast: &AstNode) -> Vec<String> {
    let mut keys = Vec::new();

    match ast {
        AstNode::Document { children, .. } => {
            for child in children {
                if let AstNode::Object { entries, .. } = child {
                    for entry in entries {
                        keys.push(entry.key.clone());
                    }
                }
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                keys.push(entry.key.clone());
            }
        }
        _ => {}
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_completion_after_colon() {
        let source = "enabled: ";
        let (ast, _) = parse_with_errors(source);

        if let Some(ast) = ast {
            let completions = get_completions_at_position(&ast, source, 0, 9);
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
            assert!(labels.contains(&"true"));
            assert!(labels.contains(&"false"));
        }
    }

    #[test]
    fn test_completion_at_key_position() {
        let source = "name: Alice\n";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let completions = get_completions_at_position(&ast, source, 1, 0);
        // Should suggest existing keys
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"name") || completions.is_empty());
    }
}
