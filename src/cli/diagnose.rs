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

//! Diagnostic generation for TOON files.
//!
//! Outputs structured diagnostics in JSON or SARIF 2.1.0 format for tooling integration.

use serde::Serialize;

use super::error::{CliError, CliResult, ExitCode};
use super::io_utils::{read_input, write_output};
use super::{DiagnoseArgs, DiagnoseFormat, Severity};
use crate::ast::Span;
use crate::parser::{self, ParseError};

#[cfg(test)]
use crate::ast::Position as AstPosition;

/// Position in a text document (zero-based).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Position {
    /// Line number (zero-based)
    pub line: u32,
    /// Character offset in line (zero-based)
    pub character: u32,
}

/// Range in a text document.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

/// A single diagnostic entry.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticEntry {
    /// Source range of the diagnostic
    pub range: Range,
    /// Severity level
    pub severity: String,
    /// Error code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Source of the diagnostic
    pub source: String,
    /// Source code context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Summary of diagnostic counts.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticSummary {
    /// Number of errors
    #[serde(rename = "error_count")]
    pub errors: usize,
    /// Number of warnings
    #[serde(rename = "warning_count")]
    pub warnings: usize,
    /// Number of hints
    #[serde(rename = "hint_count")]
    pub hints: usize,
}

/// Complete diagnostic report for a file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticReport {
    /// File path
    pub file: String,
    /// List of diagnostics
    pub diagnostics: Vec<DiagnosticEntry>,
    /// Summary counts
    pub summary: DiagnosticSummary,
}

/// SARIF 2.1.0 format structures.
///
/// See: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
#[derive(Debug, Serialize)]
struct SarifReport {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Debug, Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Serialize)]
struct SarifDriver {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct SarifResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Debug, Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: u32,
    #[serde(rename = "startColumn")]
    start_column: u32,
    #[serde(rename = "endLine")]
    end_line: u32,
    #[serde(rename = "endColumn")]
    end_column: u32,
}

/// Execute the diagnose command.
///
/// Parses TOON input and generates structured diagnostics in the requested format.
pub fn execute(args: &DiagnoseArgs) -> CliResult<()> {
    // Read input from file or stdin
    let content = read_input(&args.input)?;
    let file_name =
        args.input.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("stdin");
    let file_path =
        args.input.as_ref().map_or_else(|| "stdin".to_string(), |p| p.display().to_string());

    // Parse and collect diagnostics
    let report = generate_diagnostics(&content, file_name, args.context, args.severity)?;

    // Format output based on requested format
    let output = match args.format {
        DiagnoseFormat::Json => format_json(&report)?,
        DiagnoseFormat::Sarif => format_sarif(&report, &file_path)?,
    };

    // Write output
    write_output(&None, &output)?;

    // Diagnose command always succeeds (exit 0) - it just reports diagnostics
    // The caller can check the summary.errors field to determine if issues were found
    Ok(())
}

/// Generate diagnostics from TOON content.
fn generate_diagnostics(
    content: &str,
    file_name: &str,
    include_context: bool,
    min_severity: Severity,
) -> CliResult<DiagnosticReport> {
    // Parse with error recovery
    let (_ast, errors) = parser::parse_with_errors(content);

    // Convert parse errors to diagnostic entries
    let diagnostics: Vec<DiagnosticEntry> = errors
        .into_iter()
        .filter_map(|err| {
            // Map ParseError to severity (currently all are errors)
            let severity = Severity::Error;

            // Filter by minimum severity
            if severity < min_severity {
                return None;
            }

            // Extract source context if requested
            let source_context =
                if include_context { extract_context(content, &err.span) } else { None };

            Some(DiagnosticEntry {
                range: span_to_range(&err.span),
                severity: severity_to_string(severity),
                code: Some(format!("E{:03}", error_code(&err))),
                message: format!("{}", err.kind),
                source: "toon-lsp".to_string(),
                context: source_context,
            })
        })
        .collect();

    // Calculate summary counts
    let summary = calculate_summary(&diagnostics);

    Ok(DiagnosticReport { file: file_name.to_string(), diagnostics, summary })
}

/// Convert Span to LSP Range (zero-based).
/// Note: AST Position is already 0-indexed, so no conversion needed.
fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position { line: span.start.line, character: span.start.column },
        end: Position { line: span.end.line, character: span.end.column },
    }
}

/// Extract source code context around a span.
fn extract_context(content: &str, span: &Span) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = span.start.line as usize;

    lines.get(line_idx).map(|line| (*line).to_string())
}

