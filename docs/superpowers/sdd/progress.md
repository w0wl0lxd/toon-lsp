# SDD Progress Ledger — In-house TOON codec

- **Plan:** `docs/superpowers/plans/2026-07-11-toon-codec.md`
- **Design:** `docs/superpowers/specs/2026-07-11-toon-codec-design.md`
- **Branch:** `feat/in-house-toon-codec` (off `main` @ 5da8001)
- **Commit authority:** driver only. Subagents implement + test + report; they do NOT commit.
- **Dispatch:** serial (tasks share `mod.rs`/`Cargo.toml`/`emit.rs`/`decode.rs`).
- **Arbiter of divergence:** Task 8 spec fixtures win over any hard-coded task expectation.

## Task status

| # | Task | Impl | Review | Commit |
|---|------|------|--------|--------|
| 1 | Scaffold module + error types | DONE (driver) | verbatim-plan | 0506493 |
| 2 | Emit: string quoting + escaping | DONE (driver) | self | 2f5b123 |
| 3 | Emit: numbers + scalar dispatch | DONE (driver) | codex: PASS (0 findings) | a5017fb |
| 4 | Encoder: objects/nesting/inline arrays | DONE (driver) | with 5 | 8ae776f |
| 5 | Encoder: tabular detection + emission | DONE (driver) | claude: 4 findings (1 fixed, 3 dispositioned) | pending |
| 6 | Decoder A (Scanner-driven) | DONE (driver) | self (round-trip gate = Task 8) | pending |
| 7 | Decoder B (byte scanner) | - | - | - |
| 8 | Conformance + round-trip + differential | - | - | - |
| 9 | Benchmark decoders + pick winner | - | - | - |
| 10 | Rewire convert.rs | - | - | - |
| 11 | Refactor formatting.rs onto emit | - | - | - |
| 12 | Drop toon-format, verify, docs | - | - | - |

## Log

- Branch created; ledger initialized. Dispatching Task 1.
- Task 1: subagent flaked (empty return, incomplete). Driver completed verbatim scaffold; RED→GREEN 2/2, clippy/doc clean. Committed 0506493. Docs at 6252ec9.
- Dispatching Task 2 (emit: string quoting + escaping).
- Task 2: subagent flaked again (only RED scaffold, empty return). SUBAGENT INFRA CONFIRMED BROKEN in this environment (2/2). PIVOT: driver executes plan directly (strict TDD, per-task commits, self-review). Task 2 GREEN 5/5, clippy clean. Committed 2f5b123.
- Task 3: driver-direct, RED→GREEN 7/7 emit. codex review gate = PASS, 0 findings. Committed a5017fb.
- Task 4: driver-direct, RED→GREEN 3/3. Required enabling serde_json `preserve_order` (existing 284-test suite still green). Committed 8ae776f with Task 4.
- Task 5: driver-direct, tabular detection + emission, RED→GREEN 7/7. Review gate: codex hit usage limit mid-run → fell back to `claude -p`. Findings dispositioned:
  - F1 (error swallowing): rejected — `emit_json_scalar` returns bool (not Result) and every call site is inside a proven-scalar match arm; no error path swallowed. `EncodeResult` kept for API symmetry.
  - F2 (tabular false-negative for array field inside expanded list item): ACCEPTED + FIXED — extracted `encode_array_body` shared by plain and nested array fields; added regression test `tabular_applies_to_array_field_inside_expanded_item`.
  - F3 (indent!=2 misaligns expanded-item continuation) & F4 (root bare arrays never inline/tabular): DEFERRED to Task 8 — exact canonical layout must be pinned by spec fixtures, not guessed (spec is the authority). Tracked as known limitations.
- Task 6: driver-direct, empirically dumped the Scanner token stream (examples/dump_tokens.rs, since deleted) to design against real tokens rather than guessing. Recursive-descent parser over `Vec<Token>`: objects, nested blocks (Indent/Dedent), inline arrays, tabular, expanded arrays (scalar/object/nested items), references decoded back to `${path}` strings. Added `[features] default=[decoder_a] decoder_a decoder_b` + mutual-exclusion `compile_error!`, `DecodeError::new`. RED→GREEN 3/3, clippy/doc/full-suite clean.
  - Known round-trip edge (for Task 8): emit escapes exotic control chars as `\uXXXX` but the Scanner only understands `\n \r \t \" \\` (errors on `\u`); and hex Number tokens won't parse via serde_json. Both to be pinned/fixed under conformance.
