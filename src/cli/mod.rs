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

//! Command-line interface for TOON operations.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod check;
pub mod convert;
pub mod decode;
pub mod encode;
pub mod error;
pub mod format;
pub mod io_utils;
pub mod symbols;
pub mod diagnose;

/// TOON Language Server and CLI tools
#[derive(Debug, Parser)]
#[command(name = "toon-lsp")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Increase verbosity (-v for info, -vv for debug, -vvv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand to execute (defaults to LSP mode if omitted)
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available CLI commands
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Command {
    /// Convert JSON or YAML to TOON format
    Encode(EncodeArgs),

    /// Convert TOON to JSON or YAML format
    Decode(DecodeArgs),

    /// Check TOON syntax without output
    Check(CheckArgs),

    /// Format TOON files with consistent style
    Format(FormatArgs),

    /// Extract document symbols (keys) from TOON
    Symbols(SymbolsArgs),

    /// Run diagnostics and output structured results
    Diagnose(DiagnoseArgs),

    /// Start LSP server (stdin/stdout communication)
    Lsp,
}

/// Arguments for encode command
#[derive(Debug, Parser)]
pub struct EncodeArgs {
    /// Input file (JSON or YAML), or stdin if omitted
    #[arg(value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output file, or stdout if omitted
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Input format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    pub input_format: InputFormat,

    /// Indentation size (spaces)
    #[arg(short, long, default_value = "2")]
    pub indent: usize,

    /// Array style preference
    #[arg(short = 's', long, value_enum, default_value = "auto")]
    pub array_style: ArrayStyle,

    /// Use tabs for indentation
    #[arg(long)]
    pub tabs: bool,

    /// Maximum line width for tabular arrays
    #[arg(long, default_value = "80")]
    pub max_width: usize,
}

/// Arguments for decode command
#[derive(Debug, Parser)]
pub struct DecodeArgs {
    /// Input file (TOON), or stdin if omitted
    #[arg(value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output file, or stdout if omitted
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    pub output_format: OutputFormat,

    /// Pretty-print JSON output
    #[arg(short, long)]
    pub pretty: bool,
}

/// Arguments for check command
#[derive(Debug, Parser)]
pub struct CheckArgs {
    /// Input file(s) (TOON), or stdin if omitted or "-"
    #[arg(value_name = "FILE")]
    pub input: Vec<PathBuf>,

    /// Diagnostic output format
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: DiagnosticFormat,

    /// Minimum severity level to report
    #[arg(short, long, value_enum, default_value = "error")]
    pub severity: Severity,
}

/// Arguments for format command
#[derive(Debug, Parser)]
pub struct FormatArgs {
    /// Input file (TOON), or stdin if omitted
    #[arg(value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output file, or stdout if omitted (overwrites input if omitted)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Indentation size (spaces)
    #[arg(short, long, default_value = "2")]
    pub indent: usize,

    /// Use tabs for indentation
    #[arg(long)]
    pub tabs: bool,

    /// Array style preference
    #[arg(short = 's', long, value_enum, default_value = "auto")]
    pub array_style: ArrayStyle,

    /// Maximum line width for tabular arrays
    #[arg(long, default_value = "80")]
    pub max_width: usize,

    /// Check formatting without writing changes
    #[arg(long)]
    pub check: bool,
}

/// Arguments for symbols command
#[derive(Debug, Parser)]
pub struct SymbolsArgs {
    /// Input file (TOON), or stdin if omitted
    #[arg(value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "tree")]
    pub format: SymbolsFormat,

    /// Show value types in output
    #[arg(short, long)]
    pub types: bool,

    /// Show positions (line:column) in output
    #[arg(short, long)]
    pub positions: bool,
}

/// Arguments for diagnose command
#[derive(Debug, Parser)]
pub struct DiagnoseArgs {
    /// Input file (TOON), or stdin if omitted
    #[arg(value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    pub format: DiagnoseFormat,

    /// Include source code context in diagnostics
    #[arg(short, long)]
    pub context: bool,

    /// Minimum severity level to report
    #[arg(short, long, value_enum, default_value = "error")]
    pub severity: Severity,
}

