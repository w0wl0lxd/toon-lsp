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

//! Diagnostic conversion utilities for LSP.
//!
//! This module provides functions to convert parse errors to LSP diagnostics
//! with proper UTF-16 position encoding.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use super::utf16::span_to_range;
use crate::parser::ParseError;
use crate::resolve::{ResolveError, ResolvedRef};

/// Convert a single parse error to an LSP diagnostic.
///
/// # Arguments
/// * `error` - The parse error to convert
/// * `source` - The document source text (for UTF-16 conversion)
///
/// # Returns
/// An LSP Diagnostic with the error information
pub fn error_to_diagnostic(error: &ParseError, source: &str) -> Diagnostic {
    let range = span_to_range(&error.span, source);

    let message = if let Some(ref ctx) = error.context {
        format!("{}: {}", error.kind, ctx)
    } else {
        error.kind.to_string()
    };

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("toon-lsp".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert multiple parse errors to LSP diagnostics.
///
/// # Arguments
/// * `errors` - The parse errors to convert
/// * `source` - The document source text (for UTF-16 conversion)
///
/// # Returns
/// A vector of LSP Diagnostics
pub fn errors_to_diagnostics(errors: &[ParseError], source: &str) -> Vec<Diagnostic> {
    errors.iter().map(|err| error_to_diagnostic(err, source)).collect()
}

/// Validate a document's AST for semantic correctness.
///
/// Checks references and environment variable references.
pub fn validate_document(ast: &crate::ast::AstNode, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_node_recursive(ast, ast, source, &mut diagnostics);
    diagnostics
}

fn validate_node_recursive(
    node: &crate::ast::AstNode,
    root: &crate::ast::AstNode,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match node {
        crate::ast::AstNode::Document { children, .. } => {
            for child in children {
                validate_node_recursive(child, root, source, diagnostics);
            }
        }
        crate::ast::AstNode::Object { entries, .. } => {
            for entry in entries {
                validate_node_recursive(&entry.value, root, source, diagnostics);
            }
        }
        crate::ast::AstNode::Array { items, .. } => {
            for item in items {
                validate_node_recursive(item, root, source, diagnostics);
            }
        }
        crate::ast::AstNode::Reference { path, span, .. } => {
            let range = span_to_range(span, source);
            match crate::resolve::resolve(root, path) {
                Ok(ResolvedRef::Node { .. }) | Ok(ResolvedRef::Env(_)) => {}
                Err(ResolveError::EnvNotSet(name)) => diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: None,
                    code_description: None,
                    source: Some("toon-lsp".to_string()),
                    message: format!("Environment variable '{}' is not defined", name),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
                Err(ResolveError::NotFound(not_found)) => diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: None,
                    code_description: None,
                    source: Some("toon-lsp".to_string()),
                    message: format!("Unresolved reference: '{}'", not_found),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
                Err(ResolveError::Cycle(_)) => diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: None,
                    code_description: None,
                    source: Some("toon-lsp".to_string()),
                    message: format!("Cyclic reference: '{}'", path),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Position, Span};
    use crate::parser::ParseErrorKind;

    #[test]
    fn test_error_to_diagnostic_basic() {
        let error = ParseError {
            kind: ParseErrorKind::ExpectedColon,
            span: Span::new(Position::new(0, 4, 4), Position::new(0, 5, 5)),
            context: None,
        };

        let diag = error_to_diagnostic(&error, "name value");

        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.source, Some("toon-lsp".to_string()));
        assert!(diag.message.contains("colon"));
    }

    #[test]
    fn test_error_to_diagnostic_with_context() {
        let error = ParseError {
            kind: ParseErrorKind::ExpectedValue,
            span: Span::new(Position::new(0, 5, 5), Position::new(0, 5, 5)),
            context: Some("after colon".to_string()),
        };

        let diag = error_to_diagnostic(&error, "name:");

        assert!(diag.message.contains("after colon"));
    }

    #[test]
    fn test_errors_to_diagnostics_empty() {
        let diags = errors_to_diagnostics(&[], "");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_errors_to_diagnostics_multiple() {
        let errors = vec![
            ParseError {
                kind: ParseErrorKind::ExpectedColon,
                span: Span::new(Position::new(0, 4, 4), Position::new(0, 5, 5)),
                context: None,
            },
            ParseError {
                kind: ParseErrorKind::ExpectedColon,
                span: Span::new(Position::new(1, 3, 9), Position::new(1, 4, 10)),
                context: None,
            },
        ];

        let diags = errors_to_diagnostics(&errors, "name value\nage value");

        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].range.start.line, 0);
        assert_eq!(diags[1].range.start.line, 1);
    }

    #[test]
    fn test_validate_document_references() {
        use crate::parser::parse;
        let source = "db:\n  port: 5432\nservice:\n  db_port: ${db.invalid_port}";
        let ast = parse(source).expect("should parse");
        let diags = validate_document(&ast, source);

        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(diags[0].message.contains("Unresolved reference"));
        assert!(diags[0].message.contains("db.invalid_port"));
    }

    #[test]
    fn test_validate_document_env_vars() {
        use crate::parser::parse;
        let source = "service:\n  api_key: ${env:NONEXISTENT_ENV_VAR_XYZ}";
        let ast = parse(source).expect("should parse");
        let diags = validate_document(&ast, source);

        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(diags[0].message.contains("Environment variable"));
        assert!(diags[0].message.contains("NONEXISTENT_ENV_VAR_XYZ"));
    }
}
