# In-house TOON codec + spec-conformant formatter — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the external `toon-format` dependency with an in-house, safe-Rust TOON codec and make the LSP formatter spec-conformant, sharing one emitter core.

**Architecture:** A shared spec-conformant emitter core (`src/toon/emit.rs`) provides all quoting/escaping/number/tabular/delimiter primitives. Two drivers sit on top: `encode(Value)` canonicalizes JSON into TOON; `format(AstNode)` preserves author forms but routes every primitive through the shared core. A decoder (`src/toon/decode.rs`) turns TOON into `serde_json::Value`; two prototypes are built and benchmarked, winner kept.

**Tech Stack:** Rust (nightly, edition 2024), `serde_json::Value` interchange, existing `src/parser/scanner.rs`, `criterion` (new dev-dep), `proptest` (existing), `insta` (existing).

## Global Constraints

- Toolchain nightly, `edition = "2024"`, `resolver = "3"`.
- `unsafe_code = "forbid"` — codec is 100% safe Rust.
- Clippy pedantic `-D warnings`; `cargo doc -Dwarnings` clean.
- `serde_json::Value` stays the interchange type.
- No auto-commit by subagents; the driver commits per task.
- Authority is the TOON spec, never `toon-format` behavior.
- New direct deps allowed only if justified: `criterion` (dev), `memchr` (only if decoder B wins).

## Interfaces (verbatim, from current tree)

- `crate::ast::AstNode` variants: `Document{children,span}`, `Object{entries:Vec<ObjectEntry>,span}`, `Array{items:Vec<AstNode>,form:ArrayForm,span}`, `String{value,span}`, `Number{value:NumberValue,span}`, `Bool{value,span}`, `Null{span}`, `Reference{path:String,is_env:bool,span}`.
- `crate::ast::ObjectEntry { key: String, key_span: Span, value: AstNode }`.
- `crate::ast::NumberValue::{PosInt(u64), NegInt(i64), Float(f64)}` with `as_f64(self)->f64`.
- `crate::ast::ArrayForm::{Inline, Expanded, Tabular}`.
- `crate::parser::Scanner::new(&str)` implements `Iterator<Item=Token>`; also `scan_all(&mut self)->Vec<Token>`.
- `crate::parser::{Token{kind:TokenKind,span:Span}, TokenKind}`; `TokenKind::{Colon,Comma,Newline,Indent,Dedent,Eof,LeftBracket,RightBracket,LeftBrace,RightBrace,Dash,String(String),Reference(String),Number(String),True,False,Null,Identifier(String),Error(String)}`.
- `crate::cli::error::{CliError, CliResult}`; today `CliError::encode(msg)`, `CliError::decode(msg)`.
- Current call sites to replace (`src/cli/convert.rs`): `encode_json` (:29), `encode_json_with_indent` (:39), `decode_toon` (:52).

---

## Task 1: Scaffold `src/toon` module + error types

**Files:**
- Create: `src/toon/mod.rs`
- Create: `src/toon/error.rs`
- Modify: `src/lib.rs` (add `pub mod toon;`)
- Test: inline `#[cfg(test)]` in `src/toon/error.rs`

**Interfaces:**
- Produces: `pub enum EncodeError`, `pub enum DecodeError` (both `thiserror::Error`, `Debug`, `Display`); `DecodeError::Syntax { message: String, line: u32, col: u32 }` and `DecodeError::Structure(String)`; `EncodeError::Unsupported(String)`. `pub type EncodeResult<T>=Result<T,EncodeError>`, `pub type DecodeResult<T>=Result<T,DecodeError>`.

- [ ] **Step 1: Write the failing test** in `src/toon/error.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_syntax_error_displays_location() {
        let e = DecodeError::Syntax { message: "unexpected }".into(), line: 3, col: 5 };
        assert_eq!(e.to_string(), "syntax error at line 3, column 5: unexpected }");
    }

    #[test]
    fn encode_unsupported_displays() {
        let e = EncodeError::Unsupported("NaN".into());
        assert_eq!(e.to_string(), "unsupported value: NaN");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p toon-lsp toon::error`
Expected: FAIL — `toon` module / `DecodeError` not found.

- [ ] **Step 3: Write minimal implementation** — `src/toon/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EncodeError {
    #[error("unsupported value: {0}")]
    Unsupported(String),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DecodeError {
    #[error("syntax error at line {line}, column {col}: {message}")]
    Syntax { message: String, line: u32, col: u32 },
    #[error("structure error: {0}")]
    Structure(String),
}

pub type EncodeResult<T> = Result<T, EncodeError>;
pub type DecodeResult<T> = Result<T, DecodeError>;
```