/// Diagnostic output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum DiagnosticFormat {
    /// Human-readable text with colors
    Text,
    /// JSON output
    Json,
    /// GitHub Actions annotations
    Github,
}

/// Symbol output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum SymbolsFormat {
    /// Tree structure with indentation
    Tree,
    /// JSON output
    Json,
    /// Flat list with paths
    Flat,
}

/// Diagnostic output format for diagnose command
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum DiagnoseFormat {
    /// JSON output
    Json,
    /// SARIF (Static Analysis Results Interchange Format)
    Sarif,
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[non_exhaustive]
pub enum Severity {
    /// Hints for improvements
    Hint,
    /// Informational messages
    Info,
    /// Warnings about potential issues
    Warning,
    /// Errors that prevent parsing
    Error,
}

/// Input format for encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum InputFormat {
    /// JSON input
    Json,
    /// YAML input
    Yaml,
}

/// Output format for decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum OutputFormat {
    /// JSON output
    Json,
    /// YAML output
    Yaml,
}

/// Array formatting style
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum ArrayStyle {
    /// Automatically choose best style
    Auto,
    /// Expanded (multi-line) arrays
    Expanded,
    /// Inline (single-line) arrays
    Inline,
    /// Tabular (CSV-style) arrays
    Tabular,
}

/// Delimiter for tabular arrays
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[non_exhaustive]
pub enum Delimiter {
    /// Comma delimiter
    Comma,
    /// Tab delimiter
    Tab,
    /// Pipe delimiter
    Pipe,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["toon-lsp"]);
        assert_eq!(cli.verbose, 0);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_verbose_flags() {
        let cli = Cli::parse_from(["toon-lsp", "-v"]);
        assert_eq!(cli.verbose, 1);

        let cli = Cli::parse_from(["toon-lsp", "-vv"]);
        assert_eq!(cli.verbose, 2);

        let cli = Cli::parse_from(["toon-lsp", "-vvv"]);
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn test_encode_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "encode"]);
        if let Some(Command::Encode(args)) = cli.command {
            assert_eq!(args.indent, 2);
            assert!(!args.tabs);
            assert_eq!(args.input_format, InputFormat::Json);
            assert_eq!(args.array_style, ArrayStyle::Auto);
            assert_eq!(args.max_width, 80);
        } else {
            panic!("Expected Encode command");
        }
    }

    #[test]
    fn test_decode_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "decode"]);
        if let Some(Command::Decode(args)) = cli.command {
            assert_eq!(args.output_format, OutputFormat::Json);
            assert!(!args.pretty);
        } else {
            panic!("Expected Decode command");
        }
    }

    #[test]
    fn test_check_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "check"]);
        if let Some(Command::Check(args)) = cli.command {
            assert_eq!(args.format, DiagnosticFormat::Text);
            assert_eq!(args.severity, Severity::Error);
        } else {
            panic!("Expected Check command");
        }
    }

    #[test]
    fn test_format_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "format"]);
        if let Some(Command::Format(args)) = cli.command {
            assert_eq!(args.indent, 2);
            assert!(!args.tabs);
            assert!(!args.check);
            assert_eq!(args.array_style, ArrayStyle::Auto);
        } else {
            panic!("Expected Format command");
        }
    }

    #[test]
    fn test_symbols_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "symbols"]);
        if let Some(Command::Symbols(args)) = cli.command {
            assert_eq!(args.format, SymbolsFormat::Tree);
            assert!(!args.types);
            assert!(!args.positions);
        } else {
            panic!("Expected Symbols command");
        }
    }

    #[test]
    fn test_diagnose_defaults() {
        let cli = Cli::parse_from(["toon-lsp", "diagnose"]);
        if let Some(Command::Diagnose(args)) = cli.command {
            assert_eq!(args.format, DiagnoseFormat::Json);
            assert!(!args.context);
            assert_eq!(args.severity, Severity::Error);
        } else {
            panic!("Expected Diagnose command");
        }
    }

    #[test]
    fn test_lsp_command() {
        let cli = Cli::parse_from(["toon-lsp", "lsp"]);
        assert!(matches!(cli.command, Some(Command::Lsp)));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Info > Severity::Hint);
    }
}
