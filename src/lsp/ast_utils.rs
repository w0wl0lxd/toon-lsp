//! AST traversal utilities for LSP features.
//!
//! This module provides functions for finding nodes at positions,
//! collecting node paths, and other AST analysis tasks.

use crate::ast::{AstNode, ObjectEntry, Span};

/// A path through the AST from root to a specific node.
///
/// Each entry contains a reference to a node and context about
/// how it was reached (key name or array index).
#[derive(Debug, Clone)]
pub struct NodePathEntry<'a> {
    /// The AST node at this position in the path
    pub node: &'a AstNode,
    /// If this node is an object entry value, the key name
    pub key: Option<&'a str>,
    /// If this node is an array item, the index
    pub index: Option<usize>,
}

impl<'a> NodePathEntry<'a> {
    /// Create a new path entry for a root or standalone node.
    pub fn root(node: &'a AstNode) -> Self {
        Self {
            node,
            key: None,
            index: None,
        }
    }

    /// Create a new path entry for an object entry value.
    pub fn with_key(node: &'a AstNode, key: &'a str) -> Self {
        Self {
            node,
            key: Some(key),
            index: None,
        }
    }

    /// Create a new path entry for an array item.
    pub fn with_index(node: &'a AstNode, index: usize) -> Self {
        Self {
            node,
            key: None,
            index: Some(index),
        }
    }
}

/// Result of finding a node at a position.
#[derive(Debug)]
pub struct NodeAtPosition<'a> {
    /// Path from root to the found node
    pub path: Vec<NodePathEntry<'a>>,
    /// The innermost node containing the position
    pub node: &'a AstNode,
    /// If position is on an object key, the entry info
    pub on_key: Option<&'a ObjectEntry>,
}

/// Find the AST node at a given position.
///
/// Returns the path from root to the most specific node containing
/// the position, along with context about whether the position is
/// on a key or value.
///
/// # Arguments
/// * `root` - The root AST node (typically Document)
/// * `line` - 0-indexed line number
/// * `column` - 0-indexed column (UTF-8 bytes)
/// * `offset` - Byte offset from start of document
///
/// # Returns
/// `Some(NodeAtPosition)` if a node contains the position, `None` otherwise
pub fn find_node_at_position(
    root: &AstNode,
    line: u32,
    column: u32,
    offset: u32,
) -> Option<NodeAtPosition<'_>> {
    let pos = crate::ast::Position::new(line, column, offset);

    if !root.span().contains(pos) {
        return None;
    }

    let mut path = vec![NodePathEntry::root(root)];
    let mut current = root;
    let mut on_key: Option<&ObjectEntry> = None;

    loop {
        match current {
            AstNode::Document { children, .. } => {
                if let Some(child) = find_child_containing(children, pos) {
                    path.push(NodePathEntry::root(child));
                    current = child;
                } else {
                    break;
                }
            }
            AstNode::Object { entries, .. } => {
                if let Some((entry, is_on_key)) = find_entry_containing(entries, pos) {
                    if is_on_key {
                        on_key = Some(entry);
                        break;
                    }
                    path.push(NodePathEntry::with_key(&entry.value, &entry.key));
                    current = &entry.value;
                } else {
                    break;
                }
            }
            AstNode::Array { items, .. } => {
                if let Some((idx, item)) = find_item_containing(items, pos) {
                    path.push(NodePathEntry::with_index(item, idx));
                    current = item;
                } else {
                    break;
                }
            }
            // Leaf nodes - stop here
            AstNode::String { .. }
            | AstNode::Number { .. }
            | AstNode::Bool { .. }
            | AstNode::Null { .. } => break,
        }
    }

    Some(NodeAtPosition {
        path,
        node: current,
        on_key,
    })
}

/// Find a child node containing the position.
fn find_child_containing(children: &[AstNode], pos: crate::ast::Position) -> Option<&AstNode> {
    children.iter().find(|child| child.span().contains(pos))
}

/// Find an object entry containing the position.
/// Returns the entry and whether the position is on the key (true) or value (false).
fn find_entry_containing(
    entries: &[ObjectEntry],
    pos: crate::ast::Position,
) -> Option<(&ObjectEntry, bool)> {
    for entry in entries {
        // Check if on the key
        if entry.key_span.contains(pos) {
            return Some((entry, true));
        }
        // Check if on the value
        if entry.value.span().contains(pos) {
            return Some((entry, false));
        }
    }
    None
}

/// Find an array item containing the position.
/// Returns the index and reference to the item.
fn find_item_containing(items: &[AstNode], pos: crate::ast::Position) -> Option<(usize, &AstNode)> {
    items
        .iter()
        .enumerate()
        .find(|(_, item)| item.span().contains(pos))
}

/// Collect all keys from sibling entries in an object.
///
/// # Arguments
/// * `entries` - The object entries
/// * `exclude_key` - Optional key to exclude (e.g., the key being completed)
///
/// # Returns
/// Vector of sibling key names
pub fn collect_sibling_keys<'a>(
    entries: &'a [ObjectEntry],
    exclude_key: Option<&str>,
) -> Vec<&'a str> {
    entries
        .iter()
        .map(|e| e.key.as_str())
        .filter(|k| exclude_key != Some(*k))
        .collect()
}

/// Collect all keys from a node path (parent objects).
///
/// # Arguments
/// * `path` - The node path from root
///
/// # Returns
/// Vector of keys from ancestor objects
pub fn collect_parent_keys<'a>(path: &'a [NodePathEntry<'a>]) -> Vec<&'a str> {
    let mut keys = Vec::new();

    for entry in path {
        if let AstNode::Object { entries, .. } = entry.node {
            for obj_entry in entries {
                keys.push(obj_entry.key.as_str());
            }
        }
    }

    keys
}

/// Find all definitions of a key within an object.
///
/// Used for go-to-definition when duplicate keys exist.
///
/// # Arguments
/// * `entries` - The object entries to search
/// * `key_name` - The key to find
///
/// # Returns
/// Vector of spans where the key is defined
pub fn find_key_definitions<'a>(entries: &'a [ObjectEntry], key_name: &str) -> Vec<&'a Span> {
    entries
        .iter()
        .filter(|e| e.key == key_name)
        .map(|e| &e.key_span)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_find_node_at_root() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let result = find_node_at_position(&ast, 0, 0, 0);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_node_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "name" key
        let result = find_node_at_position(&ast, 0, 2, 2);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.on_key.is_some());
        assert_eq!(result.on_key.unwrap().key, "name");
    }

    #[test]
    fn test_find_node_on_value() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "Alice" value (after ": ")
        let result = find_node_at_position(&ast, 0, 6, 6);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.on_key.is_none());
        matches!(result.node, AstNode::String { .. });
    }

    #[test]
    fn test_collect_sibling_keys() {
        let source = "name: Alice\nage: 30\ncity: Boston";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        if let AstNode::Document { children, .. } = &ast {
            if let Some(AstNode::Object { entries, .. }) = children.first() {
                let keys = collect_sibling_keys(entries, None);
                assert_eq!(keys.len(), 3);
                assert!(keys.contains(&"name"));
                assert!(keys.contains(&"age"));
                assert!(keys.contains(&"city"));
            }
        }
    }

    #[test]
    fn test_find_key_definitions_unique() {
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        if let AstNode::Document { children, .. } = &ast {
            if let Some(AstNode::Object { entries, .. }) = children.first() {
                let defs = find_key_definitions(entries, "name");
                assert_eq!(defs.len(), 1);
            }
        }
    }
}
