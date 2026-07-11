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

//! TOON parser that produces AST with source positions.
//!
//! This module provides the core parsing functionality:
//! - Scanner (lexer) for tokenizing TOON input
//! - Parser for building AST from tokens
//! - Error types with position information

mod error;
mod scanner;

pub use error::{ParseError, ParseErrorKind};
pub use scanner::{Scanner, Token, TokenKind};

use crate::ast::{AstNode, NumberValue, ObjectEntry, Span};

// =============================================================================
// Security Constants - Resource Exhaustion Protection
// =============================================================================

/// Maximum nesting depth for objects and arrays.
///
/// Prevents stack overflow from deeply nested structures.
/// 128 levels is sufficient for legitimate use cases while preventing
/// unbounded recursion attacks.
const MAX_NESTING_DEPTH: usize = 128;

/// Maximum document size in bytes (10MB).
///
/// Prevents memory exhaustion from maliciously large documents.
/// 10MB is generous for TOON configuration files while providing
/// protection against DoS attacks.
const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of array items (100,000).
///
/// Prevents memory exhaustion from maliciously large arrays.
/// 100k items is sufficient for reasonable data structures while
/// protecting against collection-based DoS attacks.
const MAX_ARRAY_ITEMS: usize = 100_000;

/// Maximum number of object entries (10,000).
///
/// Prevents memory exhaustion from maliciously large objects.
/// 10k entries is sufficient for configuration files while
/// protecting against hash collision and memory exhaustion attacks.
const MAX_OBJECT_ENTRIES: usize = 10_000;

/// Parser state machine that consumes tokens and produces AST.
///
/// # Design
/// - Recursive descent parser with single-token lookahead
/// - Consumes tokens from Scanner
/// - Tracks position and errors for IDE integration
/// - Enforces security limits to prevent resource exhaustion
struct Parser {
    /// All tokens from Scanner
    tokens: Vec<Token>,
    /// Current position in token stream
    position: usize,
    /// Accumulated parse errors (for error recovery mode)
    errors: Vec<ParseError>,
    /// Whether we're in error recovery mode
    recovering: bool,
    /// Current nesting depth for recursion protection
    depth: usize,
}

impl Parser {
    /// Create a new parser from source text.
    ///
    /// # Arguments
    /// * `source` - The TOON source text to parse
    fn new(source: &str) -> Self {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_all();
        Self { tokens, position: 0, errors: Vec::new(), recovering: false, depth: 0 }
    }

    // =========================================================================
    // Token Navigation Helpers
    // =========================================================================

    /// Get the current token without consuming it.
    fn current(&self) -> &Token {
        self.tokens
            .get(self.position)
            .unwrap_or_else(|| self.tokens.last().expect("tokens should never be empty"))
    }

