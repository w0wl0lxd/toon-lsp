//! Shared spec-conformant emitter core for the TOON codec.
//!
//! Both the JSON encoder and the LSP formatter route every string, number, and
//! scalar through these primitives so that quoting, escaping, and delimiter
//! handling are defined in exactly one place.

/// The active field delimiter for a TOON array or row context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// Comma-separated (the default).
    Comma,
    /// Tab-separated.
    Tab,
    /// Pipe-separated.
    Pipe,
}

impl Delimiter {
    /// The single character that separates fields under this delimiter.
    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }
}

/// Returns `true` when `s` matches the TOON number grammar and would therefore
/// round-trip as a number rather than a string if emitted unquoted.
///
/// Bare sign/dot tokens and the non-finite words (`inf`, `nan`, ...) are treated
/// as strings, not numbers.
fn is_toon_number(s: &str) -> bool {
    if s.is_empty() || s == "-" || s == "+" || s == "." {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity" | "nan" | "+nan" | "-nan"
    ) {
        return false;
    }
    s.parse::<f64>().is_ok()
}

/// Returns `true` if `s` must be quoted to emit unambiguously under `delim`.
///
/// A string is quoted when it is empty, has leading/trailing ASCII space,
/// contains a quote/backslash/control character, begins with a structural
/// marker, contains a `:` followed by a space, contains the active delimiter,
/// equals a reserved word, or would parse as a number. A `-` in the middle of a
/// token does not force quoting.
#[must_use]
pub fn needs_quotes(s: &str, delim: Delimiter) -> bool {
    if s.is_empty() {
        return true;
    }
    if s.starts_with(' ') || s.ends_with(' ') {
        return true;
    }
    if matches!(s, "true" | "false" | "null") {
        return true;
    }
    if is_toon_number(s) {
        return true;
    }
    match s.as_bytes()[0] {
        b'-' | b'[' | b'{' | b'#' => return true,
        _ => {}
    }
    let delim_char = delim.as_char();
    for ch in s.chars() {
        if ch == '"' || ch == '\\' || (ch as u32) < 0x20 {
            return true;
        }
        if ch == delim_char {
            return true;
        }
        if ch == ':' || ch == ' ' {
            return true;
        }
    }
    false
}

/// Appends `s` to `out`, escaping the characters that TOON requires inside a
/// quoted string.
///
/// Escapes `\`, `"`, newline, carriage return, and tab; other control
/// characters are emitted as `\uXXXX`; everything else is copied verbatim.
pub fn escape_into(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

/// Appends `s` to `out` as a TOON scalar string, quoting and escaping only when
/// [`needs_quotes`] says so.
pub fn emit_scalar_string(out: &mut String, s: &str, delim: Delimiter) {
    if needs_quotes(s, delim) {
        out.push('"');
        escape_into(out, s);
        out.push('"');
    } else {
        out.push_str(s);
    }
}

/// Appends the canonical TOON rendering of `n` to `out`.
///
/// Delegates to [`serde_json::Number`]'s own `Display`, which is backed by
/// `ryu`/`itoa`, guaranteeing byte-for-byte round-trip parity with `serde_json`:
/// integers print without a decimal point, floats in their minimal form.
pub fn emit_number(out: &mut String, n: &serde_json::Number) {
    use std::fmt::Write as _;
    let _ = write!(out, "{n}");
}

/// Appends the TOON rendering of `v` to `out` when it is a scalar.
///
/// Returns `true` if `v` was a scalar (null, bool, number, or string) and was
/// written, `false` for objects and arrays (which the caller must handle
/// structurally, leaving `out` untouched).
#[must_use]
pub fn emit_json_scalar(out: &mut String, v: &serde_json::Value, delim: Delimiter) -> bool {
    use serde_json::Value;
    match v {
        Value::Null => {
            out.push_str("null");
            true
        }
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
            true
        }
        Value::Number(n) => {
            emit_number(out, n);
            true
        }
        Value::String(s) => {
            emit_scalar_string(out, s, delim);
            true
        }
        Value::Array(_) | Value::Object(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_word_unquoted() {
        assert!(!needs_quotes("hello", Delimiter::Comma));
        assert!(!needs_quotes("a-b-c", Delimiter::Comma)); // internal dash is fine
    }

    #[test]
    fn reserved_and_numeric_quoted() {
        assert!(needs_quotes("true", Delimiter::Comma));
        assert!(needs_quotes("42", Delimiter::Comma));
        assert!(needs_quotes("-1.5", Delimiter::Comma));
    }

    #[test]
    fn delimiter_and_structure_quoted() {
        assert!(needs_quotes("a,b", Delimiter::Comma));
        assert!(!needs_quotes("a,b", Delimiter::Pipe)); // comma safe under pipe
        assert!(needs_quotes("a|b", Delimiter::Pipe));
        assert!(needs_quotes(" leading", Delimiter::Comma));
        assert!(needs_quotes("", Delimiter::Comma));
    }

    #[test]
    fn escape_only_spec_chars() {
        let mut s = String::new();
        escape_into(&mut s, "a\"b\\c\nd\te");
        assert_eq!(s, "a\\\"b\\\\c\\nd\\te");
    }

    #[test]
    fn emit_quotes_when_needed() {
        let mut s = String::new();
        emit_scalar_string(&mut s, "true", Delimiter::Comma);
        assert_eq!(s, "\"true\"");
        let mut s2 = String::new();
        emit_scalar_string(&mut s2, "hello", Delimiter::Comma);
        assert_eq!(s2, "hello");
    }

    #[test]
    fn numbers_canonical() {
        let mut s = String::new();
        emit_number(&mut s, &serde_json::Number::from(42));
        assert_eq!(s, "42");
        let mut s2 = String::new();
        emit_number(&mut s2, &serde_json::Number::from_f64(1.5).unwrap());
        assert_eq!(s2, "1.5");
        let mut s3 = String::new();
        emit_number(&mut s3, &serde_json::Number::from_f64(11.0).unwrap());
        assert_eq!(s3, "11.0");
        let mut s4 = String::new();
        emit_number(&mut s4, &serde_json::Number::from_f64(0.0).unwrap());
        assert_eq!(s4, "0.0");
        let mut s5 = String::new();
        emit_number(&mut s5, &serde_json::Number::from_f64(-0.0).unwrap());
        assert_eq!(s5, "-0.0");
    }

    #[test]
    fn scalar_dispatch() {
        let mut s = String::new();
        assert!(emit_json_scalar(&mut s, &serde_json::json!(true), Delimiter::Comma));
        assert_eq!(s, "true");
        let mut s2 = String::new();
        assert!(!emit_json_scalar(&mut s2, &serde_json::json!({"a":1}), Delimiter::Comma));
        assert!(s2.is_empty());
    }
}
