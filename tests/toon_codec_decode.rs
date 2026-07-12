//! Decoder conformance tests, shared by prototype A (default) and prototype B
//! (`--no-default-features --features decoder_b`).

use serde_json::json;
use toon_lsp::toon::decode;

#[test]
fn decode_flat_object() {
    assert_eq!(decode("name: Alice\nage: 30\n").unwrap(), json!({"name":"Alice","age":30}));
}

#[test]
fn decode_nested_and_inline_array() {
    assert_eq!(decode("user:\n  tags[2]: a,b\n").unwrap(), json!({"user":{"tags":["a","b"]}}));
}

#[test]
fn decode_tabular() {
    assert_eq!(
        decode("rows[2]{x,y}:\n  1,2\n  3,4\n").unwrap(),
        json!({"rows":[{"x":1,"y":2},{"x":3,"y":4}]})
    );
}
