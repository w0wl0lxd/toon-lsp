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
//! This module provides functions to generate code actions (refactorings and
//! source actions) for TOON documents.

use std::collections::HashMap;

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, Position as LspPosition, Range as LspRange, TextEdit, Url,
    WorkspaceEdit,
};

use crate::ast::{AstNode, ObjectEntry};

use super::ast_utils::find_node_at_position;
use super::utf16::utf16_to_utf8_col;

/// Collect code actions for a document at the given range.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
/// * `uri` - The document URI
/// * `range` - The range for which to provide code actions
/// * `diagnostics` - Diagnostics reported for the document
///
/// # Returns
/// A vector of code actions (currently the "Sort Object Keys" source action
/// for any object under the cursor whose keys are out of order).
pub fn collect_code_actions(
    ast: &AstNode,
    source: &str,
    uri: &Url,
    range: LspRange,
    _diagnostics: &[tower_lsp::lsp_types::Diagnostic],
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    if let Some(action) = generate_sort_keys_action(ast, source, uri, range) {
        actions.push(action);
    }

    actions
}

/// Generate a "Sort Object Keys Alphabetically" source action for the object
/// under the cursor, when its keys are not already sorted.
///
/// The edit reorders each entry's *verbatim* source text (key plus value,
/// including its trailing separator/newline) so formatting and comments inside
/// entries are preserved.
fn generate_sort_keys_action(
    ast: &AstNode,
    source: &str,
    uri: &Url,
    range: LspRange,
) -> Option<CodeAction> {
    // Resolve the cursor to the innermost object in the AST path.
    let offset = lsp_pos_to_offset(source, range.start.line, range.start.character);
    let found = find_node_at_position(ast, range.start.line, range.start.character, offset)?;

    let entries: &[ObjectEntry] = found.path.iter().rev().find_map(|entry| match entry.node {
        AstNode::Object { entries, .. } => Some(entries.as_slice()),
        _ => None,
    })?;

    // Need at least two keys to be worth sorting.
    if entries.len() < 2 {
        return None;
    }

    // Already sorted? Then there is nothing to do.
    let already_sorted = entries.windows(2).all(|w| w[0].key <= w[1].key);
    if already_sorted {
        return None;
    }

    let n = entries.len();
    let region_start = entries[0].key_span.start.offset;
    let region_end = entries[n - 1].value.span().end.offset;

    // Leading context (newline + indentation) that precedes each entry in
    // document order. `pre[0]` is empty because the region starts at
    // the first key; every other entry's `pre` is the separator and
    // indentation that belong before it.
    let mut pres: Vec<String> = Vec::with_capacity(n);
    for i in 0..n {
        let start = if i == 0 { region_start } else { entries[i - 1].value.span().end.offset };
        let end = entries[i].key_span.start.offset;
        pres.push(source[start as usize..end as usize].to_string());
    }

    // Indentation shared by sibling entries, used as a fallback for an entry
    // that was originally first (and so has no leading context) but is
    // reordered below another entry.
    let common_indent: String = pres
        .iter()
        .skip(1)
        .find(|p| !p.is_empty())
        .map(|p| trailing_ws(p).to_string())
        .unwrap_or_default();

    // Order entry indices by key, then rebuild the object text.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| entries[a].key.cmp(&entries[b].key));

    let mut new_text = String::new();
    for (pos, &idx) in order.iter().enumerate() {
        if pos != 0 {
            // Preserve the leading separator + indentation of the entry;
            // fall back to a newline + common indent when it was the
            // originally-first entry and so carries no leading context.
            let pre = &pres[idx];
            if pre.is_empty() {
                new_text.push('\n');
                new_text.push_str(&common_indent);
            } else {
                new_text.push_str(pre);
            }
        }
        let start = entries[idx].key_span.start.offset as usize;
        let end = entries[idx].value.span().end.offset as usize;
        new_text.push_str(&source[start..end]);
    }

    let edit_range = LspRange {
        start: offset_to_lsp_pos(source, region_start),
        end: offset_to_lsp_pos(source, region_end),
    };
    let text_edit = TextEdit { range: edit_range, new_text };

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![text_edit]);
    let workspace_edit = WorkspaceEdit { changes: Some(changes), ..Default::default() };

    Some(CodeAction {
        title: "Sort Object Keys Alphabetically".to_string(),
        // A distinct source action kind: reusing `SOURCE_ORGANIZE_IMPORTS`
        // would let editors with `codeActionsOnSave: { "source.organizeImports": true }`
        // accidentally reorder keys on every save.
        kind: Some(CodeActionKind::new("source.sortObjectKeys")),
        edit: Some(workspace_edit),
        ..Default::default()
    })
}

