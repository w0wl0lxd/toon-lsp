//! TOON lexer/scanner for tokenizing input.
//!
//! The scanner converts TOON source text into a stream of tokens
//! with position information.

use crate::ast::{Position, Span};

/// Token types in TOON.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Structural
    Colon,
    Comma,
    Newline,
    Indent,
    Dedent,
    Eof,

    // Literals
    String(String),
    Number(String),
    True,
    False,
    Null,

    // Identifiers (keys)
    Identifier(String),

    // Special
    Error(String),
}

/// A token with its span.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Scanner state for tokenizing TOON input.
pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: u32,
    column: u32,
    offset: u32,
    indent_stack: Vec<u32>,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner for the given source.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 0,
            column: 0,
            offset: 0,
            indent_stack: vec![0],
        }
    }

    /// Get current position in source.
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Scan all tokens from the source.
    pub fn scan_all(&mut self) -> Vec<Token> {
        // TODO: Implement full tokenization
        // Reference: https://github.com/toon-format/toon-rust/blob/main/src/decode/scanner.rs
        let _ = self.source;
        todo!("Implement scanner")
    }

    /// Scan the next token.
    pub fn next_token(&mut self) -> Token {
        // TODO: Implement token scanning
        todo!("Implement next_token")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_position() {
        let scanner = Scanner::new("test");
        let pos = scanner.current_position();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }
}
