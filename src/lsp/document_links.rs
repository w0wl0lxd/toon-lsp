// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd

//! Document link generation for LSP.
//!
//! This module provides functions to detect URLs and file paths in TOON
//! string values and create clickable document links.

use tower_lsp::lsp_types::{DocumentLink, Url};

use super::utf16::span_to_range;
use crate::ast::AstNode;

/// Collect document links from string values in the AST.
///
/// Detects URLs (http://, https://, ftp://) in string values.
/// Returns document links that editors can make clickable.
///
/// # Arguments
/// * `ast` - The root AST node
/// * `source` - The document source text
///
/// # Returns
/// Vector of document links
pub fn collect_document_links(ast: &AstNode, source: &str) -> Vec<DocumentLink> {
    let mut links = Vec::new();
    collect_links_recursive(ast, source, &mut links);
    links
}

fn collect_links_recursive(
    node: &AstNode,
    source: &str,
    links: &mut Vec<DocumentLink>,
) {
    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                collect_links_recursive(child, source, links);
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                collect_links_recursive(&entry.value, source, links);
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                collect_links_recursive(item, source, links);
            }
        }
        AstNode::String { value, span } => {
            // Check if the string value is a URL
            if let Some(url) = detect_url(value) {
                let lsp_range = span_to_range(span, source);
                links.push(DocumentLink {
                    range: lsp_range,
                    target: Some(url),
                    tooltip: Some(format!("Open {}", value)),
                    data: None,
                });
            }
        }
        _ => {}
    }
}

/// Detect if a string is a URL.
fn detect_url(value: &str) -> Option<Url> {
    let trimmed = value.trim();

    // Check for common URL schemes
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("ftp://")
    {
        return Url::parse(trimmed).ok();
    }

    // Check for absolute file paths
    if trimmed.starts_with("file://") || trimmed.starts_with('/') {
        let url_str = if trimmed.starts_with('/') {
            format!("file://{}", trimmed)
        } else {
            trimmed.to_string()
        };
        return Url::parse(&url_str).ok();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_with_errors;

    #[test]
    fn test_detect_http_url() {
        let source = "website: https://example.com";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let links = collect_document_links(&ast, source);
        assert_eq!(links.len(), 1);
        assert_eq!(
            links[0].target.as_ref().map(|u| u.as_str()),
            Some("https://example.com/")
        );
    }

    #[test]
    fn test_detect_ftp_url() {
        let source = "data: ftp://files.example.com";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let links = collect_document_links(&ast, source);
        assert_eq!(links.len(), 1);
        assert_eq!(
            links[0].target.as_ref().map(|u| u.as_str()),
            Some("ftp://files.example.com/")
        );
    }

    #[test]
    fn test_no_link_in_non_url_string() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let links = collect_document_links(&ast, source);
        assert!(links.is_empty());
    }

    #[test]
    fn test_detect_file_path() {
        let source = "config: /etc/config.toml";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let links = collect_document_links(&ast, source);
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn test_multiple_links() {
        let source = "homepage: https://example.com\napi: https://api.example.com";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let links = collect_document_links(&ast, source);
        assert_eq!(links.len(), 2);
    }
}
