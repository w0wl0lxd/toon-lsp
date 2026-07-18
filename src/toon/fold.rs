//! Key folding / path expansion for the TOON codec.
//!
//! Key folding (spec v1.5, optional) collapses chains of single-key objects
//! into dotted paths (`{"a":{"b":{"c":1}}}` -> `{"a.b.c":1}`), reducing token
//! count for deeply nested data bound for LLM prompts. Path expansion is the
//! inverse, applied on decode so a folded document round-trips losslessly.
//!
//! Both are opt-in via [`crate::toon::ToonConfig`] and default to off, keeping
//! the codec spec-conformant by default. Folding only rewrites keys; values
//! are never altered, so `fold_keys` followed by `expand_paths` is lossless on
//! documents that contain no pre-existing dotted keys.

use serde_json::{Map, Value};

/// A segment is foldable when it is a bare TOON identifier: it starts with a
/// letter or underscore and contains only letters, digits, and underscores.
/// Such segments can be joined with `.` and emitted unquoted.
fn is_foldable_segment(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// True when `a` and `b` (dot-split key paths) conflict: one is a prefix of
/// the other (including equality). Such a pair cannot both be expanded
/// losslessly, so folding must avoid producing it.
fn paths_conflict(a: &[&str], b: &[&str]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let n = a.len().min(b.len());
    a[..n] == b[..n]
}

/// Folds a single chain rooted at `key` -> `value` into `(dotted_key, leaf)`
/// when every object in the chain has exactly one key and all segments are
/// foldable identifiers. Returns `None` when the chain is not foldable.
fn fold_chain(key: &str, value: &Value) -> Option<(String, Value)> {
    match value {
        Value::Object(m) => {
            if m.is_empty() {
                // An empty object is a valid fold leaf.
                return Some((key.to_string(), value.clone()));
            }
            if m.len() == 1 {
                let (nk, nv) = m.iter().next().unwrap();
                if is_foldable_segment(nk) {
                    if let Some((suffix, leaf)) = fold_chain(nk, nv) {
                        return Some((format!("{key}.{suffix}"), leaf));
                    }
                }
                return None;
            }
            None
        }
        _ => Some((key.to_string(), value.clone())),
    }
}

/// Recursively folds object key chains in `value`. Objects, not arrays, are
/// descended (the spec treats arrays as fold leaves).
pub fn fold_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(fold_object(map)),
        other => other.clone(),
    }
}

struct Entry {
    emit_key: String,
    folded: Option<Value>,
    orig_key: String,
    orig_val: Value,
}

fn fold_object(map: &Map<String, Value>) -> Map<String, Value> {
    let mut entries: Vec<Entry> = Vec::with_capacity(map.len());
    for (k, v) in map {
        if is_foldable_segment(k) {
            if let Some((fk, leaf)) = fold_chain(k, v) {
                entries.push(Entry {
                    emit_key: fk,
                    folded: Some(leaf),
                    orig_key: k.clone(),
                    orig_val: v.clone(),
                });
                continue;
            }
        }
        entries.push(Entry {
            emit_key: k.clone(),
            folded: None,
            orig_key: k.clone(),
            orig_val: v.clone(),
        });
    }

    // Dot-split paths used for sibling collision detection.
    let paths: Vec<Vec<&str>> = entries.iter().map(|e| e.emit_key.split('.').collect()).collect();

    let mut out = Map::new();
    for (i, e) in entries.iter().enumerate() {
        if let Some(leaf) = &e.folded {
            let conflict =
                paths.iter().enumerate().any(|(j, pj)| j != i && paths_conflict(&paths[i], pj));
            if conflict {
                // Suppress the fold: emit the original nested value, recursing so
                // any inner chains that do not collide still fold.
                out.insert(e.orig_key.clone(), fold_keys(&e.orig_val));
            } else {
                out.insert(e.emit_key.clone(), leaf.clone());
            }
        } else {
            out.insert(e.emit_key.clone(), fold_keys(&e.orig_val));
        }
    }
    out
}

/// Recursively expands dotted keys into nested objects: the inverse of
/// [`fold_keys`] and [`flatten_keys`], used on decode so folded documents
/// round-trip.
pub fn expand_paths(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let segments: Vec<&str> = k.split('.').collect();
                if segments.len() > 1 && segments.iter().all(|s| is_foldable_segment(s)) {
                    insert_path(&mut out, &segments, expand_paths(v));
                } else {
                    out.insert(k.clone(), expand_paths(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(expand_paths).collect()),
        other => other.clone(),
    }
}

/// True when one path is a strict prefix of or equal to the other. Unlike
/// [`paths_conflict`] (used for single-chain folding), this allows siblings
/// that merely share a common ancestor (e.g. `a.b` and `a.c` do not conflict).
fn path_prefix_conflict(a: &[&str], b: &[&str]) -> bool {
    let n = a.len().min(b.len());
    a[..n] == b[..n]
}

/// Recursively flattens nested objects into dotted keys. Unlike [`fold_keys`],
/// which only collapses single-key chains, this inlines every nested object
/// whose keys are foldable identifiers and whose resulting dotted keys do not
/// conflict with siblings. Arrays are descended. Inverse is [`expand_paths`].
pub fn flatten_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(flatten_object(map)),
        Value::Array(arr) => Value::Array(arr.iter().map(flatten_keys).collect()),
        other => other.clone(),
    }
}

