//! Zero-allocation-style round-trip verification for TOON text.
//!
//! The fast path works by re-encoding the expected `serde_json::Value` into a
//! caller-provided scratch buffer and comparing it to the supplied TOON text.
//! This avoids constructing an intermediate `Value` tree and is allocation-free
//! when the scratch buffer has sufficient capacity.
//!
//! For the comparison to be exact, `text` must be canonical TOON produced by
//! the same encoder configuration (typically [`crate::toon::encode_into`]).

use serde_json::Value;

use crate::toon::error::{DecodeError, DecodeResult};
use crate::toon::{ToonConfig, encode_into};

/// Verifies that `text` is the canonical TOON encoding of `expected` under
/// `config`.
///
/// This function creates a temporary `String` scratch buffer, so it performs at
/// least one heap allocation. For an allocation-free variant, use
/// [`verify_round_trip_with_scratch`] with a pre-sized buffer.
///
/// # Errors
/// Returns [`DecodeError`] if the text cannot be matched against the expected
/// value (currently this is reported as a generic mismatch).
pub fn verify_round_trip(text: &str, expected: &Value, config: &ToonConfig) -> DecodeResult<()> {
    let mut scratch = String::new();
    verify_round_trip_with_scratch(text, expected, config, &mut scratch)
}

/// Verifies that `text` is the canonical TOON encoding of `expected` under
/// `config`, using the caller-provided `scratch` buffer.
///
/// `scratch` is cleared and then populated with the canonical encoding of
/// `expected`. If `scratch` has sufficient capacity, no heap allocation occurs
/// on this path.
///
/// # Errors
/// Returns [`DecodeError`] if `text` is not byte-identical to the canonical
/// encoding of `expected`.
pub fn verify_round_trip_with_scratch(
    text: &str,
    expected: &Value,
    config: &ToonConfig,
    scratch: &mut String,
) -> DecodeResult<()> {
    scratch.clear();
    encode_into(expected, config, scratch)
        .map_err(|e| DecodeError::new(format!("encode failed during verify: {e}")))?;
    if text == scratch.as_str() {
        Ok(())
    } else {
        Err(DecodeError::new("TOON text does not match canonical encoding of expected value"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identical_encoding_matches() {
        let value = json!({"name":"Alice","age":30});
        let mut out = String::new();
        encode_into(&value, &ToonConfig::default(), &mut out).unwrap();
        let mut scratch = String::new();
        assert!(
            verify_round_trip_with_scratch(&out, &value, &ToonConfig::default(), &mut scratch)
                .is_ok()
        );
    }

    #[test]
    fn different_value_mismatches() {
        let encoded = "name: Alice\nage: 30\n";
        let expected = json!({"name":"Bob","age":30});
        let mut scratch = String::new();
        assert!(
            verify_round_trip_with_scratch(
                encoded,
                &expected,
                &ToonConfig::default(),
                &mut scratch
            )
            .is_err()
        );
    }

    #[test]
    fn scalar_number_round_trip() {
        let value = json!(1.5);
        let mut out = String::new();
        encode_into(&value, &ToonConfig::default(), &mut out).unwrap();
        let mut scratch = String::new();
        assert!(
            verify_round_trip_with_scratch(&out, &value, &ToonConfig::default(), &mut scratch)
                .is_ok()
        );
    }

    #[test]
    fn verify_round_trip_without_scratch_matches() {
        let value = json!({"tags": ["a", "b", "c"]});
        let mut out = String::new();
        encode_into(&value, &ToonConfig::default(), &mut out).unwrap();
        assert!(verify_round_trip(&out, &value, &ToonConfig::default()).is_ok());
    }

    #[test]
    fn verify_round_trip_without_scratch_mismatches() {
        let expected = json!({"a": 1});
        assert!(verify_round_trip("a: 2\n", &expected, &ToonConfig::default()).is_err());
    }

    #[test]
    fn mismatch_error_message_is_descriptive() {
        let expected = json!({"a": 1});
        let err = verify_round_trip("a: 2\n", &expected, &ToonConfig::default()).unwrap_err();
        assert!(
            err.to_string().contains("does not match canonical encoding"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn trailing_whitespace_difference_is_a_mismatch() {
        // Verification is byte-exact: canonical TOON never has trailing
        // spaces, so text with an extra trailing space must not match even
        // though it is "semantically" the same document.
        let value = json!({"a": 1});
        let mut canonical = String::new();
        encode_into(&value, &ToonConfig::default(), &mut canonical).unwrap();
        let trimmed = canonical.trim_end_matches('\n');
        let with_trailing_space = format!("{trimmed} \n");
        assert!(verify_round_trip(&with_trailing_space, &value, &ToonConfig::default()).is_err());
    }

    #[test]
    fn respects_custom_config_delimiter() {
        let mut config = ToonConfig::default();
        config.delimiter = crate::toon::Delimiter::Pipe;
        let value = json!({"tags": ["a", "b", "c"]});
        let mut out = String::new();
        encode_into(&value, &config, &mut out).unwrap();
        assert_eq!(out, "tags[3]: a|b|c\n");
        assert!(verify_round_trip(&out, &value, &config).is_ok());
        // The same text does not match under the default (comma) config.
        assert!(verify_round_trip(&out, &value, &ToonConfig::default()).is_err());
    }

    #[test]
    fn scratch_buffer_is_cleared_and_reused_across_calls() {
        // The scratch buffer must not leak content from a previous call: it
        // is `clear()`ed at the top of `verify_round_trip_with_scratch`, so
        // reusing one buffer across multiple verifications with different
        // (and differently sized) values must behave identically to using a
        // fresh buffer each time.
        let mut scratch = String::new();

        let big = json!({"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]});
        let mut big_encoded = String::new();
        encode_into(&big, &ToonConfig::default(), &mut big_encoded).unwrap();
        assert!(
            verify_round_trip_with_scratch(
                &big_encoded,
                &big,
                &ToonConfig::default(),
                &mut scratch
            )
            .is_ok()
        );

        let small = json!({"a": 1});
        let mut small_encoded = String::new();
        encode_into(&small, &ToonConfig::default(), &mut small_encoded).unwrap();
        assert!(
            verify_round_trip_with_scratch(
                &small_encoded,
                &small,
                &ToonConfig::default(),
                &mut scratch
            )
            .is_ok()
        );
        // Scratch should hold exactly the small encoding now, not a
        // concatenation with the earlier (larger) content.
        assert_eq!(scratch, small_encoded);
    }

    #[test]
    fn empty_object_round_trip() {
        let value = json!({});
        let mut out = String::new();
        encode_into(&value, &ToonConfig::default(), &mut out).unwrap();
        assert!(verify_round_trip(&out, &value, &ToonConfig::default()).is_ok());
    }
}
