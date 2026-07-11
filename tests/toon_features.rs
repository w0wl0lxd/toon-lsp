//! TOON language feature tests: comments, block strings, hexadecimal literals.
//!
//! These verify the parser/scanner handle the enriched TOON literal & comment
//! syntax. Assertion-based (not snapshot) so failures are explicit.

use toon_lsp::NumberValue;
use toon_lsp::ast::AstNode;
use toon_lsp::parser::{Scanner, TokenKind, parse};

/// Walk a top-level document and return a key's string value, if present.
fn find_string(ast: &AstNode, key: &str) -> Option<String> {
    if let AstNode::Document { children, .. } = ast {
        for node in children {
            if let AstNode::Object { entries, .. } = node {
                for e in entries {
                    if e.key == key
                        && let AstNode::String { value, .. } = &e.value
                    {
                        return Some(value.clone());
                    }
                }
            }
        }
    }
    None
}

/// Walk a top-level document and return a key's numeric value, if present.
fn find_number(ast: &AstNode, key: &str) -> Option<NumberValue> {
    if let AstNode::Document { children, .. } = ast {
        for node in children {
            if let AstNode::Object { entries, .. } = node {
                for e in entries {
                    if e.key == key
                        && let AstNode::Number { value, .. } = &e.value
                    {
                        return Some(*value);
                    }
                }
            }
        }
    }
    None
}

fn scan_kinds(source: &str) -> Vec<TokenKind> {
    Scanner::new(source).scan_all().into_iter().map(|t| t.kind).collect()
}

#[cfg(test)]
mod comments {
    use super::*;

    #[test]
    fn line_comment_is_ignored() {
        let ast = parse("# a leading comment\nname: Alice").expect("comment should parse");
        assert_eq!(find_string(&ast, "name").as_deref(), Some("Alice"));
    }

    #[test]
    fn trailing_line_comment_is_ignored() {
        let ast = parse("name: Alice # trailing note").expect("comment should parse");
        assert_eq!(find_string(&ast, "name").as_deref(), Some("Alice"));
    }

    #[test]
    fn block_comment_inline_is_ignored() {
        let ast =
            parse("name: Alice /* inline note */\nage: 30").expect("block comment should parse");
        assert_eq!(find_string(&ast, "name").as_deref(), Some("Alice"));
        assert_eq!(find_number(&ast, "age"), Some(NumberValue::PosInt(30)));
    }

    #[test]
    fn block_comment_multiline_is_ignored() {
        let ast = parse("/*\n multi\n line\n*/\nname: Bob")
            .expect("multi-line block comment should parse");
        assert_eq!(find_string(&ast, "name").as_deref(), Some("Bob"));
    }

    #[test]
    fn scanner_skips_line_comment_tokens() {
        let kinds = scan_kinds("# comment\n42");
        assert!(!kinds.iter().any(|k| matches!(k, TokenKind::Error(_))));
        // The comment is skipped; a Number token is still produced for `42`.
        assert!(kinds.iter().any(|k| matches!(k, TokenKind::Number(_))));
    }
}

#[cfg(test)]
mod block_strings {
    use super::*;

    #[test]
    fn block_string_preserves_newlines() {
        let src = "desc: \"\"\"line one\nline two\nline three\"\"\"";
        let ast = parse(src).expect("block string should parse");
        let value = find_string(&ast, "desc").expect("desc present");
        assert!(value.contains("line one"));
        assert!(value.contains("line two"));
        assert!(value.contains('\n'));
        assert_eq!(value.matches('\n').count(), 2);
    }

    #[test]
    fn scanner_emits_block_string_as_string_token() {
        let kinds = scan_kinds("\"\"\"a\nb\"\"\"");
        assert!(matches!(kinds[0], TokenKind::String(ref s) if s.contains('\n')));
    }
}

#[cfg(test)]
mod hexadecimal {
    use super::*;

    #[test]
    fn upper_hex() {
        let ast = parse("code: 0xFF").expect("hex should parse");
        assert_eq!(find_number(&ast, "code"), Some(NumberValue::PosInt(255)));
    }

    #[test]
    fn lower_hex() {
        let ast = parse("code: 0x1f").expect("hex should parse");
        assert_eq!(find_number(&ast, "code"), Some(NumberValue::PosInt(31)));
    }

    #[test]
    fn negative_hex() {
        let ast = parse("code: -0x10").expect("negative hex should parse");
        assert_eq!(find_number(&ast, "code"), Some(NumberValue::NegInt(-16)));
    }

    #[test]
    fn scanner_keeps_hex_text() {
        let kinds = scan_kinds("0xFF");
        assert!(matches!(kinds[0], TokenKind::Number(ref s) if s == "0xFF"));
    }
}
