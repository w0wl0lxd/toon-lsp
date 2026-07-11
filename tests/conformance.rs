//! TOON spec conformance harness over vectors fetched from
//! `github.com/toon-format/spec` (committed under `tests/fixtures/toon_spec`).
//!
//! Scope: this codec implements DEFAULT TOON (comma delimiter, 2-space indent,
//! no key-folding, no path expansion). Fixtures exercising non-default options
//! are reported but not gated. The hard gate is:
//!   * `decode(toon) == json` for default-option, non-error decode vectors, and
//!   * round-trip `decode(encode(json)) == json` for default-option encode
//!     vectors.

use std::fs;
use std::path::Path;

use serde_json::Value;
use toon_lsp::toon::{decode, encode};

const FIXTURES: &str = "tests/fixtures/toon_spec";

/// Option keys that put a vector outside this codec's default-TOON scope.
fn has_unsupported_options(options: &Value) -> bool {
    let Some(map) = options.as_object() else {
        return false;
    };
    for (k, v) in map {
        match k.as_str() {
            // strict only tightens decode; our default behavior is a superset.
            "strict" => {}
            "indent" => {
                if v.as_u64() != Some(2) {
                    return true;
                }
            }
            _ => return true, // delimiter, keyFolding, flattenDepth, expandPaths, ...
        }
    }
    false
}

struct Case {
    name: String,
    input: Value,
    expected: Value,
    should_error: bool,
    unsupported: bool,
}

fn load_cases(kind: &str) -> Vec<(String, Vec<Case>)> {
    let dir = Path::new(FIXTURES).join(kind);
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read fixtures dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();

    files
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("read fixture file");
            let doc: Value = serde_json::from_str(&text).expect("parse fixture json");
            let cases = doc["tests"]
                .as_array()
                .expect("tests array")
                .iter()
                .map(|t| Case {
                    name: t["name"].as_str().unwrap_or("<unnamed>").to_string(),
                    input: t["input"].clone(),
                    expected: t["expected"].clone(),
                    should_error: t["shouldError"].as_bool().unwrap_or(false),
                    unsupported: has_unsupported_options(&t["options"]),
                })
                .collect();
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            (name, cases)
        })
        .collect()
}

#[test]
fn scorecard() {
    let mut total = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<String> = Vec::new();

    // Decode vectors: input is a TOON string, expected is JSON.
    for (file, cases) in load_cases("decode") {
        for c in cases {
            total += 1;
            if c.unsupported {
                skipped += 1;
                continue;
            }
            let Some(toon) = c.input.as_str() else {
                skipped += 1;
                continue;
            };
            let got = decode(toon);
            if c.should_error {
                if got.is_ok() {
                    failures.push(format!("decode/{file}: '{}' expected error, got Ok", c.name));
                }
            } else {
                match got {
                    Ok(v) if v == c.expected => {}
                    Ok(v) => failures.push(format!(
                        "decode/{file}: '{}' mismatch\n  toon: {toon:?}\n  want: {}\n  got:  {}",
                        c.name, c.expected, v
                    )),
                    Err(e) => failures.push(format!(
                        "decode/{file}: '{}' errored: {e}\n  toon: {toon:?}",
                        c.name
                    )),
                }
            }
        }
    }

    // Encode vectors: input is JSON, expected is a TOON string. Hard gate is
    // round-trip equivalence, not byte-exact encoder output.
    for (file, cases) in load_cases("encode") {
        for c in cases {
            total += 1;
            if c.unsupported {
                skipped += 1;
                continue;
            }
            match encode(&c.input) {
                Ok(toon) => match decode(&toon) {
                    Ok(v) if v == c.input => {}
                    Ok(v) => failures.push(format!(
                        "roundtrip/{file}: '{}' not equal\n  json: {}\n  toon: {toon:?}\n  back: {}",
                        c.name, c.input, v
                    )),
                    Err(e) => failures.push(format!(
                        "roundtrip/{file}: '{}' decode failed: {e}\n  json: {}\n  toon: {toon:?}",
                        c.name, c.input
                    )),
                },
                Err(e) => failures.push(format!("encode/{file}: '{}' failed: {e}", c.name)),
            }
        }
    }

    let ran = total - skipped;
    eprintln!(
        "TOON conformance: {} passed / {ran} in-scope ({skipped} out-of-scope skipped, {total} total)",
        ran - failures.len()
    );
    for f in &failures {
        eprintln!("FAIL {f}");
    }
    assert!(failures.is_empty(), "{} conformance failures", failures.len());
}
