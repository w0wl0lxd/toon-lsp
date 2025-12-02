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

//! Thin wrappers around toon-format encoding/decoding operations.

use super::error::{CliError, CliResult};
use serde_json::Value as JsonValue;
use std::io::{Read, Write};

/// Encode JSON value to TOON format.
///
/// This is a thin wrapper around `toon_format::encode_default()`.
///
/// # Errors
///
/// Returns `CliError::Encode` if encoding fails.
pub fn encode_json(value: &JsonValue) -> CliResult<String> {
    toon_format::encode_default(value)
        .map_err(|e| CliError::encode(format!("Failed to encode JSON to TOON: {e}")))
}

/// Decode TOON string to JSON value.
///
/// This is a thin wrapper around `toon_format::decode_default()`.
///
/// # Errors
///
/// Returns `CliError::Decode` if decoding fails.
pub fn decode_toon(toon: &str) -> CliResult<JsonValue> {
    toon_format::decode_default(toon)
        .map_err(|e| CliError::decode(format!("Failed to decode TOON to JSON: {e}")))
}

/// Read JSON from a reader.
///
/// # Errors
///
/// Returns `CliError::Json` if parsing fails or `CliError::Io` if reading fails.
pub fn read_json<R: Read>(reader: R) -> CliResult<JsonValue> {
    serde_json::from_reader(reader).map_err(Into::into)
}

/// Read YAML from a reader.
///
/// # Errors
///
/// Returns `CliError::Yaml` if parsing fails or `CliError::Io` if reading fails.
pub fn read_yaml<R: Read>(reader: R) -> CliResult<JsonValue> {
    serde_yaml::from_reader(reader).map_err(Into::into)
}

/// Read TOON from a reader.
///
/// # Errors
///
/// Returns `CliError::Decode` if parsing fails or `CliError::Io` if reading fails.
pub fn read_toon<R: Read>(mut reader: R) -> CliResult<JsonValue> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    decode_toon(&content)
}

/// Write JSON to a writer with optional pretty printing.
///
/// # Errors
///
/// Returns `CliError::Json` if serialization fails or `CliError::Io` if writing fails.
pub fn write_json<W: Write>(writer: W, value: &JsonValue, pretty: bool) -> CliResult<()> {
    if pretty {
        serde_json::to_writer_pretty(writer, value)?;
    } else {
        serde_json::to_writer(writer, value)?;
    }
    Ok(())
}

/// Write YAML to a writer.
///
/// # Errors
///
/// Returns `CliError::Yaml` if serialization fails or `CliError::Io` if writing fails.
pub fn write_yaml<W: Write>(writer: W, value: &JsonValue) -> CliResult<()> {
    serde_yaml::to_writer(writer, value).map_err(Into::into)
}

/// Write TOON to a writer.
///
/// # Errors
///
/// Returns `CliError::Encode` if encoding fails or `CliError::Io` if writing fails.
pub fn write_toon<W: Write>(mut writer: W, value: &JsonValue) -> CliResult<()> {
    let toon = encode_json(value)?;
    writer.write_all(toon.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encode_decode_roundtrip() {
        let value = json!({
            "name": "test",
            "count": 42,
            "active": true
        });

        let toon = encode_json(&value).expect("encode failed");
        let decoded = decode_toon(&toon).expect("decode failed");

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_encode_simple_object() {
        let value = json!({"key": "value"});
        let toon = encode_json(&value).expect("encode failed");
        assert!(toon.contains("key"));
        assert!(toon.contains("value"));
    }

    #[test]
    fn test_decode_simple_object() {
        let toon = "key: value\n";
        let value = decode_toon(toon).expect("decode failed");
        assert_eq!(value, json!({"key": "value"}));
    }

    #[test]
    fn test_read_json() {
        let json_str = r#"{"key": "value"}"#;
        let value = read_json(json_str.as_bytes()).expect("read_json failed");
        assert_eq!(value, json!({"key": "value"}));
    }

    #[test]
    fn test_read_yaml() {
        let yaml_str = "key: value\n";
        let value = read_yaml(yaml_str.as_bytes()).expect("read_yaml failed");
        assert_eq!(value, json!({"key": "value"}));
    }

    #[test]
    fn test_read_toon() {
        let toon_str = "key: value\n";
        let value = read_toon(toon_str.as_bytes()).expect("read_toon failed");
        assert_eq!(value, json!({"key": "value"}));
    }

    #[test]
    fn test_write_json() {
        let value = json!({"key": "value"});
        let mut buffer = Vec::new();
        write_json(&mut buffer, &value, false).expect("write_json failed");
        let output = String::from_utf8(buffer).expect("invalid UTF-8");
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }

    #[test]
    fn test_write_json_pretty() {
        let value = json!({"key": "value"});
        let mut buffer = Vec::new();
        write_json(&mut buffer, &value, true).expect("write_json failed");
        let output = String::from_utf8(buffer).expect("invalid UTF-8");
        assert!(output.contains("key"));
        assert!(output.contains("value"));
        assert!(output.contains('\n')); // Pretty print adds newlines
    }

    #[test]
    fn test_write_yaml() {
        let value = json!({"key": "value"});
        let mut buffer = Vec::new();
        write_yaml(&mut buffer, &value).expect("write_yaml failed");
        let output = String::from_utf8(buffer).expect("invalid UTF-8");
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }

    #[test]
    fn test_write_toon() {
        let value = json!({"key": "value"});
        let mut buffer = Vec::new();
        write_toon(&mut buffer, &value).expect("write_toon failed");
        let output = String::from_utf8(buffer).expect("invalid UTF-8");
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }

    #[test]
    fn test_decode_invalid_toon() {
        let invalid_toon = "key: [unclosed array";
        let result = decode_toon(invalid_toon);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_invalid_json() {
        let invalid_json = "{invalid json}";
        let result = read_json(invalid_json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_complex_structure() {
        let value = json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "metadata": {
                "count": 2,
                "version": "1.0"
            }
        });

        let toon = encode_json(&value).expect("encode failed");
        let decoded = decode_toon(&toon).expect("decode failed");
        assert_eq!(value, decoded);
    }
}
