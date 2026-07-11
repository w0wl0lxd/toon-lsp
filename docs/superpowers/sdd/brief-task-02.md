# Task 2 Brief — Shared emitter: string quoting + escaping

Authoritative source: `docs/superpowers/plans/2026-07-11-toon-codec.md` → "## Task 2".
Implement exactly what that section specifies, including the verbatim test bodies in Step 1.

## File
- Modify: `src/toon/emit.rs` (currently `//! placeholder`)
- Tests: inline `#[cfg(test)]` in `src/toon/emit.rs`

## Interfaces to produce
- `pub enum Delimiter { Comma, Tab, Pipe }` with `pub fn as_char(self) -> char` (Comma→`,`, Tab→`\t`, Pipe→`|`).
- `pub fn needs_quotes(s: &str, delim: Delimiter) -> bool`
- `pub fn escape_into(out: &mut String, s: &str)`
- `pub fn emit_scalar_string(out: &mut String, s: &str, delim: Delimiter)`

## `needs_quotes` — quote if ANY of these hold
- empty string
- leading or trailing ASCII space
- contains `"` or `\`
- contains any control char (`< 0x20`)
- contains a structural char ambiguous unquoted: `:` immediately followed by a space; OR a leading `-`, `[`, `{`, or `#`; OR the active `delim.as_char()`
- equals a reserved word: `true`, `false`, `null`
- parses as a TOON number (would round-trip as a number)

Do NOT quote merely because the string contains `-` mid-token. `"a-b-c"` stays unquoted. This fixes the over-quoting bug in `formatting.rs:348`.

## `escape_into`
Map ONLY these: `\\`→`\\\\`, `"`→`\\"`, newline→`\\n`, carriage return→`\\r`, tab→`\\t`. Other control chars (`< 0x20`) → `\\u{XX}` style per plan (use `\uXXXX` 4-hex form). All other chars pushed verbatim. Note the Task 2 test only asserts `\\ \" \n \t` mapping; keep `\r` and `\uXXXX` consistent with that scheme.

## `is_toon_number` helper (private)
Return true iff `s` matches the TOON number grammar (optional leading `-`, digits, optional `.digits`, optional exponent). Guard so bare `-`, `.`, `inf`, `nan`, and empty are treated as strings (NOT numbers). A pragmatic impl: `!s.is_empty() && s.parse::<f64>().is_ok()` AND reject the special cases (`inf`, `-inf`, `nan`, `infinity` case-insensitive, and a lone `-`/`.`/`+`). Confirm against the test cases below.

## Tests (verbatim — see plan Task 2 Step 1 for exact code)
Key assertions:
- `needs_quotes("hello", Comma)` == false; `needs_quotes("a-b-c", Comma)` == false
- `needs_quotes("true", Comma)` == true; `needs_quotes("42", Comma)` == true; `needs_quotes("-1.5", Comma)` == true
- `needs_quotes("a,b", Comma)` == true; `needs_quotes("a,b", Pipe)` == false; `needs_quotes("a|b", Pipe)` == true
- `needs_quotes(" leading", Comma)` == true; `needs_quotes("", Comma)` == true
- `escape_into` of `a"b\c\nd\te` yields `a\"b\\c\nd\te` (i.e. Rust string `"a\\\"b\\\\c\\nd\\te"`)
- `emit_scalar_string` of `true` → `"true"` (quoted); of `hello` → `hello` (bare)

## TDD steps
1. Replace the placeholder with the test module (failing).
2. `cargo test -p toon-lsp toon::emit` → FAIL (undefined). Capture RED.
3. Implement the four items + `Delimiter` + `is_toon_number`.
4. `cargo test -p toon-lsp toon::emit` → PASS. Capture GREEN.

## Verification before reporting
- `cargo build -p toon-lsp` compiles.
- `cargo test -p toon-lsp toon::emit` passes, pristine output.
- `cargo clippy -p toon-lsp --all-targets -- -D warnings` clean.

## Constraints
- Nightly, edition 2024, safe Rust only (`unsafe_code = "forbid"`).
- DO NOT COMMIT and do not run git state-mutating commands. Leave changes in the working tree; the controller commits after review.
- Touch ONLY `src/toon/emit.rs`.
- Do not add re-exports to `mod.rs` yet (Task 2 doesn't require them).
