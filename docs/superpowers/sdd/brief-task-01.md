# Task 1 Brief — Scaffold `src/toon` module + error types

This is the verbatim task text from the plan. Implement exactly this.

Source of truth: `docs/superpowers/plans/2026-07-11-toon-codec.md` → "## Task 1".

## Files
- Create: `src/toon/mod.rs`
- Create: `src/toon/error.rs`
- Modify: `src/lib.rs` (add `pub mod toon;` next to other module decls)
- Test: inline `#[cfg(test)]` in `src/toon/error.rs`

## Interfaces to produce
- `pub enum EncodeError` and `pub enum DecodeError` — both derive `thiserror::Error`, `Debug`, `Display`.
- `DecodeError::Syntax { message: String, line: u32, col: u32 }` and `DecodeError::Structure(String)`.
- `EncodeError::Unsupported(String)`.
- `pub type EncodeResult<T> = Result<T, EncodeError>;`
- `pub type DecodeResult<T> = Result<T, DecodeError>;`

## Exact Display formats (asserted by tests)
- `DecodeError::Syntax { line: 3, col: 5, message: "unexpected }" }` → `"syntax error at line 3, column 5: unexpected }"`
- `EncodeError::Unsupported("NaN")` → `"unsupported value: NaN"`

## TDD steps
1. Write the two failing tests in `src/toon/error.rs` (see plan Task 1 Step 1 for exact test bodies).
2. Run `cargo test -p toon-lsp toon::error` — expect FAIL (module/type not found). Capture output.
3. Implement `error.rs` and `mod.rs` per plan Task 1 Step 3, add `pub mod toon;` to `src/lib.rs`.
   - IMPORTANT: create empty `emit.rs`, `encode.rs`, `decode.rs` stubs (each just `//! placeholder`) so the module compiles. In `mod.rs`, declare the submodules but TEMPORARILY comment out / omit the `pub use encode::encode;` and `pub use decode::decode;` re-exports until those items exist (later tasks re-add them). Keep `pub use error::{...}`.
4. Run `cargo test -p toon-lsp toon::error` — expect PASS. Capture output.

## Verification before reporting
- `cargo build -p toon-lsp` compiles.
- `cargo test -p toon-lsp toon::error` passes, output pristine (no warnings from your new code).
- `cargo clippy -p toon-lsp --all-targets -- -D warnings` clean for the new files.

## Constraints
- Nightly, edition 2024, `unsafe_code = "forbid"` — safe Rust only.
- Do NOT commit. Leave changes in the working tree; the controller commits.
- Do NOT touch any files beyond those listed above.
