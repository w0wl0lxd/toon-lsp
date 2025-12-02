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

//! Decode command implementation: TOON → JSON/YAML conversion.
//!
//! This module wraps `toon_format::decode()` with CLI functionality including:
//! - Stdin support via `-` argument
//! - Output file support via `-o/--output`
//! - Output format selection (JSON or YAML)
//! - Pretty-printing option for JSON

use std::fs::File;
use std::io::{self, Write};

use super::convert::{decode_toon, write_json, write_yaml};
use super::error::{CliError, CliResult, ExitCode};
use super::io_utils::{read_file, read_stdin};
use super::{DecodeArgs, OutputFormat};

/// Execute the decode command.
///
/// Converts TOON input to JSON or YAML format.
///
/// # Errors
///
/// Returns `CliError` if:
/// - Input file cannot be read
/// - TOON parsing fails (syntax error) - returns exit code 2
/// - Output file cannot be written
pub fn execute(args: &DecodeArgs) -> CliResult<()> {
    // Read TOON input
    let toon_content = read_input(args)?;

    // Decode TOON to JSON value
    let value = decode_toon(&toon_content).map_err(|e| {
        // Convert decode errors to validation errors (exit code 2)
        CliError::Validation(e.to_string())
    })?;

    // Write output in requested format
    write_output(args, &value)?;

    Ok(())
}

/// Read input from file or stdin based on args.
fn read_input(args: &DecodeArgs) -> CliResult<String> {
    match &args.input {
        Some(path) if path.as_os_str() == "-" => read_stdin(),
        Some(path) => read_file(path),
        None => read_stdin(),
    }
}

/// Write output to file or stdout based on args.
fn write_output(args: &DecodeArgs, value: &serde_json::Value) -> CliResult<()> {
    if let Some(path) = &args.output {
        let file = File::create(path).map_err(|e| {
            CliError::Io(io::Error::new(
                e.kind(),
                format!("Failed to create '{}': {}", path.display(), e),
            ))
        })?;
        write_to_writer(file, args, value)
    } else {
        let stdout = io::stdout();
        let handle = stdout.lock();
        write_to_writer(handle, args, value)
    }
}

/// Write value to a writer in the requested format.
fn write_to_writer<W: Write>(
    writer: W,
    args: &DecodeArgs,
    value: &serde_json::Value,
) -> CliResult<()> {
    match args.output_format {
        OutputFormat::Json => write_json(writer, value, args.pretty),
        OutputFormat::Yaml => write_yaml(writer, value),
    }
}

/// Get exit code for decode errors.
///
/// TOON parse errors return exit code 2 (validation failure).
pub fn error_exit_code(error: &CliError) -> ExitCode {
    match error {
        CliError::Validation(_) | CliError::Decode(_) => ExitCode::ValidationFailed,
        _ => error.exit_code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple_toon() {
        let toon = "key: value\n";
        let value = decode_toon(toon).expect("decode");
        assert_eq!(value.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_decode_nested_toon() {
        let toon = "server:\n  host: localhost\n  port: 8080\n";
        let value = decode_toon(toon).expect("decode");
        let server = value.get("server").expect("server key");
        assert_eq!(server.get("host").and_then(|v| v.as_str()), Some("localhost"));
    }

    #[test]
    fn test_error_exit_code() {
        let validation_err = CliError::Validation("test".to_string());
        assert_eq!(error_exit_code(&validation_err), ExitCode::ValidationFailed);

        let decode_err = CliError::Decode("test".to_string());
        assert_eq!(error_exit_code(&decode_err), ExitCode::ValidationFailed);

        let io_err = CliError::Io(io::Error::new(io::ErrorKind::NotFound, "not found"));
        assert_eq!(error_exit_code(&io_err), ExitCode::Error);
    }
}