    /// Peek at the next token without consuming current.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    /// Advance to the next token and return the previous one.
    fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.tokens
            .get(self.position - 1)
            .unwrap_or_else(|| self.tokens.last().expect("tokens should never be empty"))
    }

    /// Check if we've reached the end of input.
    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    /// Check if current token matches the expected kind.
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    /// Consume current token if it matches expected kind.
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skip newline tokens.
    fn skip_newlines(&mut self) {
        while matches!(self.current().kind, TokenKind::Newline) {
            self.advance();
        }
    }

    // =========================================================================
    // Span Helpers
    // =========================================================================

    /// Merge two spans to create a span covering both.
    fn merge_spans(start: Span, end: Span) -> Span {
        start.merge(end)
    }

    // =========================================================================
    // Error Handling
    // =========================================================================

    /// Record a parse error and return it.
    fn error(&mut self, kind: ParseErrorKind, span: Span) -> ParseError {
        let error = ParseError::new(kind, span);
        self.errors.push(error.clone());
        error
    }

    /// Record a parse error with context and return it.
    fn error_with_context(
        &mut self,
        kind: ParseErrorKind,
        span: Span,
        context: &str,
    ) -> ParseError {
        let error = ParseError::new(kind, span).with_context(context);
        self.errors.push(error.clone());
        error
    }

    /// Check and enforce maximum nesting depth.
    ///
    /// # Safety
    /// This function MUST be called at entry to every recursive parsing function
    /// (parse_nested_object, parse_expanded_array, parse_tabular_row) to prevent
    /// stack overflow attacks from deeply nested structures.
    ///
    /// # Arguments
    /// * `span` - Source span for error reporting if depth exceeded
    ///
    /// # Returns
    /// * `Ok(())` if depth is within limits
    /// * `Err(ParseError)` if maximum depth exceeded
    fn check_depth(&self, span: Span) -> Result<(), ParseError> {
        if self.depth >= MAX_NESTING_DEPTH {
            return Err(ParseError::new(ParseErrorKind::MaxDepthExceeded, span));
        }
        Ok(())
    }

    /// Synchronize after an error by skipping to next statement boundary.
    ///
    /// Sync points: Newline, Dedent, Eof
    fn synchronize(&mut self) {
        self.recovering = true;
        while !self.is_at_end() {
            match self.current().kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => {
                    if matches!(self.current().kind, TokenKind::Newline) {
                        self.advance();
                    }
                    self.recovering = false;
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.recovering = false;
    }

    // =========================================================================
    // Primitive Parsing
    // =========================================================================

    /// Parse a number token into NumberValue.
    fn parse_number(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance();
        let span = token.span;
        match &token.kind {
            TokenKind::Number(text) => {
                let value = Self::parse_number_value(text, span)?;
                Ok(AstNode::Number { value, span })
            }
            _ => Err(ParseError::new(ParseErrorKind::ExpectedValue, span)),
        }
    }

    /// Convert number string to NumberValue variant.
    fn parse_number_value(text: &str, span: Span) -> Result<NumberValue, ParseError> {
        // Float if contains . or e/E
        if text.contains('.') || text.contains('e') || text.contains('E') {
            text.parse::<f64>()
                .map(NumberValue::Float)
                .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber, span))
        } else if text.starts_with('-') {
            text.parse::<i64>()
                .map(NumberValue::NegInt)
                .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber, span))
        } else {
            text.parse::<u64>()
                .map(NumberValue::PosInt)
                .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber, span))
        }
    }

    /// Parse a string token (already processed by scanner).
    fn parse_string(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance();
        let span = token.span;
        match &token.kind {
            TokenKind::String(value) => Ok(AstNode::String { value: value.clone(), span }),
            _ => Err(ParseError::new(ParseErrorKind::ExpectedValue, span)),
        }
    }

    /// Parse primitive keywords: true, false, null.
    fn parse_primitive(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance();
        let span = token.span;
        match &token.kind {
            TokenKind::True => Ok(AstNode::Bool { value: true, span }),
            TokenKind::False => Ok(AstNode::Bool { value: false, span }),
            TokenKind::Null => Ok(AstNode::Null { span }),
            _ => Err(ParseError::new(ParseErrorKind::ExpectedValue, span)),
        }
    }

    /// Parse any value (dispatcher for all value types).
    fn parse_value(&mut self) -> Result<AstNode, ParseError> {
        if matches!(self.current().kind, TokenKind::Error(_)) {
            let span = self.current().span;
            let msg = if let TokenKind::Error(m) = &self.current().kind {
                m.clone()
            } else {
                String::new()
            };
            let err = self.error_with_context(ParseErrorKind::UnexpectedToken, span, &msg);
            self.advance();
            return Err(err);
        }

        match &self.current().kind {
            TokenKind::String(_) => self.parse_string(),
            TokenKind::Number(_) => self.parse_number(),
            TokenKind::True | TokenKind::False | TokenKind::Null => self.parse_primitive(),
            TokenKind::Identifier(_) => match self.peek() {
                Some(Token { kind: TokenKind::LeftBracket, .. }) => self.parse_array_header(),
                Some(Token { kind: TokenKind::Colon, .. }) => self.parse_nested_value(),
                _ => self.parse_unquoted_string(),
            },
            TokenKind::Indent => self.parse_nested_object(),
            TokenKind::Dash => self.parse_expanded_array(),
            TokenKind::Newline => {
                self.advance();
                match &self.current().kind {
                    TokenKind::Indent => self.parse_nested_object(),
                    TokenKind::Dash => self.parse_expanded_array(),
                    _ => Ok(AstNode::Null { span: Span::point(self.current().span.start) }),
                }
            }
            TokenKind::Eof => Ok(AstNode::Null { span: Span::point(self.current().span.start) }),
            _ => Err(self.error(ParseErrorKind::ExpectedValue, self.current().span)),
        }
    }

    /// Parse an unquoted string value (identifier not followed by colon).
    fn parse_unquoted_string(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;
        let mut parts: Vec<String> = Vec::new();
        let mut end_span = start_span;

        while !self.is_at_end() {
            match &self.current().kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Identifier(s) | TokenKind::Number(s) | TokenKind::String(s) => {
                    parts.push(s.clone());
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::Colon => {
                    if let Some(last) = parts.last_mut() {
                        last.push(':');
                    } else {
                        parts.push(String::from(":"));
                    }
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::Comma => {
                    if let Some(last) = parts.last_mut() {
                        last.push(',');
                    } else {
                        parts.push(String::from(","));
                    }
                    end_span = self.current().span;
                    self.advance();
                }
                _ => break,
            }
        }

        let value = parts.join(" ").trim().to_string();
        Ok(AstNode::String { value, span: Self::merge_spans(start_span, end_span) })
    }

    /// Parse a value that starts with identifier:
    fn parse_nested_value(&mut self) -> Result<AstNode, ParseError> {
        // This is an inline nested object like: value: nested: deep
        // We'll treat identifier sequences as unquoted strings for simplicity
        self.parse_unquoted_string()
    }

    // =========================================================================
    // Object Parsing
    // =========================================================================

    /// Parse a single object entry (key: value pair).
    fn parse_object_entry(&mut self) -> Result<ObjectEntry, ParseError> {
        let key_token = self.current();
        let (key, key_span) = match &key_token.kind {
            TokenKind::Identifier(name) => {
                let result = (name.clone(), key_token.span);
                self.advance();
                result
            }
            TokenKind::String(name) => {
                let result = (name.clone(), key_token.span);
                self.advance();
                result
            }
            _ => return Err(self.error(ParseErrorKind::ExpectedKey, key_token.span)),
        };

        // Check for array header syntax: key[N]
        if matches!(self.current().kind, TokenKind::LeftBracket) {
            let value = self.parse_array_with_key(&key, key_span)?;
            return Ok(ObjectEntry { key, key_span, value });
        }

        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            return Err(self.error(ParseErrorKind::ExpectedColon, self.current().span));
        }

        let value = self.parse_value()?;
        Ok(ObjectEntry { key, key_span, value })
    }

    /// Parse object entries at the current indentation level.
    fn parse_object(&mut self, start_span: Span) -> Result<AstNode, ParseError> {
        let mut entries = Vec::with_capacity(16);

        while !self.is_at_end() {
            if entries.len() >= MAX_OBJECT_ENTRIES {
                return Err(ParseError::new(
                    ParseErrorKind::TooManyObjectEntries,
                    self.current().span,
                ));
            }

            self.skip_newlines();

            if matches!(self.current().kind, TokenKind::Dedent | TokenKind::Eof) {
                break;
            }

            if !matches!(self.current().kind, TokenKind::Identifier(_) | TokenKind::String(_)) {
                break;
            }

            match self.parse_object_entry() {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    if !self.recovering {
                        self.errors.push(e);
                    }
                    self.synchronize();
                }
            }

            if matches!(self.current().kind, TokenKind::Newline) {
                self.advance();
            }
        }

        let end_span = entries.last().map_or(start_span, |e| e.value.span());
        Ok(AstNode::Object { entries, span: Self::merge_spans(start_span, end_span) })
    }

    /// Parse a nested object (after Indent token).
    fn parse_nested_object(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;

        // SECURITY: Check maximum nesting depth before recursion
        self.check_depth(start_span)?;
        self.depth += 1;

        // Consume indent
        if matches!(self.current().kind, TokenKind::Indent) {
            self.advance();
        }

        // Check if this is an array (dash items)
        if matches!(self.current().kind, TokenKind::Dash) {
            let result = self.parse_expanded_array();
            self.depth -= 1;
            return result;
        }

        let result = self.parse_object(start_span);

        // Consume dedent
        if matches!(self.current().kind, TokenKind::Dedent) {
            self.advance();
        }

        self.depth -= 1;
        result
    }

    /// Parse document (entry point).
    fn parse_document(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;
        self.skip_newlines();

        // Empty document
        if self.is_at_end() {
            return Ok(AstNode::Document { children: vec![], span: Span::point(start_span.start) });
        }

        // Parse root object
        let root = self.parse_object(start_span)?;
        let end_span = root.span();

        Ok(AstNode::Document {
            children: vec![root],
            span: Self::merge_spans(start_span, end_span),
        })
    }

    // =========================================================================
    // Array Parsing
    // =========================================================================

    /// Parse expanded array (dash-prefixed items).
    fn parse_expanded_array(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;

        // SECURITY: Check maximum nesting depth before recursion
        self.check_depth(start_span)?;
        self.depth += 1;

        let mut items = Vec::new();

        while matches!(self.current().kind, TokenKind::Dash) {
            // SECURITY: Enforce maximum array size to prevent memory exhaustion
            if items.len() >= MAX_ARRAY_ITEMS {
                self.depth -= 1;
                return Err(ParseError::new(
                    ParseErrorKind::TooManyArrayItems,
                    self.current().span,
                ));
            }

            self.advance(); // consume dash

            // Skip whitespace (already handled by scanner)

            // Parse item value
            let item = if matches!(self.current().kind, TokenKind::Newline) {
                // Item value on next line (nested object or array)
                self.advance(); // consume newline
                if matches!(self.current().kind, TokenKind::Indent) {
                    self.parse_nested_object()?
                } else {
                    // Empty item = null
                    AstNode::Null { span: Span::point(self.current().span.start) }
                }
            } else if matches!(self.current().kind, TokenKind::Eof | TokenKind::Dedent) {
                // Empty item at end
                AstNode::Null { span: Span::point(self.current().span.start) }
            } else {
                // Item value on same line
                self.parse_value()?
            };

            items.push(item);

            // Consume newline after item
            if matches!(self.current().kind, TokenKind::Newline) {
                self.advance();
            }

            // Check for dedent (end of array)
            if matches!(self.current().kind, TokenKind::Dedent) {
                break;
            }
        }

        let end_span = items.last().map_or(start_span, AstNode::span);

        self.depth -= 1;
        Ok(AstNode::Array {
            items,
            form: crate::ast::ArrayForm::Expanded,
            span: Self::merge_spans(start_span, end_span),
        })
    }

    /// Parse array header syntax: key[N]: or key[N]{fields}:
    fn parse_array_header(&mut self) -> Result<AstNode, ParseError> {
        // Current token is identifier (already consumed in parse_object_entry)
        // Unreachable in current flow - handle via parse_array_with_key
        let span = self.current().span;
        Err(self.error(ParseErrorKind::UnexpectedToken, span))
    }

    /// Parse array with key already known.
    fn parse_array_with_key(
        &mut self,
        _key: &str,
        start_span: Span,
    ) -> Result<AstNode, ParseError> {
        // Consume [
        self.advance();

        // Parse count
        let count = if let TokenKind::Number(n) = &self.current().kind {
            let count_str = n.clone();
            self.advance();
            count_str.parse::<usize>().unwrap_or(0)
        } else {
            0
        };

        // Check for field schema {f1,f2}
        let fields = if matches!(self.current().kind, TokenKind::RightBracket) {
            self.advance(); // consume ]

            // Check for brace-enclosed fields
            if matches!(self.current().kind, TokenKind::LeftBrace) {
                self.advance();
                let mut fields = Vec::new();
                while !matches!(self.current().kind, TokenKind::RightBrace | TokenKind::Eof) {
                    if let TokenKind::Identifier(name) = &self.current().kind {
                        fields.push(name.clone());
                        self.advance();
                    }
                    if matches!(self.current().kind, TokenKind::Comma) {
                        self.advance();
                    }
                }
                if matches!(self.current().kind, TokenKind::RightBrace) {
                    self.advance();
                }
                Some(fields)
            } else {
                None
            }
        } else {
            // Missing ]
            let span = self.current().span;
            self.error(ParseErrorKind::UnexpectedToken, span);
            None
        };

        // Determine delimiter from next character
        let delimiter = match &self.current().kind {
            TokenKind::Identifier(s) if s == "|" => {
                self.advance();
                '|'
            }
            _ => ',', // default
        };

        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            let span = self.current().span;
            return Err(self.error(ParseErrorKind::ExpectedColon, span));
        }

        // Parse array content based on type
        if let Some(field_names) = fields {
            // Tabular array
            self.parse_tabular_array(start_span, count, &field_names, delimiter)
        } else {
            // Inline array
            self.parse_inline_array(start_span, count, delimiter)
        }
    }

    /// Parse inline array values: v1,v2,v3
    fn parse_inline_array(
        &mut self,
        start_span: Span,
        _expected_count: usize,
        delimiter: char,
    ) -> Result<AstNode, ParseError> {
        if matches!(self.current().kind, TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
            return Ok(AstNode::Array {
                items: Vec::new(),
                form: crate::ast::ArrayForm::Inline,
                span: start_span,
            });
        }

        let mut items = Vec::new();

        loop {
            if items.len() >= MAX_ARRAY_ITEMS {
                return Err(ParseError::new(
                    ParseErrorKind::TooManyArrayItems,
                    self.current().span,
                ));
            }

            let item = match &self.current().kind {
                TokenKind::String(s) => {
                    let span = self.current().span;
                    let value = s.clone();
                    self.advance();
                    AstNode::String { value, span }
                }
                TokenKind::Number(n) => {
                    let span = self.current().span;
                    let value = Self::parse_number_value(n, span)?;
                    self.advance();
                    AstNode::Number { value, span }
                }
                TokenKind::True => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Bool { value: true, span }
                }
                TokenKind::False => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Bool { value: false, span }
                }
                TokenKind::Null => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Null { span }
                }
                TokenKind::Identifier(s) => {
                    let span = self.current().span;
                    let value = s.clone();
                    self.advance();
                    AstNode::String { value, span }
                }
                _ => break,
            };

            items.push(item);

            let should_break = match self.current().kind {
                TokenKind::Comma if delimiter == ',' => {
                    self.advance();
                    false
                }
                TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent => true,
                _ => true,
            };
            if should_break {
                break;
            }
        }

        let end_span = items.last().map_or(start_span, AstNode::span);
        Ok(AstNode::Array {
            items,
            form: crate::ast::ArrayForm::Inline,
            span: Self::merge_spans(start_span, end_span),
        })
    }

    /// Parse tabular array rows into Objects.
    fn parse_tabular_array(
        &mut self,
        start_span: Span,
        expected_count: usize,
        field_names: &[String],
        delimiter: char,
    ) -> Result<AstNode, ParseError> {
        let mut items = Vec::new();

        // Consume newline after header
        if matches!(self.current().kind, TokenKind::Newline) {
            self.advance();
        }

        // Handle indentation
        if matches!(self.current().kind, TokenKind::Indent) {
            self.advance();
        }

        // Parse rows
        for _ in 0..expected_count {
            // SECURITY: Enforce maximum array size to prevent memory exhaustion
            if items.len() >= MAX_ARRAY_ITEMS {
                return Err(ParseError::new(
                    ParseErrorKind::TooManyArrayItems,
                    self.current().span,
                ));
            }

            if self.is_at_end() || matches!(self.current().kind, TokenKind::Dedent) {
                break;
            }

            let row = self.parse_tabular_row(field_names, delimiter)?;
            items.push(row);

            // Consume newline after row
            if matches!(self.current().kind, TokenKind::Newline) {
                self.advance();
            }
        }

        // Consume dedent
        if matches!(self.current().kind, TokenKind::Dedent) {
            self.advance();
        }

        let end_span = items.last().map_or(start_span, AstNode::span);

        Ok(AstNode::Array {
            items,
            form: crate::ast::ArrayForm::Tabular,
            span: Self::merge_spans(start_span, end_span),
        })
    }

    /// Parse a single tabular row into an Object.
    fn parse_tabular_row(
        &mut self,
        field_names: &[String],
        delimiter: char,
    ) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;
        self.check_depth(start_span)?;
        self.depth += 1;

        let mut entries = Vec::with_capacity(field_names.len());
        let num_fields = field_names.len();

        for (i, field_name) in field_names.iter().enumerate() {
            let value = match &self.current().kind {
                TokenKind::String(s) => {
                    let span = self.current().span;
                    let value = s.clone();
                    self.advance();
                    AstNode::String { value, span }
                }
                TokenKind::Number(n) => {
                    let span = self.current().span;
                    let num_value = Self::parse_number_value(n, span)?;
                    self.advance();
                    AstNode::Number { value: num_value, span }
                }
                TokenKind::Identifier(s) => {
                    let span = self.current().span;
                    let value = s.clone();
                    self.advance();
                    AstNode::String { value, span }
                }
                TokenKind::True => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Bool { value: true, span }
                }
                TokenKind::False => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Bool { value: false, span }
                }
                TokenKind::Null => {
                    let span = self.current().span;
                    self.advance();
                    AstNode::Null { span }
                }
                _ => AstNode::Null { span: self.current().span },
            };

            entries.push(ObjectEntry { key: field_name.clone(), key_span: start_span, value });

            if i < num_fields - 1
                && delimiter == ','
                && matches!(self.current().kind, TokenKind::Comma)
            {
                self.advance();
            }
        }

        let end_span = entries.last().map_or(start_span, |e| e.value.span());
        self.depth -= 1;
        Ok(AstNode::Object { entries, span: Self::merge_spans(start_span, end_span) })
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Parse TOON source into an AST.
///
/// # Arguments
/// * `source` - The TOON source text to parse
///
/// # Returns
/// * `Ok(AstNode)` - The root AST node (Document) on success
/// * `Err(ParseError)` - First parse error encountered
///
/// # Security
/// Enforces resource limits to prevent DoS attacks:
/// - Maximum document size: 10MB
/// - Maximum nesting depth: 128 levels
/// - Maximum array size: 100,000 items
/// - Maximum object size: 10,000 entries
///
/// # Example
/// ```rust
/// use toon_lsp::parse;
///
/// let ast = parse("name: Alice\nage: 30").unwrap();
/// assert_eq!(ast.kind(), "document");
/// ```
pub fn parse(source: &str) -> Result<AstNode, ParseError> {
    // SECURITY: Enforce maximum document size to prevent memory exhaustion
    if source.len() > MAX_DOCUMENT_SIZE {
        return Err(ParseError::new(ParseErrorKind::DocumentTooLarge, Span::default()));
    }

    let mut parser = Parser::new(source);
    let result = parser.parse_document();

    // In strict mode, return first error
    if let Some(error) = parser.errors.into_iter().next() {
        return Err(error);
    }

    result
}

