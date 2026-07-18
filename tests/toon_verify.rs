//! End-to-end tests for the public zero-allocation encode/verify surface:
//! `toon::encode_into` and `toon::verify_round_trip[_with_scratch]`.

use serde_json::json;
use toon_lsp::toon::{
    ToonConfig, decode, encode, encode_into, encode_with_config, verify_round_trip,
    verify_round_trip_with_scratch,
};

#[test]
fn encode_into_matches_encode_for_a_variety_of_shapes() {
    let cases = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"tags": ["a", "b", "c"]}),
        json!({"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}),
        json!([1, 2, 3]),
        json!("just a string"),
        json!(42),
        json!(null),
        json!({}),
        json!([]),
    ];
    let config = ToonConfig::default();
    for value in cases {
        let expected = encode_with_config(&value, &config).unwrap();
        let mut actual = String::new();
        encode_into(&value, &config, &mut actual).unwrap();
        assert_eq!(actual, expected, "mismatch encoding {value:?}");
    }
}

#[test]
fn encode_into_appends_after_existing_buffer_content() {
    let mut out = String::from("# generated\n");
    encode_into(&json!({"a": 1}), &ToonConfig::default(), &mut out).unwrap();
    encode_into(&json!({"b": 2}), &ToonConfig::default(), &mut out).unwrap();
    assert_eq!(out, "# generated\na: 1\nb: 2\n");
}

#[test]
fn verify_round_trip_accepts_canonical_encoder_output() {
    let value = json!({"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]});
    let text = encode(&value).unwrap();
    assert!(verify_round_trip(&text, &value, &ToonConfig::default()).is_ok());
}

#[test]
fn verify_round_trip_rejects_hand_written_text_that_differs_in_value() {
    let expected = json!({"name": "Alice", "age": 30});
    let hand_written = "name: Alice\nage: 31\n";
    assert!(verify_round_trip(hand_written, &expected, &ToonConfig::default()).is_err());
}

#[test]
fn verify_round_trip_rejects_non_canonical_but_equivalent_text() {
    // `decode` accepts either representation, but `verify_round_trip` checks
    // byte-for-byte canonical form, not semantic equivalence.
    let expected = json!({"tags": ["a", "b"]});
    let non_canonical_expanded = "tags[2]:\n  - a\n  - b\n";
    // Sanity: decode agrees on the value...
    assert_eq!(decode(non_canonical_expanded).unwrap(), expected);
    // ...but verify_round_trip requires the canonical inline form.
    assert!(verify_round_trip(non_canonical_expanded, &expected, &ToonConfig::default()).is_err());
    let canonical = encode(&expected).unwrap();
    assert!(verify_round_trip(&canonical, &expected, &ToonConfig::default()).is_ok());
}

#[test]
fn verify_round_trip_with_scratch_reuses_buffer_across_many_values() {
    let mut scratch = String::with_capacity(256);
    let values = vec![
        json!({"a": 1}),
        json!({"a": 1, "b": [1, 2, 3]}),
        json!({"nested": {"x": {"y": "z"}}}),
        json!({"a": 1}), // repeat a small value after a larger one
    ];
    for value in values {
        let text = encode(&value).unwrap();
        assert!(
            verify_round_trip_with_scratch(&text, &value, &ToonConfig::default(), &mut scratch)
                .is_ok(),
            "round trip failed for {value:?}"
        );
    }
}

#[test]
fn hex_string_literal_round_trips_through_encode_and_decode() {
    // Regression for the "Hexadecimal literals quoted correctly" fix: a JSON
    // *string* that looks like a hex integer literal must be quoted on
    // encode so it decodes back as the same string, not a number.
    let value = json!({"code": "0x0"});
    let text = encode(&value).unwrap();
    assert_eq!(text, "code: \"0x0\"\n");
    assert_eq!(decode(&text).unwrap(), value);
    assert!(verify_round_trip(&text, &value, &ToonConfig::default()).is_ok());
}
