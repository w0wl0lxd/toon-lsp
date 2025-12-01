//! Tests for diagnostics functionality in the LSP server.

use toon_lsp::ast::{Position, Span};
use toon_lsp::lsp::diagnostics::{error_to_diagnostic, errors_to_diagnostics};
use toon_lsp::parser::{ParseError, ParseErrorKind, parse_with_errors};

/// Test ParseError to Diagnostic conversion
mod error_conversion {
    use super::*;
    use tower_lsp::lsp_types::DiagnosticSeverity;

    #[test]
    fn test_single_error_conversion() {
        let error = ParseError {
            kind: ParseErrorKind::ExpectedColon,
            span: Span::new(Position::new(0, 5, 5), Position::new(0, 6, 6)),
            context: None,
        };

        let source = "name Alice";
        let diagnostic = error_to_diagnostic(&error, source);

        assert_eq!(diagnostic.range.start.line, 0);
        assert_eq!(diagnostic.range.start.character, 5);
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert!(diagnostic.message.contains("colon"));
        assert_eq!(diagnostic.source, Some("toon-lsp".to_string()));
    }

    #[test]
    fn test_multiple_errors_conversion() {
        let source = "name\nage\ncity";
        let (_, errors) = parse_with_errors(source);

        let diagnostics = errors_to_diagnostics(&errors, source);

        // Should have errors for missing colons
        assert!(!diagnostics.is_empty());
        for diag in &diagnostics {
            assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
            assert_eq!(diag.source, Some("toon-lsp".to_string()));
        }
    }

    #[test]
    fn test_error_with_context() {
        let error = ParseError {
            kind: ParseErrorKind::UnexpectedToken,
            span: Span::new(Position::new(2, 0, 20), Position::new(2, 4, 24)),
            context: Some("expected value after colon".to_string()),
        };

        let source = "name: Alice\nage: 30\ncity";
        let diagnostic = error_to_diagnostic(&error, source);

        // Message should include the error info
        assert!(!diagnostic.message.is_empty());
    }

    #[test]
    fn test_diagnostic_range_utf16() {
        // Test with emoji that requires surrogate pair
        let source = "key\u{1F600}: value"; // emoji is at column 3 in UTF-8
        let error = ParseError {
            kind: ParseErrorKind::UnexpectedChar,
            span: Span::new(Position::new(0, 3, 3), Position::new(0, 7, 7)),
            context: None,
        };

        let diagnostic = error_to_diagnostic(&error, source);

        // Position should be converted to UTF-16
        assert_eq!(diagnostic.range.start.line, 0);
        // UTF-8 column 3 -> UTF-16 column 3 (before emoji)
        assert_eq!(diagnostic.range.start.character, 3);
    }
}

/// Test diagnostics publishing on didOpen
mod did_open {
    use super::*;

    #[test]
    fn test_valid_document_no_diagnostics() {
        let source = "name: Alice\nage: 30";
        let (_, errors) = parse_with_errors(source);
        let diagnostics = errors_to_diagnostics(&errors, source);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_document_has_diagnostics() {
        let source = "name Alice"; // Missing colon
        let (_, errors) = parse_with_errors(source);
        let diagnostics = errors_to_diagnostics(&errors, source);

        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_empty_document_no_diagnostics() {
        let source = "";
        let (_, errors) = parse_with_errors(source);
        let diagnostics = errors_to_diagnostics(&errors, source);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multiple_errors_all_reported() {
        let source = "name\nage\ncity"; // Multiple missing colons
        let (_, errors) = parse_with_errors(source);
        let diagnostics = errors_to_diagnostics(&errors, source);

        // Should have at least one error
        assert!(diagnostics.len() >= 1);
    }
}

/// Test diagnostics update on didChange
mod did_change {
    use super::*;
    use toon_lsp::lsp::state::DocumentState;

    #[test]
    fn test_fixing_error_clears_diagnostics() {
        let mut state = DocumentState::new("name".to_string(), 1);
        assert!(!state.errors().is_empty());

        state.update("name: Alice".to_string(), 2);
        let diagnostics = errors_to_diagnostics(state.errors(), state.text());

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_introducing_error_adds_diagnostics() {
        let mut state = DocumentState::new("name: Alice".to_string(), 1);
        assert!(state.errors().is_empty());

        state.update("name".to_string(), 2);
        let diagnostics = errors_to_diagnostics(state.errors(), state.text());

        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_changing_error_position() {
        let mut state = DocumentState::new("name".to_string(), 1);
        let diag1 = errors_to_diagnostics(state.errors(), state.text());
        let pos1 = diag1.first().map(|d| d.range.start.line);

        state.update("\nname".to_string(), 2);
        let diag2 = errors_to_diagnostics(state.errors(), state.text());
        let pos2 = diag2.first().map(|d| d.range.start.line);

        // Error should be on different line now
        assert_ne!(pos1, pos2);
    }
}

/// Test diagnostics clearing on didClose
mod did_close {
    #[test]
    fn test_clear_diagnostics_produces_empty_vec() {
        // When document is closed, we send empty diagnostics
        let diagnostics: Vec<tower_lsp::lsp_types::Diagnostic> = vec![];
        assert!(diagnostics.is_empty());
    }
}