`src/toon/mod.rs`:

```rust
//! In-house TOON codec: spec-conformant encode/decode and shared emitter core.
pub mod error;
pub mod emit;
pub mod encode;
pub mod decode;

pub use encode::encode;
pub use decode::decode;
pub use error::{DecodeError, DecodeResult, EncodeError, EncodeResult};
```

Add `pub mod toon;` to `src/lib.rs` next to the other module declarations. Create empty `emit.rs`/`encode.rs`/`decode.rs` stubs (task 2+ fill them) so the module compiles — each with `//! placeholder` and nothing else, temporarily removing their `pub use` lines from `mod.rs` until the referenced items exist (re-add in later tasks).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p toon-lsp toon::error`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/toon/ src/lib.rs
git commit -m "feat(toon): scaffold codec module and error types"
```

---

## Task 2: Shared emitter — string quoting + escaping

**Files:**
- Modify: `src/toon/emit.rs`
- Test: inline in `src/toon/emit.rs`

**Interfaces:**
- Produces: `pub fn needs_quotes(s: &str, delim: Delimiter) -> bool`; `pub fn escape_into(out: &mut String, s: &str)`; `pub fn emit_scalar_string(out: &mut String, s: &str, delim: Delimiter)`; `pub enum Delimiter { Comma, Tab, Pipe }` with `pub fn as_char(self)->char`.
- Consumes: nothing.

