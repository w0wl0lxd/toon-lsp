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
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str("[]\n");
            } else {
                encode_array_body(&mut out, arr, 0, indent, delim)?;
            }
        }
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
    if arr.is_empty() {
        out.push_str(": []\n");
        Ok(())
    } else {
        encode_array_body(out, arr, level, indent, delim)
    }
}

/// Emits an array value starting from the `[count]...` header, choosing inline,
/// tabular, or expanded form. The key (and any leading indent) must already be
/// written by the caller.
fn encode_array_body(
    out: &mut String,
    arr: &[Value],
    level: usize,
    indent: usize,
    delim: Delimiter,
) -> EncodeResult<()> {
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
    } else if let Some(fields) = tabular_fields(arr) {
        emit_tabular(out, arr, &fields, level, indent, delim);
    } else {
        out.push('[');
        out.push_str(&arr.len().to_string());
        out.push_str("]:\n");
        encode_expanded_items(out, arr, level + 1, indent, delim)?;
    }
    Ok(())
}

/// Returns the ordered field list if `arr` qualifies for TOON tabular emission:
/// non-empty, every element an object with an identical key set (field order
/// taken from the first element), and every value a scalar.
fn tabular_fields(arr: &[Value]) -> Option<Vec<String>> {
    let Some(Value::Object(first)) = arr.first() else {
        return None;
    };
    if first.is_empty() || first.values().any(|v| !is_scalar(v)) {
        return None;
    }
    let fields: Vec<String> = first.keys().cloned().collect();
    for item in &arr[1..] {
        let Value::Object(map) = item else {
            return None;
        };
        if map.len() != fields.len() || map.values().any(|v| !is_scalar(v)) {
            return None;
        }
        if fields.iter().any(|f| !map.contains_key(f)) {
            return None;
        }
    }
    Some(fields)
}

fn emit_tabular(
    out: &mut String,
    arr: &[Value],
    fields: &[String],
    level: usize,
    indent: usize,
    delim: Delimiter,
) {
    out.push('[');
    out.push_str(&arr.len().to_string());
    out.push_str("]{");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(delim.as_char());
        }
        emit_scalar_string(out, field, delim);
    }
    out.push_str("}:\n");
    for item in arr {
        let Value::Object(map) = item else {
            continue;
        };
        push_indent(out, level + 1, indent);
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                out.push(delim.as_char());
            }
            if let Some(value) = map.get(field) {
                let _ = emit_json_scalar(out, value, delim);
            }
        }
        out.push('\n');
    }
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
                out.push_str("- ");
                encode_array_body(out, inner, level, indent, delim)?;
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
    encode_array_body(out, arr, level, indent, delim)
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

    #[test]
    fn uniform_object_array_is_tabular() {
        let out = encode(&json!({
            "users": [{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]
        }))
        .unwrap();
        assert_eq!(out, "users[2]{id,name}:\n  1,Alice\n  2,Bob\n");
    }

    #[test]
    fn non_uniform_object_array_falls_back_to_expanded() {
        let out = encode(&json!({"rows":[{"x":1},{"y":2}]})).unwrap();
        assert!(out.contains("- "), "expected expanded rows, got: {out}");
        assert!(!out.contains('{'), "must not emit tabular header, got: {out}");
    }

    #[test]
    fn nested_values_are_not_tabular() {
        let out = encode(&json!({"rows":[{"a":{"b":1}},{"a":{"b":2}}]})).unwrap();
        assert!(!out.contains('{'), "nested values must not be tabular: {out}");
    }

    #[test]
    fn tabular_applies_to_array_field_inside_expanded_item() {
        let out = encode(&json!([
            {"members":[{"id":1,"n":"x"},{"id":2,"n":"y"}]}
        ]))
        .unwrap();
        assert!(
            out.contains("members[2]{id,n}:"),
            "nested array field should be tabular: {out}"
        );
    }
}
