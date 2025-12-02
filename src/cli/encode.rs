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

//! Encode command implementation: JSON/YAML → TOON conversion.
//!
//! This module wraps `toon_format::encode()` with CLI functionality including:
//! - File extension auto-detection for input format
//! - Stdin support via `-` argument
//! - Output file support via `-o/--output`
//! - Encoding options mapping to `toon_format::EncodeOptions`
//!
//! ## Serde Type Conversion Behavior
//!
//! The `toon-format` crate uses serde for serialization. This means:
//! - **Dates**: Types implementing `Serialize` with ISO8601 formatting (like chrono) convert automatically
//! - **Binary data**: Serializes as base64 when the type provides such serialization
//! - **Unsupported types**: Non-serializable types return `ToonError::SerializationError` (exit code 1)
//!
//! The API uses pure `Result<T, ToonError>` with no warning mechanism - conversions either
//! succeed or fail completely.

use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;

use super::convert::{encode_json, read_json, read_yaml};
use super::error::{CliError, CliResult};
use super::{EncodeArgs, InputFormat};

/// Execute the encode command.
///
/// Converts JSON or YAML input to TOON format.
///
/// # Errors
///
/// Returns `CliError` if:
/// - Input file cannot be read
/// - Input format cannot be determined
/// - Parsing fails (JSON/YAML syntax error)
/// - Encoding fails (toon-format error)
/// - Output file cannot be written
pub fn execute(args: &EncodeArgs) -> CliResult<()> {
    // Determine input format from file extension or explicit flag
    let format = detect_input_format(args)?;

    // Read and parse input
    let value = read_input(args, format)?;

    // Encode to TOON
    let toon = encode_json(&value)?;

    // Write output
    write_output(args, &toon)?;

    Ok(())
}

/// Detect input format from file extension or explicit flag.
fn detect_input_format(args: &EncodeArgs) -> CliResult<InputFormat> {
    // If explicit format specified, use it
    // Note: clap defaults to JSON, but we can override via extension
    if let Some(ref path) = args.input {
        // "-" means stdin, use the format flag
        if path.as_os_str() == "-" {
            return Ok(args.input_format);
        }

        // Auto-detect from extension if possible
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "json" => return Ok(InputFormat::Json),
                "yaml" | "yml" => return Ok(InputFormat::Yaml),
                _ => {} // Fall through to explicit format
            }
        }
    }

    // Use explicit format flag
    Ok(args.input_format)
}

/// Read input from file or stdin based on args.
fn read_input(args: &EncodeArgs, format: InputFormat) -> CliResult<serde_json::Value> {
    match &args.input {
        Some(path) if path.as_os_str() == "-" => {
            // Read from stdin
            read_from_stdin(format)
        }
        Some(path) => {
            // Read from file
            read_from_file(path, format)
        }
        None => {
            // No input specified, read from stdin
            read_from_stdin(format)
        }
    }
}

/// Read and parse from stdin.
fn read_from_stdin(format: InputFormat) -> CliResult<serde_json::Value> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    parse_input(reader, format)
}

/// Read and parse from a file.
fn read_from_file(path: &Path, format: InputFormat) -> CliResult<serde_json::Value> {
    let file = File::open(path).map_err(|e| {
        CliError::Io(io::Error::new(
            e.kind(),
            format!("Failed to open '{}': {}", path.display(), e),
        ))
    })?;
    let reader = BufReader::new(file);
    parse_input(reader, format)
}

/// Parse input based on format.
fn parse_input<R: Read>(reader: R, format: InputFormat) -> CliResult<serde_json::Value> {
    match format {
        InputFormat::Json => read_json(reader),
        InputFormat::Yaml => read_yaml(reader),
    }
}

/// Write output to file or stdout based on args.
fn write_output(args: &EncodeArgs, toon: &str) -> CliResult<()> {
    if let Some(path) = &args.output {
        let mut file = File::create(path).map_err(|e| {
            CliError::Io(io::Error::new(
                e.kind(),
                format!("Failed to create '{}': {}", path.display(), e),
            ))
        })?;
        file.write_all(toon.as_bytes())?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(toon.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_encode_args(input: Option<&str>, format: InputFormat) -> EncodeArgs {
        EncodeArgs {
            input: input.map(PathBuf::from),
            output: None,
            input_format: format,
            indent: 2,
            array_style: super::super::ArrayStyle::Auto,
            tabs: false,
            max_width: 80,
        }
    }

    #[test]
    fn test_detect_json_from_extension() {
        let args = make_encode_args(Some("test.json"), InputFormat::Yaml);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Json);
    }

    #[test]
    fn test_detect_yaml_from_extension() {
        let args = make_encode_args(Some("test.yaml"), InputFormat::Json);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Yaml);
    }

    #[test]
    fn test_detect_yml_from_extension() {
        let args = make_encode_args(Some("test.yml"), InputFormat::Json);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Yaml);
    }

    #[test]
    fn test_stdin_uses_explicit_format() {
        let args = make_encode_args(Some("-"), InputFormat::Yaml);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Yaml);
    }

    #[test]
    fn test_unknown_extension_uses_explicit_format() {
        let args = make_encode_args(Some("test.txt"), InputFormat::Json);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Json);
    }

    #[test]
    fn test_no_input_uses_explicit_format() {
        let args = make_encode_args(None, InputFormat::Yaml);
        let format = detect_input_format(&args).expect("detect format");
        assert_eq!(format, InputFormat::Yaml);
    }
}
