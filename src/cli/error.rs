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

//! Error types and exit codes for CLI operations.

use std::io;
use thiserror::Error;

/// CLI operation errors with structured exit codes.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CliError {
    /// I/O error reading or writing files
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// TOON parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// TOON encoding error
    #[error("Encode error: {0}")]
    Encode(String),

    /// TOON decoding error
    #[error("Decode error: {0}")]
    Decode(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Validation error (syntax is valid but content has issues)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Formatting error
    #[error("Format error: {0}")]
    Format(String),

    /// Format check mismatch (file needs formatting)
    #[error("File needs formatting")]
    FormatMismatch,

    /// Symbol extraction error
    #[error("Symbol error: {0}")]
    Symbol(String),

    /// Diagnostic generation error
    #[error("Diagnostic error: {0}")]
    Diagnostic(String),

    /// Generic operation error
    #[error("{0}")]
    Other(String),
}

/// Exit codes for CLI operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Operation completed successfully
    Success = 0,
    /// General error (I/O, encoding, decoding)
    Error = 1,
    /// Validation or format check failed
    ValidationFailed = 2,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

impl From<&CliError> for ExitCode {
    fn from(error: &CliError) -> Self {
        match error {
            CliError::Validation(_) | CliError::Format(_) => ExitCode::ValidationFailed,
            _ => ExitCode::Error,
        }
    }
}

impl CliError {
    /// Get the appropriate exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        ExitCode::from(self)
    }

    /// Create a parse error from a message.
    #[must_use]
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create an encode error from a message.
    #[must_use]
    pub fn encode(msg: impl Into<String>) -> Self {
        Self::Encode(msg.into())
    }

    /// Create a decode error from a message.
    #[must_use]
    pub fn decode(msg: impl Into<String>) -> Self {
        Self::Decode(msg.into())
    }

    /// Create a validation error from a message.
    #[must_use]
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a format error from a message.
    #[must_use]
    pub fn format(msg: impl Into<String>) -> Self {
        Self::Format(msg.into())
    }

    /// Create a symbol error from a message.
    #[must_use]
    pub fn symbol(msg: impl Into<String>) -> Self {
        Self::Symbol(msg.into())
    }

    /// Create a diagnostic error from a message.
    #[must_use]
    pub fn diagnostic(msg: impl Into<String>) -> Self {
        Self::Diagnostic(msg.into())
    }
}

/// Result type alias for CLI operations.
pub type CliResult<T> = Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::Error as i32, 1);
        assert_eq!(ExitCode::ValidationFailed as i32, 2);
    }

    #[test]
    fn test_exit_code_from_error() {
        assert_eq!(CliError::parse("test").exit_code(), ExitCode::Error);
        assert_eq!(CliError::validation("test").exit_code(), ExitCode::ValidationFailed);
        assert_eq!(CliError::format("test").exit_code(), ExitCode::ValidationFailed);
    }

    #[test]
    fn test_error_constructors() {
        let err = CliError::parse("test parse");
        assert_eq!(err.to_string(), "Parse error: test parse");

        let err = CliError::encode("test encode");
        assert_eq!(err.to_string(), "Encode error: test encode");

        let err = CliError::decode("test decode");
        assert_eq!(err.to_string(), "Decode error: test decode");

        let err = CliError::validation("test validation");
        assert_eq!(err.to_string(), "Validation error: test validation");

        let err = CliError::format("test format");
        assert_eq!(err.to_string(), "Format error: test format");

        let err = CliError::symbol("test symbol");
        assert_eq!(err.to_string(), "Symbol error: test symbol");

        let err = CliError::diagnostic("test diagnostic");
        assert_eq!(err.to_string(), "Diagnostic error: test diagnostic");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cli_err = CliError::from(io_err);
        assert!(matches!(cli_err, CliError::Io(_)));
        assert_eq!(cli_err.exit_code(), ExitCode::Error);
    }

    #[test]
    fn test_exit_code_to_i32() {
        assert_eq!(i32::from(ExitCode::Success), 0);
        assert_eq!(i32::from(ExitCode::Error), 1);
        assert_eq!(i32::from(ExitCode::ValidationFailed), 2);
    }
}