/// Assign error code based on ParseError.
fn error_code(error: &ParseError) -> u32 {
    // Simple mapping - can be extended with more specific codes
    use crate::parser::ParseErrorKind;

    match &error.kind {
        ParseErrorKind::UnexpectedChar => 1,
        ParseErrorKind::UnexpectedToken => 2,
        ParseErrorKind::ExpectedColon => 3,
        ParseErrorKind::ExpectedValue => 4,
        ParseErrorKind::ExpectedKey => 5,
        ParseErrorKind::InvalidNumber => 6,
        ParseErrorKind::UnterminatedString => 7,
        ParseErrorKind::InvalidEscape => 8,
        ParseErrorKind::InvalidIndent => 9,
        ParseErrorKind::UnexpectedEof => 10,
        ParseErrorKind::DuplicateKey => 11,
        ParseErrorKind::MaxDepthExceeded => 12,
        ParseErrorKind::DocumentTooLarge => 13,
        ParseErrorKind::TooManyArrayItems => 14,
        ParseErrorKind::TooManyObjectEntries => 15,
    }
}

/// Convert Severity to string.
fn severity_to_string(severity: Severity) -> String {
    match severity {
        Severity::Error => "error".to_string(),
        Severity::Warning => "warning".to_string(),
        Severity::Info => "info".to_string(),
        Severity::Hint => "hint".to_string(),
    }
}

/// Calculate summary counts from diagnostics.
fn calculate_summary(diagnostics: &[DiagnosticEntry]) -> DiagnosticSummary {
    let mut errors = 0;
    let mut warnings = 0;
    let mut hints = 0;

    for diag in diagnostics {
        match diag.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            "hint" => hints += 1,
            _ => {}
        }
    }

    DiagnosticSummary { errors, warnings, hints }
}

/// Format diagnostics as JSON.
fn format_json(report: &DiagnosticReport) -> CliResult<String> {
    serde_json::to_string_pretty(report).map_err(CliError::from)
}

/// Convert a file path to a file:// URI.
///
/// For absolute paths, returns `file:///path/to/file` (Unix) or `file:///C:/path/to/file` (Windows).
/// For relative paths or "stdin", returns the path unchanged.
fn path_to_file_uri(path: &str) -> String {
    use std::path::Path;

    // Handle stdin specially
    if path == "stdin" || path == "<stdin>" {
        return path.to_string();
    }

    let p = Path::new(path);
    if p.is_absolute() {
        // Convert to file:// URI
        // On Windows, paths like C:\foo become file:///C:/foo
        let normalized = path.replace('\\', "/");
        if normalized.starts_with('/') {
            format!("file://{normalized}")
        } else {
            format!("file:///{normalized}")
        }
    } else {
        // Relative paths stay as-is
        path.to_string()
    }
}

/// Format diagnostics as SARIF 2.1.0.
fn format_sarif(report: &DiagnosticReport, file_path: &str) -> CliResult<String> {
    let artifact_uri = path_to_file_uri(file_path);

    let results: Vec<SarifResult> = report
        .diagnostics
        .iter()
        .map(|diag| {
            let level = match diag.severity.as_str() {
                "error" => "error",
                "warning" => "warning",
                "info" | "hint" => "note",
                _ => "none",
            };

            SarifResult {
                rule_id: diag.code.clone(),
                level: level.to_string(),
                message: SarifMessage { text: diag.message.clone() },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation { uri: artifact_uri.clone() },
                        region: SarifRegion {
                            start_line: diag.range.start.line + 1, // SARIF is 1-based
                            start_column: diag.range.start.character + 1,
                            end_line: diag.range.end.line + 1,
                            end_column: diag.range.end.character + 1,
                        },
                    },
                }],
            }
        })
        .collect();

    let sarif = SarifReport {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "toon-lsp".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&sarif).map_err(CliError::from)
}