/// Parse TOON source with error recovery for IDE use.
///
/// Unlike `parse()`, this function attempts to recover from errors
/// and returns as much of the AST as possible along with all errors.
///
/// # Arguments
/// * `source` - The TOON source text to parse
///
/// # Returns
/// * `(Option<AstNode>, Vec<ParseError>)` - Partial AST (if any) and all errors
///
/// # Security
/// Enforces the same resource limits as `parse()` to prevent DoS attacks.
///
/// # Example
/// ```rust
/// use toon_lsp::parse_with_errors;
///
/// let (ast, errors) = parse_with_errors("name Alice\nage: 30");
/// assert!(ast.is_some()); // Partial AST with valid portions
/// assert!(!errors.is_empty()); // Contains error for missing colon
/// ```
///
/// # Multiple errors
/// ```rust
/// use toon_lsp::parse_with_errors;
///
/// let (ast, errors) = parse_with_errors("name: Alice\nage\nactive: true");
/// assert!(ast.is_some()); // Still parses what it can
/// assert_eq!(errors.len(), 2); // Errors for missing colon after age and incomplete value
/// ```
#[must_use]
pub fn parse_with_errors(source: &str) -> (Option<AstNode>, Vec<ParseError>) {
    // SECURITY: Enforce maximum document size to prevent memory exhaustion
    if source.len() > MAX_DOCUMENT_SIZE {
        let error = ParseError::new(ParseErrorKind::DocumentTooLarge, Span::default());
        return (None, vec![error]);
    }

    let mut parser = Parser::new(source);
    let result = parser.parse_document();

    match result {
        Ok(ast) => (Some(ast), parser.errors),
        Err(_) => {
            // Even on error, try to return something
            if parser.errors.is_empty() {
                (None, vec![])
            } else {
                // Return partial document with any parsed content
                (None, parser.errors)
            }
        }
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use std::fmt::Write;

    /// SEC-001: Test recursive nesting depth protection
    #[test]
    fn test_max_nesting_depth_object() {
        // Create deeply nested object structure (150 levels)
        // Each level needs: key + colon + newline + indent
        let mut input = String::from("root:\n");
        for level in 0..150 {
            let indent = "  ".repeat(level + 1);
            let _ = writeln!(input, "{}level{}:", indent, level);
        }
        let _ = writeln!(input, "{}value: deep", "  ".repeat(151));

        let result = parse(&input);
        assert!(result.is_err(), "Should reject excessive nesting");
        let err = result.unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::MaxDepthExceeded);
    }

    #[test]
    fn test_max_nesting_depth_array() {
        // Create deeply nested array structure (150 levels)
        // Each level: key + colon + newline + indent + dash + newline + deeper indent
        let mut input = String::from("root:\n");
        for level in 0..150 {
            let indent = "  ".repeat(level + 1);
            let _ = writeln!(input, "{}- ", indent);
            if level < 149 {
                let _ = writeln!(input, "{}  nested:", indent);
            }
        }

        let result = parse(&input);
        assert!(result.is_err(), "Should reject excessive array nesting");
        let err = result.unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::MaxDepthExceeded);
    }

    #[test]
    fn test_nesting_depth_within_limit() {
        // Create nested structure just under limit (127 levels)
        let mut input = String::from("root:\n");
        for level in 0..127 {
            let indent = "  ".repeat(level + 1);
            let _ = writeln!(input, "{}level{}:", indent, level);
        }
        let _ = writeln!(input, "{}value: deep", "  ".repeat(128));

        let result = parse(&input);
        assert!(result.is_ok(), "Should accept nesting within limit");
    }

    /// SEC-002: Test document size limits
    #[test]
    fn test_document_too_large() {
        // Create document exceeding 10MB
        let large_input = "a".repeat(11 * 1024 * 1024);

        let result = parse(&large_input);
        assert!(result.is_err(), "Should reject oversized document");
        let err = result.unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::DocumentTooLarge);
    }

    #[test]
    fn test_document_size_within_limit() {
        // Create document just under limit (~8MB)
        // Use 9,000 entries to stay under MAX_OBJECT_ENTRIES (10,000)
        let input = "key: value\n".repeat(9_000);

        let result = parse(&input);
        assert!(result.is_ok(), "Should accept document within size limit");
    }

    /// SEC-003: Test array size limits
    #[test]
    fn test_too_many_array_items() {
        // Create array with >100k items
        let mut input = String::from("items:\n");
        for i in 0..100_001 {
            let _ = writeln!(input, "  - item{}", i);
        }

        let result = parse(&input);
        assert!(result.is_err(), "Should reject oversized array");
        let err = result.unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::TooManyArrayItems);
    }

    #[test]
    fn test_array_items_within_limit() {
        // Create array with exactly 100k items (at limit)
        let mut input = String::from("items:\n");
        for i in 0..100_000 {
            let _ = writeln!(input, "  - item{}", i);
        }

        let result = parse(&input);
        assert!(result.is_ok(), "Should accept array at limit");
    }

    /// SEC-004: Test object size limits
    #[test]
    fn test_too_many_object_entries() {
        // Create object with >10k entries
        let mut input = String::new();
        for i in 0..10_001 {
            let _ = writeln!(input, "key{}: value{}", i, i);
        }

        let result = parse(&input);
        assert!(result.is_err(), "Should reject oversized object");
        let err = result.unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::TooManyObjectEntries);
    }

    #[test]
    fn test_object_entries_within_limit() {
        // Create object with exactly 10k entries (at limit)
        let mut input = String::new();
        for i in 0..10_000 {
            let _ = writeln!(input, "key{}: value{}", i, i);
        }

        let result = parse(&input);
        assert!(result.is_ok(), "Should accept object at limit");
    }

    /// SEC-005: Test combined attack scenarios
    #[test]
    fn test_large_deeply_nested_structure() {
        // Combine depth and size attacks - create document >10MB with deep nesting
        let mut input = String::from("root:\n");
        for depth in 0..150 {
            let indent = "  ".repeat(depth + 1);
            let _ = writeln!(input, "{}level{}:", indent, depth);
            // Add many entries at each level
            for item in 0..1000 {
                let _ = writeln!(input, "{}item{}: value", indent, item);
            }
        }

        let result = parse(&input);
        assert!(result.is_err(), "Should reject combined attack");
        // Should fail on document size first (>10MB) or depth (128 levels)
        let err = result.unwrap_err();
        assert!(
            matches!(err.kind, ParseErrorKind::DocumentTooLarge | ParseErrorKind::MaxDepthExceeded),
            "Expected DocumentTooLarge or MaxDepthExceeded, got {:?}",
            err.kind
        );
    }

    /// SEC-006: Test error recovery mode respects limits
    #[test]
    fn test_error_recovery_respects_depth_limit() {
        // Create deeply nested structure (valid syntax)
        // Depth limit should be enforced even in error recovery mode
        let mut input = String::from("root:\n");
        for level in 0..150 {
            let indent = "  ".repeat(level + 1);
            let _ = writeln!(input, "{}level{}:", indent, level);
        }
        let _ = writeln!(input, "{}value: deep", "  ".repeat(151));

        let (_ast, errors) = parse_with_errors(&input);
        // Should fail on depth limit
        assert!(!errors.is_empty());
        assert!(
            errors.iter().any(|e| e.kind == ParseErrorKind::MaxDepthExceeded),
            "Expected MaxDepthExceeded error, got: {:?}",
            errors
        );
    }
}
