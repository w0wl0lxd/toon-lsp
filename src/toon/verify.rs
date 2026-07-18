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
pub fn verify_round_trip(text: &str, expected: &Value, config: &ToonConfig) -> DecodeResult<bool> {
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
) -> DecodeResult<bool> {
    scratch.clear();
    encode_into(expected, config, scratch)
        .map_err(|e| DecodeError::new(format!("encode failed during verify: {e}")))?;
    if text == scratch.as_str() {
        Ok(true)
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
                .unwrap()
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
                .unwrap()
        );
    }
}
