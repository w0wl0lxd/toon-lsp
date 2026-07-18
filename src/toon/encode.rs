//! JSON-to-TOON encoder.
//!
//! Canonicalizes a [`serde_json::Value`] into TOON text, routing every scalar
//! through the shared [`crate::toon::emit`] core. Object key order is preserved
//! (the crate enables `serde_json`'s `preserve_order` feature).

use std::borrow::Cow;
use std::fmt::Write as _;

use serde_json::{Map, Value};

use crate::toon::emit::{Delimiter, emit_json_scalar, emit_scalar_string};
use crate::toon::error::EncodeResult;

pub fn encode(value: &Value) -> EncodeResult<String> {
    encode_with_config(value, &crate::toon::ToonConfig::default())
}

/// Encodes `value` as TOON using `indent` spaces per nesting level.
///
/// # Errors
/// Returns [`crate::toon::EncodeError`] if `value` contains something with no
/// TOON representation.
pub fn encode_with_indent(value: &Value, indent: usize) -> EncodeResult<String> {
    let mut config = crate::toon::ToonConfig::default();
    config.indent = indent;
    encode_with_config(value, &config)
}

/// Encodes `value` as TOON using custom configuration options.
///
/// # Errors
/// Returns [`crate::toon::EncodeError`] if `value` contains something with no
/// TOON representation.
pub fn encode_with_config(value: &Value, config: &crate::toon::ToonConfig) -> EncodeResult<String> {
    let mut out = String::new();
    encode_into(value, config, &mut out)?;
    Ok(out)
}

/// Encodes `value` as TOON into the supplied `out` buffer, appending starting at
/// `out.len()` and without clearing `out`.
///
/// When `config.fold_keys` and `config.flatten_keys` are both `false` (the
/// default) and `out` has sufficient capacity, this performs no heap
/// allocations on the encode path.
///
/// # Errors
/// Returns [`crate::toon::EncodeError`] if `value` contains something with no
/// TOON representation. If `out` has insufficient capacity, `out` may be
/// reallocated by `String` growth.
pub fn encode_into(
    value: &Value,
    config: &crate::toon::ToonConfig,
    out: &mut String,
) -> EncodeResult<()> {
    let value_to_encode: Cow<'_, Value> = if config.flatten_keys {
        Cow::Owned(crate::toon::fold::flatten_keys(value))
    } else if config.fold_keys {
        Cow::Owned(crate::toon::fold::fold_keys(value))
    } else {
        Cow::Borrowed(value)
    };

    let delim = config.delimiter;
    let indent = config.indent;
    match value_to_encode.as_ref() {
        Value::Object(map) => encode_object(out, map, 0, indent, delim)?,
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str("[]\n");
            } else {
                encode_array_body(out, arr, 0, indent, delim)?;
            }
        }
        scalar => {
            let _ = emit_json_scalar(out, scalar, delim);
            out.push('\n');
        }
    }
    Ok(())
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
        let _ = write!(out, "{}", arr.len());
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
    } else if let Some(first) = arr.first().and_then(|v| v.as_object()).filter(|m| !m.is_empty())
        && first.values().all(is_scalar)
        && arr.iter().all(|v| is_uniform_object(v, first))
    {
        // Tabular form. Avoid allocating a Vec<String> for field names by
        // iterating the first object's keys directly for the header and each
        // row.
        emit_tabular(out, arr, first, level, indent, delim);
    } else {
        out.push('[');
        let _ = write!(out, "{}", arr.len());
        out.push_str("]:\n");
        encode_expanded_items(out, arr, level + 1, indent, delim)?;
    }
    Ok(())
}

/// Returns `true` when `value` is a uniform tabular object matching `first`:
/// same field set and all scalar values. Key order is not required to match;
/// emission uses `first`'s order and looks up values in each row.
fn is_uniform_object(value: &Value, first: &Map<String, Value>) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    if map.len() != first.len() {
        return false;
    }
    for field in first.keys() {
        let Some(v) = map.get(field) else {
            return false;
        };
        if !is_scalar(v) {
            return false;
        }
    }
    true
}

