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

/// Parser state machine that consumes tokens and produces AST.
///
/// # Design
/// - Recursive descent parser with single-token lookahead
/// - Consumes tokens from Scanner
/// - Tracks position and errors for IDE integration
struct Parser {
    /// All tokens from Scanner
    tokens: Vec<Token>,
    /// Current position in token stream
    position: usize,
    /// Accumulated parse errors (for error recovery mode)
    errors: Vec<ParseError>,
    /// Whether we're in error recovery mode
    recovering: bool,
}

impl Parser {
    /// Create a new parser from source text.
    ///
    /// # Arguments
    /// * `source` - The TOON source text to parse
    fn new(source: &str) -> Self {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_all();
        Self {
            tokens,
            position: 0,
            errors: Vec::new(),
            recovering: false,
        }
    }

    // =========================================================================
    // Token Navigation Helpers
    // =========================================================================

    /// Get the current token without consuming it.
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or_else(|| {
            // Return last token (should be Eof)
            self.tokens.last().expect("tokens should never be empty")
        })
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

    /// Record a parse error.
    fn error(&mut self, kind: ParseErrorKind, span: Span) -> ParseError {
        let error = ParseError::new(kind, span);
        self.errors.push(error.clone());
        error
    }

    /// Record a parse error with context.
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
        let token = self.advance().clone();
        if let TokenKind::Number(text) = &token.kind {
            let value = Self::parse_number_value(text, token.span)?;
            Ok(AstNode::Number {
                value,
                span: token.span,
            })
        } else {
            Err(ParseError::new(ParseErrorKind::ExpectedValue, token.span))
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
        let token = self.advance().clone();
        if let TokenKind::String(value) = token.kind {
            Ok(AstNode::String {
                value,
                span: token.span,
            })
        } else {
            Err(ParseError::new(ParseErrorKind::ExpectedValue, token.span))
        }
    }