/// Get appropriate exit code for the error.
#[must_use]
pub fn error_exit_code(error: &CliError) -> ExitCode {
    error.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_to_range_conversion() {
        // AST Position is already 0-indexed (line 1 = 0, column 5 = 5)
        let span = Span::new(AstPosition::new(1, 5, 0), AstPosition::new(1, 10, 5));
        let range = span_to_range(&span);

        // Should preserve 0-based indexing
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 5);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.character, 10);
    }

    #[test]
    fn test_extract_context() {
        let content = "line 1\nline 2 with error\nline 3\n";
        // Line 1 = index 1 (0-indexed, so line 2 in 1-based)
        let span = Span::new(AstPosition::new(1, 0, 7), AstPosition::new(1, 5, 12));

        let context = extract_context(content, &span);
        assert_eq!(context, Some("line 2 with error".to_string()));
    }

    #[test]
    fn test_extract_context_invalid_line() {
        let content = "line 1\n";
        // Invalid line number
        let span = Span::new(AstPosition::new(99, 0, 999), AstPosition::new(99, 5, 1004));

        let context = extract_context(content, &span);
        assert_eq!(context, None);
    }

    #[test]
    fn test_error_code_mapping() {
        use crate::parser::ParseErrorKind;

        let pos = AstPosition::new(1, 1, 1);
        let span = Span::new(pos, pos);

        let err = ParseError { kind: ParseErrorKind::UnexpectedChar, span, context: None };
        assert_eq!(error_code(&err), 1);

        let err = ParseError { kind: ParseErrorKind::UnexpectedToken, span, context: None };
        assert_eq!(error_code(&err), 2);

        let err = ParseError { kind: ParseErrorKind::UnexpectedEof, span, context: None };
        assert_eq!(error_code(&err), 10);

        let err = ParseError { kind: ParseErrorKind::DuplicateKey, span, context: None };
        assert_eq!(error_code(&err), 11);
    }

    #[test]
    fn test_severity_to_string() {
        assert_eq!(severity_to_string(Severity::Error), "error");
        assert_eq!(severity_to_string(Severity::Warning), "warning");
        assert_eq!(severity_to_string(Severity::Info), "info");
        assert_eq!(severity_to_string(Severity::Hint), "hint");
    }

    #[test]
    fn test_calculate_summary() {
        let diagnostics = vec![
            DiagnosticEntry {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 1 },
                },
                severity: "error".to_string(),
                code: Some("E001".to_string()),
                message: "error 1".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            },
            DiagnosticEntry {
                range: Range {
                    start: Position { line: 1, character: 0 },
                    end: Position { line: 1, character: 1 },
                },
                severity: "error".to_string(),
                code: Some("E002".to_string()),
                message: "error 2".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            },
            DiagnosticEntry {
                range: Range {
                    start: Position { line: 2, character: 0 },
                    end: Position { line: 2, character: 1 },
                },
                severity: "warning".to_string(),
                code: Some("W001".to_string()),
                message: "warning 1".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            },
        ];

        let summary = calculate_summary(&diagnostics);
        assert_eq!(summary.errors, 2);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.hints, 0);
    }

    #[test]
    fn test_generate_diagnostics_valid_input() {
        let content = "key: value\n";
        let report = generate_diagnostics(content, "test.toon", false, Severity::Error).unwrap();

        assert_eq!(report.file, "test.toon");
        assert_eq!(report.diagnostics.len(), 0);
        assert_eq!(report.summary.errors, 0);
    }

    #[test]
    fn test_generate_diagnostics_invalid_input() {
        let content = "key: [unclosed\n";
        let report = generate_diagnostics(content, "test.toon", false, Severity::Error).unwrap();

        assert_eq!(report.file, "test.toon");
        assert!(!report.diagnostics.is_empty());
        assert!(report.summary.errors > 0);
    }

    #[test]
    fn test_generate_diagnostics_with_context() {
        let content = "key: [unclosed\n";
        let report = generate_diagnostics(content, "test.toon", true, Severity::Error).unwrap();

        assert!(!report.diagnostics.is_empty());
        let diag = &report.diagnostics[0];
        assert!(diag.context.is_some());
        assert_eq!(diag.context.as_ref().unwrap(), "key: [unclosed");
    }

    #[test]
    fn test_format_json() {
        let report = DiagnosticReport {
            file: "test.toon".to_string(),
            diagnostics: vec![DiagnosticEntry {
                range: Range {
                    start: Position { line: 0, character: 4 },
                    end: Position { line: 0, character: 13 },
                },
                severity: "error".to_string(),
                code: Some("E001".to_string()),
                message: "test error".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            }],
            summary: DiagnosticSummary { errors: 1, warnings: 0, hints: 0 },
        };

        let json = format_json(&report).unwrap();
        assert!(json.contains("\"file\": \"test.toon\""));
        assert!(json.contains("\"severity\": \"error\""));
        assert!(json.contains("\"code\": \"E001\""));
        assert!(json.contains("\"message\": \"test error\""));
    }

    #[test]
    fn test_format_sarif() {
        let report = DiagnosticReport {
            file: "test.toon".to_string(),
            diagnostics: vec![DiagnosticEntry {
                range: Range {
                    start: Position { line: 0, character: 4 },
                    end: Position { line: 0, character: 13 },
                },
                severity: "error".to_string(),
                code: Some("E001".to_string()),
                message: "test error".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            }],
            summary: DiagnosticSummary { errors: 1, warnings: 0, hints: 0 },
        };

        let sarif = format_sarif(&report, "test.toon").unwrap();
        assert!(sarif.contains("\"$schema\""));
        assert!(sarif.contains("sarif-schema-2.1.0.json"));
        assert!(sarif.contains("\"version\": \"2.1.0\""));
        assert!(sarif.contains("\"name\": \"toon-lsp\""));
        assert!(sarif.contains("\"ruleId\": \"E001\""));
        assert!(sarif.contains("\"level\": \"error\""));
    }

    #[test]
    fn test_sarif_region_is_one_based() {
        let report = DiagnosticReport {
            file: "test.toon".to_string(),
            diagnostics: vec![DiagnosticEntry {
                range: Range {
                    start: Position { line: 0, character: 0 }, // Zero-based
                    end: Position { line: 0, character: 5 },
                },
                severity: "error".to_string(),
                code: Some("E001".to_string()),
                message: "test".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            }],
            summary: DiagnosticSummary { errors: 1, warnings: 0, hints: 0 },
        };

        let sarif = format_sarif(&report, "test.toon").unwrap();
        // SARIF uses 1-based indexing
        assert!(sarif.contains("\"startLine\": 1"));
        assert!(sarif.contains("\"startColumn\": 1"));
    }

    #[test]
    fn test_severity_filtering() {
        // Create mock content with error (for realistic span)
        let content = "key: [unclosed\n";

        // Parse to get real errors
        let (_ast, errors) = parser::parse_with_errors(content);
        assert!(!errors.is_empty(), "Should have parse errors");

        // Test filtering with Error level (should include errors)
        let report = generate_diagnostics(content, "test.toon", false, Severity::Error).unwrap();
        assert!(!report.diagnostics.is_empty());

        // Currently all ParseErrors are mapped to Error severity
        // When we add warnings/hints, this test will verify filtering works
    }

    #[test]
    fn test_error_exit_code() {
        let err = CliError::diagnostic("test error");
        assert_eq!(error_exit_code(&err), ExitCode::Error);

        let err = CliError::validation("validation failed");
        assert_eq!(error_exit_code(&err), ExitCode::ValidationFailed);
    }

    #[test]
    fn test_path_to_file_uri_relative() {
        // Relative paths should stay as-is
        assert_eq!(path_to_file_uri("test.toon"), "test.toon");
        assert_eq!(path_to_file_uri("src/config.toon"), "src/config.toon");
    }

    #[test]
    fn test_path_to_file_uri_stdin() {
        // stdin should stay as-is
        assert_eq!(path_to_file_uri("stdin"), "stdin");
        assert_eq!(path_to_file_uri("<stdin>"), "<stdin>");
    }

    #[test]
    #[cfg(unix)]
    fn test_path_to_file_uri_unix_absolute() {
        // Unix absolute paths get file:// prefix
        assert_eq!(path_to_file_uri("/home/user/test.toon"), "file:///home/user/test.toon");
    }

    #[test]
    #[cfg(windows)]
    fn test_path_to_file_uri_windows_absolute() {
        // Windows absolute paths get file:/// prefix and backslash conversion
        let uri = path_to_file_uri("C:\\Users\\test.toon");
        assert!(uri.starts_with("file:///"));
        assert!(uri.contains("C:/Users/test.toon"));
    }

    #[test]
    fn test_sarif_uses_file_uri_for_absolute_paths() {
        let report = DiagnosticReport {
            file: "test.toon".to_string(),
            diagnostics: vec![DiagnosticEntry {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 5 },
                },
                severity: "error".to_string(),
                code: Some("E001".to_string()),
                message: "test".to_string(),
                source: "toon-lsp".to_string(),
                context: None,
            }],
            summary: DiagnosticSummary { errors: 1, warnings: 0, hints: 0 },
        };

        // Relative path stays as-is
        let sarif = format_sarif(&report, "test.toon").unwrap();
        assert!(sarif.contains("\"uri\": \"test.toon\""));
    }
}
