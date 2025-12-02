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

//! Check command implementation: TOON syntax validation.
//!
//! This module validates TOON files without producing output, reporting
//! errors in various formats for CI/CD integration.
//!
//! ## Output Formats
//!
//! - **text**: Human-readable error messages (default)
//! - **json**: Machine-parseable JSON output
//! - **github**: GitHub Actions annotation format (::error)
//!
//! ## Batch Processing
//!
//! When checking multiple files, all files are processed (not fail-fast)
//! and all errors are reported. Exit code is 2 if any file has errors.

use std::path::{Path, PathBuf};

use super::error::{CliError, CliResult, ExitCode};
use super::io_utils::{read_file, read_stdin};
use super::{CheckArgs, DiagnosticFormat};
use crate::parser;

/// A diagnostic message from validation.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    /// Format as text (human-readable).
    pub fn format_text(&self) -> String {
        format!(
            "{}:{}:{}: error: {}",
            self.file.display(),
            self.line,
            self.column,
            self.message
        )
    }

    /// Format as GitHub Actions annotation.
    ///
    /// Special characters are URL-encoded per GitHub Actions workflow commands:
    /// - `%` -> `%25`
    /// - `\r` -> `%0D`
    /// - `\n` -> `%0A`
    pub fn format_github(&self) -> String {
        // URL-encode special characters in message
        let encoded_message = self
            .message
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A");
        format!(
            "::error file={},line={},col={}::{}",
            self.file.display(),
            self.line,
            self.column,
            encoded_message
        )
    }

    /// Format as JSON.
    pub fn format_json(&self) -> String {
        serde_json::json!({
            "file": self.file.to_string_lossy(),
            "line": self.line,
            "column": self.column,
            "message": self.message,
            "severity": "error"
        })
        .to_string()
    }
}

/// Result of checking a single file.
pub struct CheckResult {
    pub file: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckResult {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Execute the check command.
///
/// Validates TOON syntax for one or more files.
///
/// # Returns
///
/// - `Ok(())` if all files are valid
/// - `Err(CliError::Validation(...))` if any file has errors
/// - `Err(CliError::Io(...))` if a file cannot be read
pub fn execute(args: &CheckArgs) -> CliResult<()> {
    let results = check_files(args)?;

    // Collect all diagnostics
    let all_diagnostics: Vec<&Diagnostic> = results
        .iter()
        .flat_map(|r| &r.diagnostics)
        .collect();

    // If there are errors, report them and fail
    if !all_diagnostics.is_empty() {
        report_diagnostics(&all_diagnostics, args.format);
        return Err(CliError::Validation(format!(
            "{} error(s) found",
            all_diagnostics.len()
        )));
    }

    Ok(())
}

/// Check files based on args.
fn check_files(args: &CheckArgs) -> CliResult<Vec<CheckResult>> {
    // No input files or single "-" means stdin
    if args.input.is_empty() || (args.input.len() == 1 && args.input[0].as_os_str() == "-") {
        let content = read_stdin()?;
        let diagnostics = check_content(&content, Path::new("<stdin>"));
        return Ok(vec![CheckResult {
            file: PathBuf::from("<stdin>"),
            diagnostics,
        }]);
    }

    // Check all provided files
    let mut results = Vec::with_capacity(args.input.len());
    for path in &args.input {
        results.push(check_single_file(path)?);
    }
    Ok(results)
}

/// Check a single file.
fn check_single_file(path: &Path) -> CliResult<CheckResult> {
    let content = read_file(path)?;
    let diagnostics = check_content(&content, path);
    Ok(CheckResult {
        file: path.to_path_buf(),
        diagnostics,
    })
}

/// Check TOON content and return diagnostics.
fn check_content(content: &str, file: &Path) -> Vec<Diagnostic> {
    // Use the parser to check syntax
    let (_, errors) = parser::parse_with_errors(content);

    errors
        .iter()
        .map(|e| Diagnostic {
            file: file.to_path_buf(),
            // Convert from 0-indexed to 1-indexed for display
            line: (e.span.start.line as usize) + 1,
            column: (e.span.start.column as usize) + 1,
            message: e.kind.to_string(),
        })
        .collect()
}

/// Report diagnostics to stderr in the requested format.
fn report_diagnostics(diagnostics: &[&Diagnostic], format: DiagnosticFormat) {
    for diag in diagnostics {
        let output = match format {
            DiagnosticFormat::Text => diag.format_text(),
            DiagnosticFormat::Github => diag.format_github(),
            DiagnosticFormat::Json => diag.format_json(),
        };
        eprintln!("{output}");
    }
}

/// Get exit code for check errors.
pub fn error_exit_code(error: &CliError) -> ExitCode {
    match error {
        CliError::Validation(_) => ExitCode::ValidationFailed,
        _ => error.exit_code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_check_valid_content() {
        let content = "key: value\n";
        let diagnostics = check_content(content, Path::new("test.toon"));
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_check_invalid_content() {
        let content = "key: [unclosed";
        let diagnostics = check_content(content, Path::new("test.toon"));
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_diagnostic_format_text() {
        let diag = Diagnostic {
            file: PathBuf::from("test.toon"),
            line: 5,
            column: 10,
            message: "test error".to_string(),
        };
        let text = diag.format_text();
        assert!(text.contains("test.toon"));
        assert!(text.contains("5"));
        assert!(text.contains("10"));
        assert!(text.contains("test error"));
    }

    #[test]
    fn test_diagnostic_format_github() {
        let diag = Diagnostic {
            file: PathBuf::from("test.toon"),
            line: 5,
            column: 10,
            message: "test error".to_string(),
        };
        let github = diag.format_github();
        assert!(github.starts_with("::error"));
        assert!(github.contains("file=test.toon"));
        assert!(github.contains("line=5"));
        assert!(github.contains("col=10"));
    }

    #[test]
    fn test_diagnostic_format_github_encodes_special_chars() {
        let diag = Diagnostic {
            file: PathBuf::from("test.toon"),
            line: 1,
            column: 1,
            message: "100% complete\nwith newline\rand carriage return".to_string(),
        };
        let github = diag.format_github();
        // % should be encoded as %25
        assert!(github.contains("100%25 complete"));
        // \n should be encoded as %0A
        assert!(github.contains("%0Awith newline"));
        // \r should be encoded as %0D
        assert!(github.contains("%0Dand carriage return"));
        // Original special chars should not appear
        assert!(!github.contains('\n'));
        assert!(!github.contains('\r'));
    }

    #[test]
    fn test_diagnostic_format_json() {
        let diag = Diagnostic {
            file: PathBuf::from("test.toon"),
            line: 5,
            column: 10,
            message: "test error".to_string(),
        };
        let json = diag.format_json();
        assert!(json.contains("\"file\""));
        assert!(json.contains("\"line\":5"));
        assert!(json.contains("\"column\":10"));
    }

    #[test]
    fn test_error_exit_code() {
        let validation_err = CliError::Validation("test".to_string());
        assert_eq!(error_exit_code(&validation_err), ExitCode::ValidationFailed);

        let io_err = CliError::Io(io::Error::new(io::ErrorKind::NotFound, "not found"));
        assert_eq!(error_exit_code(&io_err), ExitCode::Error);
    }
}
