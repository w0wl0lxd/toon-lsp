//! Error types for the in-house TOON codec.

use thiserror::Error;

/// Errors produced while encoding a [`serde_json::Value`] into TOON.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EncodeError {
    /// A value that has no TOON representation (for example a non-finite float).
    #[error("unsupported value: {0}")]
    Unsupported(String),
}

/// Errors produced while decoding TOON text into a [`serde_json::Value`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DecodeError {
    /// A lexical or grammatical error at a specific source location.
    #[error("syntax error at line {line}, column {col}: {message}")]
    Syntax {
        /// Human-readable description of the problem.
        message: String,
        /// 1-based line number where the error was detected.
        line: u32,
        /// 1-based column number where the error was detected.
        col: u32,
    },
    /// A structurally invalid document that is lexically well-formed.
    #[error("structure error: {0}")]
    Structure(String),
}

impl DecodeError {
    /// Creates a [`DecodeError::Structure`] from a message.
    ///
    /// Convenience for decoders that report structural problems without precise
    /// line/column information.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self::Structure(message.into())
    }
}

/// Result alias for encoding operations.
pub type EncodeResult<T> = Result<T, EncodeError>;

/// Result alias for decoding operations.
pub type DecodeResult<T> = Result<T, DecodeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_syntax_error_displays_location() {
        let e = DecodeError::Syntax { message: "unexpected }".into(), line: 3, col: 5 };
        assert_eq!(e.to_string(), "syntax error at line 3, column 5: unexpected }");
    }

    #[test]
    fn encode_unsupported_displays() {
        let e = EncodeError::Unsupported("NaN".into());
        assert_eq!(e.to_string(), "unsupported value: NaN");
    }
}
