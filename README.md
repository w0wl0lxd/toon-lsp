# toon-lsp

[![CI](https://github.com/w0wl0lxd/toon-lsp/actions/workflows/ci.yml/badge.svg)](https://github.com/w0wl0lxd/toon-lsp/actions/workflows/ci.yml)
[![Build Extensions](https://github.com/w0wl0lxd/toon-lsp/actions/workflows/build-extensions.yml/badge.svg)](https://github.com/w0wl0lxd/toon-lsp/actions/workflows/build-extensions.yml)
[![Release](https://img.shields.io/github/v/release/w0wl0lxd/toon-lsp?include_prereleases)](https://github.com/w0wl0lxd/toon-lsp/releases)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0--only-blue.svg)](LICENSE)

TOON is a compact encoding of the JSON data model for LLM prompts and config files. This repo gives you a Rust library and a binary that runs as an LSP server or as a CLI. The parser recovers from errors and returns a partial AST for incomplete input.

## Why TOON

TOON drops the quotes and braces that JSON needs. The same config in two encodings:

```toon
service:
  name: gateway
  port: 8080
  features: auth,rate-limit,metrics
```

```json
{
  "service": {
    "name": "gateway",
    "port": 8080,
    "features": ["auth", "rate-limit", "metrics"]
  }
}
```

TOON is about 9% smaller on text heavy docs, see [Benchmarks](#benchmarks).

## Install

```bash
cargo install toon-lsp
```

Build from source:

```bash
git clone https://github.com/w0wl0lxd/toon-lsp && cd toon-lsp && cargo build --release
```

The binary runs as an LSP server with no args. Most editors start it for you.

## Quick start

```bash
echo '{"name": "Alice", "age": 30}' | toon-lsp encode -
printf 'name: Alice\nage: 30\n' | toon-lsp decode - --pretty
toon-lsp check config.toon && toon-lsp format config.toon -o config.toon
```

## Language features

- Arrays: expanded, inline with commas, and tabular rows
- Comments: `#` line comments and `/* */` block comments
- Block strings: `"""` preserve newlines and skip escape processing
- Hex integers: `0xFF`, `-0x10`
- References: `${path}` and `${env:VAR}` with cycle detection

```toon
db:
  port: 5432
service:
  db_port: ${db.port}        # 5432
  token: ${env:API_TOKEN}    # from environment
```

## Editor support

11 editors, 10 with LSP and 1 highlight only (Notepad++), see [`docs/ide-support.md`](docs/ide-support.md).

## Language server features

The server provides 18 features. All work on incomplete input. Features include diagnostics, hover, completion, go to definition, find references, rename, formatting, code actions, and code lens. Others are document highlight, document link, folding, inlay hints, linked editing, selection ranges, semantic tokens, document symbols, and workspace symbols.

## Command-line interface

With no subcommand the binary starts the LSP server. Otherwise:

| Command | Purpose | Key flags |
| --- | --- | --- |
| `encode` | JSON or YAML to TOON | `-f, --input-format json\|yaml`, `--indent 2`, `-o` |
| `decode` | TOON to JSON or YAML | `-f, --output-format json\|yaml`, `--pretty`, `-o` |
| `check` | Validate TOON | `-f, --format text\|json\|github`, `-s, --severity` |
| `format` | Format TOON files | `--indent 2`, `--check`, `-o` |
| `symbols` | List document keys | `-f, --format tree\|json\|flat`, `--types`, `--positions` |
| `diagnose` | Structured errors | `-f, --format json\|sarif`, `--context`, `-s, --severity` |
| `graph` | Reference graph | `-o` (Mermaid flowchart) |

```bash
toon-lsp encode data.json -f json --indent 4 | toon-lsp decode - -f yaml
toon-lsp format --check config.toon; toon-lsp graph config.toon -o graph.mmd
```

## Using the library

```rust
use toon_lsp::parse_with_errors;
let (ast, errors) = parse_with_errors("user:\n  name: Alice\n  age: 30");
if let Some(ast) = &ast { /* walk AstNode, or resolve with toon_lsp::resolve::resolve(&ast, "user.name") */ }
for e in &errors { eprintln!("L{}: {}", e.span.start.line + 1, e.kind); }
```

## Architecture

The scanner tracks a source position on every token. The parser recovers past errors. One `AstNode` tree feeds the LSP server, the CLI, the `resolve` module, and the tree-sitter grammar.

```text
source text -> scanner -> parser -> AST with spans -> LSP server / CLI / resolve / tree-sitter
```

## Benchmarks

We measured these with `cargo bench --bench comparison` and `cargo run --example token_savings --release` using `o200k_base` as of v0.7.21. Numbers come from this repo, so re-run on your workload.

Text heavy doc (system prompt plus notes):

| Format | Bytes | Tokens |
| --- | ---: | ---: |
| TOON | 762 | 177 |
| JSON | 834 | 182 |
| YAML | 806 | 193 |
| TOML | 859 | 200 |

Compact config (short keys and numbers):

| Format | Bytes | Tokens |
| --- | ---: | ---: |
| TOON | 596 | 227 |
| JSON | 612 | 185 |
| YAML | 594 | 230 |
| TOML | 629 | 219 |

## Development

Requires Rust 1.85 plus, toolchain pinned in `rust-toolchain.toml`.

```bash
cargo test && cargo clippy --all-features -- -D warnings && cargo fmt --all -- --check
```

## Related

- [toon-format/toon](https://github.com/toon-format/toon) spec and SDKs
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) LSP framework
- [docs/ide-support.md](docs/ide-support.md) editor setup

## License

AGPL-3.0-only, see [LICENSE](LICENSE) and [LICENSING.md](LICENSING.md) for commercial use, contact w0wl0lxd@tuta.com.
