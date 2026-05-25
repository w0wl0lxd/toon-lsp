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

//! Parse error types with source position information.
//!
//! # Example
//! ```rust
//! use toon_lsp::{parse, ParseErrorKind};
//!
//! let result = parse("name Alice"); // Missing colon
//! assert!(result.is_err());
//! let error = result.unwrap_err();
//! assert_eq!(error.kind, ParseErrorKind::ExpectedColon);
//! ```

use crate::ast::Span;
use thiserror::Error;

/// Error that occurred during parsing.
///
/// Contains the error kind, source span, and optional context.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The kind of error
    pub kind: ParseErrorKind,
    /// Source span where the error occurred
    pub span: Span,
    /// Optional context message
    pub context: Option<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(ref ctx) = self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Create a new parse error.
    ///
    /// # Example
    /// ```rust
    /// use toon_lsp::{ParseError, ParseErrorKind, Span};
    ///
    /// let error = ParseError::new(ParseErrorKind::UnexpectedEof, Span::default());
    /// assert_eq!(error.kind, ParseErrorKind::UnexpectedEof);
    /// assert_eq!(error.context, None);
    /// ```
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span, context: None }
    }

    /// Add context to this error.
    ///
    /// # Example
    /// ```rust
    /// use toon_lsp::{ParseError, ParseErrorKind, Span};
    ///
    /// let error = ParseError::new(ParseErrorKind::ExpectedKey, Span::default())
    ///     .with_context("at object start");
    /// assert_eq!(error.context, Some("at object start".to_string()));
    /// ```
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Kinds of parse errors.
///
/// Each variant represents a specific category of parsing failure,
/// useful for targeted error recovery in IDEs.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("unexpected character")]
    UnexpectedChar,

    #[error("unexpected token")]
    UnexpectedToken,

    #[error("expected colon")]
    ExpectedColon,

    #[error("expected value")]
    ExpectedValue,

    #[error("expected key")]
    ExpectedKey,

    #[error("invalid number")]
    InvalidNumber,

    #[error("unterminated string")]
    UnterminatedString,

    #[error("invalid escape sequence")]
    InvalidEscape,

    #[error("invalid indentation")]
    InvalidIndent,

    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("duplicate key")]
    DuplicateKey,

    // Security error variants for resource exhaustion protection
    #[error("maximum nesting depth exceeded")]
    MaxDepthExceeded,

    #[error("document too large")]
    DocumentTooLarge,

    #[error("too many array items")]
    TooManyArrayItems,

    #[error("too many object entries")]
    TooManyObjectEntries,
}
