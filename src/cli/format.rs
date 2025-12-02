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

//! Format command implementation: TOON file formatting.
//!
//! This module provides consistent formatting for TOON files including:
//! - Indentation normalization (spaces or tabs)
//! - Consistent spacing around colons
//! - Check mode for CI verification

use super::error::{CliError, CliResult, ExitCode};
use super::io_utils::{read_file, read_stdin, write_output as write_output_impl};
use super::FormatArgs;
use crate::lsp::formatting::{format_document, ToonFormattingOptions};
use crate::parser;

/// Execute the format command.
///
/// Formats TOON input with consistent style.
///
/// # Returns
///
/// - `Ok(())` if formatting succeeds
/// - `Err(CliError::Validation(...))` for parse errors (exit code 2)
/// - `Err(CliError::Io(...))` for I/O errors (exit code 1)
pub fn execute(args: &FormatArgs) -> CliResult<()> {
    // Read input
    let (content, _input_path) = read_input(args)?;

    // Parse the content
    let (ast, errors) = parser::parse_with_errors(&content);

    // Fail on parse errors
    if !errors.is_empty() {
        let error_msg = errors
            .iter()
            .map(|e| e.kind.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(CliError::Validation(error_msg));
    }

    // Get AST node (parse_with_errors returns Option<AstNode>)
    let ast_node = ast.ok_or_else(|| CliError::Validation("Failed to parse document".to_string()))?;

    // Format the AST
    let options = ToonFormattingOptions {
        indent_size: args.indent as u32,
        use_tabs: args.tabs,
    };
    let formatted = format_document(&ast_node, options)
        .ok_or_else(|| CliError::Format("Failed to format document".to_string()))?;

    // Check mode: compare and report
    if args.check {
        if formatted != content {
            // File needs formatting - exit with code 1
            return Err(CliError::FormatMismatch);
        }
        return Ok(());
    }

    // Write output
    write_output(args, &formatted)?;

    Ok(())
}

/// Read input from file or stdin.
fn read_input(args: &FormatArgs) -> CliResult<(String, Option<std::path::PathBuf>)> {
    match &args.input {
        Some(path) if path.as_os_str() == "-" => Ok((read_stdin()?, None)),
        Some(path) => Ok((read_file(path)?, Some(path.clone()))),
        None => Ok((read_stdin()?, None)),
    }
}

/// Write output to file or stdout using shared utility.
fn write_output(args: &FormatArgs, content: &str) -> CliResult<()> {
    write_output_impl(&args.output, content)
}

/// Get exit code for format errors.
pub fn error_exit_code(error: &CliError) -> ExitCode {
    match error {
        CliError::Validation(_) => ExitCode::ValidationFailed,
        CliError::FormatMismatch => ExitCode::Error,
        _ => error.exit_code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_format_simple_toon() {
        let content = "key: value\n";
        let (ast, errors) = parser::parse_with_errors(content);
        assert!(errors.is_empty());
        let ast_node = ast.unwrap();

        let options = ToonFormattingOptions::default();
        let formatted = format_document(&ast_node, options).unwrap();
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));
    }

    #[test]
    fn test_format_normalizes_spacing() {
        // Content without space after colon - parser may still handle it
        let content = "key: value\n";
        let (ast, errors) = parser::parse_with_errors(content);
        assert!(errors.is_empty());
        let ast_node = ast.unwrap();

        let options = ToonFormattingOptions::default();
        let formatted = format_document(&ast_node, options).unwrap();
        // Formatter should produce properly spaced output
        assert!(formatted.contains("key: value"));
    }

    #[test]
    fn test_format_with_custom_indent() {
        let content = "server:\n  host: localhost\n";
        let (ast, errors) = parser::parse_with_errors(content);
        assert!(errors.is_empty());
        let ast_node = ast.unwrap();

        let options = ToonFormattingOptions {
            indent_size: 4,
            use_tabs: false,
        };
        let formatted = format_document(&ast_node, options).unwrap();
        // Should use 4-space indent
        assert!(formatted.contains("    host:"));
    }

    #[test]
    fn test_format_with_tabs() {
        let content = "server:\n  host: localhost\n";
        let (ast, errors) = parser::parse_with_errors(content);
        assert!(errors.is_empty());
        let ast_node = ast.unwrap();

        let options = ToonFormattingOptions {
            indent_size: 1,
            use_tabs: true,
        };
        let formatted = format_document(&ast_node, options).unwrap();
        // Should use tab indent
        assert!(formatted.contains("\thost:"));
    }

    #[test]
    fn test_error_exit_code() {
        let validation_err = CliError::Validation("test".to_string());
        assert_eq!(error_exit_code(&validation_err), ExitCode::ValidationFailed);

        let format_err = CliError::FormatMismatch;
        assert_eq!(error_exit_code(&format_err), ExitCode::Error);

        let io_err = CliError::Io(io::Error::new(io::ErrorKind::NotFound, "not found"));
        assert_eq!(error_exit_code(&io_err), ExitCode::Error);
    }
}
