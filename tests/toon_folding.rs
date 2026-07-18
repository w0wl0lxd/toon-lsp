//! End-to-end key-folding tests through the public TOON codec API.

use serde_json::json;
use toon_lsp::toon::{
    DecodeResult, EncodeResult, ToonConfig, decode_with_config, encode_with_config,
};

fn encode_folded(v: &serde_json::Value) -> EncodeResult<String> {
    let mut cfg = ToonConfig::default();
    cfg.fold_keys = true;
    encode_with_config(v, &cfg)
}

fn decode_expanded(s: &str) -> DecodeResult<serde_json::Value> {
    let mut cfg = ToonConfig::default();
    cfg.expand_paths = true;
    decode_with_config(s, &cfg)
}

#[test]
fn folded_encode_emits_dotted_keys() {
    let out = encode_folded(&json!({"a":{"b":{"c":1}}})).unwrap();
    assert_eq!(out, "a.b.c: 1\n");
}

#[test]
fn folded_round_trips_through_expand() {
    let v = json!({"user":{"profile":{"name":"Alice"}},"tags":["x","y"]});
    let out = encode_folded(&v).unwrap();
    assert_eq!(out, "user.profile.name: Alice\ntags[2]: x,y\n");
    let decoded = decode_expanded(&out).unwrap();
    assert_eq!(decoded, v);
    let re_dec = encode_with_config(&decoded, &ToonConfig::default()).unwrap();
    let re_v = encode_with_config(&v, &ToonConfig::default()).unwrap();
    assert_eq!(re_dec, re_v);
}

#[test]
fn default_encode_is_unchanged() {
    let out = encode_with_config(&json!({"a":{"b":{"c":1}}}), &ToonConfig::default()).unwrap();
    assert_eq!(out, "a:\n  b:\n    c: 1\n");
}

#[test]
fn expand_is_noop_without_flag() {
    // Dotted keys survive literally when expand_paths is off.
    let out = encode_folded(&json!({"a":{"b":1}})).unwrap();
    assert_eq!(decode_with_config(&out, &ToonConfig::default()).unwrap(), json!({"a.b":1}));
}

#[test]
fn collision_keeps_nested_and_still_round_trips() {
    let v = json!({"a":{"b":{"c":1}},"a.b.c":2});
    let out = encode_folded(&v).unwrap();
    // `a.b.c` collides with the plain key, so `a` stays nested.
    assert!(out.contains("a:"), "expected nested `a`, got:\n{out}");
    // Without expansion the dotted keys are literal; with expansion the
    // collision is resolved by last-write-wins (documented limitation).
    let decoded = decode_with_config(&out, &ToonConfig::default()).unwrap();
    assert_eq!(decoded, json!({"a":{"b.c":1},"a.b.c":2}));
}