fn flatten_object(map: &Map<String, Value>) -> Map<String, Value> {
    // First pass: recursively flatten each value and, for child objects that
    // contain only foldable keys, lift their keys into the parent namespace.
    struct Entry {
        key: String,
        value: Value,
    }
    let mut entries: Vec<Entry> = Vec::with_capacity(map.len());
    for (k, v) in map {
        let flat_v = flatten_keys(v);
        if is_foldable_segment(k) {
            if let Value::Object(child) = &flat_v {
                let mut lifted = Vec::new();
                let mut can_lift = true;
                for (ck, cv) in child {
                    let new_key = format!("{k}.{ck}");
                    let segs: Vec<&str> = new_key.split('.').collect();
                    if segs.iter().all(|s| is_foldable_segment(s)) {
                        lifted.push((new_key, cv.clone()));
                    } else {
                        can_lift = false;
                        break;
                    }
                }
                if can_lift && !lifted.is_empty() {
                    for (new_key, cv) in lifted {
                        entries.push(Entry { key: new_key, value: cv });
                    }
                    continue;
                }
            }
        }
        entries.push(Entry { key: k.clone(), value: flat_v });
    }

    // Second pass: detect prefix conflicts among the resulting keys. If any
    // exist, keep all original top-level keys (values are still recursively
    // flattened internally). This preserves lossless round-trip expansion.
    let paths: Vec<Vec<&str>> = entries.iter().map(|e| e.key.split('.').collect()).collect();
    let conflict = paths.iter().enumerate().any(|(i, pi)| {
        paths.iter().enumerate().any(|(j, pj)| i != j && path_prefix_conflict(pi, pj))
    });

    let mut out = Map::new();
    if conflict {
        for (k, v) in map {
            out.insert(k.clone(), flatten_keys(v));
        }
    } else {
        for e in entries {
            out.insert(e.key, e.value);
        }
    }
    out
}

/// Inserts `value` at `segments` within `map`, deep-merging objects (last
/// write wins on conflict, per spec `expandPaths="safe"`).
fn insert_path(map: &mut Map<String, Value>, segments: &[&str], value: Value) {
    if segments.is_empty() {
        return;
    }
    if segments.len() == 1 {
        map.insert(segments[0].to_string(), value);
        return;
    }
    let child = map.entry(segments[0].to_string()).or_insert_with(|| Value::Object(Map::new()));
    match child {
        Value::Object(cm) => insert_path(cm, &segments[1..], value),
        slot => {
            // Existing leaf where an object is required: rebuild as the object
            // path (last write wins).
            let mut new_obj = Map::new();
            insert_path(&mut new_obj, &segments[1..], value);
            *slot = Value::Object(new_obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn folds_single_chain() {
        let v = json!({"a":{"b":{"c":1}}});
        let f = fold_keys(&v);
        assert_eq!(f, json!({"a.b.c":1}));
    }

    #[test]
    fn folds_array_leaf() {
        let v = json!({"x":{"y":[1,2,3]}});
        assert_eq!(fold_keys(&v), json!({"x.y":[1,2,3]}));
    }

    #[test]
    fn folds_empty_object_leaf() {
        let v = json!({"a":{"b":{}}});
        assert_eq!(fold_keys(&v), json!({"a.b":{}}));
    }

    #[test]
    fn multi_key_stops_chain() {
        let v = json!({"a":{"b":1,"c":2}});
        // `a` has two keys -> not a single-key chain -> stays nested.
        assert_eq!(fold_keys(&v), json!({"a":{"b":1,"c":2}}));
    }

    #[test]
    fn non_identifier_segment_not_folded() {
        let v = json!({"a-b":{"c":1}});
        assert_eq!(fold_keys(&v), json!({"a-b":{"c":1}}));
    }

    #[test]
    fn collision_suppresses_fold() {
        let v = json!({"a":{"b":{"c":1}},"a.b.c":2});
        // `a.b.c` would collide with the plain key `a.b.c`; keep nested.
        let f = fold_keys(&v);
        assert_eq!(f, json!({"a":{"b.c":1},"a.b.c":2}));
    }

    #[test]
    fn fold_then_expand_is_lossless() {
        let v = json!({"a":{"b":{"c":1}},"x":{"y":[1,2,3]}});
        let f = fold_keys(&v);
        assert_eq!(f, json!({"a.b.c":1,"x.y":[1,2,3]}));
        assert_eq!(expand_paths(&f), v);
    }

    #[test]
    fn expand_splits_nested_dotted() {
        let v = json!({"a.b.c":1});
        assert_eq!(expand_paths(&v), json!({"a":{"b":{"c":1}}}));
    }

    #[test]
    fn api_round_trip_diag() {
        let v = json!({"a":{"b":{"c":1}},"x":{"y":[1,2,3]}});
        let mut ec = crate::toon::ToonConfig::default();
        ec.fold_keys = true;
        let out = crate::toon::encode_with_config(&v, &ec).unwrap();
        let mut dc = crate::toon::ToonConfig::default();
        dc.expand_paths = true;
        let back = crate::toon::decode_with_config(&out, &dc).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn flatten_lifts_sibling_keys() {
        let v = json!({"a":{"b":1,"c":2}});
        assert_eq!(flatten_keys(&v), json!({"a.b":1,"a.c":2}));
    }

    #[test]
    fn flatten_avoids_prefix_conflicts() {
        let v = json!({"a":{"b":1},"a.b":2});
        // Lifting `a.b` would conflict with the existing dotted key.
        assert_eq!(flatten_keys(&v), json!({"a":{"b":1},"a.b":2}));
    }

    #[test]
    fn flatten_then_expand_is_lossless() {
        let v = json!({"a":{"b":{"c":1,"d":2}},"x":3});
        let f = flatten_keys(&v);
        assert_eq!(expand_paths(&f), v);
    }
}