    /// Parse primitive keywords: true, false, null.
    fn parse_primitive(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::True => Ok(AstNode::Bool {
                value: true,
                span: token.span,
            }),
            TokenKind::False => Ok(AstNode::Bool {
                value: false,
                span: token.span,
            }),
            TokenKind::Null => Ok(AstNode::Null { span: token.span }),
            _ => Err(ParseError::new(ParseErrorKind::ExpectedValue, token.span)),
        }
    }

    /// Parse any value (dispatcher for all value types).
    fn parse_value(&mut self) -> Result<AstNode, ParseError> {
        // Handle scanner errors
        if let TokenKind::Error(msg) = &self.current().kind {
            let span = self.current().span;
            let msg_clone = msg.clone();
            let err = self.error_with_context(ParseErrorKind::UnexpectedToken, span, &msg_clone);
            self.advance();
            return Err(err);
        }

        match &self.current().kind {
            TokenKind::String(_) => self.parse_string(),
            TokenKind::Number(_) => self.parse_number(),
            TokenKind::True | TokenKind::False | TokenKind::Null => self.parse_primitive(),
            TokenKind::Identifier(_) => {
                // Could be start of nested object or inline array
                if let Some(next) = self.peek() {
                    match &next.kind {
                        TokenKind::LeftBracket => self.parse_array_header(),
                        TokenKind::Colon => self.parse_nested_value(),
                        _ => self.parse_unquoted_string(),
                    }
                } else {
                    self.parse_unquoted_string()
                }
            }
            TokenKind::Indent => self.parse_nested_object(),
            TokenKind::Dash => self.parse_expanded_array(),
            TokenKind::Newline => {
                // Value on next line - check for indent
                self.advance(); // consume newline
                if matches!(self.current().kind, TokenKind::Indent) {
                    self.parse_nested_object()
                } else if matches!(self.current().kind, TokenKind::Dash) {
                    self.parse_expanded_array()
                } else {
                    // Implicit null
                    let span = self.current().span;
                    Ok(AstNode::Null {
                        span: Span::point(span.start),
                    })
                }
            }
            TokenKind::Eof => {
                // Implicit null at end
                let span = self.current().span;
                Ok(AstNode::Null {
                    span: Span::point(span.start),
                })
            }
            _ => {
                let span = self.current().span;
                Err(self.error(ParseErrorKind::ExpectedValue, span))
            }
        }
    }

    /// Parse an unquoted string value (identifier not followed by colon).
    fn parse_unquoted_string(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;
        let mut text = String::new();
        let mut end_span = start_span;

        // Collect tokens until newline or end
        while !self.is_at_end() {
            match &self.current().kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Identifier(s) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s);
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::Number(s) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s);
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::String(s) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s);
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::Colon => {
                    text.push(':');
                    end_span = self.current().span;
                    self.advance();
                }
                TokenKind::Comma => {
                    text.push(',');
                    end_span = self.current().span;
                    self.advance();
                }
                _ => {
                    // Stop on other tokens
                    break;
                }
            }
        }

        Ok(AstNode::String {
            value: text.trim().to_string(),
            span: Self::merge_spans(start_span, end_span),
        })
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
        // Expect identifier (key)
        let key_token = self.current().clone();
        let (key, key_span) = match &key_token.kind {
            TokenKind::Identifier(name) => {
                self.advance();
                (name.clone(), key_token.span)
            }
            TokenKind::String(name) => {
                self.advance();
                (name.clone(), key_token.span)
            }
            _ => {
                let span = key_token.span;
                return Err(self.error(ParseErrorKind::ExpectedKey, span));
            }
        };

        // Check for array header syntax: key[N]
        if matches!(self.current().kind, TokenKind::LeftBracket) {
            // Rewind isn't possible, so handle inline
            let value = self.parse_array_with_key(&key, key_span)?;
            return Ok(ObjectEntry {
                key,
                key_span,
                value,
            });
        }

        // Expect colon
        if !self.match_token(&TokenKind::Colon) {
            let span = self.current().span;
            return Err(self.error(ParseErrorKind::ExpectedColon, span));
        }

        // Parse value
        let value = self.parse_value()?;

        Ok(ObjectEntry {
            key,
            key_span,
            value,
        })
    }

    /// Parse object entries at the current indentation level.
    fn parse_object(&mut self, start_span: Span) -> Result<AstNode, ParseError> {
        let mut entries = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();

            // Stop at dedent or end
            if matches!(self.current().kind, TokenKind::Dedent | TokenKind::Eof) {
                break;
            }

            // Stop if not at a key
            if !matches!(
                self.current().kind,
                TokenKind::Identifier(_) | TokenKind::String(_)
            ) {
                break;
            }

            match self.parse_object_entry() {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    if !self.recovering {
                        self.errors.push(e);
                    }
                    self.synchronize();
                    continue;
                }
            }

            // Consume trailing newline
            if matches!(self.current().kind, TokenKind::Newline) {
                self.advance();
            }
        }

        let end_span = if let Some(last) = entries.last() {
            last.value.span()
        } else {
            start_span
        };

        Ok(AstNode::Object {
            entries,
            span: Self::merge_spans(start_span, end_span),
        })
    }

    /// Parse a nested object (after Indent token).
    fn parse_nested_object(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;

        // Consume indent
        if matches!(self.current().kind, TokenKind::Indent) {
            self.advance();
        }

        // Check if this is an array (dash items)
        if matches!(self.current().kind, TokenKind::Dash) {
            return self.parse_expanded_array();
        }

        let result = self.parse_object(start_span);

        // Consume dedent
        if matches!(self.current().kind, TokenKind::Dedent) {
            self.advance();
        }

        result
    }

    /// Parse document (entry point).
    fn parse_document(&mut self) -> Result<AstNode, ParseError> {
        let start_span = self.current().span;
        self.skip_newlines();

        // Empty document
        if self.is_at_end() {
            return Ok(AstNode::Document {
                children: vec![],
                span: Span::point(start_span.start),
            });
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
        let mut items = Vec::new();

        while matches!(self.current().kind, TokenKind::Dash) {
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
                    AstNode::Null {
                        span: Span::point(self.current().span.start),
                    }
                }
            } else if matches!(self.current().kind, TokenKind::Eof | TokenKind::Dedent) {
                // Empty item at end
                AstNode::Null {
                    span: Span::point(self.current().span.start),
                }
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

        let end_span = items.last().map(|i| i.span()).unwrap_or(start_span);

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
        let mut items = Vec::new();

        // Handle empty array
        if matches!(
            self.current().kind,
            TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent
        ) {
            return Ok(AstNode::Array {
                items,
                form: crate::ast::ArrayForm::Inline,
                span: start_span,
            });
        }

        // Parse items
        loop {
            // Skip leading whitespace (handled by scanner)

            // Parse value
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
                    // Unquoted string value
                    let span = self.current().span;
                    let value = s.clone();
                    self.advance();
                    AstNode::String { value, span }
                }
                _ => break,
            };

            items.push(item);

            // Check for delimiter
            if delimiter == ',' && matches!(self.current().kind, TokenKind::Comma) {
                self.advance();
            } else if matches!(
                self.current().kind,
                TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent
            ) {
                break;
            } else {
                // For other delimiters or end of items
                break;
            }
        }

        let end_span = items.last().map(|i| i.span()).unwrap_or(start_span);

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

        let end_span = items.last().map(|i| i.span()).unwrap_or(start_span);

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
        let mut entries = Vec::new();

        for (i, field_name) in field_names.iter().enumerate() {
            // Parse value
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
                    AstNode::Number {
                        value: num_value,
                        span,
                    }
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
                _ => {
                    // Missing value
                    AstNode::Null {
                        span: self.current().span,
                    }
                }
            };

            entries.push(ObjectEntry {
                key: field_name.clone(),
                key_span: start_span, // Use row start as key span
                value,
            });

            // Check for delimiter (except after last field)
            // For tab/pipe delimiters, values should already be split by scanner
            if i < field_names.len() - 1
                && delimiter == ','
                && matches!(self.current().kind, TokenKind::Comma)
            {
                self.advance();
            }
        }

        let end_span = entries.last().map(|e| e.value.span()).unwrap_or(start_span);

        Ok(AstNode::Object {
            entries,
            span: Self::merge_spans(start_span, end_span),
        })
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
/// # Example
/// ```rust
/// use toon_lsp::parse;
///
/// let ast = parse("name: Alice\nage: 30").unwrap();
/// assert_eq!(ast.kind(), "document");
/// ```
pub fn parse(source: &str) -> Result<AstNode, ParseError> {
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
/// # Example
/// ```rust
/// use toon_lsp::parse_with_errors;
///
/// let (ast, errors) = parse_with_errors("name Alice\nage: 30");
/// assert!(ast.is_some()); // Partial AST with valid portions
/// assert!(!errors.is_empty()); // Contains error for missing colon
/// ```
#[must_use]
pub fn parse_with_errors(source: &str) -> (Option<AstNode>, Vec<ParseError>) {
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