/// Convert an LSP (line, UTF-16 character) position to a byte offset.
fn lsp_pos_to_offset(source: &str, line: u32, utf16_char: u32) -> u32 {
    let mut line_start = 0usize;
    let mut cur_line = 0u32;
    for (i, ch) in source.char_indices() {
        if cur_line == line {
            break;
        }
        if ch == '\n' {
            cur_line += 1;
            line_start = i + 1;
        }
    }
    // Bound `line_text` to the current line so an out-of-range cursor
    // column from the client cannot walk into the next line.
    let line_end = source[line_start..].find('\n').map(|i| line_start + i).unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let utf8_col = utf16_to_utf8_col(line_text, utf16_char);
    (line_start as u32) + utf8_col
}

/// Return the trailing whitespace of `s` (the text after its last newline),
/// i.e. the indentation of the line that `s` ends on.
fn trailing_ws(s: &str) -> &str {
    s.rsplit('\n').next().unwrap_or("")
}

/// Convert a byte offset to an LSP position (line, UTF-16 character).
fn offset_to_lsp_pos(source: &str, offset: u32) -> LspPosition {
    let offset = offset as usize;
    let mut line = 0u32;
    let mut line_start = 0usize;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    let line_text = &source[line_start..offset];
    let utf16_col = line_text.chars().map(|c| c.len_utf16() as u32).sum();
    LspPosition { line, character: utf16_col }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;
    use tower_lsp::lsp_types::Position;

    fn actions_for(source: &str, line: u32, character: u32) -> Vec<CodeAction> {
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");
        let uri: Url = "file:///test.toon".parse().unwrap();
        let range =
            LspRange { start: Position { line, character }, end: Position { line, character } };
        collect_code_actions(&ast, source, &uri, range, &[])
    }

    #[test]
    fn no_action_when_sorted() {
        let source = "apple: 2\nbanana: 1\ncherry: 3";
        assert!(actions_for(source, 0, 0).is_empty());
    }

    #[test]
    fn sort_action_when_unsorted() {
        let source = "banana: 1\napple: 2\ncherry: 3";
        let actions = actions_for(source, 1, 0);
        assert_eq!(actions.len(), 1);
        let action = &actions[0];
        assert_eq!(action.title, "Sort Object Keys Alphabetically");
        let &CodeAction { edit: Some(we), .. } = &action else {
            panic!("expected a workspace edit");
        };
        let changes = we.changes.as_ref().expect("workspace edit must have changes");
        let edits = changes.values().next().expect("one document");
        assert_eq!(edits.len(), 1);
        let new = &edits[0].new_text;
        // Keys should now be in order: apple, banana, cherry.
        let order: Vec<&str> = new
            .split('\n')
            .filter_map(|l| l.split(':').next())
            .map(|k| k.trim())
            .filter(|k| !k.is_empty())
            .collect();
        assert_eq!(order, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn no_action_for_single_key() {
        let source = "only: 1";
        assert!(actions_for(source, 0, 0).is_empty());
    }

    #[test]
    fn multiline_values_preserved() {
        // `alpha` is the originally-last entry; reordering it to the
        // front must not merge it with `zeta` (the originally-first entry)
        // or drop the separator between them.
        let source = "zeta:\n  a: 1\nalpha:\n  b: 2";
        let actions = actions_for(source, 0, 0);
        assert_eq!(actions.len(), 1);
        let we = actions[0].edit.as_ref().unwrap();
        let changes = we.changes.as_ref().expect("workspace edit must have changes");
        let new = &changes.values().next().unwrap()[0].new_text;
        // Exact reordered text: alpha first, zeta second, with a newline
        // separator between them and both nested values intact.
        assert_eq!(new, "alpha:\n  b: 2\nzeta:\n  a: 1");
        assert!(new.contains("a: 1"));
        assert!(new.contains("b: 2"));
    }

    #[test]
    fn nested_object_indentation_preserved() {
        // Sorting an object whose entries are indented under a parent
        // must keep the indentation so entries stay in scope.
        let source = "user:\n  name: Bob\n  age: 30";
        let actions = actions_for(source, 1, 2);
        assert_eq!(actions.len(), 1);
        let we = actions[0].edit.as_ref().unwrap();
        let changes = we.changes.as_ref().expect("workspace edit must have changes");
        let new = &changes.values().next().unwrap()[0].new_text;
        // `age` (originally last) moves first but keeps its 2-space indent.
        assert_eq!(new, "age: 30\n  name: Bob");
    }
}
