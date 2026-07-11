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

//! TOON lexer/scanner for tokenizing input.
//!
//! The scanner converts TOON source text into a stream of tokens
//! with position information.

use crate::ast::{Position, Span};

/// Token types in TOON.
///
/// # Example
///
/// ```
/// use toon_lsp::parser::TokenKind;
///
/// let colon = TokenKind::Colon;
/// assert_eq!(colon, TokenKind::Colon);
///
/// let string = TokenKind::String("hello".to_string());
/// assert!(matches!(string, TokenKind::String(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Structural
    Colon,
    Comma,
    Newline,
    Indent,
    Dedent,
    Eof,

    // Brackets and braces for arrays and objects
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Dash,         // - (for array items)

    // Literals
    String(String),
    Reference(String),
    Number(String),
    True,
    False,
    Null,

    // Identifiers (keys)
    Identifier(String),

    // Special
    Error(String),
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Indent => write!(f, "indent"),
            TokenKind::Dedent => write!(f, "dedent"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::Dash => write!(f, "-"),
            TokenKind::String(s) => write!(f, "string {:?}", s),
            TokenKind::Reference(s) => write!(f, "reference {:?}", s),
            TokenKind::Number(n) => write!(f, "number {}", n),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Identifier(id) => write!(f, "identifier '{}'", id),
            TokenKind::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

/// A token with its span.
///
/// # Example
///
/// ```
/// use toon_lsp::parser::{Token, TokenKind};
/// use toon_lsp::ast::Span;
///
/// let span = Span::new(
///     toon_lsp::ast::Position::new(0, 0, 0),
///     toon_lsp::ast::Position::new(0, 1, 1)
/// );
/// let token = Token::new(TokenKind::Colon, span);
/// assert_eq!(token.kind, TokenKind::Colon);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Stack of indentation levels for tracking nested blocks.
    indent_stack: Vec<u32>,
    /// Number of dedent tokens pending to be emitted.
    pending_dedents: u32,
    /// Whether the scanner is at the start of a line.
    at_line_start: bool,
    /// Whether EOF has been yielded (for Iterator impl).
    done: bool,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner for the given source.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_lsp::parser::Scanner;
    ///
    /// let mut scanner = Scanner::new("key: value");
    /// assert_eq!(scanner.current_position().line, 0);
    /// assert_eq!(scanner.current_position().column, 0);
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 0,
            column: 0,
            offset: 0,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
            done: false,
        }
    }

    /// Get current position in source.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_lsp::parser::Scanner;
    ///
    /// let mut scanner = Scanner::new("test");
    /// let pos = scanner.current_position();
    /// assert_eq!(pos.line, 0);
    /// assert_eq!(pos.column, 0);
    /// assert_eq!(pos.offset, 0);
    /// ```
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some((_, ch)) = self.chars.next() {
            // Update offset by UTF-8 byte length for source slicing
            self.offset += ch.len_utf8() as u32;

            // Handle newline: increment line, reset column
            // NOTE: '\r' is handled in skip_whitespace(); we only increment on '\n'
            // This prevents double-counting '\r\n' sequences on Windows
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else if ch != '\r' {
                // Only update column for non-newline, non-carriage-return characters
                // LSP standard: columns use UTF-16 code units
                self.column += ch.len_utf16() as u32;
            }
            // '\r' is silently skipped (handled by skip_whitespace), no column/line update

            Some(ch)
        } else {
            None
        }
    }

    /// Look at next character without consuming.
    ///
    /// Note: Requires `&mut self` because `Peekable::peek()` needs mutable access
    /// to internal state, even though it doesn't consume the iterator.
    /// This is the standard Rust pattern and does not violate CQS in practice.
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    /// Look ahead two characters (for escape sequences, etc).
    ///
    /// # Performance
    /// O(1) - Cloning `CharIndices` copies only the iterator position (two `usize`
    /// values), not the underlying string reference. The string remains shared
    /// across all clones.
    fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next()?;
        iter.peek().map(|(_, ch)| *ch)
    }

    /// Skip whitespace AND comments (trivia) between significant tokens.
    ///
    /// Handles three trivia kinds:
    /// - Spaces, tabs, and carriage returns (NOT newlines - those are tokens).
    /// - Line comments `# ...` (skipped up to but NOT including the newline, so
    ///   indentation structure is preserved).
    /// - Block comments `/* ... */` (may span newlines; consumed until `*/`,
    ///   or to EOF if unterminated, to avoid infinite loops).
    ///
    /// # Windows Compatibility
    /// Skips `\r` to handle CRLF line endings transparently.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.advance();
                }
                Some('#') => {
                    // Line comment: skip to end of line, but leave the newline
                    // token intact so indentation structure is preserved.
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('/') if self.peek_next() == Some('*') => {
                    // Block comment: consume until '*/' (newlines allowed).
                    self.advance(); // '/'
                    self.advance(); // '*'
                    let mut closed = false;
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_next() == Some('/') {
                            self.advance(); // '*'
                            self.advance(); // '/'
                            closed = true;
                            break;
                        }
                        self.advance();
                    }
                    if !closed {
                        // Unterminated block comment: consume to EOF.
                        while self.peek().is_some() {
                            self.advance();
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Look ahead three characters (for triple-quote detection).
    fn peek_next2(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()?;
        iter.peek().map(|(_, ch)| *ch)
    }

    /// Check if the cursor is at a closing triple quote `"""`.
    fn is_closing_triple(&self) -> bool {
        let mut iter = self.chars.clone();
        let c0 = iter.next().map(|(_, c)| c);
        let c1 = iter.next().map(|(_, c)| c);
        let c2 = iter.peek().map(|(_, c)| *c);
        c0 == Some('"') && c1 == Some('"') && c2 == Some('"')
    }

    /// Count leading spaces at line start.
    ///
    /// Returns (space_count, has_tab). Tabs are an error in TOON indentation.
    ///
    /// # Design Rationale
    /// TOON requires consistent space-based indentation. Tabs are explicitly
    /// forbidden to prevent mixed indentation styles that could lead to
    /// ambiguous parsing.
    fn count_leading_spaces(&mut self) -> (u32, bool) {
        let mut count = 0u32;
        let mut has_tab = false;

        while let Some(ch) = self.peek() {
            match ch {
                ' ' => {
                    count += 1;
                    self.advance();
                }
                '\t' => {
                    has_tab = true;
                    self.advance();
                }
                _ => break,
            }
        }
        (count, has_tab)
    }

    /// Handle indentation at line start, emitting Indent/Dedent tokens.
    fn handle_indentation(&mut self) -> Option<Token> {
        let start = self.current_position();
        let (spaces, has_tab) = self.count_leading_spaces();

        if has_tab {
            return Some(
                self.make_token(TokenKind::Error("Tabs not allowed in indentation".into()), start),
            );
        }

        let &current_indent = self.indent_stack.last().unwrap_or(&0);

        match spaces.cmp(&current_indent) {
            std::cmp::Ordering::Greater => {
                self.indent_stack.push(spaces);
                Some(self.make_token(TokenKind::Indent, start))
            }
            std::cmp::Ordering::Less => {
                // Pop all levels deeper than current indentation
                while self.indent_stack.last().is_some_and(|&top| top > spaces) {
                    self.indent_stack.pop();
                    self.pending_dedents += 1;
                }

                // Verify we matched a valid indent level
                if self.indent_stack.last().copied() != Some(spaces) && spaces != 0 {
                    return Some(self.make_token(
                        TokenKind::Error(format!(
                            "Indentation mismatch: {spaces} spaces does not match any previous level"
                        )),
                        start,
                    ));
                }

                self.pending_dedents.checked_sub(1).map(|remaining| {
                    self.pending_dedents = remaining;
                    self.make_token(TokenKind::Dedent, start)
                })
            }
            std::cmp::Ordering::Equal => None,
        }
    }

    /// Create a Token with span from start_pos to current position.
    fn make_token(&self, kind: TokenKind, start: Position) -> Token {
        let end = self.current_position();
        let span = Span::new(start, end);
        Token::new(kind, span)
    }

    /// Scan a single structural character: : , [ ] { } -
    ///
    /// # Panics
    /// Unreachable panic if called with non-structural character (defensive).
    fn scan_structural(&mut self, ch: char) -> Token {
        let start = self.current_position();
        self.advance();
        let kind = match ch {
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '-' => TokenKind::Dash,
            _ => unreachable!("scan_structural called with non-structural char: {}", ch),
        };
        self.make_token(kind, start)
    }

    /// Scan newline token, handling both LF and CRLF.
    ///
    /// # Windows Compatibility
    /// CRLF sequences are handled by skip_whitespace() which removes `\r`.
    /// This method only needs to handle `\n`.
    fn scan_newline(&mut self) -> Token {
        let start = self.current_position();
        self.advance(); // consume \n
        self.at_line_start = true;
        self.make_token(TokenKind::Newline, start)
    }

    /// Scan identifier or keyword (true/false/null).
    ///
    /// # Grammar
    /// Identifiers match: `^[A-Za-z_][A-Za-z0-9_]*$`
    /// Keywords are contextual — parsed as identifiers first, then
    /// converted to keyword tokens for cleaner separation of concerns.
    fn scan_identifier_or_keyword(&mut self) -> Token {
        let start = self.current_position();
        let start_offset = self.offset as usize;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start_offset..self.offset as usize];
        let kind = match text {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => TokenKind::Identifier(text.into()),
        };
        self.make_token(kind, start)
    }

    /// Check if current position looks like start of a number.
    ///
    /// Numbers start with digit or `-` followed by digit.
    ///
    /// # Design Rationale
    /// Distinguishes numbers from identifiers and dash tokens.
    /// The lookahead for `-` is critical: `-` followed by space is a Dash token,
    /// but `-` followed by digit is a negative number.
    fn is_number_start(&mut self) -> bool {
        match self.peek() {
            Some('0'..='9') => true,
            Some('-') => matches!(self.peek_next(), Some('0'..='9')),
            _ => false,
        }
    }

    /// Scan a number literal (integer, float, or scientific notation).
    ///
    /// Returns String token if number format is invalid (e.g., leading zeros).
    ///
    /// # Grammar
    /// Valid numbers match:
    /// - Integer: `-?[1-9][0-9]*|0`
    /// - Float: `-?([1-9][0-9]*|0)\.[0-9]+`
    /// - Scientific: `<int|float>[eE][+-]?[0-9]+`
    ///
    /// # Design Rationale
    /// Leading zeros (e.g., `05`) are treated as strings per TOON spec.
    /// This prevents ambiguity with octal notation and maintains JSON compatibility.
    /// The special case `-0` is a valid number (IEEE 754 signed zero).
    fn scan_number(&mut self) -> Token {
        let start = self.current_position();
        let start_offset = self.offset as usize;

        // Handle optional negative sign
        if self.peek() == Some('-') {
            self.advance();
        }

        // Check for hexadecimal literal: `0x` / `0X` followed by hex digits
        if self.peek() == Some('0') && matches!(self.peek_next(), Some('x' | 'X')) {
            self.advance(); // '0'
            self.advance(); // 'x' / 'X'
            while let Some(ch) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    self.advance();
                } else {
                    break;
                }
            }
            let text = &self.source[start_offset..self.offset as usize];
            return self.make_token(TokenKind::Number(text.to_string()), start);
        }

        // Check for leading zero (invalid unless just "0" or "-0")
        let first_digit = self.peek();
        if first_digit == Some('0') {
            self.advance();
            if let Some(ch) = self.peek()
                && ch.is_ascii_digit()
            {
                // Leading zero like "05" - treat as string
                while let Some(ch) = self.peek() {
                    if !ch.is_ascii_alphanumeric() && ch != '.' && ch != '_' {
                        break;
                    }
                    self.advance();
                }
                let text = &self.source[start_offset..self.offset as usize];
                return self.make_token(TokenKind::String(text.to_string()), start);
            }
        } else {
            // Consume integer part
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check for decimal part
        if self.peek() == Some('.')
            && let Some(ch) = self.peek_next()
            && ch.is_ascii_digit()
        {
            self.advance(); // consume '.'
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check for exponent part
        if let Some(ch) = self.peek()
            && (ch == 'e' || ch == 'E')
        {
            self.advance();
            // Optional sign
            if let Some(sign) = self.peek()
                && (sign == '+' || sign == '-')
            {
                self.advance();
            }
            // Exponent digits
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let text = &self.source[start_offset..self.offset as usize];
        self.make_token(TokenKind::Number(text.to_string()), start)
    }

    /// Scan a quoted string literal with escape sequence processing.
    ///
    /// # Grammar
    /// - Starts and ends with `"`
    /// - Escape sequences: `\\`, `\"`, `\n`, `\r`, `\t`
    /// - Invalid escapes produce Error token
    /// - Unterminated strings produce Error token
    fn scan_quoted_string(&mut self) -> Token {
        let start = self.current_position();
        self.advance(); // consume opening "

        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    // Unterminated string
                    return self.make_token(
                        TokenKind::Error("Unterminated string literal".to_string()),
                        start,
                    );
                }
                Some('"') => {
                    self.advance(); // consume closing "
                    break;
                }
                Some('\\') => {
                    self.advance(); // consume backslash
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            value.push('\n');
                        }
                        Some('r') => {
                            self.advance();
                            value.push('\r');
                        }
                        Some('t') => {
                            self.advance();
                            value.push('\t');
                        }
                        Some('"') => {
                            self.advance();
                            value.push('"');
                        }
                        Some('\\') => {
                            self.advance();
                            value.push('\\');
                        }
                        Some(ch) => {
                            return self.make_token(
                                TokenKind::Error(format!("Invalid escape sequence: \\{}", ch)),
                                start,
                            );
                        }
                        None => {
                            return self.make_token(
                                TokenKind::Error("Unterminated escape sequence".to_string()),
                                start,
                            );
                        }
                    }
                }
                Some(ch) => {
                    self.advance();
                    value.push(ch);
                }
            }
        }

        self.make_token(TokenKind::String(value), start)
    }

    /// Scan a triple-quoted block string literal.
    ///
    /// # Grammar
    /// - Starts and ends with `"""`
    /// - Content is preserved verbatim (including newlines, no escape processing)
    /// - Terminates at the first `"""`
    /// - Unterminated block strings produce an Error token
    fn scan_block_string(&mut self) -> Token {
        let start = self.current_position();
        // Consume opening `"""`
        self.advance();
        self.advance();
        self.advance();

        let start_offset = self.offset as usize;

        loop {
            match self.peek() {
                None => {
                    return self.make_token(
                        TokenKind::Error("Unterminated block string literal".to_string()),
                        start,
                    );
                }
                Some('"') if self.is_closing_triple() => {
                    // Consume closing `"""`
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }
                Some(_) => {
                    self.advance();
                }
            }
        }

        let text = &self.source[start_offset..self.offset as usize];
        self.make_token(TokenKind::String(text.to_string()), start)
    }

    /// Scan a reference / environment substitution token: `${ ... }`.
    ///
    /// # Grammar
    /// - Starts with `${` and ends with `}`
    /// - Interior is preserved verbatim (e.g. `foo.bar` or `env:VAR`)
    /// - Unterminated references produce an Error token
    fn scan_reference(&mut self) -> Token {
        let start = self.current_position();
        // Consume `$` and `{`
        self.advance();
        self.advance();

        let start_offset = self.offset as usize;

        loop {
            match self.peek() {
                None => {
                    return self.make_token(
                        TokenKind::Error("Unterminated reference literal".to_string()),
                        start,
                    );
                }
                Some('}') => {
                    self.advance(); // consume closing `}`
                    break;
                }
                Some(_) => {
                    self.advance();
                }
            }
        }

        let raw = &self.source[start_offset..self.offset as usize - 1];
        self.make_token(TokenKind::Reference(raw.to_string()), start)
    }

    /// Scan all tokens from the source.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_lsp::parser::{Scanner, TokenKind};
    ///
    /// let mut scanner = Scanner::new("key: [1, 2, 3]");
    /// let tokens = scanner.scan_all();
    ///
    /// // Check we got the expected tokens: identifier, colon, bracket, numbers, etc.
    /// assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Identifier(ref s) if s == "key")));
    /// assert!(tokens.iter().any(|t| t.kind == TokenKind::Colon));
    /// assert!(tokens.iter().any(|t| t.kind == TokenKind::LeftBracket));
    /// assert!(tokens.iter().any(|t| t.kind == TokenKind::RightBracket));
    /// ```
    pub fn scan_all(&mut self) -> Vec<Token> {
        // Pre-size the token buffer from the source length to avoid repeated
        // doubling-reallocations on large documents. Tokens are typically a
        // handful of bytes each, so `len/16` is a reasonable upper bound that
        // keeps reallocations to at most one or two for big inputs.
        let capacity = self.source.len() / 16 + 1024;
        let mut tokens = Vec::with_capacity(capacity);
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Scan the next token.
    ///
    /// # Token Emission Strategy
    /// 1. Emit pending dedents first (from indent stack unwinding)
    /// 2. Handle indentation at line start (emit Indent/Dedent)
    /// 3. Skip whitespace (spaces, tabs, `\r`)
    /// 4. Dispatch based on next character:
    ///    - Structural: `:`, `,`, `[`, `]`, `{`, `}`
    ///    - Dash: `-` (only if not followed by digit)
    ///    - Newline: `\n`
    ///    - String: `"`
    ///    - Number: digits or `-` followed by digit
    ///    - Identifier/Keyword: `[A-Za-z_]`
    ///    - Error: Unknown character
    ///
    /// This implementation handles number literals including negative numbers.
    /// The `-` character requires lookahead to distinguish between Dash token and
    /// negative number: `-` followed by space is Dash, `-` followed by digit is number.
    pub fn next_token(&mut self) -> Token {
        // Emit pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            let pos = self.current_position();
            return self.make_token(TokenKind::Dedent, pos);
        }

        // Handle indentation at line start
        if self.at_line_start {
            self.at_line_start = false;
            if let Some(token) = self.handle_indentation() {
                return token;
            }
        }

        self.skip_trivia();

        let start = self.current_position();

        let Some(ch) = self.peek() else {
            // Flush any outstanding indentation as dedents before EOF so that
            // blocks ending at end-of-input (no trailing newline) are closed.
            if self.indent_stack.last().copied().unwrap_or(0) > 0 {
                self.indent_stack.pop();
                return self.make_token(TokenKind::Dedent, start);
            }
            return self.make_token(TokenKind::Eof, start);
        };

        // Check for numbers BEFORE checking for dash or identifier
        if self.is_number_start() {
            return self.scan_number();
        }

        match ch {
            ':' | ',' | '[' | ']' | '{' | '}' | '-' => self.scan_structural(ch),
            '\n' => self.scan_newline(),
            '"' => {
                // Triple-quoted block strings start with `"""`.
                if self.peek_next() == Some('"') && self.peek_next2() == Some('"') {
                    self.scan_block_string()
                } else {
                    self.scan_quoted_string()
                }
            }
            '$' => {
                // References / env substitution start with `${`.
                if self.peek_next() == Some('{') {
                    self.scan_reference()
                } else {
                    self.advance();
                    self.make_token(TokenKind::Error("Unexpected character: $".into()), start)
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier_or_keyword(),
            // Control characters (except handled \n, \r, \t) are invalid
            '\x00'..='\x08' | '\x0B' | '\x0C' | '\x0E'..='\x1F' | '\x7F' => {
                self.advance();
                self.make_token(
                    TokenKind::Error(format!("Invalid control character: U+{:04X}", ch as u32)),
                    start,
                )
            }
            _ => {
                // Unknown character - emit error and advance
                self.advance();
                self.make_token(TokenKind::Error(format!("Unexpected character: {}", ch)), start)
            }
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    /// Yields tokens including EOF once, then returns None.
    ///
    /// # Design Rationale
    /// - `scan_all()` includes EOF in the vector for parser use
    /// - Iterator yields EOF once for consistency, then stops
    /// - Prevents infinite EOF token generation
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let token = self.next_token();

        // Mark done after yielding EOF
        if matches!(token.kind, TokenKind::Eof) {
            self.done = true;
        }

        Some(token)
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
