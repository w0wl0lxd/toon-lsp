//! JSON-to-TOON encoder.
//!
//! Canonicalizes a [`serde_json::Value`] into TOON text, routing every scalar
//! through the shared [`crate::toon::emit`] core. Object key order is preserved
//! (the crate enables `serde_json`'s `preserve_order` feature).

use serde_json::{Map, Value};

use crate::toon::emit::{Delimiter, emit_json_scalar, emit_scalar_string};
use crate::toon::error::EncodeResult;

/// Encodes `value` as TOON using the default 2-space indent.
///
/// # Errors
/// Returns [`crate::toon::EncodeError`] if `value` contains something with no
/// TOON representation.
pub fn encode(value: &Value) -> EncodeResult<String> {
    encode_with_indent(value, 2)
}

/// Encodes `value` as TOON using `indent` spaces per nesting level.
///
/// # Errors
/// Returns [`crate::toon::EncodeError`] if `value` contains something with no
/// TOON representation.
pub fn encode_with_indent(value: &Value, indent: usize) -> EncodeResult<String> {
    let mut out = String::new();
    let delim = Delimiter::Comma;
    match value {
        Value::Object(map) => encode_object(&mut out, map, 0, indent, delim)?,
        Value::Array(arr) => encode_expanded_items(&mut out, arr, 0, indent, delim)?,
        scalar => {
            let _ = emit_json_scalar(&mut out, scalar, delim);
            out.push('\n');
        }
    }
    Ok(out)
}

fn push_indent(out: &mut String, level: usize, indent: usize) {
    for _ in 0..(level * indent) {
        out.push(' ');
    }
}

fn encode_object(
    out: &mut String,
    map: &Map<String, Value>,
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
    for (key, value) in map {
        match value {
            Value::Array(arr) => encode_array_field(out, key, arr, level, indent, delim)?,
            Value::Object(child) => {
                push_indent(out, level, indent);
                emit_key(out, key, delim);
                out.push_str(":\n");
                encode_object(out, child, level + 1, indent, delim)?;
            }
            scalar => {
                push_indent(out, level, indent);
                emit_key(out, key, delim);
                out.push_str(": ");
                let _ = emit_json_scalar(out, scalar, delim);
                out.push('\n');
            }
        }
    }
    Ok(())
}

fn encode_array_field(
    out: &mut String,
    key: &str,
    arr: &[Value],
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
    push_indent(out, level, indent);
    emit_key(out, key, delim);
    if arr.iter().all(is_scalar) {
        out.push('[');
        out.push_str(&arr.len().to_string());
        out.push_str("]:");
        if !arr.is_empty() {
            out.push(' ');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(delim.as_char());
                }
                let _ = emit_json_scalar(out, item, delim);
            }
        }
        out.push('\n');
    } else {
        out.push('[');
        out.push_str(&arr.len().to_string());
        out.push_str("]:\n");
        encode_expanded_items(out, arr, level + 1, indent, delim)?;
    }
    Ok(())
}

fn encode_expanded_items(
    out: &mut String,
    arr: &[Value],
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
    for item in arr {
        match item {
            Value::Object(map) => {
                encode_expanded_object(out, map, level, indent, delim)?;
            }
            Value::Array(inner) => {
                push_indent(out, level, indent);
                out.push_str("-\n");
                encode_expanded_items(out, inner, level + 1, indent, delim)?;
            }
            scalar => {
                push_indent(out, level, indent);
                out.push_str("- ");
                let _ = emit_json_scalar(out, scalar, delim);
                out.push('\n');
            }
        }
    }
    Ok(())
}

fn encode_expanded_object(
    out: &mut String,
    map: &Map<String, Value>,
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
    if map.is_empty() {
        push_indent(out, level, indent);
        out.push_str("-\n");
        return Ok(());
    }
    for (i, (key, value)) in map.iter().enumerate() {
        if i == 0 {
            push_indent(out, level, indent);
            out.push_str("- ");
        } else {
            push_indent(out, level + 1, indent);
        }
        match value {
            Value::Object(child) => {
                emit_key(out, key, delim);
                out.push_str(":\n");
                encode_object(out, child, level + 2, indent, delim)?;
            }
            Value::Array(arr) => {
                encode_array_field_inline_key(out, key, arr, level + 1, indent, delim)?;
            }
            scalar => {
                emit_key(out, key, delim);
                out.push_str(": ");
                let _ = emit_json_scalar(out, scalar, delim);
                out.push('\n');
            }
        }
    }
    Ok(())
}

fn encode_array_field_inline_key(
    out: &mut String,
    key: &str,
    arr: &[Value],
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
    emit_key(out, key, delim);
    if arr.iter().all(is_scalar) {
        out.push('[');
        out.push_str(&arr.len().to_string());
        out.push_str("]:");
        if !arr.is_empty() {
            out.push(' ');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(delim.as_char());
                }
                let _ = emit_json_scalar(out, item, delim);
            }
        }
        out.push('\n');
    } else {
        out.push('[');
        out.push_str(&arr.len().to_string());
        out.push_str("]:\n");
        encode_expanded_items(out, arr, level + 1, indent, delim)?;
    }
    Ok(())
}

fn emit_key(out: &mut String, key: &str, delim: Delimiter) {
    emit_scalar_string(out, key, delim);
}

fn is_scalar(v: &Value) -> bool {
    !matches!(v, Value::Array(_) | Value::Object(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flat_object() {
        let out = encode(&json!({"name":"Alice","age":30})).unwrap();
        assert_eq!(out, "name: Alice\nage: 30\n");
    }

    #[test]
    fn nested_object_indents() {
        let out = encode(&json!({"user":{"name":"Bob"}})).unwrap();
        assert_eq!(out, "user:\n  name: Bob\n");
    }

    #[test]
    fn scalar_array_inline_with_count() {
        let out = encode(&json!({"tags":["a","b","c"]})).unwrap();
        assert_eq!(out, "tags[3]: a,b,c\n");
    }
}
