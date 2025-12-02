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

use crate::ast::Span;
use thiserror::Error;

/// Error that occurred during parsing.
#[derive(Debug, Error, Clone)]
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
        write!(
            f,
            "{} at line {}, column {}",
            self.kind,
            self.span.start.line + 1,
            self.span.start.column + 1
        )?;
        if let Some(ctx) = &self.context {
            write!(f, ": {}", ctx)?;
        }
        Ok(())
    }
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span, context: None }
    }

    /// Add context to this error.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Kinds of parse errors.
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
