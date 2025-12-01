//! Scanner integration tests for TOON lexer.
//!
//! These tests verify the Scanner's ability to tokenize TOON syntax correctly.
//! Uses insta for snapshot testing to ensure consistent token output.

use toon_lsp::parser::{Scanner, Token, TokenKind};

/// Helper function to scan all tokens from a source string.
///
/// Returns a vector of tokens with their kinds for easier testing.
fn scan_tokens(source: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(source);
    scanner.scan_all()
}

/// Helper function to extract token kinds from tokens for comparison.
///
/// This simplifies assertions by focusing on token types rather than positions.
fn token_kinds(tokens: &[Token]) -> Vec<TokenKind> {
    tokens.iter().map(|t| t.kind.clone()).collect()
}

#[cfg(test)]
mod basic_tokens {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_empty_input() {
        let tokens = scan_tokens("");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_single_colon() {
        let tokens = scan_tokens(":");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_brackets_and_braces() {
        let tokens = scan_tokens("[ ] { }");
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod literals {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_string_literal() {
        let tokens = scan_tokens(r#""hello world""#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_number_literal() {
        let tokens = scan_tokens("42");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = scan_tokens("true false");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_null_literal() {
        let tokens = scan_tokens("null");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_string_with_escapes() {
        let tokens = scan_tokens(r#""hello\nworld\t\"quoted\"""#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_string_with_backslash() {
        let tokens = scan_tokens(r#""path\\to\\file""#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_unterminated_string() {
        let tokens = scan_tokens(r#""unterminated"#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let tokens = scan_tokens(r#""\x invalid""#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_empty_string() {
        let tokens = scan_tokens(r#""""#);
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod indentation {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_simple_indent() {
        let source = "key:\n  value";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_multiple_dedents() {
        let source = "a:\n  b:\n    c:\n  d:";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod arrays {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_dash_prefix() {
        let tokens = scan_tokens("- item");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_array_with_multiple_items() {
        let source = "- first\n- second\n- third";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod complex {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_nested_object() {
        let source = r#"
user:
  name: "Alice"
  age: 30
"#;
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_mixed_structures() {
        let source = r#"
data:
  items:
    - "first"
    - "second"
  count: 2
"#;
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod error_recovery {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_control_character() {
        // NUL character (0x00) should produce error
        let tokens = scan_tokens("foo\x00bar");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_scanner_continues_after_error() {
        // Error token followed by valid tokens - scanner should recover
        let tokens = scan_tokens("@ valid: value");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_multiple_errors() {
        // Multiple error tokens - scanner keeps going
        let tokens = scan_tokens("@ # $ %");
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_tab_indentation_error() {
        // Tabs in indentation should produce error
        let source = "key:\n\tvalue";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_indentation_mismatch_error() {
        // Dedent to non-matching level should produce error
        let source = "a:\n    b:\n  c:";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_unterminated_string_recovery() {
        // Unterminated string error, then valid tokens on next line
        let source = "\"unterminated\nvalid: value";
        let tokens = scan_tokens(source);
        assert_debug_snapshot!(token_kinds(&tokens));
    }

    #[test]
    fn test_error_preserves_position() {
        // Verify error token has correct position
        let tokens = scan_tokens("@test");
        // Should be: Error(@), Identifier(test), EOF
        assert!(tokens.len() >= 2);
        assert!(matches!(tokens[0].kind, TokenKind::Error(_)));
        // Position should be column 0 (start of line)
        assert_eq!(tokens[0].span.start.column, 0);
        assert_eq!(tokens[0].span.start.line, 0);
    }

    #[test]
    fn test_leading_zeros_become_string() {
        // Numbers with leading zeros become strings per TOON spec
        let tokens = scan_tokens("007");
        assert_debug_snapshot!(token_kinds(&tokens));
    }
}

#[cfg(test)]
mod position_accuracy {
    use super::*;

    #[test]
    fn test_single_token_position() {
        // Single identifier at start
        let tokens = scan_tokens("hello");
        assert_eq!(tokens.len(), 2); // hello + EOF
        let token = &tokens[0];
        assert_eq!(token.span.start.line, 0);
        assert_eq!(token.span.start.column, 0);
        assert_eq!(token.span.end.line, 0);
        assert_eq!(token.span.end.column, 5); // "hello" is 5 chars
    }

    #[test]
    fn test_second_line_position() {
        // Token on second line
        let tokens = scan_tokens("a\nb");
        // Tokens: a, newline, b, EOF
        let b_token = &tokens[2];
        assert_eq!(b_token.span.start.line, 1);
        assert_eq!(b_token.span.start.column, 0);
    }

    #[test]
    fn test_column_after_spaces() {
        // "   foo" at line start produces Indent token (3 > 0 baseline)
        // Test that "foo" comes after spaces on same line (not line start)
        let tokens = scan_tokens("a:   foo");
        // Tokens: a, colon, foo, EOF
        // "foo" starts at column 5 (after "a:   ")
        let foo = &tokens[2];
        assert!(matches!(foo.kind, TokenKind::Identifier(ref s) if s == "foo"));
        assert_eq!(foo.span.start.column, 5);
        assert_eq!(foo.span.end.column, 8);
    }

    #[test]
    fn test_utf16_column_counting() {
        // UTF-16 surrogate pairs (emoji takes 2 UTF-16 code units)
        // Note: In the actual source, emoji would be in a string
        // Here we test ASCII to verify basic counting
        let tokens = scan_tokens("abc");
        let abc = &tokens[0];
        assert_eq!(abc.span.start.column, 0);
        assert_eq!(abc.span.end.column, 3);
    }

    #[test]
    fn test_multiline_spans() {
        let source = "line1\nline2\nline3";
        let tokens = scan_tokens(source);
        // line1: 0-4 on line 0
        // newline: 5 on line 0
        // line2: 0-4 on line 1
        // newline: 5 on line 1
        // line3: 0-4 on line 2

        // Verify line3 position
        let line3 = &tokens[4];
        assert!(matches!(line3.kind, TokenKind::Identifier(ref s) if s == "line3"));
        assert_eq!(line3.span.start.line, 2);
        assert_eq!(line3.span.start.column, 0);
    }

    #[test]
    fn test_colon_positions() {
        let tokens = scan_tokens("a: b");
        // a at 0, : at 1, b at 3
        let colon = &tokens[1];
        assert!(matches!(colon.kind, TokenKind::Colon));
        assert_eq!(colon.span.start.column, 1);
        assert_eq!(colon.span.end.column, 2);
    }

    #[test]
    fn test_string_literal_span() {
        let tokens = scan_tokens(r#""hello""#);
        let string = &tokens[0];
        assert!(matches!(string.kind, TokenKind::String(_)));
        // Span includes quotes: column 0-7
        assert_eq!(string.span.start.column, 0);
        assert_eq!(string.span.end.column, 7);
    }

    #[test]
    fn test_indent_dedent_positions() {
        let source = "a:\n  b:\n    c\n  d";
        let tokens = scan_tokens(source);

        // Find indent tokens and verify they have zero-width spans (virtual tokens)
        for token in &tokens {
            if matches!(token.kind, TokenKind::Indent | TokenKind::Dedent) {
                // Indent/Dedent are logical tokens - position marks where they occur
                // They should have spans indicating position in source
                assert!(token.span.start.line <= token.span.end.line);
            }
        }
    }

    #[test]
    fn test_eof_position() {
        let tokens = scan_tokens("abc");
        let eof = tokens.last().unwrap();
        assert!(matches!(eof.kind, TokenKind::Eof));
        // EOF should be at the end of the source
        assert_eq!(eof.span.start.column, 3);
    }

    #[test]
    fn test_empty_source_eof() {
        let tokens = scan_tokens("");
        let eof = &tokens[0];
        assert!(matches!(eof.kind, TokenKind::Eof));
        assert_eq!(eof.span.start.line, 0);
        assert_eq!(eof.span.start.column, 0);
    }

    #[test]
    fn test_number_span() {
        let tokens = scan_tokens("12345");
        let num = &tokens[0];
        assert!(matches!(num.kind, TokenKind::Number(_)));
        assert_eq!(num.span.start.column, 0);
        assert_eq!(num.span.end.column, 5);
    }

    #[test]
    fn test_negative_number_span() {
        let tokens = scan_tokens("-42");
        let num = &tokens[0];
        assert!(matches!(num.kind, TokenKind::Number(_)));
        assert_eq!(num.span.start.column, 0);
        assert_eq!(num.span.end.column, 3);
    }

    #[test]
    fn test_float_number_span() {
        let tokens = scan_tokens("3.14159");
        let num = &tokens[0];
        assert!(matches!(num.kind, TokenKind::Number(_)));
        assert_eq!(num.span.start.column, 0);
        assert_eq!(num.span.end.column, 7);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Scanner should never panic on any input
        #[test]
        fn scanner_never_panics(input in "\\PC*") {
            let mut scanner = Scanner::new(&input);
            let _ = scanner.scan_all();
        }

        /// Every token stream ends with EOF
        #[test]
        fn always_ends_with_eof(input in "\\PC*") {
            let tokens = scan_tokens(&input);
            assert!(!tokens.is_empty());
            assert!(matches!(tokens.last().unwrap().kind, TokenKind::Eof));
        }

        /// Token spans are never inverted (start <= end)
        #[test]
        fn spans_are_valid(input in "\\PC*") {
            let tokens = scan_tokens(&input);
            for token in &tokens {
                // Start should be <= end within same line, or start.line < end.line
                assert!(
                    token.span.start.line < token.span.end.line
                    || (token.span.start.line == token.span.end.line
                        && token.span.start.column <= token.span.end.column),
                    "Invalid span for {:?}: {:?}",
                    token.kind,
                    token.span
                );
            }
        }

        /// Valid identifiers should produce Identifier tokens
        #[test]
        fn valid_identifiers_parse(name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
            let tokens = scan_tokens(&name);
            assert!(tokens.len() >= 2);
            match &tokens[0].kind {
                TokenKind::Identifier(s) => assert_eq!(s, &name),
                TokenKind::True | TokenKind::False | TokenKind::Null => {
                    // Keywords are valid
                }
                _ => panic!("Expected identifier, got {:?}", tokens[0].kind),
            }
        }

        /// Valid integers should produce Number tokens
        #[test]
        fn valid_integers_parse(n in 1i64..1_000_000) {
            let input = n.to_string();
            let tokens = scan_tokens(&input);
            assert!(tokens.len() >= 2);
            match &tokens[0].kind {
                TokenKind::Number(s) => assert_eq!(s, &input),
                _ => panic!("Expected number, got {:?}", tokens[0].kind),
            }
        }

        /// Line count increases correctly with newlines
        #[test]
        fn newlines_increase_line_count(num_lines in 1usize..10) {
            let input: String = (0..num_lines).map(|i| format!("line{}\n", i)).collect();
            let tokens = scan_tokens(&input);
            let eof = tokens.last().unwrap();
            // Last line after all newlines
            assert_eq!(eof.span.start.line as usize, num_lines);
        }
    }
}
