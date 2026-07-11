// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Command execution for exporting dependency graphs.

use std::collections::HashMap;
use std::fmt::Write;

use super::GraphArgs;
use super::error::CliError;
use super::io_utils::{read_input, write_output};
use crate::ast::AstNode;
use crate::parser::parse_with_errors;

/// Run the dependency graph command.
pub fn execute(args: &GraphArgs) -> Result<(), CliError> {
    let source = read_input(&args.input)?;

    let (ast, errors) = parse_with_errors(&source);
    if !errors.is_empty() {
        let error_msg = format!("Document has {} syntax error(s)", errors.len());
        return Err(CliError::Validation(error_msg));
    }

    let ast = ast.ok_or_else(|| {
        CliError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to construct AST",
        ))
    })?;

    let graph = generate_mermaid_graph(&ast, &source);

    write_output(&args.output, &graph)?;

    Ok(())
}

fn collect_all_defined_keys(
    node: &AstNode,
    current_path: &mut Vec<String>,
    keys: &mut Vec<String>,
) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_all_defined_keys(child, current_path, keys);
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                current_path.push(entry.key.clone());
                keys.push(current_path.join("."));
                collect_all_defined_keys(&entry.value, current_path, keys);
                current_path.pop();
            }
        }
        AstNode::Array { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                current_path.push(i.to_string());
                keys.push(current_path.join("."));
                collect_all_defined_keys(item, current_path, keys);
                current_path.pop();
            }
        }
        _ => {}
    }
}

fn collect_reference_edges(
    node: &AstNode,
    current_path: &mut Vec<String>,
    edges: &mut Vec<(String, String)>,
) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_reference_edges(child, current_path, edges);
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                current_path.push(entry.key.clone());
                collect_reference_edges(&entry.value, current_path, edges);
                current_path.pop();
            }
        }
        AstNode::Array { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                current_path.push(i.to_string());
                collect_reference_edges(item, current_path, edges);
                current_path.pop();
            }
        }
        AstNode::Reference { path, .. } => {
            let dependent = current_path.join(".");
            edges.push((path.clone(), dependent));
        }
        _ => {}
    }
}

/// Escape a key for a Mermaid quoted node label (`["..."]`); a raw `"` or
/// newline would otherwise produce invalid flowchart syntax.
fn escape_mermaid_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for ch in label.chars() {
        match ch {
            '"' => out.push_str("&quot;"),
            '\n' | '\r' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

/// Generate a Mermaid flowchart from AST references.
pub fn generate_mermaid_graph(ast: &AstNode, _source: &str) -> String {
    let mut current_path = Vec::new();
    let mut keys = Vec::new();
    collect_all_defined_keys(ast, &mut current_path, &mut keys);

    let mut edges = Vec::new();
    collect_reference_edges(ast, &mut current_path, &mut edges);

    let mut id_map = HashMap::new();
    let mut next_id = 0;

    let mut output = String::new();
    output.push_str("flowchart TD\n");

    let mut ensure_node = |key: &str, output: &mut String, id_map: &mut HashMap<String, String>| {
        if !id_map.contains_key(key) {
            let id = format!("n{}", next_id);
            next_id += 1;
            let _ = writeln!(output, "    {}[\"{}\"]", id, escape_mermaid_label(key));
            id_map.insert(key.to_string(), id);
        }
    };

    for key in &keys {
        ensure_node(key, &mut output, &mut id_map);
    }

    for (src, dest) in &edges {
        ensure_node(src, &mut output, &mut id_map);
        ensure_node(dest, &mut output, &mut id_map);
    }

    for (src, dest) in &edges {
        // Endpoints registered above; skip defensively rather than panic.
        if let (Some(src_id), Some(dest_id)) = (id_map.get(src), id_map.get(dest)) {
            let _ = writeln!(output, "    {} --> {}", src_id, dest_id);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_double_quotes_in_labels() {
        assert_eq!(escape_mermaid_label("a\"b"), "a&quot;b");
    }

    #[test]
    fn replaces_newlines_with_spaces() {
        assert_eq!(escape_mermaid_label("a\nb\rc"), "a b c");
    }

    #[test]
    fn leaves_plain_labels_unchanged() {
        assert_eq!(escape_mermaid_label("service.name"), "service.name");
    }

    #[test]
    fn mermaid_output_escapes_quoted_reference_keys() {
        // A key containing a quote must not break the flowchart syntax.
        let source = "\"we\\\"ird\": ${target}\ntarget: 42\n";
        let (ast, _) = parse_with_errors(source);
        if let Some(ast) = ast {
            let out = generate_mermaid_graph(&ast, source);
            assert!(!out.contains("we\"ird"), "raw quote must be escaped in output");
        }
    }
}