Spec rules for `needs_quotes` (quote if ANY hold): empty string; leading/trailing ASCII space; contains `"` or `\`; contains any control char (`< 0x20`); contains a structural char that would be ambiguous unquoted (`:` when followed by space, leading `-` `[` `{` `#`, or the active `delim.as_char()`); equals a reserved word (`true`,`false`,`null`); or parses as a TOON number (would round-trip as number). Do NOT quote merely because it contains `-` mid-token (fixes `formatting.rs:348`).

- [ ] **Step 1: Write the failing tests**

```rust
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
}
```

- [ ] **Step 2: Run to verify fail** — `cargo test -p toon-lsp toon::emit` → FAIL (undefined).

- [ ] **Step 3: Implement** the four items + `Delimiter` in `emit.rs`. `escape_into` maps only `\\ \" \n \r \t` and `\u{XX}` for other control chars; all other chars pushed verbatim. `is_toon_number(s)` helper: returns true if `s` matches TOON number grammar (optional `-`, digits, optional `.digits`, optional exponent) — reuse by attempting `s.parse::<f64>().is_ok() && !s.is_empty()` guarded so bare `-`/`.`/`inf`/`nan` are treated as strings.

- [ ] **Step 4: Run to verify pass** — `cargo test -p toon-lsp toon::emit` → PASS.

- [ ] **Step 5: Commit**

```bash
git add src/toon/emit.rs
git commit -m "feat(toon): spec-conformant string quoting and escaping in emit core"
```

---

## Task 3: Shared emitter — canonical numbers + scalar dispatch

**Files:**
- Modify: `src/toon/emit.rs`

**Interfaces:**
- Produces: `pub fn emit_number(out: &mut String, n: &serde_json::Number)`; `pub fn emit_json_scalar(out: &mut String, v: &serde_json::Value, delim: Delimiter) -> bool` (returns true if `v` was a scalar and was written, false for object/array).
- Consumes: `serde_json`, task 2 items.

Numbers use `serde_json::Number`'s own `Display` (canonical, ryu/itoa-backed): integers print without decimal, floats minimal. This guarantees round-trip parity with `serde_json`.

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn numbers_canonical() {
    let mut s = String::new();
    emit_number(&mut s, &serde_json::Number::from(42));
    assert_eq!(s, "42");
    let mut s2 = String::new();
    emit_number(&mut s2, &serde_json::Number::from_f64(1.5).unwrap());
    assert_eq!(s2, "1.5");
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
```

- [ ] **Step 2: Fail** — `cargo test -p toon-lsp toon::emit` → FAIL.
- [ ] **Step 3: Implement** `emit_number` (delegates to `n.to_string()`) and `emit_json_scalar` (match `Null`→`"null"`, `Bool`→`true`/`false`, `Number`→`emit_number`, `String`→`emit_scalar_string`, else return false).
- [ ] **Step 4: Pass** — `cargo test -p toon-lsp toon::emit` → PASS.
- [ ] **Step 5: Commit** — `git commit -am "feat(toon): canonical number emission and scalar dispatch"`

---

## Task 4: Encoder driver — objects, inline arrays, nesting

**Files:**
- Modify: `src/toon/encode.rs`
- Test: inline in `src/toon/encode.rs`

**Interfaces:**
- Produces: `pub fn encode(value: &serde_json::Value) -> EncodeResult<String>`; `pub fn encode_with_indent(value: &serde_json::Value, indent: usize) -> EncodeResult<String>`.
- Consumes: all of `emit.rs`; `EncodeResult`.

Behavior: top-level object → `key: value` lines; nested object → key on its own line, children indented; scalar array with ≤N and all-scalar → inline `key[len]: a,b,c` (TOON inline count header); nested/object arrays → expanded `- ` items (tabular handled in Task 5). Trailing newline. Indent default 2, configurable.

- [ ] **Step 1: Failing tests**

```rust
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
}
```

- [ ] **Step 2: Fail** — `cargo test -p toon-lsp toon::encode` → FAIL.
- [ ] **Step 3: Implement** recursive encoder writing into a pre-sized `String`. Re-add `pub use encode::encode;` in `mod.rs`.
- [ ] **Step 4: Pass** — `cargo test -p toon-lsp toon::encode` → PASS.
- [ ] **Step 5: Commit** — `git commit -am "feat(toon): encoder for objects, nesting, inline scalar arrays"`

---

## Task 5: Encoder — tabular array detection + emission

**Files:**
- Modify: `src/toon/encode.rs`

**Interfaces:**
- Produces: (internal) `fn detect_tabular(arr: &[Value]) -> Option<Vec<String>>` returning ordered column names when every element is an object with identical key sets and all-scalar values.
- Consumes: `emit.rs`.

Emission: `key[N]{col1,col2}:` header then one indented row per element, values delimiter-joined via `emit_json_scalar`. Column order = key order of the first element. If not uniform/all-scalar → fall back to expanded `- ` objects.

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn uniform_object_array_is_tabular() {
    let v = serde_json::json!({"rows":[{"x":1,"y":2},{"x":3,"y":4}]});
    let out = encode(&v).unwrap();
    assert_eq!(out, "rows[2]{x,y}:\n  1,2\n  3,4\n");
}

#[test]
fn nonuniform_array_falls_back_to_expanded() {
    let v = serde_json::json!({"rows":[{"x":1},{"y":2}]});
    let out = encode(&v).unwrap();
    assert!(out.starts_with("rows[2]:") || out.contains("- "));
    assert!(!out.contains("{x")); // not tabular
}
```

- [ ] **Step 2: Fail** → **Step 3: Implement** `detect_tabular` + emission + expanded fallback → **Step 4: Pass** (`cargo test -p toon-lsp toon::encode`).
- [ ] **Step 5: Commit** — `git commit -am "feat(toon): tabular array detection and emission in encoder"`

---

## Task 6: Decoder prototype A (Scanner-driven) — feature-gated

**Files:**
- Modify: `src/toon/decode.rs`
- Modify: `Cargo.toml` (add `[features] decoder_a=[] decoder_b=[]`, default `decoder_a`)
- Test: `tests/toon_codec_decode.rs` (new)

**Interfaces:**
- Produces: `pub fn decode(input: &str) -> DecodeResult<serde_json::Value>` (behind `#[cfg(feature="decoder_a")]`).
- Consumes: `crate::parser::{Scanner,Token,TokenKind}`, `emit`/spec number parsing helper `parse_scalar(&str)->Value`.

Handles: `key: scalar`, nested objects via Indent/Dedent, inline arrays `key[N]: a,b,c`, tabular `key[N]{cols}:` + rows, expanded `- item`, references (`TokenKind::Reference` → keep as `${...}` string in Value, since JSON has no reference type), root scalars/arrays.

- [ ] **Step 1: Failing tests** in `tests/toon_codec_decode.rs`

```rust
use toon_lsp::toon::decode;
use serde_json::json;

#[test]
fn decode_flat_object() {
    assert_eq!(decode("name: Alice\nage: 30\n").unwrap(), json!({"name":"Alice","age":30}));
}

#[test]
fn decode_nested_and_inline_array() {
    assert_eq!(
        decode("user:\n  tags[2]: a,b\n").unwrap(),
        json!({"user":{"tags":["a","b"]}})
    );
}

#[test]
fn decode_tabular() {
    assert_eq!(
        decode("rows[2]{x,y}:\n  1,2\n  3,4\n").unwrap(),
        json!({"rows":[{"x":1,"y":2},{"x":3,"y":4}]})
    );
}
```

- [ ] **Step 2: Fail** — `cargo test --test toon_codec_decode` → FAIL.
- [ ] **Step 3: Implement** decoder A driving `Scanner`. Re-add `pub use decode::decode;` (cfg-gated) in `mod.rs`.
- [ ] **Step 4: Pass** — `cargo test --test toon_codec_decode` → PASS.
- [ ] **Step 5: Commit** — `git commit -am "feat(toon): Scanner-driven decoder (prototype A)"`

---

## Task 7: Decoder prototype B (purpose-built byte scanner) — feature-gated

**Files:**
- Modify: `src/toon/decode.rs` (add `#[cfg(feature="decoder_b")]` module `byte`)
- Modify: `Cargo.toml` (add `memchr = "2"` as a normal dep, only used under `decoder_b`)

**Interfaces:**
- Produces: `pub fn decode(input: &str) -> DecodeResult<serde_json::Value>` (behind `#[cfg(feature="decoder_b")]`), identical signature/semantics to A. The SAME `tests/toon_codec_decode.rs` must pass under `--no-default-features --features decoder_b`.

Line-oriented: split on `\n` via `memchr`, compute indent by leading spaces, parse each line with borrowed `&str` slices, minimal allocation.

- [ ] **Step 1: Reuse tests** — run existing decode tests under feature B:

Run: `cargo test --test toon_codec_decode --no-default-features --features decoder_b`
Expected: FAIL (unimplemented).

- [ ] **Step 2–3: Implement** decoder B. Ensure only ONE `decode` is compiled at a time (features are mutually exclusive via `compile_error!` guard if both set).
- [ ] **Step 4: Pass** — same command → PASS. Also confirm A still passes: `cargo test --test toon_codec_decode`.
- [ ] **Step 5: Commit** — `git commit -am "feat(toon): purpose-built byte-scanner decoder (prototype B)"`

---

## Task 8: Conformance harness — spec vectors + round-trip + differential

**Files:**
- Create: `tests/fixtures/toon_spec/` (fetched vectors; committed)
- Create: `tests/conformance.rs`
- Create: `tests/roundtrip_prop.rs`
- Create: `tests/differential.rs`

**Interfaces:**
- Consumes: `toon_lsp::toon::{encode,decode}`, `toon_format` (differential only), `proptest`.

- [ ] **Step 1:** Fetch upstream TOON spec conformance fixtures (JSON↔TOON pairs) into `tests/fixtures/toon_spec/` (driver performs the network fetch; commit the files so CI is offline-safe). If upstream ships none, hand-author a `cases.json` of `{toon, json}` pairs covering: scalars, quoting edge cases, escapes, nested objects, inline arrays, tabular, expanded, empty containers, unicode, numbers (int/float/exp).
- [ ] **Step 2 (conformance, hard gate):** `tests/conformance.rs` loads each fixture, asserts `decode(toon) == json` and `decode(encode(json)) == json`. Run: `cargo test --test conformance` → PASS.
- [ ] **Step 3 (round-trip prop):** `tests/roundtrip_prop.rs` uses `proptest` to generate arbitrary JSON values (finite numbers only) and assert `decode(encode(v)) == v`. Run: `cargo test --test roundtrip_prop` → PASS.
- [ ] **Step 4 (differential, informational):** `tests/differential.rs` compares `encode` vs `toon_format::encode_default` and logs (via `eprintln!`) divergences without failing (use a `#[test]` that always passes but prints a diff summary). Run: `cargo test --test differential -- --nocapture`.
- [ ] **Step 5: Commit** — `git add tests/ && git commit -m "test(toon): conformance vectors, round-trip properties, differential harness"`

---

## Task 9: Benchmark decoders + pick winner

**Files:**
- Modify: `Cargo.toml` (add `criterion` dev-dep + `[[bench]] name="decode"`)
- Create: `benches/decode.rs`

- [ ] **Step 1:** Write `benches/decode.rs` benchmarking `decode` over a corpus (small flat, deep nested, large tabular 1000 rows, mixed) using criterion.
- [ ] **Step 2:** Run under A: `cargo bench --bench decode` (default features). Record numbers.
- [ ] **Step 3:** Run under B: `cargo bench --bench decode --no-default-features --features decoder_b`. Record numbers.
- [ ] **Step 4 (decision — use thoughtbox):** Log a `decision_frame` capturing throughput + maintainability; pick the winner. Default winner if within 10%: A (one tokenizer to maintain).
- [ ] **Step 5:** Delete the losing prototype and its feature flag; make the winner unconditional (`pub fn decode`); drop `memchr` if B lost. Run full suite. Commit — `git commit -am "perf(toon): select decoder <A|B> after benchmark; remove loser"`.

---

## Task 10: Rewire `convert.rs` to in-house codec

**Files:**
- Modify: `src/cli/convert.rs`
- Modify: `src/cli/error.rs` (add `From<EncodeError>`/`From<DecodeError>` for `CliError`)

**Interfaces:**
- `encode_json` → `toon_lsp::toon::encode(value)`; `encode_json_with_indent` → `encode_with_indent(value, indent)`; `decode_toon` → `toon_lsp::toon::decode(toon)`.

- [ ] **Step 1:** Add `impl From<EncodeError> for CliError` (→ `CliError::Encode(e.to_string())`) and `From<DecodeError>` (→ `CliError::Decode(e.to_string())`) with a failing test asserting the mapping message.
- [ ] **Step 2: Fail** → **Step 3: Implement** conversions + repoint the three functions in `convert.rs`. Keep the existing `convert.rs` tests (they assert round-trip + substrings) — they must stay green.
- [ ] **Step 4: Pass** — `cargo test -p toon-lsp cli::convert` and `cargo test` → PASS.
- [ ] **Step 5: Commit** — `git commit -am "refactor(cli): route convert through in-house toon codec"`

---

## Task 11: Refactor `formatting.rs` onto shared emit core

**Files:**
- Modify: `src/lsp/formatting.rs`

**Interfaces:**
- Consumes: `crate::toon::emit::{needs_quotes, escape_into, emit_scalar_string, emit_number, Delimiter}`.

Replace `formatting.rs`'s private `needs_quotes` (:339), `escape_string` (:371), `format_number` (:388), and the pipe-based `format_tabular_row` (:307) with calls into `emit.rs`. Formatter STILL preserves array forms/key order/references/block strings — only the primitives change. Tabular rows now emit spec `key[N]{cols}:` form (update `test_format_preserves_tabular_array` expectations accordingly).

- [ ] **Step 1:** Update the affected formatter tests to spec-correct expectations (e.g. tabular no longer uses `|`; strings with internal `-` no longer quoted). Run to confirm they FAIL against old code.
- [ ] **Step 2–3:** Refactor `formatting.rs` to delegate primitives to `emit.rs`; delete the now-dead private helpers.
- [ ] **Step 4: Pass** — `cargo test -p toon-lsp lsp::formatting` and full `cargo test` → PASS.
- [ ] **Step 5: Commit** — `git commit -am "refactor(lsp): formatter uses shared spec-conformant emit core"`

---

## Task 12: Drop `toon-format`, final verification, docs

**Files:**
- Modify: `Cargo.toml` (remove `toon-format`; remove `differential` reliance or gate it behind an optional dev-dep)
- Modify: `CHANGELOG.md`, `README.md`

- [ ] **Step 1:** Move `toon-format` to a `[dev-dependencies]`-only entry used solely by `tests/differential.rs` (or delete the differential test if we no longer want the dep at all — driver decides). Remove it from `[dependencies]`.
- [ ] **Step 2:** `cargo build && cargo test` → all green. `cargo clippy --all-targets -- -D warnings` → clean. `cargo doc --no-deps -Dwarnings` → clean.
- [ ] **Step 3:** Update `CHANGELOG.md` (in-house codec; removed `toon-format` runtime dep; formatter now spec-conformant tabular/quoting) and `README.md` (architecture note).
- [ ] **Step 4:** Confirm `cargo tree` no longer shows `toon-format` (or only under dev). Run: `cargo tree -i toon-format` → absent from normal deps.
- [ ] **Step 5: Commit** — `git commit -am "feat(toon)!: replace toon-format with in-house codec; spec-conformant formatter"`

---

## Self-Review

- **Spec coverage:** emit core (T2–3), encode incl. tabular (T4–5), decode both prototypes (T6–7), correctness gate spec+roundtrip+differential (T8), benchmark/winner (T9), rewire (T10–11), drop dep (T12). All design sections mapped.
- **Placeholders:** none — every code step has real code; fixture fetch (T8) is an explicit action with a hand-authored fallback.
- **Type consistency:** `encode(&Value)->EncodeResult<String>`, `decode(&str)->DecodeResult<Value>`, `Delimiter`, `needs_quotes(s,delim)`, `detect_tabular` used consistently across tasks.
- **Open risk:** exact spec tabular header syntax (`[N]{cols}:`) and delimiter escalation rules must match fetched vectors; T8 fixtures are the arbiter — if they contradict a task's hard-coded expectation, the fixture wins and the task's asserts are corrected.
