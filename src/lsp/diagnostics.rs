//! Diagnostic conversion utilities for LSP.
//!
//! This module provides functions to convert parse errors to LSP diagnostics
//! with proper UTF-16 position encoding.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use super::utf16::span_to_range;
use crate::parser::ParseError;

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
    errors
        .iter()
        .map(|err| error_to_diagnostic(err, source))
        .collect()
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
}