fn emit_tabular(
    out: &mut String,
    arr: &[Value],
    first: &Map<String, Value>,
    level: usize,
    indent: usize,
    delim: Delimiter,
) {
    out.push('[');
    let _ = write!(out, "{}", arr.len());
    out.push_str("]{");
    for (i, field) in first.keys().enumerate() {
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
        for (i, field) in first.keys().enumerate() {
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
        assert!(out.contains("members[2]{id,n}:"), "nested array field should be tabular: {out}");
    }

    #[test]
    fn encode_into_appends_without_clearing() {
        let mut out = String::from("prefix\n");
        encode_into(&json!({"k":"v"}), &crate::toon::ToonConfig::default(), &mut out).unwrap();
        assert_eq!(out, "prefix\nk: v\n");
    }

    #[test]
    fn encode_into_matches_encode_with_config() {
        let value = json!({
            "users": [{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]
        });
        let config = crate::toon::ToonConfig::default();
        let owned = encode_with_config(&value, &config).unwrap();
        let mut into = String::new();
        encode_into(&value, &config, &mut into).unwrap();
        assert_eq!(owned, into);
    }

    #[test]
    fn encode_into_preserves_existing_prefix_across_scalar_arrays() {
        let mut out = String::from("header: 1\n");
        encode_into(&json!({"tags":["a","b"]}), &crate::toon::ToonConfig::default(), &mut out)
            .unwrap();
        assert_eq!(out, "header: 1\ntags[2]: a,b\n");
    }

    #[test]
    fn encode_into_with_sufficient_capacity_does_not_reallocate() {
        // Regression check for the zero-allocation claim: when `out` already
        // has enough spare capacity and no key folding/flattening is
        // requested, encoding must not grow (and therefore not reallocate)
        // the buffer.
        let value = json!({"k": "v"});
        let config = crate::toon::ToonConfig::default();
        let needed = encode_with_config(&value, &config).unwrap().len();
        let mut out = String::with_capacity(needed + 64);
        let ptr_before = out.as_ptr();
        let cap_before = out.capacity();
        encode_into(&value, &config, &mut out).unwrap();
        assert_eq!(out.as_ptr(), ptr_before, "buffer should not have reallocated");
        assert_eq!(out.capacity(), cap_before, "capacity should be unchanged");
    }

    #[test]
    fn encode_into_respects_fold_keys_config() {
        let mut config = crate::toon::ToonConfig::default();
        config.fold_keys = true;
        let mut out = String::new();
        encode_into(&json!({"a":{"b":{"c":1}}}), &config, &mut out).unwrap();
        assert_eq!(out, "a.b.c: 1\n");
    }

    #[test]
    fn encode_into_respects_flatten_keys_config() {
        let mut config = crate::toon::ToonConfig::default();
        config.flatten_keys = true;
        let mut out = String::new();
        encode_into(&json!({"a":{"b":1,"c":2}}), &config, &mut out).unwrap();
        assert_eq!(out, "a.b: 1\na.c: 2\n");
    }

    #[test]
    fn encode_into_top_level_scalar() {
        let mut out = String::new();
        encode_into(&json!(1.5), &crate::toon::ToonConfig::default(), &mut out).unwrap();
        assert_eq!(out, "1.5\n");
    }

    #[test]
    fn encode_into_top_level_empty_array() {
        let mut out = String::new();
        encode_into(&json!([]), &crate::toon::ToonConfig::default(), &mut out).unwrap();
        assert_eq!(out, "[]\n");
    }

    #[test]
    fn tabular_uses_first_row_field_order_even_when_later_rows_differ() {
        // is_uniform_object() only checks the field *set*, not the underlying
        // map's iteration order, so the header (and every row's field order)
        // must follow the first object's key order regardless of how later
        // rows declare their keys.
        let out = encode(&json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"name": "Bob", "id": 2}
            ]
        }))
        .unwrap();
        assert_eq!(out, "users[2]{id,name}:\n  1,Alice\n  2,Bob\n");
    }

    #[test]
    fn same_length_but_different_field_names_falls_back_to_expanded() {
        // Same key *count* as the first object, but a different field name,
        // so `is_uniform_object` must reject it via the `map.get(field)`
        // lookup even though `map.len() == first.len()`.
        let out = encode(&json!({"rows": [{"a": 1}, {"b": 2}]})).unwrap();
        assert!(!out.contains('{'), "must not emit tabular header, got: {out}");
        assert!(out.contains("- a: 1"), "expected expanded rows, got: {out}");
    }

    #[test]
    fn tabular_rejects_when_later_row_has_nested_value() {
        // The first row is fully scalar and matches the field set, but a
        // later row's value for a shared field is non-scalar, so
        // `is_uniform_object` must reject it.
        let out = encode(&json!({"rows": [{"a": 1}, {"a": {"b": 2}}]})).unwrap();
        assert!(!out.contains('{'), "non-scalar in later row must block tabular: {out}");
    }

    #[test]
    fn empty_object_first_row_falls_back_to_expanded() {
        // `first.filter(|m| !m.is_empty())` excludes an empty first object
        // from tabular consideration.
        let out = encode(&json!({"rows": [{}, {}]})).unwrap();
        assert!(!out.contains('{'), "empty-object rows must not be tabular: {out}");
    }
}
