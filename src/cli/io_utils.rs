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

//! Shared I/O utilities for CLI commands.
//!
//! This module provides common I/O operations used across multiple CLI commands,
//! eliminating code duplication and ensuring consistent error handling.

use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use super::error::{CliError, CliResult};

/// Read content from stdin with automatic lock management (RAII).
///
/// # Errors
///
/// Returns `CliError::Io` if stdin cannot be read.
pub fn read_stdin() -> CliResult<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
}

/// Read content from a file with enhanced error messages.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Errors
///
/// Returns `CliError::Io` if the file cannot be opened or read.
/// Error message includes the file path for better diagnostics.
pub fn read_file(path: &Path) -> CliResult<String> {
    let file = File::open(path).map_err(|e| {
        CliError::Io(io::Error::new(
            e.kind(),
            format!("Failed to open '{}': {}", path.display(), e),
        ))
    })?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
}

/// Read from either stdin or file based on input path.
///
/// Input source determination:
/// - `None` or `Some("-")` → read from stdin
/// - `Some(path)` → read from file at path
///
/// # Arguments
///
/// * `input` - Optional path; None or "-" means stdin
///
/// # Errors
///
/// Returns `CliError::Io` if the source cannot be read.
pub fn read_input(input: &Option<PathBuf>) -> CliResult<String> {
    match input {
        Some(path) if path.as_os_str() == "-" => read_stdin(),
        Some(path) => read_file(path),
        None => read_stdin(),
    }
}

/// Write string content to stdout or file.
///
/// Output destination determination:
/// - `None` → write to stdout
/// - `Some(path)` → write to file at path
///
/// # Arguments
///
/// * `output` - Optional path; None means stdout
/// * `content` - String content to write
///
/// # Errors
///
/// Returns `CliError::Io` if the destination cannot be written.
pub fn write_output(output: &Option<PathBuf>, content: &str) -> CliResult<()> {
    if let Some(path) = output {
        let mut file = File::create(path).map_err(|e| {
            CliError::Io(io::Error::new(
                e.kind(),
                format!("Failed to create '{}': {}", path.display(), e),
            ))
        })?;
        file.write_all(content.as_bytes())?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(content.as_bytes())?;
    }
    Ok(())
}

/// Write bytes to stdout or file.
///
/// Same as `write_output` but for byte slices.
pub fn write_output_bytes(output: &Option<PathBuf>, bytes: &[u8]) -> CliResult<()> {
    if let Some(path) = output {
        let mut file = File::create(path).map_err(|e| {
            CliError::Io(io::Error::new(
                e.kind(),
                format!("Failed to create '{}': {}", path.display(), e),
            ))
        })?;
        file.write_all(bytes)?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_file_success() {
        let mut temp = NamedTempFile::new().expect("create temp file");
        writeln!(temp, "test content").expect("write content");

        let content = read_file(temp.path()).expect("read file");
        assert!(content.contains("test content"));
    }

    #[test]
    fn test_read_file_not_found() {
        let result = read_file(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::Io(_)));
    }

    #[test]
    fn test_read_input_with_path() {
        let mut temp = NamedTempFile::new().expect("create temp file");
        writeln!(temp, "file content").expect("write content");

        let content = read_input(&Some(temp.path().to_path_buf())).expect("read input");
        assert!(content.contains("file content"));
    }

    #[test]
    fn test_write_output_to_file() {
        let temp = NamedTempFile::new().expect("create temp file");
        let path = temp.path().to_path_buf();

        write_output(&Some(path.clone()), "written content").expect("write output");

        let content = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(content, "written content");
    }

    #[test]
    fn test_write_output_bytes_to_file() {
        let temp = NamedTempFile::new().expect("create temp file");
        let path = temp.path().to_path_buf();

        write_output_bytes(&Some(path.clone()), b"byte content").expect("write bytes");

        let content = std::fs::read(&path).expect("read back");
        assert_eq!(content, b"byte content");
    }
}
