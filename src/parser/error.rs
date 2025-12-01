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
        Self {
            kind,
            span,
            context: None,
        }
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
}
